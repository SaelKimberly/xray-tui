# Database Query Monitoring — Design Spec

Status: proposed (awaiting user review → writing-plans)
Date: 2026-09-24
Surface: `xray-tui-db` (`Database` methods, `retry_on_busy`), `xray-tui` (`main.rs` `LogVisitor` + new aggregating tracing `Layer`), Logs tab dump.

## 1. Problem

No visibility into which Turso/toasty queries are slow or contended. Two symptoms:

1. Bottleneck queries (the `.include()` page-load class, 7–13 s — `docs/database.md`) are only found by manual profiling.
2. toasty already emits a `WARN "slow query"` event, but the TUI Logs panel renders it as the **bare string "slow query"** — no statement, no duration. Reproduces after subscription updates (two subs). Root cause: `main.rs` `LogVisitor` (lines 28-39) captures only the `message` field and discards the structured fields toasty puts the data in.

Goal: trace every query (or its normalized *kind*), its duration, retry count, and success/failure, and surface the top offenders on demand.

## 2. Grounding — toasty's built-in facility

toasty 0.10 ships per-query tracing (`toasty-core/src/driver/log.rs`), called by the turso driver (`toasty-driver-turso/src/lib.rs:769-778`):

- One `tracing` event **per physical DB operation**, target `toasty::query`.
- Fields: `duration_ms`, `rows`, `error`, `db.system`, `db.statement` (full SQL text), `db.operation`, `db.collection`, `db.params` (opt-in).
- Level `DEBUG`, escalated to `WARN` past `slow_statement_threshold` (default `1s`). Message: `query executed` / `slow query` / `query failed`.
- Configurable on `Db::builder()`: `.slow_statement_threshold(Option<Duration>)`, `.log_statement_params(bool)`.

**Consequence:** statement text / duration / rows / outcome are free from `toasty::query`. **Retry count is NOT** — retry lives entirely in our `retry_on_busy` wrapper; a retried statement re-executes, so toasty emits N uncorrelated events. Retry count can only come from our own method seam.

## 3. Requirement coverage

| Requirement | Source | Notes |
|---|---|---|
| every query / kind | `toasty::query` `db.statement` | statement-level |
| start time / duration | `toasty::query` `duration_ms` | — |
| success/failure | `toasty::query` `error` + message | — |
| retry count | our `Database` method span + `retry_on_busy` | final only on method return |

## 4. Decisions (from grilling interview)

- **Seam: method-level (A1).** Wrap the ~40 public `Database` methods; each opens a span carrying the method name. `toasty::query` events firing inside are attributed to that method via current-span lookup.
- **Aggregation key: `Database` method name (A1).** A stable, curated ~40-key space no literal touches. Statement text is an attached **sample**, never the key.
  - **Rationale (rejected: normalized-statement key):** the hot paths (`profiles_query.rs` page hydration, `endpoint_rank.rs`, `endpoint_ip.rs`) inline ids as **integer literals + hex blobs**, not `?N` binds (`docs/database-manual-sql.md`). Keying on statement text would explode into near-unique keys; literal-stripping normalization is fragile (over-strip merges distinct queries, under-strip explodes). Method-name key avoids this entirely.
  - Bare PRAGMA / raw sites with no enclosing named method fall into an `"unattributed"` bucket (acceptable).
- **Surface: on-demand dump (B).** A key/command dumps the ranked registry to the Logs tab; also flushed at quit. No new render surface in v1; the registry is reusable by a live panel later.
- **Lifecycle: in-process, session-only.** Registry is in-memory. NOT persisted to the Turso DB (writing monitoring rows into the contended DB is self-defeating) nor to heed in v1.

## 5. Mechanism — two-source aggregating Layer

The aggregating `tracing::Layer` is **two-source**, not a single `on_event` fold (pinned per advisory):

- **`on_event`** (target `toasty::query`): read the enclosing `Database`-method span from current context; fold this statement's `duration_ms` / `rows` / `error` / `db.statement` into that method's bucket (count, total/p50/p99 duration, fail count, slowest-statement sample). Fires mid-flight, per physical attempt.
- **`on_new_span` / `on_close`** (the `Database`-method span): retry count is only final when `retry_on_busy` returns, so it is recorded as a **span field at method close** and folded in `on_close` (method-level invocation count + retry rollup). A single `on_event` fold cannot see the final retry count.

Span-field carrier: each wrapped method records `db_method = <name>` and, at completion, `retries = <n>` (0 when the method does not go through `retry_on_busy`).

