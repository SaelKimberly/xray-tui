//! Record-framed `AsyncRead`/`AsyncWrite` stream: TLS 1.3 application data
//! over any duplex transport.
//!
//! This is the layer `thirdparty/tls-fingerprint` LACKED — its
//! `TlsConnection` was `TcpStream`-bound with sync read/write helpers. Here
//! the same decryption (`strip_padding`) and framing (`make_app_data_record`)
//! logic is implemented as a poll-based stream: buffered plaintext on read
//! (handles TLS record fragmentation), 16 KiB plaintext split and
//! partial-write buffering on write. Post-handshake `NewSessionTicket`
//! records (encrypted handshake content, message type 0x04) are consumed and
//! dropped;
//! any other inner handshake message is a protocol error (no renegotiation).

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::crypto::AeadKey;
use crate::error::{Result, TlsError};

/// TLS record content types (RFC 8446 §5.1).
use super::{
    CONTENT_ALERT, CONTENT_APPLICATION_DATA, CONTENT_HANDSHAKE, HS_NEW_SESSION_TICKET,
    MAX_RECORD_PAYLOAD, TlsRecord, aead_aad, make_app_data_record,
};

/// Maximum plaintext bytes per TLS 1.3 record: 2^14 (RFC 8446 §5.2).
const MAX_RECORD_PLAINTEXT: usize = 16_384;

/// AEAD tag length for the TLS 1.3 AEADs in this engine (AES-GCM and
/// ChaCha20-Poly1305 both use 16-byte tags).
const AEAD_TAG_LEN: usize = 16;

/// Key material plus per-direction sequence counters for a TLS 1.3
/// connection.
#[derive(Clone)]
pub struct AppKeys {
    pub read_key: AeadKey,
    pub write_key: AeadKey,
    pub read_seq: u64,
    pub write_seq: u64,
}

/// A TLS 1.3 application-data stream: decrypts records on read, encrypts and
/// frames plaintext on write.
///
/// # EOF semantics (deliberate deviation from RFC 8446 §6.1)
///
/// Transport EOF *without* `close_notify` is surfaced as a clean EOF
/// (`Ready(Ok(()))` with zero bytes), deliberately: this stream is a proxy
/// tunnel whose servers close with TCP FIN and do not guarantee `close_notify`
/// (RST arrives as an `io::Error` before the record layer observes EOF).
/// Truncation attacks per RFC 8446 §6.1 are out of scope for this tunnel
/// semantic; a `close_notify` is still honored as EOF when present.
pub struct TlsStream<S> {
    inner: S,
    keys: AppKeys,
    /// Decrypted plaintext from the current record, served before more I/O.
    read_buf: Vec<u8>,
    read_pos: usize,
    /// EOF reached (`close_notify` or clean inner close); reads return 0.
    closed: bool,
    /// Framed ciphertext not yet fully written to `inner`.
    pending: Vec<u8>,
    /// Record-read state machine (header, then payload).
    rec: RecordState,
}

