//! Integration tests for the typed 7-table read paths.
//!
//! Data is seeded through the public pooled-connection accessor with
//! `toasty::create!` (typed writes land in Task 10; until then the read
//! paths are exercised against directly-created rows). Deleted legacy
//! machinery (ping sessions, extensions, server stats) has no tests here —
//! Task 24 rewrites the suite properly.

#![allow(
    clippy::significant_drop_tightening,
    reason = "test db lifetime is the function scope"
)]

use jiff::Timestamp;
use toasty::{Deferred, Json};
use xray_tui_db::Database;
use xray_tui_db::models::{
    ConfigType, DnsSetting, Endpoint, EndpointGroup, EndpointId, ErrorInfo, Group, HostType,
    Latency, ProfileErr, ProfileStats, Protocol, ProtocolId, RoutingRule, Security, TrafficStats,
    Transport,
};
use xray_tui_proto::proto_spec::common::TransportConfig;
use xray_tui_proto::proto_spec::{
    CoreType, ProtocolConfig, ProtocolKind, SecurityConfig, SecurityType, TransportType,
    VlessConfig,
};

/// Helper: create in-memory database.
async fn test_db() -> Database {
    Database::in_memory().await.expect("open in-memory db")
}

fn ts(secs: i64) -> Timestamp {
    Timestamp::from_second(secs).expect("valid ts")
}

fn tcp_transport() -> Transport {
    Transport {
        r#type: TransportType::Tcp,
        data: Deferred::from(Json(TransportConfig::Tcp)),
    }
}

fn no_security() -> Security {
    Security {
        r#type: SecurityType::None,
        sni: None,
        fp: None,
        insecure: None,
        data: Deferred::from(Json(SecurityConfig::default())),
    }
}

fn vless_config() -> ProtocolConfig {
    ProtocolConfig::Vless(VlessConfig {
        uuid: "00000000-0000-0000-0000-000000000000".to_string(),
        uuid_origin: None,
        security: SecurityConfig::default(),
        transport: TransportConfig::Tcp,
        encryption: None,
        flow: None,
        path: None,
        splice: None,
        remarks: None,
    })
}

const fn zero_traffic() -> TrafficStats {
    TrafficStats {
        today_up: 0,
        today_down: 0,
        total_up: 0,
        total_down: 0,
    }
}

/// Insert one endpoint with one protocol and one link at `last_seen`.
async fn seed_endpoint(
    conn: &mut toasty::Connection,
    endpoint_id: i64,
    protocol_id: i64,
    host: &str,
    host_type: HostType,
    port: u16,
    last_seen: i64,
) {
    toasty::create!(Endpoint {
        id: EndpointId::new(endpoint_id),
        host: host.to_string(),
        host_type,
        port,
        ports: Vec::<u16>::new(),
        resolved_as: Vec::<String>::new(),
    })
    .exec(conn)
    .await
    .expect("create endpoint");
    seed_link(conn, endpoint_id, protocol_id, last_seen).await;
}

/// Insert one additional protocol + link for an existing endpoint.
async fn seed_link(
    conn: &mut toasty::Connection,
    endpoint_id: i64,
    protocol_id: i64,
    last_seen: i64,
) {
    toasty::create!(Protocol {
        id: ProtocolId::new(protocol_id),
        sig: protocol_id,
        cred_hash: 0,
        proto_kind: ProtocolKind::Vless,
        transport: tcp_transport(),
        security: no_security(),
        config: Deferred::from(Json(vless_config())),
    })
    .exec(conn)
    .await
    .expect("create protocol");

    toasty::create!(ProfileStats {
        protocol_id: ProtocolId::new(protocol_id),
        endpoint_id: EndpointId::new(endpoint_id),
        core_type: CoreType::Xray,
        config_type: ConfigType::ShareUrl,
        last_seen_at: ts(last_seen),
        task_queue: Vec::<u16>::new(),
        traffic: zero_traffic(),
    })
    .exec(conn)
    .await
    .expect("create link");
}

// ── EndpointRow assembly ────────────────────────────────────────────────

#[tokio::test]
async fn get_active_endpoints_assembles_rows_with_links_and_protocols() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");

    seed_endpoint(&mut conn, 1, 1001, "1.2.3.4", HostType::Ipv4, 443, 10).await;
    seed_link(&mut conn, 1, 1002, 20).await;

    let rows = db.get_active_endpoints(ts(0)).await.expect("active");
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.endpoint.id, EndpointId::new(1));
    assert_eq!(row.endpoint.host, "1.2.3.4");
    assert_eq!(row.endpoint.port, 443);
    assert_eq!(row.links.len(), 2, "links carried per endpoint");
    assert_eq!(row.protocols.len(), 2, "protocols map built from links");
    assert!(row.protocols.contains_key(&ProtocolId::new(1001)));
    assert!(row.protocols.contains_key(&ProtocolId::new(1002)));

    // Newest link first (untested tier, recency order).
    assert_eq!(row.links[0].protocol_id, ProtocolId::new(1002));
    assert_eq!(row.links[1].protocol_id, ProtocolId::new(1001));

    let (link, proto) = row.active_protocol().expect("active protocol");
    assert_eq!(link.protocol_id, ProtocolId::new(1002));
    assert_eq!(proto.proto_kind, ProtocolKind::Vless);
}

