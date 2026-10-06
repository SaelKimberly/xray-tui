# Database rewamp — split identity, binned ranking, minimal indexes, direct page read — Design Spec

Status: **draft rev. 2 — awaiting user review** (rev. 1 was the pre-grilling sketch; the
grilling of 2026-10-06 changed identity, the host model, the ranking law, the sort
contract, and the FK posture — see the Challenge Result).
Date: 2026-10-06.
Supersedes/extends: `specs/2026-09-11-profiles-page-query-design.md` (the page),
`specs/2026-09-24-profiles-view-band-design.md` (band), `specs/2026-09-14-profiles-stored-sort-key-design.md`
(the stored keys), ADR 0001 / ADR 0003 / ADR 0010, `specs/2026-10-01-static-config-weight-design.md`.
Authority: `docs/database.md`, `docs/database-manual-sql.md`.
Evidence: measured 2026-10-06 on the real feed — **74,723 endpoints / 146,744 links /
77,395 protocols / 7,721 addresses** (`data.db`, `user_version=14`). The §9 **timings**
are the SQLite planner (python `sqlite3`) on a copy; the **index SELECTION is now verified
on turso 0.7.2** via `crates/xray-tui-db/tests/turso_planner.rs` (db-rewamp T2): Active/Purgatory
= `SEARCH … USING INDEX endpoint_rank_key (band=?)`, All `(band, key)` = `SCAN … USING COVERING
INDEX`, scope-bin = `SCAN … COVERING`, reband = `SEARCH endpoint_rank_window (band=? AND
rank_newest_seen<?)`, and the `band IN (0,1)` trap = `USE SORTER FOR ORDER BY` (turso's name for
the filesort). Only the turso **timings** remain to be captured (§6/§9).

## 1. Problem

Five distinct defects, each measured (`docs/database.md`, AGENTS decision 23, §9):

1. **Sort parametrization multiplies columns and indexes.** Every non-law sort is a
   materialized `endpoint_rank` column + index (`rank_host`/`endpoint_rank_band_host`,
   `rank_speed`, `rank_traffic`, `rank_config`, `rank_display_seen`) written on every rank
   refresh (ping result, import chunk, DNS resolution).
2. **The Active-view page filesorts.** The locked default already reads
   `endpoint_rank_band_window (band=?)` + `USE TEMP B-TREE FOR ORDER BY` (§9.1) — the cost
   scales with the Active set, not the page.
3. **The page's cost is toasty's execution layer**, not the plan: ~98 ms through toasty vs
   ~1.2 ms for identical raw SQL (AGENTS decision 23).