/// Incremental state of reading one TLS record off the wire.
enum RecordState {
    Header {
        buf: [u8; 5],
        filled: usize,
    },
    Payload {
        content_type: u8,
        len: usize,
        buf: Vec<u8>,
        filled: usize,
    },
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send> TlsStream<S> {
    pub const fn new(inner: S, keys: AppKeys) -> Self {
        Self {
            inner,
            keys,
            read_buf: Vec::new(),
            read_pos: 0,
            closed: false,
            pending: Vec::new(),
            rec: RecordState::Header {
                buf: [0; 5],
                filled: 0,
            },
        }
    }

    /// Read the next complete raw record off `inner`.
    ///
    /// Returns `Ok(None)` on a clean EOF at a record boundary, `Ok(Some(..))`
    /// once the full header + payload have been read. Partial progress across
    /// polls is preserved in `self.rec`.
    fn poll_record(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<Option<TlsRecord>>> {
        loop {
            let step = match &mut self.rec {
                RecordState::Header { buf, filled } => {
                    while *filled < buf.len() {
                        let mut rb = ReadBuf::new(&mut buf[*filled..]);
                        match Pin::new(&mut self.inner).poll_read(cx, &mut rb) {
                            Poll::Pending => return Poll::Pending,
                            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                            Poll::Ready(Ok(())) => {
                                let n = rb.filled().len();
                                if n == 0 {
                                    if *filled == 0 {
                                        return Poll::Ready(Ok(None));
                                    }
                                    return Poll::Ready(Err(io::Error::new(
                                        io::ErrorKind::UnexpectedEof,
                                        "truncated TLS record header",
                                    )));
                                }
                                *filled += n;
                            }
                        }
                    }
                    let content_type = buf[0];
                    let len = usize::from(u16::from_be_bytes([buf[3], buf[4]]));
                    if len > MAX_RECORD_PAYLOAD {
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("TLS record too large: {len} bytes"),
                        )));
                    }
                    Step::HeaderComplete { content_type, len }
                }
                RecordState::Payload {
                    content_type,
                    len,
                    buf,
                    filled,
                } => {
                    while *filled < *len {
                        let mut rb = ReadBuf::new(&mut buf[*filled..]);
                        match Pin::new(&mut self.inner).poll_read(cx, &mut rb) {
                            Poll::Pending => return Poll::Pending,
                            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                            Poll::Ready(Ok(())) => {
                                let n = rb.filled().len();
                                if n == 0 {
                                    return Poll::Ready(Err(io::Error::new(
                                        io::ErrorKind::UnexpectedEof,
                                        "truncated TLS record payload",
                                    )));
                                }
                                *filled += n;
                            }
                        }
                    }
                    Step::RecordReady {
                        content_type: *content_type,
                        payload: std::mem::take(buf),
                    }
                }
            };
            match step {
                Step::HeaderComplete { content_type, len } => {
                    self.rec = RecordState::Payload {
                        content_type,
                        len,
                        buf: vec![0; len],
                        filled: 0,
                    };
                }
                Step::RecordReady {
                    content_type,
                    payload,
                } => {
                    self.rec = RecordState::Header {
                        buf: [0; 5],
                        filled: 0,
                    };
                    return Poll::Ready(Ok(Some(TlsRecord {
                        content_type,
                        payload,
                    })));
                }
            }
        }
    }

    /// Decrypt one record and dispatch on its inner content type.
    ///
    /// Returns the plaintext for `CONTENT_APPLICATION_DATA`, `CloseNotify`
    /// for `close_notify`, `Skip` for `NewSessionTicket` records, and an
    /// error for alerts other than `close_notify`, other handshake records
    /// (no renegotiation) and unknown content types.
    fn process_record(&mut self, mut rec: TlsRecord) -> io::Result<DecryptedRecord> {
        let plaintext = self
            .keys
            .read_key
            .open(
                self.keys.read_seq,
                &aead_aad(rec.payload.len()),
                &mut rec.payload,
            )
            .map_err(to_io_error)?;
        self.keys.read_seq += 1;
        let (inner_type, content) = strip_padding(&plaintext).map_err(to_io_error)?;
        match inner_type {
            CONTENT_APPLICATION_DATA => Ok(DecryptedRecord::Data(content.to_vec())),
            CONTENT_ALERT => {
                if content.len() >= 2 && content[1] == 0 {
                    // close_notify: clean EOF.
                    Ok(DecryptedRecord::CloseNotify)
                } else {
                    let level = content.first().copied().unwrap_or(2);
                    let desc = content.get(1).copied().unwrap_or(0);
                    Err(io::Error::new(
                        io::ErrorKind::ConnectionReset,
                        format!("TLS alert: level {level} description {desc}"),
                    ))
                }
            }
            CONTENT_HANDSHAKE => {
                // NewSessionTicket (0x04) legitimately follows the server
                // Finished as an encrypted handshake record; drop it. Any
                // other handshake message post-handshake is a protocol
                // violation — TLS 1.3 has no renegotiation.
                match content.first() {
                    Some(&HS_NEW_SESSION_TICKET) => Ok(DecryptedRecord::Skip),
                    Some(&msg_type) => Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("unexpected handshake message type {msg_type} after handshake"),
                    )),
                    None => Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "empty handshake record during application data",
                    )),
                }
            }
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected inner TLS content type {other:#04x}"),
            )),
        }
    }

    /// Write as much of `pending` to `inner` as possible.
    fn flush_pending(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        while !self.pending.is_empty() {
            match Pin::new(&mut self.inner).poll_write(cx, &self.pending) {
                // Ok(0) with a non-empty buffer means "would block" —
                // wait for writability instead of spinning.
                Poll::Pending | Poll::Ready(Ok(0)) => return Poll::Pending,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Ready(Ok(n)) => {
                    self.pending.drain(..n);
                }
            }
        }
        Poll::Ready(Ok(()))
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send> AsyncRead for TlsStream<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        loop {
            // Serve buffered plaintext before touching the wire.
            if this.read_pos < this.read_buf.len() {
                let n = (this.read_buf.len() - this.read_pos).min(buf.remaining());
                buf.put_slice(&this.read_buf[this.read_pos..this.read_pos + n]);
                this.read_pos += n;
                if this.read_pos == this.read_buf.len() {
                    this.read_buf.clear();
                    this.read_pos = 0;
                }
                return Poll::Ready(Ok(()));
            }
            if this.closed {
                return Poll::Ready(Ok(()));
            }
            let rec = match this.poll_record(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Ready(Ok(None)) => {
                    // Transport EOF without close_notify: clean EOF by design
                    // (tunnel semantics — see the type-level docs).
                    this.closed = true;
                    return Poll::Ready(Ok(()));
                }
                Poll::Ready(Ok(Some(rec))) => rec,
            };
            // Raw records other than application data are invalid post
            // handshake (TLS 1.3 §5.4) — but mirror `read_record`'s
            // leniency for CCS middlebox-compat records: skip them.
            if rec.content_type == CONTENT_ALERT && rec.payload.len() >= 2 {
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("TLS alert: {} {}", rec.payload[0], rec.payload[1]),
                )));
            }
            if rec.content_type != CONTENT_APPLICATION_DATA {
                continue;
            }
            match this.process_record(rec) {
                Ok(DecryptedRecord::Data(plaintext)) => {
                    this.read_buf.extend_from_slice(&plaintext);
                    this.read_pos = 0;
                }
                Ok(DecryptedRecord::CloseNotify) => {
                    this.closed = true;
                    return Poll::Ready(Ok(()));
                }
                // NewSessionTicket: consumed, nothing to deliver — read on.
                Ok(DecryptedRecord::Skip) => {}
                Err(e) => return Poll::Ready(Err(e)),
            }
        }
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send> AsyncWrite for TlsStream<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        // A partial record from a previous call must drain first.
        if !this.pending.is_empty() {
            match this.flush_pending(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Ready(Ok(())) => {}
            }
        }
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }
        // Split at 2^14 plaintext bytes (RFC 8446 §5.2); the inner content
        // type byte rides at the end of the TLSInnerPlaintext.
        let take = buf.len().min(MAX_RECORD_PLAINTEXT);
        let mut tls_inner = Vec::with_capacity(take + 1);
        tls_inner.extend_from_slice(&buf[..take]);
        tls_inner.push(CONTENT_APPLICATION_DATA);
        let ciphertext = this
            .keys
            .write_key
            .seal(
                this.keys.write_seq,
                &aead_aad(tls_inner.len() + AEAD_TAG_LEN),
                &tls_inner,
            )
            .map_err(to_io_error)?;
        this.keys.write_seq += 1;
        this.pending = make_app_data_record(&ciphertext);
        match this.flush_pending(cx) {
            // Data stays buffered in `pending`; a later poll drains it.
            Poll::Pending | Poll::Ready(Ok(())) => {}
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
        }
        Poll::Ready(Ok(take))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        match this.flush_pending(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Ready(Ok(())) => {}
        }
        Pin::new(&mut this.inner).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        match this.flush_pending(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Ready(Ok(())) => {}
        }
        Pin::new(&mut this.inner).poll_shutdown(cx)
    }
}

