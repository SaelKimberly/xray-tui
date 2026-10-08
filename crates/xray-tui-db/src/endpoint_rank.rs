//! Materialized per-endpoint ordering keys.
//!
//! The Profiles page used to derive each endpoint's position from a
//! `ROW_NUMBER() OVER (PARTITION BY endpoint_id …)` window over every link
//! plus correlated subqueries for the display-link sorts. On the reference
//! feed (7,672 endpoints / 9,048 links) that cost ~975 ms per page fetch: the
//! window re-evaluates the ordering law for the whole table on every page.
//! Storing the keys and reading them through an index makes the same page
//! ~1 ms (measured; the plan is `SCAN … USING COVERING INDEX`).
//!
//! The keys are DERIVED STATE of the decision-16 law. The law is implemented
//! ONCE, here, over a minimal [`RankLink`] view — `EndpointRow::link_test_key`
//! delegates to it, so the panel order, the stored keys, and the parity golden
//! cannot drift apart. SQL only stores the numbers and reads them back; it
//! never re-derives the law.

use std::collections::HashMap;

use toasty_core::driver::operation::TransactionMode;
use toasty_core::stmt::Value;

use crate::models_toasty::{
    Endpoint, EndpointId, EndpointRank, EndpointRow, Latency, ProfileErr, ProfileStats, Protocol,
    ProtocolId,
};
use xray_tui_proto::proto_spec::{
    SecurityType, TransportType,
    weight::{ConfigWeight, ZERO_WEIGHT, weight_of},
};

/// Sentinel for "no display link": sorts before any real timestamp, matching
/// the retired SQL `COALESCE(<seen>, '')` (empty text sorts first ascending).
pub const NO_SEEN: i64 = i64::MIN;
/// Process-wide Active-view TTL (seconds). `band` is materialized against
/// `now − this` at write and sweep time, so the DB layer needs the same ttl
/// the page uses. Set once at startup (and on settings save) from the config;
/// defaults to 7 days so a write before the setter runs is still sensible.
static ACTIVE_TTL_SECS: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(7 * 86400);

/// Set the Active-view TTL used to materialize `band` (startup + settings save).
pub fn set_active_ttl_secs(secs: i64) {
    ACTIVE_TTL_SECS.store(secs, std::sync::atomic::Ordering::Relaxed);
}

/// The link facts the ordering law reads. Built from a typed `ProfileStats`
/// when one is in hand, or straight from the stored columns when the refresh
/// path would otherwise pay ~0.8 ms per bound id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RankLink {
    pub protocol_id: i64,
    /// `Some(real)` / `Some(fast)` for a measured link (the flag is "is it a
    /// real ping"), `None` when the link carries no latency.
    pub measured: Option<bool>,
    pub delay: i32,
    pub error_kind: Option<ProfileErr>,
    /// Epoch seconds of the link's `last_seen_at` (the recency tiebreak).
    pub seen_secs: i64,
    pub speed: Option<i64>,
    pub traffic: i64,
    /// The link carries a purge verdict (spec `2026-09-17-purge-reason`). It can
    /// no longer represent the endpoint while a live link exists.
    pub purged: bool,
    /// The static config weight — the compiled "which stack is likelier to
    /// work" prior (spec `2026-10-01-static-config-weight-design`).
    ///
    /// A FIELD, not something a `ProfileStats` conversion can derive: that row
    /// carries no protocol. Every construction site resolves it and passes it
    /// to [`RankLink::new`]; there is deliberately no weight-less constructor.
    pub weight: ConfigWeight,
}

/// The weight as the packed `u64` the law compares and the DB stores.
///
/// Big-endian packing puts the dominant `security` band in the most significant
/// bytes, so this number and the memcmp order of the stored BLOB are the same
/// order by construction.
#[must_use]
pub const fn weight_u64(weight: ConfigWeight) -> u64 {
    u64::from_be_bytes(weight.to_be_bytes())
}

impl RankLink {
    /// Build a link view. The weight is REQUIRED, not defaulted: it comes from
    /// the link's protocol, which this row does not carry.
    #[must_use]
    pub fn new(link: &ProfileStats, weight: ConfigWeight) -> Self {
        let (measured, delay) = match link.latency {
            Some(Latency::Real { delay, .. }) => (Some(true), delay),
            Some(Latency::Fast { delay }) => (Some(false), delay),
            None => (None, i32::MAX),
        };
        Self {
            protocol_id: link.protocol_id.get(),
            measured,
            delay,
            error_kind: link.error.as_ref().map(|e| e.kind),
            seen_secs: link.last_seen_at,
            speed: link.speed_bps,
            traffic: link
                .traffic
                .total_up
                .saturating_add(link.traffic.total_down),
            purged: link.purge_reason.is_some(),
            weight,
        }
    }

    /// Ascending key `(tier, !weight, latency, -seen, protocol_id)` — the
    /// decision-16 law with the static weight inserted INSIDE the tier.
    /// 0 real-ok, 1 fast-ok, 2 untested, 3 real/name-err, 4 fast-err,
    /// 5 dns-unresolved, 6 purged. Only success tiers carry a delay.
    ///
    /// The weight is NEGATED (`u64::MAX -`) so this stays ONE plain ascending
    /// tuple consumed by a single `.min()`, exactly like `neg_seen` below: a
    /// `Reverse` or any non-tuple wrapper would fork the single-implementation
    /// property the panel order and the stored keys both depend on. Higher
    /// weight = better, and — deliberately, at the accepted cost of the
    /// feature — it outranks `latency` inside a tier, so a 900 ms REALITY link
    /// sorts above a 40 ms TCP one.
    ///
    /// Tier 6 sits below every live band, so an endpoint's representative key is
    /// a live link whenever one exists; an endpoint whose links are all purged
    /// still gets a deterministic position for the Purgatory/All views.
    #[must_use]
    pub const fn key(&self, dns_unresolved: bool) -> (u8, u64, i64, i64) {
        // The representative-LINK key (db-rewamp D11): delay BIN, negated
        // weight, then recency and protocol id. `latency` is gone (the bin
        // buckets it); recency stays as a LINK tiebreak so the newest link
        // represents its endpoint — it is deliberately NOT in the endpoint key
        // (which orders by `domain`/`sub_domain`/`addr`), only in this
        // same-endpoint selection.
        (
            self.bin(dns_unresolved),
            u64::MAX - weight_u64(self.weight),
            -self.seen_secs,
            self.protocol_id,
        )
    }

