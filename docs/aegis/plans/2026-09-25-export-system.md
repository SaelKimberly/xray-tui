# 2026-09-25 — Export system implementation plan

Parent spec: `docs/aegis/specs/2026-09-25-export-system-design.md` (approved by user 2026-09-25).

## Goal

Implement Profiles `Ctrl+E` export with four scopes, deterministic streaming output, clipboard/file destinations, resolved-IP transformation, exact header, bounded memory, and WAL/MVCC snapshot safety.

## Architecture

```text
Profiles UI
  → xray-tui/src/ops/export.rs
      → xray-tui-db dedicated per-export Turso file reader
          → protocols/endpoints/profile_stats/endpoint_ip/endpoint_rank
      → existing ProtocolConfig::reconstruct_proto
      → file spool or one clipboard String
```

Canonical owners:

- DB: raw read-only streaming projection, file reader lifecycle, transaction/mode, scope SQL, row decoding.
- Proto: exhaustive default SNI/transport-authority mutation helpers; existing URL serializer remains canonical.
- TUI ops: scope transformation, reconstruction, counts, destination sinks.
- TUI UI: popup/path/overwrite interaction only.
- `arboard` and `tokio::fs`: destination mechanisms only.

No schema tag bump. No persisted data migration. No new URL serializer. No fallback to `load_page_rows`, per-link `load_protocol_with_config`, OFFSET, or feed-wide `Vec`.

## Tech stack

Rust 2024, Toasty 0.10, Turso 0.7.2, Tokio, `arboard`, `tokio::fs`, ratatui/tui-popup, existing `RankLink` ordering law.

## Baseline / authority refs

- `AGENTS.md` decisions 4, 16, 21, 22.
- `docs/aegis/specs/2026-09-25-export-system-design.md` §§2–10.
- `docs/database.md` schema/query-map/derived-state contracts.
- `docs/database-manual-sql.md` §§1–3 and §6 raw-statement checklist.
- `docs/aegis/specs/2026-09-17-purge-reason-design.md` for `purge_reason` and view semantics.
- `docs/aegis/specs/2026-09-24-profiles-view-band-design.md` for Active membership.
- `TUI_MANUAL.md` Profiles/Speed Test popup/keyboard conventions.
- `crates/xray-tui-db/src/endpoint_rank.rs` `RankLink::key` per-link canonical tier.
- `crates/xray-tui-proto/src/proto_spec/{mod,common}.rs` reconstruct/security types.
- `crates/xray-tui-config/src/import_export.rs` `format_share_url` contract.

## Compatibility boundary

- Existing persisted schema/data remains unchanged.
- Existing `ProtocolConfig::reconstruct_proto` behavior remains unchanged outside cloned export-time config.
- Existing page/profile/purge/rank writers remain owners; export read path does not add write facts.
- `Database::in_memory()` remains valid for existing tests; only streaming export returns unsupported there.
- New direct dependency is `turso = 0.7.2` in `xray-tui-db`; no dependency on private Toasty driver internals.
- Existing untracked `crates/xray-tui/examples/c9f7bad` remains untouched.

## TDD route

