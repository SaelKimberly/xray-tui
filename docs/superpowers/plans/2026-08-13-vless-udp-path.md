# VLESS UDP Path Implementation Plan (SP1)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the VLESS UDP command path (command 0x02) to the xray-tui-native VLESS client — 2-byte-length packet framing both directions, a `PacketConn` datagram API, packetaddr mode (sing-box), vision+UDP rejection, and UDP e2e rows.

**Architecture:** The existing chain (dial → security → transport) is reused unchanged; only the protocol layer differs — command byte 0x02, header dest = the UDP target, and the post-header byte stream carries `[2B len][payload]` frames. A new `PacketConn` wraps the framed stream with the datagram API; the response-header peel is extracted from `VlessClientStream` into a shared helper. packetaddr = magic-address per-packet destinations (sing-box/mihomo mode).

**Tech Stack:** tokio (AsyncRead/AsyncWrite/UdpSocket), bytes, ring (unused here — no new deps), the existing engine TLS/REALITY, rstest e2e vs xray 26.3.27 + sing-box 1.13.16.

**Spec:** `docs/superpowers/specs/2026-08-13-vless-udp-path-design.md` (wire authority: xray 26.3.27 `proxy/vless/encoding/addons.go` LengthPacketWriter/Reader; sing-vmess packetaddr).

## Global Constraints

