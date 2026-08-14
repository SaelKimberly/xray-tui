//! VLESS v1.mux.cool frame codec (SP2 of the parity program).
//!
//! Wire format (xray-core `common/mux/frame.go` + `writer.go`):
//!
//! ```text
//! [2B meta_len BE][2B session_id BE][1B status][1B option]
//! [New: 1B network + port-first addr][2B data_len][payload]
//! ```
//!
//! The 2-byte `meta_len` covers ONLY the metadata (session id, status,
//! option, and the New-frame target); `writeMetaWithFrame` appends
//! `[2B data_len][data]` after it. Status: `New` `0x01` / `Keep` `0x02` /
//! `End` `0x03` / `KeepAlive` `0x04`. Option: `Data` `0x01` / `Error`
//! `0x02`. Addresses are port-first (`[2B port][atyp][addr]`, atyp IPv4
//! `0x01` / Domain `0x02` / IPv6 `0x03`) exactly like the VLESS request
//! header. `meta_len` is capped at 512 on the server side (mirrored on
//! read); application data is chunked at 8 KiB by the client. The mux
//! connection itself targets the fixed `v1.mux.cool:9527` destination.
//!
//! Scope: TCP targets only — `MuxTarget` has no UDP variant (UDP mux /
//! XUDP is a later plan), and a New frame with a non-TCP network byte is
//! a protocol error. The server's response writer starts at Keep
//! (`NewResponseWriter` sets `followup: true`), so `target` is only ever
//! `Some` on the New frames the client writes.
//!
//! The multiplexer ([`MuxClient`] + [`SessionStream`], spec §5.2) sits on
//! top of the codec: [`MuxClient::new`] splits the tunnel and spawns a
//! demux task (reads frames, routes payloads/End/Error to per-session
//! channels), a writer task (serializes frames onto the tunnel), and a
//! keepalive task (tunnel-level `KeepAlive` every 10s, spec §8 deviation
//! 2). [`MuxClient::open_session`] allocates a session id, registers the
//! channel, and sends an eager `New` frame (deviation 1); the returned
//! [`SessionStream`] reads its channel and writes 8 KiB-chunked `Keep`
//! frames, ending with a meta-only `End` on close/drop.
//!
//! Items are `pub` inside the `pub(crate)` module (effective
//! `pub(crate)`), mirroring `udp.rs`.

use std::collections::HashMap;
use std::future::Future;
use std::io;
use std::marker::PhantomData;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::task::{Context as TaskCx, Poll};
use std::time::Duration;

use bytes::Bytes;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::sync::mpsc::OwnedPermit;
use tokio::sync::mpsc::error::{SendError, TrySendError};
use tokio::sync::{mpsc, watch};

use crate::addr::{ADDR_TYPE_DOMAIN, ADDR_TYPE_IPV4, ADDR_TYPE_IPV6, Host, decode_addr};

/// The mux connection's fixed destination inside the VLESS header
/// (`v1.mux.cool:9527` — the VLESS Mux command target).
///
/// Not consumed by the [`MuxClient`] itself (it only sees the byte stream
/// of the already-open tunnel); the `connect_mux` path of the next SP2
/// task writes the `command=0x03` header with this destination.
#[allow(dead_code)] // wired by the SP2 connect_mux task
pub const MUX_DEST: &str = "v1.mux.cool";
#[allow(dead_code)] // wired by the SP2 connect_mux task
pub const MUX_PORT: u16 = 9527;

pub const STATUS_NEW: u8 = 0x01;
pub const STATUS_KEEP: u8 = 0x02;
pub const STATUS_END: u8 = 0x03;
pub const STATUS_KEEPALIVE: u8 = 0x04;

pub const OPT_DATA: u8 = 0x01;
pub const OPT_ERROR: u8 = 0x02;

/// Application data chunk size written per Keep frame (xray
/// `SplitSize(mb, 8*1024)` for stream transfers).
#[allow(dead_code)] // used by the (test-only until SP2 Task 3) SessionStream
pub const CHUNK_SIZE: usize = 8 * 1024;

/// Server-side `meta_len` rejection cap (`FrameMetadata.Unmarshal`), used
/// to bound the metadata read on the client too.
pub const MAX_META: usize = 512;

/// Network byte for New frames. Always TCP in this codec — `MuxTarget` is
/// TCP-only (see the module doc).
const NETWORK_TCP: u8 = 0x01;

/// A mux session target (the New frame's `network + port-first addr`).
/// TCP-only: UDP mux is out of scope (a later plan).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MuxTarget {
    /// IP target.
    Tcp(SocketAddr),
    /// Domain target: name + port.
    TcpDomain(String, u16),
}

/// One decoded v1.mux.cool frame.
#[derive(Debug)]
pub struct Frame {
    pub session_id: u16,
    pub status: u8,
    pub option: u8,
    /// `Some` on New frames (the client's first frame per session); the
    /// server's response writer starts at Keep, so `None` otherwise.
    pub target: Option<MuxTarget>,
    /// Empty unless the Data option is set.
    pub payload: Bytes,
}

/// Reads one frame. `Ok(None)` on a clean EOF at a frame boundary; a
/// truncated length/metadata/payload at EOF is `UnexpectedEof`, a
/// `meta_len` over [`MAX_META`] or an unparseable/unsupported New target
/// is `InvalidData`.
pub async fn read_frame<R: AsyncRead + Unpin>(r: &mut R) -> io::Result<Option<Frame>> {
    let mut len_buf = [0u8; 2];
    // Byte-by-byte so a clean EOF (0 bytes) is distinguishable from a
    // truncated length (1 byte then EOF).
    let mut filled = 0;
    while filled < len_buf.len() {
        match r.read(&mut len_buf[filled..]).await {
            Ok(0) if filled == 0 => return Ok(None),
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "vless mux frame truncated (partial meta length)",
                ));
            }
            Ok(n) => filled += n,
            Err(e) => return Err(e),
        }
    }
    let meta_len = usize::from(u16::from_be_bytes(len_buf));
    if meta_len > MAX_META {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("vless mux invalid metalen {meta_len}"),
        ));
    }
    // The allocation is capped at meta_len (≤ MAX_META) by the check above.
    let mut meta = vec![0u8; meta_len];
    r.read_exact(&mut meta).await?;

    let mut frame = parse_meta(&meta)?;
    if frame.option & OPT_DATA != 0 {
        let mut dlen = [0u8; 2];
        r.read_exact(&mut dlen).await?;
        // Cap the payload allocation to data_len (defensive — the
        // server's frames are ≤ 8 KiB, so 64 KiB is the absolute worst
        // case; a truncated payload then fails the read_exact).
        let n = usize::from(u16::from_be_bytes(dlen));
        let mut payload = vec![0u8; n];
        r.read_exact(&mut payload).await?;
        frame.payload = Bytes::from(payload);
    }
    Ok(Some(frame))
}