#[tokio::test]
async fn large_page_loads_via_batched_in_list() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");

    // 1000 endpoints, each with exactly one link. `load_endpoint_rows`
    // batches the link load with ONE `endpoint_id IN (1000 ids)` statement —
    // comfortably below SQLite's `SQLITE_MAX_VARIABLE_NUMBER` (default
    // 32766; the T8+9 note's ">32k" ceiling). A page at or above the limit
    // would need chunking; 1000 proves the batched path at a realistic
    // large-page scale without a slow 32k+ test.
    for i in 0..1000 {
        seed_endpoint(
            &mut conn,
            i + 1,
            10_000 + i,
            &format!("10.0.{}.{}", i / 250, i % 250),
            HostType::Ipv4,
            443,
            50,
        )
        .await;
    }

    let rows = db.get_active_endpoints(ts(0)).await.expect("page");
    assert_eq!(rows.len(), 1000, "every endpoint on the page");
    assert!(
        rows.iter().all(|r| r.links.len() == 1),
        "each endpoint carries its single link through the batched in_list load"
    );
    assert_eq!(
        rows.iter().map(|r| r.endpoint.id.get()).min(),
        Some(1),
        "page ordered by id"
    );
    assert_eq!(
        rows.iter().map(|r| r.endpoint.id.get()).max(),
        Some(1000),
        "page ordered by id"
    );
}

#[tokio::test]
async fn rows_are_sorted_by_test_priority() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");

    seed_endpoint(&mut conn, 1, 1001, "1.2.3.4", HostType::Ipv4, 443, 10).await;
    // fast-ok with the best latency.
    seed_link_latency(&mut conn, 1, 1002, 20, Some(Latency::Fast { delay: 5 })).await;
    // real-ok (tier 0 beats tier 1 regardless of latency).
    seed_link_latency(
        &mut conn,
        1,
        1003,
        30,
        Some(Latency::Real {
            delay: 200,
            ip: None,
        }),
    )
    .await;

    let rows = db.get_active_endpoints(ts(0)).await.expect("active");
    let row = &rows[0];
    let order: Vec<i64> = row.links.iter().map(|l| l.protocol_id.get()).collect();
    assert_eq!(order, vec![1003, 1002, 1001], "real-ok, fast-ok, untested");
}

#[tokio::test]
async fn dns_unresolved_endpoint_sinks_links_to_bottom() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");

    seed_endpoint(
        &mut conn,
        1,
        1001,
        "unresolved.example",
        HostType::Dns,
        443,
        10,
    )
    .await;
    seed_link_latency(
        &mut conn,
        1,
        1002,
        20,
        Some(Latency::Real { delay: 5, ip: None }),
    )
    .await;

    let rows = db.get_active_endpoints(ts(0)).await.expect("active");
    let row = &rows[0];
    assert_eq!(
        row.best_test_priority_key(true).expect("key").0,
        5,
        "dns-unresolved dominates every link tier"
    );
}

#[tokio::test]
async fn manual_override_shapes_active_link() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");

    seed_endpoint(&mut conn, 1, 1001, "10.10.10.10", HostType::Ipv4, 53, 10).await;
    seed_link(&mut conn, 1, 1002, 20).await;

    let row = db.get_endpoint(EndpointId::new(1)).await.expect("row");
    let mut row = row.expect("row");
    assert_eq!(
        row.active_link().expect("link").protocol_id,
        ProtocolId::new(1002)
    );

    // Override selects the older protocol.
    toasty::update!(row.endpoint {
        manual_protocol_override: Some(ProtocolId::new(1001)),
    })
    .exec(&mut conn)
    .await
    .expect("set override");
    let row = db
        .get_endpoint(EndpointId::new(1))
        .await
        .expect("row")
        .expect("row");
    assert_eq!(
        row.active_link().expect("link").protocol_id,
        ProtocolId::new(1001)
    );
    assert_eq!(
        row.endpoint.manual_protocol_override,
        Some(ProtocolId::new(1001))
    );
}

// ── Active / stale windows ──────────────────────────────────────────────

