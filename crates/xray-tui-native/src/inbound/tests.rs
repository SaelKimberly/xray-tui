//! Integration tests for the SOCKS5 inbound: full accept → route → outbound
//! flows over real TCP.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use xray_tui_route::ir::{
    Action, Cond, DefaultRoute, MatchItem, RejectMethod, ResolveStrategy, Rule, RuleSet,
};
use xray_tui_route::{Engine, NetAddr, NetHost};

use super::outbound::OutboundKind;
use super::{Outbound, Socks5Inbound, Socks5InboundConfig};
use crate::addr::{Host, TargetAddr};

// Helpers
// *****************************************************************************

fn engine(rules: Vec<Rule>, default: DefaultRoute) -> Arc<Engine> {
    Arc::new(
        Engine::build(RuleSet {
            rules,
            default,
            resolve_strategy: ResolveStrategy::AsIs,
            probes: Vec::new(),
        })
        .expect("engine builds"),
    )
}

fn route_default(tag: &str) -> DefaultRoute {
    DefaultRoute::Route {
        tag: tag.to_owned(),
    }
}

fn block_domain(domain: &str) -> Rule {
    Rule {
        name: None,
        cond: Cond::All(vec![MatchItem::Domain {
            exact: vec![domain.to_owned()],
            suffix: Vec::new(),
            keywords: Vec::new(),
            regexes: Vec::new(),
        }]),
        action: Action::Route {
            tag: "block".to_owned(),
            override_addr: None,
        },
    }
}

async fn spawn_inbound(config: Socks5InboundConfig) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let inbound = Socks5Inbound::bind(config).await.expect("bind inbound");
    let addr = inbound.local_addr().expect("local addr");
    let handle = tokio::spawn(async move {
        let _ = inbound.serve().await;
    });
    (addr, handle)
}

/// A TCP echo server; returns its bound address.
async fn spawn_echo() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind echo listener");
    let addr = listener.local_addr().expect("echo addr");
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let (mut r, mut w) = sock.split();
                let _ = tokio::io::copy(&mut r, &mut w).await;
            });
        }
    });
    addr
}

/// Outcome of a test SOCKS5 client connection attempt.
enum ClientResult {
    /// The server refused authentication (RFC 1929 status ≠ 0).
    AuthRejected,
    /// The server replied to the CONNECT request with `code`.
    Connected { code: u8, stream: TcpStream },
}

/// A minimal SOCKS5 client: greeting, optional RFC 1929 auth, CONNECT.
async fn client_connect(
    addr: SocketAddr,
    auth: Option<(&str, &str)>,
    target: &TargetAddr,
) -> ClientResult {
    let mut stream = TcpStream::connect(addr).await.expect("connect to inbound");
    let mut methods = vec![0x00u8];
    if auth.is_some() {
        methods.push(0x02);
    }
    stream
        .write_all(&[0x05, u8::try_from(methods.len()).expect("methods ≤ 255")])
        .await
        .expect("greeting header");
    stream.write_all(&methods).await.expect("greeting methods");
    let mut selection = [0u8; 2];
    stream
        .read_exact(&mut selection)
        .await
        .expect("method selection");
    assert_eq!(selection[0], 0x05, "method selection version");

    if selection[1] == 0x02 {
        let (user, pass) = auth.expect("auth offered but no credentials given");
        stream
            .write_all(&[0x01, u8::try_from(user.len()).expect("user ≤ 255")])
            .await
            .expect("auth header");
        stream.write_all(user.as_bytes()).await.expect("auth user");
        stream
            .write_all(&[u8::try_from(pass.len()).expect("pass ≤ 255")])
            .await
            .expect("auth pass len");
        stream.write_all(pass.as_bytes()).await.expect("auth pass");
        let mut status = [0u8; 2];
        stream.read_exact(&mut status).await.expect("auth status");
        assert_eq!(status[0], 0x01, "auth subnegotiation version");
        if status[1] != 0 {
            return ClientResult::AuthRejected;
        }
    }

    stream
        .write_all(&[0x05, 0x01, 0x00])
        .await
        .expect("request head");
    write_target(&mut stream, target).await;
    let mut head = [0u8; 4];
    stream.read_exact(&mut head).await.expect("reply head");
    assert_eq!(head[0], 0x05, "reply version");
    assert_eq!(head[2], 0x00, "reply reserved");
    skip_reply_addr(&mut stream, head[3]).await;
    ClientResult::Connected {
        code: head[1],
        stream,
    }
}

async fn write_target<S: tokio::io::AsyncWrite + Unpin>(stream: &mut S, target: &TargetAddr) {
    match &target.host {
        Host::Ip(IpAddr::V4(ip)) => {
            stream.write_all(&[0x01]).await.expect("atyp v4");
            stream.write_all(&ip.octets()).await.expect("ipv4");
        }
        Host::Ip(IpAddr::V6(ip)) => {
            stream.write_all(&[0x04]).await.expect("atyp v6");
            stream.write_all(&ip.octets()).await.expect("ipv6");
        }
        Host::Domain(domain) => {
            stream
                .write_all(&[0x03, u8::try_from(domain.len()).expect("domain ≤ 255")])
                .await
                .expect("atyp domain");
            stream.write_all(domain.as_bytes()).await.expect("domain");
        }
    }
    stream
        .write_all(&target.port.to_be_bytes())
        .await
        .expect("port");
}

