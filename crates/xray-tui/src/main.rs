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

// ── Custom tracing layer that forwards events to the TUI event loop ──

struct TuiLogLayer {
    tx: tokio::sync::mpsc::UnboundedSender<String>,
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
        let mut visitor = LogVisitor(String::new());
        event.record(&mut visitor);
        let message = visitor.0;
        let target = event.metadata().target().to_string();
        let level = event.metadata().level().to_string();
        let payload = serde_json::json!({
            "target": target,
            "level": level.to_lowercase(),
            "message": message,
        });
        let _ = self.tx.send(payload.to_string());
    }
}

// ── Application entry point ──

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

    // 4. Install tracing subscriber with TuiLogLayer
    let (tui_log_tx, mut tui_log_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    // Do not crash if global subscriber was already set (e.g., in tests)
    if tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_filter(tracing_subscriber::EnvFilter::new("xray_tui=info")),
        )
        .with(TuiLogLayer {
            tx: tui_log_tx.clone(),
        })
        .try_init()
        .is_err()
    {
        eprintln!("xray-tui: tracing subscriber already set — skipping TUI log layer");
    }

    // Spawn task to relay TUI log events into the core event channel
    if let Some(core_tx) = state.core_event_tx.clone() {
        tokio::spawn(async move {
            while let Some(line) = tui_log_rx.recv().await {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&line) {
                    let target = parsed["target"].as_str().unwrap_or("tui").to_string();
                    let level = parsed["level"].as_str().unwrap_or("info").to_string();
                    let message = parsed["message"].as_str().unwrap_or("").to_string();
                    let _ = core_tx.send(xray_tui::CoreEvent::TuiLog {
                        target,
                        level,
                        message,
                    });
                }
            }
        });
    }

    // 5. Enter ratatui event loop
    xray_tui::ui::run(&mut state).await?;

    Ok(())
}
