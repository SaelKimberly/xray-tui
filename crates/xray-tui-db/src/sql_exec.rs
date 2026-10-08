//! One SQL-execution seam over a toasty executor or a raw turso connection.
//!
//! The toasty driver routes EVERY statement through the connection's
//! `prepare_cached`, whose map (`turso_sdk_kit::rsapi::TursoConnection::
//! cached_statements`) is **unbounded and keyed by SQL text**: each distinct
//! statement is compiled (`Arc<PreparedProgram>`) and retained for the
//! connection's life. Our bulk writers INLINE their literals (ids, values,
//! timestamps) for the engine's bind cost (~0.8 ms/parameter, C6), so every
//! window is a new text — a fresh compiled program cached forever. Measured:
//! an identical statement saturates (`+1.7 MB` flat); a varying one grows
//! linearly (`~130 KiB`/call on a 20-link window, `~310 KiB`/call on a 200-id
//! page read).
//!
//! The raw `query`/`execute` methods use the UNCACHED `prepare`, so
//! running those statements on a raw connection keeps the inlining perf and
//! drops the retention — this trait is the seam that lets one helper body
//! serve both: the toasty path (for statements whose text is stable, and for
//! typed statements) and the raw path (for the inlined-literal writers).

use toasty_core::stmt::{Value, ValueRecord};

/// Run raw SQL on either a toasty executor (pooled, statement-cached) or a raw
/// `turso::Connection` (uncached `prepare`).
///
/// The futures are declared `+ Send` (the shape `CacheSpec` uses) so a caller
/// awaiting one of these inside a `tokio::spawn`ed future keeps ITS future
/// `Send`; the impls may still be `async fn`, which the compiler checks against
/// the bound.
pub trait SqlConn: Send {
    /// Execute a statement that returns no rows (`INSERT`/`UPDATE`/`DELETE`/DDL).
    fn exec_sql(&mut self, sql: String) -> impl Future<Output = crate::Result<()>> + Send;
    /// Execute a query and return every row as a `Value::Record` (the shape the
    /// toasty raw-SQL path and [`crate::profiles_query`] both decode).
    fn query_sql(&mut self, sql: String) -> impl Future<Output = crate::Result<Vec<Value>>> + Send;
}

/// Blanket impl: any toasty executor (a `Connection`, a `Transaction`, `Db`)
/// is a `SqlConn`. This is what lets the existing helpers — whose callers pass
/// a generic `&mut impl toasty::Executor` — keep working unchanged while the
/// same helpers are also callable on a raw connection.
impl<E: toasty::Executor> SqlConn for E {
    async fn exec_sql(&mut self, sql: String) -> crate::Result<()> {
        toasty::sql::statement(sql).exec(self).await?;
        Ok(())
    }

    async fn query_sql(&mut self, sql: String) -> crate::Result<Vec<Value>> {
        Ok(toasty::sql::query(sql).exec(self).await?)
    }
}

/// A raw, UNCACHED turso connection (the app's `Database::direct`).
pub(crate) struct RawConn<'a>(pub &'a turso::Connection);

impl SqlConn for RawConn<'_> {
    async fn exec_sql(&mut self, sql: String) -> crate::Result<()> {
        self.0.execute(sql, ()).await.map_err(raw_turso_error)?;
        Ok(())
    }

    async fn query_sql(&mut self, sql: String) -> crate::Result<Vec<Value>> {
        let mut rows = self.0.query(sql, ()).await.map_err(raw_turso_error)?;
        let mut out = Vec::new();
        while let Some(row) = rows.next().await.map_err(raw_turso_error)? {
            let mut items = Vec::with_capacity(row.column_count());
            for index in 0..row.column_count() {
                let value = row.get_value(index).map_err(raw_turso_error)?;
                items.push(from_turso_value(value));
            }
            out.push(Value::Record(ValueRecord::from_vec(items)));
        }
        Ok(out)
    }
}

/// Classify a raw turso error exactly as `toasty_driver_turso::error::
/// classify_turso_error` does, so `retry_on_busy` RECOGNIZES raw-path
/// contention.
///
/// [`crate::export::turso_error`] maps everything to `DatabaseError::Generic`,
/// which `is_busy_error` rejects — so an MVCC commit conflict or a WAL `BEGIN`
/// wait on the raw connection would be a PERMANENT failure that re-stages the
/// window instead of retrying, the one class [`crate::retry_on_busy`] exists
/// for. `Busy`/`BusySnapshot` and a `"conflict"` message map to a toasty
/// serialization failure, which `is_busy_error` accepts.
pub(crate) fn raw_turso_error(err: turso::Error) -> crate::DatabaseError {
    match err {
        turso::Error::Busy(msg) | turso::Error::BusySnapshot(msg) => {
            crate::DatabaseError::Toasty(toasty::Error::serialization_failure(msg))
        }
        turso::Error::Error(msg) if msg.contains("conflict") => {
            crate::DatabaseError::Toasty(toasty::Error::serialization_failure(msg))
        }
        other => crate::export::turso_error(other),
    }
}

/// turso `Value` → toasty `Value` by SQLite storage class — the mirror of the
/// driver's own `from_turso_infer` for a raw statement (`SqlReturn::Infer`).
pub(crate) fn from_turso_value(v: turso::Value) -> Value {
    match v {
        turso::Value::Null => Value::Null,
        turso::Value::Integer(n) => Value::I64(n),
        turso::Value::Real(f) => Value::F64(f),
        turso::Value::Text(s) => Value::String(s),
        turso::Value::Blob(b) => Value::Bytes(b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The raw path's contention errors must be RETRYABLE. `retry_on_busy`
    /// accepts a toasty serialization failure; turso surfaces MVCC commit
    /// conflicts and WAL `BEGIN` waits as `Busy`/`BusySnapshot` (or a
    /// `"conflict"` message), which the blanket `export::turso_error` would
    /// flatten into an opaque `Generic` — a permanent failure that re-stages
    /// the write instead of retrying it.
    #[test]
    fn raw_busy_errors_are_retryable() {
        for err in [
            turso::Error::Busy("busy".to_string()),
            turso::Error::BusySnapshot("busy snapshot".to_string()),
            turso::Error::Error("write conflict".to_string()),
        ] {
            assert!(
                crate::retry::is_busy_error(&raw_turso_error(err)),
                "contention must classify as retryable"
            );
        }
        assert!(
            !crate::retry::is_busy_error(&raw_turso_error(turso::Error::Error(
                "no such table: nope".to_string()
            ))),
            "a genuine error is not retryable"
        );
    }
}
