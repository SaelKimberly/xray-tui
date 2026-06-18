pub mod bin_manager;
pub mod config_builder;
pub mod core_type;
pub mod process;
pub mod protocol;
pub mod protocol_core_mapping;

pub use config_builder::{BackendConfig, BuildError, BuildParams, ConfigBuilder};
pub use core_type::CoreType;
pub use process::CoreManager;
pub use bin_manager::{find_binary, get_core_info, CoreBinInfo};
pub use protocol::Protocol;
pub use protocol::SINGBOX_ONLY_PROTOCOLS;
pub use protocol_core_mapping::resolve_core;
