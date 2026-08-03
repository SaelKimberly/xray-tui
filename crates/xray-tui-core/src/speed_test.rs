use futures_util::StreamExt;
use std::io;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Kinds of speed tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestType {
    TcpPing,
    RealPing,
    SpeedTest,
    UdpTest,
}

/// Result of a real ping (HTTP through SOCKS5 proxy) test.
#[derive(Debug, Clone)]
pub struct RealPingResult {
    pub latency_ms: u64,
    pub ip_info: Option<String>,
}

#[derive(Error, Debug)]
pub enum SpeedTestError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Timeout after {0:?}")]
    Timeout(Duration),
    #[error("Proxy error: {0}")]
    Proxy(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
}

/// TCP ping: measure time to complete TCP handshake with `addr:port`.
/// Connects directly (not through proxy). Timeout applied.
pub async fn tcp_ping(
    addr: &str,
    port: u16,
    test_timeout: Duration,
) -> Result<Duration, SpeedTestError> {
    let start = std::time::Instant::now();
    match timeout(test_timeout, TcpStream::connect((addr, port))).await {
        Ok(Ok(_)) => Ok(start.elapsed()),
        Ok(Err(e)) => Err(SpeedTestError::Io(e)),
        Err(_) => Err(SpeedTestError::Timeout(test_timeout)),
    }
}

/// Poll a SOCKS5 proxy port until it responds with a valid server-selection
/// (VER=5, METHOD=0x00), indicating the proxy stack is fully initialized.
/// Polls every 50ms up to `deadline`. Returns `Ok(())` on success, `Err(())` on timeout.
///
/// Pattern from v2rayN's `WaitForProxyPort()` — more reliable than raw TCP connect
/// because it confirms the SOCKS5 handshake layer is ready, not just the TCP listener.
pub async fn wait_for_socks5(addr: &str, port: u16, deadline: Duration) -> Result<(), ()> {
    let start = Instant::now();
    let greeting: [u8; 3] = [0x05, 0x01, 0x00]; // VER=5, NMETHODS=1, METHOD=0 (no auth)
    let mut buf = [0u8; 2];
    loop {
        match TcpStream::connect((addr, port)).await {
            Ok(mut stream) => {
                // Send SOCKS5 greeting and check server selection
                if stream.write_all(&greeting).await.is_ok()
                    && stream.read_exact(&mut buf).await.is_ok()
                    && buf == [0x05, 0x00]
                {
                    return Ok(());
                }
            }
            Err(_) if start.elapsed() < deadline => {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(_) => return Err(()),
        }
        if start.elapsed() >= deadline {
            return Err(());
        }
    }
}

/// Direct UDP ping: send a probe to `addr:port`, measure time to any response.
/// Uses raw UDP (not through proxy). Timeout applied.
/// Good for WireGuard endpoints, ShadowsocksR, and other UDP-based protocols.
pub async fn udp_ping(
    addr: &str,
    port: u16,
    test_timeout: Duration,
) -> Result<Duration, SpeedTestError> {
    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect((addr, port)).await?;

    let start = std::time::Instant::now();
    socket.send(&[0u8]).await?;

    let mut buf = [0u8; 64];
    match timeout(test_timeout, socket.recv(&mut buf)).await {
        Ok(Ok(_)) => Ok(start.elapsed()),
        Ok(Err(e)) => Err(SpeedTestError::Io(e)),
        Err(_) => Err(SpeedTestError::Timeout(test_timeout)),
    }
}

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

type ClientCacheInner = HashMap<(String, u16, bool), reqwest::Client>;

fn client_cache() -> &'static Mutex<ClientCacheInner> {
    static CACHE: OnceLock<Mutex<ClientCacheInner>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Reset the client cache — exposed for testing.
#[doc(hidden)]
pub fn reset_client_cache() {
    if let Ok(mut cache) = client_cache().lock() {
        cache.clear();
    }
}

/// Create a `reqwest::Client` with SOCKS5 proxy configured, using a cache to
/// avoid per-call connection pool creation overhead.
async fn create_socks5_client(
    proxy: &str,
    port: u16,
    socks5h: bool,
    timeout: Duration,
) -> Result<reqwest::Client, SpeedTestError> {
    let key = (proxy.to_string(), port, socks5h);
    if let Some(client) = client_cache().lock().unwrap().get(&key) {
        return Ok(client.clone()); // Client::clone() is cheap (Arc)
    }
    let scheme = if socks5h { "socks5h" } else { "socks5" };
    let proxy_url = format!("{scheme}://{proxy}:{port}");
    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| SpeedTestError::Proxy(e.to_string()))?)
        .timeout(timeout)
        .build()
        .map_err(SpeedTestError::Http)?;
    client_cache().lock().unwrap().insert(key, client.clone());
    Ok(client)
}