#[tokio::test]
async fn active_and_stale_windows() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");
    let now = 5_000i64;

    seed_endpoint(&mut conn, 1, 1001, "5.6.7.8", HostType::Ipv4, 443, now).await;
    seed_endpoint(
        &mut conn,
        2,
        2001,
        "9.10.11.12",
        HostType::Ipv4,
        80,
        now - 7_200,
    )
    .await;

    let active = db
        .get_active_endpoints(ts(now - 3_600))
        .await
        .expect("active");
    let active_ids: Vec<i64> = active.iter().map(|r| r.endpoint.id.get()).collect();
    assert_eq!(active_ids, vec![1]);

    let stale = db
        .get_stale_endpoints(ts(now - 3_600), ts(now - 7_200))
        .await
        .expect("stale");
    let stale_ids: Vec<i64> = stale.iter().map(|r| r.endpoint.id.get()).collect();
    assert_eq!(stale_ids, vec![2]);

    let count = db
        .get_stale_count(ts(now - 3_600), ts(now - 7_200))
        .await
        .expect("count");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn stale_ids_match_assembled_rows_on_mixed_dataset() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");
    let now = 10_000i64;
    let active = now - 3_600; // 6400
    let stale = now - 7_200; // 2800

    // 1: active — fresh link (>= active_threshold).
    seed_endpoint(&mut conn, 1, 1001, "1.1.1.1", HostType::Ipv4, 443, now).await;
    // 2: stale — both links inside [stale_threshold, active_threshold).
    seed_endpoint(
        &mut conn,
        2,
        2001,
        "2.2.2.2",
        HostType::Ipv4,
        443,
        now - 5_000,
    )
    .await;
    seed_link(&mut conn, 2, 2002, now - 6_000).await;
    // 3: expired — all links older than stale_threshold.
    seed_endpoint(
        &mut conn,
        3,
        3001,
        "3.3.3.3",
        HostType::Ipv4,
        443,
        now - 10_000,
    )
    .await;
    seed_link(&mut conn, 3, 3002, now - 12_000).await;
    // 4: linkless — vacuously outside both windows (never stale).
    toasty::create!(Endpoint {
        id: EndpointId::new(4),
        host: "4.4.4.4".to_string(),
        host_type: HostType::Ipv4,
        port: 443,
        ports: Vec::<u16>::new(),
        resolved_as: Vec::<String>::new(),
    })
    .exec(&mut conn)
    .await
    .expect("linkless endpoint");
    // 5: active — one stale-aged link plus one fresh link (max decides).
    seed_endpoint(
        &mut conn,
        5,
        5001,
        "5.5.5.5",
        HostType::Ipv4,
        443,
        now - 6_000,
    )
    .await;
    seed_link(&mut conn, 5, 5002, now - 1_000).await;
    // 6: boundary — max exactly == stale_threshold -> stale.
    seed_endpoint(&mut conn, 6, 6001, "6.6.6.6", HostType::Ipv4, 443, stale).await;
    // 7: boundary — max exactly == active_threshold -> NOT stale.
    seed_endpoint(&mut conn, 7, 7001, "7.7.7.7", HostType::Ipv4, 443, active).await;

    let stale_ids = db.get_stale_ids(ts(active), ts(stale)).await.expect("ids");
    let count = db
        .get_stale_count(ts(active), ts(stale))
        .await
        .expect("count");
    let rows = db
        .get_stale_endpoints(ts(active), ts(stale))
        .await
        .expect("rows");

    let expected: Vec<i64> = vec![2, 6];
    let mut got: Vec<i64> = stale_ids.iter().map(|id| id.get()).collect();
    got.sort_unstable();
    assert_eq!(
        got, expected,
        "id-only path selects exactly the stale window"
    );
    assert_eq!(count, expected.len(), "count == id-only path length");
    let mut row_ids: Vec<i64> = rows.iter().map(|r| r.endpoint.id.get()).collect();
    row_ids.sort_unstable();
    assert_eq!(
        row_ids, expected,
        "id-only path agrees with the assembled-row count"
    );
}

// ── Group filter ────────────────────────────────────────────────────────

#[tokio::test]
async fn group_filter_selects_by_group_membership() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");

    seed_endpoint(&mut conn, 1, 1001, "192.168.1.1", HostType::Ipv4, 8080, 100).await;

    // Link endpoint 1 to two groups.
    toasty::create!(EndpointGroup {
        endpoint_id: EndpointId::new(1),
        group_id: "source-a".to_string(),
        last_seen_at: ts(100),
    })
    .exec(&mut conn)
    .await
    .expect("link group a");
    toasty::create!(EndpointGroup {
        endpoint_id: EndpointId::new(1),
        group_id: "source-b".to_string(),
        last_seen_at: ts(100),
    })
    .exec(&mut conn)
    .await
    .expect("link group b");

    let from_a = db
        .get_active_endpoints_by_group("source-a", ts(0))
        .await
        .expect("group a");
    let from_b = db
        .get_active_endpoints_by_group("source-b", ts(0))
        .await
        .expect("group b");
    let from_c = db
        .get_active_endpoints_by_group("source-c", ts(0))
        .await
        .expect("group c");
    assert_eq!(from_a.len(), 1);
    assert_eq!(from_a[0].endpoint.id, EndpointId::new(1));
    assert_eq!(from_b.len(), 1);
    assert!(from_c.is_empty(), "unlinked group matches nothing");
}

// ── Single-row lookups ──────────────────────────────────────────────────

#[tokio::test]
async fn get_endpoint_and_get_by_protocol_id() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");

    seed_endpoint(&mut conn, 7, 3001, "10.0.0.1", HostType::Ipv4, 53, 100).await;

    let row = db.get_endpoint(EndpointId::new(7)).await.expect("get");
    assert_eq!(row.as_ref().expect("row").endpoint.host, "10.0.0.1");
    assert_eq!(row.unwrap().links.len(), 1);

    let by_proto = db
        .get_endpoint_by_protocol_id(ProtocolId::new(3001))
        .await
        .expect("by protocol");
    assert_eq!(by_proto.expect("row").endpoint.id, EndpointId::new(7));

    assert!(
        db.get_endpoint(EndpointId::new(999))
            .await
            .expect("missing")
            .is_none()
    );
    assert!(
        db.get_endpoint_by_protocol_id(ProtocolId::new(9999))
            .await
            .expect("missing")
            .is_none()
    );
}

