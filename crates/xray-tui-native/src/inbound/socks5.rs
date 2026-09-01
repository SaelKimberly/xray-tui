//! SOCKS5 server wire codec (RFC 1928) + username/password auth (RFC 1929).
//!
//! Adapted from the `async-socks5` client library (MIT, `thirdparty/`) into
//! the server role: this module reads the client greeting, negotiates the
//! auth method, and parses the CONNECT/BIND/UDP request. The mirror-image
//! client codec is the `protocol/socks` placeholder (not yet implemented).
//!
//! Modern-Rust notes on the upstream FIXMEs: the client's auth dispatch used
//! `AuthMethod::UsernamePassword if auth.is_some() => … auth.unwrap()` —
//! the "if-let-in-match" guard — and is now written with `if let`/`is_some`
//! directly (no unwrap). `AddrKind::to_socket_addr`'s string hack is
//! unneeded here: wire addresses decode straight into [`TargetAddr`].

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::string::FromUtf8Error;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::addr::{Host, TargetAddr};

/// SOCKS5 protocol version (RFC 1928 §1).
pub const VERSION: u8 = 0x05;

/// A SOCKS5 command (RFC 1928 §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// TCP stream connection — the only command this inbound implements.
    Connect,
    /// TCP port binding (refused: `CommandNotSupported`).
    Bind,
    /// UDP association (refused: `CommandNotSupported`).
    UdpAssociate,
}

impl Command {
    const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::Connect),
            0x02 => Some(Self::Bind),
            0x03 => Some(Self::UdpAssociate),
            _ => None,
        }
    }
}

/// SOCKS5 reply codes (RFC 1928 §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ReplyCode {
    Succeeded = 0x00,
    GeneralFailure = 0x01,
    ConnectionNotAllowed = 0x02,
    NetworkUnreachable = 0x03,
    HostUnreachable = 0x04,
    ConnectionRefused = 0x05,
    TtlExpired = 0x06,
    CommandNotSupported = 0x07,
    AddressTypeNotSupported = 0x08,
}

/// A parsed SOCKS5 request, after method negotiation.
#[derive(Debug, Clone)]
pub struct Socks5Request {
    pub cmd: Command,
    pub target: TargetAddr,
}

/// Auth methods the server speaks (RFC 1928 §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Method {
    None = 0x00,
    UsernamePassword = 0x02,
}

/// Errors from the SOCKS5 server wire codec.
#[derive(Debug, thiserror::Error)]
pub enum Socks5Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid SOCKS version: {0:#x}")]
    InvalidVersion(u8),
    #[error("invalid command: {0:#x}")]
    InvalidCommand(u8),
    #[error("invalid reserved byte: {0:#x}")]
    InvalidReserved(u8),
    #[error("invalid address type: {0:#x}")]
    InvalidAtyp(u8),
    #[error("no acceptable authentication method")]
    NoAcceptableMethod,
    #[error("username/password authentication failed")]
    AuthFailed,
    #[error("invalid authentication subnegotiation version: {0:#x}")]
    InvalidAuthVersion(u8),
    #[error("non-UTF8 string: {0}")]
    Utf8(#[from] FromUtf8Error),
    #[error("domain longer than 255 bytes")]
    DomainTooLong,
}

/// Perform the SOCKS5 server-side method negotiation.
///
/// Reads the client greeting, selects a supported method, writes the
/// selection, and runs the RFC 1929 username/password sub-negotiation when
/// `auth` is configured.
///
/// `auth: None` requires no authentication; `Some((user, pass))` requires
/// username/password. Returns `Err` after writing the failure response when
/// the client offers no acceptable method or supplies bad credentials.
pub async fn negotiate<S>(
    stream: &mut S,
    auth: Option<&(String, String)>,
) -> Result<(), Socks5Error>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let methods = read_greeting(stream).await?;

    let selected = match auth {
        Some(_) if methods.contains(&Method::UsernamePassword) => Some(Method::UsernamePassword),
        None if methods.contains(&Method::None) => Some(Method::None),
        _ => None,
    };

    let Some(selected) = selected else {
        write_no_acceptable(stream).await?;
        return Err(Socks5Error::NoAcceptableMethod);
    };
    write_selection(stream, selected).await?;

    if selected == Method::UsernamePassword {
        let Some((expected_user, expected_pass)) = auth else {
            // Selection only ever picks UsernamePassword when auth is Some.
            unreachable!("username/password selected without configured auth");
        };
        let (user, pass) = read_credentials(stream).await?;
        let ok = expected_user == &user && expected_pass == &pass;
        write_auth_status(stream, ok).await?;
        if !ok {
            return Err(Socks5Error::AuthFailed);
        }
    }
    Ok(())
}

