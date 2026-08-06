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
    ConfigType, DnsSetting, Endpoint, EndpointGroup, EndpointId, Group, HostType, Latency,
    ProfileStats, Protocol, ProtocolId, RoutingRule, Security, TrafficStats, Transport,
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

fn zero_traffic() -> TrafficStats {
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
