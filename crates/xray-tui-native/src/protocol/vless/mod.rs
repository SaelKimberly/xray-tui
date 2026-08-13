//! VLESS — the reference protocol for the native core.

use tokio::io::AsyncWriteExt;

use xray_tui_proto::proto_spec::VlessConfig;

use crate::BoxStream;
use crate::context::LinkContext;
use crate::error::{NativeError, timeouts};
use crate::protocol::vless::stream::VlessClientStream;

pub mod header;
pub mod stream;
pub(crate) mod vision;

/// Connect through a VLESS outbound over an already-secured stream.
///
/// Writes the request header and returns a tunnel that strips the response
/// header on its first read (see `stream.rs` for the eager-vs-lazy header
/// semantics of xray-core vs sing-box).
pub async fn connect(
    ctx: &LinkContext,
    stream: BoxStream,
    cfg: &VlessConfig,
) -> Result<BoxStream, NativeError> {
    // Flow guard: M1 supports no flow. `xtls-rprx-vision` needs TLS1.3 +
    // stream splicing (not implemented yet); anything else is a config error.
    if let Some(flow) = cfg.flow.as_ref() {
        let flow = flow.to_string();
        if !flow.is_empty() {
            return Err(NativeError::NotImplemented {
                feature: format!("vless flow {flow}"),
            });
        }
    }

    let uuid = header::uuid_bytes(&cfg.uuid)?;
    let request = header::encode_request(&uuid, &ctx.target, header::CMD_TCP)?;
    let timeout = timeouts::PROTOCOL;
    let mut stream = stream;
    tokio::time::timeout(timeout, stream.write_all(&request))
        .await
        .map_err(|_| NativeError::Timeout {
            step: "vless request write",
            limit: timeout,
        })??;

    Ok(Box::new(VlessClientStream::new(stream)))
}
