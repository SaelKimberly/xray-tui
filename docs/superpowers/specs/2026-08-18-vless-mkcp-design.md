# VLESS mKCP Transport — Design Spec (SP4 of VLESS Parity)

Date: 2026-08-18. Branch: `native-core-stub`.
SP4 of the VLESS parity program. SP1 (UDP path) + SP2 (v1.mux.cool) + SP3
(XUDP + vision-udp443) done. SP4 = the **mKCP transport** — xray's fork of
the KCP reliable-stream protocol over UDP — as a dial-side transport in
`xray-tui-native`, interoperable with xray-core 26.3.27 servers.

## 1. Goal

Add a native mKCP client: a UDP-dialing KCP session (segments, send/recv
windows, retransmit + fast-resend, RTO/RTT estimation, congestion control,
ping/terminate state machine) exposing `AsyncRead + AsyncWrite`, wired into
the existing transport composition so the security layer (TLS/REALITY) wraps
the KCP stream exactly as it wraps the TCP stream today. Interoperable with
xray-core 26.3.27 `transport` type `kcp`.

## 2. Non-Goals (scope boundaries for SP4)

- **Server-side mKCP (listener)** — client only; e2e uses a real xray-core
  kcp listener.
- **udpmask / header obfuscation** — xray's `UdpmaskManager` is an optional,
  non-default wrapper (default kcp config sets none; the dialer checks
  `streamSettings.UdpmaskManager != nil` and default configs leave it nil).
  Raw KCP datagrams only.
- **FEC / legacy header options** — removed from the 26.3.27 wire (segment
  types: Data/Ack/Ping/Terminate only; no `SegmentOptionFEC`).
- **sing-box parity** — sing-box has no mKCP; e2e rows are xray-single-core.
- **Any new Rust dependency** — the KCP state machine is implemented in-crate
  (tokio `UdpSocket` + `Notify` + `time` are already deps).

## 3. Reference Inventory (wire authority)

- `thirdparty/Xray-core/transport/internet/kcp/` — `segment.go`
  (serialization), `connection.go` (session + updaters + flush + states),
  `sending.go` (send window), `receiving.go` (recv window + ack list),
  `output.go` (one segment per UDP datagram), `dialer.go` (conv counter),
  `config.go` + `config.proto` (settings + getters). Version: 26.3.27.

## 4. Wire Spec

All integers big-endian. **One segment per UDP datagram** (`SimpleSegmentWriter`
serializes a single segment then writes to the UDP conn — no batching).

### 4.1 Segment formats (18B data overhead)

**DataSegment** (cmd=1):
```
[conv u16][cmd=1 u8][opt u8][ts u32][sn u32][una u32][len u16][payload]
```
`mss = mtu - 18` (default 1350-18 = 1332). `opt`: 0 = normal,
`SegmentOptionClose` = 1 (set when the local state is ReadyToClose).

