use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use xray_tui_db::{DbConnection, LogEntry, LogRepository};

/// Parameters for a filtered log query.
#[derive(Debug, Clone)]
pub struct QueryParams {
    pub start_nanos: Option<i64>,
    pub end_nanos: Option<i64>,
    pub min_level: Option<String>,
    pub target_contains: Option<String>,
    pub source: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

/// Messages sent to the LogStorageWorker.
pub enum LogWorkerMessage {
    /// Persist a log entry (batched before DB write).
    Entry(LogEntry),
    /// Query logs — the worker merges its pending batch with DB results.
    Query {
        params: QueryParams,
        resp_tx: oneshot::Sender<Vec<LogEntry>>,
    },
}

/// Background worker that batches log writes and serves filtered queries.
///
/// Architecture:
/// - Receives `LogWorkerMessage` entries via an unbounded mpsc channel.
/// - Batches entries and flushes to the DB every 100ms or when batch_size is reached.
/// - Serves filtered queries by merging the pending (not-yet-flushed) batch
///   with DB results, making it the unified source of truth for log queries.
pub struct LogStorageWorker {
    rx: mpsc::UnboundedReceiver<LogWorkerMessage>,
    /// Dedicated Turso connection for log writes, avoiding lock contention
    /// with the main application connection (allows BEGIN CONCURRENT).
    log_conn: DbConnection,
    batch_size: usize,
    flush_interval: Duration,
}
impl LogStorageWorker {
    pub fn new(
        rx: mpsc::UnboundedReceiver<LogWorkerMessage>,
        log_conn: DbConnection,
        batch_size: usize,
    ) -> Self {
        Self {
            rx,
            log_conn,
            batch_size,
            flush_interval: Duration::from_millis(100),
        }
    }

    pub async fn run(mut self) {
        let mut batch: Vec<LogEntry> = Vec::with_capacity(self.batch_size);
        let mut flush_timer = tokio::time::interval(self.flush_interval);
        // Skip the first immediate tick — flush only after the interval elapses.
        flush_timer.tick().await;
        loop {
            tokio::select! {
                msg = self.rx.recv() => {
                    match msg {
                        Some(LogWorkerMessage::Entry(entry)) => {
                            batch.push(entry);
                            if batch.len() >= self.batch_size {
                                self.flush(&mut batch).await;
                            }
                        }
                        Some(LogWorkerMessage::Query { params, resp_tx }) => {
                            let results = self.handle_query(&batch, &params).await;
                            let _ = resp_tx.send(results);
                        }
                        None => {
                            // Channel closed — flush remaining and exit.
                            self.flush(&mut batch).await;
                            break;
                        }
                    }
                }
                _ = flush_timer.tick() => {
                    if !batch.is_empty() {
                        self.flush(&mut batch).await;
                    }
                }
            }
        }
    }

    /// Flush the current batch to the database.
    async fn flush(&self, batch: &mut Vec<LogEntry>) {
        if batch.is_empty() {
            return;
        }
        if let Err(e) = LogRepository::new(&self.log_conn).insert_batch(batch).await {
            eprintln!("LogStorageWorker flush error: {e}");
            // Entries remain in the batch for retry on the next flush.
            return;
        }
        batch.clear();
    }

    /// Run a filtered query across the pending batch AND the database.
    ///
    /// Strategy:
    /// 1. Collect matching pending entries in **newest-first** order (reverse batch).
    /// 2. Apply offset/limit to the pending slice.
    /// 3. Count how many entries were taken from pending.
    /// 4. Query the DB with adjusted limit and offset (offset reduced by pending count).
    /// 5. Merge: pending entries first (newest), then DB entries.
    async fn handle_query(&self, batch: &[LogEntry], params: &QueryParams) -> Vec<LogEntry> {
        // Collect matching pending entries, newest-first.
        let pending_matches: Vec<LogEntry> = batch
            .iter()
            .rev()
            .filter(|e| matches_filter(e, params))
            .cloned()
            .collect();

        let total_pending = pending_matches.len() as i64;

        // Take from pending batch (offset/limit apply to newest-first order).
        let taken_from_pending: Vec<LogEntry> = if params.offset < total_pending {
            let start = params.offset as usize;
            let end = (start + params.limit as usize).min(pending_matches.len());
            pending_matches[start..end].to_vec()
        } else {
            Vec::new()
        };

        let taken = taken_from_pending.len() as i64;

        // Adjust DB query: what offset and limit are left.
        let db_limit = (params.limit - taken).max(0);
        let db_offset = if params.offset >= total_pending {
            params.offset - total_pending
        } else {
            0 // pending entries already consumed the offset
        };

        // Query the database for the remaining entries.
        let mut results = taken_from_pending;
        if db_limit > 0
            && let Ok(mut db_results) = LogRepository::new(&self.log_conn)
                .get_filtered(
                    params.start_nanos,
                    params.end_nanos,
                    params.min_level.as_deref(),
                    params.target_contains.as_deref(),
                    params.source.as_deref(),
                    db_limit,
                    db_offset,
                )
                .await
        {
            results.append(&mut db_results);
        }

        results
    }
}

/// Check whether a log entry matches all non-None filter parameters.
fn matches_filter(entry: &LogEntry, params: &QueryParams) -> bool {
    // Time range
    if let Some(start) = params.start_nanos
        && entry.timestamp_nanos < start
    {
        return false;
    }
    if let Some(end) = params.end_nanos
        && entry.timestamp_nanos > end
    {
        return false;
    }
    // Level hierarchy: level must be >= min_level (more severe or equal)
    if let Some(ref min_level) = params.min_level
        && !level_at_least(&entry.level, min_level)
    {
        return false;
    }
    // Substring match on target
    if let Some(ref target) = params.target_contains
        && !entry.target.contains(target.as_str())
    {
        return false;
    }
    // Exact match on source
    if let Some(ref source) = params.source
        && entry.source != *source
    {
        return false;
    }
    true
}

/// Returns true if `entry_level` is at least as severe as `min_level`.
/// Hierarchy (most to least severe): error > warning > info > debug > trace
fn level_at_least(entry_level: &str, min_level: &str) -> bool {
    let levels = ["error", "warning", "info", "debug", "trace"];
    let entry_pos = levels.iter().position(|l| *l == entry_level);
    let min_pos = levels.iter().position(|l| *l == min_level);
    match (entry_pos, min_pos) {
        (Some(e), Some(m)) => e <= m, // lower index = more severe
        _ => false,
    }
}