/// Real ping: send HTTP HEAD requests through SOCKS5 proxy to `url`, measure fastest response time.
///
/// Uses `socks5://` (NOT `socks5h://`) — the proxy resolves DNS locally.
/// Up to `retries` requests are sent concurrently; the fastest 2xx response wins.
/// On success, optionally fetches IP info from `ip_api_url` through the same proxy.
///
/// Optimizations (from sing-box/mihomo patterns):
/// - HEAD instead of GET (avoids downloading response body)
/// - No redirect following (probe URL doesn't redirect)
/// - Parallel retries (first success wins, instead of sequential with sleep)
/// - Pool max idle per host = 0 (no keepalive reuse across measurements)
pub async fn real_ping(
    proxy: &str,
    port: u16,
    url: &str,
    ip_api_url: &str,
    test_timeout: Duration,
    retries: u32,
) -> Result<RealPingResult, SpeedTestError> {
    let scheme = "socks5";
    let proxy_url = format!("{scheme}://{proxy}:{port}");
    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| SpeedTestError::Proxy(e.to_string()))?)
        .timeout(test_timeout)
        .redirect(reqwest::redirect::Policy::none())
        .pool_max_idle_per_host(0)
        .build()
        .map_err(SpeedTestError::Http)?;

    let start = std::time::Instant::now();
    let mut best_latency: Option<u64> = None;
    let mut last_error: Option<SpeedTestError> = None;

    let retries = retries.max(1);
    let futs: Vec<_> = (0..retries).map(|_| client.head(url).send()).collect();
    let mut stream = futures_util::stream::iter(futs).buffer_unordered(retries as usize);
    while let Some(result) = stream.next().await {
        match result {
            Ok(resp) if resp.status().is_success() => {
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "elapsed ms fits in u64 (max ~584M years)"
                )]
                let elapsed = start.elapsed().as_millis() as u64;
                match best_latency {
                    None => best_latency = Some(elapsed),
                    Some(best) if elapsed < best => best_latency = Some(elapsed),
                    _ => {}
                }
            }
            Ok(resp) => {
                last_error = Some(SpeedTestError::Http(resp.error_for_status().unwrap_err()));
            }
            Err(e) => {
                last_error = Some(SpeedTestError::Http(e));
            }
        }
    }

    let latency_ms = best_latency.ok_or_else(|| {
        last_error.unwrap_or_else(|| SpeedTestError::Proxy("all retries failed".to_string()))
    })?;

    // Fetch IP info on success — use separate client with default redirect policy
    let ip_info = {
        match reqwest::Client::builder()
            .proxy(
                reqwest::Proxy::all(&proxy_url)
                    .map_err(|e| SpeedTestError::Proxy(e.to_string()))?,
            )
            .timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(0)
            .build()
        {
            Ok(ip_client) => match ip_client.get(ip_api_url).send().await {
                Ok(resp) => match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        let ip = json.get("query").and_then(|v| v.as_str()).unwrap_or("-");
                        let country = json.get("country").and_then(|v| v.as_str()).unwrap_or("-");
                        Some(format!("{ip} | {country}"))
                    }
                    Err(_) => None,
                },
                Err(_) => None,
            },
            Err(_) => None,
        }
    };

    Ok(RealPingResult {
        latency_ms,
        ip_info,
    })
}

/// Speed test: download `url` through SOCKS5 proxy, measure throughput.
/// Streams for at least `min_duration` up to `max_duration`.
/// Returns bits per second.
pub async fn speed_test(
    proxy: &str,
    port: u16,
    url: &str,
    min_duration: Duration,
    max_duration: Duration,
) -> Result<u64, SpeedTestError> {
    let client =
        create_socks5_client(proxy, port, true, max_duration + Duration::from_secs(5)).await?;

    let start = std::time::Instant::now();
    let resp = client.get(url).send().await?;
    let resp = resp.error_for_status()?;

    let mut total_bytes: u64 = 0;
    let deadline = start + max_duration;

    // Use streaming to read chunks
    let stream = resp.bytes_stream();
    tokio::pin!(stream);

    while std::time::Instant::now() < deadline {
        match timeout(Duration::from_secs(10), stream.as_mut().next()).await {
            Ok(Some(Ok(chunk))) => {
                total_bytes += chunk.len() as u64;
                // If we've met min duration and have data, we can stop
                if start.elapsed() >= min_duration {
                    break;
                }
            }
            Ok(Some(Err(e))) => return Err(SpeedTestError::Http(e)),
            Ok(None) | Err(_) => break,
        }
    }

    let elapsed = start.elapsed();
    if elapsed.as_secs() == 0 {
        return Err(SpeedTestError::Timeout(max_duration));
    }
    // bits per second = (bytes * 8) / seconds
    let bits = total_bytes * 8;
    Ok(bits / elapsed.as_secs())
}

