# Checkpoint — DB rewamp

## Snapshot
- root `/home/user/oss/xray-tui`; branch `native-core-stub`; HEAD `5bd2d64`.
- Upstream `origin/native-core-stub`. Single worktree.
- Preexisting dirty: `docs/aegis/INDEX.md` (M), spec+plan (untracked). Preserved.

## Todo map
- S1: T0 baseline · T1 harness · T2 turso gate
- S2 (one non-green window): T3 psl2+meta · T4 identity/host · T5 validation · T6 core_type · T7 ConfigType+JSON · T8 binned law+index · T9 page/search · T10 sort-UI · T11 wipe · T12 FK
- S3: T13 long-lived reader · S4: T14 gated WR · S5: T15 docs/ADR/AGENTS

## Active
T0/T2 — results being collected (`bg_4` → `/tmp/rewamp_t0t2.txt`).

## Completed / evidence
- **T0 (partial):** HEAD index inventory + size captured: 11 raw/secondary
  indexes + 10 PK autoindexes; `user_version=14`; 74,723 endpoints / 146,744
  links / 77,395 protocols / 7,721 addresses; file 121.8 MB (with WAL).
- **T1 (done):** `PageSort::Id` and `profiles_walk_page` have NO production
  caller (grep: lab `flow_cost.rs` + its integration test only). The lab port is
  folded into the T10 slice (porting now targets an API about to change).
- **T2 (written):** `crates/xray-tui-db/tests/turso_planner.rs` — ignored gate
  seeding the proposed `endpoint_rank` shape at 74,723 rows on a direct turso
  connection and printing `EXPLAIN QUERY PLAN` for Active/Purgatory/All/scope/reband.
- **Advisory rejected:** a claimed workspace/hakari self-cycle — `cargo metadata`
  RC=0 and `cargo check -p xray-tui-db` RC=0; false.

## Execution refinement (deviation from plan wording, not scope)
S2 executes as **sequential green sub-slices** (each drops a column AND its
readers in the same commit) instead of one non-green window: the plan's
atomicity argument is per-file-list, and every sub-slice keeps the crate
green + testable. Same total change; safer verification. Recorded here.

## Blockers
(none)

## Next step
Collect T2/T0 output; run T2 gate; then S2 first sub-slice.