4. **The endpoint model has a host/endpoint_ip asymmetry and dead duplication.**
   `endpoints.host`/`host_type` hold the identity string; `endpoint_ip` holds *some*
   addresses (and, as legacy residue, IP hosts' literals); `protocols.transport_data` +
   `security_data` are write-only copies of parts of `protocols.config` (11.66 MB, §9.6).
5. **The ranking law uses raw latency and no address term**, so it can neither bucket
   quality nor order by address without a second index.

## 2. Decisions (user, 2026-10-06)

- **D1 — One order.** The Profiles tab renders exactly the new ranking law (§3.1). `SortColumn`
  and the sort cycle are deleted; the tab has no sort parameters.
- **D2 — DNS split by eTLD+1 (psl2), and the split is the identity input (option C).**
  `psl2 = 0.1.31` (idna default); use **`psl2::analyze(host)`** — one normalization
  returning the ASCII name + registrable domain + subdomain (calling
  `registrable_domain` and `subdomain` separately normalizes twice and can drift).
  A DNS name with no registrable domain is rejected at validation (per-profile,
  counted skip — option A), and the **same rule applies to the form as to imports**
  (D9). **`psl2::psl_version()` is an identity input**: the embedded list is
  republished when upstream changes, so a `cargo update` that moves it can
  re-key or reject hosts with no code change. Record the version (one-row meta,
  the `rank_weight_meta` pattern) and treat a mismatch as an identity change to
  surface — a loud warning plus re-import, the same class as any identity re-key,
  never a silent shift.
- **D3 — Drop `core_type`** (`profile_stats.core_type`, `groups.core_type`); connect derives
  the core via `resolve_core(kind, protocol-level override, ss_method)`.
- **D4 — The page read bypasses toasty** (direct `turso::Connection`).
- **D5 — `endpoint_rank` stays a side table**, reshaped (§3.2).
- **D6 — Indexed prefix search** on `domain` / `sub_domain` / IP (§3.8).
- **D7 — Schema bump 14 → 15 (wipe).**
- **D8 — Drop `protocols.transport_data` + `security_data`** (redundant with `config`).
- **D9 — One rule for every import path.** Form and imports validate and key identically;
  `ConfigType::{ShareUrl, Form}` leaves identity and the schema.
- **D10 — Drop `endpoints.host` + `host_type`; every address lives in `endpoint_ip`.**
  `endpoints` carries `domain` + `sub_domain`; the IP/dispatch kind is *derived* (§3.4).
- **D11 — Binned ranking law** (§3.1): 12 delay bins (real above fast) + untested + real-err
  + fast-err + dns-err + purged; recency dropped from the key; materialized whole key on
  `endpoint_rank`; **one** covering index; the All view orders by `(band, key)`.
- **D12 — Foreign keys: adopt `REFERENCES … ON DELETE CASCADE`** and remove the manual
  ordered deletes (§4.2).
- **WITHOUT ROWID** — apply only if turso verifies the experimental flag end-to-end (§4.1).

## 3. Mechanism

### 3.1 Ranking law (bins)

`key = (bin, ⌐weight, domain, sub_domain, addr, endpoint_id)` — ascending; `⌐weight` is
`u64::MAX - packed` (higher static weight leads). `bin` (better → worse):

```
 0 real<50   1 real<100  2 real<250  3 real<500  4 real<1000  5 real≥1000
 6 fast<50   7 fast<100  8 fast<250  9 fast<500 10 fast<1000 11 fast≥1000
12 untested  13 real-err  14 fast-err  15 dns-err  16 purged
```

- A real measurement stays above a fast one of any delay (D11 keeps the "verified tunnel
  beats unverified handshake" property; only the *bucketing* is new).
- `bin` folds the old `rank_dns`+`rank_tier`+`rank_latency` into one integer.
- Recency (`rank_seen`) and `rank_display_seen` leave the key — the view band already
  separates stale from Active.
- `addr` (`rank_addr`) = the endpoint's **own** packed literal key for an IP host
  (its `endpoint_ip` row), a fixed sentinel for a DNS host. **Not `min(ip_key)`** —
  a DNS host's lowest resolved key is unstable under re-resolution, so it would churn
  the stored ordering key on every DNS refresh; the own-literal sentinel keeps the key
  stable and still orders IP hosts by address inside a bin+weight tie.
- `endpoint_id` is the final term (total order even for same-domain/port variants).
- The Test **cell** still shows the exact delay (colored by threshold); only the order uses bins.

### 3.2 `endpoint_rank` shape and index

Columns become: `endpoint_id` (PK), `rank_bin`, `rank_weight`, `rank_domain`,
`rank_sub_domain`, `rank_addr`, `rank_newest_seen`, `band`. Dropped: `rank_dns`, `rank_tier`,
`rank_latency`, `rank_seen`, `rank_protocol`, `rank_display_seen`, `rank_speed`,
`rank_traffic`, `rank_config`, `rank_host`.

**One** raw covering index (C2 — toasty `#[index]` is single-column):

```
endpoint_rank_key(band, rank_bin, rank_weight DESC, rank_domain, rank_sub_domain, rank_addr, endpoint_id)
```

Access paths (all measured SQLite, §9.2 — re-verify on turso):
- Active `WHERE band=0 ORDER BY key` → covering seek.
- Purgatory `WHERE band=1 ORDER BY key` → covering seek.
- All / `PlanScope::All` walk `WHERE 1=1 ORDER BY band, key` → covering scan, **no temp b-tree**.
  A literal `band IN (0,1) ORDER BY key` must NOT be used — it filesorts (§9.2).
- Scope walks (`rank_bin IN (…) ORDER BY band, key`) → the same covering **scan** with the
  bin predicate applied inline (verified, §9.2) — so `endpoint_rank_test_v2` is not needed.
- The reband sweep keeps `(band, rank_newest_seen)`.

Dropped indexes: `endpoint_rank_test`, `endpoint_rank_test_v2`, `endpoint_rank_band_host`,
`endpoint_rank_window`. **`endpoint_ip_by_key` STAYS** — it served the retired Ip sort *and*
now serves the IP/CIDR search range (§3.8). Model drops: `profile_stats.protocol_id` and
`endpoint_groups.endpoint_id` (PK-prefix redundant).

### 3.3 Sort contract

`types::SortColumn` deleted; `ui/mod.rs` sort cycle + key-11 reset deleted; `ops/profiles.rs`
`page_sort` deleted; `profiles_query::PageSort` collapses to the law (rename `PageOrder`),
with the view predicate selecting the `(band, …)` vs `(band, key)` order.

### 3.4 Endpoint model, identity, validation

- `endpoints`: `id`, `domain`, `sub_domain`, `port`, `ports`, `last_source`,
  `manual_protocol_override`, `resolved_at`, `created_at`. `host`/`host_type` **gone**.
- **Identity** (option C — the split is the input):
  - dns: `stable_hash(canonical_name, port)`, `canonical_name = sub_domain + "." + domain`
    (psl2/idna A-label, lowercased) — same *principle* as today's `stable_hash(host, port)`.
  - ip: `stable_hash(address_text, port)`.
  - exotic/`undefined` (Redirect/TProxy/Mixed, empty host): the existing special case
    `stable_hash("undefined", config_uid)` is kept.
- **Every address in `endpoint_ip`** — an IPv4/IPv6 host writes its literal as one
  `endpoint_ip` row at import, so the dial and the `addr` key term read one table. `host_type`
  is **derived**: `domain` non-empty → dns; else an `endpoint_ip` row exists → ip (family
  from the packed key's first byte); else undefined.
- **The parse boundary keeps `host`.** `xray-tui-proto::EndpointEssentials` still carries
  `host` + `host_type: HostKind` — it is the *parse* output every consumer reads. The split
  and the address materialization are the **DB layer's** job (`state::endpoint_from_essentials`),
  and the three `host_type → HostKind` reconstruction sites must rebuild `host`/`HostKind`
  from `domain`+`sub_domain` (dns) or the `endpoint_ip` literal (ip):
  `config_builder/mod.rs:67`, `ops/native_connect.rs:191`, `export.rs:304` (plus its SQL at
  :210/:217, and `ops/export.rs:136` which rewrites `endpoint.host_type`). **One owner per
  direction**: the proto crate owns `host`, the DB layer owns the split — no builder invents
  its own.
- **`undefined`/exotic rows survive.** `domain`/`sub_domain` are empty-tolerant (not `NOT
  NULL`-strict in a way that breaks an empty host); the derived kind is `undefined` for an
  empty host with no address; identity stays `stable_hash("undefined", config_uid)`. 0 in the
  current feed, but legal (Redirect/TProxy/Mixed) and in scope.
- **Reject rule, verified against psl2's PRIVATE section.** `psl2::analyze` returns `None`
  registrable for a bare public suffix, so `github.io` / `blogspot.com` / `pages.dev` are
  rejected while `foo.github.io` / `foo.blogspot.com` are kept — the rule fires for
  single-label and bare-suffix hosts, not for hosts under private suffixes. Pin both with a test.
- **Validation** (`xray-tui-config::import_export::validate_host`, the shared owner): a DNS
  name must yield a psl2 registrable domain, else `ImportError::Validation`. IP hosts and
  exotic kinds are exempt. Per-profile — the import continues and **counts/reports the skip**
  (never silent). The form path (`ops/profiles.rs::build_typed_config`) routes through the
  **same** validator (D9).
- **One split helper** (`psl2`-based) is shared by validation and the row-build owner
  (`state::endpoint_from_essentials`) so the accepted name and the stored/identity name
  cannot drift.
- **Behavior change to flag:** identity becomes case-insensitive (idna lowercases), so the
  801 uppercase hosts in the feed merge with their lowercase twins. Deliberate (the dedup
  fix), not a regression.

### 3.5 `core_type` removal
As rev. 1 §3.4 — model, upsert SQL, projections, `connect.rs`, `ops/profiles.rs`
(`resolved_core`, `set_core_field`, form field). If the `ProtocolEssentials::write_identity`
term is also dropped, bump `IDENTITY_VERSION` (free — the wipe is happening).

### 3.6 Redundant `transport.data` / `security.data`
As rev. 1 §3.5. `protocol_from_parsed` builds both from the same `config`; no production
reader exists (`ConfigBuilder::protocol_config` reads `config`). Drop both columns;
`load_protocol_with_config` drops the two `.include(…)`; file **107.6 → 93.0 MB (−13.6%)** (§9.6).

### 3.7 `ConfigType` removal
`protocols.config_type` + `profile_stats.config_type` and the `ConfigType` enum leave the
model and identity; the AddServer form's `config_type` field goes.

### 3.8 Search
`PageRequest.search` becomes a prefix predicate over three index-servable columns: lowercased
`domain`, lowercased `sub_domain` (range `>= x AND < x||'\xff'`), and — when the term parses
as an IP or CIDR — a byte-range on the packed `endpoint_ip.ip_key` (family byte + octets; no
text column is added — two spellings of one address is the rejected shape). Exact
`LIKE '%…%'` full scans are retired.

### 3.9 Direct page reader
New `crates/xray-tui-db/src/page_reader.rs`: a **long-lived** `turso::Connection` created
once beside the main `toasty::Db` (NOT a per-load `Builder::build`, which would pay the open
cost on every page tick and erase the win), its own guard, `BEGIN` matching the handle's
journal mode (`uses_concurrent_writes()` → MVCC vs WAL, as `export.rs:143`), and the three
statements (count / ordered ids / hydration) with endpoint ids inlined as integer literals
(ADR 0001 rule). Reopen only on error. `profiles_query.rs` keeps the SQL builders and the
typed `load_page_rows` parity oracle.

### 3.10 Schema tag 15
`database.rs` `SCHEMA_VERSION` 14 → 15 (wipe). Docs updated: `docs/database.md`,
`docs/database-manual-sql.md` (new raw sites: `endpoint_rank_key`, the `endpoints(domain,
sub_domain)` search index, the FK DDL, any WITHOUT ROWID DDL).

## 4. Storage-shape questions (measured)

### 4.1 WITHOUT ROWID — apply only if turso verifies

Measured 2026-10-06 **per table, isolated** (§9.4). Every integer/composite-key
table was checked — rev. 1 listed five and omitted `endpoint_groups`:

| Table | Key | Δdisk | Verdict |
| --- | --- | --- | --- |
| `endpoint_rank` | int | −1.7 MB | candidate |
| `endpoints` | int | −1.5 MB | candidate; hydration join 0.331 → 0.255 ms (−23%) |
| `endpoint_groups` | int+text | −1.4 MB | **candidate (omitted from rev. 1)** |
| `profile_stats` | int+int | −0.5 MB | candidate |
| `endpoint_ip` | int+blob | −0.2 MB | candidate |
| `protocols` | int | **+8.1 MB** | **reject — measured loss (large JSON rows)** |
| `groups`, `routing_rules`, `dns_settings`, `route_probes` | TEXT PK, tiny/empty | ~0 | not applicable |

Five candidates together: **≈ −5.3 MB (−4.9%)** — for the experimental flag + hand
DDL on five toasty-owned tables. turso 0.7.2 gates it behind
`experimental_without_rowid` (default off); the driver exposes it
(`Turso::experimental_without_rowid`, `toasty-driver-turso-0.11.0/src/lib.rs:706`)
but it must be set on **every** connection path (`file_driver()`, `export.rs`'s direct
builder, test helpers); `push_schema` cannot emit it, so those tables need raw
`DROP`+`CREATE` DDL at the wipe; typed toasty read/write against a WITHOUT ROWID
table must be verified. **Take only if a turso run proves it**; otherwise defer.

### 4.2 Foreign keys — adopt cascade (D12)

`foreign_keys=ON` is already set but **inert** (no `REFERENCES`), and it is set **only in
`open()`/`in_memory()`, not in `Database::conn()`** — `foreign_keys` is per-connection, so on
the pooled connections the delete paths actually use, enforcement is OFF. Under D12:

1. Add `PRAGMA foreign_keys=ON` beside `busy_timeout`/`synchronous` in `conn()`
   (`database.rs:378-393`), or the cascade silently never fires.
2. Add `REFERENCES … ON DELETE CASCADE` on `endpoint_ip.endpoint_id`,
   `profile_stats.protocol_id`/`endpoint_id`, `endpoint_groups.endpoint_id`/`group_id`,
   `endpoint_rank.endpoint_id`.
3. **Creation mechanism:** SQLite cannot `ALTER`-add a constraint and `push_schema` emits no
   `REFERENCES` — so on the tag-15 recreate (fresh, empty tables) `DROP`+`CREATE … REFERENCES`
   the four child tables, **gated on "schema just created"**, never on every `ensure` open.
4. Delete the manual ordered deletes (`delete_endpoints`, `purge_expired`, `delete_group`,
   `endpoint_ip::delete_for`) so the DB is the single owner of referential integrity.

FK enforcement measured free (347 vs 358 ms / 200k child upserts, §9.5). Test that a cascade
fires **through a pooled connection** (`conn()`), not only the `open()` connection.

### 4.3 `host` vs `endpoint_ip`
Superseded by D10: every address now lives in `endpoint_ip`, including IP hosts' literals.
The 6,314 IP-owned rows in the current DB are legacy residue; the wipe removes them, and the
import writes the literal explicitly.

## 5. Components / ownership

| Change | Owner |
| --- | --- |
| delete `SortColumn`, sort cycle, `page_sort`, `PageSort` variants | `types.rs`, `ui/mod.rs`, `ops/profiles.rs`, `profiles_query.rs` |
| binned law + `bin` compute | `endpoint_rank.rs` (`RankLink::key`, `compute_rank`, `write`) |
| `endpoint_rank_key` (new) + index/column drops | `endpoint_rank.rs::ensure_in` (raw, C2) |
| `domain`/`sub_domain` split + identity + validation (ONE `psl2::analyze` helper) | `state.rs`, `import_export.rs`, new psl2 helper |
| parse-boundary consumers reconstruct `host`/`HostKind` | `config_builder/mod.rs`, `native_connect.rs`, `export.rs`, `ops/export.rs` |
| PSL-version meta row + mismatch surfacing | `database.rs` / `endpoint_rank`-style meta |
| `endpoint_ip` all-addresses write for IP hosts | `state.rs` / `ops/enrich.rs` / import |
| `core_type` removal | `models_toasty.rs`, `database.rs`, `connect.rs`, `profiles.rs`, `export.rs` |
| `ConfigType` removal | same set + `import_export.rs` |
| drop `transport.data`/`security.data` | `models_toasty.rs`, `database.rs`, `state.rs` |
| prefix search | `profiles_query.rs::base_from_where` |
| long-lived direct reader | new `page_reader.rs` |
| FK cascade DDL (fresh-schema DROP+CREATE) + `conn()` pragma + delete the manual ordered deletes | `database.rs` (raw) |
| WITHOUT ROWID (gated) | `database.rs` open path, raw DDL |
| `SCHEMA_VERSION` 15 | `database.rs` |
| **AGENTS.md decision updates** | decisions 4 (tag), 11f/20 (`core_type`), 15/16 (sort cycle, ranking law), 21 (index/ordering inventory) |

## 6. Acceptance (observable, re-verified on turso at 74k)

- On a **direct `turso::Connection`** (`EXPLAIN QUERY PLAN`) and on the app's real timing:
  Active/Purgatory `SEARCH … USING COVERING INDEX endpoint_rank_key`, All `SCAN … USING
  COVERING INDEX` — **no `USE TEMP B-TREE`** in any of the three; if turso will not pick the
  index without stats, add `ANALYZE` (or the needed hint) to the design, not just SQLite's number.
- Page ≤ ~1 ms raw / within the render tick budget app-side (baseline 19.9 → 73.6 ms SQLite).
- The scope/feed walk stays a covering scan (no filesort).
- Hot-table index count equals §3.2 "after"; `endpoints` has no `host`/`host_type`.
- Search is an index range (no full scan); an IP/CIDR term hits the packed key.
- `protocols` has no `transport_data`/`security_data`/`core_type`/`config_type`; file ≤ ~95 MB
  (baseline 107.6 MB).
- A DM without a psl2 registrable domain is rejected per-profile with a **counted** skip, on
  both the import and form paths; the two paths accept/reject identically.
- Every address (dns resolved + ip literal) is in `endpoint_ip`; `host_kind` derived matches
  today's `host_type` for every feed row.
- FK cascade deletes exactly what the manual ordered deletes did (parity test).
- `flow_cost` still builds: the lab's `PageSort::Id`/`profiles_walk_page` uses are ported or
  kept lab-only in the same slice.

## 7. Risks

- **turso ≠ SQLite.** All §9 numbers are SQLite; the production engine's plan may differ.
  Re-verify before the number is quoted; the `band_law`/`endpoint_rank_key` win is only real
  if turso picks the index.
- **Wipe** (tag 15) — accepted; also removes the 6,314 legacy `endpoint_ip` rows.
- **Identity re-key + case merge.** Under C every endpoint re-keys and the 801 uppercase
  hosts merge. Free under the wipe; deliberate.
- **psl2 dependency + version + rejection blast radius.** New crate (new hakari
  set); a counted skip is mandatory or a 70k import loses profiles invisibly;
  and `psl2::psl_version()` is an identity input — a routine crate bump can
  re-key or reject hosts with no code change, so the version is stamped in a meta
  row and a mismatch is surfaced (re-import), never silent.
- **`rank_addr` source.** The rank refresh reads the endpoint's own literal packed key
  (IP hosts) from `endpoint_ip` (it already reads the dns-band `EXISTS`); a DNS host's
  term is a constant, so the key is stable under re-resolution. Measure the refresh cost.
- **Binned real/fast order** is a user-visible semantics change (keeps real above fast by
  D11 option B).
- **Baseline must be recorded at HEAD** before the first DDL — several deletions are
  irreversible in place, and the wipe destroys the "before".

## 8. ADR signal

Durable surfaces: the ranking law (bins), the endpoint identity + host model, the
`endpoint_rank` shape + single index, the sort contract, `core_type`/`ConfigType` retirement,
the all-addresses-in-`endpoint_ip` model, the FK cascade, and the page read path. Warrants an
ADR extending ADR 0003/0010 + a retirement note on ADR 0001. Write on acceptance.

## 9. Measurements — **SQLite planner, NOT turso** (74,723 endpoints / 146,744 links; copy of `data.db`)

### 9.1 Page (Active, current law) — the filesort
| | plan | offset 0 | offset 70,000 |
| --- | --- | --- | --- |
| before | `SEARCH … band_window (band=?)` + **TEMP B-TREE** | 19.9 ms | **73.6 ms** |
| after `band_law` | `SEARCH … COVERING band_law (band=?)`, no sort | 0.037 ms | **1.0 ms** |

### 9.2 New key / one-index shape
| query | plan | ms |
| --- | --- | --- |
| Active `band=0 ORDER BY key` | `COVERING endpoint_rank_key` | 0.25 |
| All `ORDER BY band, key` | `SCAN COVERING endpoint_rank_key` | 0.13 |
| All `band IN (0,1) ORDER BY key` | `+ USE TEMP B-TREE` | **5.47** (must not be used) |
| scope `rank_bin IN (…) ORDER BY band, key` | `SCAN COVERING endpoint_rank_key` (filter inline) | 0.15 |
| IP search range on `ip_key` | `SEARCH COVERING endpoint_ip_by_key` | — |

### 9.3 Writes
| | before | after |
| --- | --- | --- |
| rank refresh (−3 sort indexes) | 5.7 µs | 4.2 µs |
| link upsert (−1 redundant index) | 5.4 µs | 5.3 µs (noise) |

### 9.4 WITHOUT ROWID (per-table isolated Δ, cumulative run)
| table | Δdisk |
| --- | --- |
| `endpoint_rank` | −1.7 MB |
| `endpoints` | −1.5 MB (hydration join 0.331 → 0.255 ms) |
| `endpoint_groups` | −1.4 MB |
| `profile_stats` | −0.5 MB |
| `endpoint_ip` | −0.2 MB |
| `protocols` | **+8.1 MB** (worse) |
| `groups`/`routing_rules`/`dns_settings`/`route_probes` | ~0 (TEXT PK, tiny/empty) |

### 9.5 Foreign keys and host_type
FK ON vs OFF, 200k child upserts: 347 vs 358 ms (noise). `PRAGMA foreign_key_list` empty
(no `REFERENCES`). `host_type`: ipv4 56,212 / dns 18,084 / ipv6 427; 0 IP endpoints carry
`resolved_at`; 6,314 legacy IP-owned `endpoint_ip` rows. 801 uppercase dns hosts; 23
single-label; 30 trailing-dot; multi-part TLDs present.

### 9.6 Redundant transport/security JSON
`DROP COLUMN transport_data, security_data` + `VACUUM`: **107.6 → 93.0 MB (−13.6%)**.
`transport_data` 6.37 MB + `security_data` 5.29 MB vs `config` 24.49 MB.

## 10. Non-goals

- Routing/DNS/group tables, import parsing internals, the native core.
- No keyset pagination (the covering seek makes the page flat enough; revisit only if the
  app-side page exceeds the tick budget on turso).
- No folding of `endpoint_rank` into `endpoints` (D5).
- No `endpoint_ip` text column (rejected — one spelling of an address).