- xray-tui-proto NEVER modified. No new dependencies. `NativeError`-only error surface, existing variants.
- Wire (spec §4): header `version|uuid|addons|cmd=0x02|port-first dest`; frames `[2B BE len][payload]` both directions; empty frames skipped; outgoing ≤ 65535; packetaddr payload = `"sp.packet-addr.v2fly.arpa" | atyp(1B) | addr | port(2B BE)` (atyp 0x01 IPv4 / 0x02 IPv6; fqdn = error).
- **Correction (spec §4.3, verified at e2e):** packetaddr frames carry `atyp | addr | port` with NO magic — the magic fqdn is the header destination only (the wire bullet above predates the correction; see the spec's §4.3 for the sing-vmess evidence).
- vision + UDP → `NativeError` (guard; SP3 will lift it via XUDP).
- Clippy workspace pedantic+nursery 0; `cargo fmt --check` clean; every network step timeout-bounded (harness convention).
- e2e: `native-e2e` feature + `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin`. Existing rows untouched (vless 56+4 → 61+4, vmess 48).

---

### Task 1: Packet framing codec + shared response peel

**Files:**
- Create: `crates/xray-tui-native/src/protocol/vless/udp.rs`
- Modify: `crates/xray-tui-native/src/protocol/vless/mod.rs` (declare `mod udp;`)
- Modify: `crates/xray-tui-native/src/protocol/vless/stream.rs` (extract the peel into a reusable helper)

**Interfaces:**
- Produces:
```rust
// udp.rs — pure framing codec, unit-testable over duplex.
pub(crate) async fn read_packet<R: tokio::io::AsyncRead + Unpin>(r: &mut R) -> io::Result<Option<Vec<u8>>>;
pub(crate) async fn write_packet<W: tokio::io::AsyncWrite + Unpin>(w: &mut W, payload: &[u8]) -> io::Result<()>;
// None on clean EOF at a frame boundary; empty frames (len 0) skipped.

// stream.rs — extracted peel helper (VlessClientStream uses it; PacketConn uses it in Task 2).
pub(crate) struct Peel { /* existing state machine */ }
impl Peel {
    pub(crate) fn new() -> Self;
    /// Reads the VLESS response header (version|addons_len|addons) once.
    /// Reuses the EXACT current VlessClientStream logic (eager/lazy-server-safe,
    /// fragmented header, non-vless passthrough, EOF-before-header error).
    pub(crate) async fn ensure_peeled<R: tokio::io::AsyncRead + Unpin>(&mut self, r: &mut R) -> io::Result<()>;
}
```

- [ ] **Step 1: Write the failing tests (udp.rs)**

```rust
#[tokio::test]
async fn write_then_read_roundtrip() {
    let (mut a, mut b) = tokio::io::duplex(1024);
    write_packet(&mut a, b"hello").await.unwrap();
    drop(a);
    assert_eq!(read_packet(&mut b).await.unwrap().unwrap(), b"hello");
}

#[tokio::test]
async fn exact_wire_bytes() {
    let (mut a, mut b) = tokio::io::duplex(64);
    write_packet(&mut a, b"hi").await.unwrap();
    drop(a);
    let mut raw = Vec::new();
    b.read_to_end(&mut raw).await.unwrap();
    assert_eq!(raw, [0x00, 0x02, b'h', b'i']);
}

#[tokio::test]
async fn split_frame_reads() {
    // Feed [0x00, 0x04, 0xAA] then [0xBB, 0xCC, 0xDD] in two writes (duplex 64,
    // two write_packet calls on the peer side); read_packet must reassemble.
}

#[tokio::test]
async fn eof_at_boundary_is_none() {
    // Peer closes cleanly (no partial frame): read_packet → Ok(None).
}

#[tokio::test]
async fn empty_frame_skipped() {
    // Peer writes len=0 frame then a real frame: read_packet returns the real
    // payload, never an empty Vec.
}

#[tokio::test]
async fn truncated_frame_is_error() {
    // Peer writes [0x00, 0x05, 'a'] then closes: read_packet → Err (UnexpectedEof).
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p xray-tui-native protocol::vless::udp`
Expected: FAIL (module doesn't exist).

- [ ] **Step 3: Implement udp.rs**

```rust
use std::io;
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};

pub(crate) async fn read_packet<R: AsyncRead + Unpin>(r: &mut R) -> io::Result<Option<Vec<u8>>> {
    let mut len = [0u8; 2];
    match r.read_exact(&mut len).await {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let n = usize::from(u16::from_be_bytes(len));
    if n == 0 { return read_packet(r).await; }  // skip empty frames (recursion depth bounded by caller)
    let mut payload = vec![0u8; n];
    r.read_exact(&mut payload).await?;
    Ok(Some(payload))
}

pub(crate) async fn write_packet<W: AsyncWrite + Unpin>(w: &mut W, payload: &[u8]) -> io::Result<()> {
    debug_assert!(payload.len() <= u16::MAX as usize);
    let mut frame = Vec::with_capacity(payload.len() + 2);
    frame.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    frame.extend_from_slice(payload);
    w.write_all(&frame).await
}
```

(Clippy note: the recursive empty-skip is fine but a `loop` is cleaner — prefer a loop. Keep the EOF/truncation semantics exact.)

- [ ] **Step 4: Extract the Peel helper (stream.rs)**

Read `stream.rs` (VlessClientStream, ~28-130). Extract the response-header peel state machine into `pub(crate) struct Peel` with the `ensure_peeled` method — the EXACT same logic, no behavior change. `VlessClientStream` holds `Peel` and delegates. The TCP e2e rows are the regression net.

- [ ] **Step 5: Run tests + commit**

Run: `cargo test -p xray-tui-native` (unit, no e2e feature)
Expected: all green (new udp tests + existing; the peel refactor is behavior-identical).

```bash
git add crates/xray-tui-native/src/protocol/vless/udp.rs crates/xray-tui-native/src/protocol/vless/mod.rs crates/xray-tui-native/src/protocol/vless/stream.rs
git commit -m "feat(native): VLESS UDP packet framing codec + shared response peel"
```

---

### Task 2: PacketConn + packetaddr codec

**Files:**
- Create: `crates/xray-tui-native/src/protocol/vless/packet.rs` (PacketConn)
- Create: `crates/xray-tui-native/src/protocol/vless/packetaddr.rs` (destination header codec)
- Modify: `crates/xray-tui-native/src/protocol/vless/mod.rs` (declare modules)

**Interfaces:**
- Consumes: Task 1 (`read_packet`/`write_packet`, `Peel`).
- Produces:
```rust
// packet.rs
pub(crate) enum PacketMode { Raw, PacketAddr }
pub(crate) struct PacketConn<S> {
    inner: S,
    mode: PacketMode,
    peel: Peel,
    in_buf: bytes::BytesMut,
}
impl<S: AsyncRead + AsyncWrite + Unpin> PacketConn<S> {
    pub(crate) fn new(inner: S, mode: PacketMode) -> Self;
    /// Send one datagram. Raw: one [len][payload] frame. PacketAddr: prepends
    /// the magic-address destination header (spec §4.3) to the payload.
    pub(crate) async fn send(&mut self, target: Option<SocketAddr>, payload: &[u8]) -> io::Result<()>;
    /// Receive one datagram. Raw: returns (None, payload). PacketAddr: returns
    /// (Some(dest), payload) after validating the magic. None = clean EOF.
    pub(crate) async fn recv(&mut self) -> io::Result<Option<(Option<SocketAddr>, Vec<u8>)>>;
}
// packetaddr.rs
pub(crate) const MAGIC: &str = "sp.packet-addr.v2fly.arpa";
pub(crate) fn encode_dest(addr: SocketAddr) -> io::Result<Vec<u8>>;  // Err on fqdn — SocketAddr is IP-only, so always Ok
pub(crate) fn decode_dest(data: &[u8]) -> io::Result<(SocketAddr, &[u8])>;  // magic + atyp + addr + port, rest = payload
```

- [ ] **Step 1: Write the failing tests**

```rust
// packetaddr.rs
#[test] fn encode_ipv4() { /* 127.0.0.1:8080 → MAGIC.as_bytes() + [0x01, 127,0,0,1, 0x1F,0x90] */ }
#[test] fn encode_ipv6() { /* [::1]:53 → MAGIC + [0x02, 16×0x00…0x01, 0x00,0x35] */ }
#[test] fn decode_roundtrip() { /* encode then decode → same addr, payload intact */ }
#[test] fn decode_bad_magic_is_error() { /* wrong magic prefix → Err */ }
#[test] fn decode_truncated_is_error() { /* short atyp/addr → Err */ }

// packet.rs
#[tokio::test] async fn raw_send_recv() {
    // duplex pair: PacketConn over one end; raw read_packet on the other.
    // send(b"hi") → peer reads [0x00,0x02,'h','i']; peer writes frame; recv() returns it.
}
#[tokio::test] async fn packetaddr_send_recv() {
    // PacketConn in PacketAddr mode: send(Some(127.0.0.1:8080), b"p") →
    // peer reads magic+0x01+127.0.0.1+0x1F90+'p'; peer replies with a
    // magic-address frame for a different dest; recv returns (Some(dest), payload).
}
#[tokio::test] async fn peel_before_first_recv() {
    // Peer sends [0x00,0x00] (response header) then a frame: first recv()
    // returns the frame payload (header consumed), second recv() the next frame.
}
#[tokio::test] async fn eof_returns_none() { /* peer closes at a frame boundary → Ok(None) */ }
#[tokio::test] async fn oversized_send_is_error() {
    // send with a 70000-byte payload (PacketAddr or Raw) → Err, no partial frame.
}
```

- [ ] **Step 2: Run to verify they fail** — `cargo test -p xray-tui-native protocol::vless::packet` (FAIL: modules missing).

- [ ] **Step 3: Implement**

`packetaddr.rs`: `encode_dest` = `MAGIC.as_bytes()` + `[atyp]` + addr bytes (4/16) + `port.to_be_bytes()`; `decode_dest` validates the magic prefix, reads atyp → IPv4 4B / IPv6 16B (else Err), then 2B port; returns `(SocketAddr, &data[..])`.

`packet.rs`:
- `send`: if `mode == PacketAddr`, encode the dest header and append the payload, then ONE `write_packet` frame of the combined bytes (the sing encoder writes `AddrPortLen + payload` in one buffer — same). `target == None` in Raw mode → write the payload as-is; in PacketAddr mode → `target` is REQUIRED (Err if None). Reject combined length > 65535.
- `recv`: loop `read_packet`; on None → Ok(None). If not yet peeled → `peel.ensure_peeled(&mut inner).await?` before the first frame read (peel AFTER the first packet read returns? — no: the header precedes ALL frames, so ensure the peel on the FIRST recv call BEFORE reading the frame; the peel itself reads from `inner`). Then if PacketAddr: `decode_dest(&payload)` → validate magic; malformed → Err (never deliver garbage); return `(Some(dest), rest)`. Raw → `(None, payload)`.
- Buffer management: `read_packet` allocates per frame — acceptable at this layer (no zero-copy requirement; the payload IS the datagram). `in_buf` is only needed if the peel can leave partial bytes (it can't — the peel consumes exactly the header); drop `in_buf` if unused.

- [ ] **Step 4: Run tests + commit**

```bash
git add crates/xray-tui-native/src/protocol/vless/packet.rs crates/xray-tui-native/src/protocol/vless/packetaddr.rs crates/xray-tui-native/src/protocol/vless/mod.rs
git commit -m "feat(native): VLESS PacketConn datagram API + packetaddr destination codec"
```

---

### Task 3: UDP mode plumbing — connect_udp, vision guard, params field

**Files:**
- Modify: `crates/xray-tui-native/src/context.rs` (`NativeConnectParams`)
- Modify: `crates/xray-tui-native/src/protocol/vless/mod.rs` (UDP connect path + guard)
- Modify: `crates/xray-tui-native/src/lib.rs` (crate-root `connect_udp` entry — read the current root API and mirror the TCP entry)

**Interfaces:**
- Consumes: Task 2 (`PacketConn`, `PacketMode`).
- Produces:
```rust
// context.rs
// Reuse Task 2's PacketMode (re-export it from protocol::vless): one enum,
// one name. It lives in packet.rs; `use crate::protocol::vless::PacketMode;`
// (derive Clone/Copy/Debug/PartialEq on it in Task 2).
// NativeConnectParams gains: pub udp: Option<PacketMode>   (None = TCP path, current behavior)
// All existing constructors/defaults keep udp: None (no call-site churn).

// lib.rs (or wherever the crate-root connect entry lives — mirror the TCP one)
pub async fn connect_udp(params: &NativeConnectParams) -> Result<PacketConn<Box<dyn Stream>>, NativeError>;
```

- [ ] **Step 1: Read the current entry + write the guard test**

Read `crates/xray-tui-native/src/lib.rs` + `protocol/vless/mod.rs` connect() + `context.rs` NativeConnectParams + how the e2e harness calls connect (find the TCP connect entry name). Note the exact signatures in your report.

Write the failing guard test in `mod.rs` (unit):
```rust
#[tokio::test]
async fn vision_udp_is_rejected() {
    // Build params with flow = xtls-rprx-vision and udp = Some(Raw), a tcp
    // transport (mock the dial/security if the existing tests do — follow the
    // crate's test conventions; if connect needs real sockets, assert at the
    // guard function level: extract the guard as `fn check_udp_allowed(flow,
    // udp) -> Result<(), NativeError>` and unit-test THAT).
}
```
(If the crate has no unit-testable seam, extract `fn check_udp_allowed(flow: Option<&str>, udp: Option<PacketMode>) -> Result<(), NativeError>` in mod.rs and test it directly — simpler and deterministic.)

- [ ] **Step 2: Implement the UDP connect path**

In `protocol/vless/mod.rs`:
```rust
// Guard (extracted, unit-tested):
fn check_udp_allowed(flow: Option<&str>, udp: Option<PacketMode>) -> Result<(), NativeError> {
    if udp.is_some() && flow == Some(FLOW_XTLS_RPRX_VISION) {
        return Err(NativeError::NotImplemented(
            "VLESS vision does not support UDP (no XUDP mux)".into(),
        ));
    }
    Ok(())
}
```
The UDP connect (parallel to `connect_plain`): runs the same dial → security →
transport chain (reuse the existing helpers — read how `connect_plain` obtains
the secured tunnel), encodes the header with `cmd = CMD_UDP` and the target
(port-first), and for PacketAddr mode the header destination = the magic fqdn
(`packetaddr::MAGIC`, fqdn atyp — check `header.rs` encode_request supports
fqdn targets; it does — the existing code encodes domain addresses). Then
returns `PacketConn::new(tunnel, mode)`.

The crate-root `connect_udp(params)`: mirrors the TCP entry — runs the chain
with the UDP mode and returns the PacketConn.

- [ ] **Step 3: Compile + tests**

Run: `cargo test -p xray-tui-native` — all green (guard test + existing). Clippy 0 on the crate.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-native/src/context.rs crates/xray-tui-native/src/protocol/vless/mod.rs crates/xray-tui-native/src/lib.rs
git commit -m "feat(native): VLESS connect_udp + vision-UDP guard + packetaddr mode plumbing"
```

---

### Task 4: Hermetic fake UDP server

**Files:**
- Modify: `crates/xray-tui-native/src/protocol/vless/udp.rs` (or a new test module — mirror the T4 hermetic pattern from the vision plan: the raw fake server over the rustls TLS fixture)

**Interfaces:**
- Consumes: Task 3 (`connect_udp` — the unit under test).

- [ ] **Step 1: Write the hermetic test**

Mirror the vision-plan hermetic fixture (fake server = the rustls server double + raw socket):
1. Fake server accepts, does the outer TLS (rustls fixture), reads the VLESS header: assert `cmd == 0x02`, port-first dest matches.
2. Sends the `[0,0]` response header.
3. Reads a client frame `[2B len][payload]`; asserts the payload.
4. Sends a frame back; the client's `recv` returns it.
5. Optional (if the fixture makes it cheap): a packetaddr-mode variant asserting the magic-address bytes in the client's frame.

Run: `cargo test -p xray-tui-native protocol::vless::udp` — green.

- [ ] **Step 2: Commit**

```bash
git add crates/xray-tui-native/src/protocol/vless/udp.rs
git commit -m "test(native): hermetic fake UDP server — header cmd 0x02, packet frames"
```

---

### Task 5: e2e harness — UDP echo target + probe

**Files:**
- Modify: `crates/xray-tui-native/src/e2e/harness.rs` (UdpEchoServer + probe_udp)
- Modify: `crates/xray-tui-native/src/e2e/mod.rs` (ServerEnv gains udp_echo; run_against spawns it for UDP rows)
- Modify: `crates/xray-tui-native/src/e2e/case.rs` (CaseSpec gains an app-kind or target-kind for UDP — read the current AppKind {Plain, InnerTls} and extend with Udp; or a `network: Udp` axis — follow the existing pattern)

**Interfaces:**
- Consumes: Task 3 (`connect_udp`).
- Produces: `probe_udp(...)` used by Task 6 rows.

- [ ] **Step 1: Read the harness**

Read `e2e/harness.rs` (EchoServer, probe), `e2e/mod.rs` (ServerEnv, run_against), `e2e/case.rs` (CaseSpec, AppKind). Note how the native client is started and how `probe` drives HTTP through it.

- [ ] **Step 2: Add the UDP echo target + probe**

- `UdpEchoServer`: `tokio::net::UdpSocket` bound to `127.0.0.1:0`; a spawned task echoes every received datagram verbatim back to its source. `addr() -> SocketAddr`.
- `probe_udp(params_or_tunnel)`: connects via `connect_udp`, sends N (e.g. 3) distinct payloads (`b"udp-0"`, `b"udp-1"`, `b"udp-2"`), receives until all N echoes arrive (match by payload, order-independent) with a timeout; returns `(sent, received)` for assertion.
- The UDP row's target = the UdpEchoServer address. ServerEnv: `udp_echo: Option<SocketAddr>` spawned when the case needs it (mirror `inner_tls_echo`).

- [ ] **Step 3: Compile + existing sweeps**

Run: `cargo test -p xray-tui-native --features native-e2e --lib` (green) — the harness changes must not affect existing rows.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-native/src/e2e/
git commit -m "feat(native): e2e UDP echo target + probe_udp"
```

---

### Task 6: e2e UDP rows + sweeps

**Files:**
- Modify: `crates/xray-tui-native/tests/vless.rs`

**Interfaces:**
- Consumes: Task 5 (`probe_udp`, CaseSpec UDP axis).

- [ ] **Step 1: Add the 5 rows**

Follow the existing row-macro style. All rows: `protocol = vless`, UDP app kind, `udp = Some(Raw)` unless noted:

| Row | Cores | Security | Packet mode |
|---|---|---|---|
| udp_tls_plain | xray + sing-box | tls | Raw |
| udp_reality_plain | xray + sing-box | reality | Raw |
| udp_packetaddr_tls | sing-box (single-core) | tls | PacketAddr |

Use the `vless_against_cores`/`vless_single_core` patterns as established. The sing-box packetaddr row requires the sing-box server to accept packetaddr — the server-side unwrap is automatic (sing-box inbound packetaddr support); the row just needs the client JSON to carry `packet_encoding: "packetaddr"` — check how the client JSON is built in `e2e/config.rs` (the vless outbound may need a `packet_encoding` field in the client params — Task 3's params plumbing or a CaseSpec-driven config tweak; follow what Task 5 established).

- [ ] **Step 2: Run the vless sweep**

Run: `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vless`
Expected: 61 passed + 4 ignored (56+4 existing + 5 new). If the packetaddr row hits a server config issue, debug against spec §4.3 — do not relax assertions.

- [ ] **Step 3: Run the vmess sweep (regression)**

Run: `... --test vmess` — expected 48 passed unchanged.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-native/tests/vless.rs
git commit -m "feat(native): VLESS UDP e2e — 5 rows (xray+sing-box × tls+reality, sing-box packetaddr)"
```

---

### Task 7: Docs + full verification

**Files:**
- Modify: `NATIVE_CORE.md` (VLESS capability: UDP command path, PacketConn, packetaddr, vision+UDP rejection; e2e counts 104+4 → 109+4)
- Modify: `AGENTS.md` (native section: udp.rs/packet.rs/packetaddr.rs in the module list)

- [ ] **Step 1: Update docs** — actuals only (counts verified in Step 2 first).

- [ ] **Step 2: Full verification sweep**

```bash
cargo clippy --workspace --all-targets     # 0 warnings
cargo fmt --check                          # clean
cargo test -p xray-tui-native --features native-e2e --lib   # unit green
XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vless   # 61+4
XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vmess   # 48
```

- [ ] **Step 3: Commit**

```bash
git add NATIVE_CORE.md AGENTS.md
git commit -m "docs(native): VLESS UDP path — NATIVE_CORE + AGENTS + e2e counts"
```
