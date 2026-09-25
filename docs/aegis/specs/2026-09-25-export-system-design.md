# Export system: streaming raw subscriptions — Design Spec

Date: `2026-09-25`
Status: `design — awaiting written-spec review`
Scope: `xray-tui-db`, `xray-tui-proto`, `xray-tui`
Related: `docs/database.md`, `docs/database-manual-sql.md`, `docs/aegis/specs/2026-09-17-purge-reason-design.md`, `docs/aegis/specs/2026-09-24-profiles-view-band-design.md`, `TUI_MANUAL.md`.

## 1. Goal and scope

Add a Profiles-tab export workflow that emits stored configurations as raw subscription text, one share URL per line, with four user-selected scopes:

- `Only Alive`
- `Only Resolved`
- `Only Active`
- `All Valid`

Destinations:

- system clipboard through `arboard`;
- file through `tokio::fs`.

Export reads the whole database, not the currently loaded page, current search, current group, current view, or current multi-selection.

Non-goals:

- schema changes or schema-tag bump;
- import changes;
- encrypted/compressed subscription output;
- native file-picker dependency;
- a second URL serializer;
- changing persisted protocol or endpoint facts.

## 2. Selection contract

Selection runs against stored `(endpoint, protocol)` links before any Resolved host substitution or serializer failure.

### Only Alive

Include each link whose canonical link tier is `0` or `1` under `RankLink::key`:

- tier `0`: real success;
- tier `1`: fast success.

The tier is computed per streamed `ProfileStats` link from the same canonical link facts used by the ordering law, not from endpoint-representative `endpoint_rank.rank_tier`. Do not use `latency.is_some()` as the predicate. A later failure intentionally leaves a prior latency in storage; a stale latency is not alive. Exclude links with a current error marker or a permanent `purge_reason`.

### Only Resolved

Include every link on a dialable endpoint:

- DNS endpoint: at least one `endpoint_ip` row;
- IPv4/IPv6 literal: dialable without DNS lookup.

For each matching DNS endpoint, emit one output line per stored resolved IP. This intentionally expands multi-address endpoints. All protocol variants remain separate lines.

Before reconstruction for each IP:

1. clone the endpoint essentials;
2. replace only `host` and `host_type` with the resolved IP;
3. retain `port` and the complete `ports` vector, so Hysteria2 port-hopping remains one URL per IP;
4. clone the loaded protocol config;
5. if TLS/Reality SNI is absent, set it to the original DNS host;
6. if WS/gRPC/HTTP transport authority is absent, set it to the original DNS host;
7. call the existing `reconstruct_proto` serializer.

This is an export-time transformation over `SecurityConfig` and transport configuration, not per-protocol URL post-processing. A new protocol variant must compile against an exhaustive `ProtocolConfig::security_mut()`/transport mutation surface.

Explicit stored SNI and explicit transport authority always win. Original DNS is only a default when the stored value is absent.

### Only Active

Mirror the current Profiles `Active` view exactly: materialized active band membership and live links only. It is not a synonym for “not purged”; aged/stale Purgatory rows are excluded by band membership.

### All Valid

Include all links with no `purge_reason`.

This includes aged/stale Purgatory links that have no permanent purge verdict. It excludes links carrying any permanent purge verdict, including `ConfigInvalid` and other evidence-based purge reasons.

The UI label/title uses `Full`; it means this non-purged set, not every row in the All view.

## 3. Deterministic output order

Rows are emitted in this complete stable order:

```text
(protocol_kind,
 transport_type,
 security_type,
 address_rank,
 address_sort_value,
 host,
 port,
 protocol_id,
 endpoint_id)
```

`protocol_kind`, `transport_type`, and `security_type` use their stored canonical text values. The address expression is executable without a schema change:

- `address_rank = 0` for a DNS endpoint with at least one `endpoint_ip` row; `address_sort_value = min(endpoint_ip.ip_key)`;
- `address_rank = 1` for an IP-literal endpoint; no `endpoint_ip.ip_key` exists, so `address_sort_value` is the endpoint host text and IP literals use lexical host order;
- `address_rank = 2` for unresolved DNS; `address_sort_value` is the endpoint host text.

