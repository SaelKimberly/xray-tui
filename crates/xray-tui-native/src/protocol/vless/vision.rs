//! XTLS Vision codec (`xtls-rprx-vision`): padded frames, TLS filter, and
//! the Direct splice state machine.
//!
//! Faithful Rust port of xray-core's `VisionWriter` / `VisionReader`
//! (`proxy/proxy.go` `XtlsPadding` / `XtlsUnpadding` / `XtlsFilterTls`):
//! uplink writes are padded into `[uuid?][cmd][clen:2][plen:2][content]
//! [padding]` frames to camouflage the inner TLS handshake, downlink reads
//! are unpadded, the inner TLS is sniffed to detect TLS 1.3, and both
//! directions switch to a raw socket relay on the `Direct` command. Wire
//! spec: docs/superpowers/specs/2026-08-13-vless-flow-vision-design.md
//! §4.2-§4.5.
//!
//! Layer order (client): `app -> VisionStream -> engine TlsStream -> tcp`.
//! The caller MUST write the VLESS header and the camouflage frame through
//! the raw stream before wrapping ([`VisionStream::camouflage_frame`] — the
//! camouflage consumes the writer's UUID, so `VisionStream` frames never
//! carry one).
//!
//! Everything here is consumed by the next plan task (the VLESS `connect`
//! wiring): until then the module is unreferenced, so dead code is allowed
//! crate-wide.

#![allow(dead_code)]

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use ring::rand::SecureRandom;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use bytes::Buf;

/// The vision flow name (VLESS `addons.flow`).
pub const FLOW_XTLS_RPRX_VISION: &str = "xtls-rprx-vision";

/// Padding command: keep padding, more frames follow.
pub const CMD_CONTINUE: u8 = 0x00;
/// Padding command: stop padding, stay inside the outer TLS.
pub const CMD_END: u8 = 0x01;
/// Padding command: stop padding, switch to a raw socket relay.
pub const CMD_DIRECT: u8 = 0x02;

/// Inner TLS application-data record start (xray `TlsApplicationDataStart`).
const TLS_APP_DATA_START: [u8; 3] = [0x17, 0x03, 0x03];
/// Inner TLS `ServerHello` record start (xray `TlsServerHandShakeStart`).
const TLS_SERVER_HELLO_START: [u8; 3] = [0x16, 0x03, 0x03];
/// Inner TLS `ClientHello` record start (xray `TlsClientHandShakeStart`).
const TLS_CLIENT_HELLO_START: [u8; 2] = [0x16, 0x03];
/// TLS handshake message types (RFC 8446 §4).
const TLS_HANDSHAKE_TYPE_CLIENT_HELLO: u8 = 0x01;
const TLS_HANDSHAKE_TYPE_SERVER_HELLO: u8 = 0x02;
/// The `supported_versions` extension announcing TLS 1.3
/// (xray `Tls13SupportedVersions`): `00 2b 00 02 03 04`.
const TLS13_SUPPORTED_VERSIONS: [u8; 6] = [0x00, 0x2b, 0x00, 0x02, 0x03, 0x04];

/// Maximum frame size — xray `buf.Size`. One padded frame always fits one
/// outer TLS record.
const MAX_FRAME: usize = 8192;
/// Frame overhead without content: 16 uuid + 1 cmd + 2 content + 2 pad.
const FRAME_OVERHEAD: usize = 21;
/// Filter chunk budget (xray `NumberOfPacketToFilter`).
const FILTER_BUDGET: i32 = 8;

/// Direct-mode splice seam: the engine `TlsStream` exposes per-direction
/// record-layer bypass; the codec calls it through this trait so it stays
/// generic over the stream.
pub trait DirectMode {
    /// Switch the write side to raw writes to the underlying transport.
    fn set_write_direct(&mut self);
    /// Switch the read side to raw reads from the underlying transport.
    fn set_read_direct(&mut self);
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send> DirectMode
    for xray_tui_tls::record::stream::TlsStream<S>
{
    fn set_write_direct(&mut self) {
        Self::set_write_direct(self);
    }
    fn set_read_direct(&mut self) {
        Self::set_read_direct(self);
    }
}

/// Client-side vision stream: pads uplink writes, unpads downlink reads,
/// sniffs the inner TLS, and switches to a raw socket relay on `Direct`.
pub struct VisionStream<S> {
    inner: S,
    rng: ring::rand::SystemRandom,
    writer: WriterState,
    reader: ReaderState,
    /// Shared TLS filter — fed by BOTH directions; the budget is
    /// per-connection and `enable_xtls` is set by the downlink `ServerHello`
    /// sniff (spec §4.5-§4.6).
    filter: TlsFilter,
    /// Outgoing bytes not yet flushed to `inner` (padded frames, possibly
    /// followed by the raw tail after a Direct frame).
    write_buf: bytes::BytesMut,
    /// When set, the number of frame bytes still ahead of the raw tail in
    /// `write_buf`: once zero, the Direct frame is fully written and the
    /// write side must switch to raw before the tail goes out.
    direct_boundary: Option<usize>,
    /// Unpadded downlink content awaiting the app.
    read_buf: bytes::BytesMut,
    /// Bytes after a Direct frame — the start of the raw stream.
    raw_leftover: bytes::BytesMut,
    /// App bytes accepted by the in-flight write (returned once the
    /// buffered frames drain).
    accepted: usize,
    /// Whether the inner stream supports the Direct splice. The real path
    /// (`new`) sets it true; the test constructor sets it false so the
    /// codec runs over plain duplexes without touching the record layer.
    directable: bool,
}

/// Uplink writer state (xray `OutboundState.IsPadding`).
struct WriterState {
    /// True while writes are wrapped in vision frames.
    is_padding: bool,
    /// True once the Direct frame was flushed — raw writes from here on.
    direct: bool,
}

/// Downlink reader state.
struct ReaderState {
    unpad: Unpadder,
    /// True once a Direct frame arrived — raw reads from here on.
    direct: bool,
}

impl<S: AsyncRead + AsyncWrite + Unpin> VisionStream<S> {
    /// `user_uuid` = the VLESS user id bytes. The caller MUST have already
    /// written the VLESS header AND the camouflage frame (Task 3) before
    /// wrapping; the writer's UUID is consumed by the camouflage frame.
    #[must_use]
    pub fn new(inner: S, user_uuid: [u8; 16], rng: ring::rand::SystemRandom) -> Self {
        Self {
            inner,
            rng,
            writer: WriterState {
                is_padding: true,
                direct: false,
            },
            reader: ReaderState {
                unpad: Unpadder::new(user_uuid),
                direct: false,
            },
            filter: TlsFilter::new(),
            write_buf: bytes::BytesMut::new(),
            direct_boundary: None,
            read_buf: bytes::BytesMut::new(),
            raw_leftover: bytes::BytesMut::new(),
            accepted: 0,
            directable: true,
        }
    }

    /// Test constructor: the same codec over a non-directable duplex.
    #[cfg(test)]
    fn new_test(inner: S, user_uuid: [u8; 16], rng: ring::rand::SystemRandom) -> Self {
        let mut this = Self::new(inner, user_uuid, rng);
        this.directable = false;
        this
    }

