//! Native-capability predicate: which protocol+config rows the in-process
//! core may serve (spec brief §2).
//!
//! [`kind_supported`] is the cheap kind-only gate (display/sort paths that
//! cannot load the full config); [`supported`] is the config-aware runtime
//! gate (connect/ping paths). A row native serves *worse* than a subprocess
//! — notably VLESS `mlkem768x25519plus` account encryption, where native
//! diverges from real-xray interop (`NATIVE_CORE.md` `SP7` pq-enc) — returns
//! `false` so Auto resolution falls back to the subprocess.
//!
//! The predicate mirrors the native dispatch arms, not xray's feature set:
//! [`crate::protocol`], [`crate::transport`], and [`crate::security`]. A
//! combo both cores reject (equal-failure) stays `true` per brief D5; only
//! native-worse/deferred markers return `false`.
//!
//! Unknown values fail CLOSED. A row carrying an id or field native cannot
//! parse (an xray-only uTLS fingerprint, an mKCP seed) returns `false`: a
//! false positive is a connection that dies (or worse, hangs) where the
//! subprocess would have worked, while a false negative costs only the
//! in-process fast path.

use xray_tui_proto::proto_spec::common::{KcpConfig, TransportConfig};
use xray_tui_proto::proto_spec::{
    Hysteria2Config, ProtocolConfig, ProtocolKind, SecurityConfig, SsConfig, TrojanConfig,
    VlessConfig, VmessConfig,
};

use crate::protocol::ss::method::password_key;
use crate::protocol::ss::resolve_method;
use crate::security::fingerprint::parse_fingerprint_id;

/// The protocols with a native implementation, in canonical order.
///
/// Must stay exactly `[Vless, Vmess, Trojan, Hysteria2, Shadowsocks,
/// Shadowsocks2022]` — the e2e-verified protocols the TUI may Auto-resolve
/// onto native. The two Shadowsocks kinds are separate entries because the
/// typed kind carries the cipher family (`2022-blake3-*` → `Shadowsocks2022`)
/// even though both share one [`SsConfig`].
pub const NATIVE_KINDS: &[ProtocolKind] = &[
    ProtocolKind::Vless,
    ProtocolKind::Vmess,
    ProtocolKind::Trojan,
    ProtocolKind::Hysteria2,
    ProtocolKind::Shadowsocks,
    ProtocolKind::Shadowsocks2022,
];

/// True when the protocol kind has a native implementation at all.
///
/// Config-blind: used where the loaded [`ProtocolConfig`] is unavailable
/// (display/sort). Connect-time may still downgrade on a
/// capability-deferred config (see [`supported`]).
#[must_use]
pub const fn kind_supported(kind: ProtocolKind) -> bool {
    matches!(
        kind,
        ProtocolKind::Vless
            | ProtocolKind::Vmess
            | ProtocolKind::Trojan
            | ProtocolKind::Hysteria2
            | ProtocolKind::Shadowsocks
            | ProtocolKind::Shadowsocks2022
    )
}

/// True when native should serve this exact protocol+config row.
///
/// `kind_supported(kind)` AND the config requests no native-worse/deferred
/// feature, AND the config variant matches `kind` (a kind/config mismatch
/// is never servable).
///
/// The verdict is TCP-truthful — it covers the byte-stream path only.
/// SOCKS5 UDP ASSOCIATE through the native proxy outbound is not
/// implemented: `inbound::outbound::proxy_params` never sets `params.udp`,
/// and `protocol::vless::connect_udp` rejects `params.udp == None` as a
/// config error, so a native session drops the proxy UDP leg (debug-logged)
/// no matter what this predicate answers. Gating UDP-capable shapes off
/// that gap would cost them their native TCP path for a UDP leg no config
/// can reach today — see `vless_supported` for the vision flows.
///
/// The Shadowsocks row is the sharpest case of that stance: `SsConfig` has no
/// transport field at all, so the verdict covers the plain-TCP dial plus
/// optional `security` and nothing else — the UDP dial-end refuses a
/// non-empty `security` inside `ss::udp::connect_udp`, and refuses a chain in
/// `chain.rs::ss_udp_guard`, which is where both refusals belong.
#[must_use]
pub fn supported(kind: ProtocolKind, config: &ProtocolConfig) -> bool {
    if !kind_supported(kind) {
        return false;
    }
    match (kind, config) {
        (ProtocolKind::Vless, ProtocolConfig::Vless(cfg)) => vless_supported(cfg),
        (ProtocolKind::Vmess, ProtocolConfig::Vmess(cfg)) => vmess_supported(cfg),
        (ProtocolKind::Trojan, ProtocolConfig::Trojan(cfg)) => trojan_supported(cfg),
        (ProtocolKind::Hysteria2, ProtocolConfig::Hysteria2(cfg)) => hysteria2_supported(cfg),
        (ProtocolKind::Shadowsocks | ProtocolKind::Shadowsocks2022, ProtocolConfig::Ss(cfg)) => {
            ss_supported(kind, cfg)
        }
        _ => false,
    }
}

