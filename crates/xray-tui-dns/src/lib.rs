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
};
use tokio::io::AsyncWriteExt;

const DNSCRYPT_RESOLVERS_URL: &str =
    "https://raw.githubusercontent.com/DNSCrypt/dnscrypt-resolvers/master/v3/public-resolvers.md";
const DNSCRYPT_RESOLVERS_CACHE: &str = "dsncrypt.resolvers.txt";

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

async fn get_dns_servers(cache_dir: &Path) -> anyhow::Result<ResolverConfig> {
    let cache_file = cache_dir.join(DNSCRYPT_RESOLVERS_CACHE);
    let links_raw = if cache_file.is_file() {
        let contents = tokio::fs::read_to_string(&cache_file).await?;
        contents
            .lines()
            .filter_map(|s| s.parse::<url::Url>().ok())
            .filter_map(|url| sdns_to_nsc(&url, false))
            .collect::<Vec<_>>()
    } else {
        let response = reqwest::get(DNSCRYPT_RESOLVERS_URL)
            .await?
            .error_for_status()?;
        let result: HashSet<url::Url> = response
            .text()
            .await?
            .lines()
            .filter_map(|s| {
                if s.starts_with("sdns://") {
                    s.parse::<url::Url>().ok()
                } else {
                    None
                }
            })
            .filter(|url| sdns_to_nsc(url, false).is_some())
            .collect();

        tokio::fs::create_dir_all(cache_dir).await?;
        match tokio::fs::File::create_new(&cache_file).await {
            Ok(mut file) => {
                for link in &result {
                    file.write_all(link.as_str().as_bytes()).await?;
                    file.write_all(b"\n").await?;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e.into()),
        }

        result
            .into_iter()
            .filter_map(|url| sdns_to_nsc(&url, false))
            .collect()
    };

    let mut config = ResolverConfig::default();
    for link in links_raw {
        config.add_name_server(link);
    }
    Ok(config)
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
