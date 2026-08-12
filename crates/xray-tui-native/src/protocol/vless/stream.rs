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
}

/// Tunnel stream over the secured connection; strips the VLESS response
/// header once, then transparently streams the body.
pub struct VlessClientStream {
    inner: BoxStream,
    /// Whether the response-header version byte has been read (persisted so
    /// the peel is resumable across Pending polls).
    ver_read: bool,
    addons: [u8; 255],
    addons_total: usize,
    peel: Peel,
}

impl VlessClientStream {
    #[must_use]
    pub fn new(inner: BoxStream) -> Self {
        Self {
            inner,
            ver_read: false,
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
                Peel::Head => {
                    // The response header is `version(1) | addon_len(1) |
                    // addons`. A vless server always sends version 0x00; a
                    // NON-vless endpoint (REALITY transparent-proxy
                    // fallback — the client's TLS session terminates at the
                    // dest, not at a vless server) sends no header at all,
                    // so the first byte is plain payload. The version byte
                    // is read first (persisted in `ver_read` so a Pending
                    // addon-length poll does not re-consume it).
                    if !self.ver_read {
                        let mut tmp = [0u8; 1];
                        let mut rb = ReadBuf::new(&mut tmp);
                        ready!(Pin::new(&mut self.inner).poll_read(cx, &mut rb))?;
                        let got = rb.filled().len();
                        if got == 0 {
                            return Poll::Ready(Err(io::Error::new(
                                io::ErrorKind::UnexpectedEof,
                                "vless response header truncated (EOF)",
                            )));
                        }
                        self.ver_read = true;
                        if tmp[0] != header::VERSION {
                            // No vless response header: hand the byte back
                            // as payload.
                            self.peel = Peel::Done;
                            buf.put_slice(&[tmp[0]]);
                            return Poll::Ready(Ok(()));
                        }
                    }
                    // Version matches: read the addon length.
                    let mut tmp = [0u8; 1];
                    let mut rb = ReadBuf::new(&mut tmp);
                    ready!(Pin::new(&mut self.inner).poll_read(cx, &mut rb))?;
                    let got = rb.filled().len();
                    if got == 0 {
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "vless response header truncated (EOF)",
                        )));
                    }
                    self.addons_total = usize::from(tmp[0]);
                    self.peel = if self.addons_total > 0 {
                        Peel::Addons {
                            total: self.addons_total,
                            filled: 0,
                        }
                    } else {
                        Peel::Done
                    };
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
    async fn peels_fragmented_header_across_writes() {
        // The 2-byte response header arrives byte-by-byte: the first header
        // byte lands alone, the second is bundled with the first payload byte
        // (e.g. two TCP segments). A peel state that reads without bounding
        // to the header remainder swallows that payload byte or mis-parses
        // the header — this used to corrupt the stream.
        let (client, server) = pair();
        let stream = VlessClientStream::new(Box::new(client));
        let writer = tokio::spawn(async move {
            let mut server = server;
            server.write_all(&[0x00]).await.unwrap();
            tokio::task::yield_now().await;
            server.write_all(&[0x00, b'h']).await.unwrap();
            tokio::task::yield_now().await;
            server.write_all(b"ello native core").await.unwrap();
        });
        assert_eq!(&read_n(stream, 5).await, b"hello");
        writer.await.unwrap();
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
    async fn passes_through_non_vless_response() {
        // A REALITY transparent-proxy fallback terminates the client's TLS
        // session at a non-vless endpoint: the first byte is payload, not a
        // vless response header, and must pass through untouched.
        use tokio::io::AsyncReadExt;
        let (client, mut server) = pair();
        let mut stream = VlessClientStream::new(Box::new(client));
        let expected = b"HTTP/1.1 200 OK\r\nhello";
        server.write_all(expected).await.unwrap();
        let mut out = vec![0u8; expected.len()];
        stream.read_exact(&mut out).await.unwrap();
        assert_eq!(&out, expected);
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
