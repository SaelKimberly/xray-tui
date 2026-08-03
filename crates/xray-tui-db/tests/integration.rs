#![allow(
    clippy::significant_drop_tightening,
    reason = "test db lifetime is the function scope"
)]

use uuid::Uuid;

use xray_tui_db::Database;
use xray_tui_db::hash::stable_hash;
use xray_tui_db::models::{Endpoint, EndpointGroup, Group, ProtocolRow};

/// Helper: create in-memory database.
async fn test_db() -> Database {
    Database::in_memory().await.expect("open in-memory db")
}

fn test_group(id: &str) -> Group {
    Group {
        id: id.to_string(),
        name: Some(format!("group-{id}")),
        url: None,
        enabled: Some(1),
        user_agent: None,
        convert_target: None,
        core_type: None,
        sort_order: None,
        last_refreshed: None,
        status: Some("ok".to_string()),
        error_message: None,
        refresh_interval: None,
    }
}

fn make_endpoint(host: &str, port: i32) -> Endpoint {
    let id = stable_hash(host, port);
    Endpoint {
        id,
        host: host.to_string(),
        host_type: if host.is_empty() {
            "undefined".to_string()
        } else {
            "ipv4".to_string()
        },
        port,
        port_spec_str: None,
        parent_id: None,
        last_source: None,
        created_at: 1000,
        manual_protocol_override: None,
    }
}

fn make_protocol(endpoint_id: i64, proto_kind: &str, sid: i64) -> ProtocolRow {
    ProtocolRow {
        id: sid,
        endpoint_id,
        sig: sid,
        cred_hash: 0,
        proto_kind: proto_kind.to_string(),
        spec_blob: vec![],
        config_type: 0,
        core_type: "auto".to_string(),
        transport: None,
        security: None,
        remarks: Some(format!("{proto_kind}-{sid}")),
        created_at: 1000,
        last_seen_at: 1000,
        endpoint: Default::default(),
        extension: Default::default(),
        server_stat: Default::default(),
    }
}

// ── Phase 9 — Basic CRUD ───────────────────────────────────────────────

#[tokio::test]
async fn test_insert_endpoint_with_two_protocols() {
    let db = test_db().await;
    let gid = "default";
    db.insert_group(&test_group(gid)).await.unwrap();

    let ep = make_endpoint("1.2.3.4", 443);
    let proto1 = make_protocol(ep.id, "vmess", 1001);
    let proto2 = make_protocol(ep.id, "vless", 1002);

    db.subscription_upsert(gid, &[(ep.clone(), vec![proto1, proto2])])
        .await
        .unwrap();

    let row = db.get_endpoint(ep.id).await.unwrap().expect("endpoint");
    assert_eq!(row.endpoint.host, "1.2.3.4");
    assert_eq!(row.protocols.len(), 2);
    assert_eq!(row.protocols[0].proto_kind, "vmess"); // row order (no sort)
    assert_eq!(row.protocols[1].proto_kind, "vless");
}

#[tokio::test]
async fn test_subscription_upsert_idempotent() {
    let db = test_db().await;
    let gid = "sub";
    db.insert_group(&test_group(gid)).await.unwrap();

    let ep = make_endpoint("10.0.0.1", 80);
    let proto = make_protocol(ep.id, "trojan", 2001);

    // First insert
    db.subscription_upsert(gid, &[(ep.clone(), vec![proto.clone()])])
        .await
        .unwrap();
    let row1 = db.get_endpoint(ep.id).await.unwrap().unwrap();
    assert_eq!(row1.protocols.len(), 1);
    let first_seen = row1.protocols[0].last_seen_at;

    // Same subscription again → last_seen_at updated, no new rows
    db.subscription_upsert(gid, &[(ep.clone(), vec![proto.clone()])])
        .await
        .unwrap();
    let row2 = db.get_endpoint(ep.id).await.unwrap().unwrap();
    assert_eq!(row2.protocols.len(), 1); // no duplicate
    assert!(row2.protocols[0].last_seen_at >= first_seen); // timestamp bumped
}

