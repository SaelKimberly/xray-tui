# Turso MVCC production rollout plan

## Goal

Evaluate Turso MVCC for fresh or intentionally recreated embedded file databases as an opt-in mode to remove WAL writer starvation during concurrent import, link-writer flushes, DNS resolution, and geo-country persistence. Existing WAL files remain WAL; Turso 0.7.2 does not support safe in-place WAL→MVCC conversion.

## Architecture

`toasty-driver-turso 0.10.0` exposes `Turso::file(path).concurrent_writes()`. The option is a local embedded-driver setting: each connection selects `journal_mode='mvcc'`, and Toasty's default `TransactionMode::Default` becomes `BEGIN CONCURRENT`. It is not a remote-server capability and not a crate feature.

`Database::open` selects mode from the existing file header: fresh/recreated files use MVCC only when `XRAY_TUI_TURSO_CONCURRENT_WRITES=1`; otherwise they use WAL. WAL-header files use WAL; MVCC-header files retain MVCC. `Database::in_memory()` remains WAL because its schema/DDL test lifecycle is not an MVCC workload. `Database::conn()` remains the per-connection owner for `busy_timeout=5000` and `synchronous=NORMAL`.

Conflict safety belongs at transaction owners. `retry_on_busy` recognizes both WAL busy errors and MVCC `BusySnapshot`/commit conflicts because the driver maps them to Toasty serialization failures. The write-behind writer re-stages failed windows; all public database mutators now retry from a fresh operation.

## Baseline and evidence

- Locked backend: embedded `turso 0.7.2`, `toasty-driver-turso 0.10.0`.
- `turso_core 0.7.2` defines `SyncMode::Normal = 1` and parses `NORMAL`/`1`; current public Turso docs advertising only `0|2` describe a different/newer contract and are not evidence about this lockfile.
- Corrected release synthetic probe with per-connection `NORMAL=1`: WAL competing writer reached 5-second busy timeout in 5/5 samples; MVCC competing 32-row writer completed in 0.170–0.197 ms in 5/5 samples with no conflict.
- Probe used a synthetic `hits` table, not the production import/geo mix. It proves engine semantics, not application throughput.
- Existing old MVCC evidence: reader p50 56.5→85.9 µs and p95 66.8→97.0 µs; `wal_checkpoint(PASSIVE)` failed under MVCC without the passive-checkpoint flag. Current batch completion skips that checkpoint for MVCC handles.
- A WAL→MVCC conversion probe failed: `PRAGMA journal_mode='mvcc'` returned success on a WAL-header file but left the header WAL and created a logical-log sidecar; the next MVCC open rejected the file. No in-place migration is shipped.

## Compatibility boundary

- No schema/tag change. MVCC is runtime journal/transaction behavior.
- Existing WAL files remain readable and remain WAL; do not delete sidecars during normal preflight.
- Intentional schema wipe/recovery removes the main file and all `-wal`, `-shm`, and `-log` sidecars before recreating a fresh MVCC file.
- `PRAGMA synchronous=NORMAL` remains per-connection and unchanged.
- `XRAY_TUI_TURSO_CONCURRENT_WRITES=1` opts fresh/recreated files into MVCC; unset or `0` keeps fresh files on WAL. The switch does not downgrade an existing MVCC file.
- MVCC is process-local in the pinned engine. This application owns one embedded database process.

## Change Necessity

Code change required. A driver-construction flag cannot solve the reported long waits by itself because MVCC conflicts require whole-transaction rollback/retry. The minimum safe boundary is header-aware driver selection plus retry behavior at every production transaction owner. The implementation also needs DDL-specific handling: rank schema setup uses `TransactionMode::Immediate`, while normal writes use default concurrent transactions.

## TDD route

`off`; user requested implementation, not strict RED/GREEN. Focused regression tests and full suites verify the behavior. Conflict behavior is timing-sensitive, so the deterministic test asserts a real MVCC write conflict and fresh-transaction replay.

## Tasks

1. **Driver switch — complete.** Header-aware file selection; opt-in fresh/recreated MVCC; existing WAL compatibility; in-memory WAL; schema-wipe sidecar cleanup.
2. **Retry boundary — complete.** Public mutators and rank maintenance retry from fresh state; import retains its existing retry wrapper; link writer re-stages failed windows.
3. **Regression coverage — complete.** Fresh MVCC readback, existing WAL compatibility, schema-wipe cleanup, real MVCC conflict replay, in-memory lifecycle, checkpoint guard.
4. **Workload evidence — partial.** A disposable DB-API import+geo-style probe ran five independent repetitions per arm. WAL medians: total 1089.214 ms, import p50 120.349 ms, geo p50 2.372 ms. MVCC medians: total 1437.411 ms, import p50 176.979 ms, geo p50 3.141 ms. MVCC was ~32–47% slower in this workload; no lock-wait benefit appeared. A real-feed production-shaped benchmark remains open.
5. **Report — partial.** `docs/aegis/specs/2026-09-24-turso-mvcc-rollout-design.md` records engine evidence, compatibility boundary, and the negative A/B result. Final default decision remains conditional on a contention-heavy real-feed benchmark.

## Verification

- `cargo check -p xray-tui-db`
- `cargo check -p xray-tui`
- `cargo test -p xray-tui-db` — 123 passed
- `cargo test -p xray-tui --lib` — 229 passed, 4 ignored
- `cargo test --workspace` — 2090 passed, 15 ignored
- `cargo fmt --all -- --check`
- `git diff --check`
- no claim that MVCC is worth default enablement until a contention-heavy production benchmark proves otherwise

## Risks and retirement
- MVCC solves a real WAL wait for disjoint writers, but the measured import+geo-style workload pays a material tax.
- Passive checkpoint incompatibility is handled by skipping the WAL checkpoint for MVCC handles; no nonexistent driver flag is called.
- MVCC reader overhead may affect reload latency.
- Default fresh-file mode remains WAL after the negative A/B. Keep MVCC opt-in/recreated-only until a real contention-heavy benchmark proves its benefit exceeds the tax; otherwise retire the mode rather than adding another fallback layer.
