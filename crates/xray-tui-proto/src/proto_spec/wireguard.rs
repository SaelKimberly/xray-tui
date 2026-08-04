//! `WireGuard` (`wireguard://`) URL parsing.
//!
//! # Format
//! ```text
//! wireguard://<percent-encoded(private_key)>@<host>:<port>?<query_params>#<remarks>
//! ```
//!
//! Private key is percent-encoded in userinfo. Server endpoint in host:port.
//! Interface and peer configuration in query parameters.
//!
//! # Query Parameters
//!
//! | Key          | Values              | Purpose                          | Required |
//! |--------------|---------------------|----------------------------------|----------|
//! | `address`    | CIDR notation       | Interface address (e.g., 10.0.0.2/32) | Yes |
//! | `publickey`  | base64 key          | Peer's public key                | Yes      |
//! | `presharedkey`| base64 key         | Pre-shared key                   | No       |
//! | `reserved`   | comma-separated bytes| Reserved bytes (exactly 3)      | No       |
//! | `mtu`        | integer             | Interface MTU                    | No       |
//!
//! # Edge Cases
//! - `publickey` also accepted as `public_key`
//! - `presharedkey` also accepted as `psk`
//! - `reserved` accepts both comma-separated decimals and base64-encoded bytes
//! - All query values are percent-decoded
//! - Default MTU: 1420 (Xray-core)
//! - Default port: 2408 (v2rayN parser), 51820 (`WireGuard` native)
//!
//! # References
//! - v2rayN: `WireguardFmt.cs`
//! - Xray-core: `proxy/wireguard/config.proto`
//! - sing-box: `option/wireguard.go`
//! - wireguard-go: `device/uapi.go`

use serde::{Deserialize, Serialize};

use crate::urlx::{HostSpec, RawUrlX, SchemeX, TinyText, host_serde, port_serde};

use super::ProtoIdentity;
use super::common::SecurityConfig;
use super::utils;
use super::{ParseError, ProtoSpec};
use crate::clash::{ClashProxy, ClashWireGuard};
use crate::proto_spec::ProtoSpecError;
use crate::proto_spec::common::{clash_server_to_host, host_spec_to_string};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "snake_case")]
pub struct WireguardConfig {
    pub private_key: String,
    #[serde(with = "host_serde")]
    pub host: HostSpec,
    #[serde(with = "port_serde")]
    pub port: u16,
    #[serde(default, skip_serializing_if = "SecurityConfig::is_empty")]
    pub security: SecurityConfig,
    pub address: TinyText,
    pub public_key: String,
    pub preshared_key: Option<String>,
    pub reserved: Option<TinyText>,
    pub mtu: Option<TinyText>,
    /// Persistent keepalive interval in seconds (from Clash format)
    pub persistent_keepalive: Option<u32>,
    /// DNS servers (from Clash format)
    pub dns: Option<Vec<String>>,
    /// Force remote DNS resolution (from Clash format)
    pub remote_dns_resolve: Option<bool>,
    pub remarks: Option<TinyText>,
}

