//! Subscription data parsing: chunked base64 decoder + URL splitting + batch parse.

use std::mem::MaybeUninit;

use crate::import_export::{
    ImportError, ParsedProtocol, ValidationSettings, ValidationSummary, parse_share_url,
};
use aho_corasick::AhoCorasick;
use base64_simd::{STANDARD_NO_PAD, URL_SAFE_NO_PAD};
use xray_tui_proto::proto_spec::ProtocolConfig;

/// Maximum input chunk size for `StreamingDecoder::feed()`.
const INPUT_CHUNK_SIZE: usize = 65536;

/// Maximum bytes to carry over between chunks (incomplete lines).
const CARRY_OVER_SIZE: usize = 262_144;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EncodingState {
    Unknown,
    StdB64,
    UrlSafeB64,
    Raw,
}

/// Streaming base64 decoder with encoding auto-detection.
///
/// Handles chunked subscription data by aligning to 4-byte base64 boundaries,
/// auto-detecting encoding (URL-safe / standard / raw), and splitting on `\n`.
pub struct StreamingDecoder {
    state: EncodingState,
    pending_input: [MaybeUninit<u8>; 4],
    pending_input_len: usize,
    carry_over: Box<[MaybeUninit<u8>; CARRY_OVER_SIZE]>,
    carry_over_len: usize,
}

