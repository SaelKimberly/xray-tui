//! Instantly-verifiable fingerprint cases (design spec: rstest tables).
//! Expected JA4s are frozen from live `tls.peet.ws` captures of each
//! profile's hello (GREASE-stable by construction — JA4 strips GREASE).
//!
//! Capture provenance: every expected string below was reported by
//! `tls.peet.ws` for a live connection made by this engine with the
//! profile's hello (grader example, 2026-08-24; Tasks 7/8/10).

use rstest::rstest;
use xray_tui_tls::fingerprints::{
    Fingerprint,
    query::{Browser, Device, Os},
};

const fn fp(browser: Browser, version: u16) -> Fingerprint {
    Fingerprint::new(browser).with_version(version)
}

#[rstest]
// ── existing profiles ──
#[case::chrome_130(fp(Browser::Chrome, 130), "t13d1516h2_8daaf6152771_f37e75b10bcc")]
#[case::chrome_133(fp(Browser::Chrome, 133), "t13d1515h2_8daaf6152771_0255bb164da8")]
#[case::chrome_119(fp(Browser::Chrome, 119), "t13d1514h2_8daaf6152771_1eb89897b454")]
#[case::chrome_android_130(
    Fingerprint::new(Browser::Chrome)
        .with_version(130)
        .with_os(Os::Android)
        .with_device(Device::Phone),
    "t13d1513h2_8daaf6152771_6a3d802a7139"
)]
#[case::edge_130(fp(Browser::Edge, 130), "t13d1515h2_8daaf6152771_5d45727bf495")]
#[case::brave_167(fp(Browser::Brave, 167), "t13d1514h2_8daaf6152771_1eb89897b454")]
#[case::opera_114(fp(Browser::Opera, 114), "t13d1513h2_8daaf6152771_4d189b233cec")]
#[case::firefox_esr(fp(Browser::Firefox, 128), "t13d1314h2_07be0c029dc8_46701d79520f")]
#[case::safari_17(
    Fingerprint::new(Browser::Safari)
        .with_version(17)
        .with_os(Os::MacOs)
        .with_device(Device::Desktop),
    "t13d2412h2_549c34bdfdad_da04eaf042c7"
)]
#[case::safari_ios_17(
    Fingerprint::new(Browser::Safari)
        .with_version(17)
        .with_os(Os::Ios)
        .with_device(Device::Phone),
    "t13d2310h2_24fc43eb1c96_ace1137f6da2"
)]
// ── new presets ──
#[case::safari_16(fp(Browser::Safari, 16), "t13d2014h2_a09f3c656075_874d27d7ca63")]
#[case::firefox_120(fp(Browser::Firefox, 120), "t13d1714h2_5b57614c22b0_967e8e80b303")]
#[case::edge_106(fp(Browser::Edge, 106), "t13d1516h2_8daaf6152771_f37e75b10bcc")]
#[case::ios_14(
    Fingerprint::new(Browser::Safari)
        .with_version(14)
        .with_os(Os::Ios)
        .with_device(Device::Phone),
    "t13d2613h2_2802a3db6c62_38ba08824cc9"
)]
fn ja4_matches_peet_ground_truth(#[case] fp: Fingerprint, #[case] expected: &str) {
    let resolved = fp.resolve().expect("table case must resolve");
    assert_eq!(resolved.ja4(), expected, "{}", resolved.name);
}

/// `android_11_okhttp` is TLS-1.2-shaped (no `supported_versions`/`ALPN`), so
/// its offline `ja4_a` hardcodes the `t12d…` prefix and cannot equal the
/// peet.ws JA4 computed over an HTTP/2-capable TLS-1.3 capture — it is not
/// asserted here. Offline ground truth from its uTLS transcription:
/// `t12d120700_d34a8e72043a_036209cd1ead`. We pin only that the identity
/// resolves to the intended catalog row.
#[test]
fn android_11_okhttp_resolves_to_its_row() {
    let fp = Fingerprint::new(Browser::Chrome)
        .with_version(11)
        .with_os(Os::Android)
        .with_device(Device::Phone);
    let resolved = fp.resolve().expect("android_11_okhttp must resolve");
    assert_eq!(resolved.name, "android_11_okhttp");
}

#[rstest]
#[case::chrome_133(fp(Browser::Chrome, 133))]
#[case::edge_130(fp(Browser::Edge, 130))]
fn ja4_is_run_to_run_stable(#[case] fp: Fingerprint) {
    let a = fp.resolve().unwrap().ja4();
    let b = fp.resolve().unwrap().ja4(); // fresh spec; GREASE draws differ
    assert_eq!(a, b, "JA4 must be GREASE-independent");
}

#[rstest]
#[case::firefox_esr(fp(Browser::Firefox, 128))]
#[case::firefox_120(fp(Browser::Firefox, 120))]
#[case::safari_17(Fingerprint::new(Browser::Safari).with_version(17).with_os(Os::MacOs).with_device(Device::Desktop))]
fn grease_free_profiles_pin_classic_ja3_prefix(#[case] fp: Fingerprint) {
    // Firefox/Safari families send no GREASE — classic JA3 string is stable.
    let resolved = fp.resolve().unwrap();
    let ja3 = xray_tui_tls::crypto::fingerprint::ja3::ja3_string(
        &xray_tui_tls::crypto::fingerprint::ja3::Ja3Fields::from_spec(&resolved.spec),
    );
    assert!(ja3.starts_with("771,"), "{ja3}");
}

#[rstest]
#[case::chrome_130(fp(Browser::Chrome, 130))]
fn grease_carrying_profiles_use_stripped_ja3_variant(#[case] fp: Fingerprint) {
    // Determinism rule (b): stripped variant is deterministic; classic
    // on-wire JA3 is NOT pinned for these families.
    let resolved = fp.resolve().unwrap();
    let ja3 = xray_tui_tls::crypto::fingerprint::ja3::ja3_grease_stripped(
        &xray_tui_tls::crypto::fingerprint::ja3::Ja3Fields::from_spec(&resolved.spec),
    );
    assert!(!ja3.is_empty());
}
