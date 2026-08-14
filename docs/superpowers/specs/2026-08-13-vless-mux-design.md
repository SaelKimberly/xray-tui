# VLESS Mux (v1.mux.cool) Client — Design Spec (SP2 of VLESS Parity)

Date: 2026-08-13. Branch: `native-core-stub`.
Part of the VLESS feature-parity program. SP1 (UDP path) done; SP2 = the
v1.mux.cool multiplexing CLIENT; SP3 = XUDP + vision-udp443 on top.

## 1. Goal

Add the VLESS v1.mux.cool multiplexing client to `xray-tui-native`: a
`MuxClient` that opens one VLESS `command=0x03` tunnel to the magic
destination `v1.mux.cool` and multiplexes multiple concurrent TCP sessions
over it, interoperable with xray-core 26.3.27, sing-box 1.13.16 and mihomo
VLESS inbounds (all accept v1.mux.cool via command=0x03).

## 2. Non-Goals (scope boundaries for SP2)

- **XUDP (UDP over mux)** — SP3. SP2 multiplexes TCP sessions only.
- **vision-udp443** — SP3 (needs XUDP).
- **sing-box's own mux protocols** (h2mux / smux / yamux) — a different wire;
  sing-box's VLESS **inbound** accepts v1.mux.cool, which is what we target.
- **Server-side mux** — client only.
- **Session error recovery / re-dialing a dead tunnel** — the MuxClient fails
  the whole tunnel on a transport error (mirror xray's behavior); the app
  reconnects. KeepAlive keeps idle sessions alive but does not resurrect a
  dead tunnel.
- **mux config knobs** (concurrency limits, max sessions per tunnel, idle
  TTL) — out of scope; one tunnel, unbounded sessions (bounded in practice
  by the app).

## 3. Reference Inventory (wire authority)

| Ref | Role |
|---|---|
| `/tmp/xray-main-26327/common/mux/` (26.3.27) | **wire authority** — `frame.go` (FrameMetadata: WriteTo/Unmarshal), `writer.go` (ClientWriter: New/Keep/End), `session.go` (SessionManager, Session), `client.go` (muxCoolAddress `v1.mux.cool`, muxCoolPort `9527`, DialingWorkerFactory), `server.go` (server-side handling, KeepAlive) |
| `thirdparty/sing-box` + `sing-vmess` | e2e server #2 — `mux.go` (HandleMuxConnection, v1.mux.cool server) |
| `thirdparty/mihomo` | parity reference (v1.mux.cool via sing-vmess fork) |

## 4. Wire Spec

### 4.1 Tunnel

The mux client opens ONE VLESS connection with:
- `command = 0x03` (Mux)
- header destination = the magic fqdn `v1.mux.cool` port `9527`
  (xray `common/mux/client.go`: `muxCoolAddress = "v1.mux.cool"`,
  `muxCoolPort = 9527`; the VLESS-internal XUDP rewrite uses :666, but the
  standalone mux client and the server's `mux.Server` interceptor accept the
  magic dest at any port — we use the standard :9527).

The tunnel rides the normal chain (dial → security → transport); it is a
byte stream carrying mux frames.

### 4.2 Frame format

```
2 bytes    meta_len (big-endian) — length of the metadata below, NOT the payload
2 bytes    session_id (big-endian)
1 byte     status
1 byte     option
[if status == New: 1 byte network (0x01 TCP / 0x02 UDP) + port-first address]
[if option & Data: 2 bytes data_len (big-endian) + payload (data_len bytes)]
```

(xray `frame.go` `WriteTo`: the leading 2-byte `len` covers the metadata
(session/status/option/addr) only; `writer.go` `writeMetaWithFrame` appends
`[2B data_len][data]` after the metadata for data frames.)

Statuses (`frame.go`): `New = 0x01`, `Keep = 0x02`, `End = 0x03`,
`KeepAlive = 0x04`.
Options: `Data = 0x01`, `Error = 0x02`.
Address (`addrParser`, `protocol.PortThenAddress()`): port-first — `[2B port]
[1B atyp][addr]` with atyp IPv4=0x01, Domain=0x02, IPv6=0x03.
`meta_len` cap: 512 (server rejects larger — `Unmarshal` "invalid metalen").

### 4.3 Client writer semantics (`writer.go`)

- First data write on a session: status = **New** (metadata carries the
  target: network + port-first addr).
- Subsequent data writes: status = **Keep** (`followup = true`).
- Data is chunked at **8 KiB** per frame (`SplitSize(mb, 8*1024)`).
- Each data frame = `[metadata + Data option][2B data_len][payload]`.
- Stream close: a meta-only **End** frame (`Writer.Close` → status End).
- Idle sessions: the client sends **KeepAlive** (status 0x04) frames
  periodically; the server answers/echoes (xray `server.go` handleStatusKeepAlive).

### 4.4 Client reader semantics

- Read `[2B meta_len]`, then the metadata, then (if `OptionData`) the
  `[2B data_len][payload]`.
- Frames route by `session_id` to the session's byte stream; payloads of
  consecutive data frames on the same session concatenate into the stream.
- `End` on a session: signal EOF to that session's reader.
- `Error` option: surface as an error on that session.
- `KeepAlive`: consumed (no data).
- A session's stream reassembly: buffered across frames (a frame's payload
  may be split across reads; the session reader delivers the byte stream).

