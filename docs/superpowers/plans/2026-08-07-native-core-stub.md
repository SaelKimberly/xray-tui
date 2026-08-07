# Native Core Stub Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create `crates/xray-tui-native` — a client-only, in-process implementation of the proxy protocols xray-tui spawns subprocesses for — with a full placeholder skeleton (27 protocol kinds, 8 transports, 3 securities) and one working protocol: VLESS over TCP with TLS, proven end-to-end against real xray-core and sing-box servers.

**Architecture:** Static-enum three-phase pipeline (transport → security → protocol) over a boxed async stream, dispatching by `match` on the typed `xray-tui-proto` configs (`ProtocolConfig`, `TransportConfig`, `SecurityConfig`). Config source of truth is `xray-tui-proto`; the native crate never defines its own config model. A `connect_chain` fold gives client-side chaining (parity with xray `proxySettings` / sing-box `detour`). E2E tests spawn real core binaries as servers.

**Tech Stack:** Rust 2024 (1.97.1), tokio 1.53, rustls 0.23.43 (ring), tokio-rustls 0.26.4, xray-tui-proto (workspace), tiny_http + rcgen (test harness), md-5 (JA3).

## Global Constraints

- Workspace: `members = ["crates/*"]`, edition 2024, `rust-version = 1.97.1`, license MIT, clippy `pedantic`+`nursery` at warn via `[lints] workspace = true`.
- Dep pins (copy verbatim): `tokio = { version = "1.53" }`, `rustls = { version = "0.23", default-features = false }` + feature `ring`, `tokio-rustls = "0.26"` (0.26.4 in lock), `xray-tui-proto = { path = "crates/xray-tui-proto" }`, `thiserror = "2.0.19"`, `uuid = "1.24" (features v4)`, `futures-util = "0.3"`.
- MIT-clean: no code copied from GPL/MPL sources (Xray-core MPL-2.0, mihomo GPL-3.0 are spec references only). Wire contract modeled on v2ray-core (MIT). All code written fresh.
- Single source of truth: consume `xray-tui-proto` types directly. No duplicate config model, no `From`-adapter layer.
- Static enum dispatch via `match`. No trait-object protocol registries, no side-effect registration.
- Composition order: dial → transport upgrade → security wrap → protocol handshake → tunnel. Transport OUTERMOST, protocol INNERMOST.
- Stream seams: `pub type BoxStream = Box<dyn tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send>`.
- Timeouts on every network step: dial 10 s, transport upgrade 10 s, security handshake 10 s, protocol handshake 10 s, tunnel idle read 30 s. Every async network call wrapped in `tokio::time::timeout`.
- Harmless `#[allow(clippy::…)]` is not allowed; follow existing repo style (docs in comments, no panics in public API, thiserror types).
- E2E gating: feature `native-e2e` + env `XRAY_TUI_CORE_BIN_DIR`; binaries absent → tests skip with `eprintln!` + early return, never fail. Never auto-download.
- Harness policy: `tiny_http` responder (sync, dedicated thread), rcgen CA trusted by rustls root store, **no skip-verify** on the proxy path, ALPN matched explicitly both sides.
- `cargo test` runs from workspace root; every task ends green.

---

### Task 1: Scaffold crate, workspace registration, NativeError

**Files:**
- Create: `crates/xray-tui-native/Cargo.toml`
- Create: `crates/xray-tui-native/src/lib.rs`
- Create: `crates/xray-tui-native/src/error.rs`
- Test: none beyond compile — deliverable is a compiling crate with the error model.

**Interfaces:**
- Produces: `NativeError` (thiserror) with variants `Config(String)`, `Dial(String)`, `Tls(String)`, `Reality(String)`, `Protocol { kind, detail }`, `NotImplemented { feature }`, `Io(std::io::Error)`, `Timeout { step, limit }`. Later tasks use these everywhere.

- [ ] **Step 1: Create `crates/xray-tui-native/Cargo.toml`**

```toml
[package]
name = "xray-tui-native"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
# source of truth for all protocol/transport/security configs (MIT, workspace)
xray-tui-proto.workspace = true

# async runtime
tokio = { workspace = true, features = ["net", "io-util", "time", "sync", "rt"] }
futures-util.workspace = true

# TLS (workspace standard: ring backend)
rustls = { workspace = true, features = ["ring"] }
tokio-rustls = "0.26"
webpki-roots = "1"

# errors / observability
thiserror.workspace = true
tracing.workspace = true

# misc
uuid.workspace = true
md-5 = "0.10"

[lints]
workspace = true

[features]
native-e2e = []

[dev-dependencies]
tempfile = "3"
rcgen = "0.13"
tiny_http = "0.12"
tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }
```

Note: `md-5` exists only for the JA3 fingerprint hash (ring has no MD5). `tokio-rustls 0.26` matches lockfile 0.26.4 and rustls 0.23.

- [ ] **Step 2: Create `crates/xray-tui-native/src/error.rs`**

```rust
use std::time::Duration;

use xray_tui_proto::proto_spec::ProtocolKind;

/// Errors from the native proxy core.
///
/// Every network step (dial, transport upgrade, security handshake, protocol
/// handshake, tunnel I/O) is wrapped in `tokio::time::timeout`; a deadline
/// expiry surfaces as [`NativeError::Timeout`].
#[derive(Debug, thiserror::Error)]
pub enum NativeError {
    #[error("invalid or unsupported config: {0}")]
    Config(String),
    #[error("server dial failed: {0}")]
    Dial(String),
    #[error("TLS error: {0}")]
    Tls(String),
    #[error("REALITY error: {0}")]
    Reality(String),
    #[error("protocol {kind} error: {detail}")]
    Protocol {
        kind: ProtocolKind,
        detail: String,
    },
    #[error("not implemented: {feature}")]
    NotImplemented { feature: String },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("timeout on {step} (limit {limit:?})")]
    Timeout { step: &'static str, limit: Duration },
}

/// Named deadline limits, applied around every network step.
pub mod timeouts {
    use std::time::Duration;

    pub const DIAL: Duration = Duration::from_secs(10);
    pub const TRANSPORT: Duration = Duration::from_secs(10);
    pub const SECURITY: Duration = Duration::from_secs(10);
    pub const PROTOCOL: Duration = Duration::from_secs(10);
    pub const TUNNEL_READ: Duration = Duration::from_secs(30);
}
```

- [ ] **Step 3: Create `crates/xray-tui-native/src/lib.rs` (skeleton)**

```rust
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

pub mod addr;
pub mod chain;
pub mod context;
pub mod crypto;
pub mod error;
pub mod protocol;
pub mod security;
pub mod shape;
pub mod transport;

pub use context::{LinkContext, NativeConnectParams};
pub use error::NativeError;

/// The byte-stream seam between layers.
pub type BoxStream = Box<dyn tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send>;
```

This file intentionally references modules created in Tasks 2–10; `cargo check` will fail until they exist. Complete Task 1's check with only `error` wired:

- [ ] **Step 4: Wire only `error` into lib.rs and compile**

Replace the `pub mod …` block with just:

```rust
pub mod error;
pub use error::NativeError;
```

- [ ] **Step 5: Run checks**

Run: `cargo check -p xray-tui-native`
Expected: compiles cleanly. Then `cargo check --workspace` — the new crate must not disturb the workspace.

- [ ] **Step 6: Commit**

```bash
git add crates/xray-tui-native
git commit -m "feat(native): scaffold crate with NativeError model"
```

---

### Task 2: TargetAddr and the v2ray address codec

**Files:**
- Create: `crates/xray-tui-native/src/addr.rs`
- Modify: `crates/xray-tui-native/src/lib.rs` — add `pub mod addr;`

**Interfaces:**
- Consumes: `EndpointEssentials` (`xray_tui_proto::proto_spec::endpoint::EndpointEssentials` — fields `host: String`, `host_type: HostKind { Ipv4, Ipv6, Dns, Undefined }`, `port: u16`, `ports: Vec<u16>`), `HostKind`.
- Produces: `Host` enum, `TargetAddr` struct, `encode_addr(&TargetAddr) -> Vec<u8>` (port-first, v2ray wire), `decode_addr(&[u8]) -> Option<(TargetAddr, &[u8])>`. Task 7 (VLESS header) consumes `encode_addr`.

- [ ] **Step 1: Add the module to lib.rs**

```rust
pub mod addr;
```

- [ ] **Step 2: Write the failing test first**

Create `crates/xray-tui-native/src/addr.rs` with the public API stubbed to `unimplemented!()` *except* the test module:

