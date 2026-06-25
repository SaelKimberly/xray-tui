use xray_tui_core::protocol::Protocol;

/// A single input field in a protocol form.
#[derive(Debug, Clone)]
pub enum FormFieldType {
    Text,
    Number,
    Select(&'static [&'static str]),
    Boolean,
    Password,
}

#[derive(Debug, Clone)]
pub struct FormField {
    pub key: &'static str,
    pub label: &'static str,
    pub field_type: FormFieldType,
    pub default: &'static str,
    pub required: bool,
    pub section: FieldSection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldSection {
    /// Stored directly as Profile fields (remarks, address, port, etc.)
    Common,
    /// Serialized to `Profile.stream_settings` JSON blob
    StreamSetting,
    /// Serialized to `Profile.protocol_settings` JSON blob
    ProtocolSetting,
}

/// Returns the form fields for a given protocol.
#[must_use]
pub fn form_fields_for(protocol: Protocol) -> Vec<FormField> {
    let mut fields = common_fields();
    fields.extend(protocol_specific_fields(protocol));
    fields
}

fn common_fields() -> Vec<FormField> {
    vec![
        FormField {
            key: "remarks",
            label: "Remarks",
            field_type: FormFieldType::Text,
            default: "",
            required: false,
            section: FieldSection::Common,
        },
        FormField {
            key: "address",
            label: "Address",
            field_type: FormFieldType::Text,
            default: "",
            required: true,
            section: FieldSection::Common,
        },
        FormField {
            key: "port",
            label: "Port",
            field_type: FormFieldType::Number,
            default: "443",
            required: true,
            section: FieldSection::Common,
        },
        FormField {
            key: "core_type",
            label: "Core",
            field_type: FormFieldType::Select(&["auto", "xray", "sing-box"]),
            default: "auto",
            required: false,
            section: FieldSection::Common,
        },
    ]
}

fn protocol_specific_fields(protocol: Protocol) -> Vec<FormField> {
    match protocol {
        Protocol::Vmess => vec![
            field(
                "user_id",
                "User ID",
                FormFieldType::Text,
                "",
                true,
                FieldSection::Common,
            ),
            field(
                "security",
                "Encryption",
                FormFieldType::Select(&["auto", "aes-128-gcm", "chacha20-poly1305", "none"]),
                "auto",
                false,
                FieldSection::Common,
            ),
            field(
                "network",
                "Network",
                FormFieldType::Select(&["tcp", "kcp", "ws", "http", "h2", "grpc", "quic"]),
                "tcp",
                false,
                FieldSection::Common,
            ),
            field(
                "tls.enable",
                "TLS",
                FormFieldType::Boolean,
                "false",
                false,
                FieldSection::StreamSetting,
            ),
            field(
                "reality.show",
                "Reality",
                FormFieldType::Boolean,
                "false",
                false,
                FieldSection::StreamSetting,
            ),
            field(
                "ws.path",
                "WebSocket Path",
                FormFieldType::Text,
                "",
                false,
                FieldSection::StreamSetting,
            ),
            field(
                "ws.host",
                "WebSocket Host",
                FormFieldType::Text,
                "",
                false,
                FieldSection::StreamSetting,
            ),
            field(
                "grpc.serviceName",
                "gRPC Service",
                FormFieldType::Text,
                "",
                false,
                FieldSection::StreamSetting,
            ),
            field(
                "tcp.headerType",
                "TCP Header",
                FormFieldType::Select(&["none", "http"]),
                "none",
                false,
                FieldSection::StreamSetting,
            ),
        ],
        Protocol::Vless => vec![
            field(
                "user_id",
                "User ID",
                FormFieldType::Text,
                "",
                true,
                FieldSection::Common,
            ),
            field(
                "flow",
                "Flow",
                FormFieldType::Select(&["", "xtls-rprx-vision", "xtls-rprx-vision-udp443"]),
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "network",
                "Network",
                FormFieldType::Select(&["tcp", "kcp", "ws", "http", "h2", "grpc", "quic"]),
                "tcp",
                false,
                FieldSection::Common,
            ),
            field(
                "tls.enable",
                "TLS",
                FormFieldType::Boolean,
                "false",
                false,
                FieldSection::StreamSetting,
            ),
            field(
                "ws.path",
                "WebSocket Path",
                FormFieldType::Text,
                "",
                false,
                FieldSection::StreamSetting,
            ),
            field(
                "ws.host",
                "WebSocket Host",
                FormFieldType::Text,
                "",
                false,
                FieldSection::StreamSetting,
            ),
            field(
                "grpc.serviceName",
                "gRPC Service",
                FormFieldType::Text,
                "",
                false,
                FieldSection::StreamSetting,
            ),
        ],
        Protocol::Shadowsocks | Protocol::Shadowsocks2022 => vec![
            field(
                "method",
                "Method",
                FormFieldType::Select(match protocol {
                    Protocol::Shadowsocks => &[
                        "aes-256-gcm",
                        "aes-128-gcm",
                        "chacha20-ietf-poly1305",
                        "xchacha20-ietf-poly1305",
                        "none",
                    ],
                    Protocol::Shadowsocks2022 => {
                        &["2022-blake3-aes-128-gcm", "2022-blake3-aes-256-gcm", "none"]
                    }
                    _ => unreachable!(),
                }),
                "aes-256-gcm",
                true,
                FieldSection::ProtocolSetting,
            ),
            field(
                "password",
                "Password",
                FormFieldType::Password,
                "",
                true,
                FieldSection::Common,
            ),
            field(
                "plugin",
                "Plugin",
                FormFieldType::Text,
                "",
                false,
                FieldSection::StreamSetting,
            ),
            field(
                "plugin_opts",
                "Plugin Opts",
                FormFieldType::Text,
                "",
                false,
                FieldSection::StreamSetting,
            ),
        ],
        Protocol::Socks => vec![
            field(
                "username",
                "Username",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "password",
                "Password",
                FormFieldType::Password,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "udp",
                "UDP",
                FormFieldType::Boolean,
                "false",
                false,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::Http => vec![
            field(
                "username",
                "Username",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "password",
                "Password",
                FormFieldType::Password,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "tls",
                "TLS",
                FormFieldType::Boolean,
                "false",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "path",
                "Path",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::Trojan => vec![
            field(
                "password",
                "Password",
                FormFieldType::Password,
                "",
                true,
                FieldSection::Common,
            ),
            field(
                "flow",
                "Flow",
                FormFieldType::Select(&["", "xtls-rprx-vision"]),
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "sni",
                "SNI",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "alpn",
                "ALPN",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "fingerprint",
                "Fingerprint",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "allow_insecure",
                "Allow Insecure",
                FormFieldType::Boolean,
                "false",
                false,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::WireGuard => vec![
            field(
                "private_key",
                "Private Key",
                FormFieldType::Password,
                "",
                true,
                FieldSection::ProtocolSetting,
            ),
            field(
                "public_key",
                "Public Key",
                FormFieldType::Text,
                "",
                true,
                FieldSection::ProtocolSetting,
            ),
            field(
                "endpoint",
                "Endpoint",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "allowed_ips",
                "Allowed IPs",
                FormFieldType::Text,
                "0.0.0.0/0",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "dns",
                "DNS",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "mtu",
                "MTU",
                FormFieldType::Number,
                "1420",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "workers",
                "Workers",
                FormFieldType::Number,
                "4",
                false,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::Hysteria2 => vec![
            field(
                "password",
                "Password",
                FormFieldType::Password,
                "",
                false,
                FieldSection::Common,
            ),
            field(
                "ports",
                "Ports",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "hop",
                "Hop Interval",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "obfs",
                "Obfs",
                FormFieldType::Select(&["", "salamander", "other"]),
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "obfs_password",
                "Obfs Password",
                FormFieldType::Password,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "sni",
                "SNI",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "insecure",
                "Insecure",
                FormFieldType::Boolean,
                "false",
                false,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::Hysteria => vec![
            field(
                "protocol",
                "Protocol",
                FormFieldType::Select(&["udp", "faketcp", "wechat-video"]),
                "udp",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "auth",
                "Auth",
                FormFieldType::Password,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "obfs",
                "Obfs",
                FormFieldType::Select(&["", "salamander", "other"]),
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "up_mbps",
                "Upload Mbps",
                FormFieldType::Number,
                "100",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "down_mbps",
                "Download Mbps",
                FormFieldType::Number,
                "100",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "sni",
                "SNI",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "insecure",
                "Insecure",
                FormFieldType::Boolean,
                "false",
                false,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::Tuic => vec![
            field(
                "uuid",
                "UUID",
                FormFieldType::Text,
                "",
                true,
                FieldSection::Common,
            ),
            field(
                "password",
                "Password",
                FormFieldType::Password,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "congestion_control",
                "Congestion Ctrl",
                FormFieldType::Select(&["bbr", "cubic", "new_reno"]),
                "bbr",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "udp_relay_mode",
                "UDP Relay",
                FormFieldType::Select(&["native", "quic"]),
                "native",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "sni",
                "SNI",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "alpn",
                "ALPN",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "insecure",
                "Insecure",
                FormFieldType::Boolean,
                "false",
                false,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::Naive => vec![
            field(
                "user",
                "User",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "password",
                "Password",
                FormFieldType::Password,
                "",
                true,
                FieldSection::ProtocolSetting,
            ),
            field(
                "extra_headers",
                "Extra Headers",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "padding",
                "Padding",
                FormFieldType::Boolean,
                "true",
                false,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::AnyTls => vec![
            field(
                "password",
                "Password",
                FormFieldType::Password,
                "",
                true,
                FieldSection::ProtocolSetting,
            ),
            field(
                "sni",
                "SNI",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "alpn",
                "ALPN",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "insecure",
                "Insecure",
                FormFieldType::Boolean,
                "false",
                false,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::ShadowTls => vec![
            field(
                "password",
                "Password",
                FormFieldType::Password,
                "",
                true,
                FieldSection::ProtocolSetting,
            ),
            field(
                "version",
                "Version",
                FormFieldType::Number,
                "3",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "sni",
                "SNI",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::Tor => vec![
            field(
                "socks_port",
                "SOCKS Port",
                FormFieldType::Number,
                "9050",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "control_port",
                "Control Port",
                FormFieldType::Number,
                "9051",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "control_password",
                "Control Password",
                FormFieldType::Password,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "data_dir",
                "Data Dir",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::Ssh => vec![
            field(
                "host",
                "SSH Host",
                FormFieldType::Text,
                "",
                true,
                FieldSection::ProtocolSetting,
            ),
            field(
                "ssh_port",
                "SSH Port",
                FormFieldType::Number,
                "22",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "username",
                "Username",
                FormFieldType::Text,
                "",
                true,
                FieldSection::ProtocolSetting,
            ),
            field(
                "private_key",
                "Private Key",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "auth_method",
                "Auth Method",
                FormFieldType::Select(&["password", "key"]),
                "password",
                false,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::Tailscale => vec![
            field(
                "auth_key",
                "Auth Key",
                FormFieldType::Password,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "control_url",
                "Control URL",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "ephemeral",
                "Ephemeral",
                FormFieldType::Boolean,
                "false",
                false,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::ShadowsocksR => vec![
            field(
                "method",
                "Method",
                FormFieldType::Select(&[
                    "aes-256-cfb",
                    "aes-128-cfb",
                    "rc4-md5",
                    "chacha20",
                    "none",
                ]),
                "aes-256-cfb",
                true,
                FieldSection::ProtocolSetting,
            ),
            field(
                "password",
                "Password",
                FormFieldType::Password,
                "",
                true,
                FieldSection::Common,
            ),
            field(
                "obfs",
                "Obfs",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "obfs_param",
                "Obfs Param",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "protocol",
                "Protocol",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "protocol_param",
                "Protocol Param",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::DokodemoDoor => vec![
            field(
                "doko_address",
                "Target Address",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "doko_port",
                "Target Port",
                FormFieldType::Number,
                "0",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "network",
                "Network",
                FormFieldType::Select(&["tcp", "udp", "tcp,udp"]),
                "tcp",
                false,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::Freedom => vec![
            field(
                "domain_strategy",
                "Domain Strategy",
                FormFieldType::Select(&["AsIs", "UseIP", "UseIPv4", "UseIPv6"]),
                "AsIs",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "redirect",
                "Redirect",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::Blackhole => vec![field(
            "response_type",
            "Response",
            FormFieldType::Select(&["none", "http"]),
            "none",
            false,
            FieldSection::ProtocolSetting,
        )],
        Protocol::Loopback => vec![field(
            "proxy_tag",
            "Proxy Tag",
            FormFieldType::Text,
            "",
            false,
            FieldSection::ProtocolSetting,
        )],
        Protocol::Dns => vec![
            field(
                "network",
                "Network",
                FormFieldType::Select(&["tcp", "udp"]),
                "tcp",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "dns_address",
                "Address",
                FormFieldType::Text,
                "",
                true,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::Redirect => vec![
            field(
                "redirect_address",
                "Address",
                FormFieldType::Text,
                "",
                false,
                FieldSection::ProtocolSetting,
            ),
            field(
                "redirect_port",
                "Port",
                FormFieldType::Number,
                "0",
                false,
                FieldSection::ProtocolSetting,
            ),
        ],
        Protocol::Custom => vec![field(
            "config_json",
            "Config JSON",
            FormFieldType::Text,
            "",
            true,
            FieldSection::ProtocolSetting,
        )],
        Protocol::TProxy | Protocol::Mixed => {
            // Inbound-only protocols — no outbound form
            vec![]
        }
    }
}

const fn field(
    key: &'static str,
    label: &'static str,
    field_type: FormFieldType,
    default: &'static str,
    required: bool,
    section: FieldSection,
) -> FormField {
    FormField {
        key,
        label,
        field_type,
        default,
        required,
        section,
    }
}

/// URL scheme prefix for each protocol's share link.
#[must_use]
pub const fn url_scheme_for(protocol: Protocol) -> &'static str {
    match protocol {
        Protocol::Vmess => "vmess://",
        Protocol::Vless => "vless://",
        Protocol::Shadowsocks | Protocol::Shadowsocks2022 => "ss://",
        Protocol::Trojan => "trojan://",
        Protocol::Socks => "socks://",
        Protocol::Hysteria2 => "hysteria2://",
        Protocol::Hysteria => "hysteria://",
        Protocol::Tuic => "tuic://",
        Protocol::Naive => "naive+https://",
        Protocol::AnyTls => "anytls://",
        Protocol::ShadowTls => "shadowtls://",
        Protocol::WireGuard => "wireguard://",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_fields_all_protocols() {
        for proto in &[
            Protocol::Vmess,
            Protocol::Vless,
            Protocol::Shadowsocks,
            Protocol::Shadowsocks2022,
            Protocol::Socks,
            Protocol::Http,
            Protocol::Trojan,
            Protocol::WireGuard,
            Protocol::Hysteria2,
            Protocol::Hysteria,
            Protocol::Tuic,
            Protocol::Naive,
            Protocol::AnyTls,
            Protocol::ShadowTls,
            Protocol::Tor,
            Protocol::Ssh,
            Protocol::Tailscale,
            Protocol::ShadowsocksR,
            Protocol::DokodemoDoor,
            Protocol::Freedom,
            Protocol::Blackhole,
            Protocol::Loopback,
            Protocol::Dns,
            Protocol::Redirect,
            Protocol::Custom,
        ] {
            let fields = form_fields_for(*proto);
            assert!(!fields.is_empty(), "{} should have fields", proto);
            // Common fields always present
            assert!(
                fields.iter().any(|f| f.key == "remarks"),
                "{} missing remarks",
                proto
            );
            assert!(
                fields.iter().any(|f| f.key == "address"),
                "{} missing address",
                proto
            );
            assert!(
                fields.iter().any(|f| f.key == "port"),
                "{} missing port",
                proto
            );
            assert!(
                fields.iter().any(|f| f.key == "core_type"),
                "{} missing core_type",
                proto
            );
            // No duplicate keys
            let mut keys = std::collections::HashSet::new();
            for f in &fields {
                assert!(keys.insert(f.key), "duplicate key '{}' in {}", f.key, proto);
            }
        }
    }

    fn form_fields_inbound_only_have_common() {
        let tproxy = form_fields_for(Protocol::TProxy);
        assert_eq!(tproxy.len(), 4); // only common fields
        assert!(tproxy.iter().all(|f| f.section == FieldSection::Common));
        let mixed = form_fields_for(Protocol::Mixed);
        assert_eq!(mixed.len(), 4);
        assert!(mixed.iter().all(|f| f.section == FieldSection::Common));
    }

    #[test]
    fn vmess_required_fields() {
        let fields = form_fields_for(Protocol::Vmess);
        let user_id = fields.iter().find(|f| f.key == "user_id").unwrap();
        assert!(user_id.required);
        assert!(matches!(user_id.field_type, FormFieldType::Text));
        let addr = fields.iter().find(|f| f.key == "address").unwrap();
        assert!(addr.required);
    }

    #[test]
    fn url_scheme_mapping() {
        assert_eq!(url_scheme_for(Protocol::Vmess), "vmess://");
        assert_eq!(url_scheme_for(Protocol::Vless), "vless://");
        assert_eq!(url_scheme_for(Protocol::Shadowsocks), "ss://");
        assert_eq!(url_scheme_for(Protocol::Trojan), "trojan://");
        assert_eq!(url_scheme_for(Protocol::WireGuard), "wireguard://");
        assert_eq!(url_scheme_for(Protocol::Freedom), "");
    }
}