/// Transports native can build — a POSITIVE match of the arms
/// `transport::connect` / `transport::upgrade` actually dispatch:
/// tcp/ws/grpc/httpupgrade/xhttp/http (v2rayhttp) ride the dial + upgrade
/// chain, xhttp+h3 replaces the dial with its own QUIC one, and kcp is a
/// fresh UDP dial.
///
/// Bare `TransportConfig::Quic` has no arm (`NotImplemented("transport
/// quic")`) — xray-only, so deferred. The match is deliberately exhaustive
/// (no wildcard): a variant added to `TransportConfig` breaks THIS function
/// at compile time rather than inheriting `true`, which is the strongest
/// fail-closed shape available — nothing new can pass unreviewed.
///
/// `path` is the protocol row's own `path` field, forwarded for the mKCP
/// seed check ([`kcp_supported`]); every other arm ignores it (their path
/// lives inside the transport config).
fn transport_supported(transport: &TransportConfig, path: Option<&str>) -> bool {
    match transport {
        TransportConfig::Tcp
        | TransportConfig::Ws(_)
        | TransportConfig::Grpc(_)
        | TransportConfig::Http(_)
        | TransportConfig::HttpUpgrade(_)
        | TransportConfig::XHttp(_) => true,
        TransportConfig::Kcp(cfg) => kcp_supported(cfg, path),
        TransportConfig::Quic => false,
    }
}

/// mKCP row: only the settings the native dial actually reads.
///
/// `transport::kcp::connect` reads `mtu` + `tti` and nothing else (spec §4.5
/// defaults otherwise). Capacity/congestion/buffer fields are local pacing
/// knobs — ignoring them still interoperates. Two fields are not knobs but
/// WIRE FORMAT, so ignoring them frames every datagram differently than the
/// server expects: the packets are dropped and the dial hangs instead of
/// failing loudly, the worst failure shape there is.
///
/// - `seed`: mKCP's global obfuscation key. Clash configs carry it in
///   `KcpConfig::seed`; share links carry it in the protocol row's `path`
///   (`vless://…?type=kcp&path=<seed>`), which is why `path` is checked here
///   too. Native has no obfuscator at all.
/// - `header_type`: the packet camouflage header (`srtp`, `utp`,
///   `wechat-video`, `dtls`, `wireguard`, `dns`). Only the default `none`
///   is a bare mKCP datagram.
fn kcp_supported(cfg: &KcpConfig, path: Option<&str>) -> bool {
    let seeded =
        cfg.seed.as_deref().is_some_and(|s| !s.is_empty()) || path.is_some_and(|s| !s.is_empty());
    let camouflaged = cfg
        .header_type
        .as_deref()
        .is_some_and(|h| !(h.is_empty() || h == "none"));
    !seeded && !camouflaged
}

/// True when the row's TLS fingerprint id is one native parses.
///
/// `security::wrap` feeds `fp` straight to [`parse_fingerprint_id`] in both
/// arms (plain TLS and REALITY's default provisioner), and an id it does not
/// know is a fatal `NativeError::Config` — the dial fails where xray's uTLS
/// would have connected. Gating on the SAME parser is what keeps the two
/// lists from drifting: accepted ids are exactly `chrome`,
/// `chrome-randomized`, `firefox`, `safari`, `random`, so xray-only ids
/// (`randomized`, `ios`, `android`, `edge`, `360`, `qq`, …) defer to the
/// subprocess.
///
/// No `fp` at all is supported: plain TLS then uses the engine default and
/// REALITY the fixed chrome spec.
fn security_supported(security: &SecurityConfig) -> bool {
    security
        .fp()
        .is_none_or(|fp| parse_fingerprint_id(fp).is_ok())
}