Resolved addresses use packed `endpoint_ip.ip_key` bytes, so IPv4 precedes IPv6 and each family is numerically ordered. IP literals are explicitly lexical because the schema has no packed address for literal endpoints. Final IDs prevent duplicate suppression and make repeated exports diffable.

The same order is used for every scope. Resolved IP expansion repeats the stable protocol/transport/security prefix for each address.

## 4. Output format

Exact header:

```text
# profile-title: xray-tui export • <Scope>
# profile-update-interval: 1
# Date/Time: YYYY-MM-DD / HH:MM (Moscow)
# Количество: <candidate-link-count>

```

`<Scope>` is `Alive`, `Resolved`, `Active`, or `Full`.

`Количество` counts matching stored links before IP expansion and before serializer skips. It is therefore a candidate-link count, not an emitted-URL count. The completion status reports candidates, emitted lines, and skipped links separately.

The file ends with one final newline. No base64 wrapper, JSON array, or extra per-line metadata is emitted.

Date/time is generated in fixed Moscow time (`UTC+03:00`) at export start.

## 5. Database reader and snapshot

### Why a dedicated reader exists

Toasty 0.10 public query and raw-SQL terminals materialize result values. Its `.paginate()` API is page-based, not row-at-a-time. The Turso driver drains physical `Rows` into a `Vec` before returning through Toasty, and its physical connection is private.

Therefore typed Toasty cannot provide the required row-at-a-time export stream without a Toasty/driver fork.

### Chosen production architecture

For file-backed `Database::open(path)`, `xray-tui-db` stores the canonical file path and resolved journal-mode authority needed by export. The DB crate adds a direct `turso = 0.7.2` dependency; transitive use is not allowed. At export start, a dedicated per-export reader builds/connects a fresh Turso database handle for that same file and applies the recorded WAL/MVCC mode. No Toasty-private handle is assumed or exposed.

`Database::in_memory()` has no raw streaming reader. Streaming export against an in-memory database returns an unsupported error. Streaming tests use temporary file-backed databases.

Each export:

1. serializes exports with a DB-level export lock;
2. creates a fresh `turso::Connection` from the stored handle;
3. configures the same journal mode authority as the primary database (`WAL` or `MVCC`);
4. begins one read transaction;
5. runs candidate count and ordered row streaming in that same transaction;
6. emits rows to the selected sink;
7. drains or drops the active `Rows` safely;
8. commits on success or rolls back on error/cancellation;
9. drops the dedicated connection before releasing the export lock.

Snapshot transaction:

- WAL: `BEGIN DEFERRED`;
- MVCC: `BEGIN CONCURRENT`.

Do not use `BEGIN TRANSACTION READONLY` until this Turso version explicitly verifies support. The driver’s MVCC default is `BEGIN CONCURRENT`.

Turso transaction drop is only a marker; rollback is lazy on later connection access. Every export path therefore performs explicit drain/rollback before dropping the connection. No pooled Toasty connection can be left holding an unfinished statement.

No persistent second connection or reader pool is required. A fresh per-export connection keeps ownership and teardown unambiguous.

The raw reader returns one physical row at a time. It must not build `Vec<EndpointRow>`, `Vec<String>`, a full endpoint-id vector, or a joined feed-wide result.

There are two read projections, both joining `protocols.config` in the same SQL row:

- link projection for Alive, Active, and Full: one row per matching stored link, with a correlated minimum resolved `ip_key` only for sort purposes;
- resolved-address projection for Resolved: one row per matching link and stored address, using that address row’s `ip_key`.

The projection carries only fields required for:

- scope predicate;
- deterministic sort;
- endpoint essentials;
- protocol config JSON;
- link state;
- resolved address.

Because config JSON is part of the projection, there is no per-link `load_protocol_with_config` call and no page-loader/N+1 protocol fetch. The raw statement is the one read boundary for export.

If a schema-neutral covering index is required to keep the requested sort from filesorting at reference-feed scale, add it as an idempotent `CREATE INDEX` at database open and record its measured plan/cost; no schema tag bump is allowed.

The direct Turso dependency and dedicated reader are the only new DB/dependency boundary. The statement is read-only and must be added to `docs/database-manual-sql.md` with cause, measurement, and test evidence.

