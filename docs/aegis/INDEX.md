# Index

| Date | Kind | Path | Title | Status |
| --- | --- | --- | --- | --- |
| 2026-09-11 | baseline | `baseline/2026-09-11-initial-baseline.md` | Initial dual-baseline snapshot | active |
| 2026-09-11 | spec | `specs/2026-09-11-profiles-page-query-design.md` | Profiles tab page query | implemented |
| 2026-09-11 | plan | `plans/2026-09-11-profiles-page-query.md` | Profiles tab page query — implementation | executed (T1-T3, T7 done; T4 windowing superseded by the 200-row page, T5/T6 delivered by the write-behind slice) |
| 2026-09-11 | spec | `specs/2026-09-11-write-behind-link-writer-design.md` | Write-behind link writer (batch ping persistence) | implemented |
| 2026-09-11 | plan | `plans/2026-09-11-write-behind-link-writer.md` | Write-behind link writer — implementation | executed (W1-W4 + W5.2; W5.4 throughput harness deliberately not built — per-statement cost measured directly) |
| 2026-09-11 | adr | `adr/0001-raw-sql-profiles-page-query.md` | Raw SQL for the Profiles page query | accepted |
| 2026-09-11 | adr | `adr/0002-write-behind-link-writer.md` | Write-behind profile_stats persistence | accepted |
| 2026-09-14 | spec | `specs/2026-09-14-profiles-stored-sort-key-design.md` | Profiles sort keys: materialize the per-endpoint ordering | implemented |
| 2026-09-14 | adr | `adr/0003-stored-profile-ordering-keys.md` | Materialized per-endpoint ordering keys | accepted |
| 2026-09-14 | spec | `specs/2026-09-14-db-tui-flow-audit.md` | DB ↔ TUI flow audit (queries, entities, indexes, memory) | research complete; plan in §6 |
| 2026-09-14 | spec | `specs/2026-09-14-native-ping-pipeline-design.md` | Native ping pipeline: real ping on the in-process core | implemented |
| 2026-09-14 | plan | `plans/2026-09-14-native-ping-pipeline.md` | Native real-ping pipeline — implementation | executed (T1-T6, T8; `build_multi` dead-code sweep deferred) |
| 2026-09-14 | adr | `adr/0004-native-real-ping.md` | The real-ping probe engine is the native core | accepted |
| 2026-09-15 | plan | `plans/2026-09-15-ping-pipeline-integrity.md` | Ping-pipeline persistence and observability — implementation | executed (T1-T11; the transient TLS class left unfixed by evidence, FD-at-scale verification deferred) |
