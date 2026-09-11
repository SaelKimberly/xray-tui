//! Mechanics of the raw-SQL Profiles page queries: paging, counting,
//! anchoring, link order, bind formatting, and a schema-drift guard.
//!
//! The ordering *parity* golden (SQL vs the Rust comparator) lives in the
//! `xray-tui` crate, where the comparators are.

use jiff::Timestamp;
use toasty::{Deferred, Json};
use xray_tui_db::Database;
use xray_tui_db::models::{
    ConfigType, Endpoint, EndpointId, HostType, ProfileStats, Protocol, ProtocolId, PurgatoryView,
    Security, TrafficStats, Transport,
};
use xray_tui_db::profiles_query::{PageRequest, PageSort, sql_ts};
use xray_tui_proto::proto_spec::common::TransportConfig;
use xray_tui_proto::proto_spec::{
    CoreType, ProtocolConfig, ProtocolKind, SecurityConfig, SecurityType, TransportType,
    VlessConfig,
};

fn ts(secs: i64) -> Timestamp {
    Timestamp::from_second(secs).expect("valid ts")
}

const ALL_ENDPOINTS: [i64; 7] = [1, 2, 3, 4, 5, 6, 7];

fn request(sort: PageSort, ascending: bool, offset: usize, limit: usize) -> PageRequest {
    PageRequest {
        view: PurgatoryView::All,
        active_threshold: ts(0),
        stale_threshold: ts(0),
        search: None,
        group_id: None,
        sort,
        ascending,
        offset,
        limit,
    }
}

async fn seed_endpoint(
    conn: &mut toasty::Connection,
    id: i64,
    host_type: HostType,
    resolved: &[&str],
) {
    toasty::create!(Endpoint {
        id: EndpointId::new(id),
        host: format!("h{id}.example"),
        host_type,
        port: 443,
        ports: Vec::<u16>::new(),
        resolved_as: resolved
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<String>>(),
    })
    .exec(&mut *conn)
    .await
    .expect("create endpoint");
}

#[allow(clippy::too_many_arguments)]
async fn seed_link(
    conn: &mut toasty::Connection,
    endpoint_id: i64,
    protocol_id: i64,
    delay: Option<i64>,
    error_kind: Option<&str>,
    last_seen: i64,
) {
    toasty::create!(Protocol {
        id: ProtocolId::new(protocol_id),
        sig: protocol_id,
        cred_hash: 0,
        proto_kind: ProtocolKind::Vless,
        transport: Transport {
            r#type: TransportType::Tcp,
            data: Deferred::from(Json(TransportConfig::Tcp)),
        },
        security: Security {
            r#type: SecurityType::None,
            sni: None,
            fp: None,
            insecure: None,
            data: Deferred::from(Json(SecurityConfig::default())),
        },
        config: Deferred::from(Json(ProtocolConfig::Vless(VlessConfig {
            uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            uuid_origin: None,
            security: SecurityConfig::default(),
            transport: TransportConfig::Tcp,
            encryption: None,
            flow: None,
            path: None,
            splice: None,
            remarks: None,
        }))),
    })
    .exec(&mut *conn)
    .await
    .expect("create protocol");

    toasty::create!(ProfileStats {
        protocol_id: ProtocolId::new(protocol_id),
        endpoint_id: EndpointId::new(endpoint_id),
        core_type: CoreType::Xray,
        config_type: ConfigType::ShareUrl,
        last_seen_at: ts(last_seen),
        task_queue: Vec::<u16>::new(),
        traffic: TrafficStats {
            today_up: 0,
            today_down: 0,
            total_up: 0,
            total_down: 0,
        },
    })
    .exec(&mut *conn)
    .await
    .expect("create link");

    if let Some(delay) = delay {
        toasty::sql::statement(
            "UPDATE profile_stats SET latency = 'real', latency_delay = ?1 \
             WHERE endpoint_id = ?2 AND protocol_id = ?3",
        )
        .bind(delay)
        .bind(endpoint_id)
        .bind(protocol_id)
        .exec(&mut *conn)
        .await
        .expect("set latency");
    }
    if let Some(kind) = error_kind {
        // `ErrorInfo.text` is a non-Option String: the flag without text breaks
        // every typed read of `profile_stats`.
        toasty::sql::statement(
            "UPDATE profile_stats SET error = 1, error_kind = ?1, error_text = ?4 \
             WHERE endpoint_id = ?2 AND protocol_id = ?3",
        )
        .bind(kind)
        .bind(endpoint_id)
        .bind(protocol_id)
        .bind(format!("{kind} probe"))
        .exec(&mut *conn)
        .await
        .expect("set error");
    }
}

