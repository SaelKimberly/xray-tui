//! Identity → concrete hello resolution over a two-tier table.
//!
//! The hand-written wire-exact rows are authoritative: wherever they and
//! a generated band claim the same identity/band overlap, the hand row
//! wins (design's wire-exact precedence). Generated rows are derived from
//! the roster as contiguous-major bands and answer only what the hand
//! tier strictly refuses.
//!
//! Strict per the design spec: unknown combinations error with the list
//! of what IS resolvable. Never silently substitute a different browser,
//! and never serve a hello older than the requested version.

use crate::fingerprints::error::FingerprintError;
use crate::fingerprints::query::{Browser, Device, Fingerprint, Os};
use crate::profiles::generated::{GENERATED, GenEntry};
use crate::spec::ClientHelloSpec;
use std::sync::LazyLock;

/// One resolvable identity row.
#[derive(Debug, Clone, Copy)]
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
    /// Generated band rows set it to their run's lowest major.
    pub min_version: u16,
    pub spec: fn() -> ClientHelloSpec,
}

use crate::profiles::{
    android11_okhttp, brave167, chrome, chrome_android130, chrome119, chrome133, edge, edge106,
    firefox120, firefox128esr, ios14, opera114, safari, safari_ios17, safari16,
};

/// The hand-written wire-exact tier. Ordered ascending by `max_version`
/// within each `(browser, os, device)`.
static HAND_ROWS: &[Row] = &[
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
/// The merged resolution table: [`HAND_ROWS`] then the derived generated
/// bands ([`generated_rows`]). Built once on first use.
static TABLE: LazyLock<Vec<Row>> = LazyLock::new(|| {
    let mut rows = Vec::with_capacity(HAND_ROWS.len() + GENERATED.len());
    rows.extend(HAND_ROWS.iter().copied());
    rows.extend(generated_rows());
    rows
});

/// The merged resolution table (see [`TABLE`]).
pub(crate) fn table() -> &'static [Row] {
    &TABLE
}

/// The generated-tier slice of [`table`]: derived bands only.
fn generated_rows_slice() -> &'static [Row] {
    &table()[HAND_ROWS.len()..]
}

/// A generated-roster identity group key.
type GroupKey = (Browser, Option<Os>, Device);

/// Derives the generated-tier rows from the roster: one [`Row`] per
/// contiguous-major run within each `(browser, os, device)` group. The
/// row's band is the run's major span and its spec is the run's
/// greatest-major entry (last declared on ties). Group and run order is
/// deterministic so ties resolve to the first-declared row.
fn generated_rows() -> Vec<Row> {
    let mut groups: Vec<(GroupKey, Vec<GenEntry>)> = Vec::new();
    for entry in GENERATED {
        let key = (entry.browser, entry.os, entry.device);
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, entries)) => entries.push(*entry),
            None => groups.push((key, vec![*entry])),
        }
    }
    groups.sort_by_key(|(k, _)| (k.0.name(), k.1.map(Os::name), k.2.name()));
    let mut rows = Vec::new();
    for (_, mut entries) in groups {
        entries.sort_by_key(|e| e.major); // stable: ties keep GENERATED order
        let mut run: Vec<GenEntry> = Vec::new();
        for entry in entries {
            let extending = run.last().is_some_and(|last| entry.major == last.major + 1);
            if !extending && !run.is_empty() {
                rows.push(band_row(&run));
                run.clear();
            }
            run.push(entry);
        }
        if !run.is_empty() {
            rows.push(band_row(&run));
        }
    }
    rows
}

/// One table row from a non-empty contiguous-major run: band = the run's
/// major span, spec = the greatest-major entry.
fn band_row(run: &[GenEntry]) -> Row {
    let rep = run[run.len() - 1];
    Row {
        name: rep.name,
        browser: rep.browser,
        os: rep.os,
        device: rep.device,
        max_version: rep.major,
        min_version: run[0].major,
        spec: rep.spec_fn,
    }
}

/// Greatest `max_version` row (ties → first declared).
fn greatest_row<'a>(rows: impl Iterator<Item = &'a Row>) -> Option<&'a Row> {
    rows.reduce(|best, r| {
        if r.max_version > best.max_version {
            r
        } else {
            best
        }
    })
}

