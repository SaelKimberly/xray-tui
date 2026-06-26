use heed::byteorder::BE;
use heed::types::{Bytes, Str, Unit};
use heed::{Database, Env, EnvOpenOptions};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ── Constants ──────────────────────────────────────────────────────────

/// Default virtual map size: 256 MB virtual address reservation (not physical RAM).
/// LMDB only uses physical pages for actually written data.
const DEFAULT_MAP_SIZE: usize = 268_435_456;

/// Database name for log entries.
const LOG_DB_NAME: &str = "logs";

/// Database name for seen targets (set).
const TARGETS_DB_NAME: &str = "targets";

// ── Types ──────────────────────────────────────────────────────────────

/// Big-endian u64 key type for heed — sort order matches chronological order.
type U64BE = heed::types::U64<BE>;

/// A single log entry stored in LMDB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessage {
    pub level: String,
    pub target: String,
    pub message: String,
    /// Timestamp in nanoseconds since UNIX epoch — used as storage key,
    /// not serialized inside the value. Stored here so `LogMessage` can
    /// be returned standalone from read queries.
    pub timestamp_nanos: u64,
}

// ── HeedLogStorage ─────────────────────────────────────────────────────

/// Persistent log storage backed by heed (LMDB).
///
/// ## Architecture
///
/// Two LMDB databases:
/// - `logs`: key = `u64` big-endian timestamp_ns, value = postcard-encoded `LogMessage`
/// - `targets`: key = target string, value = () — a set of seen targets
///
/// All reads/writes are synchronous (heed uses mmap). Callers should wrap
/// this in `Arc` for shared access across async tasks.
pub struct HeedLogStorage {
    env: Env,
    /// Key = u64 timestamp_ns (big-endian), Value = postcard(LogMessage)
    logs: Database<U64BE, Bytes>,
    /// Key = target string, Value = () — a set of seen targets
    targets: Database<Str, Unit>,
}

impl HeedLogStorage {
    /// Open or create the heed environment at `path`.
    ///
    /// # Safety
    ///
    /// LMDB uses a memory-mapped file which comes with safety precautions
    /// regarding concurrent access.
    ///
    /// # Errors
    ///
    /// Returns an error if the LMDB environment cannot be opened or the
    /// databases cannot be created.
    pub fn new(path: &Path) -> Result<Self> {
        std::fs::create_dir_all(path).map_err(|e| HeedError::Io(e.to_string()))?;

        let mut options = EnvOpenOptions::new();
        options.map_size(DEFAULT_MAP_SIZE);
        options.max_dbs(2);

        // SAFETY: We use default flags (no unsafe flags like NO_SYNC).
        let env = unsafe { options.open(path) }.map_err(|e| HeedError::Env(e.to_string()))?;

        let mut wtxn = env.write_txn().map_err(|e| HeedError::Txn(e.to_string()))?;

        let logs: Database<U64BE, Bytes> = env
            .create_database(&mut wtxn, Some(LOG_DB_NAME))
            .map_err(|e| HeedError::Db(e.to_string()))?;

        let targets: Database<Str, Unit> = env
            .create_database(&mut wtxn, Some(TARGETS_DB_NAME))
            .map_err(|e| HeedError::Db(e.to_string()))?;

        wtxn.commit().map_err(|e| HeedError::Txn(e.to_string()))?;

        Ok(Self { env, logs, targets })
    }

    /// Write a single log entry to the storage.
    ///
    /// If writing fails (e.g. map full), logs an error via `tracing::error!`
    /// and returns `Ok(())` — the UI must keep running even if logs fail.
    pub fn write_log(
        &self,
        timestamp_nanos: u64,
        level: &str,
        target: &str,
        message: &str,
    ) -> Result<()> {
        let msg = LogMessage {
            level: level.to_owned(),
            target: target.to_owned(),
            message: message.to_owned(),
            timestamp_nanos,
        };
        let value = postcard::to_allocvec(&msg).map_err(|e| HeedError::Serde(e.to_string()))?;

        let mut wtxn = match self.env.write_txn() {
            Ok(txn) => txn,
            Err(e) => {
                tracing::error!(target: "log_worker", "Heed write_txn error: {e}");
                return Ok(());
            }
        };

        if let Err(e) = self.logs.put(&mut wtxn, &timestamp_nanos, &value) {
            tracing::error!(target: "log_worker", "Heed log write error: {e}");
            // Don't return error — swallow so the UI keeps running
            return Ok(());
        }

        // Upsert target (put with existing key is a no-op for Unit values)
        if let Err(e) = self.targets.put(&mut wtxn, target, &()) {
            tracing::error!(target: "log_worker", "Heed target upsert error: {e}");
        }

        if let Err(e) = wtxn.commit() {
            tracing::error!(target: "log_worker", "Heed commit error: {e}");
        }

        Ok(())
    }

