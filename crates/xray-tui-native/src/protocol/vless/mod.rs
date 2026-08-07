//! VLESS — the reference protocol for the native core.

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use xray_tui_proto::proto_spec::VlessConfig;

use crate::context::LinkContext;
use crate::error::{NativeError, timeouts};
use crate::BoxStream;

pub mod header;

/// Connect through a VLESS outbound over an already-secured stream.
///
/// Writes the request header, validates the response header, then returns the
/// stream unchanged (raw passthrough body).
pub async fn connect(
    ctx: &LinkContext,
    mut stream: BoxStream,
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
    let request = header::encode_request(&uuid, &ctx.target, header::CMD_TCP);
    let timeout = timeouts::PROTOCOL;
    tokio::time::timeout(timeout, stream.write_all(&request))
        .await
        .map_err(|_| NativeError::Timeout {
            step: "vless request write",
            limit: timeout,
        })??;

    let mut head = [0u8; 2];
    tokio::time::timeout(timeout, stream.read_exact(&mut head))
        .await
        .map_err(|_| NativeError::Timeout {
            step: "vless response read",
            limit: timeout,
        })??;
    let addon_len = header::check_response_header(&head)?;
    if addon_len > 0 {
        let mut addons = vec![0u8; addon_len];
        tokio::time::timeout(timeout, stream.read_exact(&mut addons))
            .await
            .map_err(|_| NativeError::Timeout {
                step: "vless response addons",
                limit: timeout,
            })??;
    }
    Ok(stream)
}
