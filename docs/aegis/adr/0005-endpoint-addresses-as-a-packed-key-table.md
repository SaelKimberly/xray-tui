# 0005 — Resolved addresses are a packed-key table, not a JSON column

Date: 2026-09-15
Status: accepted
Related: decisions 4 (schema tag / wipe), 11 (identity), 13 (enrichment), 16
(the ordering law), 21 (SQL page + ADR 0003's materialized keys)
Spec: `docs/aegis/specs/2026-09-15-endpoint-ip-storage-design.md`

## Decision

A DNS endpoint's resolved addresses live in their own table,
`endpoint_ip(endpoint_id, ip_key)`, one row per address, PK
`(endpoint_id, ip_key)`, with the covering index
`endpoint_ip_by_key(ip_key, endpoint_id)`.

`ip_key` is the address itself in a **sortable packed encoding**: a family byte
(`4` = IPv4, `6` = IPv6 — the `IpAddr` discriminant) followed by the
big-endian octets, 5 or 17 bytes. Byte order is address order and every IPv4
sorts before every IPv6, so the B-tree (memcmp) orders addresses correctly with
no operator, no collation, and no engine feature.

`endpoints.resolved_as` (a JSON array in a TEXT column) is **gone**. Its
sibling `endpoints.resolved_at` **stays**: it is the TTL gate for a lookup
*attempt*, which is a different fact — a failed lookup stamps it and leaves the
address set empty, and that empty set is what the tier-5 band reads.

Consumers take `Vec<IpAddr>` end to end (`EndpointRow::resolved_ips`,
`Database::update_endpoint_resolution`), so no text form is parsed or produced
on any path except display.

## Why

- **The JSON column could not be queried or ordered.** Sorting endpoints by IP
  was impossible; so was "which endpoints resolve to this address", or any
  relational use of the set. A table with an index on the packed key makes both
  index-driven.
- **The engine's own `inet` type cannot do the sort the goal needs.** Turso
  0.7.2 declares it with no `OPERATOR '<'`, so `ORDER BY ip` and
  `CREATE INDEX … (ip)` are parse errors; declaring the operator (the docs'
  mechanism) makes them legal but orders the *text* — `10.0.0.1 < 9.0.0.1` —
  and a function comparator is silently ignored unless its name is one of four
  hardcoded ones. Measured, with the probe transcript in the spec (§3).
- **The packed key is the smallest correct *per-address* form** — 5/17 bytes,
  against the JSON array's ~20–40 characters plus a parse on every read. It is
  NOT a storage reduction overall, and the record says so: a child row carries
  two B-trees (the PK autoindex and the covering index), so the file grows on a
  1–3-address feed — measured 4,026,368 → 4,325,376 bytes (+7.4%) at 6,528
  addresses. The win is capability (query, join, dedup, correct order) with a
  bounded per-address cost, not bytes; the live database holds ~15 addresses
  today, so nothing here is a measured win on current data.
- **The key is derived, the writer is single, the source is typed.** The
  address is the fact; the key is its order-preserving rendering, produced by
  `endpoint_ip::key_of` at the one site that writes addresses, and validated
  there (`key_of_str` refuses anything that is not an address). Storing only
  the packed form means the two cannot disagree, which is why there is no
  parallel text column.

## Reachability

Everything rejected here was re-checked on the route that actually reaches the
engine — the driver's `experimental_custom_types`/`experimental_materialized_views`
flags plus raw DDL at open (toasty emits neither STRICT tables nor `CREATE TYPE`,
and toasty cannot emit STRICT, which is what a custom type needs to be live —
its column-type hatch is real but reaches only a *name*: `#[column(type = "inet")]`
compiles over a `String` and emits `"addr" inet NOT NULL` into a non-STRICT
`CREATE TABLE`, where `'not-an-ip'` inserts and `ORDER BY` is lexicographic; the
typed variant `#[column(type = inet)]` is refused outright (compile error over a
`String`, `native_inet: false` downstream). On that
route `inet` still cannot be ordered or indexed, an MV still cannot take the
page query's shape, and validation does work — so the verdicts are properties
of the engine, not of how this app happens to open it. Spec §5b has the
transcript.

## Alternatives considered

1. **`inet` column, as proposed.** Rejected: cannot be ordered or indexed by
   the engine; needs STRICT (which `toasty` cannot emit) and the
   `experimental_custom_types` switch; and even with a hand-declared
   `OPERATOR '<'` its order is lexicographic, i.e. wrong for the stated goal.
2. **`smallint` for `port`.** Rejected: needs STRICT for the range check
   (inert otherwise), saves no storage (SQLite INTEGER is variable-length
   already), and cannot be expressed by `toasty`'s schema builder.
3. **A materialized view for the page query.** Rejected: the required shapes
   are unsupported (LEFT JOIN, LIMIT, window functions, scalar subqueries), an
   index on an MV is ignored by the planner (`SCAN mv`), and maintenance is
   charged per write — 425 µs per insert, i.e. ~13 s added to a 30k-link batch
   to save ~10 ms of read. `endpoint_rank` already is this page's materialized
   derived state.
4. **A `USING` custom index method for addresses.** Rejected: the only shipped
   methods are `fts` and a vector index; `USING btree`/`USING inet` are
   `unknown module name`.
5. **Keeping a text column beside the key** (so `group_concat(ip)` could carry
   the display form to the page). Rejected as a duplicated fact: the page's
   decoder renders the address from the key instead, and the packed key is what
   the aggregate carries (`hex(ip_key)`), for +3.8 ms on the hydrate statement.
6. **`array_agg(ip_key)`** (a real array in one column, no hex). Rejected: gated
   behind `--experimental-custom-types` ("Array features require
   --experimental-custom-types flag"), and it returns an internal SQLite record
   payload that the driver hands over as an opaque blob — decoding it would
   mean re-implementing the record format here.
7. **A plain `LEFT JOIN endpoint_ip`** (raw blobs, no encoding). Measured
   equivalent (30.1 ms vs 29.6–30.1 ms) but it inflates every link row per
   address and makes the decoder accumulate; the aggregate keeps one row per
   link.

## Consequences

- **Schema tag 10.** A v9 file is WIPED (decision 4), not migrated: the old
  column is gone, and the enrichment pipeline re-resolves on the next pass.
- **The page's hydrate statement is ~3.8 ms slower** (40.2 → 43.9 ms of a
  48.1 → 51.7 ms page load, +7%) on the 7,656-endpoint reference feed, and the
  file grows ~7.5% (4.03 → 4.33 MB) — the price of the addresses being
  relational. The escalation, if it ever matters, is in the spec §7.
- **A new sort exists**: `PageSort::Ip` (165.7 ms per fetch, flat across
  offsets) — in the same band as the pre-existing Address/Port sorts
  (166–220 ms, ADR 0003). It is not index-driven: the ordering key is a
  correlated aggregate. Making it index-driven means materializing a `rank_ip`
  column on `endpoint_rank` — deliberately not taken (a third B-tree and more
  per-write work for a sort that is not the default).
- **Display order is canonical**, not the resolver's: IPv4 by address, then
  IPv6. The first entry of `EndpointInfo::resolved_ips` is what the outbound
  probe picks, so this is an observable change — a stable one, since the order
  no longer depends on the A/AAAA response order.
- **The ordering law's input changed shape** (`dns_unresolved_endpoint` takes
  "has an address"), which is the same question the tier-5 band always asked;
  the rank refresh reads it with one correlated `EXISTS` in the statement that
  already loads the endpoints it touches.
- Deletions keep their cascade: `purge_expired` and `delete_endpoints` remove
  the address rows in the same transaction (toasty emits no `REFERENCES`, so
  nothing cascades on this engine).
- Verification: `endpoint_ip`'s own codec tests (round-trip, byte order ==
  address order, rejections), the parity test between the typed hydration and
  the raw projection (now comparing `Vec<IpAddr>`), the 8-sort × 2-direction
  ordering oracle plus the anchor and paging goldens, the statement-vs-schema
  drift guard, and the schema-tag tests.
