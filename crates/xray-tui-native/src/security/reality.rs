//! REALITY client: the `xray-tui-tls` ring port wired into the security phase.
//!
//! `wrap()` routes `TlsConfig::Reality` here. [`connect`] reads the REALITY
//! opts (`sni`/`pbk`/`sid`) from the link's security config, selects the
//! `HelloProvisioner` (default: [`FixedChrome133`], or a caller-supplied
//! custom provisioner via `NativeConnectParams::reality_provisioner`), and
//! runs the full client handshake — fingerprint-shaped `ClientHello` with a
//! sealed `SessionId`, X25519 auth key, HMAC/Ed25519 server auth — bounded by
//! `timeouts::SECURITY`.

use std::sync::Arc;

use base64::Engine as _;
use xray_tui_proto::proto_spec::TlsConfig;
use xray_tui_tls::reality::{RealityParams, connect_reality};

use crate::BoxStream;
use crate::context::LinkContext;
use crate::error::{NativeError, timeouts};
use crate::security::tls_provider::{TlsConnector, TlsParams};

pub use xray_tui_tls::reality::{
    FixedChrome133, HelloProvisionParams, HelloProvisioner, ProvisionedHello,
};

/// Chosen provisioner for a REALITY connect.
#[derive(Clone, Default)]
pub enum HelloProvisionerChoice {
    #[default]
    FixedChrome133,
    Custom(Arc<dyn HelloProvisioner>),
}

impl std::fmt::Debug for HelloProvisionerChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FixedChrome133 => f.write_str("FixedChrome133"),
            // `dyn HelloProvisioner` is deliberately not `Debug`; the
            // concrete provisioner is observable through the type itself.
            Self::Custom(_) => f.write_str("Custom(..)"),
        }
    }
}

impl HelloProvisionerChoice {
    fn provisioner(&self) -> &dyn HelloProvisioner {
        match self {
            Self::FixedChrome133 => &FixedChrome133,
            Self::Custom(p) => p.as_ref(),
        }
    }
}

/// Decode a REALITY `pbk` (base64url, no padding — Xray's `privateKey`
/// encoding) to its 32 bytes.
fn decode_pbk(s: &str) -> Result<[u8; 32], NativeError> {
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(s)
        .map_err(|e| NativeError::Reality(format!("invalid pbk base64url: {e}")))?;
    let pbk: [u8; 32] = bytes
        .try_into()
        .map_err(|_| NativeError::Reality("pbk must decode to 32 bytes".into()))?;
    Ok(pbk)
}

/// Decode a REALITY short id (hex, ≤8 bytes) to its bytes.
fn decode_sid(s: &str) -> Result<Vec<u8>, NativeError> {
    if s.len() > 16 || !s.len().is_multiple_of(2) {
        return Err(NativeError::Reality(format!(
            "short id {s:?} must be hex, at most 8 bytes"
        )));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| NativeError::Reality(format!("invalid short id {s:?}: {e}")))
        })
        .collect()
}

/// The REALITY client handshake.
///
/// Reads the REALITY opts from the link's security config; the SNI defaults
/// to the endpoint host when the config carries none. The provisioner comes
/// from `NativeConnectParams::reality_provisioner` (`FixedChrome133` unless
/// the caller injected a custom one).
pub async fn connect(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    let Some(sec) = ctx.security() else {
        return Err(NativeError::Reality(
            "link has no security config for REALITY".into(),
        ));
    };
    if !matches!(sec.tls, Some(TlsConfig::Reality(_))) {
        return Err(NativeError::Reality(
            "security config is not REALITY".into(),
        ));
    }
    let sni = sec.sni().unwrap_or(&ctx.params.server.host).to_string();
    let pbk = sec
        .pbk()
        .ok_or_else(|| NativeError::Reality("reality config missing pbk".into()))?;
    let sid = sec.sid().unwrap_or_default();
    run_handshake(
        stream,
        &sni,
        ctx.params.reality_provisioner.provisioner(),
        &decode_pbk(pbk)?,
        &decode_sid(sid)?,
    )
    .await
}

/// The REALITY handshake driver: runs `connect_reality` over `stream`,
/// bounded by `timeouts::SECURITY`.
async fn run_handshake(
    stream: BoxStream,
    server_name: &str,
    provisioner: &dyn HelloProvisioner,
    public_key: &[u8; 32],
    short_id: &[u8],
) -> Result<BoxStream, NativeError> {
    let rng = ring::rand::SystemRandom::new();
    let timeout = timeouts::SECURITY;
    let tls = tokio::time::timeout(
        timeout,
        connect_reality(
            stream,
            RealityParams {
                server_name,
                public_key,
                short_id,
                provisioner,
                rng: &rng,
            },
        ),
    )
    .await
    .map_err(|_| NativeError::Timeout {
        step: "reality handshake",
        limit: timeout,
    })?
    .map_err(|e| NativeError::Reality(format!("reality handshake: {e}")))?;
    Ok(Box::new(tls) as BoxStream)
}

/// REALITY `TlsConnector`: runs the `xray-tui-tls` REALITY handshake over a
/// transport stream.
///
/// Carries the server material (static X25519 public key, short id) and the
/// provisioner choice; [`connect`] builds one from the link's REALITY opts.
/// Usable directly through `TlsProvider::Custom` when the caller holds the
/// material itself (e.g. a custom `HelloProvisioner`).
pub struct RealityConnector {
    /// The provisioner shaping the `ClientHello` (default: [`FixedChrome133`]).
    pub provisioner: HelloProvisionerChoice,
    /// The server's static X25519 public key (decoded `pbk`).
    pub public_key: [u8; 32],
    /// The REALITY short id (decoded `sid`, ≤8 bytes).
    pub short_id: Vec<u8>,
}

impl TlsConnector for RealityConnector {
    fn connect(
        &self,
        stream: BoxStream,
        params: TlsParams,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<BoxStream, NativeError>> + Send>> {
        // Copy the connection material so the future does not borrow `self`.
        let provisioner = self.provisioner.clone();
        let public_key = self.public_key;
        let short_id = self.short_id.clone();
        Box::pin(async move {
            run_handshake(
                stream,
                &params.sni,
                provisioner.provisioner(),
                &public_key,
                &short_id,
            )
            .await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_pbk_accepts_base64url_32_bytes() {
        let pbk = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([0xABu8; 32]);
        assert_eq!(decode_pbk(&pbk).unwrap(), [0xAB; 32]);
    }

    #[test]
    fn decode_pbk_rejects_short_and_malformed() {
        assert!(decode_pbk("Zm9vYmFy").is_err()); // 6 bytes
        assert!(decode_pbk("!!!").is_err()); // not base64
    }

    #[test]
    fn decode_sid_accepts_hex_up_to_8_bytes() {
        assert_eq!(
            decode_sid("0011223344556677").unwrap(),
            vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]
        );
        assert_eq!(decode_sid("ab").unwrap(), vec![0xAB]);
        assert_eq!(decode_sid("").unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn decode_sid_rejects_odd_length_and_too_long() {
        assert!(decode_sid("abc").is_err());
        assert!(decode_sid("001122334455667788").is_err()); // 9 bytes
        assert!(decode_sid("zz").is_err()); // not hex
    }

    #[test]
    fn provisioner_choice_defaults_to_fixed_chrome133() {
        let choice = HelloProvisionerChoice::default();
        assert_eq!(format!("{choice:?}"), "FixedChrome133");
        let _ = choice.provisioner(); // resolves without panicking
    }
}
