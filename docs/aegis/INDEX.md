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
| 2026-09-15 | plan | `plans/2026-09-15-batch-feed-scope.md` | Batch scope: the feed, not the viewport — implementation | executed (T1-T4) |
| 2026-09-15 | spec | `specs/2026-09-15-endpoint-ip-storage-design.md` | Endpoint addresses: storage, ordering, and what Turso 0.7.2 actually offers | implemented (schema tag 10) |
| 2026-09-15 | adr | `adr/0005-endpoint-addresses-as-a-packed-key-table.md` | Resolved addresses are a packed-key table | accepted |
| 2026-09-16 | spec | `specs/2026-09-16-db-claim-verification.md` | Database performance claims: verification and disposition | implemented (two write-path changes shipped; 4 reports + PDF dispositioned) |
| 2026-09-16 | plan | `plans/2026-09-16-ping-engine-tls-fixes.md` | Ping-run log analysis: engine TLS fixes and batch observability | executed (T1-T11; subscription-retry design, class precision and toasty labels deferred) |
| 2026-09-17 | plan | `plans/2026-09-17-ping-run-analysis.md` | Ping-run analysis: two malformed extension bodies, phase-2 cost, and a truncated import | executed (P1/P1b/P2/P3/P5/P6 landed; P4/P7/P8/P9/P10 deferred with reasons) |
| 2026-09-17 | spec | `specs/2026-09-17-batch-ping-pipeline-design.md` | Batch ping: streaming plan, per-link fast→real pipeline, two-level progress | implemented |
| 2026-09-17 | plan | `plans/2026-09-17-batch-ping-pipeline.md` | Batch ping pipeline — implementation | executed (T1-T6; live verification in the commit that closes it) |
| 2026-09-17 | spec | `specs/2026-09-17-purge-reason-design.md` | Purge reasons: evidence-based permanent moves to Purgatory | implemented (schema tag 12) |
| 2026-09-17 | adr | `adr/0006-purge-evidence.md` | Purge evidence is persisted per link; Purgatory is "not confirmed live and recent" | accepted |
| 2026-09-17 | plan | `plans/2026-09-17-purge-reason.md` | Purge reasons — implementation | executed (T1-T7 green; T8 is the live re-run) |
| 2026-09-18 | spec | `specs/2026-09-18-test-cell-truth-and-plan-scopes-design.md` | Test-cell truth, resolution ownership, and batch plan scopes | approved; implementation in progress |
| 2026-09-18 | spec | `specs/2026-09-18-crypto-dependency-hardening-design.md` | Cryptographic dependency hardening: RustCrypto ML-KEM, hazmat sites, advisory triage | implemented |
| 2026-09-18 | plan | `plans/2026-09-18-crypto-dependency-hardening.md` | Cryptographic dependency hardening — implementation | executed (T1-T9; zstd→ruzstd deliberately deferred) |
| 2026-09-18 | adr | `adr/0007-cryptographic-dependency-posture.md` | Cryptographic dependency posture: RustCrypto ML-KEM, ctr and ring retained, hazmat governance | accepted |
| 2026-09-18 | adr | `adr/0008-batch-feed-scaling.md` | Batch-feed scaling: id-ordered walk, per-protocol loads, throttled page refetch | accepted |
| 2026-09-22 | adr | `adr/0009-fingerprint-approximation.md` | Unhonourable TLS fingerprints are approximated and marked, never refused | accepted |
| 2026-09-22 | spec | `specs/2026-09-22-native-testability-improvement-design.md` | Native testability: trustworthy verdicts on a real feed | implemented, with two dispositions **decided terminal by the user (2026-09-22)**: T4 deferred (`min` semantics kept), and mlkem item 2 returned to design (localized to our client's wire) |
| 2026-09-22 | plan | `plans/2026-09-22-native-testability-improvement.md` | Native testability — implementation | executed: T0–T3, T5–T11 landed; T4 and T9's fix terminal by user decision, each with its reasoning and scope recorded |
| 2026-09-23 | spec | `specs/2026-09-23-ws-path-canonicalization-design.md` | WS path canonicalization at parse: dialable configs, one uid per logical path | implemented |
| 2026-09-23 | plan | `plans/2026-09-23-ws-path-canonicalization.md` | WS path canonicalization — implementation | executed (T1–T6; e2e green against a real xray-core server; `SCHEMA_VERSION` 12 → 13) |
| 2026-09-24 | spec | `specs/2026-09-24-profiles-view-band-design.md` | Profiles view-band: kill the Active-view Address filesort | draft — awaiting user review |
| 2026-09-24 | adr | `adr/0010-materialized-active-band.md` | Materialized Active band: view membership is a stored column | accepted |
| 2026-09-24 | spec | `specs/2026-09-24-turso-mvcc-rollout-design.md` | Turso MVCC rollout: evidence and compatibility boundary | opt-in MVCC; default WAL; real-feed A/B pending |
| 2026-09-24 | plan | `plans/2026-09-24-turso-mvcc-production-rollout.md` | Turso MVCC production rollout — implementation | T1–T3 complete; production workload benchmark open |
| 2026-09-30 | spec | `specs/2026-09-30-ss-plugin-design.md` | Shadowsocks SIP003 plugins (`v2ray-plugin`, `obfs-local`) + VLESS TCP mux | **approved** (rev. 14) — scope decided: plugin feature, generic mux reachability, and full VLESS `mux` field/parse/identity/form/Clash (smux-family rows refused; pooled tunnel deferred); carries a schema-tag wipe (13 → 14) |
| 2026-09-30 | plan | `plans/2026-09-30-ss-plugin.md` | Shadowsocks SIP003 plugins + VLESS TCP mux — implementation | **Slices 1–4 + T24 complete; Slice 5 stopped at its own T22 gate; Slice 6 (docs) closed.** SIP003 plugin rows connect natively (obfs http/tls/plain + v2ray-plugin WebSocket, mux as the protocol phase with per-session SS codecs); VLESS `mux` has the full field/parse/identity/form/Clash surface, unusable spellings stored and refused by name in `capability` (never at parse), and vision×mux refused. **T22 evidence note: `mode=quic` CANNOT be pinned — the oracle embeds V2Ray 4.38.3 (source absent) and three divergent QUIC wires are in-tree — so the task STOPS per the plan, and quic is instead refused by name rather than dialled over TCP.** T24 landed regardless: a quic row is no longer retired by its TCP fast failure and that marker is retracted. **FINAL GATE over `--workspace`**: fmt clean, `clippy --workspace --all-targets -D warnings` **0 errors** (the four long-carried `xray-tui-db` errors are FIXED, not waived), **2192 tests pass**; deps gate green (machete/hakari/audit/deny); opt-in `plugin_sip003` (4) and `plugin_3a` (1) green. Release notes: `2026-10-01-ss-plugin-release-notes.md` |


