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
// ── Custom tracing layer that forwards events to the TUI event loop ──

/// Non-blocking tracing layer that sends log events through a channel
/// instead of writing to heed synchronously under the subscriber lock.
/// Optionally writes to a file when `log_to_file` is enabled.
struct TuiLogLayer {
    core_event_tx: tokio::sync::mpsc::Sender<xray_tui::CoreEvent>,
    log_sender: std::sync::mpsc::Sender<xray_tui_core::log_heed::LogMessage>,
    log_file: Option<std::sync::Mutex<std::fs::File>>,
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

        let timestamp_nanos = u64::try_from(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        )
        .unwrap_or(u64::MAX);

        // Optional file log — write JSON line when enabled
        if let Some(ref file_mutex) = self.log_file {
            use std::io::Write;
            if let Ok(mut file) = file_mutex.lock() {
                let escaped_msg = serde_json::to_string(&message).unwrap_or_else(|_| "null".into());
                let _ = writeln!(
                    file,
                    r#"{{"ts":{timestamp_nanos},"level":"{level}","target":"{target}","msg":{escaped_msg}}}"#,
                );
            }
        }

        // Non-blocking send to the log storage channel (batched, async writer).
        // If the channel is closed (writer panicked), silently drop — UI must keep running.
        let _ = self.log_sender.send(xray_tui_core::log_heed::LogMessage {
            level: level.clone(),
            target: target.clone(),
            message: message.clone(),
            timestamp_nanos,
        });

        // Send lightweight notification to TUI for the actions panel
        let _ = self.core_event_tx.try_send(xray_tui::CoreEvent::TuiLog {
            target,
            level,
            message,
        });
    }
}

// ── Application entry point ──

/// Install the process-level rustls `CryptoProvider` before anything can
/// build TLS. reqwest 0.13 is built with `rustls-no-provider` (workspace
/// standard: ring backend) and panics at `Client::build` unless a provider is
/// installed process-wide. Idempotent: a second call returns `Err` from
/// `install_default` and is ignored. rustls 0.23 also auto-selects when the
/// crate features name exactly one backend — the workspace pins ring only —
/// but reqwest checks `get_default()` explicitly and never auto-installs.
fn install_tls_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

