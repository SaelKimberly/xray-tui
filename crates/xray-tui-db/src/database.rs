use std::collections::HashMap;
use std::path::Path;

use jiff::Timestamp;
use toasty::stmt::IntoStatement;
use toasty_core::stmt::Value;

use crate::error::{DatabaseError, Result};
use crate::models_toasty::{
    DnsSetting, Endpoint, EndpointGroup, EndpointId, EndpointRow, Group, HostType, ProfileStats,
    Protocol, ProtocolId, RoutingRule,
};

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
}