/// VLESS row: no deferred account encryption or flow, a fingerprint native
/// parses, implemented transport.
///
/// Any non-empty `encryption` other than `"none"` defers — in particular
/// `mlkem768x25519plus.*`, whose native handshake diverges from real xray
/// (fails where xray works), so it must never be Auto-selected. Flows are
/// limited to the vision pair native encodes (`connect_vision`); any other
/// non-empty flow is a `NotImplemented` guard.
///
/// Both vision flows stay supported even though a native session cannot
/// carry their UDP leg (`xtls-rprx-vision-udp443` forces XUDP, and the proxy
/// leg never sets `params.udp` — see [`supported`]): vision over REALITY is
/// the most common native shape and its TCP path is fully implemented, so
/// deferring it would trade a live fast path for a UDP leg that is dead on
/// both sides of the decision.
fn vless_supported(cfg: &VlessConfig) -> bool {
    if let Some(enc) = cfg.encryption.as_deref()
        && !enc.is_empty()
        && enc != "none"
    {
        return false;
    }
    if let Some(flow) = cfg.flow.as_deref()
        && !(flow.is_empty() || flow == "xtls-rprx-vision" || flow == "xtls-rprx-vision-udp443")
    {
        return false;
    }
    security_supported(&cfg.security) && transport_supported(&cfg.transport, cfg.path.as_deref())
}

/// `VMess` row: modern AEAD payload security only, a fingerprint native
/// parses, implemented transport.
///
/// Native maps `security.enc` to the header security byte
/// (`protocol::vmess::security_byte`): absent/`auto`/`aes-128-gcm`/
/// `chacha20-poly1305` only. Legacy `none`/`zero`/`aes-128-cfb`/bare
/// `chacha20` are xray-only (rejected server-side by xray 26.x too, but
/// native has no arm at all). A non-zero `alter_id` selects the legacy
/// pre-AEAD session scheme native never implemented.
fn vmess_supported(cfg: &VmessConfig) -> bool {
    if let Some(enc) = cfg.security.enc.as_deref()
        && !(enc.is_empty() || enc == "auto" || enc == "aes-128-gcm" || enc == "chacha20-poly1305")
    {
        return false;
    }
    if let Some(aid) = cfg.alter_id.as_deref()
        && !(aid.is_empty() || aid == "0")
    {
        return false;
    }
    security_supported(&cfg.security) && transport_supported(&cfg.transport, cfg.path.as_deref())
}

/// Trojan row: a fingerprint native parses, implemented transport.
///
/// Trojan has no account-level encryption/flow variants in the typed config;
/// security is none/tls/reality, all of which `security::wrap` implements —
/// for the ids [`security_supported`] accepts.
fn trojan_supported(cfg: &TrojanConfig) -> bool {
    security_supported(&cfg.security) && transport_supported(&cfg.transport, cfg.path.as_deref())
}

/// Hysteria2 row: always servable — and it MUST stay that way.
///
/// It is a self-contained QUIC dial (`protocol::hysteria2`, quinn's internal
/// rustls), so there is no transport matrix and no fingerprint gate: the TLS
/// side comes from `transport::quic::quic_tls_config`, which reads only
/// `insecure` and never looks at `fp`, so an xray-only fingerprint id on a
/// hysteria2 row is inert rather than fatal.
///
/// `false` here is not fatal — xray-core DOES have a hysteria2 outbound
/// (`protocol: "hysteria"`, `version: 2`; see `Hysteria2Config::inject_xray`
/// in xray-tui-proto, unit-tested there), so a downgrade would serve the
/// profile. The predicate stays `true` because nothing in the typed config
/// requests a feature native lacks:
///
/// Fields native reads: `auth`, `obfs_password` (Salamander — keyed off the
/// password alone; the `obfs` TYPE string is never read), `down`
/// (→ the `hysteria-cc-rx` auth header) and `security`'s `insecure`. Fields
/// native ignores: `up` (the client advertises no send cap), `hop_interval`
/// (no port hopping — the dial pins to the endpoint's base port) and
/// `pin_sha256` (the QUIC dial carries no SPKI pin; only the plain-TLS path
/// honours one). All of those are refinements: ignoring them still
/// interoperates, so none of them gates. If a future config field DOES
/// require xray semantics native lacks, return `false` for it — the Auto
/// downgrade lands on xray-core, which can build the row.
const fn hysteria2_supported(_cfg: &Hysteria2Config) -> bool {
    true
}

