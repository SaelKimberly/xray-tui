use std::collections::HashSet;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use dns_stamp_parser::{Addr, DnsStamp, Props};
use hickory_resolver::{
    Resolver, TokioResolver,
    config::{
        ConnectionConfig, NameServerConfig, ResolverConfig, ResolverOpts, ServerOrderingStrategy,
    },
    net::runtime::TokioRuntimeProvider,
    system_conf,
};
use tokio::io::AsyncWriteExt;

const DNSCRYPT_RESOLVERS_URL: &str =
    "https://raw.githubusercontent.com/DNSCrypt/dnscrypt-resolvers/master/v3/public-resolvers.md";
const DNSCRYPT_RESOLVERS_CACHE: &str = "dsncrypt.resolvers.txt";
/// Cached resolver lists older than this are refreshed on the next lookup.
const DNSCRYPT_CACHE_MAX_AGE: Duration = Duration::from_hours(168);

static DEFAULT_RESOLVER_OPTS: std::sync::LazyLock<ResolverOpts> = std::sync::LazyLock::new(|| {
    let mut resolver_opts = ResolverOpts::default();
    resolver_opts.timeout = Duration::from_millis(500);
    resolver_opts.server_ordering_strategy = ServerOrderingStrategy::RoundRobin;
    resolver_opts
});

