//! Normalize every subscription source to a streaming shape.
//!
//! Fetch bytes in bounded chunks, gather complete share-URLs into batches,
//! parse + persist each batch immediately (partial-load semantics — a source
//! dying mid-update keeps everything already stored). One shared pipeline
//! serves HTTP today and streaming sources (e.g. Telegram) later.

use std::sync::Arc;
use std::time::Duration;

use xray_tui_config::import_export::{ValidationSettings, ValidationSummary};
use xray_tui_db::Database;

use std::future::Future;
use std::pin::Pin;

use crate::ops::subscriptions::PERSIST_CHUNK;

///
/// `Err` = the source failed mid-stream (network drop, channel closed, …).
///
/// The import loop treats it as a premature but LEGAL end: everything
/// batched so far is already stored, so the error is logged and the run
/// finishes with partial results instead of being discarded.
pub type SourceBatch = Result<bytes::Bytes, String>;

/// Streaming byte source — the seam every source kind normalizes to.
///
/// HTTP subscriptions wrap `Response::chunk()`; a Telegram-channel source
/// later wraps its update poller. Each `next_chunk` returns an arbitrarily
/// sized piece of the logical stream (bounded above by the source, not by
/// the decoder — the batcher re-chunks).
pub trait UrlSource: Send {
    fn next_chunk(&mut self) -> Pin<Box<dyn Future<Output = Option<SourceBatch>> + Send + '_>>;
}

/// HTTP-backed source: one response body, streamed.
pub struct HttpSource {
    response: reqwest::Response,
}

impl HttpSource {
    #[must_use]
    pub const fn new(response: reqwest::Response) -> Self {
        Self { response }
    }
}

impl UrlSource for HttpSource {
    fn next_chunk(&mut self) -> Pin<Box<dyn Future<Output = Option<SourceBatch>> + Send + '_>> {
        Box::pin(async {
            match self.response.chunk().await {
                Ok(Some(chunk)) => Some(Ok(chunk)),
                Ok(None) => None,
                Err(e) => Some(Err(e.to_string())),
            }
        })
    }
}

/// Collect a stream of arbitrary byte chunks into complete share-URLs,
/// drained in batches of `batch_size` URLs. Wraps the existing
/// [`xray_tui_config::subscription::StreamingDecoder`] so base64 and
/// plain-text bodies work unchanged.
struct UrlBatcher {
    decoder: xray_tui_config::subscription::StreamingDecoder,
    queue: std::collections::VecDeque<String>,
    batch_size: usize,
    /// Staging buffer: the decoder's encoding auto-detection locks state on
    /// its first aligned segment, so it must see LARGE feeds — a tiny feed
    /// like `vmes` is valid base64 and would corrupt plain-text decoding.
    /// Feeding whole `INPUT_CHUNK_SIZE` slices reproduces the buffered
    /// path's behavior exactly.
    staging: Vec<u8>,
}

impl UrlBatcher {
    fn new(batch_size: usize) -> Self {
        Self {
            decoder: xray_tui_config::subscription::StreamingDecoder::new(),
            queue: std::collections::VecDeque::new(),
            batch_size,
            staging: Vec::with_capacity(xray_tui_config::subscription::INPUT_CHUNK_SIZE * 2),
        }
    }

    fn feed(&mut self, chunk: &[u8]) -> Result<(), String> {
        self.staging.extend_from_slice(chunk);
        let feed_size = xray_tui_config::subscription::INPUT_CHUNK_SIZE;
        while self.staging.len() >= feed_size {
            let urls = self.decoder.feed(&self.staging[..feed_size])?;
            self.queue.extend(urls);
            self.staging.drain(..feed_size);
        }
        Ok(())
    }

    /// Drain one batch of complete URLs (shorter only at end-of-stream).
    fn take_batch(&mut self) -> Option<Vec<String>> {
        if self.queue.is_empty() {
            return None;
        }
        let take = self.batch_size.min(self.queue.len());
        Some(self.queue.drain(..take).collect())
    }

