# VLESS XUDP + vision-udp443 Implementation Plan (SP3)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add UDP-over-mux (XUDP) to xray-tui-native — SP2's MuxClient gains UDP sessions (network=0x02 frames with per-packet destinations + an 8-byte GlobalID), SP1's PacketConn gains an XUDP mode, and the `xtls-rprx-vision-udp443` flow is accepted (wire truncation to `xtls-rprx-vision` + UDP via the vision mux tunnel).

**Architecture:** UDP sessions reuse SP2's MuxClient demux/channel machinery — the New frame carries network=UDP + the first dest + GlobalID; every Keep data frame carries its per-packet destination. `UdpSession` (send_to/recv_from) wraps the channel; SP1's `PacketConn` delegates its XUdp mode to it. The udp443 flow string is truncated to 16 bytes at encode; SP1's vision+UDP guard is lifted only for the mux path.

**Tech Stack:** tokio, bytes, ring (`ring::rand` for the random GlobalID — already a dep), the SP1 PacketConn + SP2 MuxClient, rstest e2e vs xray 26.3.27 + sing-box 1.13.16.

**Spec:** `docs/superpowers/specs/2026-08-13-vless-xudp-design.md` (wire authority: xray 26.3.27 `common/xudp/xudp.go` + `common/mux/{frame,server}.go` + `proxy/vless/outbound/outbound.go`).

## Global Constraints

