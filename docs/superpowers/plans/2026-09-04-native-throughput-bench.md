# Native Throughput Benchmarks Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Criterion throughput benches (send + recv) for each curated native core config, runnable locally against harness-spawned xray/sing-box servers.

**Architecture:** New `benches/throughput.rs` in `xray-tui-native` reuses `e2e::{config, harness}` builders + `CaseSpec::client_params` for configs; adds tokio sink/source helpers to `e2e/harness.rs`; setup-once lifecycle (spawn core + connect once per row, `b.iter` only bulk transfer over shared runtime).

**Tech Stack:** criterion 0.8 (new dev-dep) + cargo-criterion runner, tokio AsyncRead/Write, existing e2e harness (`native-e2e` feature).

**Spec:** `docs/superpowers/specs/2026-09-04-native-throughput-bench-design.md`

## Global Constraints

- Run ONLY via `cargo criterion` (criterion.toml `criterion_home = "./.benchmarks"`); `cargo bench` is not used.
- `[[bench]] harness = false` + `required-features = ["native-e2e"]`; default `cargo bench/test` without the feature must compile.
- Hysteria2 rows are sing-box-only (xray has no hysteria2 inbound); all TCP rows in v1 target xray.
- Version pins: xray 26.3.27, sing-box 1.13.16 (`e2e::{XRAY_VERSION, SINGBOX_VERSION}`); missing `XRAY_TUI_CORE_BIN_DIR` → clean skip message.
- Workspace lints apply (`[lints] workspace = true` in crate manifest); rustfmt max_width 100.
- One shared tokio Runtime for all iters, `new_multi_thread` (sink/source
  accept loops are spawned at setup outside `block_on`; current-thread
  would never poll them); never spawn/connect inside `b.iter`.

---

### Task 1: Sink/source harness helpers

**Files:**
- Modify: `crates/xray-tui-native/src/e2e/harness.rs`
- Test: `crates/xray-tui-native/src/e2e/harness.rs` (same-file unit test)

- Consumes: existing `free_port()` (harness.rs:383), tokio net (already a dependency).
- Produces: `pub struct SinkServer { pub addr: SocketAddr, pub received: Arc<AtomicU64> }` + `pub fn spawn_sink() -> SinkServer`; `pub struct SourceServer { pub addr: SocketAddr }` + `pub fn spawn_source() -> SourceServer`. The sink's per-conn read loop adds every `n` to `received` (cumulative across conns/tunnels) — send iters snapshot it pre-iter and spin until `received >= target` INSIDE the timed section, so samples measure tunnel goodput, not local socket buffering. Both `#[must_use]`; module is already `#[cfg(feature = "native-e2e")]` gated.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod bench_helper_tests {
    use super::{spawn_sink, spawn_source};

    #[tokio::test]
    async fn sink_discards_and_source_streams() {
        use std::sync::atomic::Ordering;
        let sink = spawn_sink();
        let stream = tokio::net::TcpStream::connect(sink.addr).await.unwrap();
        let (mut rd, mut wr) = stream.into_split();
        // Source streams infinite zeros: read 1 MiB, all zeros.
        let source = spawn_source();
        let mut stream = tokio::net::TcpStream::connect(source.addr).await.unwrap();
        let mut buf = vec![0u8; 1 << 20];
        use tokio::io::AsyncReadExt as _;
        stream.read_exact(&mut buf).await.unwrap();
        assert!(buf.iter().all(|&b| b == 0));
        // Sink accepts a write AND counts it.
        use tokio::io::AsyncWriteExt as _;
        wr.write_all(&buf[..4096]).await.unwrap();
        drop(wr);
        let _ = &mut rd;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while sink.received.load(Ordering::SeqCst) < 4096 {
            assert!(std::time::Instant::now() < deadline, "sink never counted bytes");
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }
}
```
- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p xray-tui-native --features native-e2e --lib e2e::harness::bench_helper_tests`
Expected: FAIL with "cannot find function `spawn_sink`"

- [ ] **Step 3: Write minimal implementation**

