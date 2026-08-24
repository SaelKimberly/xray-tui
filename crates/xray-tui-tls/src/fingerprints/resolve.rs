//! Identity → concrete hello resolution over a hand-maintained table.
//!
//! Strict per the design spec: unknown combinations error with the list
//! of what IS resolvable. Never silently substitute a different browser,
//! and never serve a hello older than the requested version.

use crate::fingerprints::error::FingerprintError;
use crate::fingerprints::query::{Browser, Device, Fingerprint, Os};
use crate::spec::ClientHelloSpec;

/// One resolvable identity row.
pub(crate) struct Row {
    /// Stable spec name (`chrome_130`) — appears in errors and UI.
    pub name: &'static str,
    pub browser: Browser,
    /// `None` = row serves any OS (desktop hellos are largely OS-independent
    /// within a browser family).
    pub os: Option<Os>,
    pub device: Device,
    /// Inclusive maximum requested major this row answers.
    pub max_version: u16,
    pub spec: fn() -> ClientHelloSpec,
}

use crate::profiles::{
    brave167, chrome, chrome_android130, chrome119, chrome133, edge, firefox128esr, opera114,
    safari, safari_ios17,
};

/// Ordered ascending by `max_version` within each `(browser, os, device)`.
pub(crate) static TABLE: &[Row] = &[
    Row {
        name: "chrome_119",
        browser: Browser::Chrome,
        os: None,
        device: Device::Desktop,
        max_version: 119,
        spec: chrome119::spec,
    },
    Row {
        name: "chrome_130",
        browser: Browser::Chrome,
        os: None,
        device: Device::Desktop,
        max_version: 130,
        spec: chrome::spec,
    },
    Row {
        name: "chrome_133",
        browser: Browser::Chrome,
        os: None,
        device: Device::Desktop,
        max_version: 133,
        spec: chrome133::spec,
    },
    Row {
        name: "chrome_android_130",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Phone,
        max_version: 130,
        spec: chrome_android130::spec,
    },
    Row {
        name: "edge_130",
        browser: Browser::Edge,
        os: None,
        device: Device::Desktop,
        max_version: 130,
        spec: edge::spec,
    },
    Row {
        name: "brave_167",
        browser: Browser::Brave,
        os: None,
        device: Device::Desktop,
        max_version: 167,
        spec: brave167::spec,
    },
    Row {
        name: "opera_114",
        browser: Browser::Opera,
        os: None,
        device: Device::Desktop,
        max_version: 114,
        spec: opera114::spec,
    },
    Row {
        name: "firefox_128_esr",
        browser: Browser::Firefox,
        os: None,
        device: Device::Desktop,
        max_version: 128,
        spec: firefox128esr::spec,
    },
    Row {
        name: "safari_17",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        max_version: 17,
        spec: safari::spec,
    },
    Row {
        name: "safari_ios_17",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        max_version: 17,
        spec: safari_ios17::spec,
    },
    // Tasks 7/8 append: safari_16 (15), firefox_120 (120), edge_106 (106),
    // ios_14, android_11_okhttp — keeping per-browser ascending order.
];

impl Row {
    fn matches(&self, q_os: Option<Os>, q_device: Option<Device>) -> bool {
        if let Some(d) = q_device {
            if d != self.device {
                return false;
            }
        } else if self.device != Device::Desktop {
            return false; // unspecified device defaults to desktop rows
        }
        // Os-unpinned rows serve any os; an unspecified query os accepts
        // pinned-os rows.
        match (q_os, self.os) {
            (Some(a), Some(b)) => a == b,
            _ => true,
        }
    }
}

/// The table row answering `browser` at its greatest `max_version`.
/// Ties resolve to the first (earliest-declared) row.
pub(crate) fn latest_row(browser: Browser) -> Option<&'static Row> {
    TABLE
        .iter()
        .filter(|r| r.browser == browser)
        .reduce(|best, r| {
            if r.max_version > best.max_version {
                r
            } else {
                best
            }
        })
}

/// Every resolvable identity name, sorted (error payloads, UI pickers).
#[must_use]
pub fn available_identities() -> Vec<String> {
    let mut names: Vec<String> = TABLE.iter().map(|r| r.name.to_string()).collect();
    names.sort();
    names
}

impl Fingerprint {
    /// Strict resolution to a concrete hello. Fallback chain: exact →
    /// same browser/device(+compatible os) row covering the requested
    /// version → error listing alternatives.
    ///
    /// A row covers `v` when `v` is its exact major, or `v` lies inside
    /// its band `(nearest lower candidate's max_version, row.max_version]`.
    /// Below the oldest row of a platform group we refuse — never serve a
    /// hello from a wildly different era than asked for.
    ///
    /// # Errors
    /// [`FingerprintError::Unknown`] when no row satisfies the query.
    pub fn resolve(&self) -> Result<Resolved, FingerprintError> {
        let candidates: Vec<&Row> = TABLE
            .iter()
            .filter(|r| r.browser == self.browser && r.matches(self.os, self.device))
            .collect();
        let chosen = self.version.map_or_else(
            || candidates.iter().max_by_key(|r| r.max_version),
            |v| {
                let covering = candidates
                    .iter()
                    .filter(|r| r.max_version >= v)
                    .min_by_key(|r| r.max_version);
                let covered = covering.is_some_and(|row| {
                    row.max_version == v
                        || candidates
                            .iter()
                            .filter(|r| r.max_version < v)
                            .max_by_key(|r| r.max_version)
                            .is_some()
                });
                if covered { covering } else { None }
            },
        );
        let Some(row) = chosen else {
            return Err(FingerprintError::Unknown {
                query: self.render(),
                available: available_identities(),
            });
        };
        Ok(Resolved {
            name: row.name,
            fingerprint: Self {
                browser: row.browser,
                version: Some(self.version.unwrap_or(row.max_version)),
                os: self.os.or(row.os),
                device: Some(self.device.unwrap_or(row.device)),
            },
            spec: (row.spec)(),
        })
    }
}

