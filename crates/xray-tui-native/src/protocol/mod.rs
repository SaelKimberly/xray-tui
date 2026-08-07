//! Protocol phase: the INNERMOST layer — write the protocol handshake onto
//! the secured stream and produce the byte tunnel.
//!
//! Dispatch strategy (see `shape.rs`): the uniform handshake-over-stream
//! pipeline applies to the TCP-stream family. Device tunnels
//! (WireGuard/Tailscale), own-handshake protocols (SSH/Tor), and the
//! outbound-only kinds (Redirect/TProxy/Mixed) take divergent paths at their
//! own connect() and return NotImplemented here until those paths exist.

pub mod vless;
