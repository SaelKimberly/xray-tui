# Test-Priority Sorting Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Order protocol sub-rows and endpoints by test priority — real-ping success first, then fast-ping success, untested, then failures (real < fast), DNS-unresolved last — live as results arrive and via the main-table Test column sort.

**Architecture:** A single shared priority comparator on `EndpointRow` in the db crate, fed persisted state (`delay`, new `delay_source` provenance) plus session state (round failures, DNS resolution). It runs at DB load and live in the event loop (`SpeedTestResult`, `EndpointInfoUpdated`). The main-table `SortColumn::Test` (renamed from `Delay`) ranks endpoints by their best protocol's key.

**Tech Stack:** Rust 2024, toasty ORM v0.9 (turso driver), ratatui.

## Global Constraints

- Spec: `docs/superpowers/specs/2026-08-05-test-priority-sort-design.md`
- Tier order (best to worst): real-ok (0) > fast-ok (1) > untested (2) > real-err (3) > fast-err (4) > name/dns-unresolved (5)
- Fresh failure dominates stored success (user decision)
- Main-table representative = best protocol (min tier, then min latency) (user decision)
- Sub-table re-sort triggers: TcpPing + RealPing results only (success or failure). UDP stores `delay_source` but never triggers a re-sort.
- `delay_source` values: `0` = fast/TCP, `1` = real, `2` = udp; `None`/`-1` = no measurement (legacy)
- `ProfileExtension.delay_source` upsert convention: `unwrap_or(-1)` (toasty upsert methods take concrete values, matching the existing `.delay(ext.delay.unwrap_or(0))` pattern)
- `SCHEMA_VERSION` 3 → 4, additive `ensure_column` migration (existing pattern in `Database::open`)
- Rename `SortColumn::Delay` → `SortColumn::Test` (in-memory only, never serialized — verified)
- Dead code to delete: `state.rs` `compute_filtered_indices` (lines 403-493) and `state.rs` `batch_upsert_buffer` (lines 1352-1374) — both private, zero callers
- Do NOT use `ast_edit`; use `edit` for all code changes

---

### Task 1: `delay_source` schema column + migration + read/write plumbing

**Files:**
- Modify: `crates/xray-tui-db/src/models_toasty.rs` (ProfileExtension struct)
- Modify: `crates/xray-tui-db/src/database.rs` (migration, 4 SELECTs, `deserialize_endpoint_rows`, 3 upsert fns)
- Modify: `crates/xray-tui/src/ops/events.rs:240-260` (`or_insert_with` default)
- Modify: `crates/xray-tui/src/ops/ping.rs:404` (batch buffer extension — `delay_source: None` for now; Task 3 sets real values)
- Delete: `crates/xray-tui/src/state.rs:1352-1374` (dead `batch_upsert_buffer` — another `ProfileExtension` construction site, no callers)

**Interfaces:**
- Produces: `ProfileExtension.delay_source: Option<i32>`; pub consts `DELAY_SOURCE_FAST: i32 = 0`, `DELAY_SOURCE_REAL: i32 = 1`, `DELAY_SOURCE_UDP: i32 = 2` (in `models_toasty.rs`); DB SELECTs with `ext.delay_source` between `ext.ip_info` and `s.protocol_id`.

- [ ] **Step 1: Write the failing migration test** (in `crates/xray-tui-db/src/database.rs` `mod tests`, next to `test_open_migrates_old_schema`)

```rust
/// Simulate a v3 database (no delay_source column) and prove `Database::open`
/// re-adds it in place — push_schema never runs on existing databases.
#[tokio::test]
async fn test_open_adds_delay_source_column() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("v3.db");
    {
        let driver = toasty_driver_turso::Turso::file(&path);
        let db = toasty::Db::builder()
            .models(toasty::models!(
                Endpoint,
                ProtocolRow,
                EndpointGroup,
                Group,
                ProfileExtension,
                ServerStat,
                PingSession,
                RoutingRule,
                DnsSetting
            ))
            .build(driver)
            .await
            .expect("build db");
        db.push_schema().await.expect("push schema");
        let mut conn = db.connection().await.expect("connection");
        toasty::sql::statement("ALTER TABLE profile_extensions DROP COLUMN delay_source")
            .exec(&mut conn)
            .await
            .expect("drop delay_source");
        toasty::sql::query("PRAGMA user_version = 3")
            .exec(&mut conn)
            .await
            .expect("set version 3");
    }
    let db = Database::open(&path).await.expect("open migrates delay_source");
    let mut conn = db.db.connection().await.expect("connection");
    let rows = toasty::sql::query(
        "SELECT COUNT(*) FROM pragma_table_info('profile_extensions') WHERE name = 'delay_source'",
    )
    .exec(&mut conn)
    .await
    .expect("pragma");
    let count: i64 = rows
        .first()
        .and_then(|v| {
            if let Value::Record(fields) = v {
                fields.first().and_then(|f| match f {
                    Value::I64(n) => Some(*n),
                    _ => None,
                })
            } else {
                None
            }
        })
        .unwrap_or(0);
    assert_eq!(count, 1, "open() did not add profile_extensions.delay_source");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p xray-tui-db test_open_adds_delay_source_column`
Expected: FAIL — column dropped in setup is not re-added by open().

- [ ] **Step 3: Add the model field + constants** (`models_toasty.rs`)

Add to `ProfileExtension` (after `ip_info`, before the `#[belongs_to]`):

```rust
    /// Provenance of `delay`: `DELAY_SOURCE_FAST` / `DELAY_SOURCE_REAL` /
    /// `DELAY_SOURCE_UDP`. `None` = no measurement recorded.
    pub delay_source: Option<i32>,
```

Add near the top of the file (after the `use` block):

