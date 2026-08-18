//! Transport phase: get a byte stream to the proxy server (or reuse an
//! existing tunnel for the later hops of a chain).
//!
//! `connect` dials the server (`base: None`) or reuses an existing tunnel
//! (`base: Some`). `upgrade` runs the transport framing (ws/grpc/httpupgrade)
//! over an ALREADY-established stream — the engine TLS/REALITY session (Xray
//! composition order: dial → security → transport upgrade → protocol; TLS is
//! OUTERMOST, transport framing runs inside it).

use crate::BoxStream;
use crate::context::LinkContext;
use crate::error::NativeError;

pub mod grpc;
pub mod http;
pub mod httpupgrade;
pub mod kcp;
pub mod tcp;
pub mod v2rayhttp;
pub mod ws;
pub mod xhttp;

/// Run the transport step: dial the server (`base: None`) or reuse the
/// previous tunnel (`base: Some`) — always TCP; framing is an upgrade over
/// the secured stream (see [`upgrade`]).
pub async fn connect(ctx: &LinkContext, base: Option<BoxStream>) -> Result<BoxStream, NativeError> {
    match ctx.transport_type() {
        None | Some("tcp" | "ws" | "grpc" | "httpupgrade" | "xhttp" | "http") => {
            tcp::connect(ctx, base).await
        }
        Some(t) => Err(NativeError::NotImplemented {
            feature: format!("transport {t}"),
        }),
    }
}

/// Run the transport-upgrade step over an established (secured) stream:
/// TCP = passthrough; ws/grpc/httpupgrade/xhttp/v2rayhttp = framing handshake.
pub async fn upgrade(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    match ctx.transport_type() {
        Some("ws") => ws::connect(ctx, stream).await,
        Some("grpc") => grpc::connect(ctx, stream).await,
        Some("httpupgrade") => httpupgrade::connect(ctx, stream).await,
        Some("xhttp") => xhttp::connect(ctx, stream).await,
        Some("http") => v2rayhttp::connect(ctx, stream).await,
        None | Some("tcp") => Ok(stream),
        Some(t) => Err(NativeError::NotImplemented {
            feature: format!("transport {t}"),
        }),
    }
}