    /// The delay bin (db-rewamp D11): real<50..real≥1000 = 0-5,
    /// fast<50..fast≥1000 = 6-11, untested = 12, real-err = 13, fast-err = 14,
    /// dns-err = 15, purged = 16. A real measurement stays above a fast one of
    /// any delay.
    #[must_use]
    pub const fn bin(&self, dns_unresolved: bool) -> u8 {
        if self.purged {
            return 16;
        }
        if dns_unresolved {
            return 15;
        }
        if let Some(kind) = &self.error_kind {
            return match kind {
                ProfileErr::Real | ProfileErr::Name => 13,
                ProfileErr::Fast => 14,
            };
        }
        match self.measured {
            Some(true) => delay_bin(self.delay),
            Some(false) => 6 + delay_bin(self.delay),
            None => 12,
        }
    }

    /// "Measured" rank of the display preference: real (0) before fast (1).
    const fn display_rank(&self) -> Option<u8> {
        match self.measured {
            Some(true) => Some(0),
            Some(false) => Some(1),
            None => None,
        }
    }
}

/// `0..5` for a delay: `<50 <100 <250 <500 <1000 ≥1000`.
const fn delay_bin(delay: i32) -> u8 {
    if delay < 50 {
        0
    } else if delay < 100 {
        1
    } else if delay < 250 {
        2
    } else if delay < 500 {
        3
    } else if delay < 1000 {
        4
    } else {
        5
    }
}

/// The endpoint's own packed address key for the rank `addr` tiebreak (db-rewamp
/// D10): an IP host's literal lives in `endpoint_ip`; a DNS host gets an empty key.
fn endpoint_addr_key(row: &EndpointRow) -> Vec<u8> {
    if row.endpoint.is_dns() {
        Vec::new()
    } else {
        row.resolved_ips
            .first()
            .map(|ip| crate::endpoint_ip::key_of(*ip))
            .unwrap_or_default()
    }
}

/// True when the endpoint is a DNS host whose resolution has not landed —
/// the flag that collapses its links into one band (decision 16, tier 5).
///
/// The second input is "does it have any resolved address", which the
/// `endpoint_ip` table answers (`EndpointRow::resolved_ips`). A failed lookup
/// is not a different state: it stamps `resolved_at` and leaves the address
/// set empty, which is exactly the unresolved band this flag is for.
#[must_use]
pub const fn dns_unresolved_endpoint(domain: &str, has_address: bool) -> bool {
    !domain.is_empty() && !has_address
}

/// True for an [`EndpointRow`].
#[must_use]
pub const fn dns_unresolved(row: &EndpointRow) -> bool {
    row.endpoint.is_dns() && row.resolved_ips.is_empty()
}

/// Index of the endpoint's display link.
///
/// The rule the retired SQL used: a manual override naming an existing link,
/// else the best measured link (real before fast, lowest delay, then protocol
/// id), else nothing. Deliberately not `active_link()`: that one falls back to
/// the selected link (the first link when nothing was ever measured), while
/// the ordering law treats "no measurement" as a sentinel that sorts first
/// ascending.
///
/// The purge gate: a purged link never sources the endpoint's displayed values
/// while a live link exists — the tier-6 sink applied to the display
/// preference, and it covers the override branch too (checking `display_rank`
/// would not: the override early-returns on the protocol id). A pin therefore
/// cannot resurrect a purged link as the Active row's delay/exit-IP/speed/
/// traffic/config source. An endpoint whose links are ALL purged falls back to
/// them, so a Purgatory row still shows the values it has.
#[must_use]
pub fn display_link_index(links: &[RankLink], override_protocol: Option<i64>) -> Option<usize> {
    let live = links.iter().any(|l| !l.purged);
    let eligible = |l: &RankLink| !live || !l.purged;
    if let Some(pid) = override_protocol
        && let Some(index) = links
            .iter()
            .position(|l| l.protocol_id == pid && eligible(l))
    {
        return Some(index);
    }
    links
        .iter()
        .enumerate()
        .filter(|(_, l)| eligible(l))
        .filter_map(|(i, l)| {
            l.display_rank()
                .map(|rank| (i, rank, l.delay, l.protocol_id))
        })
        .min_by_key(|&(_, rank, delay, protocol)| (rank, delay, protocol))
        .map(|(i, _, _, _)| i)
}

/// A rank row plus the one key that is NOT a model field.
///
/// `rank_weight` is a RAW column (added with `ALTER TABLE ADD COLUMN` beside
/// `band`), and the toasty model has no seat for a raw column —
/// declaring it on `EndpointRank` instead would need schema tag 15, which
/// decision 4 defines as a full database wipe. So the write path carries this
/// plain row instead of the model.
#[derive(Debug, Clone)]
pub struct RankRow {
    pub rank: EndpointRank,
    /// The REPRESENTATIVE link's weight — the one the law picked, so the stored
    /// key and the stored weight always describe the same link.
    pub weight: ConfigWeight,
}

/// The weight of a protocol row.
///
/// From the discriminators it already stores in scalar columns
/// (`transport.type`, `security.type`/`sni`/`fp`). The deferred `config` JSON
/// is NOT read: nothing in the formula needs it.
#[must_use]
pub fn weight_of_protocol(protocol: &Protocol) -> ConfigWeight {
    weight_of(
        protocol.transport.r#type,
        protocol.security.r#type,
        protocol.security.sni.as_deref(),
        protocol.security.fp.as_deref(),
    )
}

/// The weight for one link read from the raw `protocols` columns.
///
/// A NULL (the LEFT JOIN missed) or an unrecognised spelling is
/// [`ZERO_WEIGHT`] — "worst", never a guess, so a schema drift surfaces as a
/// worst-case weight instead of a panic on the write path.
///
/// The transport column is read with `TransportType::from_db_label`, NOT
/// `FromStr`: toasty's embed stores `http_upgrade`/`x_http` where the wire form
/// says `httpupgrade`/`xhttp`, and `FromStr` rejects those — which would
/// silently persist a ZERO weight for every such link while every typed path
/// computes a real one, splitting the stored page order from the in-memory
/// comparator. Verified against a real feed (145 `http_upgrade` + 169 `x_http`
/// protocol rows).
#[must_use]
pub fn weight_from_discriminators(
    transport: Option<&str>,
    security: Option<&str>,
    sni: Option<&str>,
    fp: Option<&str>,
) -> ConfigWeight {
    let (Some(transport), Some(security)) = (
        transport.and_then(TransportType::from_db_label),
        security.and_then(|s| s.parse::<SecurityType>().ok()),
    ) else {
        return ZERO_WEIGHT;
    };
    weight_of(transport, security, sni, fp)
}

