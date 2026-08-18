# VLESS mKCP Transport Implementation Plan (SP4)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the mKCP transport (xray's fork of the KCP reliable-stream protocol over UDP) to xray-tui-native: a UDP-dialing KCP session (segments, send/recv windows, retransmit + fast-resend, RTO/RTT, congestion control, ping/terminate state machine) exposing `AsyncRead + AsyncWrite`, wired into the transport composition so TLS/REALITY wrap the KCP stream exactly as they wrap the TCP stream today. Interoperable with xray-core 26.3.27 kcp listeners.

**Architecture:** New `crates/xray-tui-native/src/transport/kcp/` module: `wire.rs` (byte-exact segment codec), `session.rs` (the KCP core — windows, RTO/RTT, timers, flush, state machine), `stream.rs` (`KcpStream`: AsyncRead/AsyncWrite over the session). `transport/mod.rs` gains a kcp dial arm producing the same stream shape the security layer consumes. One segment per UDP datagram; no new dependencies (tokio UdpSocket + Notify + time, bytes — already deps). e2e rows are xray-single-core (sing-box has no kcp).

**Tech Stack:** tokio (UdpSocket/Notify/time), bytes, the existing transport composition (`transport::dial` + `security::wrap`), rstest e2e vs xray 26.3.27.

**Spec:** `docs/superpowers/specs/2026-08-18-vless-mkcp-design.md` (wire authority: xray 26.3.27 `transport/internet/kcp/{segment,connection,sending,receiving,output,dialer,config}.go`).

## Global Constraints

- xray-tui-proto NEVER modified. No new dependencies (KCP state machine in-crate). `NativeError`-only surface.
- The wire is xray's KCP fork: `[conv u16][cmd][opt][ts u32][sn u32][una u32][len u16][payload]` (18B data overhead), Ack `[conv][cmd=0][opt][rcv_wnd][rcv_nxt][ts][count u8][numbers…]`, CmdOnly Ping=3/Terminate=2 `[conv][cmd][opt][snd_nxt][rcv_nxt][peer_rto]` (14B). Big-endian. One segment per datagram.
- Defaults: mtu 1350, tti 50, uplink 5 Mbps, downlink 20 Mbps, cwnd_multiplier 1, max_sending_window 2 MiB → mss 1332, snd_inflight 194, rcv_inflight 777, snd_buf 1553 (all min-8 or division per spec §4.5).
- State machine + updaters + congestion per spec §4.3/§4.4/§5.2 (xray-identical semantics: RFC 6298-style RTO init 100 cap 10000 ×5/4, fast-resend rto/3, loss ≥15% cwnd×3/4 ≤5% +1/4 min 16, 30s idle-close, ping every 3000ms, ping-updater 5000ms, data-updater tti).
- Every completion re-reads the spec's wire/semantics section before coding; byte-exactness is verified by unit tests, not assumptions.
- e2e rows: xray-single-core only (kcp plain/tls/reality); count vless 72+4 → 75+4, vmess 48, total 123 → 126.
- Gates per spec §7.4; run BOTH `--features native-e2e` builds when the feature-gated e2e module is touched (the established lesson).

---

### Task 1: `transport/kcp/wire.rs` — segment codec

- [ ] Create `crates/xray-tui-native/src/transport/kcp/wire.rs` with `Segment` enum + `encode` + `parse_datagram` per spec §4.1.
- [ ] Unit tests: byte-exact encode for Data (18B hdr + payload, opt/Close), Ack (rcv_wnd/rcv_nxt/ts/count + numbers), CmdOnly Ping/Terminate (14B); parse roundtrips; truncation (4B minimum, short Data/Ack/CmdOnly bodies), bad cmd, conv-mismatch drop.
- [ ] Module wiring (`transport/kcp/mod.rs` + parent `transport/mod.rs` re-exports).

- Consumes: spec §4.1.
- Produces: the codec the session uses.

### Task 2: `transport/kcp/session.rs` — the KCP core (windows, RTO/RTT, timers, state machine)

