//! Tailscale configuration (JSON config only — no standard share URL format).
//!
//! # Format
//! Tailscale does not have a standard share URL format. Configuration is only
//! available via JSON config file or UI form.
//!
//! # Fields (sing-box `TailscaleEndpointOptions`)
//!
//! | Field | Type | Description |
//! |-------|------|-------------|
//! | `hostname` | `Option<String>` | Node hostname |
//! | `auth_key` | `Option<String>` | Auth key for node registration |
//! | `control_url` | `Option<String>` | Control server URL |
//! | `state_directory` | `Option<String>` | Tailscale state directory |
//! | `ephemeral` | `Option<bool>` | Ephemeral node |
//! | `accept_routes` | `Option<bool>` | Accept advertised routes |
//! | `exit_node` | `Option<String>` | Exit node |
//! | `exit_node_allow_lan_access` | `Option<bool>` | Allow LAN while using exit node |
//! | `advertise_routes` | `Option<Vec<String>>` | Routes to advertise |
//!
//! # References
//! - sing-box: `option/tailscale.go` — `TailscaleEndpointOptions`

use serde::{Deserialize, Serialize};

use crate::clash::{ClashProxy, ClashTailscale};
use crate::proto_spec::ProtoSpecError;
use crate::proto_spec::common::SecurityConfig;
use crate::proto_spec::common::{clash_server_to_host, host_spec_to_string};
use crate::proto_spec::utils;
use crate::proto_spec::{ParseError, ProtoIdentity, ProtoSpec};
use crate::urlx::HostSpec;
use crate::urlx::TinyText;
use crate::urlx::{host_serde, port_serde};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "snake_case")]
pub struct TailscaleConfig {
    #[serde(with = "host_serde")]
    pub host: HostSpec,
    #[serde(with = "port_serde")]
    pub port: u16,
    pub hostname: Option<String>,
    pub auth_key: Option<String>,
    pub control_url: Option<String>,
    pub state_directory: Option<String>,
    pub ephemeral: Option<bool>,
    pub accept_routes: Option<bool>,
    pub exit_node: Option<String>,
    pub exit_node_allow_lan_access: Option<bool>,
    pub advertise_routes: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "SecurityConfig::is_empty")]
    pub security: SecurityConfig,
    pub remarks: Option<TinyText>,
}

impl ProtoSpec for TailscaleConfig {
    /// Tailscale does not support URL format — always returns an error.
    fn try_parse(_raw: &crate::urlx::RawUrlX<'_>) -> Result<Self, ParseError> {
        Err(ParseError::Unknown(
            "Tailscale does not support URL format".into(),
        ))
    }

    /// Tailscale does not support URL format — always returns an error.
    fn reconstruct(&self) -> Result<String, ParseError> {
        Err(ParseError::Unknown(
            "Tailscale does not support URL format".into(),
        ))
    }

    fn schema(&self) -> crate::urlx::SchemeX {
        crate::urlx::SchemeX::Tailscale
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
            ClashProxy::Tailscale(c) => Ok(Self {
                host: clash_server_to_host(&c.server)?,
                port: c.port,
                hostname: match c.hostname.as_str() {
                    "" => None,
                    s => Some(s.to_string()),
                },
                auth_key: c.auth_key.clone(),
                control_url: c.control_url.clone(),
                state_directory: c.state_dir.clone(),
                ephemeral: Some(c.ephemeral),
                accept_routes: Some(c.accept_routes),
                exit_node: c.exit_node.clone(),
                exit_node_allow_lan_access: c.exit_node_allow_lan_access,
                advertise_routes: None,
                security: SecurityConfig::default(),
                remarks: match c.name.as_str() {
                    "" => None,
                    s => Some(TinyText::from(s)),
                },
            }),
            _ => Err(ParseError::Unknown("expected tailscale clash proxy".into())),
        }
    }

    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        let name = self.remarks.as_deref().unwrap_or("").to_string();
        Ok(ClashProxy::Tailscale(ClashTailscale {
            name,
            server: host_spec_to_string(&self.host),
            port: self.port,
            hostname: self.hostname.clone().unwrap_or_default(),
            auth_key: self.auth_key.clone(),
            control_url: self.control_url.clone(),
            state_dir: self.state_directory.clone(),
            ephemeral: self.ephemeral.unwrap_or(false),
            accept_routes: self.accept_routes.unwrap_or(false),
            exit_node: self.exit_node.clone(),
            exit_node_allow_lan_access: self.exit_node_allow_lan_access,
        }))
    }
}

impl ProtoIdentity for TailscaleConfig {
    fn compute_sig(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(b"tailscale");
        if let Some(v) = &self.control_url {
            hasher.write(v.as_bytes());
        }
        hasher.finish()
    }
    fn compute_cred_hash(&self) -> u64 {
        utils::compute_cred_hash(&[("auth_key", self.auth_key.as_deref().unwrap_or(""))])
    }
}

#[cfg(test)]
mod tests {
    use super::TailscaleConfig;
    use crate::proto_spec::ProtoSpec;
    use crate::urlx::RawUrlX;

    #[test]
    fn test_tailscale_no_url() {
        let url = "tailscale://host:100";
        let raw = RawUrlX::from(url);
        assert!(TailscaleConfig::try_parse(&raw).is_err());
    }
}
