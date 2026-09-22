//! Cost lab for the Fast + Real Ping flow (perf evidence, not a gate).
//!
//! Ignored by default. Run with:
//!
//! ```text
//! cargo test -p xray-tui --release --lib -- --ignored --nocapture flow_cost
//! ```
//!
//! Every row is the MEDIAN of its per-operation samples on this machine (a
//! mean is unusable: the first sample of a section pays page-cache, statement
//! compilation and pool setup that steady state never pays).
//!
//! Allocation COUNTS are not available in-process: `turso` installs a
//! `#[global_allocator]` (its default `mimalloc` feature) and a second one is a
//! compile error, while `valgrind --tool=dhat` on the test binary dies with
//! SIGILL at startup under that same allocator. Allocation figures in the lab
//! report are therefore derived from the code path (one `String::clone` is one
//! allocation) and backed by A/B timing of the allocation-free variant.
//!
//! `XRAY_TUI_MEASURE_DB=/path/to/data.db` additionally measures the per-tick
//! profiles reload against a copy of a real feed (never the live file).
#![allow(clippy::all, clippy::pedantic, clippy::nursery, dead_code)]

use std::hint::black_box;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use xray_tui_config::IpProvider;
use xray_tui_db::models::{
    Endpoint, EndpointRow, Latency, ProfileErr, ProfileStats, PurgatoryView,
};
use xray_tui_db::profiles_query::{PageRequest, PageSort, PlanScope};
use xray_tui_proto::proto_spec::ProtocolConfig;

use super::*;
use crate::ops::profiles::test_support::{fake_row, test_state};

// ── timing ─────────────────────────────────────────────────────────────────

/// Accumulates one sample per operation.
///
/// The report uses the MEDIAN, not the mean: the first call of an async section
/// pays one-off costs a steady-state batch never pays (page-cache fill on a
/// freshly opened database copy, statement compilation, pool creation), and
/// with a handful of samples a single cold one drags the mean by a factor — the
/// same page query read 12.4 ms cold against ~0.5 ms warm on consecutive runs.
/// A median is also what makes two runs comparable.
struct Acc {
    samples: Vec<u64>,
}

impl Acc {
    const fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    fn add(&mut self, elapsed: Duration) {
        self.samples
            .push(u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX));
    }

    /// Per-operation row, with `divisor` folding a batched op down to one item.
    fn row(&self, name: &str, divisor: f64) -> TableRow {
        let mut samples = self.samples.clone();
        samples.sort_unstable();
        let ns = match samples.len() {
            0 => 0.0,
            n if n % 2 == 1 => samples[n / 2] as f64,
            n => (samples[n / 2 - 1] as f64 + samples[n / 2] as f64) / 2.0,
        };
        TableRow {
            name: name.to_owned(),
            ns: ns / divisor,
            n: samples.len() as u32,
        }
    }
}

struct TableRow {
    name: String,
    ns: f64,
    n: u32,
}

fn print_table(title: &str, rows: &[TableRow]) {
    println!("\n== {title} ==");
    println!("{:<56} {:>14} {:>4}", "operation", "ns/op", "n");
    for row in rows {
        println!("{:<56} {:>14.1} {:>4}", row.name, row.ns, row.n);
    }
}

fn bench_sync(iters: u32, mut op: impl FnMut()) -> Acc {
    let mut acc = Acc::new();
    for _ in 0..3 {
        op();
    }
    for _ in 0..iters {
        let started = Instant::now();
        op();
        acc.add(started.elapsed());
    }
    acc
}

// ── synthetic feed ─────────────────────────────────────────────────────────

fn synth_rows(endpoints: usize, protos: usize) -> Vec<EndpointRow> {
    (1..=endpoints)
        .map(|i| {
            let host = format!("10.{}.{}.{}", i / 65536, (i / 256) % 256, i % 256);
            fake_row(i as i64, &host, protos)
        })
        .collect()
}

async fn seed_db(state: &crate::AppState, rows: &[EndpointRow]) {
    let mut conn = state.db.connection().await.expect("conn");
    let mut tx = conn.transaction().await.expect("tx");
    let endpoints: Vec<Endpoint> = rows.iter().map(|r| r.endpoint.clone()).collect();
    let protocols: Vec<xray_tui_db::models::Protocol> = rows
        .iter()
        .flat_map(|r| r.protocols.values().cloned())
        .collect();
    let links: Vec<ProfileStats> = rows.iter().flat_map(|r| r.links.iter().cloned()).collect();
    xray_tui_db::upsert_endpoints_bulk(&mut tx, &endpoints)
        .await
        .expect("endpoints");
    xray_tui_db::upsert_protocols_bulk(&mut tx, &protocols)
        .await
        .expect("protocols");
    xray_tui_db::upsert_links_bulk(&mut tx, &links)
        .await
        .expect("links");
    tx.commit().await.expect("commit");
}

// ── stub probes ────────────────────────────────────────────────────────────

struct NullRunner;

impl BatchProbeRunner for NullRunner {
    fn fast<'a>(
        &'a self,
        _config_type: i32,
        _addr: &'a str,
        _port: u16,
        _timeout: Duration,
    ) -> Pin<Box<dyn Future<Output = ProbeOutcome> + Send + 'a>> {
        Box::pin(async move {
            ProbeOutcome::Ok {
                latency_ms: Some(10),
                ip_info: None,
            }
        })
    }

    fn real<'a>(
        &'a self,
        _endpoint: &'a Endpoint,
        _config: &'a ProtocolConfig,
        _req: NativeProbeReq<'a>,
    ) -> Pin<Box<dyn Future<Output = ProbeOutcome> + Send + 'a>> {
        Box::pin(async move {
            ProbeOutcome::Ok {
                latency_ms: Some(50),
                ip_info: Some("1.2.3.4".to_owned()),
            }
        })
    }
}