/// A successfully resolved fingerprint: identity + concrete hello.
#[derive(Debug, Clone)]
pub struct Resolved {
    /// Resolution-table name (`chrome_133`).
    pub name: &'static str,
    /// Fully concretized identity (unset query fields filled from the row).
    pub fingerprint: Fingerprint,
    /// The hello specification.
    pub spec: ClientHelloSpec,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_exact_query() {
        let fp = Fingerprint::new(Browser::Chrome).with_version(133);
        let r = fp.resolve().unwrap();
        assert_eq!(r.name, "chrome_133");
        assert!(!r.spec.cipher_suites.is_empty());
    }

    #[test]
    fn falls_forward_to_nearest_covering_version_same_platform() {
        let fp = Fingerprint::new(Browser::Chrome)
            .with_version(132)
            .with_os(Os::Windows)
            .with_device(Device::Desktop);
        let r = fp.resolve().unwrap();
        // No chrome_132 row exists; the nearest row whose max_version
        // covers 132 is chrome_133.
        assert_eq!(r.name, "chrome_133");
        assert_eq!(r.fingerprint.version, Some(132));
    }

    #[test]
    fn unset_fields_default_to_desktop_rows() {
        // Bare Firefox query lands on firefox_128_esr: os-unpinned row,
        // so the resolved os stays unset; device fills from the row.
        let r = Fingerprint::new(Browser::Firefox).resolve().unwrap();
        assert_eq!(r.name, "firefox_128_esr");
        assert_eq!(r.fingerprint.os, None);
        assert_eq!(r.fingerprint.device, Some(Device::Desktop));
    }

    #[test]
    fn pinned_os_fills_from_row_when_query_unspecified() {
        // Safari with no os/device: desktop macos row answers and pins os.
        let r = Fingerprint::new(Browser::Safari)
            .with_version(17)
            .resolve()
            .unwrap();
        assert_eq!(r.name, "safari_17");
        assert_eq!(r.fingerprint.os, Some(Os::MacOs));
        assert_eq!(r.fingerprint.device, Some(Device::Desktop));
    }

    #[test]
    fn version_inside_band_picks_covering_row() {
        // 125 sits in chrome_130's band (119, 130].
        let r = Fingerprint::new(Browser::Chrome)
            .with_version(125)
            .resolve()
            .unwrap();
        assert_eq!(r.name, "chrome_130");
    }

    #[test]
    fn oldest_row_answers_only_its_own_major() {
        let ok = Fingerprint::new(Browser::Chrome)
            .with_version(119)
            .resolve()
            .unwrap();
        assert_eq!(ok.name, "chrome_119");
        // 118 is below the oldest row and not an exact major: strict error.
        let err = Fingerprint::new(Browser::Chrome)
            .with_version(118)
            .resolve()
            .unwrap_err();
        assert!(matches!(err, FingerprintError::Unknown { .. }));
    }

    #[test]
    fn version_below_every_row_errors_with_available_list() {
        let fp = Fingerprint::new(Browser::Chrome).with_version(3);
        let err = fp.resolve().unwrap_err();
        let FingerprintError::Unknown { query, available } = err else {
            panic!("wrong variant")
        };
        assert_eq!(query, "chrome/3/-/-");
        assert!(available.iter().any(|a| a.starts_with("chrome")));
    }

    #[test]
    fn mobile_query_lands_on_mobile_row() {
        let fp = Fingerprint::new(Browser::Chrome)
            .with_os(Os::Android)
            .with_device(Device::Phone);
        let r = fp.resolve().unwrap();
        assert_eq!(r.name, "chrome_android_130");
        assert_eq!(r.fingerprint.version, Some(130));
    }

    #[test]
    fn incompatible_os_device_combo_errors_strictly() {
        // Android phone Safari: no such row — must error, not substitute.
        let err = Fingerprint::new(Browser::Safari)
            .with_os(Os::Android)
            .with_device(Device::Phone)
            .resolve()
            .unwrap_err();
        assert!(matches!(err, FingerprintError::Unknown { .. }));
    }

    #[test]
    fn tablet_queries_have_no_rows_yet() {
        let err = Fingerprint::new(Browser::Chrome)
            .with_device(Device::Tablet)
            .resolve()
            .unwrap_err();
        assert!(matches!(err, FingerprintError::Unknown { .. }));
    }

    #[test]
    fn every_table_row_targets_existing_profile_fn() {
        // Compile-time completeness: building each row's spec proves the fn exists.
        for entry in TABLE {
            let _ = (entry.spec)();
        }
    }

    #[test]
    fn available_identities_sorted_and_complete() {
        let names = available_identities();
        assert_eq!(names.len(), TABLE.len());
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
        assert!(names.contains(&"chrome_android_130".to_string()));
    }

    #[test]
    fn latest_row_finds_greatest_max_version() {
        assert_eq!(latest_row(Browser::Chrome).unwrap().name, "chrome_133");
        assert_eq!(
            latest_row(Browser::Firefox).unwrap().name,
            "firefox_128_esr"
        );
        assert!(latest_row(Browser::SamsungInternet).is_none());
    }
}
