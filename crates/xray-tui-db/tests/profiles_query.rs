//! Mechanics of the raw-SQL Profiles page queries: paging, counting,
//! anchoring, link order, bind formatting, and a schema-drift guard.
//!
//! The ordering *parity* golden (SQL vs the Rust comparator) lives in the
//! `xray-tui` crate, where the comparators are.

use toasty::{Deferred, Json};
use xray_tui_db::Database;
use xray_tui_db::models::{
    ConfigType, Endpoint, EndpointId, EndpointIp, ErrorInfo, HostType, Latency, ProfileStats,
    Protocol, ProtocolId, PurgatoryView, Security, TrafficStats, Transport,
};
use xray_tui_db::profiles_query::{PageRequest, PageSort};
use xray_tui_db::{LinkGroups, LinkPatch};
use xray_tui_proto::proto_spec::common::TransportConfig;
use xray_tui_proto::proto_spec::{
    CoreType, ProtocolConfig, ProtocolKind, SecurityConfig, SecurityType, TransportType,
    VlessConfig,
};

/// Epoch seconds — the storage unit of every timestamp column.
const fn ts(secs: i64) -> i64 {
    secs
}

const ALL_ENDPOINTS: [i64; 7] = [1, 2, 3, 4, 5, 6, 7];

const ALL_SORTS: [PageSort; 8] = [
    PageSort::Test,
    PageSort::Address,
    PageSort::Port,
    PageSort::LastSeen,
    PageSort::Speed,
    PageSort::Traffic,
    PageSort::ConfigType,
    PageSort::Ip,
];

const fn request(sort: PageSort, ascending: bool, offset: usize, limit: usize) -> PageRequest {
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
        created_at: 0,
        id: EndpointId::new(id),
        host: format!("h{id}.example"),
        host_type,
        port: 443,
        ports: Vec::<u16>::new(),
    })
    .exec(&mut *conn)
    .await
    .expect("create endpoint");
    // The address set is `endpoint_ip`'s: a DNS host with no rows is the
    // unresolved band's input, and a resolved one carries its addresses there.
    for ip in resolved {
        let key = xray_tui_db::endpoint_ip::key_of_str(ip).expect("a real address");
        toasty::create!(EndpointIp {
            endpoint_id: EndpointId::new(id),
            ip_key: key,
        })
        .exec(&mut *conn)
        .await
        .expect("create address");
    }
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
        created_at: 0,
        id: ProtocolId::new(protocol_id),
        sig: protocol_id,
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
        created_at: 0,
        updated_at: 0,
        protocol_id: ProtocolId::new(protocol_id),
        endpoint_id: EndpointId::new(endpoint_id),
        core_type: CoreType::Xray,
        config_type: ConfigType::ShareUrl,
        last_seen_at: ts(last_seen),
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

