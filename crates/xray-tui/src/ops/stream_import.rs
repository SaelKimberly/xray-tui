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

/// Dial deadline for a subscription fetch.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Idle deadline for ONE body read, reset by every frame the source yields.
///
/// The body outlives any total deadline worth setting (a 26 MB / 170k-URL
/// feed), and this loop persists a batch between two reads, so a total
/// deadline would also bill the consumer's own DB work to the network
/// budget. Kept BELOW the loop's `CHUNK_TIMEOUT` so a stalled body surfaces
/// as the HTTP error (status + context), not as the loop's generic stall
/// warning.
const READ_TIMEOUT: Duration = Duration::from_secs(60);

/// Result of one streaming import run.
///
/// `ended_early` carries the source's failure text when the stream died
/// before its natural end. The rows already stored are KEPT (partial-load
/// semantics), but the run was NOT clean and the caller must say so — a
/// truncated import used to be indistinguishable from a complete one.
#[derive(Debug, Default)]
#[must_use]
pub struct ImportOutcome {
    /// Links STORED. Writes are deferred to the write-behind driver, so this
    /// is what the end-of-import flush reported, NOT the parsed count — the
    /// "only what was STORED" invariant.
    pub links: usize,
    pub summary: ValidationSummary,
    pub ended_early: Option<String>,
}

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
///
/// Returns the links persisted, the whole-run summary, and — when the source
/// died before its natural end — the failure text.
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
    budget: xray_tui_config::app_config::ImportConfig,
) -> ImportOutcome
where
    F: Fn(usize) + Send + Sync,
{
    // Bound each source chunk: a hung source (open socket, no data) must
    // degrade to a partial result, never hang the import.
    const CHUNK_TIMEOUT: Duration = Duration::from_secs(120);
    // Aggregate budgets for one feed, now CONFIGURABLE
    // (`specs/2026-10-02-import-budget-design.md`). reqwest transparently
    // decodes gzip, so `chunk.len()` is the DECODED size: a feed host answering
    // with an unbounded body (or an endless stream) must be cut off, and a feed
    // that yields unbounded rows must stop too (finding f12). `0` disables.
    //
    // The defaults are the constants that used to live here, so a config with no
    // `import` section behaves exactly as before.
    let max_feed_bytes = if budget.max_feed_bytes == 0 {
        usize::MAX
    } else {
        usize::try_from(budget.max_feed_bytes).unwrap_or(usize::MAX)
    };
    let max_feed_links = if budget.max_feed_links == 0 {
        usize::MAX
    } else {
        budget.max_feed_links
    };
    let mut feed_bytes = 0usize;
    let mut batcher = UrlBatcher::new(batch_size);
    let mut summary = ValidationSummary::default();
    // Parsed-link total: what the run STAGED, which is what `on_progress`
    // reports (parsed, not yet necessarily stored). Reconciled against the
    // final flush's `stored` below.
    let mut staged_count = 0usize;
    // Mint of the per-batch staging ids. Monotonic and unique: two batches
    // pending at once must be two pending-map entries, or the later one
    // overwrites the earlier and its rows are never written.
    let mut next_seq = 0u64;
    let driver = import_driver(db);
    // Set by every non-natural end of the stream (stall, source error,
    // undecodable bytes): the loop still drains what it has, but the caller
    // learns the run was cut short.
    let mut ended_early: Option<String> = None;
    // URLs that parsed but whose batch could not be persisted after every
    // retry. Distinct from `links`, which counts only what was STORED, and
    // from the budget paths: a dropped batch is silent data loss, so it must
    // surface through `ended_early` like any other short run.
    let mut dropped_total = 0usize;
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
                ended_early = Some(format!("source stalled: no chunk within {CHUNK_TIMEOUT:?}"));
                break None;
            };
            match next {
                None => break None, // clean end-of-stream
                Some(Err(e)) => {
                    tracing::warn!(
                        target: "tui::ops::subscriptions",
                        "Source ended early: {e} — keeping already-stored batches"
                    );
                    ended_early = Some(e);
                    break None;
                }
                Some(Ok(chunk)) => {
                    feed_bytes = feed_bytes.saturating_add(chunk.len());
                    if feed_bytes > max_feed_bytes {
                        tracing::warn!(
                            target: "tui::ops::subscriptions",
                            "Feed exceeded the {max_feed_bytes}-byte budget ({feed_bytes} bytes) — stopping with partial results"
                        );
                        ended_early = Some(format!("feed over the {max_feed_bytes}-byte budget"));
                        break None;
                    }
                    if let Err(e) = batcher.feed(&chunk) {
                        tracing::error!(
                            target: "tui::ops::subscriptions",
                            "Source data could not be decoded: {e} — keeping already-stored batches"
                        );
                        ended_early = Some(format!("undecodable source data: {e}"));
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
                let (rows, s) = parse_batch(
                    next_seq,
                    &tail,
                    group_id,
                    validation,
                    &mut seen_protocols,
                    &mut seen_endpoints,
                    &mut seen_links,
                );
                next_seq += 1;
                staged_count += rows.links.len();
                driver.push(rows);
                summary.merge(&s);
                if let Some(cb) = on_progress {
                    cb(staged_count);
                }
            }
            break;
        };

        let (rows, s) = parse_batch(
            next_seq,
            &batch,
            group_id,
            validation,
            &mut seen_protocols,
            &mut seen_endpoints,
            &mut seen_links,
        );
        next_seq += 1;
        staged_count += rows.links.len();
        driver.push(rows);
        summary.merge(&s);
        if let Some(cb) = on_progress {
            cb(staged_count);
        }
        if staged_count > max_feed_links {
            tracing::warn!(
                target: "tui::ops::subscriptions",
                "Feed exceeded the {max_feed_links}-link budget ({staged_count} links) — stopping with partial results"
            );
            ended_early = Some(format!("feed over the {max_feed_links}-link budget"));
            break;
        }
        tokio::task::yield_now().await;
    }

    // End of import: the coordinated flush. Persist-time failures surface HERE
    // (not per batch) and are attributed to the run as a whole — still
    // reported, never swallowed.
    let flushed = flush_import(&driver).await;
    let staged_left = flushed.staged_left;
    // A driver re-stages a window it could not write, so the shortfall is what
    // the caller loses. `saturating_sub` because a link written by an EARLIER
    // window counts in both totals only once; the reconciliation is exact
    // because every staged link is written by at most one window.
    let flush_dropped = staged_count.saturating_sub(flushed.stored);
    if flush_dropped > 0 || staged_left > 0 {
        tracing::error!(
            target: "tui::ops::subscriptions",
            "{flush_dropped} parsed link(s) were NOT stored ({} batch(es) still staged): \
             every flush attempt failed",
            staged_left,
        );
    }
    dropped_total += flush_dropped;

    if dropped_total > 0 {
        tracing::error!(
            target: "tui::ops::subscriptions",
            "{dropped_total} URL(s) were parsed but NOT stored: every retry of their batch failed",
        );
        let base = format!("{dropped_total} URL(s) dropped after failed persists");
        ended_early = Some(
            ended_early.map_or_else(|| base.clone(), |reason| format!("{reason}; plus {base}")),
        );
    }

    ImportOutcome {
        // STORED, not staged: the "only what was STORED" invariant the
        // `links` doc comment names.
        links: flushed.stored,
        summary,
        ended_early,
    }
}

