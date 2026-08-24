//! Chromium family Android hellos (the `chrome_android` wire template)
//!
//! Emitter output (`gen_specs.py --emit`); do not edit by hand.
//! Regeneration is byte-deterministic (`--selftest` verifies the
//! committed files match a fresh render).

use super::GenEntry;
use crate::fingerprints::{Browser, Device, Os};

#[rustfmt::skip]
pub const GENERATED: &[GenEntry] = &[
    GenEntry {
        name: "chrome_77_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 77,
        ja4: "t13d121000_0ed44715e6cd_78e6aca7449b",
        spec_fn: chrome_77_android_desktop,
    },
    GenEntry {
        name: "chrome_90_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 90,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_90_android_desktop,
    },
    GenEntry {
        name: "chrome_105_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 105,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_105_android_desktop,
    },
    GenEntry {
        name: "chrome_114_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 114,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_114_android_desktop,
    },
    GenEntry {
        name: "samsung_23_android_desktop",
        browser: Browser::SamsungInternet,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 23,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: samsung_23_android_desktop,
    },
    GenEntry {
        name: "chrome_80_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 80,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_80_android_desktop,
    },
    GenEntry {
        name: "opera_80_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 80,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: opera_80_android_desktop,
    },
    GenEntry {
        name: "chrome_118_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 118,
        ja4: "t13d131100_f57a46bbacb6_ab7e3b40a677",
        spec_fn: chrome_118_android_desktop,
    },
    GenEntry {
        name: "chrome_45_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 45,
        ja4: "t13d131100_f57a46bbacb6_ab7e3b40a677",
        spec_fn: chrome_45_android_tablet,
    },
    GenEntry {
        name: "chrome_73_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 73,
        ja4: "t13d1311h2_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_73_android_desktop,
    },
    GenEntry {
        name: "chrome_48_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 48,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_48_android_desktop,
    },
    GenEntry {
        name: "chrome_78_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 78,
        ja4: "t13d1515h2_8daaf6152771_45f260be83e2",
        spec_fn: chrome_78_android_desktop,
    },
    GenEntry {
        name: "chrome_87_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 87,
        ja4: "t13d1515h2_8daaf6152771_de4a06bb82e3",
        spec_fn: chrome_87_android_desktop,
    },
    GenEntry {
        name: "chrome_86_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 86,
        ja4: "t13d1515h2_8daaf6152771_de4a06bb82e3",
        spec_fn: chrome_86_android_desktop,
    },
    GenEntry {
        name: "chrome_89_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 89,
        ja4: "t13d1515h2_8daaf6152771_de4a06bb82e3",
        spec_fn: chrome_89_android_desktop,
    },
    GenEntry {
        name: "chrome_86_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 86,
        ja4: "t13d1515h2_8daaf6152771_de4a06bb82e3",
        spec_fn: chrome_86_android_tablet,
    },
    GenEntry {
        name: "chrome_55_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 55,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_55_android_desktop,
    },
    GenEntry {
        name: "chrome_49_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 49,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_49_android_desktop,
    },
    GenEntry {
        name: "chrome_47_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 47,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_47_android_desktop,
    },
    GenEntry {
        name: "chrome_54_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 54,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_54_android_desktop,
    },
    GenEntry {
        name: "chrome_53_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 53,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_53_android_desktop,
    },
    GenEntry {
        name: "chrome_51_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 51,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_51_android_desktop,
    },
    GenEntry {
        name: "chrome_52_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 52,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_52_android_desktop,
    },
    GenEntry {
        name: "chrome_48_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 48,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_48_android_desktop_2,
    },
    GenEntry {
        name: "chrome_54_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 54,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_54_android_desktop_2,
    },
    GenEntry {
        name: "chrome_51_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 51,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_51_android_desktop_2,
    },
    GenEntry {
        name: "chrome_50_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 50,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_50_android_desktop,
    },
    GenEntry {
        name: "chrome_55_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 55,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_55_android_desktop_2,
    },
    GenEntry {
        name: "chrome_52_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 52,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_52_android_desktop_2,
    },
    GenEntry {
        name: "chrome_47_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 47,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_47_android_desktop_2,
    },
    GenEntry {
        name: "chrome_51_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 51,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_51_android_desktop_3,
    },
    GenEntry {
        name: "chrome_55_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 55,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_55_android_desktop_3,
    },
    GenEntry {
        name: "chrome_53_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 53,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_53_android_desktop_2,
    },
    GenEntry {
        name: "chrome_51_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 51,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_51_android_desktop_4,
    },
    GenEntry {
        name: "chrome_49_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 49,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_49_android_desktop_2,
    },
    GenEntry {
        name: "chrome_53_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 53,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_53_android_desktop_3,
    },
    GenEntry {
        name: "chrome_53_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 53,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_53_android_desktop_4,
    },
    GenEntry {
        name: "chrome_50_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 50,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_50_android_desktop_2,
    },
    GenEntry {
        name: "chrome_49_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 49,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_49_android_desktop_3,
    },
    GenEntry {
        name: "chrome_123_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 123,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_123_android_desktop,
    },
    GenEntry {
        name: "chrome_126_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_126_android_desktop,
    },
    GenEntry {
        name: "chrome_122_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 122,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_122_android_desktop,
    },
    GenEntry {
        name: "chrome_129_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_129_android_desktop,
    },
    GenEntry {
        name: "chrome_125_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 125,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_125_android_desktop,
    },
    GenEntry {
        name: "chrome_124_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_124_android_desktop,
    },
    GenEntry {
        name: "chrome_115_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 115,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_115_android_desktop,
    },
    GenEntry {
        name: "chrome_116_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 116,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_116_android_desktop,
    },
    GenEntry {
        name: "chrome_117_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 117,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_117_android_desktop,
    },
    GenEntry {
        name: "chrome_118_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 118,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_118_android_desktop_2,
    },
    GenEntry {
        name: "edge_123_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 123,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_123_android_desktop,
    },
    GenEntry {
        name: "edge_125_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 125,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_125_android_desktop,
    },
    GenEntry {
        name: "chrome_125_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 125,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_125_android_tablet,
    },
    GenEntry {
        name: "edge_126_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_126_android_desktop,
    },
    GenEntry {
        name: "opera_83_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 83,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_83_android_desktop,
    },
    GenEntry {
        name: "chrome_126_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 126,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_126_android_tablet,
    },
    GenEntry {
        name: "edge_127_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_127_android_desktop,
    },
    GenEntry {
        name: "opera_84_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 84,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_84_android_desktop,
    },
    GenEntry {
        name: "chrome_127_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 127,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_127_android_tablet,
    },
    GenEntry {
        name: "chrome_129_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_129_android_tablet,
    },
    GenEntry {
        name: "chrome_130_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_130_android_desktop,
    },
    GenEntry {
        name: "edge_131_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 131,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_131_android_desktop,
    },
    GenEntry {
        name: "chrome_131_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 131,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_131_android_tablet,
    },
    GenEntry {
        name: "edge_132_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_132_android_desktop,
    },
    GenEntry {
        name: "opera_87_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 87,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_87_android_desktop,
    },
    GenEntry {
        name: "edge_133_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_133_android_desktop,
    },
    GenEntry {
        name: "edge_134_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_134_android_desktop,
    },
    GenEntry {
        name: "opera_88_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 88,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_88_android_desktop,
    },
    GenEntry {
        name: "samsung_25_android_desktop",
        browser: Browser::SamsungInternet,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 25,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: samsung_25_android_desktop,
    },
    GenEntry {
        name: "samsung_26_android_desktop",
        browser: Browser::SamsungInternet,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 26,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: samsung_26_android_desktop,
    },
    GenEntry {
        name: "samsung_27_android_desktop",
        browser: Browser::SamsungInternet,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 27,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: samsung_27_android_desktop,
    },
    GenEntry {
        name: "samsung_28_android_desktop",
        browser: Browser::SamsungInternet,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 28,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: samsung_28_android_desktop,
    },
    GenEntry {
        name: "chrome_131_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 131,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_131_android_desktop,
    },
    GenEntry {
        name: "chrome_120_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 120,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_120_android_desktop,
    },
    GenEntry {
        name: "chrome_134_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_134_android_desktop,
    },
    GenEntry {
        name: "chrome_122_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 122,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_122_android_desktop_2,
    },
    GenEntry {
        name: "chrome_133_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_133_android_desktop,
    },
    GenEntry {
        name: "chrome_134_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_134_android_desktop_2,
    },
    GenEntry {
        name: "chrome_130_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_130_android_desktop_2,
    },
    GenEntry {
        name: "chrome_124_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_124_android_desktop_2,
    },
    GenEntry {
        name: "chrome_129_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_129_android_desktop_2,
    },
    GenEntry {
        name: "chrome_131_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 131,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_131_android_desktop_2,
    },
    GenEntry {
        name: "chrome_128_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_128_android_desktop,
    },
    GenEntry {
        name: "chrome_131_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 131,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_131_android_desktop_3,
    },
    GenEntry {
        name: "chrome_129_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_129_android_desktop_3,
    },
    GenEntry {
        name: "chrome_120_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 120,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_120_android_desktop_2,
    },
    GenEntry {
        name: "opera_82_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 82,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_82_android_desktop,
    },
    GenEntry {
        name: "chrome_130_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_130_android_desktop_3,
    },
    GenEntry {
        name: "chrome_134_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_134_android_desktop_3,
    },
    GenEntry {
        name: "chrome_132_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_132_android_desktop,
    },
    GenEntry {
        name: "chrome_120_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 120,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_120_android_desktop_3,
    },
    GenEntry {
        name: "chrome_49_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 49,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_49_android_desktop_4,
    },
    GenEntry {
        name: "chrome_59_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 59,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_59_android_desktop,
    },
    GenEntry {
        name: "chrome_125_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 125,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_125_android_desktop_2,
    },
    GenEntry {
        name: "chrome_134_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_134_android_desktop_4,
    },
    GenEntry {
        name: "chrome_41_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 41,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_41_android_desktop,
    },
    GenEntry {
        name: "chrome_52_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 52,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_52_android_desktop_3,
    },
    GenEntry {
        name: "chrome_100_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 100,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_100_android_desktop,
    },
    GenEntry {
        name: "chrome_123_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 123,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_123_android_desktop_2,
    },
    GenEntry {
        name: "chrome_123_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 123,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_123_android_desktop_3,
    },
    GenEntry {
        name: "chrome_123_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 123,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_123_android_desktop_4,
    },
    GenEntry {
        name: "chrome_132_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_132_android_desktop_2,
    },
    GenEntry {
        name: "chrome_133_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_133_android_desktop_2,
    },
    GenEntry {
        name: "chrome_133_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_133_android_desktop_3,
    },
    GenEntry {
        name: "chrome_131_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 131,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_131_android_desktop_4,
    },
    GenEntry {
        name: "chrome_144_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 144,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_144_android_desktop,
    },
    GenEntry {
        name: "chrome_146_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 146,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_146_android_desktop,
    },
    GenEntry {
        name: "chrome_147_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 147,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_147_android_desktop,
    },
    GenEntry {
        name: "chrome_146_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 146,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_146_android_desktop_2,
    },
    GenEntry {
        name: "chrome_140_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_140_android_desktop,
    },
    GenEntry {
        name: "chrome_135_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 135,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_135_android_desktop,
    },
    GenEntry {
        name: "chrome_47_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 47,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_47_android_desktop_3,
    },
    GenEntry {
        name: "chrome_120_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 120,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_120_android_desktop_4,
    },
    GenEntry {
        name: "chrome_124_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_124_android_desktop_3,
    },
    GenEntry {
        name: "chrome_127_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_127_android_desktop,
    },
    GenEntry {
        name: "chrome_128_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_128_android_desktop_2,
    },
    GenEntry {
        name: "chrome_134_android_desktop_5",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_134_android_desktop_5,
    },
    GenEntry {
        name: "chrome_137_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 137,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_137_android_desktop,
    },
    GenEntry {
        name: "chrome_138_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 138,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_138_android_desktop,
    },
    GenEntry {
        name: "chrome_144_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 144,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_144_android_desktop_2,
    },
    GenEntry {
        name: "chrome_136_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 136,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_136_android_desktop,
    },
    GenEntry {
        name: "chrome_130_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 130,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_130_android_tablet,
    },
    GenEntry {
        name: "edge_135_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 135,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_135_android_desktop,
    },
    GenEntry {
        name: "opera_89_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 89,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_89_android_desktop,
    },
    GenEntry {
        name: "chrome_135_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 135,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_135_android_tablet,
    },
    GenEntry {
        name: "chrome_136_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 136,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_136_android_tablet,
    },
    GenEntry {
        name: "edge_137_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 137,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_137_android_desktop,
    },
    GenEntry {
        name: "opera_90_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 90,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_90_android_desktop,
    },
    GenEntry {
        name: "chrome_137_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 137,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_137_android_tablet,
    },
    GenEntry {
        name: "edge_138_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 138,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_138_android_desktop,
    },
    GenEntry {
        name: "chrome_138_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 138,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_138_android_tablet,
    },
    GenEntry {
        name: "chrome_139_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 139,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_139_android_desktop,
    },
    GenEntry {
        name: "edge_139_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 139,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_139_android_desktop,
    },
    GenEntry {
        name: "opera_91_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 91,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_91_android_desktop,
    },
    GenEntry {
        name: "chrome_139_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 139,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_139_android_tablet,
    },
    GenEntry {
        name: "chrome_140_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_140_android_desktop_2,
    },
    GenEntry {
        name: "edge_140_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_140_android_desktop,
    },
    GenEntry {
        name: "opera_92_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 92,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_92_android_desktop,
    },
    GenEntry {
        name: "chrome_141_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 141,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_141_android_desktop,
    },
    GenEntry {
        name: "edge_141_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 141,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_141_android_desktop,
    },
    GenEntry {
        name: "chrome_141_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 141,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_141_android_tablet,
    },
    GenEntry {
        name: "chrome_142_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 142,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_142_android_desktop,
    },
    GenEntry {
        name: "opera_93_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 93,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_93_android_desktop,
    },
    GenEntry {
        name: "chrome_142_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 142,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_142_android_tablet,
    },
    GenEntry {
        name: "chrome_143_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 143,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_143_android_desktop,
    },
    GenEntry {
        name: "chrome_143_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 143,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_143_android_tablet,
    },
    GenEntry {
        name: "chrome_144_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 144,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_144_android_tablet,
    },
    GenEntry {
        name: "edge_144_android_tablet",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 144,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_144_android_tablet,
    },
    GenEntry {
        name: "chrome_145_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 145,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_145_android_desktop,
    },
    GenEntry {
        name: "opera_96_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 96,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_96_android_desktop,
    },
    GenEntry {
        name: "chrome_145_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 145,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_145_android_tablet,
    },
    GenEntry {
        name: "edge_146_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 146,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_146_android_desktop,
    },
    GenEntry {
        name: "chrome_146_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 146,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_146_android_tablet,
    },
    GenEntry {
        name: "chrome_147_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 147,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_147_android_tablet,
    },
    GenEntry {
        name: "chrome_148_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 148,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_148_android_desktop,
    },
    GenEntry {
        name: "samsung_29_android_desktop",
        browser: Browser::SamsungInternet,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 29,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: samsung_29_android_desktop,
    },
    GenEntry {
        name: "chrome_145_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 145,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_145_android_desktop_2,
    },
    GenEntry {
        name: "chrome_140_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_140_android_desktop_3,
    },
    GenEntry {
        name: "chrome_142_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 142,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_142_android_desktop_2,
    },
    GenEntry {
        name: "chrome_144_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 144,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_144_android_desktop_3,
    },
    GenEntry {
        name: "chrome_136_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 136,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_136_android_desktop_2,
    },
    GenEntry {
        name: "chrome_122_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 122,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_122_android_desktop_3,
    },
    GenEntry {
        name: "chrome_136_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 136,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_136_android_desktop_3,
    },
    GenEntry {
        name: "chrome_132_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_132_android_desktop_3,
    },
    GenEntry {
        name: "chrome_133_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_133_android_desktop_4,
    },
    GenEntry {
        name: "chrome_142_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 142,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_142_android_desktop_3,
    },
    GenEntry {
        name: "chrome_143_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 143,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_143_android_desktop_2,
    },
    GenEntry {
        name: "chrome_141_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 141,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_141_android_desktop_2,
    },
    GenEntry {
        name: "chrome_137_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 137,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_137_android_desktop_2,
    },
    GenEntry {
        name: "chrome_138_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 138,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_138_android_desktop_2,
    },
    GenEntry {
        name: "chrome_143_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 143,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_143_android_desktop_3,
    },
    GenEntry {
        name: "chrome_146_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 146,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_146_android_desktop_3,
    },
    GenEntry {
        name: "chrome_140_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_140_android_desktop_4,
    },
    GenEntry {
        name: "chrome_145_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 145,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_145_android_desktop_3,
    },
    GenEntry {
        name: "chrome_142_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 142,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_142_android_desktop_4,
    },
    GenEntry {
        name: "chrome_142_android_desktop_5",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 142,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_142_android_desktop_5,
    },
    GenEntry {
        name: "chrome_143_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 143,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_143_android_desktop_4,
    },
    GenEntry {
        name: "chrome_146_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 146,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_146_android_desktop_4,
    },
    GenEntry {
        name: "chrome_138_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 138,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_138_android_desktop_3,
    },
    GenEntry {
        name: "chrome_145_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 145,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_145_android_desktop_4,
    },
    GenEntry {
        name: "opera_3_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 3,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_3_android_desktop,
    },
    GenEntry {
        name: "chrome_134_android_desktop_6",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_134_android_desktop_6,
    },
    GenEntry {
        name: "chrome_38_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 38,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_38_android_desktop,
    },
    GenEntry {
        name: "chrome_63_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 63,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_63_android_desktop,
    },
    GenEntry {
        name: "chrome_57_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 57,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_57_android_desktop,
    },
    GenEntry {
        name: "chrome_58_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 58,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_58_android_desktop,
    },
    GenEntry {
        name: "chrome_136_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 136,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_136_android_desktop_4,
    },
    GenEntry {
        name: "chrome_137_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 137,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_137_android_desktop_3,
    },
    GenEntry {
        name: "chrome_143_android_desktop_5",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 143,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_143_android_desktop_5,
    },
    GenEntry {
        name: "chrome_144_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 144,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_144_android_desktop_4,
    },
    GenEntry {
        name: "chrome_136_android_desktop_5",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 136,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_136_android_desktop_5,
    },
    GenEntry {
        name: "chrome_87_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 87,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_87_android_desktop_2,
    },
    GenEntry {
        name: "chrome_39_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 39,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_39_android_desktop,
    },
    GenEntry {
        name: "chrome_138_android_tablet_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 138,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_138_android_tablet_2,
    },
    GenEntry {
        name: "chrome_137_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 137,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_137_android_desktop_4,
    },
    GenEntry {
        name: "chrome_113_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 113,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_113_android_desktop,
    },
    GenEntry {
        name: "chrome_100_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 100,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_100_android_desktop_2,
    },
    GenEntry {
        name: "chrome_113_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 113,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_113_android_desktop_2,
    },
    GenEntry {
        name: "chrome_91_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 91,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_91_android_desktop,
    },
    GenEntry {
        name: "chrome_107_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 107,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_107_android_desktop,
    },
    GenEntry {
        name: "chrome_108_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 108,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_108_android_desktop,
    },
    GenEntry {
        name: "chrome_111_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 111,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_111_android_desktop,
    },
    GenEntry {
        name: "chrome_120_android_desktop_5",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 120,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_120_android_desktop_5,
    },
    GenEntry {
        name: "chrome_91_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 91,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_91_android_tablet,
    },
    GenEntry {
        name: "chrome_111_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 111,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_111_android_desktop_2,
    },
    GenEntry {
        name: "chrome_103_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 103,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_103_android_desktop,
    },
    GenEntry {
        name: "chrome_99_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 99,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_99_android_desktop,
    },
    GenEntry {
        name: "chrome_102_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 102,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_102_android_desktop,
    },
    GenEntry {
        name: "chrome_110_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 110,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_110_android_desktop,
    },
    GenEntry {
        name: "chrome_115_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 115,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_115_android_desktop_2,
    },
    GenEntry {
        name: "chrome_123_android_desktop_5",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 123,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_123_android_desktop_5,
    },
    GenEntry {
        name: "opera_64_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 64,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: opera_64_android_desktop,
    },
    GenEntry {
        name: "chrome_107_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 107,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_107_android_desktop_2,
    },
    GenEntry {
        name: "chrome_108_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 108,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_108_android_desktop_2,
    },
    GenEntry {
        name: "chrome_96_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 96,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_96_android_desktop,
    },
    GenEntry {
        name: "chrome_99_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 99,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_99_android_desktop_2,
    },
    GenEntry {
        name: "chrome_114_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 114,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_114_android_desktop_2,
    },
    GenEntry {
        name: "chrome_107_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 107,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_107_android_desktop_3,
    },
    GenEntry {
        name: "chrome_99_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 99,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_99_android_desktop_3,
    },
    GenEntry {
        name: "samsung_21_android_desktop",
        browser: Browser::SamsungInternet,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 21,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: samsung_21_android_desktop,
    },
    GenEntry {
        name: "chrome_68_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 68,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_68_android_desktop,
    },
    GenEntry {
        name: "chrome_100_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 100,
        ja4: "t13d1516ht_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_100_android_desktop_3,
    },
    GenEntry {
        name: "chrome_136_android_desktop_6",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 136,
        ja4: "t13d1517h2_8daaf6152771_46b8896bec77",
        spec_fn: chrome_136_android_desktop_6,
    },
    GenEntry {
        name: "chrome_126_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1517h2_8daaf6152771_b1ff8ab2d16f",
        spec_fn: chrome_126_android_desktop_2,
    },
    GenEntry {
        name: "chrome_124_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d1517h2_8daaf6152771_e903c750b005",
        spec_fn: chrome_124_android_desktop_4,
    },
    GenEntry {
        name: "chrome_52_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 52,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_52_android_desktop_4,
    },
    GenEntry {
        name: "chrome_78_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 78,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_78_android_desktop_2,
    },
    GenEntry {
        name: "chrome_63_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 63,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_63_android_desktop_2,
    },
    GenEntry {
        name: "chrome_116_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 116,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_116_android_desktop_2,
    },
    GenEntry {
        name: "chrome_101_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 101,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_101_android_desktop,
    },
    GenEntry {
        name: "samsung_17_android_desktop",
        browser: Browser::SamsungInternet,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 17,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: samsung_17_android_desktop,
    },
    GenEntry {
        name: "samsung_17_android_desktop_2",
        browser: Browser::SamsungInternet,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 17,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: samsung_17_android_desktop_2,
    },
    GenEntry {
        name: "chrome_42_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 42,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_42_android_tablet,
    },
    GenEntry {
        name: "chrome_71_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 71,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_71_android_tablet,
    },
    GenEntry {
        name: "chrome_55_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 55,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_55_android_tablet,
    },
    GenEntry {
        name: "chrome_45_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 45,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_45_android_desktop,
    },
    GenEntry {
        name: "chrome_74_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 74,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_74_android_desktop,
    },
    GenEntry {
        name: "chrome_76_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 76,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_76_android_desktop,
    },
    GenEntry {
        name: "chrome_76_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 76,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_76_android_desktop_2,
    },
    GenEntry {
        name: "chrome_56_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 56,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_56_android_desktop,
    },
    GenEntry {
        name: "chrome_74_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 74,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_74_android_desktop_2,
    },
    GenEntry {
        name: "chrome_58_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 58,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_58_android_desktop_2,
    },
    GenEntry {
        name: "chrome_72_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 72,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_72_android_tablet,
    },
    GenEntry {
        name: "chrome_75_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 75,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_75_android_desktop,
    },
    GenEntry {
        name: "chrome_76_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 76,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_76_android_desktop_3,
    },
    GenEntry {
        name: "chrome_71_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 71,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_71_android_desktop,
    },
    GenEntry {
        name: "chrome_76_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 76,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_76_android_desktop_4,
    },
    GenEntry {
        name: "chrome_72_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 72,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_72_android_desktop,
    },
    GenEntry {
        name: "samsung_9_android_desktop",
        browser: Browser::SamsungInternet,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 9,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: samsung_9_android_desktop,
    },
    GenEntry {
        name: "chrome_92_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 92,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_92_android_desktop,
    },
    GenEntry {
        name: "chrome_83_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 83,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_83_android_desktop,
    },
    GenEntry {
        name: "samsung_16_android_desktop",
        browser: Browser::SamsungInternet,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 16,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: samsung_16_android_desktop,
    },
    GenEntry {
        name: "chrome_128_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d1711h2_5b57614c22b0_e7c285222651",
        spec_fn: chrome_128_android_desktop_3,
    },
    GenEntry {
        name: "chrome_106_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 106,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: chrome_106_android_desktop,
    },
    GenEntry {
        name: "chrome_108_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 108,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: chrome_108_android_desktop_3,
    },
    GenEntry {
        name: "chrome_127_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: chrome_127_android_desktop_2,
    },
    GenEntry {
        name: "edge_128_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: edge_128_android_desktop,
    },
    GenEntry {
        name: "chrome_118_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 118,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: chrome_118_android_desktop_3,
    },
    GenEntry {
        name: "chrome_126_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: chrome_126_android_desktop_3,
    },
    GenEntry {
        name: "edge_104_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 104,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: edge_104_android_desktop,
    },
    GenEntry {
        name: "chrome_121_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 121,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: chrome_121_android_desktop,
    },
    GenEntry {
        name: "chrome_114_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 114,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: chrome_114_android_desktop_3,
    },
    GenEntry {
        name: "chrome_130_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d171300_5b57614c22b0_43ade6aba3df",
        spec_fn: chrome_130_android_desktop_4,
    },
    GenEntry {
        name: "chrome_129_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d171300_5b57614c22b0_43ade6aba3df",
        spec_fn: chrome_129_android_desktop_4,
    },
    GenEntry {
        name: "chrome_128_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d171300_5b57614c22b0_43ade6aba3df",
        spec_fn: chrome_128_android_desktop_4,
    },
    GenEntry {
        name: "chrome_59_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 59,
        ja4: "t13d171300_5b57614c22b0_43ade6aba3df",
        spec_fn: chrome_59_android_desktop_2,
    },
    GenEntry {
        name: "chrome_138_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 138,
        ja4: "t13d171300_5b57614c22b0_43ade6aba3df",
        spec_fn: chrome_138_android_desktop_4,
    },
    GenEntry {
        name: "chrome_127_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d171300_5b57614c22b0_43ade6aba3df",
        spec_fn: chrome_127_android_desktop_3,
    },
    GenEntry {
        name: "chrome_125_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 125,
        ja4: "t13d171300_5b57614c22b0_43ade6aba3df",
        spec_fn: chrome_125_android_desktop_3,
    },
    GenEntry {
        name: "chrome_126_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d171300_5b57614c22b0_43ade6aba3df",
        spec_fn: chrome_126_android_desktop_4,
    },
    GenEntry {
        name: "chrome_131_android_desktop_5",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 131,
        ja4: "t13d171300_5b57614c22b0_43ade6aba3df",
        spec_fn: chrome_131_android_desktop_5,
    },
    GenEntry {
        name: "chrome_132_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d171300_5b57614c22b0_43ade6aba3df",
        spec_fn: chrome_132_android_desktop_4,
    },
    GenEntry {
        name: "chrome_133_android_desktop_5",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d171300_5b57614c22b0_43ade6aba3df",
        spec_fn: chrome_133_android_desktop_5,
    },
    GenEntry {
        name: "chrome_135_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 135,
        ja4: "t13d171300_5b57614c22b0_43ade6aba3df",
        spec_fn: chrome_135_android_desktop_2,
    },
    GenEntry {
        name: "chrome_139_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 139,
        ja4: "t13d171300_5b57614c22b0_43ade6aba3df",
        spec_fn: chrome_139_android_desktop_2,
    },
    GenEntry {
        name: "chrome_140_android_desktop_5",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d171300_5b57614c22b0_43ade6aba3df",
        spec_fn: chrome_140_android_desktop_5,
    },
    GenEntry {
        name: "chrome_141_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 141,
        ja4: "t13d171300_5b57614c22b0_43ade6aba3df",
        spec_fn: chrome_141_android_desktop_3,
    },
    GenEntry {
        name: "chrome_116_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 116,
        ja4: "t13d201100_2b729b4bf6f3_36bf25f296df",
        spec_fn: chrome_116_android_desktop_3,
    },
    GenEntry {
        name: "chrome_116_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 116,
        ja4: "t13d201100_2b729b4bf6f3_36bf25f296df",
        spec_fn: chrome_116_android_desktop_4,
    },
    GenEntry {
        name: "chrome_117_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 117,
        ja4: "t13d201100_2b729b4bf6f3_36bf25f296df",
        spec_fn: chrome_117_android_desktop_2,
    },
    GenEntry {
        name: "chrome_78_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 78,
        ja4: "t13d301000_1d37bd780c83_1f22a2ca17c4",
        spec_fn: chrome_78_android_desktop_3,
    },
    GenEntry {
        name: "chrome_37_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 37,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_37_android_tablet,
    },
    GenEntry {
        name: "chrome_53_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 53,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_53_android_tablet,
    },
    GenEntry {
        name: "chrome_33_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 33,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_33_android_tablet,
    },
    GenEntry {
        name: "chrome_54_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 54,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_54_android_tablet,
    },
    GenEntry {
        name: "chrome_24_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 24,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_24_android_tablet,
    },
    GenEntry {
        name: "chrome_57_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 57,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_57_android_tablet,
    },
    GenEntry {
        name: "chrome_59_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 59,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_59_android_tablet,
    },
    GenEntry {
        name: "chrome_30_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 30,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_30_android_tablet,
    },
    GenEntry {
        name: "chrome_55_android_tablet_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 55,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_55_android_tablet_2,
    },
    GenEntry {
        name: "chrome_17_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 17,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_17_android_tablet,
    },
    GenEntry {
        name: "chrome_50_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 50,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_50_android_tablet,
    },
    GenEntry {
        name: "chrome_38_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 38,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_38_android_tablet,
    },
    GenEntry {
        name: "chrome_41_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 41,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_41_android_tablet,
    },
    GenEntry {
        name: "chrome_46_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 46,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_46_android_tablet,
    },
    GenEntry {
        name: "chrome_32_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 32,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_32_android_tablet,
    },
    GenEntry {
        name: "chrome_33_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 33,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_33_android_desktop,
    },
    GenEntry {
        name: "chrome_62_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 62,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_62_android_tablet,
    },
    GenEntry {
        name: "chrome_24_android_tablet_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 24,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_24_android_tablet_2,
    },
    GenEntry {
        name: "chrome_50_android_tablet_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 50,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_50_android_tablet_2,
    },
    GenEntry {
        name: "chrome_47_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 47,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_47_android_tablet,
    },
    GenEntry {
        name: "chrome_53_android_tablet_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 53,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_53_android_tablet_2,
    },
    GenEntry {
        name: "chrome_14_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 14,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_14_android_tablet,
    },
    GenEntry {
        name: "chrome_81_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 81,
        ja4: "t13d301100_1d37bd780c83_24695f2957a7",
        spec_fn: chrome_81_android_desktop,
    },
    GenEntry {
        name: "chrome_79_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 79,
        ja4: "t13d301100_1d37bd780c83_24695f2957a7",
        spec_fn: chrome_79_android_desktop,
    },
    GenEntry {
        name: "chrome_48_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 48,
        ja4: "t13d301100_1d37bd780c83_d41ae481755e",
        spec_fn: chrome_48_android_desktop_3,
    },
    GenEntry {
        name: "chrome_41_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 41,
        ja4: "t13d301100_1d37bd780c83_d41ae481755e",
        spec_fn: chrome_41_android_desktop_2,
    },
    GenEntry {
        name: "chrome_42_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 42,
        ja4: "t13d301100_1d37bd780c83_d41ae481755e",
        spec_fn: chrome_42_android_desktop,
    },
    GenEntry {
        name: "chrome_68_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 68,
        ja4: "t13d301100_1d37bd780c83_d41ae481755e",
        spec_fn: chrome_68_android_desktop_2,
    },
    GenEntry {
        name: "chrome_122_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 122,
        ja4: "t13d3012h2_1d37bd780c83_b26ce05bbdd6",
        spec_fn: chrome_122_android_desktop_4,
    },
    GenEntry {
        name: "edge_121_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 121,
        ja4: "t13d3012h2_1d37bd780c83_b26ce05bbdd6",
        spec_fn: edge_121_android_desktop,
    },
    GenEntry {
        name: "edge_113_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 113,
        ja4: "t13d3012h2_1d37bd780c83_b26ce05bbdd6",
        spec_fn: edge_113_android_desktop,
    },
    GenEntry {
        name: "chrome_80_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 80,
        ja4: "t13d3012ht_1d37bd780c83_8d633dac7124",
        spec_fn: chrome_80_android_desktop_2,
    },
    GenEntry {
        name: "chrome_93_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 93,
        ja4: "t13d3012ht_1d37bd780c83_b26ce05bbdd6",
        spec_fn: chrome_93_android_desktop,
    },
    GenEntry {
        name: "chrome_83_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 83,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_83_android_desktop_2,
    },
    GenEntry {
        name: "opera_57_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 57,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_57_android_desktop,
    },
    GenEntry {
        name: "chrome_81_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 81,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_81_android_desktop_2,
    },
    GenEntry {
        name: "samsung_11_android_desktop",
        browser: Browser::SamsungInternet,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 11,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: samsung_11_android_desktop,
    },
    GenEntry {
        name: "chrome_18_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 18,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_18_android_desktop,
    },
    GenEntry {
        name: "opera_50_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 50,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_50_android_desktop,
    },
    GenEntry {
        name: "chrome_81_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 81,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_81_android_desktop_3,
    },
    GenEntry {
        name: "chrome_47_android_tablet_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 47,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_47_android_tablet_2,
    },
    GenEntry {
        name: "chrome_81_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 81,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_81_android_desktop_4,
    },
    GenEntry {
        name: "chrome_68_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 68,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_68_android_desktop_3,
    },
    GenEntry {
        name: "chrome_72_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 72,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_72_android_desktop_2,
    },
    GenEntry {
        name: "chrome_80_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 80,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_80_android_desktop_3,
    },
    GenEntry {
        name: "chrome_81_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 81,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_81_android_tablet,
    },
    GenEntry {
        name: "opera_52_android_tablet",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 52,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_52_android_tablet,
    },
    GenEntry {
        name: "chrome_81_android_tablet_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 81,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_81_android_tablet_2,
    },
    GenEntry {
        name: "chrome_81_android_desktop_5",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 81,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_81_android_desktop_5,
    },
    GenEntry {
        name: "chrome_71_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 71,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_71_android_desktop_2,
    },
    GenEntry {
        name: "chrome_77_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 77,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_77_android_desktop_2,
    },
    GenEntry {
        name: "chrome_79_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 79,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_79_android_desktop_2,
    },
    GenEntry {
        name: "chrome_66_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 66,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_66_android_desktop,
    },
    GenEntry {
        name: "chrome_81_android_desktop_6",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 81,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_81_android_desktop_6,
    },
    GenEntry {
        name: "chrome_69_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 69,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_69_android_desktop,
    },
    GenEntry {
        name: "chrome_81_android_tablet_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 81,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_81_android_tablet_3,
    },
    GenEntry {
        name: "chrome_81_android_desktop_7",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 81,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_81_android_desktop_7,
    },
    GenEntry {
        name: "chrome_80_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 80,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_80_android_desktop_4,
    },
    GenEntry {
        name: "chrome_67_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 67,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_67_android_desktop,
    },
    GenEntry {
        name: "samsung_10_android_desktop",
        browser: Browser::SamsungInternet,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 10,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: samsung_10_android_desktop,
    },
    GenEntry {
        name: "chrome_79_android_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 79,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_79_android_desktop_3,
    },
    GenEntry {
        name: "chrome_81_android_desktop_8",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 81,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_81_android_desktop_8,
    },
    GenEntry {
        name: "chrome_80_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 80,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_80_android_tablet,
    },
    GenEntry {
        name: "chrome_83_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 83,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_83_android_tablet,
    },
    GenEntry {
        name: "edge_45_android_desktop",
        browser: Browser::Edge,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 45,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: edge_45_android_desktop,
    },
    GenEntry {
        name: "chrome_80_android_desktop_5",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 80,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_80_android_desktop_5,
    },
    GenEntry {
        name: "opera_57_android_desktop_2",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 57,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_57_android_desktop_2,
    },
    GenEntry {
        name: "samsung_11_android_desktop_2",
        browser: Browser::SamsungInternet,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 11,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: samsung_11_android_desktop_2,
    },
    GenEntry {
        name: "chrome_79_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 79,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_79_android_desktop_4,
    },
    GenEntry {
        name: "opera_47_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 47,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_47_android_desktop,
    },
    GenEntry {
        name: "opera_47_android_tablet",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 47,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_47_android_tablet,
    },
    GenEntry {
        name: "opera_47_android_desktop_2",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 47,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_47_android_desktop_2,
    },
    GenEntry {
        name: "opera_61_android_desktop",
        browser: Browser::Opera,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 61,
        ja4: "t13d3013h2_1d37bd780c83_ce5650b735ce",
        spec_fn: opera_61_android_desktop,
    },
    GenEntry {
        name: "chrome_132_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 132,
        ja4: "t13d351100_bfa337485184_53312b8d909f",
        spec_fn: chrome_132_android_tablet,
    },
    GenEntry {
        name: "chrome_89_android_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 89,
        ja4: "t13d361100_c014a34ff1af_fa269c3d986d",
        spec_fn: chrome_89_android_desktop_2,
    },
    GenEntry {
        name: "chrome_46_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 46,
        ja4: "t13d3613ht_bcee18a5b459_8537cf56674e",
        spec_fn: chrome_46_android_desktop,
    },
    GenEntry {
        name: "chrome_60_android_tablet",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 60,
        ja4: "t13d421000_49900ac2774e_1f22a2ca17c4",
        spec_fn: chrome_60_android_tablet,
    },
    GenEntry {
        name: "chrome_60_android_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 60,
        ja4: "t13d4212ht_49900ac2774e_b26ce05bbdd6",
        spec_fn: chrome_60_android_desktop,
    },
    GenEntry {
        name: "chrome_55_android_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 55,
        ja4: "t13d481000_c08b26b7ea02_5ac7197df9d2",
        spec_fn: chrome_55_android_desktop_4,
    },
];

