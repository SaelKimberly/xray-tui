//! `WebKit` hellos on iOS (the `safari_ios` wire template; `WKWebView` reality)
//!
//! Emitter output (`gen_specs.py --emit`); do not edit by hand.
//! Regeneration is byte-deterministic (`--selftest` verifies the
//! committed files match a fresh render).

use super::GenEntry;
use crate::fingerprints::{Browser, Device, Os};

#[rustfmt::skip]
pub const GENERATED: &[GenEntry] = &[
    GenEntry {
        name: "safari_17_ios_tablet",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 17,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: safari_17_ios_tablet,
    },
    GenEntry {
        name: "chrome_122_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 122,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_122_ios_phone,
    },
    GenEntry {
        name: "firefox_121_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 121,
        ja4: "t13d131100_f57a46bbacb6_ab7e3b40a677",
        spec_fn: firefox_121_ios_phone,
    },
    GenEntry {
        name: "firefox_124_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 124,
        ja4: "t13d1513h2_8daaf6152771_eca864cca44a",
        spec_fn: firefox_124_ios_phone,
    },
    GenEntry {
        name: "chrome_52_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 52,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_52_ios_tablet,
    },
    GenEntry {
        name: "chrome_50_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 50,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_50_ios_tablet,
    },
    GenEntry {
        name: "chrome_47_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 47,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_47_ios_tablet,
    },
    GenEntry {
        name: "chrome_55_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 55,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_55_ios_phone,
    },
    GenEntry {
        name: "chrome_50_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 50,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_50_ios_phone,
    },
    GenEntry {
        name: "chrome_49_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 49,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_49_ios_phone,
    },
    GenEntry {
        name: "chrome_53_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 53,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_53_ios_phone,
    },
    GenEntry {
        name: "chrome_52_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 52,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_52_ios_phone,
    },
    GenEntry {
        name: "chrome_55_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 55,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_55_ios_phone_2,
    },
    GenEntry {
        name: "chrome_49_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 49,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_49_ios_phone_2,
    },
    GenEntry {
        name: "chrome_53_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 53,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_53_ios_phone_2,
    },
    GenEntry {
        name: "chrome_47_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 47,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_47_ios_phone,
    },
    GenEntry {
        name: "chrome_48_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 48,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_48_ios_phone,
    },
    GenEntry {
        name: "chrome_55_ios_phone_3",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 55,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_55_ios_phone_3,
    },
    GenEntry {
        name: "chrome_53_ios_phone_3",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 53,
        ja4: "t13d1515ht_8daaf6152771_4769d65a485e",
        spec_fn: chrome_53_ios_phone_3,
    },
    GenEntry {
        name: "chrome_47_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 47,
        ja4: "t13d1515ht_8daaf6152771_4769d65a485e",
        spec_fn: chrome_47_ios_phone_2,
    },
    GenEntry {
        name: "chrome_47_ios_tablet_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 47,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_47_ios_tablet_2,
    },
    GenEntry {
        name: "chrome_49_ios_phone_3",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 49,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_49_ios_phone_3,
    },
    GenEntry {
        name: "chrome_47_ios_phone_3",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 47,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_47_ios_phone_3,
    },
    GenEntry {
        name: "chrome_52_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 52,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_52_ios_phone_2,
    },
    GenEntry {
        name: "chrome_54_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 54,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_54_ios_phone,
    },
    GenEntry {
        name: "chrome_48_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 48,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_48_ios_phone_2,
    },
    GenEntry {
        name: "chrome_49_ios_phone_4",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 49,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_49_ios_phone_4,
    },
    GenEntry {
        name: "chrome_54_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 54,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_54_ios_phone_2,
    },
    GenEntry {
        name: "chrome_51_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 51,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_51_ios_phone,
    },
    GenEntry {
        name: "chrome_55_ios_phone_4",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 55,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_55_ios_phone_4,
    },
    GenEntry {
        name: "chrome_51_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 51,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_51_ios_tablet,
    },
    GenEntry {
        name: "chrome_55_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 55,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_55_ios_tablet,
    },
    GenEntry {
        name: "chrome_50_ios_tablet_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 50,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_50_ios_tablet_2,
    },
    GenEntry {
        name: "chrome_54_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 54,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_54_ios_tablet,
    },
    GenEntry {
        name: "chrome_47_ios_tablet_3",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 47,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_47_ios_tablet_3,
    },
    GenEntry {
        name: "chrome_51_ios_tablet_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 51,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_51_ios_tablet_2,
    },
    GenEntry {
        name: "chrome_55_ios_tablet_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 55,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_55_ios_tablet_2,
    },
    GenEntry {
        name: "chrome_51_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 51,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_51_ios_phone_2,
    },
    GenEntry {
        name: "chrome_52_ios_phone_3",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 52,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_52_ios_phone_3,
    },
    GenEntry {
        name: "chrome_50_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 50,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_50_ios_phone_2,
    },
    GenEntry {
        name: "chrome_51_ios_phone_3",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 51,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_51_ios_phone_3,
    },
    GenEntry {
        name: "chrome_49_ios_phone_5",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 49,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_49_ios_phone_5,
    },
    GenEntry {
        name: "chrome_50_ios_phone_3",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 50,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_50_ios_phone_3,
    },
    GenEntry {
        name: "chrome_51_ios_phone_4",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 51,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_51_ios_phone_4,
    },
    GenEntry {
        name: "safari_11_ios_phone",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 11,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: safari_11_ios_phone,
    },
    GenEntry {
        name: "safari_15_ios_phone",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 15,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: safari_15_ios_phone,
    },
    GenEntry {
        name: "safari_10_ios_phone",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 10,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: safari_10_ios_phone,
    },
    GenEntry {
        name: "chrome_129_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_129_ios_phone,
    },
    GenEntry {
        name: "chrome_39_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 39,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_39_ios_phone,
    },
    GenEntry {
        name: "chrome_130_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 130,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_130_ios_phone,
    },
    GenEntry {
        name: "safari_18_ios_phone",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 18,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: safari_18_ios_phone,
    },
    GenEntry {
        name: "chrome_130_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 130,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_130_ios_phone_2,
    },
    GenEntry {
        name: "chrome_129_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_129_ios_phone_2,
    },
    GenEntry {
        name: "safari_18_ios_phone_2",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 18,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: safari_18_ios_phone_2,
    },
    GenEntry {
        name: "chrome_129_ios_phone_3",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_129_ios_phone_3,
    },
    GenEntry {
        name: "chrome_129_ios_phone_4",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_129_ios_phone_4,
    },
    GenEntry {
        name: "chrome_123_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 123,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_123_ios_phone,
    },
    GenEntry {
        name: "chrome_128_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 128,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_128_ios_phone,
    },
    GenEntry {
        name: "chrome_130_ios_phone_3",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 130,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_130_ios_phone_3,
    },
    GenEntry {
        name: "chrome_138_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 138,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_138_ios_phone,
    },
    GenEntry {
        name: "chrome_143_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 143,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_143_ios_phone,
    },
    GenEntry {
        name: "safari_18_ios_phone_3",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 18,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: safari_18_ios_phone_3,
    },
    GenEntry {
        name: "safari_13_ios_phone",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 13,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: safari_13_ios_phone,
    },
    GenEntry {
        name: "safari_16_ios_phone",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 16,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: safari_16_ios_phone,
    },
    GenEntry {
        name: "safari_17_ios_phone",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 17,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: safari_17_ios_phone,
    },
    GenEntry {
        name: "safari_10_ios_phone_2",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 10,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: safari_10_ios_phone_2,
    },
    GenEntry {
        name: "chrome_59_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 59,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_59_ios_phone,
    },
    GenEntry {
        name: "safari_6_ios_tablet",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 6,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: safari_6_ios_tablet,
    },
    GenEntry {
        name: "safari_26_ios_phone",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 26,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: safari_26_ios_phone,
    },
    GenEntry {
        name: "chrome_41_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 41,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_41_ios_phone,
    },
    GenEntry {
        name: "chrome_42_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 42,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_42_ios_phone,
    },
    GenEntry {
        name: "chrome_48_ios_phone_3",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 48,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_48_ios_phone_3,
    },
    GenEntry {
        name: "safari_26_ios_phone_2",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 26,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: safari_26_ios_phone_2,
    },
    GenEntry {
        name: "safari_26_ios_phone_3",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 26,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: safari_26_ios_phone_3,
    },
    GenEntry {
        name: "chrome_133_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 133,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_133_ios_phone,
    },
    GenEntry {
        name: "safari_26_ios_phone_4",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 26,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: safari_26_ios_phone_4,
    },
    GenEntry {
        name: "safari_17_ios_phone_2",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 17,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: safari_17_ios_phone_2,
    },
    GenEntry {
        name: "edge_114_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 114,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: edge_114_ios_phone,
    },
    GenEntry {
        name: "safari_16_ios_phone_2",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 16,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: safari_16_ios_phone_2,
    },
    GenEntry {
        name: "chrome_120_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 120,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_120_ios_phone,
    },
    GenEntry {
        name: "edge_135_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 135,
        ja4: "t13d1517h2_8daaf6152771_46b8896bec77",
        spec_fn: edge_135_ios_phone,
    },
    GenEntry {
        name: "edge_140_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 140,
        ja4: "t13d1517h2_8daaf6152771_46b8896bec77",
        spec_fn: edge_140_ios_phone,
    },
    GenEntry {
        name: "chrome_74_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 74,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_74_ios_phone,
    },
    GenEntry {
        name: "chrome_75_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 75,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_75_ios_phone,
    },
    GenEntry {
        name: "safari_6_ios_phone",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 6,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: safari_6_ios_phone,
    },
    GenEntry {
        name: "safari_9_ios_phone",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 9,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: safari_9_ios_phone,
    },
    GenEntry {
        name: "safari_3_ios_phone",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 3,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: safari_3_ios_phone,
    },
    GenEntry {
        name: "chrome_113_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 113,
        ja4: "t13d1710h2_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_113_ios_phone,
    },
    GenEntry {
        name: "safari_16_ios_tablet",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 16,
        ja4: "t13d171100_ab0a1bf427ad_d41ae481755e",
        spec_fn: safari_16_ios_tablet,
    },
    GenEntry {
        name: "chrome_56_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 56,
        ja4: "t13d171100_ab0a1bf427ad_d41ae481755e",
        spec_fn: chrome_56_ios_phone,
    },
    GenEntry {
        name: "edge_116_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 116,
        ja4: "t13d171100_ab0a1bf427ad_d41ae481755e",
        spec_fn: edge_116_ios_phone,
    },
    GenEntry {
        name: "safari_12_ios_phone",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 12,
        ja4: "t13d1711h2_5b57614c22b0_d811adc85aab",
        spec_fn: safari_12_ios_phone,
    },
    GenEntry {
        name: "edge_119_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 119,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: edge_119_ios_phone,
    },
    GenEntry {
        name: "firefox_138_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 138,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: firefox_138_ios_phone,
    },
    GenEntry {
        name: "chrome_132_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 132,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: chrome_132_ios_phone,
    },
    GenEntry {
        name: "chrome_119_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 119,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: chrome_119_ios_phone,
    },
    GenEntry {
        name: "edge_137_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 137,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: edge_137_ios_phone,
    },
    GenEntry {
        name: "safari_13_ios_phone_2",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 13,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: safari_13_ios_phone_2,
    },
    GenEntry {
        name: "edge_133_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 133,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: edge_133_ios_phone,
    },
    GenEntry {
        name: "firefox_115_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 115,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: firefox_115_ios_phone,
    },
    GenEntry {
        name: "firefox_128_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 128,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: firefox_128_ios_phone,
    },
    GenEntry {
        name: "chrome_136_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 136,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: chrome_136_ios_phone,
    },
    GenEntry {
        name: "safari_12_ios_phone_2",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 12,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: safari_12_ios_phone_2,
    },
    GenEntry {
        name: "firefox_136_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 136,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: firefox_136_ios_phone,
    },
    GenEntry {
        name: "chrome_107_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 107,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: chrome_107_ios_phone,
    },
    GenEntry {
        name: "firefox_115_ios_phone_2",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 115,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: firefox_115_ios_phone_2,
    },
    GenEntry {
        name: "safari_16_ios_phone_3",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 16,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: safari_16_ios_phone_3,
    },
    GenEntry {
        name: "edge_106_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 106,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: edge_106_ios_phone,
    },
    GenEntry {
        name: "firefox_120_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 120,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: firefox_120_ios_phone,
    },
    GenEntry {
        name: "firefox_115_ios_phone_3",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 115,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: firefox_115_ios_phone_3,
    },
    GenEntry {
        name: "safari_14_ios_phone",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 14,
        ja4: "t13d1712ht_ab0a1bf427ad_d41ae481755e",
        spec_fn: safari_14_ios_phone,
    },
    GenEntry {
        name: "chrome_140_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 140,
        ja4: "t13d1712ht_ab0a1bf427ad_d41ae481755e",
        spec_fn: chrome_140_ios_phone,
    },
    GenEntry {
        name: "chrome_143_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 143,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: chrome_143_ios_tablet,
    },
    GenEntry {
        name: "brave_1_ios_tablet",
        browser: Browser::Brave,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 1,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: brave_1_ios_tablet,
    },
    GenEntry {
        name: "chrome_146_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 146,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: chrome_146_ios_tablet,
    },
    GenEntry {
        name: "chrome_145_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 145,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: chrome_145_ios_tablet,
    },
    GenEntry {
        name: "firefox_148_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 148,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: firefox_148_ios_phone,
    },
    GenEntry {
        name: "chrome_137_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 137,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: chrome_137_ios_phone,
    },
    GenEntry {
        name: "chrome_139_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 139,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: chrome_139_ios_phone,
    },
    GenEntry {
        name: "brave_1_ios_phone",
        browser: Browser::Brave,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 1,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: brave_1_ios_phone,
    },
    GenEntry {
        name: "safari_26_ios_phone_5",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 26,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: safari_26_ios_phone_5,
    },
    GenEntry {
        name: "chrome_139_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 139,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: chrome_139_ios_phone_2,
    },
    GenEntry {
        name: "chrome_140_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 140,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: chrome_140_ios_phone_2,
    },
    GenEntry {
        name: "chrome_141_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 141,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: chrome_141_ios_phone,
    },
    GenEntry {
        name: "chrome_142_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 142,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: chrome_142_ios_phone,
    },
    GenEntry {
        name: "firefox_143_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 143,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: firefox_143_ios_phone,
    },
    GenEntry {
        name: "firefox_145_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 145,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: firefox_145_ios_phone,
    },
    GenEntry {
        name: "firefox_146_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 146,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: firefox_146_ios_phone,
    },
    GenEntry {
        name: "chrome_143_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 143,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: chrome_143_ios_phone_2,
    },
    GenEntry {
        name: "chrome_144_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 144,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: chrome_144_ios_phone,
    },
    GenEntry {
        name: "edge_143_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 143,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: edge_143_ios_phone,
    },
    GenEntry {
        name: "edge_144_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 144,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: edge_144_ios_phone,
    },
    GenEntry {
        name: "edge_145_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 145,
        ja4: "t13d1713h2_5b57614c22b0_7f0f34a4126d",
        spec_fn: edge_145_ios_phone,
    },
    GenEntry {
        name: "chrome_147_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 147,
        ja4: "t13d1713ht_5b57614c22b0_7f0f34a4126d",
        spec_fn: chrome_147_ios_phone,
    },
    GenEntry {
        name: "safari_26_ios_tablet",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 26,
        ja4: "t13d1713ht_5b57614c22b0_7f0f34a4126d",
        spec_fn: safari_26_ios_tablet,
    },
    GenEntry {
        name: "chrome_147_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 147,
        ja4: "t13d1713ht_5b57614c22b0_7f0f34a4126d",
        spec_fn: chrome_147_ios_tablet,
    },
    GenEntry {
        name: "chrome_146_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 146,
        ja4: "t13d1713ht_5b57614c22b0_7f0f34a4126d",
        spec_fn: chrome_146_ios_phone,
    },
    GenEntry {
        name: "chrome_145_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 145,
        ja4: "t13d1713ht_5b57614c22b0_7f0f34a4126d",
        spec_fn: chrome_145_ios_phone,
    },
    GenEntry {
        name: "chrome_148_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 148,
        ja4: "t13d1713ht_5b57614c22b0_7f0f34a4126d",
        spec_fn: chrome_148_ios_phone,
    },
    GenEntry {
        name: "firefox_149_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 149,
        ja4: "t13d1713ht_5b57614c22b0_7f0f34a4126d",
        spec_fn: firefox_149_ios_phone,
    },
    GenEntry {
        name: "firefox_150_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 150,
        ja4: "t13d1713ht_5b57614c22b0_7f0f34a4126d",
        spec_fn: firefox_150_ios_phone,
    },
    GenEntry {
        name: "chrome_92_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 92,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_92_ios_phone,
    },
    GenEntry {
        name: "chrome_131_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 131,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_131_ios_phone,
    },
    GenEntry {
        name: "chrome_137_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 137,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_137_ios_tablet,
    },
    GenEntry {
        name: "chrome_125_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 125,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_125_ios_tablet,
    },
    GenEntry {
        name: "chrome_126_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 126,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_126_ios_tablet,
    },
    GenEntry {
        name: "chrome_127_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 127,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_127_ios_tablet,
    },
    GenEntry {
        name: "edge_126_ios_tablet",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 126,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: edge_126_ios_tablet,
    },
    GenEntry {
        name: "chrome_128_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 128,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_128_ios_tablet,
    },
    GenEntry {
        name: "chrome_130_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 130,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_130_ios_tablet,
    },
    GenEntry {
        name: "chrome_141_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 141,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_141_ios_tablet,
    },
    GenEntry {
        name: "chrome_131_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 131,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_131_ios_tablet,
    },
    GenEntry {
        name: "edge_131_ios_tablet",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 131,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: edge_131_ios_tablet,
    },
    GenEntry {
        name: "chrome_130_ios_tablet_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 130,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_130_ios_tablet_2,
    },
    GenEntry {
        name: "safari_18_ios_tablet",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 18,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: safari_18_ios_tablet,
    },
    GenEntry {
        name: "chrome_134_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 134,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_134_ios_tablet,
    },
    GenEntry {
        name: "chrome_135_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 135,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_135_ios_tablet,
    },
    GenEntry {
        name: "safari_17_ios_phone_3",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 17,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: safari_17_ios_phone_3,
    },
    GenEntry {
        name: "chrome_137_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 137,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_137_ios_phone_2,
    },
    GenEntry {
        name: "chrome_123_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 123,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_123_ios_phone_2,
    },
    GenEntry {
        name: "chrome_134_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 134,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_134_ios_phone,
    },
    GenEntry {
        name: "firefox_134_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 134,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: firefox_134_ios_phone,
    },
    GenEntry {
        name: "chrome_143_ios_phone_3",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 143,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_143_ios_phone_3,
    },
    GenEntry {
        name: "chrome_132_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 132,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_132_ios_phone_2,
    },
    GenEntry {
        name: "chrome_124_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 124,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_124_ios_phone,
    },
    GenEntry {
        name: "chrome_125_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 125,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_125_ios_phone,
    },
    GenEntry {
        name: "edge_125_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 125,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: edge_125_ios_phone,
    },
    GenEntry {
        name: "chrome_144_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 144,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_144_ios_phone_2,
    },
    GenEntry {
        name: "safari_15_ios_phone_2",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 15,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: safari_15_ios_phone_2,
    },
    GenEntry {
        name: "firefox_126_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 126,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: firefox_126_ios_phone,
    },
    GenEntry {
        name: "chrome_126_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 126,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_126_ios_phone,
    },
    GenEntry {
        name: "chrome_127_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 127,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_127_ios_phone,
    },
    GenEntry {
        name: "edge_126_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 126,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: edge_126_ios_phone,
    },
    GenEntry {
        name: "firefox_127_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 127,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: firefox_127_ios_phone,
    },
    GenEntry {
        name: "chrome_111_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 111,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_111_ios_phone,
    },
    GenEntry {
        name: "chrome_130_ios_phone_4",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 130,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_130_ios_phone_4,
    },
    GenEntry {
        name: "edge_128_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 128,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: edge_128_ios_phone,
    },
    GenEntry {
        name: "edge_143_ios_phone_2",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 143,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: edge_143_ios_phone_2,
    },
    GenEntry {
        name: "chrome_131_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 131,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_131_ios_phone_2,
    },
    GenEntry {
        name: "chrome_133_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 133,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_133_ios_phone_2,
    },
    GenEntry {
        name: "chrome_135_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 135,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_135_ios_phone,
    },
    GenEntry {
        name: "firefox_129_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 129,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: firefox_129_ios_phone,
    },
    GenEntry {
        name: "firefox_130_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 130,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: firefox_130_ios_phone,
    },
    GenEntry {
        name: "firefox_132_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 132,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: firefox_132_ios_phone,
    },
    GenEntry {
        name: "opera_5_ios_phone",
        browser: Browser::Opera,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 5,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: opera_5_ios_phone,
    },
    GenEntry {
        name: "firefox_137_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 137,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: firefox_137_ios_phone,
    },
    GenEntry {
        name: "chrome_128_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 128,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_128_ios_phone_2,
    },
    GenEntry {
        name: "chrome_132_ios_phone_3",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 132,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_132_ios_phone_3,
    },
    GenEntry {
        name: "chrome_133_ios_phone_3",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 133,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: chrome_133_ios_phone_3,
    },
    GenEntry {
        name: "edge_132_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 132,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: edge_132_ios_phone,
    },
    GenEntry {
        name: "firefox_132_ios_phone_2",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 132,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: firefox_132_ios_phone_2,
    },
    GenEntry {
        name: "firefox_133_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 133,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: firefox_133_ios_phone,
    },
    GenEntry {
        name: "firefox_134_ios_phone_2",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 134,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: firefox_134_ios_phone_2,
    },
    GenEntry {
        name: "safari_15_ios_phone_3",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 15,
        ja4: "t13d1714h2_5b57614c22b0_3dd24b5ebec4",
        spec_fn: safari_15_ios_phone_3,
    },
    GenEntry {
        name: "safari_26_ios_phone_6",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 26,
        ja4: "t13d1714h2_5b57614c22b0_d0a99439f9b1",
        spec_fn: safari_26_ios_phone_6,
    },
    GenEntry {
        name: "edge_131_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 131,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: edge_131_ios_phone,
    },
    GenEntry {
        name: "brave_1_ios_phone_2",
        browser: Browser::Brave,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 1,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: brave_1_ios_phone_2,
    },
    GenEntry {
        name: "chrome_133_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 133,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_133_ios_tablet,
    },
    GenEntry {
        name: "chrome_136_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 136,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_136_ios_tablet,
    },
    GenEntry {
        name: "chrome_137_ios_tablet_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 137,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_137_ios_tablet_2,
    },
    GenEntry {
        name: "chrome_138_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 138,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_138_ios_tablet,
    },
    GenEntry {
        name: "chrome_139_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 139,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_139_ios_tablet,
    },
    GenEntry {
        name: "chrome_142_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 142,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_142_ios_tablet,
    },
    GenEntry {
        name: "chrome_143_ios_tablet_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 143,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_143_ios_tablet_2,
    },
    GenEntry {
        name: "chrome_141_ios_tablet_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 141,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_141_ios_tablet_2,
    },
    GenEntry {
        name: "chrome_147_ios_tablet_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 147,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_147_ios_tablet_2,
    },
    GenEntry {
        name: "chrome_148_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 148,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_148_ios_tablet,
    },
    GenEntry {
        name: "edge_122_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 122,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: edge_122_ios_phone,
    },
    GenEntry {
        name: "chrome_127_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 127,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_127_ios_phone_2,
    },
    GenEntry {
        name: "chrome_141_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 141,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_141_ios_phone_2,
    },
    GenEntry {
        name: "safari_15_ios_phone_4",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 15,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: safari_15_ios_phone_4,
    },
    GenEntry {
        name: "chrome_134_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 134,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_134_ios_phone_2,
    },
    GenEntry {
        name: "firefox_135_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 135,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: firefox_135_ios_phone,
    },
    GenEntry {
        name: "chrome_135_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 135,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_135_ios_phone_2,
    },
    GenEntry {
        name: "chrome_137_ios_phone_3",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 137,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_137_ios_phone_3,
    },
    GenEntry {
        name: "edge_134_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 134,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: edge_134_ios_phone,
    },
    GenEntry {
        name: "edge_135_ios_phone_2",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 135,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: edge_135_ios_phone_2,
    },
    GenEntry {
        name: "opera_5_ios_phone_2",
        browser: Browser::Opera,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 5,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: opera_5_ios_phone_2,
    },
    GenEntry {
        name: "chrome_136_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 136,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_136_ios_phone_2,
    },
    GenEntry {
        name: "firefox_138_ios_phone_2",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 138,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: firefox_138_ios_phone_2,
    },
    GenEntry {
        name: "firefox_139_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 139,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: firefox_139_ios_phone,
    },
    GenEntry {
        name: "firefox_140_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 140,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: firefox_140_ios_phone,
    },
    GenEntry {
        name: "firefox_141_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 141,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: firefox_141_ios_phone,
    },
    GenEntry {
        name: "chrome_139_ios_phone_3",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 139,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_139_ios_phone_3,
    },
    GenEntry {
        name: "edge_137_ios_phone_2",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 137,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: edge_137_ios_phone_2,
    },
    GenEntry {
        name: "chrome_142_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 142,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_142_ios_phone_2,
    },
    GenEntry {
        name: "chrome_147_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 147,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_147_ios_phone_2,
    },
    GenEntry {
        name: "edge_140_ios_phone_2",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 140,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: edge_140_ios_phone_2,
    },
    GenEntry {
        name: "edge_141_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 141,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: edge_141_ios_phone,
    },
    GenEntry {
        name: "edge_142_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 142,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: edge_142_ios_phone,
    },
    GenEntry {
        name: "edge_143_ios_phone_3",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 143,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: edge_143_ios_phone_3,
    },
    GenEntry {
        name: "firefox_143_ios_phone_2",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 143,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: firefox_143_ios_phone_2,
    },
    GenEntry {
        name: "firefox_145_ios_phone_2",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 145,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: firefox_145_ios_phone_2,
    },
    GenEntry {
        name: "firefox_146_ios_phone_2",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 146,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: firefox_146_ios_phone_2,
    },
    GenEntry {
        name: "chrome_144_ios_phone_3",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 144,
        ja4: "t13d1714h2_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_144_ios_phone_3,
    },
    GenEntry {
        name: "chrome_146_ios_tablet_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 146,
        ja4: "t13d1714ht_5b57614c22b0_e42f34c56612",
        spec_fn: chrome_146_ios_tablet_2,
    },
    GenEntry {
        name: "edge_146_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 146,
        ja4: "t13d1714ht_5b57614c22b0_e42f34c56612",
        spec_fn: edge_146_ios_phone,
    },
    GenEntry {
        name: "firefox_12_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 12,
        ja4: "t13d1714ht_5b57614c22b0_e42f34c56612",
        spec_fn: firefox_12_ios_phone,
    },
    GenEntry {
        name: "firefox_35_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 35,
        ja4: "t13d1715h2_5b57614c22b0_3d5424432f57",
        spec_fn: firefox_35_ios_phone,
    },
    GenEntry {
        name: "firefox_118_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 118,
        ja4: "t13d201100_2b729b4bf6f3_36bf25f296df",
        spec_fn: firefox_118_ios_phone,
    },
    GenEntry {
        name: "safari_26_ios_phone_7",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 26,
        ja4: "t13d2913h2_723694b0fccc_5671b5df5029",
        spec_fn: safari_26_ios_phone_7,
    },
    GenEntry {
        name: "chrome_132_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 132,
        ja4: "t13d301000_1d37bd780c83_1f22a2ca17c4",
        spec_fn: chrome_132_ios_tablet,
    },
    GenEntry {
        name: "safari_4_ios_phone",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 4,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: safari_4_ios_phone,
    },
    GenEntry {
        name: "firefox_18_ios_tablet",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 18,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_18_ios_tablet,
    },
    GenEntry {
        name: "firefox_10_ios_tablet",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 10,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_10_ios_tablet,
    },
    GenEntry {
        name: "firefox_11_ios_tablet",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 11,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_11_ios_tablet,
    },
    GenEntry {
        name: "chrome_61_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 61,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_61_ios_tablet,
    },
    GenEntry {
        name: "firefox_17_ios_tablet",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 17,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_17_ios_tablet,
    },
    GenEntry {
        name: "chrome_57_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 57,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_57_ios_tablet,
    },
    GenEntry {
        name: "chrome_46_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 46,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_46_ios_tablet,
    },
    GenEntry {
        name: "firefox_15_ios_tablet",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 15,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_15_ios_tablet,
    },
    GenEntry {
        name: "chrome_42_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 42,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_42_ios_tablet,
    },
    GenEntry {
        name: "firefox_11_ios_tablet_2",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 11,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_11_ios_tablet_2,
    },
    GenEntry {
        name: "chrome_28_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 28,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_28_ios_tablet,
    },
    GenEntry {
        name: "firefox_12_ios_tablet",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 12,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_12_ios_tablet,
    },
    GenEntry {
        name: "chrome_18_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 18,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_18_ios_tablet,
    },
    GenEntry {
        name: "chrome_29_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 29,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_29_ios_tablet,
    },
    GenEntry {
        name: "firefox_13_ios_tablet",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 13,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_13_ios_tablet,
    },
    GenEntry {
        name: "firefox_14_ios_tablet",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 14,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_14_ios_tablet,
    },
    GenEntry {
        name: "firefox_10_ios_tablet_2",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 10,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_10_ios_tablet_2,
    },
    GenEntry {
        name: "chrome_30_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 30,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_30_ios_tablet,
    },
    GenEntry {
        name: "firefox_16_ios_tablet",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 16,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_16_ios_tablet,
    },
    GenEntry {
        name: "chrome_49_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 49,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_49_ios_tablet,
    },
    GenEntry {
        name: "firefox_9_ios_tablet",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 9,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_9_ios_tablet,
    },
    GenEntry {
        name: "chrome_38_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 38,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_38_ios_tablet,
    },
    GenEntry {
        name: "firefox_13_ios_tablet_2",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 13,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_13_ios_tablet_2,
    },
    GenEntry {
        name: "firefox_14_ios_tablet_2",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 14,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_14_ios_tablet_2,
    },
    GenEntry {
        name: "firefox_17_ios_tablet_2",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 17,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_17_ios_tablet_2,
    },
    GenEntry {
        name: "firefox_13_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 13,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_13_ios_phone,
    },
    GenEntry {
        name: "chrome_48_ios_phone_4",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 48,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_48_ios_phone_4,
    },
    GenEntry {
        name: "firefox_10_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 10,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_10_ios_phone,
    },
    GenEntry {
        name: "chrome_50_ios_phone_4",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 50,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_50_ios_phone_4,
    },
    GenEntry {
        name: "firefox_14_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 14,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_14_ios_phone,
    },
    GenEntry {
        name: "firefox_13_ios_phone_2",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 13,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_13_ios_phone_2,
    },
    GenEntry {
        name: "firefox_18_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 18,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_18_ios_phone,
    },
    GenEntry {
        name: "firefox_12_ios_phone_2",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 12,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_12_ios_phone_2,
    },
    GenEntry {
        name: "chrome_46_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 46,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_46_ios_phone,
    },
    GenEntry {
        name: "chrome_59_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 59,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_59_ios_phone_2,
    },
    GenEntry {
        name: "chrome_42_ios_phone_2",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 42,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_42_ios_phone_2,
    },
    GenEntry {
        name: "firefox_16_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 16,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_16_ios_phone,
    },
    GenEntry {
        name: "chrome_44_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 44,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_44_ios_phone,
    },
    GenEntry {
        name: "chrome_22_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 22,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_22_ios_phone,
    },
    GenEntry {
        name: "firefox_18_ios_phone_2",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 18,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_18_ios_phone_2,
    },
    GenEntry {
        name: "firefox_17_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 17,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_17_ios_phone,
    },
    GenEntry {
        name: "chrome_45_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 45,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_45_ios_phone,
    },
    GenEntry {
        name: "chrome_17_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 17,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_17_ios_phone,
    },
    GenEntry {
        name: "firefox_11_ios_phone",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 11,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_11_ios_phone,
    },
    GenEntry {
        name: "chrome_53_ios_phone_4",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 53,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_53_ios_phone_4,
    },
    GenEntry {
        name: "firefox_10_ios_phone_2",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 10,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_10_ios_phone_2,
    },
    GenEntry {
        name: "firefox_14_ios_phone_2",
        browser: Browser::Firefox,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 14,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_14_ios_phone_2,
    },
    GenEntry {
        name: "chrome_14_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 14,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_14_ios_phone,
    },
    GenEntry {
        name: "chrome_24_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 24,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_24_ios_phone,
    },
    GenEntry {
        name: "chrome_62_ios_phone",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 62,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_62_ios_phone,
    },
    GenEntry {
        name: "safari_5_ios_phone",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 5,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: safari_5_ios_phone,
    },
    GenEntry {
        name: "safari_4_ios_phone_2",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 4,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: safari_4_ios_phone_2,
    },
    GenEntry {
        name: "safari_3_ios_phone_2",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 3,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: safari_3_ios_phone_2,
    },
    GenEntry {
        name: "safari_12_ios_tablet",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 12,
        ja4: "t13d301200_1d37bd780c83_d339722ba4af",
        spec_fn: safari_12_ios_tablet,
    },
    GenEntry {
        name: "edge_121_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 121,
        ja4: "t13d3012h2_1d37bd780c83_b26ce05bbdd6",
        spec_fn: edge_121_ios_phone,
    },
    GenEntry {
        name: "opera_4_ios_phone",
        browser: Browser::Opera,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 4,
        ja4: "t13d3012h2_1d37bd780c83_b26ce05bbdd6",
        spec_fn: opera_4_ios_phone,
    },
    GenEntry {
        name: "edge_120_ios_phone",
        browser: Browser::Edge,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 120,
        ja4: "t13d3012h2_1d37bd780c83_b26ce05bbdd6",
        spec_fn: edge_120_ios_phone,
    },
    GenEntry {
        name: "opera_2_ios_phone",
        browser: Browser::Opera,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 2,
        ja4: "t13d3013h2_1d37bd780c83_ce5650b735ce",
        spec_fn: opera_2_ios_phone,
    },
    GenEntry {
        name: "chrome_43_ios_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 43,
        ja4: "t13d320900_47c5e39c651d_371c70bbb337",
        spec_fn: chrome_43_ios_tablet,
    },
    GenEntry {
        name: "safari_8_ios_phone",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 8,
        ja4: "t13d320900_47c5e39c651d_371c70bbb337",
        spec_fn: safari_8_ios_phone,
    },
    GenEntry {
        name: "safari_10_ios_tablet",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 10,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: safari_10_ios_tablet,
    },
    GenEntry {
        name: "safari_5_ios_phone_2",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Phone,
        major: 5,
        ja4: "t13d3613h2_c014a34ff1af_aac333855136",
        spec_fn: safari_5_ios_phone_2,
    },
    GenEntry {
        name: "safari_9_ios_tablet",
        browser: Browser::Safari,
        os: Some(Os::Ios),
        device: Device::Tablet,
        major: 9,
        ja4: "t13d481000_c08b26b7ea02_5ac7197df9d2",
        spec_fn: safari_9_ios_tablet,
    },
];

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    safari_17_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], status, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          ecpf, sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8730
#[rustfmt::skip]
spec! {
    chrome_122_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], status, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          ecpf, sct,
}