/// Parses the metadata body: `[sid 2B][status 1B][option 1B]`, plus
/// `[network 1B][port-first addr]` for New frames.
fn parse_meta(meta: &[u8]) -> io::Result<Frame> {
    if meta.len() < 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "vless mux metadata too short",
        ));
    }
    let session_id = u16::from_be_bytes([meta[0], meta[1]]);
    let status = meta[2];
    let option = meta[3];
    let mut rest = &meta[4..];
    let mut target = None;
    if status == STATUS_NEW {
        let (network, tail) = rest.split_first().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "vless mux new frame missing network byte",
            )
        })?;
        if *network != NETWORK_TCP {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("vless mux unsupported network type {network}"),
            ));
        }
        // port-first: [2B port][atyp][addr] — same wire layout as the
        // VLESS request header (addr.rs `decode_addr`).
        let (addr, tail) = decode_addr(tail).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "vless mux truncated target address",
            )
        })?;
        target = Some(match addr.host {
            Host::Ip(ip) => MuxTarget::Tcp(SocketAddr::new(ip, addr.port)),
            Host::Domain(domain) => MuxTarget::TcpDomain(domain, addr.port),
        });
        rest = tail;
    }
    // The metadata is exactly what we parsed; anything left over is a
    // frame format we don't understand (e.g. source/local metadata).
    if !rest.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "vless mux trailing metadata bytes",
        ));
    }
    Ok(Frame {
        session_id,
        status,
        option,
        target,
        payload: Bytes::new(),
    })
}

/// Writes one frame in a single `write_all`: `[2B meta_len][metadata]`
/// plus `[2B data_len][payload]` when the Data option is set. The
/// metadata (sid, status, option, New target) is what `meta_len` covers;
/// `writeMetaWithFrame` appends the data after it.
pub async fn write_frame<W: AsyncWrite + Unpin>(w: &mut W, f: &Frame) -> io::Result<()> {
    // sid(2) + status(1) + opt(1) + [net(1) + port(2) + atyp(1) + IPv6(16)]
    let mut meta = Vec::with_capacity(2 + 1 + 1 + 1 + 2 + 1 + 16);
    meta.extend_from_slice(&f.session_id.to_be_bytes());
    meta.push(f.status);
    meta.push(f.option);
    if f.status == STATUS_NEW {
        meta.push(NETWORK_TCP);
        let target = f.target.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "vless mux new frame requires a target",
            )
        })?;
        meta.extend_from_slice(&encode_new_target(target));
    }
    let meta_len = u16::try_from(meta.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "vless mux metadata exceeds the 2-byte meta length",
        )
    })?;
    if usize::from(meta_len) > MAX_META {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("vless mux metadata exceeds the {MAX_META}-byte limit"),
        ));
    }

    let mut out = Vec::with_capacity(2 + meta.len() + 2 + f.payload.len());
    out.extend_from_slice(&meta_len.to_be_bytes());
    out.extend_from_slice(&meta);
    if f.option & OPT_DATA != 0 {
        let data_len = u16::try_from(f.payload.len()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "vless mux frame payload exceeds the 2-byte data length (65535)",
            )
        })?;
        out.extend_from_slice(&data_len.to_be_bytes());
        out.extend_from_slice(&f.payload);
    } else if !f.payload.is_empty() {
        // Without the Data option the reader never consumes a data_len —
        // the bytes would be misparsed as the next frame's meta_len.
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "vless mux frame with payload requires the data option",
        ));
    }
    w.write_all(&out).await
}

/// Encodes a New-frame target as the port-first address bytes:
/// `[2B port][atyp][addr]` (IPv4 4 / Domain 1+len / IPv6 16). The network
/// byte (`NETWORK_TCP`) is written separately by [`write_frame`].
#[must_use]
pub fn encode_new_target(t: &MuxTarget) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + 1 + 16);
    match t {
        MuxTarget::Tcp(sa) => {
            out.extend_from_slice(&sa.port().to_be_bytes());
            match sa.ip() {
                IpAddr::V4(v4) => {
                    out.push(ADDR_TYPE_IPV4);
                    out.extend_from_slice(&v4.octets());
                }
                IpAddr::V6(v6) => {
                    out.push(ADDR_TYPE_IPV6);
                    out.extend_from_slice(&v6.octets());
                }
            }
        }
        MuxTarget::TcpDomain(domain, port) => {
            out.extend_from_slice(&port.to_be_bytes());
            out.push(ADDR_TYPE_DOMAIN);
            // The wire address caps domain length at 255; the VLESS header
            // encode rejects longer domains before a mux target is ever
            // built (addr.rs `encode_addr` → Config error).
            let len =
                u8::try_from(domain.len()).expect("mux target domain fits in one length byte");
            out.push(len);
            out.extend_from_slice(domain.as_bytes());
        }
    }
    out
}

// ---------------------------------------------------------------------
// MuxClient + SessionStream — the multiplexing core (spec §5.2)
// ---------------------------------------------------------------------

/// Tunnel-level `KeepAlive` interval (spec §8 deviation 2): xray sends
/// `KeepAlive` per idle session; a single tunnel-level `KeepAlive` keeps
/// the tunnel alive and is answered/ignored by both cores.
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(10);

/// Writer-channel capacity in frames. Frames are ≤ 8 KiB + header, so this
/// bounds the queued-but-unwritten bytes (~256 KiB) between session writes
/// and the tunnel (the app-side backpressure point).
const WRITER_CHANNEL_CAPACITY: usize = 32;

/// Per-session event-channel capacity. Mirrors xray's 64 KiB pipe bound
/// (`pipe.WithSizeLimit(64 * 1024)`): at most 8 × 8 KiB payloads buffer
/// before the demux backpressures (head-of-line, like xray's `buf.Copy`).
const SESSION_CHANNEL_CAPACITY: usize = 8;

/// One event the demux task delivers to a session.
#[allow(dead_code)] // wired by the SP2 connect_mux task
enum SessionEvent {
    /// Application payload bytes (a `Keep`+`Data` frame's payload).
    Data(Bytes),
    /// The peer closed the session (`End` frame) — the reader sees EOF.
    End,
    /// The peer reported an error for the session (`Error` option) — the
    /// reader sees the `io::Error`.
    Error(io::Error),
}

/// Locks the session map, recovering from poisoning (a panicked holder
/// leaves the map intact; the lock is only ever held across short,
/// non-awaiting sections).
fn lock_map(
    map: &Mutex<HashMap<u16, mpsc::Sender<SessionEvent>>>,
) -> MutexGuard<'_, HashMap<u16, mpsc::Sender<SessionEvent>>> {
    map.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Multiplexes app TCP streams over one v1.mux.cool tunnel (spec §5.2).
///
/// `new` splits the tunnel and spawns three tasks: a **demux** task reads
/// frames off the tunnel and routes payloads/`End`/`Error` to per-session
/// channels (ending every session on tunnel EOF/error); a **writer** task
/// drains the frame channel into the tunnel in order (one writer = no
/// interleaving); a **keepalive** task sends a tunnel-level `KeepAlive`
/// every [`KEEPALIVE_INTERVAL`] while the handle is alive.
///
/// Session ids are allocated from 1. The client itself does not own the
/// tunnel (it lives inside the spawned tasks); dropping the handle stops
/// the keepalive, and the tunnel tears down once the last session drops.
///
/// [`MuxClient`] and [`SessionStream`] are consumed by the `connect_mux`
/// path of the next SP2 task — until then nothing outside this module
/// (or the tests) uses them, so the file-level allow removed in this task
/// is scoped to exactly these two types.
#[allow(dead_code)] // wired by the SP2 connect_mux task
pub struct MuxClient<S> {
    next_id: AtomicU16,
    sessions: Arc<Mutex<HashMap<u16, mpsc::Sender<SessionEvent>>>>,
    write_tx: mpsc::Sender<Frame>,
    keepalive_tx: mpsc::Sender<()>,
    /// Set when the demux task exits — `open_session` fails fast after a
    /// dead tunnel instead of waiting on a writer that is gone.
    dead: Arc<AtomicBool>,
    tunnel: PhantomData<fn() -> S>,
}

