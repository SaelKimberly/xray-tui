//! TEMPORARY [`InjectToCoreConf`] stub impls.
//!
//! The sing-box-native protocols (tuic/hysteria1/naive/anytls/shadowtls/tor/
//! ssh/tailscale/ssr) got real `inject_to` impls in their own config files in
//! T15 Step 1; the shared-set protocols got theirs in T14 (xray) + T15 Step 2
//! (sing-box). This file keeps only the [`PlaceholderConfig`] impl until T15
//! Step 3 moves it next to the struct in `mod.rs` (then this file is deleted).
//!
//! Kind strings follow the [`ProtocolKind::as_str`](crate::proto_spec::ProtocolKind::as_str)
//! dialect ("hy2", "any-tls", "shadow-tls", ...).

use serde_json::Value;

use super::{
    CoreType, EndpointEssentials, InjectOptions, InjectToCoreConf, PlaceholderConfig, SupportError,
};

impl InjectToCoreConf for PlaceholderConfig {
    fn inject_to(
        &self,
        _core_conf: &mut Value,
        core_type: CoreType,
        _endpoint: Option<&EndpointEssentials>,
        _opts: InjectOptions,
    ) -> Result<(), SupportError> {
        // Redirect / TProxy / Mixed share this one type; the variant is carried
        // in `proto_name` ("redirect" / "tproxy" / "mixed" as written by
        // `from_legacy_parse` and `try_parse_proto`), so it is the kind string.
        Err(SupportError::UnsupportedProtocol(
            self.proto_name.clone(),
            core_type,
        ))
    }
}