## 6. Fix — informative slow-query line (change owner: `main.rs` `LogVisitor`)

`LogVisitor` is extended to capture the `toasty::query` structured fields (`duration_ms`, `db.statement`) so the panel line becomes e.g. `slow query [312.4ms]: SELECT … FROM profile_stats …` instead of the bare `slow query`. This fixes the reported subscription-update symptom immediately, independent of the aggregating Layer. (The `DEBUG` per-query events stay filtered out of the heed store at `main.rs:60`; only the WARN slow line is enriched for the panel.)

## 7. Components / ownership

- `xray-tui-db`: method-name span instrumentation on `Database` public methods; `retry_on_busy` records the final `retries` field. `Db::builder()` wiring for `slow_statement_threshold` (configurable) at `open()`.
- `xray-tui` new module (e.g. `ops/db_monitor.rs`): the aggregating Layer + in-memory registry + ranked dump formatter.
- `xray-tui` `main.rs`: register the Layer in the subscriber stack; extend `LogVisitor`.
- Logs tab / key handler: on-demand dump command; quit-time flush.

## 8. Acceptance (observable)

1. A key/command prints top-N `Database` methods ranked by p99, each row: call count, p50/p99 duration, retry count, fail count, slowest-statement sample.
2. **Falsifier A (regression class):** reproduce/simulate the `.include()` page-load class and confirm it surfaces as the top method with its statement sample.
3. **Falsifier B (reported symptom):** after a subscription update, the Logs panel slow-query line shows the statement + `duration_ms`, not the bare `slow query`.

## 9. Risks

- Span context must propagate across toasty's async driver `.await` points for `on_event` attribution to find the method span — **verify during implementation** (instrument-across-await; a dropped span → `unattributed`).
- Aggregating Layer must bound its subscription (level/target cap) so `DEBUG` `toasty::query` volume on the hot path does not cost render/alloc.
- Registry keyed by ~40 methods is bounded; the `unattributed` bucket must not accumulate distinct statement text as keys (store only aggregate + one sample there too).

## 10. Non-goals (v1)

- Live continuously-refreshing TUI panel (registry reusable later).
- Persisting monitoring data to Turso or heed.
- Normalized-statement-text keying.
- Instrumenting non-`Database` SQL (route/sniff paths do not touch Turso).

## 11. Amendment — two-destination field capture, and `.log_statement_params` (2026-10-02)

§6 and `plans/2026-09-24-db-query-monitoring.md` Task 5 pinned `LogVisitor` to capture **only**
`duration_ms` + `db.statement`, and that pinning test rejected anything wider. That was correct when
written, and it was too narrow: every other field hit `_ => {}` and was lost. The 2026-10-01 run
left **12 stuck-deferral warnings in heed naming no row**, because `ops/ping.rs` emits
`endpoint_id`, `half` and `waited_ms` on that line and the visitor discarded them.

**What changed.** `LogVisitor` now renders **two strings with two destinations**:

| | contents | destination |
| --- | --- | --- |
| `into_message` / `split().0` | message + `duration_ms` + `db.statement` | **`LogMessage` → heed** (persisted) |
| `split().1` (`detail`) | **every** other field, including `db.params` | `CoreEvent::TuiLog` → the Logs panel (**session only**) |

`.log_statement_params(true)` is now wired on the ONE production `Db::builder()` site —
`try_open_db` (`database.rs`). `in_memory` is deliberately NOT wired: it backs the test
suites, where param formatting on every statement is noise and costs time, not signal. It was
named in §3 as opt-in and was referenced **nowhere in the tree**, so the `OFFSET` bind of a paged
query was invisible — which is why the `profiles_page` p99 could not be attributed to a shape or an
offset from the logs.

**The split is a SECURITY boundary, not a convenience.** toasty's own docstring on the knob warns
that parameter values "are application data and may contain secrets (password hashes, personal
information)" and to enable it "only when the log destination is trusted". One of our binds is
`protocols.config` — the parsed protocol config, carrying UUIDs and passwords. heed is not a trusted
destination. So `db.params` **must never enter `LogMessage`**, which is what §10's "persisting
monitoring data to Turso or heed" forbids. The Logs panel *is* the trusted destination, so the bind
values are rendered there and nowhere else. Non-goal 10 is **strengthened, not relaxed**: it now
constrains the capture side too, not just the registry.

**Amended in place:** §6's capture list and the Task 5 pinning test (replaced by
`structured_fields_split_between_persisted_and_session_only`, which asserts the persisted line does
NOT contain the bind value and the session detail does).