```rust
/// `ProfileExtension.delay_source` provenance values.
pub const DELAY_SOURCE_FAST: i32 = 0;
pub const DELAY_SOURCE_REAL: i32 = 1;
pub const DELAY_SOURCE_UDP: i32 = 2;
```

- [ ] **Step 4: Implement the migration** (`database.rs` `open()`)

Change `const SCHEMA_VERSION: i64 = 3;` to `= 4;`. Inside the migration transaction, after the `dns_settings` `cache_ttl_secs` `ensure_column`, add:

```rust
            Self::ensure_column(
                &mut tx,
                "profile_extensions",
                "delay_source",
                "ALTER TABLE profile_extensions ADD COLUMN delay_source INTEGER",
            )
            .await?;
```

- [ ] **Step 5: Update the four endpoint SELECTs** (`database.rs`: `get_active_endpoints`, `get_active_endpoints_by_group`, `get_stale_endpoints`, `get_endpoint`)

In each query, insert `ext.delay_source, \` immediately after `ext.ip_info, \` so the extension column group becomes:

```
                    ext.protocol_id, ext.delay, ext.speed, ext.sort_order, ext.ip_info, ext.delay_source, \
```

- [ ] **Step 6: Update `deserialize_endpoint_rows` indices** (`database.rs`)

- Update the doc comment: extension fields 22-27 (delay_source = 27), stats 28-33, `resolved_as`/`resolved_at` 34-35.
- Extension `or_insert_with` — add `delay_source: get_opt_i64(&fields, 27).map(|v| v as i32),`
- Stats indices shift by +1: `get_opt_i64(&fields, 27)` → `28` (protocol_id), `28` → `29` (today_up), `29` → `30` (today_down), `30` → `31` (total_up), `31` → `32` (total_down), `32` → `33` (last_updated).
- Endpoint resolved fields shift by +1: `resolved_as` `get_opt_string(&fields, 33)` → `34`, `resolved_at` `get_opt_i64(&fields, 34)` → `35`.

- [ ] **Step 7: Update the three extension upserts** (`database.rs`)

In `upsert_profile_extension`, `batch_upsert_profile_extensions`, and the extension loop of `batch_flush_ping_buffer`, add after `.ip_info(...)`:

```rust
            .delay_source(ext.delay_source.unwrap_or(-1))
```

- [ ] **Step 8: Fix all remaining `ProfileExtension` construction sites** (compiler-driven)

- `crates/xray-tui/src/ops/events.rs` `or_insert_with` default: add `delay_source: None,`
- `crates/xray-tui/src/ops/ping.rs:404` batch buffer: add `delay_source: None,` (Task 3 replaces with real mapping)
- `crates/xray-tui/src/state.rs:1352-1374`: delete the dead `batch_upsert_buffer` method entirely (private, no callers — verified; it is another construction site)
- Any remaining sites the compiler reports (e.g. test fixtures): add `delay_source: None,`

- [ ] **Step 9: Write the load round-trip test** (covers SELECT + index shifts: extension, stats, resolved columns)

```rust
/// `delay_source` plus the shifted stats/resolved column indices round-trip
/// through `get_active_endpoints` (a misaligned SELECT breaks these reads).
#[tokio::test]
async fn delay_source_roundtrips_through_get_active_endpoints() {
    let db = Database::in_memory().await.expect("in-memory db");
    let mut conn = db.db.connection().await.expect("connection");
    toasty::sql::statement(
        "INSERT INTO endpoints (id, host, host_type, port, created_at, resolved_as, resolved_at) \
         VALUES (1001, 'dns.example', 'dns', 443, 0, '1.2.3.4', 100)",
    )
    .exec(&mut conn)
    .await
    .expect("insert endpoint");
    toasty::sql::statement(
        "INSERT INTO protocol_rows \
           (id, endpoint_id, sig, cred_hash, proto_kind, spec_blob, config_type, core_type, created_at, last_seen_at) \
         VALUES (2001, 1001, 1, 0, 'vmess', X'', 0, 'xray', 0, 10)",
    )
    .exec(&mut conn)
    .await
    .expect("insert protocol");
    toasty::sql::statement(
        "INSERT INTO profile_extensions (protocol_id, delay, speed, sort_order, ip_info, delay_source) \
         VALUES (2001, 123, 0, 0, '1.2.3.4|US', 1)",
    )
    .exec(&mut conn)
    .await
    .expect("insert extension");
    toasty::sql::statement(
        "INSERT INTO server_stats (protocol_id, total_down, total_up) VALUES (2001, 5, 6)",
    )
    .exec(&mut conn)
    .await
    .expect("insert stats");

    let rows = db.get_active_endpoints(0).await.expect("load");
    assert_eq!(rows.len(), 1);
    let ext = &rows[0].extensions[&2001];
    assert_eq!(ext.delay, Some(123));
    assert_eq!(ext.delay_source, Some(1));
    assert_eq!(ext.ip_info.as_deref(), Some("1.2.3.4|US"));
    let st = &rows[0].stats[&2001];
    assert_eq!(st.total_down, Some(5));
    assert_eq!(st.total_up, Some(6));
    assert_eq!(rows[0].endpoint.resolved_as.as_deref(), Some("1.2.3.4"));
    assert_eq!(rows[0].endpoint.resolved_at, Some(100));
}
```

- [ ] **Step 10: Run the db crate tests + clippy**

Run: `cargo test -p xray-tui-db && cargo clippy -p xray-tui-db --all-targets`
Expected: migration test PASS, round-trip test PASS, no new warnings (construction-site fixes above are required for the workspace to compile).

- [ ] **Step 11: Workspace compile check + commit**

Run: `cargo build` (workspace — catches any construction site missed by the compiler)
Expected: builds.

```bash
git add crates/xray-tui-db/src/models_toasty.rs crates/xray-tui-db/src/database.rs \
        crates/xray-tui/src/ops/events.rs crates/xray-tui/src/ops/ping.rs \
        crates/xray-tui/src/state.rs