### 4.5 Session model (`session.go`)

- Session ids allocated from 1, unique per tunnel.
- One `Session` per app stream: owns its New/Keep/End writer + the reader
  channel for incoming frames.
- The tunnel's frames are demultiplexed to sessions by id.

## 5. Architecture

### 5.1 Mux framing codec (native `protocol/vless/mux.rs`)

```rust
pub(crate) const MUX_DEST: &str = "v1.mux.cool";
pub(crate) const MUX_PORT: u16 = 9527;
pub(crate) const STATUS_NEW: u8 = 0x01;
pub(crate) const STATUS_KEEP: u8 = 0x02;
pub(crate) const STATUS_END: u8 = 0x03;
pub(crate) const STATUS_KEEPALIVE: u8 = 0x04;
pub(crate) const OPT_DATA: u8 = 0x01;
pub(crate) const OPT_ERROR: u8 = 0x02;
pub(crate) const CHUNK_SIZE: usize = 8 * 1024;

/// One parsed frame header (+ optional payload).
pub(crate) struct Frame {
    pub session_id: u16,
    pub status: u8,
    pub option: u8,
    pub target: Option<Target>,   // Some for New frames (network + addr)
    pub payload: bytes::Bytes,    // empty unless OptionData
}

pub(crate) async fn read_frame<R: AsyncRead + Unpin>(r: &mut R) -> io::Result<Option<Frame>>;  // None = clean EOF
pub(crate) async fn write_frame<W: AsyncWrite + Unpin>(w: &mut W, f: &Frame) -> io::Result<()>;
pub(crate) fn encode_new_target(target: &Target, network_tcp: bool) -> Vec<u8>;  // port-first addr bytes
```

Unit-testable over duplex (exact bytes, split reads, End/KeepAlive/Data/Error,
meta_len cap).

### 5.2 MuxClient (native `protocol/vless/mux.rs`)

```rust
/// Multiplexes app TCP streams over one v1.mux.cool tunnel.
pub(crate) struct MuxClient<S> {
    tunnel: S,                       // the command=0x03 byte stream
    next_id: u16,                    // session id allocator (from 1)
    sessions: Mutex<HashMap<u16, SessionHandle>>,
    // demux task: reads frames off the tunnel, routes payloads to sessions
}

pub(crate) struct SessionHandle { /* mpsc channel for this session's data + end/error */ }

impl<S: AsyncRead + AsyncWrite + Unpin + Send + 'static> MuxClient<S> {
    /// Open a new TCP session to `target`; returns the session's byte stream.
    pub(crate) async fn open_session(&self, target: Target) -> io::Result<SessionStream>;
}

/// The app-side stream for one mux session: AsyncRead + AsyncWrite.
pub(crate) struct SessionStream { /* reads from the session channel, writes Keep frames */ }
```

- `MuxClient::new(tunnel)`: spawns a demux task reading frames; routing
  payloads by session_id; End → channel close; Error → error.
- `open_session`: allocate the next id, register a channel, write a **New**
  frame (target), return the `SessionStream` (first payload frame follows on
  the first app write).
- `SessionStream` write: chunk the app bytes at 8 KiB, write **Keep** frames
  (first write after New is Keep — the New was sent by open_session, or the
  first payload write carries New if open_session defers it — pick the xray
  order: New is written with the first data or eagerly; xray writes New with
  the first data write, but an eager New-with-no-payload is valid (meta-only).
  **Decision: open_session sends an eager New frame immediately** (so the
  server allocates the session before any app data) — matches xray's
  `writeMetaOnly` path for empty writes and is simpler to reason about.
  Document as a deviation note if the reviewer flags it.)
- `SessionStream` close/drop: send an **End** frame (flush first).
- KeepAlive: a background task on the client sends KeepAlive frames at a
  fixed interval (e.g. 10s) while the tunnel is alive — enough to keep idle
  sessions from timing out; the demux consumes incoming KeepAlive replies.
  (xray sends per-idle-session; a tunnel-level keepalive is sufficient for
  interop — the server answers/ignores it. Document.)
- Demux task owns the tunnel reads; concurrent session writes are serialized
  by a write mutex (frames are self-delimiting; no interleaving corruption).