- [ ] `RoundTripInfo` (RFC 6298-style per spec §5.2: init rto 100, min_rtt=tti, cap 10000, ×5/4, peer-rto adoption ≥3000ms).
- [ ] `SendingWorker`: window (ordered sn → {payload, timeout, transmit}), push (reject at snd_buf), flush (cwnd = min(snd_inflight, remote_rcv_nxt - first_unacked, control_window) × multiplier; retransmit on timeout; fast-resend via ack rto/3), congestion (loss ≥15% ×3/4, ≤5% +1/4, min 16, cap snd_inflight).
- [ ] `ReceivingWorker`: recv window (sn → payload, drop wrapped-out-of-window), reassembly (consecutive from next_number), ack list (per-number due times, batch flush with `(mss-17)/4` cap + flush-candidates, rcv_wnd/rcv_nxt).
- [ ] `KcpSession`: states + transitions (Active/ReadyToClose/PeerClosed/Terminating/PeerTerminating/Terminated), Close(), HandleOption, OnTerminate, idle guard 30s, ping every 3000ms, input dispatch, read (wait notifier; EOF per states), write backpressure (mss split, wait ack-notifier when window full).
- [ ] Timers: one session task with the data-updater (tti tick while send window nonempty || acklist nonempty) + ping-updater (5000ms tick while not Terminated, immediate first wake) semantics; flush per §4.3.
- [ ] Unit tests: RTO/RTT arithmetic; send flush (order, cwnd stop, timeout retransmit, fast-resend, loss cwnd); recv reassembly (in-order, gap, duplicate, wrap-around); ack-list batching; state machine (close→terminate→terminated, peer-close, terminate handshake, 30s idle); u32 wraparound arithmetic.

- Consumes: Task 1 (codec).
- Produces: the session core.

### Task 3: Hermetic fake-peer session test (real UDP, no cores)

- [ ] Fake mKCP peer over a loopback UDP socket (server-side mirror: parse datagrams, recv window + ack echo, optional loss/out-of-order injection, close/terminate handling).
- [ ] End-to-end session tests: bidirectional data flow, retransmit under injected loss, close handshake, ping cadence, idle-close.
- [ ] Gated as unit tests (no `native-e2e`), same hermetic pattern as the mux/vision fake servers.

- Consumes: Task 2.
- Produces: interop-level confidence without real cores.

### Task 4: `stream.rs` + transport integration

- [ ] `KcpStream`: AsyncRead/AsyncWrite over the session (notifier-driven await, no extra buffering).
- [ ] `transport/mod.rs` kcp dial arm: `dial_kcp(addr, params)` — `UdpSocket::bind("0.0.0.0:0")` + connect, spawn the input task (recv loop → session.input), return the stream; the transport params gain the kcp variant (`mtu`/`tti` optional, defaults per spec §4.5).
- [ ] Verify the composition: kcp → `security::wrap` (TLS/REALITY) → VLESS — the existing connect path consumes the stream unchanged.

- Consumes: Tasks 2-3.
- Produces: the full client path.

### Task 5: e2e rows (xray kcp: plain/tls/reality)

- [ ] Harness: transport=kcp server config emission (xray `"streamSettings":{"network":"kcp","kcpSettings":{...}}`) + row entries (kcp+plain, kcp+tls, kcp+reality).
- [ ] Run the sweeps: vless 75+4, vmess 48. Document the rows in the e2e module.

- Consumes: Task 4.
- Produces: the 3 new green rows.

### Task 6: Docs + full verification

- [ ] NATIVE_CORE.md + AGENTS.md: mKCP transport row (wire, composition, xray-only e2e), counts (unit incl. new tests, vless 75+4, vmess 48, total 126).
- [ ] Full gates per spec §7.4: clippy 0 (both feature sets), fmt, unit lib, `--features native-e2e` lib + vless + vmess sweeps.

- Consumes: Tasks 1-5.
- Produces: SP4 closed.
