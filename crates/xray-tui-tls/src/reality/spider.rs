//! Spider-X: keep the established TLS session alive after a REALITY fallback.
//!
//! When the server fails REALITY auth (a real certificate — transparent
//! proxy / possible MITM), xray-core's `reality.go` `!Verified` behavior
//! keeps the session looking like a browsing session instead of closing
//! abruptly: this module is the engine's bounded implementation — HTTP/2
//! GETs to the real site so a DPI observer sees traffic, not a teardown.

use crate::Stream;
use crate::http2;
use crate::reality::SpiderConfig;
use crate::record::stream::TlsStream;

/// Padding cookie zeros (xray `SpiderY[0..1]`).
const PADDING_MAX: usize = 512;

/// Bounded Spider-X session: `max_gets` HTTP/2 GETs to `https://<sni><path>`
/// with a padding cookie, Referer chaining, and jittered delays. All errors
/// are swallowed — the caller already received its `RealityFallback` error
/// and this task owns the connection.
pub(crate) async fn run<S: Stream + 'static>(
    conn: TlsStream<S>,
    spider: SpiderConfig,
    sni: String,
) {
    let mut conn = conn;
    let mut client = http2::Client::new();
    let rng = ring::rand::SystemRandom::new();
    let mut prev_path: Option<String> = None;
    for idx in 0..spider.max_gets {
        let path = &spider.paths[idx % spider.paths.len()];
        let mut extra: Vec<(&str, String)> = Vec::new();
        // Padding cookie: `padding=0…0` (xray SpiderY, 0..=512 zeros).
        let pad =
            usize::try_from(crate::crypto::fingerprint::rand_u64(&rng) % (PADDING_MAX as u64 + 1))
                .unwrap_or(0);
        extra.push(("cookie", format!("padding={}", "0".repeat(pad))));
        // Referer chain: each later request refers to the previous path.
        if let Some(prev) = &prev_path {
            extra.push(("referer", format!("https://{sni}{prev}")));
        }
        let extra_refs: Vec<(&str, &str)> = extra.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let result = client.get(&mut conn, path, &sni, &extra_refs).await;
        prev_path = Some(path.clone());
        if result.is_err() {
            break;
        }
        tokio::time::sleep(spider.request_interval).await;
    }
}