/// UDP test: verify UDP forwarding through SOCKS5 proxy via UDP ASSOCIATE.
/// Sends a small DNS-like packet to 1.1.1.1:53 and checks for response.
/// Returns round-trip duration.
pub async fn udp_test(
    proxy: &str,
    port: u16,
    test_timeout: Duration,
) -> Result<Duration, SpeedTestError> {
    // 1. Establish UDP ASSOCIATE via TCP to SOCKS5 proxy
    let proxy_addr = format!("{proxy}:{port}");
    let tcp = timeout(test_timeout, TcpStream::connect(&proxy_addr))
        .await
        .map_err(|_| SpeedTestError::Timeout(test_timeout))??;

    // SOCKS5 handshake: no auth
    let handshake = [5u8, 1, 0]; // VER=5, NMETHODS=1, METHOD=0(no auth)
    let (mut r, mut w) = tcp.into_split();

    w.write_all(&handshake).await?;
    let mut response = [0u8; 2];
    r.read_exact(&mut response).await?;
    if response != [5, 0] {
        return Err(SpeedTestError::Proxy("SOCKS5 handshake failed".into()));
    }

    // UDP ASSOCIATE request
    // VER=5, CMD=3(UDP ASSOCIATE), RSV=0, ATYP=1(IPv4), BND.ADDR=0, BND.PORT=0
    let mut req = Vec::with_capacity(10);
    req.extend_from_slice(&[5, 3, 0, 1, 0, 0, 0, 0, 0, 0]);
    w.write_all(&req).await?;

    let mut header = [0u8; 10];
    r.read_exact(&mut header).await?;
    if header[1] != 0 {
        return Err(SpeedTestError::Proxy(
            "SOCKS5 UDP ASSOCIATE rejected".into(),
        ));
    }

    // Parse relay address from response
    // ATYP=1 => IPv4 (4 bytes addr + 2 bytes port = 6 bytes after header[3])
    // ATYP=4 => IPv6 (16 bytes addr + 2 bytes port = 18 bytes after header[3])
    let atyp = header[3];
    let (relay_addr, relay_port) = if atyp == 1 {
        let ip = std::net::Ipv4Addr::new(header[4], header[5], header[6], header[7]);
        let port = u16::from_be_bytes([header[8], header[9]]);
        (ip.to_string(), port)
    } else if atyp == 4 {
        // Read remaining bytes for full IPv6 response (header is only 10 bytes)
        let mut extra = [0u8; 10];
        r.read_exact(&mut extra).await?;
        let mut full = [0u8; 20];
        full[..10].copy_from_slice(&header);
        full[10..].copy_from_slice(&extra);
        let ip = std::net::Ipv6Addr::new(
            u16::from_be_bytes([full[4], full[5]]),
            u16::from_be_bytes([full[6], full[7]]),
            u16::from_be_bytes([full[8], full[9]]),
            u16::from_be_bytes([full[10], full[11]]),
            u16::from_be_bytes([full[12], full[13]]),
            u16::from_be_bytes([full[14], full[15]]),
            u16::from_be_bytes([full[16], full[17]]),
            u16::from_be_bytes([full[18], full[19]]),
        );
        let mut port_buf = [0u8; 2];
        r.read_exact(&mut port_buf).await?;
        let port = u16::from_be_bytes(port_buf);
        (ip.to_string(), port)
    } else {
        return Err(SpeedTestError::Proxy("UDP ASSOCIATE unknown ATYP".into()));
    };

    // 2. Send a DNS query packet through UDP relay
    // DNS query: A record for example.com
    let dns_query: Vec<u8> = vec![
        // DNS header: id=0x1234, flags=0x0100 (standard query), QDCOUNT=1
        0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // Question: example.com (7 letters)
        0x07, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 0x03, b'c', b'o', b'm', 0x00, 0x00,
        0x01, // QTYPE=A
        0x00, 0x01, // QCLASS=IN
    ];

    // SOCKS5 UDP request header: RSV, FRAG, ATYP, DST.ADDR, DST.PORT
    let mut udp_packet = Vec::with_capacity(dns_query.len() + 16);
    udp_packet.extend_from_slice(&[0, 0, 0]); // RSV=0, FRAG=0
    udp_packet.push(1); // ATYP=1 (IPv4)
    udp_packet.extend_from_slice(&[1, 1, 1, 1]); // 1.1.1.1
    udp_packet.extend_from_slice(&[0, 53]); // port 53
    udp_packet.extend_from_slice(&dns_query);

    // Bind to a local UDP socket and send to the relay address
    let udp_sock = timeout(test_timeout, tokio::net::UdpSocket::bind("0.0.0.0:0"))
        .await
        .map_err(|_| SpeedTestError::Timeout(test_timeout))??;
    udp_sock.connect((relay_addr.as_str(), relay_port)).await?;

    let test_start = std::time::Instant::now();
    udp_sock.send(&udp_packet).await?;

    // 3. Wait for response
    let mut buf = vec![0u8; 1500];
    let recv_fut = udp_sock.recv(&mut buf);
    match timeout(test_timeout, recv_fut).await {
        Ok(Ok(n)) if n > 10 => {
            // Got a response — UDP forwarding works
            Ok(test_start.elapsed())
        }
        Ok(Ok(_)) => Err(SpeedTestError::Proxy("UDP response too short".into())),
        Ok(Err(e)) => Err(SpeedTestError::Io(e)),
        Err(_) => Err(SpeedTestError::Timeout(test_timeout)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn wait_for_socks5_timeout() {
        // Port 1 is privileged — no real service listens there
        let result = wait_for_socks5("127.0.0.1", 1, Duration::from_millis(100)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn client_cache_reset() {
        reset_client_cache();
        assert!(client_cache().lock().unwrap().is_empty());
    }
}
