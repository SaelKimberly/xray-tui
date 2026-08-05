//! Retry helpers for `SQLite` write contention.
//!
//! toasty transactions that collide with a concurrent writer surface
//! `serialization failure: database is locked`. The enrichment pipeline and
//! the ping-buffer flush write from many spawned tasks at once, so a busy DB
//! is normal under load — retry with backoff instead of dropping the write.

use std::future::Future;
use std::time::Duration;

use crate::error::{DatabaseError, Result};

/// True when the error is `SQLite` write contention: toasty's
/// serialization-failure classification, or the raw "database is locked"
/// driver message.
#[must_use]
pub fn is_busy_error(err: &DatabaseError) -> bool {
    matches!(err, DatabaseError::Toasty(e) if e.is_serialization_failure())
        || err.to_string().contains("database is locked")
}

/// Run `op`, retrying up to `attempts` extra times when it fails with `SQLite`
/// write contention, with exponential backoff (20ms doubling, 1.28s cap).
/// Non-busy errors pass through immediately, unchanged.
pub async fn retry_on_busy<T, F, Fut>(mut op: F, attempts: u32) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempt = 0u32;
    loop {
        match op().await {
            Err(err) if is_busy_error(&err) && attempt < attempts => {
                tokio::time::sleep(Duration::from_millis(20 << attempt.min(6))).await;
                attempt += 1;
            }
            other => return other,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::*;
    use crate::error::DatabaseError;

    fn busy() -> DatabaseError {
        DatabaseError::Toasty(toasty::Error::serialization_failure("database is locked"))
    }

    #[tokio::test]
    async fn retries_serialization_failures_then_succeeds() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let result = retry_on_busy(
            move || {
                let c = c.clone();
                async move {
                    let n = c.fetch_add(1, Ordering::SeqCst) + 1;
                    if n < 3 { Err(busy()) } else { Ok(42u32) }
                }
            },
            5,
        )
        .await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn gives_up_after_attempts() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let result = retry_on_busy(
            move || {
                let c = c.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>(busy())
                }
            },
            2,
        )
        .await;
        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 3); // 1 initial + 2 retries
    }

    #[tokio::test]
    async fn does_not_retry_non_busy_errors() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let result = retry_on_busy(
            move || {
                let c = c.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>(DatabaseError::Generic("boom".into()))
                }
            },
            5,
        )
        .await;
        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn classifies_busy_errors() {
        assert!(is_busy_error(&busy()));
        assert!(is_busy_error(&DatabaseError::Toasty(
            toasty::Error::from_args(format_args!(
                "transaction serialization failure: database is locked"
            ))
        )));
        assert!(!is_busy_error(&DatabaseError::Generic("boom".into())));
    }
}