/// Result of advancing the record state machine, resolved outside the
/// `self.rec` borrow so the state can be reassigned.
enum Step {
    HeaderComplete { content_type: u8, len: usize },
    RecordReady { content_type: u8, payload: Vec<u8> },
}

/// What a decrypted record contributes to the application byte stream.
enum DecryptedRecord {
    /// Plaintext to buffer and serve.
    Data(Vec<u8>),
    /// `close_notify` — subsequent reads are EOF.
    CloseNotify,
    /// Record consumed with nothing to deliver (`NewSessionTicket`).
    Skip,
}

/// Remove TLS 1.3 record padding and extract the inner content type (port of
/// `tls-fingerprint/src/handshake.rs::strip_padding`).
///
/// Plaintext layout: `actual_content` || `content_type_byte` || optional
/// zero padding (RFC 8446 §5.4).
fn strip_padding(plaintext: &[u8]) -> Result<(u8, &[u8])> {
    let mut end = plaintext.len();
    while end > 0 && plaintext[end - 1] == 0 {
        end -= 1;
    }
    if end == 0 {
        return Err(TlsError::Handshake("all-zero TLSInnerPlaintext".into()));
    }
    let content_type = plaintext[end - 1];
    Ok((content_type, &plaintext[..end - 1]))
}