/// Compute the stored keys for one endpoint. `None` when it has no links: the
/// page only ever lists endpoints that have at least one.
#[must_use]
pub fn compute_rank(
    endpoint_id: EndpointId,
    domain: &str,
    sub_domain: &str,
    addr: Vec<u8>,
    dns_unresolved: bool,
    links: &[RankLink],
) -> Option<RankRow> {
    let (bin, neg_weight, _, _) = links.iter().map(|l| l.key(dns_unresolved)).min()?;
    // The view windows ask whether any LIVE link falls in the band. A
    // purged-only endpoint therefore reports `NO_SEEN`, which is below every
    // window bound — that is exactly "it belongs to Purgatory" (spec §5), and it
    // is why one index range serves both populations.
    let newest_seen = links
        .iter()
        .filter(|l| !l.purged)
        .map(|l| l.seen_secs)
        .max()
        .unwrap_or(NO_SEEN);
    // The minimum key's weight term is `u64::MAX - weight`; invert it back.
    let weight = ConfigWeight::from_be_bytes((u64::MAX - neg_weight).to_be_bytes());
    Some(RankRow {
        rank: EndpointRank {
            endpoint_id,
            bin: i64::from(bin),
            domain: domain.to_string(),
            sub_domain: sub_domain.to_string(),
            addr,
            newest_seen,
        },
        weight,
    })
}

/// Keys for a whole row (the backfill/typed entry point).
#[must_use]
pub fn rank_of_row(row: &EndpointRow) -> Option<RankRow> {
    let links: Vec<RankLink> = row
        .links
        .iter()
        .map(|link| {
            let weight = row
                .protocols
                .get(&link.protocol_id)
                .map_or(ZERO_WEIGHT, weight_of_protocol);
            RankLink::new(link, weight)
        })
        .collect();
    compute_rank(
        row.endpoint.id,
        &row.endpoint.domain,
        &row.endpoint.sub_domain,
        endpoint_addr_key(row),
        dns_unresolved(row),
        &links,
    )
}

// ── Persistence ─────────────────────────────────────────────────────────
//
// A side table, not a `toasty` model: adding a model would need `push_schema`
// (skipped at the current tag) and therefore a tag bump, which decision 4
// defines as a full wipe. `CREATE TABLE IF NOT EXISTS` is additive and
// idempotent, so an existing database keeps its rows, and dropping the table
// restores the previous behaviour.
//
// Reads and writes inline their integer ids instead of binding them: the
// engine parses ~0.8 ms per bound parameter (200 ids = 174 ms; the same
// statement with literals = 9.7 ms), which would dominate every refresh.

/// The Test order (decision 16): the default sort, and the one the tab
/// scrolls under.
///
/// It stays a raw statement because toasty's `#[index]` is single-column and
/// cannot express a composite whose last-but-one term is DESCENDING — and this
/// index is what makes a page an index scan (~1 ms) instead of a sort over
/// every endpoint (~240 ms). Additive (`IF NOT EXISTS`), no data of its own.
const COVERING_INDEX: &str = "CREATE INDEX IF NOT EXISTS endpoint_rank_key ON endpoint_rank(\
     band, rank_bin, rank_weight DESC, rank_domain, rank_sub_domain, rank_addr, endpoint_id)";

/// Rows per bulk statement.
const RANK_CHUNK: usize = 400;

/// Column order the bulk insert writes (explicit so a schema reorder cannot
/// silently mis-map values).
///
/// `rank_weight` is a RAW column and IS listed here. `INSERT OR REPLACE`
/// re-inserts the row, so any column missing from this list is reset to NULL
/// on every single refresh — and a NULL there makes `profiles_anchor` fail
/// rather than merely mis-sort, because the anchor binds each term's value back
/// into a comparison.
const RANK_COLUMNS: &str = "endpoint_id, rank_bin, rank_weight, rank_domain, \
     rank_sub_domain, rank_addr, rank_newest_seen";

/// The weight column: an 8-byte big-endian blob, `NOT NULL` so an un-refreshed
/// row is still a legal ordering term.
///
/// BLOB, not INTEGER: SQLite orders blobs by memcmp, which makes `ORDER BY
/// rank_weight DESC` the Rust comparator's order BY CONSTRUCTION over the whole
/// 64-bit range. An INTEGER would store any value ≥ 2^63 as negative and sort
/// it LAST, and would cost bit 15 of the dominant security band as well.
/// `NOT NULL DEFAULT` materializes "worst" for every pre-existing row, which is
/// a valid position; the `WEIGHT_VERSION` check below is what replaces it with
/// real weights.
const WEIGHT_COLUMN: &str = "ALTER TABLE endpoint_rank ADD COLUMN rank_weight BLOB \
     NOT NULL DEFAULT x'0000000000000000'";

/// The weight column's covering index — a NEW NAME, never an edit in place:
/// `CREATE INDEX IF NOT EXISTS` makes a changed column list a silent no-op on
/// One-row stamp for the compiled weight tables. A table of opinions that lives
/// in code has no other way to know it is out of date.
const WEIGHT_META_TABLE: &str = "CREATE TABLE IF NOT EXISTS rank_weight_meta \
     (id INTEGER PRIMARY KEY CHECK (id = 0), weight_version INTEGER NOT NULL)";

/// The directional reband sweep seeks `band = 0 AND rank_newest_seen < ?`.
const BAND_WINDOW_INDEX: &str = "CREATE INDEX IF NOT EXISTS endpoint_rank_band_window \
     ON endpoint_rank(band, rank_newest_seen)";

/// Create the rank table and, on a database that has none yet, fill it from
/// the current link state. Runs once per database: an upgraded one pays the
/// backfill here, at open, instead of on its first page.
pub(crate) async fn ensure(conn: &mut toasty::Connection) -> crate::Result<()> {
    // The table itself comes from the schema (tag 8) — this only creates the
    // indexes and, on a database whose keys are not materialized yet, fills
    // them. One transaction: the indexes and the fill land together, and no
    // implicit write lock outlives the call (leaving one behind made the next
    // writer on the pool time out with "database is locked").
    let mut tx = conn
        .transaction_builder()
        .mode(TransactionMode::Immediate)
        .begin()
        .await?;
    let result = ensure_in(&mut tx).await;
    match result {
        Ok(()) => tx.commit().await?,
        Err(e) => return Err(e),
    }
    Ok(())
}