/// Parse one URL batch into the four row families the bulk upserts take, with
/// NO database access (the exact body of one chunk iteration from
/// `persist_parsed_urls`, so both paths share dedup semantics).
///
/// The parse half is deliberately separate from the write: the write goes
/// through [`xray_tui_db::WriteBehind`] (which owns the transaction), so
/// everything here is CPU plus the caller's dedup sets. `batch_links` is the
/// staged batch's own link count — the run's `staged_count`.
fn parse_batch(
    seq: u64,
    batch: &[String],
    group_id: Option<&str>,
    validation: &ValidationSettings,
    seen_protocols: &mut std::collections::HashSet<i64>,
    seen_endpoints: &mut std::collections::HashSet<i64>,
    seen_links: &mut std::collections::HashSet<(i64, i64)>,
) -> (xray_tui_db::SourceBatch, ValidationSummary) {
    let (profiles, batch_summary) =
        xray_tui_config::subscription::parse_url_batch(batch, validation);

    let mut endpoints: Vec<xray_tui_db::models::Endpoint> = Vec::new();
    let mut protocols: Vec<xray_tui_db::models::Protocol> = Vec::new();
    let mut links: Vec<xray_tui_db::models::ProfileStats> = Vec::new();
    let mut group_links: Vec<xray_tui_db::models::EndpointGroup> = Vec::new();

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
            }
        }
    }

    (
        xray_tui_db::SourceBatch {
            seq,
            endpoints,
            protocols,
            links,
            group_links,
        },
        batch_summary,
    )
}