/// Convert a crate error into an I/O error for the stream traits (the
/// `AsyncRead`/`AsyncWrite` surface speaks `std::io`).
fn to_io_error(e: TlsError) -> io::Error {
    match e {
        TlsError::Io(io) => io,
        other => io::Error::new(io::ErrorKind::InvalidData, other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use tokio::io::{AsyncReadExt, AsyncWriteExt, duplex};

    use super::*;
    use crate::crypto::CipherSuiteId;
    use crate::record::{aead_aad, make_app_data_record, read_record};

    fn test_keys() -> AppKeys {
        let key = AeadKey::new(CipherSuiteId::Aes128GcmSha256, &[0x11; 16]).unwrap();
        AppKeys {
            read_key: key.clone_key(),
            write_key: key.clone_key(),
            read_seq: 0,
            write_seq: 0,
        }
    }

    /// Build a raw application-data record whose `TLSInnerPlaintext` is
    /// `content || inner_type`, sealed under `keys.read_key` with `seq`.
    fn raw_record(keys: &AppKeys, seq: u64, inner_type: u8, content: &[u8]) -> Vec<u8> {
        let mut inner = content.to_vec();
        inner.push(inner_type);
        let ct = keys
            .read_key
            .seal(seq, &aead_aad(inner.len() + AEAD_TAG_LEN), &inner)
            .unwrap();
        make_app_data_record(&ct)
    }

    #[tokio::test]
    async fn encrypted_duplex_roundtrip() {
        let (a, b) = tokio::io::duplex(4096);
        let key = AeadKey::new(CipherSuiteId::Aes128GcmSha256, &[0x11; 16]).unwrap();
        let keys = AppKeys {
            read_key: key.clone_key(),
            write_key: key.clone_key(),
            read_seq: 0,
            write_seq: 0,
        };
        let mut client = TlsStream::new(a, keys.clone());
        let mut server = TlsStream::new(b, keys);
        client.write_all(b"ping").await.unwrap();
        let mut buf = [0u8; 4];
        server.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"ping");
        // and the reverse direction (independent per-direction sequences)
        server.write_all(b"pong").await.unwrap();
        let mut buf = [0u8; 4];
        client.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"pong");
    }

    #[tokio::test]
    async fn roundtrip_with_explicit_key_iv() {
        // from_key_iv keys: non-zero write IV must round-trip through the
        // record stream (nonce = IV XOR seq).
        let (a, b) = duplex(4096);
        let key =
            AeadKey::from_key_iv(CipherSuiteId::Aes128GcmSha256, &[0x22; 16], [0x01; 12]).unwrap();
        let keys = AppKeys {
            read_key: key.clone_key(),
            write_key: key.clone_key(),
            read_seq: 0,
            write_seq: 0,
        };
        let mut client = TlsStream::new(a, keys.clone());
        let mut server = TlsStream::new(b, keys);
        client.write_all(b"hello").await.unwrap();
        let mut buf = [0u8; 5];
        server.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"hello");
    }

    #[tokio::test]
    async fn fragmented_records_reassemble_across_records() {
        // "hello" and " world" arrive as two separate TLS records; the reader
        // must present them as one continuous byte stream.
        let (a, b) = duplex(4096);
        let keys = test_keys();
        let mut writer = TlsStream::new(a, keys.clone());
        let mut reader = TlsStream::new(b, keys);
        writer.write_all(b"hello").await.unwrap();
        writer.write_all(b" world").await.unwrap();
        let mut buf = [0u8; 11];
        reader.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"hello world");
    }

    #[tokio::test]
    async fn record_split_across_inner_reads() {
        // Feed a single record one byte per write through a tiny duplex so the
        // reader's header/payload state machine must survive partial reads.
        let (a, b) = duplex(8);
        let keys = test_keys();
        let rec = raw_record(&keys, 0, CONTENT_APPLICATION_DATA, b"fragmented");
        let writer = tokio::spawn(async move {
            let mut b = b;
            for chunk in rec.chunks(1) {
                b.write_all(chunk).await.unwrap();
            }
        });
        let mut reader = TlsStream::new(a, keys);
        let mut buf = [0u8; 10];
        reader.read_exact(&mut buf).await.unwrap();
        writer.await.unwrap();
        assert_eq!(&buf, b"fragmented");
    }

    #[tokio::test]
    async fn close_notify_returns_eof_after_buffered_data() {
        // Buffered plaintext is served first; close_notify then surfaces as
        // EOF (0 bytes), not as an error.
        let (a, mut b) = duplex(4096);
        let keys = test_keys();
        let data_rec = raw_record(&keys, 0, CONTENT_APPLICATION_DATA, b"bye");
        let cn_rec = raw_record(&keys, 1, CONTENT_ALERT, &[1, 0]);
        tokio::spawn(async move {
            b.write_all(&data_rec).await.unwrap();
            b.write_all(&cn_rec).await.unwrap();
        });
        let mut reader = TlsStream::new(a, keys);
        let mut buf = [0u8; 8];
        let n = reader.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"bye");
        let n = reader.read(&mut buf).await.unwrap();
        assert_eq!(n, 0, "close_notify must surface as EOF");
        // EOF is sticky
        let n = reader.read(&mut buf).await.unwrap();
        assert_eq!(n, 0);
    }

    #[tokio::test]
    async fn non_close_alerts_error() {
        let (a, mut b) = duplex(4096);
        let keys = test_keys();
        let rec = raw_record(&keys, 0, CONTENT_ALERT, &[2, 40]); // fatal handshake_failure
        tokio::spawn(async move {
            b.write_all(&rec).await.unwrap();
        });
        let mut reader = TlsStream::new(a, keys);
        let mut buf = [0u8; 8];
        let err = reader.read(&mut buf).await.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::ConnectionReset);
    }

    #[tokio::test]
    async fn new_session_ticket_records_are_skipped() {
        // TLS 1.3 servers send NewSessionTicket as encrypted handshake
        // records right after the handshake; they must be consumed and
        // dropped so the following application data is delivered.
        let (a, mut b) = duplex(4096);
        let keys = test_keys();
        let nst = raw_record(
            &keys,
            0,
            CONTENT_HANDSHAKE,
            &[HS_NEW_SESSION_TICKET, 0, 0, 0],
        );
        let data = raw_record(&keys, 1, CONTENT_APPLICATION_DATA, b"ticket?");
        tokio::spawn(async move {
            b.write_all(&nst).await.unwrap();
            b.write_all(&data).await.unwrap();
        });
        let mut reader = TlsStream::new(a, keys);
        let mut buf = [0u8; 7];
        reader.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"ticket?");
    }

    #[tokio::test]
    async fn non_ticket_handshake_inner_content_is_rejected() {
        // Post-handshake handshake records other than NewSessionTicket (e.g.
        // a ServerHello — renegotiation) are refused.
        let (a, mut b) = duplex(4096);
        let keys = test_keys();
        let rec = raw_record(&keys, 0, CONTENT_HANDSHAKE, &[0x02, 0, 0, 0]);
        tokio::spawn(async move {
            b.write_all(&rec).await.unwrap();
        });
        let mut reader = TlsStream::new(a, keys);
        let mut buf = [0u8; 8];
        let err = reader.read(&mut buf).await.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(
            err.to_string().contains("after handshake"),
            "unexpected error message: {err}"
        );
    }

    #[tokio::test]
    async fn large_write_is_split_into_16kib_records() {
        // 20 000 bytes must leave as 2 records: 16 KiB + remainder, each
        // carrying at most 2^14 plaintext bytes.
        let (a, mut b) = duplex(1 << 16);
        let keys = test_keys();
        let payload: Vec<u8> = (0..20_000u32).map(|i| (i % 251) as u8).collect();
        let mut writer = TlsStream::new(a, keys.clone());
        writer.write_all(&payload).await.unwrap();
        drop(writer);

        let mut read_keys = keys;
        let mut got = Vec::new();
        let mut lens = Vec::new();
        for _ in 0..2 {
            let rec = read_record(&mut b).await.unwrap();
            assert_eq!(rec.content_type, CONTENT_APPLICATION_DATA);
            let mut ct = rec.payload;
            let pt = read_keys
                .read_key
                .open(read_keys.read_seq, &aead_aad(ct.len()), &mut ct)
                .unwrap();
            read_keys.read_seq += 1;
            let (inner_type, content) = strip_padding(&pt).unwrap();
            assert_eq!(inner_type, CONTENT_APPLICATION_DATA);
            lens.push(content.len());
            got.extend_from_slice(content);
        }
        assert_eq!(
            lens,
            vec![MAX_RECORD_PLAINTEXT, 20_000 - MAX_RECORD_PLAINTEXT]
        );
        assert_eq!(got, payload);
    }
}