### File sink

After user confirms overwrite, file export streams directly to the requested destination:

1. create or truncate the destination with `tokio::fs::File`;
2. write the exact header and stream reconstructed URLs with `AsyncWriteExt`;
3. flush/sync as required by the destination contract;
4. record candidate count, emitted line count, and skipped-link count.

No complete export body is held in memory and no temporary body or atomic-replace pass is used. A failed write may leave a partial destination; report the error and do not report success.

### Clipboard sink

Clipboard export uses one owned `String`:

1. reserve a fixed-width candidate-count placeholder in the header;
2. stream reconstructed URLs into that same `String`;
3. patch the placeholder in place with candidate count;
4. call `arboard::Clipboard::set_text(&string)`.

No `Vec<String>`, `join`, or second full export buffer is allowed.

## 7. Popup and interaction

Open with `Ctrl+E` on Profiles.

Two-stage popup:

1. scope selection:
   - `Only Alive`
   - `Only Resolved`
   - `Only Active`
   - `All Valid`
2. destination selection:
   - `Clipboard`
   - `File`

File action opens an editable path field, prefilled with:

```text
./xray-tui-export-<scope>-<YYYYMMDD-HHMM>.txt
```

If destination exists, `y/n` confirmation is required before any body spool or destination write. `n` and `Esc` cancel without writing.

Export runs asynchronously. Popup closes on successful dispatch. Actions status reports:

- scope;
- destination/path;
- candidate links;
- emitted lines;
- skipped links;
- elapsed time.

Errors use red Actions/log feedback. File errors do not report success.

## 8. Owners and boundaries

- `xray-tui-db`: dedicated file reader, transaction/snapshot lifecycle, raw read-only row projection, bounded config loading.
- `xray-tui-proto`: exhaustive export-time SNI/transport default helpers; existing per-protocol URL serializers remain canonical.
- `xray-tui/src/ops/export.rs`: scope policy, row-to-URL transformation, candidate/emitted accounting, sink orchestration.
- `xray-tui/src/ui/export.rs`: popup state, rendering, path input, overwrite confirmation.
- `arboard`: clipboard destination only.
- `tokio::fs`: file destination and bounded file I/O.

No raw SQL writes. No schema migration. No compatibility alias or fallback serializer.

## 9. Acceptance and verification

1. Scope predicate tests cover real/fast success, later failure with stale latency, untested, transient failure, aged Purgatory, and permanent purge.
2. Resolved transformation tests cover:
   - one IP and multiple IPs;
   - IPv4/IPv6 ordering;
   - Hysteria2 `ports` preservation;
   - absent SNI becoming original DNS;
   - explicit SNI remaining unchanged;
   - absent WS/gRPC/HTTP authority becoming original DNS;
   - explicit authority remaining unchanged;
   - VLESS query and VMess JSON authority/SNI cases.
3. Serializer failures are skipped and counted; they do not abort valid output.
4. Header tests pin exact labels, Moscow time, candidate count semantics, final newline, and scope titles.
5. Sort tests pin complete tuple order and repeated-export stability.
6. Reader tests use temp-file databases in WAL and MVCC modes and prove count plus rows use one snapshot.
7. Reader cleanup tests prove cancellation and row errors leave no active transaction or reused connection.
8. RSS sampling during large-reference-DB export proves:
   - file path memory remains bounded by row/config/sink buffers, not feed size;
   - clipboard holds one export `String`, not `Vec<String>` plus joined body.
9. Real TUI smoke:
   - `Ctrl+E` opens scope popup;
   - destination popup works;
   - file path and overwrite confirmation work;
   - clipboard export writes exact text;
   - file export can be inspected externally.
10. Raw reader statement receives direct execution coverage and a `docs/database-manual-sql.md` inventory entry with measured cost.

## 10. ADR signal

This adds a dedicated read connection owner and a raw streaming read path around Toasty. It is an architecture-boundary change: document rationale, snapshot/mode contract, memory bound, and retirement trigger in the existing database manual/ADR surface. No schema ADR is required.

Retirement trigger: a Toasty/driver release exposing a public row stream over the existing pooled connection. Then the dedicated reader can be removed after parity and RSS verification.
