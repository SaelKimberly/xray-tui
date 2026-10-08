//! Every hand-written DDL statement in the crate, in one place.
//!
//! The rule (`docs/database-manual-sql.md` §1) is that the typed toasty path
//! is the default and raw SQL needs a recorded cause. This module is where the
//! DDL half of that list LIVES, so "which indexes exist" is one file rather
//! than a scatter of `ensure` functions. Each statement below carries its cause
//! from `docs/database-manual-sql.md` §2.
//!
//! # Why the tables are not here yet
//!
//! A toasty model's table is emitted by `push_schema` from the model
//! definition, and that emission IS the schema of record until the typed layer
//! is retired (`docs/aegis/specs/2026-10-08-db-adapter-and-migrations-design.md`
//! §6.1, stage S7). Re-writing those `CREATE TABLE` statements by hand here
//! would create a second spelling of the same fact with no reader forcing them
//! to agree, so the v18 migration calls `push_schema` for the tables and this
//! file for everything toasty cannot express. When S7 lands, the tables move
//! here and `push_schema` stops being called.
//!
//! # The raw half
//!
//! Two capability gaps of toasty's `#[index]` (single-column only, no
//! mixed-direction composite) force hand-written `CREATE INDEX`; the rest are
//! derived-state tables and columns on `endpoint_rank` that are deliberately
//! NOT model fields, so adding them costs no schema-tag bump and no wipe.

/// `endpoint_ip`'s covering index: the address-ordered read path
/// (`PageSort::Ip`'s `min(ip_key)` and the IP/CIDR search range).
///
/// Cause C2 — the sort wants `(ip_key, endpoint_id)` together, which toasty's
/// per-field `#[index]` cannot express. Additive, so it is re-run safely.
pub const ENDPOINT_IP_BY_KEY: &str =
    "CREATE INDEX IF NOT EXISTS endpoint_ip_by_key ON endpoint_ip (ip_key, endpoint_id)";

/// The Test order (decision 16): the default sort, and the one the tab scrolls
/// under.
///
/// Cause C2 — a composite whose last-but-one term is DESCENDING is outside
/// `#[index]`. This index is what makes a page an index scan (~1 ms) instead of
/// a sort over every endpoint (~240 ms). **A NEW NAME, never an edit in place**:
/// `IF NOT EXISTS` makes a changed column list a silent no-op on every database
/// that already has the old index, which would keep serving the new `ORDER BY`
/// and drop the page back to the filesort.
pub const ENDPOINT_RANK_KEY: &str = "CREATE INDEX IF NOT EXISTS endpoint_rank_key ON endpoint_rank(\
     band, rank_bin, rank_weight DESC, rank_domain, rank_sub_domain, rank_addr, endpoint_id)";

/// The directional reband sweep seeks `band = 0 AND rank_newest_seen < ?`.
pub const ENDPOINT_RANK_BAND_WINDOW: &str =
    "CREATE INDEX IF NOT EXISTS endpoint_rank_band_window ON endpoint_rank(band, rank_newest_seen)";

/// One-row stamp for the compiled weight tables. A table of opinions that lives
/// in code has no other way to know it is out of date.
pub const RANK_WEIGHT_META: &str = "CREATE TABLE IF NOT EXISTS rank_weight_meta \
     (id INTEGER PRIMARY KEY CHECK (id = 0), weight_version INTEGER NOT NULL)";

/// `endpoint_rank`'s two RAW columns, added with `ALTER TABLE`.
///
/// Both are deliberately NOT toasty model fields — they are derived columns on
/// a derived table, so declaring them would mean a schema-tag bump and (under
/// the old tag world) a wipe of a large enriched feed. They cannot be part of
/// `CREATE TABLE` for the same reason.
///
/// `ALTER TABLE ... ADD COLUMN` is NOT idempotent: it errors on a column that
/// already exists. There is no `IF NOT EXISTS` form on this engine, so the
/// runner applies these and TOLERATES the duplicate-column error — the same
/// behaviour the pre-migration `ensure_in` had. `rank_weight` is `NOT NULL
/// DEFAULT`, which is what makes the column legal for pre-existing rows (and
/// why a NULL weight in the anchor query would otherwise error).
pub const ENDPOINT_RANK_ADD_COLUMNS: &[&str] = &[
    "ALTER TABLE endpoint_rank ADD COLUMN band INTEGER",
    "ALTER TABLE endpoint_rank ADD COLUMN rank_weight BLOB \
     NOT NULL DEFAULT x'0000000000000000'",
];

/// Indexes and tables created AFTER the ALTERs above (the covering index names
/// `band`, so it cannot run before the column exists). All idempotent.
pub const V18_AFTER_ALTERS: &[&str] = &[
    ENDPOINT_IP_BY_KEY,
    ENDPOINT_RANK_KEY,
    ENDPOINT_RANK_BAND_WINDOW,
    RANK_WEIGHT_META,
];