/// Shadowsocks row (both kinds share this payload type): a native method
/// family matching `kind`, no SIP003 plugin, a key the connect path can
/// derive, a fingerprint native parses.
///
/// SS has no transport dimension — `SsConfig` carries no transport field, so
/// the row always rides a plain TCP dial, plus optional `security` applied by
/// the chain. The UDP dial-end's own refusals (a non-empty `security` on the
/// UDP path, chaining) live in `ss::udp::connect_udp`/`chain`, not here: this
/// predicate stays TCP-truthful, exactly as [`supported`] documents.
///
/// Every refusal below is a dead native dial the subprocess would have
/// served:
///
/// - [`resolve_method`] is the connect path's ONE config entry point
///   (`ss::connect` calls it before dispatching to a family codec), so the
///   gate and the codec cannot drift: routing through the same function
///   rejects exactly what the dial would reject — legacy stream ciphers
///   (`aes-*-cfb/ctr`, `rc4-md5`, `chacha20-ietf`, `none`, …) and unknown
///   names — and keeps those rows on sing-box. Should `resolve_method` ever
///   grow a check, this gate inherits it.
/// - Method family vs `kind`: the two kinds share `SsConfig`, and
///   `2022-blake3-*` selects the BLAKE3 schedule while everything else uses
///   HKDF-SHA1, so a mismatch means the wrong KDF, not a fallback.
/// - `plugin`/`plugin_opts`: SIP003 is unimplemented — neither `ss::connect`
///   nor the UDP carrier ever reads either field, so a plugin row would dial
///   the bare server without its obfuscation wrapper. These are the only
///   plugin fields on the typed config; the share-link and clash parsers both
///   land here.
/// - `password_key`: a malformed 2022 PSK (not base64, or the wrong length
///   for the method) is a fatal `NativeError::Config` in the connect path, so
///   refusing at gate time keeps Auto resolution on the subprocess.
/// - `security_supported`: an xray-only uTLS fingerprint id is fatal on the
///   SS TCP path too, because `security::wrap` parses it there like any
///   other TLS/REALITY row.
fn ss_supported(kind: ProtocolKind, cfg: &SsConfig) -> bool {
    let Ok(method) = resolve_method(cfg) else {
        return false;
    };
    if method.kind() != kind {
        return false;
    }
    if cfg.plugin.is_some() || cfg.plugin_opts.is_some() {
        return false;
    }
    if password_key(method, &cfg.password).is_err() {
        return false;
    }
    security_supported(&cfg.security)
}

#[cfg(test)]
mod tests {
    use super::*;
    use xray_tui_proto::proto_spec::common::{
        GrpcConfig, HttpConfig, KcpConfig, TransportConfig, WebSocketConfig,
    };
    use xray_tui_proto::proto_spec::{
        HttpUpgradeConfig, RealityOpts, SecurityConfig, TlsConfig, TlsOpts, XHttpConfig,
    };
    use xray_tui_proto::urlx::TinyText;

    fn vless_cfg() -> VlessConfig {
        VlessConfig {
            uuid: "00000000-0000-0000-0000-000000000000".into(),
            uuid_origin: None,
            security: SecurityConfig::default(),
            transport: TransportConfig::Tcp,
            encryption: None,
            flow: None,
            path: None,
            splice: None,
            remarks: None,
        }
    }

    fn vmess_cfg() -> VmessConfig {
        VmessConfig {
            uuid: "00000000-0000-0000-0000-000000000000".into(),
            security: SecurityConfig::default(),
            transport: TransportConfig::Tcp,
            alter_id: None,
            path: None,
            remarks: None,
        }
    }

    fn trojan_cfg() -> TrojanConfig {
        TrojanConfig {
            password: "secret".into(),
            security: SecurityConfig::default(),
            transport: TransportConfig::Tcp,
            path: None,
            remarks: None,
        }
    }

    fn hysteria2_cfg() -> Hysteria2Config {
        Hysteria2Config {
            auth: "secret".into(),
            security: SecurityConfig::default(),
            obfs: None,
            obfs_password: None,
            up: None,
            down: None,
            hop_interval: None,
            pin_sha256: None,
            remarks: None,
        }
    }

    /// A plain-TLS security config carrying the `fp` id.
    fn tls_fp(fp: &str) -> SecurityConfig {
        SecurityConfig {
            tls: Some(TlsConfig::Tls(TlsOpts {
                sni: Some(TinyText::from("example.com")),
                fp: Some(TinyText::from(fp)),
                ..TlsOpts::default()
            })),
            enc: None,
        }
    }

    /// A REALITY security config carrying the `fp` id (the other arm of
    /// `security::wrap` that parses it).
    fn reality_fp(fp: &str) -> SecurityConfig {
        SecurityConfig {
            tls: Some(TlsConfig::Reality(RealityOpts {
                sni: Some(TinyText::from("example.com")),
                fp: Some(TinyText::from(fp)),
                pbk: Some("cHVibGljLWtleS0zMi1ieXRlcy1iYXNlNjR1cmw".to_owned()),
                ..RealityOpts::default()
            })),
            enc: None,
        }
    }

    /// A shadowsocks row with no plugin and default (no-op) security.
    fn ss_cfg(method: &str, password: &str) -> SsConfig {
        SsConfig {
            method: method.into(),
            password: password.into(),
            security: SecurityConfig::default(),
            remarks: None,
            plugin: None,
            plugin_opts: None,
        }
    }

    /// `supported` for an SS row (both kinds share the payload type).
    fn ss_row(kind: ProtocolKind, cfg: SsConfig) -> bool {
        supported(kind, &ProtocolConfig::Ss(cfg))
    }

