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
    /// Inclusive minimum requested major this row answers (`0` =
    /// unbounded below). Rows covering disjoint bands of the same
    /// platform group set this so overlap is impossible by construction
    /// (e.g. `android_11_okhttp` ≤ 11 vs `chrome_android_130` ≥ 12).
    pub min_version: u16,
    pub spec: fn() -> ClientHelloSpec,
}

use crate::profiles::{
    android11_okhttp, brave167, chrome, chrome_android130, chrome119, chrome133, edge, edge106,
    firefox120, firefox128esr, ios14, opera114, safari, safari_ios17, safari16,
};

/// Ordered ascending by `max_version` within each `(browser, os, device)`.
pub(crate) static TABLE: &[Row] = &[
    Row {
        name: "chrome_119",
        browser: Browser::Chrome,
        os: None,
        device: Device::Desktop,
        max_version: 119,
        min_version: 0,
        spec: chrome119::spec,
    },
    Row {
        name: "chrome_130",
        browser: Browser::Chrome,
        os: None,
        device: Device::Desktop,
        max_version: 130,
        min_version: 0,
        spec: chrome::spec,
    },
    Row {
        name: "chrome_133",
        browser: Browser::Chrome,
        os: None,
        device: Device::Desktop,
        max_version: 133,
        min_version: 0,
        spec: chrome133::spec,
    },
    Row {
        name: "chrome_android_130",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Phone,
        max_version: 130,
        min_version: 12,
        spec: chrome_android130::spec,
    },
    Row {
        name: "edge_106",
        browser: Browser::Edge,
        os: None,
        device: Device::Desktop,
        max_version: 106,
        min_version: 0,
        spec: edge106::spec,
    },
    Row {
        name: "edge_130",
        browser: Browser::Edge,
        os: None,
        device: Device::Desktop,
        max_version: 130,
        min_version: 0,
        spec: edge::spec,
    },
    Row {
        name: "brave_167",
        browser: Browser::Brave,
        os: None,
        device: Device::Desktop,
        max_version: 167,
        min_version: 0,
        spec: brave167::spec,
    },
    Row {
        name: "opera_114",
        browser: Browser::Opera,
        os: None,
        device: Device::Desktop,
        max_version: 114,
        min_version: 0,
        spec: opera114::spec,
    },
    Row {
        name: "firefox_120",
        browser: Browser::Firefox,
        os: None,
        device: Device::Desktop,
        max_version: 120,
        min_version: 0,
        spec: firefox120::spec,
    },
    Row {
        name: "firefox_128_esr",
        browser: Browser::Firefox,
        os: None,
        device: Device::Desktop,
        max_version: 128,
        min_version: 0,
        spec: firefox128esr::spec,
    },
    Row {
        name: "safari_16",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        max_version: 16,
        min_version: 0,
        spec: safari16::spec,
    },
    Row {
        name: "safari_17",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        max_version: 17,
        min_version: 0,
        spec: safari::spec,
    },
    Row {
        name: "safari_ios_17",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        max_version: 17,
        min_version: 0,
        spec: safari_ios17::spec,
    },
    Row {
        name: "ios_14",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        max_version: 14,
        min_version: 0,
        spec: ios14::spec,
    },
    Row {
        name: "android_11_okhttp",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Phone,
        max_version: 11,
        // Disjoint from chrome_android_130 (min_version 12): the
        // Chromium/Android/Phone bands can never overlap.
        min_version: 0,
        spec: android11_okhttp::spec,
    },
];

