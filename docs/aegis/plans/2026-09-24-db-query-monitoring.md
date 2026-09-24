# Implementation Plan — Database Query Monitoring

Date: 2026-09-24
Spec: `docs/aegis/specs/2026-09-24-db-query-monitoring-design.md` (approved)
Execution context: current branch/workspace.

## Goal

Trace every Turso/toasty query (or its *kind*), duration, retry count, success/failure; surface top offenders on demand in the Logs tab; and make the existing "slow query" panel line informative.

## Architecture

- **Query metrics source**: toasty's built-in `toasty::query` tracing events (statement, `duration_ms`, `rows`, `error`) — free, no driver fork.
- **Retry source**: our `retry_on_busy` + `Database`-method span (`retries` field, final at method close).
- **Aggregation**: an in-process `tracing::Layer`, **two-source** — `on_event` folds per-statement query metrics into the enclosing `Database` method's bucket (via current-span lookup); `on_close` folds the method's final retry count + invocation rollup.
- **Key**: `Database` method name (~40, curated, literal-free). Statement text = attached sample only. Unattributed statements → `"unattributed"` bucket (aggregate + one sample, never keyed by text).
- **Surface**: on-demand dump to Logs tab + quit-time flush. In-memory, session-only.

## Tech stack

`tracing` / `tracing-subscriber` `Layer`, `tokio::tracing::instrument`-style spans, existing `TuiLogLayer` subscriber stack in `main.rs`.

## Baseline / authority refs

- `docs/aegis/specs/2026-09-24-db-query-monitoring-design.md`
- `docs/database-manual-sql.md` (inlined-literal hot paths → why method-name key)
- `docs/database.md` (`.include()` regression class — falsifier A)
- toasty-core `driver/log.rs`, toasty `db/builder.rs` (facility + config knobs)

## Compatibility boundary

- No schema change, no DB write, no persisted state (§4 lifecycle). Nothing touches `PRAGMA user_version`.
- `LogVisitor` change is additive (still captures `message`; adds field capture). No log-format break for non-`toasty::query` events.
- `Database` public method signatures unchanged — instrumentation is internal (span entry only).

## Change Necessity

Code required: the informative slow-query line needs `LogVisitor` to read structured fields it currently discards (`main.rs:28-39`); aggregation needs a new Layer + registry. No config/docs-only path fixes the field-drop. Minimum boundary: one new module + additive edits to `main.rs`, `database.rs`, `retry.rs`.

## Ripple Signal Triage

Fires: shared/core (`xray-tui-db` `Database` methods), producer/consumer (Layer consumes toasty + our span events). Canonical owner: new `ops/db_monitor.rs` for the registry; `Database` remains owner of the method seam. Risk: span context must cross toasty's async `.await` boundaries for attribution (carried into Task 4 verification). No new contract/fallback; no second owner retained.

## Existence Check

- New surface: `ops/db_monitor.rs` (Layer + registry + dump formatter).
- Reuse candidate: none — no monitoring/aggregation owner exists; `TuiLogLayer` is a routing layer, not an aggregator.
- Decision: `add-with-proof`. Retirement trigger: fold into a live stats panel if/when one is built (registry reused, module retired as standalone).

## TDD Route

- Mode `auto`; decision `strict` for pure units (registry fold math: p50/p99/count/fail/retry; `LogVisitor` field capture), `skipped` for subscriber/span wiring (no isolated check — proven by falsifiers A/B).
- Authority: auto risk-based (shared-core + reported-bug signal). Not an explicit user TDD request; no strict RED/GREEN prescribed beyond the pure units.

## Tasks

### Task 1 — Registry + fold math (`xray-tui/src/ops/db_monitor.rs`, new)
- Define `MethodStat { count, total_ms, durations (for p50/p99), fail_count, retries, slowest: Option<(f64, String)> }` and a `Registry` (method-name → stat, plus `"unattributed"`).
- `fold_query(method, duration_ms, is_err, statement)` and `fold_close(method, retries)`.
- Ranked dump formatter: top-N by p99, one line per method with count / p50 / p99 / retries / fail / slowest-statement sample.
- **TDD strict**: unit tests for percentile math, fail counting, slowest-sample retention, unattributed bucket.
- Verify: `cargo test -p xray-tui db_monitor`.