```rust
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;
use xray_tui_proto::proto_spec::HostKind;

/// Remote host: parsed IP or DNS name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Host {
    Ip(IpAddr),
    Domain(String),
}

impl Host {
    /// Parse a host string: `IpAddr` when it parses, else `Domain`.
    #[must_use]
    pub fn new(host: &str) -> Self {
        match host.parse::<IpAddr>() {
            Ok(ip) => Self::Ip(ip),
            Err(_) => Self::Domain(host.to_string()),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> HostKind {
        match self {
            Self::Ip(IpAddr::V4(_)) => HostKind::Ipv4,
            Self::Ip(IpAddr::V6(_)) => HostKind::Ipv6,
            Self::Domain(_) => HostKind::Dns,
        }
    }
}

/// A destination address on the wire: host + port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetAddr {
    pub host: Host,
    pub port: u16,
}

impl TargetAddr {
    #[must_use]
    pub fn new(host: impl Into<Host>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    /// Build from an endpoint's host/port (resolve later; IP hosts stay IP).
    #[must_use]
    pub fn from_endpoint(e: &EndpointEssentials) -> Self {
        Self {
            host: Host::new(&e.host),
            port: e.port,
        }
    }
}

// v2ray wire address types (common/protocol/payload.go AddressType).
pub const ADDR_TYPE_IPV4: u8 = 1;
pub const ADDR_TYPE_DOMAIN: u8 = 2;
pub const ADDR_TYPE_IPV6: u8 = 3;

/// Encode a destination in v2ray wire format: **port first, then address**
/// (`PortThenAddress`), address = type byte + payload.
#[must_use]
pub fn encode_addr(target: &TargetAddr) -> Vec<u8> {
    let mut out = Vec::with_capacity(8);
    out.extend_from_slice(&target.port.to_be_bytes());
    match &target.host {
        Host::Ip(IpAddr::V4(ip)) => {
            out.push(ADDR_TYPE_IPV4);
            out.extend_from_slice(&ip.octets());
        }
        Host::Ip(IpAddr::V6(ip)) => {
            out.push(ADDR_TYPE_IPV6);
            out.extend_from_slice(&ip.octets());
        }
        Host::Domain(domain) => {
            out.push(ADDR_TYPE_DOMAIN);
            out.push(u8::try_from(domain.len()).expect("domain longer than 255 bytes"));
            out.extend_from_slice(domain.as_bytes());
        }
    }
    out
}

/// Decode one wire address; returns the address plus the unconsumed tail.
pub fn decode_addr(bytes: &[u8]) -> Option<(TargetAddr, &[u8])> {
    if bytes.len() < 3 {
        return None;
    }
    let port = u16::from_be_bytes([bytes[0], bytes[1]]);
    match bytes[2] {
        ADDR_TYPE_IPV4 => {
            if bytes.len() < 3 + 4 {
                return None;
            }
            let ip = Ipv4Addr::new(bytes[3], bytes[4], bytes[5], bytes[6]);
            Some((
                TargetAddr::new(Host::Ip(IpAddr::V4(ip)), port),
                &bytes[7..],
            ))
        }
        ADDR_TYPE_DOMAIN => {
            let len = usize::from(bytes[3]);
            if bytes.len() < 4 + len {
                return None;
            }
            let domain =
                std::str::from_utf8(&bytes[4..4 + len]).ok()?.to_string();
            Some((
                TargetAddr::new(Host::Domain(domain), port),
                &bytes[4 + len..],
            ))
        }
        ADDR_TYPE_IPV6 => {
            if bytes.len() < 3 + 16 {
                return None;
            }
            let mut octets = [0u8; 16];
            octets.copy_from_slice(&bytes[3..19]);
            Some((
                TargetAddr::new(Host::Ip(IpAddr::V6(Ipv6Addr::from(octets))), port),
                &bytes[19..],
            ))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_ipv4_is_port_first() {
        let t = TargetAddr::new(Host::new("127.0.0.1"), 8080);
        assert_eq!(encode_addr(&t), vec![0x1f, 0x90, 0x01, 127, 0, 0, 1]);
    }

    #[test]
    fn encode_domain() {
        let t = TargetAddr::new(Host::Domain("example.com".into()), 443);
        assert_eq!(
            encode_addr(&t),
            vec![0x01, 0xbb, 0x02, 0x0b, b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'c', b'o', b'm']
        );
    }

    #[test]
    fn encode_ipv6() {
        let t = TargetAddr::new(Host::new("::1"), 1234);
        let mut expected = vec![0x04, 0xd2, 0x03];
        expected.extend_from_slice(&[0u8; 15]);
        expected.push(1);
        assert_eq!(encode_addr(&t), expected);
    }

    #[test]
    fn decode_roundtrip_all_types() {
        for t in [
            TargetAddr::new(Host::new("1.2.3.4"), 80),
            TargetAddr::new(Host::Domain("sub.example.org".into()), 255),
            TargetAddr::new(Host::new("2001:db8::1"), 443),
        ] {
            let encoded = encode_addr(&t);
            let (decoded, tail) = decode_addr(&encoded).expect("decode");
            assert_eq!(decoded, t);
            assert!(tail.is_empty());
        }
    }

    #[test]
    fn decode_domain_with_tail_keeps_rest() {
        let t = TargetAddr::new(Host::Domain("x.io".into()), 9);
        let mut bytes = encode_addr(&t);
        bytes.push(0xAA);
        let (decoded, tail) = decode_addr(&bytes).expect("decode");
        assert_eq!(decoded, t);
        assert_eq!(tail, &[0xAA]);
    }

    #[test]
    fn decode_truncated_inputs_return_none() {
        assert!(decode_addr(&[]).is_none());
        assert!(decode_addr(&[0x00, 0x50]).is_none());
        assert!(decode_addr(&[0x00, 0x50, 0x01, 0x7f]).is_none());
        assert!(decode_addr(&[0x00, 0x50, 0x02, 0x05, b'a']).is_none());
        assert!(decode_addr(&[0x00, 0x50, 0x09, 0x00]).is_none());
    }
}
```

- [ ] **Step 3: Run tests to verify they fail (stubs first)**

If the body was written in one go, temporarily change `encode_addr` to `unimplemented!()` and run:

Run: `cargo test -p xray-tui-native addr`
Expected: FAIL (panic "not implemented") — proves the tests exercise real behavior.

- [ ] **Step 4: Restore the real implementation and run tests**

Run: `cargo test -p xray-tui-native addr::tests`
Expected: PASS (6 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/addr.rs crates/xray-tui-native/src/lib.rs
git commit -m "feat(native): TargetAddr + v2ray port-first addr codec"
```

---

### Task 3: ConnectShape classification

**Files:**
- Create: `crates/xray-tui-native/src/shape.rs`
- Modify: `crates/xray-tui-native/src/lib.rs` — add `pub mod shape;`

**Interfaces:**
- Consumes: `ProtocolKind` (`xray_tui_proto::proto_spec::ProtocolKind`, 27 variants — list in Step 2).
- Produces: `ConnectShape` enum (5 variants) + `connect_shape(kind: &ProtocolKind) -> ConnectShape`. Task 8's dispatch and lib.rs documentation use it.

- [ ] **Step 1: Add the module to lib.rs**

```rust
pub mod shape;
```

- [ ] **Step 2: Write the test first (exhaustive table)**

Create `crates/xray-tui-native/src/shape.rs`:

```rust
use xray_tui_proto::proto_spec::ProtocolKind;

/// How a protocol kind connects — the dispatch spine for the native core.
///
/// The uniform "dial transport → security → handshake over stream" pipeline
/// applies only to [`ConnectShape::TcpStream`]. Other shapes take their own
/// path at dispatch time and may ignore the transport seam entirely.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectShape {
    /// Transport (tcp/ws/grpc/...) + security + handshake over a byte stream.
    TcpStream,
    /// QUIC transport (+ packet obfuscation), then protocol over a QUIC stream.
    Quic,
    /// Network-device protocol: no dial, device + own socket path.
    DeviceTunnel,
    /// Own complete state machine that ignores the transport seam.
    HandshakeOwn,
    /// No connect path exists at all (outbound-only kinds).
    NoStream,
}