    /// `supported` for a vless row whose security is `security`.
    fn vless_with(security: SecurityConfig) -> bool {
        let mut cfg = vless_cfg();
        cfg.security = security;
        supported(ProtocolKind::Vless, &ProtocolConfig::Vless(cfg))
    }

    /// `supported` for a kcp-transport row of every stream protocol, with
    /// `path` on the protocol row (the share-link seed carrier).
    fn kcp_rows_supported(kcp: &KcpConfig, path: Option<&str>) -> [bool; 3] {
        let transport = TransportConfig::Kcp(kcp.clone());
        let path = path.map(TinyText::from);
        [
            supported(
                ProtocolKind::Vless,
                &ProtocolConfig::Vless(VlessConfig {
                    transport: transport.clone(),
                    path: path.clone(),
                    ..vless_cfg()
                }),
            ),
            supported(
                ProtocolKind::Vmess,
                &ProtocolConfig::Vmess(VmessConfig {
                    transport: transport.clone(),
                    path: path.clone(),
                    ..vmess_cfg()
                }),
            ),
            supported(
                ProtocolKind::Trojan,
                &ProtocolConfig::Trojan(TrojanConfig {
                    transport,
                    path,
                    ..trojan_cfg()
                }),
            ),
        ]
    }

    #[test]
    fn native_kinds_exact_order() {
        assert_eq!(
            NATIVE_KINDS,
            &[
                ProtocolKind::Vless,
                ProtocolKind::Vmess,
                ProtocolKind::Trojan,
                ProtocolKind::Hysteria2,
                ProtocolKind::Shadowsocks,
                ProtocolKind::Shadowsocks2022,
            ]
        );
    }

    #[test]
    fn plain_defaults_supported() {
        assert!(supported(
            ProtocolKind::Vless,
            &ProtocolConfig::Vless(vless_cfg())
        ));
        assert!(supported(
            ProtocolKind::Vmess,
            &ProtocolConfig::Vmess(vmess_cfg())
        ));
        assert!(supported(
            ProtocolKind::Trojan,
            &ProtocolConfig::Trojan(trojan_cfg())
        ));
        assert!(supported(
            ProtocolKind::Hysteria2,
            &ProtocolConfig::Hysteria2(hysteria2_cfg())
        ));
    }

    #[test]
    fn pq_enc_vless_deferred() {
        let mut cfg = vless_cfg();
        cfg.encryption = Some(TinyText::from(
            "mlkem768x25519plus.native.1rtt.100-35-70.0-0-0.a2V5",
        ));
        assert!(!supported(ProtocolKind::Vless, &ProtocolConfig::Vless(cfg)));
    }

    #[test]
    fn unknown_vless_encryption_deferred() {
        let mut cfg = vless_cfg();
        cfg.encryption = Some(TinyText::from("some-future-scheme"));
        assert!(!supported(ProtocolKind::Vless, &ProtocolConfig::Vless(cfg)));
    }

    #[test]
    fn unknown_vless_flow_deferred() {
        let mut cfg = vless_cfg();
        cfg.flow = Some(TinyText::from("xtls-rprx-splice"));
        assert!(!supported(ProtocolKind::Vless, &ProtocolConfig::Vless(cfg)));
    }

    #[test]
    fn vision_flows_supported() {
        for flow in ["xtls-rprx-vision", "xtls-rprx-vision-udp443"] {
            let mut cfg = vless_cfg();
            cfg.flow = Some(TinyText::from(flow));
            assert!(
                supported(ProtocolKind::Vless, &ProtocolConfig::Vless(cfg)),
                "{flow}"
            );
        }
    }

    #[test]
    fn legacy_vmess_ciphers_deferred() {
        for enc in ["none", "zero", "aes-128-cfb", "chacha20"] {
            let mut cfg = vmess_cfg();
            cfg.security.enc = Some(TinyText::from(enc));
            assert!(
                !supported(ProtocolKind::Vmess, &ProtocolConfig::Vmess(cfg)),
                "{enc}"
            );
        }
    }

    #[test]
    fn modern_vmess_ciphers_supported() {
        for enc in ["auto", "aes-128-gcm", "chacha20-poly1305"] {
            let mut cfg = vmess_cfg();
            cfg.security.enc = Some(TinyText::from(enc));
            assert!(
                supported(ProtocolKind::Vmess, &ProtocolConfig::Vmess(cfg)),
                "{enc}"
            );
        }
    }

    #[test]
    fn nonzero_alter_id_deferred() {
        let mut cfg = vmess_cfg();
        cfg.alter_id = Some(TinyText::from("4"));
        assert!(!supported(ProtocolKind::Vmess, &ProtocolConfig::Vmess(cfg)));
    }