    /// The header-camouflage frame bytes: one Continue long-padding frame
    /// carrying the UUID. The caller writes it via the raw stream right
    /// after the VLESS header (spec §4.6 step 3, deviation 1 — xray's
    /// 500 ms timer is skipped; same wire bytes, emitted immediately).
    #[must_use]
    pub fn camouflage_frame(user_uuid: &[u8; 16], rng: &ring::rand::SystemRandom) -> bytes::Bytes {
        bytes::Bytes::from(encode_frame(
            Some(user_uuid),
            CMD_CONTINUE,
            &[],
            padding_len(0, true, rng),
        ))
    }

    fn has_pending_write(&self) -> bool {
        !self.write_buf.is_empty() || self.direct_boundary.is_some()
    }

    /// Writer state machine (spec §4.3, sing-vmess model): pad each app
    /// chunk into vision frames. The Direct frame is the LAST padded frame;
    /// any trailing bytes of the chunk go out raw.
    fn pad_chunk(&mut self, chunk: &[u8]) -> (Vec<Vec<u8>>, Vec<u8>, bool) {
        let mut out = Vec::new();
        let mut raw_tail = Vec::new();
        let mut direct = false;
        if !self.writer.is_padding {
            // Padding ended earlier (End/Direct): the chunk passes through
            // the record layer unpadded (the outer TLS continues).
            out.push(chunk.to_vec());
            return (out, raw_tail, direct);
        }
        // Reshape chunks >= 8171 at the last 0x17 0x03 0x03 boundary
        // (ReshapeMultiBuffer, spec §4.3) so each piece keeps the frame cap.
        let pieces: Vec<&[u8]> = if chunk.len() >= MAX_FRAME - FRAME_OVERHEAD {
            reshape(chunk)
        } else {
            vec![chunk]
        };
        let long_padding = self.filter.is_tls;
        for (i, piece) in pieces.iter().enumerate() {
            let is_last = i == pieces.len() - 1;
            // Go gates on `b.Len() >= 6` (proxy.go:365): a 6-byte record is
            // the 5-byte header plus one payload byte. The `IsCompleteRecord`
            // check is deliberately omitted — sanctioned divergence: the
            // inner engine writes one complete TLS record per poll (spec
            // §9.2, sing-vmess model), so a `0x17 0x03 0x03`-prefixed piece
            // is a record start.
            let is_app_data = piece.len() >= 6 && piece.starts_with(&TLS_APP_DATA_START);
            if self.filter.is_tls && is_app_data {
                // Inner TLS app data with the filter warmed: Direct (TLS 1.3,
                // splice) or End (TLS 1.2, padding stops but the outer TLS
                // continues) — this is the last padded frame.
                let (cmd, d) = if self.filter.enable_xtls {
                    (CMD_DIRECT, true)
                } else {
                    (CMD_END, false)
                };
                out.push(encode_frame(
                    None,
                    cmd,
                    piece,
                    padding_len(piece.len(), true, &self.rng),
                ));
                self.writer.is_padding = false;
                if d {
                    direct = true;
                    raw_tail.extend_from_slice(&pieces[i + 1..].concat());
                } else {
                    // End: the remaining pieces stay inside the outer TLS,
                    // unpadded — never dropped (sing-vmess writer model).
                    for rest in &pieces[i + 1..] {
                        out.push(rest.to_vec());
                    }
                }
                return (out, raw_tail, direct);
            }
            if !self.filter.is_tls12_or_above && self.filter.budget <= 1 {
                // Non-TLS / TLS 1.2 early finish (the filter budget is
                // exhausted before any ServerHello): End frame, the rest
                // unpadded via the outer TLS.
                out.push(encode_frame(
                    None,
                    CMD_END,
                    piece,
                    padding_len(piece.len(), long_padding, &self.rng),
                ));
                self.writer.is_padding = false;
                for rest in &pieces[i + 1..] {
                    out.push(rest.to_vec());
                }
                return (out, raw_tail, direct);
            }
            let cmd = if is_last && !self.writer.is_padding {
                CMD_END
            } else {
                CMD_CONTINUE
            };
            // (is_padding only flips false in the branches above, which
            // return; this generic branch always runs while is_padding, so
            // cmd is CMD_CONTINUE — kept for parity with xray's command
            // selection.)
            out.push(encode_frame(
                None,
                cmd,
                piece,
                padding_len(piece.len(), long_padding, &self.rng),
            ));
        }
        (out, raw_tail, direct)
    }