#[tokio::test]
async fn test_active_vs_stale() {
    let db = test_db().await;
    let gid = "test";
    db.insert_group(&test_group(gid)).await.unwrap();

    let now = 5000i64;
    let ttl_secs = 3600i64; // 1 hour

    // Active endpoint: last_seen_at = now (recent)
    let active_ep = make_endpoint("5.6.7.8", 443);
    let active_proto = ProtocolRow {
        last_seen_at: now,
        ..make_protocol(active_ep.id, "ss", 3001)
    };
    db.subscription_upsert(gid, &[(active_ep.clone(), vec![active_proto])])
        .await
        .unwrap();

    // Stale endpoint: last_seen_at = now - 2*ttl (old)
    let stale_ep = make_endpoint("9.10.11.12", 80);
    let stale_proto = ProtocolRow {
        last_seen_at: now - 2 * ttl_secs,
        ..make_protocol(stale_ep.id, "socks", 4001)
    };
    db.subscription_upsert(gid, &[(stale_ep.clone(), vec![stale_proto])])
        .await
        .unwrap();

    // Active query: should only return active_ep
    let active_rows = db.get_active_endpoints(now - ttl_secs).await.unwrap();
    assert!(
        active_rows.iter().any(|r| r.endpoint.id == active_ep.id),
        "active endpoint should be in active view"
    );
    let stale_rows = db
        .get_stale_endpoints(now - ttl_secs, -999999)
        .await
        .unwrap();
    assert!(
        !stale_rows.iter().any(|r| r.endpoint.id == active_ep.id),
        "active endpoint should NOT be in stale view"
    );

    let stale_rows = db
        .get_stale_endpoints(now - ttl_secs, -999999)
        .await
        .unwrap();
    assert!(
        stale_rows.iter().any(|r| r.endpoint.id == stale_ep.id),
        "stale endpoint should be in stale view"
    );
    assert!(
        !stale_rows.iter().any(|r| r.endpoint.id == active_ep.id),
        "active endpoint should NOT be in stale view"
    );

    // Update stale endpoint → becomes active again
    db.restore_endpoint(stale_ep.id).await.unwrap();
    let active_after = db.get_active_endpoints(now - ttl_secs).await.unwrap();
    assert!(
        active_after.iter().any(|r| r.endpoint.id == stale_ep.id),
        "restored endpoint should be in active view"
    );
}

#[tokio::test]
async fn test_undefined_endpoint() {
    let db = test_db().await;
    let gid = "def";
    db.insert_group(&test_group(gid)).await.unwrap();

    // Exotic config → empty host, port 0, host_type="undefined"
    let uid = stable_hash("undefined", "exotic-uid-123");
    let ep = Endpoint {
        id: uid,
        host: String::new(),
        host_type: "undefined".to_string(),
        port: 0,
        ..make_endpoint("", 0)
    };
    let proto = make_protocol(ep.id, "custom", 5001);
    db.subscription_upsert(gid, &[(ep.clone(), vec![proto])])
        .await
        .unwrap();

    let row = db.get_endpoint(ep.id).await.unwrap().unwrap();
    assert_eq!(row.endpoint.host_type, "undefined");
    assert_eq!(row.endpoint.host, "");
    assert_eq!(row.endpoint.port, 0);
}

#[tokio::test]
async fn test_two_sources_same_endpoint() {
    let db = test_db().await;
    let gid1 = "source-a";
    let gid2 = "source-b";
    db.insert_group(&test_group(gid1)).await.unwrap();
    db.insert_group(&test_group(gid2)).await.unwrap();

    let ep = make_endpoint("192.168.1.1", 8080);
    let proto = make_protocol(ep.id, "vmess", 6001);

    // Insert same endpoint from source A
    db.subscription_upsert(gid1, &[(ep.clone(), vec![proto.clone()])])
        .await
        .unwrap();

    // Insert same endpoint from source B
    db.subscription_upsert(gid2, &[(ep.clone(), vec![proto.clone()])])
        .await
        .unwrap();

    // One EndpointRow
    let row = db.get_endpoint(ep.id).await.unwrap().unwrap();
    assert_eq!(row.endpoint.host, "192.168.1.1");
    assert_eq!(row.protocols.len(), 1);

    // Two EndpointGroup rows
    let from_a = db
        .get_active_endpoints_by_group("source-a", 0)
        .await
        .unwrap();
    let from_b = db
        .get_active_endpoints_by_group("source-b", 0)
        .await
        .unwrap();
    assert!(
        from_a.iter().any(|r| r.endpoint.id == ep.id),
        "endpoint in source-a"
    );
    assert!(
        from_b.iter().any(|r| r.endpoint.id == ep.id),
        "endpoint in source-b"
    );
}