/// Batches per driver window on the import path.
///
/// The driver's `flush_rows` counts staged BATCHES (each carries up to
/// `PERSIST_CHUNK` links), so this is the transaction width in link terms:
/// eight 500-URL batches ≈ 4,000 links, close to what the old per-batch
/// transactions moved and far below the point where one transaction's WAL
/// growth stalls the UI.
pub const IMPORT_FLUSH_BATCHES: usize = 8;

/// Flush interval of the import driver's timer path.
const IMPORT_FLUSH_INTERVAL: Duration = Duration::from_millis(200);

/// Attempts for the import's end-of-run flush. Copied from the ping batch's
/// `FINAL_FLUSH_ATTEMPTS`: contention with a concurrent writer is transient
/// and the driver already waits `busy_timeout` inside each attempt, so this is
/// a short bounded retry, not a stacked backoff.
const FINAL_FLUSH_ATTEMPTS: u32 = 3;

/// Build the import's write-behind driver over `db`.
#[must_use]
pub fn import_driver(db: &Arc<Database>) -> Arc<xray_tui_db::WriteBehind<xray_tui_db::SourceSpec>> {
    xray_tui_db::WriteBehind::new(Arc::clone(db), IMPORT_FLUSH_BATCHES, IMPORT_FLUSH_INTERVAL)
}

/// The result of the end-of-import coordinated flush: links actually STORED,
/// and staged entries still pending afterwards (non-zero only when every
/// attempt failed).
#[derive(Debug, Clone, Copy)]
pub(crate) struct FinalFlush {
    pub(crate) stored: usize,
    pub(crate) staged_left: usize,
}