    /// Drain `write_buf` through `inner`. When the buffered bytes include a
    /// Direct frame (a boundary is set), the record layer is flushed and
    /// the write side switched to raw exactly at the boundary, so the raw
    /// tail bypasses the record layer.
    fn poll_drain(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>>
    where
        S: DirectMode,
    {
        loop {
            if let Some(b) = self.direct_boundary {
                if b == 0 {
                    // The Direct frame is fully written through the record
                    // layer: flush it, then splice before the raw tail.
                    ready!(Pin::new(&mut self.inner).poll_flush(cx))?;
                    if self.directable {
                        self.inner.set_write_direct();
                    }
                    self.writer.direct = true;
                    self.direct_boundary = None;
                    continue;
                }
                // Write only frame bytes up to the boundary — never past it
                // in one call, or the raw tail would leak through the
                // record layer.
                let take = b.min(self.write_buf.len());
                let n = ready!(Pin::new(&mut self.inner).poll_write(cx, &self.write_buf[..take]))?;
                if n == 0 {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "vision pending write returned 0",
                    )));
                }
                self.write_buf.advance(n);
                self.direct_boundary = Some(b - n);
                continue;
            }
            if self.write_buf.is_empty() {
                return Poll::Ready(Ok(()));
            }
            let n = ready!(Pin::new(&mut self.inner).poll_write(cx, &self.write_buf))?;
            if n == 0 {
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "vision pending write returned 0",
                )));
            }
            self.write_buf.advance(n);
        }
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin + DirectMode> AsyncWrite for VisionStream<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = &mut *self;
        // Drain an in-flight write (frame bytes and/or raw tail) first —
        // also after the splice, when the buffered tail is already raw.
        if this.has_pending_write() {
            ready!(this.poll_drain(cx))?;
            let accepted = this.accepted;
            this.accepted = 0;
            return Poll::Ready(Ok(accepted));
        }
        if this.writer.direct {
            return Pin::new(&mut this.inner).poll_write(cx, buf);
        }
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }
        // Process the new chunk: filter, then pad per the writer state.
        this.filter.feed(buf);
        let (frames, raw_tail, direct) = this.pad_chunk(buf);
        // Write the frames through the record layer.
        let mut pending = None;
        'frames: for (i, f) in frames.iter().enumerate() {
            let mut off = 0;
            while off < f.len() {
                match Pin::new(&mut this.inner).poll_write(cx, &f[off..]) {
                    Poll::Ready(Ok(0)) => {
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::WriteZero,
                            "vision frame write returned 0",
                        )));
                    }
                    Poll::Ready(Ok(n)) => off += n,
                    Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                    Poll::Pending => {
                        pending = Some((i, off));
                        break 'frames;
                    }
                }
            }
        }
        if let Some((i, off)) = pending {
            // Buffer everything unwritten — the current frame's remainder,
            // the later frames, and the raw tail — so no byte is lost; the
            // splice happens once the boundary drains.
            let mut rest = Vec::with_capacity(
                frames[i].len() - off
                    + frames[i + 1..].iter().map(Vec::len).sum::<usize>()
                    + raw_tail.len(),
            );
            for (j, f) in frames.iter().enumerate().skip(i) {
                rest.extend_from_slice(if j == i { &f[off..] } else { f });
            }
            let frame_bytes = rest.len();
            this.write_buf.clear();
            this.write_buf.extend_from_slice(&rest);
            this.write_buf.extend_from_slice(&raw_tail);
            this.direct_boundary = direct.then_some(frame_bytes);
            this.accepted = buf.len();
            return Poll::Pending;
        }
        // All frames written. If this chunk carried the Direct frame, flush
        // the record layer and splice the write side before the raw tail.
        if direct {
            ready!(Pin::new(&mut this.inner).poll_flush(cx))?;
            if this.directable {
                this.inner.set_write_direct();
            }
            this.writer.direct = true;
        }
        if !raw_tail.is_empty() {
            // The inner is raw now (Direct case): write the tail directly.
            let mut off = 0;
            loop {
                match Pin::new(&mut this.inner).poll_write(cx, &raw_tail[off..]) {
                    Poll::Ready(Ok(0)) => {
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::WriteZero,
                            "vision raw tail write returned 0",
                        )));
                    }
                    Poll::Ready(Ok(n)) => {
                        off += n;
                        if off == raw_tail.len() {
                            break;
                        }
                    }
                    Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                    Poll::Pending => {
                        this.write_buf.clear();
                        this.write_buf.extend_from_slice(&raw_tail[off..]);
                        this.direct_boundary = None;
                        this.accepted = buf.len();
                        return Poll::Pending;
                    }
                }
            }
        }
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = &mut *self;
        if this.has_pending_write() {
            ready!(this.poll_drain(cx))?;
            this.accepted = 0;
        }
        Pin::new(&mut this.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = &mut *self;
        if this.has_pending_write() {
            ready!(this.poll_drain(cx))?;
            this.accepted = 0;
        }
        Pin::new(&mut this.inner).poll_shutdown(cx)
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin + DirectMode> AsyncRead for VisionStream<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = &mut *self;
        // Direct mode: drain the spliced leftover first, then raw reads.
        if this.reader.direct {
            if !this.raw_leftover.is_empty() {
                let n = this.raw_leftover.len().min(buf.remaining());
                buf.put_slice(&this.raw_leftover[..n]);
                this.raw_leftover.advance(n);
                return Poll::Ready(Ok(()));
            }
            return Pin::new(&mut this.inner).poll_read(cx, buf);
        }
        // Serve previously unpadded content before touching the wire.
        if !this.read_buf.is_empty() {
            let n = this.read_buf.len().min(buf.remaining());
            buf.put_slice(&this.read_buf[..n]);
            this.read_buf.advance(n);
            return Poll::Ready(Ok(()));
        }
        // Read a decrypted chunk from the record layer, unpad it, retry.
        let mut chunk = [0u8; 16384];
        let mut rb = ReadBuf::new(&mut chunk);
        ready!(Pin::new(&mut this.inner).poll_read(cx, &mut rb))?;
        let n = rb.filled().len();
        if n == 0 {
            return Poll::Ready(Ok(())); // EOF
        }
        let data = &chunk[..n];
        if this.reader.unpad.plain_passthrough {
            // Passthrough delivers everything unchanged (still sniffing).
            this.filter.feed(data);
            this.read_buf.extend_from_slice(data);
        } else {
            let mut content = Vec::with_capacity(data.len());
            this.reader.unpad.feed(data, &mut content);
            this.filter.feed(&content);
            if this.reader.unpad.direct {
                this.reader.direct = true;
                this.reader.unpad.direct = false;
                // Everything the unpadder produced after the Direct frame —
                // its own payload plus any raw bytes that followed it in
                // the same chunk — is the start of the raw stream.
                this.raw_leftover.extend_from_slice(&content);
                if this.directable {
                    this.inner.set_read_direct();
                }
                let n = this.raw_leftover.len().min(buf.remaining());
                buf.put_slice(&this.raw_leftover[..n]);
                this.raw_leftover.advance(n);
                return Poll::Ready(Ok(()));
            }
            this.read_buf.extend_from_slice(&content);
        }
        let n = this.read_buf.len().min(buf.remaining());
        buf.put_slice(&this.read_buf[..n]);
        this.read_buf.advance(n);
        Poll::Ready(Ok(()))
    }
}

/// Encode one padded frame: `[uuid?][cmd][clen:2][plen:2][content][zeros]`.
/// `uuid: Option<&[u8; 16]>` — `Some` only for the first frame of a
/// direction. Returns the frame bytes.
///
/// Panics if `content.len() + 21 + pad_len > MAX_FRAME` — the caller must
/// reshape chunks >= 8171 first (the writer's `reshape` guard).
fn encode_frame(uuid: Option<&[u8; 16]>, cmd: u8, content: &[u8], pad_len: usize) -> Vec<u8> {
    assert!(
        content.len() + FRAME_OVERHEAD + pad_len <= MAX_FRAME,
        "vision frame exceeds MAX_FRAME: content {} + overhead {FRAME_OVERHEAD} + pad {pad_len} > {MAX_FRAME}",
        content.len()
    );
    let mut out = Vec::with_capacity(content.len() + FRAME_OVERHEAD + pad_len);
    if let Some(u) = uuid {
        out.extend_from_slice(u);
    }
    let clen = u16::try_from(content.len()).expect("content fits u16 (frame cap)");
    let plen = u16::try_from(pad_len).expect("pad fits u16 (frame cap)");
    out.push(cmd);
    out.extend_from_slice(&clen.to_be_bytes());
    out.extend_from_slice(&plen.to_be_bytes());
    out.extend_from_slice(content);
    out.resize(out.len() + pad_len, 0);
    out
}

/// `XtlsPadding` port (spec §4.2): compute `pad_len` for the given content.
///
/// Mirrors xray exactly: long padding (`900 + rand(0..500) - content`) only
/// when `long_padding && content < 900`, else `rand(0..256)`, capped so one
/// frame never exceeds `MAX_FRAME`. The saturating cap also guards the
/// (unreachable) underflow path; the caller reshapes chunks >= 8171 first.
fn padding_len(content_len: usize, long_padding: bool, rng: &ring::rand::SystemRandom) -> usize {
    let content_len = u32::try_from(content_len).expect("content fits u32");
    let raw = if long_padding && content_len < 900 {
        // Long padding: 900 + rand(0..500) - content; no underflow possible
        // (content <= 899 leaves at least 900 + 0 - 899 = 1).
        900 + rand_u32(rng, 500) - content_len
    } else {
        rand_u32(rng, 256)
    };
    let cap = u32::try_from(MAX_FRAME - FRAME_OVERHEAD)
        .expect("cap fits u32")
        .saturating_sub(content_len);
    usize::try_from(raw.min(cap)).expect("pad fits usize")
}

