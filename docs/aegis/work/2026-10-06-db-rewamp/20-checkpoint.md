# Checkpoint — DB rewamp

## Snapshot
- root `/home/user/oss/xray-tui`; branch `native-core-stub`; HEAD `8097cb9`.
- Tree **clean**; the workstream is the last 5 commits.

## Commits landed (each green: nextest + workspace check)
| commit | slice |
| --- | --- |
| `544d441` | S1 — spec rev.2, plan, baseline, harness guard, turso planner gate |
| `068bbb6` | drop write-only `transport_data`/`security_data` (−13.6% file) |
| `80111de` | drop `core_type` (per-pair + group) + the form override (D3) |
| `c41e63f` | drop `ConfigType` from identity + schema (D9), `IDENTITY_VERSION` 2→3, golden re-pinned; `rank_config` removed |
| `1f89de5` | lock the Profiles order — delete the sort UI (D1) |
| `8097cb9` | `psl2` dep + `xray_tui_config::domain::split` (the DNS-split owner, D2/D6) |

## Todo map
- **Done:** T0 baseline · T1 harness guard · T2 turso gate · T3 psl2 helper
  (meta row still to wire) · T6 core_type · T7 ConfigType+JSON · T10 sort UI
- **Remaining:** T3-meta (PSL version row) · T4 identity/host model ·
  T5 validation + counted skip · T8 binned law + one index · T9 page order/search ·
  T11 tag-15 wipe · T12 FK cascade · T13 direct reader · T14 gated WITHOUT ROWID ·
  T15 docs/ADR/AGENTS

## Verification at this HEAD
- `cargo check --workspace --all-targets` → 0 errors.
- `cargo nextest`: proto/config/core **732**, db **170**, tui **261** — all green.

## Blockers
- none technical. **Budget**: the remaining slices (psl2 identity, binned law,
  page/search, sort UI, wipe, FK, reader, docs) are each large; the session did
  not have room to finish them.

## Phase 1 DONE (committed 4318c11) — Phase 2 (the swap) remains
**Phase 1 (green):** `Endpoint::is_dns()`/`is_ip()`/`display_host()` accessors,
and the DNS-predicate readers migrated onto them (`ui/profiles` ×3, `ops/ping`,
`ops/events`, `ops/enrich`, `endpoint_rank::dns_unresolved`). Behavior identical.

**Phase 2 (the model swap) — do it by hand, NOT by bulk regex.** Attempted and
reverted this turn. Measured remaining shape after the model change
(`Endpoint`: drop `host`/`host_type`, add `domain`/`sub_domain`):

**The crux (why this is a multi-file, multi-session change):**
`endpoint_essentials(e: &Endpoint)` (config_builder/mod.rs:64, and a copy in
`ops/native_connect.rs:188`) builds the dial host from `e.host`. Once `host` is
gone, an IP host's literal lives only in `endpoint_ip`, so the function must take
the ADDRESSES: `endpoint_essentials(e, &[IpAddr])`, using `e.dns_name()` for a
DNS host and `addresses[0]` for an IP host. That cascades through its five
callers — `build_proxy_outbound` (xray.rs:296, singbox.rs:221), `build`
(connect.rs:418), `run_native_session`/`NativeConnectParams.server`
(native_connect.rs:85), `ping_native.rs:224`, `ui/mod.rs:927` — each of which
must be handed the endpoint's addresses (available as `EndpointRow::resolved_ips`
at the page/connect layer, but the `&Endpoint`-only signatures must change).

- `dns_unresolved_endpoint(host_type, has_address)` → `(domain: &str, has_address)`
  — callers `dns_unresolved(row)` (`&row.endpoint.domain`), `load_raw_endpoints`
  (SELECT `e.domain`), `compute_rank`'s caller (`&endpoint.domain`), and the page
  projection decode. Exotic stays out of tier 5: `!domain.is_empty()` is false
  for it.
- Raw SQL sites: `upsert_endpoints_bulk` (done in the model edit), the typed
  `upsert_endpoint` builder (`.domain`/`.sub_domain`), `rank_host`'s
  `SELECT host FROM endpoints` → reconstruct `subdomain.domain`, the page
  `PAGE_PROJECTION` (`e.host`/`e.host_type` → `e.domain`/`e.sub_domain`) + its
  decode, `export.rs` SELECT + decode + **`ExportScope::predicate()`** (semantic:
  `e.host_type != 'dns'` → `domain != ''`, `IN ('ipv4','ipv6')` → `domain = ''
  AND EXISTS(endpoint_ip)`), and the `endpoint_essentials(&Endpoint)` dial host
  (2 copies) which now needs the ADDRESSES.
- ~37 `Endpoint { … }` fixtures across 14 files, PLUS test helper *signatures*
  that take `host`/`host_type` params.
