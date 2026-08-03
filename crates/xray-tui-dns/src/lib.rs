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
/// file is stale (mtime older than [`DNSCRYPT_CACHE_MAX_AGE`]). Missing,
/// unreadable, or empty/corrupt caches yield `None` so they get re-downloaded
/// instead of serving a 0-server config forever.
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
    let stale = metadata
        .modified()
        .ok()
        .and_then(|m| m.elapsed().ok())
        .is_some_and(|age| age > DNSCRYPT_CACHE_MAX_AGE);
    (Some(parsed), stale)
}

/// Downloads the `DNSCrypt` public resolver list under a hard 10s deadline;
/// without the timeout a blocked network hangs every lookup forever.
async fn download_dnscrypt_resolvers() -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let response = client
        .get(DNSCRYPT_RESOLVERS_URL)
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
    let cache_file = cache_dir.join(DNSCRYPT_RESOLVERS_CACHE);
    let (cached, cache_stale) = read_dnscrypt_cache(&cache_file).await;

    // Re-download when the cache is missing/empty or stale. A failed refresh
    // keeps the (stale) cache; an empty download is never written and falls
    // back to the system resolver config below.
    let downloaded = if cached.is_none() || cache_stale {
        match download_dnscrypt_resolvers().await {
            Ok(text) => {
                let parsed = parse_dnscrypt_lines(&text);
                if parsed.is_empty() {
                    None
                } else {
                    write_dnscrypt_cache(cache_dir, &cache_file, &text).await?;
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

    #[test]
    fn parse_dnscrypt_lines_skips_garbage_and_ipv6_stamps() {
        // dns.aa.net.uk published as an IPv4 DoH stamp and as an IPv6 DoH
        // stamp; the IPv6-only entry must be filtered out (allow_ipv6=false).
        let text = concat!(
            "# DNSCrypt resolver list (markdown header)\n",
            "sdns://AgcAAAAAAAAADTIxNy4xNjkuMjAuMjIADWRucy5hYS5uZXQudWsKL2Rucy1xdWVyeQ\n",
            "not a stamp\n",
            "sdns://AgcAAAAAAAAAEFsyMDAxOjhiMDo6MjAyMl0ADWRucy5hYS5uZXQudWsKL2Rucy1xdWVyeQ\n",
        );
        let parsed = parse_dnscrypt_lines(text);
        assert_eq!(parsed.len(), 1, "only the IPv4 stamp should parse");
        assert!(parsed[0].ip.is_ipv4());
    }

    #[tokio::test]
    async fn stale_cache_is_refreshed_or_falls_back_to_stale() -> anyhow::Result<()> {
        // A cache older than 7 days triggers a refresh attempt; a failed
        // refresh keeps the stale entries, a successful one rewrites the
        // cache. Either way a non-empty config must come back.
        let dir = tempfile::tempdir()?;
        let cache = dir.path().join(DNSCRYPT_RESOLVERS_CACHE);
        tokio::fs::write(
            &cache,
            "sdns://AgcAAAAAAAAADTIxNy4xNjkuMjAuMjIADWRucy5hYS5uZXQudWsKL2Rucy1xdWVyeQ\n",
        )
        .await?;
        let file = std::fs::File::options().write(true).open(&cache)?;
        let eight_days = Duration::from_hours(192);
        file.set_times(
            std::fs::FileTimes::new().set_modified(std::time::SystemTime::now() - eight_days),
        )?;
        drop(file);

        let cfg = get_dns_servers(dir.path()).await?;
        assert!(
            !cfg.name_servers().is_empty(),
            "stale cache must still yield resolvers"
        );
        Ok(())
    }

    #[tokio::test]
    async fn empty_cache_is_not_used() -> anyhow::Result<()> {
        // An empty cache file (e.g. a rate-limit HTML page saved as 200) must
        // never yield a 0-server config: get_dns_servers re-downloads the
        // list or falls back to the system resolver config.
        let dir = tempfile::tempdir()?;
        let cache = dir.path().join(DNSCRYPT_RESOLVERS_CACHE);
        tokio::fs::write(&cache, "").await?;
        let cfg = get_dns_servers(dir.path()).await.expect("resolver config");
        assert!(
            !cfg.name_servers().is_empty(),
            "empty cache must not yield 0 servers"
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
