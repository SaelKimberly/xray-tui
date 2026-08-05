#![allow(
    clippy::option_if_let_else,
    reason = "code clarity decisions override style lints"
)]
#![allow(
    clippy::derive_partial_eq_without_eq,
    reason = "prost-generated code, Eq not in default derive set"
)]

pub mod bin_manager;
pub mod config_builder;
pub mod core_type;
pub mod dns;
pub mod grpc_client;
pub mod log_heed;
pub mod ping;
pub mod process;
pub mod protocol;
pub mod protocol_core_mapping;
pub mod speed_test;
pub mod updater;

pub use bin_manager::{CoreBinInfo, find_binary, get_core_info};

pub use config_builder::{
    BackendConfig, BuildError, BuildParams, CLASH_API_PORT, ConfigBuilder, MultiInboundItem,
    shadowsocks_method,
};
pub use core_type::CoreType;
pub use grpc_client::{
    API_ENDPOINT, GrpcError, StatsProvider, SysStats, create_stats_provider, format_bytes,
    format_uptime,
};
pub use log_heed::HeedLogStorage;
#[cfg(feature = "quic-ping")]
pub use ping::QuicPingAdapter;
pub use ping::{
    CorePool, FastPingAdapter, FastPingManager, PingCapability, PingError, PingResult, ProfileKey,
    RealPingManager, TcpPingAdapter, UdpPingAdapter,
};
pub use process::{CoreManager, MockCoreManager, ProcessError, RealCoreManager};
pub use protocol::Protocol;
pub use protocol::SINGBOX_ONLY_PROTOCOLS;
pub use protocol_core_mapping::resolve_core;
pub use speed_test::{
    SpeedTestError, TestType, real_ping, speed_test, tcp_ping, udp_ping, udp_test, wait_for_socks5,
};