- **HAZARD (cost me a revert):** a bulk regex for `host: X,\n host_type: Y,`
  ALSO rewrites `host: &str,\n host_type: HostType,` in FN SIGNATURES and
  `host`/`host_type` shorthand in `toasty::create!`. Edit literals individually;
  never regex the pair.
- Bump `SCHEMA_VERSION` to 17.

### T4 surface (the ~18-file reality, from a full scan)
Beyond the files already listed, these read `Endpoint.host`/`host_type` and MUST
move in Phase 2 (or their helper): `xray-tui/src/lib.rs:87` (Edit-form
`address`), `ui/actions_log.rs:50`, `ui/statistics.rs:35`, `ops/profiles.rs:811,912`,
`ops/connect.rs:354,365` (warn `host=`), `ops/ping.rs` (`:173,177` probe addr,
`:2122` fast-probe addr, `:2970,3043,3101,3475,3479` test hosts),
`ops/ping/flow_cost.rs:912,925,931,1075,1255` (the lab's raw `ORDER BY e.host` —
the other half of the lab port), `db/profiles_query.rs:336` (search
`lower(e.host) LIKE`), `db/export.rs:226` (`ORDER BY … e.host`).

### Design notes for Phase 2 / T8
- **`is_ip` cannot be `domain.is_empty()` on `Endpoint`**: `domain` is empty for
  BOTH an IP literal AND an exotic host. Put `is_ip` on `EndpointRow` as
  `domain.is_empty() && !resolved_ips.is_empty()` — which makes the "IP literal →
  `endpoint_ip` at import" writer load-bearing (else `is_ip()` is false for every
  IP host and they fall out of the enrich seed). Exotic stays distinguishable
  (neither `domain` nor an address).
- **No single `Endpoint` host accessor**: a DNS name is `sub_domain + "." +
  domain` (reconstructed) and an IP literal lives in `endpoint_ip` — so
  `display_host`/`dns_name` belong on `EndpointRow` with the addresses threaded,
  never a `&str` off `self.host`. (`display_host` was dead and is removed.)
- **T8 can drop the RAW-column apparatus**: `band`/`rank_host`/`rank_weight` are
  raw `ALTER TABLE` columns only because declaring them on the model "would need
  a schema tag = a wipe". The rewamp already wipes (tag 16, T4/T8 bump again), so
  declare `rank_bin`/`rank_weight`/`rank_domain`/`rank_sub_domain`/`rank_addr`/
  `rank_newest_seen`/`band` as MODEL fields and delete the `ALTER TABLE` loop,
  `backfill_bands`, `RankRow`'s raw-weight split, and the post-write
  `rank_host` `UPDATE … SELECT host`.

## T5 DONE (committed e40afed, fcd41c9)
`validate_host` rejects a DNS name with no registrable domain (psl2 split is
`None`), run BEFORE the `allow_private_ips` gate; `validate_host` is now `pub`
and the Add/Edit form path calls it (one rule for form + import); the
subscription importer's `file_profile` classifies the new message into
`host_validation_count`. Behavior change: a domainless DNS name is rejected even
with `allow_private_ips = true`.

## T4 Phase-2 collapse (the advisory that makes it handable)
- **Fixtures: derive INSIDE the helper, not 53 hand-edits.** `seed_endpoint(conn,
  id, proto, host, host_type, port, seen)` (integration.rs:131, database.rs:2401)
  and `endpoint_struct`/`endpoint`/`endpoint_row` helpers (database.rs:3025,
  export.rs:424, config_builder:263, write_behind:1521, profiles_query:53) keep
  their `(host, host_type)` signature and compute `domain`/`sub_domain` in the
  body — so every call site compiles untouched. Same for the inline
  `toasty::create!(Endpoint { … })` literals: give each a `derived(host,kind)`
  pair. The `EndpointCreate` builder sites: `.domain(..)/.sub_domain(..)`.
- **Address threading is 5 mechanical sites, not a wall.** `connect.rs:151` has
  `row.resolved_ips` in hand — carry `(endpoint, resolved_ips, …)` and update
  `ConfigBuilder::build` → `build_proxy_outbound` (xray.rs:296/singbox.rs:221),
  `native_connect::endpoint_essentials` (:188), `ping_native.rs:224`,
  `ui/mod.rs:927`. `endpoint_essentials(e, addresses)` =
  `if e.is_dns() { e.dns_name() } else { addresses.first() }`.
- `dns_name()` does NOT exist yet (it was never added) — write it on `Endpoint`.