async fn skip_reply_addr<S: tokio::io::AsyncRead + Unpin>(stream: &mut S, atyp: u8) {
    match atyp {
        0x01 => {
            let mut buf = [0u8; 4 + 2];
            stream.read_exact(&mut buf).await.expect("bnd v4");
        }
        0x04 => {
            let mut buf = [0u8; 16 + 2];
            stream.read_exact(&mut buf).await.expect("bnd v6");
        }
        0x03 => {
            let mut len = [0u8; 1];
            stream.read_exact(&mut len).await.expect("bnd domain len");
            let mut buf = vec![0u8; usize::from(len[0]) + 2];
            stream.read_exact(&mut buf).await.expect("bnd domain");
        }
        other => panic!("unexpected reply atyp {other}"),
    }
}

// Tests
// *****************************************************************************

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn direct_relay_echoes() {
    let echo = spawn_echo().await;
    let config = Socks5InboundConfig::new(
        "127.0.0.1:0".parse().expect("listen addr"),
        engine(Vec::new(), route_default("direct")),
        vec![Outbound {
            tag: "direct".into(),
            kind: OutboundKind::Direct,
        }],
    );
    let (addr, handle) = spawn_inbound(config).await;

    let target = TargetAddr::new(Host::new("127.0.0.1"), echo.port());
    let ClientResult::Connected { code, mut stream } = client_connect(addr, None, &target).await
    else {
        panic!("expected CONNECT success");
    };
    assert_eq!(code, 0x00, "reply code");

    stream.write_all(b"hello socks5").await.expect("write");
    let mut buf = [0u8; 12];
    stream.read_exact(&mut buf).await.expect("read echo");
    assert_eq!(&buf, b"hello socks5");

    handle.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn block_outbound_replies_not_allowed() {
    let config = Socks5InboundConfig::new(
        "127.0.0.1:0".parse().expect("listen addr"),
        engine(Vec::new(), route_default("block")),
        vec![Outbound {
            tag: "block".into(),
            kind: OutboundKind::Block,
        }],
    );
    let (addr, handle) = spawn_inbound(config).await;

    let target = TargetAddr::new(Host::new("93.184.216.34"), 80);
    let ClientResult::Connected { code, .. } = client_connect(addr, None, &target).await else {
        panic!("expected a reply");
    };
    assert_eq!(code, 0x02, "ConnectionNotAllowed");

    handle.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn domain_rule_routes_block_and_direct() {
    let echo = spawn_echo().await;
    let config = Socks5InboundConfig::new(
        "127.0.0.1:0".parse().expect("listen addr"),
        engine(
            vec![block_domain("blocked.example")],
            route_default("direct"),
        ),
        vec![
            Outbound {
                tag: "direct".into(),
                kind: OutboundKind::Direct,
            },
            Outbound {
                tag: "block".into(),
                kind: OutboundKind::Block,
            },
        ],
    );
    let (addr, handle) = spawn_inbound(config).await;

    let blocked = TargetAddr::new(Host::Domain("blocked.example".into()), 80);
    let ClientResult::Connected { code, .. } = client_connect(addr, None, &blocked).await else {
        panic!("expected a reply for blocked domain");
    };
    assert_eq!(code, 0x02, "blocked domain");

    let target = TargetAddr::new(Host::new("127.0.0.1"), echo.port());
    let ClientResult::Connected { code, mut stream } = client_connect(addr, None, &target).await
    else {
        panic!("expected CONNECT success for unblocked target");
    };
    assert_eq!(code, 0x00, "unblocked target");
    stream.write_all(b"ping").await.expect("write");
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf).await.expect("read echo");
    assert_eq!(&buf, b"ping");

    handle.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reject_rule_replies_not_allowed() {
    let rule = Rule {
        name: None,
        cond: Cond::All(vec![MatchItem::Ports(vec![
            xray_tui_route::addr::PortRange { start: 80, end: 80 },
        ])]),
        action: Action::Reject {
            method: RejectMethod::Drop,
        },
    };
    let config = Socks5InboundConfig::new(
        "127.0.0.1:0".parse().expect("listen addr"),
        engine(vec![rule], route_default("direct")),
        vec![Outbound {
            tag: "direct".into(),
            kind: OutboundKind::Direct,
        }],
    );
    let (addr, handle) = spawn_inbound(config).await;

    let target = TargetAddr::new(Host::new("93.184.216.34"), 80);
    let ClientResult::Connected { code, .. } = client_connect(addr, None, &target).await else {
        panic!("expected a reply");
    };
    assert_eq!(code, 0x02, "ConnectionNotAllowed");

    handle.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn override_addr_rewrites_target() {
    let echo = spawn_echo().await;
    let rule = Rule {
        name: None,
        cond: Cond::All(vec![MatchItem::Domain {
            exact: vec!["rewritten.example".into()],
            suffix: Vec::new(),
            keywords: Vec::new(),
            regexes: Vec::new(),
        }]),
        action: Action::Route {
            tag: "direct".into(),
            override_addr: Some(NetAddr {
                host: NetHost::Ip("127.0.0.1".parse().expect("ip")),
                port: echo.port(),
            }),
        },
    };
    let config = Socks5InboundConfig::new(
        "127.0.0.1:0".parse().expect("listen addr"),
        engine(vec![rule], route_default("block")),
        vec![
            Outbound {
                tag: "direct".into(),
                kind: OutboundKind::Direct,
            },
            Outbound {
                tag: "block".into(),
                kind: OutboundKind::Block,
            },
        ],
    );
    let (addr, handle) = spawn_inbound(config).await;

    // The request asks for rewritten.example:9999, but the rule rewrites the
    // target to the local echo server before dialing.
    let target = TargetAddr::new(Host::Domain("rewritten.example".into()), 9999);
    let ClientResult::Connected { code, mut stream } = client_connect(addr, None, &target).await
    else {
        panic!("expected CONNECT success");
    };
    assert_eq!(code, 0x00, "rewritten target dialed");
    stream.write_all(b"rewrite").await.expect("write");
    let mut buf = [0u8; 7];
    stream.read_exact(&mut buf).await.expect("read echo");
    assert_eq!(&buf, b"rewrite");

    handle.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unknown_outbound_tag_replies_general_failure() {
    let config = Socks5InboundConfig::new(
        "127.0.0.1:0".parse().expect("listen addr"),
        engine(Vec::new(), route_default("missing")),
        Vec::new(), // no outbound tagged "missing"
    );
    let (addr, handle) = spawn_inbound(config).await;

    let target = TargetAddr::new(Host::new("127.0.0.1"), 1);
    let ClientResult::Connected { code, .. } = client_connect(addr, None, &target).await else {
        panic!("expected a reply");
    };
    assert_eq!(code, 0x01, "GeneralFailure");

    handle.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn auth_required_accepts_good_credentials() {
    let echo = spawn_echo().await;
    let config = Socks5InboundConfig::new(
        "127.0.0.1:0".parse().expect("listen addr"),
        engine(Vec::new(), route_default("direct")),
        vec![Outbound {
            tag: "direct".into(),
            kind: OutboundKind::Direct,
        }],
    )
    .with_auth("user", "pass");
    let (addr, handle) = spawn_inbound(config).await;

    let target = TargetAddr::new(Host::new("127.0.0.1"), echo.port());
    let ClientResult::Connected { code, mut stream } =
        client_connect(addr, Some(("user", "pass")), &target).await
    else {
        panic!("expected CONNECT success");
    };
    assert_eq!(code, 0x00, "authenticated connect");
    stream.write_all(b"hi").await.expect("write");
    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf).await.expect("read echo");
    assert_eq!(&buf, b"hi");

    handle.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn auth_required_rejects_bad_credentials() {
    let config = Socks5InboundConfig::new(
        "127.0.0.1:0".parse().expect("listen addr"),
        engine(Vec::new(), route_default("direct")),
        vec![Outbound {
            tag: "direct".into(),
            kind: OutboundKind::Direct,
        }],
    )
    .with_auth("user", "pass");
    let (addr, handle) = spawn_inbound(config).await;

    let target = TargetAddr::new(Host::new("127.0.0.1"), 1);
    let result = client_connect(addr, Some(("user", "wrong")), &target).await;
    assert!(
        matches!(result, ClientResult::AuthRejected),
        "bad credentials must be refused"
    );

    handle.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bind_command_is_unsupported() {
    let config = Socks5InboundConfig::new(
        "127.0.0.1:0".parse().expect("listen addr"),
        engine(Vec::new(), route_default("direct")),
        vec![Outbound {
            tag: "direct".into(),
            kind: OutboundKind::Direct,
        }],
    );
    let (addr, handle) = spawn_inbound(config).await;

    let mut stream = TcpStream::connect(addr).await.expect("connect to inbound");
    stream
        .write_all(&[0x05, 0x01, 0x00])
        .await
        .expect("greeting");
    let mut selection = [0u8; 2];
    stream.read_exact(&mut selection).await.expect("selection");
    assert_eq!(selection, [0x05, 0x00]);

    // BIND command to 127.0.0.1:0.
    stream
        .write_all(&[0x05, 0x02, 0x00, 0x01, 127, 0, 0, 1, 0, 0])
        .await
        .expect("bind request");
    let mut head = [0u8; 4];
    stream.read_exact(&mut head).await.expect("reply head");
    assert_eq!(head[0], 0x05, "reply version");
    assert_eq!(head[1], 0x07, "CommandNotSupported");
    skip_reply_addr(&mut stream, head[3]).await;

    handle.abort();
}