async fn ensure_in(conn: &mut impl toasty::Executor) -> crate::Result<()> {
    // `band` is a RAW column on this derived table (NOT a toasty model field),
    // so the page's Active membership is a stored `band = 0` — no schema-tag
    // bump, no file wipe. ADD COLUMN is attempted every open; the
    // duplicate-column error on an already-migrated database is expected and
    // ignored. (`rank_host` died with `PageSort::Address` — db-rewamp §3.2.)
    for alter in [
        "ALTER TABLE endpoint_rank ADD COLUMN band INTEGER",
        WEIGHT_COLUMN,
    ] {
        let _ = toasty::sql::query(alter).exec(conn).await;
    }
    // A NEW index name, not an edited one: `IF NOT EXISTS` makes a changed
    // column list a no-op on every database that already has the old index,
    // which would keep serving the new ORDER BY and drop the page back to the
    // ~240 ms filesort. The old index is left in place (it costs only write
    // time) so a rollback still has one.
    for ddl in [COVERING_INDEX, BAND_WINDOW_INDEX, WEIGHT_META_TABLE] {
        toasty::sql::query(ddl).exec(conn).await?;
    }
    // The compiled weight tables are opinions that live in code, so an upgrade
    // can invalidate every stored weight. This check MUST sit on the populated
    // branch below: an upgraded database takes the `> 0` path, and a NULL-based
    // fill cannot help it — `ADD COLUMN … NOT NULL DEFAULT` materializes
    // "worst" for every pre-existing row, so there are no NULLs to find.
    // Absence of the meta table reads as a mismatch too: that is exactly "this
    // database predates the weight".
    let stored_version = scalar_i64(
        conn,
        "SELECT weight_version FROM rank_weight_meta WHERE id = 0",
    )
    .await
    .unwrap_or(0);
    let weight_stale = stored_version != i64::from(crate::weight::WEIGHT_VERSION);

    if scalar_i64(conn, "SELECT COUNT(*) FROM endpoint_rank").await? > 0 {
        // A database whose keys are absent or stale (written before a refresh
        // path existed, or by a path that bypassed one) heals here rather than
        // hiding rows from the page.
        repair_missing(conn).await?;
        // One-time band backfill for rows that predate the column:
        // `repair_missing` fills only ABSENT rows, so an existing rank row
        // carries a NULL band until it is next refreshed. Fill them once here.
        backfill_bands(conn).await?;
        if weight_stale {
            let written = backfill_all(conn).await?;
            stamp_weight_version(conn).await?;
            tracing::info!(target: "xray_tui_db",
                stored = stored_version, want = crate::weight::WEIGHT_VERSION, written,
                "endpoint_rank: weight tables changed, keys recomputed");
        }
        return Ok(());
    }
    let written = backfill_all(conn).await?;
    stamp_weight_version(conn).await?;
    tracing::info!(target: "xray_tui_db", "endpoint_rank: backfilled {written} rows");
    Ok(())
}

/// Record the compiled tables' version, so the next open can tell whether the
/// stored weights were produced by the code that is running now.
async fn stamp_weight_version(conn: &mut impl toasty::Executor) -> crate::Result<()> {
    toasty::sql::query(format!(
        "INSERT INTO rank_weight_meta (id, weight_version) VALUES (0, {}) \
         ON CONFLICT(id) DO UPDATE SET weight_version = excluded.weight_version",
        i64::from(crate::weight::WEIGHT_VERSION)
    ))
    .exec(conn)
    .await?;
    Ok(())
}

/// Fill `band` for rows that predate the column (`band IS NULL`) — the
/// non-destructive upgrade's one-time cost. Same derivation as `write`'s
/// follow-up: the ttl membership.
async fn backfill_bands(conn: &mut impl toasty::Executor) -> crate::Result<()> {
    if scalar_i64(
        conn,
        "SELECT COUNT(*) FROM endpoint_rank WHERE band IS NULL",
    )
    .await?
        == 0
    {
        return Ok(());
    }
    let threshold = crate::models_toasty::now_epoch()
        - ACTIVE_TTL_SECS.load(std::sync::atomic::Ordering::Relaxed);
    toasty::sql::query(format!(
        "UPDATE endpoint_rank SET \
         band = CASE WHEN rank_newest_seen >= {threshold} THEN 0 ELSE 1 END \
         WHERE band IS NULL"
    ))
    .exec(conn)
    .await?;
    Ok(())
}

/// One integer from a single-value query.
async fn scalar_i64(conn: &mut impl toasty::Executor, sql: &str) -> crate::Result<i64> {
    let rows = toasty::sql::query(sql).exec(conn).await?;
    Ok(rows
        .first()
        .and_then(|row| match row {
            Value::Record(record) => record.fields.first().cloned(),
            _ => None,
        })
        .and_then(|v| match v {
            Value::I64(n) => Some(n),
            _ => None,
        })
        .unwrap_or(0))
}