/// A `ProfileStats` value for the value-taking write paths (`upsert_link`,
/// `apply_link_patches`) — the seeded equivalent of a ping result.
fn link_value(
    endpoint_id: i64,
    protocol_id: i64,
    delay: Option<i32>,
    error_kind: Option<&str>,
    last_seen: i64,
) -> ProfileStats {
    ProfileStats {
        protocol_id: ProtocolId::new(protocol_id),
        endpoint_id: EndpointId::new(endpoint_id),
        core_type: CoreType::Xray,
        config_type: ConfigType::ShareUrl,
        last_used_at: None,
        last_seen_at: ts(last_seen),
        latency: delay.map(|delay| Latency::Real { delay, ip: None }),
        speed_bps: None,
        error: error_kind.map(|kind| ErrorInfo {
            kind: match kind {
                "real" => xray_tui_db::models::ProfileErr::Real,
                "fast" => xray_tui_db::models::ProfileErr::Fast,
                _ => xray_tui_db::models::ProfileErr::Name,
            },
            text: format!("{kind} probe"),
        }),
        purge_reason: None,
        traffic: TrafficStats {
            today_up: 0,
            today_down: 0,
            total_up: 0,
            total_down: 0,
        },
        created_at: ts(last_seen),
        updated_at: ts(last_seen),
        version: 1,
        protocol: Deferred::default(),
        endpoint: Deferred::default(),
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
    drop(conn);

    // These fixtures write links with raw statements, bypassing the write
    // paths that keep `endpoint_rank` current (see the freshness test below).
    // Establish the same invariant they do, explicitly.
    let ids: Vec<EndpointId> = (1..=7).map(EndpointId::new).collect();
    db.refresh_endpoint_ranks(&ids).await.expect("seed ranks");
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

/// The walk page is the page query minus its per-page count: same ids in the
/// same order at every offset, and the total is returned only when the walk asks
/// for it — once, not once per page (the `O(feed²/200)` the batch plan loader
/// used to pay).
#[tokio::test]
async fn walk_pages_match_the_page_ids_and_count_once() {
    let db = seed_fixture().await;
    for sort in ALL_SORTS {
        for ascending in [true, false] {
            for limit in [1_usize, 3, 100] {
                let mut walked = Vec::new();
                let mut totals = Vec::new();
                let mut offset = 0;
                loop {
                    let (ids, total) = db
                        .profiles_walk_page(&request(sort, ascending, offset, limit), offset == 0)
                        .await
                        .expect("walk page");
                    totals.push(total);
                    if ids.is_empty() {
                        break;
                    }
                    walked.extend(ids.iter().map(|id| id.get()));
                    offset += limit;
                }
                let page = db
                    .profiles_page(&request(sort, ascending, 0, 100))
                    .await
                    .expect("page");
                assert_eq!(
                    walked,
                    page.ids.iter().map(|id| id.get()).collect::<Vec<_>>(),
                    "sort {sort:?} asc {ascending} limit {limit}"
                );
                assert_eq!(totals.first().copied().flatten(), Some(page.total));
                assert!(
                    totals[1..].iter().all(Option::is_none),
                    "only the first walk page pays the count"
                );
            }
        }
    }
}

#[tokio::test]
async fn count_narrows_with_search_and_group_filters() {
    let db = seed_fixture().await;
    let total = |req: PageRequest| {
        let db = &db;
        async move { db.profiles_page(&req).await.expect("count").total }
    };
    assert_eq!(total(request(PageSort::Test, true, 0, 10)).await, 7);

    let searched = total(PageRequest {
        search: Some("H1.".to_string()),
        ..request(PageSort::Test, true, 0, 10)
    })
    .await;
    assert_eq!(searched, 1, "search is case-insensitive on host");

    let port = total(PageRequest {
        search: Some("443".to_string()),
        ..request(PageSort::Test, true, 0, 10)
    })
    .await;
    assert_eq!(port, 7, "port match");

    let none = total(PageRequest {
        search: Some("%".to_string()),
        ..request(PageSort::Test, true, 0, 10)
    })
    .await;
    assert_eq!(none, 0, "LIKE metacharacters are escaped");
}

#[tokio::test]
async fn anchor_returns_each_rows_position_for_every_sort_and_direction() {
    let db = seed_fixture().await;
    for sort in ALL_SORTS {
        for ascending in [true, false] {
            let page = db
                .profiles_page(&request(sort, ascending, 0, 100))
                .await
                .expect("page");
            assert_eq!(page.ids.len(), 7, "{sort:?} {ascending}");
            for (position, id) in page.ids.iter().enumerate() {
                let anchored = db
                    .profiles_anchor(&request(sort, ascending, 0, 10), *id)
                    .await
                    .expect("anchor");
                assert_eq!(
                    anchored,
                    Some(position),
                    "anchor {sort:?} {ascending} {id:?}"
                );
            }
        }
    }
    let missing = db
        .profiles_anchor(&request(PageSort::Test, true, 0, 10), EndpointId::new(999))
        .await
        .expect("anchor missing");
    assert_eq!(missing, None);
}

#[tokio::test]
async fn descending_order_is_the_reverse_of_ascending_for_every_sort() {
    // The UI reverses the whole comparator (`cmp.reverse()`), so a descending
    // page must be the exact reverse of the ascending one, tiebreaks included.
    let db = seed_fixture().await;
    for sort in ALL_SORTS {
        let asc = db
            .profiles_page(&request(sort, true, 0, 100))
            .await
            .expect("asc");
        let desc = db
            .profiles_page(&request(sort, false, 0, 100))
            .await
            .expect("desc");
        let mut expected = asc.ids.clone();
        expected.reverse();
        assert_eq!(desc.ids, expected, "{sort:?}");
    }
}

#[tokio::test]
async fn link_writes_keep_the_stored_keys_current() {
    let db = Database::in_memory().await.expect("db");
    let mut conn = db.connection().await.expect("conn");
    seed_endpoint(&mut conn, 1, HostType::Ipv4, &[]).await;
    seed_endpoint(&mut conn, 2, HostType::Ipv4, &[]).await;
    drop(conn);

    let page = || async {
        db.profiles_page(&request(PageSort::Test, true, 0, 100))
            .await
            .expect("page")
            .ids
    };

    // `upsert_link` (the import / single-row path).
    db.upsert_link(&link_value(1, 101, None, None, 100))
        .await
        .expect("upsert a");
    db.upsert_link(&link_value(2, 102, None, None, 100))
        .await
        .expect("upsert b");
    assert_eq!(page().await, vec![EndpointId::new(1), EndpointId::new(2)]);

    // `apply_link_patches` (the batch-result path): a real result leads.
    db.apply_link_patches(&[LinkPatch {
        link: link_value(2, 102, Some(20), None, 100),
        groups: LinkGroups::RESULT,
    }])
    .await
    .expect("patch");
    assert_eq!(page().await, vec![EndpointId::new(2), EndpointId::new(1)]);

    // And the failure band follows the same route.
    db.apply_link_patches(&[LinkPatch {
        link: link_value(2, 102, None, Some("real"), 100),
        groups: LinkGroups::RESULT,
    }])
    .await
    .expect("patch");
    assert_eq!(page().await, vec![EndpointId::new(1), EndpointId::new(2)]);
}

/// The address set is part of the ordering law: a DNS host that resolves moves
/// out of the unresolved band, and the page must show it there immediately.
#[tokio::test]
async fn resolving_a_dns_host_moves_its_stored_key() {
    let db = seed_fixture().await;
    let unresolved = EndpointId::new(6); // dns host, no address, real 5 ms
    let before = db
        .profiles_page(&request(PageSort::Test, true, 0, 100))
        .await
        .expect("page")
        .ids;
    assert_eq!(before.last(), Some(&unresolved), "unresolved sinks");

    db.update_endpoint_resolution(
        unresolved,
        vec!["203.0.113.6".parse().expect("addr")],
        ts(200),
    )
    .await
    .expect("resolve");

    let after = db
        .profiles_page(&request(PageSort::Test, true, 0, 100))
        .await
        .expect("page")
        .ids;
    assert_eq!(after.first(), Some(&unresolved), "a 5 ms real link leads");
}

/// A TASK-only patch is key-neutral for an EXISTING link, but not when it
/// inserts a missing one: the insert writes the whole snapshot, so the
/// endpoint's key appears (or moves) and the page must show it.
#[tokio::test]
async fn a_result_patch_that_inserts_a_link_still_moves_the_key() {
    let db = Database::in_memory().await.expect("db");
    let mut conn = db.connection().await.expect("conn");
    seed_endpoint(&mut conn, 1, HostType::Ipv4, &[]).await;
    drop(conn);

    let page = db
        .profiles_page(&request(PageSort::Test, true, 0, 100))
        .await
        .expect("page")
        .ids;
    assert!(page.is_empty(), "no links, no key, not listed");

    // For a link that has never been persisted, the whole snapshot goes in
    // whatever groups the patch names — and the endpoint's keys follow.
    let link = link_value(1, 101, Some(25), None, 100);
    db.apply_link_patches(&[LinkPatch {
        link,
        groups: LinkGroups::RESULT,
    }])
    .await
    .expect("patch");

    let page = db
        .profiles_page(&request(PageSort::Test, true, 0, 100))
        .await
        .expect("page")
        .ids;
    assert_eq!(page, vec![EndpointId::new(1)], "the inserted link keys it");
}

/// Deleting an endpoint's links removes its key: the page drives from the rank
/// table, so a lingering key would list a linkless row.
#[tokio::test]
async fn purging_endpoints_drops_their_keys() {
    let db = seed_fixture().await;
    let purged = db.purge_expired(ts(100_000)).await.expect("purge");
    assert!(purged > 0, "the fixture's rows are older than the cutoff");
    let page = db
        .profiles_page(&request(PageSort::Test, true, 0, 100))
        .await
        .expect("page");
    assert!(page.ids.is_empty(), "got {:?}", page.ids);
    assert_eq!(page.total, 0);
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
    // The raw statement bypassed the write paths, so refresh like they do.
    db.refresh_endpoint_ranks(&[moved]).await.expect("ranks");

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

/// The tiers the panel renders, pinned per fixture endpoint. The order is the
/// Rust law's (there is no SQL expression left to disagree with it).
#[tokio::test]
async fn link_order_is_decision_16_order() {
    let db = seed_fixture().await;
    let ids: Vec<EndpointId> = ALL_ENDPOINTS
        .iter()
        .map(|id| EndpointId::new(*id))
        .collect();
    let rows = db.load_page_rows(&ids).await.expect("rows");
    assert_eq!(rows.len(), ids.len());
    assert!(rows.iter().all(|r| !r.links.is_empty()));

    let ids_of = |endpoint: i64| -> Vec<i64> {
        rows.iter()
            .find(|r| r.endpoint.id == EndpointId::new(endpoint))
            .expect("endpoint present")
            .links
            .iter()
            .map(|l| l.protocol_id.get())
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
}

#[tokio::test]
async fn every_statement_runs_against_a_pushed_schema() {
    // Drift guard: a renamed or removed column must fail here, not in the TUI.
    let db = Database::in_memory().await.expect("in-memory db");
    let base = request(PageSort::Test, true, 0, 10);
    db.profiles_page(&base).await.expect("page");
    db.profiles_page(&base).await.expect("count");
    db.profiles_anchor(&base, EndpointId::new(1))
        .await
        .expect("anchor");
    db.load_page_projection(&[EndpointId::new(1)])
        .await
        .expect("projection");

    for sort in ALL_SORTS {
        for ascending in [true, false] {
            let req = PageRequest {
                sort,
                ascending,
                ..base.clone()
            };
            db.profiles_page(&req).await.expect("sorted page");
            db.profiles_page(&req).await.expect("sorted count");
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

/// The page's rows come back in the page's order, with each row's links in the
/// decision-16 order the panel renders. That order comes from ONE place
/// ([`RankLink::key`]) — the SQL expression that used to re-derive it is gone,
/// because the two disagreed for a DNS-unresolved endpoint (its measured link
/// sorted above a failure; the law sinks every link of such an endpoint to
/// tier 5).
#[tokio::test]
async fn load_page_rows_preserves_page_and_link_order() {
    let db = seed_fixture().await;
    let req = request(PageSort::Test, true, 0, 2);
    let page = db.profiles_page(&req).await.expect("page");
    assert_eq!(page.ids.len(), 2);

    let rows = db.load_page_rows(&page.ids).await.expect("rows");
    let row_ids: Vec<i64> = rows.iter().map(|r| r.endpoint.id.get()).collect();
    let page_ids: Vec<i64> = page.ids.iter().map(|id| id.get()).collect();
    assert_eq!(row_ids, page_ids, "page order, not id order");

    for row in &rows {
        let unresolved = dns_unresolved(row);
        let keys: Vec<_> = row
            .links
            .iter()
            .map(|l| xray_tui_db::endpoint_rank::RankLink::from(l).key(unresolved))
            .collect();
        assert!(
            keys.windows(2).all(|w| w[0] <= w[1]),
            "endpoint {} links are not in decision-16 order: {keys:?}",
            row.endpoint.id.get()
        );
    }

    // Empty page: no query, no rows.
    assert!(db.load_page_rows(&[]).await.expect("empty").is_empty());
}

// ── Ordering parity: the SQL vs the Rust oracle ─────────────────────────
//
// The oracle is the production comparator (`EndpointRow::best_test_priority_key`
// and the display-link accessors), i.e. the decision-16 law and the
// display-preference rule as the rest of the codebase defines them. A mismatch
// means the SQL is wrong — the oracle is never adjusted to match it.

fn dns_unresolved(row: &xray_tui_db::models::EndpointRow) -> bool {
    row.endpoint.host_type == HostType::Dns && row.resolved_ips.is_empty()
}

/// The DISPLAY link as the spec defines it: the manual override when it names
/// an existing link, else the best measured link (real before fast, lowest
/// delay, then protocol id), else none.
///
/// Not `EndpointRow::active_link()`: that falls back to `links[selected_protocol]`
/// — the untested link when nothing is measured — while the ordering rule is
/// "no measured link, no value" (the SQL's `COALESCE(..., sentinel)`).
fn display_link(
    row: &xray_tui_db::models::EndpointRow,
) -> Option<&xray_tui_db::models::ProfileStats> {
    if let Some(pid) = row.endpoint.manual_protocol_override
        && let Some(link) = row.links.iter().find(|l| l.protocol_id == pid)
    {
        return Some(link);
    }
    row.links
        .iter()
        .filter(|l| l.latency.is_some())
        .min_by_key(|l| {
            let (rank, delay) = match l.latency {
                Some(xray_tui_db::models::Latency::Real { delay, .. }) => (0i32, delay),
                Some(xray_tui_db::models::Latency::Fast { delay }) => (1, delay),
                None => (2, i32::MAX),
            };
            (rank, delay, l.protocol_id.get())
        })
}

const fn config_type_rank(link: &xray_tui_db::models::ProfileStats) -> i32 {
    match link.config_type {
        ConfigType::Form => 0,
        ConfigType::ShareUrl => 1,
    }
}

/// Ascending sort key per endpoint, mirroring `order_terms`.
///
/// A tuple, not a packed integer: the Test key is the FULL decision-16 tuple
/// `(tier, latency, -last_seen, protocol_id)` — dropping the recency term makes
/// the oracle disagree with the law on ties, which is how this test first
/// failed.
/// The lowest address key of a row, from its own values; the `x'ff'` sentinel
/// when it has none — the same value the SQL's `COALESCE` substitutes.
fn min_address_key(row: &xray_tui_db::models::EndpointRow) -> Vec<u8> {
    row.resolved_ips
        .iter()
        .map(|ip| xray_tui_db::endpoint_ip::key_of(*ip))
        .min()
        .unwrap_or_else(|| vec![0xff])
}

fn oracle_key(row: &xray_tui_db::models::EndpointRow, sort: PageSort) -> (i64, i64, i64, i64) {
    match sort {
        PageSort::Test => {
            let (tier, latency, neg_seen, pid) = row
                .best_test_priority_key(dns_unresolved(row))
                .unwrap_or((u8::MAX, i32::MAX, i64::MAX, i64::MAX));
            (
                i64::from(tier) + i64::from(dns_unresolved(row)) * 8,
                i64::from(latency),
                neg_seen,
                pid,
            )
        }
        PageSort::Address | PageSort::Ip => (0, 0, 0, 0),
        PageSort::Port => (i64::from(row.endpoint.port), 0, 0, 0),
        PageSort::LastSeen => (
            display_link(row).map_or(i64::MIN, |l| l.last_seen_at),
            0,
            0,
            0,
        ),
        PageSort::Speed => (
            display_link(row).and_then(|l| l.speed_bps).unwrap_or(-1),
            0,
            0,
            0,
        ),
        PageSort::Traffic => (
            display_link(row).map_or(0, |l| l.traffic.total_up + l.traffic.total_down),
            0,
            0,
            0,
        ),
        PageSort::ConfigType => (
            i64::from(display_link(row).map_or(2, config_type_rank)),
            0,
            0,
            0,
        ),
    }
}

#[tokio::test]
async fn page_order_matches_the_rust_oracle_for_every_sort() {
    let db = seed_fixture().await;
    let all = db
        .profiles_page(&request(PageSort::Test, true, 0, 1000))
        .await
        .expect("all ids");
    let rows = db.load_page_rows(&all.ids).await.expect("rows");
    assert_eq!(rows.len(), 7, "fixture rows");

    for sort in ALL_SORTS {
        for ascending in [true, false] {
            // Address compares host text, which the numeric key cannot carry;
            // every other sort uses the oracle tuple.
            let mut expected: Vec<i64> = if sort == PageSort::Address {
                let mut v: Vec<(String, i64)> = rows
                    .iter()
                    .map(|r| (r.endpoint.host.clone(), r.endpoint.id.get()))
                    .collect();
                v.sort_unstable();
                v.into_iter().map(|(_, id)| id).collect()
            } else if sort == PageSort::Ip {
                // The address key, derived from the row's own values through
                // the production codec: `min` over the keys is what the SQL's
                // `min(ip_key)` reads, an endpoint with no address takes the
                // `x'ff'` sentinel (after every real key ascending), and the
                // DESC direction is the plain reversal.
                let mut v: Vec<(Vec<u8>, i64)> = rows
                    .iter()
                    .map(|r| (min_address_key(r), r.endpoint.id.get()))
                    .collect();
                v.sort_unstable();
                v.into_iter().map(|(_, id)| id).collect()
            } else {
                let mut v: Vec<((i64, i64, i64, i64), i64)> = rows
                    .iter()
                    .map(|r| (oracle_key(r, sort), r.endpoint.id.get()))
                    .collect();
                v.sort_unstable();
                v.into_iter().map(|(_, id)| id).collect()
            };
            if !ascending {
                expected.reverse();
            }

            let page = db
                .profiles_page(&request(sort, ascending, 0, 1000))
                .await
                .expect("page");
            let got: Vec<i64> = page.ids.iter().map(|id| id.get()).collect();
            assert_eq!(got, expected, "{sort:?} ascending={ascending}");
        }
    }
}

// ── The page projection (H1) ────────────────────────────────────────────

/// Seed rows that fill every projected column: `ports`/`last_source`/
/// `resolved_at` on the endpoint, both latency variants (one with an exit IP),
/// speed, traffic, both config types, a task slot, an error of each kind, a
/// dangling protocol reference, and protocols whose transport/security columns
/// differ. Raw statements, so the fixture reaches columns the typed writers
/// never set (`task_id`, `version`, a dangling `protocol_id`).
async fn seed_projection_fixture() -> Database {
    let db = Database::in_memory().await.expect("in-memory db");
    let mut conn = db.connection().await.expect("conn");

    for stmt in [
        // e1: IPv4, multi-port spec, subscription source, override, two links.
        "INSERT INTO endpoints (id, host, host_type, port, ports, last_source, \
         manual_protocol_override, resolved_at, created_at) VALUES \
         (1, 'a.example', 'ipv4', 443, '[443,8443]', 'src-hash', 11, NULL, \
          1788220800)",
        // e2: DNS with a persisted resolution and a name failure.
        "INSERT INTO endpoints (id, host, host_type, port, ports, last_source, \
         manual_protocol_override, resolved_at, created_at) VALUES \
         (2, 'b.example', 'dns', 8443, '[]', NULL, NULL, 1788352245, 1788220801)",
        // e3: DNS with no resolution, a link whose protocol row is missing.
        "INSERT INTO endpoints (id, host, host_type, port, ports, last_source, \
         manual_protocol_override, resolved_at, created_at) VALUES \
         (3, 'c.example', 'dns', 443, '[]', NULL, NULL, NULL, \
          1788220802)",
        // e1's address: the Ip sort needs more than one endpoint to have a
        // key, and the fixture's only other DNS host (e3) must stay
        // unresolved for the tier-5 case above.
        "INSERT INTO endpoint_ip (endpoint_id, ip_key) VALUES \
         (1, x'04' || x'c0a80101')",
        // e2's addresses, written in the REVERSE of address order: the stored
        // key is what orders them, so a row order that disagrees with it still
        // reads back sorted (the property the whole `ip_key` design rests on).
        "INSERT INTO endpoint_ip (endpoint_id, ip_key) VALUES \
         (2, x'06' || x'20010db8000000000000000000000007')",
        "INSERT INTO endpoint_ip (endpoint_id, ip_key) VALUES \
         (2, x'04' || x'cb007108')",
        "INSERT INTO endpoint_ip (endpoint_id, ip_key) VALUES \
         (2, x'04' || x'cb007107')",
        // Protocols: ws+tls with every optional pinned, and tcp+reality bare.
        "INSERT INTO protocols (id, sig, proto_kind, transport_type, transport_data, \
         security_type, security_sni, security_fp, security_insecure, security_data, config, \
         created_at) VALUES (11, 111, 'vless', 'ws', 'null', 'tls', 'sni.example', 'chrome', \
         1, 'null', 'null', 1788220803)",
        "INSERT INTO protocols (id, sig, proto_kind, transport_type, transport_data, \
         security_type, security_sni, security_fp, security_insecure, security_data, config, \
         created_at) VALUES (13, 333, 'shadowsocks2022', 'tcp', 'null', 'reality', \
         'steal.example', NULL, NULL, 'null', 'null', 1788220804)",
        // e1/link A: real ping with an exit IP, speed, traffic, a task slot.
        "INSERT INTO profile_stats (protocol_id, endpoint_id, core_type, config_type, last_used_at, \
         last_seen_at, latency, latency_delay, latency_ip, speed_bps, error, \
         error_kind, error_text, traffic_today_up, traffic_today_down, traffic_total_up, \
         traffic_total_down, created_at, updated_at, version) VALUES \
         (11, 1, 'xray', 'share_url', 1789034400, \
          1789038000, 'real', 42, '198.51.100.9', 1234567, \
          NULL, NULL, NULL, 11, 22, 33, 44, 1788220805, \
          1789038000, 3)",
        // e1/link B: fast ping, a fast failure, form config.
        "INSERT INTO profile_stats (protocol_id, endpoint_id, core_type, config_type, last_used_at, \
         last_seen_at, latency, latency_delay, latency_ip, speed_bps, error, \
         error_kind, error_text, traffic_today_up, traffic_today_down, traffic_total_up, \
         traffic_total_down, created_at, updated_at, version) VALUES \
         (13, 1, 'sing_box', 'form', NULL, 1789041600, \
          'fast', 8, NULL, NULL, 1, 'fast', 'fast probe', 0, 0, 0, 0, \
          1788220806, 1789041600, 1)",
        // e2: name-resolution failure, no measurement.
        "INSERT INTO profile_stats (protocol_id, endpoint_id, core_type, config_type, last_used_at, \
         last_seen_at, latency, latency_delay, latency_ip, speed_bps, error, \
         error_kind, error_text, traffic_today_up, traffic_today_down, traffic_total_up, \
         traffic_total_down, created_at, updated_at, version) VALUES \
         (11, 2, 'xray', 'share_url', NULL, 1789045200, NULL, \
          NULL, NULL, NULL, 1, 'name', 'name probe', 0, 0, 0, 0, \
          1788220807, 1789045200, 2)",
        // e3: a measured success AND a real failure on one endpoint, plus a
        // link whose protocol row does not exist (LEFT JOIN -> no entry).
        "INSERT INTO profile_stats (protocol_id, endpoint_id, core_type, config_type, last_used_at, \
         last_seen_at, latency, latency_delay, latency_ip, speed_bps, error, \
         error_kind, error_text, traffic_today_up, traffic_today_down, traffic_total_up, \
         traffic_total_down, created_at, updated_at, version) VALUES \
         (13, 3, 'sing_box', 'share_url', NULL, 1789048800, \
          'real', 5, NULL, NULL, NULL, NULL, NULL, 0, 0, 0, 0, \
          1788220808, 1789048800, 1)",
        "INSERT INTO profile_stats (protocol_id, endpoint_id, core_type, config_type, last_used_at, \
         last_seen_at, latency, latency_delay, latency_ip, speed_bps, error, \
         error_kind, error_text, traffic_today_up, traffic_today_down, traffic_total_up, \
         traffic_total_down, created_at, updated_at, version) VALUES \
         (99, 3, 'xray', 'form', NULL, 1789052400, NULL, \
          NULL, NULL, NULL, 1, 'real', 'real probe', 0, 0, 0, 0, \
          1788220809, 1789052400, 1)",
    ] {
        toasty::sql::statement(stmt)
            .exec(&mut conn)
            .await
            .unwrap_or_else(|e| panic!("seed: {e}\n{stmt}"));
    }
    drop(conn);

    let ids: Vec<EndpointId> = (1..=3).map(EndpointId::new).collect();
    db.refresh_endpoint_ranks(&ids).await.expect("seed ranks");
    db
}

fn assert_same_link(typed: &ProfileStats, projected: &ProfileStats, ctx: &str) {
    assert_eq!(
        typed.protocol_id, projected.protocol_id,
        "{ctx}: protocol_id"
    );
    assert_eq!(
        typed.endpoint_id, projected.endpoint_id,
        "{ctx}: endpoint_id"
    );
    assert_eq!(typed.core_type, projected.core_type, "{ctx}: core_type");
    assert_eq!(
        typed.config_type, projected.config_type,
        "{ctx}: config_type"
    );
    assert_eq!(
        typed.last_used_at, projected.last_used_at,
        "{ctx}: last_used_at"
    );
    assert_eq!(
        typed.last_seen_at, projected.last_seen_at,
        "{ctx}: last_seen_at"
    );
    assert_eq!(typed.latency, projected.latency, "{ctx}: latency");
    assert_eq!(typed.speed_bps, projected.speed_bps, "{ctx}: speed_bps");
    assert_eq!(typed.error, projected.error, "{ctx}: error");
    assert_eq!(typed.traffic, projected.traffic, "{ctx}: traffic");
    assert_eq!(typed.created_at, projected.created_at, "{ctx}: created_at");
    assert_eq!(typed.updated_at, projected.updated_at, "{ctx}: updated_at");
    assert_eq!(typed.version, projected.version, "{ctx}: version");
}

/// The projection is the typed hydration's equal: same endpoints, same links
/// in the same order, same protocol columns — and the only difference is that
/// the three deferred JSON carriers come back unloaded.
#[tokio::test]
async fn page_projection_matches_the_orm_rows() {
    let db = seed_projection_fixture().await;
    let ids: Vec<EndpointId> = (1..=3).map(EndpointId::new).collect();
    let typed = db.load_page_rows(&ids).await.expect("typed rows");
    let projected = db.load_page_projection(&ids).await.expect("projected rows");

    assert_eq!(typed.len(), projected.len(), "one row per id");
    for (typed, projected) in typed.iter().zip(&projected) {
        let ctx = format!("endpoint {}", typed.endpoint.id.get());
        assert_eq!(typed.endpoint.id, projected.endpoint.id, "{ctx}: id");
        assert_eq!(typed.endpoint.host, projected.endpoint.host, "{ctx}: host");
        assert_eq!(
            typed.endpoint.host_type, projected.endpoint.host_type,
            "{ctx}: host_type"
        );
        assert_eq!(typed.endpoint.port, projected.endpoint.port, "{ctx}: port");
        assert_eq!(
            typed.endpoint.ports, projected.endpoint.ports,
            "{ctx}: ports"
        );
        assert_eq!(
            typed.endpoint.last_source, projected.endpoint.last_source,
            "{ctx}: last_source"
        );
        assert_eq!(
            typed.endpoint.manual_protocol_override, projected.endpoint.manual_protocol_override,
            "{ctx}: manual_protocol_override"
        );
        assert_eq!(
            typed.resolved_ips, projected.resolved_ips,
            "{ctx}: resolved_ips"
        );
        assert_eq!(
            typed.endpoint.resolved_at, projected.endpoint.resolved_at,
            "{ctx}: resolved_at"
        );
        assert_eq!(
            typed.endpoint.created_at, projected.endpoint.created_at,
            "{ctx}: created_at"
        );

        assert_eq!(
            typed.links.len(),
            projected.links.len(),
            "{ctx}: link count"
        );
        for i in 0..typed.links.len() {
            assert_same_link(
                &typed.links[i],
                &projected.links[i],
                &format!("{ctx}: link {i}"),
            );
        }
        assert_eq!(
            typed.selected_protocol, projected.selected_protocol,
            "{ctx}: selected_protocol"
        );
        assert_eq!(typed.expanded, projected.expanded, "{ctx}: expanded");

        assert_eq!(
            typed.protocols.len(),
            projected.protocols.len(),
            "{ctx}: protocol count"
        );
        for (id, typed_protocol) in &typed.protocols {
            let projected_protocol = projected
                .protocols
                .get(id)
                .unwrap_or_else(|| panic!("{ctx}: protocol {id:?} missing from the projection"));
            assert_eq!(
                typed_protocol.id, projected_protocol.id,
                "{ctx}: protocol id"
            );
            assert_eq!(
                typed_protocol.sig, projected_protocol.sig,
                "{ctx}: protocol sig"
            );
            assert_eq!(
                typed_protocol.proto_kind, projected_protocol.proto_kind,
                "{ctx}: proto_kind"
            );
            assert_eq!(
                typed_protocol.transport.r#type, projected_protocol.transport.r#type,
                "{ctx}: transport type"
            );
            assert_eq!(
                typed_protocol.security.r#type, projected_protocol.security.r#type,
                "{ctx}: security type"
            );
            assert_eq!(
                typed_protocol.security.sni, projected_protocol.security.sni,
                "{ctx}: security sni"
            );
            assert_eq!(
                typed_protocol.security.fp, projected_protocol.security.fp,
                "{ctx}: security fp"
            );
            assert_eq!(
                typed_protocol.security.insecure, projected_protocol.security.insecure,
                "{ctx}: security insecure"
            );
            assert_eq!(
                typed_protocol.created_at, projected_protocol.created_at,
                "{ctx}: protocol created_at"
            );
            // The page never reads a protocol's JSON: the projection leaves it
            // unloaded, so a consumer that needs it must re-read the row.
            assert!(
                projected_protocol.config.is_unloaded(),
                "{ctx}: the projection must not load `protocols.config`"
            );
            assert!(
                projected_protocol.transport.data.is_unloaded(),
                "{ctx}: the projection must not load `transport_data`"
            );
            assert!(
                projected_protocol.security.data.is_unloaded(),
                "{ctx}: the projection must not load `security_data`"
            );
        }
    }

    // The DNS-unresolved collapse (decision 16, tier 5) reached the page rows:
    // every link of such an endpoint sinks, so the NEWEST one leads even though
    // its sibling carries a live measurement — the retired SQL link order put
    // the measured link first here.
    let dns_row = projected
        .iter()
        .find(|r| r.endpoint.id.get() == 3)
        .expect("endpoint 3");
    assert!(xray_tui_db::endpoint_rank::dns_unresolved(dns_row));
    assert_eq!(
        dns_row
            .links
            .iter()
            .map(|l| l.protocol_id.get())
            .collect::<Vec<_>>(),
        vec![99, 13]
    );
    assert!(
        dns_row.links[0].latency.is_none(),
        "the newer failure leads a tier-5 row"
    );
}

/// The page ids are the only input: an empty page costs no statement, and the
/// returned rows keep the caller's order rather than the id order.
#[tokio::test]
async fn page_projection_keeps_the_requested_order_and_skips_empty_pages() {
    let db = seed_projection_fixture().await;
    assert!(
        db.load_page_projection(&[])
            .await
            .expect("empty page")
            .is_empty()
    );

    let shuffled = vec![EndpointId::new(3), EndpointId::new(1), EndpointId::new(2)];
    let rows = db
        .load_page_projection(&shuffled)
        .await
        .expect("projected rows");
    let got: Vec<i64> = rows.iter().map(|r| r.endpoint.id.get()).collect();
    assert_eq!(got, vec![3, 1, 2]);
}

/// A patch writes ONLY its own column groups: the columns another writer owns
/// (`last_used_at`, `last_seen_at`, `created_at`) keep their persisted values.
/// This is why the flush writes per-group UPDATEs instead of replacing the
/// whole row — `update_last_used` runs on every connect, and a batch's staged
/// snapshot is older than that write.
#[tokio::test]
async fn link_patches_leave_columns_outside_their_groups_alone() {
    let db = seed_projection_fixture().await;
    let link = db
        .read_link_row(ProtocolId::new(11), EndpointId::new(1))
        .await
        .expect("read")
        .expect("link");
    let untouched_used = link.last_used_at;
    let untouched_seen = link.last_seen_at;
    let untouched_created = link.created_at;
    assert!(untouched_used.is_some(), "the fixture pins a connect time");

    // A RESULT-only patch that clears the measurement, with a stale snapshot
    // for the columns it does not own.
    let mut stale = link.clone();
    stale.last_seen_at = ts(0);
    stale.created_at = ts(0);
    stale.last_used_at = None;
    stale.speed_bps = None;
    stale.latency = None;
    assert_eq!(
        db.apply_link_patches(&[xray_tui_db::LinkPatch {
            link: stale,
            groups: LinkGroups::RESULT,
        }])
        .await
        .expect("patch"),
        1
    );

    let after = db
        .read_link_row(ProtocolId::new(11), EndpointId::new(1))
        .await
        .expect("read")
        .expect("link");
    assert_eq!(after.latency, None, "the RESULT group landed");
    assert_eq!(after.error, None, "the RESULT group landed");
    assert_eq!(
        after.last_seen_at, untouched_seen,
        "last_seen_at is not the patch's to write"
    );
    assert_eq!(
        after.created_at, untouched_created,
        "created_at is not the patch's to write"
    );
    assert_eq!(
        after.last_used_at, untouched_used,
        "a connect time is not the patch's to write"
    );
}

/// A patch for a link that has never been persisted inserts the whole
/// snapshot, and the endpoint's ordering key reflects it immediately.
#[tokio::test]
async fn link_patch_inserts_a_missing_row_and_refreshes_its_key() {
    let db = seed_projection_fixture().await;
    let mut fresh = db
        .read_link_row(ProtocolId::new(11), EndpointId::new(1))
        .await
        .expect("read")
        .expect("link");
    fresh.protocol_id = ProtocolId::new(4242);
    fresh.latency = Some(xray_tui_db::models::Latency::Real { delay: 5, ip: None });
    fresh.error = None;

    assert_eq!(
        db.apply_link_patches(&[xray_tui_db::LinkPatch {
            link: fresh,
            groups: LinkGroups::RESULT,
        }])
        .await
        .expect("patch"),
        1
    );

    let inserted = db
        .read_link_row(ProtocolId::new(4242), EndpointId::new(1))
        .await
        .expect("read")
        .expect("inserted");
    assert_eq!(
        inserted.latency,
        Some(xray_tui_db::models::Latency::Real { delay: 5, ip: None })
    );
}
