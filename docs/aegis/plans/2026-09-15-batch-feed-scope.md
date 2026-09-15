# Batch scope: the feed, not the viewport — implementation plan

Goal: the "all profiles" batch entry points test the whole database (every
endpoint × protocol link), not the loaded 200-row page; links imported after the
plan is built are the only ones a run may miss. While in the same code: a
phase-2 patch must not erase the measurement phase 1 took.

Evidence:
- 2026-09-15 11:50 run (507 log lines, `logs.lmdb`): `batch summary: links=272`
  against a database holding **4,523 links / 3,346 endpoints** — 6% of the feed.
  The plan came from `plan_all_visible` = `state.filtered_profiles()` = the
  200-row page.
- The same run's DB state shows the composition bug: 221 fast successes, 218
  real failures, and only 58 rows carrying both a delay and a marker. The
  batch's real-failure patch is built from the plan-time snapshot
  (`latency = None`), and the RESULT group writes latency and error together —
  so it wipes the delay the same batch just measured. (Before the batch-side
  staging of the previous workstream, the events handler masked this by staging
  from its live page row.)

Architecture: `ops/ping.rs` (plan source + resolution) → `xray-tui-db`
(`profiles_page` + `load_page_projection`, the tab's own read path) →
`LinkWriter` (RESULT patches) → `profile_stats`.

Baseline / authority refs: decision 15 (batch pipeline), decision 22
(write-behind + group ownership), ADR 0001 (the page query is the tab's read
path), ADR 0002 amendment 3 (the batch stages its own results), the previous
plan `2026-09-15-ping-pipeline-integrity.md`.

Compatibility boundary: no schema change, no uid change, no change to the tier
law or the label precedence. New behaviour: the two "all profiles" menu items
cover the feed; `batch_progress` widens to `u32` (a feed can exceed 65,535
links); the selected-endpoint entry points are unchanged.

TDD Route:
- Mode: off
- Decision: skipped
- Strict authority: not applicable
- Strict signals: none recorded
- Light eligibility: n/a
- Test posture: post-change regression (both defects get the test that fails
  before and passes after)
- Reason: project default is off; no explicit request
- Verification: focused `cargo test -p xray-tui --lib ops::ping`, then
  `just quality-gate code`

Change Necessity:
- User-visible need: "all profiles" must test the database the user imported,
  not the rows a filtered/paged tab happens to show; a delay must survive the
  real-ping verdict for the same link.
- No-change / non-code option: none — the plan source and the patch composition
  are code.
- Why code change is necessary: the scope is chosen in `start_batch`, and the
  patch is composed in the batch.
- Minimum change boundary: `crates/xray-tui/src/ops/ping.rs`,
  `crates/xray-tui/src/{types,state}.rs` (progress width),
  `crates/xray-tui/src/ui/mod.rs` (menu labels),
  `crates/xray-tui/src/ops/events.rs` (progress width), docs.
- Decision: code-change

Existence Check:
- Proposed new surface: `PlanSource` (where a plan comes from), `load_feed_plan`
  (the page walk), `BatchProgress` (a named `Arc<(AtomicU32, AtomicU32)>`).
- Existing owner / reuse candidate: `profiles_page` + `load_page_projection`
  already own "read the tab's rows in display order";
  `plan_row_links` already owns "endpoint row → PlanLink".
- Why existing surface is insufficient: `plan_all_visible` only reads the
  in-memory page, and the progress pair was `u16` (a 39k-link feed saturates).
- Creation proof: no new dependency; `load_feed_plan` is a loop over two
  existing calls, and the plan source is a two-variant enum consumed by
  `run_batch`.
- Entropy / retirement impact: net negative — the page-scoped "all" semantics
  (a bug, not a feature) is gone, and the width fix removes a saturation trap.
- Decision: add-with-proof

Plan-Time Complexity Check:
- Target files: `ops/ping.rs` (≈2.2k lines) — already over budget, and this
  change adds no new responsibility to it: the plan source is one enum, the
  loader is one function beside the existing planners, and the composition is
  three lines inside `stage_result`.
- Pre-Edit Owner-Fit Decision: `local-fix-without-new-responsibility` →
  edit-in-place; no extraction task.

## Tasks

### T1 — a phase-2 patch keeps the phase-1 measurement

`BatchShared.fast_latency: Mutex<HashMap<(ProtocolId, EndpointId), i32>>`,
recorded for every `TcpPing` success and composed into the row `stage_result`
builds (before `apply_test_result`, so a real *success* still overwrites it).
Doc the rule: the RESULT group writes `latency` and `error` together, so a patch
built from the plan-time snapshot must have the batch's own measurement composed
back in.

Verification: `a_real_failure_keeps_the_fast_measurement` (latency `Fast { 10 }`
+ `error.kind == Real` on the persisted row; measured `latency: None` before the
fix).

### T2 — the "all profiles" entry points plan the feed

1. `PlanSource { Feed, Links(Vec<PlanLink>) }`; `BatchParams.plan` becomes a
   `PlanSource` and `run_batch` resolves it (setting `progress.0` and emitting
   the loading-time event) before `BatchShared::new(params, plan)`.
2. `load_feed_plan(db, page_size)`: walk `profiles_page` (`PurgatoryView::All`,
   no search/group) page by page and hydrate with `load_page_projection`, until
   `meta.total` is covered. Logs `planned N link(s) over M endpoint(s)`.
   `page_size` is a parameter so the multi-page walk is testable.
3. Entry points: `start_batch_ping` / `start_batch_then_real_ping` →
   `PlanSource::Feed`; the selected-endpoint ones → `PlanSource::Links(...)`.
   The empty-plan early return stays for explicit plans only (a feed plan is
   unknown until loaded; `run_batch` logs "no links to test").
4. Menu labels: "(All Visible)" → "(All Profiles)"; `TUI_MANUAL.md` states that
   the entry points cover the database, not the view.

Verification: `the_feed_plan_covers_every_page_of_links` (a 5-endpoint DB with
`page_size = 2` — three pages, one partial), `a_feed_batch_probes_links_outside_the_loaded_page`
(empty page + `PlanSource::Feed` → all 3 links probed and persisted).

### T3 — progress and counters widen to u32

`BatchProgress = Arc<(AtomicU32, AtomicU32)>` in `types.rs`; `CoreEvent::BatchProgress`
and every counter/`done` in `ops/ping.rs` become `u32`. `u16` saturates at
65,535, which a feed-wide plan reaches.

Verification: the existing progress tests (`progress_events_track_total_and_done`,
`batch_progress_event_updates_shared_pair_and_clears_on_zero`) pass at the new
width; `cargo clippy` stays clean (no `u16` truncation lints).

### T4 — the startup envelope reports the loaded page

`page=0 rows` was logged before the first page landed. The envelope is emitted
from the first loop tick where `state.endpoints` is non-empty.

Verification: manual read-back of one startup (the line must carry the real page
size).

## Docs

`AGENTS.md` decision 15: the plan source (feed vs explicit), the loader, the
fast-latency composition rule, the u32 progress. Plan + INDEX entry.

## Risks / retirement

- A feed-wide plan holds one `PlanLink` per link (ProfileStats + Endpoint +
  unloaded Protocol): ~650 B/link, so ~25 MB at 39k links. Accepted; the plan is
  built in the batch task, off the UI thread, and dropped at batch end.
- Links imported *after* the plan load are not tested — the accepted boundary.
- The page-scoped "all visible" semantics is retired outright (no flag): the
  selected-endpoint entry points are the narrow scope.
