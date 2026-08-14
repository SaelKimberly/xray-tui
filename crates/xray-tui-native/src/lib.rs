//! Native proxy core — client-side, in-process implementations of the proxy
//! protocols xray-tui otherwise obtains by spawning xray-core / sing-box
//! subprocesses.
//!
//! # Layering
//!
//! Transports are OUTERMOST, protocols INNERMOST (Xray composition order):
//! dial → transport upgrade → security wrap → protocol handshake → tunnel.
//!
//! # Placeholder semantics
//!
//! Every protocol kind, transport, and security has a module and a dispatch
//! arm; unsupported combinations return [`NativeError::NotImplemented`].
//! The uniform `handshake(stream)` shape applies to the TCP-stream family
//! only — WireGuard/Tailscale (device tunnels), SSH (own handshake), and the
//! outbound-only kinds (Redirect/TProxy/Mixed/DokodemoDoor/...) diverge at
//! dispatch time per [`shape::ConnectShape`]. See `shape.rs`.
//!
//! # Config source of truth
//!
//! [`NativeConnectParams`] wraps `xray-tui-proto` typed models
//! ([`ProtocolConfig`], [`EndpointEssentials`]). No config model is defined
//! here.

/// Byte-stream capability: readable, writable, `Unpin`, `Send` — the seam
/// between layers.
///
/// `Any` (so every stream is `'static`) lets a layer recover the concrete
/// stream behind the box: the vision codec splices the engine `TlsStream`'s
/// per-direction direct mode through the `DirectMode` impl on [`BoxStream`]
/// (see `protocol/vless/vision.rs`).
pub trait Stream:
    tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + std::any::Any
{
}
impl<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static> Stream for T {}

/// The boxed byte-stream seam between layers.
pub type BoxStream = Box<dyn Stream>;

pub mod addr;
pub mod chain;
pub mod context;
pub mod crypto;
#[cfg(feature = "native-e2e")]
pub mod e2e;
pub mod error;
pub mod protocol;
pub mod security;
pub mod shape;
pub mod transport;

pub use chain::{connect_chain, connect_chain_udp};
pub use context::{LinkContext, NativeConnectParams};
pub use error::NativeError;
pub use protocol::vless::PacketConn;

/// Connect through a single proxy to `params.target`.
pub async fn connect(params: NativeConnectParams) -> Result<NativeTunnel, NativeError> {
    let target = params.target.clone();
    connect_chain(&[params], target).await
}

/// Connect through a single proxy with a UDP datagram tunnel to
/// `params.target` (VLESS command 0x02; `params.udp` selects the packet
/// mode: `Raw` or `PacketAddr`).
///
/// The same dial → security → transport chain as [`connect`] runs
/// unchanged; only the protocol phase differs (command 0x02 + packet
/// framing). The returned [`PacketConn`] is a datagram API over the tunnel.
pub async fn connect_udp(
    params: &NativeConnectParams,
) -> Result<PacketConn<BoxStream>, NativeError> {
    let target = params.target.clone();
    connect_chain_udp(std::slice::from_ref(params), target).await
}

/// A completed native tunnel: the byte stream after the full layer stack.
///
/// Dropping the tunnel closes the underlying connection.
pub struct NativeTunnel {
    inner: BoxStream,
}

impl NativeTunnel {
    /// Wrap a completed layer stack (crate-internal; external users get
    /// tunnels from `connect`/`connect_chain`).
    pub(crate) fn from_stream(inner: BoxStream) -> Self {
        Self { inner }
    }
}

impl tokio::io::AsyncRead for NativeTunnel {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for NativeTunnel {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}
