//! Client-side VLESS tunnel stream: peels the response header.
//!
//! The VLESS response header (`version(1B) | addon_len(1B) | addons`) may be
//! sent EAGERLY (xray-core writes it right after parsing the request) or
//! LAZILY (sing-box writes it only before the first application-data write;
//! see sing-vmess `vless/service.go` `serverConn.responseWritten`). A client
//! that blocks reading the header before sending payload deadlocks against
//! sing-box — xray's own outbound copies the request payload FIRST and only
//! then decodes the response header (`Xray-core/proxy/vless/outbound/
//! outbound.go` `getResponse`). We mirror that: the header is peeled on the
//! first read, whatever its arrival order.

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::BoxStream;
use crate::protocol::vless::header;

#[derive(Clone, Copy)]
enum Peel {
    Head,
    Addons { total: usize, filled: usize },
    Done,
    Dead(&'static str),
}

/// Tunnel stream over the secured connection; strips the VLESS response
/// header once, then transparently streams the body.
pub struct VlessClientStream {
    inner: BoxStream,
    head: [u8; 2],
    head_filled: usize,
    addons: [u8; 255],
    addons_total: usize,
    peel: Peel,
}

impl VlessClientStream {
    #[must_use]
    pub fn new(inner: BoxStream) -> Self {
        Self {
            inner,
            head: [0; 2],
            head_filled: 0,
            addons: [0; 255],
            addons_total: 0,
            peel: Peel::Head,
        }
    }
}

impl AsyncRead for VlessClientStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            let peel = self.peel; // Peel: Copy — no borrow of `self` held across awaits
            match peel {
                Peel::Done => break,
                Peel::Dead(msg) => {
                    return Poll::Ready(Err(io::Error::new(io::ErrorKind::InvalidData, msg)));
                }
                Peel::Head => {
                    // Fill the 2 header bytes through a local buffer (a read
                    // sized to the remainder can never overshoot).
                    let mut tmp = [0u8; 2];
                    let mut rb = ReadBuf::new(&mut tmp);
                    ready!(Pin::new(&mut self.inner).poll_read(cx, &mut rb))?;
                    let got = rb.filled().len();
                    if got == 0 {
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "vless response header truncated (EOF)",
                        )));
                    }
                    let start = self.head_filled;
                    self.head[..got].copy_from_slice(&tmp[..got]);
                    self.head_filled = start + got;
                    if self.head_filled < 2 {
                        continue;
                    }
                    // Validate version and read addon length.
                    match header::check_response_header(&self.head) {
                        Ok(len) => {
                            self.addons_total = len;
                            self.peel = if len > 0 {
                                Peel::Addons {
                                    total: len,
                                    filled: 0,
                                }
                            } else {
                                Peel::Done
                            };
                        }
                        Err(_) => {
                            self.peel = Peel::Dead("vless response header rejected");
                        }
                    }
                }
                Peel::Addons { total, filled } => {
                    // Bound the staging read to the addons remainder — a
                    // fixed-size buffer would swallow payload bytes.
                    let need = total - filled;
                    let mut tmp = [0u8; 255];
                    let mut rb = ReadBuf::new(&mut tmp[..need]);
                    ready!(Pin::new(&mut self.inner).poll_read(cx, &mut rb))?;
                    let got = rb.filled().len();
                    if got == 0 {
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "vless response addons truncated (EOF)",
                        )));
                    }
                    self.addons[filled..filled + got].copy_from_slice(&tmp[..got]);
                    let filled = filled + got;
                    self.peel = if filled == total {
                        Peel::Done
                    } else {
                        Peel::Addons { total, filled }
                    };
                }
            }
        }
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for VlessClientStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    /// Pair: `write` half receives what the SERVER writes; `read` half is
    /// what the client tunnel reads from.
    fn pair() -> (tokio::io::DuplexStream, tokio::io::DuplexStream) {
        let (a, b) = tokio::io::duplex(256);
        // a -> client side (tunnel reads a); b -> server side (server writes b)
        (a, b)
    }

    async fn read_n(mut stream: VlessClientStream, n: usize) -> Vec<u8> {
        use tokio::io::AsyncReadExt;
        let mut out = vec![0u8; n];
        stream.read_exact(&mut out).await.unwrap();
        out
    }

    #[tokio::test]
    async fn peels_header_sent_alone_then_payload() {
        let (client, mut server) = pair();
        let stream = VlessClientStream::new(Box::new(client));
        server.write_all(&[0x00, 0x00]).await.unwrap();
        server.write_all(b"hello native core").await.unwrap();
        assert_eq!(&read_n(stream, 5).await, b"hello");
    }

    #[tokio::test]
    async fn peels_header_and_payload_in_one_write() {
        let (client, mut server) = pair();
        let stream = VlessClientStream::new(Box::new(client));
        server.write_all(&[0x00, 0x00, b'h', b'i']).await.unwrap();
        assert_eq!(&read_n(stream, 2).await, b"hi");
    }

    #[tokio::test]
    async fn skips_addons() {
        let (client, mut server) = pair();
        let stream = VlessClientStream::new(Box::new(client));
        server
            .write_all(&[0x00, 0x02, 0xaa, 0xbb, b'd', b'a', b't', b'a'])
            .await
            .unwrap();
        assert_eq!(&read_n(stream, 4).await, b"data");
    }

    #[tokio::test]
    async fn rejects_bad_version() {
        use tokio::io::AsyncReadExt;
        let (client, mut server) = pair();
        let mut stream = VlessClientStream::new(Box::new(client));
        server.write_all(&[0x01, 0x00]).await.unwrap();
        let mut out = [0u8; 4];
        let res = stream.read(&mut out).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn eof_before_header_is_error() {
        use tokio::io::AsyncReadExt;
        let (client, server) = pair();
        let mut stream = VlessClientStream::new(Box::new(client));
        drop(server);
        let mut out = [0u8; 4];
        assert!(stream.read(&mut out).await.is_err());
    }
}
