use anyhow::Result;
use std::path::Path;
use xray_tui::AppState;
use xray_tui_config::AppConfig;
use xray_tui_db::Database;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load app config (~/.config/xray-tui/config.json) — returns Default on missing
    let config = AppConfig::load()?;

    // 2. Open database (~/.config/xray-tui/data.db)
    let db_path = dirs::config_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join("xray-tui")
        .join("data.db");
    let db = Database::open(&db_path)?;

    // 3. Create shared state
    let mut state = AppState::new(db, config);

    // 4. Enter ratatui event loop
    xray_tui::ui::run(&mut state)?;

    Ok(())
}