#[allow(dead_code)] // wired by the SP2 connect_mux task
impl<S: AsyncRead + AsyncWrite + Unpin + Send + 'static> MuxClient<S> {
    /// Splits `tunnel` into read (demux) + write (writer) halves and
    /// spawns the demux/writer/keepalive tasks.
    pub(crate) fn new(tunnel: S) -> Self {
        let (read_half, write_half) = tokio::io::split(tunnel);
        let (write_tx, write_rx) = mpsc::channel(WRITER_CHANNEL_CAPACITY);
        let (keepalive_tx, keepalive_rx) = mpsc::channel(1);
        let (writer_done_tx, writer_done_rx) = watch::channel(());
        let sessions = Arc::new(Mutex::new(HashMap::new()));
        let dead = Arc::new(AtomicBool::new(false));

        tokio::spawn(demux_loop(
            read_half,
            Arc::clone(&sessions),
            Arc::clone(&dead),
            writer_done_rx,
        ));
        tokio::spawn(writer_loop(write_half, write_rx, writer_done_tx));
        tokio::spawn(keepalive_loop(write_tx.clone(), keepalive_rx));

        Self {
            next_id: AtomicU16::new(1),
            sessions,
            write_tx,
            keepalive_tx,
            dead,
            tunnel: PhantomData,
        }
    }

    /// Opens a new TCP session to `target` and returns its byte stream.
    ///
    /// The `New` frame is sent eagerly (spec §8 deviation 1) so the server
    /// allocates the session before any app data; the first app write then
    /// goes out as `Keep`. Fails when the tunnel is dead.
    pub(crate) async fn open_session(&self, target: MuxTarget) -> io::Result<SessionStream> {
        if self.dead.load(Ordering::Acquire) {
            return Err(tunnel_closed());
        }
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        // Reject id 0 — the KeepAlive frames' session id. The counter
        // wraps around only after 65k live sessions (impossible in
        // practice), and a wrap to 0 is caught here.
        if id == 0 {
            return Err(io::Error::other("vless mux session id exhausted"));
        }
        let (tx, rx) = mpsc::channel(SESSION_CHANNEL_CAPACITY);
        lock_map(&self.sessions).insert(id, tx);
        let frame = Frame {
            session_id: id,
            status: STATUS_NEW,
            option: 0,
            target: Some(target),
            payload: Bytes::new(),
        };
        if self.write_tx.send(frame).await.is_err() {
            // The writer task is gone — the tunnel is dead.
            lock_map(&self.sessions).remove(&id);
            self.dead.store(true, Ordering::Release);
            return Err(tunnel_closed());
        }
        Ok(SessionStream::new(
            id,
            rx,
            self.write_tx.clone(),
            Arc::clone(&self.sessions),
        ))
    }
}

/// `BrokenPipe`-style error for a dead tunnel, shared by the session write
/// path and `open_session`.
// `io::Error::new` is not const (clippy's const-checker thinks it is).
#[allow(clippy::missing_const_for_fn)]
fn tunnel_closed() -> io::Error {
    io::Error::new(io::ErrorKind::BrokenPipe, "vless mux tunnel is closed")
}

/// Reads frames off the tunnel and routes them to per-session channels
/// (spec §4.4). Exits on tunnel EOF/error, an unknown frame status (a
/// protocol violation — mirroring xray's fatal "unknown status"), or when
/// the writer task has died. On exit it marks the client dead and drops
/// every session channel's sender (the receivers see EOF — spec §6 "End
/// all sessions").
async fn demux_loop<R: AsyncRead + Unpin>(
    mut read_half: R,
    sessions: Arc<Mutex<HashMap<u16, mpsc::Sender<SessionEvent>>>>,
    dead: Arc<AtomicBool>,
    mut writer_done: watch::Receiver<()>,
) {
    loop {
        tokio::select! {
            // Resolves (Err) when the writer task exits and drops its
            // sender — the write side of the tunnel is gone.
            _ = writer_done.changed() => break,
            res = read_frame(&mut read_half) => {
                match res {
                    Ok(Some(frame)) => {
                        if !route(frame, &sessions).await {
                            break;
                        }
                    }
                    Ok(None) => break, // tunnel EOF
                    Err(e) => {
                        tracing::debug!(error = %e, "vless mux: tunnel read failed");
                        break;
                    }
                }
            }
        }
    }
    dead.store(true, Ordering::Release);
    lock_map(&sessions).clear();
}

/// Routes one inbound frame to its session. Returns `false` on a fatal
/// protocol error (an unknown status — tear the tunnel down).
async fn route(frame: Frame, sessions: &Mutex<HashMap<u16, mpsc::Sender<SessionEvent>>>) -> bool {
    let event = match frame.status {
        // KeepAlive: consumed (the codec drained any payload). New: a
        // server-initiated New — nothing to route. Both are no-ops.
        STATUS_KEEPALIVE | STATUS_NEW => return true,
        STATUS_END => {
            if frame.option & OPT_ERROR != 0 {
                SessionEvent::Error(session_error())
            } else {
                SessionEvent::End
            }
        }
        STATUS_KEEP => {
            if frame.option & OPT_ERROR != 0 {
                SessionEvent::Error(session_error())
            } else if frame.option & OPT_DATA != 0 {
                SessionEvent::Data(frame.payload)
            } else {
                return true; // meta-only Keep — nothing to deliver
            }
        }
        status => {
            tracing::warn!(status, "vless mux: unknown frame status; closing tunnel");
            return false;
        }
    };
    // Clone the sender out so the await never holds the map lock (a full
    // session channel backpressures the demux, not the map).
    let terminal = matches!(event, SessionEvent::End | SessionEvent::Error(_));
    let tx = lock_map(sessions).get(&frame.session_id).cloned();
    match tx {
        Some(tx) => {
            let delivered = tx.send(event).await.is_ok();
            // A closed session (End/Error delivered, or the receiver
            // dropped) is no longer routable — remove it so stray frames
            // for it can't accumulate in the channel.
            if !delivered || terminal {
                lock_map(sessions).remove(&frame.session_id);
            }
        }
        None => {
            tracing::debug!(
                session_id = frame.session_id,
                "vless mux: frame for unknown session"
            );
        }
    }
    true
}

/// The error surfaced on a session whose peer sent the `Error` option.
#[allow(clippy::missing_const_for_fn)] // `io::Error::new` is not const
fn session_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::ConnectionReset,
        "vless mux: session terminated by peer",
    )
}