    /// Read the most recent `limit` log entries (newest first).
    pub fn read_recent(&self, limit: usize) -> Result<Vec<LogMessage>> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| HeedError::Txn(e.to_string()))?;

        let mut results = Vec::with_capacity(limit);
        let iter = self
            .logs
            .rev_iter(&rtxn)
            .map_err(|e| HeedError::Db(e.to_string()))?;

        for item in iter.take(limit) {
            let (key, value) = item.map_err(|e| HeedError::Db(e.to_string()))?;
            if let Some(msg) = decode_entry(key, value) {
                results.push(msg);
            }
        }
        Ok(results)
    }

    /// Read entries strictly older than `before_ns` (newest-first within range).
    pub fn read_older_than(&self, before_ns: u64, limit: usize) -> Result<Vec<LogMessage>> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| HeedError::Txn(e.to_string()))?;

        let mut results = Vec::with_capacity(limit);
        // rev_range(0..before_ns) iterates from before_ns-1 down to 0
        let range = 0u64..before_ns;
        let iter = self
            .logs
            .rev_range(&rtxn, &range)
            .map_err(|e| HeedError::Db(e.to_string()))?;

        for item in iter.take(limit) {
            let (key, value) = item.map_err(|e| HeedError::Db(e.to_string()))?;
            if let Some(msg) = decode_entry(key, value) {
                results.push(msg);
            }
        }
        Ok(results)
    }

    /// Read entries strictly newer than `after_ns` (newest-first within range).
    pub fn read_newer_than(&self, after_ns: u64, limit: usize) -> Result<Vec<LogMessage>> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| HeedError::Txn(e.to_string()))?;

        let mut results = Vec::with_capacity(limit);
        // rev_range(after_ns+1..=MAX) gives us newest entries > after_ns first
        let start = after_ns.saturating_add(1);
        let range = start..=u64::MAX;
        let iter = self
            .logs
            .rev_range(&rtxn, &range)
            .map_err(|e| HeedError::Db(e.to_string()))?;

        for item in iter.take(limit) {
            let (key, value) = item.map_err(|e| HeedError::Db(e.to_string()))?;
            if let Some(msg) = decode_entry(key, value) {
                results.push(msg);
            }
        }
        Ok(results)
    }

    /// List all known target names (from the targets set).
    pub fn get_targets(&self) -> Result<Vec<String>> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| HeedError::Txn(e.to_string()))?;

        let mut results = Vec::new();
        let iter = self
            .targets
            .iter(&rtxn)
            .map_err(|e| HeedError::Db(e.to_string()))?;

        for item in iter {
            let (key, _value) = item.map_err(|e| HeedError::Db(e.to_string()))?;
            results.push(key.to_owned());
        }
        Ok(results)
    }

    /// Delete all log entries with keys strictly less than `cutoff_ns`.
    /// Returns the count of deleted entries.
    pub fn delete_older_than(&self, cutoff_ns: u64) -> Result<usize> {
        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| HeedError::Txn(e.to_string()))?;

        let range = 0u64..cutoff_ns;
        let deleted = self
            .logs
            .delete_range(&mut wtxn, &range)
            .map_err(|e| HeedError::Db(e.to_string()))?;

        wtxn.commit().map_err(|e| HeedError::Txn(e.to_string()))?;
        Ok(deleted)
    }
}

// ── Helpers ────────────────────────────────────────────────────────────

fn decode_entry(key: u64, value: &[u8]) -> Option<LogMessage> {
    let mut msg: LogMessage = postcard::from_bytes(value).ok()?;
    msg.timestamp_nanos = key;
    Some(msg)
}

// ── Error type ─────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum HeedError {
    #[error("heed env error: {0}")]
    Env(String),
    #[error("heed txn error: {0}")]
    Txn(String),
    #[error("heed db error: {0}")]
    Db(String),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serde(String),
}

/// Convenience alias.
pub type Result<T, E = HeedError> = std::result::Result<T, E>;