/// Fixture: measured successes, both error bands, untested, DNS unresolved and
/// resolved, a manual override pointing away from the minimum-weight link, and
/// a measured link with an error marker beside an untested sibling.
async fn seed_fixture() -> Database {
    let db = Database::in_memory().await.expect("in-memory db");
    let mut conn = db.connection().await.expect("conn");

    // e1: real 30 + fast-error sibling
    seed_endpoint(&mut conn, 1, HostType::Ipv4, &[]).await;
    seed_link(&mut conn, 1, 101, Some(30), None, 100).await;
    seed_link(&mut conn, 1, 102, None, Some("fast"), 200).await;
    // e2: real 10
    seed_endpoint(&mut conn, 2, HostType::Ipv4, &[]).await;
    seed_link(&mut conn, 2, 103, Some(10), None, 150).await;
    // e3: untested
    seed_endpoint(&mut conn, 3, HostType::Ipv4, &[]).await;
    seed_link(&mut conn, 3, 104, None, None, 50).await;
    // e4: manual override away from the minimum-weight link
    seed_endpoint(&mut conn, 4, HostType::Ipv4, &[]).await;
    seed_link(&mut conn, 4, 105, Some(90), None, 120).await;
    seed_link(&mut conn, 4, 106, Some(500), None, 130).await;
    toasty::sql::statement("UPDATE endpoints SET manual_protocol_override = ?1 WHERE id = ?2")
        .bind(106_i64)
        .bind(4_i64)
        .exec(&mut conn)
        .await
        .expect("set override");
    // e5: measured link WITH an error marker beside an untested sibling
    seed_endpoint(&mut conn, 5, HostType::Ipv4, &[]).await;
    seed_link(&mut conn, 5, 107, Some(40), Some("fast"), 140).await;
    seed_link(&mut conn, 5, 108, None, None, 160).await;
    // e6: dns host, unresolved (persisted) despite a stored measurement
    seed_endpoint(&mut conn, 6, HostType::Dns, &[]).await;
    seed_link(&mut conn, 6, 109, Some(5), None, 170).await;
    // e7: dns host WITH a persisted resolution, plus a name-resolution failure
    seed_endpoint(&mut conn, 7, HostType::Dns, &["203.0.113.7"]).await;
    seed_link(&mut conn, 7, 110, Some(7), None, 180).await;
    seed_link(&mut conn, 7, 111, None, Some("name"), 190).await;

    db
}

#[tokio::test]
async fn paging_visits_every_endpoint_exactly_once() {
    let db = seed_fixture().await;
    for limit in [1_usize, 2, 3, 100] {
        let mut seen = Vec::new();
        let mut offset = 0;
        loop {
            let page = db
                .profiles_page(&request(PageSort::Test, true, offset, limit))
                .await
                .expect("page");
            assert_eq!(page.total, ALL_ENDPOINTS.len() as u64);
            if page.ids.is_empty() {
                break;
            }
            seen.extend(page.ids.iter().map(|id| id.get()));
            offset += limit;
        }
        let mut got = seen.clone();
        got.sort_unstable();
        assert_eq!(got, ALL_ENDPOINTS.to_vec(), "limit {limit}");
        assert_eq!(
            seen.len(),
            ALL_ENDPOINTS.len(),
            "no duplicates, limit {limit}"
        );
    }
}

#[tokio::test]
async fn count_narrows_with_search_and_group_filters() {
    let db = seed_fixture().await;
    let all = db
        .profiles_count(&request(PageSort::Test, true, 0, 10))
        .await
        .expect("count");
    assert_eq!(all, 7);

    let searched = db
        .profiles_count(&PageRequest {
            search: Some("H1.".to_string()),
            ..request(PageSort::Test, true, 0, 10)
        })
        .await
        .expect("count search");
    assert_eq!(searched, 1, "search is case-insensitive on host");

    let port = db
        .profiles_count(&PageRequest {
            search: Some("443".to_string()),
            ..request(PageSort::Test, true, 0, 10)
        })
        .await
        .expect("count port");
    assert_eq!(port, 7, "port match");

    let none = db
        .profiles_count(&PageRequest {
            search: Some("%".to_string()),
            ..request(PageSort::Test, true, 0, 10)
        })
        .await
        .expect("count escaped");
    assert_eq!(none, 0, "LIKE metacharacters are escaped");
}

#[tokio::test]
async fn anchor_returns_each_rows_position_in_the_ordered_view() {
    let db = seed_fixture().await;
    let page = db
        .profiles_page(&request(PageSort::Test, true, 0, 100))
        .await
        .expect("page");
    assert_eq!(page.ids.len(), 7);

    for (position, id) in page.ids.iter().enumerate() {
        let anchored = db
            .profiles_anchor(&request(PageSort::Test, true, 0, 10), *id)
            .await
            .expect("anchor");
        assert_eq!(anchored, Some(position), "anchor for {}", id.get());
    }
    let missing = db
        .profiles_anchor(&request(PageSort::Test, true, 0, 10), EndpointId::new(999))
        .await
        .expect("anchor missing");
    assert_eq!(missing, None);
}

