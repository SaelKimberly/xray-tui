# 2026-09-17 — Batch ping: streaming plan, per-link pipeline, two-level progress

Status: `proposed` (awaiting user review)
Scope: `crates/xray-tui` (batch pipeline + Actions Log panel), `crates/xray-tui-db`
(one walk API, one batched country write). No engine, protocol, or identity change.

## 1. Trigger

Four items from the UI/UX backlog, all on the batch ping path:

1. **Long delay before Fast + Real ping start.** "Research for reason."
2. **No visible progress during Fast Ping.** Wanted: two progress bars in the
   Actions Log panel (Real on the first row, Fast on the second; Fast alone takes
   the first row), with a remaining-time estimate.
3. **Country ISO update causes very frequent transactions on large imports.**
4. **Parallel Fast + Real ping.** Real probes should start as fast results land,
   not after the whole fast phase finishes.

Item 1 and item 4 are the same defect in two views: the batch resolves its whole
plan and finishes its whole fast phase before any real work is dispatched. The
design therefore restructures dispatch once and derives both fixes from it.

## 2. Evidence base

All anchors `path:line`; `FACT` = observed in code or docs, `INFERENCE` = derived.

### 2.1 The start delay is the plan walk, not the probes

- `run_batch` resolves `PlanSource::Feed` through `load_feed_plan`
  (`crates/xray-tui/src/ops/ping.rs:1820-1853`) before it schedules anything; the
  walk is a serial `loop` over 200-endpoint pages, each page running
  `profiles_page` = `SELECT COUNT(*)` + the ordered-id `SELECT`
  (`crates/xray-tui-db/src/profiles_query.rs:344-355`) followed by
  `load_page_projection` (`:830`) — 3 statements per page, each acquiring a
  pooled connection. The per-page cost is measured: 13.2 ms (count + ids) +
  49.6 ms (projection) = **63 ms**
  (`docs/aegis/specs/2026-09-14-db-tui-flow-audit.md:141-146`).
- The 2026-09-17 run had 18,334 endpoints (`docs/aegis/plans/2026-09-17-ping-run-analysis.md`
  §1) ⇒ ~92 pages ⇒ **~5.8 s of serial work before the first socket dial**
  (`INFERENCE`, from the measured per-page cost; no end-to-end timing is
  recorded anywhere).
- The COUNT is recomputed on every page (`profiles_page` never caches it), so
  count work alone is `O(feed²/200)` — 92 COUNTs for one batch.
- `BatchShared::new` then builds three `O(feed)` maps
  (`ops/ping.rs:813-873`) and the dispatch loop calls `schedule()` once per link
  serially (`ops/ping.rs:930-947`). `states` grows to `O(feed)`
  (`ops/scheduler.rs:147-195`).
- Config hydration is **not** on this path: `load_page_projection` returns the
  three `Deferred<Json<ProtocolConfig>>` carriers unloaded
  (`profiles_query.rs:826-829`); phase 1 uses only in-memory `proto_kind`.
- **No progress reaches the UI during any of it.** For a fast+real batch
  `publish_final_total` is skipped entirely (`ops/ping.rs:911-924`), so the
  status bar reads bare `Testing...` (`ui/status_bar.rs:31-46`) through the plan
  walk *and* the whole fast phase — the 2026-09-16 run showed `0 / 34,562` for
  five minutes (plan §F5).

### 2.2 What "phase 1 fully settles first" actually holds

`FACT`, exhaustive list (scout report `agent://MapParallelPhaseCoupling` §A):

| # | assumption | anchor |
| --- | --- | --- |
| 1 | phase-2 launch is behind `pending_fast == 0` + a `JoinHandle` join | `ops/ping.rs:973-984` |
| 2 | `hard_fast` (connect-class fast failures, phase-2 skip) is final | `:1638-1645`, read at `:1009-1023`, `:1065-1073` |
| 3 | best-first order comes from a snapshot of `fast_latency` | `:1041-1049` |
| 4 | `emit_untestable_markers` runs once, after phase 1, so a fast success cannot clear the marker | `:994`, `:1584-1592` |
| 5 | one progress pair, restated to phase 2's candidate count | `:1009-1027`, `:1778-1787` |
| 6 | fast settles do not advance progress for a fast+real run | `:1700-1712` |
| 7 | `phase1_ms`/`phase2_ms` are sequential wall-clock spans | `:986-989`, `:1165-1168` |
| 8 | one global `real_concurrency` semaphore, one endpoint group at a time, links in a group fired sequentially for `dedup_endpoints` | `:1106-1140` |