    #[test]
    fn bare_quic_transport_deferred() {
        let mut cfg = vless_cfg();
        cfg.transport = TransportConfig::Quic;
        assert!(!supported(ProtocolKind::Vless, &ProtocolConfig::Vless(cfg)));
    }

    #[test]
    fn kind_config_mismatch_deferred() {
        // Right kind, wrong payload — never servable.
        assert!(!supported(
            ProtocolKind::Vless,
            &ProtocolConfig::Vmess(vmess_cfg())
        ));
    }

    #[test]
    fn non_native_kinds_unsupported() {
        let non_native = [
            ProtocolKind::Socks,
            ProtocolKind::Http,
            ProtocolKind::WireGuard,
            ProtocolKind::DokodemoDoor,
            ProtocolKind::Freedom,
            ProtocolKind::Blackhole,
            ProtocolKind::Dns,
            ProtocolKind::Loopback,
            ProtocolKind::Custom,
            ProtocolKind::Tuic,
            ProtocolKind::Hysteria,
            ProtocolKind::Naive,
            ProtocolKind::AnyTls,
            ProtocolKind::ShadowTls,
            ProtocolKind::Tor,
            ProtocolKind::Ssh,
            ProtocolKind::Tailscale,
            ProtocolKind::ShadowsocksR,
            ProtocolKind::Redirect,
            ProtocolKind::TProxy,
            ProtocolKind::Mixed,
        ];
        assert_eq!(non_native.len(), 21);
        let fallback = ProtocolConfig::Vless(vless_cfg());
        for kind in non_native {
            assert!(!kind_supported(kind), "{kind:?}");
            assert!(!supported(kind, &fallback), "{kind:?}");
        }
    }

    #[test]
    fn xray_only_fingerprints_deferred() {
        // ids xray's uTLS accepts but `parse_fingerprint_id` refuses: native
        // would fail the dial with `NativeError::Config` where the
        // subprocess connects. The empty value is refused by the same
        // parser, so it defers too.
        for fp in [
            "randomized",
            "ios",
            "android",
            "edge",
            "360",
            "qq",
            "chrome-130",
            "",
        ] {
            assert!(!vless_with(tls_fp(fp)), "vless tls fp={fp:?}");
            assert!(!vless_with(reality_fp(fp)), "vless reality fp={fp:?}");

            let mut vmess = vmess_cfg();
            vmess.security = tls_fp(fp);
            assert!(
                !supported(ProtocolKind::Vmess, &ProtocolConfig::Vmess(vmess)),
                "vmess fp={fp:?}"
            );

            let mut trojan = trojan_cfg();
            trojan.security = tls_fp(fp);
            assert!(
                !supported(ProtocolKind::Trojan, &ProtocolConfig::Trojan(trojan)),
                "trojan fp={fp:?}"
            );
        }
    }

    #[test]
    fn native_fingerprints_supported() {
        // Exactly the ids the parser accepts — the gate must not narrow it.
        for fp in ["chrome", "chrome-randomized", "firefox", "safari", "random"] {
            assert!(vless_with(tls_fp(fp)), "vless tls fp={fp:?}");
            assert!(vless_with(reality_fp(fp)), "vless reality fp={fp:?}");

            let mut vmess = vmess_cfg();
            vmess.security = tls_fp(fp);
            assert!(
                supported(ProtocolKind::Vmess, &ProtocolConfig::Vmess(vmess)),
                "vmess fp={fp:?}"
            );

            let mut trojan = trojan_cfg();
            trojan.security = tls_fp(fp);
            assert!(
                supported(ProtocolKind::Trojan, &ProtocolConfig::Trojan(trojan)),
                "trojan fp={fp:?}"
            );
        }
    }

    #[test]
    fn tls_without_fingerprint_supported() {
        // No `fp`: plain TLS uses the engine default, REALITY the fixed
        // chrome spec — nothing is parsed, nothing defers.
        assert!(vless_with(SecurityConfig {
            tls: Some(TlsConfig::Tls(TlsOpts {
                sni: Some(TinyText::from("example.com")),
                ..TlsOpts::default()
            })),
            enc: None,
        }));
        assert!(vless_with(SecurityConfig {
            tls: Some(TlsConfig::Reality(RealityOpts {
                sni: Some(TinyText::from("example.com")),
                pbk: Some("cHVibGljLWtleS0zMi1ieXRlcy1iYXNlNjR1cmw".to_owned()),
                ..RealityOpts::default()
            })),
            enc: None,
        }));
    }