// ja4=t13d121000_0ed44715e6cd_78e6aca7449b obs=23
#[rustfmt::skip]
spec! {
    chrome_77_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x009c, 0x002f,
             0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, keyshare[x25519],
          psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8714
#[rustfmt::skip]
spec! {
    chrome_90_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, status, sct,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    chrome_105_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, status, sct,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8728
#[rustfmt::skip]
spec! {
    chrome_114_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, status, sct,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8726
#[rustfmt::skip]
spec! {
    samsung_23_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, status, sct,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    chrome_80_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, status, sct,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8722
#[rustfmt::skip]
spec! {
    opera_80_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, status, sct,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d131100_f57a46bbacb6_ab7e3b40a677 obs=588
#[rustfmt::skip]
spec! {
    chrome_118_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, status, sct,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603],
          raw[0x0032, "00140804040308070805080604010501060105030603"],
}

// ja4=t13d131100_f57a46bbacb6_ab7e3b40a677 obs=588
#[rustfmt::skip]
spec! {
    chrome_45_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, status, sct,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603],
          raw[0x0032, "00140804040308070805080604010501060105030603"],
}

// ja4=t13d1311h2_f57a46bbacb6_e7c285222651 obs=436
#[rustfmt::skip]
spec! {
    chrome_73_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          status, sct, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=463
#[rustfmt::skip]
spec! {
    chrome_48_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          raw[0x0011, ""],
          raw[0x0032, "002a040305030603080708080804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d1515h2_8daaf6152771_45f260be83e2 obs=329
#[rustfmt::skip]
spec! {
    chrome_78_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], padding,
          compress[],
}

// ja4=t13d1515h2_8daaf6152771_de4a06bb82e3 obs=267
#[rustfmt::skip]
spec! {
    chrome_87_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[],
}

// ja4=t13d1515h2_8daaf6152771_de4a06bb82e3 obs=264
#[rustfmt::skip]
spec! {
    chrome_86_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[],
}

// ja4=t13d1515h2_8daaf6152771_de4a06bb82e3 obs=264
#[rustfmt::skip]
spec! {
    chrome_89_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[],
}

// ja4=t13d1515h2_8daaf6152771_de4a06bb82e3 obs=263
#[rustfmt::skip]
spec! {
    chrome_86_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_55_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_49_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_47_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1616
#[rustfmt::skip]
spec! {
    chrome_54_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_53_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_51_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1618
#[rustfmt::skip]
spec! {
    chrome_52_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_48_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_54_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_51_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_50_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_55_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1617
#[rustfmt::skip]
spec! {
    chrome_52_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_47_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_51_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_55_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1616
#[rustfmt::skip]
spec! {
    chrome_53_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_51_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1616
#[rustfmt::skip]
spec! {
    chrome_49_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1616
#[rustfmt::skip]
spec! {
    chrome_53_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_53_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_50_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_49_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, "0012040308040401050308050501080606010201"],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17176
#[rustfmt::skip]
spec! {
    chrome_123_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17231
#[rustfmt::skip]
spec! {
    chrome_126_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    chrome_122_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17213
#[rustfmt::skip]
spec! {
    chrome_129_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17432
#[rustfmt::skip]
spec! {
    chrome_125_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17175
#[rustfmt::skip]
spec! {
    chrome_124_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17175
#[rustfmt::skip]
spec! {
    chrome_115_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    chrome_116_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_117_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_118_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    edge_123_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17182
#[rustfmt::skip]
spec! {
    edge_125_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_125_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    edge_126_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    opera_83_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_126_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17176
#[rustfmt::skip]
spec! {
    edge_127_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    opera_84_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17175
#[rustfmt::skip]
spec! {
    chrome_127_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17175
#[rustfmt::skip]
spec! {
    chrome_129_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17229
#[rustfmt::skip]
spec! {
    chrome_130_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    edge_131_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_131_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17176
#[rustfmt::skip]
spec! {
    edge_132_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    opera_87_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    edge_133_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    edge_134_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    opera_88_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17188
#[rustfmt::skip]
spec! {
    samsung_25_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17177
#[rustfmt::skip]
spec! {
    samsung_26_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17182
#[rustfmt::skip]
spec! {
    samsung_27_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17177
#[rustfmt::skip]
spec! {
    samsung_28_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_131_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    chrome_120_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_134_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17248
#[rustfmt::skip]
spec! {
    chrome_122_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17177
#[rustfmt::skip]
spec! {
    chrome_133_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    chrome_134_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_130_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_124_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_129_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_131_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    chrome_128_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    chrome_131_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_129_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_120_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    opera_82_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17175
#[rustfmt::skip]
spec! {
    chrome_130_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_134_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_132_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_120_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17182
#[rustfmt::skip]
spec! {
    chrome_49_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_59_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17181
#[rustfmt::skip]
spec! {
    chrome_125_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17183
#[rustfmt::skip]
spec! {
    chrome_134_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_41_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_52_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_100_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_123_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_123_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    chrome_123_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35874
#[rustfmt::skip]
spec! {
    chrome_132_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35876
#[rustfmt::skip]
spec! {
    chrome_133_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35859
#[rustfmt::skip]
spec! {
    chrome_133_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35868
#[rustfmt::skip]
spec! {
    chrome_131_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35948
#[rustfmt::skip]
spec! {
    chrome_144_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35957
#[rustfmt::skip]
spec! {
    chrome_146_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35933
#[rustfmt::skip]
spec! {
    chrome_147_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_146_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_140_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35896
#[rustfmt::skip]
spec! {
    chrome_135_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_47_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35859
#[rustfmt::skip]
spec! {
    chrome_120_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_124_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_127_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_128_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35953
#[rustfmt::skip]
spec! {
    chrome_134_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35938
#[rustfmt::skip]
spec! {
    chrome_137_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35953
#[rustfmt::skip]
spec! {
    chrome_138_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_144_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35905
#[rustfmt::skip]
spec! {
    chrome_136_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_130_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35853
#[rustfmt::skip]
spec! {
    edge_135_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    opera_89_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_135_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35855
#[rustfmt::skip]
spec! {
    chrome_136_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_137_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    opera_90_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_137_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35855
#[rustfmt::skip]
spec! {
    edge_138_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35856
#[rustfmt::skip]
spec! {
    chrome_138_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35905
#[rustfmt::skip]
spec! {
    chrome_139_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_139_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    opera_91_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35856
#[rustfmt::skip]
spec! {
    chrome_139_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35976
#[rustfmt::skip]
spec! {
    chrome_140_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_140_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    opera_92_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35919
#[rustfmt::skip]
spec! {
    chrome_141_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    edge_141_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_141_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35957
#[rustfmt::skip]
spec! {
    chrome_142_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    opera_93_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_142_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36010
#[rustfmt::skip]
spec! {
    chrome_143_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35853
#[rustfmt::skip]
spec! {
    chrome_143_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_144_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    edge_144_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35951
#[rustfmt::skip]
spec! {
    chrome_145_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    opera_96_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_145_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    edge_146_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_146_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_147_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35861
#[rustfmt::skip]
spec! {
    chrome_148_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35866
#[rustfmt::skip]
spec! {
    samsung_29_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_145_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    chrome_140_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_142_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_144_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_136_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_122_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_136_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_132_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_133_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_142_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_143_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35856
#[rustfmt::skip]
spec! {
    chrome_141_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_137_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    chrome_138_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_143_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_146_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_140_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_145_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_142_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35856
#[rustfmt::skip]
spec! {
    chrome_142_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_143_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_146_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_138_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_145_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35853
#[rustfmt::skip]
spec! {
    opera_3_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_134_android_desktop_6,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_38_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_63_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_57_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_58_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_136_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_137_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_143_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    chrome_144_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_136_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_87_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_39_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35855
#[rustfmt::skip]
spec! {
    chrome_138_android_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_137_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2111
#[rustfmt::skip]
spec! {
    chrome_113_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2098
#[rustfmt::skip]
spec! {
    chrome_100_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2095
#[rustfmt::skip]
spec! {
    chrome_113_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2101
#[rustfmt::skip]
spec! {
    chrome_91_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    chrome_107_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2095
#[rustfmt::skip]
spec! {
    chrome_108_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2094
#[rustfmt::skip]
spec! {
    chrome_111_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    chrome_120_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    chrome_91_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    chrome_111_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2094
#[rustfmt::skip]
spec! {
    chrome_103_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    chrome_99_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    chrome_102_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    chrome_110_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    chrome_115_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    chrome_123_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    opera_64_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    chrome_107_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2094
#[rustfmt::skip]
spec! {
    chrome_108_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2094
#[rustfmt::skip]
spec! {
    chrome_96_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    chrome_99_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2095
#[rustfmt::skip]
spec! {
    chrome_114_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2094
#[rustfmt::skip]
spec! {
    chrome_107_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2133
#[rustfmt::skip]
spec! {
    chrome_99_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    samsung_21_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2096
#[rustfmt::skip]
spec! {
    chrome_68_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"],
}

// ja4=t13d1516ht_8daaf6152771_e5627efa2ab1 obs=48
#[rustfmt::skip]
spec! {
    chrome_100_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["http/1.1"],
}

// ja4=t13d1517h2_8daaf6152771_46b8896bec77 obs=7041
#[rustfmt::skip]
spec! {
    chrome_136_android_desktop_6,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          compress[], raw[0x0029, ""], raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1517h2_8daaf6152771_b1ff8ab2d16f obs=542
#[rustfmt::skip]
spec! {
    chrome_126_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], padding,
          compress[], appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1517h2_8daaf6152771_e903c750b005 obs=3652
#[rustfmt::skip]
spec! {
    chrome_124_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          compress[], raw[0x0029, ""], appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8607
#[rustfmt::skip]
spec! {
    chrome_52_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8966
#[rustfmt::skip]
spec! {
    chrome_78_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_63_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_116_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_101_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    samsung_17_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    samsung_17_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_42_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_71_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_55_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_45_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8607
#[rustfmt::skip]
spec! {
    chrome_74_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_76_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_76_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_56_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_74_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_58_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_72_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8608
#[rustfmt::skip]
spec! {
    chrome_75_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8609
#[rustfmt::skip]
spec! {
    chrome_76_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_71_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8623
#[rustfmt::skip]
spec! {
    chrome_76_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_72_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8607
#[rustfmt::skip]
spec! {
    samsung_9_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384], ecpf, status, sct, keyshare[x25519],
          versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_92_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, status, sct,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_83_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, status, sct,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    samsung_16_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, status, sct,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d1711h2_5b57614c22b0_e7c285222651 obs=182
#[rustfmt::skip]
spec! {
    chrome_128_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          status, sct, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    chrome_106_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["http/1.1"], keyshare[x25519],
          psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    chrome_108_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["http/1.1"], keyshare[x25519],
          psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    chrome_127_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["http/1.1"], keyshare[x25519],
          psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    edge_128_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["http/1.1"], keyshare[x25519],
          psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    chrome_118_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["http/1.1"], keyshare[x25519],
          psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    chrome_126_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["http/1.1"], keyshare[x25519],
          psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    edge_104_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["http/1.1"], keyshare[x25519],
          psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    chrome_121_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["http/1.1"], keyshare[x25519],
          psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    chrome_114_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["http/1.1"], keyshare[x25519],
          psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d171300_5b57614c22b0_43ade6aba3df obs=17936
#[rustfmt::skip]
spec! {
    chrome_130_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, status, sct,
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], padding,
}

// ja4=t13d171300_5b57614c22b0_43ade6aba3df obs=17932
#[rustfmt::skip]
spec! {
    chrome_129_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, status, sct,
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], padding,
}

// ja4=t13d171300_5b57614c22b0_43ade6aba3df obs=17931
#[rustfmt::skip]
spec! {
    chrome_128_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, status, sct,
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], padding,
}

// ja4=t13d171300_5b57614c22b0_43ade6aba3df obs=17992
#[rustfmt::skip]
spec! {
    chrome_59_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, status, sct,
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], padding,
}

// ja4=t13d171300_5b57614c22b0_43ade6aba3df obs=17930
#[rustfmt::skip]
spec! {
    chrome_138_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, status, sct,
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], padding,
}

// ja4=t13d171300_5b57614c22b0_43ade6aba3df obs=17935
#[rustfmt::skip]
spec! {
    chrome_127_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, status, sct,
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], padding,
}

// ja4=t13d171300_5b57614c22b0_43ade6aba3df obs=17941
#[rustfmt::skip]
spec! {
    chrome_125_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, status, sct,
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], padding,
}

// ja4=t13d171300_5b57614c22b0_43ade6aba3df obs=17938
#[rustfmt::skip]
spec! {
    chrome_126_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, status, sct,
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], padding,
}

// ja4=t13d171300_5b57614c22b0_43ade6aba3df obs=17938
#[rustfmt::skip]
spec! {
    chrome_131_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, status, sct,
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], padding,
}

// ja4=t13d171300_5b57614c22b0_43ade6aba3df obs=17934
#[rustfmt::skip]
spec! {
    chrome_132_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, status, sct,
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], padding,
}

// ja4=t13d171300_5b57614c22b0_43ade6aba3df obs=17945
#[rustfmt::skip]
spec! {
    chrome_133_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, status, sct,
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], padding,
}

// ja4=t13d171300_5b57614c22b0_43ade6aba3df obs=17934
#[rustfmt::skip]
spec! {
    chrome_135_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, status, sct,
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], padding,
}

// ja4=t13d171300_5b57614c22b0_43ade6aba3df obs=17935
#[rustfmt::skip]
spec! {
    chrome_139_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, status, sct,
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], padding,
}

// ja4=t13d171300_5b57614c22b0_43ade6aba3df obs=17931
#[rustfmt::skip]
spec! {
    chrome_140_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, status, sct,
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], padding,
}

// ja4=t13d171300_5b57614c22b0_43ade6aba3df obs=17931
#[rustfmt::skip]
spec! {
    chrome_141_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, status, sct,
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], padding,
}

// ja4=t13d201100_2b729b4bf6f3_36bf25f296df obs=270
#[rustfmt::skip]
spec! {
    chrome_116_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x009c, 0x009d, 0x002f,
             0x0035, 0x003c, 0x003d, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, keyshare[x25519],
          psk, versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0201, 0x0403, 0x0503, 0x0203, 0x0202, 0x0601, 0x0603],
          raw[0x0031, ""],
}

// ja4=t13d201100_2b729b4bf6f3_36bf25f296df obs=269
#[rustfmt::skip]
spec! {
    chrome_116_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x009c, 0x009d, 0x002f,
             0x0035, 0x003c, 0x003d, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, keyshare[x25519],
          psk, versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0201, 0x0403, 0x0503, 0x0203, 0x0202, 0x0601, 0x0603],
          raw[0x0031, ""],
}

// ja4=t13d201100_2b729b4bf6f3_36bf25f296df obs=270
#[rustfmt::skip]
spec! {
    chrome_117_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x009c, 0x009d, 0x002f,
             0x0035, 0x003c, 0x003d, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, ticket, keyshare[x25519],
          psk, versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0201, 0x0403, 0x0503, 0x0203, 0x0202, 0x0601, 0x0603],
          raw[0x0031, ""],
}

// ja4=t13d301000_1d37bd780c83_1f22a2ca17c4 obs=99
#[rustfmt::skip]
spec! {
    chrome_78_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, ticket, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_37_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_53_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_33_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_54_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_24_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_57_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_59_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_30_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_55_android_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_17_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_50_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_38_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_41_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1186
#[rustfmt::skip]
spec! {
    chrome_46_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_32_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_33_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_62_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_24_android_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_50_android_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_47_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_53_android_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_14_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301100_1d37bd780c83_24695f2957a7 obs=223
#[rustfmt::skip]
spec! {
    chrome_81_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, ticket, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301100_1d37bd780c83_24695f2957a7 obs=220
#[rustfmt::skip]
spec! {
    chrome_79_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, ticket, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301100_1d37bd780c83_d41ae481755e obs=5253
#[rustfmt::skip]
spec! {
    chrome_48_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, ticket, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301100_1d37bd780c83_d41ae481755e obs=5253
#[rustfmt::skip]
spec! {
    chrome_41_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, ticket, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301100_1d37bd780c83_d41ae481755e obs=5253
#[rustfmt::skip]
spec! {
    chrome_42_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, ticket, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301100_1d37bd780c83_d41ae481755e obs=5253
#[rustfmt::skip]
spec! {
    chrome_68_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, ticket, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d3012h2_1d37bd780c83_b26ce05bbdd6 obs=28317
#[rustfmt::skip]
spec! {
    chrome_122_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d3012h2_1d37bd780c83_b26ce05bbdd6 obs=28316
#[rustfmt::skip]
spec! {
    edge_121_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d3012h2_1d37bd780c83_b26ce05bbdd6 obs=28317
#[rustfmt::skip]
spec! {
    edge_113_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d3012ht_1d37bd780c83_8d633dac7124 obs=787
#[rustfmt::skip]
spec! {
    chrome_80_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["http/1.1"], keyshare[x25519],
          psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d3012ht_1d37bd780c83_b26ce05bbdd6 obs=396
#[rustfmt::skip]
spec! {
    chrome_93_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["http/1.1"], keyshare[x25519],
          psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_83_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4459
#[rustfmt::skip]
spec! {
    opera_57_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4469
#[rustfmt::skip]
spec! {
    chrome_81_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    samsung_11_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_18_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4458
#[rustfmt::skip]
spec! {
    opera_50_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4459
#[rustfmt::skip]
spec! {
    chrome_81_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_47_android_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4464
#[rustfmt::skip]
spec! {
    chrome_81_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_68_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_72_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4459
#[rustfmt::skip]
spec! {
    chrome_80_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_81_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    opera_52_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4461
#[rustfmt::skip]
spec! {
    chrome_81_android_tablet_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4466
#[rustfmt::skip]
spec! {
    chrome_81_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_71_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_77_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_79_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_66_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4473
#[rustfmt::skip]
spec! {
    chrome_81_android_desktop_6,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_69_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_81_android_tablet_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4477
#[rustfmt::skip]
spec! {
    chrome_81_android_desktop_7,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4459
#[rustfmt::skip]
spec! {
    chrome_80_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_67_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    samsung_10_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4458
#[rustfmt::skip]
spec! {
    chrome_79_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4489
#[rustfmt::skip]
spec! {
    chrome_81_android_desktop_8,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_80_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_83_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4458
#[rustfmt::skip]
spec! {
    edge_45_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_80_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    opera_57_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    samsung_11_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_79_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    opera_47_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4458
#[rustfmt::skip]
spec! {
    opera_47_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    opera_47_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_ce5650b735ce obs=1199
#[rustfmt::skip]
spec! {
    opera_61_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d351100_bfa337485184_53312b8d909f obs=61
#[rustfmt::skip]
spec! {
    chrome_132_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc027, 0xc09c, 0xc09d, 0xc09e, 0xc09f, 0xc0ac,
             0xc0ad, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301],
          padding, raw[0x0016, ""], compress[],
}

// ja4=t13d361100_c014a34ff1af_fa269c3d986d obs=213
#[rustfmt::skip]
spec! {
    chrome_89_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, status, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          raw[0x0011, ""],
          raw[0x0032, "00260403050306030804080508060809080a080b0401050106010402030303010302020302010202"],
}

// ja4=t13d3613ht_bcee18a5b459_8537cf56674e obs=2
#[rustfmt::skip]
spec! {
    chrome_46_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0ac, 0xc0ad, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384], ecpf, alpn["http/1.1"],
          keyshare[x25519], psk, versions[0x0304, 0x0303],
          sigalgs[0x0905, 0x0906, 0x0904, 0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x081a, 0x081b, 0x081c, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          raw[0x0016, ""], compress[], raw[0x0031, ""],
}

// ja4=t13d421000_49900ac2774e_1f22a2ca17c4 obs=45
#[rustfmt::skip]
spec! {
    chrome_60_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, ticket, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          raw[0x0016, ""],
}

// ja4=t13d4212ht_49900ac2774e_b26ce05bbdd6 obs=864
#[rustfmt::skip]
spec! {
    chrome_60_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, alpn["http/1.1"], keyshare[x25519],
          psk, versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d481000_c08b26b7ea02_5ac7197df9d2 obs=92
#[rustfmt::skip]
spec! {
    chrome_55_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac,
             0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384], ecpf, ticket, keyshare[x25519], psk,
          versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          raw[0x0016, ""],
}
