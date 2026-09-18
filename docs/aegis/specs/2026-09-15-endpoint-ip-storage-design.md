# Endpoint addresses: storage, ordering, and what Turso 0.7.2 actually offers

Date: 2026-09-15
Status: implemented (schema tag 10)
ADR: `docs/aegis/adr/0005-endpoint-addresses-as-a-packed-key-table.md`
Method: every engine claim below was produced by running the pinned engine
(`turso_core` 0.7.2, `toasty` 0.10, `toasty-driver-turso` 0.10) on this
machine, from throwaway probe binaries listed in §8. Source cites are to
`~/.cargo/registry/.../turso_core-0.7.2/`. Numbers are debug-build; the feed is
7,656 endpoints × 2 links with 6,528 addresses unless stated.

## 1. The ask, and the three corrections

The proposal was: split resolved addresses out of `endpoints` into an
`endpoint_ip` table with an `inet` column, use `smallint` for ports, use custom
indexes and a materialized view for the big page query, and sort endpoints by
IP correctly.

What the engine actually supports changes three of those:

1. **`inet` cannot order or index.** Turso 0.7.2 declares it with no `<`
   operator:
   `CREATE TYPE inet(value text) BASE text ENCODE validate_ipaddr(value) DECODE value`
   (`turso_core-0.7.2/schema.rs:840`). Both `ORDER BY ip` and
   `CREATE INDEX … (ip)` are parse errors — the engine says so explicitly:
   `cannot ORDER BY column 'ip' of type 'inet': type does not declare OPERATOR '<'`
   (`translate/order_by.rs:170-184`, `translate/index.rs:170`). Declaring the
   operator makes them legal but orders the TEXT form, i.e. `10.0.0.1 < 9.0.0.1`
   — see §3. So `inet` is a *validator*, and only inside a STRICT table.