```rust
/// A TCP discard target: every connection's bytes are read and dropped.
/// The bench client writes N MB through the tunnel toward this address.
/// `received` counts cumulative bytes across all connections — send iters
/// snapshot it pre-iter and wait for the delta INSIDE the timed section
/// (`write_all` alone only measures local socket buffering).
pub struct SinkServer {
    pub addr: SocketAddr,
    pub received: Arc<AtomicU64>,
    handle: tokio::task::JoinHandle<()>,
}

impl Drop for SinkServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// Spawn a `tokio::net::TcpListener` sink on 127.0.0.1:ephemeral.
///
/// Synchronous (mirrors `spawn_udp_echo`): binds a std socket, flips
/// nonblocking, converts to tokio.
#[must_use]
pub fn spawn_sink() -> SinkServer {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    let socket = std::net::TcpListener::bind("127.0.0.1:0").expect("bind sink");
    let addr = socket.local_addr().expect("sink addr");
    socket.set_nonblocking(true).expect("sink nonblocking");
    let listener = tokio::net::TcpListener::from_std(socket).expect("sink tokio");
    let received = Arc::new(AtomicU64::new(0));
    let received_loop = received.clone();
    let handle = tokio::spawn(async move {
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                break;
            };
            let received = received_loop.clone();
            tokio::spawn(async move {
                use tokio::io::AsyncReadExt as _;
                while let Ok(n) = sock.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    received.fetch_add(n as u64, Ordering::SeqCst);
                }
            });
        }
    });
    SinkServer { addr, received, handle }
}
```
/// A TCP infinite-zeros target: every connection streams zeros forever
/// (never EOF/half-close) so one bench tunnel stays reusable across
/// iters — the client reads exactly N per iter.
pub struct SourceServer {
    pub addr: SocketAddr,
    handle: tokio::task::JoinHandle<()>,
}

impl Drop for SourceServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// Spawn a `tokio::net::TcpListener` zeros source on 127.0.0.1:ephemeral.
#[must_use]
pub fn spawn_source() -> SourceServer {
    let socket = std::net::TcpListener::bind("127.0.0.1:0").expect("bind source");
    let addr = socket.local_addr().expect("source addr");
    socket.set_nonblocking(true).expect("source nonblocking");
    let listener = tokio::net::TcpListener::from_std(socket).expect("source tokio");
    let handle = tokio::spawn(async move {
        let zeros = vec![0u8; 64 * 1024];
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                break;
            };
            let zeros = zeros.clone();
            tokio::spawn(async move {
                use tokio::io::AsyncWriteExt as _;
                while sock.write_all(&zeros).await.is_ok() {}
            });
        }
    });
    SourceServer { addr, handle }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p xray-tui-native --features native-e2e --lib e2e::harness::bench_helper_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/e2e/harness.rs
git commit -m "feat(native): sink/source bench helpers in e2e harness"
```

### Task 2: Criterion wiring (manifest + bench skeleton)

**Files:**
- Modify: `crates/xray-tui-native/Cargo.toml`
- Create: `crates/xray-tui-native/benches/throughput.rs`
- Test: compile-gate check (no functional test; verification is the build commands)

**Interfaces:**
- Consumes: Task 1 helpers; `e2e::{CaseSpec, CoreKind, CoreUnderTest, E2eCase, XRAY_VERSION, SINGBOX_VERSION}`.
- Produces: `benches/throughput.rs` with `criterion_main!` + one smoke bench (`vless/tcp/plain` send) proving the full lifecycle; remaining rows arrive in Task 3.

- [ ] **Step 1: Write the bench skeleton (fails: no criterion dep, no [[bench]])**

```rust
//! Throughput benches: bulk send/recv per native config (spec
//! `docs/superpowers/specs/2026-09-04-native-throughput-bench-design.md`).
#![cfg(feature = "native-e2e")]

use std::time::Duration;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use xray_tui_native::e2e::{
    CaseSpec, Certs, CoreKind, CoreUnderTest, E2eCase, SINGBOX_VERSION, XRAY_VERSION,
    config, generate_certs, harness::{free_port, spawn_core, spawn_sink, spawn_source},
    ServerEnv,
};

fn bench_mb() -> u64 {
    std::env::var("XRAY_TUI_BENCH_MB")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|&v| v > 0)
        .unwrap_or(64)
}

fn resolve_or_skip(kind: CoreKind) -> Option<CoreUnderTest> {
    let want = match kind {
        CoreKind::Xray => XRAY_VERSION,
        CoreKind::SingBox => SINGBOX_VERSION,
    };
    match CoreUnderTest::resolve(kind, want) {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!("SKIP throughput benches: {e}");
            None
        }
    }
}