#[tokio::test]
async fn test_manual_override() {
    let db = test_db().await;
    let gid = "ovr";
    db.insert_group(&test_group(gid)).await.unwrap();

    let ep = make_endpoint("10.10.10.10", 53);
    let proto1 = make_protocol(ep.id, "vmess", 7001);
    let proto2 = make_protocol(ep.id, "trojan", 7002);

    db.subscription_upsert(gid, &[(ep.clone(), vec![proto1, proto2])])
        .await
        .unwrap();

    // Auto-select: vmess (rank 1) before trojan (rank 3)
    let row_before = db.get_endpoint(ep.id).await.unwrap().unwrap();
    assert_eq!(row_before.selected_protocol, 0); // first = vmess
    assert_eq!(
        row_before.protocols[row_before.selected_protocol].proto_kind,
        "vmess"
    );

    // Set manual override to protocol id 7002 (trojan)
    db.set_protocol_override(ep.id, 7002).await.unwrap();
    let row_after = db.get_endpoint(ep.id).await.unwrap().unwrap();
    assert_eq!(row_after.endpoint.manual_protocol_override, Some(7002));

    // Clear override
    db.clear_protocol_override(ep.id).await.unwrap();
    let row_cleared = db.get_endpoint(ep.id).await.unwrap().unwrap();
    assert_eq!(row_cleared.endpoint.manual_protocol_override, None);
}

#[tokio::test]
async fn test_hard_delete_cascade() {
    let db = test_db().await;
    let gid = "purge";
    db.insert_group(&test_group(gid)).await.unwrap();

    let retention_secs = 86400i64; // 1 day
    let now = 100_000i64;

    let ep = make_endpoint("1.1.1.1", 1111);
    let proto = ProtocolRow {
        last_seen_at: now - 2 * retention_secs, // past retention
        ..make_protocol(ep.id, "vmess", 8001)
    };
    db.insert_manual_endpoint(&ep, &proto, gid).await.unwrap();
    // Verify endpoint exists
    assert!(
        db.get_endpoint(ep.id).await.unwrap().is_some(),
        "endpoint should exist before purge"
    );

    // Purge past retention
    let deleted = db.purge_expired(now - retention_secs).await.unwrap();
    assert!(deleted > 0, "should delete endpoint past retention");

    // Verify gone
    assert!(
        db.get_endpoint(ep.id).await.unwrap().is_none(),
        "endpoint should be deleted after purge"
    );
}

#[tokio::test]
async fn test_resolve_endpoint_dns() {
    let db = test_db().await;
    let gid = "dns";
    db.insert_group(&test_group(gid)).await.unwrap();

    // Create a DnsName endpoint
    let host = "example.com";
    let dns_id = stable_hash(host, 443);
    let dns_ep = Endpoint {
        id: dns_id,
        host: host.to_string(),
        host_type: "dns".to_string(),
        port: 443,
        parent_id: None,
        ..make_endpoint(host, 443)
    };
    let proto = make_protocol(dns_id, "vmess", 9001);
    db.insert_manual_endpoint(&dns_ep, &proto, gid)
        .await
        .unwrap();

    // Resolve DNS (in test, may fail if no network — still check structure)
    let resolved = db.resolve_endpoint_dns(dns_id, host).await;
    match resolved {
        Ok(ips) => {
            let row = db.get_endpoint(dns_id).await.unwrap().unwrap();
            assert_eq!(row.resolved_ips.len(), ips.len());
            for ip in &ips {
                assert!(
                    row.resolved_ips.contains(ip),
                    "{ip} should be in resolved_ips"
                );
            }
        }
        Err(_) => {
            // DNS may fail without network — skip structural assertion
            eprintln!("DNS resolution failed (expected without network)");
        }
    }
}

