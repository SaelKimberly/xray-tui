# VLESS UDP Path (Command 0x02) — Design Spec (SP1 of VLESS Parity)

Date: 2026-08-13. Branch: `native-core-stub`.
Part of the VLESS feature-parity program (native vs xray-core/sing-box/mihomo).
Sub-project order approved: SP1 UDP → SP2 mux → SP3 XUDP+udp443 → SP4 mKCP →
SP5 QUIC → SP6 stream-one+polish → SP7 ML-KEM.

## 1. Goal

Add the VLESS UDP command path (`command = 0x02`) to the `xray-tui-native`
VLESS client: length-prefixed packet framing in both directions over the
existing tunnel transports, a datagram API (`PacketConn`) on the app side,
UDP e2e rows against xray 26.3.27 + sing-box 1.13.16, and the sing-box/mihomo
`packetaddr` packet encoding as a sing-box-single-core row.

## 2. Non-Goals (scope boundaries for SP1)

- **XUDP (UDP over mux)** — SP3. In SP1, UDP flows as `command=0x02` on the
  main tunnel stream only.
- **`xtls-rprx-vision-udp443` flow** — SP3 (needs XUDP).
- **UDP + `xtls-rprx-vision`** — rejected with a clear error in SP1 (xray
  rejects UDP/443 under vision; other UDP would need the XUDP rewrite — SP3).
  The rejection is a client-side guard, not a server-interop test.
- **mux / v1.mux.cool** — SP2.
- **MultiLengthPacket batching** — xray's `MultiLengthPacketWriter` batches
  several packets into one tunnel write; SP1 sends one frame per datagram
  (wire-identical to a length-1 batch; the server's `LengthPacketReader` is
  agnostic). Batching is a later optimization, not a wire feature.
- **UoT (UDP-over-TCP magic address)** — server-side inbound feature (xray
  UoT is sing-box inbound only); not a client wire path. Out.
- **Local UDP listeners / production app wiring** — the native core remains
  e2e-driven; the app side of UDP is the `PacketConn` API used by the harness.
- **fqdn destinations in packetaddr** — unsupported by the sing encoder
  (`ErrFqdnUnsupported`); only IP targets. Mirrored.

## 3. Reference Inventory (wire authority)

| Ref | Role |
|---|---|
| `/tmp/xray-main-26327` (26.3.27, the e2e binary) | **wire authority** — `proxy/vless/encoding/addons.go` (`LengthPacketWriter`/`LengthPacketReader`/`MultiLengthPacketWriter`), `proxy/vless/encoding/encoding.go` (header), `proxy/vless/outbound/outbound.go` (UDP command selection) |
| `thirdparty/sing-box` + `sing-vmess@0.2.8-…` | e2e server #2; UDP handling + `packetaddr` (`packetaddr/packetaddr.go`, `packetaddr/conn.go`; `sing@v0.5.1/common/metadata/serializer.go` WriteAddrPort) |
| `thirdparty/mihomo` | parity reference only (native 2B-length UDP framing + XUDP + packetaddr modes; not an e2e server) |
| `thirdparty/Xray-core` (fork 26.7.28) | diffed-identical for the UDP paths |

## 4. Wire Spec

### 4.1 Request header (UDP command)

```
0x00            version (1 byte)
uuid            VLESS user id (16 bytes)
addons_len      (1 byte; 0 for no flow — flow=vision + UDP is rejected, §6)
addons          (only when flow set)
command         0x02 = UDP (1 byte)
port            big-endian (2 bytes)  ← port-first addressing (26.3.27)
atyp            (1 byte) + address (IPv4 4 / IPv6 16 / domain 1+len)
```

Identical to the TCP path except `command = 0x02`. The header destination =
the UDP target (host:port).

### 4.2 Packet framing (both directions)

After the header, the tunnel stream carries length-prefixed packets:

```
[2 bytes big-endian length][payload (length bytes)]
```

- **Client → server (uplink)**: `LengthPacketWriter` semantics — one
  `[len][payload]` frame per datagram (xray `LengthPacketWriter.WriteMultiBuffer`
  writes `byte(len>>8), byte(len)` then the payload; `MultiLengthPacketWriter`
  concatenates `[len][payload]` per packet — our per-datagram frame is the
  n=1 case of that, wire-identical).
- **Server → client (downlink)**: the server's `DecodeBodyAddons` picks
  `NewLengthPacketReader(reader)` for UDP commands — same `[2B len][payload]`
  framing. The client reads frames with `LengthPacketReader` semantics
  (read 2 bytes, then that many bytes; EOF between frames = clean end).
- Empty packets (len 0): skipped by xray's writers (`length == 0 → nil` /
  `if length == 0 { continue }` in MultiLengthPacketWriter). Client mirrors:
  never emit, skip on read.