impl ProtoSpec for WireguardConfig {
    /// Parse a `WireGuard` URL.
    ///
    /// Private key is percent-encoded in userinfo (may contain `+`, `/`, `=`).
    /// `address` and `publickey`/`public_key` are required; `presharedkey`/`psk`
    /// and `reserved` are optional. All query values are percent-decoded.
    fn try_parse(raw: &RawUrlX<'_>) -> Result<Self, ParseError> {
        let private_key = urlencoding::decode(raw.userinfo)
            .map_err(|_| {
                ParseError::InvalidUserInfo("invalid percent-encoding in private_key".into())
            })?
            .into_owned();

        let hostport = raw.hostport.ok_or(ParseError::MissingHost)?;
        let (parsed_host, parsed_port) = utils::parse_hostport(hostport)?;
        let parsed_port = parsed_port
            .first()
            .ok_or_else(|| ParseError::InvalidPort("empty port spec".into()))?;

        let query = utils::parse_query(raw.query);

        // address: interface address in CIDR notation (required)
        let address = utils::query_get(&query, "address")
            .ok_or_else(|| ParseError::MissingConf("address".into()))
            .map(TinyText::from)?;

        // publickey/public_key: peer's base64-encoded public key (required)
        let public_key = utils::query_get_multi(&query, &["publickey", "public_key"])
            .ok_or_else(|| ParseError::MissingConf("publickey".into()))
            .map(str::to_string)?;

        // presharedkey/psk: optional pre-shared key
        let preshared_key = utils::query_get_multi(&query, &["presharedkey", "psk"])
            .filter(|s| !s.is_empty())
            .map(str::to_string);

        // reserved: 3 bytes, comma-separated decimal or base64
        let reserved = utils::query_get(&query, "reserved").map(TinyText::from);

        // mtu: interface MTU (defaults vary: 1420 Xray, 1280 WireGuard-go)
        let mtu = utils::query_get(&query, "mtu").map(TinyText::from);

        // persistent_keepalive/keepalive: optional keepalive interval
        let persistent_keepalive =
            utils::query_get_multi(&query, &["persistent_keepalive", "keepalive"])
                .and_then(|s| s.parse::<u32>().ok());

        // dns: comma-separated DNS servers
        let dns = utils::query_get(&query, "dns")
            .map(|s| {
                s.split(',')
                    .map(|part| part.trim().to_string())
                    .filter(|part| !part.is_empty())
                    .collect::<Vec<String>>()
            })
            .filter(|v| !v.is_empty());

        // remote_dns_resolve/remote_dns: force remote DNS resolution
        let remote_dns_resolve =
            utils::query_get_multi(&query, &["remote_dns_resolve", "remote_dns"]).and_then(|s| {
                match s.to_lowercase().as_str() {
                    "true" | "1" | "yes" => Some(true),
                    "false" | "0" | "no" => Some(false),
                    _ => None,
                }
            });

        let remarks = utils::decode_fragment(raw)?;

        Ok(Self {
            private_key,
            host: parsed_host,
            port: parsed_port,
            security: SecurityConfig::default(),
            address,
            public_key,
            preshared_key,
            reserved,
            mtu,
            persistent_keepalive,
            dns,
            remote_dns_resolve,
            remarks,
        })
    }

    fn reconstruct(&self) -> Result<String, ParseError> {
        let host = self.host.to_str();
        let hostport = if host.contains(':') {
            format!("[{host}]:{}", self.port)
        } else {
            format!("{host}:{}", self.port)
        };

        let mut parts: Vec<String> = Vec::new();
        parts.push(format!("address={}", urlencoding::encode(&self.address)));
        parts.push(format!(
            "publickey={}",
            urlencoding::encode(&self.public_key)
        ));
        if let Some(v) = &self.preshared_key
            && !v.is_empty()
        {
            parts.push(format!("presharedkey={}", urlencoding::encode(v)));
        }
        if let Some(v) = &self.reserved {
            parts.push(format!("reserved={}", urlencoding::encode(v)));
        }
        if let Some(v) = &self.mtu {
            parts.push(format!("mtu={}", urlencoding::encode(v)));
        }
        if let Some(v) = &self.persistent_keepalive {
            parts.push(format!("persistent_keepalive={v}"));
        }
        if let Some(v) = &self.dns {
            parts.push(format!("dns={}", urlencoding::encode(&v.join(","))));
        }
        if let Some(v) = &self.remote_dns_resolve {
            parts.push(format!("remote_dns_resolve={v}"));
        }

        let query_string = if parts.is_empty() {
            String::new()
        } else {
            format!("?{}", parts.join("&"))
        };

        let fragment = self
            .remarks
            .as_ref()
            .map(|f| format!("#{}", urlencoding::encode(f)))
            .unwrap_or_default();

        Ok(format!(
            "wireguard://{private_key}@{hostport}{query_string}{fragment}",
            private_key = urlencoding::encode(&self.private_key),
        ))
    }