### 5.3 Integration (native `protocol/vless/mod.rs` + crate root)

- The mux tunnel: a new connect path that sends `command = 0x03` with dest
  `v1.mux.cool:9527` (reuse the SP1/plain connect chain; the header encode
  already has a CMD_MUX arm — `header.rs:73-77`).
- Public API: a `connect_mux(params) -> Result<MuxClient<Box<dyn Stream>>>`
  (mirror `connect_udp`), or a params flag `mux: bool` that makes `connect()`
  return the MuxClient. **Decision: `connect_mux` as a distinct entry** (the
  MuxClient is a different return type than the byte-stream `connect`);
  the app opens sessions via `open_session`.
- Vision + mux: vision is a TCP flow on a single stream; muxing over a vision
  tunnel is legal in xray (vision then mux inside). **Scope decision: mux
  tunnel supports vision when the flow is set (the header carries the flow
  addon + camouflage + VisionStream, then mux frames over the vision stream).**
  e2e rows cover mux+tls and mux+reality; a mux+vision row is optional
  (added if cheap — the vision splice + mux framing compose like any
  transport). Confirm during implementation; the spec allows it.

### 5.4 Transport composition

Mux rides any existing transport (tcp/ws/grpc/httpupgrade/xhttp/h2) + the
outer TLS/REALITY — the tunnel is a byte stream. `command=0x03` over the
transport; mux frames inside.

## 6. Error Handling

- Transport error on the tunnel: the demux task fails all live sessions
  (channel error/close); subsequent writes on any session return the error.
- End frame on a session: clean EOF for that session's reader (not an error).
- Error option: surface as `io::Error` on that session.
- meta_len > 512: protocol error (close the tunnel — a misbehaving peer).
- Unknown session_id on incoming data: ignore (or error — xray ignores; we
  ignore + log).
- Session id exhaustion (u16): error (unrealistic for a client).

## 7. Testing

### 7.1 Unit (native crate)

- Frame codec: exact bytes for New (with TCP target, port-first addr) / Keep
  (data) / End / KeepAlive / Error frames; split-frame reads; meta_len cap.
- encode_new_target: port-first IPv4/Domain/IPv6.
- MuxClient: two concurrent sessions over one duplex tunnel — independent
  byte streams, End closes one without killing the other; error on one
  session; KeepAlive consumed.
- SessionStream: chunking at 8 KiB (a >8KiB write → multiple frames);
  drop → End frame.

### 7.2 Hermetic

Fake mux server over the rustls TLS fixture (mirror the SP1 T4 pattern):
assert the client's `command=0x03` header + `v1.mux.cool:9527` dest; read
New/Keep/End frames; reply with Keep/End; verify the client's sessions
deliver/close correctly. Optionally a KeepAlive round-trip.

### 7.3 e2e (tier-3, `native-e2e` + `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin`)

Rows (append to `tests/vless.rs`, protocol=vless, network=tcp, mux on):

| Row | Core | Security | Proves |
|---|---|---|---|
| mux_tls | xray + sing-box | tls | concurrent TCP sessions over one tunnel |
| mux_reality | xray + sing-box | reality | mux over REALITY |

= **4 rows** (2+2). The probe: open N (e.g. 4) concurrent sessions through
one MuxClient, send distinct HTTP GETs per session, assert each response.
Existing rows untouched (vless 61+4 → 65+4; total 109+4 → 113+4). Optional
mux+vision row added if cheap.

### 7.4 Gates

- clippy workspace pedantic+nursery 0; `cargo fmt --check` clean.
- unit: existing + new green; e2e sweeps vless 65+4, vmess 48.
- `xray-tui-proto` NOT modified. No new deps. `NativeError`-only surface.
- Every network step timeout-bounded.

## 8. Deviations from Upstream

1. **Eager New frame** on `open_session` (xray sends New with the first data
   write; an eager meta-only New is valid and simpler — the server allocates
   the session immediately). Wire-identical effect; the first data frame is
   then Keep (not New).
2. **Tunnel-level KeepAlive** (xray sends per-idle-session; a single
   tunnel-level KeepAlive keeps the tunnel alive and is answered/ignored by
   both cores). Cost if wrong: an idle session the server times out
   individually would not be resurrected — acceptable for a client.
3. **No mux config knobs** (concurrency/idle TTL) — one tunnel, unbounded
   sessions.
4. **`connect_mux` as a distinct entry** vs a `mux` param flag — the
   MuxClient return type differs from the byte-stream connect.

## 9. Verification Order

1. Unit: frame codec + MuxClient multiplexing + chunking + End.
2. Hermetic: fake mux server (command=0x03 header, New/Keep/End frames).
3. e2e: vless sweep (65+4) + vmess (48) + clippy + fmt.
