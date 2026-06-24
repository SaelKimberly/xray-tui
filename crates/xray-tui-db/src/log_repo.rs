use crate::{LogEntry, Result};
use std::time::Duration;
use turso::params;
pub struct LogRepository<'a> {
    conn: &'a turso::Connection,
}

impl<'a> LogRepository<'a> {
    pub fn new(conn: &'a turso::Connection) -> Self {
        Self { conn }
    }

    /// Batch insert logs within a single `BEGIN IMMEDIATE` transaction.
    ///
    /// Uses a dedicated connection (from `Database::new_connection()`) that
    /// has `busy_timeout = 500ms` set, so `BEGIN IMMEDIATE` waits up to 500ms
    /// for the main connection's transaction to complete. Once acquired, the
    /// RESERVED write lock guarantees no write-write conflicts on COMMIT.
    ///
    /// Retries automatically on `Busy`/`BusySnapshot` errors (up to 5
    /// attempts with exponential backoff) as a safety net.
    pub async fn insert_batch(&self, entries: &[LogEntry]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }
        let max_retries = 5;
        let mut attempt = 0usize;
        loop {
            attempt += 1;
            let last = attempt >= max_retries;
            match self.conn.execute("BEGIN IMMEDIATE", ()).await {
                Err(e)
                    if !last
                        && matches!(&e, turso::Error::Busy(_) | turso::Error::BusySnapshot(_)) =>
                {
                    tokio::time::sleep(Duration::from_millis(backoff_ms(attempt))).await;
                    continue;
                }
                Err(e) => return Err(e.into()),
                Ok(_) => {}
            }
            let mut stmt = match self
                .conn
                .prepare(
                    "INSERT INTO logs
                     (timestamp_nanos, level, target, message, metadata_json, source)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                )
                .await
            {
                Ok(s) => s,
                Err(e) => {
                    let _ = self.conn.execute("ROLLBACK", ()).await;
                    return Err(e.into());
                }
            };
            let insert_result: Result<()> = async {
                for e in entries {
                    stmt.execute(params![
                        e.timestamp_nanos,
                        e.level.as_str(),
                        e.target.as_str(),
                        e.message.as_str(),
                        e.metadata_json.as_deref(),
                        e.source.as_str(),
                    ])
                    .await?;
                }
                Ok(())
            }
            .await;
            match insert_result {
                Ok(()) => {
                    self.conn.execute("COMMIT", ()).await?;
                    return Ok(());
                }
                Err(e) => {
                    let _ = self.conn.execute("ROLLBACK", ()).await;
                    return Err(e);
                }
            }
        }
    }

    /// Query logs with optional filters and pagination.
    ///
    /// Filters:
    /// - `start_nanos` / `end_nanos`: inclusive time range
    /// - `min_level`: minimum log level (error > warning > info > debug > trace).
    ///   Matches the specified level and all higher (more severe) levels.
    /// - `target_contains`: substring match on target field
    /// - `source`: exact match on source field
    /// - `limit` / `offset`: pagination (offset is applied after filtering)
    pub async fn get_filtered(
        &self,
        start_nanos: Option<i64>,
        end_nanos: Option<i64>,
        min_level: Option<&str>,
        target_contains: Option<&str>,
        source: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<LogEntry>> {
        let mut sql = String::from(
            "SELECT id, timestamp_nanos, level, target, message, metadata_json, source
             FROM logs WHERE 1=1",
        );
        use turso::Value;
        let mut params: Vec<Value> = Vec::new();

        if let Some(start) = start_nanos {
            sql.push_str(&format!(" AND timestamp_nanos >= ?{}", params.len() + 1));
            params.push(Value::Integer(start));
        }
        if let Some(end) = end_nanos {
            sql.push_str(&format!(" AND timestamp_nanos <= ?{}", params.len() + 1));
            params.push(Value::Integer(end));
        }
        if let Some(level) = min_level {
            let levels = level_hierarchy(level);
            if levels.is_empty() {
                sql.push_str(" AND 1=0");
            } else {
                let placeholders: Vec<String> = (0..levels.len())
                    .map(|i| format!("?{}", params.len() + 1 + i))
                    .collect();
                sql.push_str(&format!(" AND level IN ({})", placeholders.join(",")));
                for lvl in &levels {
                    params.push(Value::Text(lvl.clone()));
                }
            }
        }
        if let Some(target) = target_contains {
            sql.push_str(&format!(" AND target LIKE ?{}", params.len() + 1));
            params.push(Value::Text(format!("%{}%", target)));
        }
        if let Some(src) = source {
            sql.push_str(&format!(" AND source = ?{}", params.len() + 1));
            params.push(Value::Text(src.to_owned()));
        }

        sql.push_str(" ORDER BY timestamp_nanos DESC, id DESC");

        // LIMIT before OFFSET (SQLite requirement)
        sql.push_str(&format!(" LIMIT ?{}", params.len() + 1));
        params.push(Value::Integer(limit));
        if offset > 0 {
            sql.push_str(&format!(" OFFSET ?{}", params.len() + 1));
            params.push(Value::Integer(offset));
        }

        let mut stmt = self.conn.prepare_cached(&sql).await?;
        let mut rows = stmt.query(turso::params_from_iter(params)).await?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await? {
            results.push(LogEntry {
                id: Some(row.get::<i64>(0)?),
                timestamp_nanos: row.get::<i64>(1)?,
                level: row.get::<String>(2)?,
                target: row.get::<String>(3)?,
                message: row.get::<String>(4)?,
                metadata_json: row.get::<Option<String>>(5)?,
                source: row.get::<String>(6)?,
            });
        }
        Ok(results)
    }

    /// Delete logs older than the given timestamp (in nanoseconds).
    /// Returns the number of rows deleted.
    pub async fn delete_older_than(&self, cutoff_nanos: i64) -> Result<u64> {
        let affected = self
            .conn
            .execute(
                "DELETE FROM logs WHERE timestamp_nanos < ?1",
                params![cutoff_nanos],
            )
            .await?;
        Ok(affected)
    }
}

/// Exponential backoff delay: 5ms, 10ms, 20ms, 40ms, 80ms
fn backoff_ms(attempt: usize) -> u64 {
    5u64 * 2u64.pow(attempt as u32 - 1)
}

/// Returns the list of levels that match `level` and above (more severe).
/// Example: "info" → ["info", "warning", "error"]
fn level_hierarchy(level: &str) -> Vec<String> {
    let all = ["error", "warning", "info", "debug", "trace"];
    let pos = all.iter().position(|l| *l == level);
    match pos {
        Some(p) => all[..=p].iter().map(|l| l.to_string()).rev().collect(),
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_hierarchy() {
        assert_eq!(level_hierarchy("error"), vec!["error"]);
        assert_eq!(level_hierarchy("info"), vec!["info", "warning", "error"]);
        assert_eq!(
            level_hierarchy("debug"),
            vec!["debug", "info", "warning", "error"]
        );
        assert_eq!(
            level_hierarchy("trace"),
            vec!["trace", "debug", "info", "warning", "error"]
        );
        assert!(level_hierarchy("unknown").is_empty());
    }
}
