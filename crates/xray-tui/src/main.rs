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
use xray_tui_core::log_heed::HeedLogStorage;
use xray_tui_db::Database;

// ── Custom tracing layer that forwards events to the TUI event loop and heed storage ──

struct TuiLogLayer {
    core_event_tx: tokio::sync::mpsc::Sender<xray_tui::CoreEvent>,
    heed: Arc<HeedLogStorage>,
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

        let mut visitor = LogVisitor(String::new());
        event.record(&mut visitor);
        let message = visitor.0;
        let target = event.metadata().target().to_string();
        let level = event.metadata().level().to_string().to_lowercase();

        // Skip trace and debug levels — they are too verbose
        if level == "trace" || level == "debug" {
            return;
        }

        let timestamp_nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Write to heed storage (swallow errors — UI must keep running)
        let _ = self
            .heed
            .write_log(timestamp_nanos, &level, &target, &message);

        // Send lightweight notification to TUI for the actions panel
        // The logs tab reads from heed directly via polling.
        let _ = self.core_event_tx.try_send(xray_tui::CoreEvent::TuiLog {
            target,
            level,
            message,
        });
    }
}

// ── Application entry point ──

#[allow(
    clippy::significant_drop_tightening,
    reason = "trivial drop timing differences, adding explicit drops adds noise"
)]
#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load app config (~/.config/xray-tui/config.json) — returns Default on missing
    let config = AppConfig::load()?;

    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join("xray-tui");

    // 2. Open database (~/.config/xray-tui/data.db)
    let db_path = config_dir.join("data.db");
    let db = Database::open(&db_path).await?;
    db.normalize_all_remarks().await?;

    // 3. Open heed log storage (~/.config/xray-tui/logs.lmdb)
    let log_path = config_dir.join("logs.lmdb");
    let heed = Arc::new(HeedLogStorage::new(&log_path)?);

    let mut state = AppState::new(Arc::new(db), config).await;
    state.heed_storage = Some(heed.clone());

    // 4. Install tracing subscriber with TuiLogLayer
    // Do not crash if global subscriber was already set (e.g., in tests)
    if tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_filter(tracing_subscriber::EnvFilter::new("xray_tui=info")),
        )
        .with(TuiLogLayer {
            core_event_tx: state.core_event_tx.clone().unwrap(),
            heed: heed.clone(),
        })
        .try_init()
        .is_err()
    {
        eprintln!("xray-tui: tracing subscriber already set — skipping TUI log layer");
    }

    // 5. Spawn TTL maintenance task for old logs
    let ttl_heed = heed.clone();
    let ttl_dur = state.config.logging.ttl_secs.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_mins(10)); // 10 min
        interval.tick().await; // skip first tick
        loop {
            interval.tick().await;
            if ttl_dur.is_zero() {
                continue; // 0 = keep forever
            }
            let cutoff = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
                - (ttl_dur.as_nanos() as u64);
            if let Err(e) = ttl_heed.delete_older_than(cutoff) {
                tracing::error!(target: "log_worker", "TTL cleanup error: {e}");
            }
        }
    });

    // Panic hook to restore terminal on unexpected crashes
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
        prev_hook(panic_info);
    }));

    // 6. Enter ratatui event loop
    xray_tui::ui::run(&mut state).await?;

    Ok(())
}