#[tokio::main]
async fn main() -> Result<()> {
    // 0. Install rustls CryptoProvider first — every later TLS consumer
    //    (reqwest, QUIC ping adapter, hickory DoH) needs it.
    install_tls_provider();
    // 1. Load app config (~/.config/xray-tui/config.json) — returns Default on missing
    let config = AppConfig::load()?;

    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join("xray-tui");

    // 2. Open database (~/.config/xray-tui/data.db)
    let db_path = config_dir.join("data.db");
    let db = Database::open(&db_path).await?;
    // 3. Open heed log storage (~/.config/xray-tui/logs.lmdb)
    let log_path = config_dir.join("logs.lmdb");
    let heed = Arc::new(HeedLogStorage::new(&log_path)?);

    // 3b. Create channel for non-blocking log persistence and spawn background batched writer.
    //     TuiLogLayer sends messages via the channel; the background task batches up to
    //     100 messages per heed write_transaction and runs heed ops on the blocking pool.
    //     Uses unbounded std::sync::mpsc channel so TuiLogLayer::on_event never blocks.
    let (log_sender_tx, log_rx) = std::sync::mpsc::channel::<xray_tui_core::log_heed::LogMessage>();

    let writer_heed = heed.clone();
    let _writer_handle = tokio::task::spawn_blocking(move || {
        let mut batch: Vec<xray_tui_core::log_heed::LogMessage> = Vec::with_capacity(100);
        loop {
            // Wait for at least one message
            let Ok(msg) = log_rx.recv() else {
                // Channel closed (sender dropped) — flush and exit
                if !batch.is_empty() {
                    let _ = writer_heed.write_log_batch(&batch);
                }
                return;
            };
            let batch_deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
            batch.push(msg);
            // Drain until deadline or 100 entries
            while batch.len() < 100 {
                let remaining = batch_deadline.saturating_duration_since(std::time::Instant::now());
                match log_rx.recv_timeout(remaining) {
                    Ok(msg) => batch.push(msg),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => break,
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        let _ = writer_heed.write_log_batch(&batch);
                        return;
                    }
                }
            }
            let _ = writer_heed.write_log_batch(&batch);
            batch.clear();
        }
    });
    // 3c. Capture log config before moving config into AppState
    let log_to_file = config.logging.log_to_file;
    let log_file_path = config.logging.log_file_path.clone();

    // Canonicalize log file path to prevent path traversal
    let log_file_path = if log_to_file {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::Path::new(".").to_path_buf())
            .join("xray-tui");
        let p = Path::new(&log_file_path);
        // Reject any path with parent-dir traversal (`..`)
        if p.components().any(|c| c == std::path::Component::ParentDir) {
            tracing::warn!("log file path contains '..', falling back to default");
            config_dir.join("xray-tui.log")
        } else if p.is_relative() {
            config_dir.join(&log_file_path)
        } else {
            p.to_path_buf()
        }
    } else {
        Path::new(&log_file_path).to_path_buf()
    };

    let mut state = AppState::new(Arc::new(db), config).await;
    state.heed_storage = Some(heed.clone());
    state.log_sender_tx = Some(log_sender_tx.clone());
    // Create core process log channel (stdout/stderr lines from xray-core/sing-box subprocesses).
    let (core_log_tx, mut core_log_rx) = tokio::sync::mpsc::channel::<String>(512);
    state.core_log_tx = Some(core_log_tx);
    {
        let log_sender = log_sender_tx.clone();
        tokio::spawn(async move {
            while let Some(line) = core_log_rx.recv().await {
                let (target, level, message, _ts) =
                    xray_tui::parse_core_log_line(&line, xray_tui_core::CoreType::Auto);
                let msg = xray_tui_core::log_heed::LogMessage {
                    timestamp_nanos: u64::try_from(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos(),
                    )
                    .unwrap_or(u64::MAX),
                    level,
                    target,
                    message,
                };
                let _ = log_sender.send(msg);
            }
        });
    }
    // 4. Install tracing subscriber with TuiLogLayer (non-blocking channel send).
    //    Do not crash if global subscriber was already set (e.g., in tests)
    let log_file = if log_to_file {
        Some(std::sync::Mutex::new(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file_path)
                .expect("failed to open log file for appending"),
        ))
    } else {
        None
    };
    if tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                // Only xray_tui-* targets reach stderr via the fmt layer.
                // "log_worker" and "tui" targets reach TuiLogLayer (unfiltered)
                // for the Logs tab and actions panel.
                .with_filter(tracing_subscriber::EnvFilter::new("xray_tui=info")),
        )
        .with(
            TuiLogLayer {
                core_event_tx: state
                    .core_event_tx
                    .clone()
                    .expect("core_event_tx must be set before tracing init"),
                log_sender: log_sender_tx,
                log_file,
            }
            // Logs tab stays unfiltered (trace+) except hickory's DoH h2
            // transport chatter — those connections retry transparently and
            // spammed the dump (391 lines of "peer closed connection").
            .with_filter(tracing_subscriber::EnvFilter::new(
                "trace,hickory_net::h2=error",
            )),
        )
        .try_init()
        .is_err()
    {
        eprintln!("xray-tui: tracing subscriber already set — skipping TUI log layer");
    }

    // 5. Spawn TTL maintenance task for old logs (uses heed directly — infrequent)
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
            let now_nanos = u64::try_from(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos(),
            )
            .unwrap_or(u64::MAX);
            let cutoff = now_nanos.saturating_sub(u64::try_from(ttl_dur.as_nanos()).unwrap_or(0));
            // Spawn_blocking for heed delete operation
            let h = ttl_heed.clone();
            tokio::task::spawn_blocking(move || {
                if let Err(e) = h.delete_older_than(cutoff) {
                    // Can't use tracing inside spawn_blocking from a tokio task without
                    // the right context — but eprintln! is safe here (TTL runs rarely)
                    eprintln!("xray-tui: TTL cleanup error: {e}");
                }
            })
            .await
            .ok();
        }
    });
    // Panic hook to restore terminal on unexpected crashes
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = std::panic::catch_unwind(|| {
            let _ = crossterm::terminal::disable_raw_mode();
        });
        let _ = std::panic::catch_unwind(|| {
            let _ =
                crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
        });
        prev_hook(panic_info);
    }));

    // 6. Enter ratatui event loop
    xray_tui::ui::run(&mut state).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::install_tls_provider;

    /// Regression: release panic "Could not automatically determine the
    /// process-level `CryptoProvider` from Rustls crate" on Fast+Real ping.
    /// rustls is single-backend ring, but reqwest 0.13 `rustls-no-provider`
    /// requires an installed default before `Client::build`.
    #[test]
    fn tls_provider_installed_before_any_use() {
        install_tls_provider();
        assert!(
            rustls::crypto::CryptoProvider::get_default().is_some(),
            "startup install_tls_provider() must leave a default CryptoProvider installed"
        );
        // The exact panic site from the report must no longer fire.
        let _builder = rustls::ClientConfig::builder();
        // Idempotent: a second install returns Err, never panics.
        install_tls_provider();
    }
}
