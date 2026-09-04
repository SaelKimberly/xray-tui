//! Throughput benches: bulk send/recv per native config (spec
//! `docs/superpowers/specs/2026-09-04-native-throughput-bench-design.md`).
#![cfg(feature = "native-e2e")]

use std::time::Duration;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use tokio::io::AsyncWriteExt as _;
use xray_tui_native::e2e::{
    CaseSpec, Certs, CoreKind, CoreUnderTest, E2eCase, SINGBOX_VERSION, XRAY_VERSION,
    generate_certs,
    harness::{free_port, spawn_core, spawn_sink, spawn_source},
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
    // `spawn_sink`/`spawn_source` use `tokio::spawn` internally: they need
    // an entered runtime context at spawn time. The guard MUST drop before
    // any `rt.block_on` — `block_on` panics inside an entered context
    // ("cannot start a runtime from within a runtime"). Spawned tasks keep
    // running on the multi-thread workers after the guard drops.
    let certs: Certs = generate_certs();
    let sink = {
        let _enter = rt.enter();
        spawn_sink()
    };
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
    {
        let _enter = rt.enter();
        let _ = spawn_source(); // keep helper linked until recv rows land (Task 3)
    }
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