/// Write everything the run staged, with a bounded retry, and report what
/// landed.
///
/// Whatever did not reach the database is the caller's `dropped_total` (see
/// [`run_streaming_import`]), which reconciles against `staged_count`.
pub(crate) async fn flush_import(
    driver: &xray_tui_db::WriteBehind<xray_tui_db::SourceSpec>,
) -> FinalFlush {
    let mut stored = 0usize;
    let mut last_error = None;
    for attempt in 0..FINAL_FLUSH_ATTEMPTS {
        match driver.flush().await {
            Ok(written) => {
                stored += written;
                last_error = None;
                break;
            }
            Err(error) => {
                // A failed flush re-stages the unwritten remainder, so the
                // next attempt writes all of it.
                tracing::warn!(
                    target: "tui::ops::subscriptions",
                    "import flush attempt {} failed: {error}",
                    attempt + 1,
                );
                last_error = Some(error);
                tokio::time::sleep(Duration::from_millis(50u64 << attempt.min(3))).await;
            }
        }
    }
    if let Some(error) = last_error {
        tracing::error!(
            target: "tui::ops::subscriptions",
            "import flush failed after {FINAL_FLUSH_ATTEMPTS} attempts: {error}",
        );
    }
    FinalFlush {
        stored,
        staged_left: driver.staged_len(),
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
    budget: xray_tui_config::app_config::ImportConfig,
) -> ImportOutcome {
    // Two budgets, never one total deadline: the total deadline used to cover
    // the whole body AND the consumer's persist work between reads, so a
    // large feed was killed mid-stream and looked like a clean run.
    let client = match reqwest::Client::builder()
        .user_agent(user_agent)
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(READ_TIMEOUT)
        .redirect(xray_tui_core::updater::safe_redirect_policy(false))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(target: "tui::ops::subscriptions", "HTTP client build failed: {e}");
            return ImportOutcome {
                ended_early: Some(format!("HTTP client build failed: {e}")),
                ..ImportOutcome::default()
            };
        }
    };
    let response = match client.get(url).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(target: "tui::ops::subscriptions", "HTTP fetch failed: {e}");
            return ImportOutcome {
                ended_early: Some(format!("HTTP fetch failed: {e}")),
                ..ImportOutcome::default()
            };
        }
    };
    run_streaming_import(
        &mut HttpSource::new(response),
        db,
        group_id,
        validation,
        PERSIST_CHUNK,
        None::<&fn(usize)>,
        budget,
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
    struct ScriptedSource<I> {
        chunks: I,
    }

    impl<I> UrlSource for ScriptedSource<I>
    where
        I: Iterator<Item = SourceBatch> + Send,
    {
        fn next_chunk(&mut self) -> Pin<Box<dyn Future<Output = Option<SourceBatch>> + Send + '_>> {
            Box::pin(async { self.chunks.next() })
        }
    }

    /// A page request filtered to one group (the retired
    /// `get_active_endpoints_by_group` coverage, now on the query).
    fn group_page_request(group: &str) -> xray_tui_db::profiles_query::PageRequest {
        xray_tui_db::profiles_query::PageRequest {
            view: xray_tui_db::models::PurgatoryView::All,
            active_threshold: 0,
            scope: xray_tui_db::profiles_query::PlanScope::All,
            search: None,
            group_id: Some(group.to_string()),
            sort: xray_tui_db::profiles_query::PageSort::Test,
            ascending: true,
            offset: 0,
            limit: 10_000,
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
        let mut source = ScriptedSource {
            chunks: body
                .as_bytes()
                .chunks(3)
                .map(|c| Ok(bytes::Bytes::copy_from_slice(c))),
        };

        let outcome = run_streaming_import(
            &mut source,
            &db,
            Some("g1"),
            &validation,
            2,
            None::<&fn(usize)>,
            Default::default(),
        )
        .await;
        assert_eq!(outcome.links, 4, "one link per unique host");
        assert_eq!(outcome.summary.total_errors, 0);
        assert_eq!(outcome.ended_early, None, "clean end-of-stream");
        let meta = db
            .profiles_page(&group_page_request("g1"))
            .await
            .expect("rows");
        assert_eq!(meta.ids.len(), 4);
    }

    /// A feed that streams past the decode budget must stop early instead of
    /// buffering/parsing an unbounded body (finding f12).
    #[tokio::test]
    async fn streaming_import_stops_at_the_byte_budget() {
        let db = Arc::new(Database::in_memory().await.expect("db"));
        let validation = ValidationSettings::default();
        // One chunk over the 64 MiB decode budget; content is irrelevant —
        // the budget is checked before the bytes are decoded.
        let huge = bytes::Bytes::from(vec![b'x'; 64 * 1024 * 1024 + 1]);
        let mut source = ScriptedSource {
            chunks: std::iter::once(Ok(huge)),
        };
        let outcome = run_streaming_import(
            &mut source,
            &db,
            None,
            &validation,
            2,
            None::<&fn(usize)>,
            Default::default(),
        )
        .await;
        let reason = outcome.ended_early.expect("budget must end the run early");
        assert!(
            reason.contains("byte budget"),
            "reason must name the budget: {reason}"
        );
        assert_eq!(outcome.links, 0);
    }

    /// A CONFIGURED byte budget must cut the feed off, and the reason must name
    /// it — the path the 2026-10-01 run took with the hard-coded constant.
    #[tokio::test]
    async fn a_configured_byte_budget_ends_the_run_and_names_itself() {
        let db = Arc::new(Database::in_memory().await.expect("db"));
        let validation = ValidationSettings::default();
        let mut source = ScriptedSource {
            chunks: std::iter::once(Ok(bytes::Bytes::from(vec![b'x'; 4096]))),
        };
        let budget = xray_tui_config::app_config::ImportConfig {
            max_feed_bytes: 1024,
            max_feed_links: 0,
        };
        let outcome = run_streaming_import(
            &mut source,
            &db,
            None,
            &validation,
            2,
            None::<&fn(usize)>,
            budget,
        )
        .await;
        let reason = outcome
            .ended_early
            .expect("a configured budget must end the run");
        assert!(
            reason.contains("byte budget"),
            "the reason must name the budget that fired: {reason}",
        );
    }

    /// `0` disables a budget. A feed larger than any non-zero ceiling must still
    /// import whole when the ceiling is 0 — otherwise "unbounded" is a lie and
    /// the knob is a trap.
    #[tokio::test]
    async fn a_zero_budget_is_unbounded() {
        let db = Arc::new(Database::in_memory().await.expect("db"));
        let validation = ValidationSettings::default();
        let url =
            "vless://11111111-1111-1111-1111-111111111111@a.example:443?security=tls&type=tcp#n1";
        let mut source = ScriptedSource {
            chunks: std::iter::once(Ok(bytes::Bytes::from(url.as_bytes().to_vec()))),
        };
        let budget = xray_tui_config::app_config::ImportConfig {
            max_feed_bytes: 0,
            max_feed_links: 0,
        };
        let outcome = run_streaming_import(
            &mut source,
            &db,
            None,
            &validation,
            2,
            None::<&fn(usize)>,
            budget,
        )
        .await;
        assert_eq!(
            outcome.ended_early, None,
            "a zero budget must not truncate anything",
        );
        assert_eq!(outcome.links, 1, "the link must still import");
    }

    #[tokio::test]
    async fn streaming_import_reports_early_end_and_keeps_stored_links() {
        let db = Arc::new(Database::in_memory().await.expect("db"));
        let validation = ValidationSettings::default();
        // Batch size 2 with 4 URL-sized chunks: first TWO URLs persist, then
        // the source errors. Partial-load semantics keep the stored batch —
        // and the run must REPORT the truncation instead of looking clean.
        let mut source = ScriptedSource {
            chunks: vec![
                Ok(bytes::Bytes::from(valid_vmess_url("21.22.23.24"))),
                Ok(bytes::Bytes::from(valid_vmess_url("25.26.27.28"))),
                Err("connection reset".to_string()),
                Ok(bytes::Bytes::from(valid_vmess_url("29.30.31.32"))),
            ]
            .into_iter(),
        };

        let outcome = run_streaming_import(
            &mut source,
            &db,
            Some("g1"),
            &validation,
            2,
            None::<&fn(usize)>,
            Default::default(),
        )
        .await;
        assert_eq!(
            outcome.links, 2,
            "exactly the two URLs before the failure persist"
        );
        assert_eq!(
            outcome.ended_early.as_deref(),
            Some("connection reset"),
            "the source's own error text reaches the caller"
        );
        let meta = db
            .profiles_page(&group_page_request("g1"))
            .await
            .expect("rows");
        assert_eq!(meta.ids.len(), 2, "no rows after the failure point");
    }

    /// A dropped batch must reach `ended_early`, so the group row comes back
    /// `Error` and the "succeeded" line is suppressed.
    ///
    /// This is the regression T5 exists for: `persist_batch` returned
    /// `(0, summary)` on final failure, so `links` under-counted, `ended_early`
    /// stayed `None`, and the user was told "N profiles" for a feed that had
    /// silently lost whole 500-URL batches. A budget-exit test does NOT pin this
    /// — it would still pass with the wiring removed — so the failure is injected
    /// the way `link_writer`'s own contention tests do it: a second connection
    /// holds the write lock, so every persist attempt and every retry fails.
    #[tokio::test]
    async fn a_dropped_batch_is_reported_not_swallowed() {
        let db = Arc::new(Database::in_memory().await.expect("db"));
        let validation = ValidationSettings::default();

        // Seed one real link so the batch has something to write.
        let url =
            "vless://11111111-1111-1111-1111-111111111111@a.example:443?security=tls&type=tcp#n1";
        let parsed =
            xray_tui_config::import_export::parse_share_url(url, &validation).expect("parse");
        for profile in &parsed.parsed.endpoints {
            let ep = crate::state::endpoint_from_essentials(profile);
            db.upsert_endpoint(&ep).await.expect("endpoint");
        }

        // Hold the write lock on a SEPARATE connection: every write the import
        // attempts now fails, and `retry_on_busy` exhausts all five attempts.
        let mut blocker = db.connection().await.expect("blocker connection");
        let mut lock = blocker.transaction().await.expect("lock transaction");
        toasty::sql::statement("UPDATE profile_stats SET version = version + 0")
            .exec(&mut lock)
            .await
            .expect("take the write lock");

        let mut source = ScriptedSource {
            chunks: std::iter::once(Ok(bytes::Bytes::from(url.as_bytes().to_vec()))),
        };
        let outcome = run_streaming_import(
            &mut source,
            &db,
            None,
            &validation,
            1,
            None::<&fn(usize)>,
            Default::default(),
        )
        .await;

        lock.rollback().await.expect("release the lock");
        drop(blocker);

        assert_eq!(
            outcome.links, 0,
            "nothing could be stored while the lock was held",
        );
        let reason = outcome
            .ended_early
            .clone()
            .expect("a dropped batch must be reported, not swallowed");
        assert!(
            reason.contains("dropped after failed persists"),
            "the reason must name the dropped batch: {reason}",
        );
        let message = crate::ops::subscriptions::partial_import_message_for_test(&outcome);
        assert!(
            message.contains("dropped"),
            "the user-facing message must say rows were lost: {message}",
        );
    }

    #[tokio::test]
    async fn a_short_run_names_its_reason() {
        let db = Arc::new(Database::in_memory().await.expect("db"));
        let validation = ValidationSettings::default();
        // One chunk over the 64 MiB decode budget: a guaranteed-early exit whose
        // reason must survive into the outcome and the user-facing message.
        let huge = bytes::Bytes::from(vec![b'x'; 64 * 1024 * 1024 + 1]);
        let mut source = ScriptedSource {
            chunks: std::iter::once(Ok(huge)),
        };
        let outcome = run_streaming_import(
            &mut source,
            &db,
            None,
            &validation,
            2,
            None::<&fn(usize)>,
            Default::default(),
        )
        .await;

        let reason = outcome
            .ended_early
            .clone()
            .expect("an early exit must carry a reason");
        // The message is the ONE owner both the group row and the log line read.
        let message = crate::ops::subscriptions::partial_import_message_for_test(&outcome);
        assert!(
            message.contains(&reason),
            "the user-facing message must carry the reason: {message}"
        );
        assert!(
            message.contains("stored"),
            "the message must say what survived, not imply a full feed: {message}"
        );
    }
}