2. **`smallint` needs STRICT, and toasty cannot emit STRICT.** The port column
   already stores an INTEGER (SQLite INTEGER is a variable-length encoding, so
   `smallint` saves nothing); the only thing it adds is range validation, and
   on a non-STRICT table a custom-type name is inert ("Custom types work only
   with STRICT tables"). `toasty` serializes its own `CREATE TABLE`
   (`toasty-driver-turso/src/lib.rs:74-96`) with **no STRICT flag** — and STRICT
   is exactly what it cannot reach. The column-type hatch does exist and does
   reach a *name*: `#[column(type = "inet")]` compiles over a `String` field and
   emits `"addr" inet NOT NULL` verbatim. It buys nothing, because toasty's
   `CREATE TABLE` is not STRICT, where a custom type is inert — measured:
   `'not-an-ip'` inserts fine into that column, `ORDER BY addr` returns
   `2, 3, 1` (i.e. `10.0.0.1, 9.0.0.1, not-an-ip`), and an index on it is a
   plain text index. Measured: with custom
   types disabled — the state the app runs in — `CREATE TABLE … STRICT` with a
   `smallint` column fails outright: `unknown datatype for s.port: "smallint"`.
3. **The materialized view cannot express the page query, and it would cost
   more than it saves.** See §4.

## 2. What was implemented

- `endpoint_ip(endpoint_id, ip_key)` — a **BLOB** column holding the packed
  address, PK `(endpoint_id, ip_key)`, plus a covering index
  `endpoint_ip_by_key(ip_key, endpoint_id)`. One row per address, so the set is
  deduped by the database and a repeated address in one DNS answer cannot
  collide.
- `endpoints.resolved_as` (a JSON array in TEXT) is **gone**;
  `endpoints.resolved_at` stays (it is the TTL gate for an *attempt*, distinct
  from "has an address" — a failed lookup stamps it and leaves the set empty).
- The packed key: family byte (`4` = IPv4, `6` = IPv6, matching
  `IpAddr::V4`/`V6`) then the big-endian octets — 5 or 17 bytes. Byte order is
  address order, IPv4 before IPv6, and the B-tree compares BLOBs with memcmp.
  `IpAddr` values are 17 bytes inline and own their bytes, so the decode is a
  4/16-byte copy with no parse and no allocation (the only allocation on the
  read path belongs to the driver's `Vec<u8>`).
- `EndpointRow.resolved_ips: Vec<IpAddr>`, and the write path takes
  `Vec<IpAddr>` — the JSON column forced a text → `IpAddr` parse on every seed;
  now nothing renders or parses addresses except the display.
- A new page sort (`PageSort::Ip`, the TUI's sort cycle includes it): the
  lowest resolved address, sentinel `x'ff'` when there is none. It is the only
  sort the JSON column could not express at all.

## 3. The `inet` question, measured

The docs' ordering mechanism for a custom type is `OPERATOR '<'`
(`create-type#ordering-with-operator-<`), so the route was tested directly:

| Probe | Result |
| --- | --- |
| `CREATE TYPE ipnaked(value text) BASE text ENCODE validate_ipaddr(value) DECODE value OPERATOR '<'` on a STRICT table | creates; `ORDER BY ip` and `CREATE INDEX t(ip)` both work; equality seeks the index (`SEARCH t USING INDEX t_ip (ip=?)`) |
| the same type's order | **lexicographic, i.e. wrong**: `10.0.0.1, 100.0.0.1, 2001:db8::1, 9.0.0.1, ::1` |
| the same type on a non-STRICT table | inert: `not-an-ip` inserts, `typeof` is `text` |
| `OPERATOR '<' ip_lt` (a function comparator) | creates and runs, order unchanged (still lexicographic). The sorter only recognises four names — `numeric_lt`, `test_uint_lt`, `string_reverse`, `array_lt` (`translate/order_by.rs:31-37`) — so an unknown comparator is a silent no-op, not an error |
| conversion helpers | none: `inet_ntoa`/`ip_to_blob` do not exist; `CAST(x'0a000001' AS INTEGER)` = 0; `lpad(replace('1.2.3.45','.',''),12,'0') = lpad(replace('12.3.4.5','.',''),12,'0')` is **1** (the obvious pure-SQL sortable encoding collides) |
| the shipped packed-key column (plain BLOB) | correct order both directions, covering-index ordered scan, index seek for equality, index-only range scan |

Conclusion: the entity that gives a correct IP order is a **stored sortable
key**, not a type. Turso has no conversion function that could produce one in
SQL, so it is produced in Rust at the single write site — which is also where
it is validated (`key_of_str` refuses anything that is not an address).

## 4. Materialized views, and why not

Creating MVs is gated behind `experimental_materialized_views`; the shapes the
page query needs are each rejected at `CREATE`:

| MV body | Result |
| --- | --- |
| `SELECT eid, min(tier) FROM p GROUP BY eid` | ok |
| `SELECT … FROM e JOIN p ON p.eid = e.id WHERE p.tier = 0` | ok |
| `… LEFT JOIN …` | `LEFT OUTER JOIN is not yet supported in incremental views` |
| `… ORDER BY … LIMIT 200` | `only Filter, Projection, Join, Aggregate, and Union are supported, got: "Limit"` |
| `row_number() OVER (…)` | `window functions are not yet supported` |
| a correlated scalar subquery | `Cannot convert LogicalExpr to AST Expr: ScalarSubquery(…)` |

The page query uses a LEFT JOIN (the protocol columns), an ORDER BY over the
stored rank keys with LIMIT/OFFSET, and (for the address column) a correlated
aggregate — none of them expressible. Two further facts close the case:

- **An index on an MV is not used.** `CREATE INDEX … ON mv(col)` is accepted,
  but `EXPLAIN QUERY PLAN SELECT * FROM mv WHERE tier = 1` reports `SCAN mv` —
  so an MV is a scan-and-sort, where the shipped covering index serves the page
  as an index scan (8.6 ms at 7,672 endpoints, ADR 0003).
- **Maintenance is charged to every write.** A projection-only MV over one
  table cost **8.5 s for 20,000 inserts (425 µs/insert)**. A ping batch writes
  ~30k links and an import writes tens of thousands more; the whole batch
  pipeline's write budget is measured in seconds. Buying ~10 ms of read with
  ~13 s of write per batch is a loss at any batch size.

An MV is also redundant here: `endpoint_rank` (ADR 0003) *is* the materialized
derived state for this page, with the ordering computed once in Rust — and ADR
0003 rejected deriving the keys in SQL precisely because a second
implementation of the law drifts from the oracle. A view that maintains the
same numbers would reintroduce that risk for no read win.

## 5. Other engine options, measured

| Option | Result | Verdict |
| --- | --- | --- |
| `USING <method>` custom indexes | `USING fts` works; `USING btree`, `USING inet` → `unknown module name`. The shipped index methods are `fts` and a vector index (`turso_core-0.7.2/index_method/`) | not applicable to an address column |
| `ANALYZE` (the real "optimizer hint" mechanism: the planner loads `sqlite_stat1` into `analyze_stats` at schema reparse, and without it assumes **1,000,000 rows per table**, `translate/optimizer/cost_params.rs`) | works, writes 13 `sqlite_stat1` rows in ~120 ms (3,346 endpoints) / ~185 ms (7,656) — **and changed no plan and no timing on either database**: every page-path statement was already index-driven | measured-neutral; not shipped. Cheap to add if some future query ever plans badly |
| `likely()` / `unlikely()` / `likelihood()` | **inert-to-harmful.** The translator validates the probability argument and then translates only the inner expression (`translate/expr/translator.rs:1673-1791`), and wrapping a predicate *hides* it from index-seek detection: `… WHERE k.rank_newest_seen >= X` plans as `SEARCH … USING INDEX endpoint_rank_window (rank_newest_seen>=?)`, while `… WHERE unlikely(k.rank_newest_seen >= X)` plans as `SCAN … USING COVERING INDEX endpoint_rank_window` | do not use |
| `array_agg(ip_key)` (Turso extension, would carry the address set in one column) | refused: `Array features require --experimental-custom-types flag`; it is gated behind the same engine-wide switch. Even enabled it returns an internal SQLite record payload — `ImmutableRecord::from_values(elements).into_payload()` (`vdbe/execute.rs:6418`) — and the driver's `turso::Value` has no array variant (Null/Integer/Real/Text/Blob only), so decoding it means re-implementing the record format in this crate | not usable |
| `concurrent_writes()` / `PRAGMA journal_mode='mvcc'` | page-read latency while a writer flushes 512-row windows: p50 27.0 ms / max 36.3 ms without MVCC, p50 27.6 ms / max 36.7 ms with it (10 flush windows each) | no benefit; not shipped |

### 5b. Reachability — checked before the design was committed

The features above are only reachable through `Database::open` if the driver's
experimental flags are set (`toasty-driver-turso` mirrors every
`turso::Builder::experimental_*`) and everything toasty cannot emit is created
by raw DDL at open. That route was verified end to end, through our own stack:

| Step | Result |
| --- | --- |
| `Turso::in_memory().experimental_custom_types(true).experimental_materialized_views(true)` → `Db::builder().models(..).build(driver)` → `push_schema()` | all OK |
| raw DDL `CREATE TABLE strict_probe(id INTEGER PRIMARY KEY, port smallint, ip inet) STRICT` | OK, and **validation is live**: `'not-an-ip'` → `invalid input for type inet`; port `99999` → `integer out of range for smallint` |
| `SELECT id FROM strict_probe ORDER BY ip` | `Parse error: cannot ORDER BY column 'ip' of type 'inet': type does not declare OPERATOR '<'` |
| `CREATE INDEX … ON strict_probe(ip)` | `Parse error: cannot create index on column 'ip' of type 'inet': …` |
| `CREATE MATERIALIZED VIEW … LEFT JOIN …` | `Parse error: LEFT OUTER JOIN is not yet supported in incremental views` |
| a toasty column type, **string form**: `#[column(type = "inet")] addr: String` | **compiles**, `push_schema` succeeds, and the DDL emits it verbatim — `CREATE TABLE "custom_probe" ("id" BIGINT NOT NULL, "addr" inet NOT NULL, PRIMARY KEY ("id"))` — but the column is **inert** on a non-STRICT table: `'not-an-ip'` inserts, `ORDER BY addr` gives the lexicographic `10.0.0.1, 9.0.0.1, not-an-ip`, and its index is a text index. The hatch reaches a *name*; STRICT is what it cannot reach |
| a **toasty-typed** column: `#[column(type = inet)] addr: String` | **compile error**: ``field type `std::string::String` is not compatible with the requested column storage `toasty::codegen_support::storage::tag::Inet` `` — it wants a `cidr::IpInet` field (the `net` feature) *and* a capability with `native_inet`, which SQLITE/Turso does not have (`toasty-core/src/schema/db/ty.rs:385`; the SQLite serializer's `db::Type::Inet` arm is `todo!("Only PostgreSQL supports INET")`) |

So the rejections in §1/§3/§4 are not reachability artifacts: with the flags on
and the table hand-created, `inet` still cannot be ordered or indexed, and an
MV still cannot take the page query's shape. A typed toasty column is not
merely unhelpful, it is not expressible on this driver at all.

## 6. Carrying the address set to the page

One feed (7,656 endpoints / 15,312 links / 6,528 addresses), one decode path:
every variant is drained through the same column-reading loop, so the deltas
are statement-level and comparable. Best of 5. `checksum` is the decoded
payload — an identical checksum proves the decode work was identical, and a
larger one proves payload actually arrived.

| Statement (the same 38 columns throughout) | Cost | Checksum |
| --- | --- | --- |
| no address work at all | 37.9 ms | 316819 |
| + `(SELECT group_concat(hex(ip_key), ',') …)` (shipped) | 41.2 ms | 316819 |
| + `JOIN endpoint_rank k` for `rank_dns` instead of addresses | 45.6 ms | — |
| shipped `load_page_projection` (that statement + row assembly) | 44.1 ms | — |

The offset-0 page is Test-ordered, i.e. `rank_dns` ascending: those 200
endpoints have NO addresses, so every aggregate above returned NULL and the
+3.3–4.6 ms it costs is a **floor**. A fully-resolved page — 200 DNS endpoints
with 1–3 addresses each, checksum +8,422 characters of hex over the empty-page
run — costs **+6.1 ms**. The independent same-code A/B (§7) measured **+3.8 ms**
on the same offset-0 page, so the two agree; their absolute levels differ (DB
state, column count), which is why only each probe's internal deltas are
quoted. Row assembly is ≈2.8 ms of the shipped 44.1 ms.

Rejected alternatives, priced the same way:

| Alternative | Result |
| --- | --- |
| `group_concat(ip_key)` on the raw blobs | **lossy and unusable at any cost**: `group_concat` is a TEXT function, so the bytes come back as mangled UTF-8 (`Text("\u{4}��\u{1}\u{1}")`) |
| `LEFT JOIN endpoint_ip` (one row per address) | 30.1 ms against the aggregate's 29.6 ms in the first probe, *and* it multiplies every link row, so the decoder would have to dedup by `(protocol_id, endpoint_id)` |
| `array_agg(ip_key)` | refused without the experimental flag (§5) |
| a second id-inlined statement (`endpoint_ip::load`'s shape) | 12.0–13.6 ms — strictly worse than the aggregate it would replace |
| taking the unresolved flag from the materialized `rank_dns` and dropping the aggregate | **+7.7 ms** for the extra PK seek per row, before the addresses are read at all. This figure is *address-independent* — it is a PK seek on `endpoint_rank`, so it holds on a resolved page unchanged — while the aggregate it would replace is the smaller +3.3…+6.1 ms, measured on both page kinds below |

Both "obvious" restructures cost more than they save; there is no free reshape
here. The aggregate is the cheapest measured way to carry the bytes.

## 7. End-to-end: before and after

Same probe binary, same feed (7,656 endpoints / 15,312 links / 6,528
addresses), best of 5, run on the working tree and on the stashed (pre-change)
tree:

| Path | before (`resolved_as` JSON) | after (`endpoint_ip`) | Δ |
| --- | --- | --- | --- |
| Test sort: `profiles_page` (count + ids) | 7.94 ms | 7.75 ms | −0.2 ms |
| Test sort: `load_page_projection` (200 rows) | **40.15 ms** | **43.93 ms** | **+3.8 ms** |
| — those 200 rows are the offset-0 Test page, i.e. all `rank_dns = 0` (ipv4 hosts, no address rows), so **+3.8 ms is a lower bound**: the aggregate ran 300× and always found nothing. The fully-resolved page is §6's +6.1 ms | | | |
| Test sort: page load end to end | **48.1 ms** | **51.7 ms** | **+3.6 ms (+7%)** |
| Address sort end to end | 259.3 ms | 263.0 ms | +3.7 ms |
| Port sort end to end | 266.1 ms | 270.3 ms | +4.2 ms |
| database file | 4.03 MB | 4.33 MB | +0.30 MB (+7.5%) |
| new: IP sort (`PageSort::Ip`) | — | 165.7 ms per fetch, flat at offsets 0 and 7,400 | — |

The regression is the correlated address aggregate inside the hydrate
statement; it is the price of the addresses being relational. For scale, the
pre-existing non-indexed sorts (Address/Port, ADR 0003) cost 166–220 ms per
fetch, so the new IP sort (165.7 ms) sits in the band the tab already accepts.

Write cost per resolution update (2,000 updates, same feed; earlier probe):
635 ms as an in-place JSON rewrite (318 µs each) versus 1.82 s as
delete+insert of one address row (908 µs each). A full resolution pass over
1,105 DNS hosts therefore moves from ~0.35 s to ~1.0 s of write work — spent
on the write-behind/enrichment path, never on the UI task.

**Escalation if the page load ever becomes hot again — measured, and the
obvious two are both losses:** reading the unresolved flag from the
already-materialized `rank_dns` costs more than the aggregate (+7.7…+8.6 ms
across two probe runs, the extra PK seek per row — address-independent, so it
holds on a resolved page too), and moving the addresses out of the statement
into their own id-inlined read costs +13.6 ms (12.0 ms on an address-free id
set). The remaining lever is the sort, not
the hydration: a **materialized per-endpoint address key**
(`endpoint_rank.rank_ip BLOB` + `(rank_ip, endpoint_id)` index, the same
derived-key pattern as ADR 0003) turns the IP sort from **166.9 ms into 0.94 ms**
for ~832 µs per resolution update (1,105 single-row `UPDATE`s measured at
919 ms). It is deliberately not taken: the sort is not the default, the tab
already accepts 166–220 ms for Address/Port, and it would add a third B-tree
plus another derived column to every rank refresh.

## 8. Probes (deleted after the results were recorded)

| Probe | Answers |
| --- | --- |
| `crates/xray-tui-db/examples/turso_feature_probe.rs` | STRICT/custom types, `inet`/`smallint`, STRICT-less behaviour, MV shapes and maintenance, `USING` methods |
| `crates/xray-tui-db/examples/inet_operator_probe.rs` | the `OPERATOR '<'` route: ordering, indexing, function comparators, conversion functions, the packed key |
| `crates/xray-tui-db/examples/analyze_probe.rs` | `ANALYZE` plans and timings before/after, `likely()`/`unlikely()` plans |
| `crates/xray-tui-db/examples/endpoint_storage_probe.rs` | at-scale costs: page ids, hydration shapes, IP ordering, write costs, MVCC reader latency |
| `crates/xray-tui-db/examples/ip_shape_probe.rs` | the address-carrying shapes (§6) |
| `crates/xray-tui-db/examples/page_load_ab_probe.rs` | the before/after page load (§7), version-agnostic seeding |
| `crates/xray-tui-db/examples/advisory_probe.rs` | the reachability gate (§5b): driver flags through our stack, raw-DDL STRICT/`inet` validation on the reachable route, the refused order/index/MV, and the page-statement restructure pricing (§6) |
| `crates/xray-tui-db/examples/reconcile_probe.rs` | §6 re-timed through one decode path on one feed (checksums proving identical decode work), the fully-resolved-page floor, and the `#[column(type = "inet")]` hatch's DDL/inertness |

## Amendment — 2026-09-18: one resolution source, and resolution driven by persisted state

Three defects found in real use, all in the resolution ownership this spec
established. The storage decision (`endpoint_ip` owns the address set,
`endpoints.resolved_at` owns the attempt) is unchanged.

1. **The label and the band read different sources.** The Test cell's `[name]`
   read the in-memory `endpoint_info` cache, while the ordering law's tier-5
   band (`endpoint_rank::dns_unresolved`) reads the row's persisted addresses.
   The cell now reads the row — `endpoint_dns_unresolved(row)` — so the label
   and the band are one fact. The cache stays a cache.

2. **An outbound-only entry poisoned the resolver gate.** `spawn_outbound_enrich`
   materializes an `endpoint_info` entry carrying `outbound_ip` with an EMPTY
   address set and no attempt timestamp (it knows the endpoint's exit IP, not
   its inbound address). Two readers treated that shape as "already resolved":
   `should_resolve`'s `None => false` arm and the startup seed's
   `!contains_key` filter. A DNS host whose first successful real ping landed
   before its first resolution attempt was therefore never resolved — `[name]`
   for good. Both readers now treat "no address AND no attempt" as "still needs
   resolving", so materializing an outbound-only entry is safe and the exit IP
   is not lost.

3. **Resolution was driven by page membership, not by state.** The only
   non-forced trigger lived in the `SpeedTestResult` handler, which resolved the
   endpoint through the LOADED PAGE. A feed-wide batch persists `latency_ip` for
   every planned link but holds only 200 rows, so every off-page DNS host kept
   `[name]` while its exit IP and country were persisted and rendered. The batch
   now carries the endpoint facts with its request
   (`CoreEvent::DnsResolveRequest { endpoint_id, host, host_type, sni }`),
   deduped per page, so the trigger no longer depends on what is loaded.

Consequence for the columns: the Outbound/Country pair is gated on the endpoint
being resolved. The exit IP is evidence that a tunnel to this endpoint worked,
so a DNS host that has never resolved — or whose resolution failed — must not
assert one.