/// Upsert the rank rows on the caller's executor, one multi-row statement per
/// chunk.
///
/// Per-row `upsert_by_endpoint_id` costs ~1.2 ms per statement on this engine,
/// so a flush window's 400 endpoints took ~470 ms and a 7.7k-endpoint import
/// ~8 s of key writes; this form does the same rows in one statement per chunk
/// (measured ~10× cheaper). The values are inlined for the same reason the
/// reads are: they are integers the database produced, never user text, and
/// the engine charges ~0.8 ms per bound parameter.
pub(crate) async fn write(
    conn: &mut impl toasty::Executor,
    ranks: &[RankRow],
) -> crate::Result<usize> {
    let threshold = crate::models_toasty::now_epoch()
        - ACTIVE_TTL_SECS.load(std::sync::atomic::Ordering::Relaxed);
    for chunk in ranks.chunks(RANK_CHUNK) {
        let values = chunk
            .iter()
            .map(|row| {
                let r = &row.rank;
                format!(
                    "({},{},{},{},{},{},{})",
                    r.endpoint_id.get(),
                    r.bin,
                    row.weight.sql_literal(),
                    crate::database::sql_lit(&r.domain),
                    crate::database::sql_lit(&r.sub_domain),
                    blob_lit(&r.addr),
                    r.newest_seen
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        toasty::sql::statement(format!(
            "INSERT OR REPLACE INTO endpoint_rank ({RANK_COLUMNS}) VALUES {values}"
        ))
        .exec(conn)
        .await?;
        // `band` is a RAW column (not in the toasty model): INSERT OR REPLACE
        // re-inserts the row and nulls it, so re-set it for this chunk — the
        // ttl membership from the just-written rank_newest_seen.
        //
        // `rank_weight` is NOT re-set here: it is Rust-computed, so it went
        // into the INSERT's values tuple above and needs no second statement.
        let ids = chunk
            .iter()
            .map(|row| row.rank.endpoint_id.get().to_string())
            .collect::<Vec<_>>()
            .join(",");
        toasty::sql::statement(format!(
            "UPDATE endpoint_rank SET \
             band = CASE WHEN rank_newest_seen >= {threshold} THEN 0 ELSE 1 END \
             WHERE endpoint_id IN ({ids})"
        ))
        .exec(conn)
        .await?;
    }
    Ok(ranks.len())
}

/// The raw facts the rank law needs, straight from the stored columns.
struct RawEndpoint {
    domain: String,
    sub_domain: String,
    addr: Vec<u8>,
    dns_unresolved: bool,
}

impl crate::Database {
    /// Create the keys of any endpoint that has links but no rank row.
    ///
    /// The page drives from `endpoint_rank`, so a write that bypassed the
    /// refresh would hide its endpoint until this ran. It runs at `open` and
    /// is available to callers that write links outside the normal paths
    /// (fixtures, maintenance scripts).
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn repair_endpoint_ranks(&self) -> crate::Result<usize> {
        let db = self;
        crate::retry_on_busy(
            move || async move {
                let mut conn = db.connection().await?;
                repair_missing(&mut conn).await
            },
            5,
        )
        .await
    }
    /// Recompute the stored ordering keys for `endpoint_ids`.
    ///
    /// Called by every write path that can change a link, so a stored key is
    /// never older than the write that invalidated it.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn refresh_endpoint_ranks(
        &self,
        endpoint_ids: &[EndpointId],
    ) -> crate::Result<usize> {
        let db = self;
        let endpoint_ids = endpoint_ids.to_vec();
        crate::retry_on_busy(
            move || {
                let endpoint_ids = endpoint_ids.clone();
                async move {
                    let mut conn = db.connection().await?;
                    refresh(&mut conn, &endpoint_ids).await
                }
            },
            5,
        )
        .await
    }

    /// Set the Active-view TTL used to materialize `band` (startup + settings
    /// save). Band is stored against `now − ttl`, so this must match the ttl
    /// the page's view means by "Active".
    pub fn set_band_ttl_secs(&self, secs: i64) {
        set_active_ttl_secs(secs);
    }

    /// Directional reband sweep: demote endpoints whose newest LIVE link has
    /// aged past the Active threshold (`band` 0 → 1). Monotonic and
    /// continuity-independent — it seeks exactly the drifted rows through the
    /// `(band, rank_newest_seen)` index, so an arbitrary downtime gap is
    /// corrected in one range scan (not a fixed window that a long gap could
    /// skip). Run at startup after the ttl is set (before the first page) and
    /// on the retention tick.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn reband_expired(&self) -> crate::Result<()> {
        let db = self;
        crate::retry_on_busy(
            move || async move {
                let threshold = crate::models_toasty::now_epoch()
                    - ACTIVE_TTL_SECS.load(std::sync::atomic::Ordering::Relaxed);
                let mut conn = db.connection().await?;
                toasty::sql::query(format!(
                    "UPDATE endpoint_rank SET band = 1 WHERE band = 0 AND rank_newest_seen < {threshold}"
                ))
                .exec(&mut conn)
                .await?;
                Ok(())
            },
            5,
        )
        .await
    }

    /// Full reband: recompute `band` for EVERY row against the current
    /// threshold, in BOTH directions. Used at startup, where the configured
    /// ttl may differ from the open-time default either way (a larger ttl must
    /// PROMOTE rows the default demoted), so the directional sweep alone is not
    /// enough. One indexed pass, once.
    #[tracing::instrument(target = "db_method", skip_all, fields(retries = tracing::field::Empty))]
    pub async fn reband_all(&self) -> crate::Result<()> {
        let db = self;
        crate::retry_on_busy(
            move || async move {
                let threshold = crate::models_toasty::now_epoch()
                    - ACTIVE_TTL_SECS.load(std::sync::atomic::Ordering::Relaxed);
                let mut conn = db.connection().await?;
                toasty::sql::query(format!(
                    "UPDATE endpoint_rank SET \
                     band = CASE WHEN rank_newest_seen >= {threshold} THEN 0 ELSE 1 END"
                ))
                .exec(&mut conn)
                .await?;
                Ok(())
            },
            5,
        )
        .await
    }
}

/// Delete rank rows whose endpoint no longer has any link.
///
/// The page drives from this table, so a lingering row would list a linkless
/// endpoint. Called by the deletion owners (`purge_expired`) rather than on a
/// timer: they are the only writers that remove links.
pub(crate) async fn prune(
    conn: &mut impl toasty::Executor,
    endpoint_ids: &[EndpointId],
) -> crate::Result<usize> {
    for id in endpoint_ids {
        EndpointRank::filter_by_endpoint_id(*id)
            .delete()
            .exec(conn)
            .await?;
    }
    Ok(endpoint_ids.len())
}

/// The first packed address key per endpoint (for the rank `addr` tiebreak) —
/// an IP host's literal lives in `endpoint_ip` now (db-rewamp D10).
async fn first_ip_keys(
    conn: &mut impl toasty::Executor,
) -> crate::Result<std::collections::HashMap<i64, Vec<u8>>> {
    let rows = toasty::sql::query(
        "SELECT endpoint_id, ip_key FROM endpoint_ip ORDER BY endpoint_id, ip_key",
    )
    .exec(conn)
    .await?;
    let mut out: std::collections::HashMap<i64, Vec<u8>> = std::collections::HashMap::new();
    for row in &rows {
        let Value::Record(record) = row else { continue };
        let Some(id) = record.fields.first().and_then(as_i64) else {
            continue;
        };
        if let Some(key) = record.fields.get(1).and_then(as_blob) {
            out.entry(id).or_insert(key);
        }
    }
    Ok(out)
}

/// Recompute EVERY endpoint's stored keys from its current links.
///
/// For the wholesale resets (`clear_all_stats`), for an upgraded database whose
/// stored weights came from a different version of the compiled tables, and for
/// a database that has no keys at all: every case where nothing narrower is
/// correct, because every link just lost — or never had — the columns the keys
/// are made of.
pub(crate) async fn backfill_all(conn: &mut impl toasty::Executor) -> crate::Result<usize> {
    let endpoints: Vec<Endpoint> = Endpoint::all().exec(conn).await?;
    let links: Vec<ProfileStats> = ProfileStats::all().exec(conn).await?;
    // The weight lives on the PROTOCOL row, not the link, so a third typed load
    // is what puts it in scope here. It is a scan of a small table (one row per
    // distinct config, shared by every endpoint carrying it) and the deferred
    // `config` JSON stays unloaded.
    let weights = protocol_weights(conn).await?;
    let resolved = resolved_endpoint_ids(conn).await?;
    let addr_keys = first_ip_keys(conn).await?;
    let mut by_endpoint: HashMap<EndpointId, Vec<RankLink>> = HashMap::new();
    for link in links {
        let weight = weights
            .get(&link.protocol_id)
            .copied()
            .unwrap_or(ZERO_WEIGHT);
        by_endpoint
            .entry(link.endpoint_id)
            .or_default()
            .push(RankLink::new(&link, weight));
    }
    let ranks: Vec<RankRow> = endpoints
        .into_iter()
        .filter_map(|endpoint| {
            let links = by_endpoint.remove(&endpoint.id)?;
            let addr = addr_keys
                .get(&endpoint.id.get())
                .cloned()
                .unwrap_or_default();
            compute_rank(
                endpoint.id,
                &endpoint.domain,
                &endpoint.sub_domain,
                addr,
                dns_unresolved_endpoint(&endpoint.domain, resolved.contains(&endpoint.id.get())),
                &links,
            )
        })
        .collect();
    write(conn, &ranks).await
}

/// Every protocol's weight, keyed by protocol id — the one map that lets a
/// `ProfileStats`-only row be turned into a [`RankLink`] with its weight.
pub(crate) async fn protocol_weights(
    conn: &mut impl toasty::Executor,
) -> crate::Result<HashMap<ProtocolId, ConfigWeight>> {
    let protocols: Vec<Protocol> = Protocol::all().exec(conn).await?;
    Ok(protocols
        .iter()
        .map(|p| (p.id, weight_of_protocol(p)))
        .collect())
}

/// The endpoints that have at least one resolved address, as a set.
///
/// The ordering law's DNS band is the only thing that reads it, and both
/// readers (the wholesale backfill and the per-window refresh) need the same
/// answer: "is this endpoint's address set non-empty". One grouped scan for
/// the backfill, a correlated `EXISTS` for the window refresh.
async fn resolved_endpoint_ids(
    conn: &mut impl toasty::Executor,
) -> crate::Result<std::collections::HashSet<i64>> {
    let rows = toasty::sql::query("SELECT DISTINCT endpoint_id FROM endpoint_ip")
        .exec(conn)
        .await?;
    let mut out = std::collections::HashSet::new();
    for row in &rows {
        if let Value::Record(record) = row
            && let Some(Value::I64(id)) = record.fields.first()
        {
            out.insert(*id);
        }
    }
    Ok(out)
}

/// Backfill rank rows for endpoints that have links but no row yet.
pub(crate) async fn repair_missing(conn: &mut impl toasty::Executor) -> crate::Result<usize> {
    let rows = toasty::sql::query(
        "SELECT e.id FROM endpoints e WHERE \
         EXISTS (SELECT 1 FROM profile_stats p WHERE p.endpoint_id = e.id) \
         AND NOT EXISTS (SELECT 1 FROM endpoint_rank k WHERE k.endpoint_id = e.id)",
    )
    .exec(conn)
    .await?;
    let ids: Vec<EndpointId> = rows
        .iter()
        .filter_map(|row| match row {
            Value::Record(record) => record.fields.first().cloned(),
            _ => None,
        })
        .filter_map(|v| match v {
            Value::I64(id) => Some(EndpointId::new(id)),
            _ => None,
        })
        .collect();
    if ids.is_empty() {
        return Ok(0);
    }
    refresh(conn, &ids).await
}

/// Recompute the stored keys for `endpoint_ids` from their current links.
///
/// The single refresh entry point: every write path that can change a link
/// (patch flush, bulk upsert, error sweep) calls it for the endpoints it
/// touched, so a stored key is never older than the write that invalidated it.
pub(crate) async fn refresh(
    conn: &mut impl toasty::Executor,
    endpoint_ids: &[EndpointId],
) -> crate::Result<usize> {
    let mut ids: Vec<i64> = endpoint_ids.iter().map(|id| id.get()).collect();
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return Ok(0);
    }
    let id_list = ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");

    let endpoints = load_raw_endpoints(conn, &id_list).await?;
    // The weight comes from the link's PROTOCOL row, joined into the statement
    // that already reads every link for these endpoints: five small
    // discriminator columns cost one indexed lookup per link, where a separate
    // load would re-scan the whole protocols table on every flush window. The
    // deferred `config` JSON is NOT touched — nothing in the formula reads it.
    let mut links: HashMap<i64, Vec<RankLink>> = HashMap::new();
    let rows = toasty::sql::query(format!(
        "SELECT ps.endpoint_id, ps.protocol_id, ps.error_kind, ps.latency, ps.latency_delay, \
         ps.last_seen_at, ps.speed_bps, ps.traffic_total_up, ps.traffic_total_down, \
         ps.purge_reason, \
         pr.transport_type, pr.security_type, pr.security_sni, pr.security_fp \
         FROM profile_stats ps LEFT JOIN protocols pr ON pr.id = ps.protocol_id \
         WHERE ps.endpoint_id IN ({id_list})"
    ))
    .exec(conn)
    .await?;
    for row in &rows {
        let Value::Record(record) = row else { continue };
        let field = |i: usize| record.fields.get(i);
        let Some(endpoint_id) = field(0).and_then(as_i64) else {
            continue;
        };
        let protocol_id = field(1).and_then(as_i64).unwrap_or(0);
        links.entry(endpoint_id).or_default().push(RankLink {
            protocol_id,
            error_kind: field(2)
                .and_then(as_text)
                .and_then(|k| parse_error_kind(&k)),
            measured: field(3).and_then(as_text).and_then(|k| match k.as_str() {
                "real" => Some(true),
                "fast" => Some(false),
                _ => None,
            }),
            delay: field(4)
                .and_then(as_i64)
                .and_then(|d| i32::try_from(d).ok())
                .unwrap_or(0),
            seen_secs: field(5).and_then(as_i64).unwrap_or(0),
            speed: field(6).and_then(as_i64),
            traffic: field(7).and_then(as_i64).unwrap_or(0)
                + field(8).and_then(as_i64).unwrap_or(0),
            // A NULL column is a live link; any stored spelling is a verdict
            // (the value itself is the page's business, not the rank law's).
            purged: field(9).and_then(as_text).is_some(),
            // A link whose protocol row is absent (the join is LEFT) has no
            // known stack, so it sorts as "worst" — the same value an
            // un-refreshed endpoint carries, never a silent average.
            weight: weight_from_discriminators(
                field(10).and_then(as_text).as_deref(),
                field(11).and_then(as_text).as_deref(),
                field(12).and_then(as_text).as_deref(),
                field(13).and_then(as_text).as_deref(),
            ),
        });
    }

    let ranks: Vec<RankRow> = ids
        .iter()
        .filter_map(|id| {
            let endpoint = endpoints.get(id)?;
            let empty: Vec<RankLink> = Vec::new();
            let endpoint_links = links.get(id).unwrap_or(&empty);
            compute_rank(
                EndpointId::new(*id),
                &endpoint.domain,
                &endpoint.sub_domain,
                endpoint.addr.clone(),
                endpoint.dns_unresolved,
                endpoint_links,
            )
        })
        .collect();
    write(conn, &ranks).await
}