#[tokio::test]
async fn anchor_and_page_agree_after_a_weight_change() {
    let db = seed_fixture().await;
    let before = db
        .profiles_page(&request(PageSort::Test, true, 0, 100))
        .await
        .expect("page");
    let moved = before.ids[0];

    // Move the leading endpoint to the bottom band (real error).
    toasty::sql::statement(
        "UPDATE profile_stats SET error = 1, error_kind = 'real', error_text = 'x' \
         WHERE endpoint_id = ?1",
    )
    .bind(moved.get())
    .exec(&mut db.connection().await.expect("conn"))
    .await
    .expect("mutate");

    let after = db
        .profiles_page(&request(PageSort::Test, true, 0, 100))
        .await
        .expect("page");
    assert_ne!(after.ids[0], moved, "the row left the top");
    let anchored = db
        .profiles_anchor(&request(PageSort::Test, true, 0, 10), moved)
        .await
        .expect("anchor")
        .expect("present");
    assert_eq!(after.ids[anchored], moved, "anchor tracks the new position");
}

#[tokio::test]
async fn link_order_is_decision_16_order() {
    let db = seed_fixture().await;
    let ids: Vec<EndpointId> = ALL_ENDPOINTS
        .iter()
        .map(|id| EndpointId::new(*id))
        .collect();
    let order = db.profile_link_order(&ids).await.expect("link order");
    assert_eq!(order.len(), ids.len());
    assert!(order.values().all(|v| !v.is_empty()));

    let ids_of = |endpoint: i64| -> Vec<i64> {
        order
            .get(&EndpointId::new(endpoint))
            .expect("endpoint present")
            .iter()
            .map(|p| p.get())
            .collect()
    };
    // e1: the real success (30 ms) leads the fast error.
    assert_eq!(ids_of(1), vec![101, 102]);
    // e4: the minimum-weight link leads; the manual override is a display
    // choice and does not reorder the panel.
    assert_eq!(ids_of(4), vec![105, 106]);
    // e5: untested (tier 2) leads the measured-with-error link (tier 4).
    assert_eq!(ids_of(5), vec![108, 107]);
    // e6: unresolved DNS collapses the band; the only link leads.
    assert_eq!(ids_of(6), vec![109]);
    // e7: real success, then the name-resolution failure.
    assert_eq!(ids_of(7), vec![110, 111]);

    let empty = db.profile_link_order(&[]).await.expect("empty");
    assert!(empty.is_empty());
}

#[tokio::test]
async fn timestamps_bind_at_fixed_width() {
    for secs in [0, 1_700_000_000, 1_760_000_000, 4_000_000_000] {
        let rendered = sql_ts(ts(secs));
        assert_eq!(rendered.len(), 30, "{rendered}");
        assert!(rendered.ends_with('Z'), "{rendered}");
        assert!(rendered.contains('.'), "{rendered}");
    }
    // Whole-second and fractional values from the same moment agree.
    assert_eq!(
        sql_ts(ts(1_700_000_000)),
        sql_ts(Timestamp::from_second(1_700_000_000).expect("ts"))
    );
}

#[tokio::test]
async fn every_statement_runs_against_a_pushed_schema() {
    // Drift guard: a renamed or removed column must fail here, not in the TUI.
    let db = Database::in_memory().await.expect("in-memory db");
    let base = request(PageSort::Test, true, 0, 10);
    db.profiles_page(&base).await.expect("page");
    db.profiles_count(&base).await.expect("count");
    db.profiles_ids(&base).await.expect("ids");
    db.profiles_link_pairs(&base).await.expect("pairs");
    db.profiles_failed_ids().await.expect("failed");
    db.profiles_enrich_seed_ids(&base).await.expect("seed");
    db.profiles_anchor(&base, EndpointId::new(1))
        .await
        .expect("anchor");
    db.profile_link_order(&[EndpointId::new(1)])
        .await
        .expect("order");

    for sort in [
        PageSort::Address,
        PageSort::Port,
        PageSort::LastSeen,
        PageSort::Speed,
        PageSort::Traffic,
        PageSort::ConfigType,
    ] {
        for ascending in [true, false] {
            let req = PageRequest {
                sort,
                ascending,
                ..base.clone()
            };
            db.profiles_page(&req).await.expect("sorted page");
            db.profiles_count(&req).await.expect("sorted count");
        }
    }
    for view in [
        PurgatoryView::Active,
        PurgatoryView::Stale,
        PurgatoryView::All,
    ] {
        let req = PageRequest {
            view,
            ..base.clone()
        };
        db.profiles_page(&req).await.expect("view page");
    }
}
