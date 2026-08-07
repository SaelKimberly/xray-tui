//! Transport phase: get a byte stream to the proxy server (or reuse an
//! existing tunnel for the later hops of a chain).
//!
//! OUTERMOST layer (Xray composition order): dial → transport → security →
//! protocol. A transport answers two questions: "dial the server" (`base:
//! None`) and "upgrade an existing tunnel in place" (`base: Some`, e.g. a
//! WebSocket handshake on a later chain hop).

use crate::context::LinkContext;
use crate::error::NativeError;
use crate::BoxStream;

pub mod tcp;

/// Run the transport step. `base` is the stream from the previous chain hop
/// (or `None` for the first hop, which dials the server directly).
pub async fn connect(
    ctx: &LinkContext,
    base: Option<BoxStream>,
) -> Result<BoxStream, NativeError> {
    match ctx.transport_type() {
        None | Some("tcp") => tcp::connect(ctx, base).await,
        Some(t) => Err(NativeError::NotImplemented {
            feature: format!("transport {t}"),
        }),
    }
}
