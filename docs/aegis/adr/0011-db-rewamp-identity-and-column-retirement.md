# ADR 0011 — DB rewamp: locked order, column retirement, and the split identity

Date: 2026-10-06.
Status: **accepted — PARTIAL.** Records the slices already shipped; the
host/identity model, the binned ranking law, the direct page reader and the FK
cascade are **accepted but not yet implemented** (see "Deferred").
Spec: `docs/aegis/specs/2026-10-06-db-rewamp-design.md` (rev. 2).
Plan: `docs/aegis/plans/2026-10-06-db-rewamp.md`.
Supersedes/extends: ADR 0001 (page raw SQL), ADR 0003 (rank keys), ADR 0010 (band).

## Context

The Profiles tab's ordering was an 8-column sort cycle, and every non-law sort
was a materialized `endpoint_rank` column + index written on every rank refresh.
`protocols` carried two write-only JSON copies of `config`; `profile_stats` and
`groups` carried a `core_type` the native core no longer needs; and a config's
ORIGIN (`ShareUrl` vs `Form`) participated in identity. Measured on the 74,723-
endpoint feed (`data.db`, `user_version=14`): the Active-view Test page paid a
**108 ms** filesort; `transport_data`+`security_data` were **11.66 MB** (13.6% of
the file) with no production reader.

## Decisions (shipped)

1. **D1 — one order.** The Profiles tab renders exactly the decision-16 law.
   `SortColumn`, the sort cycle, `AppState.sort_column`/`sort_ascending`/`set_sort`
   and `PageSort::ConfigType` are deleted. `PageSort` keeps its variants for the
   perf lab (`flow_cost`).
2. **D3 — `core_type` retired.** `profile_stats.core_type` and `groups.core_type`
   drop; the core is derived at connect from the kind, the config-level
   `protocol_core_overrides` and the Shadowsocks method. The derivation runs on
   the **loaded** protocol (the page row's config is unloaded, so a legacy-cipher
   SS row would otherwise default to xray-core and fail at build).
3. **D8 — the write-only JSON drops.** `protocols.transport_data`/`security_data`
   were exact projections of `config`; removing them cuts **−13.6%** of the file.
4. **D9 — one rule, no config origin.** `ConfigType` leaves identity and the
   schema: a form config and an identical share URL share one `Protocol` row.
   `IDENTITY_VERSION` 2 → 3; the identity goldens are re-pinned.
5. **D2 (helper only) — the split owner + its version guard.** `psl2 = 0.1.31`
   backs `xray_tui_config::domain::split` (ONE normalizer: `analyze`, IDNA→
   punycode). The PSL version is stamped in a generic `app_meta(key,value)` table
   at startup; a mismatch logs a re-import warning (the split re-keys endpoints).
6. **Schema tag 14 → 15** (a wipe): the dropped columns would be NOT NULL on a
   v14 file while the model no longer has them, so every import INSERT would fail.

## Deferred (accepted, not implemented)

The identity/host model (drop `endpoints.host`/`host_type`; `domain`/`sub_domain`
as the identity input; every address — including IP literals — into
`endpoint_ip`), the binned ranking law `(bin, ⌐weight, domain, sub, addr,
endpoint_id)` with ONE covering index, the uniform psl2 validation + counted
skip, the long-lived direct `turso::Connection` page reader, and the FK cascade.
The plan's slices T4/T5/T8/T9/T12/T13/T14 carry them; each is a large atomic
change (T4 alone touches ~10 files and cannot compile until every consumer
moves; T12 additionally implies hand-owning four child tables' DDL, which
`docs/database-manual-sql.md` §5 rejected for STRICT). Measured evidence for the
deferred parts is in the spec §9 — **and is the SQLite planner, NOT turso**: the
T2 gate (`crates/xray-tui-db/tests/turso_planner.rs`) confirms turso selects the
proposed `endpoint_rank_key` shape (Active/Purgatory seek, All `(band,key)`
covering scan; a literal `band IN (0,1)` filesorts, turso's `USE SORTER`), but no
turso *timing* is quoted yet.

## Consequences

- The re-key (identity v3) and the column drops ride the schema-15 wipe; the
  pre-alpha wipe policy (decision 4) applies.
- `psl2` is a new dependency; the split is one function so validation, the
  row-build owner and the search predicate cannot drift.
- Retirement: the old sort cycle, the two JSON columns, `core_type`/
  `config_type`/`rank_config` are gone; `host`/`host_type` remain until the
  deferred identity slice lands.

## Verification

Each shipped slice is green at its commit: `cargo nextest` proto/config/core
732, db 171, tui 259; `cargo check --workspace --all-targets` clean. The
workspace clippy gate is **pre-existing-red** on this WIP branch (unchanged
files such as `native/src/context.rs`, `tls/src/spec/mod.rs`,
`route/src/compiler/mod.rs`); the rewamp diffs add **zero** clippy errors
(verified against base `5bd2d64`).
