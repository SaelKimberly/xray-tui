//! Hysteria2 — native client (`ConnectShape::Quic`).
//!
//! Wire contract: the hysteria2 client (`thirdparty/hysteria`, MIT; sing-box
//! uses the same wire via `sing-quic`). The link is a QUIC connection with
//! ALPN `h3`:
//! 1. **Auth** — an HTTP/3 POST `https://hysteria/auth` carrying the
//!    password in the `hysteria-auth` request header; status 233 = OK.
//! 2. **TCP tunnel** — open a RAW QUIC bidi stream (not h3-framed), write
//!    the `TCPRequest` frame (`0x401 | addrLen | addr | padLen | padding`),
//!    read the `TCPResponse` (status byte 0 = OK), then relay the bytes.
//!
//! Optional Salamander packet obfuscation wraps the UDP datagrams
//! (`crypto/salamander`). UDP relay (datagram sessions) stays
//! `NotImplemented` — the crate's UDP datagram path is VLESS-only.

use xray_tui_proto::proto_spec::Hysteria2Config;

use crate::BoxStream;
use crate::context::LinkContext;
use crate::error::NativeError;

mod quic;

/// The h3 ALPN used for the QUIC handshake (`http3.NextProtoH3`).
const ALPN_H3: &[u8] = b"h3";

/// QUIC frame type for a TCP request (`FrameTypeTCPRequest`).
const FRAME_TYPE_TCP_REQUEST: u64 = 0x401;
/// Overflow guard (`MaxAddressLength`).
const MAX_ADDRESS_LEN: usize = 2048;

/// TCP response status byte: `0` = OK, anything else = the remote error
/// message follows.
const TCP_RESPONSE_OK: u8 = 0;

/// Connect through a Hysteria2 outbound.
///
/// Replaces the dial + security + upgrade chain entirely: a fresh QUIC dial
/// (spec §5.2 — quinn's rustls is internal) with ALPN `h3`, the auth
/// exchange, then one open TCP tunnel to `ctx.target`.
pub async fn connect(ctx: &LinkContext, cfg: &Hysteria2Config) -> Result<BoxStream, NativeError> {
    let (conn, keepalive) = quic::dial(ctx, cfg).await?;
    let stream = quic::open_tcp_tunnel(&conn, keepalive, &ctx.target).await?;
    Ok(Box::new(stream))
}
