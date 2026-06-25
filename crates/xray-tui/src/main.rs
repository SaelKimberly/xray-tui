#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    reason = "known-safe casts on port/len/display"
)]
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tracing::field::{Field, Visit};
use tracing_subscriber::layer::Layer as _;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use xray_tui::AppState;
use xray_tui_config::AppConfig;
use xray_tui_db::Database;

// ── Custom tracing layer that forwards events to the TUI event loop and storage ──

struct TuiLogLayer {
    core_event_tx: tokio::sync::mpsc::Sender<xray_tui::CoreEvent>,
    log_worker_tx: tokio::sync::mpsc::Sender<xray_tui_core::log_worker::LogWorkerMessage>,
}

/// A field visitor that captures the `message` field.
struct LogVisitor(String);

impl Visit for LogVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        }
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.0 = value.to_string();
        }
    }
}

impl<S> tracing_subscriber::Layer<S> for TuiLogLayer
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        use std::time::SystemTime;
        use xray_tui_core::log_worker::LogWorkerMessage;
        use xray_tui_db::LogEntry;

        let mut visitor = LogVisitor(String::new());
        event.record(&mut visitor);
        let message = visitor.0;
        let target = event.metadata().target().to_string();
        let level = event.metadata().level().to_string().to_lowercase();

        // Skip trace and debug levels — they are too verbose for both
        // the TUI log buffer and persistent storage.
        if level == "trace" || level == "debug" {
            return;
        }
        // Determine source: validation crate logs tagged specially
        let source = if target.starts_with("xray_tui_config") || target == "validation" {
            "validation"
        } else {
            "tui"
        };

        // Send to TUI display (non-blocking, unbounded)
        let _ = self.core_event_tx.try_send(xray_tui::CoreEvent::TuiLog {
            target: target.clone(),
            level: level.clone(),
            message: message.clone(),
        });

        // Send to persistent storage (non-blocking, unbounded)
        let timestamp_nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;
        let entry = LogEntry {
            id: None,
            timestamp_nanos,
            level,
            target,
            message,
            metadata_json: None,
            source: source.to_owned(),
        };
        let _ = self.log_worker_tx.try_send(LogWorkerMessage::Entry(entry));
    }
}

// ── Application entry point ──

#[allow(clippy::significant_drop_tightening, reason = "trivial drop timing differences, adding explicit drops adds noise")]
#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load app config (~/.config/xray-tui/config.json) — returns Default on missing
    let config = AppConfig::load()?;

    // 2. Open database (~/.config/xray-tui/data.db)
    let db_path = dirs::config_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join("xray-tui")
        .join("data.db");
    let db = Database::open(&db_path).await?;
    db.normalize_all_remarks().await?;
    let mut state = AppState::new(Arc::new(db), config).await;

    // 4. Create channel for log storage worker
    let (log_worker_tx, log_worker_rx) = tokio::sync::mpsc::channel(1024);

    // 5. Install tracing subscriber with TuiLogLayer
    // Do not crash if global subscriber was already set (e.g., in tests)
    if tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_filter(tracing_subscriber::EnvFilter::new("xray_tui=info")),
        )
        .with(TuiLogLayer {
            core_event_tx: state.core_event_tx.clone().unwrap(),
            log_worker_tx: log_worker_tx.clone(),
        })
        .try_init()
        .is_err()
    {
        eprintln!("xray-tui: tracing subscriber already set — skipping TUI log layer");
    }

    // 6. Pass log_worker_tx to AppState (for non-tracing log sources)
    state.log_worker_tx = Some(log_worker_tx.clone());

    // 7. Spawn LogStorageWorker background task with dedicated connection
    let log_conn = state
        .db
        .new_connection()
        .await
        .expect("Failed to create log storage connection");
    tokio::spawn(
        xray_tui_core::log_worker::LogStorageWorker::new(
            log_worker_rx,
            log_conn,
            state.config.logging.batch_size,
        )
        .run(),
    );
    // 8. Spawn TTL maintenance task for old logs
    let db_ttl = state.db.clone();
    let ttl_hours = state.config.logging.ttl_hours;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_hours(1));
        interval.tick().await; // skip first tick
        loop {
            interval.tick().await;
            if ttl_hours == 0 {
                continue; // 0 = keep forever
            }
            let cutoff = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as i64
                - (ttl_hours as i64 * 3_600_000_000_000);
            if let Err(e) = xray_tui_db::LogRepository::new(db_ttl.connection())
                .delete_older_than(cutoff)
                .await
            {
                tracing::error!(target: "log_worker", "TTL cleanup error: {e}");
            }
        }
    });

    // 5. Enter ratatui event loop
    xray_tui::ui::run(&mut state).await?;

    Ok(())
}
