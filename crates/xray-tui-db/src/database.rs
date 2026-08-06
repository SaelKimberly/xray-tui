use std::collections::HashMap;
use std::net::IpAddr;
use std::path::Path;
use std::time::Duration;

use jiff::Timestamp;
use toasty::stmt::IntoStatement;
use toasty_core::stmt::Value;

use crate::error::{DatabaseError, Result};
use crate::hash::stable_hash;
use crate::models_toasty::{
    DnsSetting, Endpoint, EndpointGroup, EndpointId, EndpointRow, Group, HostType, ProfileStats,
    Protocol, ProtocolId, RoutingRule, TrafficStats,
};
use crate::retry_on_busy;

// ── Database handle ─────────────────────────────────────────────────────

pub struct Database {
    db: toasty::Db,
}

// ── Constructors ────────────────────────────────────────────────────────

impl Database {
    /// Opens existing DB or creates fresh. Recovers from corruption by recreating.
    #[allow(
        clippy::significant_drop_tightening,
        reason = "driver is moved into try_open_db; clippy's drop suggestion is a false positive"
    )]
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        // Tag for databases created by this 7-table schema. toasty 0.9's
        // push_schema emits CREATE TABLE without IF NOT EXISTS, so it can
        // only run on a database that has no tables yet; the tag lets reopen
        // skip it. Any other tag is a pre-T8 9-table database (incompatible
        // with the typed models) and is recreated from scratch.
        const SCHEMA_VERSION: i64 = 5;

        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| DatabaseError::Generic("invalid db path".into()))?;

        // If file is empty (0 bytes), delete so toasty can create it fresh
        if Path::new(path_str).exists() && std::fs::metadata(path_str)?.len() == 0 {
            std::fs::remove_file(path_str)?;
        }

        let driver = toasty_driver_turso::Turso::file(path_str);
        let mut db = match Self::try_open_db(driver).await {
            Ok(db) => db,
            Err(e) => {
                // DB might be corrupted — log warning, delete, recreate
                tracing::warn!(error = %e, "DB open failed, attempting recovery by recreating");
                if Path::new(path_str).exists() {
                    std::fs::remove_file(path_str)?;
                }
                let driver = toasty_driver_turso::Turso::file(path_str);
                Self::try_open_db(driver).await?
            }
        };

        let mut conn = db.connection().await?;

        let rows = toasty::sql::query("PRAGMA user_version")
            .exec(&mut conn)
            .await?;
        let current_version = first_i64(&rows).unwrap_or(0);
        if current_version != SCHEMA_VERSION {
            match db.push_schema().await {
                Ok(()) => {
                    toasty::sql::query(format!("PRAGMA user_version = {SCHEMA_VERSION}"))
                        .exec(&mut conn)
                        .await?;
                }
                Err(e) => {
                    // Existing tables (pre-T8 schema or half-created DB):
                    // drop the file and rebuild with the 7-table schema.
                    tracing::warn!(
                        version = current_version,
                        error = %e,
                        "incompatible DB schema, recreating from scratch"
                    );
                    drop(conn);
                    if Path::new(path_str).exists() {
                        std::fs::remove_file(path_str)?;
                    }
                    let driver = toasty_driver_turso::Turso::file(path_str);
                    db = Self::try_open_db(driver).await?;
                    let mut fresh = db.connection().await?;
                    db.push_schema().await?;
                    toasty::sql::query(format!("PRAGMA user_version = {SCHEMA_VERSION}"))
                        .exec(&mut fresh)
                        .await?;
                    conn = fresh;
                }
            }
        }

        let _ = toasty::sql::query("PRAGMA journal_mode=WAL")
            .exec(&mut conn)
            .await?;
        toasty::sql::query("PRAGMA busy_timeout=5000")
            .exec(&mut conn)
            .await?;
        toasty::sql::query("PRAGMA foreign_keys=ON")
            .exec(&mut conn)
            .await?;

        Self::init_default_groups(&mut conn).await?;
        Ok(Self { db })
    }

    /// Open a toasty DB by constructing builder. Separate for recovery logic.
    async fn try_open_db(driver: toasty_driver_turso::Turso) -> Result<toasty::Db> {
        let db = toasty::Db::builder()
            .models(toasty::models!(
                Endpoint,
                Protocol,
                ProfileStats,
                EndpointGroup,
                Group,
                RoutingRule,
                DnsSetting
            ))
            .build(driver)
            .await?;
        Ok(db)
    }

    /// Acquire a pooled connection with the `SQLite` busy-wait configured.
    /// `PRAGMA busy_timeout` is per-connection, so the pragma set in `open()`
    /// never reaches pool-created connections. Without it, concurrent
    /// writers (enrichment resolutions, ping-buffer flushes) fail instantly
    /// with "database is locked" instead of queuing behind the lock holder —
    /// this was the 850-line "database is locked" class in the dumps.
    async fn conn(&self) -> Result<toasty::Connection> {
        let mut conn = self.db.connection().await?;
        toasty::sql::query("PRAGMA busy_timeout=5000")
            .exec(&mut conn)
            .await?;
        Ok(conn)
    }

    /// Public pooled-connection accessor for callers building typed
    /// create/update statements (Task 10 writes; integration tests).
    pub async fn connection(&self) -> Result<toasty::Connection> {
        self.conn().await
    }

    pub async fn in_memory() -> Result<Self> {
        let driver = toasty_driver_turso::Turso::in_memory();
        let db = toasty::Db::builder()
            .models(toasty::models!(
                Endpoint,
                Protocol,
                ProfileStats,
                EndpointGroup,
                Group,
                RoutingRule,
                DnsSetting
            ))
            .build(driver)
            .await?;

        let mut conn = db.connection().await?;
        db.push_schema().await?;

        toasty::sql::query("PRAGMA busy_timeout=5000")
            .exec(&mut conn)
            .await?;
        toasty::sql::query("PRAGMA foreign_keys=ON")
            .exec(&mut conn)
            .await?;

        Self::init_default_groups(&mut conn).await?;
        Ok(Self { db })
    }

    async fn init_default_groups(conn: &mut impl toasty::Executor) -> Result<()> {
        let count = Group::all().count().exec(conn).await?;
        if count == 0 {
            Group::create()
                .id(uuid::Uuid::new_v4().to_string())
                .name(Some("Default".to_string()))
                .enabled(true)
                .sort_order(Some(0))
                .into_statement()
                .exec(conn)
                .await?;
        }
        Ok(())
    }
}

/// Extract the first INTEGER column of the first row (used for PRAGMA reads).
fn first_i64(rows: &[Value]) -> Option<i64> {
    rows.first().and_then(|v| {
        if let Value::Record(fields) = v {
            fields.first().and_then(|f| match f {
                Value::I64(n) => Some(*n),
                _ => None,
            })
        } else {
            None
        }
    })
}

// ── Read queries (public API) ───────────────────────────────────────────

