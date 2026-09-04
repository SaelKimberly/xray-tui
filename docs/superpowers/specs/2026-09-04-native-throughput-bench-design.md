# Native Throughput Benchmarks — Design

Date: 2026-09-04. Status: approved. Option A (e2e reuse).

## Goal

Throughput benches for each native core config (curated rows), send and
receive paths separately, runnable locally via criterion. First full run
is the baseline for future perf-delta comparison.

## Matrix (one criterion bench per row × {send, recv})

Fixed server core per row — no core axis in v1:

- vless/tcp/plain, vless/tcp/tls-chrome, vless/tcp/reality,
  vless/vision+reality → xray
- vless/ws/plain, vless/grpc/tls → xray
- vmess/tcp/plain, vmess/ws/tls → xray
- trojan/tcp/tls → xray
- hysteria2/tcp → sing-box only (default `CaseSpec::hysteria2(None)`
  TCP path via plain `connect`, same as e2e; the datagram `connect_udp`
  shape is out of v1 scope — no byte-stream, needs its own
  datagram-loop design)

10 configs × 2 directions = 20 benches. `Throughput::Bytes(N)`.

## Method: bulk sink/source

- Send: client `write_all` N MB through `NativeTunnel` to a sink target
  (TCP read-discard, counts bytes).
- Recv: client drains N MB from a source target (writes N MB zeros
  per connection).
- Isolates each direction; no echo round-trip mixing, no HTTP framing
  noise. N sized so one iter ≈ 0.5–2 s. Default 64 MB, override via
  env `XRAY_TUI_BENCH_MB` (parsed as u64 megabytes, fallback 64 on
  missing/invalid).

## Lifecycle: setup-once

- Spawn core ONCE per row (reuse `spawn_core`/`CoreGuard`), establish
  ONE tunnel, then `b.iter()` only the bulk transfer.
- Never spawn/connect inside `iter` (~100 ms spawn + handshake would
  dominate samples).
- One shared tokio `Runtime`, `block_on` per iter; never construct a
  runtime per iter.
- Hysteria2 rows use the QUIC datagram/stream path per existing
  `connect` shape; TCP rows use `NativeTunnel` AsyncRead/Write.

## Harness additions (crate `xray-tui-native`, `native-e2e` gated)

- `spawn_sink() -> SinkServer { addr, received: Arc<AtomicU64> }`:
  TCP listener on 127.0.0.1:ephemeral, per-conn read-discard loop.
- `spawn_source() -> SourceServer { addr }`: per-conn INFINITE zeros
  stream (never EOF/half-close). Client reads exactly N per iter —
  tunnel stays reusable across iters.
- Reuse `e2e::{config, harness}` builders for server configs +
  client params (no second config source). Targets point at sink /
  source addrs through the tunnel.
- Existing `spawn_echo` NOT reused (fixed small BODY, wrong shape).

## Criterion wiring

- `crates/xray-tui-native/benches/throughput.rs`, `[[bench]]
  harness = false`, `required-features = ["native-e2e"]`.
- criterion dev-dependency on `xray-tui-native`.
- Run ONLY via `cargo criterion` (repo rule; `criterion.toml`
  `criterion_home = "./.benchmarks"`, survives `cargo clean`).
  `cargo bench` is not used.
- Gate: needs `XRAY_TUI_CORE_BIN_DIR` with version-pinned binaries
  (same as tier-3 e2e: xray 26.3.27, sing-box 1.13.16); absent →
  clean skip message, same convention as e2e harness.
- Default `cargo bench/test` (no `native-e2e`) must still compile:
  `required-features` + `#[cfg(feature)]` guards on shared helpers.

## Baselines

- First full local run = baseline, stored under `.benchmarks/`.
- Later runs compare automatically (criterion delta reports).
- No in-code perf thresholds; human judges regressions.

## Non-goals

- No ×both-cores axis, no micro codec benches, no CI wiring (local
  only; tier-3 binaries not in CI).
- No ad-hoc runner binary; criterion stats are the output.

## Files touched

- NEW `crates/xray-tui-native/benches/throughput.rs`
- `crates/xray-tui-native/Cargo.toml` (`[[bench]]`, criterion dev-dep)
- `crates/xray-tui-native/src/e2e/harness.rs` (+sink/source helpers)
