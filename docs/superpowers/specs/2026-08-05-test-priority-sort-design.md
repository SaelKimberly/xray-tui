# Test-Priority Sorting (Real Ping Latency Ordering)

**Date:** 2026-08-05
**Status:** Approved

## Problem

The expandable per-protocol sub-table inside an endpoint panel is ordered by
`last_seen_at` descending (newest variant on top, set at DB load in
`deserialize_endpoint_rows`). Ping results update the delay cell but never
reorder the panel, so after a Real Ping / Fast Ping round the user must scan
rows to find the fastest protocol. There is also no way to rank *endpoints* by
test outcome: the existing `SortColumn::Delay` sorts the main table by the
active protocol's raw latency only (untested = `-1` sorts *first* — the
opposite of intent), with no notion of test type, failures, or DNS state.

## Goals

1. Real-time sub-table ordering: as TCP/Real Ping results arrive (success or
   failure), re-sort the endpoint's protocols by a priority model — best on
   top, worst on bottom.
2. A main-table "Test" column sort (the existing Test-column sort, upgraded)
   ranking endpoints by their **best** protocol's priority + latency, manually
   selectable exactly as today (`o`/`O` cycle, speed-test menu item 7).
3. Durability: real-vs-fast success provenance survives restarts.

## Design

### 1. Priority tiers (per protocol, best → worst)

