# 2026-09-17 — Batch ping pipeline: streaming plan, pipelined probes, two-level progress

Parent spec: `docs/aegis/specs/2026-09-17-batch-ping-pipeline-design.md` (approved).

## Goal

Batch ping starts probing within a page of its plan load instead of after the
whole walk, real probes run as each link's fast probe settles, both probe levels
report themselves with rates and ETAs in the Actions Log panel, and the country
step stops paying one commit per endpoint on large imports.

## Architecture

One incremental pipeline in `crates/xray-tui/src/ops/ping.rs`:

- `BatchShared` holds increments (`plan_len` + `DashMap`s filled page by page),
  not a buffered `Vec<PlanLink>`; `run_batch` walks pages and dispatches per page.
- Each link gets **one chain** (`run_task_chain`) that runs its fast half and then
  dispatches its real half from that settle. No phase barrier, no gate queue.
- `types::BatchMeters` (plan pages + fast + real, each with a 1 Hz rate) replaces
  the `(total, completed)` tuple; per-settle progress events are deleted and one
  `CoreEvent::BatchEnded` closes the batch.
- The Actions Log rows 3-4 gain a right region with one bar per level, sized from
  a constant stat block; the bar renderer is hoisted into `ui/widgets/progress.rs`.
- The DB gains a count-free walk page and a one-transaction country batch.

## Tech Stack

Rust 2024, tokio, ratatui 0.30 + ratatui-cheese 0.7, toasty 0.10 (turso),
dashmap (already used by `ops/scheduler.rs`), criterion/nextest for gates.

## Baseline / Authority Refs

