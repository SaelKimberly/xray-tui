use heed::byteorder::BE;
use heed::types::{Bytes, Str, Unit};
use heed::{Database, Env, EnvOpenOptions};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// ── Constants ──────────────────────────────────────────────────────────
/// Default virtual map size: 1 GB virtual address reservation (not physical RAM).
/// Increased from 256 MB to reduce MapFull occurrences under sustained log output.
/// LMDB only uses physical pages for actually written data.
const DEFAULT_MAP_SIZE: usize = 1_073_741_824;

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
    /// Counter of MapFull events encountered (incremented instead of emitting tracing events).
    pub mapsize_full_count: AtomicU64,
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

        Ok(Self {
            env,
            logs,
            targets,
            mapsize_full_count: AtomicU64::new(0),
        })
    }

    /// Write a single log entry to the storage.
    ///
    /// Convenience wrapper around [`Self::write_log_batch`].
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
        self.write_log_batch(&[msg])
    }

    /// Write a batch of log entries in a single transaction.
    ///
    /// If the map is full, doubles the map size (up to 8 GB max) and retries once.
    /// On persistent failure, increments [`mapsize_full_count`] instead of emitting tracing events.
    pub fn write_log_batch(&self, messages: &[LogMessage]) -> Result<()> {
        if messages.is_empty() {
            return Ok(());
        }

        match self.try_write_batch(messages) {
            Ok(()) => Ok(()),
            Err(HeedError::MapFull) => {
                // MapFull — try to resize and retry once
                let current = self.env.info().map_size;
                let new_size = current.saturating_mul(2).min(8_589_934_592); // cap at 8 GB
                if new_size <= current {
                    self.mapsize_full_count.fetch_add(1, Ordering::Relaxed);
                    return Ok(()); // can't grow further, swallow
                }
                // SAFETY: no active transactions at this point (the failed write_txn was
                // already dropped when `try_write_batch` returned the MapFull error).
                unsafe { self.env.resize(new_size) }.map_err(|e| HeedError::Env(e.to_string()))?;
                // Retry once after resize
                if self.try_write_batch(messages).is_err() {
                    self.mapsize_full_count.fetch_add(1, Ordering::Relaxed);
                }
                Ok(())
            }
            Err(other) => Err(other),
        }
    }

    /// Internal: write a batch in one transaction. Returns [`HeedError::MapFull`] on MapFull.
    fn try_write_batch(&self, messages: &[LogMessage]) -> Result<()> {
        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| HeedError::Txn(e.to_string()))?;

        for msg in messages {
            let value = postcard::to_allocvec(msg).map_err(|e| HeedError::Serde(e.to_string()))?;

            if let Err(e) = self.logs.put(&mut wtxn, &msg.timestamp_nanos, &value) {
                return Err(match e {
                    heed::Error::Mdb(heed::MdbError::MapFull) => HeedError::MapFull,
                    other => HeedError::Db(other.to_string()),
                });
            }

            // Upsert target (put with existing key is a no-op for Unit values)
            if let Err(_e) = self.targets.put(&mut wtxn, &msg.target, &()) {
                // Non-critical — ignore individual target upsert failures
            }
        }

        wtxn.commit().map_err(|e| HeedError::Txn(e.to_string()))?;
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
    // ── Async wrappers (spawn_blocking for use from async context) ──

    /// Async version of [`Self::read_recent`] that wraps the heed call in `spawn_blocking`.
    pub async fn read_recent_async(self: &Arc<Self>, limit: usize) -> Result<Vec<LogMessage>> {
        let this = self.clone();
        tokio::task::spawn_blocking(move || this.read_recent(limit))
            .await
            .map_err(|e| HeedError::Io(e.to_string()))?
    }

    /// Async version of [`Self::read_newer_than`] that wraps the heed call in `spawn_blocking`.
    pub async fn read_newer_than_async(
        self: &Arc<Self>,
        after_ns: u64,
        limit: usize,
    ) -> Result<Vec<LogMessage>> {
        let this = self.clone();
        tokio::task::spawn_blocking(move || this.read_newer_than(after_ns, limit))
            .await
            .map_err(|e| HeedError::Io(e.to_string()))?
    }

    /// Async version of [`Self::read_older_than`] that wraps the heed call in `spawn_blocking`.
    pub async fn read_older_than_async(
        self: &Arc<Self>,
        before_ns: u64,
        limit: usize,
    ) -> Result<Vec<LogMessage>> {
        let this = self.clone();
        tokio::task::spawn_blocking(move || this.read_older_than(before_ns, limit))
            .await
            .map_err(|e| HeedError::Io(e.to_string()))?
    }

    /// Async version of [`Self::get_targets`] that wraps the heed call in `spawn_blocking`.
    pub async fn get_targets_async(self: &Arc<Self>) -> Result<Vec<String>> {
        let this = self.clone();
        tokio::task::spawn_blocking(move || this.get_targets())
            .await
            .map_err(|e| HeedError::Io(e.to_string()))?
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
    #[error("heed map full")]
    MapFull,
    #[error("I/O error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serde(String),
}

/// Convenience alias.
pub type Result<T, E = HeedError> = std::result::Result<T, E>;
