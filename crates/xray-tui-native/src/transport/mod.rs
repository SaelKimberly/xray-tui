//! Transport phase: get a byte stream to the proxy server (or reuse an
//! existing tunnel for the later hops of a chain).
//!
//! `connect` dials the server (`base: None`) or reuses an existing tunnel
//! (`base: Some`). `upgrade` runs the transport framing (ws/grpc) over an
//! ALREADY-established stream — the engine TLS/REALITY session (Xray
//! composition order: dial → security → transport upgrade → protocol; TLS is
//! OUTERMOST, transport framing runs inside it).

use crate::BoxStream;
use crate::context::LinkContext;
use crate::error::NativeError;

pub mod grpc;
pub mod tcp;
pub mod ws;

/// Run the transport step. `base` is the stream from the previous chain hop
/// (or `None` for the first hop, which dials the server directly).
pub async fn connect(ctx: &LinkContext, base: Option<BoxStream>) -> Result<BoxStream, NativeError> {
    match ctx.transport_type() {
        None | Some("tcp") => tcp::connect(ctx, base).await,
        Some(t) => Err(NativeError::NotImplemented {
            feature: format!("transport {t}"),
        }),
    }
}

/// Run the transport-upgrade step over an established stream (the secured
/// engine stream). TCP = passthrough; ws/grpc = framing handshake over the
/// stream (implemented in `ws`/`grpc`).
pub async fn upgrade(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    match ctx.transport_type() {
        Some("ws") => ws::connect(ctx, stream).await,
        Some("grpc") => grpc::connect(ctx, stream).await,
        None | Some("tcp") => Ok(stream),
        Some(t) => Err(NativeError::NotImplemented {
            feature: format!("transport {t}"),
        }),
    }
}