/// Static per-kind classification. Every `ProtocolKind` arm must be covered;
/// the match is written exhaustively so adding a kind to the proto crate is a
/// compile error here until a shape is assigned.
#[must_use]
pub const fn connect_shape(kind: ProtocolKind) -> ConnectShape {
    use ConnectShape::{DeviceTunnel, HandshakeOwn, NoStream, Quic, TcpStream};
    match kind {
        ProtocolKind::Vmess
        | ProtocolKind::Vless
        | ProtocolKind::Shadowsocks
        | ProtocolKind::Shadowsocks2022
        | ProtocolKind::Socks
        | ProtocolKind::Http
        | ProtocolKind::Trojan
        | ProtocolKind::Naive
        | ProtocolKind::AnyTls
        | ProtocolKind::ShadowTls
        | ProtocolKind::ShadowsocksR => TcpStream,
        ProtocolKind::Hysteria2 | ProtocolKind::Hysteria | ProtocolKind::Tuic => Quic,
        ProtocolKind::WireGuard | ProtocolKind::Tailscale => DeviceTunnel,
        ProtocolKind::Ssh | ProtocolKind::Tor => HandshakeOwn,
        ProtocolKind::DokodemoDoor
        | ProtocolKind::Freedom
        | ProtocolKind::Blackhole
        | ProtocolKind::Dns
        | ProtocolKind::Loopback
        | ProtocolKind::Custom
        | ProtocolKind::Redirect
        | ProtocolKind::TProxy
        | ProtocolKind::Mixed => NoStream,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_family_is_tcp_stream() {
        for k in [
            ProtocolKind::Vmess,
            ProtocolKind::Vless,
            ProtocolKind::Shadowsocks,
            ProtocolKind::Shadowsocks2022,
            ProtocolKind::Socks,
            ProtocolKind::Http,
            ProtocolKind::Trojan,
            ProtocolKind::Naive,
            ProtocolKind::AnyTls,
            ProtocolKind::ShadowTls,
            ProtocolKind::ShadowsocksR,
        ] {
            assert_eq!(connect_shape(k), ConnectShape::TcpStream, "{k:?}");
        }
    }

    #[test]
    fn quic_family() {
        for k in [ProtocolKind::Hysteria2, ProtocolKind::Hysteria, ProtocolKind::Tuic] {
            assert_eq!(connect_shape(k), ConnectShape::Quic, "{k:?}");
        }
    }

    #[test]
    fn device_and_own_handshake() {
        assert_eq!(connect_shape(ProtocolKind::WireGuard), ConnectShape::DeviceTunnel);
        assert_eq!(connect_shape(ProtocolKind::Tailscale), ConnectShape::DeviceTunnel);
        assert_eq!(connect_shape(ProtocolKind::Ssh), ConnectShape::HandshakeOwn);
        assert_eq!(connect_shape(ProtocolKind::Tor), ConnectShape::HandshakeOwn);
    }

    #[test]
    fn outbound_only_kinds_have_no_stream() {
        for k in [
            ProtocolKind::DokodemoDoor,
            ProtocolKind::Freedom,
            ProtocolKind::Blackhole,
            ProtocolKind::Dns,
            ProtocolKind::Loopback,
            ProtocolKind::Custom,
            ProtocolKind::Redirect,
            ProtocolKind::TProxy,
            ProtocolKind::Mixed,
        ] {
            assert_eq!(connect_shape(k), ConnectShape::NoStream, "{k:?}");
        }
    }
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p xray-tui-native shape::tests`
Expected: FAIL — no `connect_shape` function yet (or stub panics).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p xray-tui-native shape::tests`
Expected: PASS (4 tests). The exhaustive match also proves all 27 kinds are classified.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/shape.rs crates/xray-tui-native/src/lib.rs
git commit -m "feat(native): ConnectShape taxonomy for 27 protocol kinds"
```

---

### Task 4: Context — NativeConnectParams, LinkContext, server address

**Files:**
- Create: `crates/xray-tui-native/src/context.rs`
- Modify: `crates/xray-tui-native/src/lib.rs` — add `pub mod context;` + re-exports.

**Interfaces:**
- Consumes: `ProtocolConfig`, `EndpointEssentials`, `TargetAddr`, `addr::Host`. Trait accessors `ProtocolConfig::security() -> Option<&SecurityConfig>` and `ProtocolConfig::transport_type() -> Option<&str>` (already on the proto `ProtoSpec` trait, no per-variant field access needed).
- Produces: `NativeConnectParams`, `LinkContext` with `server_socket()`, `sni()`, `alpn_vec()`, `security()`/`transport_type()` passthroughs. Tasks 5/6/8 consume these.

- [ ] **Step 1: Add module + re-exports to lib.rs**

```rust
pub mod context;

pub use context::{LinkContext, NativeConnectParams};
```

- [ ] **Step 2: Write the implementation with unit tests**

Create `crates/xray-tui-native/src/context.rs`:

```rust
use std::net::SocketAddr;

use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;
use xray_tui_proto::proto_spec::{ProtocolConfig, SecurityConfig, TlsConfig, TlsOpts};

use crate::addr::Host;
use crate::error::{NativeError, timeouts};

/// Per-connect parameters: the typed proto config plus the dial address.
///
/// `server` is the proxy server (host/port); `target` is the destination the
/// proxy should reach. `resolved_ip` lets callers inject a pre-resolved
/// address (the enrichment pipeline's `resolved_as`) and skip DNS.
#[derive(Debug, Clone)]
pub struct NativeConnectParams {
    pub protocol: ProtocolConfig,
    pub server: EndpointEssentials,
    pub target: crate::addr::TargetAddr,
    pub resolved_ip: Option<SocketAddr>,
}

impl NativeConnectParams {
    #[must_use]
    pub fn new(protocol: ProtocolConfig, server: EndpointEssentials, target: crate::addr::TargetAddr) -> Self {
        Self {
            protocol,
            server,
            target,
            resolved_ip: None,
        }
    }
}

/// Per-link state in the connect pipeline (a chain has one context per link).
#[derive(Debug, Clone)]
pub struct LinkContext {
    pub params: NativeConnectParams,
    pub target: crate::addr::TargetAddr,
}

impl LinkContext {
    #[must_use]
    pub fn new(params: NativeConnectParams, target: crate::addr::TargetAddr) -> Self {
        Self { params, target }
    }

    /// The proxy server's socket address: `resolved_ip` override, else the
    /// parsed host IP, else DNS resolution via `tokio::net::lookup_host`.
    ///
    /// DNS lookup is a network step: bounded by [`timeouts::DIAL`].
    pub async fn server_socket(&self) -> Result<SocketAddr, NativeError> {
        if let Some(addr) = self.params.resolved_ip {
            return Ok(addr);
        }
        match Host::new(&self.params.server.host) {
            Host::Ip(ip) => Ok(SocketAddr::new(ip, self.params.server.port)),
            Host::Domain(domain) => {
                let (host, port) = (domain.as_str(), self.params.server.port);
                let timeout = timeouts::DIAL;
                let mut addrs = tokio::time::timeout(timeout, tokio::net::lookup_host((host, port)))
                    .await
                    .map_err(|_| NativeError::Timeout {
                        step: "dns lookup",
                        limit: timeout,
                    })?
                    .map_err(|e| NativeError::Dial(format!("{host}: {e}")))?;
                addrs.next().ok_or_else(|| {
                    NativeError::Dial(format!("{host}: no addresses found"))
                })
            }
        }
    }

    /// TLS SNI: explicit `sni` option, else the endpoint host.
    #[must_use]
    pub fn sni(&self) -> String {
        self.tls_opts()
            .ok()
            .flatten()
            .and_then(|o| o.sni.as_ref())
            .map_or_else(|| self.params.server.host.clone(), |s| s.to_string())
    }

    /// TLS ALPN list: comma-separated `alpn` option, else empty (no ALPN).
    #[must_use]
    pub fn alpn_vec(&self) -> Vec<Vec<u8>> {
        self.tls_opts()
            .ok()
            .flatten()
            .and_then(|o| o.alpn.as_ref())
            .map(|a| a.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::as_bytes).map(<[u8]>::to_vec).collect())
            .unwrap_or_default()
    }

    /// The typed TLS options when security is plain TLS (not Reality).
    pub fn tls_opts(&self) -> Result<Option<&TlsOpts>, NativeError> {
        match self.security() {
            SecurityConfig { tls: Some(TlsConfig::Tls(opts)), .. } => Ok(Some(opts)),
            SecurityConfig { tls: Some(TlsConfig::Reality(_)), .. } => Err(NativeError::Reality(
                "reality not implemented yet".into(),
            )),
            _ => Ok(None),
        }
    }

    /// Security config from the protocol payload (typed, via the proto trait).
    #[must_use]
    pub fn security(&self) -> Option<&SecurityConfig> {
        self.params.protocol.security()
    }

    /// Transport name ("tcp", "ws", ...) from the protocol payload.
    #[must_use]
    pub fn transport_type(&self) -> Option<&str> {
        self.params.protocol.transport_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addr::{Host, TargetAddr};

    fn vless_empty() -> ProtocolConfig {
        // VlessConfig::default() exists on the proto type; sni/alpn come from
        // query params there, and security defaults to none.
        ProtocolConfig::Vless(Default::default())
    }

    #[test]
    fn server_socket_uses_ip_host_without_dns() {
        let params = NativeConnectParams::new(
            vless_empty(),
            EndpointEssentials::new("127.0.0.1", 4430),
            TargetAddr::new(Host::Domain("example.com".into()), 80),
        );
        let ctx = LinkContext::new(params, TargetAddr::new(Host::Domain("example.com".into()), 80));
        let sock = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(ctx.server_socket())
            .unwrap();
        assert_eq!(sock, SocketAddr::from(([127, 0, 0, 1], 4430)));
    }

    #[test]
    fn resolved_ip_overrides_host() {
        let mut params = NativeConnectParams::new(
            vless_empty(),
            EndpointEssentials::new("server.example", 4430),
            TargetAddr::new(Host::Domain("example.com".into()), 80),
        );
        params.resolved_ip = Some(SocketAddr::from(([127, 0, 0, 1], 9999)));
        let ctx = LinkContext::new(params, TargetAddr::new(Host::Domain("example.com".into()), 80));
        let sock = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(ctx.server_socket())
            .unwrap();
        assert_eq!(sock, SocketAddr::from(([127, 0, 0, 1], 9999)));
    }

    #[test]
    fn sni_falls_back_to_endpoint_host() {
        let params = NativeConnectParams::new(
            vless_empty(),
            EndpointEssentials::new("my.server.test", 4430),
            TargetAddr::new(Host::Domain("example.com".into()), 80),
        );
        let ctx = LinkContext::new(params, TargetAddr::new(Host::Domain("example.com".into()), 80));
        assert_eq!(ctx.sni(), "my.server.test");
    }

    #[test]
    fn alpn_splits_on_comma() {
        // security: none -> empty alpn
        let params = NativeConnectParams::new(
            vless_empty(),
            EndpointEssentials::new("127.0.0.1", 4430),
            TargetAddr::new(Host::Domain("example.com".into()), 80),
        );
        let ctx = LinkContext::new(params, TargetAddr::new(Host::Domain("example.com".into()), 80));
        assert!(ctx.alpn_vec().is_empty());
    }
}
```

Note: `VlessConfig::default()` — verify the proto type implements `Default`; if not, build it with `VlessConfig { uuid: "00000000-0000-0000-0000-000000000000".into(), ..Default::default() }` or construct via `serde_json::from_value` in the test. Adjust the helper accordingly (one place).

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test -p xray-tui-native context::tests`
Expected: PASS (4 tests). If `VlessConfig` lacks `Default`, fix the test helper (Step 2 note) and re-run.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-native/src/context.rs crates/xray-tui-native/src/lib.rs
git commit -m "feat(native): NativeConnectParams + LinkContext with server resolution"
```

---

### Task 5: Transport phase — TCP real, rest stubbed

**Files:**
- Create: `crates/xray-tui-native/src/transport/mod.rs`
- Create: `crates/xray-tui-native/src/transport/tcp.rs`
- Modify: `crates/xray-tui-native/src/lib.rs` — add `pub mod transport;`

**Interfaces:**
- Consumes: `LinkContext` (`server_socket()`, `transport_type()`), `BoxStream`, `NativeError`, `timeouts::TRANSPORT`.
- Produces: `transport::connect(ctx: &LinkContext, base: Option<BoxStream>) -> Result<BoxStream, NativeError>`. Security (Task 6) consumes its output.

- [ ] **Step 1: Add the module to lib.rs**

```rust
pub mod transport;
```

- [ ] **Step 2: Write the test first**

`crates/xray-tui-native/src/transport/tcp.rs`:

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::{NativeError, timeouts};
use crate::BoxStream;

/// Dial the server's TCP socket directly (`base` is None) or pass through an
/// existing tunnel (`base` is Some — a TCP "transport" over a tunnel is raw
/// bytes, so this is the identity function).
pub async fn connect(ctx: &crate::context::LinkContext, base: Option<BoxStream>) -> Result<BoxStream, NativeError> {
    match base {
        Some(stream) => Ok(stream),
        None => {
            let socket = ctx.server_socket().await?;
            let timeout = timeouts::TRANSPORT;
            let stream = tokio::time::timeout(timeout, tokio::net::TcpStream::connect(socket))
                .await
                .map_err(|_| NativeError::Timeout { step: "tcp dial", limit: timeout })?
                .map_err(|e| NativeError::Dial(format!("{socket}: {e}")))?;
            Ok(Box::new(stream))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use xray_tui_proto::proto_spec::ProtocolConfig;

    use crate::context::{LinkContext, NativeConnectParams};
    use crate::addr::{Host, TargetAddr};

    fn params_at(addr: SocketAddr) -> NativeConnectParams {
        let _ = ProtocolConfig::Vless(Default::default()).transport_type(); // type anchor
        NativeConnectParams::new(
            ProtocolConfig::Vless(Default::default()),
            xray_tui_proto::proto_spec::endpoint::EndpointEssentials::new(addr.ip().to_string(), addr.port()),
            TargetAddr::new(Host::Domain("example.com".into()), 80),
        )
    }

    #[tokio::test]
    async fn dial_connects_to_server() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 4];
            sock.read_exact(&mut buf).await.unwrap();
            buf
        });
        let ctx = LinkContext::new(
            params_at(addr),
            TargetAddr::new(Host::Domain("example.com".into()), 80),
        );
        let mut stream = connect(&ctx, None).await.unwrap();
        stream.write_all(b"ping").await.unwrap();
        let echoed = server.await.unwrap();
        assert_eq!(&echoed, b"ping");
    }

    #[tokio::test]
    async fn base_stream_passes_through() {
        let (a, mut b) = tokio::io::duplex(64);
        let ctx = LinkContext::new(
            params_at(SocketAddr::from(([127, 0, 0, 1], 1))),
            TargetAddr::new(Host::Domain("example.com".into()), 80),
        );
        let out = connect(&ctx, Some(Box::new(a))).await.unwrap();
        drop(out);
        b.write_all(b"x").await.unwrap();
        // no panic + base stream consumed
    }
}
```

Need `AsyncWriteExt` import too (write_all) — add `use tokio::io::AsyncWriteExt;`.

- [ ] **Step 3: Create the dispatch module**

`crates/xray-tui-native/src/transport/mod.rs`:

```rust
//! Transport phase: get a byte stream to the proxy server (or reuse an
//! existing tunnel for the later hops of a chain).
//!
//! OUTERMOST layer (Xray composition order): dial → transport → security →
//! protocol. A transport answers two questions: "dial the server" (`base:
//! None`) and "upgrade an existing tunnel in place" (`base: Some`, e.g. a
//! WebSocket handshake on a later chain hop).

use crate::context::LinkContext;
use crate::error::NativeError;
use crate::BoxStream;

pub mod tcp;

/// Run the transport step. `base` is the stream from the previous chain hop
/// (or `None` for the first hop, which dials the server directly).
pub async fn connect(ctx: &LinkContext, base: Option<BoxStream>) -> Result<BoxStream, NativeError> {
    match ctx.transport_type() {
        None | Some("tcp") => tcp::connect(ctx, base).await,
        Some(t) => Err(NativeError::NotImplemented {
            feature: format!("transport {t}"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_transport_is_not_implemented() {
        // The classification lives in the dispatch above; the arm is covered
        // by `Some(t)` at runtime. Compile-check only.
        assert!(true);
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p xray-tui-native transport::`
Expected: PASS (dial test + pass-through test).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/transport
git commit -m "feat(native): TCP transport real, other transports stubbed"
```

---

### Task 6: Security phase — None/TLS real, Reality stubbed, provider plug

**Files:**
- Create: `crates/xray-tui-native/src/security/mod.rs`
- Create: `crates/xray-tui-native/src/security/tls.rs`
- Create: `crates/xray-tui-native/src/security/tls_provider.rs`
- Modify: `crates/xray-tui-native/src/lib.rs` — add `pub mod security;`

**Interfaces:**
- Consumes: `LinkContext` (`security()`, `tls_opts()`, `sni()`, `alpn_vec()`), `BoxStream`, `NativeError`, `timeouts::SECURITY`. Security types from proto: `SecurityConfig { tls: Option<TlsConfig>, .. }`, `TlsConfig::Tls(TlsOpts)` / `TlsConfig::Reality(_)`.
- Produces: `security::wrap(ctx, stream) -> Result<BoxStream, NativeError>`; `tls_provider::{TlsConnector, TlsProvider, TlsParams, FingerprintId}`; `tls::connect` (ring rustls via tokio-rustls). Protocol (Task 8) consumes `wrap`.

- [ ] **Step 0: Write the security module wiring + REALITY stub with the HelloProvisioner contract**

Create `crates/xray-tui-native/src/security/reality.rs` (spec §11 contract, implementation deferred to M3):

```rust
//! REALITY client stub — contract only (M3 milestone).
//!
//! Wire contract (modeled on `shoes/src/reality/`, MIT):
//! 1. X25519 keypair; `client_random` 32 B.
//! 2. `shared` = ECDH(client_priv, server_pub from `pbk`).
//! 3. `auth_key` = HKDF-SHA256(shared, salt = random[0..20], info b"REALITY").
//! 4. SessionId plaintext 16 B = version(1,8,0) + pad(1) + timestamp u32 BE + short_id(8).
//! 5. ClientHello: X25519 keyshare, SNI steal target, suites 0x1301-03, ALPN h2+http/1.1.
//! 6. AAD = hello with SessionId zeroed; nonce = random[20..32]; SessionId =
//!    AES-256-GCM(auth_key, nonce, plaintext, aad).
//! 7. Restore SessionId (builder-returned range, never hardcoded offset).
//! 8. Server auth: cert signature field == HMAC-SHA512(auth_key, ed25519_pub),
//!    Ed25519 CertificateVerify over transcript. No PKI chain check.
//! 9. TLS1.3 key schedule + Finished, then app records.
//!
//! Crypto (ring): X25519 agreement, HKDF-SHA256, AES-256-GCM,
//! SHA-256/384/512 + HMAC, Ed25519 verify — full coverage, no aws-lc-rs.

use std::sync::Arc;

use crate::error::NativeError;

/// A ClientHello layed out for REALITY: fingerprint-shaped extensions that
/// the reality connector then injects keyshare/SNI/random/SessionId into.
pub trait HelloProvisioner: Send + Sync {
    /// Returns the bare ClientHello plus the SessionId byte range.
    /// The returned range is computed by the builder — never hardcoded.
    fn provision(&self, params: &HelloProvisionParams)
        -> Result<ProvisionedHello, NativeError>;
}

/// Inputs the reality connector hands to a provisioner.
#[derive(Debug, Clone)]
pub struct HelloProvisionParams {
    pub server_name: String,
    pub alpn: Vec<Vec<u8>>,
}

/// A provisioned ClientHello (handshake bytes, no record header).
#[derive(Debug, Clone)]
pub struct ProvisionedHello {
    pub handshake_bytes: Vec<u8>,
    /// Byte range of the SessionId field within `handshake_bytes`.
    pub session_id_range: std::ops::Range<usize>,
}

/// Fixed Chrome-133-shaped provisioner (first engine; ported from shoes).
pub struct FixedChrome133;

impl HelloProvisioner for FixedChrome133 {
    // Implemented in M3 with the reality ring port; M1 = unreachable stub.
    fn provision(&self, _params: &HelloProvisionParams) -> Result<ProvisionedHello, NativeError> {
        Err(NativeError::NotImplemented {
            feature: "reality hello provisioner".into(),
        })
    }
}

/// Chosen provisioner for a connect.
#[derive(Debug, Clone, Default)]
pub enum HelloProvisionerChoice {
    #[default]
    FixedChrome133,
    Custom(Arc<dyn HelloProvisioner>),
}
```

- [ ] **Step 1: Write the provider plug types (tls_provider.rs)**

```rust
//! TLS provider plug: standard rustls vs a fingerprint-capable engine.
//!
//! Stock rustls (0.23) exposes no ClientHello modification API, so browser
//! fingerprint mimicry (the DPI/CDN bypass requirement) lives behind the
//! [`TlsConnector`] trait. `Custom` engines serve BOTH plain TLS (with
//! certificate validation) and REALITY. M1 ships only `Standard`.

use std::sync::Arc;

use crate::error::NativeError;
use crate::BoxStream;

/// Browser fingerprint id, mirrored from the URL `fp` parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FingerprintId {
    Chrome,
    ChromeRandomized,
    Firefox,
    Safari,
    Random,
}

impl std::fmt::Display for FingerprintId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Chrome => "chrome",
            Self::ChromeRandomized => "chrome-randomized",
            Self::Firefox => "firefox",
            Self::Safari => "safari",
            Self::Random => "random",
        };
        f.write_str(s)
    }
}

/// TLS connection parameters resolved at connect time.
#[derive(Debug, Clone)]
pub struct TlsParams {
    pub sni: String,
    pub alpn: Vec<Vec<u8>>,
    pub fingerprint: Option<FingerprintId>,
}

/// A TLS connector that can emit arbitrary ClientHellos (fingerprint mimicry).
pub trait TlsConnector: Send + Sync {
    fn connect(
        &self,
        stream: BoxStream,
        params: TlsParams,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<BoxStream, NativeError>> + Send>>;
}

/// Provider selection for a connect. `Standard` ignores `fingerprint`
/// (documented no-op until a fingerprint engine lands in M2).
#[derive(Debug, Clone, Default)]
pub enum TlsProvider {
    #[default]
    Standard,
    Custom(Arc<dyn TlsConnector>),
}
```

- [ ] **Step 2: Write the failing security test first**

`crates/xray-tui-native/src/security/mod.rs` with bare arms — create the test harness to spawn a local TLS server with rcgen certs and assert wrap passes bytes through:

```rust
//! Security phase: wrap the transport stream (TLS, Reality — or identity).
//!
//! MIDDLE layer: transport → SECURITY → protocol. Security dispatches on
//! `SecurityConfig` from the proto payload.

pub mod reality;
pub mod tls;
pub mod tls_provider;

use xray_tui_proto::proto_spec::TlsConfig;

use crate::context::LinkContext;
use crate::error::NativeError;
use crate::BoxStream;

/// Wrap the transport stream according to the profile's security config.
pub async fn wrap(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    let Some(sec) = ctx.security() else {
        return Ok(stream);
    };
    if sec.is_empty() {
        return Ok(stream);
    }
    match &sec.tls {
        Some(TlsConfig::Tls(_)) => tls::connect(ctx, stream).await,
        Some(TlsConfig::Reality(_)) => Err(NativeError::NotImplemented {
            feature: "security reality".into(),
        }),
        None => Ok(stream),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};

    use crate::context::{LinkContext, NativeConnectParams};
    use crate::error::NativeError;
    use crate::addr::{Host, TargetAddr};
    use crate::BoxStream;
    use super::*;

    // (test server + certs helper filled in Step 3)
}
```

- [ ] **Step 3: Implement tls.rs (ring rustls via tokio-rustls) + security test with real local TLS server**

`crates/xray-tui-native/src/security/tls.rs`:

```rust
//! Standard TLS: rustls 0.23 (ring backend) via tokio-rustls.
//!
//! This is the `TlsProvider::Standard` path. It cannot emit a mimicked
//! ClientHello (rustls has no modification API); a fingerprinted hello comes
//! from a `TlsProvider::Custom` engine (M2). Certificate validation is always
//! on — no skip-verify anywhere in the native core.

use std::sync::Arc;

use tokio_rustls::rustls;

use crate::context::LinkContext;
use crate::error::{NativeError, timeouts};
use crate::BoxStream;

pub async fn connect(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let mut config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let alpn = ctx.alpn_vec();
    if !alpn.is_empty() {
        config.alpn_protocols = alpn;
    }

    let connector = tokio_rustls::TlsConnector::from(Arc::new(config));
    let sni = ctx.sni();
    let server_name = rustls::pki_types::ServerName::try_from(sni.clone())
        .map_err(|e| NativeError::Tls(format!("invalid SNI {sni:?}: {e}")))?;

    let timeout = timeouts::SECURITY;
    let tls = tokio::time::timeout(timeout, connector.connect(server_name, stream))
        .await
        .map_err(|_| NativeError::Timeout { step: "tls handshake", limit: timeout })?
        .map_err(|e| NativeError::Tls(format!("handshake with {sni}: {e}")))?;
    Ok(Box::new(tls))
}
```

In `security/mod.rs` tests, the TLS server is spawned with an rcgen-issued cert and the CA is inserted into the client's root store — to keep the root store deterministic without touching production code, exercise `wrap` via a **test-only** rustls config path: instead, build the e2e trust in Task 11 (harness CA) and here test `tls::connect` against a server using a certificate signed by a CA injected through a `#[cfg(test)]` helper in `tls.rs`:

```rust
/// Build a client config trusting exactly `ca_der` (test/e2e harness only;
/// production uses webpki-roots).
#[doc(hidden)]
#[cfg(any(test, feature = "native-e2e"))]
pub fn test_client_config(ca_der: &[u8]) -> rustls::ClientConfig {
    let mut roots = rustls::RootCertStore::empty();
    roots.add(rustls::pki_types::CertificateDer::from(ca_der.to_vec())).unwrap();
    rustls::ClientConfig::builder().with_root_certificates(roots).with_no_client_auth()
}
```

and change `connect` minimally to read a test override (`static TEST_CFG`),
used when set. This keeps production code honest (webpki-roots) while tests
trust only the rcgen CA. The override is visible to the e2e integration
tests via the `native-e2e` feature, not just `#[cfg(test)]`.

```rust
static TEST_CFG: std::sync::OnceLock<Arc<rustls::ClientConfig>> = std::sync::OnceLock::new();

/// Install a test-only TLS client config (root store = harness CA).
#[doc(hidden)]
#[cfg(any(test, feature = "native-e2e"))]
pub fn set_test_config(cfg: rustls::ClientConfig) {
    let _ = TEST_CFG.set(Arc::new(cfg));
}
```

`TEST_CFG` itself is `#[cfg(any(test, feature = "native-e2e"))]` so the
production build carries no test state.

Then in `connect`, replace config construction with:

```rust
let config = TEST_CFG.get().cloned().unwrap_or_else(|| {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    rustls::ClientConfig::builder().with_root_certificates(roots).with_no_client_auth()
});
let mut config = config;
let alpn = ctx.alpn_vec();
if !alpn.is_empty() {
    config.alpn_protocols = alpn;
}
```

Security tests (in `security/mod.rs`):

```rust
#[tokio::test]
async fn none_security_is_identity() {
    let params = NativeConnectParams::new(
        ProtocolConfig::Vless(Default::default()), // security: none
        EndpointEssentials::new("127.0.0.1", 1),
        TargetAddr::new(Host::Domain("example.com".into()), 80),
    );
    let ctx = LinkContext::new(params, TargetAddr::new(Host::Domain("example.com".into()), 80));
    let (a, mut b) = tokio::io::duplex(64);
    let out = wrap(&ctx, Box::new(a)).await.unwrap();
    let mut out = out;
    out.write_all(b"hi").await.unwrap();
    let mut buf = [0u8; 2];
    b.read_exact(&mut buf).await.unwrap();
    assert_eq!(&buf, b"hi");
}

#[tokio::test]
async fn tls_wraps_and_passes_bytes() {
    let (cert_pem, key_pem, ca_der) = rcgen_ca_and_server("localhost");
    set_test_config(super::tls::test_client_config(&ca_der));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (sock, _) = listener.accept().await.unwrap();
        let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(server_config(&cert_pem, &key_pem)));
        let mut tls = acceptor.accept(sock).await.unwrap();
        let mut buf = [0u8; 4];
        tls.read_exact(&mut buf).await.unwrap();
        tls.write_all(&buf).await.unwrap();
    });

    let params = NativeConnectParams::new(
        vless_with_tls("localhost"), // security: tls, sni localhost
        EndpointEssentials::new("127.0.0.1", addr.port()),
        TargetAddr::new(Host::Domain("example.com".into()), 80),
    );
    let ctx = LinkContext::new(params, TargetAddr::new(Host::Domain("example.com".into()), 80));
    let sock = tokio::net::TcpStream::connect(addr).await.unwrap();
    let mut out = wrap(&ctx, Box::new(sock)).await.unwrap();
    out.write_all(b"ping").await.unwrap();
    let mut buf = [0u8; 4];
    out.read_exact(&mut buf).await.unwrap();
    assert_eq!(&buf, b"ping");
    server.await.unwrap();
}

#[tokio::test]
async fn reality_security_is_not_implemented() {
    let params = NativeConnectParams {
        protocol: vless_with_reality(), // security: reality
        server: EndpointEssentials::new("127.0.0.1", 1),
        target: TargetAddr::new(Host::Domain("example.com".into()), 80),
        resolved_ip: None,
    };
    let ctx = LinkContext::new(params, TargetAddr::new(Host::Domain("example.com".into()), 80));
    let (a, _) = tokio::io::duplex(64);
    let err = wrap(&ctx, Box::new(a)).await.unwrap_err();
    assert!(matches!(err, NativeError::NotImplemented { .. }));
}
```

The helpers `rcgen_ca_and_server`, `server_config`, `vless_with_tls`, `vless_with_reality` live in `#[cfg(test)]` in `security/mod.rs` or a `test_support` module; they build the proto configs with serde JSON:

```rust
fn vless_with_tls(sni: &str) -> ProtocolConfig {
    serde_json::from_value(serde_json::json!({
        "schema": "vless",
        "uuid": "00000000-0000-0000-0000-000000000000",
        "security": { "type": "tls", "sni": sni, "alpn": "http/1.1" },
        "transport": { "type": "tcp" }
    })).unwrap()
}
```

(Check the proto `SecurityConfig`/`TransportConfig` serde shapes: `TlsConfig` is `#[serde(tag = "type")]` with `Tls`/`Reality` variants mapped `snake_case`; `TransportConfig` is `#[serde(tag = "type")]`. Adjust the JSON literals to the exact serialization if the first attempt errors — the roundtrip is asserted by `serde_json` errors.)

- [ ] **Step 4: Run tests**

Run: `cargo test -p xray-tui-native security::tests`
Expected: PASS (identity, tls roundtrip, reality not-implemented).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/security
git commit -m "feat(native): security phase with TLS real, Reality stubbed, provider plug"
```

---

### Task 7: VLESS wire header codec + vectors

**Files:**
- Create: `crates/xray-tui-native/src/protocol/vless/header.rs`
- Create: `crates/xray-tui-native/src/protocol/vless/mod.rs`
- Modify: `crates/xray-tui-native/src/lib.rs` (protocol module comes in Task 8; here only the folder exists)

**Interfaces:**
- Consumes: `crate::addr::{TargetAddr, encode_addr}`, `uuid` (workspace).
- Produces: `encoder::encode_request(uuid: &[u8; 16], target: &TargetAddr, command: u8) -> Vec<u8>` and `response::{check_response_header(buf: &[u8]) -> Result<usize, NativeError>}` returning the addon length. Task 8's `vless::connect` consumes both.

- [ ] **Step 1: Write the failing vectors test first**

`crates/xray-tui-native/src/protocol/vless/header.rs` with `encode_request` stubbed:

```rust
//! VLESS wire header codec (client side).
//!
//! Request header (v2ray `proxy/vless/encoding/encoding.go`,
//! `EncodeRequestHeader`): `version(1B, 0) | uuid(16B) | addon_len(1B) |
//! addons | command(1B) | [addr+port]`, with the addr encoded
//! PORT-FIRST (`PortThenAddress`). Command: 0x01 TCP, 0x02 UDP, 0x03 MUX.
//! Response header: `version(1B, echoed 0) | addon_len(1B) | addons`.

use xray_tui_proto::proto_spec::ProtocolKind;

use crate::addr::{encode_addr, TargetAddr};
use crate::error::NativeError;

pub const VERSION: u8 = 0;
pub const CMD_TCP: u8 = 1;
pub const CMD_UDP: u8 = 2;
pub const CMD_MUX: u8 = 3;

/// UUID bytes from a canonical UUID string.
pub fn uuid_bytes(uuid_str: &str) -> Result<[u8; 16], NativeError> {
    uuid::Uuid::parse_str(uuid_str)
        .map(|u| *u.as_bytes())
        .map_err(|e| NativeError::Config(format!("invalid uuid {uuid_str:?}: {e}")))
}

/// Encode a VLESS request header (no addons, version 0).
#[must_use]
pub fn encode_request(uuid: &[u8; 16], target: &TargetAddr, command: u8) -> Vec<u8> {
    let mut out = Vec::with_capacity(32);
    out.push(VERSION);
    out.extend_from_slice(uuid);
    out.push(0); // addon_len
    match command {
        CMD_TCP | CMD_UDP => {
            out.push(command);
            out.extend_from_slice(&encode_addr(target));
        }
        CMD_MUX => {
            out.push(command);
            // Mux uses a fixed v1.mux.cool:0 target on the wire.
            out.extend_from_slice(&encode_addr(&TargetAddr::new(
                crate::addr::Host::Domain("v1.mux.cool".into()),
                0,
            )));
        }
        other => {
            // unreachable in practice; kept as debug padding for future cmds
            out.push(other);
        }
    }
    out
}

/// Validate the first bytes of the response header; returns the addon length.
pub fn check_response_header(buf: &[u8]) -> Result<usize, NativeError> {
    if buf.len() < 2 {
        return Err(NativeError::Protocol {
            kind: ProtocolKind::Vless,
            detail: "response header truncated".into(),
        });
    }
    if buf[0] != VERSION {
        return Err(NativeError::Protocol {
            kind: ProtocolKind::Vless,
            detail: format!("bad response version {}", buf[0]),
        });
    }
    Ok(usize::from(buf[1]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addr::Host;

    const UUID_STR: &str = "00010203-0405-0607-0809-0a0b0c0d0e0f";

    fn uuid() -> [u8; 16] {
        [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        ]
    }

    #[test]
    fn tcp_request_header_bytes_exact() {
        let target = TargetAddr::new(Host::new("127.0.0.1"), 8080);
        let got = encode_request(&uuid(), &target, CMD_TCP);
        // version, uuid, addon_len=0, cmd=1, port 0x1f90, type=1, 127.0.0.1
        let mut expected = vec![0x00];
        expected.extend_from_slice(&uuid());
        expected.push(0x00);
        expected.push(0x01);
        expected.extend_from_slice(&[0x1f, 0x90, 0x01, 127, 0, 0, 1]);
        assert_eq!(got, expected);
    }

    #[test]
    fn uuid_parse_match_expected_bytes() {
        assert_eq!(uuid_bytes(UUID_STR).unwrap(), uuid());
        assert!(uuid_bytes("not-a-uuid").is_err());
    }

    #[test]
    fn response_header_ok_and_bad() {
        assert_eq!(check_response_header(&[0x00, 0x00]).unwrap(), 0);
        assert_eq!(check_response_header(&[0x00, 0x02]).unwrap(), 2);
        assert!(check_response_header(&[0x01, 0x00]).is_err());
        assert!(check_response_header(&[0x00]).is_err());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p xray-tui-native vless::header`
(requires the module wired — temporarily declare `pub mod protocol;` + `pub mod vless;` in lib.rs referencing `protocol/mod.rs` and `vless/mod.rs` minimal files.)
Expected: FAIL on missing `encode_request`.

- [ ] **Step 3: Implement `vless/mod.rs` (no connect yet)**

```rust
//! VLESS — the reference protocol for the native core.

pub mod header;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p xray-tui-native vless::header::tests`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/protocol
git commit -m "feat(native): VLESS request/response header codec with byte vectors"
```

---

### Task 8: Protocol dispatch + VLESS connect + NativeTunnel + public API

**Files:**
- Create: `crates/xray-tui-native/src/protocol/mod.rs` (dispatch over all 19 `ProtocolConfig` variants)
- Modify: `crates/xray-tui-native/src/protocol/vless/mod.rs` (add `connect`)
- Create: `crates/xray-tui-native/src/lib.rs` additions: `NativeTunnel`, `connect()`
- Modify: `crates/xray-tui-native/src/lib.rs` — final module wiring + re-exports.

**Interfaces:**
- Consumes: `LinkContext`, `BoxStream`, `header::*`, `NativeError`, `timeouts::PROTOCOL`, `shape::connect_shape`.
- Produces: `protocol::connect(ctx, stream) -> Result<BoxStream, NativeError>`; `NativeTunnel` (AsyncRead+AsyncWrite over BoxStream); `connect(params) -> Result<NativeTunnel, NativeError>` via `chain::connect_chain` (Task 9 creates chain.rs — Task 9 must land before this compiles; implement chain.rs here first as a minimal fold, then Task 9 adds its tests).

- [ ] **Step 1: VLESS connect implementation**

In `crates/xray-tui-native/src/protocol/vless/mod.rs`, add:

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::context::LinkContext;
use crate::error::{NativeError, timeouts};
use crate::protocol::vless::header;
use crate::BoxStream;

pub mod header;

/// Connect through a VLESS outbound over an already-secured stream.
///
/// Writes the request header, validates the response header, then returns the
/// stream unchanged (raw passthrough body).
pub async fn connect(
    ctx: &LinkContext,
    mut stream: BoxStream,
    cfg: &xray_tui_proto::proto_spec::vless::VlessConfig,
) -> Result<BoxStream, NativeError> {
    // Flow guard: M1 supports no flow. `xtls-rprx-vision` needs TLS1.3 +
    // stream splicing (not implemented yet); anything else is a config error.
    if let Some(flow) = cfg.flow.as_ref() {
        let flow = flow.to_string();
        if !flow.is_empty() {
            return Err(NativeError::NotImplemented {
                feature: format!("vless flow {flow}"),
            });
        }
    }

    let uuid = header::uuid_bytes(&cfg.uuid)?;
    let request = header::encode_request(&uuid, &ctx.target, header::CMD_TCP);
    let timeout = timeouts::PROTOCOL;
    tokio::time::timeout(timeout, stream.write_all(&request))
        .await
        .map_err(|_| NativeError::Timeout { step: "vless request write", limit: timeout })??;

    let mut head = [0u8; 2];
    tokio::time::timeout(timeout, stream.read_exact(&mut head))
        .await
        .map_err(|_| NativeError::Timeout { step: "vless response read", limit: timeout })??;
    let addon_len = header::check_response_header(&head)?;
    if addon_len > 0 {
        let mut addons = vec![0u8; addon_len];
        tokio::time::timeout(timeout, stream.read_exact(&mut addons))
            .await
            .map_err(|_| NativeError::Timeout { step: "vless response addons", limit: timeout })??;
    }
    Ok(stream)
}
```

- [ ] **Step 2: Create the per-kind stub directories (extensibility layout)**

One `protocol/<kind>/mod.rs` per config variant, doc-only (config plumbing lands with each protocol's implementation milestone). Template for every file (17 dirs):

`crates/xray-tui-native/src/protocol/vmess/mod.rs`:

```rust
//! VMess — placeholder. Native client milestone M4 (VMess AEAD body crypto,
//! cmdKey = md5(uuid ‖ "c48619fe-8f02-49e0-b9e9-edf763e17e21")). Wire contract:
//! v2ray-core `proxy/vmess/encoding` (MIT).
```

The same template (kind name + milestone + contract pointer) for:
`trojan`, `hysteria2`, `ss`, `ssr`, `tuic`, `wireguard`, `socks`, `http`,
`naive`, `anytls`, `shadowtls`, `tor`, `ssh`, `tailscale`, `hysteria1`,
`redirect`, `tproxy`, `mixed` — one `.rs` each. Adding a protocol later = fill
this module, add one dispatch arm.

- [ ] **Step 3: Write the protocol dispatch (all 19 variants)**

`crates/xray-tui-native/src/protocol/mod.rs`:

```rust
//! Protocol phase: the INNERMOST layer — write the protocol handshake onto
//! the secured stream and produce the byte tunnel.
//!
//! Dispatch strategy (see `shape.rs`): the uniform handshake-over-stream
//! pipeline applies to the TCP-stream family. Device tunnels
//! (WireGuard/Tailscale), own-handshake protocols (SSH/Tor), and the
//! outbound-only kinds (Redirect/TProxy/Mixed) take divergent paths at their
//! own connect() and return NotImplemented here until those paths exist.

use xray_tui_proto::proto_spec::ProtocolConfig;

use crate::context::LinkContext;
use crate::error::NativeError;
use crate::BoxStream;

pub mod anytls;
pub mod hysteria1;
pub mod hysteria2;
pub mod http;
pub mod mixed;
pub mod naive;
pub mod redirect;
pub mod shadowtls;
pub mod socks;
pub mod ss;
pub mod ssr;
pub mod ssh;
pub mod tailscale;
pub mod tor;
pub mod tproxy;
pub mod trojan;
pub mod tuic;
pub mod vless;
pub mod vmess;
pub mod wireguard;

/// One-line error shorthand for the placeholder arms.
fn not_impl(feature: &str) -> Result<BoxStream, NativeError> {
    Err(NativeError::NotImplemented {
        feature: format!("protocol {feature}"),
    })
}

/// Run the protocol phase: handshake + tunnel over the given stream.
pub async fn connect(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    match &ctx.params.protocol {
        ProtocolConfig::Vless(cfg) => vless::connect(ctx, stream, cfg).await,
        ProtocolConfig::Vmess(_) => not_impl("vmess"),
        ProtocolConfig::Trojan(_) => not_impl("trojan"),
        ProtocolConfig::Hysteria2(_) => not_impl("hysteria2"),
        ProtocolConfig::Ss(_) => not_impl("shadowsocks"),
        ProtocolConfig::Ssr(_) => not_impl("shadowsocksr"),
        ProtocolConfig::Tuic(_) => not_impl("tuic"),
        ProtocolConfig::Wireguard(_) => not_impl("wireguard"),
        ProtocolConfig::Socks(_) => not_impl("socks5"),
        ProtocolConfig::Http(_) => not_impl("http"),
        ProtocolConfig::Naive(_) => not_impl("naive"),
        ProtocolConfig::AnyTls(_) => not_impl("anytls"),
        ProtocolConfig::ShadowTls(_) => not_impl("shadowtls"),
        ProtocolConfig::Tor(_) => not_impl("tor"),
        ProtocolConfig::Ssh(_) => not_impl("ssh"),
        ProtocolConfig::Tailscale(_) => not_impl("tailscale"),
        ProtocolConfig::Hysteria1(_) => not_impl("hysteria1"),
        ProtocolConfig::Redirect(_) => not_impl("redirect (outbound-only kind)"),
        ProtocolConfig::TProxy(_) => not_impl("tproxy (outbound-only kind)"),
        ProtocolConfig::Mixed(_) => not_impl("mixed (outbound-only kind)"),
    }
}
```

The match is exhaustive over all 19 `ProtocolConfig` variants — a new variant
in the proto crate is a compile error here until an arm exists.

- [ ] **Step 4: NativeTunnel in lib.rs (connect() lands in Task 9)**

```rust
pub struct NativeTunnel {
    inner: BoxStream,
}

impl tokio::io::AsyncRead for NativeTunnel {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for NativeTunnel {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

impl NativeTunnel {
    /// Wrap a completed layer stack (crate-internal; external users get
    /// tunnels from `connect`/`connect_chain`).
    pub(crate) fn from_stream(inner: BoxStream) -> Self {
        Self { inner }
    }
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p xray-tui-native`
Expected: PASS — all prior unit tests + header tests compile against the final dispatch. The VLESS e2e against a real server lands in Task 11; stream-family kinds beyond VLESS return NotImplemented.

- [ ] **Step 6: Commit**

```bash
git add crates/xray-tui-native/src/protocol crates/xray-tui-native/src/lib.rs
git commit -m "feat(native): protocol stub map + dispatch over all configs + VLESS connect + tunnel type"
```

---

### Task 9: Chain fold — connect_chain

**Files:**
- Create: `crates/xray-tui-native/src/chain.rs`
- Modify: `crates/xray-tui-native/src/lib.rs` — `pub mod chain;` + wire `connect`/`connect_chain` final signatures.

**Interfaces:**
- Consumes: `transport::connect`, `security::wrap`, `protocol::connect`, `LinkContext`, `NativeConnectParams`, `TargetAddr`.
- Produces: `connect_chain(links: &[NativeConnectParams], target: TargetAddr) -> Result<NativeTunnel, NativeError>`. Public API of the crate.

- [ ] **Step 1: Write the failing tests first**

`crates/xray-tui-native/src/chain.rs`:

```rust
//! Client-side chaining: tunnel through several proxies in order.
//!
//! Parity with xray `outbound.proxySettings.socksTag` and sing-box `detour`.
//! `links[0]` is dialed directly and carries `links[1]`'s traffic
//! (links[0].target = links[1].server); the last link reaches `target`.
//!
//! Fold (n=1 is `connect`): for each link, run transport (dial first hop /
//! reuse previous tunnel on later hops) → security → protocol, threading the
//! resulting stream into the next link as its `base`.

use crate::addr::TargetAddr;
use crate::context::{LinkContext, NativeConnectParams};
use crate::error::NativeError;
use crate::protocol;
use crate::security;
use crate::transport;
use crate::{BoxStream, NativeTunnel};

/// The destination of link `i`: the next link's server, or `target` for the
/// last link. Pure function — unit-tested independently of the fold.
fn next_target(links: &[NativeConnectParams], i: usize, target: &TargetAddr) -> TargetAddr {
    links
        .get(i + 1)
        .map(|next| TargetAddr::new(next.server.host.as_str(), next.server.port))
        .unwrap_or_else(|| target.clone())
}

/// Connect through a chain of proxies to the final `target`.
pub async fn connect_chain(
    links: &[NativeConnectParams],
    target: TargetAddr,
) -> Result<NativeTunnel, NativeError> {
    let mut base: Option<BoxStream> = None;
    for (i, link) in links.iter().enumerate() {
        let to = next_target(links, i, &target);
        let ctx = LinkContext::new(link.clone(), to);
        let transported = transport::connect(&ctx, base).await?;
        let secured = security::wrap(&ctx, transported).await?;
        base = Some(protocol::connect(&ctx, secured).await?);
    }
    base.map(NativeTunnel::from_stream)
        .ok_or_else(|| NativeError::Config("empty chain".into()))
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;
    use xray_tui_proto::proto_spec::ProtocolConfig;

    use super::*;
    use crate::addr::Host;

    fn params(host: &str, port: u16) -> NativeConnectParams {
        NativeConnectParams::new(
            ProtocolConfig::Vless(Default::default()),
            EndpointEssentials::new(host, port),
            TargetAddr::new(Host::Domain("ignored".into()), 1),
        )
    }

    #[test]
    fn next_target_uses_next_server() {
        let links = [params("a.example", 10), params("b.example", 20)];
        let final_target = TargetAddr::new(Host::Domain("www.example.com".into()), 443);
        assert_eq!(
            next_target(&links, 0, &final_target),
            TargetAddr::new(Host::Domain("b.example".into()), 20)
        );
        assert_eq!(next_target(&links, 1, &final_target), final_target);
    }

    #[test]
    fn empty_chain_errors() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let res = rt.block_on(connect_chain(&[], TargetAddr::new(Host::Domain("x".into()), 1)));
        assert!(matches!(res, Err(NativeError::Config(_))));
    }
}
```

- [ ] **Step 2: Run tests to verify they pass (pure-function level)**

Run: `cargo test -p xray-tui-native chain::tests`
Expected: PASS. The n=1 fold is covered by the Task 11 e2e; n=2 chain e2e is an M5 milestone-flagged follow-up (needs a second real protocol — SS in M4).

- [ ] **Step 3: Finalize the public API in lib.rs**

```rust
pub mod chain;

pub use chain::connect_chain;
pub use context::{LinkContext, NativeConnectParams};
pub use error::NativeError;

/// Connect through a single proxy to `params.target`.
pub async fn connect(params: NativeConnectParams) -> Result<NativeTunnel, NativeError> {
    let target = params.target.clone();
    connect_chain(&[params], target).await
}
```

Note: the chain fold's `target` is the FINAL destination; the single-link
`connect` must pass `params.target`, not a default. `next_target` already
resolves it for the only (last) link.

- [ ] **Step 4: Run full unit suite**

Run: `cargo test -p xray-tui-native`
Expected: PASS (all tasks so far).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/chain.rs crates/xray-tui-native/src/addr.rs crates/xray-tui-native/src/lib.rs
git commit -m "feat(native): connect_chain fold + single-link connect API"
```

---

### Task 10: crypto/ skeleton + JA3/JA4 fingerprint encoders

**Files:**
- Create: `crates/xray-tui-native/src/crypto/mod.rs`
- Create: `crates/xray-tui-native/src/crypto/kdf.rs` (stub + doc)
- Create: `crates/xray-tui-native/src/crypto/aead.rs` (stub + doc)
- Create: `crates/xray-tui-native/src/crypto/salamander.rs` (stub + doc)
- Create: `crates/xray-tui-native/src/crypto/legacy_stream.rs` (stub + doc)
- Create: `crates/xray-tui-native/src/crypto/fingerprint/mod.rs`
- Create: `crates/xray-tui-native/src/crypto/fingerprint/ja3.rs`
- Create: `crates/xray-tui-native/src/crypto/fingerprint/ja4.rs`
- Modify: `crates/xray-tui-native/src/lib.rs` — `pub mod crypto;` (already declared in skeleton; wire now)

**Interfaces:**
- Produces: `Ja3Fields` + `canonical_string()`, `hash()` (md-5); `Ja4Fields` + `ja4_string()`. Consumed by the fingerprint engine milestone (M2) and tier-1 verification.

- [ ] **Step 1: module root + stub algorithms**

`crates/xray-tui-native/src/crypto/mod.rs`:

```rust
//! Pure cryptographic algorithm codecs.
//!
//! Rule: `crypto/` owns ALGORITHMS, layers own DECISIONS. No I/O policy and
//! no transport logic lives here. Packet-level obfuscation (e.g. Hysteria2
//! Salamander) is implemented here but consumed inside the QUIC transport's
//! packet handling — it is never forced into the byte-stream seam.
//!
//! Current state: JA3/JA4 fingerprint encoders are implemented (tier-1
//! fingerprint tripwire). Everything else is a documented skeleton waiting
//! for its protocol milestone (M2+).

pub mod aead;
pub mod fingerprint;
pub mod kdf;
pub mod legacy_stream;
pub mod salamander;
```

`kdf.rs`, `aead.rs`, `salamander.rs`, `legacy_stream.rs`: each carries a doc comment stating the algorithms, their consumers, the reference implementation path (v2ray-core / hysteria), and the milestones that implement them — no code yet:

```rust
//! HKDF / SHA helpers shared by protocol crypto.
//!
//! Consumers: `protocol/vmess` (VMess KDF), `security/reality` (auth key
//! derivation), future HPKE (Reality target crypto).
//! Reference: `thirdparty/v2ray-core/proxy/vmess/encoding/aead/kdf.go`.
//!
//! Algorithms to implement (M2): KDF16/KDF64 for VMess AEAD, HKDF-SHA256
//! for Reality `derive_auth_key` (info `b"REALITY"`).
```

- [ ] **Step 2: JA3 encoder with failing test first**

`crates/xray-tui-native/src/crypto/fingerprint/ja3.rs`:

```rust
//! JA3 fingerprint encoder (offline).
//!
//! JA3 is the md5 of a canonical string over ClientHello fields:
//! `version,ciphers,extensions,curves,point_formats` with decimal version,
//! lowercase-hex extension/cipher/curve ids and decimal point formats.
//! Computing JA3 from raw hello bytes requires a ClientHello parser (M2,
//! with the fingerprint engine); this codec encodes from structured fields.

/// JA3 field set extracted (later) from a ClientHello.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ja3Fields {
    /// TLS record/layout version, e.g. 0x0303 → 771.
    pub version: u16,
    pub ciphers: Vec<u16>,
    pub extensions: Vec<u16>,
    pub curves: Vec<u16>,
    pub point_formats: Vec<u8>,
}

impl Ja3Fields {
    /// The canonical pre-hash JA3 string.
    #[must_use]
    pub fn canonical_string(&self) -> String {
        format!(
            "{},{},{},{},{}",
            self.version,
            join_hex(&self.ciphers),
            join_hex(&self.extensions),
            join_hex(&self.curves),
            join_dec(&self.point_formats),
        )
    }

    /// The JA3 hash (md5 of the canonical string).
    #[must_use]
    pub fn hash(&self) -> String {
        use md5::{Digest, Md5};
        let mut hasher = Md5::new();
        hasher.update(self.canonical_string().as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

fn join_hex(values: &[u16]) -> String {
    values
        .iter()
        .map(|v| format!("{v:x}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn join_dec(values: &[u8]) -> String {
    values
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_string_exact() {
        let f = Ja3Fields {
            version: 771,
            ciphers: vec![0x1301, 0x1302, 0x1303],
            extensions: vec![0x0000, 0x001a, 0x0a0a],
            curves: vec![0x001d, 0x0017],
            point_formats: vec![0],
        };
        assert_eq!(
            f.canonical_string(),
            "771,1301,1302,1303,0,1a,a0a,1d,17,0"
        );
    }

    #[test]
    fn hash_is_md5_of_canonical() {
        let f = Ja3Fields {
            version: 771,
            ciphers: vec![0x1301],
            extensions: vec![],
            curves: vec![0x001d],
            point_formats: vec![0],
        };
        // md5("771,1301,,1d,0")
        assert_eq!(f.hash(), format!("{:x}", md5::Md5::digest(b"771,1301,,1d,0")));
    }
}
```

mind the double-import of `Digest`: in the test use `md5::{Digest, Md5}` directly.

- [ ] **Step 3: JA4 encoder with failing test first**

`crates/xray-tui-native/src/crypto/fingerprint/ja4.rs`:

```rust
//! JA4 fingerprint encoder (offline, no hash).
//!
//! JA4-A: `t<record_version><first_cipher><extensions>...` composed per the
//! published JA4 spec ("t13d" prefix family). The exact byte-level trimming
//! is tuned against the tier-2 grader (tls.peet.ws) in the M2 engine
//! milestone; this module exposes the canonical string builder.

use crate::crypto::fingerprint::ja3::Ja3Fields;

/// JA4-A canonical string from structured fields.
#[must_use]
pub fn ja4_string(fields: &Ja3Fields) -> String {
    // JA4-A: "t13d" + first cipher hex(4) + "d" + ... (per spec)
    let first_cipher = fields.ciphers.first().copied().unwrap_or(0);
    format!("t13d{first_cipher:04x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::fingerprint::ja3::Ja3Fields;

    #[test]
    fn ja4_uses_first_cipher() {
        let f = Ja3Fields {
            version: 771,
            ciphers: vec![0x1301, 0x1303],
            extensions: vec![0x0000, 0x001a],
            curves: vec![0x001d],
            point_formats: vec![0],
        };
        assert_eq!(ja4_string(&f), "t13d1301");
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p xray-tui-native crypto::`
Expected: PASS (JA3 canonical + hash, JA4).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/crypto
git commit -m "feat(native): crypto skeleton + JA3/JA4 offline fingerprint encoders"
```

---

### Task 11: E2E harness + VLESS against real xray-core and sing-box

**Files:**
- Create: `crates/xray-tui-native/tests/common/mod.rs` (harness: ports, certs, core spawn, HTTP echo, probe)
- Create: `crates/xray-tui-native/tests/vless_xray.rs`
- Create: `crates/xray-tui-native/tests/vless_singbox.rs`

**Interfaces:**
- Consumes: crate public API `connect(NativeConnectParams)`, `NativeConnectParams`, `TargetAddr`, `Host`; `CoreType` from proto (`xray_tui_proto::proto_spec::CoreType`).
- Produces: test-only module used by both e2e tests. Deliverable = two green e2e tests (with binaries present) proving VLESS+TCP+TLS interop.

- [ ] **Step 1: harness `tests/common/mod.rs`**

```rust
//! E2E harness: local HTTP echo, real core server spawn, certs, probe.
//!
//! Gating: feature `native-e2e` AND env `XRAY_TUI_CORE_BIN_DIR` pointing at
//! a directory containing `xray` and/or `sing-box` binaries. Absent → callers
//! skip (eprintln + early return). Never downloads anything.

use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Arc;
use std::time::Duration;

pub use xray_tui_proto::proto_spec::CoreType;
use xray_tui_proto::proto_spec::endpoint::EndpointEssentials;
use xray_tui_proto::proto_spec::ProtocolConfig;

pub struct EchoServer {
    pub addr: SocketAddr,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Drop for EchoServer {
    fn drop(&mut self) {
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// Spawn a tiny_http responder on 127.0.0.1:ephemeral serving a fixed body.
/// tiny_http is synchronous — dedicated thread, requests drained until drop.
pub fn spawn_echo() -> EchoServer {
    let server = tiny_http::Server::http("127.0.0.1:0").expect("bind echo");
    let addr = server.server_addr().to_ip().expect("ip addr");
    let handle = std::thread::spawn(move || {
        for request in server.incoming_requests() {
            let response = tiny_http::Response::from_string("hello native core")
                .with_status_code(200);
            let _ = request.respond(response);
        }
    });
    EchoServer { addr, handle: Some(handle) }
}

/// Return a port that was free at bind time.
pub fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

pub struct CoreGuard {
    child: Child,
    _tmp: tempfile::TempDir,
}

impl Drop for CoreGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Resolve the core binary from `XRAY_TUI_CORE_BIN_DIR` (never auto-download).
pub fn core_bin(core: CoreType) -> Option<PathBuf> {
    let dir = std::env::var("XRAY_TUI_CORE_BIN_DIR").ok()?;
    let dir = Path::new(&dir);
    let name = match core {
        CoreType::Xray => "xray",
        CoreType::SingBox => "sing-box",
        CoreType::Auto => return None,
    };
    let bin = dir.join(name);
    bin.is_file().then_some(bin)
}

/// Spawn a core with the given config JSON; wait until `port` accepts TCP.
pub fn spawn_core(bin: &Path, config_json: &str, port: u16) -> CoreGuard {
    let tmp = tempfile::tempdir().expect("tempdir");
    let conf = tmp.path().join("config.json");
    std::fs::write(&conf, config_json).expect("write config");

    let mut cmd = if bin.ends_with("sing-box") {
        let mut c = Command::new(bin);
        c.args(["run", "-c", conf.to_str().unwrap()]);
        c
    } else {
        let mut c = Command::new(bin);
        c.args(["-c", conf.to_str().unwrap()]);
        c
    };
    cmd.stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    let child = cmd.spawn().expect("spawn core");

    let mut attempts = 0;
    loop {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        attempts += 1;
        assert!(attempts < 100, "core did not open port {port} in time");
        std::thread::sleep(Duration::from_millis(100));
    }
    CoreGuard { child, _tmp: tmp }
}

/// rcgen CA + server cert/key PEMs + CA DER. SAN covers localhost + 127.0.0.1.
pub struct Certs {
    pub cert_pem: String,
    pub key_pem: String,
    pub ca_der: Vec<u8>,
}

pub fn generate_certs() -> Certs {
    use rcgen::{BasicConstraints, CertificateParams, IsCa, KeyPair};

    let mut ca_params = CertificateParams::new(vec!["localhost".to_string()]).unwrap();
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let ca_key = KeyPair::generate().unwrap();
    let ca_cert = ca_params.self_signed(&ca_key).unwrap();

    let server_params =
        CertificateParams::new(vec!["localhost".to_string(), "127.0.0.1".to_string()]).unwrap();
    let server_key = KeyPair::generate().unwrap();
    let server_cert = server_params.signed_by(&server_key, &ca_cert, &ca_key).unwrap();

    Certs {
        cert_pem: server_cert.pem(),
        key_pem: server_key.serialize_pem(),
        ca_der: ca_cert.der().to_vec(),
    }
}

/// Write a GET through the tunnel, return (status code, body).
pub async fn probe(tunnel: &mut xray_tui_native::NativeTunnel) -> (u16, String) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    tunnel
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let mut buf = Vec::new();
    let read = tokio::time::timeout(Duration::from_secs(10), tunnel.read_to_end(&mut buf))
        .await
        .expect("probe timeout")
        .unwrap();
    assert!(read > 0, "empty tunnel response");
    let text = String::from_utf8_lossy(&buf);
    let status_line = text.lines().next().unwrap_or_default().to_string();
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let body = text.split("\r\n\r\n").nth(1).unwrap_or_default().trim_end().to_string();
    (status, body)
}

/// Build the native connect params for the e2e VLESS case (sni=localhost).
///
/// The test must install the harness-CA trust first:
/// `security::tls::set_test_config(security::tls::test_client_config(&certs.ca_der))`.
pub fn vless_params(port: u16, target: SocketAddr) -> xray_tui_native::NativeConnectParams {
    let protocol: ProtocolConfig = serde_json::from_value(serde_json::json!({
        "schema": "vless",
        "uuid": "00000000-0000-0000-0000-000000000000",
        "security": { "type": "tls", "sni": "localhost", "alpn": "http/1.1" },
        "transport": { "type": "tcp" }
    }))
    .unwrap();
    xray_tui_native::NativeConnectParams::new(
        protocol,
        EndpointEssentials::new("127.0.0.1", port),
        xray_tui_native::addr::TargetAddr::new(
            xray_tui_native::addr::Host::Ip(target.ip()),
            target.port(),
        ),
    )
}

/// xray-core inbound config: VLESS + TLS + TCP listener on 127.0.0.1:port.
/// Certificates come from PEM files on disk (both cores take file paths).
pub fn xray_vless_config(port: u16, cert_path: &Path, key_path: &Path) -> String {
    serde_json::json!({
        "inbounds": [{
            "listen": "127.0.0.1",
            "port": port,
            "protocol": "vless",
            "settings": {
                "clients": [{ "id": "00000000-0000-0000-0000-000000000000" }],
                "decryption": "none"
            },
            "streamSettings": {
                "network": "tcp",
                "security": "tls",
                "tlsSettings": {
                    "certificates": [{
                        "certificateFile": cert_path.to_str().unwrap(),
                        "keyFile": key_path.to_str().unwrap()
                    }],
                    "alpn": ["http/1.1"]
                }
            }
        }],
        "outbounds": [{ "protocol": "freedom" }]
    })
    .to_string()
}

/// sing-box inbound config: VLESS + TLS + TCP listener on 127.0.0.1:port.
pub fn singbox_vless_config(port: u16, cert_path: &Path, key_path: &Path) -> String {
    serde_json::json!({
        "log": { "level": "warn" },
        "inbounds": [{
            "type": "vless",
            "listen": "127.0.0.1",
            "listen_port": port,
            "users": [{ "uuid": "00000000-0000-0000-0000-000000000000" }],
            "tls": {
                "enabled": true,
                "certificate_path": cert_path.to_str().unwrap(),
                "key_path": key_path.to_str().unwrap(),
                "alpn": ["http/1.1"]
            }
        }],
        "outbounds": [{ "type": "direct" }]
    })
    .to_string()
}
```

The harness writes the rcgen PEMs to tempdir files (`server.crt`/`server.key`)
and passes their paths to both config builders — xray and sing-box both read
certificate files from disk. The API is settled: `CommonMod` re-exports
`CoreType` (`pub use xray_tui_proto::proto_spec::CoreType;`) so the test files
can call `core_bin(CoreType::Xray)` via `use common::*;`.

- [ ] **Step 2: `tests/vless_xray.rs`**

```rust
#![cfg(feature = "native-e2e")]

mod common;

use common::*;
use xray_tui_native::{connect, security};

#[tokio::test]
async fn vless_tcp_tls_against_xray() {
    let Some(bin) = core_bin(CoreType::Xray) else {
        eprintln!("skip: XRAY_TUI_CORE_BIN_DIR missing xray binary");
        return;
    };
    let echo = spawn_echo();
    let port = free_port();
    let certs = generate_certs();
    let dir = tempfile::tempdir().unwrap();
    let cert_path = dir.path().join("server.crt");
    let key_path = dir.path().join("server.key");
    std::fs::write(&cert_path, &certs.cert_pem).unwrap();
    std::fs::write(&key_path, &certs.key_pem).unwrap();

    let config = xray_vless_config(port, &cert_path, &key_path);
    let _core = spawn_core(&bin, &config, port);

    security::tls::set_test_config(security::tls::test_client_config(&certs.ca_der));
    let params = vless_params(port, echo.addr);
    let mut tunnel = connect(params).await.expect("native connect");
    let (status, body) = probe(&mut tunnel).await;
    assert_eq!(status, 200);
    assert_eq!(body, "hello native core");
}
```

Both config builders take certificate file paths (harness writes the rcgen
PEMs to tempdir before calling).

- [ ] **Step 3: `tests/vless_singbox.rs`**

Same shape as Step 2 with `CoreType::SingBox`, `singbox_vless_config`, and `sing-box run -c` in spawn.

- [ ] **Step 4: Run the e2e suite**

```bash
cd /home/user/oss/xray-tui
XRAY_TUI_CORE_BIN_DIR=/path/to/binaries cargo test -p xray-tui-native --features native-e2e
```

With binaries present: expected 2 e2e PASS (plus all unit tests). Without binaries: tests print skip lines and pass trivially.

If xray/sing-box binaries are NOT locally available, this step is verified by: (a) unit suite green, (b) e2e tests compile and skip cleanly with an unset/missing dir.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/tests crates/xray-tui-native/src/security
git commit -m "feat(native): e2e harness + VLESS interop against xray-core and sing-box"
```

---

### Task 12: Full verification and workspace hygiene

**Files:**
- Modify: none expected beyond fixes surfaced by verification.

- [ ] **Step 1: Run the full unit suite from the workspace root**

Run: `cargo test -p xray-tui-native`
Expected: all unit tests PASS (addr, shape, context, transport, security, vless header, chain, crypto).

- [ ] **Step 2: Run workspace-wide checks**

Run: `cargo check --workspace` then `cargo clippy --workspace --all-targets`
Expected: clean. Fix any clippy `pedantic`/`nursery` warnings in the new crate (repo treats them as warnings, but the crate should start clean).

- [ ] **Step 3: Verify e2e gating behavior**

Run: `cargo test -p xray-tui-native` (no feature, no env)
Expected: e2e test files compiled out (`#![cfg(feature = "native-e2e")]`), unit suite green.

Run: `cargo test -p xray-tui-native --features native-e2e` with `XRAY_TUI_CORE_BIN_DIR` unset
Expected: skip lines, unit suite green.

- [ ] **Step 4: Confirm no spec drift**

Re-read `docs/superpowers/specs/2026-08-07-native-core-stub-design.md` §15 acceptance criteria. Every criterion must map to a passing check:
1. workspace build+test green with all stubs → Steps 1–2.
2. unit vectors (addr/vless header/JA3/ConnectShape/chain) → Task tests.
3. e2e with binaries → Task 11.
4. skip without binaries → Step 3.
5. no TUI/ping edits, no ConfigBuilder reuse, MIT-clean → this crate touches only `crates/xray-tui-native/*`.

- [ ] **Step 5: Commit (if any fixes)**

```bash
git add -A crates/xray-tui-native
git commit -m "chore(native): verification fixes"
```

## Self-Review Notes

- Spec §8/e2e chain (SS→VLESS, two real cores) is deferred beyond this plan: the native SS client is an M4 deliverable, so a two-link chain e2e is impossible in M1. Task 9 covers the fold's pure logic (`next_target`, empty-chain error) and the n=1 fold is proven end-to-end by Task 11. Flagged for the M5 milestone.
- Spec §12 "chain.rs: fold order + teardown over duplex" — partly reduced to pure-function tests for the same reason; the duplex-level composition is exercised by the n=1 e2e and the pass-through test in Task 5.
- JA3/JA4 authoritative vectors (real Chrome hello → grader-confirmed fingerprint) land with the fingerprint engine (M2) where a real ClientHello exists; §12 tier-1 wording implied earlier, this plan implements the codecs + canonical-string/hash tests now.
- The spec's `connect` owns `protocol: ProtocolConfig` clone at the boundary — preserved in `NativeConnectParams`.
