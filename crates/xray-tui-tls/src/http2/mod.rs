//! Minimal HTTP/2 implementation for the tls.peet.ws grader.
//!
//! Port of `thirdparty/tls-fingerprint/src/http2.rs`, adapted from the
//! reference's `TlsConnection` (TcpStream-bound, custom read helpers) to
//! this crate's [`TlsStream`] (a generic `AsyncRead + AsyncWrite` record
//! stream): `read_exact`/`write_all` come from the tokio IO traits instead
//! of hand-rolled buffer helpers.
//!
//! Scope: one GET request on a fresh connection, h2 ALPN negotiated.
//! Implements: client preface, SETTINGS exchange, HPACK-encoded HEADERS,
//! DATA frame accumulation, PING/GOAWAY handling.
//! Deliberately excludes: dynamic HPACK table, `PUSH_PROMISE`, flow-control
//! `WINDOW_UPDATE` (default 64 KiB window is enough for the JSON response).

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::error::{Result, TlsError};
use crate::record::stream::TlsStream;

// ── Frame type constants ────────────────────────────────────────────────────
pub const FRAME_DATA: u8 = 0x00;
pub const FRAME_HEADERS: u8 = 0x01;
pub const FRAME_PRIORITY: u8 = 0x02;
pub const FRAME_RST_STREAM: u8 = 0x03;
pub const FRAME_SETTINGS: u8 = 0x04;
pub const FRAME_PUSH_PROMISE: u8 = 0x05;
pub const FRAME_PING: u8 = 0x06;
pub const FRAME_GOAWAY: u8 = 0x07;
pub const FRAME_WINDOW_UPDATE: u8 = 0x08;
pub const FRAME_CONTINUATION: u8 = 0x09;

// ── Flag constants ──────────────────────────────────────────────────────────
pub const FLAG_END_STREAM: u8 = 0x01;
pub const FLAG_END_HEADERS: u8 = 0x04;
pub const FLAG_ACK: u8 = 0x01;

// ── HTTP/2 client connection preface (RFC 7540 §3.5) ───────────────────────
pub const PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

// ── Frame builder ───────────────────────────────────────────────────────────

/// Serialize an HTTP/2 frame (9-byte header + payload, RFC 7540 §4.1).
#[must_use]
pub fn make_frame(frame_type: u8, flags: u8, stream_id: u32, payload: &[u8]) -> Vec<u8> {
    let len = payload.len();
    let mut f = Vec::with_capacity(9 + len);
    // 24-bit payload length; our frames are small, so a panic here is a
    // programming error, not a runtime condition.
    let len32 = u32::try_from(len).expect("HTTP/2 frame length fits u32");
    let [_, b0, b1, b2] = len32.to_be_bytes();
    f.push(b0);
    f.push(b1);
    f.push(b2);
    f.push(frame_type);
    f.push(flags);
    f.extend_from_slice(&(stream_id & 0x7FFF_FFFF).to_be_bytes());
    f.extend_from_slice(payload);
    f
}

// ── Minimal HPACK encoder ───────────────────────────────────────────────────
//
// Uses only:
//   - Indexed header field representation  (RFC 7541 §6.1) for static table entries
//   - Literal without indexing + indexed name (RFC 7541 §6.2.2) for custom values
//   - Literal without indexing + new name   (RFC 7541 §6.2.2) for custom name+value
//
// HPACK static table (subset we use):
//   1 = :authority
//   2 = :method  GET
//   4 = :path    /
//   7 = :scheme  https

/// Appends a raw (non-Huffman) HPACK string with a single-byte length
/// prefix. The `assert!` mirrors the reference: our headers are all short.
fn hpack_string(buf: &mut Vec<u8>, s: &[u8]) {
    // H=0 (no Huffman), length as 7-bit integer (sufficient for our short strings)
    assert!(s.len() < 128, "hpack_string: value too long for single-byte length");
    buf.push(u8::try_from(s.len()).expect("hpack length < 128 per the assert above"));
    buf.extend_from_slice(s);
}

/// Encode the request headers for a plain `GET <path>` on `host`.
#[must_use]
pub fn encode_get_headers(path: &str, host: &str) -> Vec<u8> {
    let mut h = Vec::new();

    // :method: GET  →  indexed, static[2]  →  0x82
    h.push(0x82);

    // :scheme: https  →  indexed, static[7]  →  0x87
    h.push(0x87);

    // :path: <path>  →  literal without indexing, name = static[4] (:path)
    //   prefix byte: 0000_nnnn  where nnnn=4
    h.push(0x04);
    hpack_string(&mut h, path.as_bytes());

    // :authority: <host>  →  literal without indexing, name = static[1] (:authority)
    h.push(0x01);
    hpack_string(&mut h, host.as_bytes());

    // accept: application/json  →  literal without indexing, new name
    h.push(0x00);
    hpack_string(&mut h, b"accept");
    hpack_string(&mut h, b"application/json");

    h
}