fn batch_params(
    state: &crate::AppState,
    tx: mpsc::Sender<CoreEvent>,
    plan: Vec<PlanLink>,
    real_phase: bool,
    page_size: usize,
    runner: Arc<dyn BatchProbeRunner>,
) -> BatchParams {
    BatchParams {
        scheduler: state.scheduler.clone(),
        db: state.db.clone(),
        writer: state.link_writer.clone(),
        tx,
        runner,
        stop: state.speed_test_stop.clone(),
        meters: Arc::new(crate::types::BatchMeters::default()),
        plan: PlanSource::Links(plan),
        real_phase,
        dedup_endpoints: false,
        fast_timeout: Duration::from_secs(2),
        real_timeout: Duration::from_secs(2),
        real_retries: 1,
        ping_url: "http://127.0.0.1/".to_owned(),
        ip_provider: IpProvider::IpApi,
        defer_delay: Duration::from_millis(50),
        real_concurrency: 64,
        fast_concurrency: 64,
        page_size,
        error_ttl_hours: None,
        dns_cache_ttl_secs: 86400,
        batch_slot: Arc::new(OnceLock::new()),
    }
}

// ── the report ─────────────────────────────────────────────────────────────

#[ignore = "perf lab: run explicitly with --ignored"]
#[test]
fn flow_cost_sizes() {
    use std::mem::size_of;
    println!("\n== sizes (bytes) ==");
    for (name, size) in [
        ("ProfileStats", size_of::<ProfileStats>()),
        ("Endpoint", size_of::<Endpoint>()),
        (
            "Protocol (db row)",
            size_of::<xray_tui_db::models::Protocol>(),
        ),
        ("EndpointRow", size_of::<EndpointRow>()),
        ("PlanLink", size_of::<PlanLink>()),
        ("CoreEvent", size_of::<crate::types::CoreEvent>()),
        ("Latency", size_of::<Latency>()),
        ("ErrorInfo", size_of::<xray_tui_db::models::ErrorInfo>()),
        ("ProbeOutcome", size_of::<ProbeOutcome>()),
        ("ProtocolConfig", size_of::<ProtocolConfig>()),
        ("LinkPatch", size_of::<xray_tui_db::LinkPatch>()),
    ] {
        println!("{name:<56} {size:>6}");
    }
}

