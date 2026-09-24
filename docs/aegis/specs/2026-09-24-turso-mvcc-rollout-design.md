# Turso MVCC rollout — evidence and compatibility boundary

**Status:** implemented as an opt-in mode for fresh/recreated embedded files; existing WAL files remain WAL. Engine contention benefit is real; measured DB-API import+geo workload shows a material MVCC tax.

## Decision

Keep WAL as the default. Offer MVCC as an opt-in mode for fresh or intentionally recreated embedded database files through `XRAY_TUI_TURSO_CONCURRENT_WRITES=1`. Keep an existing WAL-header database on WAL. Turso 0.7.2 does not safely convert an existing WAL-header file to MVCC in place.

This is a compatibility split, not a silent fallback:

- New file with opt-in: `Turso::file(path).concurrent_writes()`, `journal_mode=mvcc`, default Toasty transactions use `BEGIN CONCURRENT`.
- New file without opt-in: normal `Turso::file(path)`, `journal_mode=wal`.
- Existing WAL header (`read_version=2`): normal `Turso::file(path)`, `journal_mode=wal`.
- Existing MVCC header (`read_version=255`): retain MVCC and concurrent transactions.
- Intentional schema wipe/recovery: remove the main file and all `-wal`, `-shm`, `-log` sidecars before recreating; opt-in controls the recreated mode.
- `XRAY_TUI_TURSO_CONCURRENT_WRITES=0` or unset keeps fresh files on WAL; it does not downgrade an existing MVCC file.

The mode is stored on `Database`; checkpoint code reads `Database::uses_concurrent_writes()`, not process environment state.

## Locked backend semantics

Locked versions:

- `turso 0.7.2`
- `toasty-driver-turso 0.10.0`

`Turso::concurrent_writes()` is a local embedded-driver option. The driver sets `PRAGMA journal_mode='mvcc'` on connections and maps Toasty `TransactionMode::Default` to `BEGIN CONCURRENT`. It is not a crate feature and not a hosted/remote capability.

The driver maps `Busy`, `BusySnapshot`, and conflict-text errors to Toasty serialization failures. The same-row conflict probe showed the conflict can surface at the second write statement, before commit; therefore retry wraps the whole transaction body.

`PRAGMA synchronous=NORMAL` is supported by the locked engine:

- `SyncMode::Normal = 1` in `turso_core`.
- Translator accepts `NORMAL` and `1`.
- Per-connection readback returned `1` in the corrected probe.
- Current public Turso docs advertise only `0|2`; that is a version/documentation drift, not the pinned 0.7.2 contract.

## Engine-level contention evidence

Temporary release probe, same synthetic `hits` table, same 32-row competing writer, `busy_timeout=5000`, and `synchronous=NORMAL=1` on every connection:

| Arm | Result |
|---|---|
| WAL | competing writer hit 5,000 ms busy timeout in 5/5 samples; 0 successful competing commits |
| MVCC | competing writer completed 0.170–0.197 ms in 5/5 samples; 0 conflicts |

This establishes the mechanism: MVCC removes the observed WAL writer wait for disjoint concurrent writes. It is not an import/geo throughput result. The old reader measurements remain relevant regression evidence: reader p50 56.5→85.9 µs and p95 66.8→97.0 µs under MVCC.

## DB-API import+geo workload A/B

Disposable probe used actual `Database` write APIs: eight 800-link import transactions (`upsert_endpoints_bulk` + `upsert_protocols_bulk` + `upsert_links_bulk`) concurrent with sixteen 100-row `set_endpoint_ip_countries` flushes. Five independent repetitions per arm, same binary and workload; no failures. `retry_on_busy` count was not instrumented, so this table does not claim retry counts.

| Metric | WAL median (range) | MVCC median (range) | MVCC tax |
|---|---:|---:|---:|
| Total wall time | 1089.214 ms (1030.810–1089.492) | 1437.411 ms (1412.074–1506.143) | +32.0% |
| Import p50 | 120.349 ms (117.662–124.687) | 176.979 ms (172.766–181.403) | +47.0% |
| Import p95 | 126.780 ms (126.366–140.607) | 185.338 ms (179.359–205.014) | +46.2% |
| Geo flush p50 | 2.372 ms (2.348–2.551) | 3.141 ms (2.982–3.297) | +32.4% |
| Geo flush p95 | 3.047 ms (2.824–3.634) | 4.639 ms (3.271–5.906) | +52.2% |

This is a production-shaped DB-API workload, not a real-feed result. It did not reproduce the reported long wait: WAL completed every window and geo flush. Therefore MVCC's contention mechanism is real, but its measured cost is not negligible in this mix. Default enablement is not justified by current evidence.

## Retry and checkpoint coverage

All public `Database` mutators now retry busy/serialization failures from a fresh operation: endpoint/protocol/link upserts, group writes, activity/override writes, link patches, resolution/country writes, purge/delete/group/reset/restore operations, and rank repair/refresh/reband operations. Import batch transactions retain their existing retry wrapper. `LinkWriter` still re-stages failed windows and every transaction retries from fresh state.

Rank DDL is intentionally opened with `TransactionMode::Immediate`; MVCC's default `BEGIN CONCURRENT` is reserved for normal writes. This preserves the existing additive `band`/`rank_host` schema setup and avoids treating DDL as a concurrent data transaction.

`finish_batch` skips `PRAGMA wal_checkpoint(PASSIVE)` for MVCC handles. The pinned driver exposes no passive-checkpoint builder flag, and the prior probe showed that checkpoint call failing under MVCC. WAL handles retain the bounded passive checkpoint.

## Verification completed

- `cargo check -p xray-tui-db`
- `cargo check -p xray-tui`
- `cargo test -p xray-tui-db` — 123 passed
- `cargo test -p xray-tui --lib` — 229 passed, 4 ignored
- `cargo test --workspace` — 2090 passed, 15 ignored
- `cargo fmt --all -- --check`
- `git diff --check`
- focused tests: fresh file MVCC readback, existing WAL compatibility, schema-wipe sidecar cleanup, MVCC conflict replay, in-memory WAL lifecycle, checkpoint guard

## Unverified production claim

No real-feed import+geo A/B has run. The disposable DB-API probe above is evidence against default enablement for its synthetic mix, not evidence about a real 7.6k/50k/200k feed. The next benchmark must use a database copy and report total wall time, per-window p50/p95/p99 wait, conflict/retry/failure counts, throughput, reader/reload latency, and checkpoint viability.

## Rollout rule

Do not convert an existing WAL database in place. To move an existing feed to MVCC, perform the project's existing explicit destructive schema reset/reimport path, then open the recreated file with `XRAY_TUI_TURSO_CONCURRENT_WRITES=1`. Keep WAL default unless a contention-heavy real-feed benchmark shows that avoided waits outweigh the measured import/geo tax.