    #[test]
    fn hysteria2_ignores_fingerprint_and_stays_supported() {
        // quinn's internal rustls never reads `fp` (`quic_tls_config` looks
        // at `insecure` only), so an xray-only id is inert here — the test
        // pins that the fingerprint gate does not over-reach onto the QUIC
        // dial. (A `false` here WOULD downgrade to xray-core, which builds
        // hysteria2 outbounds — the gate is simply never triggered.)
        let mut cfg = hysteria2_cfg();
        cfg.security = tls_fp("ios");
        cfg.obfs = Some(TinyText::from("salamander"));
        cfg.obfs_password = Some(TinyText::from("obfs-shared-secret"));
        cfg.up = Some(TinyText::from("100 mbps"));
        cfg.down = Some(TinyText::from("200 mbps"));
        cfg.hop_interval = Some(30);
        cfg.pin_sha256 = Some(TinyText::from("YmFzZTY0LXBpbg"));
        assert!(supported(
            ProtocolKind::Hysteria2,
            &ProtocolConfig::Hysteria2(cfg)
        ));
    }

    #[test]
    fn plain_kcp_supported() {
        // `mtu`/`tti` are the two fields the native dial reads; the pacing
        // knobs it ignores do not change the wire format, and an explicit
        // `none`/empty header is the bare-datagram default.
        for kcp in [
            KcpConfig::default(),
            KcpConfig {
                mtu: Some(1350),
                tti: Some(50),
                ..KcpConfig::default()
            },
            KcpConfig {
                header_type: Some(TinyText::from("none")),
                ..KcpConfig::default()
            },
            KcpConfig {
                header_type: Some(TinyText::from("")),
                seed: Some(TinyText::from("")),
                ..KcpConfig::default()
            },
            KcpConfig {
                uplink_capacity: Some(50),
                downlink_capacity: Some(100),
                congestion: Some(true),
                read_buffer: Some(2),
                write_buffer: Some(2),
                ..KcpConfig::default()
            },
        ] {
            assert_eq!(kcp_rows_supported(&kcp, None), [true; 3], "{kcp:?}");
        }
    }

    #[test]
    fn kcp_seed_deferred() {
        // Clash carrier: `mkcp-opts.seed` → `KcpConfig::seed`. Native has no
        // obfuscator, so it would frame every datagram unmasked and the
        // server would silently drop them.
        let kcp = KcpConfig {
            seed: Some(TinyText::from("hunter2")),
            ..KcpConfig::default()
        };
        assert_eq!(kcp_rows_supported(&kcp, None), [false; 3]);
    }

    #[test]
    fn kcp_share_link_seed_path_deferred() {
        // Share-link carrier: `?type=kcp&path=<seed>` lands in the protocol
        // row's `path`, never in `KcpConfig`.
        assert_eq!(
            kcp_rows_supported(&KcpConfig::default(), Some("hunter2")),
            [false; 3]
        );
        // An empty path is no seed at all.
        assert_eq!(
            kcp_rows_supported(&KcpConfig::default(), Some("")),
            [true; 3]
        );
    }

    #[test]
    fn kcp_header_type_deferred() {
        // Packet camouflage: native writes bare mKCP datagrams, so any
        // header type frames the packets differently than the server reads
        // them — a silent drop + hang, not an error.
        for header in ["srtp", "utp", "wechat-video", "dtls", "wireguard", "dns"] {
            let kcp = KcpConfig {
                header_type: Some(TinyText::from(header)),
                ..KcpConfig::default()
            };
            assert_eq!(kcp_rows_supported(&kcp, None), [false; 3], "{header}");
        }
    }

    #[test]
    fn non_kcp_path_is_not_a_seed() {
        // The `path` gate is kcp-only: a ws/grpc row's path is its transport
        // path, not an obfuscation seed.
        for transport in [
            TransportConfig::Ws(WebSocketConfig::default()),
            TransportConfig::Grpc(GrpcConfig::default()),
            TransportConfig::HttpUpgrade(HttpUpgradeConfig::default()),
        ] {
            let mut cfg = vless_cfg();
            cfg.transport = transport.clone();
            cfg.path = Some(TinyText::from("/ws"));
            assert!(
                supported(ProtocolKind::Vless, &ProtocolConfig::Vless(cfg)),
                "{transport:?}"
            );
        }
    }