TDD Route:
- Mode: `auto`.
- Decision: `strict`.
- Authority: approved spec covers persistence/read boundary, cross-crate transformation, cancellation/cleanup, memory bound, and consumer-visible output.
- Test posture: focused RED/GREEN for scope predicates, resolved transformation, raw row projection, WAL/MVCC snapshot/cleanup, sink behavior, and popup state; then existing workspace suite and live TUI smoke.
- Strict reason: this changes a persistence read boundary and protocol transformation shared by multiple protocols. A regression can silently emit dead, unusable, or non-deterministic configs.
- Verification: `cargo nextest run -p xray-tui-db -p xray-tui-proto -p xray-tui`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo fmt --all --check`; large-reference RSS smoke; actual TUI popup/export smoke.

## Requirement ready check

- Requirement source: user request + approved design spec.
- Goals/scope: four scopes, two destinations, raw subscription format, exact header.
- Scenarios: whole DB independent of current UI filters; resolved DNS expansion; stale/purged selection; overwrite confirmation; cancellation.
- Acceptance: spec §9; all observable checks listed there.
- Open blockers: none. Dedicated reader is file-only; same-mode construction and explicit transaction teardown are task acceptance criteria.
- Decision: `ready`.

## Change necessity

- User-visible need: export usable/deterministic subscription text without loading whole DB or duplicating clipboard body.
- No-change option: impossible; no current export sink/popup or bounded cross-join reader exists.
- Minimum boundary: one DB read owner, one TUI export owner, one UI owner, two narrow proto mutation helpers, manifest/docs/tests.
- Decision: `code-change`.

## Existence / architecture check

- New surface: dedicated streaming read path and TUI export orchestration/UI.
- Existing candidates: `xray-tui-db` typed page readers and `ProtocolConfig::reconstruct_proto`.
- Why insufficient: typed readers materialize pages/relations; Toasty 0.10 and its Turso driver buffer rows; existing serializer is correct but does not own feed selection or sink policy.
- Creation proof: spec requires row-at-a-time memory bound, cross-table deterministic ordering, and direct file connection lifecycle absent from existing owners.
- Duplicate owner/fallback: none permitted. No second serializer, no per-link lookup, no old export path.
- Retirement trigger: future Toasty/driver public pooled row stream; then remove dedicated reader after parity and RSS verification.
- Verdict: `add-with-proof`; proceed inline.

## Plan pressure test

- Owner/contract: DB owns raw read; proto owns mutation helpers/serializer; TUI owns policy/sinks/UI.
- Higher-level path: preserve existing serializer and typed config model; add one export transform instead of protocol-specific URL branches.
- Verification: DB tests, proto tests, TUI unit/TUI smoke, RSS sampling, raw SQL inventory.
- Executability: ordered slices avoid shared-file races; no concurrent implementation of DB reader and TUI wiring.
- Result: `proceed`.

## Complexity budget

- Artifact class: maintained source + durable plan/spec/docs.
- Pressure: `database.rs` is already large; `ui/mod.rs` is large; new owner files are required for reader/export/UI separation.
- Budget: `at-risk` if export logic is added to existing orchestrators; `within-budget` with `db/export.rs`, `ops/export.rs`, `ui/export.rs`.
- Recommendation: add owner files; edit existing files only for wiring, manifests, and lifecycle.

## Files

| Task | Create | Modify |
|---|---|---|
| T1 | `crates/xray-tui-db/src/export.rs` | `crates/xray-tui-db/Cargo.toml`, `crates/xray-tui-db/src/lib.rs`, `crates/xray-tui-db/src/database.rs` |
| T2 | — | `crates/xray-tui-proto/src/proto_spec/common.rs`, `crates/xray-tui-proto/src/proto_spec/mod.rs`, proto tests |
| T3 | `crates/xray-tui/src/ops/export.rs` | `crates/xray-tui/src/ops/mod.rs`, `crates/xray-tui/src/lib.rs` if export event wiring needs it |
| T4 | `crates/xray-tui/src/ui/export.rs` | `crates/xray-tui/src/types.rs`, `crates/xray-tui/src/ui/mod.rs`, `crates/xray-tui/src/ui/status_bar.rs`, `crates/xray-tui/Cargo.toml` only if Tokio features require `fs`/`io-util` |
| T5 | — | `docs/database-manual-sql.md`, `docs/aegis/INDEX.md` if workspace index owner requires registration, `TUI_MANUAL.md` |
| T6 | throwaway smoke only; delete after | no maintained scaffold |

## Ordered tasks

### T1 — DB streaming reader and scope row projection

Files: create `crates/xray-tui-db/src/export.rs`; modify DB manifest/module exports/`database.rs`.

RED tests first:

- `alive_uses_per_link_rank_tier_not_endpoint_rank`.
- `full_includes_aged_live_links_and_excludes_purge_verdicts`.
- `active_matches_materialized_band_and_live_links`.
- `resolved_projection_emits_one_row_per_endpoint_ip`.
- `ordered_projection_has_complete_deterministic_tuple`.
- `raw_statement_runs_in_wal_and_mvcc_temp_file_dbs`.
- `reader_count_and_rows_share_one_snapshot`.
- `reader_error_and_cancel_drain_rows_and_rollback`.

Implementation:

1. Add direct `turso = "0.7.2"` dependency. Add `tokio` features required by direct file/transaction use where necessary.
2. Extend `Database` with file path + journal mode authority and an export serialization primitive. Keep `in_memory()` reader unavailable.
3. Add `db/export.rs` with a typed row struct and `ExportScope` input. Do not expose raw `turso::Rows` to TUI.
4. Build two read-only SQL projections:
   - link projection for Alive/Active/Full, one row per link, with minimum resolved `ip_key` only for sort;
   - resolved projection for Resolved, one row per link/address.
5. Run the candidate count with the same scope predicate and transaction snapshot as the selected projection. Count stored links before Resolved address expansion; do not count SQL output rows.
6. Join `protocols.config` in each row. Do not call `load_protocol_with_config` per link and do not add a page loader.
7. Decode one row at a time. Keep the row/config owned only until sink emission.
8. Enforce per-link Alive via canonical `RankLink::key` facts. Do not use endpoint-representative `rank_tier`.
9. Use explicit `BEGIN DEFERRED` for WAL and `BEGIN CONCURRENT` for MVCC. Run count and stream in same transaction.
10. On success drain `Rows` to `None`, commit, then drop connection. On error/cancel, drain when possible, rollback, then drop connection. Do not return an active cursor/transaction to caller.
11. Measure `EXPLAIN QUERY PLAN`; add only schema-neutral idempotent covering indexes if required. Record every raw site and test in `docs/database-manual-sql.md`.

Check:

- `cargo test -p xray-tui-db export -- --nocapture`.
- `cargo test -p xray-tui-db --test profiles_query`.
- `cargo test -p xray-tui-db --test integration`.

### T2 — Exhaustive export-time config transformation

Files: `crates/xray-tui-proto/src/proto_spec/common.rs`, `mod.rs`, adjacent protocol tests.

RED tests first:

- `resolved_vless_preserves_dns_sni_while_ip_authority_changes`.
- `resolved_vmess_preserves_dns_sni_while_ip_authority_changes`.
- `resolved_http_family_preserves_dns_authority_when_host_absent`.
- `explicit_sni_and_authority_are_not_overwritten`.
- `new_protocol_variant_requires_exhaustive_mutation_dispatch`.

Implementation:

1. Add `SecurityConfig::set_default_sni(&str)` that fills only absent TLS/Reality SNI.
2. Add exhaustive `ProtocolConfig::security_mut()` dispatch and transport mutation/accessor path covering each existing protocol variant.
3. Add export-time default authority mutation for WS/gRPC/HTTP transport only when host/authority is absent. Keep explicit values unchanged.
4. Keep this logic out of every `reconstruct_proto`; existing serializer remains canonical.
5. Keep endpoint `ports` untouched; export clones endpoint and changes only `host`/`host_type`.

Check:

- `cargo test -p xray-tui-proto export -- --nocapture`.
- `cargo test -p xray-tui-proto --lib`.

### T3 — TUI export policy, reconstruction, and sinks

Files: create `crates/xray-tui/src/ops/export.rs`; modify `ops/mod.rs`; add event/status wiring only where existing patterns require.

RED tests first:

- `all_scopes_build_expected_candidate_count`.
- `resolved_rewrites_host_and_preserves_sni_authority_and_ports`.
- `serializer_failure_is_skipped_without_aborting`.
- `file_sink_streams_body_and_writes_exact_header`.
- `clipboard_sink_uses_one_string_and_patches_candidate_count`.
- `candidate_count_precedes_ip_expansion_and_serializer_skips`.

Implementation:

1. Define export scope, destination, and immutable report types.
2. Consume DB rows through a callback/sink interface owned by `ops/export.rs`; no whole-feed collection.
3. Clone loaded protocol config and endpoint per output target. Apply T2 helpers only for Resolved. Preserve `port` and `ports`.
4. Call existing `ProtocolConfig::reconstruct_proto(&endpoint)`.
5. Count candidates before expansion, emitted lines after successful reconstruction, skipped links separately.
6. File sink: after overwrite confirmation, create/truncate the destination and stream the exact header and URLs directly through `tokio::fs::File` + `AsyncWriteExt`; flush as required. No temporary body or atomic-replace pass. A failed write may leave a partial destination; report the error and do not report success.
7. Clipboard sink: one `String`, fixed-width count placeholder patched in-place, then `arboard::Clipboard::set_text`.
8. Serialize exports through DB-level lock; do not allow overlapping raw readers.
9. Add session-only Actions/log success/error report.

Check:

- `cargo test -p xray-tui export -- --nocapture`.
- `cargo test -p xray-tui --lib`.

### T4 — Profiles popup, keyboard routing, and feedback

Files: create `crates/xray-tui/src/ui/export.rs`; modify `types.rs`, `ui/mod.rs`, `ui/status_bar.rs`.

RED/state tests first where existing TUI test infrastructure permits:

- `ctrl_e_opens_scope_popup_on_profiles`.
- `scope_then_destination_navigation`.
- `file_path_prefill_and_editing`.
- `existing_file_requires_yes_confirmation`.
- `escape_cancels_without_write`.
- `export_completion_reports_candidates_emitted_skipped`.

Implementation:

1. Add `AppMode` variants for export scope, destination, and file path/overwrite state. Keep popup state typed; do not overload `SpeedTestMenu`.
2. Route `Ctrl+E` only on Profiles.
3. Reuse Speed Test popup rendering/navigation style, with separate export owner.
4. Enter selects scope then destination. Clipboard dispatches export. File opens path input.
5. Existing destination triggers y/n confirmation before any spool/write. `n`/Esc cancel.
6. Render status/action feedback and errors without blocking UI task.
7. Ensure form/log key filtering does not swallow popup keys.

Check:

- `cargo test -p xray-tui --lib`.
- Actual TUI smoke using existing `tui-test` launch/session tools: open popup, navigate, choose file, confirm/cancel, export, inspect screen.

### T5 — Documentation and manual SQL inventory

Files: `docs/database-manual-sql.md`, `TUI_MANUAL.md`, existing Aegis index if plan/spec registration is required.

Implementation:

1. Add dedicated reader/projection row to raw SQL inventory with cause, measured plan/cost, and test name.
2. Document `Ctrl+E`, scope names, candidate-count semantics, destinations, overwrite confirmation, and async status.
3. Update `AGENTS.md` only if a durable project key/architecture decision needs a short canonical pointer; do not duplicate full spec.
4. Update `docs/aegis/INDEX.md` only through existing workspace registration mechanism if this repo requires it.

Check:

- Documentation grep finds no stale claim that export uses page loader, OFFSET, emitted count, in-memory raw reader, or overwrite without confirmation.
- `cargo fmt --all --check` after source tasks.

### T6 — Verification and cleanup

No maintained files unless a real defect is found.

1. Run focused tests from T1–T4.
2. Run full affected workspace tests/clippy/fmt.
3. Run large-reference export RSS probe. Record peak RSS and output size for file/clipboard; prove no feed-sized temporary vector.
4. Run WAL and MVCC temp-file snapshot/cancellation checks.
5. Run actual TUI popup and file/clipboard smoke. Inspect output header, line count, sort order, resolved IP/SNI/authority, and overwrite behavior.
6. Delete throwaway probes/examples. Re-read `git status --short`; preserve unrelated `crates/xray-tui/examples/c9f7bad` untouched.
7. If any acceptance check fails, fix root owner, rerun focused check, then rerun affected suite. Do not claim done from tests alone.

## Sequencing and ownership

Strict order: T1 → T2 → T3 → T4 → T5 → T6. T2 can be developed independently only after T1’s row/config contract is pinned, but do not run concurrent edits in this worktree. T1/T3/T4 share DB/TUI contracts; single integration owner required.

## Retirement / compatibility

- No old export path exists; no migration shim.
- Dedicated reader retires only when Toasty exposes pooled row streaming.
- Existing URL serializer remains; export clone/mutation is additive and local.
- Existing `Database::in_memory()` behavior stays; export returns explicit unsupported error.
- Existing file overwrite behavior only applies after explicit confirmation.

## Plan self-review

- Every approved spec section maps to task/check: selection T1/T3, sort T1, format T3, reader T1, sinks T3, popup T4, docs T5, RSS/TUI smoke T6.
- No unresolved product decision remains.
- No page-loader contradiction remains in plan: one joined config projection per streamed row.
- No per-link N+1 config load remains in plan.
- IP-literal sort is executable lexical; resolved sort uses packed bytes; unresolved fallback is explicit.
- Count is explicitly candidate-link count; emitted/skipped are separate.
- Overwrite confirmation is explicit before spool/write.
- Snapshot mode and cleanup are explicit.
- TDD strict route is justified by persistence/cross-crate/user-visible output.
- User confirmation required: none; approved spec supplies product decisions. No destructive persistent action is planned.

## Execution route

Decision: `inline`.

Evidence: T1–T4 share database row/config contracts and TUI state; parallel edits would coordinate stale signatures. No user confirmation required. Fallback: inline if subagent coordination is unavailable; no subagent task starts before T1 contracts are written.