#[tokio::test]
async fn get_endpoint_returns_linkless_row_with_empty_links() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");

    // Endpoint with NO profile_stats rows. Pre-T8+9 this was an INNER JOIN
    // that returned None; the typed path must return Some with empty links.
    toasty::create!(Endpoint {
        id: EndpointId::new(41),
        host: "linkless.example".to_string(),
        host_type: HostType::Dns,
        port: 443,
        ports: Vec::<u16>::new(),
        resolved_as: Vec::<String>::new(),
    })
    .exec(&mut conn)
    .await
    .expect("linkless endpoint");

    let row = db
        .get_endpoint(EndpointId::new(41))
        .await
        .expect("get")
        .expect("linkless endpoint resolves to Some");
    assert!(row.links.is_empty(), "no ProfileStats rows -> empty links");
    assert!(row.protocols.is_empty(), "protocols map empty");
    assert!(row.active_link().is_none(), "no links -> no active link");
    assert!(
        row.active_protocol().is_none(),
        "no links -> no active protocol"
    );

    assert!(
        db.get_endpoint(EndpointId::new(999_999))
            .await
            .expect("missing")
            .is_none(),
        "nonexistent id -> None"
    );
}

#[tokio::test]
async fn endpoints_by_parent_orders_by_id() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");

    toasty::create!(Endpoint {
        id: EndpointId::new(50),
        host: "dns.example".to_string(),
        host_type: HostType::Dns,
        port: 443,
        ports: Vec::<u16>::new(),
        resolved_as: Vec::<String>::new(),
    })
    .exec(&mut conn)
    .await
    .expect("parent");
    for (id, ip) in [(51, "1.1.1.1"), (52, "2.2.2.2")] {
        toasty::create!(Endpoint {
            id: EndpointId::new(id),
            host: ip.to_string(),
            host_type: HostType::Ipv4,
            port: 443,
            ports: Vec::<u16>::new(),
            parent_id: Some(EndpointId::new(50)),
            resolved_as: Vec::<String>::new(),
        })
        .exec(&mut conn)
        .await
        .expect("child");
    }

    let children = db
        .endpoints_by_parent(EndpointId::new(50))
        .await
        .expect("children");
    let ids: Vec<i64> = children.iter().map(|e| e.id.get()).collect();
    assert_eq!(ids, vec![51, 52]);
}

// ── Newtype columns ─────────────────────────────────────────────────────

#[tokio::test]
async fn newtype_ids_roundtrip_through_reads() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");

    let ep_id = EndpointId::new(424_242);
    let proto_id = ProtocolId::new(1_001_001);
    seed_endpoint(
        &mut conn,
        ep_id.get(),
        proto_id.get(),
        "1.1.1.1",
        HostType::Ipv4,
        443,
        50,
    )
    .await;

    let row = db.get_endpoint(ep_id).await.expect("get").expect("row");
    assert_eq!(row.endpoint.id, ep_id);
    assert_eq!(row.links[0].protocol_id, proto_id);
    let proto = row.protocols.get(&proto_id).expect("protocol in map");
    assert_eq!(proto.id, proto_id);

    let by_proto = db
        .get_endpoint_by_protocol_id(proto_id)
        .await
        .expect("by protocol")
        .expect("row");
    assert_eq!(by_proto.endpoint.id, ep_id);
}

// ── Groups ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn default_group_created_on_init() {
    let db = test_db().await;

    let groups = db.get_all_groups().await.expect("groups");
    assert!(!groups.is_empty(), "should have at least the default group");

    let default_groups: Vec<&Group> = groups
        .iter()
        .filter(|g| g.name.as_deref() == Some("Default"))
        .collect();
    assert_eq!(default_groups.len(), 1, "exactly one Default group");
    assert_eq!(default_groups[0].sort_order, Some(0));
}

