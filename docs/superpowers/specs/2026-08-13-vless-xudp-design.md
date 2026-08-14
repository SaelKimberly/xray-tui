# VLESS XUDP + xtls-rprx-vision-udp443 — Design Spec (SP3 of VLESS Parity)

Date: 2026-08-14. Branch: `native-core-stub`.
SP3 of the VLESS parity program. SP1 (UDP command path) + SP2 (v1.mux.cool
client) done; SP3 = UDP over the mux tunnel (XUDP) + the
`xtls-rprx-vision-udp443` flow.

## 1. Goal

Add UDP-over-mux (XUDP) to `xray-tui-native`: the SP2 `MuxClient` gains UDP
sessions (mux frames with `network=0x02 UDP` + per-packet destinations + an
8-byte GlobalID), the SP1 `PacketConn` gains an XUDP mode (datagrams over the
mux UDP session instead of the raw `command=0x02` tunnel), and the
`xtls-rprx-vision-udp443` flow is accepted (wire truncation to
`xtls-rprx-vision` + UDP allowed through vision via XUDP). Interoperable with
xray-core 26.3.27 and sing-box 1.13.16.

## 2. Non-Goals (scope boundaries for SP3)

- **Plain UDP without mux** — SP1's raw `command=0x02` path stays as-is.
- **XUDPManager details / server-side XUDP** — client only.
- **`xtls-rprx-vision-udp443` over the RAW (non-mux) UDP path** — the udp443
  variant's UDP allowance is via XUDP only (mux), matching xray's rewrite
  (`command=Mux` → `v1.mux.cool:666`).
- **GlobalID determinism** — xray computes `blake3-8(BaseKey, source-addr)`
  per source; our e2e-driven client has no real inbound source, so a
  per-tunnel random 8-byte GlobalID is used (the server requires only
  non-empty — deviation, §8).
- **UDP/443 rejection under plain vision** — kept for the non-XUDP path
  (SP1's guard); the udp443 flow lifts it only via the mux rewrite.

## 3. Reference Inventory (wire authority)

| Ref | Role |
|---|---|
| `/tmp/xray-main-26327/common/xudp/xudp.go` (26.3.27) | XUDP packet writer/reader + GlobalID |
| `/tmp/xray-main-26327/common/mux/frame.go` + `server.go` | UDP session frames + the server XUDP handling (GlobalID-keyed, empty ignored) |
| `/tmp/xray-main-26327/proxy/vless/outbound/outbound.go` | the client XUDP rewrite (UDP + vision/cone → command=Mux + v1.mux.cool:666; `-udp443` truncation) |
| `/home/user/go/pkg/mod/…/sing-vmess@v0.2.8-…/xudp.go` | sing-box server's xray-compat XUDP |
| `thirdparty/sing-box` | e2e server #2 |
| `thirdparty/Xray-core` (fork 26.7.28) | diffed-identical |

## 4. Wire Spec

### 4.1 UDP sessions over the mux tunnel

The SP2 mux frames carry UDP sessions when `network = 0x02 UDP` in the New
frame and in every data frame:

```
New frame (first packet of the session):
  [2B meta_len][2B session_id][0x01 New][option][0x02 UDP][port-first dest]
  [8B GlobalID][2B data_len][payload]

Keep data frames (subsequent packets):
  [2B meta_len][2B session_id][0x02 Keep][0x01 Data][0x02 UDP]
  [port-first dest — PER-PACKET][2B data_len][payload]
```

(xray `frame.go` `WriteTo`: New frames write the target when the network is
UDP; `else if b.UDP != nil { WriteByte(TargetNetworkUDP); WriteAddressPort(b,
b.UDP.Address, b.UDP.Port) }` — every UDP data frame carries its own
destination. The xudp `PacketWriter` builds the same shape raw with sid=0;
we use the frame-metadata path with real session ids — the server accepts
both.)

### 4.2 GlobalID

- 8 bytes; written in the New frame (xray: `if b.UDP != nil {
  b.Write(f.GlobalID[:]) }` — after the target).
- Server (`server.go`): `if meta.GlobalID != [8]byte{}` → the XUDPManager
  path (keyed by GlobalID, per-packet dispatch via `NewPacketReader`).
  **Empty GlobalID → the plain UDP session path** (also valid; per-packet
  dests still work).
- Client: a **random 8-byte GlobalID per tunnel** (any non-empty value is
  accepted; the server never validates the bytes). Deviation from xray's
  blake3(BaseKey, source) — our client has no inbound-source context.

### 4.3 The udp443 flow (`xtls-rprx-vision-udp443`)

- The flow name is accepted at the config/param level.
- On the wire, the addon flow string is **truncated to the first 16 bytes**
  → `xtls-rprx-vision` (xray `outbound.go`: `requestAddons.Flow =
  requestAddons.Flow[:16]`).
- UDP traffic is NOT rejected under this flow — it is rewritten to the mux
  tunnel (XUDP), so vision's UDP/443 rejection (SP1's guard) is lifted for
  the mux path. (xray: `allowUDP443 = true` → the UDP/443 rejection is
  skipped; the mux rewrite already applies to all UDP under vision/cone.)
