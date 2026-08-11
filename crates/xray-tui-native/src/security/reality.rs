//! REALITY client stub — contract only (M3 milestone).
//!
//! The provisioner contract (`HelloProvisioner` / `HelloProvisionParams` /
//! `ProvisionedHello` / `FixedChrome133`) now lives in
//! `xray-tui-tls::reality` (moved with the engine); this module re-exports
//! it for source compat until the connector lands. `connect` is still a stub
//! returning `NotImplemented` — the REALITY handshake is `Task 14`.

use std::sync::Arc;

use crate::BoxStream;
use crate::context::LinkContext;
use crate::error::NativeError;

pub use xray_tui_tls::reality::{
    FixedChrome133, HelloProvisionParams, HelloProvisioner, ProvisionedHello,
};

/// Chosen provisioner for a connect.
#[derive(Clone, Default)]
pub enum HelloProvisionerChoice {
    #[default]
    FixedChrome133,
    Custom(Arc<dyn HelloProvisioner>),
}

/// REALITY client handshake — implemented in `Task 14`; stub today.
#[allow(clippy::unused_async)] // signature is the Task 14 seam; body lands then
pub async fn connect(_ctx: &LinkContext, _stream: BoxStream) -> Result<BoxStream, NativeError> {
    Err(NativeError::NotImplemented {
        feature: "security reality".into(),
    })
}