    #[test]
    fn transport_positive_match_known_set() {
        // The dispatch set of `transport::{connect, upgrade}` — bare quic has
        // no arm. `transport_supported` matches exhaustively, so a variant
        // added to `TransportConfig` fails to compile there instead of
        // silently inheriting `true`; this pins the known answers.
        for (transport, want) in [
            (TransportConfig::Tcp, true),
            (TransportConfig::Ws(WebSocketConfig::default()), true),
            (TransportConfig::Grpc(GrpcConfig::default()), true),
            (TransportConfig::Http(HttpConfig::default()), true),
            (
                TransportConfig::HttpUpgrade(HttpUpgradeConfig::default()),
                true,
            ),
            (TransportConfig::XHttp(XHttpConfig::default()), true),
            (TransportConfig::Kcp(KcpConfig::default()), true),
            (TransportConfig::Quic, false),
        ] {
            assert_eq!(
                transport_supported(&transport, None),
                want,
                "{transport:?} direct"
            );
            let mut cfg = vless_cfg();
            cfg.transport = transport.clone();
            assert_eq!(
                supported(ProtocolKind::Vless, &ProtocolConfig::Vless(cfg)),
                want,
                "{transport:?} via supported"
            );
        }
    }

    #[test]
    fn ss_kinds_are_native_and_legacy_methods_defer() {
        assert!(kind_supported(ProtocolKind::Shadowsocks));
        assert!(kind_supported(ProtocolKind::Shadowsocks2022));
        assert!(ss_row(
            ProtocolKind::Shadowsocks,
            ss_cfg("aes-128-gcm", "pw")
        ));
        assert!(ss_row(
            ProtocolKind::Shadowsocks2022,
            ss_cfg(
                "2022-blake3-aes-256-gcm",
                "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
            )
        ));
        // legacy + unknown methods stay on sing-box
        for method in ["aes-256-cfb", "none", "rc4-md5", "chacha20-ietf"] {
            assert!(
                !ss_row(ProtocolKind::Shadowsocks, ss_cfg(method, "pw")),
                "{method}"
            );
        }
    }

    #[test]
    fn ss_kind_and_method_family_must_agree() {
        // A 2022 method on the classic kind (and vice versa) would pick the
        // wrong KDF, so mismatched rows defer.
        assert!(!ss_row(
            ProtocolKind::Shadowsocks,
            ss_cfg("2022-blake3-aes-256-gcm", "AAAAAAAAAAAAAAAAAAAAAA==")
        ));
        assert!(!ss_row(
            ProtocolKind::Shadowsocks2022,
            ss_cfg("aes-128-gcm", "pw")
        ));
    }

    #[test]
    fn ss_plugin_rows_defer() {
        // SIP003: neither `ss::connect` nor the UDP carrier reads the plugin
        // fields, so a plugin row must never reach the native dial.
        let mut cfg = ss_cfg("aes-128-gcm", "pw");
        cfg.plugin = Some(TinyText::from("obfs-local"));
        assert!(!ss_row(ProtocolKind::Shadowsocks, cfg));

        let mut cfg = ss_cfg("aes-128-gcm", "pw");
        cfg.plugin_opts = Some(std::collections::HashMap::from([(
            "obfs".to_owned(),
            "http".to_owned(),
        )]));
        assert!(!ss_row(ProtocolKind::Shadowsocks, cfg));
    }

    #[test]
    fn ss2022_password_length_is_validated_at_gate_time() {
        // A wrong-length or non-base64 PSK is a config error in
        // `password_key`, not a fallback: refusing here keeps Auto
        // resolution on the subprocess instead of a dead native dial.
        assert!(!ss_row(
            ProtocolKind::Shadowsocks2022,
            ss_cfg("2022-blake3-aes-128-gcm", "c2hvcnQ=")
        ));
        assert!(!ss_row(
            ProtocolKind::Shadowsocks2022,
            ss_cfg("2022-blake3-aes-128-gcm", "!!!")
        ));
        // The classic KDF accepts any password, so an SS AEAD row with an
        // arbitrary password stays supported.
        assert!(ss_row(
            ProtocolKind::Shadowsocks,
            ss_cfg("chacha20-ietf-poly1305", "")
        ));
    }

    #[test]
    fn ss_xray_only_fingerprint_defers() {
        // `security::wrap` parses `fp` on the SS TCP path too, so an
        // xray-only id is fatal there exactly as on vless/vmess/trojan.
        let mut cfg = ss_cfg("aes-128-gcm", "pw");
        cfg.security = tls_fp("ios");
        assert!(!ss_row(ProtocolKind::Shadowsocks, cfg));

        let mut cfg = ss_cfg(
            "2022-blake3-aes-256-gcm",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        );
        cfg.security = reality_fp("edge");
        assert!(!ss_row(ProtocolKind::Shadowsocks2022, cfg));
    }

    #[test]
    fn ss_native_fingerprint_and_plain_tls_supported() {
        for security in [tls_fp("chrome"), reality_fp("firefox")] {
            let mut cfg = ss_cfg("aes-192-gcm", "pw");
            cfg.security = security.clone();
            assert!(ss_row(ProtocolKind::Shadowsocks, cfg), "{security:?}");
        }
    }
}