    fn schema(&self) -> SchemeX {
        SchemeX::WireGuard
    }

    fn host(&self) -> Option<&HostSpec> {
        Some(&self.host)
    }

    fn port(&self) -> Option<u16> {
        Some(self.port)
    }

    fn remarks(&self) -> Option<&str> {
        self.remarks.as_deref()
    }

    fn transport_type(&self) -> Option<&str> {
        None
    }

    fn try_from_clash(proxy: &ClashProxy) -> Result<Self, ParseError> {
        match proxy {
            ClashProxy::Wireguard(c) => {
                // ClashWireGuard.ip may not have CIDR suffix; add /32 if missing
                let address =
                    c.ip.as_deref()
                        .map(|ip| {
                            if ip.contains('/') {
                                TinyText::from(ip)
                            } else {
                                TinyText::from(format!("{ip}/32"))
                            }
                        })
                        .unwrap_or_default();

                Ok(Self {
                    private_key: c.private_key.clone(),
                    host: clash_server_to_host(&c.server)?,
                    port: c.port,
                    security: SecurityConfig::default(),
                    address,
                    public_key: c.public_key.clone(),
                    preshared_key: c.pre_shared_key.clone(),
                    reserved: c.reserved.clone().map(TinyText::from),
                    mtu: c.mtu.map(|v| TinyText::from(v.to_string())),
                    persistent_keepalive: c.persistent_keepalive,
                    dns: c.dns.clone(),
                    remote_dns_resolve: None,
                    remarks: match c.name.as_str() {
                        "" => None,
                        s => Some(TinyText::from(s)),
                    },
                })
            }
            _ => Err(ParseError::Unknown("expected wireguard clash proxy".into())),
        }
    }

    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        let name = self.remarks.as_deref().unwrap_or("").to_string();
        Ok(ClashProxy::Wireguard(ClashWireGuard {
            name,
            server: host_spec_to_string(&self.host),
            port: self.port,
            private_key: self.private_key.clone(),
            public_key: self.public_key.clone(),
            ip: Some(
                self.address
                    .split('/')
                    .next()
                    .unwrap_or(&self.address)
                    .to_string(),
            ),
            ipv6: None,
            pre_shared_key: self.preshared_key.clone(),
            reserved: self.reserved.as_ref().map(std::string::ToString::to_string),
            mtu: self.mtu.as_ref().and_then(|v| v.parse::<u32>().ok()),
            dns: self.dns.clone(),
            persistent_keepalive: self.persistent_keepalive,
        }))
    }
}
impl ProtoIdentity for WireguardConfig {
    fn compute_sig(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(b"wireguard");
        hasher.write(self.host.to_str().as_bytes());
        hasher.write(&self.port.to_le_bytes());
        hasher.write(self.address.as_bytes());
        if let Some(v) = &self.reserved {
            hasher.write(v.as_bytes());
        }
        if let Some(v) = &self.mtu {
            hasher.write(v.as_bytes());
        }
        if let Some(v) = &self.persistent_keepalive {
            hasher.write(&v.to_le_bytes());
        }
        if let Some(v) = &self.dns {
            for svr in v {
                hasher.write(svr.as_bytes());
            }
        }
        if let Some(v) = &self.remote_dns_resolve {
            hasher.write(&[u8::from(*v)]);
        }
        hasher.finish()
    }
    fn compute_cred_hash(&self) -> u64 {
        utils::compute_cred_hash(&[
            ("private_key", self.private_key.as_str()),
            ("public_key", self.public_key.as_str()),
            ("preshared_key", self.preshared_key.as_deref().unwrap_or("")),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Proto, ProtoSpec, ProtocolConfig};
    use crate::urlx::SchemeX;

    #[test]
    fn test_wireguard_basic() {
        let url = "wireguard://eERuOncn22jnY3uYp8WLcy0SCuOkEbSDa0j%2BwAPSEH4%3D@162.159.192.1:2408?address=172.16.0.2%2F32&presharedkey=&reserved=236%2C163%2C162&publickey=bmXOC%2BF1FxEMF9dyiK2H5%2F1SUtzH0JuVo51h2wPfgyo%3D&mtu=1280";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = WireguardConfig::try_parse(&raw).expect("failed");
        assert_eq!(config.schema(), SchemeX::WireGuard);
        assert_eq!(config.host.to_str(), "162.159.192.1");
        assert_eq!(config.port, 2408_u16);
        assert_eq!(config.address, "172.16.0.2/32");
        assert_eq!(config.mtu.as_deref(), Some("1280"));
        assert_eq!(config.remarks, None);
    }

    #[test]
    fn wireguard_keys_are_credentials_not_sig() {
        // Same server/address/mtu, different preshared keys: sig identical,
        // cred_hash differs (public_key/preshared_key are credentials).
        let url_a = "wireguard://eERuOncn22jnY3uYp8WLcy0SCuOkEbSDa0j%2BwAPSEH4%3D@162.159.192.1:2408?address=172.16.0.2%2F32&presharedkey=AAA&publickey=bmXOC%2BF1FxEMF9dyiK2H5%2F1SUtzH0JuVo51h2wPfgyo%3D&mtu=1280";
        let url_b = "wireguard://eERuOncn22jnY3uYp8WLcy0SCuOkEbSDa0j%2BwAPSEH4%3D@162.159.192.1:2408?address=172.16.0.2%2F32&presharedkey=BBB&publickey=bmXOC%2BF1FxEMF9dyiK2H5%2F1SUtzH0JuVo51h2wPfgyo%3D&mtu=1280";
        let a = Proto::new(ProtocolConfig::Wireguard(
            WireguardConfig::try_parse(&crate::urlx::RawUrlX::from(url_a)).expect("failed"),
        ));
        let b = Proto::new(ProtocolConfig::Wireguard(
            WireguardConfig::try_parse(&crate::urlx::RawUrlX::from(url_b)).expect("failed"),
        ));
        assert_eq!(a.sig(), b.sig(), "preshared_key is a credential, not sig");
        assert_ne!(a.cred_hash(), b.cred_hash());
        assert_ne!(a.uid(), b.uid());
    }

    #[test]
    fn wireguard_host_port_are_identity_in_sig() {
        // Both URLs carry the same mtu so the sig assertion isolates host/port.
        let url_a = "wireguard://eERuOncn22jnY3uYp8WLcy0SCuOkEbSDa0j%2BwAPSEH4%3D@162.159.192.1:2408?address=172.16.0.2%2F32&publickey=bmXOC%2BF1FxEMF9dyiK2H5%2F1SUtzH0JuVo51h2wPfgyo%3D&mtu=1280";
        let url_b = "wireguard://eERuOncn22jnY3uYp8WLcy0SCuOkEbSDa0j%2BwAPSEH4%3D@162.159.192.1:2409?address=172.16.0.2%2F32&publickey=bmXOC%2BF1FxEMF9dyiK2H5%2F1SUtzH0JuVo51h2wPfgyo%3D&mtu=1280";
        let a = Proto::new(ProtocolConfig::Wireguard(
            WireguardConfig::try_parse(&crate::urlx::RawUrlX::from(url_a)).expect("failed"),
        ));
        let b = Proto::new(ProtocolConfig::Wireguard(
            WireguardConfig::try_parse(&crate::urlx::RawUrlX::from(url_b)).expect("failed"),
        ));
        assert_ne!(a.sig(), b.sig(), "different port -> different sig");
        assert_eq!(a.cred_hash(), b.cred_hash());
    }

    #[test]
    fn test_wireguard_with_remarks() {
        let url = "wireguard://eERuOncn22jnY3uYp8WLcy0SCuOkEbSDa0j%2BwAPSEH4%3D@162.159.192.1:2408?address=172.16.0.2%2F32&publickey=bmXOC%2BF1FxEMF9dyiK2H5%2F1SUtzH0JuVo51h2wPfgyo%3D&mtu=1280#%40V2rayBaaz";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = WireguardConfig::try_parse(&raw).expect("failed");
        assert_eq!(config.schema(), SchemeX::WireGuard);
        assert_eq!(config.remarks.as_deref(), Some("@V2rayBaaz"));
    }

    #[test]
    fn test_wireguard_hostname() {
        let url = "wireguard://privatekey==@wg.example.com:51820?address=10.0.0.2%2F32&publickey=serverpubkey==";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = WireguardConfig::try_parse(&raw).expect("failed");
        assert_eq!(config.host.to_str(), "wg.example.com");
        assert_eq!(config.port, 51820_u16);
        assert_eq!(config.address, "10.0.0.2/32");
    }

    #[test]
    fn test_wireguard_missing_address() {
        let url = "wireguard://key@1.2.3.4:51820?publickey=pubkey";
        let raw = crate::urlx::RawUrlX::from(url);
        let result = WireguardConfig::try_parse(&raw);
        assert!(result.is_err(), "expected error for missing address");
    }

    #[test]
    fn test_wireguard_missing_publickey() {
        let url = "wireguard://key@1.2.3.4:51820?address=10.0.0.1%2F32";
        let raw = crate::urlx::RawUrlX::from(url);
        let result = WireguardConfig::try_parse(&raw);
        assert!(result.is_err(), "expected error for missing publickey");
    }

    #[test]
    fn test_reconstruct_roundtrip() {
        let input = "wireguard://eERuOncn22jnY3uYp8WLcy0SCuOkEbSDa0j%2BwAPSEH4%3D@162.159.192.1:2408?address=172.16.0.2%2F32&presharedkey=&reserved=236%2C163%2C162&publickey=bmXOC%2BF1FxEMF9dyiK2H5%2F1SUtzH0JuVo51h2wPfgyo%3D&mtu=1280#%40V2rayBaaz";
        let raw = crate::urlx::RawUrlX::from(input);
        let parsed = WireguardConfig::try_parse(&raw).expect("failed to parse");
        let reconstructed = parsed.reconstruct().expect("failed to reconstruct");

        let raw2 = crate::urlx::RawUrlX::from(reconstructed.as_str());
        let reparsed = WireguardConfig::try_parse(&raw2).expect("failed to re-parse");

        assert_eq!(parsed.host, reparsed.host, "host mismatch");
        assert_eq!(parsed.port, reparsed.port, "port mismatch");
        assert_eq!(
            parsed.private_key, reparsed.private_key,
            "private_key mismatch"
        );
        assert_eq!(parsed.address, reparsed.address, "address mismatch");
        assert_eq!(
            parsed.public_key, reparsed.public_key,
            "public_key mismatch"
        );
        assert_eq!(parsed.mtu, reparsed.mtu, "mtu mismatch");
        assert_eq!(parsed.remarks, reparsed.remarks, "remarks mismatch");
    }

    #[test]
    fn test_serde_roundtrip() {
        let input = "wireguard://eERuOncn22jnY3uYp8WLcy0SCuOkEbSDa0j%2BwAPSEH4%3D@162.159.192.1:2408?address=172.16.0.2%2F32&publickey=bmXOC%2BF1FxEMF9dyiK2H5%2F1SUtzH0JuVo51h2wPfgyo%3D&mtu=1280";
        let raw = crate::urlx::RawUrlX::from(input);
        let parsed = WireguardConfig::try_parse(&raw).expect("failed");
        let json = serde_json::to_string(&parsed).expect("serialize");
        let deserialized: WireguardConfig = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.host, deserialized.host, "host mismatch");
        assert_eq!(parsed.port, deserialized.port, "port mismatch");
        assert_eq!(
            parsed.private_key, deserialized.private_key,
            "private_key mismatch"
        );
        assert_eq!(parsed.address, deserialized.address, "address mismatch");
        assert_eq!(
            parsed.public_key, deserialized.public_key,
            "public_key mismatch"
        );
    }

    use super::super::test_helpers::check_roundtrip;
    use super::WireguardConfig;

    #[test]
    fn test_wireguard_full() {
        // Full WireGuard with all new fields in query params
        let url = "wireguard://eERuOncn22jnY3uYp8WLcy0SCuOkEbSDa0j%2BwAPSEH4%3D@162.159.192.1:2408?address=172.16.0.2%2F32&publickey=bmXOC%2BF1FxEMF9dyiK2H5%2F1SUtzH0JuVo51h2wPfgyo%3D&persistent_keepalive=25&dns=1.1.1.1%2C8.8.8.8&remote_dns_resolve=true&presharedkey=psk123&mtu=1280";
        let raw = crate::urlx::RawUrlX::from(url);
        let config = WireguardConfig::try_parse(&raw).expect("failed");
        assert_eq!(config.schema(), SchemeX::WireGuard);
        assert_eq!(config.persistent_keepalive, Some(25));
        assert_eq!(
            config.dns.as_deref(),
            Some(&["1.1.1.1".to_string(), "8.8.8.8".to_string()][..])
        );
        assert_eq!(config.remote_dns_resolve, Some(true));
        assert_eq!(config.mtu.as_deref(), Some("1280"));
        assert_eq!(config.preshared_key.as_deref(), Some("psk123"));

        // Roundtrip: reconstruct and re-parse
        let reconstructed = config.reconstruct().expect("reconstruct");
        let raw2 = crate::urlx::RawUrlX::from(reconstructed.as_str());
        let reparsed = WireguardConfig::try_parse(&raw2).expect("re-parse");
        assert_eq!(config.persistent_keepalive, reparsed.persistent_keepalive);
        assert_eq!(config.dns, reparsed.dns);
        assert_eq!(config.remote_dns_resolve, reparsed.remote_dns_resolve);
        assert_eq!(config.mtu, reparsed.mtu);
        assert_eq!(config.host, reparsed.host);
        assert_eq!(config.port, reparsed.port);

        // Test alternative query key names
        let url2 = "wireguard://key@1.2.3.4:51820?address=10.0.0.1%2F32&publickey=pubkey&keepalive=30&remote_dns=false";
        let raw2 = crate::urlx::RawUrlX::from(url2);
        let config2 = WireguardConfig::try_parse(&raw2).expect("failed");
        assert_eq!(config2.persistent_keepalive, Some(30));
        assert_eq!(config2.remote_dns_resolve, Some(false));
    }

    #[test]
    fn test_roundtrip() {
        check_roundtrip::<WireguardConfig>(
            "wireguard://eERuOncn22jnY3uYp8WLcy0SCuOkEbSDa0j%2BwAPSEH4%3D@162.159.192.1:2408?address=172.16.0.2%2F32&presharedkey=&reserved=236%2C163%2C162&publickey=bmXOC%2BF1FxEMF9dyiK2H5%2F1SUtzH0JuVo51h2wPfgyo%3D&mtu=1280#%40V2rayBaaz",
        );
    }

    #[test]
    fn test_clash_roundtrip() {
        use super::super::test_helpers::check_clash_roundtrip;
        check_clash_roundtrip::<WireguardConfig>(
            "wireguard://eERuOncn22jnY3uYp8WLcy0SCuOkEbSDa0j%2BwAPSEH4%3D@162.159.192.1:2408?address=172.16.0.2%2F32&presharedkey=&reserved=236%2C163%2C162&publickey=bmXOC%2BF1FxEMF9dyiK2H5%2F1SUtzH0JuVo51h2wPfgyo%3D&mtu=1280#%40V2rayBaaz",
        );
    }
}