    /// Flush the staging remainder + decoder carry-over at end-of-stream.
    fn finalize(&mut self) -> Result<(), String> {
        if !self.staging.is_empty() {
            let urls = self.decoder.feed(&self.staging)?;
            self.queue.extend(urls);
            self.staging.clear();
        }
        let urls = self.decoder.finalize()?;
        self.queue.extend(urls);
        Ok(())
    }
}

/// Run a full streaming import: gather URL batches off `source`, parse +
/// persist each batch immediately, deliver progress through `on_progress`.
/// Returns `(links persisted, whole-run summary)`.
///
/// Partial-load semantics: a source error mid-stream ends the run with
/// everything already committed; only a failure INSIDE a batch persist is
/// logged-and-skipped per batch (same as the buffered path).
pub async fn run_streaming_import<F>(
    source: &mut dyn UrlSource,
    db: &Arc<Database>,
    group_id: Option<&str>,
    validation: &ValidationSettings,
    batch_size: usize,
    on_progress: Option<&F>,
) -> (usize, ValidationSummary)
where
    F: Fn(usize) + Send + Sync,
{
    // Bound each source chunk: a hung source (open socket, no data) must
    // degrade to a partial result, never hang the import.
    const CHUNK_TIMEOUT: Duration = Duration::from_secs(120);
    let mut batcher = UrlBatcher::new(batch_size);
    let mut summary = ValidationSummary::default();
    let mut count = 0usize;
    // Deterministic-id dedup across the WHOLE run (protocol rows are shared
    // across endpoints; feeds repeat one protocol config over many servers).
    let mut seen_protocols = std::collections::HashSet::new();
    let mut seen_endpoints = std::collections::HashSet::new();
    let mut seen_links = std::collections::HashSet::new();

    loop {
        // Gather until one batch is ready or the stream ends.
        let batch = loop {
            // Drain whatever the queue already satisfies.
            if let Some(batch) = batcher.take_batch() {
                break Some(batch);
            }
            let Ok(next) = tokio::time::timeout(CHUNK_TIMEOUT, source.next_chunk()).await else {
                tracing::warn!(
                    target: "tui::ops::subscriptions",
                    "Source chunk timed out after {CHUNK_TIMEOUT:?} — finishing import with partial results"
                );
                break None;
            };
            match next {
                None => break None, // clean end-of-stream
                Some(Err(e)) => {
                    tracing::warn!(
                        target: "tui::ops::subscriptions",
                        "Source ended early: {e} — keeping already-stored batches"
                    );
                    break None;
                }
                Some(Ok(chunk)) => {
                    if let Err(e) = batcher.feed(&chunk) {
                        tracing::error!(
                            target: "tui::ops::subscriptions",
                            "Source data could not be decoded: {e} — keeping already-stored batches"
                        );
                        break None;
                    }
                }
            }
        };

        let Some(batch) = batch else {
            // End of stream (clean or partial): flush the decoder tail, then
            // persist every remaining batch until the queue is drained.
            if let Err(e) = batcher.finalize() {
                // trailing carry-over bytes undecodable — queued URLs still valid
                tracing::debug!(target: "tui::ops::subscriptions", "Finalize had undecodable trailing bytes: {e}");
            }
            while let Some(tail) = batcher.take_batch() {
                let (n, s) = persist_batch(
                    db,
                    &tail,
                    group_id,
                    validation,
                    &mut seen_protocols,
                    &mut seen_endpoints,
                    &mut seen_links,
                )
                .await;
                count += n;
                summary.merge(&s);
                if let Some(cb) = on_progress {
                    cb(count);
                }
            }
            break;
        };

        let (n, s) = persist_batch(
            db,
            &batch,
            group_id,
            validation,
            &mut seen_protocols,
            &mut seen_endpoints,
            &mut seen_links,
        )
        .await;
        count += n;
        summary.merge(&s);
        if let Some(cb) = on_progress {
            cb(count);
        }
        tokio::task::yield_now().await;
    }

    (count, summary)
}