async fn load_raw_endpoints(
    conn: &mut impl toasty::Executor,
    id_list: &str,
) -> crate::Result<HashMap<i64, RawEndpoint>> {
    // The DNS band's input is "has a resolved address", which is a question
    // for `endpoint_ip`, not for the endpoint row — one correlated EXISTS in
    // the statement that already reads the ids, so the refresh stays a single
    // round trip.
    let rows = toasty::sql::query(format!(
        "SELECT e.id, e.domain, e.sub_domain, \
         (SELECT ip.ip_key FROM endpoint_ip ip WHERE ip.endpoint_id = e.id LIMIT 1), \
         EXISTS (SELECT 1 FROM endpoint_ip ip WHERE ip.endpoint_id = e.id) \
         FROM endpoints e WHERE e.id IN ({id_list})"
    ))
    .exec(conn)
    .await?;
    let mut out = HashMap::new();
    for row in &rows {
        let Value::Record(record) = row else { continue };
        let field = |i: usize| record.fields.get(i);
        let Some(id) = field(0).and_then(as_i64) else {
            continue;
        };
        let domain = field(1).and_then(as_text).unwrap_or_default();
        let sub_domain = field(2).and_then(as_text).unwrap_or_default();
        let addr = field(3).and_then(as_blob).unwrap_or_default();
        let has_address = field(4).and_then(as_i64).unwrap_or(0) != 0;
        out.insert(
            id,
            RawEndpoint {
                dns_unresolved: !domain.is_empty() && !has_address,
                domain,
                sub_domain,
                addr,
            },
        );
    }
    Ok(out)
}