- Vision TCP semantics unchanged (SP2's mux+vision composition applies —
  peel-inside; xray servers still reject vision+mux TCP by design, so the
  e2e for the mux+vision+udp443 row is sing-box-single-core).

### 4.4 Server response

Each packet dispatched to its dest returns via the same session; the
downlink frames are `[meta_len][sid][Keep][Data][0x02 UDP][port-first
dest][2B data_len][payload]` (the server echoes the per-packet dest it
received, or the client matches by dest). The client's XUDP receive:
unframes per-packet dest + payload; `recv` returns `(dest, payload)`.

## 5. Architecture

### 5.1 MuxClient UDP sessions (native `protocol/vless/mux.rs`)

The SP2 `MuxClient` gains a UDP-session path alongside `open_session`:

```rust
/// A UDP packet handle over one mux UDP session.
pub(crate) struct UdpSession { /* same channel model as SessionStream */ }
impl UdpSession {
    /// Send one datagram to `dest` — writes a Keep data frame with the
    /// per-packet dest (New frame on the first send, carrying the GlobalID).
    pub(crate) async fn send_to(&self, dest: SocketAddr, payload: &[u8]) -> io::Result<()>;
    /// Receive one datagram — returns (dest, payload). EOF/End → None.
    pub(crate) async fn recv_from(&mut self) -> io::Result<Option<(SocketAddr, Vec<u8>)>>;
}
```

- `open_udp_session(global_id: [u8; 8]) -> UdpSession`: allocates a session
  id, registers the channel; the FIRST `send_to` writes the **New** frame
  (network=UDP + dest + GlobalID); subsequent writes are **Keep** frames
  with the per-packet dest. (Eager-New is NOT used here — the UDP session
  needs a dest per packet; the New frame's dest = the first packet's dest,
  matching xray.)
- The demux already routes by session id; the UDP frames' per-packet dests
  are carried in the frame metadata and returned by `recv_from`.
- End/Error/KeepAlive handling identical to SP2's SessionStream.

### 5.2 PacketConn XUDP mode (native `protocol/vless/packet.rs`)

The SP1 `PacketConn` gains a third mode:

```rust
pub(crate) enum PacketMode { Raw, PacketAddr, XUdp }
```

- `PacketConn::new(inner, mode)` where `inner` is the `UdpSession` for XUdp:
  - `send(dest, payload)` → `udp_session.send_to(dest, payload)`.
  - `recv()` → `udp_session.recv_from()` → `(Some(dest), payload)`.
  - No 2-byte length framing (the mux frames carry the length) — the XUdp
    mode delegates entirely to the UdpSession.
- The raw and packetaddr modes are unchanged.

### 5.3 Integration (native `protocol/vless/mod.rs` + crate root)

- `connect_udp` with mux enabled (a params flag or the flow):
  - opens the SP2 mux tunnel (`connect_mux`) when needed,
  - `mux_client.open_udp_session(random_8_bytes())` → `PacketConn` in XUdp
    mode.