fn bench_send(c: &mut Criterion, rt: &tokio::runtime::Runtime, n: u64) {
    let Some(core) = resolve_or_skip(CoreKind::Xray) else {
        return;
    };
    let certs: Certs = generate_certs();
    let sink = spawn_sink();
    let dir = tempfile::tempdir().expect("bench tempdir");
    std::fs::write(dir.path().join("server.crt"), &certs.cert_pem).unwrap();
    std::fs::write(dir.path().join("server.key"), &certs.key_pem).unwrap();
    let port = free_port();
    let env = ServerEnv {
        port,
        certs: &certs,
        tmp: dir.path(),
        echo: sink.addr,
        tls_echo: sink.addr,
        inner_tls_echo: None,
        udp_echo: None,
    };
    let case = CaseSpec::vless();
    let config_json = case.server_config(core.kind, &env);
    let config_path = dir.path().join("config.json");
    std::fs::write(&config_path, &config_json).unwrap();
    let _core = spawn_core(&core.bin, core.kind, &config_path, port, false);
    case.client_trust(&certs);
    let params = case.client_params(port, sink.addr);
    let mut tunnel = rt.block_on(xray_tui_native::connect(params)).expect("bench connect");
    let mut group = c.benchmark_group("throughput");
    group.throughput(Throughput::Bytes(n));
    group.measurement_time(Duration::from_secs(20));
    let payload = vec![0xABu8; 1024 * 1024];
    let iters = n / payload.len() as u64;
    group.bench_function("vless/tcp/plain/send", |b| {
        b.iter(|| {
            rt.block_on(async {
                use std::sync::atomic::Ordering;
                let base = sink.received.load(Ordering::SeqCst);
                let target = base + n;
                for _ in 0..iters {
                    tunnel.write_all(&payload).await.unwrap();
                }
                tunnel.flush().await.unwrap();
                // Goodput gate: `write_all` returns on local buffering —
                // stay in the timed section until the sink counted N.
                while sink.received.load(Ordering::SeqCst) < target {
                    tokio::task::yield_now().await;
                }
            });
        });
    });
    let _ = spawn_source(); // keep helper linked until recv rows land (Task 3)
}

fn criterion_benches(c: &mut Criterion) {
    // Multi-thread: sink/source accept loops are `tokio::spawn`ed at
    // setup (outside `block_on`) — a current-thread runtime would never
    // poll them and the send goodput-gate would spin forever.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("bench runtime");
    let n = bench_mb() * 1024 * 1024;
    bench_send(c, &rt, n);
}

criterion_group!(benches, criterion_benches);
criterion_main!(benches);
```

- [ ] **Step 2: Wire the manifest, verify it compiles**

Add to `crates/xray-tui-native/Cargo.toml`:

```toml
[[bench]]
name = "throughput"
harness = false
required-features = ["native-e2e"]

[dev-dependencies]
criterion = { version = "0.8", features = ["async_tokio"] }
tempfile = "3"
```

Check first whether `generate_certs` is exported from `e2e` (it is used by `tests/common` via `xray_tui_native::e2e::{...}` — confirm name; if it is `harness::generate_certs`, adjust the import). Also confirm `ServerEnv` is exported at `e2e::ServerEnv` (mod.rs:39 `pub struct ServerEnv<'a>` — yes).

Run: `cargo bench -p xray-tui-native --features native-e2e --bench throughput -- --test` (compile check; without binaries it prints SKIP)
Expected: compiles; with `XRAY_TUI_CORE_BIN_DIR` unset prints `SKIP throughput benches: ...` and reports no benches run. Also run WITHOUT the feature: `cargo bench -p xray-tui-native --bench throughput -- --test` → hmm, `required-features` means cargo skips building the bench target without the feature; verify `cargo check -p xray-tui-native` (default features) still passes.

- [ ] **Step 3: Run the smoke bench locally with binaries**