impl Database {
    /// Active endpoints: at least one link with `last_seen_at >= active_threshold`.
    ///
    /// Assembled with a batched relation load — the endpoint query plus ONE
    /// `ProfileStats` query carrying the per-link `protocol`/`endpoint`
    /// relations (no N+1). Endpoints ordered by id.
    pub async fn get_active_endpoints(
        &self,
        active_threshold: Timestamp,
    ) -> Result<Vec<EndpointRow>> {
        let mut conn = self.conn().await?;
        let endpoints: Vec<Endpoint> = Endpoint::filter(
            Endpoint::fields()
                .links()
                .any(ProfileStats::fields().last_seen_at().ge(active_threshold)),
        )
        .exec(&mut conn)
        .await?;
        self.load_endpoint_rows(endpoints, &mut conn).await
    }

    /// Active endpoints filtered by group membership (endpoint has an
    /// `endpoint_groups` link for `group_id` and is active).
    pub async fn get_active_endpoints_by_group(
        &self,
        group_id: &str,
        active_threshold: Timestamp,
    ) -> Result<Vec<EndpointRow>> {
        let mut conn = self.conn().await?;
        let endpoints: Vec<Endpoint> = Endpoint::filter(
            Endpoint::fields()
                .links()
                .any(ProfileStats::fields().last_seen_at().ge(active_threshold))
                .and(
                    Endpoint::fields()
                        .group_links()
                        .any(EndpointGroup::fields().group_id().eq(group_id)),
                ),
        )
        .exec(&mut conn)
        .await?;
        self.load_endpoint_rows(endpoints, &mut conn).await
    }

    /// Stale endpoints: `max(last_seen_at)` < `active_threshold` AND
    /// >= `stale_threshold`.
    ///
    /// Fetched with the wide predicate (at least one link as old as
    /// `stale_threshold`); the max-window is checked in memory over the
    /// loaded links (pages are bounded).
    pub async fn get_stale_endpoints(
        &self,
        active_threshold: Timestamp,
        stale_threshold: Timestamp,
    ) -> Result<Vec<EndpointRow>> {
        let mut conn = self.conn().await?;
        let endpoints: Vec<Endpoint> = Endpoint::filter(
            Endpoint::fields()
                .links()
                .any(ProfileStats::fields().last_seen_at().ge(stale_threshold)),
        )
        .exec(&mut conn)
        .await?;
        let mut rows = self.load_endpoint_rows(endpoints, &mut conn).await?;
        rows.retain(|r| {
            r.links
                .iter()
                .map(|l| l.last_seen_at)
                .max()
                .is_some_and(|max| max >= stale_threshold && max < active_threshold)
        });
        Ok(rows)
    }

    pub async fn get_stale_count(
        &self,
        active_threshold: Timestamp,
        stale_threshold: Timestamp,
    ) -> Result<usize> {
        Ok(self
            .get_stale_endpoints(active_threshold, stale_threshold)
            .await?
            .len())
    }

    /// Single endpoint by id with all links and protocols.
    pub async fn get_endpoint(&self, id: EndpointId) -> Result<Option<EndpointRow>> {
        let mut conn = self.conn().await?;
        let endpoint = Endpoint::filter_by_id(id).first().exec(&mut conn).await?;
        let Some(endpoint) = endpoint else {
            return Ok(None);
        };
        let mut rows = self.load_endpoint_rows(vec![endpoint], &mut conn).await?;
        Ok(rows.pop())
    }

    /// Look up endpoint row by link protocol id (p.id not e.id).
    pub async fn get_endpoint_by_protocol_id(
        &self,
        protocol_id: ProtocolId,
    ) -> Result<Option<EndpointRow>> {
        let mut conn = self.conn().await?;
        let link = ProfileStats::filter(ProfileStats::fields().protocol_id().eq(protocol_id))
            .first()
            .exec(&mut conn)
            .await?;
        let Some(link) = link else {
            return Ok(None);
        };
        let endpoint = Endpoint::filter_by_id(link.endpoint_id)
            .first()
            .exec(&mut conn)
            .await?;
        let Some(endpoint) = endpoint else {
            return Ok(None);
        };
        let mut rows = self.load_endpoint_rows(vec![endpoint], &mut conn).await?;
        Ok(rows.pop())
    }

    /// Child endpoints of a `DnsName` parent (resolved IP endpoints). Ordered
    /// by id.
    pub async fn endpoints_by_parent(&self, parent_id: EndpointId) -> Result<Vec<Endpoint>> {
        let mut conn = self.conn().await?;
        let mut endpoints: Vec<Endpoint> =
            Endpoint::filter(Endpoint::fields().parent_id().eq(parent_id))
                .exec(&mut conn)
                .await?;
        endpoints.sort_by_key(|e| e.id);
        Ok(endpoints)
    }

    pub async fn get_all_groups(&self) -> Result<Vec<Group>> {
        let mut conn = self.conn().await?;
        let groups: Vec<Group> = Group::all()
            .order_by(Group::fields().sort_order().asc())
            .exec(&mut conn)
            .await?;
        Ok(groups)
    }

    /// Groups enabled with a non-empty url whose refresh window has elapsed:
    /// `last_refreshed IS NULL` or `last_refreshed + refresh_interval minutes
    /// < now` (`refresh_interval` defaults to 1440 minutes, matching the old
    /// subscription update interval). Ordered by sort_order.
    pub async fn get_groups_due_update(&self) -> Result<Vec<Group>> {
        let mut conn = self.conn().await?;
        let candidates: Vec<Group> = Group::filter(Group::fields().enabled().eq(true))
            .filter(Group::fields().url().is_some())
            .order_by(Group::fields().sort_order().asc())
            .exec(&mut conn)
            .await?;
        let now = Timestamp::now();
        let mut due = Vec::new();
        for group in candidates {
            if group.url.as_deref().is_none_or(str::is_empty) {
                continue;
            }
            match group.last_refreshed {
                None => due.push(group),
                Some(last) => {
                    let interval =
                        jiff::Span::new().minutes(group.refresh_interval.unwrap_or(1440));
                    if last.checked_add(interval).is_ok_and(|t| t < now) {
                        due.push(group);
                    }
                }
            }
        }
        Ok(due)
    }

    pub async fn get_all_routing_rules(&self) -> Result<Vec<RoutingRule>> {
        let mut conn = self.conn().await?;
        let rules: Vec<RoutingRule> = RoutingRule::all()
            .order_by(RoutingRule::fields().sort_order().asc())
            .exec(&mut conn)
            .await?;
        Ok(rules)
    }

    /// The (single) DNS settings row, if any.
    pub async fn get_dns_settings(&self) -> Result<Option<DnsSetting>> {
        let mut conn = self.conn().await?;
        let settings: Vec<DnsSetting> = DnsSetting::all().exec(&mut conn).await?;
        Ok(settings.into_iter().next())
    }

