//! Native proxy core — client-side, in-process implementations of the proxy
//! protocols xray-tui otherwise obtains by spawning xray-core / sing-box
//! subprocesses.
//!
//! # Layering
//!
//! Transports are OUTERMOST, protocols INNERMOST (Xray composition order):
//! dial → transport upgrade → security wrap → protocol handshake → tunnel.
//!
//! # Placeholder semantics
//!
//! Every protocol kind, transport, and security has a module and a dispatch
//! arm; unsupported combinations return [`NativeError::NotImplemented`].
//! The uniform `handshake(stream)` shape applies to the TCP-stream family
//! only — WireGuard/Tailscale (device tunnels), SSH (own handshake), and the
//! outbound-only kinds (Redirect/TProxy/Mixed/DokodemoDoor/...) diverge at
//! dispatch time per [`shape::ConnectShape`]. See `shape.rs`.
//!
//! # Config source of truth
//!
//! [`NativeConnectParams`] wraps `xray-tui-proto` typed models
//! ([`ProtocolConfig`], [`EndpointEssentials`]). No config model is defined
//! here.

pub mod error;

pub use error::NativeError;