- Frame length cap: xray splits >`buf.Size` reads; the writer's 2-byte length
  caps at 65535. Client: cap outgoing frames at 65535 (a larger app datagram
  is a client error, not a wire hack); incoming frames are read to completion
  regardless of size (defensive).

### 4.3 packetaddr encoding (sing-box/mihomo client mode; SP1 fold-in)

Used when the client is configured with `packet_encoding = "packetaddr"`
(sing-box option `vless.outbound.packet_encoding`, default is xudp — xudp is
SP3's mux path; SP1 implements only the packetaddr and raw modes):

- The VLESS **header destination = the magic fqdn** `sp.packet-addr.v2fly.arpa`
  (sing-vmess `SeqPacketMagicAddress`), command = 0x02, the target is NOT in
  the header.
- Each packet payload carries its own destination header (NO magic in the
  frame — the magic is the header destination only):

```
atyp (1 byte) | addr | port (2 bytes BE)
```

  (sing default serializer order: atyp-first, then addr, then port —
  `WriteAddrPort` without `PortThenAddress`, serializer.go:94-121;
  atyp: IPv4 = 0x01, IPv6 = 0x02; **fqdn unsupported** — client error for
  domain targets. The magic is NOT prefixed to the per-packet header: sing's
  packetaddr `AddressSerializer` registers only the IP family bytes, and a
  magic-prefixed frame is rejected by the sing-box server as an unknown
  address family. **Corrected from an earlier draft that asserted
  `magic | atyp | addr | port` in the frame — verified against sing-box
  1.13.16 + mihomo sing-vmess.)**
- The sing-box server (inbound.go packetaddr unwrap) decodes per-packet
  destinations; the frame payload after the destination header is the
  datagram.
- xray servers do NOT support packetaddr (their UDP = header-dest) — the
  packetaddr row is sing-box-single-core.

### 4.4 Response header

Identical to TCP: the server sends `[0x00][addons_len 0]` (i.e. `[0,0]`)
before the first downlink packet frame. The existing `VlessClientStream`
peel handles it (the `PacketConn` reuses the same peel on its first read).

## 5. Architecture

### 5.1 Packet framing codec (native `protocol/vless/udp.rs`)

```rust
/// Reads one [2B BE len][payload] frame. Returns None on clean EOF at a
/// frame boundary (0 bytes read for the length).
pub(crate) async fn read_packet<R: AsyncRead + Unpin>(r: &mut R) -> io::Result<Option<Vec<u8>>>;

/// Writes one frame. `payload.len()` must be <= 65535 (caller checks).
pub(crate) async fn write_packet<W: AsyncWrite + Unpin>(w: &mut W, payload: &[u8]) -> io::Result<()>;
```

Unit-testable over `tokio::io::duplex` (framing round-trip, split-frame
reads, EOF semantics, empty frames skipped).

### 5.2 PacketConn (app-side datagram API, native `protocol/vless/packet.rs` or lib root)

```rust
/// A VLESS UDP connection: the tunnel byte stream wrapped in packet framing.
/// Owns the response-header peel (first read).
pub struct PacketConn<S> {
    inner: S,               // the tunnel stream (post-security/transport)
    peel: PeelState,        // reused from vless::stream (response [0,0] handling)
    in_buf: bytes::BytesMut,  // current frame payload not yet consumed
}

impl<S: AsyncRead + AsyncWrite + Unpin> PacketConn<S> {
    pub async fn send(&mut self, payload: &[u8]) -> io::Result<()>;
    pub async fn recv(&mut self) -> io::Result<Option<Vec<u8>>>; // None = clean EOF
}
```

- `send`: one `[2B len][payload]` frame (reject > 65535).
- `recv`: peel the response header once, then read frames (2B len + payload);
  return the datagram. EOF between frames → `Ok(None)`.
- No `AsyncRead`/`AsyncWrite` impls (datagram semantics, not a byte stream).
- The packetaddr variant lives INSIDE the PacketConn framing: when the mode
  is packetaddr, `send` prepends `atyp + addr + port` to the payload
  and `recv` strips it (the destination is returned
  with the payload — see §5.3).

### 5.3 Packet-mode plumbing

`connect()` in `protocol/vless/mod.rs` gains a UDP mode:

```rust
pub enum VlessMode { Tcp, Udp }   // or a target-kind discriminator — see below

// The existing connect() returns Box<dyn Stream>; the UDP path needs the
// packet wrapper. The native entry (chain/connect_chain or the crate root)
// gains:
pub async fn connect_udp<S: Stream>(params: ..., target: Target) -> Result<PacketConn<Box<dyn Stream>>, NativeError>;
```

- Command byte: `CMD_UDP` (0x02) when the target is UDP (the mode is
  determined by the target kind or an explicit param — read `context.rs`
  `NativeConnectParams.target` for the cleanest seam; the header encode
  already takes a `cmd` argument).
- **Rejection guard**: if `flow == xtls-rprx-vision` && UDP → `NativeError`
  ("VLESS vision does not support UDP (no XUDP mux)" — reuse the T3 guard
  wording if present).
- packetaddr mode: the header destination = `sp.packet-addr.v2fly.arpa`
  (fqdn atyp), the PacketConn wraps with the packetaddr encoder/decoder.
  Chosen via a config/param flag (the native params have no packet_encoding
  field yet — add one, defaulting to raw).
- The response-header peel: reuse the existing `stream.rs` peel logic
  (refactor it into a shared `Peel` helper the PacketConn also uses — the
  current peel lives inside `VlessClientStream`; extract it without changing
  the TCP path's behavior).

### 5.4 Transports

UDP rides the existing transports unchanged (tcp/ws/grpc/httpupgrade/xhttp/
h2) — a byte stream is a byte stream; only the command + framing differ.
UDP + REALITY/TLS outer: fine. UDP + vision: rejected (§5.3).

## 6. Error Handling

- `flow=vision` + UDP: `NativeError` (guard, mirroring xray's UDP/443
  rejection under vision; SP3 removes it for XUDP-capable paths).
- Datagram > 65535 on send: `NativeError` (client error, never truncated).
- packetaddr with a domain target: `NativeError` (sing's `ErrFqdnUnsupported`).
- Malformed frame (truncated length, payload short): `io::Error` /
  `NativeError` propagated; no panics.
- EOF between frames: clean `Ok(None)` (the tunnel closed — mirrors xray's
  `ReadFull` EOF propagation).

## 7. Testing

### 7.1 Unit (native crate)

- Frame codec round-trip over duplex: exact bytes `[len>>8][len][payload]`,
  split-frame reads (2B header / partial payload), EOF at boundary,
  empty-frame skip.
- PacketConn: peel-then-frames on first recv; multiple datagrams; EOF.
- packetaddr encode: exact bytes `0x01 + 4B IPv4 + 2B port` and
  `0x02 + 16B IPv6 + 2B port`; decode round-trip; fqdn target error.
- Vision+UDP guard: connect() with flow=vision + UDP target → error.

### 7.2 Hermetic

A raw fake UDP server over the existing TLS-fixture pattern: assert the
client's header (cmd 0x02, port-first dest), send/receive `[2B len][payload]`
frames, verify datagram integrity. (Reuse the T4 hermetic fixture; the UDP
frames are simple enough that one focused hermetic test suffices.)

### 7.3 e2e (tier-3, `native-e2e` + `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin`)

Harness additions:
- UDP echo target: a `tokio::net::UdpSocket` echo server (returns each
  datagram verbatim).
- `probe_udp(tunnel)`: sends N distinct datagrams through the PacketConn,
  asserts each echo arrives (order not guaranteed — match by payload).

Rows (append to `tests/vless.rs`, protocol=vless):

| Row | Core | Security | Packet mode | Path |
|---|---|---|---|---|
| udp_plain_tls | xray + sing-box | tls | raw | cmd=0x02, 2B framing |
| udp_plain_reality | xray + sing-box | reality | raw | same over REALITY |
| udp_packetaddr_tls | sing-box (single-core) | tls | packetaddr | magic-addr packets |

= **5 new rows** (2 + 2 + 1). Existing rows untouched (vless 56+4 → 61+4;
total e2e 104+4 → 109+4). Vision+UDP rejection: unit-level only (no e2e).

### 7.4 Gates

- clippy workspace pedantic+nursery 0; `cargo fmt --check` clean.
- unit: existing 276 + new green; e2e sweeps: vless 61+4, vmess 48.
- `xray-tui-proto` NOT modified. No new deps (tokio/net/bytes/ring already
  present). `NativeError`-only surface.
- Every network step timeout-bounded (harness convention).

## 8. Deviations from Upstream

1. **No MultiLengthPacketWriter batching** — one frame per datagram
   (wire-identical for n=1; server `LengthPacketReader` agnostic). Cost if
   wrong: slightly more tunnel writes for bursty apps; correctness unaffected.
2. **vision+UDP rejected wholesale** (xray rejects only :443 and XUDP-
   rewrites the rest) — SP3 introduces XUDP; until then a clear error is the
   honest behavior.
3. **packetaddr fqdn rejected** — mirrors sing's encoder exactly.
4. **UoT not implemented** — inbound-side feature (sing-box), not a client
   wire path.

## 9. Verification Order

1. Unit: framing codec + PacketConn + packetaddr + vision-UDP guard.
2. Hermetic: fake UDP server frames.
3. e2e: vless sweep (61+4) + vmess sweep (48) + clippy + fmt.