/// Drains frames from the channel and writes them to the tunnel in order.
/// Exits on a write error or when every sender is gone; dropping the
/// `writer_done` sender tells the demux the write side is dead.
async fn writer_loop<W: AsyncWrite + Unpin>(
    mut write_half: W,
    mut frames: mpsc::Receiver<Frame>,
    writer_done: watch::Sender<()>,
) {
    while let Some(frame) = frames.recv().await {
        if let Err(e) = write_frame(&mut write_half, &frame).await {
            tracing::debug!(error = %e, "vless mux: tunnel write failed");
            break;
        }
    }
    drop(writer_done);
}

/// Sends a tunnel-level `KeepAlive` frame (session id 0) every
/// [`KEEPALIVE_INTERVAL`] while the [`MuxClient`] handle is alive and the
/// writer task is accepting frames (spec §8 deviation 2).
async fn keepalive_loop(write_tx: mpsc::Sender<Frame>, mut stop: mpsc::Receiver<()>) {
    let mut interval = tokio::time::interval_at(
        tokio::time::Instant::now() + KEEPALIVE_INTERVAL,
        KEEPALIVE_INTERVAL,
    );
    loop {
        tokio::select! {
            _ = stop.recv() => break, // the MuxClient handle is gone
            _ = interval.tick() => {
                let frame = Frame {
                    session_id: 0,
                    status: STATUS_KEEPALIVE,
                    option: 0,
                    target: None,
                    payload: Bytes::new(),
                };
                if write_tx.send(frame).await.is_err() {
                    break; // the writer task is gone — tunnel dead
                }
            }
        }
    }
}

/// The app-side byte stream for one mux session (spec §4.4/§5.2).
///
/// Reads pull `SessionEvent`s off the session channel (`Data` → bytes,
/// `End` → EOF, `Error` → the `io::Error`); writes chunk the app bytes at
/// [`CHUNK_SIZE`] into `Keep` frames queued on the tunnel's writer
/// channel. The `AsyncWrite` contract holds strictly: bytes are only
/// counted as accepted once their frame is queued, and a full writer
/// channel parks the writer (`Pending`) rather than ever reporting `Ok(0)`
/// for a non-empty buffer.
#[allow(dead_code)] // wired by the SP2 connect_mux task
pub struct SessionStream {
    id: u16,
    rx: mpsc::Receiver<SessionEvent>,
    write_tx: mpsc::Sender<Frame>,
    sessions: Arc<Mutex<HashMap<u16, mpsc::Sender<SessionEvent>>>>,
    /// A `Data` payload split across reads (cancellation-safe buffering).
    read_pending: Option<Bytes>,
    /// Terminal read state after `End`/`Error` — sticky, so every
    /// subsequent read keeps returning the same EOF/error.
    read_end: Option<ReadEnd>,
    /// The frame currently being handed to the writer channel (a `Data`
    /// chunk or the shutdown `End`), not yet counted as accepted.
    write_pending: Option<WriteItem>,
    write_state: WriteState,
    /// The `End` frame was handed over/queued — `Drop` must not send
    /// another.
    end_queued: bool,
}

/// The session's terminal read outcome (peer `End` / peer `Error`).
#[allow(dead_code)] // wired by the SP2 connect_mux task
enum ReadEnd {
    Eof,
    Err(io::Error),
}

/// The unit of in-flight write work.
#[allow(dead_code)] // wired by the SP2 connect_mux task
enum WriteItem {
    /// A `Keep`-frame payload chunk; its length is the accepted count.
    Data(Bytes),
    /// The meta-only `End` frame (shutdown); accepted count 0.
    End,
}

/// A writer-channel capacity reservation (owns the sender, so the future
/// is `'static` and storable across polls).
type WriteFuture = Pin<Box<dyn Future<Output = Result<OwnedPermit<Frame>, SendError<()>>> + Send>>;

#[allow(dead_code)] // wired by the SP2 connect_mux task
enum WriteState {
    Idle,
    Waiting(WriteFuture),
}

#[allow(dead_code)] // wired by the SP2 connect_mux task
impl SessionStream {
    #[must_use]
    const fn new(
        id: u16,
        rx: mpsc::Receiver<SessionEvent>,
        write_tx: mpsc::Sender<Frame>,
        sessions: Arc<Mutex<HashMap<u16, mpsc::Sender<SessionEvent>>>>,
    ) -> Self {
        Self {
            id,
            rx,
            write_tx,
            sessions,
            read_pending: None,
            read_end: None,
            write_pending: None,
            write_state: WriteState::Idle,
            end_queued: false,
        }
    }

    /// Finishes an in-flight writer-channel reservation, returning the
    /// accepted byte count of the item that was sent (0 for the `End`
    /// frame). `Ready(Ok(0))` when idle.
    fn poll_pending_write(&mut self, cx: &mut TaskCx<'_>) -> Poll<io::Result<usize>> {
        match &mut self.write_state {
            WriteState::Idle => Poll::Ready(Ok(0)),
            WriteState::Waiting(fut) => match fut.as_mut().poll(cx) {
                Poll::Ready(Ok(permit)) => {
                    self.write_state = WriteState::Idle;
                    let item = self
                        .write_pending
                        .take()
                        .expect("a reservation in flight always has an item");
                    let accepted = match &item {
                        WriteItem::Data(bytes) => bytes.len(),
                        WriteItem::End => 0,
                    };
                    let frame = match item {
                        WriteItem::Data(payload) => Frame {
                            session_id: self.id,
                            status: STATUS_KEEP,
                            option: OPT_DATA,
                            target: None,
                            payload,
                        },
                        WriteItem::End => Frame {
                            session_id: self.id,
                            status: STATUS_END,
                            option: 0,
                            target: None,
                            payload: Bytes::new(),
                        },
                    };
                    permit.send(frame);
                    Poll::Ready(Ok(accepted))
                }
                Poll::Ready(Err(_)) => Poll::Ready(Err(tunnel_closed())),
                Poll::Pending => Poll::Pending,
            },
        }
    }

    /// Removes this session from the demux's routing map. Idempotent.
    fn unregister(&self) {
        lock_map(&self.sessions).remove(&self.id);
    }
}