- Flow handling:
  - `flow == "xtls-rprx-vision-udp443"` → accepted; the header addon uses
    the truncated `xtls-rprx-vision`; UDP allowed (no rejection); the
    tunnel is the vision mux tunnel (SP2's composition: peel-inside +
    VisionStream + mux).
  - `flow == "xtls-rprx-vision"` + UDP via the RAW path → SP1's guard still
    rejects; UDP via mux → XUDP (no rejection).
- The client JSON/params: the flow string flows through `VlessConfig.flow`
  (proto unchanged — the truncation happens at encode time).

### 5.4 Transport composition

XUDP rides the same transports as mux (tcp/ws/grpc/httpupgrade/xhttp/h2 +
TLS/REALITY) — the mux tunnel is a byte stream; UDP packets ride its frames.

## 6. Error Handling

- Tunnel death → all UDP sessions fail (End events) like SP2's sessions.
- Oversized datagram (> 65535 − frame overhead): `NativeError` (client
  error, never truncated).
- Per-packet dest parsing on receive: malformed → error, never garbage.
- Empty GlobalID: the server falls back to the plain UDP session path —
  the client always sends a non-empty random one.
- `send_to` on a closed session → the tunnel-closed error (SP2 semantics).

## 7. Testing

### 7.1 Unit (native crate)

- UdpSession frame shapes: New frame (network=UDP + dest + GlobalID),
  Keep frames with per-packet dests; multiple dests across sends.
- PacketConn XUdp mode: send/recv round-trips with (dest, payload) pairs;
  EOF/End.
- Flow handling: `vision-udp443` accepted + wire truncation (addon =
  `xtls-rprx-vision`); vision + raw UDP still rejected (SP1 guard intact);
  vision + mux UDP not rejected.
- GlobalID: random 8 bytes, non-empty, per tunnel.

### 7.2 Hermetic

Extend the SP2 fake-mux-server fixture: a UDP session (New with
network=UDP + dest + GlobalID), per-packet Keep frames with distinct dests,
server replies per dest; the client's PacketConn XUdp recv returns the
( dest, payload) pairs.

### 7.3 e2e (tier-3, `native-e2e` + `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin`)

Harness: the SP1 `UdpEchoServer` + `probe_udp` already exist; add a
`probe_udp_mux` (connect_udp with mux → send N datagrams to the UDP echo
through the mux tunnel → assert echoes). Rows:

| Row | Core | Security | Flow | Proves |
|---|---|---|---|---|
| xudp_tls | xray + sing-box | tls | none | UDP over mux (XUDP) |
| xudp_reality | xray + sing-box | reality | none | XUDP over REALITY |
| vision_udp443 | sing-box (single-core) | tls | vision-udp443 | udp443 flow via vision mux |

= **5 rows** (2+2+1). Existing rows untouched (vless 66+4 → 71+4; total
114+4 → 119+4). The vision_udp443 row: sing-box server only (xray rejects
vision+mux TCP by design — SP2 finding).

### 7.4 Gates

- clippy workspace pedantic+nursery 0; `cargo fmt --check` clean.
- unit: existing + new green; e2e vless 71+4, vmess 48.
- `xray-tui-proto` NOT modified. No new deps (blake3 NOT needed — random
  GlobalID). `NativeError`-only surface. Timeout-bounded network steps.

## 8. Deviations from Upstream

1. **Random GlobalID** vs xray's `blake3-8(BaseKey, source-addr)` — the
   server requires only non-empty; the bytes are never validated. Cost if
   wrong: a server that groups by GlobalID would see one key per tunnel
   instead of per source — functionally equivalent for a single-user client.
2. **Frame-metadata path with real session ids** vs xudp's raw sid-0
   frames — the server accepts both; our path reuses the SP2 demux.
3. **First-packet dest in the New frame** (xray's PacketWriter uses the
   writer's bound dest for New, per-packet dests for Keep) — the New frame
   carries the first send's dest; per-packet dests after. Wire-compatible.
4. **udp443 over sing-box only in e2e** — xray's server-side vision+mux TCP
   rejection (AllowedNetwork=UDP) is upstream behavior, not our choice.

## 9. Verification Order

1. Unit: UdpSession frames + PacketConn XUdp + flow truncation + guards.
2. Hermetic: fake mux server UDP-session frames.
3. e2e: vless sweep (71+4) + vmess (48) + clippy + fmt.
