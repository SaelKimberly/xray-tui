//! `VMess` — native client (modern AEAD, xtls dialect).
//!
//! Wire contract: `thirdparty/Xray-core/proxy/vmess/encoding/{client,encoding,server}.go`
//! and `proxy/vmess/aead/` (MIT). Only TCP + AES-128-GCM payload security;
//! xray-core 26.x refuses `none`/`zero`/`auto` body streams server-side.

use tokio::io::AsyncWriteExt;

use xray_tui_proto::proto_spec::VmessConfig;

use crate::BoxStream;
use crate::context::LinkContext;
use crate::error::{NativeError, timeouts};
use crate::protocol::vmess::header::{Session, encode_request};
use crate::protocol::vmess::keys::cmd_key;
use crate::protocol::vmess::stream::VmessClientStream;

#[cfg(feature = "native-e2e")]
pub mod e2e;
pub mod header;
pub mod keys;
pub mod stream;

/// Validate the `VMess` payload security the config requests.
pub fn check_security(cfg: &VmessConfig) -> Result<(), NativeError> {
    match cfg.security.enc.as_deref() {
        None | Some("" | "auto" | "aes-128-gcm") => Ok(()),
        Some(other) => Err(NativeError::Config(format!(
            "vmess payload security {other:?} not supported (native core: aes-128-gcm only)"
        ))),
    }
}

/// Connect through a `VMess` outbound over an already-secured stream.
pub async fn connect(
    ctx: &LinkContext,
    stream: BoxStream,
    cfg: &VmessConfig,
) -> Result<BoxStream, NativeError> {
    check_security(cfg)?;
    let uuid = crate::protocol::vless::header::uuid_bytes(&cfg.uuid)?;
    let ck = cmd_key(&uuid);
    let session = Session::new();

    let mut entropy = |out: &mut [u8]| {
        use ring::rand::{SecureRandom, SystemRandom};
        SystemRandom::new().fill(out).expect("rng failure");
    };
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_secs()).ok())
        .unwrap_or(0);
    let request = encode_request(&ck, &session, &ctx.target, ts, &mut entropy);
    let timeout = timeouts::PROTOCOL;
    let mut stream = stream;
    tokio::time::timeout(timeout, stream.write_all(&request))
        .await
        .map_err(|_| NativeError::Timeout {
            step: "vmess request write",
            limit: timeout,
        })??;

    Ok(Box::new(VmessClientStream::new(stream, session)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vcfg(enc: &str) -> VmessConfig {
        serde_json::from_value(serde_json::json!({
            "schema": "Vmess",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "security": { "enc": enc },
            "transport": { "type": "tcp" }
        }))
        .expect("vmess config parses")
    }

    #[test]
    fn rejects_unsupported_payload_securities() {
        for enc in ["none", "zero", "chacha20-poly1305"] {
            let cfg = vcfg(enc);
            assert!(
                matches!(check_security(&cfg), Err(NativeError::Config(_))),
                "{enc}"
            );
        }
    }

    #[test]
    fn accepts_aes128_gcm_and_auto() {
        assert!(check_security(&vcfg("aes-128-gcm")).is_ok());
        assert!(check_security(&vcfg("auto")).is_ok());
        assert!(check_security(&vcfg("")).is_ok()); // absent -> auto
    }

    #[test]
    fn accepts_absent_security_key() {
        let cfg: VmessConfig = serde_json::from_value(serde_json::json!({
            "schema": "Vmess",
            "uuid": "00000000-0000-0000-0000-000000000000",
            "transport": { "type": "tcp" }
        }))
        .expect("vmess config without security key parses");
        assert!(cfg.security.enc.is_none());
        assert!(check_security(&cfg).is_ok());
    }
}
