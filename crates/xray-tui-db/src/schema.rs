pub async fn create_tables(conn: &turso::Connection) -> turso::Result<()> {
    // Use WAL mode for good concurrent read/write performance.
    // MVCC mode (libSQL extension) was considered but causes extreme slowdown
    // for large single-transaction bulk inserts (e.g., subscription upsert
    // with thousands of profiles).
    conn.pragma_update("journal_mode", "wal").await?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS profile_cores (
            sub_uid             INTEGER PRIMARY KEY NOT NULL,
            config_type         INTEGER NOT NULL,
            core_type           TEXT NOT NULL DEFAULT 'xray',
            address             TEXT,
            port                INTEGER,
            user_id             TEXT,
            security            TEXT,
            network             TEXT,
            stream_settings     TEXT,
            protocol_settings   TEXT,
            created_at          TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS group_profiles (
            id                  TEXT PRIMARY KEY NOT NULL,
            sub_uid             INTEGER NOT NULL REFERENCES profile_cores(sub_uid),
            group_id            TEXT NOT NULL REFERENCES groups(id),
            remarks             TEXT,
            is_sub              INTEGER DEFAULT 0,
            sub_id              TEXT,
            sort_order          INTEGER DEFAULT 0,
            is_active           INTEGER DEFAULT 0,
            updated_at          TEXT,
            created_at          TEXT,
            UNIQUE(group_id, sub_uid)
        );

        CREATE INDEX IF NOT EXISTS idx_group_profiles_sub_uid ON group_profiles(sub_uid);
        CREATE INDEX IF NOT EXISTS idx_group_profiles_group_id ON group_profiles(group_id);
        CREATE INDEX IF NOT EXISTS idx_group_profiles_active ON group_profiles(group_id, is_active);

        CREATE TABLE IF NOT EXISTS groups (
            id                  TEXT PRIMARY KEY NOT NULL,
            name                TEXT,
            subscription_url    TEXT,
            subscription_enabled INTEGER DEFAULT 0,
            user_agent          TEXT,
            convert_target      INTEGER,
            core_type           TEXT,
            sort_order          INTEGER DEFAULT 0,
            is_system           INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS subscriptions (
            id                  TEXT PRIMARY KEY NOT NULL,
            group_id            TEXT,
            url                 TEXT NOT NULL,
            last_updated        TEXT,
            update_interval     INTEGER DEFAULT 0,
            user_agent          TEXT,
            status              TEXT DEFAULT 'idle',
            error_message       TEXT
        );

        CREATE TABLE IF NOT EXISTS routing_rules (
            id                  TEXT PRIMARY KEY NOT NULL,
            group_id            TEXT,
            type                INTEGER NOT NULL,
            domain_matcher      TEXT,
            domains             TEXT,
            ips                 TEXT,
            inbound_tags        TEXT,
            port                TEXT,
            source_ports        TEXT,
            network             TEXT,
            protocols           TEXT,
            domain_strategy     TEXT,
            outbound_tag        TEXT,
            balancer_tag        TEXT,
            rule_set_file       TEXT,
            rule_set_url        TEXT,
            sort_order          INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS dns_settings (
            id                  TEXT PRIMARY KEY NOT NULL,
            name                TEXT,
            servers             TEXT,
            hosts               TEXT,
            query_strategy      TEXT,
            disable_cache       INTEGER,
            disable_fallback    INTEGER,
            client_ip           TEXT
        );

        CREATE TABLE IF NOT EXISTS profile_extensions (
            profile_id          TEXT PRIMARY KEY NOT NULL,
            delay               INTEGER DEFAULT -1,
            speed               INTEGER DEFAULT -1,
            sort_order          INTEGER DEFAULT 0,
            ip_info             TEXT
        );

        CREATE TABLE IF NOT EXISTS server_stats (
            profile_id          TEXT PRIMARY KEY NOT NULL,
            today_up            INTEGER DEFAULT 0,
            today_down          INTEGER DEFAULT 0,
            total_up            INTEGER DEFAULT 0,
            total_down          INTEGER DEFAULT 0,
            last_updated        TEXT
        );

        CREATE TABLE IF NOT EXISTS ping_sessions (
            id              TEXT PRIMARY KEY NOT NULL,
            batch_id        TEXT NOT NULL,
            profile_id      TEXT NOT NULL REFERENCES group_profiles(id),
            config_type     INTEGER NOT NULL,
            core_type       TEXT NOT NULL,
            address         TEXT,
            port            INTEGER,
            triplet_rank    INTEGER NOT NULL,
            ping_type       TEXT NOT NULL DEFAULT 'fast',
            status          TEXT NOT NULL DEFAULT 'queued',
            latency_ms      INTEGER,
            speed_bps       INTEGER,
            ip_info         TEXT,
            error           TEXT,
            created_at      TEXT DEFAULT (datetime('now')),
            updated_at      TEXT DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_ping_sessions_batch ON ping_sessions(batch_id, status, triplet_rank);
        ",
    )
    .await?;

    Ok(())
}