impl StreamingDecoder {
    /// Create a new decoder.
    #[must_use]
    #[allow(
        clippy::large_stack_arrays,
        reason = "carry_over is heap-allocated via Box, stack is just temporary during Box::new"
    )]
    pub fn new() -> Self {
        Self {
            state: EncodingState::Unknown,
            pending_input: [MaybeUninit::uninit(); 4],
            pending_input_len: 0,
            carry_over: Box::new([MaybeUninit::uninit(); CARRY_OVER_SIZE]),
            carry_over_len: 0,
        }
    }

    /// Reset the decoder to initial state.
    pub fn reset(&mut self) {
        self.state = EncodingState::Unknown;
        self.pending_input = [MaybeUninit::uninit(); 4];
        self.pending_input_len = 0;
        self.carry_over.fill(MaybeUninit::uninit());
        self.carry_over_len = 0;
    }

    /// Feed one chunk of raw input data. Returns any complete URLs extracted.
    ///
    /// Internally aligns input to 4-byte base64 boundaries, detects encoding
    /// and decodes, splits on `\n`, and passes complete text regions through
    /// `subscription_url_split`.
    ///
    /// # Errors
    ///
    /// Returns an error if base64 decoding fails after encoding was determined.
    pub fn feed(&mut self, chunk: &[u8]) -> Result<Vec<String>, String> {
        if chunk.is_empty() && self.pending_input_len == 0 {
            return Ok(vec![]);
        }

        if chunk.len() > INPUT_CHUNK_SIZE {
            return Err(format!(
                "Input chunk too large: got {} bytes, max {}",
                chunk.len(),
                INPUT_CHUNK_SIZE,
            ));
        }

        let total_len = self.pending_input_len + chunk.len();
        #[allow(
            clippy::large_stack_arrays,
            reason = "hot-path decoder needs fixed-size work buffer; heap alloc on every feed call too expensive"
        )]
        let mut work: [MaybeUninit<u8>; INPUT_CHUNK_SIZE + 4] =
            [MaybeUninit::uninit(); INPUT_CHUNK_SIZE + 4];
        // Prepend pending bytes
        #[allow(clippy::needless_range_loop)]
        for i in 0..self.pending_input_len {
            work[i] = MaybeUninit::new(unsafe { self.pending_input[i].assume_init() });
        }

        // Copy chunk bytes into remaining work area
        #[allow(clippy::needless_range_loop)]
        for i in 0..chunk.len() {
            work[self.pending_input_len + i] = MaybeUninit::new(chunk[i]);
        }
        self.pending_input_len = 0;

        // Align to 4-byte base64 boundary
        let aligned_len = (total_len / 4) * 4;
        let remainder = total_len - aligned_len;

        // Save trailing bytes as pending for next call
        #[allow(clippy::needless_range_loop)]
        for i in 0..remainder {
            self.pending_input[i] =
                MaybeUninit::new(unsafe { work[aligned_len + i].assume_init() });
        }
        self.pending_input_len = remainder;

        // SAFETY: work[..aligned_len] is fully initialized
        let input = unsafe { std::slice::from_raw_parts(work.as_ptr().cast::<u8>(), aligned_len) };
        let decoded = self.process_aligned(input)?;
        Ok(self.process_decoded(&decoded))
    }

    /// Flush any remaining buffered data. Call once after the last `feed()`.
    ///
    /// Returns any final URLs from the last partial line.
    ///
    /// # Errors
    ///
    /// Returns an error if base64 decoding fails.
    pub fn finalize(&mut self) -> Result<Vec<String>, String> {
        let mut result = Vec::new();

        // Process leftover pending_input bytes (may be < 4)
        if self.pending_input_len > 0 {
            let mut buf = [0u8; 4];
            #[allow(clippy::needless_range_loop)]
            for i in 0..self.pending_input_len {
                buf[i] = unsafe { self.pending_input[i].assume_init() };
            }
            let decoded = self.process_aligned(&buf[..self.pending_input_len])?;
            self.pending_input_len = 0;
            result.extend(self.process_decoded(&decoded));
        }

        // Flush remaining carry_over as complete text
        if self.carry_over_len > 0 {
            let carry = self.carry_over_slice();
            result.extend(process_text_std(carry));
            self.carry_over_len = 0;
        }

        Ok(result)
    }

    // ── internal helpers ──

    /// Detect encoding and decode one 4-byte-aligned portion.
    fn process_aligned(&mut self, data: &[u8]) -> Result<Vec<u8>, String> {
        if data.is_empty() {
            return Ok(vec![]);
        }

        // Trim trailing whitespace and '=' padding for base64 decode attempts.
        let trimmed = if matches!(self.state, EncodingState::Raw) {
            data
        } else if let Some(pos) = data
            .iter()
            .rposition(|&b| !(b.is_ascii_whitespace() || b == b'='))
        {
            &data[..=pos]
        } else {
            return Ok(vec![]);
        };

        match self.state {
            EncodingState::Unknown => {
                let (encoding, decoded) = if memchr::memchr2(b'+', b'\\', trimmed).is_some() {
                    // Has standard-base64-specific characters
                    STANDARD_NO_PAD.decode_to_vec(trimmed).map_or_else(
                        |_| (EncodingState::Raw, data.to_vec()),
                        |d| (EncodingState::StdB64, d),
                    )
                } else if memchr::memchr2(b'-', b'_', trimmed).is_some() {
                    // Has URL-safe-base64-specific characters
                    URL_SAFE_NO_PAD.decode_to_vec(trimmed).map_or_else(
                        |_| (EncodingState::Raw, data.to_vec()),
                        |d| (EncodingState::UrlSafeB64, d),
                    )
                } else {
                    // Alphanumeric-only — try standard (most common)
                    STANDARD_NO_PAD.decode_to_vec(trimmed).map_or_else(
                        |_| (EncodingState::Raw, data.to_vec()),
                        |d| (EncodingState::StdB64, d),
                    )
                };
                self.state = encoding;
                Ok(decoded)
            }
            EncodingState::StdB64 => STANDARD_NO_PAD
                .decode_to_vec(trimmed)
                .map_err(|e| format!("base64 decode error: {e}")),
            EncodingState::UrlSafeB64 => URL_SAFE_NO_PAD
                .decode_to_vec(trimmed)
                .map_err(|e| format!("base64 decode error: {e}")),
            EncodingState::Raw => Ok(data.to_vec()),
        }
    }

    /// Process decoded bytes: lossy UTF-8, prepend `carry_over`, split on last
    /// `\n`, extract URLs from complete portion, save remainder as `carry_over`.
    fn process_decoded(&mut self, decoded: &[u8]) -> Vec<String> {
        if decoded.is_empty() && self.carry_over_len == 0 {
            return Vec::new();
        }

        // Fast SIMD UTF-8 validation; fall back to lossy on invalid input
        let Ok(decoded_str) = simdutf8::basic::from_utf8(decoded) else {
            let s = String::from_utf8_lossy(decoded).into_owned();
            return self.process_text_owned(&s);
        };

        if self.carry_over_len == 0 {
            self.process_str(decoded_str)
        } else {
            let carry_bytes = self.carry_over_slice();
            let mut combined = String::with_capacity(self.carry_over_len + decoded_str.len());
            // SAFETY: carry_over bytes are valid UTF-8 (came from previous decoded chunks)
            unsafe {
                combined.as_mut_vec().extend_from_slice(carry_bytes);
            }
            combined.push_str(decoded_str);
            self.carry_over_len = 0;

            self.process_text_owned(&combined)
        }
    }

    /// Helper: split on last \n, extract URLs, save `carry_over`.
    /// Takes ownership of the string for splitting.
    fn process_text_owned(&mut self, full_text: &str) -> Vec<String> {
        if full_text.is_empty() {
            return Vec::new();
        }

        if let Some(last_nl) = full_text.rfind('\n') {
            let complete = &full_text[..last_nl];
            let remaining = &full_text[last_nl + 1..];
            let urls = process_text_std(complete.as_bytes());

            self.set_carry_over(remaining);

            urls
        } else {
            self.set_carry_over(full_text);
            Vec::new()
        }
    }

    /// Process a &str (no `carry_over` involved): split on last \n, extract URLs,
    /// save `carry_over`.
    fn process_str(&mut self, text: &str) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }
        if let Some(last_nl) = text.rfind('\n') {
            let complete = &text[..last_nl];
            let remaining = &text[last_nl + 1..];
            let urls = process_text_std(complete.as_bytes());
            self.set_carry_over(remaining);
            urls
        } else {
            self.set_carry_over(text);
            Vec::new()
        }
    }

    fn set_carry_over(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let len = bytes.len();
        if len > CARRY_OVER_SIZE {
            for (i, &b) in bytes[..CARRY_OVER_SIZE].iter().enumerate() {
                self.carry_over[i] = MaybeUninit::new(b);
            }
            self.carry_over_len = CARRY_OVER_SIZE;
            return;
        }
        for (i, &b) in bytes.iter().enumerate() {
            self.carry_over[i] = MaybeUninit::new(b);
        }
        self.carry_over_len = len;
    }

    fn carry_over_slice(&self) -> &[u8] {
        if self.carry_over_len == 0 {
            return &[];
        }
        // SAFETY: first carry_over_len bytes are initialized
        unsafe {
            std::slice::from_raw_parts(self.carry_over.as_ptr().cast::<u8>(), self.carry_over_len)
        }
    }
}

