# Intent — DB rewamp (2026-10-06)

## Outcome
Execute `docs/aegis/plans/2026-10-06-db-rewamp.md` (approved) against the approved
spec `docs/aegis/specs/2026-10-06-db-rewamp-design.md` (rev. 2): split endpoint
identity onto psl2 `domain`/`sub_domain`, all addresses in `endpoint_ip`, binned
ranking law with ONE covering index, long-lived direct turso page reader, drop
`core_type`/`ConfigType`/`transport_data`/`security_data`, FK cascade, WITHOUT
ROWID gated on turso.

## Success evidence
nextest workspace green; clippy `--all-features -D warnings`; flow_cost at 74k
**on turso** shows the covering index with no temp b-tree for Active/Purgatory/All;
identity/validation parity form==import; file ≤ ~95 MB.

## Stop states
`done | blocked | needs-verification | scope-exceeded`.

## Non-goals
Routing/DNS/group tables; native core; keyset pagination; folding `endpoint_rank`
into `endpoints`; an `endpoint_ip` text column.

## Risks
turso ≠ SQLite planner (T2 gate); the tag-15 wipe (authorised, destructive);
psl2 version as identity input; counted-skip blast radius; `rank_addr` refresh cost.

## Baseline refs
Spec rev. 2; plan rev. 2; `docs/database.md`; `docs/database-manual-sql.md`; ADR
0001/0003/0010; AGENTS decisions 4/11f/15-16/20-21.