#[ignore = "perf lab: run explicitly with --ignored"]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn flow_cost_report() {
    let mut rows_out: Vec<TableRow> = Vec::new();

    // ── 1. plan construction ───────────────────────────────────────────
    let page = synth_rows(200, 4);
    let links_all: Vec<ProfileStats> = page.iter().flat_map(|r| r.links.iter().cloned()).collect();
    let plan_acc = bench_sync(200, || {
        let plan: Vec<PlanLink> = page.iter().flat_map(plan_row_links).collect();
        black_box(plan.len());
    });
    rows_out.push(plan_acc.row("plan_row_links (per link)", 800.0));

    // ── 2. write-behind stage / patch build / window write ──────────────
    let state = test_state(page.clone()).await;
    seed_db(&state, &page).await;
    let make_staged = |i: usize| {
        let mut link = links_all[i % links_all.len()].clone();
        link.latency = Some(Latency::Fast { delay: 12 });
        link.error = Some(xray_tui_db::models::ErrorInfo {
            kind: ProfileErr::Fast,
            text: "Connection refused by the remote proxy host".to_owned(),
        });
        link
    };
    // Pre-built, so the timed region contains the stage itself and not the
    // snapshot the caller already holds.
    let prebuilt: Vec<ProfileStats> = (0..512).map(make_staged).collect();

    let writer = state.link_writer.clone();
    let stage_acc = bench_sync(200_000, || {
        let link = &prebuilt[black_box(0)];
        writer.stage(link, xray_tui_db::LinkGroups::RESULT);
    });
    rows_out.push(stage_acc.row("LinkWriter::stage (RESULT)", 1.0));
    writer.flush().await.expect("flush");

    let stage3_acc = bench_sync(100_000, || {
        let link = &prebuilt[black_box(1)];
        writer.stage(link, xray_tui_db::LinkGroups::ALL);
    });
    rows_out.push(stage3_acc.row("LinkWriter::stage (ALL: 3 group inserts)", 1.0));
    writer.flush().await.expect("flush");

    let mut patch_acc = Acc::new();
    for _ in 0..64 {
        let links: Vec<ProfileStats> = (0..512).map(make_staged).collect();
        let started = Instant::now();
        let patches: Vec<xray_tui_db::LinkPatch> = links
            .iter()
            .map(|link| xray_tui_db::LinkPatch {
                link: link.clone(),
                groups: xray_tui_db::LinkGroups::RESULT,
            })
            .collect();
        patch_acc.add(started.elapsed());
        black_box(patches.len());
    }
    rows_out.push(patch_acc.row("LinkPatch build (per patch)", 512.0));

    let mut flush_acc = Acc::new();
    for _ in 0..32 {
        for link in (0..512).map(make_staged) {
            writer.stage(&link, xray_tui_db::LinkGroups::RESULT);
        }
        let started = Instant::now();
        writer.flush().await.expect("flush");
        flush_acc.add(started.elapsed());
    }
    rows_out.push(flush_acc.row("flush window (512 links, drain+write)", 512.0));

    let patches: Vec<xray_tui_db::LinkPatch> = (0..512)
        .map(|i| xray_tui_db::LinkPatch {
            link: make_staged(i),
            groups: xray_tui_db::LinkGroups::RESULT,
        })
        .collect();
    let mut apply_acc = Acc::new();
    for _ in 0..32 {
        let started = Instant::now();
        state.db.apply_link_patches(&patches).await.expect("apply");
        apply_acc.add(started.elapsed());
    }
    rows_out.push(apply_acc.row("apply_link_patches (512 links, no drain)", 512.0));

    // ── 3. per-result staging through the batch's own path ─────────────
    let (tx, rx) = mpsc::channel::<CoreEvent>(1 << 16);
    let params = batch_params(&state, tx, Vec::new(), true, 200, Arc::new(NullRunner));
    let shared = Arc::new(BatchShared::new(params));
    let outcome_ok = ProbeOutcome::Ok {
        latency_ms: Some(37),
        ip_info: Some("1.2.3.4".to_owned()),
    };
    let outcome_err = ProbeOutcome::Failed {
        text: "connection timed out".to_owned(),
        class: ProbeClass::Timeout,
        hard: true,
        evidence: None,
    };
    let stage_result_acc = bench_sync(4096, || {
        let link = &links_all[black_box(0)];
        shared.stage_result(link, TestType::TcpPing, &outcome_ok);
        shared.stage_result(link, TestType::RealPing, &outcome_err);
    });
    rows_out.push(stage_result_acc.row("BatchShared::stage_result", 2.0));
    drop(rx);

    // ── 4. fast-probe dedup key ────────────────────────────────────────
    let host = "edge.example-subscription-domain.net".to_owned();
    let port = 443_u16;
    let mut string_map: HashMap<(String, u16), u8> = HashMap::new();
    string_map.insert((host.clone(), port), 1);
    let key_acc = bench_sync(200_000, || {
        let key = (host.clone(), port);
        black_box(string_map.get(&key));
    });
    rows_out.push(key_acc.row("dedup get, String key (clone + hash)", 1.0));

    let host_arc: Arc<str> = Arc::from(host.as_str());
    let mut arc_map: HashMap<(Arc<str>, u16), u8> = HashMap::new();
    arc_map.insert((host_arc.clone(), port), 1);
    let arc_acc = bench_sync(200_000, || {
        black_box(arc_map.get(&(host_arc.clone(), port)));
    });
    rows_out.push(arc_acc.row("dedup get, Arc<str> key (refcount bump)", 1.0));

    let mut hash_map: HashMap<(u64, u16), u8> = HashMap::new();
    hash_map.insert((fnv(host.as_bytes()), port), 1);
    let hash_acc = bench_sync(200_000, || {
        black_box(hash_map.get(&(fnv(host.as_bytes()), port)));
    });
    rows_out.push(hash_acc.row("dedup get, prehashed (u64, u16) key", 1.0));

    let outcome_map: HashMap<(String, u16), ProbeOutcome> = [((
        host.clone(),
        port,
        ProbeOutcome::Failed {
            text: "connection timed out after 5000 ms".to_owned(),
            class: ProbeClass::Timeout,
            hard: true,
            evidence: None,
        },
    ))]
    .into_iter()
    .fold(HashMap::new(), |mut m, (h, p, o)| {
        m.insert((h, p), o);
        m
    });
    let key = (host.clone(), port);
    let outcome_acc = bench_sync(200_000, || {
        black_box(outcome_map.get(&key).cloned());
    });
    rows_out.push(outcome_acc.row("dedup hit: ProbeOutcome::clone (failure)", 1.0));

    // ── 5. real-probe preparation ──────────────────────────────────────
    let protocol_id = links_all[0].protocol_id;
    let mut load_acc = Acc::new();
    for _ in 0..200 {
        let started = Instant::now();
        let loaded = load_protocol_with_config(&state.db, protocol_id)
            .await
            .expect("load")
            .expect("row");
        load_acc.add(started.elapsed());
        black_box(loaded);
    }
    rows_out.push(load_acc.row("load_protocol_with_config (per real probe)", 1.0));

    let loaded = load_protocol_with_config(&state.db, protocol_id)
        .await
        .expect("load")
        .expect("row");
    let clone_acc = bench_sync(20_000, || {
        let cfg: ProtocolConfig = loaded.config.get().0.clone();
        black_box(cfg);
    });
    rows_out.push(clone_acc.row("ProtocolConfig::clone", 1.0));

    // ── 6. scheduler transitions ───────────────────────────────────────
    let scheduler = crate::ops::scheduler::TaskScheduler::new(0, 0);
    let link = &links_all[0];
    let mut sched_acc = Acc::new();
    for _ in 0..20_000 {
        let started = Instant::now();
        let id = match scheduler.schedule(link, TaskKind::FastPing).await {
            ScheduleOutcome::Started(id) => id,
            other => panic!("unexpected {other:?}"),
        };
        scheduler.complete(link, id, TaskKind::FastPing).await;
        sched_acc.add(started.elapsed());
    }
    rows_out.push(sched_acc.row("scheduler schedule+complete (round trip)", 1.0));

    // ── 7. UI per-result work: handler + live re-sort ──────────────────
    rows_out.extend(hit_miss_rows(true).await);
    rows_out.extend(hit_miss_rows(false).await);

    let mut sort_acc = Acc::new();
    let mut sort_row = page[0].clone();
    for _ in 0..50_000 {
        let started = Instant::now();
        sort_row.sort_links_by_test_priority(false);
        sort_row.select_best_measured_link();
        sort_acc.add(started.elapsed());
        black_box(sort_row.selected_protocol);
    }
    rows_out.push(sort_acc.row("sort_links_by_test_priority + select_best", 1.0));

    // ── 8. batch dispatch bookkeeping ──────────────────────────────────
    // The plan holds `Arc<Endpoint>`s, one per row, so the measurement uses a
    // real plan page rather than a row.
    let page_links: Vec<PlanLink> = page.iter().flat_map(plan_row_links).collect();
    let dispatch_acc = bench_sync(50_000, || {
        let plan = &page_links[black_box(0)];
        let key = (plan.link.protocol_id, plan.link.endpoint_id);
        let endpoint_map: DashMap<EndpointId, Arc<Endpoint>> = DashMap::new();
        endpoint_map
            .entry(plan.endpoint.id)
            .or_insert_with(|| Arc::clone(&plan.endpoint));
        let config_map: DashMap<(ProtocolId, EndpointId), i32> = DashMap::new();
        config_map.insert(key, 0);
        black_box((endpoint_map.len(), config_map.len()));
    });
    rows_out.push(dispatch_acc.row("dispatch_page maps per link (2 DashMaps+Arc)", 1.0));

    let probe_dash: DashMap<(ProtocolId, EndpointId), ProfileStats> = DashMap::new();
    // Give it a non-empty population so the shard walk is realistic.
    for link in links_all.iter().take(512) {
        probe_dash.insert((link.protocol_id, link.endpoint_id), link.clone());
    }
    let dash_len_acc = bench_sync(200_000, || {
        black_box(probe_dash.len());
    });
    rows_out.push(dash_len_acc.row("DashMap::len() (512-entry map)", 1.0));

    let insert_dash: DashMap<(ProtocolId, EndpointId), ProfileStats> = DashMap::new();
    insert_dash.insert(
        (links_all[0].protocol_id, links_all[0].endpoint_id),
        links_all[0].clone(),
    );
    let dash_insert_acc = bench_sync(200_000, || {
        insert_dash.insert(
            (links_all[0].protocol_id, links_all[0].endpoint_id),
            links_all[0].clone(),
        );
    });
    rows_out.push(dash_insert_acc.row("DashMap::insert (replace, 1-entry map)", 1.0));

    let rank_ids: Vec<EndpointId> = page.iter().map(|r| r.endpoint.id).collect();
    let mut rank_acc = Acc::new();
    for _ in 0..20 {
        let started = Instant::now();
        state
            .db
            .refresh_endpoint_ranks(&rank_ids)
            .await
            .expect("ranks");
        rank_acc.add(started.elapsed());
    }
    rows_out.push(rank_acc.row("refresh_endpoint_ranks (200 endpoints)", 200.0));

    let probe_hash: HashMap<(ProtocolId, EndpointId), ProfileStats> = probe_dash
        .iter()
        .map(|e| (*e.key(), e.value().clone()))
        .collect();
    let hash_len_acc = bench_sync(200_000, || {
        black_box(probe_hash.len());
    });
    rows_out.push(hash_len_acc.row("HashMap::len() (512-entry map)", 1.0));

    let page_clone = page.clone();
    let drop_acc = bench_sync(2000, || {
        let rows = page_clone.clone();
        let started_drop = std::time::Instant::now();
        drop(rows);
        black_box(started_drop.elapsed());
    });
    rows_out.push(drop_acc.row("clone+drop a 200-row page (per row)", 200.0));

    let endpoint_clone_acc = bench_sync(200_000, || {
        black_box(page[0].endpoint.clone());
    });
    rows_out.push(endpoint_clone_acc.row("Endpoint::clone (host String alloc)", 1.0));

    let link_clone_acc = bench_sync(200_000, || {
        black_box(links_all[0].clone());
    });
    rows_out.push(link_clone_acc.row("ProfileStats::clone (no error set)", 1.0));

    let mut err_link = links_all[0].clone();
    err_link.error = Some(xray_tui_db::models::ErrorInfo {
        kind: ProfileErr::Real,
        text: "connection timed out after 5000 ms".to_owned(),
    });
    let err_clone_acc = bench_sync(200_000, || {
        black_box(err_link.clone());
    });
    rows_out.push(err_clone_acc.row("ProfileStats::clone (error text set)", 1.0));

    // fast dedup owner path: key clone + lock + cache insert + in-flight remove
    let mut dedup_cache: HashMap<(String, u16), ProbeOutcome> = HashMap::new();
    let mut in_flight: HashMap<(String, u16), Arc<tokio::sync::Notify>> = HashMap::new();
    let dedup_key = (host.clone(), port);
    let dedup_acc = bench_sync(20_000, || {
        let key = (host.clone(), port);
        let notify = Arc::new(tokio::sync::Notify::new());
        in_flight.insert(key.clone(), notify.clone());
        dedup_cache.insert(
            key.clone(),
            ProbeOutcome::Ok {
                latency_ms: Some(9),
                ip_info: None,
            },
        );
        in_flight.remove(&dedup_key);
        black_box(dedup_cache.len() + in_flight.len());
    });
    rows_out.push(dedup_acc.row("fast dedup owner: 2 key clones + 2 map ops", 1.0));

    // ── 9. connection acquisition (finish_batch opens one after the burst) ─
    let mut acquires: Vec<u128> = Vec::with_capacity(20);
    for _ in 0..20 {
        let started = Instant::now();
        let conn = state.db.connection().await.expect("conn");
        acquires.push(started.elapsed().as_micros());
        drop(conn);
    }
    println!("[pool] db.connection() acquisitions (us): {acquires:?}");
    let mut sorted = acquires.clone();
    sorted.sort_unstable();
    rows_out.push(TableRow {
        name: "db.connection() acquisition (median)".to_owned(),
        ns: sorted[sorted.len() / 2] as f64 * 1000.0,
        n: 20,
    });

    // ── 10. batch-end checkpoint (finish_batch runs one per batch) ──────
    let mut ckpt_acc = Acc::new();
    for _ in 0..20 {
        let Ok(mut conn) = state.db.connection().await else {
            continue;
        };
        let started = Instant::now();
        let _ = tokio::time::timeout(
            Duration::from_secs(2),
            toasty::sql::query("PRAGMA wal_checkpoint(PASSIVE)").exec(&mut conn),
        )
        .await;
        ckpt_acc.add(started.elapsed());
    }
    rows_out.push(ckpt_acc.row("finish_batch: wal_checkpoint(PASSIVE), in-memory db", 1.0));

    // ── 10. end-to-end pipeline (stub probes, no sockets) ──────────────
    // Each variant runs TWICE: the first `run_batch` of a process is the one
    // that shows the ~1.5 ms/link tail (3.1 s over 2,000 links, against 53–59
    // µs/link for every later run), so the second pass is the steady-state
    // number and the pair is itself the evidence.
    for real_phase in [false, true] {
        for pass in 0..2 {
            let feed = synth_rows(500, 4);
            let state = test_state(feed.clone()).await;
            seed_db(&state, &feed).await;
            let plan: Vec<PlanLink> = feed.iter().flat_map(plan_row_links).collect();
            let planned = plan.len();
            let (tx, rx) = mpsc::channel::<CoreEvent>(1 << 16);
            let params = batch_params(&state, tx, plan, real_phase, 500, Arc::new(NullRunner));
            // The batch publishes its shared state here at start: after the run it
            // carries the level spans, the wall time and the flush count, which is
            // what says whether the tail sits in the probes or in `finish_batch`.
            let slot = Arc::clone(&params.batch_slot);
            let started = Instant::now();
            run_batch(params).await;
            let elapsed = started.elapsed();
            drop(rx);
            if let Some(shared) = slot.get() {
                println!("[summary] {}", summary_line(shared));
            }
            let label = if real_phase { "fast+real" } else { "fast only" };
            let pass_label = if pass == 0 { "pass 1" } else { "pass 2" };
            println!(
                "[end-to-end] {label} ({pass_label}): {planned} links in {:.1} ms ({:.1} us/link)",
                elapsed.as_secs_f64() * 1000.0,
                elapsed.as_secs_f64() * 1e6 / planned as f64
            );
            rows_out.push(TableRow {
                name: format!("run_batch {label} ({pass_label}, per link)"),
                ns: elapsed.as_nanos() as f64 / planned as f64,
                n: 1,
            });
        }
    }

    // ── 11. batch end in isolation (the e2e tail's suspect) ────────────
    // The end-to-end rows show a reproducible ~3.09 s stall on the SECOND
    // batch of each kind; `finish_batch` is the only code on that path with
    // multi-second bounds (2 s `db.connection()`, 2 s checkpoint). This times
    // it alone, with a staged window to flush.
    let end_state = test_state(synth_rows(500, 4)).await;
    seed_db(&end_state, &synth_rows(500, 4)).await;
    let (end_tx, end_rx) = mpsc::channel::<CoreEvent>(1 << 16);
    let end_shared = Arc::new(BatchShared::new(batch_params(
        &end_state,
        end_tx,
        Vec::new(),
        false,
        500,
        Arc::new(NullRunner),
    )));
    for link in &links_all {
        end_shared.stage_result(link, TestType::TcpPing, &outcome_ok);
    }
    let started = Instant::now();
    finish_batch(&end_shared).await;
    println!(
        "[batch end] finish_batch alone: {:.1} ms",
        started.elapsed().as_secs_f64() * 1000.0
    );
    rows_out.push(TableRow {
        name: "finish_batch (flush + checkpoint + sweep)".to_owned(),
        ns: started.elapsed().as_nanos() as f64,
        n: 1,
    });
    drop(end_rx);

    // ── 12. real-feed page path (optional) ─────────────────────────────
    if let Ok(path) = std::env::var("XRAY_TUI_MEASURE_DB") {
        match xray_tui_db::Database::open(&path).await {
            Ok(db) => {
                let db = Arc::new(db);
                let request = PageRequest {
                    view: PurgatoryView::All,
                    active_threshold: 0,
                    scope: PlanScope::All,
                    search: None,
                    group_id: None,
                    sort: PageSort::Address,
                    ascending: true,
                    offset: 0,
                    limit: PROFILES_PAGE_SIZE,
                };
                let mut page_acc = Acc::new();
                for _ in 0..10 {
                    let started = Instant::now();
                    let page = db.profiles_page(&request).await.expect("page");
                    page_acc.add(started.elapsed());
                    black_box(page.ids.len());
                }
                rows_out.push(page_acc.row("profiles_page (one page, real feed)", 1.0));

                let ids = db.profiles_page(&request).await.expect("page").ids;
                let mut hydrate_acc = Acc::new();
                for _ in 0..10 {
                    let started = Instant::now();
                    let hydrated = db.load_page_projection(&ids, false).await.expect("hydrate");
                    hydrate_acc.add(started.elapsed());
                    black_box(hydrated.len());
                }
                rows_out.push(hydrate_acc.row("load_page_projection (real feed)", 1.0));

                // The per-tick reload the UI runs whenever a result landed.
                let feed_rows = db.load_page_projection(&ids, false).await.expect("hydrate");
                let mut ui_state = test_state(feed_rows).await;
                ui_state.db = Arc::clone(&db);
                let mut reload_acc = Acc::new();
                for _ in 0..10 {
                    let started = Instant::now();
                    crate::ops::profiles::reload_profiles_preserving_selection(&mut ui_state).await;
                    reload_acc.add(started.elapsed());
                }
                rows_out
                    .push(reload_acc.row("reload_profiles_preserving_selection (real feed)", 1.0));

                // The same reload split into its two halves: the DB read
                // (`load_profiles_rows`) and the in-memory apply.
                let mut load_half = Acc::new();
                let mut apply_half = Acc::new();
                for _ in 0..10 {
                    let load = crate::ops::profiles::ProfilesLoad::from(&ui_state);
                    let started = Instant::now();
                    let loaded = crate::ops::profiles::load_profiles_rows(&db, &load).await;
                    load_half.add(started.elapsed());
                    match loaded {
                        Ok((rows, meta)) => {
                            let started = Instant::now();
                            crate::ops::profiles::apply_profiles_rows(&mut ui_state, rows, &meta);
                            apply_half.add(started.elapsed());
                        }
                        Err(_) => {}
                    }
                }
                rows_out.push(load_half.row("reload half: load_profiles_rows", 1.0));
                rows_out.push(apply_half.row("reload half: apply_profiles_rows", 1.0));

                // Attribution of the reload: which half of it pays.
                let view_request = PageRequest {
                    view: PurgatoryView::Active,
                    active_threshold: xray_tui_db::models::to_epoch(jiff::Timestamp::now())
                        - ui_state.purgatory_ttl_secs,
                    scope: PlanScope::All,
                    search: None,
                    group_id: None,
                    sort: PageSort::Test,
                    ascending: true,
                    offset: 0,
                    limit: PROFILES_PAGE_SIZE,
                };
                let mut acc = Acc::new();
                for _ in 0..10 {
                    let started = Instant::now();
                    let page = db.profiles_page(&view_request).await.expect("page");
                    acc.add(started.elapsed());
                    black_box(page.total);
                }
                rows_out.push(acc.row("profiles_page (Active view, Test sort)", 1.0));

                // Direction × sort: the reload uses the state's own pair, and a
                // mixed-direction composite index cannot serve the reverse of
                // every term.
                for (sort, ascending) in [
                    (PageSort::Test, true),
                    (PageSort::Test, false),
                    (PageSort::Address, true),
                    (PageSort::Address, false),
                    (PageSort::LastSeen, true),
                    (PageSort::LastSeen, false),
                ] {
                    let request = PageRequest {
                        view: PurgatoryView::Active,
                        active_threshold: view_request.active_threshold,
                        scope: PlanScope::All,
                        search: None,
                        group_id: None,
                        sort,
                        ascending,
                        offset: 0,
                        limit: PROFILES_PAGE_SIZE,
                    };
                    let mut acc = Acc::new();
                    for _ in 0..5 {
                        let started = Instant::now();
                        let page = db.profiles_page(&request).await.expect("page");
                        acc.add(started.elapsed());
                        black_box(page.total);
                    }
                    rows_out.push(acc.row(
                        &format!("profiles_page view=Active {sort:?} asc={ascending}"),
                        1.0,
                    ));
                }

                let view_ids = db.profiles_page(&view_request).await.expect("page").ids;
                let mut acc = Acc::new();
                for _ in 0..10 {
                    let started = Instant::now();
                    let hydrated = db
                        .load_page_projection(&view_ids, false)
                        .await
                        .expect("hydrate");
                    acc.add(started.elapsed());
                    black_box(hydrated.len());
                }
                rows_out.push(acc.row("load_page_projection (Active view page)", 1.0));

                // NOTE: `apply_profiles_rows` also calls `spawn_enrich_ip_hosts`
                // and `spawn_outbound_countries`, whose mmdb work runs on this
                // same runtime in the background. That fan-out is deliberately
                // NOT re-measured in a loop here: 20 iterations of it pile up
                // thousands of geo lookups and inflate every later row.

                for (label, sql) in [
                    (
                        "host lengths (endpoints)",
                        "SELECT MIN(LENGTH(host)), MAX(LENGTH(host)), AVG(LENGTH(host)), \
                         SUM(CASE WHEN LENGTH(host) <= 24 THEN 1 ELSE 0 END), COUNT(*) \
                         FROM endpoints",
                    ),
                    (
                        "distinct protocols per link",
                        "SELECT COUNT(DISTINCT protocol_id), COUNT(*) FROM profile_stats",
                    ),
                    (
                        "error text lengths",
                        "SELECT AVG(LENGTH(error_text)), MAX(LENGTH(error_text)) FROM profile_stats WHERE error_text IS NOT NULL",
                    ),
                ] {
                    if let Ok(mut conn) = db.connection().await {
                        if let Ok(rows) = toasty::sql::query(sql).exec(&mut conn).await {
                            println!("[feed] {label}: {rows:?}");
                        }
                    }
                }

                // ── page-query shape experiments (same engine the app uses) ─
                // The tab's DEFAULT sort is Address; the walk uses it too. The
                // staged-key sorts are index-driven, so the question is whether
                // the host order can be driven from an index instead of a sort.
                let mut conn = db.connection().await.expect("conn");
                let _ = toasty::sql::query(
                    "CREATE INDEX IF NOT EXISTS idx_measure_host ON endpoints(host, port)",
                )
                .exec(&mut conn)
                .await;

                let variants: [(&str, String); 4] = [
                    (
                        "raw: Address order, endpoint_rank-driven",
                        "SELECT k.endpoint_id FROM endpoint_rank k JOIN endpoints e ON e.id = k.endpoint_id \
                         WHERE k.rank_newest_seen >= ?1 ORDER BY e.host ASC, k.endpoint_id ASC LIMIT 200"
                            .to_owned(),
                    ),
                    (
                        "raw: id order, endpoint_rank-driven",
                        "SELECT k.endpoint_id FROM endpoint_rank k WHERE k.rank_newest_seen >= ?1 \
                         ORDER BY k.endpoint_id ASC LIMIT 200"
                            .to_owned(),
                    ),
                    (
                        "raw: Address order, forced host index",
                        "SELECT k.endpoint_id FROM endpoints e INDEXED BY idx_measure_host \
                         JOIN endpoint_rank k ON k.endpoint_id = e.id WHERE k.rank_newest_seen >= ?1 \
                         ORDER BY e.host ASC, k.endpoint_id ASC LIMIT 200"
                            .to_owned(),
                    ),
                    (
                        "raw: Address order, rank-driven + host join",
                        "SELECT k.endpoint_id FROM endpoint_rank k JOIN endpoints e ON e.id = k.endpoint_id \
                         WHERE k.rank_newest_seen >= ?1 ORDER BY e.host ASC LIMIT 200"
                            .to_owned(),
                    ),
                ];
                for (label, sql) in variants {
                    let mut acc = Acc::new();
                    for _ in 0..8 {
                        let started = Instant::now();
                        let result = toasty::sql::query(sql.clone())
                            .bind(0_i64)
                            .exec(&mut conn)
                            .await;
                        match result {
                            Ok(rows) => {
                                acc.add(started.elapsed());
                                black_box(rows.len());
                            }
                            Err(e) => {
                                println!("[{label}] failed: {e}");
                                break;
                            }
                        }
                    }
                    if !acc.samples.is_empty() {
                        rows_out.push(acc.row(label, 1.0));
                    }
                }
                drop(conn);

                // The batch's feed walk: page + hydrate per page, no probes.
                let mut offset = 0usize;
                let mut pages = 0u32;
                let mut links_seen = 0usize;
                let started = Instant::now();
                loop {
                    let request = PageRequest {
                        view: PurgatoryView::All,
                        active_threshold: 0,
                        scope: PlanScope::All,
                        search: None,
                        group_id: None,
                        sort: PageSort::Id,
                        ascending: true,
                        offset,
                        limit: PROFILES_PAGE_SIZE,
                    };
                    let (ids, total) = db
                        .profiles_walk_page(&request, offset == 0)
                        .await
                        .expect("walk");
                    if ids.is_empty() {
                        break;
                    }
                    offset += ids.len();
                    pages += 1;
                    let walked = db.load_page_projection(&ids, false).await.expect("hydrate");
                    links_seen += walked.iter().map(|r| r.links.len()).sum::<usize>();
                    if total.is_some_and(|t| offset as u64 >= t) {
                        break;
                    }
                }
                println!(
                    "[feed] plan walk: {pages} pages, {links_seen} links, {:.1} ms total, {:.1} ms/page",
                    started.elapsed().as_secs_f64() * 1000.0,
                    started.elapsed().as_secs_f64() * 1000.0 / f64::from(pages.max(1)),
                );
                rows_out.push(TableRow {
                    name: "plan walk (page+hydrate, per page, id order)".to_owned(),
                    ns: started.elapsed().as_nanos() as f64 / f64::from(pages.max(1)),
                    n: pages,
                });
            }
            Err(e) => println!("[real feed] open failed: {e}"),
        }
    }

    print_table("fast + real ping flow", &rows_out);
}