impl AsyncRead for SessionStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskCx<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // A terminal session (peer End/Error) stays terminal: later reads
        // keep returning the same EOF/error.
        if let Some(end) = &self.read_end {
            return Poll::Ready(match end {
                ReadEnd::Eof => Ok(()),
                ReadEnd::Err(e) => Err(io::Error::new(e.kind(), e.to_string())),
            });
        }
        loop {
            // Deliver any leftover payload before pulling the channel, so a
            // read that filled the caller's buffer is never lost.
            if let Some(payload) = self.read_pending.take() {
                let n = std::cmp::min(payload.len(), buf.remaining());
                buf.put_slice(&payload[..n]);
                if n < payload.len() {
                    self.read_pending = Some(payload.slice(n..));
                }
                return Poll::Ready(Ok(()));
            }
            match self.rx.poll_recv(cx) {
                Poll::Ready(Some(SessionEvent::Data(payload))) => {
                    if payload.is_empty() {
                        continue; // empty frames carry nothing
                    }
                    let n = std::cmp::min(payload.len(), buf.remaining());
                    buf.put_slice(&payload[..n]);
                    if n < payload.len() {
                        self.read_pending = Some(payload.slice(n..));
                    }
                    return Poll::Ready(Ok(()));
                }
                // End frame or a closed channel → EOF (sticky).
                Poll::Ready(Some(SessionEvent::End) | None) => {
                    self.read_end = Some(ReadEnd::Eof);
                    return Poll::Ready(Ok(()));
                }
                // Error → the io::Error (sticky).
                Poll::Ready(Some(SessionEvent::Error(e))) => {
                    self.read_end = Some(ReadEnd::Err(io::Error::new(e.kind(), e.to_string())));
                    return Poll::Ready(Err(e));
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl AsyncWrite for SessionStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut TaskCx<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let mut accepted = 0;
        loop {
            // Finish the in-flight reservation first; its item belongs to
            // the prefix of `buf` (the caller re-polls from where we last
            // returned), so its length counts toward `accepted`.
            match self.poll_pending_write(cx) {
                Poll::Ready(Ok(n)) => accepted += n,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => {
                    return if accepted > 0 {
                        Poll::Ready(Ok(accepted))
                    } else {
                        Poll::Pending
                    };
                }
            }
            if accepted >= buf.len() {
                return Poll::Ready(Ok(accepted));
            }
            let n = std::cmp::min(CHUNK_SIZE, buf.len() - accepted);
            let frame = Frame {
                session_id: self.id,
                status: STATUS_KEEP,
                option: OPT_DATA,
                target: None,
                payload: Bytes::copy_from_slice(&buf[accepted..accepted + n]),
            };
            match self.write_tx.try_send(frame) {
                Ok(()) => accepted += n,
                Err(TrySendError::Full(frame)) => {
                    // Park until the writer task frees capacity. The chunk
                    // is kept (write_pending + reservation) so nothing is
                    // lost; the caller's buffer is not consumed for it.
                    self.write_pending = Some(WriteItem::Data(frame.payload));
                    self.write_state =
                        WriteState::Waiting(Box::pin(self.write_tx.clone().reserve_owned()));
                    // loop: poll the reservation on the next iteration.
                }
                Err(TrySendError::Closed(_)) => return Poll::Ready(Err(tunnel_closed())),
            }
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<io::Result<()>> {
        // Frames are drained by the writer task in order; flush only needs
        // the current write handed to the channel (a no-op when idle).
        match self.poll_pending_write(cx) {
            Poll::Ready(Ok(_)) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<io::Result<()>> {
        // Queue the meta-only End frame (spec §4.3) behind any in-flight
        // data, then unregister. Drop (below) covers the fire-and-forget
        // path when the app never calls shutdown.
        if !self.end_queued {
            self.end_queued = true;
            let end = Frame {
                session_id: self.id,
                status: STATUS_END,
                option: 0,
                target: None,
                payload: Bytes::new(),
            };
            match self.write_tx.try_send(end) {
                // Handed over, or the tunnel is dead — nothing to send in
                // either case.
                Ok(()) | Err(TrySendError::Closed(_)) => {}
                Err(TrySendError::Full(_)) => {
                    self.write_pending = Some(WriteItem::End);
                    self.write_state =
                        WriteState::Waiting(Box::pin(self.write_tx.clone().reserve_owned()));
                }
            }
            self.unregister();
        }
        // Wait for the End (when queued) to reach the writer channel.
        match self.poll_pending_write(cx) {
            Poll::Ready(Ok(_)) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for SessionStream {
    fn drop(&mut self) {
        // Fire-and-forget End + unregister (spec §4.3): the End is queued
        // behind any in-flight data, preserving write order.
        if !self.end_queued {
            let _ = self.write_tx.try_send(Frame {
                session_id: self.id,
                status: STATUS_END,
                option: 0,
                target: None,
                payload: Bytes::new(),
            });
        }
        self.unregister();
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use std::time::Duration;

    use bytes::Bytes;
    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt, ReadBuf};

    use super::*;

    /// Feeds a byte buffer in fixed-size pieces so the codec must
    /// reassemble frames across partial reads (no single read returns a
    /// whole field). Deterministic and non-blocking.
    struct PieceReader {
        data: Vec<u8>,
        pos: usize,
        piece: usize,
    }

    impl AsyncRead for PieceReader {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            let n = self
                .piece
                .min(self.data.len() - self.pos)
                .min(buf.remaining());
            buf.put_slice(&self.data[self.pos..self.pos + n]);
            self.pos += n;
            Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn new_frame_exact_bytes() {
        // Frame { session_id: 1, status: New, option: 0,
        //         target: Some(Tcp(127.0.0.1:8080)), payload: [] }
        // wire: [0x00,0x0C][0x00,0x01][0x01][0x00][0x01][0x1F,0x90][0x01][127,0,0,1]
        //   meta_len = 0x000C = 12 (2B sid + 1B status + 1B opt + 1B net
        //   + 2B port + 1B atyp + 4B addr)
        let (mut a, mut b) = tokio::io::duplex(64);
        write_frame(
            &mut a,
            &Frame {
                session_id: 1,
                status: STATUS_NEW,
                option: 0,
                target: Some(MuxTarget::Tcp("127.0.0.1:8080".parse().unwrap())),
                payload: Bytes::new(),
            },
        )
        .await
        .unwrap();
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        assert_eq!(
            raw,
            [
                0x00, 0x0C, 0x00, 0x01, 0x01, 0x00, 0x01, 0x1F, 0x90, 0x01, 127, 0, 0, 1
            ]
        );
    }

    #[tokio::test]
    async fn keep_data_frame_roundtrip() {
        // Keep frame with payload b"hello":
        // [meta_len 0x0004][sid 7][0x02][0x01][2B data_len 5][payload]
        let frame = Frame {
            session_id: 7,
            status: STATUS_KEEP,
            option: OPT_DATA,
            target: None,
            payload: Bytes::from_static(b"hello"),
        };
        let (mut a, mut b) = tokio::io::duplex(64);
        write_frame(&mut a, &frame).await.unwrap();
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        assert_eq!(
            raw,
            [
                0x00, 0x04, 0x00, 0x07, 0x02, 0x01, 0x00, 0x05, b'h', b'e', b'l', b'l', b'o'
            ]
        );
        // and it reads back to the same frame
        let mut c = std::io::Cursor::new(raw);
        let got = read_frame(&mut c).await.unwrap().unwrap();
        assert_eq!(got.session_id, frame.session_id);
        assert_eq!(got.status, frame.status);
        assert_eq!(got.option, frame.option);
        assert_eq!(got.target, frame.target);
        assert_eq!(&got.payload[..], &frame.payload[..]);
    }

    #[tokio::test]
    async fn end_frame_meta_only() {
        // status End, no data: [meta_len 0x0004][sid 3][0x03][0x00]
        // — no data_len on the wire (xray Close writes meta only).
        let (mut a, mut b) = tokio::io::duplex(64);
        write_frame(
            &mut a,
            &Frame {
                session_id: 3,
                status: STATUS_END,
                option: 0,
                target: None,
                payload: Bytes::new(),
            },
        )
        .await
        .unwrap();
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        assert_eq!(raw, [0x00, 0x04, 0x00, 0x03, 0x03, 0x00]);
    }

    #[test]
    fn domain_target_encode() {
        // port-first: [port 0x01BB][atyp 0x02][len 0x0B][b"example.com"]
        let bytes = encode_new_target(&MuxTarget::TcpDomain("example.com".into(), 443));
        let mut expected = vec![0x01, 0xBB, 0x02, 0x0B];
        expected.extend_from_slice(b"example.com");
        assert_eq!(bytes, expected);
    }

    #[tokio::test]
    async fn split_frame_reads() {
        // One Keep+Data frame fed two bytes at a time must reassemble.
        let wire = [
            0x00, 0x04, 0x00, 0x07, 0x02, 0x01, 0x00, 0x05, b'h', b'e', b'l', b'l', b'o',
        ];
        let mut r = PieceReader {
            data: wire.to_vec(),
            pos: 0,
            piece: 2,
        };
        let frame = read_frame(&mut r).await.unwrap().unwrap();
        assert_eq!(frame.session_id, 7);
        assert_eq!(frame.status, STATUS_KEEP);
        assert_eq!(frame.option, OPT_DATA);
        assert_eq!(frame.target, None);
        assert_eq!(&frame.payload[..], b"hello");
        // clean EOF after the frame
        assert!(read_frame(&mut r).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn eof_at_boundary_none() {
        let mut r = PieceReader {
            data: vec![],
            pos: 0,
            piece: 2,
        };
        assert!(read_frame(&mut r).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn oversized_meta_is_error() {
        // meta_len 0x0201 = 513 > 512 cap → InvalidData before any meta
        // body is read (mirrors xray's "invalid metalen").
        let mut r = PieceReader {
            data: vec![0x02, 0x01],
            pos: 0,
            piece: 2,
        };
        let err = read_frame(&mut r).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("metalen"));
    }

    #[tokio::test]
    async fn error_option_frame() {
        // option Error, status Keep, no data:
        // [meta_len 0x0004][sid 5][0x02][0x02]
        let (mut a, mut b) = tokio::io::duplex(64);
        write_frame(
            &mut a,
            &Frame {
                session_id: 5,
                status: STATUS_KEEP,
                option: OPT_ERROR,
                target: None,
                payload: Bytes::new(),
            },
        )
        .await
        .unwrap();
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        assert_eq!(raw, [0x00, 0x04, 0x00, 0x05, 0x02, 0x02]);
    }

    #[tokio::test]
    async fn new_frame_with_first_data_chunk() {
        // The client's first write is New + Data (xray writeData with
        // getNextFrameMeta New): target metadata, then [2B data_len][data].
        let (mut a, mut b) = tokio::io::duplex(64);
        write_frame(
            &mut a,
            &Frame {
                session_id: 9,
                status: STATUS_NEW,
                option: OPT_DATA,
                target: Some(MuxTarget::Tcp("192.0.2.1:443".parse().unwrap())),
                payload: Bytes::from_static(b"GET /"),
            },
        )
        .await
        .unwrap();
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        assert_eq!(
            raw,
            [
                0x00, 0x0C, 0x00, 0x09, 0x01, 0x01, 0x01, 0x01, 0xBB, 0x01, 192, 0, 2, 1, 0x00,
                0x05, b'G', b'E', b'T', b' ', b'/'
            ]
        );
        // reads back to the same frame
        let mut c = std::io::Cursor::new(raw);
        let got = read_frame(&mut c).await.unwrap().unwrap();
        assert_eq!(got.session_id, 9);
        assert_eq!(got.status, STATUS_NEW);
        assert_eq!(got.option, OPT_DATA);
        assert_eq!(
            got.target,
            Some(MuxTarget::Tcp("192.0.2.1:443".parse().unwrap()))
        );
        assert_eq!(&got.payload[..], b"GET /");
    }

    #[tokio::test]
    async fn domain_target_full_frame() {
        // New + Domain: meta_len = 2+1+1+1+2+1+1+11 = 20 = 0x0014.
        let (mut a, mut b) = tokio::io::duplex(64);
        write_frame(
            &mut a,
            &Frame {
                session_id: 2,
                status: STATUS_NEW,
                option: 0,
                target: Some(MuxTarget::TcpDomain("example.com".into(), 443)),
                payload: Bytes::new(),
            },
        )
        .await
        .unwrap();
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        let mut expected = vec![
            0x00, 0x14, 0x00, 0x02, 0x01, 0x00, 0x01, 0x01, 0xBB, 0x02, 0x0B,
        ];
        expected.extend_from_slice(b"example.com");
        assert_eq!(raw, expected);
    }

    #[tokio::test]
    async fn ipv6_target_roundtrip() {
        let target = MuxTarget::Tcp("[2001:db8::1]:443".parse().unwrap());
        let (mut a, mut b) = tokio::io::duplex(64);
        write_frame(
            &mut a,
            &Frame {
                session_id: 1,
                status: STATUS_NEW,
                option: 0,
                target: Some(target.clone()),
                payload: Bytes::new(),
            },
        )
        .await
        .unwrap();
        drop(a);
        let got = read_frame(&mut b).await.unwrap().unwrap();
        assert_eq!(got.session_id, 1);
        assert_eq!(got.status, STATUS_NEW);
        assert_eq!(got.target, Some(target));
        assert!(got.payload.is_empty());
    }

    #[tokio::test]
    async fn keepalive_frame_meta_only() {
        let (mut a, mut b) = tokio::io::duplex(64);
        write_frame(
            &mut a,
            &Frame {
                session_id: 0,
                status: STATUS_KEEPALIVE,
                option: 0,
                target: None,
                payload: Bytes::new(),
            },
        )
        .await
        .unwrap();
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        assert_eq!(raw, [0x00, 0x04, 0x00, 0x00, 0x04, 0x00]);
    }

    #[tokio::test]
    async fn partial_length_eof_is_error() {
        // One length byte then EOF → UnexpectedEof (a clean boundary is 0).
        let mut r = PieceReader {
            data: vec![0x00],
            pos: 0,
            piece: 2,
        };
        let err = read_frame(&mut r).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[tokio::test]
    async fn truncated_metadata_is_error() {
        // meta_len 10 but only 3 bytes follow.
        let mut r = PieceReader {
            data: vec![0x00, 0x0A, 0x00, 0x07, 0x02],
            pos: 0,
            piece: 3,
        };
        let err = read_frame(&mut r).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[tokio::test]
    async fn truncated_payload_is_error() {
        // meta fine (Keep+Data), data_len 5 but only 2 payload bytes.
        let mut r = PieceReader {
            data: vec![0x00, 0x04, 0x00, 0x07, 0x02, 0x01, 0x00, 0x05, b'h', b'i'],
            pos: 0,
            piece: 2,
        };
        let err = read_frame(&mut r).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[tokio::test]
    async fn unknown_network_is_error() {
        // New frame with network byte 0x02 (UDP): the codec is TCP-only
        // (mirrors xray's "unknown network type" — the target cannot be
        // represented, so the whole connection is unrecoverable).
        let mut r = PieceReader {
            data: vec![
                0x00, 0x0C, 0x00, 0x01, 0x01, 0x00, 0x02, 0x1F, 0x90, 0x01, 127, 0, 0, 1,
            ],
            pos: 0,
            piece: 2,
        };
        let err = read_frame(&mut r).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("network"));
    }

    #[tokio::test]
    async fn new_frame_without_target_is_invalid() {
        // A New frame must carry a target on the wire — writing one
        // without it is a client bug (InvalidInput), nothing written.
        let (mut a, mut b) = tokio::io::duplex(64);
        let err = write_frame(
            &mut a,
            &Frame {
                session_id: 1,
                status: STATUS_NEW,
                option: 0,
                target: None,
                payload: Bytes::new(),
            },
        )
        .await
        .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        assert!(raw.is_empty());
    }

    #[tokio::test]
    async fn payload_without_data_option_is_invalid() {
        // A payload needs the Data option — otherwise the reader would
        // misparse it as the next frame's meta_len.
        let (mut a, mut b) = tokio::io::duplex(64);
        let err = write_frame(
            &mut a,
            &Frame {
                session_id: 1,
                status: STATUS_KEEP,
                option: 0,
                target: None,
                payload: Bytes::from_static(b"x"),
            },
        )
        .await
        .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        drop(a);
        let mut raw = Vec::new();
        b.read_to_end(&mut raw).await.unwrap();
        assert!(raw.is_empty());
    }

    // ---- MuxClient + SessionStream ------------------------------------

    fn echo_target() -> MuxTarget {
        MuxTarget::TcpDomain("echo.test".into(), 80)
    }

    /// A write/reply round-trip on one session: the peer reads the Keep
    /// frame, answers with `payload`, and the app reads it back.
    async fn echo_roundtrip(
        stream: &mut SessionStream,
        peer: &mut tokio::io::DuplexStream,
        sid: u16,
        out: &[u8],
        reply: &[u8],
    ) {
        stream.write_all(out).await.expect("session write");
        let keep = read_frame(peer)
            .await
            .expect("Keep frame read")
            .expect("Keep frame");
        assert_eq!(keep.session_id, sid);
        assert_eq!(keep.status, STATUS_KEEP);
        assert_eq!(keep.option, OPT_DATA);
        assert_eq!(&keep.payload[..], out);
        write_frame(
            peer,
            &Frame {
                session_id: sid,
                status: STATUS_KEEP,
                option: OPT_DATA,
                target: None,
                payload: Bytes::copy_from_slice(reply),
            },
        )
        .await
        .expect("peer reply");
        let mut buf = [0u8; 32];
        let n = stream.read(&mut buf).await.expect("session read");
        assert_eq!(&buf[..n], reply);
    }

    #[tokio::test]
    async fn two_concurrent_sessions_independent() {
        let (client, mut peer) = tokio::io::duplex(8192);
        let mux = MuxClient::new(client);
        let target_a = MuxTarget::TcpDomain("a.example".into(), 80);
        let target_b = MuxTarget::TcpDomain("b.example".into(), 80);
        let mut a = mux.open_session(target_a.clone()).await.unwrap();
        let mut b = mux.open_session(target_b.clone()).await.unwrap();
        // The eager New frames arrive first, in open order, each carrying
        // its own target.
        let new_a = read_frame(&mut peer).await.unwrap().unwrap();
        let new_b = read_frame(&mut peer).await.unwrap().unwrap();
        assert_eq!(new_a.status, STATUS_NEW);
        assert_eq!(new_a.target.as_ref(), Some(&target_a));
        assert_eq!(new_b.status, STATUS_NEW);
        assert_eq!(new_b.target.as_ref(), Some(&target_b));

        // Interleaved writes on both sessions; the peer reassembles each
        // stream by session id.
        a.write_all(b"aaaa-1").await.unwrap();
        b.write_all(b"bbbb-1").await.unwrap();
        a.write_all(b"aaaa-2").await.unwrap();
        b.write_all(b"bbbb-2").await.unwrap();

        let mut stream_a = Vec::new();
        let mut stream_b = Vec::new();
        for _ in 0..4 {
            let frame = read_frame(&mut peer).await.unwrap().unwrap();
            assert_eq!(frame.status, STATUS_KEEP);
            assert_eq!(frame.option, OPT_DATA);
            if frame.session_id == new_a.session_id {
                stream_a.extend_from_slice(&frame.payload);
            } else {
                assert_eq!(frame.session_id, new_b.session_id);
                stream_b.extend_from_slice(&frame.payload);
            }
        }
        assert_eq!(stream_a, b"aaaa-1aaaa-2");
        assert_eq!(stream_b, b"bbbb-1bbbb-2");

        // Replies routed per session; each SessionStream gets its own data.
        write_frame(
            &mut peer,
            &Frame {
                session_id: new_a.session_id,
                status: STATUS_KEEP,
                option: OPT_DATA,
                target: None,
                payload: Bytes::from_static(b"reply-A"),
            },
        )
        .await
        .unwrap();
        write_frame(
            &mut peer,
            &Frame {
                session_id: new_b.session_id,
                status: STATUS_KEEP,
                option: OPT_DATA,
                target: None,
                payload: Bytes::from_static(b"reply-B"),
            },
        )
        .await
        .unwrap();
        let mut buf = [0u8; 16];
        let n = a.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"reply-A");
        let n = b.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"reply-B");
    }

    #[tokio::test]
    async fn end_closes_one_session_only() {
        let (client, mut peer) = tokio::io::duplex(8192);
        let mux = MuxClient::new(client);
        let mut a = mux
            .open_session(MuxTarget::TcpDomain("a.example".into(), 80))
            .await
            .unwrap();
        let mut b = mux
            .open_session(MuxTarget::TcpDomain("b.example".into(), 80))
            .await
            .unwrap();
        let new_a = read_frame(&mut peer).await.unwrap().unwrap();
        let new_b = read_frame(&mut peer).await.unwrap().unwrap();

        // The peer ends session A; A's read must see EOF, B keeps working.
        write_frame(
            &mut peer,
            &Frame {
                session_id: new_a.session_id,
                status: STATUS_END,
                option: 0,
                target: None,
                payload: Bytes::new(),
            },
        )
        .await
        .unwrap();
        let mut buf = [0u8; 16];
        assert_eq!(a.read(&mut buf).await.unwrap(), 0, "session A sees EOF");

        // B is unaffected: a full write/reply round-trip still works.
        echo_roundtrip(&mut b, &mut peer, new_b.session_id, b"b-data", b"b-reply").await;
        // A stays at EOF.
        assert_eq!(a.read(&mut buf).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn error_on_session() {
        let (client, mut peer) = tokio::io::duplex(8192);
        let mux = MuxClient::new(client);
        let mut a = mux
            .open_session(MuxTarget::TcpDomain("a.example".into(), 80))
            .await
            .unwrap();
        let mut b = mux
            .open_session(MuxTarget::TcpDomain("b.example".into(), 80))
            .await
            .unwrap();
        let new_a = read_frame(&mut peer).await.unwrap().unwrap();
        let new_b = read_frame(&mut peer).await.unwrap().unwrap();

        // The peer reports an error for A (Keep + Error option; an End +
        // Error frame routes the same way). A's read surfaces it.
        write_frame(
            &mut peer,
            &Frame {
                session_id: new_a.session_id,
                status: STATUS_KEEP,
                option: OPT_ERROR,
                target: None,
                payload: Bytes::new(),
            },
        )
        .await
        .unwrap();
        let mut buf = [0u8; 16];
        let err = a.read(&mut buf).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::ConnectionReset);

        // B is unaffected.
        echo_roundtrip(&mut b, &mut peer, new_b.session_id, b"b-data", b"b-reply").await;
    }

    #[tokio::test]
    async fn keepalive_consumed() {
        let (client, mut peer) = tokio::io::duplex(8192);
        let mux = MuxClient::new(client);
        let mut a = mux.open_session(echo_target()).await.unwrap();
        let new_a = read_frame(&mut peer).await.unwrap().unwrap();

        // Two KeepAlive frames (one meta-only, one with a payload — the
        // codec reads the payload for the Data option) are consumed, never
        // delivered as session data.
        write_frame(
            &mut peer,
            &Frame {
                session_id: 0,
                status: STATUS_KEEPALIVE,
                option: 0,
                target: None,
                payload: Bytes::new(),
            },
        )
        .await
        .unwrap();
        write_frame(
            &mut peer,
            &Frame {
                session_id: 0,
                status: STATUS_KEEPALIVE,
                option: OPT_DATA,
                target: None,
                payload: Bytes::from_static(b"tick"),
            },
        )
        .await
        .unwrap();
        write_frame(
            &mut peer,
            &Frame {
                session_id: new_a.session_id,
                status: STATUS_KEEP,
                option: OPT_DATA,
                target: None,
                payload: Bytes::from_static(b"real"),
            },
        )
        .await
        .unwrap();
        let mut buf = [0u8; 16];
        let n = a.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"real");

        // And the stream is still healthy afterwards.
        echo_roundtrip(&mut a, &mut peer, new_a.session_id, b"ping", b"pong").await;
    }

    #[tokio::test]
    async fn chunking_at_8kib() {
        let (client, mut peer) = tokio::io::duplex(8192);
        let mux = MuxClient::new(client);
        let mut a = mux.open_session(echo_target()).await.unwrap();
        let _new_a = read_frame(&mut peer).await.unwrap().unwrap();

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let data: Vec<u8> = (0..(CHUNK_SIZE * 2 + 4096))
            .map(|i| (i % 97) as u8)
            .collect();
        a.write_all(&data).await.unwrap();

        let mut total = 0usize;
        let mut frames = 0;
        let mut reassembled = Vec::new();
        while total < data.len() {
            let frame = read_frame(&mut peer).await.unwrap().unwrap();
            assert_eq!(frame.status, STATUS_KEEP);
            assert_eq!(frame.option, OPT_DATA);
            assert!(frame.payload.len() <= CHUNK_SIZE);
            frames += 1;
            total += frame.payload.len();
            reassembled.extend_from_slice(&frame.payload);
        }
        assert_eq!(frames, 3, "20 KiB splits into 8K + 8K + 4K frames");
        assert_eq!(reassembled, data);
    }

    #[tokio::test]
    async fn drop_sends_end() {
        let (client, mut peer) = tokio::io::duplex(8192);
        let mux = MuxClient::new(client);
        let mut a = mux.open_session(echo_target()).await.unwrap();
        let sid = read_frame(&mut peer).await.unwrap().unwrap().session_id;

        a.write_all(b"data").await.unwrap();
        drop(a);

        let keep = read_frame(&mut peer).await.unwrap().unwrap();
        assert_eq!(keep.session_id, sid);
        assert_eq!(keep.status, STATUS_KEEP);
        assert_eq!(&keep.payload[..], b"data");
        let end = read_frame(&mut peer).await.unwrap().unwrap();
        assert_eq!(end.session_id, sid);
        assert_eq!(end.status, STATUS_END);
        assert_eq!(end.option, 0);
        assert!(end.target.is_none());
        assert!(end.payload.is_empty());
    }

    #[tokio::test]
    async fn tunnel_eof_ends_all_sessions() {
        let (client, peer) = tokio::io::duplex(8192);
        let mux = MuxClient::new(client);
        let mut a = mux
            .open_session(MuxTarget::TcpDomain("a.example".into(), 80))
            .await
            .unwrap();
        let mut b = mux
            .open_session(MuxTarget::TcpDomain("b.example".into(), 80))
            .await
            .unwrap();
        // Dropping the peer: the tunnel read half sees EOF, the demux ends
        // every session (spec §6), and the client marks itself dead.
        drop(peer);
        let mut buf = [0u8; 8];
        assert_eq!(a.read(&mut buf).await.unwrap(), 0);
        assert_eq!(b.read(&mut buf).await.unwrap(), 0);
        // open_session now fails fast instead of queueing into a dead tunnel.
        assert!(mux.open_session(echo_target()).await.is_err());
    }

    #[tokio::test]
    async fn write_backpressure_never_ok_zero() {
        // A 300 KiB write far exceeds the 32-frame × 8 KiB writer channel:
        // poll_write must park (Pending — never Ok(0)) and the whole
        // payload must arrive intact once the peer drains.
        let (client, mut peer) = tokio::io::duplex(8192);
        let mux = MuxClient::new(client);
        let mut a = mux.open_session(echo_target()).await.unwrap();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let big: Vec<u8> = (0..(300 * 1024)).map(|i| (i % 251) as u8).collect();
        let big_len = big.len();

        let drainer = tokio::spawn(async move {
            let mut got = 0usize;
            while let Ok(Some(frame)) = read_frame(&mut peer).await {
                got += frame.payload.len();
                if got >= big_len {
                    // Payload complete; drain stragglers still in the
                    // writer channel, then stop on half a second of
                    // silence.
                    while let Ok(Ok(Some(f))) =
                        tokio::time::timeout(Duration::from_millis(500), read_frame(&mut peer))
                            .await
                    {
                        got += f.payload.len();
                    }
                    break;
                }
            }
            got
        });

        let mut written = 0;
        while written < big.len() {
            let n = a.write(&big[written..]).await.expect("session write");
            assert!(n > 0, "AsyncWrite must never report Ok(0)");
            written += n;
        }
        let got = drainer.await.expect("drainer task");
        assert_eq!(got, big.len());
    }

    #[tokio::test(start_paused = true)]
    async fn client_sends_keepalive_frames() {
        let (client, mut peer) = tokio::io::duplex(4096);
        let mux = MuxClient::new(client);
        // With no sessions the tunnel still emits a KeepAlive every
        // interval (deviation 2). Drive the virtual clock forward while
        // waiting for the first one.
        let clock = tokio::spawn(async move {
            for _ in 0..300 {
                tokio::time::advance(Duration::from_millis(100)).await;
                tokio::task::yield_now().await;
            }
        });
        let frame = tokio::time::timeout(Duration::from_secs(20), read_frame(&mut peer))
            .await
            .expect("keepalive frame within 20 virtual seconds")
            .expect("frame read")
            .expect("some frame");
        assert_eq!(frame.session_id, 0);
        assert_eq!(frame.status, STATUS_KEEPALIVE);
        assert_eq!(frame.option, 0);
        assert!(frame.payload.is_empty());
        clock.abort();
        // Dropping the client stops the keepalive; the tunnel then tears
        // down once the writer channel closes (no sessions hold senders).
        drop(mux);
    }
}
