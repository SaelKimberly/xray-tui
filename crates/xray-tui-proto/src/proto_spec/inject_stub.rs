//! TEMPORARY [`InjectToCoreConf`] stub impls.
//!
//! The xray-native protocols (vless/vmess/trojan/ss/socks/http/wireguard/
//! hysteria2) got real `inject_to` impls in their own config files in T14;
//! this file keeps only the remaining stubs. Every config below errors with
//! [`SupportError::UnsupportedProtocol`] (kind string + requested core) until
//! T15 lands the sing-box shapes. The kind string in the error identifies the
//! variant so [`ProtocolConfig`] dispatch routing is testable today.
//!
//! These impls are replaced per-config in T15 — do NOT build on them.
//! Kind strings follow the [`ProtocolKind::as_str`](crate::proto_spec::ProtocolKind::as_str)
//! dialect ("hy2", "any-tls", "shadow-tls", ...).

use serde_json::Value;

use super::{
    AnyTlsConfig, CoreType, EndpointEssentials, Hysteria1Config, InjectOptions, InjectToCoreConf,
    NaiveConfig, PlaceholderConfig, ShadowTlsConfig, SshConfig, SsrConfig, SupportError,
    TailscaleConfig, TorConfig, TuicConfig,
};

/// Stub `inject_to` for one config: always `UnsupportedProtocol(kind, core)`.
/// `opts` is ignored by the stubs (signature-only until T14/15 real impls).
macro_rules! stub_inject {
    ($config:ty, $kind:literal) => {
        impl InjectToCoreConf for $config {
            fn inject_to(
                &self,
                _core_conf: &mut Value,
                core_type: CoreType,
                _endpoint: Option<&EndpointEssentials>,
                _opts: InjectOptions,
            ) -> Result<(), SupportError> {
                Err(SupportError::UnsupportedProtocol($kind.into(), core_type))
            }
        }
    };
}

stub_inject!(SsrConfig, "ssr");
stub_inject!(TuicConfig, "tuic");
stub_inject!(NaiveConfig, "naive");
stub_inject!(AnyTlsConfig, "any-tls");
stub_inject!(ShadowTlsConfig, "shadow-tls");
stub_inject!(TorConfig, "tor");
stub_inject!(SshConfig, "ssh");
stub_inject!(TailscaleConfig, "tailscale");
stub_inject!(Hysteria1Config, "hy");

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
