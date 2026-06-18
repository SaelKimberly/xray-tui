pub mod bin_manager;
pub mod config_builder;
pub mod core_type;
pub mod grpc_client;
pub mod process;
pub mod protocol;
pub mod protocol_core_mapping;
pub mod speed_test;

pub use bin_manager::{CoreBinInfo, find_binary, get_core_info};
pub use config_builder::{BackendConfig, BuildError, BuildParams, ConfigBuilder};
pub use core_type::CoreType;
pub use grpc_client::{
    API_ENDPOINT, GrpcError, StatsProvider, SysStats, create_stats_provider, format_bytes,
    format_uptime,
};
pub use process::CoreManager;
pub use protocol::Protocol;
pub use protocol::SINGBOX_ONLY_PROTOCOLS;
pub use protocol_core_mapping::resolve_core;
pub use speed_test::{SpeedTestError, TestType, real_ping, speed_test, tcp_ping, udp_test};
