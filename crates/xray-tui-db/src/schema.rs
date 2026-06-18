pub fn create_tables(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS profiles (
            id                  TEXT PRIMARY KEY NOT NULL,
            config_type         INTEGER NOT NULL,
            core_type           TEXT NOT NULL DEFAULT 'xray',
            remarks             TEXT,
            address             TEXT,
            port                INTEGER,
            user_id             TEXT,
            security            TEXT,
            network             TEXT,
            stream_settings     TEXT,
            protocol_settings   TEXT,
            is_sub              INTEGER DEFAULT 0,
            sub_id              TEXT,
            sub_uid             INTEGER NOT NULL DEFAULT 0,
            sort_order          INTEGER DEFAULT 0,
            is_active           INTEGER DEFAULT 0,
            created_at          TEXT,
            updated_at          TEXT
        );

        CREATE TABLE IF NOT EXISTS groups (
            id                  TEXT PRIMARY KEY NOT NULL,
            name                TEXT,
            subscription_url    TEXT,
            subscription_enabled INTEGER DEFAULT 0,
            user_agent          TEXT,
            convert_target      INTEGER,
            core_type           TEXT,
            sort_order          INTEGER DEFAULT 0
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
        ",
    )?;

    // Migration v2: add sub_uid column (ignore error if already exists)
    let _ =
        conn.execute_batch("ALTER TABLE profiles ADD COLUMN sub_uid INTEGER NOT NULL DEFAULT 0;");
    let _ = conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_profiles_group_sub_uid ON profiles(group_id, sub_uid) WHERE sub_uid != 0;",
    );

    Ok(())
}