- xray-tui-proto NEVER modified. No new dependencies (blake3 NOT needed — random GlobalID via `ring::rand`). `NativeError`-only surface.
- Wire (spec §4): New frame = `[meta_len][sid][0x01][opt][0x02 UDP][port-first dest][8B GlobalID][2B data_len][payload]`; Keep data frames carry the per-packet dest; server keys XUDPManager by GlobalID (non-empty; empty = plain UDP session path).
- udp443 flow: accepted; addon truncated to `xtls-rprx-vision` (16 bytes); UDP via mux NOT rejected (SP1's guard stays for the raw path).
- Random 8-byte GlobalID per tunnel (spec §8 deviation 1).
- Clippy workspace pedantic+nursery 0; `cargo fmt --check` clean; every network step timeout-bounded.
- e2e: `native-e2e` + `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin`. Existing rows untouched (vless 66+4 → 71+4, vmess 48).

---

### Task 1: MuxClient UDP sessions

**Files:**
- Modify: `crates/xray-tui-native/src/protocol/vless/mux.rs`

**Interfaces:**
- Consumes: the SP2 MuxClient/SessionStream machinery (demux, session registry, write queue, keepalive).
- Produces:
```rust
pub(crate) struct UdpSession { /* channel rx + session write handle */ }
impl UdpSession {
    /// First send writes the New frame (network=UDP + dest + GlobalID);
    /// subsequent sends are Keep frames with the per-packet dest.
    pub(crate) async fn send_to(&self, dest: SocketAddr, payload: &[u8]) -> io::Result<()>;
    /// Returns (dest, payload); None on EOF (End frame).
    pub(crate) async fn recv_from(&mut self) -> io::Result<Option<(SocketAddr, Vec<u8>)>>;
}
// MuxClient gains:
pub(crate) async fn open_udp_session(&self, global_id: [u8; 8]) -> io::Result<UdpSession>;
```

- [ ] **Step 1: Write the failing tests**

```rust
#[tokio::test] async fn udp_session_new_frame_bytes() {
    // open_udp_session([0xAA; 8]); send_to(127.0.0.1:8080, b"hi"):
    // peer reads New frame: [sid][0x01][0x01][0x02][0x1F,0x90][0x01][127,0,0,1]
    // [0xAA × 8][0x00,0x02]'hi' — exact bytes.
}
#[tokio::test] async fn udp_session_per_packet_dests() {
    // send_to(dest1, p1); send_to(dest2, p2) → Keep frames each carrying
    // their own port-first dest; the peer reads both.
}
#[tokio::test] async fn udp_session_recv_returns_dest() {
    // Peer sends Keep frames with different dests; recv_from returns each
    // (dest, payload) pair in order.
}
#[tokio::test] async fn udp_session_eof_on_end() { /* End → Ok(None) */ }
#[tokio::test] async fn udp_session_tunnel_death() { /* tunnel EOF → sessions end */ }
```

- [ ] **Step 2: Run to verify they fail** — `cargo test -p xray-tui-native protocol::vless::mux`.

- [ ] **Step 3: Implement**

- `UdpSession` mirrors `SessionStream`'s channel model but with `SessionEvent`-carried per-packet dests. Extend the session event: `Data { dest: Option<SocketAddr>, bytes }` for UDP sessions (the demux already parses the frame — add dest extraction for network=UDP frames; the frame's `target` on Keep frames = the per-packet dest; New frames' target = the first dest).
- `open_udp_session(global_id)`: allocate an id + channel (like `open_session`), store the global_id in the session entry; return `UdpSession { rx, id, write_tx, global_id, first: true }`.
- `send_to(dest, payload)`: if `first` → build the **New** frame (network=UDP + dest + GlobalID + payload), `first = false`; else build a **Keep** frame with the per-packet dest + payload. Both via the write queue (frame's `target: Some(MuxTarget::Tcp(...))` is repurposed for UDP dests — add a `MuxTarget::Udp(SocketAddr)` variant or carry the dest separately in the Frame struct; keep the wire encode exact: `[0x02 UDP][port-first addr]`).
- `recv_from`: rx.recv() → Data{dest, bytes} → `(dest, payload)`; End → None; Error → io::Error. Cancellation-safe buffered reads.
- The demux: for UDP sessions, route the per-packet dest with the payload (the frame already parsed the target — thread it through).

- [ ] **Step 4: Run tests + commit**

```bash
git add crates/xray-tui-native/src/protocol/vless/mux.rs
git commit -m "feat(native): VLESS mux UDP sessions — per-packet dests, GlobalID, UdpSession"
```

---

### Task 2: PacketConn XUDP mode + flow handling

**Files:**
- Modify: `crates/xray-tui-native/src/protocol/vless/packet.rs` (PacketMode::XUdp)
- Modify: `crates/xray-tui-native/src/protocol/vless/mod.rs` (flow truncation + guards)
- Modify: `crates/xray-tui-native/src/lib.rs` (connect_udp mux path)

**Interfaces:**
- Consumes: Task 1 (`UdpSession`, `MuxClient::open_udp_session`).
- Produces:
```rust
pub(crate) enum PacketMode { Raw, PacketAddr, XUdp }  // PacketConn delegates XUdp to UdpSession
// connect_udp gains a mux path: params.mux + flow handling
```

- [ ] **Step 1: Write the failing tests**

```rust
#[tokio::test] async fn xudp_send_recv_roundtrip() {
    // PacketConn in XUdp mode over a UdpSession (duplex fake): send(dest,
    // payload); peer replies; recv() → (Some(dest), payload).
}
#[test] fn flow_udp443_truncated_to_vision() {
    // encode_addons(Some("xtls-rprx-vision-udp443")) == the 16-byte
    // "xtls-rprx-vision" addon bytes.
}
#[tokio::test] async fn vision_udp_raw_still_rejected() {
    // flow=vision + UDP + RAW path → Err (SP1 guard intact).
}
#[tokio::test] async fn vision_udp443_mux_not_rejected() {
    // flow=udp443 + UDP + mux path → Ok (guard lifted).
}
```

- [ ] **Step 2: Run to verify they fail.**

- [ ] **Step 3: Implement**

- `packet.rs`: `PacketMode::XUdp`; `PacketConn::new` accepts a `UdpSession` for XUdp; `send(dest, payload)` → `session.send_to`; `recv()` → `(Some(dest), payload)`. No framing (the mux frames carry length).
- `mod.rs` flow handling:
  - `encode_addons`/the addon path: truncate `xtls-rprx-vision-udp443` to the first 16 bytes (`xtls-rprx-vision`) at encode time (spec §4.3).
  - The vision-UDP guard (`check_udp_allowed`): `vision-udp443` + UDP → allowed when the path is mux (XUDP); `vision` + UDP + raw → still rejected. Restructure the guard to take the path: `check_udp_allowed(flow, udp_mode, mux: bool)`.
- `lib.rs` `connect_udp`: when the params request mux (a `mux: bool` field or the flow being udp443), run the SP2 `connect_mux` tunnel, `open_udp_session(random_8_bytes())` (via `ring::rand::SystemRandom`), wrap in `PacketConn::new(session, PacketMode::XUdp)`.
- The params: `NativeConnectParams` gains `mux: bool` (default false) — or reuse an existing flag; read `context.rs` for the cleanest seam.

- [ ] **Step 3b: Compile + tests** — `cargo test -p xray-tui-native` green; clippy 0.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-native/src/protocol/vless/packet.rs crates/xray-tui-native/src/protocol/vless/mod.rs crates/xray-tui-native/src/lib.rs crates/xray-tui-native/src/context.rs
git commit -m "feat(native): PacketConn XUDP mode + vision-udp443 flow (truncation + mux path)"
```

---

### Task 3: Hermetic fake-mux UDP-session test

**Files:**
- Modify: `crates/xray-tui-native/src/protocol/vless/mux.rs` (test module, extend the SP2 fake-mux fixture)

**Interfaces:**
- Consumes: Task 2 (the XUDP path).

- [ ] **Step 1: Write the hermetic test**

Extend the SP2 fake-mux-server hermetic fixture: the client opens a UDP session (New frame with network=UDP + dest + GlobalID), sends per-packet Keep frames with distinct dests; the server replies per dest; the client's PacketConn XUdp recv returns the (dest, payload) pairs. Assert the GlobalID bytes + the per-packet dest framing.

Run: `cargo test -p xray-tui-native protocol::vless::mux` — green.

- [ ] **Step 2: Commit**

```bash
git add crates/xray-tui-native/src/protocol/vless/mux.rs
git commit -m "test(native): hermetic fake-mux UDP session — GlobalID, per-packet dests"
```

---

### Task 4: e2e harness — probe_udp_mux

**Files:**
- Modify: `crates/xray-tui-native/src/e2e/harness.rs` (probe_udp_mux)
- Modify: `crates/xray-tui-native/src/e2e/mod.rs` (routing: mux+UDP cases)
- Modify: `crates/xray-tui-native/src/e2e/case.rs` (CaseSpec: mux+UDP + flow=udp443 axes)

**Interfaces:**
- Consumes: Task 2 (connect_udp mux path).
- Produces: `probe_udp_mux(...)` used by Task 5.

- [ ] **Step 1: Read the harness** — extend the SP1 UDP probe: connect_udp with mux → send N datagrams through the mux tunnel to the UdpEchoServer → assert echoes (order-independent, deadline).

- [ ] **Step 2: Implement** — `probe_udp_mux(params)`: connect_udp(mux) → PacketConn XUdp → the SP1 probe loop (distinct payloads, match-by-payload echoes). CaseSpec: mux+UDP cases route here.

- [ ] **Step 3: Compile + existing sweeps** — feature-gated lib green; existing rows untouched.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-native/src/e2e/
git commit -m "feat(native): e2e XUDP harness — probe_udp_mux over the mux tunnel"
```

---

### Task 5: e2e XUDP rows + sweeps

**Files:**
- Modify: `crates/xray-tui-native/tests/vless.rs`

**Interfaces:**
- Consumes: Task 4.

- [ ] **Step 1: Add the 5 rows**

| Row | Cores | Security | Flow |
|---|---|---|---|
| xudp_tls | xray + sing-box | tls | none |
| xudp_reality | xray + sing-box | reality | none |
| vision_udp443 | sing-box (single-core) | tls | vision-udp443 |

- [ ] **Step 2: Run the vless sweep** — `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vless` → expected 71+4.
- [ ] **Step 3: vmess sweep** — 48 unchanged.
- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-native/tests/vless.rs
git commit -m "feat(native): VLESS XUDP e2e — 5 rows (xray+sing-box × tls+reality, vision-udp443 sing-box)"
```

---

### Task 6: Docs + full verification

**Files:**
- Modify: `NATIVE_CORE.md` (XUDP: UDP-over-mux, GlobalID, vision-udp443; counts 114+4 → 119+4)
- Modify: `AGENTS.md`

- [ ] **Step 1: Update docs** (actuals).
- [ ] **Step 2: Full verification**

```bash
cargo clippy --workspace --all-targets     # 0
cargo fmt --check                          # clean
cargo test -p xray-tui-native --features native-e2e --lib
XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vless   # 71+4
XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vmess   # 48
```

- [ ] **Step 3: Commit**

```bash
git add NATIVE_CORE.md AGENTS.md
git commit -m "docs(native): VLESS XUDP + vision-udp443 — NATIVE_CORE + AGENTS + counts"
```
