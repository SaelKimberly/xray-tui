//! VMess record tunnel stream: peels the AEAD response header on the first
//! read, then transparently decodes response records (read side) and encodes
//! request records (write side). Mirrors Xray-core
//! `proxy/vmess/encoding/client.go` (`DecodeResponseHeader`,
//! `DecodeResponseBody`, `EncodeRequestBody`) with the request option byte 0
//! — the server chunk-decodes AEAD bodies regardless
//! (`crypto.PlainChunkSizeParser`), so no ChunkStream/ChunkMasking option bit
//! is ever set.
//!
//! Record framing (both directions): `2B BE length | AES-128-GCM(key =
//! session.{request,response}_body_key, nonce = IV-with-first-2-bytes-
//! overwritten-by-BE-counter)` — counter starts 0 and increments per record
//! (Go `GenerateChunkNonce`); the nonce is the first 12 bytes of the modified
//! IV. The request direction uses the request keys + its own counter, the
//! response direction the response keys + its own counter.

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use aes_gcm::aead::Aead;
use aes_gcm::{Aes128Gcm, KeyInit, Nonce};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::BoxStream;
use crate::protocol::vmess::header::Session;
use crate::protocol::vmess::keys;

/// Peel salt paths (Go `aead.KDFSaltConstAEADRespHeader*`), as byte slices for
/// [`keys::kdf16_bytes_path`].
const RESP_LEN_KEY_SALT: &[u8] = b"AEAD Resp Header Len Key";
const RESP_LEN_IV_SALT: &[u8] = b"AEAD Resp Header Len IV";
const RESP_PAYLOAD_KEY_SALT: &[u8] = b"AEAD Resp Header Key";
const RESP_PAYLOAD_IV_SALT: &[u8] = b"AEAD Resp Header IV";

/// Read-side state machine (Copy so no borrow of `self` is held across
/// awaits; buffers live on the struct, fill counters in the state).
#[derive(Clone, Copy)]
enum ReadState {
    /// Filling the 18-byte AEAD response-header length ciphertext.
    PeelLen { filled: usize },
    /// Filling the (len+16)-byte AEAD response-header payload ciphertext.
    PeelPayload { total: usize, filled: usize },
    /// Filling the 2-byte BE record length.
    RecordLen { filled: usize },
    /// Filling the (len+16)-byte record ciphertext.
    RecordData { total: usize, filled: usize },
    /// Terminal error: every subsequent poll returns the same error.
    Dead(&'static str),
}

/// VMess client tunnel stream: response-header peel + record codec over the
/// secured connection.
pub struct VmessClientStream {
    inner: BoxStream,
    session: Session,
    // Read side.
    read_state: ReadState,
    /// Decrypted record data awaiting the caller (also the ciphertext staging
    /// buffer during `RecordData`; the state distinguishes the two roles).
    pending: Vec<u8>,
    pending_pos: usize,
    /// Staging for the 18-byte peel length AEAD.
    peel_len: [u8; 18],
    /// Staging for the (len+16)-byte peel payload AEAD (one-shot).
    peel_buf: Vec<u8>,
    /// Staging for the 2-byte record length.
    len_buf: [u8; 2],
    // Write side.
    write_pending: Option<Vec<u8>>,
    write_pos: usize,
    /// Original caller length the pending record was sealed from — returned
    /// once the record is fully flushed.
    write_len: usize,
    /// Expanded AES keys (per-direction) so record codec never re-derives a
    /// key schedule.
    req_cipher: Aes128Gcm,
    resp_cipher: Aes128Gcm,
    req_nonce: [u8; 12],
    req_counter: u16,
    resp_nonce: [u8; 12],
    resp_counter: u16,
}

impl VmessClientStream {
    /// Wrap `inner` with a VMess session: peels the response header on the
    /// first read, then codes AEAD records.
    pub fn new(inner: BoxStream, session: Session) -> Self {
        let mut req_nonce = [0u8; 12];
        req_nonce.copy_from_slice(&session.request_body_iv[..12]);
        let mut resp_nonce = [0u8; 12];
        resp_nonce.copy_from_slice(&session.response_body_iv[..12]);
        let req_cipher =
            Aes128Gcm::new_from_slice(&session.request_body_key).expect("16-byte request key");
        let resp_cipher =
            Aes128Gcm::new_from_slice(&session.response_body_key).expect("16-byte response key");
        Self {
            inner,
            session,
            read_state: ReadState::PeelLen { filled: 0 },
            pending: Vec::new(),
            pending_pos: 0,
            peel_len: [0u8; 18],
            peel_buf: Vec::new(),
            len_buf: [0u8; 2],
            write_pending: None,
            write_pos: 0,
            write_len: 0,
            req_cipher,
            resp_cipher,
            req_nonce,
            req_counter: 0,
            resp_nonce,
            resp_counter: 0,
        }
    }