/// Rejection-sample `[0, bound)` from ring's CSPRNG — no modulo bias (the
/// xhttp padding pattern). `bound` must be > 0.
fn rand_u32(rng: &ring::rand::SystemRandom, bound: u32) -> u32 {
    debug_assert!(bound > 0);
    // 2^32 % bound: samples below this threshold are rejected.
    let threshold = bound.wrapping_neg() % bound;
    loop {
        let mut buf = [0u8; 4];
        rng.fill(&mut buf).expect("system rng failure");
        let v = u32::from_le_bytes(buf);
        if v >= threshold {
            return v % bound;
        }
    }
}

/// `XtlsUnpadding` port (spec §4.4). Splits an incoming chunk into unpadded
/// content bytes; the `Remaining*` fields carry partial frames across
/// chunks.
struct Unpadder {
    user_uuid: [u8; 16],
    remaining_command: i32, // 5 = awaiting cmd, 4..1 = awaiting len bytes
    remaining_content: i32,
    remaining_padding: i32,
    current_command: u8,
    /// False once End/Direct seen (or the UUID gate failed).
    within_padding: bool,
    /// True after a Direct frame.
    direct: bool,
    /// True after a UUID-gate mismatch — never unpad again.
    plain_passthrough: bool,
}

impl Unpadder {
    const fn new(user_uuid: [u8; 16]) -> Self {
        Self {
            user_uuid,
            remaining_command: -1,
            remaining_content: -1,
            remaining_padding: -1,
            current_command: 0,
            within_padding: true,
            direct: false,
            plain_passthrough: false,
        }
    }

    /// Feed one decrypted chunk; returns unpadded content (possibly empty).
    /// On `direct`, the caller must switch the stream to raw reads and stop
    /// feeding this unpadder (spec §4.4).
    fn feed(&mut self, chunk: &[u8], out: &mut Vec<u8>) {
        if self.plain_passthrough {
            out.extend_from_slice(chunk);
            return;
        }
        let mut i = 0;
        // Initial state (`remaining == (-1,-1,-1)` — before the first frame
        // and again after an End/Direct block, matching Go). The first
        // >=21-byte chunk must start with the user UUID or vision is not in
        // play (permanent passthrough). Shorter chunks (the server's [0,0]
        // response header, its own outer-TLS record — spec §4.6) pass
        // through WITHOUT committing, so unpadding can still engage on the
        // next chunk (spec §5.4). (The brief's `uuid_gate_passed` flag is
        // subsumed by this state check.)
        //
        // Sanctioned divergence (latching): Go re-checks the UUID on every
        // subsequent chunk in the initial state and never latches; here the
        // first >=21-byte mismatch commits passthrough forever. Behaviorally
        // equivalent for the real wire (a peer that speaks vision carries
        // the UUID on its FIRST frame, so a mismatch means no vision) and a
        // cleaner trust boundary (spec §5.4).
        if self.remaining_command == -1
            && self.remaining_content == -1
            && self.remaining_padding == -1
        {
            if chunk.len() >= 21 && chunk[..16] == self.user_uuid {
                self.remaining_command = 5;
                i = 16; // consume the UUID
            } else if chunk.len() >= 21 {
                self.plain_passthrough = true;
                self.within_padding = false;
                out.extend_from_slice(chunk);
                return;
            } else {
                out.extend_from_slice(chunk);
                return;
            }
        }
        while i < chunk.len() {
            if self.remaining_command > 0 {
                let data = chunk[i];
                i += 1;
                match self.remaining_command {
                    5 => self.current_command = data,
                    4 => self.remaining_content = i32::from(data) << 8,
                    3 => self.remaining_content |= i32::from(data),
                    2 => self.remaining_padding = i32::from(data) << 8,
                    1 => self.remaining_padding |= i32::from(data),
                    _ => {}
                }
                self.remaining_command -= 1;
            } else if self.remaining_content > 0 {
                let take = usize::try_from(self.remaining_content)
                    .expect("content remaining is non-negative")
                    .min(chunk.len() - i);
                out.extend_from_slice(&chunk[i..i + take]);
                i += take;
                self.remaining_content -= i32::try_from(take).expect("take fits i32");
            } else {
                // Padding: skip.
                let take = usize::try_from(self.remaining_padding)
                    .expect("padding remaining is non-negative")
                    .min(chunk.len() - i);
                i += take;
                self.remaining_padding -= i32::try_from(take).expect("take fits i32");
            }
            if self.remaining_command <= 0
                && self.remaining_content <= 0
                && self.remaining_padding <= 0
            {
                // This block is done.
                if self.current_command == CMD_CONTINUE {
                    self.remaining_command = 5;
                } else {
                    self.remaining_command = -1;
                    self.remaining_content = -1;
                    self.remaining_padding = -1;
                    if self.current_command == CMD_END {
                        self.within_padding = false;
                    } else if self.current_command == CMD_DIRECT {
                        self.within_padding = false;
                        self.direct = true;
                    }
                    if i < chunk.len() {
                        // Go: "shouldn't happen" — defensive: keep the
                        // bytes (they are the start of the raw stream).
                        out.extend_from_slice(&chunk[i..]);
                    }
                    return;
                }
            }
        }
    }
}

/// `XtlsFilterTls` port (spec §4.5): sniffs inner TLS records to set
/// `is_tls` / `is_tls12_or_above` / `enable_xtls` within a per-connection
/// budget of 8 chunks.
struct TlsFilter {
    budget: i32,
    is_tls: bool,
    is_tls12_or_above: bool,
    remaining_server_hello: i32, // -1 = not awaiting a ServerHello body
    cipher: u16,
    enable_xtls: bool,
}

impl TlsFilter {
    const fn new() -> Self {
        Self {
            budget: FILTER_BUDGET,
            is_tls: false,
            is_tls12_or_above: false,
            remaining_server_hello: -1,
            cipher: 0,
            enable_xtls: false,
        }
    }