#[tokio::test]
async fn groups_due_update_respects_refresh_interval() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");
    let now = Timestamp::now();
    let hour_ago = now
        .checked_sub(jiff::Span::new().hours(1))
        .expect("subtract");

    // Due: never refreshed.
    toasty::create!(Group {
        id: "g-never".to_string(),
        name: Some("never".to_string()),
        url: Some("https://example.com/sub".to_string()),
        enabled: true,
    })
    .exec(&mut conn)
    .await
    .expect("group");

    // Due: refreshed 1h ago with a 30-minute interval.
    toasty::create!(Group {
        id: "g-due".to_string(),
        name: Some("due".to_string()),
        url: Some("https://example.com/sub2".to_string()),
        enabled: true,
        refresh_interval: Some(30),
        last_refreshed: Some(hour_ago),
    })
    .exec(&mut conn)
    .await
    .expect("group");

    // Not due: refreshed now.
    toasty::create!(Group {
        id: "g-fresh".to_string(),
        name: Some("fresh".to_string()),
        url: Some("https://example.com/sub3".to_string()),
        enabled: true,
        last_refreshed: Some(now),
    })
    .exec(&mut conn)
    .await
    .expect("group");

    // Not due: disabled.
    toasty::create!(Group {
        id: "g-off".to_string(),
        name: Some("off".to_string()),
        url: Some("https://example.com/sub4".to_string()),
        enabled: false,
    })
    .exec(&mut conn)
    .await
    .expect("group");

    // Not due: no url.
    toasty::create!(Group {
        id: "g-nourl".to_string(),
        name: Some("nourl".to_string()),
        url: None,
        enabled: true,
    })
    .exec(&mut conn)
    .await
    .expect("group");

    let due = db.get_groups_due_update().await.expect("due");
    let mut due_ids: Vec<&str> = due.iter().map(|g| g.id.as_str()).collect();
    due_ids.sort_unstable();
    assert_eq!(due_ids, vec!["g-due", "g-never"]);
}

// ── Routing rules + DNS settings ────────────────────────────────────────

#[tokio::test]
async fn routing_rules_and_dns_settings_roundtrip() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");

    assert!(db.get_dns_settings().await.expect("empty").is_none());

    toasty::create!(DnsSetting {
        id: "dns-1".to_string(),
        name: Some("main".to_string()),
        servers: ["1.1.1.1".to_string()],
        hosts: Vec::<String>::new(),
        disable_cache: true,
        disable_fallback: false,
    })
    .exec(&mut conn)
    .await
    .expect("dns setting");

    let dns = db.get_dns_settings().await.expect("dns").expect("row");
    assert_eq!(dns.servers, vec!["1.1.1.1".to_string()]);
    assert!(dns.disable_cache);

    toasty::create!(RoutingRule {
        id: "rule-1".to_string(),
        r#type: 0,
        domains: ["example.com".to_string()],
        ips: Vec::<String>::new(),
        inbound_tags: Vec::<String>::new(),
        ports: [443],
        source_ports: Vec::<u16>::new(),
        protocols: Vec::<String>::new(),
        sort_order: Some(2),
    })
    .exec(&mut conn)
    .await
    .expect("routing rule");

    let rules = db.get_all_routing_rules().await.expect("rules");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].domains, vec!["example.com".to_string()]);
    assert_eq!(rules[0].ports, vec![443]);
}

/// Seed a protocol + link with an explicit latency value.
async fn seed_link_latency(
    conn: &mut toasty::Connection,
    endpoint_id: i64,
    protocol_id: i64,
    last_seen: i64,
    latency: Option<Latency>,
) {
    toasty::create!(Protocol {
        id: ProtocolId::new(protocol_id),
        sig: protocol_id,
        cred_hash: 0,
        proto_kind: ProtocolKind::Vless,
        transport: tcp_transport(),
        security: no_security(),
        config: Deferred::from(Json(vless_config())),
    })
    .exec(conn)
    .await
    .expect("create protocol");

    toasty::create!(ProfileStats {
        protocol_id: ProtocolId::new(protocol_id),
        endpoint_id: EndpointId::new(endpoint_id),
        core_type: CoreType::Xray,
        config_type: ConfigType::ShareUrl,
        last_seen_at: ts(last_seen),
        latency,
        task_queue: Vec::<u16>::new(),
        traffic: zero_traffic(),
    })
    .exec(conn)
    .await
    .expect("create link");
}

// ── Typed writes (Task 10) ──────────────────────────────────────────────