    /// Record nonce: response/request IV with the first two bytes replaced by
    /// the BE counter (Go `GenerateChunkNonce`), truncated to 12 bytes.
    fn record_nonce(iv12: &[u8; 12], counter: u16) -> [u8; 12] {
        let mut nonce = *iv12;
        nonce[..2].copy_from_slice(&counter.to_be_bytes());
        nonce
    }
    /// Flush the pending write record to `inner`. Takes the record, position
    /// and inner as separate mutable borrows so no borrow spans the awaits.
    /// Returns `Ok(())` once the record is fully written (and clears it).
    fn flush_pending(
        inner: &mut BoxStream,
        rec: &mut Option<Vec<u8>>,
        pos: &mut usize,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            let r = match rec.as_ref() {
                Some(r) => r,
                None => return Poll::Ready(Ok(())),
            };
            if *pos >= r.len() {
                *rec = None;
                *pos = 0;
                return Poll::Ready(Ok(()));
            }
            let n = ready!(Pin::new(&mut *inner).poll_write(cx, &r[*pos..]))?;
            if n == 0 {
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "vmess tunnel inner write accepted 0 bytes",
                )));
            }
            *pos += n;
        }
    }
}

impl AsyncRead for VmessClientStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = &mut *self;
        if buf.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }
        loop {
            // Serve already-decrypted record data first (`RecordData` staging
            // also uses `pending`, so only drain outside it).
            if matches!(this.read_state, ReadState::RecordLen { filled: 0 })
                && this.pending_pos < this.pending.len()
            {
                let n = buf
                    .remaining()
                    .min(this.pending.len() - this.pending_pos);
                buf.put_slice(&this.pending[this.pending_pos..this.pending_pos + n]);
                this.pending_pos += n;
                if this.pending_pos == this.pending.len() {
                    this.pending.clear();
                    this.pending_pos = 0;
                }
                return Poll::Ready(Ok(()));
            }

            let state = this.read_state; // Copy — no borrow held across awaits
            match state {
                ReadState::Dead(msg) => {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        msg,
                    )));
                }
                ReadState::PeelLen { filled } => {
                    let mut rb = ReadBuf::new(&mut this.peel_len[filled..]);
                    ready!(Pin::new(&mut this.inner).poll_read(cx, &mut rb))?;
                    let got = rb.filled().len();
                    if got == 0 {
                        this.read_state =
                            ReadState::Dead("vmess response header truncated (EOF)");
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "vmess response header truncated (EOF)",
                        )));
                    }
                    let filled = filled + got;
                    if filled < 18 {
                        this.read_state = ReadState::PeelLen { filled };
                        continue;
                    }
                    // Decrypt the 2-byte plaintext length.
                    let len_key = keys::kdf16_bytes_path(
                        &this.session.response_body_key,
                        &[RESP_LEN_KEY_SALT],
                    );
                    let len_iv = keys::kdf16_bytes_path(
                        &this.session.response_body_iv,
                        &[RESP_LEN_IV_SALT],
                    );
                    let pt = match Aes128Gcm::new_from_slice(&len_key)
                        .expect("16-byte KDF output")
                        .decrypt(Nonce::from_slice(&len_iv[..12]), &this.peel_len[..])
                    {
                        Ok(pt) => pt,
                        Err(_) => {
                            this.read_state =
                                ReadState::Dead("vmess response header length decrypt failed");
                            return Poll::Ready(Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "vmess response header length decrypt failed",
                            )));
                        }
                    };
                    if pt.len() != 2 {
                        this.read_state =
                            ReadState::Dead("vmess response header length corrupt");
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "vmess response header length corrupt",
                        )));
                    }
                    let total = u16::from_be_bytes([pt[0], pt[1]]) as usize + 16;
                    this.peel_buf.resize(total, 0);
                    this.read_state = ReadState::PeelPayload { total, filled: 0 };
                    continue;
                }
                ReadState::PeelPayload { total, filled } => {
                    let mut rb = ReadBuf::new(&mut this.peel_buf[filled..total]);
                    ready!(Pin::new(&mut this.inner).poll_read(cx, &mut rb))?;
                    let got = rb.filled().len();
                    if got == 0 {
                        this.read_state =
                            ReadState::Dead("vmess response header payload truncated (EOF)");
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "vmess response header payload truncated (EOF)",
                        )));
                    }
                    let filled = filled + got;
                    if filled < total {
                        this.read_state = ReadState::PeelPayload { total, filled };
                        continue;
                    }
                    let payload_key = keys::kdf16_bytes_path(
                        &this.session.response_body_key,
                        &[RESP_PAYLOAD_KEY_SALT],
                    );
                    let payload_iv = keys::kdf16_bytes_path(
                        &this.session.response_body_iv,
                        &[RESP_PAYLOAD_IV_SALT],
                    );
                    let pt = match Aes128Gcm::new_from_slice(&payload_key)
                        .expect("16-byte KDF output")
                        .decrypt(Nonce::from_slice(&payload_iv[..12]), &this.peel_buf[..total])
                    {
                        Ok(pt) => pt,
                        Err(_) => {
                            this.peel_buf = Vec::new();
                            this.read_state =
                                ReadState::Dead("vmess response header decrypt failed");
                            return Poll::Ready(Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "vmess response header decrypt failed",
                            )));
                        }
                    };
                    this.peel_buf = Vec::new(); // one-shot peel; drop staging
                    if pt.is_empty() {
                        this.read_state =
                            ReadState::Dead("vmess response header payload too short");
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "vmess response header payload too short",
                        )));
                    }
                    // Echo check: payload[0] must equal the request's random
                    // response header byte.
                    if pt[0] != this.session.response_header {
                        this.read_state =
                            ReadState::Dead("vmess response header echo mismatch");
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "vmess response header echo mismatch",
                        )));
                    }
                    this.read_state = ReadState::RecordLen { filled: 0 };
                    continue;
                }
                ReadState::RecordLen { filled } => {
                    let mut rb = ReadBuf::new(&mut this.len_buf[filled..]);
                    ready!(Pin::new(&mut this.inner).poll_read(cx, &mut rb))?;
                    let got = rb.filled().len();
                    if got == 0 {
                        // Clean EOF at a record boundary (Go `io.ReadFull` on
                        // the size bytes returns `io.EOF`) — read_to_end
                        // finishes normally.
                        if filled > 0 {
                            this.read_state =
                                ReadState::Dead("vmess response record header truncated (EOF)");
                            return Poll::Ready(Err(io::Error::new(
                                io::ErrorKind::UnexpectedEof,
                                "vmess response record header truncated (EOF)",
                            )));
                        }
                        return Poll::Ready(Ok(()));
                    }
                    let filled = filled + got;
                    if filled < 2 {
                        this.read_state = ReadState::RecordLen { filled };
                        continue;
                    }
                    let len = u16::from_be_bytes(this.len_buf);
                    if len == 0 {
                        // Zero-length record: skip, read the next one.
                        this.read_state = ReadState::RecordLen { filled: 0 };
                        continue;
                    }
                    let total = len as usize + 16;
                    this.pending.resize(total, 0);
                    this.pending_pos = 0;
                    this.read_state = ReadState::RecordData { total, filled: 0 };
                    continue;
                }
                ReadState::RecordData { total, filled } => {
                    let mut rb = ReadBuf::new(&mut this.pending[filled..total]);
                    ready!(Pin::new(&mut this.inner).poll_read(cx, &mut rb))?;
                    let got = rb.filled().len();
                    if got == 0 {
                        this.read_state =
                            ReadState::Dead("vmess response record truncated (EOF)");
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "vmess response record truncated (EOF)",
                        )));
                    }
                    let filled = filled + got;
                    if filled < total {
                        this.read_state = ReadState::RecordData { total, filled };
                        continue;
                    }
                    let nonce = Self::record_nonce(&this.resp_nonce, this.resp_counter);
                    this.resp_counter = this.resp_counter.wrapping_add(1);
                    match this
                        .resp_cipher
                        .decrypt(Nonce::from_slice(&nonce), &this.pending[..total])
                    {
                        Ok(pt) => {
                            this.pending = pt;
                            this.pending_pos = 0;
                            this.read_state = ReadState::RecordLen { filled: 0 };
                            continue;
                        }
                        Err(_) => {
                            this.read_state =
                                ReadState::Dead("vmess response record decrypt failed");
                            return Poll::Ready(Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "vmess response record decrypt failed",
                            )));
                        }
                    }
                }
            }
        }
    }
}

