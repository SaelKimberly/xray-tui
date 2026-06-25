//! gRPC client for the `StatsService` API shared by xray-core and sing-box.
//!
//! Both backends expose a `v2ray.core.app.stats.command.StatsService` gRPC API on
//! `127.0.0.1:62789`. This module provides a unified [`StatsProvider`] trait with
//! backend-specific implementations and a factory function.

use async_trait::async_trait;

pub(crate) mod proto_gen {
    tonic::include_proto!("v2ray.core.app.stats.command");
}

use proto_gen as proto;

/// Shared gRPC API endpoint on localhost.
pub const API_ENDPOINT: &str = "http://127.0.0.1:62789";

// ── Error type ──────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("gRPC transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    #[error("gRPC status error: {0}")]
    Status(#[from] tonic::Status),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

// ── System stats struct ─────────────────────────────────────────────

/// System statistics from the core process.
#[derive(Debug, Clone, Default)]
pub struct SysStats {
    pub num_goroutine: u32,
    pub alloc: u64,       // bytes allocated
    pub total_alloc: u64, // cumulative bytes allocated
    pub sys: u64,         // bytes obtained from OS
    pub uptime: u32,      // seconds since core start
}

// ── StatsProvider trait ─────────────────────────────────────────────

/// Unified interface for querying traffic and system stats from either
/// xray-core or sing-box via their shared `V2Ray` Stats gRPC API.
#[async_trait]
pub trait StatsProvider: Send + Sync {
    /// Query traffic stats matching the given pattern.
    ///
    /// With `reset=true` the counters are reset after query so each poll
    /// returns delta since the last query.
    async fn query_stats(&self, pattern: &str, reset: bool) -> Result<Vec<proto::Stat>, GrpcError>;
    /// Get system stats (memory, goroutines, uptime).
    async fn get_sys_stats(&self) -> Result<SysStats, GrpcError>;
    /// Return the gRPC API endpoint URL.
    fn api_endpoint(&self) -> &str;
}

// ── XrayGrpcClient ──────────────────────────────────────────────────

/// gRPC client for xray-core's `StatsService`.
pub struct XrayGrpcClient {
    channel: tonic::transport::Channel,
}

impl XrayGrpcClient {
    pub async fn connect() -> Result<Self, GrpcError> {
        let channel = tonic::transport::Endpoint::new(API_ENDPOINT)
            .expect("valid API_ENDPOINT URI")
            .connect()
            .await?;
        Ok(Self { channel })
    }
}

#[async_trait]
impl StatsProvider for XrayGrpcClient {
    async fn query_stats(&self, pattern: &str, reset: bool) -> Result<Vec<proto::Stat>, GrpcError> {
        let mut client = proto::stats_service_client::StatsServiceClient::new(self.channel.clone());
        let response = client
            .query_stats(tonic::Request::new(proto::QueryStatsRequest {
                pattern: pattern.to_string(),
                reset,
                patterns: vec![],
                regexp: false,
            }))
            .await?;
        Ok(response.into_inner().stat)
    }

    async fn get_sys_stats(&self) -> Result<SysStats, GrpcError> {
        let mut client = proto::stats_service_client::StatsServiceClient::new(self.channel.clone());
        let response = client
            .get_sys_stats(tonic::Request::new(proto::SysStatsRequest {}))
            .await?;
        let s = response.into_inner();
        Ok(SysStats {
            num_goroutine: s.num_goroutine,
            alloc: s.alloc,
            total_alloc: s.total_alloc,
            sys: s.sys,
            uptime: s.uptime,
        })
    }

    fn api_endpoint(&self) -> &str {
        API_ENDPOINT
    }
}

// ── SingBoxGrpcClient ───────────────────────────────────────────────

/// gRPC client for sing-box's `V2Ray` API `StatsService`.
pub struct SingBoxGrpcClient {
    channel: tonic::transport::Channel,
}

impl SingBoxGrpcClient {
    pub async fn connect() -> Result<Self, GrpcError> {
        let channel = tonic::transport::Endpoint::new(API_ENDPOINT)
            .expect("valid API_ENDPOINT URI")
            .connect()
            .await?;
        Ok(Self { channel })
    }
}

#[async_trait]
impl StatsProvider for SingBoxGrpcClient {
    async fn query_stats(&self, pattern: &str, reset: bool) -> Result<Vec<proto::Stat>, GrpcError> {
        let mut client = proto::stats_service_client::StatsServiceClient::new(self.channel.clone());
        let response = client
            .query_stats(tonic::Request::new(proto::QueryStatsRequest {
                pattern: pattern.to_string(),
                reset,
                patterns: vec![],
                regexp: false,
            }))
            .await?;
        Ok(response.into_inner().stat)
    }

    async fn get_sys_stats(&self) -> Result<SysStats, GrpcError> {
        let mut client = proto::stats_service_client::StatsServiceClient::new(self.channel.clone());
        let response = client
            .get_sys_stats(tonic::Request::new(proto::SysStatsRequest {}))
            .await?;
        let s = response.into_inner();
        Ok(SysStats {
            num_goroutine: s.num_goroutine,
            alloc: s.alloc,
            total_alloc: s.total_alloc,
            sys: s.sys,
            uptime: s.uptime,
        })
    }

    fn api_endpoint(&self) -> &str {
        API_ENDPOINT
    }
}

// ── Factory ─────────────────────────────────────────────────────────

use crate::core_type::CoreType;

/// Create a [`StatsProvider`] for the given resolved core type.
pub async fn create_stats_provider(
    core_type: CoreType,
) -> Result<Box<dyn StatsProvider>, GrpcError> {
    match core_type {
        CoreType::Xray => Ok(Box::new(XrayGrpcClient::connect().await?)),
        CoreType::SingBox => Ok(Box::new(SingBoxGrpcClient::connect().await?)),
        CoreType::Auto => Err(GrpcError::InvalidResponse(
            "Auto must be resolved before creating provider".into(),
        )),
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

/// Format bytes to human-readable string (B/KB/MB/GB/TB/PB).
#[must_use]
pub fn format_bytes(bytes: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut b = bytes as f64;
    for unit in UNITS {
        if b < 1024.0 {
            return format!("{b:.1} {unit}");
        }
        b /= 1024.0;
    }
    format!("{b:.1} PB")
}

/// Format uptime seconds to human-readable string.
#[must_use]
pub fn format_uptime(secs: u32) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    match (hours, minutes, seconds) {
        (0, 0, s) => format!("{s}s"),
        (0, m, s) => format!("{m}m {s}s"),
        (h, 0, 0) => format!("{h}h"),
        (h, m, s) => format!("{h}h {m}m {s}s"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0.0 B");
        assert_eq!(format_bytes(500), "500.0 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(0), "0s");
        assert_eq!(format_uptime(45), "45s");
        assert_eq!(format_uptime(120), "2m 0s");
        assert_eq!(format_uptime(3600), "1h");
        assert_eq!(format_uptime(3661), "1h 1m 1s");
    }
}