#[tokio::test]
async fn purge_expired_deletes_expired_and_linkless_keeps_fresh() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");

    // Endpoint 1: every link older than the cutoff; group link to cascade.
    seed_endpoint(&mut conn, 1, 1001, "1.1.1.1", HostType::Ipv4, 443, 100).await;
    seed_link(&mut conn, 1, 1002, 200).await;
    toasty::create!(EndpointGroup {
        endpoint_id: EndpointId::new(1),
        group_id: "g1".to_string(),
        last_seen_at: ts(50),
    })
    .exec(&mut conn)
    .await
    .expect("group link");

    // Endpoint 2: linkless — vacuously expired (old COALESCE(MAX,0) < cutoff).
    toasty::create!(Endpoint {
        id: EndpointId::new(2),
        host: "2.2.2.2".to_string(),
        host_type: HostType::Ipv4,
        port: 443,
        ports: Vec::<u16>::new(),
        resolved_as: Vec::<String>::new(),
    })
    .exec(&mut conn)
    .await
    .expect("linkless endpoint");

    // Endpoint 3: shares protocol 1002 with endpoint 1, but has a fresh link.
    seed_endpoint(&mut conn, 3, 3001, "3.3.3.3", HostType::Ipv4, 443, 1500).await;
    toasty::create!(ProfileStats {
        protocol_id: ProtocolId::new(1002),
        endpoint_id: EndpointId::new(3),
        core_type: CoreType::Xray,
        config_type: ConfigType::ShareUrl,
        last_seen_at: ts(1600),
        task_queue: Vec::<u16>::new(),
        traffic: zero_traffic(),
    })
    .exec(&mut conn)
    .await
    .expect("shared link on fresh endpoint");

    let purged = db.purge_expired(ts(1000)).await.expect("purge");
    assert_eq!(purged, 2, "endpoint 1 + linkless endpoint 2 purged");

    // Endpoints 1 and 2 gone; endpoint 3 survives with its links.
    assert!(
        Endpoint::filter_by_id(EndpointId::new(1))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .is_none()
    );
    assert!(
        Endpoint::filter_by_id(EndpointId::new(2))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .is_none()
    );
    assert!(
        Endpoint::filter_by_id(EndpointId::new(3))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .is_some()
    );

    // Cascade: endpoint 1's links and group links are gone.
    let links1: Vec<ProfileStats> =
        ProfileStats::filter(ProfileStats::fields().endpoint_id().eq(EndpointId::new(1)))
            .exec(&mut conn)
            .await
            .expect("links");
    assert!(links1.is_empty(), "expired endpoint's links cascade");
    let groups1: Vec<EndpointGroup> =
        EndpointGroup::filter(EndpointGroup::fields().endpoint_id().eq(EndpointId::new(1)))
            .exec(&mut conn)
            .await
            .expect("group links");
    assert!(groups1.is_empty(), "expired endpoint's group links cascade");

    // Orphan protocol cleanup: 1001 has zero remaining links -> deleted;
    // 1002 still has endpoint 3's link -> survives.
    assert!(
        Protocol::filter_by_id(ProtocolId::new(1001))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .is_none(),
        "orphan protocol purged"
    );
    assert!(
        Protocol::filter_by_id(ProtocolId::new(1002))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .is_some(),
        "protocol with a surviving link kept"
    );
}

#[tokio::test]
async fn delete_endpoint_cascades_and_purges_orphan_protocols() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");

    // Endpoint 1: links to 1001 and 1002; group link.
    seed_endpoint(&mut conn, 1, 1001, "1.1.1.1", HostType::Ipv4, 443, 10).await;
    seed_link(&mut conn, 1, 1002, 20).await;
    toasty::create!(EndpointGroup {
        endpoint_id: EndpointId::new(1),
        group_id: "g1".to_string(),
        last_seen_at: ts(5),
    })
    .exec(&mut conn)
    .await
    .expect("group link");

    // Endpoint 2: shares protocol 1002 (survives while its link remains) and
    // owns protocol 3001.
    toasty::create!(Endpoint {
        id: EndpointId::new(2),
        host: "2.2.2.2".to_string(),
        host_type: HostType::Ipv4,
        port: 443,
        ports: Vec::<u16>::new(),
        resolved_as: Vec::<String>::new(),
    })
    .exec(&mut conn)
    .await
    .expect("endpoint 2");
    toasty::create!(ProfileStats {
        protocol_id: ProtocolId::new(1002),
        endpoint_id: EndpointId::new(2),
        core_type: CoreType::Xray,
        config_type: ConfigType::ShareUrl,
        last_seen_at: ts(30),
        task_queue: Vec::<u16>::new(),
        traffic: zero_traffic(),
    })
    .exec(&mut conn)
    .await
    .expect("shared link");
    seed_link(&mut conn, 2, 3001, 40).await;

    db.delete_endpoint(EndpointId::new(1))
        .await
        .expect("delete");

    // Endpoint 1 gone; links + group links cascade.
    assert!(
        Endpoint::filter_by_id(EndpointId::new(1))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .is_none()
    );
    let links1: Vec<ProfileStats> =
        ProfileStats::filter(ProfileStats::fields().endpoint_id().eq(EndpointId::new(1)))
            .exec(&mut conn)
            .await
            .expect("links");
    assert!(links1.is_empty());
    let groups1: Vec<EndpointGroup> =
        EndpointGroup::filter(EndpointGroup::fields().endpoint_id().eq(EndpointId::new(1)))
            .exec(&mut conn)
            .await
            .expect("group links");
    assert!(groups1.is_empty());

    // 1001 is now orphaned -> deleted; 1002 survives via endpoint 2.
    assert!(
        Protocol::filter_by_id(ProtocolId::new(1001))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .is_none(),
        "protocol orphaned by the delete is cleaned up"
    );
    assert!(
        Protocol::filter_by_id(ProtocolId::new(1002))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .is_some(),
        "protocol with a remaining link survives"
    );

    // Deleting the last endpoint holding 1002 orphans it too.
    db.delete_endpoint(EndpointId::new(2))
        .await
        .expect("delete 2");
    assert!(
        Protocol::filter_by_id(ProtocolId::new(1002))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .is_none()
    );
    assert!(
        Protocol::filter_by_id(ProtocolId::new(3001))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .is_none()
    );
}