## T14 gate PASSED (a7be3fa)
turso 0.7.2 DOES honour WITHOUT ROWID behind `experimental_without_rowid(true)`
(default builder rejects it); round-trip + ordered scan verified. Implementation
is sequenced AFTER T4: the raw `DROP`+`CREATE … WITHOUT ROWID` DDL must
hand-write the FINAL (post-T4) schema. Also enable the flag on EVERY connection
path (`file_driver`, `export.rs`'s direct builder, test helpers) via
`toasty_driver_turso::Turso::experimental_without_rowid(true)`.

## Next step (exact resume point)
0. **T4 ≡ T8 ≡ T9 are ONE non-green commit.** Measured this turn: dropping
   `Endpoint.host`/`host_type` is ~12 files and the compiler is not the end of
   it — `profiles_query` (search/order by `e.host`), `endpoint_rank`
   (`dns_unresolved_endpoint(host_type, …)`, the rank `bin`), `export`
   (host/host_type), `config_builder`/`native_connect`/`ping_native`
   (`endpoint_essentials` dial host), `ui/profiles` (Address column + `== Dns`
   flags), `enrich`/`events`/`ping` all move. Doing T4 alone leaves the tree
   broken; the correct unit is T4+T8+T9 (host model + binned law + page/search).
   Budget: the largest slice in the plan. The T4 attempt was reverted — HEAD is
   green and committed; nothing is half-done.
1. **T4 — endpoint identity/host model.** Drop `endpoints.host`/`host_type`; add
   `domain`/`sub_domain`; identity = `stable_hash(ascii_name, port)` using
   `xray_tui_config::domain::split(host).ascii` for dns, the literal for ip,
   `("undefined", config_uid)` for exotic; write an IP host's literal as its
   `endpoint_ip` row; derive host-kind (`domain` non-empty → dns; address row →
   ip; else undefined).
   **Consumer surface (must all move in ONE commit — it cannot compile until
   done):**
   - `xray-tui-db`: `models_toasty::Endpoint`, `database.rs`
     (`upsert_endpoints_bulk` SQL + seeds), `profiles_query.rs`
     (`PAGE_PROJECTION` `e.host`/`e.host_type`, search predicate),
     `export.rs` (SELECT + decode), `endpoint_rank.rs`
     (`dns_unresolved_endpoint`, `weight_from_discriminators`, `rank_host`).
   - `xray-tui-core`: `config_builder/mod.rs::endpoint_essentials(&Endpoint)`
     — **needs the addresses** (an IP host's dial host now lives only in
     `endpoint_ip`), so its signature must take them/`EndpointRow`.
   - `xray-tui`: `state.rs::endpoint_from_essentials`, `ops/native_connect.rs`
     (its own `endpoint_essentials` copy + `:85`), `ops/ping_native.rs:224`,
     `ui/mod.rs:902`, `ui/profiles.rs` (Address column **and** the three
     `host_type == Dns` checks at `:459`/`:732`/`:799`), `ops/export.rs:136`.
   - **`host_type` consumers that read the derived kind** (semantic, not just
     compile noise — each replaces `host_type` with the derived predicate):
     `ops/enrich.rs` (`:290`, `:344-345`, `:403-404` the
     `Ipv4|Ipv6 => host.parse(), _ => resolve` branch that decides in-place vs
     resolver — the derived-kind replacement MUST preserve it — and `:633`),
     `ops/events.rs:646` (`== HostType::Dns`), `ops/ping.rs:1735`/`:1742`
     (`plan.endpoint.host_type`).
   - validation: `import_export::validate_host` DNS branch calls `domain::split`
     and STOPS normalizing via `url::Host` (one normalizer).
   - **IP literal → `endpoint_ip` at import (named sub-step).** Today the ONLY
     writer of `endpoint_ip` is `Database::update_endpoint_resolution` (the DNS
     event path); NO import writer touches it, and `spawn_enrich_ip_hosts`
     synthesizes an IP host's address in memory without persisting (its gate is
     `resolved_at_secs.is_some()`, which is None for IP hosts). So the moment
     `endpoints.host` drops, an IP endpoint (74% of the feed) has its literal in
     NEITHER place — dial host, `addr` rank term, and the page address column all
     go empty. T4 MUST add the literal write at EVERY import entry point:
     `state::persist_parsed`, `ops/stream_import.rs`, `ops/subscriptions.rs`.
     Invariant test: an imported IP endpoint has exactly one address row.
   - The **parse boundary** `xray-tui-proto::EndpointEssentials.host/host_type`
     STAYS (the split is the DB layer's job).
   Bump `SCHEMA_VERSION` to 16 again (the model changes).
2. **T5** validation counted-skip (form == import) · **T8/T9** binned law + one
   index + page order/search (needs T4's `domain`/`sub_domain`/`addr`) ·
   **T12** FK cascade · **T13** direct reader · **T14** gated WR ·
   **T15** docs/ADR/AGENTS.

## Verification at this HEAD
- `cargo check --workspace --all-targets` → 0 errors (2 pre-existing native doc warnings).
- nextest: proto/config/core **732**, db **170**, tui **259** — all green.
- Clippy: MY files clean; the workspace gate is red from PRE-EXISTING lints on
  this WIP branch (native/context.rs, tls/spec, route/compiler, proto mod.rs
  len_zero) — untouched by this work.