/// Parse one URL batch and persist it via the bulk upserts (the exact body of
/// one chunk iteration from `persist_parsed_urls`, reused so both paths share
/// dedup + error semantics).
async fn persist_batch(
    db: &Arc<Database>,
    batch: &[String],
    group_id: Option<&str>,
    validation: &ValidationSettings,
    seen_protocols: &mut std::collections::HashSet<i64>,
    seen_endpoints: &mut std::collections::HashSet<i64>,
    seen_links: &mut std::collections::HashSet<(i64, i64)>,
) -> (usize, ValidationSummary) {
    let (profiles, batch_summary) =
        xray_tui_config::subscription::parse_url_batch(batch, validation);

    let mut endpoints: Vec<xray_tui_db::models::Endpoint> = Vec::new();
    let mut protocols: Vec<xray_tui_db::models::Protocol> = Vec::new();
    let mut links: Vec<xray_tui_db::models::ProfileStats> = Vec::new();
    let mut group_links: Vec<xray_tui_db::models::EndpointGroup> = Vec::new();
    let mut batch_links = 0usize;

    for profile in &profiles {
        let parsed = &profile.parsed;
        if parsed.endpoints.is_empty() {
            continue;
        }
        let protocol = crate::state::protocol_from_parsed(parsed);
        if seen_protocols.insert(protocol.id.get()) {
            protocols.push(protocol.clone());
        }
        for ep in &parsed.endpoints {
            let endpoint = crate::state::endpoint_from_essentials(ep);
            let link = crate::state::link_from_parsed_with_id(parsed, protocol.id, endpoint.id);
            if seen_links.insert((link.protocol_id.get(), link.endpoint_id.get())) {
                if seen_endpoints.insert(endpoint.id.get()) {
                    endpoints.push(endpoint.clone());
                }
                links.push(link.clone());
                if let Some(gid) = group_id {
                    group_links.push(xray_tui_db::models::EndpointGroup {
                        endpoint_id: endpoint.id,
                        group_id: gid.to_owned(),
                        last_seen_at: link.last_seen_at,
                        sort_order: None,
                        endpoint: toasty::Deferred::default(),
                        group: toasty::Deferred::default(),
                    });
                }
                batch_links += 1;
            }
        }
    }

    // One transaction for the WHOLE batch: a crash mid-batch never leaves
    // half a batch stored, and the four bulk families share one commit.
    // Move the row Vecs into Arc slices: each retry attempt clones the Arc
    // (refcount bump) instead of deep-copying the batch contents.
    let endpoints = Arc::from(endpoints);
    let protocols = Arc::from(protocols);
    let links = Arc::from(links);
    let group_links = Arc::from(group_links);
    let persist = || {
        let (endpoints, protocols, links, group_links) = (
            Arc::clone(&endpoints),
            Arc::clone(&protocols),
            Arc::clone(&links),
            Arc::clone(&group_links),
        );
        async move {
            let mut conn = db.connection().await?;
            let mut tx = conn.transaction().await?;
            xray_tui_db::upsert_endpoints_bulk(&mut tx, &endpoints).await?;
            xray_tui_db::upsert_protocols_bulk(&mut tx, &protocols).await?;
            xray_tui_db::upsert_links_bulk(&mut tx, &links).await?;
            xray_tui_db::upsert_endpoint_group_links_bulk(&mut tx, &group_links).await?;
            tx.commit().await?;
            Ok(())
        }
    };
    match xray_tui_db::retry_on_busy(persist, 5).await {
        Err(e) => {
            tracing::error!(
                target: "tui::ops::subscriptions",
                "bulk persist failed for a {}-URL batch: {e}",
                batch.len(),
            );
            (0, batch_summary)
        }
        Ok(()) => (batch_links, batch_summary),
    }
}