#[tokio::test]
async fn clear_all_stats_zeroes_traffic_and_clears_results() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");
    seed_endpoint(&mut conn, 1, 1001, "1.1.1.1", HostType::Ipv4, 443, 10).await;
    seed_link_latency(
        &mut conn,
        1,
        1002,
        20,
        Some(Latency::Real {
            delay: 12,
            ip: None,
        }),
    )
    .await;
    seed_endpoint(&mut conn, 2, 2001, "2.2.2.2", HostType::Ipv4, 443, 30).await;

    for pid in [1001, 1002, 2001] {
        ProfileStats::filter_by_protocol_id_and_endpoint_id(
            ProtocolId::new(pid),
            EndpointId::new(if pid == 2001 { 2 } else { 1 }),
        )
        .update()
        .traffic(TrafficStats {
            today_up: 1,
            today_down: 2,
            total_up: 3,
            total_down: 4,
        })
        .speed_bps(Some(5_000))
        .error(Some(ErrorInfo {
            kind: ProfileErr::Fast,
            text: "boom".to_string(),
        }))
        .exec(&mut conn)
        .await
        .expect("seed stats");
    }

    db.clear_all_stats().await.expect("clear");

    let links: Vec<ProfileStats> = ProfileStats::all().exec(&mut conn).await.expect("links");
    assert_eq!(links.len(), 3);
    for link in links {
        assert_eq!(link.traffic, zero_traffic(), "traffic zeroed on every row");
        assert_eq!(link.latency, None, "latency cleared");
        assert_eq!(link.speed_bps, None, "speed cleared");
        assert_eq!(link.error, None, "error cleared");
    }
}

#[tokio::test]
async fn scheduler_state_occ_rejects_stale_and_retries_after_reload() {
    let db = test_db().await;
    let mut conn = db.connection().await.expect("connection");
    seed_endpoint(&mut conn, 1, 1001, "1.1.1.1", HostType::Ipv4, 443, 10).await;

    // Two handles loaded at the same version.
    let mut h1 = ProfileStats::filter_by_protocol_id_and_endpoint_id(
        ProtocolId::new(1001),
        EndpointId::new(1),
    )
    .first()
    .exec(&mut conn)
    .await
    .expect("load")
    .expect("link");
    let mut h2 = ProfileStats::filter_by_protocol_id_and_endpoint_id(
        ProtocolId::new(1001),
        EndpointId::new(1),
    )
    .first()
    .exec(&mut conn)
    .await
    .expect("load")
    .expect("link");
    assert_eq!(
        h1.version, h2.version,
        "both handles start at the same version"
    );

    // Writer A wins via an instance update.
    toasty::update!(h1 {
        task_id: Some(1),
        task_queue: vec![10, 11],
    })
    .exec(&mut conn)
    .await
    .expect("writer A");

    // Writer B from the same stale snapshot is rejected by the #[version] guard.
    let err = toasty::update!(h2 {
        task_id: Some(2),
        task_queue: vec![20, 21],
    })
    .exec(&mut conn)
    .await
    .expect_err("stale writer must fail the version check");
    assert!(err.is_condition_failed(), "OCC must reject the stale write");

    // The scheduler write reloads (fresh version) and succeeds; the final
    // state is the last writer's.
    db.update_scheduler_state(
        ProtocolId::new(1001),
        EndpointId::new(1),
        Some(3),
        &[30, 31],
    )
    .await
    .expect("scheduler update");

    let link = ProfileStats::filter_by_protocol_id_and_endpoint_id(
        ProtocolId::new(1001),
        EndpointId::new(1),
    )
    .first()
    .exec(&mut conn)
    .await
    .expect("read")
    .expect("link");
    assert_eq!(link.task_id, Some(3));
    assert_eq!(link.task_queue, vec![30, 31], "whole vector replaced");

    // Whole-vector replace + clearing task_id.
    db.update_scheduler_state(ProtocolId::new(1001), EndpointId::new(1), None, &[40])
        .await
        .expect("scheduler replace");
    let link = ProfileStats::filter_by_protocol_id_and_endpoint_id(
        ProtocolId::new(1001),
        EndpointId::new(1),
    )
    .first()
    .exec(&mut conn)
    .await
    .expect("read")
    .expect("link");
    assert_eq!(link.task_id, None);
    assert_eq!(link.task_queue, vec![40]);
}