- `AGENTS.md` decisions 4, 15, 16, 20, 21, 22; `docs/database.md`;
  `docs/database-manual-sql.md` (the page-SQL owner's rules);
  `docs/aegis/adr/0002-write-behind-link-writer.md`;
  `docs/aegis/plans/2026-09-17-ping-run-analysis.md` (§F4/F5, P4/P10 deferred);
  `docs/aegis/specs/2026-09-14-db-tui-flow-audit.md` (per-page measurements).

## Compatibility Boundary

No persisted schema, config key, protocol identity, or wire-format change. The
`label`/`BatchProgress` event is process-internal. `speed_test.task_queue_limit`
keeps its setting and its semantics; the batch simply never queues (as today).
`profile_stats` write path, markers, and TTL sweep are unchanged.

## TDD Route

```text
TDD Route:
- Mode: off (no activation config present; project has no TDD mandate)
- Decision: skipped
- Strict authority: not applicable
- Strict signals: n/a (recorded for completeness: behavior + shared-core changes)
- Light eligibility: n/a
- TDD-fit exception: n/a
- Test posture: post-change regression on the seams the change moves
  (pipeline staging/skip rules, meters arithmetic, country batch transaction,
  walk/page parity) plus the existing suite
- Reason: the plan's risk sits in restructuring an already-tested pipeline; the
  existing stubbed-runner harness covers the seams, so focused regressions
  replace RED/GREEN ceremony
- Verification: `cargo nextest run -p xray-tui -p xray-tui-db`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo fmt --all --check`, and the live run in T6
```

## Plan Basis

```text
Plan Basis:
- Spec: docs/aegis/specs/2026-09-17-batch-ping-pipeline-design.md (approved by user)
- Evidence: 4 read-only scout reports (turn-local) + the recorded 2026-09-17 run analysis
- Requirement Ready Check: ready (spec §7 acceptance list is the contract)
- BaselineUsageDraft: required = AGENTS.md 15/16/22, database-manual-sql.md,
  write-behind ADR 0002; cited = §2 evidence table; missing = none; decision = continue
- Change Necessity: user-visible need = batch start latency, absent progress, commit
  storm; non-code option = none (behavior + rendering + write granularity);
  minimum boundary = ops/ping.rs, ops/events.rs, types.rs, ui/{actions_log,status_bar,settings}.rs,
  ui/widgets/progress.rs, db/{profiles_query,endpoint_ip,database}.rs, ops/enrich.rs
- Existence Check: all five new surfaces reuse an existing owner (spec §9)
- Architecture Integrity Lens: higher-level path found and taken — the chain owns
  link sequencing instead of the gate queue, so no new gate/queue machinery and no
  queue-limit dependency; the panel's bar renderer is hoisted rather than duplicated
- Plan Pressure Test: proceed (owner/contract/retirement stated per task; verification
  is the existing harness + the live run; no task exceeds one file family except T3)
```

## Files

| Task | Create | Modify |
| --- | --- | --- |
| T1 | — | `crates/xray-tui-db/src/profiles_query.rs` |
| T2 | — | `crates/xray-tui/src/ops/ping.rs`, `ops/events.rs`, `types.rs`, `state.rs`, `ui/status_bar.rs`, `ui/mod.rs` |
| T3 | `crates/xray-tui/src/ui/widgets/progress.rs` | `ui/widgets/mod.rs`, `ui/settings.rs` |
| T4 | — | `crates/xray-tui/src/ui/actions_log.rs` (needs T2's meters) |
| T5 | — | `crates/xray-tui-db/src/{database,endpoint_ip}.rs`, `crates/xray-tui/src/ops/enrich.rs` |
| T6 | — | `AGENTS.md`, `docs/database.md`, `docs/database-manual-sql.md`, spec status, `docs/aegis/INDEX.md` |

## Tasks

### T1 — Count-free walk page (db)

**Files**: modify `crates/xray-tui-db/src/profiles_query.rs`.

**Why**: the plan walk re-counts the feed on every page
(`profiles_page` → `profiles_count_with`), `O(feed²/200)`; the walk needs ids and
one total.

**Steps**

1. Add beside `profiles_page` (same builders, same module):

```rust
    /// One page of endpoint ids for a sequential walk of the whole feed, in
    /// display order — the batch plan loader's read.
    ///
    /// `profiles_page` re-counts the filtered set on every call, which made a
    /// feed-wide walk `O(feed²/200)` (92 COUNTs for one 18k-endpoint batch,
    /// measured 7.5 ms each). A walk asks for the total once
    /// (`with_total`), then reads ids only.
    pub async fn profiles_walk_page(
        &self,
        req: &PageRequest,
        with_total: bool,
    ) -> Result<(Vec<EndpointId>, Option<u64>)> {
        let mut conn = self.connection().await?;
        let total = if with_total {
            Some(self.profiles_count_with(&mut conn, req).await?)
        } else {
            None
        };
        let mut sql = Sql::new();
        base_select(&mut sql, req, PROJ_ID, needs_endpoints(req));
        order_by(&mut sql, &req.order_terms());
        let limit = sql.bind(i64::try_from(req.limit).unwrap_or(i64::MAX));
        let offset_bind = sql.bind(i64::try_from(req.offset).unwrap_or(i64::MAX));
        sql.push(&format!(" LIMIT {limit} OFFSET {offset_bind}"));
        let rows = sql.exec(&mut conn).await?;
        let ids = rows.iter().map(decode_id).collect::<Result<Vec<_>>>()?;
        Ok((ids, total))
    }
```

2. Test in the module's `#[cfg(test)]`: seed N ≥ 2.5 pages of endpoints, walk with
   `limit = 1` (so the walk takes several pages) and assert the concatenated ids
   equal `profiles_page`'s ids read at every offset with the same `PageRequest`,
   and that `with_total` returns `Some(total)` only on the first call.

**Verification**: `cargo nextest run -p xray-tui-db`.

### T2 — Pipeline restructure (TUI core)

**Files**: modify `crates/xray-tui/src/ops/ping.rs`, `ops/events.rs`, `types.rs`,
`state.rs`, `ui/status_bar.rs`, `ui/mod.rs`.

**Why**: this is the user-visible defect (delayed start, no progress, no
pipelining).

**Steps**

1. `types.rs`: delete `pub type BatchProgress = Arc<(AtomicU32, AtomicU32)>`; add

```rust
/// Live meters for one probe level (fast or real).
#[derive(Debug, Default)]
pub struct PhaseMeters {
    pub total: AtomicU32,
    pub done: AtomicU32,
    /// results/s × 1000, sampled at most once per second by the batch task.
    pub rate_milli: AtomicU32,
}

impl PhaseMeters {
    /// Remaining seconds at the sampled rate; `None` when unknown/complete.
    #[must_use]
    pub fn eta_secs(&self) -> Option<u64> {
        let total = self.total.load(Ordering::Relaxed);
        let done = self.done.load(Ordering::Relaxed);
        let rate = self.rate_milli.load(Ordering::Relaxed);
        if total == 0 || done >= total || rate == 0 {
            return None;
        }
        Some(u64::from(total - done) * 1000 / u64::from(rate))
    }
}

/// Per-batch meters: the plan walk, then the two probe levels. Written by the
/// batch task, read by the render path — a per-result event bought nothing the
/// shared `Arc` does not already publish.
#[derive(Debug, Default)]
pub struct BatchMeters {
    pub plan_pages_done: AtomicU32,
    pub plan_pages_total: AtomicU32,
    pub fast: PhaseMeters,
    pub real: PhaseMeters,
}
```
   and replace `CoreEvent::BatchProgress { total, completed }` with
   `CoreEvent::BatchEnded` (doc: "the batch's terminal signal: everything is
   flushed and no further probe will report").

2. `state.rs`: `pub batch_progress: Option<Arc<crate::types::BatchMeters>>`; doc
   update; init `None` unchanged.

3. `ops/ping.rs` — `BatchShared`:
   - drop `plan: Vec<PlanLink>`; add `plan_len: AtomicU32`,
     `meters: Arc<BatchMeters>`,
     `fast_config: DashMap<(ProtocolId, EndpointId), i32>`,
     `endpoints: DashMap<EndpointId, Endpoint>`,
     `untestable: DashMap<(ProtocolId, EndpointId), String>`,
     `real_sem: Arc<Semaphore>`, `pending_deferred: AtomicU32`,
     `rate_fast/rate_real: Mutex<(Instant, u32)>`,
     `walk_done: AtomicBool`.
   - `BatchParams`: drop `progress`, add `meters`; keep `plan`.
   - `BatchShared::new(params)` takes no plan; the entry points build the meters
     and hand them to both `AppState` and `BatchParams`.
4. `dispatch_page(&self, rows: &[EndpointRow], handles: &mut Vec<JoinHandle<()>>, retries: &mut Vec<JoinHandle<()>>) -> usize`
   (async): per link — insert into `fast_config`/`endpoints`, fill `untestable`
   from the existing kind gate (`capability::support_reason` call site, moved out
   of `BatchShared::new`), `plan_len += 1`, `meters.fast.total += 1`, then
   `dispatch_fast_link`.
5. `dispatch_fast_link(&self, link, handles, retries)`: `schedule(FastPing)` —
   `Started(id)` → `pending_fast += 1` + spawn `run_task_chain`;
   `Queued(_)` → `pending_fast += 1` (defensive); `DnsDeferred` →
   `counters.deferred += 1` + spawn `defer_retry(Fast)`; `QueueFull` →
   `counters.queue_full += 1`. Mirrors today's phase-1 arms exactly.
6. `run_task_chain`: after the fast half settles (`emit_result` → `complete` →
   `note_settled(FastPing)`), call `after_fast_settle(&link)` — **always**, not
   only for a real-phase batch — and keep the promotion tail for the defensive
   `Queued` case.
7. `after_fast_settle(&self, link)` — the whole per-link decision, in the order
   the old code's *unconditional* post-phase-1 pass plus the phase-2 loop made it:

```rust
    /// Everything a link owes after its fast result is staged: the untestable
    /// marker (for both batch kinds — today's pass runs before the
    /// `!real_phase` return, and dropping it for a fast-only run would make
    /// `remove_failed_servers` treat such a row as a genuine failure and delete
    /// it), then its real half.
    async fn after_fast_settle(&self, link: &ProfileStats) {
        let key = (link.protocol_id, link.endpoint_id);
        if let Some(reason) = self.untestable.get(&key) {
            self.counters.untestable.fetch_add(1, Ordering::Relaxed);
            self.emit_untestable_marker(link, reason.value());
            return; // never a real candidate: nothing counted
        }
        if !self.real_phase {
            return;
        }
        if self.hard_fast.lock().contains(&key) {
            self.counters.unreachable.fetch_add(1, Ordering::Relaxed);
            return;
        }
        self.meters.real.total.fetch_add(1, Ordering::Relaxed);
        if self.dedup_endpoints
            && self.completed_endpoints.lock().contains(&link.endpoint_id.get())
        {
            self.note_real_done(); // counted, retired without a probe
            return;
        }
        self.dispatch_real_probe(Arc::new(link.clone()), true).await;
    }
```

   A test asserts a fast-only batch still writes the `[real]` marker for a
   kind-gated link (and that `is_untestable_marker` still holds on it).
8. `dispatch_real_probe(&self, link: Arc<ProfileStats>, counted: bool)`: acquire
   `real_sem`, `schedule(RealPing)` — `Started(id)` → `pending_real += 1`, probe,
   record `completed_endpoints` on `Ok` when `dedup_endpoints`, `emit_result`,
   `complete`, `note_settled(RealPing)`, `note_real_done()`; `Queued(_)` →
   `pending_real += 1` (promoted); `DnsDeferred` → `counters.deferred += 1` +
   `defer_retry(Real)` (do **not** bump `done`); `QueueFull` →
   `counters.queue_full += 1` + `note_real_done()` when `counted`.
   Invariant: every link counted into `real.total` reaches `real.done` exactly
   once (probe, dedup retire, stop retire, or queue-full).
9. `defer_retry(shared, link, half)`: `pending_deferred += 1`; loop —
   `sleep(defer_delay)`; on `stop` → `pending_deferred -= 1; return`; else
   re-enter `dispatch_fast_link`/`dispatch_real_probe(link, false)` for that half;
   `pending_deferred -= 1` on completion of each attempt that did not hand off to
   a nested retry.
10. `note_settled(kind)`: fast half → `pending_fast.fetch_sub` + `meters.fast.done
    += 1` + `sample_rate(fast)`; real half → `pending_real.fetch_sub` only.
    `note_real_done()` → `meters.real.done += 1` + `sample_rate(real)`.
    `sample_rate` writes `rate_milli` at most once per second from
    `Mutex<(Instant, u32)>` deltas (results/s × 1000).
11. `run_batch`: one `profiles_walk_page(req, true)` for the total (sets
    `meters.plan_pages_total`), then a page loop
    (`PROFILES_PAGE_SIZE`) that calls `dispatch_page` per page; a page error sets
    `counters.plan_error` + one `warn` naming the offset and breaks; a new
    `info` line reports `plan: N pages, M links in X ms`; `warn_if_real_phase_is_slow`
    runs at walk end when `real_phase`; then wait on
    `pending_fast == 0 && pending_real == 0 && pending_deferred == 0` (Notify,
    same register-before-read pattern), join the handles, record
    `fast_span_ms`/`real_span_ms` from the first-start/last-settle instants, and
    `finish_batch`.
12. Delete: `load_feed_plan`, `emit_untestable_markers` (→
    `emit_untestable_marker(link, reason)`), `retry_deferred_fast`,
    `retry_deferred_real`, `publish_final_total`, `bump_progress`, the phase-2
    candidate/`per_endpoint`/group-task block, and the `real_phase` branch inside
    `note_settled`.
13. `summary_line`: `planned=N links over P pages (plan X ms)` + fast
    ok/hard-fail/soft-fail histogram + `fast_span` + real ok/failed/skipped
    histogram + `real_span` + total wall + `flushes`/`staged-left`;
    `interrupted_summary_line` adds `settled={fast.done+real.done}
    in-flight={pending_fast+pending_real+pending_deferred}`. Update the existing
    assertions on the old text.
14. `finish_batch`: keep the flush → sweep order; send `CoreEvent::BatchEnded`
    instead of the terminal pair.
15. `ops/events.rs`: `BatchEnded` arm clears `batch_progress` + `batch` and
    re-arms `speed_test_stop`; the stale-bar guard becomes
    `state.batch_progress.is_none() && state.batch.is_none()`.
16. `ui/status_bar.rs`: render from `BatchMeters` —
    `Testing: F {fast.done}/{fast.total}` plus `· R {real.done}/{real.total}`
    when `real.total > 0`; `Testing...` while both totals are 0.

**Verification**: `cargo nextest run -p xray-tui` (the pipeline tests use the
stubbed runner), plus the new tests: a real probe dispatched before the last fast
probe settles; `task_queue_limit = 0` completes the real phase; a plan-time
deferred link schedules nothing until the window and then fast→real; a fast-only
batch never schedules a real half; meters arithmetic (`done <= total`, ETA `None`
cases).

### T3 — Shared bar renderer (widgets)

**Files**: create `crates/xray-tui/src/ui/widgets/progress.rs`; modify
`ui/widgets/mod.rs`, `ui/settings.rs`.

**Why**: two bar renderings would otherwise exist (anti-entropy).

**Steps**

1. `bar_line(label: &str, done: u32, total: u32, eta: Option<u64>, width: usize, palette: &Palette) -> Line<'static>`
   — `label`, `[███>░░░]` with the fill/track width derived from `width`, `done/total`,
   `~12m`/`~40s`/`~1h05m` from `eta`; `--` when `None`; styled with
   `ThemeStyles::progress_fill`/`progress_bar`; degrades to numbers-only below
   30 columns and to an empty line below 18 (spec §3.4 ladder). Plus
   `format_eta(secs: u64) -> String` and a unit test for the ladder boundaries.
2. Migrate `ui/settings.rs::progress_bar_line` onto it (byte formatting stays in
   `settings.rs`), delete the private copy.
3. `ui/widgets/mod.rs`: `pub mod progress;`.

**Verification**: `cargo nextest run -p xray-tui`; the existing settings tests
that assert download-bar text keep passing or are updated to the shared shape.

### T4 — Actions Log bars (panel)

**Files**: modify `crates/xray-tui/src/ui/actions_log.rs`.

**Why**: the visible half of item 2.

**Steps**

1. Pad the stat cells to constants (rows 3-4): `⏱ TCP:{v:>6}ms  RP:{v:>6}ms
   SPD:{v:>5}Mbps` and `📊 ⬆{up:>9}  ⬇{down:>9}  💾{mem:>8}` with `STAT_BLOCK_W = 42`.
2. Build rows as `(left_line, Option<right_line>)` and paint: left region
   `Rect { width: inner.width - right_w }`, a `│` separator cell, then the right
   region; `right_w = inner.width - STAT_BLOCK_W - 1` with the ladder from the
   widget. Row 3 = real bar, row 4 = fast bar; a fast-only batch puts its bar on
   row 3 and row 4 renders full width; no batch → both rows full width exactly as
   today.
3. Read meters from `state.batch_progress`; show `planning p/P` in the fast bar's
   place while `plan_pages_total > 0 && plan_pages_done < plan_pages_total`.

**Verification**: `cargo nextest run -p xray-tui` + the live TUI check in T6.

### T5 — Country writes: one transaction per page (db + enrich)

**Files**: modify `crates/xray-tui-db/src/database.rs`,
`crates/xray-tui-db/src/endpoint_ip.rs` (only if a helper is needed),
`crates/xray-tui/src/ops/enrich.rs`.

**Why**: item 3 — one commit per endpoint contends with the import's chunk
commits.

**Steps**

1. `Database::set_endpoint_ip_countries(&self, rows: &[(EndpointId, IpAddr, String)]) -> Result<()>`
   — one `conn()`, one `transaction()`, `endpoint_ip::set_country` per row inside
   it, one `commit()`, wrapped in `retry_on_busy(…, 5)`; no-op on empty.
2. `ops/enrich.rs`: the `spawn_enrich_ip_hosts` page loop collects
   `(endpoint_id, ip, iso)` and flushes once per page and once at the end; keep
   `spawn_dns_resolve`'s single-row write.
3. Observability: one `debug` line per seed pass —
   `geo seed: scanned=N pending=M countries=K flushes=F` — plus a
   `geo_seed_countries` counter on the counters struct if the pass already has one.
4. Test in `xray-tui-db`: a batch of 3 rows lands in one call and all three
   countries read back; an empty batch is a no-op; a re-resolution keeps them
   (existing country tests cover the carry).

**Verification**: `cargo nextest run -p xray-tui-db -p xray-tui`.

### T6 — Docs, gates, live verification

**Files**: modify `AGENTS.md` (decision 15 pipeline/progress text, decision 16's
untestable ordering note), `docs/database.md` (walk + country batch),
`docs/database-manual-sql.md` (the walk method beside `profiles_page`),
`docs/aegis/INDEX.md`, the spec's status.

**Steps**

1. Amend the docs to the shipped shape; answer the spec §10 baseline-sync
   question ("does decision 15 still describe two sequential phases?" → no).
2. `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
   `cargo nextest run --workspace`.
3. Live: run the release binary against the reference config, start Fast+Real,
   confirm (a) the `plan:` timing line and the first-dial gap, (b) both bars move
   during the fast phase, (c) real results land while fast is still running,
   (d) stop/quit reports the interrupted summary. Screenshot the panel.
4. One scoped commit per task; the doc commit last.
5. Evidence recorded (2026-09-17): first plan page 62.2 ms / whole walk 5.45 s on
   the 27,142-endpoint reference feed (release build); the live TUI run showing
   `Real … 46/1193` beside `planning 3/136 pages` with `Testing: F 1572/1576 ·
   R 46/1193`; 1969 workspace tests green. The walk's > 1 s escalation trigger is
   met but deliberately not taken (the walk no longer delays the first probe) —
   recorded in the spec §7/§8.

## Risks

| Risk | Mitigation |
| --- | --- |
| Counter leak ⇒ batch never finishes | `pending_fast`/`pending_real`/`pending_deferred` are asserted in a stubbed-runner test that ends with all three at 0; `finish_batch` still runs on the stop path |
| A retired real half double-counts/loses `real.done` | the invariant in T2.8 has a dedicated test per path (probe, dedup, stop, queue-full) |
| Narrow terminals lose the bars | the widget's ladder + the status-bar numbers |
| DashMap growth (34k links) | same order as today's `states` map; `plan_len` replaces the plan Vec |
| `profiles_walk_page` drift from `profiles_page` | T1's parity test |

## Retirement

Deleted by this plan: `load_feed_plan`, `emit_untestable_markers` (global pass),
`retry_deferred_fast`, `retry_deferred_real`, `publish_final_total`,
`bump_progress`, `CoreEvent::BatchProgress`, `types::BatchProgress`,
`BatchShared.plan`, the phase-2 candidate computation + best-first sort +
`per_endpoint` groups + endpoint-group tasks, `settings.rs::progress_bar_line`.
Kept: `TaskScheduler` queue machinery and `task_queue_limit` (config compat; the
batch simply never queues — flagged as a follow-up question in the spec §8).

## Execution Readiness View

```text
Execution Readiness View:
- Intent Lock: spec §3.1-§3.5 + the four backlog items
- Scope Fence: no probe/engine change, no schema/config/identity change, no result-event batching
- Baseline Lock: AGENTS.md 15/16/20/22, database-manual-sql.md, ADR 0002
- Approved Behavior: spec §7 acceptance items 1-8
- Owner / Contract Constraints: ping.rs owns batch sequencing; profiles_query owns page SQL; endpoint_ip owns country rows; widgets/progress owns bar rendering
- Compatibility Boundary: see header
- Retirement Boundary: see Retirement
- Task Batches: T1; then (T2 inline ‖ T3 ‖ T5); then T4; then T6
- Test Obligations: pipeline stubbed-runner tests, walk parity, meters arithmetic, country batch, existing suite
- Review Gates: clippy/fmt/nextest per task; live TUI run in T6
- Drift / Rewind Rules: a task that finds a spec contradiction stops and updates the spec, not the code silently
- Evidence Required Before Completion: green gates + the live run observations (a)-(d)
- Advisory Boundary: method-pack execution guidance only
```

Execution route: **inline for T2/T4/T6** (irreducible shared mutation boundary),
**subagent for T3 and T5** (independent files, mechanical).