/// The one row the lab could not produce: the REAL level's network cost.
///
/// Every other section probes through `NullRunner`, so the real level's cost —
/// the long pole a batch is bound by — is measured nowhere. This runs the
/// PRODUCTION runner (`EngineProbeRunner`) over a pinned slice of a real feed,
/// and a direct per-attempt pass for the latency distribution.
///
/// Knobs (all optional except the DB; the slice descriptor is printed so a run
/// is reproducible):
///   `XRAY_TUI_MEASURE_DB`           path to a COPY of data.db (required)
///   `XRAY_TUI_MEASURE_MAX_LINKS`    slice size in links (default 15000)
///   `XRAY_TUI_MEASURE_CONCURRENCY`  real level concurrency (default 256)
///   `XRAY_TUI_MEASURE_BUDGET_SECS`  per-attempt budget (default 5)
///   `XRAY_TUI_MEASURE_SAMPLE`       links in the latency sample (default 400)
///
/// **What the measured span includes.** The batch's real half caches a loaded
/// protocol per batch, so a run pays one `load_protocol_with_config` per
/// distinct protocol it touches — that is inside the timed region and is part
/// of what a production batch pays too, but it means the absolute is not "probe
/// only". The before/after ratio stays valid because the slice and its protocol
/// count are pinned. The per-attempt pass (below) has no such term, which is
/// why the budget rule reads its distribution and not the batch's wall time.
///
/// **The harness-only knobs are the only difference from production**:
/// `real_concurrency`/`fast_concurrency` are raised here and nowhere else;
/// `dedup_endpoints` stays OFF and `real_phase` stays true, so the run is the
/// batch's own shape rather than a tuned variant.
#[ignore = "perf lab: run explicitly with --ignored"]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn flow_cost_network() {
    let Ok(path) = std::env::var("XRAY_TUI_MEASURE_DB") else {
        println!("SKIP flow_cost_network: set XRAY_TUI_MEASURE_DB=<copy of data.db>");
        return;
    };
    let max_links = env_usize("XRAY_TUI_MEASURE_MAX_LINKS", 15_000);
    let concurrency = env_usize("XRAY_TUI_MEASURE_CONCURRENCY", 256);
    let budget = Duration::from_secs(env_usize("XRAY_TUI_MEASURE_BUDGET_SECS", 5) as u64);
    let sample_size = env_usize("XRAY_TUI_MEASURE_SAMPLE", 400);
    const PING_URL: &str = "https://www.gstatic.com/generate_204";

    let db = Arc::new(
        xray_tui_db::Database::open(&path)
            .await
            .expect("open measure db"),
    );

    // Pin the slice: page in ID order — an order no write can move — until the
    // cap. The descriptor is printed so the same slice can be re-run.
    let mut rows: Vec<EndpointRow> = Vec::new();
    let mut links = 0usize;
    let mut offset = 0usize;
    loop {
        if links >= max_links {
            break;
        }
        let request = PageRequest {
            view: PurgatoryView::All,
            active_threshold: 0,
            scope: PlanScope::All,
            search: None,
            group_id: None,
            sort: PageSort::Id,
            ascending: true,
            offset,
            limit: PROFILES_PAGE_SIZE,
        };
        let ids = db.profiles_page(&request).await.expect("page").ids;
        if ids.is_empty() {
            break;
        }
        let page = db.load_page_projection(&ids, true).await.expect("hydrate");
        links += page.iter().map(|r| r.links.len()).sum::<usize>();
        rows.extend(page);
        offset += PROFILES_PAGE_SIZE;
    }
    let plan: Vec<PlanLink> = rows
        .iter()
        .flat_map(plan_row_links)
        .take(max_links)
        .collect();
    let planned = plan.len();
    let protocols: std::collections::HashSet<_> = plan.iter().map(|p| p.link.protocol_id).collect();
    println!(
        "[network] slice: {planned} links over {} endpoints, {} distinct protocols \
         (ID order, offset 0..{offset})",
        rows.len(),
        protocols.len()
    );
    println!(
        "[network] harness-only knobs: concurrency {concurrency} (production default 100, \
         this lab's other sections 64), budget {}s; dedup_endpoints OFF, real_phase true",
        budget.as_secs()
    );

    // ── the batch's own shape, through the production runner ───────────
    let mut state = test_state(rows.clone()).await;
    // The feed for the protocol loads; the run's own writes land in the lab's
    // temp db through the writer, which is what a lab run wants.
    state.db = Arc::clone(&db);
    let (tx, rx) = mpsc::channel::<CoreEvent>(1 << 16);
    let mut params = batch_params(
        &state,
        tx,
        plan,
        true,
        PROFILES_PAGE_SIZE,
        Arc::new(EngineProbeRunner),
    );
    params.real_concurrency = concurrency;
    params.fast_concurrency = concurrency;
    params.real_timeout = budget;
    params.dedup_endpoints = false;
    params.ping_url = PING_URL.to_owned();
    params.ip_provider = IpProvider::IpApi;
    let slot = Arc::clone(&params.batch_slot);
    let started = Instant::now();
    run_batch(params).await;
    let elapsed = started.elapsed();
    drop(rx);
    // The batch's own summary line, not a re-derivation: the lab and the app
    // then report the same numbers by construction.
    if let Some(shared) = slot.get() {
        use std::sync::atomic::Ordering;
        println!("[summary] {}", summary_line(shared));
        let done = u64::from(shared.counters.real_ok.load(Ordering::Relaxed))
            + u64::from(shared.counters.real_failed.load(Ordering::Relaxed));
        println!(
            "[network] real results {done} in {:.1} s = {:.2} results/s",
            elapsed.as_secs_f64(),
            done as f64 / elapsed.as_secs_f64()
        );
    } else {
        println!("[network] no batch handle published — the run never started");
    }

    // ── the per-attempt distribution (the budget rule's input) ─────────
    //
    // `real_ping` with `retries: 1` measures ONE attempt's span — dial →
    // protocol → target → response head — with no batch, no writer and no
    // per-protocol cache term, so this is the number the budget default must
    // clear. Sequential on purpose: the point is the span, not throughput.
    let mut latencies: Vec<u64> = Vec::new();
    let mut failures = 0usize;
    // The sample pass is SEQUENTIAL on purpose — concurrent attempts contend for
    // the CPU-bound handshake and would inflate every span. Sequential makes its
    // wall time the sum of its attempts, and most attempts here are failures
    // that run to the full budget, so it gets a deadline of its own: the
    // distribution is what matters, not how many samples fit.
    let sample_deadline = Duration::from_secs(
        env_usize("XRAY_TUI_MEASURE_SAMPLE_DEADLINE_SECS", 300) as u64,
    );
    let sample_started = Instant::now();
    let req = NativeProbeReq {
        ping_url: PING_URL,
        ip_provider: IpProvider::IpApi,
        timeout: budget,
        retries: 1,
    };
    'sample: for row in rows.iter() {
        if latencies.len() >= sample_size {
            break;
        }
        for link in row.links.iter() {
            if latencies.len() >= sample_size || sample_started.elapsed() >= sample_deadline {
                break 'sample;
            }
            let Ok(Some(protocol)) =
                crate::state::load_protocol_with_config(&db, link.protocol_id).await
            else {
                continue;
            };
            // `Deferred<Json<_>>::get()` — the same accessor the batch's
            // `protocol_config` uses, so an unloaded row fails here exactly as
            // it fails on the production path.
            let config = protocol.config.get().0.clone();
            match crate::ops::ping_native::real_ping(&row.endpoint, &config, &req).await {
                Ok(r) => latencies.push(r.latency_ms),
                Err(_) => failures += 1,
            }
        }
    }
    latencies.sort_unstable();
    if let (Some(min), Some(max)) = (latencies.first(), latencies.last()) {
        let pct = |p: f64| latencies[((latencies.len() as f64 - 1.0) * p) as usize];
        println!(
            "[network] per-attempt successes: {} (failures {failures}) | min {min} ms | \
             median {} ms | p90 {} ms | p99 {} ms | max {max} ms",
            latencies.len(),
            pct(0.50),
            pct(0.90),
            pct(0.99)
        );
        if latencies.len() < 200 {
            println!(
                "[network] WARNING: {} successes is below the 200 the budget rule needs — \
                 raise XRAY_TUI_MEASURE_SAMPLE or the slice",
                latencies.len()
            );
        }
    } else {
        println!("[network] per-attempt sample produced no successes (failures {failures})");
    }
}