#[tokio::test]
async fn subscription_upsert_flow_assembles_group_rows() {
    let db = test_db().await;

    // The TUI's persist_parsed sequence: endpoint, protocol, link, and
    // group-link upserts in dependency order.
    db.upsert_endpoint(&Endpoint {
        id: EndpointId::new(1),
        host: "sub.example".to_string(),
        host_type: HostType::Dns,
        port: 443,
        ports: Vec::<u16>::new(),
        parent_id: None,
        last_source: Some("g1".to_string()),
        manual_protocol_override: None,
        resolved_as: Vec::<String>::new(),
        resolved_at: None,
        created_at: ts(0),
        links: Deferred::default(),
        group_links: Deferred::default(),
    })
    .await
    .expect("upsert endpoint");

    db.upsert_protocol(&Protocol {
        id: ProtocolId::new(1001),
        sig: 1001,
        cred_hash: 0,
        proto_kind: ProtocolKind::Vless,
        transport: tcp_transport(),
        security: no_security(),
        config: Deferred::from(Json(vless_config())),
        created_at: ts(0),
        links: Deferred::default(),
    })
    .await
    .expect("upsert protocol");

    let link = ProfileStats {
        protocol_id: ProtocolId::new(1001),
        endpoint_id: EndpointId::new(1),
        core_type: CoreType::Xray,
        config_type: ConfigType::ShareUrl,
        last_used_at: None,
        last_seen_at: ts(50),
        task_id: None,
        task_queue: Vec::<u16>::new(),
        latency: None,
        speed_bps: None,
        error: None,
        traffic: zero_traffic(),
        created_at: ts(0),
        updated_at: ts(0),
        version: 1,
        protocol: Deferred::default(),
        endpoint: Deferred::default(),
    };
    // Twice: the composite-key upsert must dedup by (endpoint_id, protocol_id).
    db.upsert_link(&link).await.expect("upsert link");
    db.upsert_link(&link).await.expect("upsert link again");

    db.upsert_endpoint_group_link(&EndpointGroup {
        endpoint_id: EndpointId::new(1),
        group_id: "g1".to_string(),
        last_seen_at: ts(50),
        sort_order: None,
        endpoint: Deferred::default(),
        group: Deferred::default(),
    })
    .await
    .expect("upsert group link");

    // The subscription-shaped sequence assembles into one group row.
    let rows = db
        .get_active_endpoints_by_group("g1", ts(0))
        .await
        .expect("group");
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.endpoint.id, EndpointId::new(1));
    assert_eq!(row.links.len(), 1, "dedup by (endpoint_id, protocol_id)");
    assert_eq!(row.links[0].protocol_id, ProtocolId::new(1001));
    let (active_link, proto) = row.active_protocol().expect("active protocol");
    assert_eq!(active_link.protocol_id, ProtocolId::new(1001));
    assert_eq!(proto.proto_kind, ProtocolKind::Vless);

    // A threshold past the link's last_seen_at drops the endpoint.
    assert!(
        db.get_active_endpoints_by_group("g1", ts(100))
            .await
            .expect("fresh threshold")
            .is_empty()
    );
}

// ── Schema behavior ─────────────────────────────────────────────────────

/// Extract the first INTEGER column of the first row (PRAGMA reads).
fn first_i64(rows: &[toasty::stmt::Value]) -> Option<i64> {
    rows.first().and_then(|v| match v {
        toasty::stmt::Value::Record(fields) => fields.first().and_then(|f| match f {
            toasty::stmt::Value::I64(n) => Some(*n),
            _ => None,
        }),
        _ => None,
    })
}

#[tokio::test]
async fn fresh_open_creates_schema_and_sets_user_version_tag() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("fresh.db");

    let db = Database::open(&path).await.expect("fresh open");
    let mut conn = db.connection().await.expect("connection");

    // Fresh open writes the 7-table schema AND tags it user_version=5 so a
    // reopen can skip push_schema.
    let rows = toasty::sql::query("PRAGMA user_version")
        .exec(&mut conn)
        .await
        .expect("read version");
    assert_eq!(
        first_i64(&rows),
        Some(5),
        "fresh open must tag the schema user_version=5"
    );
    let rows = toasty::sql::query(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' \
         AND name IN ('endpoints', 'protocols', 'profile_stats', \
                       'endpoint_groups', 'groups', 'routing_rules', 'dns_settings')",
    )
    .exec(&mut conn)
    .await
    .expect("count tables");
    assert_eq!(first_i64(&rows), Some(7), "7-table schema created");

    // Seed data, then reopen: the tag preserves both schema and data.
    toasty::create!(Endpoint {
        id: EndpointId::new(9),
        host: "9.9.9.9".to_string(),
        host_type: HostType::Ipv4,
        port: 443,
        ports: Vec::<u16>::new(),
        resolved_as: Vec::<String>::new(),
    })
    .exec(&mut conn)
    .await
    .expect("seed endpoint");
    drop(conn);
    drop(db);

    let db2 = Database::open(&path).await.expect("reopen");
    let mut conn = db2.connection().await.expect("connection");
    let rows = toasty::sql::query("PRAGMA user_version")
        .exec(&mut conn)
        .await
        .expect("read version");
    assert_eq!(first_i64(&rows), Some(5), "reopen keeps the schema tag");
    assert!(
        Endpoint::filter_by_id(EndpointId::new(9))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .is_some(),
        "reopen preserves data"
    );
}

#[tokio::test]
async fn in_memory_db_has_full_schema_and_roundtrips() {
    let db = Database::in_memory().await.expect("in-memory db");
    let mut conn = db.connection().await.expect("connection");

    let rows = toasty::sql::query(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' \
         AND name IN ('endpoints', 'protocols', 'profile_stats', \
                       'endpoint_groups', 'groups', 'routing_rules', 'dns_settings')",
    )
    .exec(&mut conn)
    .await
    .expect("count tables");
    assert_eq!(
        first_i64(&rows),
        Some(7),
        "in_memory() has the 7-table schema"
    );

    seed_endpoint(&mut conn, 1, 1001, "1.1.1.1", HostType::Ipv4, 443, 50).await;
    let row = db
        .get_endpoint(EndpointId::new(1))
        .await
        .expect("get")
        .expect("row");
    assert_eq!(
        row.links.len(),
        1,
        "write + read roundtrip through in_memory()"
    );
}