| Tier | State | Within-tier key |
|---|---|---|
| 0 | Real-ping success (`delay_source = real`) | latency asc |
| 1 | Fast/TCP or UDP success (`delay_source != real` with delay) | latency asc |
| 2 | Untested / queued (no delay, no fresh failure) | `last_seen_at` desc (today's default order) |
| 3 | Real ping failed this round | stable |
| 4 | Fast ping failed this round | stable |
| 5 | Endpoint DNS currently unresolved (`[name]`) | stable |

Rules (user decisions):

- **Fresh failure dominates**: any fresh failure this round sinks the protocol
  to that test's error tier, even if it holds a stored success from an earlier
  round.
- Both real and fast failed → tier 4 (more critical failure type wins).
- DNS-unresolved is endpoint-level and worst: it sinks all protocols of that
  endpoint together (connection impossible).
- Tier 0/1 latency ties and all stable-order rows: `last_seen_at` desc, then
  protocol id, for determinism.

State sources: `ProfileExtension.delay` + new `delay_source` (persisted),
`AppState.ping_status` rounds (session-only), `AppState.endpoint_info`
(DNS resolution state).

### 2. Schema: `delay_source` column

`profile_extensions.delay_source INTEGER` (nullable; `NULL` = no measurement /
legacy). Values: `0` = fast/TCP, `1` = real, `2` = udp.

- `SCHEMA_VERSION` 3 → 4; `ensure_column("profile_extensions", "delay_source",
  "ALTER TABLE profile_extensions ADD COLUMN delay_source INTEGER")` inside the
  existing migration transaction in `Database::open`.
- toasty model: `pub delay_source: Option<i32>` on `ProfileExtension` (fresh
  DBs get it via `push_schema`).
- Writers:
  - `ops/events.rs` `SpeedTestResult` handler — set from `test_type` whenever
    `latency_ms` is stored (`RealPing → 1`, `TcpPing → 0`, `UdpTest → 2`).
  - `ops/ping.rs` `batch_upsert_buffer` — set from the session's `ping_type`
    string: `"real" → 1`, `"udp" → 2`, everything else (`"fast"`/`"tcp"`) → 0.
- Readers — all four endpoint SELECTs (`get_active_endpoints`,
  `get_active_endpoints_by_group`, `get_stale_endpoints`, `get_endpoint`):
  `ext.delay_source` added after `ext.ip_info`; `deserialize_endpoint_rows`
  field indices shift (stats 27–32 → 28–33, `resolved_as`/`resolved_at`
  33–34 → 34–35) and the extension deserializer picks up `delay_source`.
- Upserts: `upsert_profile_extension` and `batch_upsert_profile_extensions` /
  `batch_flush_ping_buffer` gain `.delay_source(...)`.

### 3. Shared comparator (single source of truth)

In the db crate (owns `EndpointRow`), so load-time and live re-sorts can never
diverge. Session state is passed in (absent at load time):

```rust
impl EndpointRow {
    pub fn sort_protocols_by_test_priority(
        &mut self,
        dns_unresolved: bool,
        fast_failed: Option<&HashSet<i64>>,   // round sets; None = no session
        real_failed: Option<&HashSet<i64>>,
    );
}
```

Stable sort. Used:

- **DB load** (`deserialize_endpoint_rows`): replaces the `last_seen_at` sort
  with `sort_protocols_by_test_priority(false, None, None)` — persisted tiers
  and latencies apply; untested endpoints keep their current order.
- **`SpeedTestResult` handler** (`ops/events.rs`): after the round/extension
  update, for `TcpPing`/`RealPing` results (success *and* failure), re-sort the
  owning row with live session state; remap `selected_sub` by protocol id
  (`keep = id at old index` → `position(id)` after sort) so the highlighted
  row and Enter-pin never drift.
- **`EndpointInfoUpdated` handler**: when an endpoint's DNS resolved-state
  flips (empty ↔ non-empty `resolved_ips`), re-sort that endpoint — a
  DNS-unresolved endpoint's protocols sink to tier 5 and rise back on resolve.

### 4. Main-table Test sort

- Rename `SortColumn::Delay` → `SortColumn::Test` (in-memory only, never
  serialized — verified). Touches: `types.rs` enum, the live comparator in
  `ops/profiles.rs::compute_filtered_indices`, the `o`/`O` cycle list and
  speed-test menu item 7 in `ui/mod.rs`, the column-12 sort-arrow mapping in
  `ui/profiles.rs`.
- Comparator: per-endpoint key = **best** protocol's
  `(tier, latency, last_seen_at desc, id)` — min tier, then min latency.
  Live session state (`state.ping_status`, `state.endpoint_info`) feeds the
  tier computation; untested endpoints fall back to `last_seen_at` order.
- Dead code: the duplicate `compute_filtered_indices` method in `state.rs`
  (never called; `filtered_profiles` delegates to `ops/profiles.rs`) is
  deleted.

### 5. Rendering

`build_display_rows` unchanged — `panel_rows` iterate `row.protocols`, which
now holds the priority order. The Test column's `[delay]`/`[name]`/`[fast]`/
`[real]` labels and `ping_status` rounds are untouched.

## Files

- `crates/xray-tui-db/src/models_toasty.rs` — `delay_source` field
- `crates/xray-tui-db/src/database.rs` — migration, 4 SELECTs, deserialize
  indices, upserts, load-time sort, comparator tests
- `crates/xray-tui-db/src/error.rs` — no change
- `crates/xray-tui/src/ops/events.rs` — `SpeedTestResult` + `EndpointInfoUpdated`
  re-sorts, `selected_sub` remap, `delay_source` write
- `crates/xray-tui/src/ops/ping.rs` — `batch_upsert_buffer` sets `delay_source`
- `crates/xray-tui/src/ops/profiles.rs` — `SortColumn::Test` comparator
- `crates/xray-tui/src/state.rs` — delete dead `compute_filtered_indices`
- `crates/xray-tui/src/types.rs` — rename variant
- `crates/xray-tui/src/ui/mod.rs` — cycle list + menu item 7
- `crates/xray-tui/src/ui/profiles.rs` — column mapping

## Testing

- db crate: comparator unit tests (tiers, fresh-failure dominance, latency
  ordering, stability, untested fallback); load-order test via
  `get_active_endpoints` with seeded `delay`/`delay_source` rows.
- events: `SpeedTestResult` (real success) re-sorts and remaps `selected_sub`;
  failure demotes below untested; fast result sorts; other test types don't.
- profiles: main-table `SortColumn::Test` ranks by best-protocol key; errors
  sink below untested; DNS-unresolved endpoints last.
- Workspace: `cargo test`, `cargo clippy` clean on touched lines.

## Out of scope

- Speed-test results: still set `speed`, do not affect latency tiers.
- `sort_order` column: unused for this feature (delays + `delay_source` are
  the persisted state; failures are deliberately session-only).
- Persisting the sort column choice (already in-memory behavior).