git commit -m "feat(db): profile_extensions.delay_source column + migration"
```

---

### Task 2: Priority comparator + load-time ordering

**Files:**
- Modify: `crates/xray-tui-db/src/models_toasty.rs` (impl EndpointRow — comparator + key methods)
- Modify: `crates/xray-tui-db/src/database.rs` (`deserialize_endpoint_rows` load sort)
- Test: `crates/xray-tui-db/src/models_toasty.rs` new `#[cfg(test)] mod tests`

**Interfaces:**
- Consumes: `ProfileExtension.delay_source`, `DELAY_SOURCE_REAL` (Task 1)
- Produces:
  - `EndpointRow::sort_protocols_by_test_priority(&mut self, dns_unresolved: bool, rounds: Option<(&HashSet<i64>, &HashSet<i64>)>)`
  - `EndpointRow::best_test_priority_key(&self, dns_unresolved: bool, rounds: Option<(&HashSet<i64>, &HashSet<i64>)>) -> (u8, i32, i64, i64)`
  - `rounds` = `(fast_failed, real_failed)` from `PingRound.failed`; `None` at load time (failures are session-only)
  - Key tuple = `(tier, latency.unwrap_or(i32::MAX), -last_seen_at, id)`, ascending = best first

- [ ] **Step 1: Write the failing comparator tests** (`models_toasty.rs`)

Add at the end of `models_toasty.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use toasty::Deferred;

    /// Endpoint with protocols `(id, last_seen_at, delay, delay_source)`.
    /// `delay: None` = untested protocol (no extension entry).
    fn row(protos: &[(i64, i64, Option<i32>, Option<i32>)]) -> EndpointRow {
        let mut row = EndpointRow {
            endpoint: Endpoint {
                id: 1,
                host: "h.example".to_string(),
                host_type: "ipv4".to_string(),
                port: 443,
                port_spec_str: None,
                parent_id: None,
                last_source: None,
                created_at: 0,
                manual_protocol_override: None,
                resolved_as: None,
                resolved_at: None,
            },
            protocols: Vec::new(),
            extensions: HashMap::new(),
            stats: HashMap::new(),
            selected_protocol: 0,
            expanded: false,
        };
        for (id, last_seen, delay, src) in protos {
            row.protocols.push(ProtocolRow {
                id: *id,
                endpoint_id: 1,
                sig: 0,
                cred_hash: 0,
                proto_kind: String::new(),
                spec_blob: Vec::new(),
                config_type: 1,
                core_type: "xray".to_string(),
                transport: None,
                security: None,
                last_used_at: None,
                created_at: 0,
                last_seen_at: *last_seen,
                endpoint: Deferred::from(None::<Endpoint>),
                extension: Deferred::from(None::<ProfileExtension>),
                server_stat: Deferred::from(None::<ServerStat>),
            });
            if let Some(d) = delay {
                row.extensions.insert(
                    *id,
                    ProfileExtension {
                        protocol_id: *id,
                        delay: Some(*d),
                        speed: None,
                        sort_order: None,
                        ip_info: None,
                        delay_source: *src,
                        protocol_row: Deferred::from(None::<ProtocolRow>),
                    },
                );
            }
        }
        row
    }

    fn ids(r: &EndpointRow) -> Vec<i64> {
        r.protocols.iter().map(|p| p.id).collect()
    }

    fn failed(ids: &[i64]) -> HashSet<i64> {
        ids.iter().copied().collect()
    }

    #[test]
    fn real_ok_above_fast_ok_above_untested() {
        // real-ok 200ms outranks fast-ok 10ms — tier beats latency.
        let mut r = row(&[
            (10, 1, Some(200), Some(DELAY_SOURCE_REAL)), // real-ok
            (20, 2, Some(10), Some(DELAY_SOURCE_FAST)),  // fast-ok
            (30, 3, None, None),                         // untested
        ]);
        r.sort_protocols_by_test_priority(false, None);
        assert_eq!(ids(&r), vec![10, 20, 30]);
    }

    #[test]
    fn latency_orders_within_success_tiers() {
        let mut r = row(&[
            (10, 1, Some(50), Some(DELAY_SOURCE_FAST)),
            (20, 2, Some(10), Some(DELAY_SOURCE_FAST)),
            (30, 3, Some(120), Some(DELAY_SOURCE_REAL)),
            (40, 4, Some(90), Some(DELAY_SOURCE_REAL)),
        ]);
        r.sort_protocols_by_test_priority(false, None);
        // real tier first (30:120, 40:90 by latency), then fast tier (20:10, 10:50)
        assert_eq!(ids(&r), vec![40, 30, 20, 10]);
    }

    #[test]
    fn fresh_failure_dominates_stored_success() {
        // 10 has a stored real-ok delay but failed real this round -> sinks
        // below the untested 30; 20 failed fast -> below 10 (fast worse than real).
        let mut r = row(&[
            (10, 1, Some(50), Some(DELAY_SOURCE_REAL)),
            (20, 2, Some(80), Some(DELAY_SOURCE_FAST)),
            (30, 3, None, None),
        ]);
        r.sort_protocols_by_test_priority(
            false,
            Some((&failed(&[20]), &failed(&[10]))),
        );
        assert_eq!(ids(&r), vec![30, 10, 20]);
    }

    #[test]
    fn both_failed_uses_fast_tier() {
        let mut r = row(&[(10, 1, None, None), (20, 2, None, None)]);
        r.sort_protocols_by_test_priority(false, Some((&failed(&[10]), &failed(&[10]))));
        assert_eq!(ids(&r), vec![20, 10]);
    }

    #[test]
    fn dns_unresolved_sinks_all_protocols() {
        let mut r = row(&[
            (10, 1, Some(50), Some(DELAY_SOURCE_REAL)),
            (20, 2, None, None),
        ]);
        r.sort_protocols_by_test_priority(true, None);
        assert_eq!(ids(&r), vec![20, 10]); // untested first; dns tier wins for both
    }

    #[test]
    fn untested_keeps_last_seen_recency_order() {
        let mut r = row(&[(10, 5, None, None), (20, 9, None, None), (30, 1, None, None)]);
        r.sort_protocols_by_test_priority(false, None);
        assert_eq!(ids(&r), vec![20, 10, 30]); // newest first
    }

    #[test]
    fn best_key_returns_min_over_protocols() {
        let r = row(&[
            (10, 1, Some(200), Some(DELAY_SOURCE_REAL)),
            (20, 2, Some(10), Some(DELAY_SOURCE_FAST)),
            (30, 3, None, None),
        ]);
        // Best = real-ok (tier 0), latency 200
        assert_eq!(r.best_test_priority_key(false, None), (0, 200, -1, 10));
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p xray-tui-db --lib`
Expected: FAIL — `sort_protocols_by_test_priority` / `best_test_priority_key` not found.

