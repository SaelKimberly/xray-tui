pub mod core_type;
pub mod protocol;
pub mod protocol_core_mapping;

pub use core_type::CoreType;
pub use protocol::Protocol;
pub use protocol::SINGBOX_ONLY_PROTOCOLS;
pub use protocol_core_mapping::resolve_core;