    /// Assemble [`EndpointRow`]s for a page of endpoints with ONE batched
    /// query for the per-link `protocol`/`endpoint` relations (no N+1):
    ///
    /// 1. `Endpoint::filter(...)` — the page, ordered by id.
    /// 2. `ProfileStats::filter(endpoint_id IN page_ids)` with
    ///    `.include(profile_stats.protocol())` and
    ///    `.include(profile_stats.endpoint())` — every link of the page with
    ///    its relations preloaded by the engine in a single statement.
    ///
    /// Links are grouped per endpoint and sorted by test priority
    /// (`sort_links_by_test_priority`); the `protocols` map is built from the
    /// included relations. `dns_unresolved` is endpoint-level: `Dns` host
    /// with no cached `resolved_as` sinks all its links to tier 5.
    async fn load_endpoint_rows(
        &self,
        endpoints: Vec<Endpoint>,
        conn: &mut toasty::Connection,
    ) -> Result<Vec<EndpointRow>> {
        if endpoints.is_empty() {
            return Ok(Vec::new());
        }
        let ids: Vec<EndpointId> = endpoints.iter().map(|e| e.id).collect();
        // Newtype embed paths expose only `.eq()`; build the IN filter through
        // `stmt::in_list` (the field struct implements `IntoExpr<EndpointId>`).
        let links: Vec<ProfileStats> = ProfileStats::filter(toasty::stmt::in_list(
            ProfileStats::fields().endpoint_id(),
            ids,
        ))
        .include(ProfileStats::fields().protocol())
        .include(ProfileStats::fields().endpoint())
        .exec(conn)
        .await?;

        let mut by_endpoint: HashMap<EndpointId, Vec<ProfileStats>> = HashMap::new();
        for link in links {
            by_endpoint.entry(link.endpoint_id).or_default().push(link);
        }

        let mut rows = Vec::with_capacity(endpoints.len());
        for endpoint in endpoints {
            let links = by_endpoint.remove(&endpoint.id).unwrap_or_default();
            let protocols = links
                .iter()
                .filter_map(|l| l.protocol.get().as_ref().map(|p| (p.id, p.clone())))
                .collect();
            let dns_unresolved =
                endpoint.host_type == HostType::Dns && endpoint.resolved_as.is_empty();
            let mut row = EndpointRow {
                endpoint,
                links,
                protocols,
                selected_protocol: 0,
                expanded: false,
            };
            row.sort_links_by_test_priority(dns_unresolved);
            rows.push(row);
        }
        // Deterministic page order (the newtype id path cannot be ordered in SQL).
        rows.sort_by_key(|r| r.endpoint.id);
        Ok(rows)
    }
}

// ── Write methods (public API) ─────────────────────────────────────────

impl Database {
    // ── Upserts (idempotent, natural-key dedup) ──────────────────────────

