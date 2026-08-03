//! Whitelist feature extraction for server names.
//!
//! `HostFeaturesChecker` answers "is this server name on the Russian mobile
//! internet whitelist?" for SNI hostnames, exact IPv4 addresses, and IPv4 CIDR
//! ranges. Membership checks are backed by bloom filters (fast-negative guard)
//! plus exact `HashSet`/interval verification (zero false positives).
//!
//! The three whitelist data files come from the upstream repository
//! [hxehex/russia-mobile-internet-whitelist](https://github.com/hxehex/russia-mobile-internet-whitelist).
//! Like `xray-tui-geoip` and `xray-tui-dns`, this crate downloads any file that
//! is not present when it loads — no data is vendored into the repo.

use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::path::Path;

use fastbloom::BloomFilter;
use rustls::pki_types::{IpAddr, ServerName};

/// Upstream repository supplying the whitelist files
/// (https://github.com/hxehex/russia-mobile-internet-whitelist).
const SNI_WHITELIST_URL: &str =
    "https://raw.githubusercontent.com/hxehex/russia-mobile-internet-whitelist/refs/heads/main/whitelist.txt";
const IP_WHITELIST_URL: &str =
    "https://raw.githubusercontent.com/hxehex/russia-mobile-internet-whitelist/refs/heads/main/ipwhitelist.txt";
const CIDR_WHITELIST_URL: &str =
    "https://raw.githubusercontent.com/hxehex/russia-mobile-internet-whitelist/refs/heads/main/cidrwhitelist.txt";

/// Feature flags describing a server name's relationship to the whitelists.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HostFeatures {
    pub sni_whitelisted: bool,
    pub ip_whitelisted: bool,
    pub cidr_whitelisted: bool,
}

/// SNI/exact-IP/CIDR whitelist membership checker backed by bloom filters
/// (fast-negative guard) plus exact HashSet/interval verification (zero false
/// positives). IPv4-only; ported from sub-healer's `WhitelistChecker`.
#[derive(Debug)]
pub struct HostFeaturesChecker {
    /// SNI whitelist (whitelist.txt) — hostnames
    sni_bloom: BloomFilter,
    sni_set: HashSet<String>,
    /// IP whitelist (ipwhitelist.txt) — IPv4 as u32 big-endian
    ip_bloom: BloomFilter,
    ip_set: HashSet<u32>,
    /// CIDR whitelist (cidrwhitelist.txt) — sorted (start_u32, end_u32) intervals
    cidr_ranges: Vec<(u32, u32)>,
}

impl HostFeaturesChecker {
    const FP_RATE: f64 = 0.01;

    /// Load all three whitelist files.
    ///
    /// # Errors
    ///
    /// Returns an error if any file cannot be read.
    pub fn new(sni_path: &Path, ip_path: &Path, cidr_path: &Path) -> anyhow::Result<Self> {
        let (sni_bloom, sni_set) = Self::load_sni(sni_path)?;
        let (ip_bloom, ip_set) = Self::load_ip(ip_path)?;
        let cidr_ranges = Self::load_cidr(cidr_path)?;

        Ok(Self {
            sni_bloom,
            sni_set,
            ip_bloom,
            ip_set,
            cidr_ranges,
        })
    }

    /// Like [`Self::new`], but first downloads any of the three files that is
    /// missing, fetching it from the upstream
    /// hxehex/russia-mobile-internet-whitelist repository. Existing files are
    /// never re-downloaded. Errors on download or read failure.
    pub async fn load(
        sni_path: &Path,
        ip_path: &Path,
        cidr_path: &Path,
    ) -> anyhow::Result<Self> {
        Self::ensure_downloaded(sni_path, ip_path, cidr_path).await?;
        Self::new(sni_path, ip_path, cidr_path)
    }

    /// Download any of the three whitelist files that does not exist at its
    /// path. Presence check only — existing files are left untouched. Errors
    /// if any download or file write fails.
    pub async fn ensure_downloaded(
        sni_path: &Path,
        ip_path: &Path,
        cidr_path: &Path,
    ) -> anyhow::Result<()> {
        ensure_file(sni_path, SNI_WHITELIST_URL).await?;
        ensure_file(ip_path, IP_WHITELIST_URL).await?;
        ensure_file(cidr_path, CIDR_WHITELIST_URL).await?;
        Ok(())
    }