fn sdns_to_nsc(url: &url::Url, allow_ipv6: bool) -> Option<NameServerConfig> {
    let Ok(stamp) = DnsStamp::decode(url.as_str()) else {
        return None;
    };
    match stamp {
        DnsStamp::DnsPlain(s) => {
            if (allow_ipv6 || s.addr.is_ipv4())
                && s.props.contains(Props::NO_LOGS)
                && s.props.contains(Props::NO_FILTER)
            {
                Some(NameServerConfig::udp(s.addr))
            } else {
                None
            }
        }
        DnsStamp::DnsOverHttps(s) => {
            if let Some(Addr::SocketAddr(addr)) = s.addr
                && (allow_ipv6 || addr.is_ipv4())
                && s.props.contains(Props::NO_LOGS)
                && s.props.contains(Props::NO_FILTER)
            {
                let mut conn =
                    ConnectionConfig::https(Arc::from(s.hostname), Some(Arc::from(s.path)));
                conn.port = addr.port();
                Some(NameServerConfig::new(addr.ip(), true, vec![conn]))
            } else {
                None
            }
        }
        DnsStamp::DnsOverTls(s) => {
            if let Some(Addr::SocketAddr(addr)) = s.addr
                && (allow_ipv6 || addr.is_ipv4())
                && s.props.contains(Props::NO_LOGS)
                && s.props.contains(Props::NO_FILTER)
            {
                let mut conn = ConnectionConfig::tls(Arc::from(s.hostname));
                conn.port = addr.port();
                Some(NameServerConfig::new(addr.ip(), true, vec![conn]))
            } else {
                None
            }
        }
        DnsStamp::DnsOverQuic(s) => {
            if let Some(Addr::SocketAddr(addr)) = s.addr
                && (allow_ipv6 || addr.is_ipv4())
                && s.props.contains(Props::NO_LOGS)
                && s.props.contains(Props::NO_FILTER)
            {
                let mut conn = ConnectionConfig::quic(Arc::from(s.hostname));
                conn.port = addr.port();
                Some(NameServerConfig::new(addr.ip(), true, vec![conn]))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Parses `DNSCrypt` resolver list text (fetched markdown or cache file) into
/// name server configs. Lines that are not `sdns://` stamps, or that do not
/// decode to a usable resolver, are skipped.
fn parse_dnscrypt_lines(text: &str) -> Vec<NameServerConfig> {
    text.lines()
        .filter(|s| s.starts_with("sdns://"))
        .filter_map(|s| s.parse::<url::Url>().ok())
        .filter_map(|url| sdns_to_nsc(&url, false))
        .collect()
}

/// Reads the cache file, returning the parsed name servers and whether the
/// file is stale (mtime older than [`DNSCRYPT_CACHE_MAX_AGE`], or undatable).
/// Missing, unreadable, or empty/corrupt caches yield `None` so they get
/// re-downloaded instead of serving a 0-server config forever.
async fn read_dnscrypt_cache(cache_file: &Path) -> (Option<Vec<NameServerConfig>>, bool) {
    let Ok(metadata) = tokio::fs::metadata(cache_file).await else {
        return (None, false);
    };
    let Ok(contents) = tokio::fs::read_to_string(cache_file).await else {
        return (None, false);
    };
    let parsed = parse_dnscrypt_lines(&contents);
    if parsed.is_empty() {
        return (None, false);
    }
    // A future-dated mtime makes `elapsed()` fail; treat that as stale so an
    // undatable cache is eventually re-fetched instead of living forever.
    let stale = metadata
        .modified()
        .ok()
        .is_some_and(|mtime| mtime.elapsed().map_or(true, |age| age > DNSCRYPT_CACHE_MAX_AGE));
    (Some(parsed), stale)
}

/// Downloads the `DNSCrypt` public resolver list under a hard 10s deadline;
/// without the timeout a blocked network hangs every lookup forever.
async fn download_dnscrypt_resolvers(resolvers_url: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let response = client
        .get(resolvers_url)
        .send()
        .await?
        .error_for_status()?;
    Ok(response.text().await?)
}

/// Atomically writes the fetched list to the cache file (temp file + rename,
/// so a crash or concurrent reader never observes a partial cache). Only the
/// `sdns://` lines are kept, deduplicated, one per line.
async fn write_dnscrypt_cache(
    cache_dir: &Path,
    cache_file: &Path,
    text: &str,
) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(cache_dir).await?;
    let tmp_path = cache_file.with_file_name(format!(
        "{}.tmp{}",
        cache_file
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default(),
        std::process::id()
    ));
    let write = async {
        let mut file = tokio::fs::File::create(&tmp_path).await?;
        let mut seen = HashSet::new();
        for line in text.lines().filter(|s| s.starts_with("sdns://")) {
            if seen.insert(line.to_owned()) {
                file.write_all(line.as_bytes()).await?;
                file.write_all(b"\n").await?;
            }
        }
        file.sync_all().await?;
        tokio::fs::rename(&tmp_path, cache_file).await
    };
    if let Err(e) = write.await {
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return Err(e.into());
    }
    Ok(())
}

async fn get_dns_servers(cache_dir: &Path) -> anyhow::Result<ResolverConfig> {
    get_dns_servers_from(cache_dir, DNSCRYPT_RESOLVERS_URL).await
}

/// Core implementation of [`get_dns_servers`]; the resolver list URL is a
/// parameter so tests can exercise the download paths hermetically.
async fn get_dns_servers_from(
    cache_dir: &Path,
    resolvers_url: &str,
) -> anyhow::Result<ResolverConfig> {
    let cache_file = cache_dir.join(DNSCRYPT_RESOLVERS_CACHE);
    let (cached, cache_stale) = read_dnscrypt_cache(&cache_file).await;

    // Re-download when the cache is missing/empty or stale. A failed refresh
    // keeps the (stale) cache; an empty download is never written; a failed
    // cache write only costs the next run's refresh, never this lookup.
    let downloaded = if cached.is_none() || cache_stale {
        match download_dnscrypt_resolvers(resolvers_url).await {
            Ok(text) => {
                let parsed = parse_dnscrypt_lines(&text);
                if parsed.is_empty() {
                    None
                } else {
                    if let Err(e) = write_dnscrypt_cache(cache_dir, &cache_file, &text).await {
                        tracing::warn!(
                            error = %e,
                            "failed to write DNSCrypt resolver cache; using in-memory list"
                        );
                    }
                    Some(parsed)
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };

    if let Some(name_servers) = downloaded.or(cached) {
        let mut config = ResolverConfig::default();
        for ns in name_servers {
            config.add_name_server(ns);
        }
        Ok(config)
    } else {
        // Last resort: the OS-provided resolver configuration (e.g.
        // /etc/resolv.conf) so lookups still work without the DNSCrypt list.
        Ok(system_conf::read_system_conf().map(|(cfg, _)| cfg)?)
    }
}

pub struct DnsResolver {
    cache_dir: PathBuf,
    resolver: tokio::sync::OnceCell<TokioResolver>,
}

impl DnsResolver {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
            resolver: tokio::sync::OnceCell::new(),
        }
    }

    pub async fn lookup_ip(&self, hostname: &str, allow_ipv6: bool) -> anyhow::Result<Vec<IpAddr>> {
        if let Ok(ip) = hostname.parse::<IpAddr>() {
            return Ok(vec![ip]);
        }
        if self.resolver.get().is_none() {
            let resolver = self.init().await?;
            let _ = self.resolver.set(resolver);
        }
        let resolver = self.resolver.get().expect("resolver set above");
        Ok(resolver
            .lookup_ip(hostname)
            .await?
            .iter()
            .filter(|ip| allow_ipv6 || ip.is_ipv4())
            .collect())
    }

    async fn init(&self) -> anyhow::Result<TokioResolver> {
        let resolver_cfg = get_dns_servers(&self.cache_dir).await?;
        let resolver_opt = DEFAULT_RESOLVER_OPTS.clone();
        Ok(
            Resolver::builder_with_config(resolver_cfg, TokioRuntimeProvider::default())
                .with_options(resolver_opt)
                .build()?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A dns.aa.net.uk `DoH` stamp published with an IPv4 address.
    const STAMP_IPV4: &str =
        "sdns://AgcAAAAAAAAADTIxNy4xNjkuMjAuMjIADWRucy5hYS5uZXQudWsKL2Rucy1xdWVyeQ";
    /// The same resolver published with an IPv6-only address.
    const STAMP_IPV6: &str =
        "sdns://AgcAAAAAAAAAEFsyMDAxOjhiMDo6MjAyMl0ADWRucy5hYS5uZXQudWsKL2Rucy1xdWVyeQ";
    /// Nothing listens here; downloads fail fast (ECONNREFUSED).
    const UNREACHABLE_URL: &str = "http://127.0.0.1:1/unreachable";

    /// Serves `body` as a single HTTP response on an ephemeral loopback port,
    /// returning the URL. Lets the download path run without the network.
    async fn serve(body: String) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        std::mem::drop(tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let body = body.clone();
                tokio::spawn(async move {
                    use tokio::io::AsyncReadExt;
                    let mut buf = [0_u8; 4096];
                    let _ = stream.read(&mut buf).await;
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                });
            }
        }));
        format!("http://{addr}/public-resolvers.md")
    }

    /// Asserts the result is either a non-empty config, or an error that is
    /// justified by the host having no OS-level resolver config to fall back
    /// to. Never tolerates a 0-server Ok config.
    fn assert_usable(result: anyhow::Result<ResolverConfig>) {
        match result {
            Ok(cfg) => assert!(
                !cfg.name_servers().is_empty(),
                "resolver config must not have 0 name servers"
            ),
            Err(_) => assert!(
                system_conf::read_system_conf().is_err(),
                "get_dns_servers must not fail when system resolvers are available"
            ),
        }
    }

    #[test]
    fn parse_dnscrypt_lines_skips_garbage_and_ipv6_stamps() {
        // dns.aa.net.uk published as an IPv4 DoH stamp and as an IPv6 DoH
        // stamp; the IPv6-only entry must be filtered out (allow_ipv6=false).
        let text = format!(
            "# DNSCrypt resolver list (markdown header)\n{STAMP_IPV4}\nnot a stamp\n{STAMP_IPV6}\n"
        );
        let parsed = parse_dnscrypt_lines(&text);
        assert_eq!(parsed.len(), 1, "only the IPv4 stamp should parse");
        assert!(parsed[0].ip.is_ipv4());
    }

    #[tokio::test]
    async fn stale_cache_is_refreshed_or_falls_back_to_stale() -> anyhow::Result<()> {
        // A cache older than 7 days triggers a refresh attempt; with the list
        // unreachable the refresh fails and the stale entries must be kept.
        let dir = tempfile::tempdir()?;
        let cache = dir.path().join(DNSCRYPT_RESOLVERS_CACHE);
        tokio::fs::write(&cache, format!("{STAMP_IPV4}\n")).await?;
        let file = std::fs::File::options().write(true).open(&cache)?;
        file.set_times(
            std::fs::FileTimes::new()
                .set_modified(std::time::SystemTime::now() - Duration::from_hours(192)),
        )?;
        drop(file);

        let cfg = get_dns_servers_from(dir.path(), UNREACHABLE_URL)
            .await
            .expect("failed refresh must keep the stale cache");
        assert_eq!(cfg.name_servers().len(), 1, "stale cache entry must be kept");
        Ok(())
    }

    #[tokio::test]
    async fn empty_cache_is_not_used() -> anyhow::Result<()> {
        // An empty cache file (e.g. a rate-limit HTML page saved as 200) must
        // never yield a 0-server config: re-download, and if that fails, the
        // system resolver config.
        let dir = tempfile::tempdir()?;
        let cache = dir.path().join(DNSCRYPT_RESOLVERS_CACHE);
        tokio::fs::write(&cache, "").await?;
        let result = get_dns_servers_from(dir.path(), UNREACHABLE_URL).await;
        assert_usable(result);
        Ok(())
    }

    #[tokio::test]
    async fn empty_download_is_never_written() -> anyhow::Result<()> {
        // A 200 response without any stamps (e.g. a rate-limit HTML page)
        // must not be written to the cache and must not yield 0 servers.
        let url = serve("<html>rate limited</html>".to_string()).await;
        let dir = tempfile::tempdir()?;
        let cache = dir.path().join(DNSCRYPT_RESOLVERS_CACHE);
        tokio::fs::write(&cache, "").await?;
        let result = get_dns_servers_from(dir.path(), &url).await;
        assert_usable(result);
        let contents = tokio::fs::read_to_string(&cache).await?;
        assert!(
            contents.is_empty(),
            "empty download must not be written to cache"
        );
        Ok(())
    }

    #[tokio::test]
    async fn download_succeeds_writes_cache() -> anyhow::Result<()> {
        // A successful first download populates the cache (happy path).
        let url = serve(format!("# DNSCrypt resolver list\n{STAMP_IPV4}\n")).await;
        let dir = tempfile::tempdir()?;
        let cache = dir.path().join(DNSCRYPT_RESOLVERS_CACHE);
        let cfg = get_dns_servers_from(dir.path(), &url).await?;
        assert_eq!(cfg.name_servers().len(), 1);
        let contents = tokio::fs::read_to_string(&cache).await?;
        assert_eq!(
            contents,
            format!("{STAMP_IPV4}\n"),
            "cache must hold the sdns:// lines"
        );
        Ok(())
    }

    #[tokio::test]
    async fn download_write_failure_keeps_in_memory_list() -> anyhow::Result<()> {
        // The cache path is a directory, so the atomic rename fails; the
        // freshly downloaded list must still be returned — a failed cache
        // write never fails DNS init.
        let url = serve(format!("# DNSCrypt resolver list\n{STAMP_IPV4}\n")).await;
        let dir = tempfile::tempdir()?;
        tokio::fs::create_dir(dir.path().join(DNSCRYPT_RESOLVERS_CACHE)).await?;
        let cfg = get_dns_servers_from(dir.path(), &url).await?;
        assert_eq!(cfg.name_servers().len(), 1);

        let mut entries = tokio::fs::read_dir(dir.path()).await?;
        let mut count = 0;
        while entries.next_entry().await?.is_some() {
            count += 1;
        }
        assert_eq!(
            count, 1,
            "failed cache write must not leave temp files behind"
        );
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires network (fetches dnscrypt resolver list)"]
    async fn test_lookup_ip() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let resolver = DnsResolver::new(dir.path());
        let ips = resolver.lookup_ip("example.com", true).await?;
        assert!(!ips.is_empty());
        Ok(())
    }
}
