# Checkpoint — DB rewamp

## Snapshot
- root `/home/user/oss/xray-tui`; branch `native-core-stub`; HEAD `c41e63f`.
- Tree **clean**; last 4 commits are this workstream (S1 → … → ConfigType).

## Commits landed (each green: nextest + workspace check)
| commit | slice |
| --- | --- |
| `544d441` | S1 — spec rev.2, plan, baseline, harness guard, turso planner gate |
| `068bbb6` | drop write-only `transport_data`/`security_data` (−13.6% file) |
| `80111de` | drop `core_type` (per-pair + group) + the form override (D3) |
| `c41e63f` | drop `ConfigType` from identity + schema (D9), `IDENTITY_VERSION` 2→3, golden re-pinned; `rank_config` removed with it |

## Todo map
- **Done:** T0 baseline · T1 harness guard · T2 turso gate · T6 core_type · T7-part[a: JSON]
- **Remaining:** T3 psl2 helper+meta · T4 identity/host model · T5 validation ·
  T7-part[b: ConfigType — DONE] · T8 binned law+index · T9 page order/search ·
  T10 sort UI · T11 tag-15 wipe · T12 FK cascade · T13 direct reader ·
  T14 gated WITHOUT ROWID · T15 docs/ADR/AGENTS

## Verification at this HEAD
- `cargo check --workspace --all-targets` → 0 errors.
- `cargo nextest`: proto/config/core **732**, db **170**, tui **261** — all green.

## Blockers
- none technical. **Budget**: the remaining slices (psl2 identity, binned law,
  page/search, sort UI, wipe, FK, reader, docs) are each large; the session did
  not have room to finish them.

## Next step (exact resume point)
1. **T10 (smallest):** delete the sort UI — `types::SortColumn`, the `s` cycle in
   `ui/mod.rs`, `ops/profiles.rs::page_sort`/`set_sort`, `AppState.sort_column`/
   `sort_ascending`, `ui/profiles.rs`'s sort-column match. Keep `PageSort` for the
   perf lab. Green + commit.
2. **T3/T4/T5:** add `psl2`, the one split helper, `domain`/`sub_domain` on
   `endpoints`, derived host-kind, all-addresses-in-`endpoint_ip`, uniform
   validation + counted skip, the three `host_type→HostKind` reconstruction sites.
3. **T8/T9:** binned law `(bin, ⌐weight, domain, sub, addr, endpoint_id)`,
   `endpoint_rank` reshape + the one covering index; page order/view/search.
4. **T11** tag 15 wipe, **T12** FK cascade, **T13** direct reader,
   **T14** gated WR, **T15** docs/ADR/AGENTS.
