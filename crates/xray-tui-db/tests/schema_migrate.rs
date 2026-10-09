//! The migration runner's contract (`crate::schema`).
//!
//! These pin the four outcomes that replaced the old "tag matches or the file
//! is deleted" rule: a current file is a NO-OP, a fresh file is created, an
//! already-migrated file survives a reopen untouched, and a file whose cursor
//! names an unknown schema is reported as incompatible (so the caller can apply
//! its wipe) rather than silently mangled.

use toasty::stmt::Value;
use xray_tui_db::{
    Database, SCHEMA_VERSION, models::Endpoint, models::EndpointId, models::HostType,
};

fn first_i64(rows: &[Value]) -> Option<i64> {
    rows.first().and_then(|v| match v {
        Value::Record(fields) => fields.first().and_then(|f| match f {
            Value::I64(n) => Some(*n),
            _ => None,
        }),
        _ => None,
    })
}

/// The cursor the runner reads is exactly `SCHEMA_VERSION` — one source, so a
/// bump cannot land in one and not the other.
#[test]
fn migration_latest_matches_schema_version() {
    assert_eq!(xray_tui_db::schema::LATEST, SCHEMA_VERSION);
}

/// A FRESH file is created at the current version and gets the raw DDL (the
/// `endpoint_rank` columns and indexes toasty cannot express).
#[tokio::test]
async fn fresh_open_creates_tables_and_raw_ddl() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = Database::open(dir.path().join("fresh.db"))
        .await
        .expect("open");
    let mut conn = db.connection().await.expect("conn");

    let rows = toasty::sql::query("PRAGMA user_version")
        .exec(&mut conn)
        .await
        .expect("version");
    assert_eq!(first_i64(&rows), Some(SCHEMA_VERSION));

    // The raw columns the runner adds: neither is a toasty model field.
    for column in ["band", "rank_weight"] {
        let rows = toasty::sql::query(format!(
            "SELECT COUNT(*) FROM pragma_table_info('endpoint_rank') WHERE name = '{column}'"
        ))
        .exec(&mut conn)
        .await
        .expect("table_info");
        assert_eq!(first_i64(&rows), Some(1), "endpoint_rank.{column} exists");
    }
    // And the raw indexes (C2: composite / mixed-direction).
    let rows = toasty::sql::query(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' \
         AND name IN ('endpoint_rank_key_v2', 'endpoint_rank_band_window', 'endpoint_ip_by_key')",
    )
    .exec(&mut conn)
    .await
    .expect("count indexes");
    assert_eq!(first_i64(&rows), Some(3), "the three raw indexes exist");
}

/// Reopening a CURRENT file is a no-op: the rows survive, and the runner does
/// not re-run `CREATE TABLE` (which has no `IF NOT EXISTS` and would fail).
#[tokio::test]
async fn reopen_of_a_current_file_preserves_data() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("current.db");

    {
        let db = Database::open(&path).await.expect("fresh open");
        let mut conn = db.connection().await.expect("conn");
        toasty::create!(Endpoint {
            created_at: 0,
            id: EndpointId::new(31),
            domain: Endpoint::derive_domain("keep.example", HostType::Dns).0,
            sub_domain: Endpoint::derive_domain("keep.example", HostType::Dns).1,
            port: 443,
            ports: Vec::<u16>::new(),
        })
        .exec(&mut conn)
        .await
        .expect("seed");
    }

    // Reopen: cursor == LATEST, so the runner ensures the (idempotent) raw DDL
    // and returns — no wipe, no failed `CREATE TABLE`.
    let db = Database::open(&path).await.expect("reopen");
    let mut conn = db.connection().await.expect("conn");
    let kept = Endpoint::filter_by_id(EndpointId::new(31))
        .first()
        .exec(&mut conn)
        .await
        .expect("read");
    assert!(kept.is_some(), "a current-version reopen preserves data");

    let rows = toasty::sql::query("PRAGMA user_version")
        .exec(&mut conn)
        .await
        .expect("version");
    assert_eq!(first_i64(&rows), Some(SCHEMA_VERSION));
}

