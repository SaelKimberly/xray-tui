//! Payload sniffer: TLS `ClientHello` SNI + HTTP `Host` over a bounded
//! leading slice.
//!
//! `probe` returns `None` for anything indeterminate — garbage, truncated
//! wire data, an HTTP response, or an oversize slice — and never panics on
//! malformed input.

/// Protocol identified by the sniffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SniffedProtocol {
    /// TLS handshake (`ClientHello`).
    Tls,
    /// Plain HTTP request.
    Http,
}

impl From<SniffedProtocol> for crate::ir::SniffedProtocol {
    fn from(p: SniffedProtocol) -> Self {
        match p {
            SniffedProtocol::Tls => Self::Tls,
            SniffedProtocol::Http => Self::Http,
        }
    }
}

/// Sniff outcome: protocol plus the host carried on the wire, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SniffResult {
    pub protocol: SniffedProtocol,
    pub host: Option<String>,
}

/// Hard cap on the sniffed prefix: anything longer is refused up front.
const MAX_SNIFF_LEN: usize = 64 * 1024;

/// Sniffs a leading payload slice for a TLS `ClientHello` SNI or an HTTP
/// `Host` header.
///
/// `None` = indeterminate (garbage / truncated / oversize / HTTP response);
/// never panics on malformed wire data.
#[must_use]
pub fn probe(bytes: &[u8]) -> Option<SniffResult> {
    if bytes.len() > MAX_SNIFF_LEN {
        return None;
    }
    match bytes.first() {
        Some(&0x16) => probe_tls(bytes),
        Some(b) if b.is_ascii_alphabetic() => probe_http(bytes),
        _ => None,
    }
}

/// TLS arm: record walk with every length validated against the remaining
/// slice before advancing.
fn probe_tls(bytes: &[u8]) -> Option<SniffResult> {
    // Record header (5 bytes): content type 0x16, version u16 >= 0x0301,
    // record length u16 (validated, not used to bound — the slice bounds).
    if bytes.len() < 5 || u16::from_be_bytes([bytes[1], bytes[2]]) < 0x0301 {
        return None;
    }
    // Handshake header (4 bytes, after the record header): type 0x01
    // (ClientHello) + 3-byte big-endian length; all present before reading.
    if bytes.len() < 9 || bytes[5] != 0x01 {
        return None;
    }
    let hs_len = u32::from_be_bytes([0, bytes[6], bytes[7], bytes[8]]);
    let hs_end = 9usize.checked_add(hs_len as usize)?;
    if hs_end > bytes.len() {
        // Truncated handshake: refuse rather than walk a partial body.
        return None;
    }
    let hs = &bytes[9..hs_end];

    let r = &mut Reader::new(hs);
    r.take(2)?; // legacy_version
    r.take(32)?; // random
    // session_id: u8 length prefix (skipped, not needed).
    r.take_len8()?;
    // cipher_suites: u16 length prefix.
    r.take_len16()?;
    // compression_methods: u8 length prefix.
    r.take_len8()?;
    // Extensions block: u16 length prefix; absent (empty) is valid.
    let exts = r.take_len16()?;

    // Iterate (type u16, len u16) pairs until the server_name (0x0000).
    let mut e = Reader::new(exts);
    while e.remaining() > 0 {
        let ty = e.take_u16()?;
        let val = e.take_len16()?;
        if ty == 0x0000 {
            // server_name_list: u16 list length, then one entry —
            // name_type 0x00 (host_name), u16 host length, host bytes.
            let mut s = Reader::new(val);
            let list_len = usize::from(s.take_u16()?);
            if list_len + 2 > val.len() {
                return None;
            }
            let name_type = s.take(1)?;
            if name_type[0] != 0x00 {
                return None;
            }
            let host = s.take_len16()?;
            // SNI MUST be ASCII (RFC 6066); reject anything else.
            if host.is_empty() || !host.iter().all(|&b| b.is_ascii()) {
                return None;
            }
            return Some(SniffResult {
                protocol: SniffedProtocol::Tls,
                host: Some(String::from_utf8(host.to_vec()).ok()?),
            });
        }
    }
    Some(SniffResult {
        protocol: SniffedProtocol::Tls,
        host: None,
    })
}

/// HTTP arm: request-line must start with a method token (alphabetic),
/// response lines (`HTTP/`) are not requests, and a `host:` header is
/// matched case-insensitively with the value trimmed.
fn probe_http(bytes: &[u8]) -> Option<SniffResult> {
    // Find end of request line.
    let line_end = bytes.iter().position(|&b| b == b'\n')?;
    let line = &bytes[..line_end];
    // Response line starts "HTTP/" — not a request.
    if line.len() >= 5 && line[..5].eq_ignore_ascii_case(b"HTTP/") {
        return None;
    }
    // Method token: alphabetic chars then a space.
    let sp = line.iter().position(|&b| b == b' ')?;
    if sp == 0 || !line[..sp].iter().all(u8::is_ascii_alphabetic) {
        return None;
    }
    // Full header block must be present within the slice.
    let _ = bytes.windows(4).position(|w| w == b"\r\n\r\n")?;

    // Scan header lines for `host:` (case-insensitive name match). The
    // scan stops at the blank line that ends the header block: lines in
    // the request BODY are never searched, so a body line like
    // "host: attacker.com" cannot be mistaken for a header.
    let mut rest = &bytes[line_end + 1..];
    while !rest.is_empty() {
        let Some(nl) = rest.iter().position(|&b| b == b'\n') else {
            break;
        };
        let line = rest[..nl].strip_suffix(b"\r").unwrap_or(&rest[..nl]);
        rest = &rest[nl + 1..];
        if line.is_empty() {
            // Blank line: end of headers; host not found in the block.
            break;
        }
        let Some(colon) = line.iter().position(|&b| b == b':') else {
            continue;
        };
        if !line[..colon].eq_ignore_ascii_case(b"host") {
            continue;
        }
        // Name matched: trim OWS, return value as-is (case preserved).
        let value = std::str::from_utf8(&line[colon + 1..]).ok()?;
        let host = value.trim();
        if host.is_empty() {
            return None;
        }
        return Some(SniffResult {
            protocol: SniffedProtocol::Http,
            host: Some(host.to_owned()),
        });
    }
    None
}

/// Bounds-checked cursor over a wire slice: every take validates the
/// remaining length before advancing.
struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    const fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    const fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.remaining() < n {
            return None;
        }
        let out = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Some(out)
    }

    /// u8-prefixed chunk.
    fn take_len8(&mut self) -> Option<&'a [u8]> {
        let n = usize::from(*self.take(1)?.first()?);
        self.take(n)
    }

    /// u16-prefixed chunk (big-endian).
    fn take_len16(&mut self) -> Option<&'a [u8]> {
        let b = self.take(2)?;
        let n = u16::from_be_bytes([b[0], b[1]]).into();
        self.take(n)
    }

    fn take_u16(&mut self) -> Option<u16> {
        let b = self.take(2)?;
        Some(u16::from_be_bytes([b[0], b[1]]))
    }
}

#[cfg(test)]
mod tests;
