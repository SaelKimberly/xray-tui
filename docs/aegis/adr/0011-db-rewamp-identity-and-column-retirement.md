# ADR 0011 — DB rewamp: locked order, column retirement, and the split identity

Date: 2026-10-06.
Status: **accepted — SHIPPED except T12/T14, which turso 0.7.2 BLOCKS** (see
"Deferred — engine-blocked"). The host/identity model, the binned ranking law,
the uniform validation and the direct page reader are implemented and green.
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
   backs `xray_tui_proto::domain::split` (ONE normalizer: `analyze`, IDNA→
   punycode; re-exported from `xray_tui_config::domain`). It lives in `proto`
   because both `config` and `db` depend on it and the db rank refresh needs the
   split (a T8 prerequisite). The PSL version is stamped in the typed
   `app_meta(key,value)` table at startup; a mismatch logs a re-import warning
   (the split re-keys endpoints).
6. **D2/D10 — the host/identity model.** `endpoints.host`/`host_type` drop;
   `domain`/`sub_domain` (the psl2 eTLD+1 split) are the identity input and the
   host kind is DERIVED (`Endpoint::is_dns`). An IP literal's ONLY home is
   `endpoint_ip`, so every import entry point writes it alongside the row
   (`state::persist_parsed`; `SourceSpec::write_window`).
7. **D11 — the binned ranking law.** `endpoint_rank` carries the bin/weight/
   domain/sub/addr keys and ONE covering index `endpoint_rank_key`; the sort
   cycle is deleted (D1).
8. **D6 — search** is a `domain`/`sub_domain` PREFIX plus an IP/CIDR range over
   the packed `rank_addr`.
9. **D5 — the direct page reader.** A long-lived `turso::Connection` serves the
   page id+count (the 108 ms toasty path; Active Test page **1.52 ms** at 50k).
10. **Schema tag 14 → 18** (a wipe): 15 dropped the rewamp columns, 16 added
    `app_meta`, 17 reshaped `endpoint_rank` to the binned law, 18 dropped
    `endpoints.host`/`host_type`.

### Decision — the exact sort key

The endpoint key is `(band, rank_bin, rank_weight DESC, rank_domain,
rank_sub_domain, rank_addr, endpoint_id)` (ADR 0003, amended 2026-10-07): the
tier (`rank_bin`) still outranks the static config weight, which outranks the
address; the weight's ORDER is `DESC` on the stored big-endian BLOB, which IS
the comparator's `u64::MAX - weight` ascending.

## Deferred — engine-blocked (T12, T14)

Both remaining plan slices were probed directly against the vendored
**turso 0.7.2** and are **not implementable on it** (evidence:
`docs/aegis/work/2026-10-06-db-rewamp/90-evidence.md`, "T12/T14 probed and
BLOCKED"):

- **T12 (FK cascade) — blocked twice.** (a) toasty 0.11's `push_schema()` emits
  no `REFERENCES` clause at all, so the DDL would have to be hand-owned
  (`docs/database-manual-sql.md` §5 rejected that for STRICT). (b) turso rejects
  foreign keys on `WITHOUT ROWID` tables
  (`Parse error: foreign keys on WITHOUT ROWID tables are not supported`), and
  the physical index T8 ships (`endpoint_rank_key`) IS on a `WITHOUT ROWID`
  `endpoint_rank`. So T12 and T14 cannot both apply to the same table.
- **T14 (WITHOUT ROWID) — blocked.** turso's WR support is insert-only: it
  raises `Parse error: DELETE from WITHOUT ROWID tables is not supported`,
  `…UPDATE of WITHOUT ROWID tables…`, and `CREATE INDEX on WITHOUT ROWID
  tables…`. Every T14 candidate table is row-deleted in normal operation, so
  `WITHOUT ROWID` breaks the write path (the measured −4.9 % disk win is
  unreachable).

Both are **re-openable only on a turso release that lifts the limit**; the
manual ordered deletes stay meanwhile (already one transaction each, so there
is no correctness gap — only duplicated statements).

## Consequences

- The re-key (identity v3) and the column drops ride the schema-15 wipe; the
  pre-alpha wipe policy (decision 4) applies.
- `psl2` is a new dependency; the split is one function so validation, the
  row-build owner and the search predicate cannot drift.
- Retirement: the old sort cycle, the two JSON columns, `core_type`/
  `config_type`/`rank_config`, `host`/`host_type` and the `HostKind`→`HostType`
  map are gone. `WITHOUT ROWID` and the FK cascade are NOT adopted (engine
  limit above), so the manual ordered deletes and the rowid tables stay.

## Verification

Each shipped slice is green at its commit. At the T4/tag-18 HEAD: `cargo
nextest` proto/config/core **740**, db **172**, tui **263**; `cargo check
--workspace --all-targets` clean; clippy delta vs base **none**. The
workspace clippy gate is **pre-existing-red** on this WIP branch (unchanged
files such as `native/src/context.rs`, `tls/src/spec/mod.rs`,
`route/src/compiler/mod.rs`); the rewamp diffs add **zero** clippy errors
(verified against base `5bd2d64`).