/// HTTP convenience: fetch a subscription URL and stream it through
/// [`run_streaming_import`]. Equivalent to the old buffered path but never
/// holds the whole body in memory.
pub async fn import_http_subscription(
    url: &str,
    user_agent: &str,
    db: &Arc<Database>,
    group_id: Option<&str>,
    validation: &ValidationSettings,
) -> (usize, ValidationSummary) {
    let client = match reqwest::Client::builder()
        .user_agent(user_agent)
        .timeout(Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(target: "tui::ops::subscriptions", "HTTP client build failed: {e}");
            return (0, ValidationSummary::default());
        }
    };
    let response = match client.get(url).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(target: "tui::ops::subscriptions", "HTTP fetch failed: {e}");
            return (0, ValidationSummary::default());
        }
    };
    run_streaming_import(
        &mut HttpSource::new(response),
        db,
        group_id,
        validation,
        PERSIST_CHUNK,
        None::<&fn(usize)>,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use xray_tui_config::import_export::ValidationSettings;

    fn valid_vmess_url(host: &str) -> String {
        let qr = serde_json::json!({
            "v": "2", "ps": "test", "add": host, "port": "443",
            "id": "550e8400-e29b-41d4-a716-446655440000", "aid": "0", "scy": "auto",
            "net": "tcp", "type": "none", "host": "", "path": "", "tls": "",
            "sni": "", "alpn": "", "fp": "", "insecure": "0",
        });
        let b64 = base64_simd::STANDARD.encode_to_string(serde_json::to_string(&qr).unwrap());
        format!("vmess://{b64}")
    }

    /// Source that replays pre-split byte chunks, then optionally fails.
    struct ScriptedSource {
        chunks: std::vec::IntoIter<Result<bytes::Bytes, String>>,
    }

    impl UrlSource for ScriptedSource {
        fn next_chunk(&mut self) -> Pin<Box<dyn Future<Output = Option<SourceBatch>> + Send + '_>> {
            Box::pin(async { self.chunks.next() })
        }
    }

    #[tokio::test]
    async fn streaming_import_splits_and_persists_in_batches() {
        let db = Arc::new(Database::in_memory().await.expect("db"));
        let validation = ValidationSettings::default();
        // 4 hosts, 3-byte chunks to force multi-chunk URLs through the batcher.
        let body = ["1.2.3.4", "5.6.7.8", "9.10.11.12", "13.14.15.16"]
            .iter()
            .map(|h| valid_vmess_url(h))
            .collect::<Vec<_>>()
            .join("\n");
        let byte_chunks: Vec<Result<bytes::Bytes, String>> = body
            .as_bytes()
            .chunks(3)
            .map(|c| Ok(bytes::Bytes::copy_from_slice(c)))
            .collect();
        let mut source = ScriptedSource {
            chunks: byte_chunks.into_iter(),
        };

        let (count, summary) = run_streaming_import(
            &mut source,
            &db,
            Some("g1"),
            &validation,
            2,
            None::<&fn(usize)>,
        )
        .await;
        assert_eq!(count, 4, "one link per unique host");
        assert_eq!(summary.total_errors, 0);
        let rows = db
            .get_active_endpoints_by_group("g1", jiff::Timestamp::from_second(0).unwrap())
            .await
            .expect("rows");
        assert_eq!(rows.len(), 4);
    }

    #[tokio::test]
    async fn streaming_import_keeps_batches_when_source_dies_midway() {
        let db = Arc::new(Database::in_memory().await.expect("db"));
        let validation = ValidationSettings::default();
        let body = ["21.22.23.24", "25.26.27.28", "29.30.31.32"]
            .iter()
            .map(|h| valid_vmess_url(h))
            .collect::<Vec<_>>()
            .join("\n");
        // Batch size 2 with 4 URL-sized chunks: first TWO URLs persist, then
        // the source errors. Partial-load semantics keep the stored batch.
        let mut source = ScriptedSource {
            chunks: vec![
                Ok(bytes::Bytes::from(valid_vmess_url("21.22.23.24"))),
                Ok(bytes::Bytes::from(valid_vmess_url("25.26.27.28"))),
                Err("connection reset".to_string()),
                Ok(bytes::Bytes::from(valid_vmess_url("29.30.31.32"))),
            ]
            .into_iter(),
        };

        let (count, _) = run_streaming_import(
            &mut source,
            &db,
            Some("g1"),
            &validation,
            2,
            None::<&fn(usize)>,
        )
        .await;
        assert_eq!(count, 2, "exactly the two URLs before the failure persist");
        let rows = db
            .get_active_endpoints_by_group("g1", jiff::Timestamp::from_second(0).unwrap())
            .await
            .expect("rows");
        assert_eq!(rows.len(), 2, "no rows after the failure point");
    }
}