Two facts make the restructure cheap:

- `run_task_chain` is kind-agnostic: after `complete` it asks `task_of`/`kind_of`
  and fires whatever the gate promoted, looping until the gate is clear
  (`ops/ping.rs:1416-1469`). Cross-kind promotion already works.
- The gate is **per `(ProtocolId, EndpointId)`** (`ops/scheduler.rs:73-76`,
  key at `:164`), so a RealPing queued behind a live FastPing on the same link is
  `Queued` and promoted by that fast task's `complete` (`:171-190`). Default
  `task_queue_limit = 3` (`crates/xray-tui-config/src/app_config.rs:574-576`), so
  one queued real per link never hits `QueueFull`.
- `emit_result` runs **before** `complete` in the chain (`:1443-1447`), so
  `hard_fast`, `fast_latency` and the staged fast result are all visible at the
  moment a promoted real task dispatches. The per-link phase decision needs no
  barrier.

### 2.3 Progress surface

- Actions Log panel: bordered `Fieldset`, `FULL_PANEL_HEIGHT = 8`
  (`ui/mod.rs:902`) ⇒ 6 inner rows (`ratatui-cheese` strips top+bottom rules);
  six single-`Span` rows painted one full-width 1-line `Paragraph` each
  (`ui/actions_log.rs:235-250`). Rows: 1 status, 2 server, 3 test results,
  4 traffic/memory ("statistics"), 5 core log, 6 TUI log. No row has columns or
  a separator.
- The only batch-progress render site is the status bar
  (`ui/status_bar.rs:28-45`), reading `AppState.batch_progress`, today
  `Arc<(AtomicU32, AtomicU32)>` = `(total, completed)` (`types.rs:570`).
- `CoreEvent::BatchProgress` is sent on every settle (`:1798-1801`) — ~34k
  channel sends per feed-wide batch, `try_send` with the error ignored. The UI
  does not need them: the render reads the shared Arc; only publishing the Arc
  and clearing it at the end require a signal.
- `ThemeStyles::progress_bar`/`progress_fill` exist and are unused
  (`ui/theme.rs:109-115`). ratatui 0.30.2 re-exports `Gauge`/`LineGauge` and
  nothing uses them. One hand-rolled text bar already exists —
  `progress_bar_line` in `ui/settings.rs:2093` (`[███>░░░] 42% (1.2MB/3.5MB)`),
  private to that module.

### 2.4 Country writes

- One writer: `Database::set_endpoint_ip_country`
  (`crates/xray-tui-db/src/database.rs:1049-1066`) — **its own connection, its own
  transaction, its own commit**, wrapped in `retry_on_busy(…, 5)`;
  `set_country` is SELECT + UPDATE|INSERT (`endpoint_ip.rs:215-250`).
- Granularity is one commit **per endpoint** (the selected address), from two
  callers in `ops/enrich.rs` (`:296` in the per-endpoint DNS task, `:419` in the
  page-seed loop).
- The import itself writes no country rows for DNS hosts — a fresh endpoint has
  no persisted addresses and is filtered out (`ops/enrich.rs:319-327`) — but
  `spawn_enrich_ip_hosts` re-runs on **every page reload**, one sequential task,
  one commit per IP-literal or previously-resolved endpoint. Those commits
  contend with the import's own 500-URL chunk transactions
  (`ops/subscriptions.rs:449`), which is what `retry_on_busy`'s 20 ms→1.28 s
  backoff absorbs (`crates/xray-tui-db/src/retry.rs:25-40`).
- `LinkWriter` cannot cover it: `LinkGroups` has no bit for `endpoint_ip`
  (`database.rs:27-34`) and `endpoint_ip` is not `profile_stats`.
- No success counter or volume log exists — only per-failure `warn!`s
  (`ops/enrich.rs:300`, `:423`), so the frequency is unobservable today.

