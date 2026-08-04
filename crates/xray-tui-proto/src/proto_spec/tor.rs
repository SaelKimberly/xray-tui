//! Tor configuration (JSON config only — no share URL format).
//!
//! # Format
//! Tor does not have a standard share URL format. Configuration is only
//! available via JSON config file or UI form.
//!
//! # Fields (sing-box `TorOutboundOptions`)
//!
//! | Field | Type | Description |
//! |-------|------|-------------|
//! | `executable_path` | `Option<String>` | Tor binary path |
//! | `extra_args` | `Option<Vec<String>>` | Extra CLI arguments |
//! | `data_directory` | `Option<String>` | Tor data directory |
//! | `torrc` | `Option<HashMap<String, String>>` | Additional torrc options |
//!
//! # References
//! - sing-box: `option/tor.go` — `TorOutboundOptions`

use serde::{Deserialize, Serialize};

use crate::clash::{ClashProxy, ClashTor};
use crate::proto_spec::ProtoSpecError;
use crate::proto_spec::common::SecurityConfig;
use crate::proto_spec::common::{clash_server_to_host, host_spec_to_string};
use crate::proto_spec::{ParseError, ProtoIdentity, ProtoSpec};
use crate::urlx::HostSpec;
use crate::urlx::TinyText;
use crate::urlx::{host_serde, port_serde};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "snake_case")]
pub struct TorConfig {
    #[serde(with = "host_serde")]
    pub host: HostSpec,
    #[serde(with = "port_serde")]
    pub port: u16,
    pub executable_path: Option<String>,
    pub extra_args: Option<Vec<String>>,
    pub data_directory: Option<String>,
    pub torrc: Option<std::collections::HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "SecurityConfig::is_empty")]
    pub security: SecurityConfig,
    pub remarks: Option<TinyText>,
}

impl ProtoSpec for TorConfig {
    /// Tor does not support URL format — always returns an error.
    fn try_parse(_raw: &crate::urlx::RawUrlX<'_>) -> Result<Self, ParseError> {
        Err(ParseError::Unknown(
            "Tor does not support URL format".into(),
        ))
    }

    /// Tor does not support URL format — always returns an error.
    fn reconstruct(&self) -> Result<String, ParseError> {
        Err(ParseError::Unknown(
            "Tor does not support URL format".into(),
        ))
    }

    fn schema(&self) -> crate::urlx::SchemeX {
        crate::urlx::SchemeX::Tor
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
            ClashProxy::Tor(c) => Ok(Self {
                host: clash_server_to_host(&c.server)?,
                port: c.port,
                executable_path: None,
                extra_args: None,
                data_directory: None,
                torrc: None,
                security: SecurityConfig::default(),
                remarks: match c.name.as_str() {
                    "" => None,
                    s => Some(TinyText::from(s)),
                },
            }),
            _ => Err(ParseError::Unknown("expected tor clash proxy".into())),
        }
    }

    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        let name = self.remarks.as_deref().unwrap_or("").to_string();
        Ok(ClashProxy::Tor(ClashTor {
            name,
            server: host_spec_to_string(&self.host),
            port: self.port,
        }))
    }
}

impl ProtoIdentity for TorConfig {
    fn compute_sig(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(b"tor");
        hasher.finish()
    }
    fn compute_cred_hash(&self) -> u64 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::TorConfig;
    use crate::proto_spec::ProtoSpec;
    use crate::urlx::RawUrlX;

    #[test]
    fn test_tor_no_url() {
        let url = "tor://user@host:9050";
        let raw = RawUrlX::from(url);
        assert!(TorConfig::try_parse(&raw).is_err());
    }
}