- [ ] **Step 3: Implement the comparator** (`models_toasty.rs`, in the existing `impl EndpointRow`)

```rust
    /// Tier for one protocol under the test-priority model (lower = better):
    /// 0 real-ok, 1 fast/udp-ok, 2 untested, 3 real-err, 4 fast-err,
    /// 5 name/dns-unresolved. Fresh failures dominate stored successes.
    fn protocol_test_tier(
        delay: Option<i32>,
        delay_source: Option<i32>,
        dns_unresolved: bool,
        rounds: Option<(&HashSet<i64>, &HashSet<i64>)>,
        pid: i64,
    ) -> u8 {
        if dns_unresolved {
            5
        } else if let Some((fast_failed, _)) = rounds
            && fast_failed.contains(&pid)
        {
            4
        } else if let Some((_, real_failed)) = rounds
            && real_failed.contains(&pid)
        {
            3
        } else if delay_source == Some(DELAY_SOURCE_REAL) {
            0
        } else if delay.is_some() {
            1
        } else {
            2
        }
    }

    /// Ascending sort key: `(tier, latency, recency, id)`. `recency` is
    /// negated so newer `last_seen_at` sorts first on ties.
    fn protocol_test_key(
        p: &ProtocolRow,
        ext: Option<&ProfileExtension>,
        dns_unresolved: bool,
        rounds: Option<(&HashSet<i64>, &HashSet<i64>)>,
    ) -> (u8, i32, i64, i64) {
        let delay = ext.and_then(|e| e.delay);
        let tier = Self::protocol_test_tier(
            delay,
            ext.and_then(|e| e.delay_source),
            dns_unresolved,
            rounds,
            p.id,
        );
        (tier, delay.unwrap_or(i32::MAX), -p.last_seen_at, p.id)
    }

    /// Re-sort `protocols` by test priority: real-ping success first, then
    /// fast/TCP/UDP success (latency ascending), then untested (newest
    /// `last_seen_at` first), then failures (real below fast below untested),
    /// then DNS-unresolved endpoints at the bottom. Deterministic tiebreak by
    /// protocol id. `rounds` is `(fast_failed, real_failed)`; `None` when no
    /// session state exists.
    pub fn sort_protocols_by_test_priority(
        &mut self,
        dns_unresolved: bool,
        rounds: Option<(&HashSet<i64>, &HashSet<i64>)>,
    ) {
        self.protocols.sort_by_key(|p| {
            Self::protocol_test_key(p, self.extensions.get(&p.id), dns_unresolved, rounds)
        });
    }

    /// The endpoint's representative sort key = its best (minimum) protocol
    /// key — used by the main-table Test column sort.
    pub fn best_test_priority_key(
        &self,
        dns_unresolved: bool,
        rounds: Option<(&HashSet<i64>, &HashSet<i64>)>,
    ) -> (u8, i32, i64, i64) {
        self.protocols
            .iter()
            .map(|p| {
                Self::protocol_test_key(p, self.extensions.get(&p.id), dns_unresolved, rounds)
            })
            .min()
            .unwrap_or((2, i32::MAX, 0, 0))
    }
```

Add `use std::collections::HashSet;` to `models_toasty.rs` (HashMap is already imported).

- [ ] **Step 4: Swap the load-time sort** (`database.rs` `deserialize_endpoint_rows`)

Replace the existing loop:

```rust
    // Sort each endpoint's protocols by last_seen_at descending — newest
    // variant on top of the expandable sub-table. Stable sort: ties (e.g.
    // never-seen protocols) keep insertion order.
    for row in map.values_mut() {
        row.protocols
            .sort_by_key(|p| std::cmp::Reverse(p.last_seen_at));
    }
```

with:

```rust
    // Sort each endpoint's protocols by test priority: measured latency
    // (real ping first, then fast/UDP) ascending on top, untested protocols
    // by last_seen_at descending (newest variant first, today's default),
    // failures and DNS-unresolved hosts sink to the bottom. `rounds` is None
    // here — failures are session-only state, applied live in the TUI.
    // Persisted resolution counts as resolved: `resolved_as` empty means the
    // host is unresolved (name tier) until a live resolution event flips it.
    for row in map.values_mut() {
        let dns_unresolved = row.endpoint.host_type == "dns"
            && row.endpoint.resolved_as.as_deref().is_none_or(str::is_empty);
        row.sort_protocols_by_test_priority(dns_unresolved, None);
    }
```