impl Row {
    fn matches(&self, q_version: Option<u16>, q_os: Option<Os>, q_device: Option<Device>) -> bool {
        if q_version.is_some_and(|v| v < self.min_version) {
            return false; // below this row's band floor
        }
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
    /// its band `(nearest lower candidate's max_version, row.max_version]`,
    /// and never below the row's `min_version` floor. Below the oldest row
    /// of a platform group we refuse — never serve a hello from a wildly
    /// different era than asked for.
    ///
    /// # Errors
    /// [`FingerprintError::Unknown`] when no row satisfies the query.
    pub fn resolve(&self) -> Result<Resolved, FingerprintError> {
        let candidates: Vec<&Row> = TABLE
            .iter()
            .filter(|r| r.browser == self.browser && r.matches(self.version, self.os, self.device))
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

impl Resolved {
    /// The full JA4 of this resolved hello (computed, not looked up).
    #[must_use]
    pub fn ja4(&self) -> String {
        crate::crypto::fingerprint::ja4::full_ja4(
            &crate::crypto::fingerprint::ja3::Ja3Fields::from_spec(&self.spec),
        )
    }

    /// True when this resolved hello's JA4 was observed in the wild for
    /// its claimed browser identity (catalog evidence). Version is passed
    /// as `None`: recall over precision — evidence-only check, so catalog
    /// rows without a recorded browser major still count.
    ///
    /// Honest caveat: engine hellos are synthetic variants (uTLS-style
    /// transcriptions), so today NO resolvable identity's full JA4 appears
    /// in the catalog — this returns `false` for every current row. The
    /// check is forward-looking: it becomes live evidence the moment a
    /// captured hello's JA4 lands in [`super::catalog::CATALOG`].
    #[must_use]
    pub fn in_catalog(&self) -> bool {
        super::catalog::contains(self.fingerprint.browser.name(), None, &self.ja4())
    }
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
    fn android_11_hits_okhttp_row_chrome_12_hits_modern_row() {
        let old = Fingerprint::new(Browser::Chrome)
            .with_version(11)
            .with_os(Os::Android)
            .with_device(Device::Phone)
            .resolve()
            .unwrap();
        assert_eq!(old.name, "android_11_okhttp");
        let modern = Fingerprint::new(Browser::Chrome)
            .with_version(12)
            .with_os(Os::Android)
            .with_device(Device::Phone)
            .resolve()
            .unwrap();
        assert_eq!(modern.name, "chrome_android_130");
    }

    #[test]
    fn ios_14_query_hits_ios14_row() {
        // Safari/iOS 14 → the legacy iOS row; 15-17 stay on safari_ios_17.
        let old = Fingerprint::new(Browser::Safari)
            .with_version(14)
            .with_os(Os::Ios)
            .with_device(Device::Phone)
            .resolve()
            .unwrap();
        assert_eq!(old.name, "ios_14");
        let modern = Fingerprint::new(Browser::Safari)
            .with_version(15)
            .with_os(Os::Ios)
            .with_device(Device::Phone)
            .resolve()
            .unwrap();
        assert_eq!(modern.name, "safari_ios_17");
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
    fn firefox_120_query_hits_firefox_120_row() {
        let r = Fingerprint::new(Browser::Firefox)
            .with_version(120)
            .resolve()
            .unwrap();
        assert_eq!(r.name, "firefox_120");
    }

    #[test]
    fn safari_16_query_hits_safari_16_row() {
        let r = Fingerprint::new(Browser::Safari)
            .with_version(16)
            .resolve()
            .unwrap();
        assert_eq!(r.name, "safari_16");
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
    #[test]
    fn resolved_computes_full_ja4() {
        let r = Fingerprint::new(Browser::Chrome)
            .with_version(133)
            .resolve()
            .unwrap();
        let ja4 = r.ja4();
        assert!(ja4.starts_with("t13d"), "{ja4}");
        let parts: Vec<&str> = ja4.split('_').collect();
        assert_eq!(parts.len(), 3, "A_hash1_hash2");
        assert_eq!(parts[1].len(), 12);
        assert_eq!(parts[2].len(), 12);
    }

    #[test]
    fn resolved_ja4_not_in_catalog_today() {
        // Synthetic engine hellos: no resolvable identity's full JA4 is in
        // the catalog yet (documented on `Resolved::in_catalog`).
        for fp in [
            Fingerprint::new(Browser::Chrome).with_version(133),
            Fingerprint::new(Browser::Firefox).with_version(128),
            Fingerprint::new(Browser::Safari)
                .with_version(17)
                .with_os(Os::MacOs),
        ] {
            let r = fp.resolve().unwrap();
            assert!(!r.in_catalog(), "{} must not be catalog evidence", r.name);
        }
    }

    #[test]
    fn in_catalog_wiring_matches_contains() {
        // Wiring proof (not hello fidelity): a real CATALOG entry fed
        // through the same contains() call in_catalog() makes.
        use crate::fingerprints::catalog;
        let entry = catalog::CATALOG
            .iter()
            .find(|e| e.application == "chrome")
            .expect("catalog has chrome rows");
        assert!(catalog::contains("chrome", None, entry.ja4));
        // ...and a fabricated JA4 is correctly rejected.
        assert!(!catalog::contains(
            "chrome",
            None,
            "t13d9999zzzz_zzzzzzzzzzzz_zzzzzzzzzzzz"
        ));
    }
}