const fn as_i64(value: &Value) -> Option<i64> {
    match value {
        Value::I64(n) => Some(*n),
        _ => None,
    }
}

/// A `x'…'` BLOB literal for the packed address key.
fn blob_lit(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(3 + bytes.len() * 2);
    out.push_str("x'");
    for b in bytes {
        use std::fmt::Write as _;
        let _ = write!(out, "{b:02x}");
    }
    out.push('\'');
    out
}

fn as_blob(value: &Value) -> Option<Vec<u8>> {
    match value {
        Value::Bytes(b) => Some(b.clone()),
        _ => None,
    }
}

fn as_text(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn parse_error_kind(text: &str) -> Option<ProfileErr> {
    match text {
        "real" => Some(ProfileErr::Real),
        "fast" => Some(ProfileErr::Fast),
        "name" => Some(ProfileErr::Name),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn link(protocol_id: i64, measured: Option<bool>, delay: i32, seen: i64) -> RankLink {
        RankLink {
            protocol_id,
            measured,
            delay,
            error_kind: None,
            seen_secs: seen,
            speed: None,
            traffic: 0,
            purged: false,
            weight: ZERO_WEIGHT,
        }
    }

    /// A weight with a known packing, so a test can reason about the negated
    /// term without depending on any particular cell value.
    const fn weight(security: u16) -> ConfigWeight {
        ConfigWeight {
            security,
            mimicry: 0,
            sec_cost: 0,
            transport_cost: 0,
        }
    }

    #[test]
    fn a_higher_weight_outranks_within_a_bin() {
        // The bin is the delay bucket and comes FIRST (db-rewamp D11), so the
        // weight decides only WITHIN a bin: two links in the same bin, the
        // heavier stack leads.
        let mut heavy = link(1, Some(true), 900, 10);
        heavy.weight = weight(10);
        let mut light = link(2, Some(true), 940, 10);
        light.weight = weight(2);
        assert_eq!(
            heavy.key(false).0,
            light.key(false).0,
            "same bin (≥1000? no: both <1000)"
        );
        assert!(
            heavy.key(false) < light.key(false),
            "the heavier stack leads inside one bin"
        );
        // The bin still dominates the weight: a slow REAL (worse bin) never
        // leads a fast REAL whatever its weight.
        let fast = link(3, Some(true), 40, 10);
        let mut slow_heavy = link(4, Some(true), 900, 10);
        slow_heavy.weight = weight(10);
        assert!(
            fast.key(false) < slow_heavy.key(false),
            "40ms bin 0 leads 900ms bin 4"
        );
        // …and a measured success never leads a failed link.
        let mut failed = link(4, Some(true), 900, 10);
        failed.error_kind = Some(ProfileErr::Real);
        assert!(
            fast.key(false) < failed.key(false),
            "success leads a failed link"
        );
    }

    #[test]
    fn equal_weights_fall_through_to_latency_recency_and_id() {
        let fast = link(1, Some(false), 30, 100);
        let slow = link(2, Some(false), 300, 100);
        assert!(
            fast.key(false) < slow.key(false),
            "weight ties, latency decides"
        );
        let newer = link(3, Some(false), 30, 200);
        assert!(
            newer.key(false) < fast.key(false),
            "weight and latency tie, the newer link leads"
        );
        let lower_id = link(0, Some(false), 30, 200);
        assert!(
            lower_id.key(false) < newer.key(false),
            "protocol id is the last tiebreak"
        );
    }

    #[test]
    fn the_stored_weight_is_the_representative_links_own() {
        let mut weak = link(1, Some(true), 10, 5);
        weak.weight = weight(2);
        let mut strong = link(2, Some(true), 10, 5);
        strong.weight = weight(10);
        let row = compute_rank(
            EndpointId::new(7),
            "h.example",
            "",
            Vec::new(),
            false,
            &[weak, strong],
        )
        .expect("rank");
        assert_eq!(
            row.weight,
            weight(10),
            "the stored weight must describe the SAME link the tier came from"
        );
    }

    #[test]
    fn a_missing_protocol_yields_the_zero_weight_not_a_mid_band() {
        assert_eq!(
            weight_from_discriminators(None, None, None, None),
            ZERO_WEIGHT,
            "an unknown stack must sort as worst, not as an unexamined average"
        );
        assert_eq!(
            weight_from_discriminators(Some("tcp"), Some("not-a-security"), None, None),
            ZERO_WEIGHT,
            "an unparseable spelling is a schema drift, not a guess"
        );
    }

    /// The transports whose DATABASE label differs from their wire spelling.
    /// `toasty`'s embed derive writes `snake_case` idents, so these two rows used
    /// to parse as `Err` on the refresh path and persist a ZERO weight while
    /// every typed path computed a real one — the stored page order then
    /// disagreed with the panel. Pinned here so the label set cannot drift.
    #[test]
    fn toasty_storage_labels_for_multiword_transports_parse() {
        for (label, wire) in [("http_upgrade", "httpupgrade"), ("x_http", "xhttp")] {
            assert_eq!(
                TransportType::from_db_label(label).map(TransportType::as_str),
                Some(wire),
                "{label} is what toasty writes"
            );
            assert_eq!(
                label.parse::<TransportType>().ok(),
                None,
                "FromStr is the WIRE parser and must keep rejecting the storage label"
            );
        }
    }

    /// The invariant the page actually depends on: the RAW-column read and the
    /// TYPED read must produce the SAME weight for every transport.
    ///
    /// This is the revert-proof. A test that merely checks "the `x_http` weight
    /// differs from the `tcp` weight" passes under the bug too, because a ZERO
    /// weight also differs — it proves nothing. Comparing against the typed
    /// path's own answer is the only form that goes red when the raw reader
    /// stops understanding a label.
    #[test]
    fn the_raw_column_read_agrees_with_the_typed_read_for_every_transport() {
        for (label, wire) in [
            ("tcp", TransportType::Tcp),
            ("ws", TransportType::Ws),
            ("grpc", TransportType::Grpc),
            ("http", TransportType::Http),
            ("quic", TransportType::Quic),
            ("kcp", TransportType::Kcp),
            ("http_upgrade", TransportType::HttpUpgrade),
            ("x_http", TransportType::XHttp),
        ] {
            for security in ["none", "tls", "reality"] {
                let typed = weight_of(wire, security.parse::<SecurityType>().unwrap(), None, None);
                let raw = weight_from_discriminators(Some(label), Some(security), None, None);
                assert_eq!(
                    raw, typed,
                    "{label}+{security}: the stored label must read as the same weight \
                     the typed protocol does, or the SQL page order and the panel \
                     disagree"
                );
                assert_ne!(
                    raw, ZERO_WEIGHT,
                    "{label}+{security} must not collapse to the zero pack"
                );
            }
        }
    }

    #[test]
    fn purged_links_sink_below_every_live_tier_including_dns() {
        let mut purged = link(1, Some(true), 5, 100);
        purged.purged = true;
        assert_eq!(purged.key(false).0, 16, "purged is its own band");
        assert_eq!(purged.key(true).0, 16, "and it outranks the DNS collapse");
        // Below a fast/real error band: a purged real-ok link must not lead.
        let fast_err = RankLink {
            error_kind: Some(ProfileErr::Fast),
            ..link(2, None, 0, 1)
        };
        assert!(fast_err.key(false) < purged.key(false));
    }

    #[test]
    fn newest_seen_is_the_live_only_maximum() {
        let mut purged = link(1, Some(false), 10, 9_000);
        purged.purged = true;
        let links = [link(2, None, 0, 100), purged];
        let rank = compute_rank(
            EndpointId::new(1),
            "h.example",
            "",
            Vec::new(),
            false,
            &links,
        )
        .expect("rank");
        assert_eq!(
            rank.rank.newest_seen, 100,
            "a purged link's recency does not keep the endpoint in the Active window"
        );
        assert_eq!(
            rank.rank.bin, 12,
            "the live untested link is the representative"
        );

        let all_purged = [purged];
        let rank = compute_rank(
            EndpointId::new(1),
            "h.example",
            "",
            Vec::new(),
            false,
            &all_purged,
        )
        .expect("rank");
        assert_eq!(
            rank.rank.newest_seen, NO_SEEN,
            "no live link -> below every window bound (Purgatory)"
        );
        assert_eq!(rank.rank.bin, 16);
    }

    #[test]
    fn a_pin_cannot_resurrect_a_purged_link_as_the_display_link() {
        let mut purged = link(10, Some(false), 10, 5);
        purged.purged = true;
        let links = [purged, link(11, Some(false), 90, 5)];
        assert_eq!(
            display_link_index(&links, Some(10)),
            Some(1),
            "the pinned purged link yields to the live one"
        );
        assert_eq!(
            display_link_index(&links, Some(11)),
            Some(1),
            "a live pin is honoured"
        );
        let all_purged = [purged, {
            let mut l = link(11, Some(false), 90, 5);
            l.purged = true;
            l
        }];
        assert_eq!(
            display_link_index(&all_purged, None),
            Some(0),
            "all purged: the best measured purged link still displays"
        );
    }
}