    fn feed(&mut self, chunk: &[u8]) {
        if self.budget <= 0 {
            return;
        }
        self.budget -= 1;
        if chunk.len() >= 6 {
            if chunk[0..3] == TLS_SERVER_HELLO_START && chunk[5] == TLS_HANDSHAKE_TYPE_SERVER_HELLO
            {
                self.is_tls12_or_above = true;
                self.is_tls = true;
                self.remaining_server_hello = (i32::from(chunk[3]) << 8 | i32::from(chunk[4])) + 5;
                if chunk.len() >= 79 && self.remaining_server_hello >= 79 {
                    let sid_len = usize::from(chunk[43]);
                    let cipher_hi = 43 + sid_len + 1;
                    // Defensive bound: a malformed ServerHello must not
                    // panic the relay (Go slices would).
                    if chunk.len() >= cipher_hi + 2 {
                        self.cipher = u16::from_be_bytes([chunk[cipher_hi], chunk[cipher_hi + 1]]);
                    }
                }
            } else if chunk[0..2] == TLS_CLIENT_HELLO_START
                && chunk[5] == TLS_HANDSHAKE_TYPE_CLIENT_HELLO
            {
                self.is_tls = true;
            }
        }
        if self.remaining_server_hello > 0 {
            let end = usize::try_from(self.remaining_server_hello)
                .expect("server hello remaining is non-negative")
                .min(chunk.len());
            self.remaining_server_hello -= i32::try_from(chunk.len()).expect("chunk fits i32");
            if chunk[..end]
                .windows(TLS13_SUPPORTED_VERSIONS.len())
                .any(|w| w == TLS13_SUPPORTED_VERSIONS)
            {
                // TLS 1.3 announced: enable the splice only for a known
                // TLS 1.3 suite (CCM_8 0x1305 excluded, xray dictionary).
                self.enable_xtls = matches!(self.cipher, 0x1301..=0x1304);
                self.budget = 0;
                return;
            }
            if self.remaining_server_hello <= 0 {
                // ServerHello exhausted without the 1.3 marker: TLS 1.2.
                self.budget = 0;
            }
        }
    }
}

