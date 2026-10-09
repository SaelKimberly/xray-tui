//! The schema's single owner: a versioned migration cursor.
//!
//! Replaces the previous `PRAGMA user_version` tag, whose ONLY behaviours were
//! "run `push_schema`" and "otherwise DELETE the database file" (AGENTS
//! decision 4). A tag mismatch was therefore a wipe, never a migration; there
//! was no way to add a column, an index, or a table to an existing feed.
//!
//! See `docs/aegis/specs/2026-10-08-db-adapter-and-migrations-design.md` §5 and
//! §5.1, and `ddl.rs` for where every hand-written statement lives.
//!
//! # The cursor
//!
//! `PRAGMA user_version` holds the newest APPLIED migration version. The list
//! SEEDS at the schema the crate ships today (`SCHEMA_VERSION`, 18), so on the
//! day this lands an existing tag-18 file is already current and a fresh file
//! applies 18 — no data is touched either way. Every later change is 19+.
//!
//! # What a mismatch means now
//!
//! * `cursor == latest` — the schema is current. Ensure the raw DDL exists
//!   (idempotent) and return.
//! * `cursor == 0` — a fresh file. Create the tables, then the raw DDL.
//! * `0 < cursor < latest` — apply the migrations after `cursor`, in order.
//! * anything else — a foreign or pre-migration file. That is NOT a migration
//!   opportunity: those schemas are unreadable by this code, so it is reported
//!   as `Incompatible` and the caller applies its documented wipe. This is an
//!   EXPLICIT verdict; previously the same situation was inferred from a
//!   `CREATE TABLE` failing.

use crate::Result;
use crate::error::DatabaseError;

pub mod ddl;

/// The version this crate's schema is at. Distinct from
/// [`crate::database::SCHEMA_VERSION`] only in that this one is the cursor's
/// authority; they must agree, and a test pins it.
pub const LATEST: i64 = crate::database::SCHEMA_VERSION;

/// Read the migration cursor. Reuses the crate's one PRAGMA decoder so the
/// cursor and every other `PRAGMA` read cannot drift.
async fn cursor(conn: &mut impl toasty::Executor) -> Result<i64> {
    let rows = toasty::sql::query("PRAGMA user_version").exec(conn).await?;
    Ok(crate::database::first_i64(&rows).unwrap_or(0))
}

/// Apply the raw DDL: the (non-idempotent) column adds first, then the
/// idempotent indexes and tables.
///
/// The ALTERs are attempted every time and their duplicate-column error is
/// TOLERATED — this engine has no `ALTER ... IF NOT EXISTS`, and the error is
/// the only signal that the column is already there. Any OTHER error is real
/// and propagates.
///
/// Returns `true` when the `stab_mask` ALTER actually ADDED the column, i.e.
/// this open is the one that introduced the stability ring to an existing
/// database. The caller uses that to run the day-one seed exactly once (spec
/// `2026-10-09-stab-bin-design` §6.2).
async fn ensure_raw_ddl(conn: &mut impl toasty::Executor) -> Result<bool> {
    let mut stabbed = false;
    for alter in ddl::STAB_ADD_COLUMNS {
        if let Err(e) = toasty::sql::query(*alter).exec(conn).await {
            let message = e.to_string();
            if !message.contains("duplicate column") {
                return Err(e.into());
            }
        } else {
            stabbed = true;
        }
    }
    for alter in ddl::ENDPOINT_RANK_ADD_COLUMNS {
        if let Err(e) = toasty::sql::query(*alter).exec(conn).await {
            let message = e.to_string();
            if !message.contains("duplicate column") {
                return Err(e.into());
            }
        }
    }
    for statement in ddl::V18_AFTER_ALTERS {
        toasty::sql::query(*statement).exec(conn).await?;
    }
    Ok(stabbed)
}