- [ ] **Step 5: Run the db crate tests + clippy**

Run: `cargo test -p xray-tui-db && cargo clippy -p xray-tui-db --all-targets`
Expected: all comparator tests PASS, existing tests PASS (load-order test for Task 1's round-trip still passes — it has one protocol), no new warnings.

- [ ] **Step 6: Commit**

```bash
git add crates/xray-tui-db/src/models_toasty.rs crates/xray-tui-db/src/database.rs
git commit -m "feat(db): test-priority protocol sort + load-time ordering"
```

---

### Task 3: Live re-sort on ping results

**Files:**
- Modify: `crates/xray-tui/src/ops/profiles.rs` (new helpers `endpoint_dns_unresolved`, `session_rounds`)
- Modify: `crates/xray-tui/src/ops/events.rs` (`SpeedTestResult` + `EndpointInfoUpdated` arms)
- Modify: `crates/xray-tui/src/ops/ping.rs:404` (`delay_source` from `ping_type`)
- Test: `crates/xray-tui/src/ops/events.rs` `mod tests`

**Interfaces:**
- Consumes: `EndpointRow::sort_protocols_by_test_priority` (Task 2), `DELAY_SOURCE_*` consts (Task 1)
- Produces:
  - `pub(crate) fn endpoint_dns_unresolved(state: &AppState, row: &EndpointRow) -> bool`
  - `pub(crate) fn session_rounds<'a>(state: &'a AppState, row: &EndpointRow) -> Option<(&'a HashSet<i64>, &'a HashSet<i64>)>`

- [ ] **Step 1: Write the failing events tests** (`crates/xray-tui/src/ops/events.rs` `mod tests`)

Add fixtures (next to existing `row_with_protocol`) and tests:

```rust
    /// Multi-protocol endpoint fixture. Protocols get ids `[start..start+n]`.
    fn row_with_protocols(endpoint_id: i64, n: usize, start: i64) -> EndpointRow {
        let mut row = row_with_protocol(endpoint_id, start); // single-proto fixture
        for i in 1..n {
            row.protocols.push(ProtocolRow {
                id: start + i as i64,
                endpoint_id,
                ..row.protocols[0].clone()
            });
        }
        row
    }

    fn set_delay(row: &mut EndpointRow, pid: i64, delay: i32, source: Option<i32>) {
        row.extensions.insert(
            pid,
            ProfileExtension {
                protocol_id: pid,
                delay: Some(delay),
                speed: None,
                sort_order: None,
                ip_info: None,
                delay_source: source,
                protocol_row: Deferred::from(None::<ProtocolRow>),
            },
        );
    }

    async fn event_state() -> (AppState, tokio::sync::mpsc::Sender<CoreEvent>) {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(
            xray_tui_db::Database::open(dir.path().join("t.db"))
                .await
                .unwrap(),
        );
        let mut state = AppState::new(db, AppConfig::default()).await;
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        state.core_event_rx = Some(rx);
        (state, tx)
    }
```

Note: `ProfileExtension` is already imported in the events test module (used by `row_with_protocol`). `Deferred` is imported. Add `CoreEvent` import if not already in scope (`use super::*;` covers it).

```rust
    #[tokio::test]
    async fn real_ping_result_restores_sub_table_and_remaps_selection() {
        let (mut state, tx) = event_state().await;
        let mut row = row_with_protocols(100, 3, 7); // p7, p8, p9
        set_delay(&mut row, 9, 50, Some(DELAY_SOURCE_FAST)); // fast-ok
        row.endpoint.host_type = "ipv4".to_string();
        state.endpoints = vec![row];
        state.selected_index = 0;
        state.selected_sub = Some(2); // points at p9 before the sort
        state.filter_cache_valid.set(false);

        tx.send(CoreEvent::SpeedTestResult {
            protocol_id: 8,
            test_type: TestType::RealPing,
            latency_ms: Some(120),
            speed_bps: None,
            ip_info: Some("1.2.3.4|US".to_string()),
            error: None,
        })
        .await
        .unwrap();
        assert!(state.poll_core_events().await);

        // p8 (real 120) above p9 (fast 50) — tier beats latency; p7 untested last.
        let ids: Vec<i64> = state.endpoints[0].protocols.iter().map(|p| p.id).collect();
        assert_eq!(ids, vec![8, 9, 7]);
        // selected_sub followed p9 to its new index 1.
        assert_eq!(state.selected_sub, Some(1));
        assert_eq!(
            state.endpoints[0].extensions[&8].delay_source,
            Some(DELAY_SOURCE_REAL)
        );
    }

    #[tokio::test]
    async fn real_ping_failure_demotes_below_untested() {
        let (mut state, tx) = event_state().await;
        let mut row = row_with_protocols(100, 2, 7); // p7, p8
        set_delay(&mut row, 7, 50, Some(DELAY_SOURCE_REAL)); // stored real-ok
        state.endpoints = vec![row];
        state.selected_index = 0;
        state.selected_sub = None;
        state.filter_cache_valid.set(false);

        tx.send(CoreEvent::SpeedTestResult {
            protocol_id: 7,
            test_type: TestType::RealPing,
            latency_ms: None,
            speed_bps: None,
            ip_info: None,
            error: Some("timeout".to_string()),
        })
        .await
        .unwrap();
        assert!(state.poll_core_events().await);

        let ids: Vec<i64> = state.endpoints[0].protocols.iter().map(|p| p.id).collect();
        assert_eq!(ids, vec![8, 7]); // fresh real failure sinks below untested
    }

    #[tokio::test]
    async fn fast_ping_result_sorts_sub_table() {
        let (mut state, tx) = event_state().await;
        let mut row = row_with_protocols(100, 2, 7); // p7, p8
        set_delay(&mut row, 7, 200, Some(DELAY_SOURCE_REAL));
        state.endpoints = vec![row];
        state.selected_index = 0;
        state.selected_sub = None;
        state.filter_cache_valid.set(false);

        tx.send(CoreEvent::SpeedTestResult {
            protocol_id: 8,
            test_type: TestType::TcpPing,
            latency_ms: Some(15),
            speed_bps: None,
            ip_info: None,
            error: None,
        })
        .await
        .unwrap();
        assert!(state.poll_core_events().await);

        let ids: Vec<i64> = state.endpoints[0].protocols.iter().map(|p| p.id).collect();
        assert_eq!(ids, vec![7, 8]); // real-ok 200 still above fast-ok 15
        assert_eq!(
            state.endpoints[0].extensions[&8].delay_source,
            Some(DELAY_SOURCE_FAST)
        );
    }

    #[tokio::test]
    async fn dns_unresolved_endpoint_sinks_after_result() {
        let (mut state, tx) = event_state().await;
        let mut row = row_with_protocols(100, 2, 7); // p7, p8
        set_delay(&mut row, 7, 50, Some(DELAY_SOURCE_REAL));
        row.endpoint.host_type = "dns".to_string();
        state.endpoints = vec![row]; // no endpoint_info entry -> unresolved
        state.selected_index = 0;
        state.selected_sub = None;
        state.filter_cache_valid.set(false);

        tx.send(CoreEvent::SpeedTestResult {
            protocol_id: 8,
            test_type: TestType::RealPing,
            latency_ms: Some(10),
            speed_bps: None,
            ip_info: Some("1.2.3.4|US".to_string()),
            error: None,
        })
        .await
        .unwrap();
        assert!(state.poll_core_events().await);

        // dns tier sinks both; untested p8 first (stable), p7 second.
        let ids: Vec<i64> = state.endpoints[0].protocols.iter().map(|p| p.id).collect();
        assert_eq!(ids, vec![8, 7]);
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p xray-tui --lib ops::events`
Expected: FAIL — no re-sort happens yet (order stays `[7, 8, 9]` etc.).

- [ ] **Step 3: Add the helpers** (`crates/xray-tui/src/ops/profiles.rs`, near `compute_filtered_indices`)

Add `use std::collections::HashSet;` to the existing imports (HashMap already imported). Then:

```rust
/// Whether the endpoint's DNS host is currently unresolved (no known IPs).
/// Endpoints without an `endpoint_info` entry count as unresolved.
pub(crate) fn endpoint_dns_unresolved(state: &AppState, row: &EndpointRow) -> bool {
    row.endpoint.host_type == "dns"
        && state
            .endpoint_info
            .get(&row.endpoint.id)
            .is_none_or(|i| i.resolved_ips.is_empty())
}

/// The endpoint's current ping-round failure sets as `(fast_failed,
/// real_failed)`. `None` when no round is in flight.
pub(crate) fn session_rounds<'a>(
    state: &'a AppState,
    row: &EndpointRow,
) -> Option<(&'a HashSet<i64>, &'a HashSet<i64>)> {
    state
        .ping_status
        .get(&row.endpoint.id)
        .map(|ps| (&ps.fast.failed, &ps.real.failed))
}
```

- [ ] **Step 4: Update the `SpeedTestResult` handler** (`crates/xray-tui/src/ops/events.rs`)

4a. In the `or_insert_with` default, add `delay_source: None,`.

4b. In the `match test_type` arm, set provenance and skip the DB write on error (an error mutates nothing; the current unconditional upsert writes `delay = 0` for a fresh ext, which would mislabel a failed-first-test protocol as measured 0ms):

```rust
                            match test_type {
                                TestType::RealPing => {
                                    ext.delay = latency_ms.map(|v| v as i32);
                                    ext.delay_source = Some(DELAY_SOURCE_REAL);
                                    ext.ip_info = ip_info;
                                }
                                TestType::TcpPing | TestType::UdpTest => {
                                    ext.delay = latency_ms.map(|v| v as i32);
                                    ext.delay_source = Some(if test_type == TestType::UdpTest {
                                        DELAY_SOURCE_UDP
                                    } else {
                                        DELAY_SOURCE_FAST
                                    });
                                }
                                TestType::SpeedTest => {
                                    ext.speed = speed_bps.map(|v| v as i64);
                                }
                            }
                            if error.is_none() {
                                let _ = state.db.upsert_profile_extension(ext).await;
                            }
```

4c. After the `let name = { ... };` block (and the error/log handling), add the live re-sort. It runs for TcpPing/RealPing results (success or failure), remaps `selected_sub` only when the result's endpoint is the currently selected one, and invalidates the main-table sort cache:

```rust
                // Live sub-table ordering: re-sort the endpoint's protocols by
                // test priority after every fast/real ping result. `selected_sub`
                // follows its protocol to its new index (only when this result's
                // endpoint is the one currently selected).
                if matches!(test_type, TestType::TcpPing | TestType::RealPing) {
                    let keep = if state.selected_profile_id() == Some(ep_id) {
                        state
                            .selected_sub
                            .and_then(|n| {
                                state.endpoints.iter().find(|r| r.endpoint.id == ep_id)
                            })
                            .and_then(|r| r.protocols.get(n).map(|p| p.id))
                    } else {
                        None
                    };
                    let dns_unresolved = state
                        .endpoints
                        .iter()
                        .find(|r| r.endpoint.id == ep_id)
                        .is_some_and(|r| endpoint_dns_unresolved(state, r));
                    let rounds = state
                        .endpoints
                        .iter()
                        .find(|r| r.endpoint.id == ep_id)
                        .and_then(|r| session_rounds(state, r));
                    if let Some(row) = state
                        .endpoints
                        .iter_mut()
                        .find(|r| r.endpoint.id == ep_id)
                    {
                        row.sort_protocols_by_test_priority(dns_unresolved, rounds);
                        if let Some(pid) = keep {
                            state.selected_sub = row.protocols.iter().position(|p| p.id == pid);
                        }
                    }
                    state.filter_cache_valid.set(false);
                }
```

Add imports: `use xray_tui_db::models::{DELAY_SOURCE_FAST, DELAY_SOURCE_REAL, DELAY_SOURCE_UDP};` (extend the existing models import), and `use crate::ops::profiles::{endpoint_dns_unresolved, session_rounds};` (check the existing `use` path at the top of events.rs).

- [ ] **Step 5: Update the `EndpointInfoUpdated` handler** (`crates/xray-tui/src/ops/events.rs`)

At the top of the arm (before the merge block), capture the prior resolved state; at the end of the arm, re-sort when the host flipped to resolved:

```rust
                // Before the merge: was this endpoint's DNS unresolved?
                let was_resolved = state
                    .endpoint_info
                    .get(&endpoint_id)
                    .is_some_and(|i| !i.resolved_ips.is_empty());
```

and after the existing merge/persist code (end of the arm):

```rust
                // DNS flip (unresolved -> resolved): lift the endpoint's
                // protocols out of the name (bottom) tier.
                let is_resolved = state
                    .endpoint_info
                    .get(&endpoint_id)
                    .is_some_and(|i| !i.resolved_ips.is_empty());
                if !was_resolved && is_resolved {
                    let rounds = state
                        .ping_status
                        .get(&endpoint_id)
                        .map(|ps| (&ps.fast.failed, &ps.real.failed));
                    if let Some(row) = state
                        .endpoints
                        .iter_mut()
                        .find(|r| r.endpoint.id == endpoint_id)
                    {
                        row.sort_protocols_by_test_priority(false, rounds);
                    }
                    state.filter_cache_valid.set(false);
                }
```

- [ ] **Step 6: Set `delay_source` in the batch buffer** (`crates/xray-tui/src/ops/ping.rs:404`)

In the extension construction, add after `ip_info: r.ip_info.clone(),`:

```rust
                        delay_source: Some(match r.ping_type.as_str() {
                            "real" => DELAY_SOURCE_REAL,
                            "udp" => DELAY_SOURCE_UDP,
                            _ => DELAY_SOURCE_FAST,
                        }),
```

Add the consts to the existing `use xray_tui_db::models::{...}` import in ping.rs.

- [ ] **Step 7: Run the tests + clippy**

Run: `cargo test -p xray-tui --lib ops::events && cargo clippy -p xray-tui --all-targets`
Expected: all four new tests PASS, existing events tests PASS, no new warnings.

- [ ] **Step 8: Commit**

```bash
git add crates/xray-tui/src/ops/profiles.rs crates/xray-tui/src/ops/events.rs crates/xray-tui/src/ops/ping.rs
git commit -m "feat(tui): live sub-table re-sort on fast/real ping results"
```

---

### Task 4: Main-table Test column sort

**Files:**
- Modify: `crates/xray-tui/src/types.rs` (rename `SortColumn::Delay` → `Test`)
- Modify: `crates/xray-tui/src/ops/profiles.rs` (Test comparator; import helpers)
- Modify: `crates/xray-tui/src/ui/mod.rs` (menu item 7 + `o`/`O` cycle list)
- Modify: `crates/xray-tui/src/ui/profiles.rs` (column-12 sort mapping)
- Modify: `crates/xray-tui/src/state.rs` (delete dead `compute_filtered_indices`, lines 403-493)
- Test: `crates/xray-tui/src/ops/profiles.rs` `mod tests`

**Interfaces:**
- Consumes: `EndpointRow::best_test_priority_key` (Task 2), `endpoint_dns_unresolved`/`session_rounds` (Task 3)

- [ ] **Step 1: Write the failing sort tests** (`crates/xray-tui/src/ops/profiles.rs` `mod tests`)

The module already has `fake_row(id, host, n_protos)` (protocol ids `id*100 + i`). Add a helper and tests. Check how existing tests construct `AppState` (they use a tempdir `Database` + `AppConfig::default()` — see `set_active_clears_pin_and_falls_back` around line 1077); reuse that pattern:

```rust
    fn set_delay(row: &mut EndpointRow, proto_id: i64, delay: i32, source: Option<i32>) {
        row.extensions.insert(
            proto_id,
            ProfileExtension {
                protocol_id: proto_id,
                delay: Some(delay),
                speed: None,
                sort_order: None,
                ip_info: None,
                delay_source: source,
                protocol_row: toasty::Deferred::from(None::<xray_tui_db::models::ProtocolRow>),
            },
        );
    }

    #[tokio::test]
    async fn test_sort_ranks_by_best_protocol_tier() {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(
            xray_tui_db::Database::open(dir.path().join("t.db"))
                .await
                .unwrap(),
        );
        let mut state = AppState::new(db, AppConfig::default()).await;
        // E1: real-ok 300ms; E2: fast-ok 100ms; E3: untested.
        let mut e1 = fake_row(1, "e1.example", 1);
        set_delay(&mut e1, 101, 300, Some(DELAY_SOURCE_REAL));
        let mut e2 = fake_row(2, "e2.example", 1);
        set_delay(&mut e2, 201, 100, Some(DELAY_SOURCE_FAST));
        let e3 = fake_row(3, "e3.example", 1);
        state.endpoints = vec![e1, e2, e3];
        state.sort_column = SortColumn::Test;
        state.sort_ascending = true;
        state.filter_cache_valid.set(false);

        let order: Vec<i64> = state.filtered_profiles().map(|r| r.endpoint.id).collect();
        assert_eq!(order, vec![1, 2, 3]); // real beats fast beats untested
    }

    #[tokio::test]
    async fn test_sort_sinks_failures_and_dns() {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(
            xray_tui_db::Database::open(dir.path().join("t.db"))
                .await
                .unwrap(),
        );
        let mut state = AppState::new(db, AppConfig::default()).await;
        // E1 untested; E2 fast-ok but real-failed this round; E3 dns-unresolved.
        let e1 = fake_row(1, "e1.example", 1);
        let mut e2 = fake_row(2, "e2.example", 1);
        set_delay(&mut e2, 201, 100, Some(DELAY_SOURCE_FAST));
        let mut e3 = fake_row(3, "dns.example", 1);
        e3.endpoint.host_type = "dns".to_string();
        state.endpoints = vec![e1, e2, e3];
        state.ping_status.insert(
            2,
            crate::types::EndpointPingStatus {
                fast: crate::types::PingRound {
                    seen: std::collections::HashSet::new(),
                    failed: std::collections::HashSet::new(),
                },
                real: crate::types::PingRound {
                    seen: std::collections::HashSet::new(),
                    failed: std::collections::HashSet::from([201]),
                },
            },
        );
        state.sort_column = SortColumn::Test;
        state.sort_ascending = true;
        state.filter_cache_valid.set(false);

        let order: Vec<i64> = state.filtered_profiles().map(|r| r.endpoint.id).collect();
        assert_eq!(order, vec![1, 2, 3]); // untested above real-failure above dns
    }
```

Note: `fake_row`'s protocol ids are `id * 100 + i` — for `n_protos = 1` that is `id*100 + 0`. Verify against the fixture source (lines 1009-1024) and adjust the `set_delay` ids accordingly.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p xray-tui --lib ops::profiles`
Expected: FAIL — no `SortColumn::Test` variant (compile error) and/or untested endpoints sort first under the old `Delay` comparator.

- [ ] **Step 3: Rename the variant + implement the comparator**

3a. `crates/xray-tui/src/types.rs` — rename the enum variant:

```rust
pub enum SortColumn {
    ConfigType,
    Address,
    Port,
    Test,
    Speed,
    Traffic,
    Core,
    LastSeen,
}
```

3b. `crates/xray-tui/src/ops/profiles.rs` — replace the `SortColumn::Delay` arm in `compute_filtered_indices`:

```rust
            SortColumn::Test => {
                let ka = a_row.best_test_priority_key(
                    endpoint_dns_unresolved(state, a_row),
                    session_rounds(state, a_row),
                );
                let kb = b_row.best_test_priority_key(
                    endpoint_dns_unresolved(state, b_row),
                    session_rounds(state, b_row),
                );
                ka.cmp(&kb)
            }
```

The helpers are in this same file (Task 3), so no new imports.

- [ ] **Step 4: Update the UI sites**

4a. `crates/xray-tui/src/ui/mod.rs:408` — `state.sort_column = SortColumn::Delay;` → `SortColumn::Test;`
4b. `crates/xray-tui/src/ui/mod.rs:640` — cycle list entry `SortColumn::Delay,` → `SortColumn::Test,`
4c. `crates/xray-tui/src/ui/profiles.rs:589` — `SortColumn::Delay => Some(12),` → `SortColumn::Test => Some(12),`

- [ ] **Step 5: Delete the dead duplicate comparator** (`crates/xray-tui/src/state.rs:403-493`)

Delete the entire `compute_filtered_indices(&self)` method (private, zero callers — `filtered_profiles` at line 399 delegates to `profiles::filtered_profiles`). It references `SortColumn::Delay`, so it must go with the rename. Verify no remaining `SortColumn::Delay` references anywhere:

Run: `cargo build`
Expected: builds; compiler reports no `SortColumn::Delay` and no unused-method warning.

- [ ] **Step 6: Run the tests + clippy**

Run: `cargo test --workspace && cargo clippy --workspace --all-targets`
Expected: all new + existing tests PASS, clippy clean.

- [ ] **Step 7: Commit**

```bash
git add crates/xray-tui/src/types.rs crates/xray-tui/src/ops/profiles.rs \
        crates/xray-tui/src/ui/mod.rs crates/xray-tui/src/ui/profiles.rs \
        crates/xray-tui/src/state.rs
git commit -m "feat(tui): Test column sorts endpoints by test priority"
```

---

## Self-Review

- **Spec coverage:** tiers (T2 comparator tests) — real/fast/untested/real-err/fast-err/name; fresh-failure dominance (T2 `fresh_failure_dominates_stored_success`, T3 `real_ping_failure_demotes_below_untested`); best-protocol representative (T2 `best_key_returns_min_over_protocols`, T4 `test_sort_ranks_by_best_protocol_tier`); live sub-table re-sort (T3 events tests); DNS flip re-sort (T3 Step 5); load-time order (T2 Step 4); `delay_source` provenance + writers (T1 + T3 Steps 4/6); `SortColumn::Test` rename + manual trigger via `o`/menu item 7 (T4); dead-code removal (T1 `state.rs` `batch_upsert_buffer`, T4 `compute_filtered_indices`); upsert-skip-on-error fix (T3 Step 4b — protects the untested tier from the delay=0 write).
- **Placeholder scan:** every step has concrete code or an exact expected result.
- **Type consistency:** `(u8, i32, i64, i64)` key, `Option<(&HashSet<i64>, &HashSet<i64>)>` rounds, and `delay_source: Option<i32>` are consistent across Tasks 1-4.