## 3. Target state

### 3.1 S1 — streaming plan walk

The plan becomes a **page stream consumed by the dispatcher** instead of a
buffer built before dispatch.

- `BatchShared` no longer holds `plan: Vec<PlanLink>`. It holds increments:
  `plan_len: AtomicU32`, `plan_pages_done`/`plan_pages_total: AtomicU32`, and the
  per-link maps the probes read (`fast_config`, `endpoints`, `untestable`) in
  `DashMap`s (the type `TaskScheduler.states` already uses).
- `run_batch` walks pages and, for each page, schedules that page's links, then
  keeps walking. First dial lands within one page (~63 ms) instead of after the
  whole walk.
- The walk uses a new count-free id page
  (`Database::profiles_walk_page`, `crates/xray-tui-db/src/profiles_query.rs`)
  plus **one** `profiles_count` at the start for the page meter. Same owner
  module, same builders (`base_select`/`order_by`/LIMIT-OFFSET), same
  offset-paging that is measured flat; no new raw-SQL *class* (`docs/database-manual-sql.md`
  already owns this module).
- Plan-load failure mid-walk (`FACT`: today a failure aborts and emits a
  terminal event, `ops/ping.rs:888-898`): the batch keeps what it already
  dispatched, logs one `warn` naming the offset, and finishes normally; the
  summary line reports the truncated plan (`planned=N pages=P`). Rationale: the
  pages already probed carry results the user can see, and today's abort leaves
  the same rows unmeasured with a worse signal.
- `warn_if_real_phase_is_slow` moves to the walk's end (plan length known) and
  only when `real_phase`.

### 3.2 S2 — per-link fast → real pipeline

At dispatch time the batch queues the link's whole life on the gate: `FastPing`
first, then `RealPing` (only for a real-phase batch). The gate does the
sequencing; no phase barrier exists.

- Promotion path: `run_task_chain` completes the fast task, the gate promotes the
  queued real id, and the chain's real branch dispatches. `hard_fast` /
  `untestable` are consulted **there** (per link) — the same three facts the old
  phase-2 loop consulted globally, evaluated at the only moment they are knowable.
- Untestable markers are emitted per link, after that link's fast result
  (`FACT`: the old single post-phase-1 pass existed exactly to guarantee that
  ordering, `ops/ping.rs:991-994`). Behavior is unchanged for fast-only batches,
  which emit the same markers today (`:994` runs before the `!real_phase` return).
- `dedup_endpoints` becomes **best-effort** (user decision): before dispatching a
  promoted real probe, skip it (retire silently, no marker) when the endpoint is
  already in `completed_endpoints`; a success inserts the endpoint id, so later
  promotions stop. Sibling probes that are already live are not cancelled — the
  old design guaranteed at most one real probe in flight per endpoint; the new
  one allows as many as siblings whose fast results landed together, which the
  reference feed bounds at 1.74 real candidates per endpoint (§F4 of the plan).
  `retire_real` + `cancel_queued` stay for the stop path.
- Real concurrency keeps its own semaphore, acquired **in the chain** (like
  `fast_sem`), sized by `real_ping_concurrency`; fast keeps
  `fast_ping_concurrency`. Both limits are independent (user decision): peak
  in-flight probes is now their sum, which the two phases never reached.
- Dispatch order for real probes is fast-settle order. With the fast semaphore
  bound at `fast_ping_concurrency`, that order approximates ascending fast
  latency, so the old best-first property survives without a sort and without a
  barrier.
- The `per_endpoint: BTreeMap` grouping, the endpoint-group tasks and the
  sequential in-group loop are deleted.
- Completion: the batch ends when the walk is done, `pending_fast == 0`,
  `pending_real == 0`, and every DNS-deferred retry has settled. A `DnsDeferred`
  link re-enters the same per-link schedule (fast, real queued behind it) once
  the window expires, and its handle is tracked so `finish_batch` stays last.
- `phase1_ms`/`phase2_ms` become `fast_span_ms` (first fast start → last fast
  settle) and `real_span_ms` (first real start → last real settle); they overlap
  by construction and the summary says so. Total wall time is added.

### 3.3 S3 — two-level progress meters + ETA