    /// Main API. Whitelist-membership features for any `ServerName`:
    /// `DnsName` → SNI check; `IpAddress` IPv4 → exact-IP + CIDR checks;
    /// `IpAddress` IPv6 / unknown → empty feature set (whitelists are
    /// IPv4-only).
    #[must_use]
    pub fn get_host_features(&self, server_name: &ServerName<'_>) -> HostFeatures {
        match server_name {
            ServerName::DnsName(dns) => HostFeatures {
                sni_whitelisted: self.is_sni_whitelisted(dns.as_ref()),
                ..HostFeatures::default()
            },
            ServerName::IpAddress(IpAddr::V4(v4)) => HostFeatures {
                ip_whitelisted: self.is_ip_whitelisted(std::net::Ipv4Addr::from(*v4)),
                cidr_whitelisted: self.is_cidr_whitelisted(std::net::Ipv4Addr::from(*v4)),
                ..HostFeatures::default()
            },
            // IpAddr::V6 → whitelists are IPv4-only; `ServerName` is #[non_exhaustive] → catch-all
            ServerName::IpAddress(IpAddr::V6(_)) | _ => HostFeatures::default(),
        }
    }

    /// Whitelist membership for a plain `std::net::IpAddr` (IPv4 checks the
    /// exact-IP and CIDR lists; IPv6 yields an empty feature set).
    #[must_use]
    pub fn ip_features(&self, ip: std::net::IpAddr) -> HostFeatures {
        match ip {
            std::net::IpAddr::V4(v4) => HostFeatures {
                ip_whitelisted: self.is_ip_whitelisted(v4),
                cidr_whitelisted: self.is_cidr_whitelisted(v4),
                ..HostFeatures::default()
            },
            std::net::IpAddr::V6(_) => HostFeatures::default(),
        }
    }

    /// SNI whitelist membership for a hostname string. Returns `None` when the
    /// string is not a valid DNS name (IP literals, empty, malformed).
    #[must_use]
    pub fn sni_features(&self, hostname: &str) -> Option<bool> {
        ServerName::try_from(hostname)
            .ok()
            .map(|sn| self.get_host_features(&sn).sni_whitelisted)
    }

    // ── Loaders ──────────────────────────────────────────────────────────

    fn load_sni(path: &Path) -> anyhow::Result<(BloomFilter, HashSet<String>)> {
        let content = std::fs::read_to_string(path)?;
        let mut set = HashSet::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            set.insert(trimmed.to_ascii_lowercase());
        }

        let bloom = BloomFilter::with_false_pos(Self::FP_RATE).items(set.iter());
        Ok((bloom, set))
    }

    fn load_ip(path: &Path) -> anyhow::Result<(BloomFilter, HashSet<u32>)> {
        let content = std::fs::read_to_string(path)?;
        let mut set = HashSet::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let addr: Ipv4Addr = match trimmed.parse() {
                Ok(a) => a,
                Err(e) => {
                    tracing::debug!(line = trimmed, error = %e, "Skipping malformed IP");
                    continue;
                }
            };
            let key = u32::from_be_bytes(addr.octets());
            set.insert(key);
        }

        let bloom = BloomFilter::with_false_pos(Self::FP_RATE).items(set.iter());
        Ok((bloom, set))
    }

    fn load_cidr(path: &Path) -> anyhow::Result<Vec<(u32, u32)>> {
        let content = std::fs::read_to_string(path)?;
        let mut ranges = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Some((ip_str, mask_str)) = trimmed.split_once('/') else {
                tracing::debug!(line = trimmed, "Skipping malformed CIDR (no /)");
                continue;
            };
            let base: Ipv4Addr = match ip_str.parse() {
                Ok(a) => a,
                Err(e) => {
                    tracing::debug!(line = trimmed, error = %e, "Skipping malformed CIDR IP");
                    continue;
                }
            };
            let mask_bits: u8 = match mask_str.parse() {
                Ok(m) if m <= 32 => m,
                _ => {
                    tracing::debug!(line = trimmed, "Skipping malformed CIDR mask");
                    continue;
                }
            };
            let base_u32 = u32::from_be_bytes(base.octets());
            let mask = if mask_bits == 0 {
                0u32
            } else {
                (!0u32) << (32 - mask_bits)
            };
            let start = base_u32 & mask;
            let end = start | !mask;
            ranges.push((start, end));
        }

        ranges.sort_unstable_by_key(|r| r.0);
        Ok(ranges)
    }

    // ── Lookup methods ───────────────────────────────────────────────────

    /// Fast-negative bloom filter + HashSet verification.
    #[must_use]
    pub fn is_sni_whitelisted(&self, host: &str) -> bool {
        let lower = host.to_ascii_lowercase();
        if !self.sni_bloom.contains(&lower) {
            return false;
        }
        self.sni_set.contains(&lower)
    }

    /// Fast-negative bloom filter + HashSet verification.
    #[must_use]
    pub fn is_ip_whitelisted(&self, ip: Ipv4Addr) -> bool {
        let key = u32::from_be_bytes(ip.octets());
        if !self.ip_bloom.contains(&key) {
            return false;
        }
        self.ip_set.contains(&key)
    }

    /// `partition_point` on the sorted (start, end) intervals.
    #[must_use]
    pub fn is_cidr_whitelisted(&self, ip: Ipv4Addr) -> bool {
        let key = u32::from_be_bytes(ip.octets());
        let idx = self.cidr_ranges.partition_point(|&(s, _)| s <= key);
        idx > 0 && key <= self.cidr_ranges[idx - 1].1
    }
}