/// `XRAY_TUI_MEASURE_*` knob: a positive integer, or the default.
fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(default)
}

/// A 64-bit FNV-1a hash: the no-allocation dedup key a `(u64, u16)` map needs.
fn fnv(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}

/// Time `poll_core_events` over 2000 results whose endpoint is (or is not) on
/// the loaded page.
async fn hit_miss_rows(hit: bool) -> Vec<TableRow> {
    let page = synth_rows(200, 4);
    let mut state = test_state(page.clone()).await;
    let (tx, rx) = mpsc::channel::<CoreEvent>(1 << 16);
    state.core_event_tx = Some(tx.clone());
    state.core_event_rx = Some(rx);
    let (endpoint_id, protocol_id) = if hit {
        (
            page[0].endpoint.id.get(),
            page[0].links[0].protocol_id.get(),
        )
    } else {
        (9_000_001, 900_000_101)
    };
    let results = 2000;
    let started = Instant::now();
    for _ in 0..results {
        let _ = tx.try_send(CoreEvent::TestTypeUpdate {
            endpoint_id,
            protocol_id,
            test_type: TestType::TcpPing,
        });
        let _ = tx.try_send(CoreEvent::SpeedTestResult {
            endpoint_id,
            protocol_id,
            test_type: TestType::TcpPing,
            latency_ms: Some(12),
            speed_bps: None,
            ip_info: None,
            error: None,
            purge: None,
        });
    }
    while state.poll_core_events().await {}
    let elapsed = started.elapsed();
    drop(tx);
    let label = if hit { "row on page" } else { "row off page" };
    vec![TableRow {
        name: format!("poll_core_events per result ({label})"),
        ns: elapsed.as_nanos() as f64 / f64::from(results),
        n: 1,
    }]
}