impl Default for StreamingDecoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard `process_text` that splits text by newlines and extracts URLs via
/// `subscription_url_split`.
fn process_text_std(data: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(data);
    let mut result = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            result.extend(subscription_url_split(trimmed));
        }
    }
    result
}

/// Split concatenated subscription data into individual URLs using
/// Aho-Corasick to find all scheme boundaries.
///
/// Handles the case where multiple URLs are concatenated without newlines
/// (e.g., `vmess://...vless://...trojan://...`).
#[must_use]
pub fn subscription_url_split(text: &str) -> Vec<String> {
    static SCHEMA_AC: std::sync::LazyLock<AhoCorasick> = std::sync::LazyLock::new(|| {
        AhoCorasick::builder()
            .ascii_case_insensitive(true)
            .match_kind(aho_corasick::MatchKind::LeftmostLongest)
            .build([
                "vmess://",
                "vless://",
                "trojan://",
                "ss://",
                "ssr://",
                "hysteria2://",
                "hysteria://",
                "hy2://",
                "hy://",
                "tuic://",
                "socks://",
                "socks5://",
                "http://",
                "naive+https://",
                "naive+quic://",
                "anytls://",
                "shadowtls://",
                "wireguard://",
            ])
            .unwrap()
    });

    let mut last_start: Option<usize> = None;
    let mut chunks = Vec::new();

    for m in SCHEMA_AC.find_iter(text) {
        if let Some(start) = last_start.take() {
            let end = m.start();
            chunks.push(text[start..end].to_string());
        }
        last_start = Some(m.start());
    }

    if let Some(start) = last_start.take() {
        chunks.push(text[start..].to_string());
    }

    chunks
}