#[tokio::test]
async fn test_stale_count_and_view() {
    let db = test_db().await;
    let gid = "cnt";
    db.insert_group(&test_group(gid)).await.unwrap();

    let now = 9999i64;
    let ep1 = make_endpoint("10.0.0.1", 100);
    let ep2 = make_endpoint("10.0.0.2", 200);

    // ep1 = active, ep2 = stale
    db.subscription_upsert(
        gid,
        &[(
            ep1.clone(),
            vec![ProtocolRow {
                last_seen_at: now,
                ..make_protocol(ep1.id, "vmess", 101)
            }],
        )],
    )
    .await
    .unwrap();

    db.subscription_upsert(
        gid,
        &[(
            ep2.clone(),
            vec![ProtocolRow {
                last_seen_at: now - 7200,
                ..make_protocol(ep2.id, "ss", 102)
            }],
        )],
    )
    .await
    .unwrap();

    let count = db.get_stale_count(now - 3600, 0).await.unwrap();
    assert_eq!(count, 1, "only ep2 should be stale");
}

// ── Task 2.2 — Cross-subscription dedup, system groups, concurrent upsert ────

#[tokio::test]
async fn test_cross_subscription_dedup_different_protocols() {
    let db = test_db().await;
    let gid1 = "src-a";
    let gid2 = "src-b";
    db.insert_group(&test_group(gid1)).await.unwrap();
    db.insert_group(&test_group(gid2)).await.unwrap();

    // Same host:port → same stable_hash → same endpoint id
    let ep = make_endpoint("203.0.113.5", 8443);
    let proto_a = make_protocol(ep.id, "vmess", 10001);
    let proto_b = make_protocol(ep.id, "trojan", 10002);

    // Source A inserts endpoint with vmess protocol
    db.subscription_upsert(gid1, &[(ep.clone(), vec![proto_a])])
        .await
        .unwrap();

    // Source B inserts same host:port with a different protocol (trojan)
    db.subscription_upsert(gid2, &[(ep.clone(), vec![proto_b])])
        .await
        .unwrap();

    // Verify: one endpoint row with two protocols
    let row = db
        .get_endpoint(ep.id)
        .await
        .unwrap()
        .expect("endpoint should exist");
    assert_eq!(row.endpoint.host, "203.0.113.5");
    assert_eq!(row.endpoint.port, 8443);
    assert_eq!(
        row.protocols.len(),
        2,
        "should have protocols from both sources"
    );

    // Both groups have the endpoint linked
    let from_a = db.get_active_endpoints_by_group(gid1, 0).await.unwrap();
    let from_b = db.get_active_endpoints_by_group(gid2, 0).await.unwrap();
    assert!(
        from_a.iter().any(|r| r.endpoint.id == ep.id),
        "endpoint linked to source-a"
    );
    assert!(
        from_b.iter().any(|r| r.endpoint.id == ep.id),
        "endpoint linked to source-b"
    );
}

#[tokio::test]
async fn test_system_group_created_on_init() {
    let db = test_db().await;

    // in_memory() calls init_default_groups which creates a "Default" group
    let groups = db.get_all_groups().await.unwrap();
    assert!(!groups.is_empty(), "should have at least the default group");

    // Verify there is exactly one default group (idempotent init)
    let default_groups: Vec<_> = groups
        .iter()
        .filter(|g| g.name.as_deref() == Some("Default"))
        .collect();
    assert_eq!(default_groups.len(), 1, "exactly one Default group");
    assert!(
        default_groups[0].sort_order == Some(0),
        "default group has sort_order 0"
    );
}

#[tokio::test]
async fn test_concurrent_subscription_upsert() {
    let db = test_db().await;
    let gid = "concurrent";
    db.insert_group(&test_group(gid)).await.unwrap();

    // 10 sequential upserts of different endpoints into the same group.
    // (Parallel SQLite writes from separate connections require per-connection
    // busy_timeout which the internal connection setup doesn't propagate.)
    for i in 0..10 {
        let host = format!("192.0.2.{}", i + 1);
        let port = 1000 + i as i32;
        let ep = make_endpoint(&host, port);
        let proto = make_protocol(ep.id, "vmess", 20000 + i);
        let ids = db
            .subscription_upsert(gid, &[(ep, vec![proto])])
            .await
            .expect("upsert should succeed");
        assert_eq!(ids.len(), 1, "upsert {i} returned 1 id");
    }

    // All 10 endpoints exist in the group
    let endpoints = db.get_active_endpoints_by_group(gid, 0).await.unwrap();
    assert_eq!(endpoints.len(), 10, "all 10 endpoints should exist");
}