    /// Insert or update one endpoint by id.
    ///
    /// Replaces the endpoint's identity fields (`host`, `host_type`, `port`,
    /// `ports`, `parent_id`, `last_source`). The DNS-resolution cache
    /// (`resolved_as` / `resolved_at`) and the manual protocol override are
    /// owned by their dedicated writes ([`Self::update_endpoint_resolution`],
    /// [`Self::set_manual_override`]) and are preserved on update — this
    /// matches the old subscription path, which never clobbered an existing
    /// endpoint's resolution state (INSERT OR IGNORE).
    pub async fn upsert_endpoint(&self, e: &Endpoint) -> Result<()> {
        let mut conn = self.conn().await?;
        Endpoint::upsert_by_id(e.id)
            .host(e.host.clone())
            .host_type(e.host_type)
            .port(e.port)
            .ports(e.ports.clone())
            .parent_id(e.parent_id)
            .last_source(e.last_source.clone())
            .on_create(|create| create.resolved_as(Vec::<String>::new()))
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    /// Insert or update one protocol row by id.
    pub async fn upsert_protocol(&self, p: &Protocol) -> Result<()> {
        let mut conn = self.conn().await?;
        Protocol::upsert_by_id(p.id)
            .sig(p.sig)
            .cred_hash(p.cred_hash)
            .proto_kind(p.proto_kind)
            .transport(p.transport.clone())
            .security(p.security.clone())
            .config(p.config.get().0.clone())
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    /// Insert or update one per-pair link row by its composite key
    /// `(protocol_id, endpoint_id)`.
    ///
    /// Replaces the link's source- and result-state fields (`core_type`,
    /// `config_type`, `last_seen_at`, `latency`, `speed_bps`, `error`,
    /// `traffic`). Scheduler state (`task_id`, `task_queue`) and the activity
    /// timestamp (`last_used_at`) are owned by
    /// [`Self::update_scheduler_state`] / [`Self::update_last_used`] and are
    /// preserved on update; new rows start with an empty queue.
    pub async fn upsert_link(&self, s: &ProfileStats) -> Result<()> {
        let mut conn = self.conn().await?;
        ProfileStats::upsert_by_protocol_id_and_endpoint_id(s.protocol_id, s.endpoint_id)
            .core_type(s.core_type)
            .config_type(s.config_type)
            .last_seen_at(s.last_seen_at)
            .latency(s.latency.clone())
            .speed_bps(s.speed_bps)
            .error(s.error.clone())
            .traffic(s.traffic)
            .on_create(|create| create.task_queue(Vec::<u16>::new()))
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    /// Insert or update one endpoint↔group link by its composite key
    /// `(endpoint_id, group_id)`.
    pub async fn upsert_endpoint_group_link(&self, eg: &EndpointGroup) -> Result<()> {
        let mut conn = self.conn().await?;
        EndpointGroup::upsert_by_endpoint_id_and_group_id(eg.endpoint_id, eg.group_id.clone())
            .last_seen_at(eg.last_seen_at)
            .sort_order(eg.sort_order)
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    /// Insert or update one group by id (replaces `insert_group` +
    /// `update_group`).
    pub async fn upsert_group(&self, g: &Group) -> Result<()> {
        let mut conn = self.conn().await?;
        Group::upsert_by_id(g.id.clone())
            .name(g.name.clone())
            .url(g.url.clone())
            .enabled(g.enabled)
            .user_agent(g.user_agent.clone())
            .convert_target(g.convert_target)
            .core_type(g.core_type)
            .sort_order(g.sort_order)
            .last_refreshed(g.last_refreshed)
            .status(g.status)
            .error_message(g.error_message.clone())
            .refresh_interval(g.refresh_interval)
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    // ── Activity ─────────────────────────────────────────────────────────

    /// Record active use of a link: sets `last_used_at` AND `last_seen_at`
    /// to `ts`, so active use keeps a profile out of Stale/purge (old
    /// `update_last_used` semantics).
    pub async fn update_last_used(
        &self,
        protocol_id: ProtocolId,
        endpoint_id: EndpointId,
        ts: Timestamp,
    ) -> Result<()> {
        let mut conn = self.conn().await?;
        ProfileStats::filter_by_protocol_id_and_endpoint_id(protocol_id, endpoint_id)
            .update()
            .last_used_at(Some(ts))
            .last_seen_at(ts)
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    /// Persist the DNS resolution of an endpoint host: `resolved_as = ips`,
    /// `resolved_at = at`. Survives launches so the TUI does not re-resolve
    /// DNS hosts on startup. Retried on write contention (the enrichment
    /// pipeline resolves many endpoints concurrently), as the old code did.
    pub async fn update_endpoint_resolution(
        &self,
        endpoint_id: EndpointId,
        ips: Vec<String>,
        at: Timestamp,
    ) -> Result<()> {
        let db = self;
        retry_on_busy(
            move || {
                let ips = ips.clone();
                async move {
                    let mut conn = db.conn().await?;
                    Endpoint::filter_by_id(endpoint_id)
                        .update()
                        .resolved_as(ips)
                        .resolved_at(Some(at))
                        .exec(&mut conn)
                        .await?;
                    Ok(())
                }
            },
            5,
        )
        .await
    }

    /// Refresh the resolved-IP children of a DNS endpoint: insert-or-ignore
    /// one child `Endpoint` per IP (host = IP string, `host_type` from the
    /// address family, port 443, `parent_id` = parent), then delete children
    /// whose IP is no longer in `ips` (old `resolve_endpoint_dns` /
    /// `upsert_resolved_ips` behavior, in one transaction).
    ///
    /// Child id is `stable_hash(ip, 0)` (deterministic across refreshes), so
    /// re-resolving with a still-present IP keeps the existing child row —
    /// including any links it has accumulated.
    pub async fn upsert_resolved_ip_children(
        &self,
        parent_id: EndpointId,
        ips: &[IpAddr],
    ) -> Result<()> {
        let mut conn = self.conn().await?;
        let mut tx = conn.transaction().await?;

        for ip in ips {
            let id = EndpointId::new(stable_hash(ip.to_string(), 0i64));
            let host_type = match ip {
                IpAddr::V4(_) => HostType::Ipv4,
                IpAddr::V6(_) => HostType::Ipv6,
            };
            Endpoint::upsert_by_id(id)
                .host(ip.to_string())
                .host_type(host_type)
                .port(443)
                .ports(Vec::<u16>::new())
                .parent_id(Some(parent_id))
                .resolved_as(Vec::<String>::new())
                .or_ignore()
                .exec(&mut tx)
                .await?;
        }

        // Remove children whose IP is no longer in the resolution set.
        let children: Vec<Endpoint> =
            Endpoint::filter(Endpoint::fields().parent_id().eq(parent_id))
                .exec(&mut tx)
                .await?;
        let keep: Vec<String> = ips.iter().map(ToString::to_string).collect();
        for child in children {
            if !keep.contains(&child.host) {
                child.delete().exec(&mut tx).await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    /// Set or clear (`None`) the manual protocol override of an endpoint —
    /// the old `set_protocol_override` + `clear_protocol_override` merged.
    pub async fn set_manual_override(
        &self,
        endpoint_id: EndpointId,
        protocol_id: Option<ProtocolId>,
    ) -> Result<()> {
        let mut conn = self.conn().await?;
        Endpoint::filter_by_id(endpoint_id)
            .update()
            .manual_protocol_override(protocol_id)
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    // ── Purge / delete ───────────────────────────────────────────────────

    /// Purge endpoints where EVERY link's `last_seen_at < cutoff` — the old
    /// `COALESCE(MAX(p.last_seen_at), 0) < cutoff` predicate, including
    /// linkless endpoints (`links().all(...)` is vacuously true for them).
    /// Returns the number of deleted endpoints.
    ///
    /// Cascade, in one transaction: the endpoints' `endpoint_groups` links,
    /// their `profile_stats` links, then the endpoints themselves, then
    /// orphan `protocol` rows (those left with zero links).
    pub async fn purge_expired(&self, cutoff: Timestamp) -> Result<usize> {
        let mut conn = self.conn().await?;
        let mut tx = conn.transaction().await?;

        let expired: Vec<Endpoint> = Endpoint::filter(
            Endpoint::fields()
                .links()
                .all(ProfileStats::fields().last_seen_at().lt(cutoff)),
        )
        .exec(&mut tx)
        .await?;
        let count = expired.len();

        if count > 0 {
            let ids: Vec<EndpointId> = expired.iter().map(|e| e.id).collect();
            EndpointGroup::filter(toasty::stmt::in_list(
                EndpointGroup::fields().endpoint_id(),
                ids.clone(),
            ))
            .delete()
            .exec(&mut tx)
            .await?;
            ProfileStats::filter(toasty::stmt::in_list(
                ProfileStats::fields().endpoint_id(),
                ids.clone(),
            ))
            .delete()
            .exec(&mut tx)
            .await?;
            Endpoint::filter(toasty::stmt::in_list(Endpoint::fields().id(), ids))
                .delete()
                .exec(&mut tx)
                .await?;
            Self::purge_orphan_protocols(&mut tx).await?;
        }

        tx.commit().await?;
        Ok(count)
    }

    /// Delete an endpoint and cascade: its `profile_stats` links, its
    /// `endpoint_groups` links, the endpoint row, then orphan `protocol`
    /// rows (those whose last link just died). One transaction.
    pub async fn delete_endpoint(&self, endpoint_id: EndpointId) -> Result<()> {
        let mut conn = self.conn().await?;
        let mut tx = conn.transaction().await?;

        EndpointGroup::filter(EndpointGroup::fields().endpoint_id().eq(endpoint_id))
            .delete()
            .exec(&mut tx)
            .await?;
        ProfileStats::filter(ProfileStats::fields().endpoint_id().eq(endpoint_id))
            .delete()
            .exec(&mut tx)
            .await?;
        Endpoint::filter_by_id(endpoint_id)
            .delete()
            .exec(&mut tx)
            .await?;
        Self::purge_orphan_protocols(&mut tx).await?;

        tx.commit().await?;
        Ok(())
    }

    /// Remove all endpoint↔group links for `group_id` (the old `clear_group`),
    /// returning the number of links removed. Endpoints and their links stay.
    pub async fn clear_group_endpoints(&self, group_id: &str) -> Result<usize> {
        let mut conn = self.conn().await?;
        let rows: Vec<EndpointGroup> =
            EndpointGroup::filter(EndpointGroup::fields().group_id().eq(group_id))
                .exec(&mut conn)
                .await?;
        let count = rows.len();
        EndpointGroup::filter(EndpointGroup::fields().group_id().eq(group_id))
            .delete()
            .exec(&mut conn)
            .await?;
        Ok(count)
    }

    /// Delete a group: its `endpoint_groups` links, then the group row.
    /// One transaction.
    ///
    /// Note: the old `delete_group` also purged endpoints of this group that
    /// no longer belonged to ANY group. The typed model keeps group-less
    /// endpoints (the All view shows them, and `clear_group_endpoints`
    /// unlinks without deleting) — endpoint cleanup is left to
    /// [`Self::purge_expired`] by staleness, so deleting a group never
    /// silently destroys endpoints.
    pub async fn delete_group(&self, group_id: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let mut tx = conn.transaction().await?;

        EndpointGroup::filter(EndpointGroup::fields().group_id().eq(group_id))
            .delete()
            .exec(&mut tx)
            .await?;
        Group::filter_by_id(group_id.to_string())
            .delete()
            .exec(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    // ── Stats / restore ─────────────────────────────────────────────────

    /// Zero traffic (`today_up/down`, `total_up/down`) and clear
    /// `latency`, `speed_bps`, and `error` on EVERY `profile_stats` row
    /// (query-based update, one statement; the old `clear_all_stats` wiped
    /// server stats + extensions' delay/speed).
    pub async fn clear_all_stats(&self) -> Result<()> {
        let mut conn = self.conn().await?;
        ProfileStats::all()
            .update()
            .traffic(TrafficStats {
                today_up: 0,
                today_down: 0,
                total_up: 0,
                total_down: 0,
            })
            .latency(None)
            .error(None)
            .speed_bps(None)
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    /// Restore a stale endpoint by setting `last_seen_at = now` on all its
    /// links (old `restore_endpoint`).
    pub async fn restore_endpoint(&self, endpoint_id: EndpointId) -> Result<()> {
        let now = Timestamp::now();
        let mut conn = self.conn().await?;
        ProfileStats::filter(ProfileStats::fields().endpoint_id().eq(endpoint_id))
            .update()
            .last_seen_at(now)
            .exec(&mut conn)
            .await?;
        Ok(())
    }

    // ── Scheduler (OCC) ─────────────────────────────────────────────────

    /// Read-modify-write the scheduler state of one link: set `task_id` and
    /// REPLACE the whole `task_queue` vector.
    ///
    /// The mutation is optimistic-concurrency guarded: the link is loaded,
    /// the write is applied as a `#[version]`-checked instance update, and a
    /// `condition_failed` conflict (another writer raced us between load and
    /// update) triggers a reload + retry, bounded to 5 attempts with a small
    /// sleep. `SQLite` write contention is retried via [`retry_on_busy`].
    /// Query-based updates on `ProfileStats` (which bump `version` without
    /// checking) are fine here — the OCC check matters for this
    /// read-modify-write racing another writer.
    pub async fn update_scheduler_state(
        &self,
        protocol_id: ProtocolId,
        endpoint_id: EndpointId,
        task_id: Option<u16>,
        queue: &[u16],
    ) -> Result<()> {
        const MAX_SCHEDULER_ATTEMPTS: usize = 5;

        let db = self;
        retry_on_busy(
            move || async move {
                for attempt in 0..MAX_SCHEDULER_ATTEMPTS {
                    let mut conn = db.conn().await?;
                    let Some(mut link) = ProfileStats::filter_by_protocol_id_and_endpoint_id(
                        protocol_id,
                        endpoint_id,
                    )
                    .first()
                    .exec(&mut conn)
                    .await?
                    else {
                        return Err(DatabaseError::Generic(format!(
                            "scheduler state: no profile_stats row for protocol_id={} endpoint_id={}",
                            protocol_id.get(),
                            endpoint_id.get(),
                        )));
                    };

                    match toasty::update!(link {
                        task_id,
                        task_queue: queue.to_vec(),
                    })
                    .exec(&mut conn)
                    .await
                    {
                        Ok(()) => return Ok(()),
                        Err(err) if err.is_condition_failed() => {
                            // Another writer won the race; reload and retry.
                            if attempt + 1 < MAX_SCHEDULER_ATTEMPTS {
                                tokio::time::sleep(Duration::from_millis(5)).await;
                            }
                        }
                        Err(err) => return Err(err.into()),
                    }
                }
                Err(DatabaseError::Generic(
                    "scheduler state: OCC conflict retries exhausted".into(),
                ))
            },
            5,
        )
        .await
    }
}

impl Database {
    /// Delete `Protocol` rows with no remaining `profile_stats` links — the
    /// shared protocol table's orphan cleanup (the old per-row model had no
    /// shared table to clean). Runs inside the caller's transaction.
    ///
    /// `links().all(FALSE)` is vacuously true exactly when the links
    /// collection is empty, lowering to `id NOT IN (SELECT protocol_id FROM
    /// profile_stats)`. Returns the number of protocols deleted.
    async fn purge_orphan_protocols(tx: &mut dyn toasty::Executor) -> Result<usize> {
        let never: toasty::stmt::Expr<bool> =
            toasty::stmt::Expr::from_untyped(toasty_core::stmt::Expr::FALSE);
        let orphans: Vec<Protocol> = Protocol::filter(Protocol::fields().links().all(never))
            .exec(tx)
            .await?;
        let count = orphans.len();
        for protocol in orphans {
            protocol.delete().exec(tx).await?;
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models_toasty::{ConfigType, Latency, Security, TrafficStats, Transport};
    use toasty::{Deferred, Json};
    use xray_tui_proto::proto_spec::common::TransportConfig;
    use xray_tui_proto::proto_spec::{
        CoreType, ProtocolConfig, ProtocolKind, SecurityConfig, SecurityType, TransportType,
        VlessConfig,
    };

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

    #[tokio::test]
    async fn active_and_stale_windows() {
        let db = Database::in_memory().await.expect("in-memory db");
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

        // Active: only the recent endpoint.
        let active = db
            .get_active_endpoints(ts(now - 3_600))
            .await
            .expect("active");
        let active_ids: Vec<i64> = active.iter().map(|r| r.endpoint.id.get()).collect();
        assert_eq!(active_ids, vec![1]);

        // Stale: only the old endpoint.
        let stale = db
            .get_stale_endpoints(ts(now - 3_600), ts(now - 7_200))
            .await
            .expect("stale");
        let stale_ids: Vec<i64> = stale.iter().map(|r| r.endpoint.id.get()).collect();
        assert_eq!(stale_ids, vec![2]);

        // Count matches the stale view.
        let count = db
            .get_stale_count(ts(now - 3_600), ts(now - 7_200))
            .await
            .expect("count");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn rows_are_assembled_with_links_and_protocols() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "1.2.3.4", HostType::Ipv4, 443, 10).await;
        seed_link(&mut conn, 1, 1002, 20).await;

        let rows = db.get_active_endpoints(ts(0)).await.expect("load");
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.endpoint.id, EndpointId::new(1));
        assert_eq!(row.links.len(), 2, "both links present");
        assert_eq!(row.protocols.len(), 2, "protocols map built from links");
        assert!(
            row.protocols.contains_key(&ProtocolId::new(1001)),
            "protocol 1001 included"
        );
        assert!(
            row.protocols.contains_key(&ProtocolId::new(1002)),
            "protocol 1002 included"
        );
        // Newest link first (untested tier, recency order).
        assert_eq!(row.links[0].protocol_id, ProtocolId::new(1002));
        assert_eq!(row.links[1].protocol_id, ProtocolId::new(1001));

        // active_protocol resolves through the map.
        let (link, proto) = row.active_protocol().expect("active protocol");
        assert_eq!(link.protocol_id, ProtocolId::new(1002));
        assert_eq!(proto.proto_kind, ProtocolKind::Vless);
    }

    #[tokio::test]
    async fn group_filter_and_links() {
        let db = Database::in_memory().await.expect("in-memory db");
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
        assert_eq!(from_b.len(), 1);
        assert!(from_c.is_empty(), "unlinked group matches nothing");
        assert_eq!(from_a[0].endpoint.id, EndpointId::new(1));
    }

    #[tokio::test]
    async fn get_endpoint_by_id_and_protocol() {
        let db = Database::in_memory().await.expect("in-memory db");
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
        let db = Database::in_memory().await.expect("in-memory db");
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
        assert!(
            db.endpoints_by_parent(EndpointId::new(999))
                .await
                .expect("none")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn dns_unresolved_sinks_links_to_bottom() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");

        toasty::create!(Endpoint {
            id: EndpointId::new(1),
            host: "unresolved.example".to_string(),
            host_type: HostType::Dns,
            port: 443,
            ports: Vec::<u16>::new(),
            resolved_as: Vec::<String>::new(), // no cached resolution -> unresolved
        })
        .exec(&mut conn)
        .await
        .expect("endpoint");

        for (pid, last_seen) in [(1001, 1), (1002, 2)] {
            toasty::create!(Protocol {
                id: ProtocolId::new(pid),
                sig: pid,
                cred_hash: 0,
                proto_kind: ProtocolKind::Vless,
                transport: tcp_transport(),
                security: no_security(),
                config: Deferred::from(Json(vless_config())),
            })
            .exec(&mut conn)
            .await
            .expect("protocol");
            toasty::create!(ProfileStats {
                protocol_id: ProtocolId::new(pid),
                endpoint_id: EndpointId::new(1),
                core_type: CoreType::Xray,
                config_type: ConfigType::ShareUrl,
                last_seen_at: ts(last_seen),
                latency: Some(Latency::Real {
                    delay: 10,
                    ip: None
                }),
                task_queue: Vec::<u16>::new(),
                traffic: zero_traffic(),
            })
            .exec(&mut conn)
            .await
            .expect("link");
        }

        let rows = db.get_active_endpoints(ts(0)).await.expect("load");
        let row = &rows[0];
        // Both links sink to tier 5; recency decides.
        assert_eq!(row.links[0].protocol_id, ProtocolId::new(1002));
        assert_eq!(row.links[1].protocol_id, ProtocolId::new(1001));
        assert_eq!(row.best_test_priority_key(true).unwrap().0, 5);
    }

    #[tokio::test]
    async fn open_reopen_preserves_data() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("reopen.db");

        let db = Database::open(&path).await.expect("open");
        let mut conn = db.connection().await.expect("connection");
        toasty::create!(Endpoint {
            id: EndpointId::new(77),
            host: "1.1.1.1".to_string(),
            host_type: HostType::Ipv4,
            port: 443,
            ports: Vec::<u16>::new(),
            resolved_as: Vec::<String>::new(),
        })
        .exec(&mut conn)
        .await
        .expect("create endpoint");
        drop(conn);
        drop(db);

        // Reopen must NOT wipe: the schema tag skips push_schema.
        let db2 = Database::open(&path).await.expect("reopen");
        let mut conn = db2.connection().await.expect("connection");
        let endpoint = Endpoint::filter_by_id(EndpointId::new(77))
            .first()
            .exec(&mut conn)
            .await
            .expect("read");
        assert_eq!(endpoint.expect("endpoint").host, "1.1.1.1");
    }

    #[tokio::test]
    async fn open_recreates_incompatible_schema() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("pre8.db");

        // Build a database with a pre-T8 `endpoints` table (old 9-table
        // shape) — push_schema cannot run on it.
        {
            let driver = toasty_driver_turso::Turso::file(&path);
            let db = toasty::Db::builder()
                .models(toasty::models!(ScratchOnly))
                .build(driver)
                .await
                .expect("build db");
            db.push_schema().await.expect("push schema");
            let mut conn = db.connection().await.expect("connection");
            toasty::sql::statement("CREATE TABLE endpoints (id INTEGER PRIMARY KEY, host TEXT)")
                .exec(&mut conn)
                .await
                .expect("old endpoints table");
        }

        let db = Database::open(&path).await.expect("open recreates schema");
        let mut conn = db.connection().await.expect("connection");
        let rows = toasty::sql::query(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'profile_stats'",
        )
        .exec(&mut conn)
        .await
        .expect("check new table");
        assert_eq!(
            first_i64(&rows).expect("count"),
            1,
            "open() must recreate with the 7-table schema"
        );
    }

    #[derive(Debug, toasty::Model)]
    struct ScratchOnly {
        #[key]
        id: i64,
    }

    #[tokio::test]
    async fn groups_due_update_respects_refresh_interval() {
        let db = Database::in_memory().await.expect("in-memory db");
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

        // Due: refreshed 1h ago with a 30-minute interval (default is 1440).
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

    #[tokio::test]
    async fn dns_settings_and_routing_rules_roundtrip() {
        let db = Database::in_memory().await.expect("in-memory db");
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

    /// `EndpointRow.active_link` respects a manual protocol override.
    #[tokio::test]
    async fn manual_override_shapes_active_link() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "10.10.10.10", HostType::Ipv4, 53, 10).await;
        seed_link(&mut conn, 1, 1002, 20).await;

        let row = db.get_endpoint(EndpointId::new(1)).await.expect("row");
        let mut row = row.expect("row");
        assert_eq!(
            row.active_link().unwrap().protocol_id,
            ProtocolId::new(1002)
        );

        row.endpoint.manual_protocol_override = Some(ProtocolId::new(1001));
        assert_eq!(
            row.active_link().unwrap().protocol_id,
            ProtocolId::new(1001)
        );

        // Override pointing at a missing protocol falls back to selection.
        row.endpoint.manual_protocol_override = Some(ProtocolId::new(999));
        assert_eq!(
            row.active_link().unwrap().protocol_id,
            ProtocolId::new(1002)
        );
    }

    // ── Typed writes (Task 10) ───────────────────────────────────────────

    /// A full `Endpoint` struct for the typed write methods.
    fn endpoint_struct(id: i64, host: &str, host_type: HostType, port: u16) -> Endpoint {
        Endpoint {
            id: EndpointId::new(id),
            host: host.to_string(),
            host_type,
            port,
            ports: Vec::new(),
            parent_id: None,
            last_source: None,
            manual_protocol_override: None,
            resolved_as: Vec::new(),
            resolved_at: None,
            created_at: ts(0),
            links: Deferred::default(),
            group_links: Deferred::default(),
        }
    }

    /// A full `ProfileStats` struct for the typed write methods.
    fn link_struct(protocol_id: i64, endpoint_id: i64, last_seen: i64) -> ProfileStats {
        ProfileStats {
            protocol_id: ProtocolId::new(protocol_id),
            endpoint_id: EndpointId::new(endpoint_id),
            core_type: CoreType::Xray,
            config_type: ConfigType::ShareUrl,
            last_used_at: None,
            last_seen_at: ts(last_seen),
            task_id: None,
            task_queue: Vec::new(),
            latency: None,
            speed_bps: None,
            error: None,
            traffic: zero_traffic(),
            created_at: ts(0),
            updated_at: ts(0),
            version: 1,
            protocol: Deferred::default(),
            endpoint: Deferred::default(),
        }
    }

    /// A full, loaded `Protocol` struct (deferred JSON included) for
    /// `upsert_protocol`.
    fn protocol_struct(id: i64) -> Protocol {
        Protocol {
            id: ProtocolId::new(id),
            sig: id,
            cred_hash: 0,
            proto_kind: ProtocolKind::Vless,
            transport: tcp_transport(),
            security: no_security(),
            config: Deferred::from(Json(vless_config())),
            created_at: ts(0),
            links: Deferred::default(),
        }
    }

    #[tokio::test]
    async fn upsert_endpoint_is_idempotent() {
        let db = Database::in_memory().await.expect("in-memory db");
        db.upsert_endpoint(&endpoint_struct(1, "1.2.3.4", HostType::Ipv4, 443))
            .await
            .expect("upsert");
        db.upsert_endpoint(&endpoint_struct(1, "9.9.9.9", HostType::Ipv4, 8443))
            .await
            .expect("upsert again");

        let mut conn = db.connection().await.expect("connection");
        let count = Endpoint::all()
            .count()
            .exec(&mut conn)
            .await
            .expect("count");
        assert_eq!(count, 1, "second upsert must not duplicate the row");

        let ep = Endpoint::filter_by_id(EndpointId::new(1))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .expect("row");
        assert_eq!(ep.host, "9.9.9.9", "identity fields refresh on re-upsert");
        assert_eq!(ep.port, 8443);

        // Owned state (resolution cache, manual override) survives re-upserts.
        db.update_endpoint_resolution(EndpointId::new(1), vec!["1.1.1.1".to_string()], ts(77))
            .await
            .expect("resolve");
        db.set_manual_override(EndpointId::new(1), Some(ProtocolId::new(99)))
            .await
            .expect("override");
        db.upsert_endpoint(&endpoint_struct(1, "9.9.9.9", HostType::Ipv4, 8443))
            .await
            .expect("upsert after resolution");
        let ep = Endpoint::filter_by_id(EndpointId::new(1))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .expect("row");
        assert_eq!(
            ep.resolved_as,
            vec!["1.1.1.1".to_string()],
            "resolution cache preserved"
        );
        assert_eq!(ep.resolved_at, Some(ts(77)));
        assert_eq!(
            ep.manual_protocol_override,
            Some(ProtocolId::new(99)),
            "manual override preserved"
        );
    }

    #[tokio::test]
    async fn upsert_link_is_idempotent_and_updates_fields() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        db.upsert_endpoint(&endpoint_struct(1, "1.2.3.4", HostType::Ipv4, 443))
            .await
            .expect("endpoint");
        db.upsert_protocol(&protocol_struct(1001))
            .await
            .expect("protocol");

        let mut link = link_struct(1001, 1, 100);
        link.traffic = TrafficStats {
            today_up: 5,
            today_down: 6,
            total_up: 7,
            total_down: 8,
        };
        db.upsert_link(&link).await.expect("upsert link");
        db.upsert_link(&link).await.expect("upsert link again");

        let count = ProfileStats::all()
            .count()
            .exec(&mut conn)
            .await
            .expect("count");
        assert_eq!(count, 1, "composite-key upsert must not duplicate");

        // Second upsert replaces the result-state fields (e.g. latency).
        link.latency = Some(Latency::Fast { delay: 42 });
        link.last_seen_at = ts(200);
        db.upsert_link(&link)
            .await
            .expect("upsert link with latency");

        let stored = ProfileStats::filter_by_protocol_id_and_endpoint_id(
            ProtocolId::new(1001),
            EndpointId::new(1),
        )
        .first()
        .exec(&mut conn)
        .await
        .expect("read")
        .expect("row");
        assert_eq!(stored.latency, Some(Latency::Fast { delay: 42 }));
        assert_eq!(stored.last_seen_at, ts(200));
        assert_eq!(stored.traffic, link.traffic);
    }

    #[tokio::test]
    async fn upsert_link_preserves_scheduler_and_activity_state() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        db.upsert_endpoint(&endpoint_struct(1, "1.2.3.4", HostType::Ipv4, 443))
            .await
            .expect("endpoint");
        db.upsert_protocol(&protocol_struct(1001))
            .await
            .expect("protocol");

        db.upsert_link(&link_struct(1001, 1, 100))
            .await
            .expect("upsert link");
        db.update_scheduler_state(
            ProtocolId::new(1001),
            EndpointId::new(1),
            Some(7),
            &[1, 2, 3],
        )
        .await
        .expect("scheduler state");
        db.update_last_used(ProtocolId::new(1001), EndpointId::new(1), ts(300))
            .await
            .expect("last used");

        // A re-upsert must not clobber scheduler or activity state.
        db.upsert_link(&link_struct(1001, 1, 150))
            .await
            .expect("re-upsert");

        let stored = ProfileStats::filter_by_protocol_id_and_endpoint_id(
            ProtocolId::new(1001),
            EndpointId::new(1),
        )
        .first()
        .exec(&mut conn)
        .await
        .expect("read")
        .expect("row");
        assert_eq!(stored.task_id, Some(7), "scheduler task_id survives upsert");
        assert_eq!(
            stored.task_queue,
            vec![1, 2, 3],
            "scheduler queue survives upsert"
        );
        assert_eq!(
            stored.last_used_at,
            Some(ts(300)),
            "activity timestamp survives upsert"
        );
        assert_eq!(
            stored.last_seen_at,
            ts(150),
            "link last_seen replaces on upsert"
        );
    }

    #[tokio::test]
    async fn update_last_used_refreshes_both_columns() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "1.2.3.4", HostType::Ipv4, 443, 10).await;

        db.update_last_used(ProtocolId::new(1001), EndpointId::new(1), ts(500))
            .await
            .expect("update");

        let link = ProfileStats::filter_by_protocol_id_and_endpoint_id(
            ProtocolId::new(1001),
            EndpointId::new(1),
        )
        .first()
        .exec(&mut conn)
        .await
        .expect("read")
        .expect("row");
        assert_eq!(link.last_used_at, Some(ts(500)));
        assert_eq!(
            link.last_seen_at,
            ts(500),
            "active use keeps the link out of Stale"
        );
    }

    #[tokio::test]
    async fn update_endpoint_resolution_sets_cache() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "dns.example", HostType::Dns, 443, 10).await;

        db.update_endpoint_resolution(
            EndpointId::new(1),
            vec!["1.1.1.1".to_string(), "2.2.2.2".to_string()],
            ts(100),
        )
        .await
        .expect("resolve");

        let ep = Endpoint::filter_by_id(EndpointId::new(1))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .expect("row");
        assert_eq!(
            ep.resolved_as,
            vec!["1.1.1.1".to_string(), "2.2.2.2".to_string()]
        );
        assert_eq!(ep.resolved_at, Some(ts(100)));
    }

    #[tokio::test]
    async fn upsert_resolved_ip_children_upserts_and_prunes() {
        let db = Database::in_memory().await.expect("in-memory db");
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

        let ip1: IpAddr = "1.1.1.1".parse().expect("ip");
        let ip2: IpAddr = "2.2.2.2".parse().expect("ip");
        db.upsert_resolved_ip_children(EndpointId::new(50), &[ip1, ip2])
            .await
            .expect("upsert children");

        let children = db
            .endpoints_by_parent(EndpointId::new(50))
            .await
            .expect("children");
        assert_eq!(children.len(), 2);
        let cid1 = EndpointId::new(stable_hash("1.1.1.1".to_string(), 0i64));
        let cid2 = EndpointId::new(stable_hash("2.2.2.2".to_string(), 0i64));
        let mut by_host: HashMap<&str, &Endpoint> =
            children.iter().map(|c| (c.host.as_str(), c)).collect();
        for (host, id) in [("1.1.1.1", cid1), ("2.2.2.2", cid2)] {
            let child = by_host.remove(host).expect("child");
            assert_eq!(child.id, id, "deterministic id from IP");
            assert_eq!(child.host_type, HostType::Ipv4);
            assert_eq!(child.port, 443);
            assert_eq!(child.parent_id, Some(EndpointId::new(50)));
        }

        // A child accumulates a link; re-resolving with the same IP must keep it.
        toasty::create!(Protocol {
            id: ProtocolId::new(9001),
            sig: 9001,
            cred_hash: 0,
            proto_kind: ProtocolKind::Vless,
            transport: tcp_transport(),
            security: no_security(),
            config: Deferred::from(Json(vless_config())),
        })
        .exec(&mut conn)
        .await
        .expect("child protocol");
        toasty::create!(ProfileStats {
            protocol_id: ProtocolId::new(9001),
            endpoint_id: cid1,
            core_type: CoreType::Xray,
            config_type: ConfigType::ShareUrl,
            last_seen_at: ts(1),
            task_queue: Vec::<u16>::new(),
            traffic: zero_traffic(),
        })
        .exec(&mut conn)
        .await
        .expect("child link");

        // Prune: only ip1 stays, and its accumulated link survives.
        db.upsert_resolved_ip_children(EndpointId::new(50), &[ip1])
            .await
            .expect("prune children");
        let children = db
            .endpoints_by_parent(EndpointId::new(50))
            .await
            .expect("children");
        assert_eq!(children.len(), 1, "stale child removed");
        assert_eq!(children[0].host, "1.1.1.1");
        let kept = ProfileStats::filter_by_protocol_id_and_endpoint_id(ProtocolId::new(9001), cid1)
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .expect("link");
        assert_eq!(
            kept.endpoint_id, cid1,
            "re-resolution keeps the child row + links"
        );
    }

    #[tokio::test]
    async fn set_manual_override_sets_and_clears() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "10.10.10.10", HostType::Ipv4, 53, 10).await;
        seed_link(&mut conn, 1, 1002, 20).await;

        db.set_manual_override(EndpointId::new(1), Some(ProtocolId::new(1001)))
            .await
            .expect("set");
        let ep = Endpoint::filter_by_id(EndpointId::new(1))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .expect("row");
        assert_eq!(ep.manual_protocol_override, Some(ProtocolId::new(1001)));

        db.set_manual_override(EndpointId::new(1), None)
            .await
            .expect("clear");
        let ep = Endpoint::filter_by_id(EndpointId::new(1))
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .expect("row");
        assert_eq!(ep.manual_protocol_override, None);
    }

    #[tokio::test]
    async fn restore_endpoint_bumps_last_seen() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "1.2.3.4", HostType::Ipv4, 443, 10).await;
        seed_link(&mut conn, 1, 1002, 20).await;

        db.restore_endpoint(EndpointId::new(1))
            .await
            .expect("restore");

        let now = Timestamp::now().as_second();
        let links: Vec<ProfileStats> =
            ProfileStats::filter(ProfileStats::fields().endpoint_id().eq(EndpointId::new(1)))
                .exec(&mut conn)
                .await
                .expect("links");
        assert_eq!(links.len(), 2);
        for link in links {
            assert!(
                link.last_seen_at.as_second() >= now - 60,
                "every link of the endpoint is refreshed"
            );
        }
    }

    #[tokio::test]
    async fn upsert_group_replaces_row() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        let mut g = Group {
            id: "g1".to_string(),
            name: Some("A".to_string()),
            url: Some("https://example.com/sub".to_string()),
            enabled: true,
            user_agent: None,
            convert_target: None,
            core_type: None,
            sort_order: Some(0),
            last_refreshed: None,
            status: None,
            error_message: None,
            refresh_interval: Some(30),
        };
        db.upsert_group(&g).await.expect("insert");

        g.name = Some("B".to_string());
        g.enabled = false;
        db.upsert_group(&g).await.expect("update");

        let rows: Vec<Group> = Group::filter(Group::fields().id().eq("g1".to_string()))
            .exec(&mut conn)
            .await
            .expect("rows");
        assert_eq!(rows.len(), 1, "upsert replaces insert_group/update_group");
        let stored = Group::filter_by_id("g1".to_string())
            .first()
            .exec(&mut conn)
            .await
            .expect("read")
            .expect("row");
        assert_eq!(stored.name.as_deref(), Some("B"));
        assert!(!stored.enabled);
    }

    #[tokio::test]
    async fn clear_group_endpoints_and_delete_group() {
        let db = Database::in_memory().await.expect("in-memory db");
        let mut conn = db.connection().await.expect("connection");
        seed_endpoint(&mut conn, 1, 1001, "1.2.3.4", HostType::Ipv4, 443, 10).await;
        toasty::create!(EndpointGroup {
            endpoint_id: EndpointId::new(1),
            group_id: "g-a".to_string(),
            last_seen_at: ts(1),
        })
        .exec(&mut conn)
        .await
        .expect("link a");
        toasty::create!(EndpointGroup {
            endpoint_id: EndpointId::new(1),
            group_id: "g-b".to_string(),
            last_seen_at: ts(1),
        })
        .exec(&mut conn)
        .await
        .expect("link b");

        let n = db.clear_group_endpoints("g-a").await.expect("clear");
        assert_eq!(n, 1, "clear_group returns the link count");
        let remaining: Vec<EndpointGroup> =
            EndpointGroup::all().exec(&mut conn).await.expect("rows");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].group_id, "g-b");

        // delete_group removes the remaining links + the group, keeps endpoints.
        db.upsert_group(&Group {
            id: "g-b".to_string(),
            name: None,
            url: None,
            enabled: true,
            user_agent: None,
            convert_target: None,
            core_type: None,
            sort_order: None,
            last_refreshed: None,
            status: None,
            error_message: None,
            refresh_interval: None,
        })
        .await
        .expect("group b");
        db.delete_group("g-b").await.expect("delete group");

        let count = EndpointGroup::all()
            .count()
            .exec(&mut conn)
            .await
            .expect("count");
        assert_eq!(count, 0, "group links deleted");
        assert!(
            Group::filter_by_id("g-b".to_string())
                .first()
                .exec(&mut conn)
                .await
                .expect("read")
                .is_none(),
            "group deleted"
        );
        assert!(
            Endpoint::filter_by_id(EndpointId::new(1))
                .first()
                .exec(&mut conn)
                .await
                .expect("read")
                .is_some(),
            "endpoint + links survive group deletion"
        );
    }
}