- `types::BatchProgress` (a tuple alias) is replaced by a struct
  `types::BatchMeters`, shared as `Arc<BatchMeters>` in
  `AppState.batch_progress` — same slot, same "a batch is live" meaning, wider
  payload:

  ```rust
  pub struct PhaseMeters { total: AtomicU32, done: AtomicU32, rate_milli: AtomicU32 }
  pub struct BatchMeters {
      plan_pages_done: AtomicU32,
      plan_pages_total: AtomicU32,   // 0 until the first count lands
      fast: PhaseMeters,
      real: PhaseMeters,
  }
  ```

  `rate_milli` = results/s × 1000, sampled at most once per second inside the
  batch task (a `Mutex<(Instant, u32)>` last-sample slot per phase; the settle
  that crosses the 1 s boundary recomputes and stores). The render path computes
  `remaining = (total - done) / rate` itself; no timer, no thread.
- `fast.total` = planned links so far (grows with the walk; probing is ~10× slower
  than page loading, so it converges within seconds and `done` can never outrun
  it). `fast.done` = fast settles.
- `real.total` = candidates known so far (links whose fast settled, minus
  hard-fails and untestable); `real.done` = real settles of candidate items,
  including dedup-retired ones. The denominator grows monotonically; this is the
  structural fix for the old "numerator from one phase against another phase's
  total" rule (plan §F5) — the two bars can no longer be paired wrongly.
- `CoreEvent::BatchProgress` is **deleted** (34k sends per batch bought nothing
  the shared Arc does not publish) and replaced by `CoreEvent::BatchEnded`, which
  clears `batch_progress` + `batch` and re-arms `speed_test_stop` — the jobs
  `total == 0` does today (`ops/events.rs:792-810`). The stale-bar guard
  (`ops/events.rs:225-236`) is rewritten against `state.batch_progress.is_none()`.
- Status bar: `Testing: F {fast_done}/{fast_total} · R {real_done}/{real_total}`
  (`R` omitted for a fast-only batch). It keeps working when the panel is
  compact/overlaid, which is why it is not simply removed.

### 3.4 S4 — Actions Log two-row split, static-width cells

Rows 5 and 6 (core log / TUI log) are split horizontally; the panel stays 6 rows
and 8 lines (user decision, "split rows 5+6"):

```
row 5:  📋 Core: [info] …log text…            │ Real [████████░░░░] 1234/17051 ~12m
row 6:  📋 TUI:  [info] …log text…            │ Fast [██████░░░░░░░░] 8123/34562 ~3m
```

- Left region = the existing log line, truncated to its column budget; right
  region = the bar, fixed width; a vertical rule separates them at a fixed
  column (`inner_width - BAR_REGION_W`), so bar position never depends on the
  left content.
- `BAR_REGION_W = 44` when `inner_width >= 72`; below that the right region
  renders the numbers/ETA without bar glyphs (the panel is also used in the
  small-terminal overlay and on the Actions tab, where `chunks[1]` is
  `Constraint::Min(3)`).