impl AsyncWrite for VmessClientStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = &mut *self;
        if buf.is_empty() {
            // Empty writes are skipped (mirror xray: no zero-length record).
            return Poll::Ready(Ok(0));
        }
        // Seal the caller's buffer into a single record. If a previous record
        // is still being flushed, the caller is retrying with the same buffer
        // (tokio poll contract) — the pending record was built from it.
        if this.write_pending.is_none() {
            let nonce = Self::record_nonce(&this.req_nonce, this.req_counter);
            let ct = match this.req_cipher.encrypt(Nonce::from_slice(&nonce), buf) {
                Ok(ct) => ct,
                Err(_) => {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "vmess request record seal failed",
                    )));
                }
            };
            let mut rec = Vec::with_capacity(2 + ct.len());
            rec.extend_from_slice(&(buf.len() as u16).to_be_bytes());
            rec.extend_from_slice(&ct);
            this.write_pending = Some(rec);
            this.write_pos = 0;
            this.write_len = buf.len();
            this.req_counter = this.req_counter.wrapping_add(1);
        }
        // Flush the pending record; return the ORIGINAL length once fully
        // written (partial inner writes resume on subsequent polls).
        match Self::flush_pending(
            &mut this.inner,
            &mut this.write_pending,
            &mut this.write_pos,
            cx,
        ) {
            Poll::Ready(Ok(())) => {
                let original = this.write_len;
                this.write_len = 0;
                Poll::Ready(Ok(original))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests {
    use aes_gcm::aead::Aead;
    use aes_gcm::{Aes128Gcm, KeyInit, Nonce};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;
    use crate::protocol::vmess::header::Session;
    use crate::protocol::vmess::keys::{kdf16_bytes_path, sha256_first16};

    /// Server-side response header seal (mirrors Go `EncodeResponseHeader`):
    /// lenAEAD(18) + payloadAEAD(len+16), both AES-128-GCM, AAD nil, nonce = KDF [:12].
    fn seal_response_header(key: &[u8; 16], iv: &[u8; 16], payload: &[u8]) -> Vec<u8> {
        fn ae(key: &[u8; 16], iv: &[u8; 16], key_salt: &str, iv_salt: &str, msg: &[u8]) -> Vec<u8> {
            let k = kdf16_bytes_path(key, &[key_salt.as_bytes()]);
            let v = kdf16_bytes_path(iv, &[iv_salt.as_bytes()]);
            Aes128Gcm::new_from_slice(&k)
                .unwrap()
                .encrypt(Nonce::from_slice(&v[..12]), msg)
                .unwrap()
        }
        let mut wire = ae(
            key,
            iv,
            "AEAD Resp Header Len Key",
            "AEAD Resp Header Len IV",
            &(payload.len() as u16).to_be_bytes(),
        );
        wire.extend_from_slice(&ae(key, iv, "AEAD Resp Header Key", "AEAD Resp Header IV", payload));
        wire
    }

    /// Server-side response record (mirrors Go `EncodeResponseBody` chunk):
    /// 2B BE length + AES-128-GCM data, counter nonce from the response IV.
    fn seal_record(key: &[u8; 16], iv: &[u8; 16], counter: u16, data: &[u8]) -> Vec<u8> {
        let mut nonce = *iv;
        nonce[..2].copy_from_slice(&counter.to_be_bytes());
        let ct = Aes128Gcm::new_from_slice(key)
            .unwrap()
            .encrypt(Nonce::from_slice(&nonce[..12]), data)
            .unwrap();
        (data.len() as u16).to_be_bytes().into_iter().chain(ct).collect()
    }

    #[test]
    fn records_roundtrip_with_peel() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let (client_side, mut server_side) = tokio::io::duplex(8192);
            let mut session = Session::new();
            session.request_body_iv = [0x11; 16];
            session.request_body_key = [0x22; 16];
            session.response_header = 0x99;
            session.response_body_key = sha256_first16(&session.request_body_key);
            session.response_body_iv = sha256_first16(&session.request_body_iv);

            // fake server: response header ([resp_header, option 0, cmd 0, len 0])
            // then one data record with counter 0.
            let mut wire = seal_response_header(
                &session.response_body_key,
                &session.response_body_iv,
                &[0x99, 0, 0, 0],
            );
            wire.extend_from_slice(&seal_record(
                &session.response_body_key,
                &session.response_body_iv,
                0,
                b"hello",
            ));
            server_side.write_all(&wire).await.unwrap();
            drop(server_side); // EOF after the data — read_to_end can finish

            let mut tunnel = VmessClientStream::new(Box::new(client_side), session);
            let mut got = Vec::new();
            tunnel.read_to_end(&mut got).await.unwrap();
            assert_eq!(got, b"hello");
        });
    }

    #[test]
    fn peel_rejects_wrong_echo_byte() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let (client_side, mut server_side) = tokio::io::duplex(8192);
            let mut session = Session::new();
            session.request_body_iv = [0x11; 16];
            session.request_body_key = [0x22; 16];
            session.response_header = 0x99;
            session.response_body_key = sha256_first16(&session.request_body_key);
            session.response_body_iv = sha256_first16(&session.request_body_iv);

            // wrong echo byte 0x98 -> must error, not stream garbage
            let wire = seal_response_header(
                &session.response_body_key,
                &session.response_body_iv,
                &[0x98, 0, 0, 0],
            );
            server_side.write_all(&wire).await.unwrap();
            drop(server_side);

            let mut tunnel = VmessClientStream::new(Box::new(client_side), session);
            let mut got = Vec::new();
            let res = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                tunnel.read_to_end(&mut got),
            )
            .await;
            assert!(res.is_err() || matches!(res.unwrap(), Err(_)), "must fail, got {got:?}");
        });
    }

    /// Client-side inverse of [`seal_record`]: reads one record off `stream`
    /// and returns its plaintext (2B BE length, then len+16 ciphertext).
    async fn read_record(
        stream: &mut tokio::io::DuplexStream,
        key: &[u8; 16],
        iv: &[u8; 16],
        counter: u16,
    ) -> Vec<u8> {
        use aes_gcm::aead::Aead;
        let mut len_buf = [0u8; 2];
        stream.read_exact(&mut len_buf).await.unwrap();
        let len = u16::from_be_bytes(len_buf) as usize;
        let mut ct = vec![0u8; len + 16];
        stream.read_exact(&mut ct).await.unwrap();
        let mut nonce = *iv;
        nonce[..2].copy_from_slice(&counter.to_be_bytes());
        Aes128Gcm::new_from_slice(key)
            .unwrap()
            .decrypt(Nonce::from_slice(&nonce[..12]), &ct[..])
            .unwrap()
    }

    #[test]
    fn write_seals_request_records_with_counters() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let (client_side, mut server_side) = tokio::io::duplex(8192);
            let mut session = Session::new();
            session.request_body_iv = [0x11; 16];
            session.request_body_key = [0x22; 16];
            session.response_header = 0x99;
            session.response_body_key = sha256_first16(&session.request_body_key);
            session.response_body_iv = sha256_first16(&session.request_body_iv);
            let (req_key, req_iv) = (session.request_body_key, session.request_body_iv);

            let mut tunnel = VmessClientStream::new(Box::new(client_side), session);
            tunnel.write_all(b"first").await.unwrap();
            tunnel.write_all(b"second-payload").await.unwrap();
            tunnel.flush().await.unwrap();

            // Each write_all produced exactly one record with its own counter.
            assert_eq!(read_record(&mut server_side, &req_key, &req_iv, 0).await, b"first");
            assert_eq!(
                read_record(&mut server_side, &req_key, &req_iv, 1).await,
                b"second-payload"
            );
        });
    }

    #[test]
    fn write_resumes_partial_record_flush() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            // 1-byte duplex: the inner write accepts a byte per poll, so the
            // record flush must resume across polls and still report the
            // ORIGINAL length to write_all.
            let (client_side, mut server_side) = tokio::io::duplex(1);
            let mut session = Session::new();
            session.request_body_iv = [0x11; 16];
            session.request_body_key = [0x22; 16];
            session.response_header = 0x99;
            session.response_body_key = sha256_first16(&session.request_body_key);
            session.response_body_iv = sha256_first16(&session.request_body_iv);
            let (req_key, req_iv) = (session.request_body_key, session.request_body_iv);

            // Drain the wire concurrently so the writer never blocks on the
            // full 1-byte buffer.
            let reader = tokio::spawn(async move {
                read_record(&mut server_side, &req_key, &req_iv, 0).await
            });

            let mut tunnel = VmessClientStream::new(Box::new(client_side), session);
            tunnel.write_all(b"payload-larger-than-one-byte").await.unwrap();
            tunnel.flush().await.unwrap();
            drop(tunnel);

            assert_eq!(reader.await.unwrap(), b"payload-larger-than-one-byte");
        });
    }
}