/// Reopening an ALREADY-MIGRATED file re-applies the raw DDL idempotently — in
/// particular the two `ALTER TABLE ADD COLUMN`s, which have no
/// `IF NOT EXISTS` and whose duplicate-column error the runner must tolerate
/// (not propagate, which would make every reopen fail).
#[tokio::test]
async fn reopen_tolerates_duplicate_column_on_already_migrated_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("reopen.db");

    for _ in 0..3 {
        let db = Database::open(&path)
            .await
            .expect("each open must succeed — a tolerated duplicate-column ALTER");
        let mut conn = db.connection().await.expect("conn");
        let rows = toasty::sql::query(
            "SELECT COUNT(*) FROM pragma_table_info('endpoint_rank') \
             WHERE name IN ('band', 'rank_weight')",
        )
        .exec(&mut conn)
        .await
        .expect("table_info");
        assert_eq!(first_i64(&rows), Some(2), "both raw columns still present");
        drop(conn);
        drop(db);
    }
}

/// A file whose cursor names a schema this build does not know is recreated by
/// `open` (the pre-alpha wipe), NOT migrated. `17` is a pre-migration tag: the
/// old `user_version` world where 1..17 were unrelated schemas.
#[tokio::test]
async fn unknown_cursor_is_recreated_not_migrated() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("foreign.db");

    {
        let db = Database::open(&path).await.expect("fresh open");
        let mut conn = db.connection().await.expect("conn");
        toasty::create!(Endpoint {
            created_at: 0,
            id: EndpointId::new(41),
            domain: Endpoint::derive_domain("gone.example", HostType::Dns).0,
            sub_domain: Endpoint::derive_domain("gone.example", HostType::Dns).1,
            port: 443,
            ports: Vec::<u16>::new(),
        })
        .exec(&mut conn)
        .await
        .expect("seed");
        // Pretend a previous schema generation wrote the file.
        toasty::sql::query("PRAGMA user_version = 17")
            .exec(&mut conn)
            .await
            .expect("tag");
    }

    let db = Database::open(&path).await.expect("reopen wipes");
    let mut conn = db.connection().await.expect("conn");
    let rows = toasty::sql::query("PRAGMA user_version")
        .exec(&mut conn)
        .await
        .expect("version");
    assert_eq!(
        first_i64(&rows),
        Some(SCHEMA_VERSION),
        "the file is rebuilt at the current version"
    );
    assert!(
        Endpoint::filter_by_id(EndpointId::new(41))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .is_none(),
        "an unknown cursor is a wipe, not a migration"
    );
}

