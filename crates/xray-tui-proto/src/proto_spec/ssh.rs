//! SSH configuration (JSON config only — no standard share URL format).
//!
//! # Format
//! SSH does not have a standard share URL format. Configuration is only
//! available via JSON config file or UI form.
//!
//! # Fields (sing-box `SSHOutboundOptions`)
//!
//! | Field | Type | Description |
//! |-------|------|-------------|
//! | `user` | `Option<String>` | SSH username |
//! | `password` | `Option<String>` | SSH password |
//! | `private_key` | `Option<String>` | SSH private key |
//! | `private_key_path` | `Option<String>` | SSH private key path |
//! | `private_key_passphrase` | `Option<String>` | SSH private key passphrase |
//! | `host_key` | `Option<Vec<String>>` | Accepted host keys |
//! | `host_key_algorithms` | `Option<Vec<String>>` | Host key algorithms |
//! | `client_version` | `Option<String>` | SSH client version string |
//!
//! # References
//! - sing-box: `option/ssh.go` — `SSHOutboundOptions`

use serde::{Deserialize, Serialize};

use crate::clash::{ClashProxy, ClashSsh};
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
pub struct SshConfig {
    #[serde(with = "host_serde")]
    pub host: HostSpec,
    #[serde(with = "port_serde")]
    pub port: u16,
    pub user: Option<String>,
    pub password: Option<String>,
    pub private_key: Option<String>,
    pub private_key_path: Option<String>,
    pub private_key_passphrase: Option<String>,
    pub host_key: Option<Vec<String>>,
    pub host_key_algorithms: Option<Vec<String>>,
    pub client_version: Option<String>,
    #[serde(default, skip_serializing_if = "SecurityConfig::is_empty")]
    pub security: SecurityConfig,
    pub remarks: Option<TinyText>,
}

impl ProtoSpec for SshConfig {
    /// SSH does not support URL format — always returns an error.
    fn try_parse(_raw: &crate::urlx::RawUrlX<'_>) -> Result<Self, ParseError> {
        Err(ParseError::Unknown(
            "SSH does not support URL format".into(),
        ))
    }

    /// SSH does not support URL format — always returns an error.
    fn reconstruct(&self) -> Result<String, ParseError> {
        Err(ParseError::Unknown(
            "SSH does not support URL format".into(),
        ))
    }

    fn schema(&self) -> crate::urlx::SchemeX {
        crate::urlx::SchemeX::Ssh
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
            ClashProxy::Ssh(c) => Ok(Self {
                host: clash_server_to_host(&c.server)?,
                port: c.port,
                user: Some(c.user.clone()),
                password: c.password.clone(),
                private_key: c.private_key.clone(),
                private_key_path: c.private_key_path.clone(),
                private_key_passphrase: None,
                host_key: c.host_key.clone(),
                host_key_algorithms: c.host_key_algorithms.clone(),
                client_version: c.client_version.clone(),
                security: SecurityConfig::default(),
                remarks: match c.name.as_str() {
                    "" => None,
                    s => Some(TinyText::from(s)),
                },
            }),
            _ => Err(ParseError::Unknown("expected ssh clash proxy".into())),
        }
    }

    fn to_clash(&self) -> Result<ClashProxy, ProtoSpecError> {
        let name = self.remarks.as_deref().unwrap_or("").to_string();
        Ok(ClashProxy::Ssh(ClashSsh {
            name,
            server: host_spec_to_string(&self.host),
            port: self.port,
            user: self.user.clone().unwrap_or_default(),
            password: self.password.clone(),
            private_key: self.private_key.clone(),
            private_key_path: self.private_key_path.clone(),
            host_key: self.host_key.clone(),
            host_key_algorithms: self.host_key_algorithms.clone(),
            client_version: self.client_version.clone(),
        }))
    }
}

impl ProtoIdentity for SshConfig {
    fn compute_sig(&self) -> u64 {
        use rapidhash::v3::RapidStreamHasherV3;
        let mut hasher = RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS);
        hasher.write(b"ssh");
        hasher.write(self.host.to_str().as_bytes());
        hasher.write(&self.port.to_le_bytes());
        if let Some(v) = &self.user {
            hasher.write(v.as_bytes());
        }
        if let Some(v) = &self.client_version {
            hasher.write(v.as_bytes());
        }
        hasher.finish()
    }
    fn compute_cred_hash(&self) -> u64 {
        utils::compute_cred_hash(&[
            ("password", self.password.as_deref().unwrap_or("")),
            ("private_key", self.private_key.as_deref().unwrap_or("")),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::SshConfig;
    use crate::proto_spec::ProtoSpec;
    use crate::urlx::RawUrlX;

    #[test]
    fn test_ssh_no_url() {
        let url = "ssh://user@host:22";
        let raw = RawUrlX::from(url);
        assert!(SshConfig::try_parse(&raw).is_err());
    }
}