/// Parse base64-encoded subscription data into a list of Profiles.
///
/// Returns `(profiles, summary)` on success, where `summary` is a
/// `ValidationSummary` counting the types of errors encountered.
///
/// # Errors
///
/// Returns an error if the data cannot be decoded.
pub fn parse_subscription_data(
    data: &[u8],
    settings: &ValidationSettings,
) -> Result<(Vec<ParsedProtocol>, ValidationSummary), String> {
    let mut decoder = StreamingDecoder::new();
    let mut all_urls = Vec::new();

    // Process in chunks of 64KB
    for chunk in data.chunks(INPUT_CHUNK_SIZE) {
        let urls = decoder.feed(chunk)?;
        all_urls.extend(urls);
    }

    // Finalize
    let urls = decoder.finalize()?;
    all_urls.extend(urls);

    // Parse each URL into a Profile
    let mut profiles: Vec<ParsedProtocol> = Vec::new();
    let mut summary = ValidationSummary::default();
    for url in &all_urls {
        match parse_share_url(url, settings) {
            Ok(profile) => profiles.push(profile),
            Err(ImportError::Validation(msg)) => {
                let lower = msg.to_lowercase();
                if lower.starts_with("missing field") {
                    summary.missing_field_count += 1;
                } else if lower.starts_with("private ip")
                    || lower.starts_with("loopback")
                    || lower.starts_with("link-local")
                    || lower.starts_with("unique-local")
                    || lower.starts_with("localhost")
                {
                    summary.host_validation_count += 1;
                } else {
                    summary.other_count += 1;
                }
            }
            Err(_) => {
                summary.other_count += 1;
            }
        }
    }

    // Scan parsed profiles for allow_insecure / insecure settings
    summary.security_warning_count = profiles
        .iter()
        .filter(|p| {
            if let Ok(config) = serde_json::from_slice::<ProtocolConfig>(&p.spec_blob) {
                let (_, s_settings) = config.to_settings();
                s_settings
                    .get("allow_insecure")
                    .and_then(serde_json::Value::as_bool)
                    == Some(true)
            } else {
                false
            }
        })
        .count();

    summary.total_errors = summary.missing_field_count
        + summary.host_validation_count
        + summary.security_warning_count
        + summary.other_count;

    Ok((profiles, summary))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_url_split_single() {
        let urls = subscription_url_split("vmess://abc123");
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "vmess://abc123");
    }

    #[test]
    fn test_subscription_url_split_multiple() {
        let urls = subscription_url_split("vmess://abc123vless://def456trojan://ghi789");
        assert_eq!(urls.len(), 3);
        assert!(urls[0].starts_with("vmess://"));
        assert!(urls[1].starts_with("vless://"));
        assert!(urls[2].starts_with("trojan://"));
    }

    #[test]
    fn test_subscription_url_split_with_newlines() {
        let input = "vmess://abc123\nvless://def456\ntrojan://ghi789\n";
        let urls = subscription_url_split(input);
        assert_eq!(urls.len(), 3);
    }

    #[test]
    fn test_streaming_decoder_simple() {
        let mut decoder = StreamingDecoder::new();
        // Base64 of "hello\nworld\n"
        let b64 = base64_simd::STANDARD.encode_to_string(b"vmess://abc123\nvless://def456\n");
        let result = decoder.feed(b64.as_bytes()).unwrap();
        assert!(!result.is_empty(), "Should find URLs in decoded data");
    }

    #[test]
    fn test_streaming_decoder_empty() {
        let mut decoder = StreamingDecoder::new();
        let result = decoder.feed(b"").unwrap();
        assert!(result.is_empty());
        let result = decoder.finalize().unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_subscription_data_empty() {
        let settings = crate::import_export::ValidationSettings::default();
        let (profiles, summary) = parse_subscription_data(b"", &settings).unwrap();
        assert!(profiles.is_empty());
        assert_eq!(summary.total_errors, 0);
        assert_eq!(summary.missing_field_count, 0);
        assert_eq!(summary.host_validation_count, 0);
        assert_eq!(summary.security_warning_count, 0);
        assert_eq!(summary.other_count, 0);
    }
}