**AckSegment** (cmd=0):
```
[conv u16][cmd=0 u8][opt u8][rcv_wnd u32][rcv_nxt u32][ts u32][count u8][numbers u32*count]
```
`rcv_wnd = rcv_nxt + window_size`; `count` is a single byte; the ack list holds
up to `(mss-17)/4` numbers (the `ts` of each acked number is echoed in the
SendingWorker's RTT update — see §5.3).

**CmdOnlySegment** — Ping (cmd=3) and Terminate (cmd=2):
```
[conv u16][cmd u8][opt u8][snd_nxt u32][rcv_nxt u32][peer_rto u32]   (16B)
```
`snd_nxt` = first unacknowledged; `rcv_nxt` = next expected; `peer_rto` = the
peer's current RTO (adopted by the receiver every ≥3000ms).

**Parse**: read `[conv u16][cmd u8][opt u8]` (≥4B required); conv mismatch
drops the rest of the datagram; then per-cmd: Data needs a ≥15-byte body
(Go's `len(buf) < 15` check — 16 fixed bytes + payload, where the declared
`len` must match the remaining bytes; the Go check accepts a 15-byte body);
Ack needs ≥13 more (rcv_wnd/rcv_nxt/ts/count) + count×4 numbers; CmdOnly
needs 12 more.

### 4.2 Conversation id

Client picks `conv: u16` from a process-global counter
(`globalConv = dice.RollUint16(); conv = ++globalConv`), sent in every
segment. The server learns the session from the first packet's conv. Rust:
`AtomicU16` seeded randomly at first use, `fetch_add(1)` per dial.

### 4.3 Connection state machine

States: Active(0), ReadyToClose(1, closed locally), PeerClosed(2, remote
sent Close opt), Terminating(3, ready to destroy locally), PeerTerminating(4,
remote sent Terminate), Terminated(5).

- **Close()** (local): Active → ReadyToClose (outgoing Data/Ack/CmdOnly now
  carry `opt=Close`); PeerClosed → Terminating; PeerTerminating → Terminated.
- **flush()** per tick:
  - Active and now-lastIncoming ≥ 30000 → Close().
  - ReadyToClose and send window empty → Terminating.
  - Terminating → send one Terminate; if in state > 8000ms → Terminated; return.
  - PeerTerminating in state > 4000ms → Terminating.
  - ReadyToClose in state > 15000ms → Terminating.
  - receivingWorker.Flush (ack list), sendingWorker.Flush (send window).
  - now-lastPing ≥ 3000 → send Ping.
- **OnPeerClosed** (received Close opt): ReadyToClose → Terminating;
  Active → PeerClosed.
- **OnTerminate** (received cmd=2): Active/PeerClosed → PeerTerminating;
  ReadyToClose → Terminating; Terminating → Terminated.
- **Read**: Active/PeerClosed → wait for data; ReadyToClose/Terminating/
  Terminated → EOF; PeerTerminating → EOF.
- **Write**: state != Active → `io::ErrorKind::WriteZero`/closed-pipe error.
- **Idle guard**: no incoming for 30s → Close (the session dies, TLS fails
  loudly — matches xray; the e2e timeout is shorter anyway).

### 4.4 Timers (two updaters)

- **dataUpdater**: ticks every `tti` (default 50ms) while
  `send_window nonempty || ack_list nonempty`; each tick runs flush().
- **pingUpdater**: ticks every 5000ms while not Terminated; **wakes
  immediately at connect** (run flush once — sends nothing until the first
  Write, per §4.3 rules: empty ack list + empty send window + lastPing≈0 not
  ≥3000).
- Rust: one tokio task per session holding a `mpsc`/`Notify`-driven tick
  (data-tick on demand via a `Notify` when the windows become nonempty;
  ping-tick via `tokio::time::interval` at 5000ms; flush on every wake),
  or an equivalent single-loop design with the same wake/stop conditions.

### 4.5 Settings (xray defaults)

`mtu=1350, tti=50, uplink_capacity=5 (Mbps), downlink_capacity=20 (Mbps),
cwnd_multiplier=1, max_sending_window=2*1024*1024`.

- `snd_inflight = uplink*1MiB/mtu/(1000/tti)`, min 8 → default 194.
- `rcv_inflight = downlink*1MiB/mtu/(1000/tti)`, min 8 → default 777.
- `snd_buf (windowSize) = max_sending_window/mtu` → default 1553.
- `mss = mtu - 18` → 1332.
- The native params expose only mtu/tti (everything else derives; xray's
  other fields accepted if the params struct carries them, but defaults are
  the interop target).

## 5. Architecture (native crate)

New `crates/xray-tui-native/src/transport/kcp/` module; the existing
transport composition gains a kcp arm.

### 5.1 `transport/kcp/wire.rs` — segment encode/parse

- `enum Segment { Data { conv, opt, ts, sn, una, payload: Bytes }, Ack {
  conv, opt, rcv_wnd, rcv_nxt, ts, numbers: Vec<u32> }, CmdOnly { conv, cmd,
  opt, snd_nxt, rcv_nxt, peer_rto } }` + `encode(seg, buf)` /
  `parse_datagram(&[u8]) -> Result<Vec<Segment>>` (one datagram → at most one
  segment in practice; the loop mirrors `KCPPacketReader`).
- Byte-exact unit tests against the §4.1 layouts.

### 5.2 `transport/kcp/session.rs` — the KCP core

- `KcpSession { conv, state, sending: SendingWorker, receiving:
  ReceivingWorker, round_trip: RoundTripInfo, mss, config, last_incoming,
  last_ping, since (ms clock), output (UdpSocket send), ... }`.
- **RoundTripInfo**: `srtt/variation/rto/min_rtt/updated_ts`; init
  `rto=100, min_rtt=tti`. `Update(rtt)`: RFC 6298-style (§ first sample:
  srtt=rtt, variation=rtt/2; else variation=(3v+Δ)/4, srtt=(7s+rtt)/8,
  srtt≥min_rtt); `rto = (srtt + 4v) if min_rtt < 4v else (srtt + v)`, cap
  10000, `rto = rto*5/4`. `UpdatePeerRTO(rto)`: adopt if ≥3000ms since last.
- **SendingWorker**: `first_unacked, next_sn, remote_rcv_nxt=32,
  control_window=snd_inflight, window_size=snd_buf, fast_resend=2`;
  window = ordered map sn → {payload, timeout, transmit}. `push(payload)`
  (reject when window len ≥ window_size). `flush(current, rto, cwnd)`:
  `cwnd = min(snd_inflight, remote_rcv_nxt - first_unacked, control_window)
  × cwnd_multiplier`; visit in order: skip segments whose
  `current - timeout` wrapped (≥ 0x7FFFFFFF); `transmit==0` → in_flight++
  else lost++; set `timeout = current + rto`, `ts = current`, `transmit++`,
  write Data (una = first_unacked, opt = Close when ReadyToClose); stop after
  `cwnd` segments; then `on_packet_loss(lost*100/in_flight_total)`:
  ≥15% → control_window = 3/4; ≤5% → += 1/4; min 16; cap snd_inflight.
- **Ack processing**: `ProcessSegment(ack)`: update `remote_rcv_nxt = max(
  ...)`, clear window < `ack.rcv_nxt` (recompute first_unacked), then per
  number: `process_ack` (out of `[first_unacked, next_sn)` wrap-range →
  ignore) removes the segment; the max acked number removed → fast-resend
  (`HandleFastAck`: segments with sn > maxack, wrap-correct: `transmit > 0 &&
  timeout > rto/3` → `timeout -= rto/3`) and RTT update
  (`current - ack.ts < 10000` → `round_trip.Update(current - ack.ts)`).
- **ReceivingWorker**: `next_number`, `window_size = rcv_inflight`, recv
  window = map sn → payload; `process_data(seg)`: drop when
  `sn - next_number` wrapped ≥ window_size; acklist: clear numbers <
  `seg.una`, add (sn, ts). `read_available()` → consecutive payloads from
  next_number (detach, next_number++). Ack list flush per tick: build one
  AckSegment with up to `(mss - 17)/4` due numbers (per-number next_flush =
  current + max(rto/2, 20)); not-yet-due numbers appended as
  flush-candidates if space; write with `rcv_wnd = next_number +
  window_size`.
- **Timers/wake**: data-updater ticks while `send_window nonempty ||
  acklist nonempty` (Notify on push/data/ack); ping tick 5000ms; both run
  flush(). The Rust session uses one async task + `tokio::time::interval`
  (or `sleep_until` re-arm) with the same continue/terminate conditions.
- **Backpressure (write)**: split into mss chunks; `push` while the window
  accepts; when full, wait for the ack-notifier (`dataOutput` equivalent)
  and retry; state != Active → closed error.
- **Input**: `input(datagram)`: parse; conv mismatch → drop rest; dispatch:
  Data → HandleOption(close) + recv + dataInput.signal() + wake data-updater;
  Ack → HandleOption + send-worker ack process + dataOutput.signal() + wake;
  CmdOnly → HandleOption + Terminate-state transitions + ProcessReceivingNext
  + ProcessSendingNext + UpdatePeerRTO.
- **Read**: `read(buf)`: drain consecutive recv-window payloads; if none,
  wait dataInput notifier (bounded by the 16s xray-style wait — simpler: wait
  with no explicit timeout, EOF/states resolve it); states → EOF.

### 5.3 The stream adapter (`transport/kcp/stream.rs`)

`KcpStream` wraps the session and implements `AsyncRead + AsyncWrite`
(direct session.read/write with the notifier-driven await — no extra
buffering). This is what the security layer (`security::wrap`) consumes, so
the existing TLS/REALITY path works unchanged: `transport::dial` gains a kcp
arm producing `Box<dyn AsyncReadWrite>` whose inner is the KcpStream, and
`security::wrap` wraps it as it wraps the TCP stream today.

### 5.4 Transport integration (`transport/mod.rs` + params)

- The native `NativeConnectParams` transport field gains `Transport::Kcp {
  mtu: Option<u32>, tti: Option<u32> }` (or a kcp sub-struct carrying the
  xray-config fields accepted; defaults per §4.5). The xray config-builder
  side (`xray-tui-proto`/config_builder) is untouched — the native e2e
  harness constructs the params directly; TUI-side kcp support is a follow-up
  (out of scope, like the other transports' TUI wiring).
- `dial_kcp(addr, params)`: `UdpSocket::bind("0.0.0.0:0")` + `connect(addr)`;
  spawn the session's input task (`recv` loop → `session.input`); return the
  KcpStream.
- Transport composition: kcp → security wrap (TLS/REALITY) → protocol
  (VLESS), exactly the xray order.

## 6. Error Handling

- Session errors: `io::Error` (`ConnectionReset`/`BrokenPipe` for the
  close/terminate states, `TimedOut` for the 30s idle guard, `InvalidInput`
  for wire parse failures — a malformed datagram is dropped with a
  `tracing::debug!`, never fatal).
- Write backpressure never fails (bounded by the snd_buf window).
- The UDP socket error on send/recv surfaces through the stream; the input
  task logs and stops on socket error (session read then EOFs).

## 7. Testing

### 7.1 Unit (native crate)

- wire.rs: byte-exact encode/parse for all three segment types + parse
  truncation/corruption cases (conv mismatch, short datagrams, bad cmd).
- session.rs: RTO/RTT arithmetic (first-sample, srtt/variation smoothing,
  cap 10000, ×5/4, peer-rto adoption ≥3000ms); send-window flush (in-order
  transmission, cwnd stop, retransmit on timeout, fast-resend via ack,
  loss-rate cwnd adjustment); recv-window reassembly (in-order read,
  gap fill, duplicate drop, wrap-around sn arithmetic); ack-list flush
  (batching, per-number due times, flush-candidates); state machine
  (close→terminate→terminated, peer-close, terminate handshake, 30s idle).

### 7.2 Hermetic

- A Rust fake mKCP peer over UDP (loopback socket): mirrors the server side
  minimally — recv window + ack echo + optional out-of-order/loss injection —
  driving end-to-end session tests (data flow both directions, retransmit
  under injected loss, close handshake, ping cadence) without real cores.
  Gated behind the existing hermetic test pattern (unit, no `native-e2e`).

### 7.3 e2e (tier-3, `native-e2e` + `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin`)

- xray-single-core rows (sing-box has no kcp): kcp + tls, kcp + reality,
  kcp plain (no TLS) — the three composition arms, one each. Harness: the
  row's transport is Kcp, the server config carries
  `"streamSettings":{"network":"kcp"}` (xray config builder side in the
  e2e harness, like the ws/grpc rows).
- Count: vless +3 (plain/tls/reality), vmess unchanged, total 123 → **126**.

### 7.4 Gates

- `cargo clippy -p xray-tui-native` and `--features native-e2e`: 0.
- `cargo fmt --check`: clean.
- `cargo test -p xray-tui-native --lib`: green (new unit + hermetic).
- `cargo test -p xray-tui-native --features native-e2e --lib`: green.
- `cargo test -p xray-tui-native --features native-e2e --test vless` (72+4
  existing + 3 new = 75+4) and `--test vmess` (48).
- No `xray-tui-proto` changes.

## 8. Deviations from Upstream

1. **No udpmask** — xray's optional `UdpmaskManager` wrapper is skipped
   (non-default; §2).
2. **Client-side timer model** — xray's two updater goroutines collapse into
   one session task with equivalent wake/stop conditions (§4.4); behavior
   identical.
3. **`waitForDataInput` spin** — xray spins 16 Gosched then waits 16s; the
   Rust read waits on the notifier (tokio) with no artificial spin; the
   semantics (block until data or EOF) match.
4. **Read-after-Terninate timing** — Rust surfaces EOF immediately on the
   Terminated state; xray's `PeerTerminating` → EOF, `Terminating`/`Terminated`
   → EOF as well — same observable behavior.

## 9. Verification Order

1. wire.rs + unit (codec byte-exact).
2. session.rs + unit (windows, RTO, state machine) + hermetic peer (real UDP
   session, retransmit, close).
3. stream.rs + transport integration.
4. e2e rows (xray kcp: plain/tls/reality).
5. Docs (NATIVE_CORE.md mKCP row + counts) + full gates.