/// Bring `db`/`conn` to the current schema version.
///
/// `db` is needed only to create the TABLES: a toasty model's `CREATE TABLE`
/// comes from `push_schema`, which is the schema of record until the typed
/// layer is retired (see `ddl.rs`). Everything toasty cannot express comes from
/// [`ddl`].
pub async fn migrate(db: &toasty::Db, conn: &mut toasty::Connection) -> Result<()> {
    match cursor(conn).await? {
        v if v == LATEST => {
            // Current. Still ensure the derived tables/indexes exist — this is
            // the only path a database created before a new raw statement was
            // added to `ddl.rs` takes, and every statement is idempotent.
            let stabbed = ensure_raw_ddl(conn).await?;
            // Day-one stability seed (spec `2026-10-09-stab-bin-design` §6.2).
            // `latency = 'real'` is evidence a real success happened —
            // `apply_test_result` writes `Latency::Real` only on success and
            // KEEPS it when it later writes an error — but an existing
            // database's rings are empty, so every previously-successful link
            // would read `rank_proven = 0` and drop out of ALL four scoped
            // menus. Seeding the ring with that one known success keeps
            // `proven` true and the window neutral (one sample < `STAB_WARMUP`).
            //
            // Gated on the ALTER having ADDED the column, so it runs exactly
            // once; a later open tolerates the duplicate and skips this.
            if stabbed {
                let seeded = seed_stability(conn).await?;
                tracing::info!(target: "xray_tui_db",
                    "profile_stats: seeded {seeded} proven links into the stability ring");
            }
            Ok(())
        }
        0 => {
            // Cursor 0 with existing foreign tables: `push_schema` emits
            // `CREATE TABLE` without `IF NOT EXISTS`, so it FAILS on a table it
            // did not create. That failure IS the incompatibility signal — an
            // untagged, foreign or half-created file — and it is reported
            // explicitly so the caller applies its documented wipe. (The old
            // code inferred the same thing from the same failure; here it is a
            // named verdict.)
            db.push_schema().await.map_err(|e| {
                let message = e.to_string();
                if message.contains("already exists") {
                    DatabaseError::IncompatibleSchema(0)
                } else {
                    DatabaseError::Toasty(e)
                }
            })?;
            ensure_raw_ddl(conn).await?;
            toasty::sql::query(format!("PRAGMA user_version = {LATEST}"))
                .exec(conn)
                .await?;
            Ok(())
        }
        other => {
            // 0 < other < LATEST would be "apply the steps after `other`" once
            // real migrations exist. There are none yet, so any such value names
            // a schema from BEFORE this module existed — the pre-alpha tag
            // world, where 1..17 were unrelated schemas. Neither readable nor
            // migratable: report it, and `open` applies its documented wipe.
            Err(DatabaseError::IncompatibleSchema(other))
        }
    }
}

/// The day-one stability seed: give every link that has a real measurement a
/// one-sample success ring, so `rank_proven` is true for it immediately.
///
/// `latency` is the embed's discriminator column (`'real'`/`'fast'`/`''`), the
/// same one the rank refresh matches on. Only `'real'` seeds: a fast latency is
/// a TCP handshake, which proves nothing about the config.
///
/// Returns the number of rows seeded (counted before the update, since the
/// engine reports no affected-row count for a raw `UPDATE`).
///
/// Also CLEARS the key-input stamp, so `endpoint_rank::ensure` (which runs
/// later in `open`) sees a mismatch and recomputes every stored key — the
/// seeded rings must reach `rank_proven`/`rank_stab` or the seed has no effect
/// on the page.
async fn seed_stability(conn: &mut impl toasty::Executor) -> Result<i64> {
    let count = scalar_count(
        conn,
        "SELECT COUNT(*) FROM profile_stats WHERE latency = 'real'",
    )
    .await?;
    if count == 0 {
        return Ok(0);
    }
    toasty::sql::query(
        "UPDATE profile_stats SET stab_mask = 1, stab_len = 1 WHERE latency = 'real'",
    )
    .exec(conn)
    .await?;
    // `rank_weight_meta` may not exist yet on a very old file; the rank `ensure`
    // creates it. A failed delete here is therefore not an error.
    let _ = toasty::sql::query("DELETE FROM rank_weight_meta")
        .exec(conn)
        .await;
    Ok(count)
}

/// One integer from a single-value query (the schema module's own copy; the
/// rank module's is private to `endpoint_rank`).
async fn scalar_count(conn: &mut impl toasty::Executor, sql: &str) -> Result<i64> {
    let rows = toasty::sql::query(sql).exec(conn).await?;
    Ok(rows
        .first()
        .and_then(|row| match row {
            toasty_core::stmt::Value::Record(record) => record.fields.first().cloned(),
            _ => None,
        })
        .and_then(|v| match v {
            toasty_core::stmt::Value::I64(n) => Some(n),
            _ => None,
        })
        .unwrap_or(0))
}