Run: `XRAY_TUI_CORE_BIN_DIR=<dir> cargo criterion -p xray-tui-native --features native-e2e --bench throughput -- vless/tcp/plain/send`
Expected: bench completes, `.benchmarks/` gains `throughput/vless_tcp_plain_send/` data.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-native/Cargo.toml crates/xray-tui-native/benches/throughput.rs
git commit -m "feat(native): throughput bench skeleton + criterion wiring"
```

### Task 3: Full matrix (recv + all rows)

**Files:**
- Modify: `crates/xray-tui-native/benches/throughput.rs`
- Test: full local run (all benches green, baselines written)

**Interfaces:**
- Consumes: Task 2 skeleton (`bench_send`, `resolve_or_skip`, `bench_mb`, shared runtime).
- Produces: ~20 benches: send + recv for vless/tcp/plain, vless/tcp/tls-chrome, vless/tcp/reality, vless/vision+reality, vless/ws/plain, vless/grpc/tls-chrome, vmess/tcp/aes, vmess/ws/chacha, trojan/tcp/tls-chrome, hysteria2-TCP (sing-box, `CaseSpec::hysteria2(None)` default TCP path via plain `connect` — the QUIC-stream tunnel, same as e2e). Spec matrix `hysteria2/udp` (datagram `connect_udp`) is OUT of v1 scope: no byte-stream, no sink/source target — needs its own datagram-loop design (N × M-byte datagrams), deferred. Recv rows mirror with `spawn_source` + `read_exact` N per iter. Variant constructors live behind `e2e::variant` re-exports — verify paths (`FingerprintTls`, `RealityTls::fresh()`, `Aes128GcmVariant`, `Chacha20Poly1305Variant`) before writing rows.

Refactor: extract `setup_row(case, core_kind, target_kind) -> (CoreGuard, NativeTunnel, SocketAddr-ish)` returning the guard (must be held!), tunnel, and byte count; then:

```rust
struct Row {
    name: &'static str,
    case: CaseSpec,
    core: CoreKind,
}
const ROWS: &[Row] = &[...]; // can't be const (CaseSpec not const-constructible with Box) → build in fn rows() -> Vec<Row>
```

Recv iter body:

```rust
group.bench_function("vless/tcp/plain/recv", |b| {
    let mut buf = vec![0u8; 1024 * 1024];
    b.iter(|| {
        rt.block_on(async {
            let mut left = n;
            while left > 0 {
                let take = buf.len().min(left as usize);
                tunnel.read_exact(&mut buf[..take]).await.unwrap();
                left -= take as u64;
            }
        });
    });
});
```

Note REALITY rows: `case.client_trust(&certs)` is a no-op for REALITY (variant handles it); cert files still needed for the server config's dest? No — REALITY server needs `tls_echo`-like dest only for fallback rows; plain REALITY rows point dest at sink/source addrs directly. Check `config::vless_inbound` dest wiring for reality (uses `env.tls_echo` as handshake dest?) — if the REALITY server config hardcodes `env.tls_echo` as its dest, then `ServerEnv.tls_echo` must be a REAL TLS server, and sink/source addrs can't serve it. Read `config.rs` reality arm before finalizing: if dest-bound, spawn `spawn_tls_echo(&certs)` for `tls_echo` in every row's env and keep sink/source as `echo`. This matches `run_against` (both echo servers coexist).

Hysteria2 rows: `CaseSpec::hysteria2(None)` default TCP path — bulk TCP over the QUIC-stream tunnel via plain `connect` (same as the e2e default row). The datagram `connect_udp` shape is out of v1 scope (see Produces).
Run: `XRAY_TUI_CORE_BIN_DIR=<dir> cargo criterion -p xray-tui-native --features native-e2e --bench throughput`
Expected: all ~20 benches complete; `.benchmarks/throughput/` holds one dir per bench = the v1 baseline. Record the machine + commit hash in the commit message (baselines are machine-specific).

- [ ] **Step 3: Quality gate (code subset)**

Run: `cargo fmt -- crates/xray-tui-native/benches/throughput.rs crates/xray-tui-native/src/e2e/harness.rs` (i.e. `cargo fmt` then check diff is only our files), `cargo clippy -p xray-tui-native --features native-e2e --benches`, `cargo hakari generate` only if Cargo.lock changed (criterion is new → lock changes → regenerate + `cargo hakari verify`).
Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-native/benches/throughput.rs
git commit -m "feat(native): full throughput matrix, v1 baselines"
```

## Self-Review

- Spec coverage: matrix § → Task 3 rows; sink/source § → Task 1 (+ infinite-zeros fix); lifecycle § → Task 2 skeleton (spawn-once, shared rt, Throughput::Bytes); wiring § → Task 2 manifest (`harness=false`, `required-features`, `cargo criterion` only); baselines § → Task 3 Step 2. Hysteria2 sing-box-only honored (Task 3). No thresholds (spec: human judges).
- Placeholders: none — every step has exact code/commands; row-variant constructor paths flagged for verification against actual exports (FingerprintTls/RealityTls/Aes128GcmVariant re-export paths, reality dest wiring in config.rs) with fallback instructions.
- Type consistency: `SinkServer/SourceServer { addr }` (Task 1) match Task 2/3 usage; `ServerEnv` fields copied from mod.rs:39-56 verbatim; `NativeTunnel` AsyncRead/Write via existing impls.