/// Read one complete HTTP/2 frame.
/// Returns `(frame_type, flags, stream_id, payload)`.
///
/// Uses `read_exact` on the decrypted stream; a clean EOF (or truncated
/// frame) surfaces as an `UnexpectedEof` `io::Error` and propagates as
/// [`TlsError::Io`].
pub async fn read_frame<S>(conn: &mut TlsStream<S>) -> Result<(u8, u8, u32, Vec<u8>)>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    let mut hdr = [0u8; 9];
    conn.read_exact(&mut hdr).await?;
    let len = (usize::from(hdr[0]) << 16) | (usize::from(hdr[1]) << 8) | usize::from(hdr[2]);
    let frame_type = hdr[3];
    let flags = hdr[4];
    let stream_id = u32::from_be_bytes([hdr[5] & 0x7F, hdr[6], hdr[7], hdr[8]]);
    let mut payload = vec![0u8; len];
    conn.read_exact(&mut payload).await?;
    Ok((frame_type, flags, stream_id, payload))
}

// ── High-level GET ──────────────────────────────────────────────────────────

/// Perform an HTTP/2 GET request and return the response body as a String.
///
/// Flow (RFC 7540):
///   1. Send client preface (24-byte magic + empty SETTINGS)
///   2. Send HEADERS frame (stream 1, `END_STREAM` | `END_HEADERS`)
///   3. Process incoming frames:
///      - SETTINGS (not ACK) → send SETTINGS ACK
///      - PING (not ACK)     → send PING ACK
///      - DATA on stream 1   → accumulate body; stop on `END_STREAM`
///      - HEADERS on stream 1 → check `END_STREAM` (response has no body)
///      - GOAWAY             → error
///      - everything else    → ignore
pub async fn get<S>(conn: &mut TlsStream<S>, path: &str, host: &str) -> Result<String>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    // 1. Client connection preface + empty SETTINGS
    let mut preface = PREFACE.to_vec();
    preface.extend(make_frame(FRAME_SETTINGS, 0, 0, &[]));
    conn.write_all(&preface).await?;

    // 2. HEADERS (stream 1): GET request, no body → END_STREAM
    let headers_block = encode_get_headers(path, host);
    conn.write_all(&make_frame(
        FRAME_HEADERS,
        FLAG_END_STREAM | FLAG_END_HEADERS,
        1,
        &headers_block,
    ))
    .await?;

    // 3. Receive frames until END_STREAM on stream 1
    let mut body: Vec<u8> = Vec::new();
    loop {
        let (ftype, flags, stream_id, payload) = read_frame(conn).await?;

        match ftype {
            FRAME_SETTINGS => {
                // The server's SETTINGS must be ACKed (RFC 7540 §6.5.3);
                // an ACK of our own SETTINGS needs no reply.
                if flags & FLAG_ACK == 0 {
                    conn.write_all(&make_frame(FRAME_SETTINGS, FLAG_ACK, 0, &[])).await?;
                }
            }
            FRAME_PING if flags & FLAG_ACK == 0 => {
                // Ping must be ACKed with identical payload (RFC 7540 §6.7)
                conn.write_all(&make_frame(FRAME_PING, FLAG_ACK, 0, &payload)).await?;
            }
            FRAME_DATA if stream_id == 1 => {
                // Handle optional PADDED flag
                let data = if flags & 0x08 != 0 && !payload.is_empty() {
                    let pad_len = usize::from(payload[0]);
                    let end = payload.len().saturating_sub(pad_len);
                    &payload[1..end]
                } else {
                    &payload[..]
                };
                body.extend_from_slice(data);
                if flags & FLAG_END_STREAM != 0 {
                    break;
                }
            }
            FRAME_HEADERS if stream_id == 1 => {
                // Response headers (we don't need to parse them)
                if flags & FLAG_END_STREAM != 0 {
                    break; // response has no body (e.g. 204 No Content)
                }
            }
            FRAME_RST_STREAM if stream_id == 1 => {
                let code = u32::from_be_bytes(payload.try_into().unwrap_or([0; 4]));
                return Err(TlsError::Protocol(format!("HTTP/2 RST_STREAM error_code={code}")));
            }
            FRAME_GOAWAY => {
                let code = if payload.len() >= 8 {
                    u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]])
                } else {
                    0
                };
                return Err(TlsError::Protocol(format!("HTTP/2 GOAWAY error_code={code}")));
            }
            _ => {
                // WINDOW_UPDATE, PRIORITY, CONTINUATION, unknown — ignore
            }
        }
    }

    String::from_utf8(body)
        .map_err(|e| TlsError::Protocol(format!("HTTP/2 response body not UTF-8: {e}")))
}
