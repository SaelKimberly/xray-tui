pub mod schema;
pub mod models;

use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("uuid error: {0}")]
    Uuid(#[from] uuid::Error),
}

pub type Result<T, E = DatabaseError> = std::result::Result<T, E>;

/// Result tuple returned by `get_all_profiles_with_details`.
pub type ProfileWithDetails = (Profile, Option<ProfileExtension>, Option<ServerStat>);

pub struct Database {
    conn: rusqlite::Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = rusqlite::Connection::open(path)?;
        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = rusqlite::Connection::open_in_memory()?;
        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    fn initialize_schema(&self) -> Result<()> {
        schema::create_tables(&self.conn)?;
        Ok(())
    }
}

use models::{Group, Profile, ProfileExtension, ServerStat};
use rusqlite::Row;

// ── Row conversion impls ──────────────────────────────────────────────

impl TryFrom<&Row<'_>> for Profile {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            config_type: row.get("config_type")?,
            core_type: row.get("core_type")?,
            remarks: row.get("remarks")?,
            address: row.get("address")?,
            port: row.get("port")?,
            user_id: row.get("user_id")?,
            security: row.get("security")?,
            network: row.get("network")?,
            stream_settings: row.get("stream_settings")?,
            protocol_settings: row.get("protocol_settings")?,
            is_sub: row.get("is_sub")?,
            sub_id: row.get("sub_id")?,
            group_id: row.get("group_id")?,
            sort_order: row.get("sort_order")?,
            is_active: row.get("is_active")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

impl TryFrom<&Row<'_>> for Group {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            subscription_url: row.get("subscription_url")?,
            subscription_enabled: row.get("subscription_enabled")?,
            user_agent: row.get("user_agent")?,
            convert_target: row.get("convert_target")?,
            core_type: row.get("core_type")?,
            sort_order: row.get("sort_order")?,
        })
    }
}

impl TryFrom<&Row<'_>> for ProfileExtension {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            profile_id: row.get("profile_id")?,
            delay: row.get("delay")?,
            speed: row.get("speed")?,
            sort_order: row.get("sort_order")?,
            ip_info: row.get("ip_info")?,
        })
    }
}

impl TryFrom<&Row<'_>> for ServerStat {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            profile_id: row.get("profile_id")?,
            today_up: row.get("today_up")?,
            today_down: row.get("today_down")?,
            total_up: row.get("total_up")?,
            total_down: row.get("total_down")?,
            last_updated: row.get("last_updated")?,
        })
    }
}

// ── Query methods ────────────────────────────────────────────────────

impl Database {
    pub fn get_all_profiles(&self) -> Result<Vec<Profile>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM profiles ORDER BY sort_order"
        )?;
        let rows = stmt.query_map([], |row| Profile::try_from(row))?;
        let mut profiles = Vec::new();
        for row in rows {
            profiles.push(row?);
        }
        Ok(profiles)
    }

    pub fn get_profiles_by_group(&self, group_id: &str) -> Result<Vec<Profile>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM profiles WHERE group_id = ?1 ORDER BY sort_order"
        )?;
        let rows = stmt.query_map([group_id], |row| Profile::try_from(row))?;
        let mut profiles = Vec::new();
        for row in rows {
            profiles.push(row?);
        }
        Ok(profiles)
    }

    pub fn get_all_groups(&self) -> Result<Vec<Group>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM groups ORDER BY sort_order"
        )?;
        let rows = stmt.query_map([], |row| Group::try_from(row))?;
        let mut groups = Vec::new();
        for row in rows {
            groups.push(row?);
        }
        Ok(groups)
    }


    pub fn get_profile_extension(&self, profile_id: &str) -> Result<Option<ProfileExtension>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM profile_extensions WHERE profile_id = ?1"
        )?;
        let mut rows = stmt.query_map([profile_id], |row| ProfileExtension::try_from(row))?;
        match rows.next() {
            Some(Ok(ext)) => Ok(Some(ext)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn get_server_stats(&self, profile_id: &str) -> Result<Option<ServerStat>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM server_stats WHERE profile_id = ?1"
        )?;
        let mut rows = stmt.query_map([profile_id], |row| ServerStat::try_from(row))?;
        match rows.next() {
            Some(Ok(stats)) => Ok(Some(stats)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn get_all_profiles_with_details(&self) -> Result<Vec<ProfileWithDetails>> {
        let query = "
            SELECT
                p.*,
                pe.profile_id AS ext_profile_id,
                pe.delay,
                pe.speed,
                pe.sort_order AS ext_sort_order,
                pe.ip_info,
                ss.profile_id AS stats_profile_id,
                ss.today_up,
                ss.today_down,
                ss.total_up,
                ss.total_down,
                ss.last_updated
            FROM profiles p
            LEFT JOIN profile_extensions pe ON pe.profile_id = p.id
            LEFT JOIN server_stats ss ON ss.profile_id = p.id
            ORDER BY p.sort_order
        ";
        let mut stmt = self.conn.prepare(query)?;
        let rows = stmt.query_map([], |row| {
            let profile = Profile::try_from(row)?;

            let ext_profile_id: Option<String> = row.get("ext_profile_id")?;
            let extension = if ext_profile_id.is_some() {
                Some(ProfileExtension {
                    profile_id: ext_profile_id.unwrap_or_default(),
                    delay: row.get("delay").ok().flatten(),
                    speed: row.get("speed").ok().flatten(),
                    sort_order: row.get("ext_sort_order").ok().flatten(),
                    ip_info: row.get("ip_info").ok().flatten(),
                })
            } else {
                None
            };

            let stats_profile_id: Option<String> = row.get("stats_profile_id")?;
            let stats = if stats_profile_id.is_some() {
                Some(ServerStat {
                    profile_id: stats_profile_id.unwrap_or_default(),
                    today_up: row.get("today_up").ok().flatten(),
                    today_down: row.get("today_down").ok().flatten(),
                    total_up: row.get("total_up").ok().flatten(),
                    total_down: row.get("total_down").ok().flatten(),
                    last_updated: row.get("last_updated").ok().flatten(),
                })
            } else {
                None
            };

            Ok((profile, extension, stats))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
}
}