/// The day-one stability seed (spec `2026-10-09-stab-bin-design` §6.2).
///
/// An existing database's rings are empty, but `apply_test_result` KEEPS
/// `latency` when it later writes an error — so every previously-successful
/// link carries a real latency with no ring, and under R1 those endpoints would
/// match NO scope. The seed gives such a link a one-sample success ring, which
/// makes `rank_proven` true and the window neutral (one sample < `STAB_WARMUP`).
#[tokio::test]
async fn the_stab_seed_marks_real_latency_links_as_proven() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("seed.db");
    {
        let db = Database::open(&path).await.expect("open");
        let mut conn = db.connection().await.expect("conn");
        // Two endpoints, one with a REAL latency (a past success) and one with a
        // FAST one (a handshake, which proves nothing).
        // IP-literal hosts: a DNS host with no resolved address is deliberately
        // NOT provable (it cannot be dialled), so it would mask the seed.
        for (id, host, port) in [(1_i64, "198.51.100.1", 443_u16), (2, "198.51.100.2", 8443)] {
            let (domain, sub_domain) = Endpoint::derive_domain(host, HostType::Ipv4);
            toasty::create!(Endpoint {
                id: EndpointId::new(id),
                domain,
                sub_domain,
                port,
                ports: Vec::<u16>::new(),
                created_at: 0,
                last_source: None,
                manual_protocol_override: None,
                resolved_at: None,
            })
            .exec(&mut conn)
            .await
            .expect("endpoint");
        }
        for (pid, eid, kind) in [(101_i64, 1_i64, "real"), (102, 2, "fast")] {
            toasty::sql::statement(format!(
                "INSERT INTO protocols (id, sig, proto_kind, transport_type, \
                 security_type, security_sni, security_fp, security_insecure, config, \
                 created_at) VALUES ({pid}, {pid}, 'vless', 'tcp', 'none', NULL, NULL, \
                 NULL, 'null', 0)"
            ))
            .exec(&mut conn)
            .await
            .expect("protocol");
            toasty::sql::statement(format!(
                "INSERT INTO profile_stats (protocol_id, endpoint_id, last_seen_at, \
                 latency, latency_delay, speed_bps, traffic_today_up, traffic_today_down, \
                 traffic_total_up, traffic_total_down, created_at, updated_at, version, \
                 stab_mask, stab_len) VALUES \
                 ({pid}, {eid}, 0, '{kind}', 42, NULL, 0, 0, 0, 0, 0, 0, 1, 0, 0)"
            ))
            .exec(&mut conn)
            .await
            .expect("link");
        }
        // Simulate the pre-seed state: drop the ring columns so the next open's
        // ALTER re-adds them and the seed fires. (An existing table is exactly
        // the case the ALTER path exists for.)
        toasty::sql::statement("ALTER TABLE profile_stats DROP COLUMN stab_mask")
            .exec(&mut conn)
            .await
            .expect("drop stab_mask");
        toasty::sql::statement("ALTER TABLE profile_stats DROP COLUMN stab_len")
            .exec(&mut conn)
            .await
            .expect("drop stab_len");
    }

    let db = Database::open(&path).await.expect("reopen");
    let mut conn = db.connection().await.expect("conn");
    let rows = toasty::sql::query(
        "SELECT endpoint_id, stab_mask, stab_len FROM profile_stats ORDER BY endpoint_id",
    )
    .exec(&mut conn)
    .await
    .expect("read rings");
    let as_triple = |v: &Value| match v {
        Value::Record(rec) => match (&rec.fields[0], &rec.fields[1], &rec.fields[2]) {
            (Value::I64(id), Value::I64(mask), Value::I64(len)) => Some((*id, *mask, *len)),
            _ => None,
        },
        _ => None,
    };
    let triples: Vec<(i64, i64, i64)> = rows.iter().filter_map(as_triple).collect();
    assert_eq!(
        triples,
        vec![(1, 1, 1), (2, 0, 0)],
        "a real latency seeds one success; a fast one seeds nothing"
    );

    let proven: i64 = first_i64(
        &toasty::sql::query(
            "SELECT COUNT(*) FROM endpoint_rank WHERE rank_proven = 1 AND endpoint_id = 1",
        )
        .exec(&mut conn)
        .await
        .expect("count proven"),
    )
    .unwrap_or(0);
    assert_eq!(
        proven, 1,
        "the seeded endpoint is proven after the rank refresh"
    );
    let stray: i64 = first_i64(
        &toasty::sql::query(
            "SELECT COUNT(*) FROM endpoint_rank WHERE rank_proven = 1 AND endpoint_id = 2",
        )
        .exec(&mut conn)
        .await
        .expect("count stray"),
    )
    .unwrap_or(0);
    assert_eq!(stray, 0, "the fast-only endpoint must NOT become proven");

    // The end-to-end claim (spec acceptance 6): after the upgrade the
    // previously-successful endpoint is STILL in the Successful scope, not
    // dropped out of it because its ring was empty.
    let req = xray_tui_db::profiles_query::PageRequest {
        // `All`: these fixture links are stamped at epoch 0, so the Active
        // window (measured against `now - ttl`) does not hold them.
        view: xray_tui_db::models::PurgatoryView::All,
        active_threshold: 0,
        search: None,
        group_id: None,
        scope: xray_tui_db::profiles_query::PlanScope::Successful,
        sort: xray_tui_db::profiles_query::PageSort::Test,
        ascending: true,
        offset: 0,
        limit: 100,
    };
    let page = db.profiles_page(&req).await.expect("successful page");
    assert_eq!(
        page.total, 1,
        "the seeded endpoint is the Successful scope's only member"
    );
    assert_eq!(
        page.ids.iter().map(|id| id.get()).collect::<Vec<_>>(),
        vec![1],
        "and it is the REAL-latency endpoint, not the fast-only one"
    );
}