/// Read the client greeting: `VER, NMETHODS, METHODS[]`.
///
/// Unsupported methods are skipped; an empty result means the client offered
/// nothing the server can speak.
async fn read_greeting<S: AsyncRead + Unpin>(stream: &mut S) -> Result<Vec<Method>, Socks5Error> {
    let version = stream.read_u8().await?;
    if version != VERSION {
        return Err(Socks5Error::InvalidVersion(version));
    }
    let nmethods = stream.read_u8().await?;
    let mut methods = Vec::with_capacity(usize::from(nmethods));
    for _ in 0..nmethods {
        let method = match stream.read_u8().await? {
            0x00 => Method::None,
            0x02 => Method::UsernamePassword,
            _ => continue,
        };
        methods.push(method);
    }
    Ok(methods)
}

/// Write the server's method selection: `VER, METHOD`.
async fn write_selection<S: AsyncWrite + Unpin>(
    stream: &mut S,
    method: Method,
) -> Result<(), Socks5Error> {
    stream.write_u8(VERSION).await?;
    stream.write_u8(method as u8).await?;
    stream.flush().await?;
    Ok(())
}

/// Write `VER=0x05, METHOD=0xFF` — no acceptable method.
async fn write_no_acceptable<S: AsyncWrite + Unpin>(stream: &mut S) -> Result<(), Socks5Error> {
    stream.write_u8(VERSION).await?;
    stream.write_u8(0xFF).await?;
    stream.flush().await?;
    Ok(())
}

/// Read the RFC 1929 credentials: `VER=1, ULEN, UNAME, PLEN, PASSWD`.
async fn read_credentials<S: AsyncRead + Unpin>(
    stream: &mut S,
) -> Result<(String, String), Socks5Error> {
    let version = stream.read_u8().await?;
    if version != 0x01 {
        return Err(Socks5Error::InvalidAuthVersion(version));
    }
    let ulen = stream.read_u8().await?;
    let mut username = vec![0; usize::from(ulen)];
    stream.read_exact(&mut username).await?;
    let plen = stream.read_u8().await?;
    let mut password = vec![0; usize::from(plen)];
    stream.read_exact(&mut password).await?;
    Ok((String::from_utf8(username)?, String::from_utf8(password)?))
}

/// Write the RFC 1929 auth response: `VER=1, STATUS`.
async fn write_auth_status<S: AsyncWrite + Unpin>(
    stream: &mut S,
    ok: bool,
) -> Result<(), Socks5Error> {
    stream.write_u8(0x01).await?;
    stream.write_u8(u8::from(!ok)).await?;
    stream.flush().await?;
    Ok(())
}

/// Read the request: `VER, CMD, RSV, ATYP, DST.ADDR, DST.PORT`.
pub async fn read_request<S: AsyncRead + Unpin>(
    stream: &mut S,
) -> Result<Socks5Request, Socks5Error> {
    let version = stream.read_u8().await?;
    if version != VERSION {
        return Err(Socks5Error::InvalidVersion(version));
    }
    let command_byte = stream.read_u8().await?;
    let cmd = Command::from_byte(command_byte).ok_or(Socks5Error::InvalidCommand(command_byte))?;
    let reserved = stream.read_u8().await?;
    if reserved != 0 {
        return Err(Socks5Error::InvalidReserved(reserved));
    }
    let target = read_target(stream).await?;
    Ok(Socks5Request { cmd, target })
}

/// Read one wire address: `ATYP, ADDR, PORT` (port-last, SOCKS5 order).
async fn read_target<S: AsyncRead + Unpin>(stream: &mut S) -> Result<TargetAddr, Socks5Error> {
    let atyp = stream.read_u8().await?;
    match atyp {
        0x01 => {
            let mut octets = [0u8; 4];
            stream.read_exact(&mut octets).await?;
            let port = stream.read_u16().await?;
            Ok(TargetAddr::new(
                Host::Ip(IpAddr::V4(Ipv4Addr::from(octets))),
                port,
            ))
        }
        0x04 => {
            let mut octets = [0u8; 16];
            stream.read_exact(&mut octets).await?;
            let port = stream.read_u16().await?;
            Ok(TargetAddr::new(
                Host::Ip(IpAddr::V6(Ipv6Addr::from(octets))),
                port,
            ))
        }
        0x03 => {
            let len = stream.read_u8().await?;
            let mut domain = vec![0; usize::from(len)];
            stream.read_exact(&mut domain).await?;
            let port = stream.read_u16().await?;
            Ok(TargetAddr::new(
                Host::Domain(String::from_utf8(domain)?),
                port,
            ))
        }
        other => Err(Socks5Error::InvalidAtyp(other)),
    }
}