/// Hand-tier chooser: strict era-witnessed covering — a row answers `v`
/// exactly at its max, or inside a band whose lower edge some earlier
/// row witnesses; below the oldest row of a platform group we refuse.
fn choose_hand<'a>(candidates: &'a [&'a Row], version: Option<u16>) -> Option<&'a Row> {
    version.map_or_else(
        || candidates.iter().max_by_key(|r| r.max_version).copied(),
        |v| {
            let covering = candidates
                .iter()
                .filter(|r| r.max_version >= v)
                .min_by_key(|r| r.max_version)
                .copied();
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
    )
}

/// Generated-tier chooser: a band row answers exactly its contiguous
/// major run (`min_version <= v <= max_version`); run gaps strict-refuse.
fn choose_generated<'a>(candidates: &'a [&'a Row], version: Option<u16>) -> Option<&'a Row> {
    version.map_or_else(
        || candidates.iter().max_by_key(|r| r.max_version).copied(),
        |v| {
            candidates
                .iter()
                .filter(|r| r.min_version <= v && v <= r.max_version)
                .min_by_key(|r| r.max_version)
                .copied()
        },
    )
}

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
/// Hand-tier rows win: the wire-exact tier holds the authoritative
/// latest-known identities. Generated bands answer only browsers with no
/// hand rows (e.g. Samsung Internet). Ties resolve to the first
/// (earliest-declared) row.
pub(crate) fn latest_row(browser: Browser) -> Option<&'static Row> {
    greatest_row(HAND_ROWS.iter().filter(|r| r.browser == browser)).or_else(|| {
        greatest_row(
            generated_rows_slice()
                .iter()
                .filter(|r| r.browser == browser),
        )
    })
}

/// Every resolvable identity name, sorted (error payloads, UI pickers).
#[must_use]
pub fn available_identities() -> Vec<String> {
    let mut names: Vec<String> = table().iter().map(|r| r.name.to_string()).collect();
    names.sort();
    names
}

impl Fingerprint {
    /// Rows in `rows` that could answer this query (identity, band floor
    /// and os/device compat), in declaration order.
    fn matching<'a>(&self, rows: &'a [Row]) -> Vec<&'a Row> {
        rows.iter()
            .filter(|r| r.browser == self.browser && r.matches(self.version, self.os, self.device))
            .collect()
    }

    /// Strict resolution to a concrete hello. Fallback chain: hand tier
    /// (exact → same browser/device(+compatible os) row covering the
    /// requested version) → generated tier (contiguous-major band
    /// containing the version) → error listing alternatives.
    ///
    /// A hand row covers `v` when `v` is its exact major, or `v` lies
    /// inside its band `(nearest lower candidate's max_version,
    /// row.max_version]`, and never below the row's `min_version` floor.
    /// Below the oldest hand row of a platform group we fall through to
    /// the generated tier, which answers exactly the majors its roster
    /// runs contain. If neither tier answers, we refuse — never serve a
    /// hello from a wildly different era than asked for.
    ///
    /// # Errors
    /// [`FingerprintError::Unknown`] when no row satisfies the query.
    pub fn resolve(&self) -> Result<Resolved, FingerprintError> {
        // Hand tier first: wire-exact transcriptions win wherever they
        // and a generated band claim the same identity/band overlap.
        let hand_candidates = self.matching(HAND_ROWS);
        let hand = choose_hand(&hand_candidates, self.version);
        let generated_candidates = self.matching(generated_rows_slice());
        let chosen = hand.or_else(|| choose_generated(&generated_candidates, self.version));
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
    fn hand_row_wins_over_generated_band_overlap() {
        // Chrome 125 Windows desktop: the hand chrome_130 row's band
        // (119, 130] claims it AND the generated roster carries
        // chrome_125_windows_desktop. Wire-exact precedence: the query
        // must resolve to the hand-written row, never the generated one.
        let r = Fingerprint::new(Browser::Chrome)
            .with_version(125)
            .with_os(Os::Windows)
            .with_device(Device::Desktop)
            .resolve()
            .unwrap();
        assert_eq!(r.name, "chrome_130");
    }

    #[test]
    fn generated_only_identity_resolves_to_registered_ja4() {
        // edge_79_windows_desktop is a generated-roster identity the hand
        // tier strictly refuses (below edge_106 with no lower band
        // witness). It must resolve, and the resolved hello must hash to
        // the roster's registered source JA4 for that identity.
        let registered = GENERATED
            .iter()
            .find(|e| e.name == "edge_79_windows_desktop")
            .expect("roster pins edge_79_windows_desktop");
        let r = Fingerprint::new(Browser::Edge)
            .with_version(79)
            .with_os(Os::Windows)
            .with_device(Device::Desktop)
            .resolve()
            .unwrap();
        assert_eq!(r.name, registered.name);
        assert_eq!(r.fingerprint.version, Some(79));
        assert_eq!(r.ja4(), registered.ja4, "JA4 fidelity for {}", r.name);
    }

    #[test]
    fn error_listing_caps_at_eight_with_total_suffix() {
        let available: Vec<String> = (0..20).map(|i| format!("alt_{i}")).collect();
        let err = FingerprintError::Unknown {
            query: "chrome/999/-/-".to_string(),
            available,
        };
        let msg = err.to_string();
        for i in 0..8 {
            assert!(msg.contains(&format!("alt_{i}")), "missing alt_{i}: {msg}");
        }
        assert!(
            !msg.contains("alt_8"),
            "9th alternative must be hidden: {msg}"
        );
        assert!(msg.ends_with("… and 12 more"), "{msg}");
    }

    #[test]
    fn unresolvable_query_message_is_capped() {
        // Chrome 2 sits below the oldest chrome run in both tiers; the
        // ~990-name alternatives list must render capped, not verbatim.
        let err = Fingerprint::new(Browser::Chrome)
            .with_version(2)
            .resolve()
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("… and "), "truncation suffix missing: {msg}");
        assert!(msg.ends_with(" more"), "{msg}");
    }

    #[test]
    fn below_oldest_hand_row_falls_back_to_generated_tier() {
        // 119 is the oldest hand chrome row and answers only its own major.
        let ok = Fingerprint::new(Browser::Chrome)
            .with_version(119)
            .resolve()
            .unwrap();
        assert_eq!(ok.name, "chrome_119");
        // 118 is below every hand chrome row, so the hand tier strictly
        // refuses; the generated roster answers instead. Its Windows run
        // spans majors 118–126, served by the run's greatest-major spec
        // (chrome_126_windows_desktop).
        let generated = Fingerprint::new(Browser::Chrome)
            .with_version(118)
            .with_os(Os::Windows)
            .with_device(Device::Desktop)
            .resolve()
            .unwrap();
        assert_eq!(generated.name, "chrome_126_windows_desktop");
        assert_eq!(generated.fingerprint.version, Some(118));
        // 2 sits below every generated chrome run too: strict error.
        let err = Fingerprint::new(Browser::Chrome)
            .with_version(2)
            .resolve()
            .unwrap_err();
        assert!(matches!(err, FingerprintError::Unknown { .. }));
    }

    #[test]
    fn version_below_every_row_errors_with_available_list() {
        // Chrome 3 now resolves via the generated roster (its Windows
        // run starts at 3); 2 is below every run in both tiers.
        let fp = Fingerprint::new(Browser::Chrome).with_version(2);
        let err = fp.resolve().unwrap_err();
        let FingerprintError::Unknown { query, available } = err else {
            panic!("wrong variant")
        };
        assert_eq!(query, "chrome/2/-/-");
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
    fn tablet_queries_resolve_via_generated_rows() {
        // No hand row is a tablet; the generated roster carries Chrome
        // Android/iOS tablet identities and answers bare tablet queries.
        let r = Fingerprint::new(Browser::Chrome)
            .with_device(Device::Tablet)
            .resolve()
            .unwrap();
        assert_eq!(r.fingerprint.device, Some(Device::Tablet));
        assert!(r.name.starts_with("chrome_"), "{}", r.name);
    }

    #[test]
    fn every_table_row_targets_existing_profile_fn() {
        // Compile-time completeness: building each row's spec proves the fn exists.
        for entry in table() {
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
        assert_eq!(names.len(), table().len());
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
        // Samsung Internet has no hand rows; the generated roster answers.
        assert_eq!(
            latest_row(Browser::SamsungInternet).unwrap().name,
            "samsung_29_android_desktop"
        );
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