/// Split a chunk >= 8171 at the last `0x17 0x03 0x03` boundary so every
/// piece keeps the frame cap (`ReshapeMultiBuffer`, spec §4.3). A boundary
/// outside [21, 8171] falls back to a 4096 split (xray `buf.Size / 2`).
/// The split repeats — our chunks can span several frames (e.g. a full
/// 16 KiB inner TLS record), so a single split would not guarantee the cap;
/// the reassembled byte stream is unchanged either way.
fn reshape(chunk: &[u8]) -> Vec<&[u8]> {
    let mut pieces = Vec::new();
    let mut rest = chunk;
    while rest.len() >= MAX_FRAME - FRAME_OVERHEAD {
        let index = rest
            .windows(TLS_APP_DATA_START.len())
            .rposition(|w| w == TLS_APP_DATA_START)
            .unwrap_or(usize::MAX);
        let index = if (21..=MAX_FRAME - FRAME_OVERHEAD).contains(&index) {
            index
        } else {
            MAX_FRAME / 2
        };
        if index < rest.len() {
            pieces.push(&rest[..index]);
            rest = &rest[index..];
        } else {
            pieces.push(rest);
            break;
        }
    }
    if !rest.is_empty() {
        pieces.push(rest);
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};

    const UUID: [u8; 16] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f,
    ];

    fn rng() -> ring::rand::SystemRandom {
        ring::rand::SystemRandom::new()
    }

    // Duplex has no record layer, so the direct-mode calls are no-ops; the
    // codec state machine is testable end to end (the test constructor also
    // disables the splice via `directable`).
    impl DirectMode for DuplexStream {
        fn set_write_direct(&mut self) {}
        fn set_read_direct(&mut self) {}
    }

    fn test_stream(inner: DuplexStream) -> VisionStream<DuplexStream> {
        VisionStream::new_test(inner, UUID, rng())
    }

    /// Build a fake inner TLS `ServerHello` record (handshake type 0x02).
    fn server_hello(cipher: u16, tls13: bool) -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(&[0x03, 0x03]); // legacy_version
        body.extend_from_slice(&[0xaa; 32]); // random
        body.push(32); // session_id_len
        body.extend_from_slice(&[0xbb; 32]); // session_id
        body.extend_from_slice(&cipher.to_be_bytes());
        body.push(0x00); // compression_method
        if tls13 {
            // extensions: supported_versions = TLS 1.3
            body.extend_from_slice(&[0x00, 0x2b, 0x00, 0x02, 0x03, 0x04]);
        }
        let hs_len = u32::try_from(body.len()).expect("fits") + 4;
        let mut rec = vec![0x16, 0x03, 0x03];
        let len = u16::try_from(body.len() + 4).expect("fits");
        rec.extend_from_slice(&len.to_be_bytes());
        rec.push(TLS_HANDSHAKE_TYPE_SERVER_HELLO);
        rec.extend_from_slice(&[
            ((hs_len >> 16) & 0xff) as u8,
            ((hs_len >> 8) & 0xff) as u8,
            (hs_len & 0xff) as u8,
        ]);
        rec.extend_from_slice(&body);
        rec
    }

    // ---- Step 1: frame codec + padding ----

    #[test]
    fn encode_frame_exact_bytes() {
        let mut expected = Vec::new();
        expected.extend_from_slice(&UUID);
        expected.extend_from_slice(&[CMD_CONTINUE, 0x00, 0x03, 0x00, 0x02]);
        expected.extend_from_slice(b"abc");
        expected.extend_from_slice(&[0x00, 0x00]);
        assert_eq!(encode_frame(Some(&UUID), CMD_CONTINUE, b"abc", 2), expected);
    }

    #[test]
    fn encode_frame_without_uuid() {
        assert_eq!(
            encode_frame(None, CMD_DIRECT, b"x", 0),
            [CMD_DIRECT, 0x00, 0x01, 0x00, 0x00, b'x']
        );
    }

    #[test]
    fn encode_frame_accepts_exact_cap() {
        // 8171 content + 21 overhead = exactly MAX_FRAME with pad 0; the
        // 21-byte overhead includes the UUID, so a uuid-less frame is 16
        // bytes smaller.
        let frame = encode_frame(
            None,
            CMD_CONTINUE,
            &vec![0u8; MAX_FRAME - FRAME_OVERHEAD],
            0,
        );
        assert_eq!(frame.len(), MAX_FRAME - 16);
    }

    #[test]
    #[should_panic(expected = "MAX_FRAME")]
    fn encode_frame_panics_when_content_overflows_cap() {
        let _ = encode_frame(
            None,
            CMD_CONTINUE,
            &vec![0u8; MAX_FRAME - FRAME_OVERHEAD + 1],
            0,
        );
    }

    #[test]
    #[should_panic(expected = "MAX_FRAME")]
    fn encode_frame_panics_when_padding_overflows_cap() {
        let _ = encode_frame(
            None,
            CMD_CONTINUE,
            &vec![0u8; MAX_FRAME - FRAME_OVERHEAD],
            1,
        );
    }

    #[test]
    fn padding_long_branch_bounds() {
        for _ in 0..64 {
            let pad = padding_len(0, true, &rng());
            assert!((900..=1399).contains(&pad), "pad {pad}");
        }
    }

    #[test]
    fn padding_long_branch_gated_on_content_under_900() {
        // Go: long padding only when `contentLen < 900 && longPadding`
        // (proxy.go:502); content >= 900 always takes rand(0..256).
        for _ in 0..64 {
            let pad = padding_len(899, true, &rng());
            assert!((1..=500).contains(&pad), "pad {pad}");
            assert!(padding_len(900, true, &rng()) < 256);
            assert!(padding_len(1000, true, &rng()) < 256);
            assert!(padding_len(8170, true, &rng()) <= 1);
        }
    }

    #[test]
    fn padding_plain_branch_bounds() {
        for _ in 0..64 {
            let pad = padding_len(123, false, &rng());
            assert!(pad < 256, "pad {pad}");
        }
    }

    #[test]
    fn padding_capped_at_frame_cap() {
        for _ in 0..64 {
            // 8170 content leaves at most 1 byte of padding (Go: rand(256)
            // capped at 8192 - 21 - 8170 = 1).
            assert!(padding_len(8170, true, &rng()) <= 1);
            assert!(padding_len(8170, false, &rng()) <= 1);
            // 8000 content leaves at most 171 bytes.
            assert!(padding_len(8000, false, &rng()) <= 171);
            assert!(padding_len(8000, true, &rng()) <= 171);
        }
    }

    #[test]
    fn rand_u32_respects_bound() {
        for _ in 0..64 {
            assert_eq!(rand_u32(&rng(), 1), 0);
            assert!(rand_u32(&rng(), 256) < 256);
        }
    }

    // ---- Step 2: unpad state machine ----

    #[test]
    fn unpad_single_complete_frame() {
        let mut unpad = Unpadder::new(UUID);
        let frame = encode_frame(Some(&UUID), CMD_CONTINUE, b"abc", 2);
        let mut out = Vec::new();
        unpad.feed(&frame, &mut out);
        assert_eq!(out, b"abc");
        assert!(unpad.within_padding);
        assert!(!unpad.direct);
    }

    #[test]
    fn unpad_continue_chain_across_chunk_boundaries() {
        let mut unpad = Unpadder::new(UUID);
        let mut chunk = Vec::new();
        chunk.extend_from_slice(&encode_frame(Some(&UUID), CMD_CONTINUE, b"abcd", 3));
        chunk.extend_from_slice(&encode_frame(None, CMD_CONTINUE, b"xy", 1));
        // Split mid-way through the first frame's content.
        let split = 16 + 5 + 2;
        let mut out = Vec::new();
        unpad.feed(&chunk[..split], &mut out);
        unpad.feed(&chunk[split..], &mut out);
        assert_eq!(out, b"abcdxy");
    }

    #[test]
    fn unpad_content_split_across_three_chunks() {
        // Header in chunk 1, content in chunk 2, padding in chunk 3. The
        // first chunk must be >= 21 bytes for the UUID gate to engage.
        let frame = encode_frame(Some(&UUID), CMD_CONTINUE, b"0123456789", 4);
        let mut unpad = Unpadder::new(UUID);
        let mut out = Vec::new();
        unpad.feed(&frame[..24], &mut out); // uuid + header + content head
        unpad.feed(&frame[24..30], &mut out); // content middle
        unpad.feed(&frame[30..], &mut out); // content tail + padding
        assert_eq!(out, b"0123456789");
    }

    #[test]
    fn unpad_uuid_gate_mismatch_commits_passthrough() {
        let mut unpad = Unpadder::new(UUID);
        let junk = b"no vision here at all, just plain bytes.......";
        assert!(junk.len() >= 21);
        let mut out = Vec::new();
        unpad.feed(junk, &mut out);
        assert_eq!(out, junk);
        assert!(unpad.plain_passthrough);
        assert!(!unpad.within_padding);
        // ... and forever after.
        let mut out2 = Vec::new();
        unpad.feed(b"more raw bytes", &mut out2);
        assert_eq!(out2, b"more raw bytes");
    }

    #[test]
    fn unpad_short_header_passes_through_without_committing() {
        // The server's [0,0] response header is its own outer-TLS record
        // before the first padded downlink frame (spec §4.6): it passes
        // through, and unpadding still engages on the next (uuid-carrying)
        // chunk (spec §5.4).
        let mut unpad = Unpadder::new(UUID);
        let mut out = Vec::new();
        unpad.feed(&[0x00, 0x00], &mut out);
        assert_eq!(out, [0x00, 0x00]);
        assert!(!unpad.plain_passthrough);
        assert!(unpad.within_padding);
        let frame = encode_frame(Some(&UUID), CMD_CONTINUE, b"hi", 0);
        let mut out2 = Vec::new();
        unpad.feed(&frame, &mut out2);
        assert_eq!(out2, b"hi");
    }

    #[test]
    fn unpad_end_frame_stops_padding() {
        let mut unpad = Unpadder::new(UUID);
        let frame = encode_frame(Some(&UUID), CMD_END, b"bye", 0);
        let mut out = Vec::new();
        unpad.feed(&frame, &mut out);
        assert_eq!(out, b"bye");
        assert!(!unpad.within_padding);
        assert!(!unpad.direct);
    }

    #[test]
    fn unpad_direct_frame_flags_direct() {
        let mut unpad = Unpadder::new(UUID);
        let frame = encode_frame(Some(&UUID), CMD_DIRECT, b"raw", 0);
        let mut out = Vec::new();
        unpad.feed(&frame, &mut out);
        assert_eq!(out, b"raw");
        assert!(!unpad.within_padding);
        assert!(unpad.direct);
    }

    #[test]
    fn unpad_skips_padding_bytes() {
        let mut unpad = Unpadder::new(UUID);
        let frame = encode_frame(Some(&UUID), CMD_CONTINUE, b"payload", 7);
        let mut out = Vec::new();
        unpad.feed(&frame, &mut out);
        assert_eq!(out, b"payload");
    }

    // ---- Step 3: TLS filter ----

    #[test]
    fn filter_client_hello_marks_tls() {
        let mut f = TlsFilter::new();
        f.feed(&[0x16, 0x03, 0x03, 0x00, 0x00, 0x01]);
        assert!(f.is_tls);
        assert!(!f.is_tls12_or_above);
        assert_eq!(f.budget, FILTER_BUDGET - 1);
    }

    #[test]
    fn filter_tls13_server_hello_enables_xtls() {
        let sh = server_hello(0x1301, true);
        let mut f = TlsFilter::new();
        f.feed(&sh);
        assert!(f.is_tls);
        assert!(f.is_tls12_or_above);
        assert!(f.enable_xtls);
        assert_eq!(f.budget, 0);
    }

    #[test]
    fn filter_ccm8_server_hello_disables_xtls() {
        let sh = server_hello(0x1305, true);
        let mut f = TlsFilter::new();
        f.feed(&sh);
        assert!(f.is_tls12_or_above);
        assert!(!f.enable_xtls);
        assert_eq!(f.budget, 0);
    }

    #[test]
    fn filter_tls12_server_hello_disables_xtls() {
        let sh = server_hello(0x1301, false);
        let mut f = TlsFilter::new();
        f.feed(&sh);
        assert!(f.is_tls12_or_above);
        assert!(!f.enable_xtls);
        assert_eq!(f.budget, 0);
    }

    #[test]
    fn filter_budget_exhausted_without_flags() {
        let mut f = TlsFilter::new();
        for _ in 0..FILTER_BUDGET {
            f.feed(b"not tls at all");
        }
        assert_eq!(f.budget, 0);
        assert!(!f.is_tls);
        assert!(!f.is_tls12_or_above);
        assert!(!f.enable_xtls);
        // Further feeds are ignored.
        f.feed(&server_hello(0x1301, true));
        assert!(!f.is_tls);
        assert_eq!(f.budget, 0);
    }

    // ---- Step 4: VisionStream ----

    #[test]
    fn camouflage_frame_is_long_padding_continue_with_uuid() {
        let camo = VisionStream::<DuplexStream>::camouflage_frame(&UUID, &rng());
        assert_eq!(&camo[..16], &UUID);
        assert_eq!(camo[16], CMD_CONTINUE);
        assert_eq!(&camo[17..19], &[0x00, 0x00]); // no content
        let plen = usize::from(u16::from_be_bytes([camo[19], camo[20]]));
        assert!((900..=1399).contains(&plen), "plen {plen}");
        assert_eq!(camo.len(), 21 + plen);
        assert!(camo[21..].iter().all(|&b| b == 0));
    }

    #[tokio::test]
    async fn stream_padded_frames_roundtrip() {
        let (mut server, client) = tokio::io::duplex(16384);
        // The caller writes the VLESS header + camouflage frame raw first.
        let mut raw_client = client;
        let camo = VisionStream::<DuplexStream>::camouflage_frame(&UUID, &rng());
        raw_client.write_all(&camo).await.unwrap();
        let mut vs = test_stream(raw_client);

        // Server: the camouflage frame (uuid gate + empty Continue block).
        let mut buf = [0u8; 4096];
        let n = server.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], &camo[..]);
        let mut unpad = Unpadder::new(UUID);
        let mut content = Vec::new();
        unpad.feed(&buf[..n], &mut content);
        assert!(content.is_empty());

        // Client writes app bytes -> one padded frame (no uuid).
        vs.write_all(b"hello").await.unwrap();
        let n = server.read(&mut buf).await.unwrap();
        assert_eq!(buf[0], CMD_CONTINUE);
        assert_eq!(&buf[1..3], &[0x00, 0x05]);
        let plen = usize::from(u16::from_be_bytes([buf[3], buf[4]]));
        assert_eq!(n, 5 + 5 + plen);
        let mut content = Vec::new();
        unpad.feed(&buf[..n], &mut content);
        assert_eq!(content, b"hello");

        // Server replies with a padded Continue frame carrying its UUID.
        let reply = encode_frame(Some(&UUID), CMD_CONTINUE, b"world", 0);
        server.write_all(&reply).await.unwrap();
        let mut got = [0u8; 16];
        let n = vs.read(&mut got).await.unwrap();
        assert_eq!(&got[..n], b"world");
    }

    #[tokio::test]
    async fn stream_direct_splices_both_directions() {
        let (mut server, client) = tokio::io::duplex(16384);
        let mut raw_client = client;
        let camo = VisionStream::<DuplexStream>::camouflage_frame(&UUID, &rng());
        raw_client.write_all(&camo).await.unwrap();
        let mut vs = test_stream(raw_client);

        let mut buf = [0u8; 8192];
        let n = server.read(&mut buf).await.unwrap();
        let mut unpad = Unpadder::new(UUID);
        let mut content = Vec::new();
        unpad.feed(&buf[..n], &mut content);
        assert!(content.is_empty());

        // 1. Uplink inner ClientHello -> padded Continue frame; the shared
        // filter marks is_tls.
        let client_hello = b"\x16\x03\x03\x00\x05\x01\x00\x00\x01\x00";
        vs.write_all(client_hello).await.unwrap();
        let n = server.read(&mut buf).await.unwrap();
        let mut content = Vec::new();
        unpad.feed(&buf[..n], &mut content);
        assert_eq!(content, client_hello);
        assert!(vs.filter.is_tls);

        // 2. Downlink TLS 1.3 ServerHello -> the filter enables XTLS.
        let sh = server_hello(0x1301, true);
        let frame = encode_frame(Some(&UUID), CMD_CONTINUE, &sh, 0);
        server.write_all(&frame).await.unwrap();
        let mut got = [0u8; 1024];
        let n = vs.read(&mut got).await.unwrap();
        assert_eq!(&got[..n], &sh[..]);
        assert!(vs.filter.enable_xtls);

        // 3. Uplink inner app data -> Direct frame (the last padded frame).
        let app_data = b"\x17\x03\x03\x00\x05hello";
        vs.write_all(app_data).await.unwrap();
        let n = server.read(&mut buf).await.unwrap();
        assert_eq!(buf[0], CMD_DIRECT);
        let mut content = Vec::new();
        unpad.feed(&buf[..n], &mut content);
        assert!(unpad.direct);
        assert_eq!(content, app_data);

        // 4. The client's next write goes out raw.
        vs.write_all(b"RAW-NOW").await.unwrap();
        let n = server.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"RAW-NOW");

        // 5. The server sends its Direct frame + raw tail; the client's
        // read side splices and delivers the raw stream (frame payload and
        // tail are both the start of the raw stream).
        let dframe = encode_frame(None, CMD_DIRECT, b"server-data", 0);
        server.write_all(&dframe).await.unwrap();
        server.write_all(b"RAW-DOWN").await.unwrap();
        let mut got = [0u8; 64];
        let n = vs.read(&mut got).await.unwrap();
        assert_eq!(&got[..n], b"server-dataRAW-DOWN");
    }

    #[tokio::test]
    async fn stream_buffered_direct_splice_on_pending() {
        // A 64-byte duplex forces `poll_write` Pending mid-frame, driving
        // the buffered-splice machinery: `write_buf` + `direct_boundary`,
        // the drain/splice ordering, the buffered raw tail, and the
        // poll_flush/poll_shutdown drain paths.
        let (server, client) = tokio::io::duplex(64);
        let mut raw_client = client;
        let camo = VisionStream::<DuplexStream>::camouflage_frame(&UUID, &rng());

        // Server task: drains everything the client sends (the small duplex
        // blocks every write until the peer reads).
        let server_task = tokio::spawn(async move {
            let mut server = server;
            let mut all = Vec::new();
            let mut buf = [0u8; 4096];
            loop {
                let n = server.read(&mut buf).await.unwrap();
                if n == 0 {
                    break;
                }
                all.extend_from_slice(&buf[..n]);
            }
            all
        });

        // Caller writes the camouflage frame through the raw stream first.
        raw_client.write_all(&camo).await.unwrap();
        let mut vs = test_stream(raw_client);

        // Warm the shared filter directly (the downlink ServerHello would
        // arrive over the duplex; feeding the filter is the same state).
        vs.filter
            .feed(&[0x16, 0x03, 0x03, 0x00, 0x05, 0x01, 0x00, 0x00, 0x01, 0x00]);
        vs.filter.feed(&server_hello(0x1301, true));
        assert!(vs.filter.enable_xtls);

        // A large app-data chunk: reshaped into [Direct frame][raw tail];
        // the frame is far larger than the duplex, so the write must be
        // buffered across many polls.
        let mut app = vec![0x42u8; 9000];
        app[0] = 0x17;
        app[1] = 0x03;
        app[2] = 0x03;
        vs.write_all(&app).await.unwrap();
        // Splice state after the full write: clean buffers, write side raw.
        assert!(vs.writer.direct);
        assert!(!vs.has_pending_write());
        assert_eq!(vs.write_buf.len(), 0);
        assert!(vs.direct_boundary.is_none());

        // A flush after the splice drains and forwards, then a raw write,
        // then shutdown (exercises poll_shutdown's drain path).
        vs.flush().await.unwrap();
        vs.write_all(b"RAW-AFTER").await.unwrap();
        vs.shutdown().await.unwrap();
        drop(vs);

        let all = server_task.await.unwrap();

        // Wire layout: [camouflage frame][Direct frame][raw tail][RAW-AFTER].
        let camo_plen = usize::from(u16::from_be_bytes([all[19], all[20]]));
        let camo_total = 21 + camo_plen;
        assert_eq!(&all[..camo_total], &camo[..]);
        let rest = &all[camo_total..];
        assert_eq!(rest[0], CMD_DIRECT, "Direct frame is the last padded write");
        let clen = usize::from(u16::from_be_bytes([rest[1], rest[2]]));
        let plen = usize::from(u16::from_be_bytes([rest[3], rest[4]]));
        assert_eq!(clen, 4096, "reshape fallback piece size");
        assert!(plen < 256);
        assert_eq!(&rest[5..5 + clen], &app[..clen]);
        let tail = &rest[5 + clen + plen..];
        assert_eq!(tail.len(), app.len() - clen + b"RAW-AFTER".len());
        assert_eq!(&tail[..app.len() - clen], &app[clen..], "raw tail intact");
        assert_eq!(&tail[app.len() - clen..], b"RAW-AFTER");
    }

    #[tokio::test]
    async fn stream_tls12_inner_ends_padding_keeps_tls() {
        let (mut server, client) = tokio::io::duplex(16384);
        let mut raw_client = client;
        let camo = VisionStream::<DuplexStream>::camouflage_frame(&UUID, &rng());
        raw_client.write_all(&camo).await.unwrap();
        let mut vs = test_stream(raw_client);

        let mut buf = [0u8; 8192];
        let n = server.read(&mut buf).await.unwrap();
        let mut unpad = Unpadder::new(UUID);
        let mut content = Vec::new();
        unpad.feed(&buf[..n], &mut content);

        // Inner ClientHello -> Continue frame (is_tls).
        let client_hello = b"\x16\x03\x03\x00\x05\x01\x00\x00\x01\x00";
        vs.write_all(client_hello).await.unwrap();
        let n = server.read(&mut buf).await.unwrap();
        let mut content = Vec::new();
        unpad.feed(&buf[..n], &mut content);
        assert_eq!(content, client_hello);

        // Downlink TLS 1.2 ServerHello -> enable_xtls stays false.
        let sh = server_hello(0x1301, false);
        let frame = encode_frame(Some(&UUID), CMD_CONTINUE, &sh, 0);
        server.write_all(&frame).await.unwrap();
        let mut got = [0u8; 1024];
        let n = vs.read(&mut got).await.unwrap();
        assert_eq!(&got[..n], &sh[..]);
        assert!(!vs.filter.enable_xtls);

        // Uplink app data -> End frame (not Direct), padding stops.
        let app_data = b"\x17\x03\x03\x00\x05hello";
        vs.write_all(app_data).await.unwrap();
        let n = server.read(&mut buf).await.unwrap();
        assert_eq!(buf[0], CMD_END);
        let mut content = Vec::new();
        unpad.feed(&buf[..n], &mut content);
        assert_eq!(content, app_data);
        assert!(!unpad.direct);

        // Subsequent writes go unpadded through the outer TLS.
        vs.write_all(b"PLAIN").await.unwrap();
        let n = server.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"PLAIN");
    }

    #[tokio::test]
    async fn stream_non_tls_early_finish_ends_padding() {
        let (mut server, client) = tokio::io::duplex(16384);
        let mut raw_client = client;
        let camo = VisionStream::<DuplexStream>::camouflage_frame(&UUID, &rng());
        raw_client.write_all(&camo).await.unwrap();
        let mut vs = test_stream(raw_client);

        let mut buf = [0u8; 8192];
        let n = server.read(&mut buf).await.unwrap();
        let mut unpad = Unpadder::new(UUID);
        let mut content = Vec::new();
        unpad.feed(&buf[..n], &mut content);

        // Budget 8: the first 6 writes (budget 7..2) are Continue frames,
        // the 7th (budget 1) ends padding (xray: "1 packet early").
        let chunks = [
            b"chunk-0", b"chunk-1", b"chunk-2", b"chunk-3", b"chunk-4", b"chunk-5",
        ];
        for c in chunks {
            vs.write_all(c).await.unwrap();
            let n = server.read(&mut buf).await.unwrap();
            assert_eq!(buf[0], CMD_CONTINUE);
            let mut content = Vec::new();
            unpad.feed(&buf[..n], &mut content);
            assert_eq!(content, c);
        }
        vs.write_all(b"last").await.unwrap();
        let n = server.read(&mut buf).await.unwrap();
        assert_eq!(buf[0], CMD_END);
        let mut content = Vec::new();
        unpad.feed(&buf[..n], &mut content);
        assert_eq!(content, b"last");

        // Everything after goes unpadded.
        vs.write_all(b"RAW").await.unwrap();
        let n = server.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"RAW");
    }

    #[tokio::test]
    async fn stream_reshapes_oversized_chunks() {
        let (mut server, client) = tokio::io::duplex(65536);
        let mut raw_client = client;
        let camo = VisionStream::<DuplexStream>::camouflage_frame(&UUID, &rng());
        raw_client.write_all(&camo).await.unwrap();
        let mut vs = test_stream(raw_client);

        let mut buf = [0u8; 16384];
        let n = server.read(&mut buf).await.unwrap();
        let mut unpad = Unpadder::new(UUID);
        let mut content = Vec::new();
        unpad.feed(&buf[..n], &mut content);

        // A chunk larger than one frame is reshaped into Continue frames
        // that reassemble byte-identically.
        let big = vec![0x41u8; 12000];
        vs.write_all(&big).await.unwrap();
        let mut got = Vec::new();
        loop {
            let n = server.read(&mut buf).await.unwrap();
            assert!(n > 0);
            let mut content = Vec::new();
            unpad.feed(&buf[..n], &mut content);
            got.extend_from_slice(&content);
            if got.len() >= big.len() {
                break;
            }
        }
        assert_eq!(got, big);
    }

    #[tokio::test]
    async fn stream_reader_commits_passthrough_on_uuid_mismatch() {
        let (mut server, client) = tokio::io::duplex(16384);
        let mut vs = test_stream(client);
        // No camouflage: the first downlink chunk is >=21 bytes without the
        // UUID -> permanent passthrough.
        let junk = b"this is a raw server that never heard of vision!!";
        server.write_all(junk).await.unwrap();
        let mut got = [0u8; 128];
        let n = vs.read(&mut got).await.unwrap();
        assert_eq!(&got[..n], &junk[..]);
        // Everything after passes through unchanged.
        server.write_all(b"more raw").await.unwrap();
        let n = vs.read(&mut got).await.unwrap();
        assert_eq!(&got[..n], b"more raw");
    }
}