- Real ping bar on row 5, fast ping on row 6. A fast-only batch puts the fast bar
  on row 5 and renders row 6 full width (user decision: "when only one progress
  bar is used, it takes the first row"). With no batch running, both rows render
  exactly as today.
- The bar itself is one shared helper hoisted out of `ui/settings.rs` into
  `ui/widgets/progress.rs` (`bar_line(label, done, total, eta, width, palette)`),
  and `settings.rs`'s private `progress_bar_line` migrates onto it — one bar
  rendering in the crate, not two. The existing
  `ThemeStyles::progress_bar`/`progress_fill` styles paint fill and track. No
  `Gauge`/`LineGauge` (they are full-rect widgets; these bars live inside a
  `Paragraph` line beside log text).
- Static-width statistic cells (user request): rows 3 and 4 pad each value to a
  fixed width (`TCP: {v:>6}ms`, `RP:`, `SPD:`, `⬆{up:>9}`, `⬇{down:>9}`,
  `💾 {mem:>7}`) so the labels stop shifting as numbers change.

### 3.5 S5 — batched country writes

- New `Database::set_endpoint_ip_countries(&[(EndpointId, IpAddr, String)])`: one
  connection, one transaction, the existing per-row typed statements inside it,
  one commit (the shape `update_endpoint_resolution` already uses,
  `database.rs:1006-1020`). No raw SQL, so no new manual-SQL site. Keeps
  `retry_on_busy(…, 5)`.
- `spawn_enrich_ip_hosts` collects the page's `(endpoint_id, ip, iso)` results and
  flushes **once per page**, then once at the end — a page of 200 endpoints
  becomes ≤2 commits instead of ≤200.
- `spawn_dns_resolve` keeps its single-row write: it fires per user gesture
  (connect, `x`, an expired-TTL real result), not per import.
- Observability: one `debug` line per seed pass naming endpoints scanned,
  countries written, and the flush count; plus a success counter so the effect is
  measurable on the next large import (today only failures log).

## 4. Invariants preserved

- One batch at a time (`state.batch_progress.is_some()` gate, `ops/ping.rs:1933`),
  one `scheduler.reset()` per batch, task ids runtime-only.
- Probes run only for ids `schedule`/`complete` hand out; a completion with a
  stale id is a no-op (`ops/scheduler.rs:206-247`).
- Result persistence stays staged through `LinkWriter` (`LinkGroups::RESULT`),
  flushed at batch end, and a phase-2 patch still composes phase 1's measurement
  back in (`ops/ping.rs:1584-1592`) — in the pipelined world the fast result is
  staged before the real probe even starts, so the composition is structural
  rather than repaired.
- `Cancelled`/retired work never writes an error marker; the untestable marker
  text and its two readers (`ops/ping.rs::is_untestable_marker`) are unchanged.
- Stop semantics: every chain retires at a dispatch boundary, `BatchEnded` lands
  last, and quit still flushes the writer then prints
  `interrupted_summary_line` from the published handle (`ui/mod.rs:246-260`).
- The error-TTL sweep still runs in `finish_batch` after the writer flush.

## 5. Contracts changed

| surface | before | after |
| --- | --- | --- |
| `types::BatchProgress` | `Arc<(AtomicU32, AtomicU32)>` (final phase only) | `Arc<BatchMeters>` (plan pages + fast + real + rates) |
| `CoreEvent::BatchProgress` | every settle | deleted |
| `CoreEvent::BatchEnded` | — | new: clear batch state, re-arm stop |
| `BatchShared.plan` | `Vec<PlanLink>` | removed; `plan_len` + `DashMap`s filled by the walk |
| `Database::profiles_page` usage in the walk | count + ids per page | `profiles_walk_page` (no count) + one `profiles_count` |
| `Database::set_endpoint_ip_country` | single row, own commit | kept for the gesture path; new `set_endpoint_ip_countries` (one tx) for the page seed |
| `ui/widgets/progress::bar_line` | private in `settings.rs` | shared widget used by settings + the panel |

## 6. Failure and recovery

- Page-walk error mid-stream: keep dispatched work, warn with the offset, finish
  the batch, report the truncation in the summary line.
- Country batch failure: `retry_on_busy` inside the call; on final failure a
  `warn` naming the endpoint count, and the page loop continues with the next
  page (today's per-row behavior, one level up).
- Rates are sampled, so a stalled phase shows a stale ETA rather than a wrong
  one; an ETA with `rate_milli == 0` renders as `--`.
- A denominator that grows can never show >100%: `done` only counts items already
  in `total`.

## 7. Acceptance

Observable, verifiable on the reference feed (18k endpoints / 34k links):

1. A new `info` line reports `plan: N pages, M links in X ms` and
   `first dial X ms after the batch start`; the first-dial value is **< 500 ms**
   on a warm DB (today's ~5.8 s is the number the line makes falsifiable).
2. A fast+real batch shows two bars in the Actions Log panel, real above fast,
   both moving while fast is still running; a fast-only batch shows one bar on the
   first row.
3. Real results appear while `fast.done < fast.total` (asserted by a pipeline test
   with a stubbed runner: a real probe is dispatched before the last fast probe
   settles).
4. `hard_fast` links and untestable links still get no real probe; untestable
   markers still survive a fast success (ordering test).
5. The channel carries O(pages + 1) `CoreEvent`s per batch instead of O(links) —
   counted in a test.
6. Country writes per seed pass = ≤2 commits for any page count (unit test on the
   batched call: one transaction, N rows).
7. `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
   `cargo nextest run --workspace` clean.

## 8. Non-goals

- No probe-policy change: timeouts, retries, classes, `ProbeClass`, the native
  engine, and the exit-IP probe are untouched.
- No plan-specific raw SQL query yet (the walk reuses the page query's math, one
  statement lighter). Escalation trigger: if the `plan:` timing line still reads
  > 1 s on the reference feed, a purpose-built plan projection becomes a
  measured follow-up — not a speculative rewrite now.
- `dedup_endpoints` stays opt-out via `real_ping_test_all_protocols`; the policy
  question from plan §F4 (retire siblings on the first *failure*) stays deferred.
- Plan §F6/P10 (persist the batch's resolved addresses into `endpoint_ip`) stays
  deferred; this spec only makes the country path cheap once it happens.
- Compact 1-line Actions bar and the small-terminal overlay keep today's content.
- No new gauge widget, no new crate, no new thread/timer.

## 9. Existence Check

| proposed surface | existing owner / reuse | decision |
| --- | --- | --- |
| `types::BatchMeters` | replaces the `BatchProgress` tuple alias in the same slot | reuse-existing (widen) |
| `CoreEvent::BatchEnded` | `BatchProgress{0,0}` terminal path did this job | reuse-existing (rename) |
| `Database::profiles_walk_page` | `profiles_query` module owns the page SQL | reuse-existing (same owner module) |
| `Database::set_endpoint_ip_countries` | `endpoint_ip.rs` + `set_endpoint_ip_country` | reuse-existing (batch variant) |
| `ui/widgets/progress::bar_line` | `settings.rs::progress_bar_line` | reuse-existing (hoist; net −1 duplicate) |

No new owner, fallback, compatibility path, dependency, or persistent artifact.
Retirement carried by the change: `per_endpoint` grouping + endpoint-group tasks,
`emit_untestable_markers`' O(feed) pass, the phase-2 candidate computation and
sort, the per-settle progress event, and the `plan: Vec<PlanLink>` buffer (tens of
MB at feed scale) all go away.

## 10. ADR signal

No new ADR. This amends existing durable records in place when it lands:
`AGENTS.md` decision 15 (batch pipeline, progress, dedup) and decision 16's
untestable-marker ordering note; `docs/database-manual-sql.md` gains the walk
method beside `profiles_page`; `docs/database.md`'s flow section mentions the
single-transaction country batch. Baseline-sync question for closeout:
"does decision 15 still describe the pipeline as two sequential phases?"

## Appendix — working artifacts

`TaskIntentDraft`: make batch ping start immediately and report itself while it
runs; add two-level progress with ETA; stop paying one commit per country.
Success evidence = the §7 items. Stop condition = both bars live + first dial
< 500 ms + ≤2 country commits per page. Non-goals = §8.

`BaselineReadSetHint`: `AGENTS.md` (decisions 4, 15, 16, 20, 21, 22),
`docs/database.md`, `docs/database-manual-sql.md`,
`docs/aegis/plans/2026-09-17-ping-run-analysis.md`,
`docs/aegis/specs/2026-09-14-db-tui-flow-audit.md`,
`docs/aegis/adr/0002-write-behind-link-writer.md`, ADR 0005.

`BaselineUsageDraft`: required = AGENTS.md decisions 15/16/22 + the manual-SQL
doc (the walk touches the page-SQL owner) + the write-behind spec (staging
contract). Missing refs: none. Decision: continue.

`ImpactStatementDraft`: layers = TUI batch pipeline, TUI render (Actions Log +
status bar), DB page query + endpoint_ip write. Owners touched: `ops/ping.rs`
(batch), `ops/events.rs` (batch events), `ui/actions_log.rs` + `ui/status_bar.rs`
(render), `profiles_query.rs` + `endpoint_ip.rs` + `database.rs` (SQL), 
`ops/enrich.rs` (geo seed). Invariants: §4. Compatibility: none outside the
process (no persisted schema, no config key, no identity change).
