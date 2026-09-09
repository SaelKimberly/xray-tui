//! Headless reproduction of the subscription-update path against the real
//! 7000+ URL feed. #[ignore]d by default (needs network); run with:
//!   cargo test -p xray-tui --test e2e_real_feed -- --ignored --nocapture
use std::sync::Arc;
use std::time::Instant;

use xray_tui_config::import_export::ValidationSettings;

const FEED: &str = "https://raw.githubusercontent.com/kort0881/vpn-checker-backend/refs/heads/main/checked/RU_Best/ru_white_all_WHITE.txt";

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "network + real feed"]
async fn streaming_import_real_feed_completes() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let db = Arc::new(xray_tui_db::Database::in_memory().await.unwrap());
    let validation = ValidationSettings::default();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    let t0 = Instant::now();
    let resp = client.get(FEED).send().await.unwrap();
    let t_fetch = t0.elapsed();
    assert!(
        resp.status().is_success(),
        "fetch failed: {}",
        resp.status()
    );

    let (count, summary) = xray_tui::ops::stream_import::import_http_subscription(
        FEED,
        "xray-tui-e2e",
        &db,
        None,
        &validation,
    )
    .await;
    let t_all = t0.elapsed();
    println!(
        "links={count} errors={} fetch={t_fetch:?} total={t_all:?}",
        summary.total_errors
    );
    assert!(count > 5000, "expected thousands of links, got {count}");
    assert!(
        t_all.as_secs() < 600,
        "import took {:?} — frozen-path regression",
        t_all
    );
}