/// Write a reply: `VER, REP, RSV, ATYP, BND.ADDR, BND.PORT`.
pub async fn write_reply<S: AsyncWrite + Unpin>(
    stream: &mut S,
    reply: ReplyCode,
    bind: &TargetAddr,
) -> Result<(), Socks5Error> {
    stream.write_u8(VERSION).await?;
    stream.write_u8(reply as u8).await?;
    stream.write_u8(0x00).await?;
    write_addr(stream, bind).await?;
    stream.flush().await?;
    Ok(())
}

/// Write one wire address in port-last (SOCKS5) order.
async fn write_addr<S: AsyncWrite + Unpin>(
    stream: &mut S,
    addr: &TargetAddr,
) -> Result<(), Socks5Error> {
    match &addr.host {
        Host::Ip(IpAddr::V4(ip)) => {
            stream.write_u8(0x01).await?;
            stream.write_all(&ip.octets()).await?;
        }
        Host::Ip(IpAddr::V6(ip)) => {
            stream.write_u8(0x04).await?;
            stream.write_all(&ip.octets()).await?;
        }
        Host::Domain(domain) => {
            let len = u8::try_from(domain.len()).map_err(|_| Socks5Error::DomainTooLong)?;
            stream.write_u8(0x03).await?;
            stream.write_u8(len).await?;
            stream.write_all(domain.as_bytes()).await?;
        }
    }
    stream.write_u16(addr.port).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[tokio::test]
    async fn negotiate_no_auth() {
        let (mut client, mut server) = duplex(256);
        client.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
        negotiate(&mut server, None).await.unwrap();
        let mut selection = [0u8; 2];
        client.read_exact(&mut selection).await.unwrap();
        assert_eq!(selection, [0x05, 0x00]);
    }

    #[tokio::test]
    async fn negotiate_auth_success() {
        let (mut client, mut server) = duplex(256);
        client.write_all(&[0x05, 0x01, 0x02]).await.unwrap();
        client
            .write_all(&[0x01, 0x01, b'u', 0x01, b'p'])
            .await
            .unwrap();
        negotiate(&mut server, Some(&("u".into(), "p".into())))
            .await
            .unwrap();
        let mut selection = [0u8; 2];
        client.read_exact(&mut selection).await.unwrap();
        assert_eq!(selection, [0x05, 0x02]);
        let mut status = [0u8; 2];
        client.read_exact(&mut status).await.unwrap();
        assert_eq!(status, [0x01, 0x00]);
    }

    #[tokio::test]
    async fn negotiate_auth_bad_password() {
        let (mut client, mut server) = duplex(256);
        client.write_all(&[0x05, 0x01, 0x02]).await.unwrap();
        client
            .write_all(&[0x01, 0x01, b'u', 0x01, b'x'])
            .await
            .unwrap();
        let err = negotiate(&mut server, Some(&("u".into(), "p".into())))
            .await
            .unwrap_err();
        assert!(matches!(err, Socks5Error::AuthFailed));
        let mut selection = [0u8; 2];
        client.read_exact(&mut selection).await.unwrap();
        assert_eq!(selection, [0x05, 0x02], "method selection");
        let mut status = [0u8; 2];
        client.read_exact(&mut status).await.unwrap();
        assert_eq!(status, [0x01, 0x01]);
    }

    #[tokio::test]
    async fn negotiate_no_acceptable_method() {
        let (mut client, mut server) = duplex(256);
        // Client offers only GSSAPI (0x01), which this server does not speak.
        client.write_all(&[0x05, 0x01, 0x01]).await.unwrap();
        let err = negotiate(&mut server, None).await.unwrap_err();
        assert!(matches!(err, Socks5Error::NoAcceptableMethod));
        let mut selection = [0u8; 2];
        client.read_exact(&mut selection).await.unwrap();
        assert_eq!(selection, [0x05, 0xFF]);
    }

    #[tokio::test]
    async fn negotiate_auth_required_but_only_no_auth_offered() {
        let (mut client, mut server) = duplex(256);
        client.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
        let err = negotiate(&mut server, Some(&("u".into(), "p".into())))
            .await
            .unwrap_err();
        assert!(matches!(err, Socks5Error::NoAcceptableMethod));
        let mut selection = [0u8; 2];
        client.read_exact(&mut selection).await.unwrap();
        assert_eq!(selection, [0x05, 0xFF]);
    }

    #[tokio::test]
    async fn negotiate_wrong_version() {
        let (mut client, mut server) = duplex(256);
        client.write_all(&[0x04, 0x01, 0x00]).await.unwrap();
        let err = negotiate(&mut server, None).await.unwrap_err();
        assert!(matches!(err, Socks5Error::InvalidVersion(0x04)));
    }

    #[tokio::test]
    async fn read_request_ipv4() {
        let (mut client, mut server) = duplex(256);
        client
            .write_all(&[0x05, 0x01, 0x00, 0x01, 127, 0, 0, 1, 0x1F, 0x90])
            .await
            .unwrap();
        let request = read_request(&mut server).await.unwrap();
        assert_eq!(request.cmd, Command::Connect);
        assert_eq!(
            request.target,
            TargetAddr::new(Host::new("127.0.0.1"), 8080)
        );
    }

    #[tokio::test]
    async fn read_request_domain() {
        let (mut client, mut server) = duplex(256);
        client
            .write_all(&[
                0x05, 0x01, 0x00, 0x03, 11, b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'c',
                b'o', b'm', 0x01, 0xBB,
            ])
            .await
            .unwrap();
        let request = read_request(&mut server).await.unwrap();
        assert_eq!(request.cmd, Command::Connect);
        assert_eq!(
            request.target,
            TargetAddr::new(Host::Domain("example.com".into()), 443)
        );
    }

    #[tokio::test]
    async fn read_request_ipv6() {
        let (mut client, mut server) = duplex(256);
        let mut bytes = vec![0x05, 0x01, 0x00, 0x04];
        bytes.extend_from_slice(&[0u8; 15]);
        bytes.push(1);
        bytes.extend_from_slice(&[0x04, 0xD2]);
        client.write_all(&bytes).await.unwrap();
        let request = read_request(&mut server).await.unwrap();
        assert_eq!(request.target, TargetAddr::new(Host::new("::1"), 1234));
    }

    #[tokio::test]
    async fn read_request_invalid_command() {
        let (mut client, mut server) = duplex(256);
        client
            .write_all(&[0x05, 0x09, 0x00, 0x01, 127, 0, 0, 1, 0, 80])
            .await
            .unwrap();
        let err = read_request(&mut server).await.unwrap_err();
        assert!(matches!(err, Socks5Error::InvalidCommand(0x09)));
    }

    #[tokio::test]
    async fn read_request_invalid_reserved() {
        let (mut client, mut server) = duplex(256);
        client
            .write_all(&[0x05, 0x01, 0x01, 0x01, 127, 0, 0, 1, 0, 80])
            .await
            .unwrap();
        let err = read_request(&mut server).await.unwrap_err();
        assert!(matches!(err, Socks5Error::InvalidReserved(0x01)));
    }

    #[tokio::test]
    async fn read_request_invalid_atyp() {
        let (mut client, mut server) = duplex(256);
        client.write_all(&[0x05, 0x01, 0x00, 0x09]).await.unwrap();
        let err = read_request(&mut server).await.unwrap_err();
        assert!(matches!(err, Socks5Error::InvalidAtyp(0x09)));
    }

    #[tokio::test]
    async fn write_reply_encodes_zero_bind() {
        let (mut client, mut server) = duplex(256);
        let bind = TargetAddr::new(Host::new("0.0.0.0"), 0);
        write_reply(&mut server, ReplyCode::Succeeded, &bind)
            .await
            .unwrap();
        let mut bytes = [0u8; 10];
        client.read_exact(&mut bytes).await.unwrap();
        assert_eq!(&bytes, &[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0]);
    }

    #[tokio::test]
    async fn write_reply_encodes_domain_bind() {
        let (mut client, mut server) = duplex(256);
        let bind = TargetAddr::new(Host::Domain("x.io".into()), 9);
        write_reply(&mut server, ReplyCode::ConnectionNotAllowed, &bind)
            .await
            .unwrap();
        let mut bytes = [0u8; 4];
        client.read_exact(&mut bytes).await.unwrap();
        assert_eq!(&bytes, &[0x05, 0x02, 0x00, 0x03]);
        let mut len = [0u8; 1];
        client.read_exact(&mut len).await.unwrap();
        assert_eq!(len, [4]);
        let mut rest = [0u8; 4 + 2];
        client.read_exact(&mut rest).await.unwrap();
        assert_eq!(&rest, b"x.io\0\x09");
    }
}
