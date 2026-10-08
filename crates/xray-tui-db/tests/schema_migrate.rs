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
         AND name IN ('endpoint_rank_key', 'endpoint_rank_band_window', 'endpoint_ip_by_key')",
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