### Task 2 — Aggregating Layer (`ops/db_monitor.rs`)
- `impl tracing_subscriber::Layer` two-source:
  - `on_event` filtered to `target == "toasty::query"`: read `duration_ms`/`error`/`db.statement` via a field visitor; resolve enclosing `db_method` span via `ctx.event_span()`/scope; `fold_query`. Unattributed when no span.
  - `on_close`: read the span's `db_method` + recorded `retries` extension; `fold_close`.
- **Always-on** (corrected 2026-09-24): the `EnvFilter("off,toasty::query=debug,db_method=trace")` scopes WHICH targets reach the layer — it is not an off-state gate. Collection runs on every normal run because the goal is bottleneck data ready the instant `d` is pressed. Cost is a brief per-statement lock + bounded memory (`SAMPLE_CAP=4096` ⇒ ~1.3 MiB across ~40 methods). `toasty::query` fires per physical STATEMENT (batch upserts are one multi-row statement), so the event rate tracks SQLite throughput, not row count. Escalation: replace the ring with an atomic log-scale histogram (lock-free) if the per-statement lock shows in a profile.
- Verify: covered by falsifier A (Task 6).

### Task 3 — `Database` method spans + retry field (`xray-tui-db/src/database.rs`, `retry.rs`)
- Add `#[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]` to every public `Database` method (the `target = "db_method"` literal is REQUIRED — it matches the Layer's `on_new_span` gate; the attribute takes a string literal so it cannot reference the `METHOD_TARGET` const, keep them in sync). `Database` methods span `database.rs`, `profiles_query.rs`, `endpoint_rank.rs` — instrument ALL of them (the runtime page path is `profiles_page`/`load_page_projection` in `profiles_query.rs`, not `load_page_rows`).
- `retry_on_busy`: on return, record final attempt count into the current span's `retries` field.
- **Repair note**: methods NOT going through `retry_on_busy` record `retries=0`.
- Verify: `cargo build -p xray-tui-db`; a probe test asserting a span carries `retries` after a simulated busy-retry.

### Task 4 — Span-across-await verification (risk gate)
- Confirm the `db_method` span is entered across toasty's async driver `.await` so `on_event` attribution resolves (not `unattributed`). Use `#[instrument]` (enters across await) rather than a manual `enter()` guard.
- Verify: a test that runs one instrumented `Database` read under a capturing subscriber and asserts the `toasty::query` event resolved to the method bucket, not `unattributed`.

### Task 5 — `LogVisitor` enrichment + Layer registration (`xray-tui/src/main.rs`)
- Extend `LogVisitor` to capture `duration_ms` + `db.statement` (record_debug/record_f64/record_str for those field names) so the WARN "slow query" panel line renders `slow query [312.4ms]: <statement>`.
- Register the aggregating Layer in the subscriber stack.
- Wire `Db::builder().slow_statement_threshold(...)` at `Database::open()` (configurable; keep default 1s).
- Verify: build; falsifier B.

### Task 6 — Dump command (`xray-tui/src/ui/logs.rs` + `ui/mod.rs`)
- Add a key (whitelist in `mod.rs` per the Logs-key gate) that dumps the ranked registry into the Logs tab; also flush at quit.
- **Falsifier A**: simulate/repro the `.include()`-class slow read; confirm it ranks top with its statement sample.
- **Falsifier B**: trigger a subscription update; confirm the panel slow-query line shows statement + duration.
- Verify: launch TUI (`tui-test`), run the dump key, assert panel content; run the two falsifiers.

## Verification (whole plan)

- `cargo test -p xray-tui -p xray-tui-db`
- `cargo clippy` (workspace lints)
- `cargo build --release`
- Falsifier A (regression class ranks top) + Falsifier B (informative slow line) exercised in the running TUI.

## Risks

- Span-across-await attribution (Task 4 is the explicit gate; `#[instrument]` is the mitigation).
- Always-on collection cost — per-statement lock + bounded ring memory (Task 2); accepted for the "data ready on demand" goal, histogram escalation noted.
- `unattributed` bucket must aggregate, never key by statement text (Task 1).

## Retirement

- No old path retired (net-new observability). `db_monitor.rs` retires into a live stats panel if one is later built (Existence Check trigger).

## Execution route

`inline` — tasks are sequential and share the `db_monitor` owner + `main.rs` wiring; coordination cost of subagents does not pay off. `User confirmation required: no`.