/// Fetch `url` to `path` only if the file is not already present.
async fn ensure_file(path: &Path, url: &str) -> anyhow::Result<()> {
    if path.is_file() {
        return Ok(());
    }
    let bytes = reqwest::get(url).await?.error_for_status()?.bytes().await?;
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        tokio::fs::create_dir_all(parent).await?;
    }
    let mut f = tokio::fs::File::create(path).await?;
    tokio::io::AsyncWriteExt::write_all(&mut f, &bytes).await?;
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp(content: &[u8]) -> std::path::PathBuf {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content).unwrap();
        let path = f.path().to_owned();
        // Keep the file alive (leaks the temp file, fine for tests)
        std::mem::forget(f);
        path
    }

    fn sni_file(hosts: &[&str]) -> std::path::PathBuf {
        write_temp(hosts.join("\n").as_bytes())
    }

    fn ip_file(ips: &[&str]) -> std::path::PathBuf {
        write_temp(ips.join("\n").as_bytes())
    }

    fn cidr_file(cidrs: &[&str]) -> std::path::PathBuf {
        write_temp(cidrs.join("\n").as_bytes())
    }

    /// Convenience: create a checker with the given sni/ip/cidr slices.
    fn make_checker(sni: &[&str], ip: &[&str], cidr: &[&str]) -> HostFeaturesChecker {
        HostFeaturesChecker::new(&sni_file(sni), &ip_file(ip), &cidr_file(cidr)).unwrap()
    }

    #[test]
    fn sni_present() {
        let checker = make_checker(&["example.com", "test.server.org"], &[], &[]);
        assert!(checker.is_sni_whitelisted("example.com"));
        assert!(checker.is_sni_whitelisted("Example.COM")); // case insensitive
        assert!(!checker.is_sni_whitelisted("unknown.com"));
    }

    #[test]
    fn sni_absent() {
        let checker = make_checker(&["known.example"], &[], &[]);
        assert!(!checker.is_sni_whitelisted("not.known.example"));
    }

    #[test]
    fn ip_present() {
        let checker = make_checker(&[], &["1.2.3.4", "10.0.0.1"], &[]);
        assert!(checker.is_ip_whitelisted(Ipv4Addr::new(1, 2, 3, 4)));
        assert!(checker.is_ip_whitelisted(Ipv4Addr::new(10, 0, 0, 1)));
        assert!(!checker.is_ip_whitelisted(Ipv4Addr::new(9, 9, 9, 9)));
    }

    #[test]
    fn cidr_present() {
        let checker = make_checker(&[], &[], &["192.168.0.0/16", "10.0.0.0/8"]);
        assert!(checker.is_cidr_whitelisted(Ipv4Addr::new(192, 168, 1, 1)));
        assert!(checker.is_cidr_whitelisted(Ipv4Addr::new(10, 0, 0, 1)));
        assert!(!checker.is_cidr_whitelisted(Ipv4Addr::new(8, 8, 8, 8)));
    }

    #[test]
    fn ip_not_in_cidr() {
        let checker = make_checker(&[], &[], &["10.0.0.0/8"]);
        assert!(!checker.is_cidr_whitelisted(Ipv4Addr::new(11, 0, 0, 1)));
        assert!(!checker.is_cidr_whitelisted(Ipv4Addr::UNSPECIFIED));
    }

    #[test]
    fn empty_whitelists() {
        let checker = make_checker(&[], &[], &[]);
        assert!(!checker.is_sni_whitelisted("anything"));
        assert!(!checker.is_ip_whitelisted(Ipv4Addr::new(1, 2, 3, 4)));
        assert!(!checker.is_cidr_whitelisted(Ipv4Addr::new(1, 2, 3, 4)));
    }

    #[test]
    fn malformed_lines_skipped() {
        let content = b"valid.example\n  \n# comment\n\nother.valid\n";
        let sni = write_temp(content);
        let checker = HostFeaturesChecker::new(&sni, &ip_file(&[]), &cidr_file(&[])).unwrap();
        // Our loader treats every non-empty line as a hostname (no comment stripping).
        // "# comment" is stored literally as "# comment" (with hash), so it's whitelisted.
        assert!(checker.is_sni_whitelisted("valid.example"));
        assert!(checker.is_sni_whitelisted("other.valid"));
        assert!(checker.is_sni_whitelisted("# comment"));
    }

    #[test]
    fn malformed_ip_lines_skipped() {
        let content = b"1.2.3.4\n  \nnot_an_ip\n5.6.7.8\n";
        let ip = write_temp(content);
        let checker = HostFeaturesChecker::new(&sni_file(&[]), &ip, &cidr_file(&[])).unwrap();

        assert!(checker.is_ip_whitelisted(Ipv4Addr::new(1, 2, 3, 4)));
        assert!(checker.is_ip_whitelisted(Ipv4Addr::new(5, 6, 7, 8)));
        assert!(!checker.is_ip_whitelisted(Ipv4Addr::UNSPECIFIED));
    }

    #[test]
    fn malformed_cidr_lines_skipped() {
        let content = b"10.0.0.0/8\nbad\n192.168.0.0/16\n1.2.3.4/33\n";
        let cidr = write_temp(content);
        let checker = HostFeaturesChecker::new(&sni_file(&[]), &ip_file(&[]), &cidr).unwrap();

        assert!(checker.is_cidr_whitelisted(Ipv4Addr::new(10, 10, 10, 10)));
        assert!(checker.is_cidr_whitelisted(Ipv4Addr::new(192, 168, 1, 1)));
        // /33 is invalid, skipped; 0.0.0.0 is not matched
        assert!(!checker.is_cidr_whitelisted(Ipv4Addr::new(1, 2, 3, 4)));
    }

    #[test]
    fn host_features_dns_whitelisted() {
        let checker = make_checker(&["example.com"], &[], &[]);
        let feats = checker.get_host_features(&ServerName::try_from("example.com").unwrap());
        assert!(feats.sni_whitelisted);
        assert!(!feats.ip_whitelisted && !feats.cidr_whitelisted);
    }

    #[test]
    fn host_features_dns_case_insensitive() {
        let checker = make_checker(&["example.com"], &[], &[]);
        let feats = checker.get_host_features(&ServerName::try_from("EXAMPLE.COM").unwrap());
        assert!(feats.sni_whitelisted);
    }

    #[test]
    fn host_features_ipv4_sets_ip_and_cidr() {
        let checker = make_checker(&[], &["1.2.3.4"], &["10.0.0.0/8"]);
        let ip_feats = checker.get_host_features(&ServerName::try_from("1.2.3.4").unwrap());
        assert!(ip_feats.ip_whitelisted);
        assert!(!ip_feats.cidr_whitelisted);
        let cidr_feats = checker.get_host_features(&ServerName::try_from("10.20.30.40").unwrap());
        assert!(cidr_feats.cidr_whitelisted);
        assert!(!cidr_feats.ip_whitelisted);
        assert!(!ip_feats.sni_whitelisted && !cidr_feats.sni_whitelisted);
    }

    #[test]
    fn host_features_ipv6_empty() {
        let checker = make_checker(&["example.com"], &["1.2.3.4"], &["10.0.0.0/8"]);
        let feats = checker.get_host_features(&ServerName::try_from("2001:db8::1").unwrap());
        assert_eq!(feats, HostFeatures::default());
    }

    #[test]
    fn host_features_unknown_host_false() {
        let checker = make_checker(&["example.com"], &["1.2.3.4"], &["10.0.0.0/8"]);
        let feats = checker.get_host_features(&ServerName::try_from("unknown.example").unwrap());
        assert_eq!(feats, HostFeatures::default());
    }

    #[tokio::test]
    #[ignore = "downloads from hxehex/russia-mobile-internet-whitelist upstream (network)"]
    async fn real_data_smoke() {
        // End-to-end: `load` downloads all three files into a temp dir, builds the
        // checker, and known entries must be detected. Assertion hosts are the first
        // lines of each upstream file (verified against the checked-in snapshot at
        // thirdparty/russia-mobile-internet-whitelist/: "00.img.avito.st",
        // "2.78.58.1", "2.63.0.0/17").
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        let checker = HostFeaturesChecker::load(
            &base.join("whitelist.txt"),
            &base.join("ipwhitelist.txt"),
            &base.join("cidrwhitelist.txt"),
        )
        .await
        .unwrap();
        assert!(
            checker
                .get_host_features(&ServerName::try_from("00.img.avito.st").unwrap())
                .sni_whitelisted
        );
        assert!(
            checker
                .get_host_features(&ServerName::try_from("2.78.58.1").unwrap())
                .ip_whitelisted
        );
        assert!(
            checker
                .get_host_features(&ServerName::try_from("2.63.0.1").unwrap())
                .cidr_whitelisted
        );
        assert_eq!(
            checker.get_host_features(&ServerName::try_from("8.8.8.8").unwrap()),
            HostFeatures::default()
        );
    }
}