// ja4=t13d131100_f57a46bbacb6_ab7e3b40a677 obs=588
#[rustfmt::skip]
spec! {
    firefox_121_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], status, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603],
          ecpf, sct, raw[0x0032, "00140804040308070805080604010501060105030603"],
}

// ja4=t13d1513h2_8daaf6152771_eca864cca44a obs=21
#[rustfmt::skip]
spec! {
    firefox_124_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], ecpf,
          padding, ticket,
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_52_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0011, ""], ticket,
          raw[0x0032, "002a040305030603080708080804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_50_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0011, ""], ticket,
          raw[0x0032, "002a040305030603080708080804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_47_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0011, ""], ticket,
          raw[0x0032, "002a040305030603080708080804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_55_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0011, ""], ticket,
          raw[0x0032, "002a040305030603080708080804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_50_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0011, ""], ticket,
          raw[0x0032, "002a040305030603080708080804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_49_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0011, ""], ticket,
          raw[0x0032, "002a040305030603080708080804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_53_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0011, ""], ticket,
          raw[0x0032, "002a040305030603080708080804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_52_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0011, ""], ticket,
          raw[0x0032, "002a040305030603080708080804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=460
#[rustfmt::skip]
spec! {
    chrome_55_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0011, ""], ticket,
          raw[0x0032, "002a040305030603080708080804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_49_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0011, ""], ticket,
          raw[0x0032, "002a040305030603080708080804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=460
#[rustfmt::skip]
spec! {
    chrome_53_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0011, ""], ticket,
          raw[0x0032, "002a040305030603080708080804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_47_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0011, ""], ticket,
          raw[0x0032, "002a040305030603080708080804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_48_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0011, ""], ticket,
          raw[0x0032, "002a040305030603080708080804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_55_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0011, ""], ticket,
          raw[0x0032, "002a040305030603080708080804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d1515ht_8daaf6152771_4769d65a485e obs=534
#[rustfmt::skip]
spec! {
    chrome_53_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "002a040305030603080708080804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d1515ht_8daaf6152771_4769d65a485e obs=534
#[rustfmt::skip]
spec! {
    chrome_47_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "002a040305030603080708080804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_47_ios_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_49_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_47_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_52_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_54_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_48_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_49_ios_phone_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_54_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_51_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1616
#[rustfmt::skip]
spec! {
    chrome_55_ios_phone_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_51_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_55_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_50_ios_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_54_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_47_ios_tablet_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_51_ios_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_55_ios_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_51_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_52_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1616
#[rustfmt::skip]
spec! {
    chrome_50_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_51_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1617
#[rustfmt::skip]
spec! {
    chrome_49_ios_phone_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1616
#[rustfmt::skip]
spec! {
    chrome_50_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_51_ios_phone_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, raw[0x0011, ""], ticket, raw[0x0029, ""],
          raw[0x0032, "001604030503060308040805080604010501060102010203"],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    safari_11_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17176
#[rustfmt::skip]
spec! {
    safari_15_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17194
#[rustfmt::skip]
spec! {
    safari_10_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17175
#[rustfmt::skip]
spec! {
    chrome_129_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_39_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17180
#[rustfmt::skip]
spec! {
    chrome_130_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    safari_18_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17177
#[rustfmt::skip]
spec! {
    chrome_130_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_129_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    safari_18_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17176
#[rustfmt::skip]
spec! {
    chrome_129_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17177
#[rustfmt::skip]
spec! {
    chrome_129_ios_phone_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17175
#[rustfmt::skip]
spec! {
    chrome_123_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_128_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_130_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    chrome_138_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    chrome_143_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35853
#[rustfmt::skip]
spec! {
    safari_18_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35886
#[rustfmt::skip]
spec! {
    safari_13_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    safari_16_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    safari_17_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    safari_10_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_59_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    safari_6_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35856
#[rustfmt::skip]
spec! {
    safari_26_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_41_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_42_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_48_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    safari_26_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    safari_26_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_133_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35853
#[rustfmt::skip]
spec! {
    safari_26_ios_phone_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    safari_17_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          compress[], ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2094
#[rustfmt::skip]
spec! {
    edge_114_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          padding, compress[], ticket, appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    safari_16_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          padding, compress[], ticket, appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    chrome_120_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], ecpf, sct,
          padding, compress[], ticket, appsettings["h2", "http/1.1"],
}

// ja4=t13d1517h2_8daaf6152771_46b8896bec77 obs=7041
#[rustfmt::skip]
spec! {
    edge_135_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, sct, compress[], ticket, raw[0x0029, ""],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1517h2_8daaf6152771_46b8896bec77 obs=7041
#[rustfmt::skip]
spec! {
    edge_140_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          ecpf, sct, compress[], ticket, raw[0x0029, ""],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_74_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, reneg, groups[x25519, p256, p384], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          ecpf, sct,
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_75_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, reneg, groups[x25519, p256, p384], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          ecpf, sct,
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8607
#[rustfmt::skip]
spec! {
    safari_6_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, reneg, groups[x25519, p256, p384], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          ecpf, sct,
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    safari_9_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, reneg, groups[x25519, p256, p384], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          ecpf, sct,
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8607
#[rustfmt::skip]
spec! {
    safari_3_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, reneg, groups[x25519, p256, p384], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          ecpf, sct,
}

// ja4=t13d1710h2_5b57614c22b0_97f8aa674fd9 obs=186
#[rustfmt::skip]
spec! {
    chrome_113_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          ecpf, sct,
}

// ja4=t13d171100_ab0a1bf427ad_d41ae481755e obs=576
#[rustfmt::skip]
spec! {
    safari_16_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], ticket,
}

// ja4=t13d171100_ab0a1bf427ad_d41ae481755e obs=583
#[rustfmt::skip]
spec! {
    chrome_56_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], ticket,
}

// ja4=t13d171100_ab0a1bf427ad_d41ae481755e obs=576
#[rustfmt::skip]
spec! {
    edge_116_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], ticket,
}

// ja4=t13d1711h2_5b57614c22b0_d811adc85aab obs=213
#[rustfmt::skip]
spec! {
    safari_12_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          ecpf, sct, raw[0x3374, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    edge_119_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    firefox_138_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    chrome_132_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    chrome_119_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    edge_137_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    safari_13_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    edge_133_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    firefox_115_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    firefox_128_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    chrome_136_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    safari_12_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    firefox_136_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    chrome_107_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    firefox_115_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    safari_16_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    edge_106_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    firefox_120_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    firefox_115_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_d41ae481755e obs=2332
#[rustfmt::skip]
spec! {
    safari_14_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], ticket,
}

// ja4=t13d1712ht_ab0a1bf427ad_d41ae481755e obs=2329
#[rustfmt::skip]
spec! {
    chrome_140_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0x0067, 0x006b, 0x009e, 0x009f,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], ticket,
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1136
#[rustfmt::skip]
spec! {
    chrome_143_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1139
#[rustfmt::skip]
spec! {
    brave_1_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1136
#[rustfmt::skip]
spec! {
    chrome_146_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1136
#[rustfmt::skip]
spec! {
    chrome_145_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1137
#[rustfmt::skip]
spec! {
    firefox_148_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1137
#[rustfmt::skip]
spec! {
    chrome_137_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1136
#[rustfmt::skip]
spec! {
    chrome_139_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1159
#[rustfmt::skip]
spec! {
    brave_1_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1141
#[rustfmt::skip]
spec! {
    safari_26_ios_phone_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1136
#[rustfmt::skip]
spec! {
    chrome_139_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1138
#[rustfmt::skip]
spec! {
    chrome_140_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1146
#[rustfmt::skip]
spec! {
    chrome_141_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1144
#[rustfmt::skip]
spec! {
    chrome_142_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1136
#[rustfmt::skip]
spec! {
    firefox_143_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1137
#[rustfmt::skip]
spec! {
    firefox_145_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1136
#[rustfmt::skip]
spec! {
    firefox_146_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1162
#[rustfmt::skip]
spec! {
    chrome_143_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1147
#[rustfmt::skip]
spec! {
    chrome_144_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1136
#[rustfmt::skip]
spec! {
    edge_143_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1136
#[rustfmt::skip]
spec! {
    edge_144_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713h2_5b57614c22b0_7f0f34a4126d obs=1137
#[rustfmt::skip]
spec! {
    edge_145_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713ht_5b57614c22b0_7f0f34a4126d obs=1624
#[rustfmt::skip]
spec! {
    chrome_147_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713ht_5b57614c22b0_7f0f34a4126d obs=1608
#[rustfmt::skip]
spec! {
    safari_26_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713ht_5b57614c22b0_7f0f34a4126d obs=1604
#[rustfmt::skip]
spec! {
    chrome_147_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713ht_5b57614c22b0_7f0f34a4126d obs=1607
#[rustfmt::skip]
spec! {
    chrome_146_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713ht_5b57614c22b0_7f0f34a4126d obs=1607
#[rustfmt::skip]
spec! {
    chrome_145_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713ht_5b57614c22b0_7f0f34a4126d obs=1612
#[rustfmt::skip]
spec! {
    chrome_148_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713ht_5b57614c22b0_7f0f34a4126d obs=1603
#[rustfmt::skip]
spec! {
    firefox_149_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1713ht_5b57614c22b0_7f0f34a4126d obs=1603
#[rustfmt::skip]
spec! {
    firefox_150_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    chrome_92_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2661
#[rustfmt::skip]
spec! {
    chrome_131_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    chrome_137_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    chrome_125_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    chrome_126_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    chrome_127_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2640
#[rustfmt::skip]
spec! {
    edge_126_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    chrome_128_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    chrome_130_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    chrome_141_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    chrome_131_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2639
#[rustfmt::skip]
spec! {
    edge_131_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2639
#[rustfmt::skip]
spec! {
    chrome_130_ios_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2645
#[rustfmt::skip]
spec! {
    safari_18_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2639
#[rustfmt::skip]
spec! {
    chrome_134_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    chrome_135_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    safari_17_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2643
#[rustfmt::skip]
spec! {
    chrome_137_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    chrome_123_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    chrome_134_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    firefox_134_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2642
#[rustfmt::skip]
spec! {
    chrome_143_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2639
#[rustfmt::skip]
spec! {
    chrome_132_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2640
#[rustfmt::skip]
spec! {
    chrome_124_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2668
#[rustfmt::skip]
spec! {
    chrome_125_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2641
#[rustfmt::skip]
spec! {
    edge_125_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    chrome_144_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2643
#[rustfmt::skip]
spec! {
    safari_15_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2644
#[rustfmt::skip]
spec! {
    firefox_126_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2663
#[rustfmt::skip]
spec! {
    chrome_126_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2650
#[rustfmt::skip]
spec! {
    chrome_127_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2639
#[rustfmt::skip]
spec! {
    edge_126_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2641
#[rustfmt::skip]
spec! {
    firefox_127_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2639
#[rustfmt::skip]
spec! {
    chrome_111_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2640
#[rustfmt::skip]
spec! {
    chrome_130_ios_phone_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    edge_128_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    edge_143_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2641
#[rustfmt::skip]
spec! {
    chrome_131_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2640
#[rustfmt::skip]
spec! {
    chrome_133_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    chrome_135_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    firefox_129_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2641
#[rustfmt::skip]
spec! {
    firefox_130_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    firefox_132_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2641
#[rustfmt::skip]
spec! {
    opera_5_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2638
#[rustfmt::skip]
spec! {
    firefox_137_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2644
#[rustfmt::skip]
spec! {
    chrome_128_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2643
#[rustfmt::skip]
spec! {
    chrome_132_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2640
#[rustfmt::skip]
spec! {
    chrome_133_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2639
#[rustfmt::skip]
spec! {
    edge_132_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2641
#[rustfmt::skip]
spec! {
    firefox_132_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2640
#[rustfmt::skip]
spec! {
    firefox_133_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2639
#[rustfmt::skip]
spec! {
    firefox_134_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_3dd24b5ebec4 obs=232
#[rustfmt::skip]
spec! {
    safari_15_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          ecpf, compress[], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1714h2_5b57614c22b0_d0a99439f9b1 obs=3
#[rustfmt::skip]
spec! {
    safari_26_ios_phone_6,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, compress[], ticket,
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1794
#[rustfmt::skip]
spec! {
    edge_131_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1805
#[rustfmt::skip]
spec! {
    brave_1_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1794
#[rustfmt::skip]
spec! {
    chrome_133_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1798
#[rustfmt::skip]
spec! {
    chrome_136_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1794
#[rustfmt::skip]
spec! {
    chrome_137_ios_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1794
#[rustfmt::skip]
spec! {
    chrome_138_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1794
#[rustfmt::skip]
spec! {
    chrome_139_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1794
#[rustfmt::skip]
spec! {
    chrome_142_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1794
#[rustfmt::skip]
spec! {
    chrome_143_ios_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1795
#[rustfmt::skip]
spec! {
    chrome_141_ios_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1794
#[rustfmt::skip]
spec! {
    chrome_147_ios_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1794
#[rustfmt::skip]
spec! {
    chrome_148_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1794
#[rustfmt::skip]
spec! {
    edge_122_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1794
#[rustfmt::skip]
spec! {
    chrome_127_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1797
#[rustfmt::skip]
spec! {
    chrome_141_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1798
#[rustfmt::skip]
spec! {
    safari_15_ios_phone_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1803
#[rustfmt::skip]
spec! {
    chrome_134_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1794
#[rustfmt::skip]
spec! {
    firefox_135_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1809
#[rustfmt::skip]
spec! {
    chrome_135_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1807
#[rustfmt::skip]
spec! {
    chrome_137_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1794
#[rustfmt::skip]
spec! {
    edge_134_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1796
#[rustfmt::skip]
spec! {
    edge_135_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1794
#[rustfmt::skip]
spec! {
    opera_5_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1811
#[rustfmt::skip]
spec! {
    chrome_136_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1794
#[rustfmt::skip]
spec! {
    firefox_138_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1795
#[rustfmt::skip]
spec! {
    firefox_139_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1796
#[rustfmt::skip]
spec! {
    firefox_140_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1797
#[rustfmt::skip]
spec! {
    firefox_141_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1806
#[rustfmt::skip]
spec! {
    chrome_139_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1794
#[rustfmt::skip]
spec! {
    edge_137_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1802
#[rustfmt::skip]
spec! {
    chrome_142_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1797
#[rustfmt::skip]
spec! {
    chrome_147_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1808
#[rustfmt::skip]
spec! {
    edge_140_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1810
#[rustfmt::skip]
spec! {
    edge_141_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1817
#[rustfmt::skip]
spec! {
    edge_142_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1806
#[rustfmt::skip]
spec! {
    edge_143_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1794
#[rustfmt::skip]
spec! {
    firefox_143_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1797
#[rustfmt::skip]
spec! {
    firefox_145_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1796
#[rustfmt::skip]
spec! {
    firefox_146_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_e42f34c56612 obs=1795
#[rustfmt::skip]
spec! {
    chrome_144_ios_phone_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714ht_5b57614c22b0_e42f34c56612 obs=1891
#[rustfmt::skip]
spec! {
    chrome_146_ios_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714ht_5b57614c22b0_e42f34c56612 obs=1891
#[rustfmt::skip]
spec! {
    edge_146_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1714ht_5b57614c22b0_e42f34c56612 obs=1891
#[rustfmt::skip]
spec! {
    firefox_12_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          ecpf, sct, padding, compress[],
}

// ja4=t13d1715h2_5b57614c22b0_3d5424432f57 obs=676
#[rustfmt::skip]
spec! {
    firefox_35_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          ecpf, padding, rslimit[16385], raw[0x0022, ""], ticket,
}

// ja4=t13d201100_2b729b4bf6f3_36bf25f296df obs=269
#[rustfmt::skip]
spec! {
    firefox_118_ios_phone,
    ciphers: 0x1301, 0x1302, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xc024, 0xc023, 0xc028, 0xc027, 0xc00a,
             0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0201, 0x0403, 0x0503, 0x0203, 0x0202, 0x0601, 0x0603],
          ecpf, ticket, raw[0x0031, ""],
}

// ja4=t13d2913h2_723694b0fccc_5671b5df5029 obs=30
#[rustfmt::skip]
spec! {
    safari_26_ios_phone_7,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f, 0x0033, 0x0039, 0x009e, 0x009f, 0x1304,
             0xc09c, 0xc09d, 0xc09e, 0xc09f, 0xc0ac, 0xc0ad, 0xccaa,
    session: empty,
    exts: sni, reneg, groups[x25519, p256, p384], alpn["h2", "http/1.1"], status, keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0401, 0x0809, 0x0804, 0x0403, 0x0807, 0x0501, 0x080a, 0x0805, 0x0503, 0x0808, 0x0601, 0x080b, 0x0806, 0x0603, 0x0201, 0x0203],
          ecpf, padding, rslimit[16385], ticket,
}

// ja4=t13d301000_1d37bd780c83_1f22a2ca17c4 obs=98
#[rustfmt::skip]
spec! {
    chrome_132_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, raw[0x0016, ""], ticket,
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1189
#[rustfmt::skip]
spec! {
    safari_4_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_18_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_10_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_11_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_61_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_17_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_57_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_46_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_15_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_42_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_11_ios_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_28_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_12_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_18_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_29_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_13_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_14_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_10_ios_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_30_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_16_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_49_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_9_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_38_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_13_ios_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_14_ios_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_17_ios_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_13_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_48_ios_phone_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_10_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_50_ios_phone_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_14_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_13_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_18_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_12_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_46_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_59_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_42_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_16_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_44_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_22_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_18_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_17_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_45_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_17_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_11_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_53_ios_phone_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_10_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_14_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_14_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_24_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_62_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1188
#[rustfmt::skip]
spec! {
    safari_5_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1189
#[rustfmt::skip]
spec! {
    safari_4_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1193
#[rustfmt::skip]
spec! {
    safari_3_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""],
}

// ja4=t13d301200_1d37bd780c83_d339722ba4af obs=2378
#[rustfmt::skip]
spec! {
    safari_12_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], ticket, raw[0x0031, ""],
}

// ja4=t13d3012h2_1d37bd780c83_b26ce05bbdd6 obs=28316
#[rustfmt::skip]
spec! {
    edge_121_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["h2", "http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d3012h2_1d37bd780c83_b26ce05bbdd6 obs=28317
#[rustfmt::skip]
spec! {
    opera_4_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["h2", "http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d3012h2_1d37bd780c83_b26ce05bbdd6 obs=28317
#[rustfmt::skip]
spec! {
    edge_120_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["h2", "http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d3013h2_1d37bd780c83_ce5650b735ce obs=1199
#[rustfmt::skip]
spec! {
    opera_2_ios_phone,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["h2", "http/1.1"], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          ecpf, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d320900_47c5e39c651d_371c70bbb337 obs=6
#[rustfmt::skip]
spec! {
    chrome_43_ios_tablet,
    ciphers: 0x1301, 0x1302, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xc024, 0xc023, 0xc028, 0xc027, 0xc00a,
             0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035, 0x002f, 0x0032, 0x0033,
             0x0038, 0x0039, 0x0040, 0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0032, "00200403050306030804080508060809080a080b0401050106010402020302010202"],
}

// ja4=t13d320900_47c5e39c651d_371c70bbb337 obs=6
#[rustfmt::skip]
spec! {
    safari_8_ios_phone,
    ciphers: 0x1301, 0x1302, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xc024, 0xc023, 0xc028, 0xc027, 0xc00a,
             0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035, 0x002f, 0x0032, 0x0033,
             0x0038, 0x0039, 0x0040, 0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0203, 0x0201, 0x0202],
          ecpf, raw[0x0032, "00200403050306030804080508060809080a080b0401050106010402020302010202"],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1187
#[rustfmt::skip]
spec! {
    safari_10_ios_tablet,
    ciphers: 0x1301, 0x1302, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xc024, 0xc023, 0xc028, 0xc027, 0xc00a,
             0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035, 0x002f, 0x0032, 0x0033,
             0x0038, 0x0039, 0x0040, 0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          ecpf,
          raw[0x0032, "00260403050306030804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d3613h2_c014a34ff1af_aac333855136 obs=28
#[rustfmt::skip]
spec! {
    safari_5_ios_phone_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0032, 0x0033, 0x0038, 0x0039, 0x0040, 0x0067, 0x006a, 0x006b, 0x009e, 0x009f,
             0x00a2, 0x00a3, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], alpn["h2", "http/1.1"], status,
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202, 0x0101],
          ecpf, raw[0x0011, ""], ticket,
          raw[0x0032, "002c040305030603080708080804080508060809080a080b04010501060104020303030103020203020102020101"],
}

// ja4=t13d481000_c08b26b7ea02_5ac7197df9d2 obs=100
#[rustfmt::skip]
spec! {
    safari_9_ios_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0032, 0x0033, 0x0038, 0x0039, 0x0040, 0x0067, 0x006a, 0x006b, 0x009e, 0x009f,
             0x00a2, 0x00a3, 0xc09c, 0xc09d, 0xc09e, 0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac,
             0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          ecpf, raw[0x0016, ""], ticket,
}
