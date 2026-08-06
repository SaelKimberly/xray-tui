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
    RealPingManager, SinglePingReq, TcpPingAdapter, UdpPingAdapter,
};
pub use process::{CoreManager, MockCoreManager, ProcessError, RealCoreManager};
pub use speed_test::{
    SpeedTestError, TestType, real_ping, speed_test, tcp_ping, udp_ping, udp_test, wait_for_socks5,
};
pub use xray_tui_proto::proto_spec::core_mapping::{
    SINGBOX_ONLY_KINDS, SINGBOX_SS_METHODS, XRAY_SS_METHODS, resolve_core,
    singbox_supports_ss_method, ss_method_supported, xray_supports_ss_method,
};
