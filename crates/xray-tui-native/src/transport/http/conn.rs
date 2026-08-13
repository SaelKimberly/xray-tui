//! Client-connection handshakes over our byte stream. Each spawns the hyper
//! driver task (detached; it ends when the stream/connection closes).

use hyper_util::rt::{TokioExecutor, TokioIo};

use super::body::ReqBody;
use crate::BoxStream;
use crate::error::{NativeError, timeouts};

/// Handshake an HTTP/1.1 client over `stream`.
pub async fn h1_client(
    stream: BoxStream,
) -> Result<hyper::client::conn::http1::SendRequest<ReqBody>, NativeError> {
    let limit = timeouts::TRANSPORT;
    let (sender, conn) = tokio::time::timeout(
        limit,
        hyper::client::conn::http1::handshake(TokioIo::new(stream)),
    )
    .await
    .map_err(|_| NativeError::Timeout {
        step: "http/1.1 handshake",
        limit,
    })?
    .map_err(|e| NativeError::Transport(format!("http/1.1 handshake: {e}")))?;
    tokio::spawn(conn);
    Ok(sender)
}

/// Handshake an HTTP/2 client over `stream`.
pub async fn h2_client(
    stream: BoxStream,
) -> Result<hyper::client::conn::http2::SendRequest<ReqBody>, NativeError> {
    let limit = timeouts::TRANSPORT;
    let (sender, conn) = tokio::time::timeout(
        limit,
        hyper::client::conn::http2::handshake(TokioExecutor::new(), TokioIo::new(stream)),
    )
    .await
    .map_err(|_| NativeError::Timeout {
        step: "http/2 handshake",
        limit,
    })?
    .map_err(|e| NativeError::Transport(format!("http/2 handshake: {e}")))?;
    tokio::spawn(conn);
    Ok(sender)
}
