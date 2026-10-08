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

/// What the cursor comparison concluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaState {
    /// The database is at the current version — nothing to do.
    Current,
    /// The database was created (fresh) or migrated to the current version.
    Applied,
    /// The file's cursor names a schema this code does not know. The caller
    /// decides what to do (the pre-alpha answer is a wipe).
    Incompatible(i64),
}

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
async fn ensure_raw_ddl(conn: &mut impl toasty::Executor) -> Result<()> {
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
    Ok(())
}

/// Bring `db`/`conn` to the current schema version.
///
/// `db` is needed only to create the TABLES: a toasty model's `CREATE TABLE`
/// comes from `push_schema`, which is the schema of record until the typed
/// layer is retired (see `ddl.rs`). Everything toasty cannot express comes from
/// [`ddl`].
pub async fn migrate(db: &toasty::Db, conn: &mut toasty::Connection) -> Result<SchemaState> {
    match cursor(conn).await? {
        v if v == LATEST => {
            // Current. Still ensure the derived tables/indexes exist — this is
            // the only path a database created before a new raw statement was
            // added to `ddl.rs` takes, and every statement is idempotent.
            ensure_raw_ddl(conn).await?;
            Ok(SchemaState::Current)
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
            Ok(SchemaState::Applied)
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
