//! Chromium family desktop hellos (Chrome, Edge, Opera, Brave, Samsung Internet; the `chrome_desktop` wire template)
//!
//! Emitter output (`gen_specs.py --emit`); do not edit by hand.
//! Regeneration is byte-deterministic (`--selftest` verifies the
//! committed files match a fresh render).

use super::GenEntry;
use crate::fingerprints::{Browser, Device, Os};

#[rustfmt::skip]
pub const GENERATED: &[GenEntry] = &[
    GenEntry {
        name: "edge_79_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 79,
        ja4: "t13d101100_01be160bb49b_36bf25f296df",
        spec_fn: edge_79_windows_desktop,
    },
    GenEntry {
        name: "chrome_40_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 40,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_40_windows_desktop,
    },
    GenEntry {
        name: "chrome_70_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 70,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_70_windows_desktop,
    },
    GenEntry {
        name: "chrome_41_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 41,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_41_windows_desktop,
    },
    GenEntry {
        name: "chrome_41_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 41,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_41_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_37_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 37,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_37_macos_desktop,
    },
    GenEntry {
        name: "chrome_40_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 40,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_40_macos_desktop,
    },
    GenEntry {
        name: "chrome_41_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 41,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_41_macos_desktop,
    },
    GenEntry {
        name: "chrome_64_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 64,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_64_macos_desktop,
    },
    GenEntry {
        name: "opera_96_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 96,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: opera_96_macos_desktop,
    },
    GenEntry {
        name: "opera_97_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 97,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: opera_97_macos_desktop,
    },
    GenEntry {
        name: "chrome_112_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 112,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_112_macos_desktop,
    },
    GenEntry {
        name: "edge_104_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 104,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: edge_104_macos_desktop,
    },
    GenEntry {
        name: "chrome_105_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 105,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_105_macos_desktop,
    },
    GenEntry {
        name: "chrome_17_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 17,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_17_macos_desktop,
    },
    GenEntry {
        name: "opera_98_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 98,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: opera_98_macos_desktop,
    },
    GenEntry {
        name: "chrome_111_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 111,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_111_macos_desktop,
    },
    GenEntry {
        name: "chrome_112_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 112,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_112_macos_desktop_2,
    },
    GenEntry {
        name: "opera_96_macos_desktop_2",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 96,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: opera_96_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_115_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 115,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_115_macos_desktop,
    },
    GenEntry {
        name: "edge_111_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 111,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: edge_111_macos_desktop,
    },
    GenEntry {
        name: "edge_112_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 112,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: edge_112_macos_desktop,
    },
    GenEntry {
        name: "opera_97_macos_desktop_2",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 97,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: opera_97_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_107_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 107,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_107_macos_desktop,
    },
    GenEntry {
        name: "edge_108_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 108,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: edge_108_macos_desktop,
    },
    GenEntry {
        name: "edge_110_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 110,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: edge_110_macos_desktop,
    },
    GenEntry {
        name: "chrome_107_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 107,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_107_macos_desktop_2,
    },
    GenEntry {
        name: "opera_96_macos_desktop_3",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 96,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: opera_96_macos_desktop_3,
    },
    GenEntry {
        name: "chrome_108_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 108,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_108_macos_desktop,
    },
    GenEntry {
        name: "chrome_111_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 111,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_111_macos_desktop_2,
    },
    GenEntry {
        name: "edge_105_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 105,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: edge_105_macos_desktop,
    },
    GenEntry {
        name: "edge_106_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 106,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: edge_106_macos_desktop,
    },
    GenEntry {
        name: "edge_107_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 107,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: edge_107_macos_desktop,
    },
    GenEntry {
        name: "chrome_111_macos_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 111,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_111_macos_desktop_3,
    },
    GenEntry {
        name: "chrome_105_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 105,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_105_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_37_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 37,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_37_windows_desktop,
    },
    GenEntry {
        name: "chrome_64_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 64,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_64_windows_desktop,
    },
    GenEntry {
        name: "chrome_65_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 65,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_65_windows_desktop,
    },
    GenEntry {
        name: "chrome_110_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 110,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_110_windows_desktop,
    },
    GenEntry {
        name: "chrome_113_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 113,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_113_windows_desktop,
    },
    GenEntry {
        name: "chrome_58_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 58,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_58_windows_desktop,
    },
    GenEntry {
        name: "chrome_42_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 42,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_42_windows_desktop,
    },
    GenEntry {
        name: "chrome_78_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 78,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_78_windows_desktop,
    },
    GenEntry {
        name: "chrome_126_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_126_windows_desktop,
    },
    GenEntry {
        name: "chrome_40_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 40,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_40_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_64_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 64,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_64_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_65_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 65,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_65_windows_desktop_2,
    },
    GenEntry {
        name: "opera_97_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 97,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: opera_97_windows_desktop,
    },
    GenEntry {
        name: "chrome_45_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 45,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_45_windows_desktop,
    },
    GenEntry {
        name: "chrome_44_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 44,
        ja4: "t13d131100_f57a46bbacb6_ab7e3b40a677",
        spec_fn: chrome_44_windows_desktop,
    },
    GenEntry {
        name: "brave_126_windows_desktop",
        browser: Browser::Brave,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d131100_f57a46bbacb6_e5728521abd4",
        spec_fn: brave_126_windows_desktop,
    },
    GenEntry {
        name: "edge_121_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 121,
        ja4: "t13d1311h2_f57a46bbacb6_e7c285222651",
        spec_fn: edge_121_macos_desktop,
    },
    GenEntry {
        name: "edge_103_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 103,
        ja4: "t13d1311h2_f57a46bbacb6_e7c285222651",
        spec_fn: edge_103_macos_desktop,
    },
    GenEntry {
        name: "chrome_108_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 108,
        ja4: "t13d1311h2_f57a46bbacb6_e7c285222651",
        spec_fn: chrome_108_macos_desktop_2,
    },
    GenEntry {
        name: "opera_109_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 109,
        ja4: "t13d1311h2_f57a46bbacb6_e7c285222651",
        spec_fn: opera_109_macos_desktop,
    },
    GenEntry {
        name: "chrome_149_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 149,
        ja4: "t13d1511h2_8daaf6152771_6d021c4c45cd",
        spec_fn: chrome_149_windows_desktop,
    },
    GenEntry {
        name: "chrome_150_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 150,
        ja4: "t13d1511h2_8daaf6152771_6d021c4c45cd",
        spec_fn: chrome_150_windows_desktop,
    },
    GenEntry {
        name: "chrome_126_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1513h2_8daaf6152771_9249cab70c77",
        spec_fn: chrome_126_macos_desktop,
    },
    GenEntry {
        name: "edge_126_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1513h2_8daaf6152771_9249cab70c77",
        spec_fn: edge_126_macos_desktop,
    },
    GenEntry {
        name: "chrome_52_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 52,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_52_macos_desktop,
    },
    GenEntry {
        name: "chrome_49_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 49,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_49_macos_desktop,
    },
    GenEntry {
        name: "chrome_50_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 50,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_50_macos_desktop,
    },
    GenEntry {
        name: "chrome_50_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 50,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_50_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_54_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 54,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_54_windows_desktop,
    },
    GenEntry {
        name: "edge_15_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 15,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: edge_15_windows_desktop,
    },
    GenEntry {
        name: "chrome_53_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 53,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_53_windows_desktop,
    },
    GenEntry {
        name: "chrome_53_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 53,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_53_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_48_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 48,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: chrome_48_windows_desktop,
    },
    GenEntry {
        name: "edge_9_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 9,
        ja4: "t13d1515ht_8daaf6152771_4769d65a485e",
        spec_fn: edge_9_windows_desktop,
    },
    GenEntry {
        name: "chrome_47_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 47,
        ja4: "t13d1515ht_8daaf6152771_4769d65a485e",
        spec_fn: chrome_47_windows_desktop,
    },
    GenEntry {
        name: "chrome_55_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 55,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_55_macos_desktop,
    },
    GenEntry {
        name: "chrome_47_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 47,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_47_macos_desktop,
    },
    GenEntry {
        name: "chrome_54_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 54,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_54_macos_desktop,
    },
    GenEntry {
        name: "chrome_53_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 53,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_53_macos_desktop,
    },
    GenEntry {
        name: "chrome_55_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 55,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_55_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_50_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 50,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_50_windows_desktop,
    },
    GenEntry {
        name: "edge_16_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 16,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: edge_16_windows_desktop,
    },
    GenEntry {
        name: "chrome_48_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 48,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_48_macos_desktop,
    },
    GenEntry {
        name: "chrome_51_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 51,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_51_macos_desktop,
    },
    GenEntry {
        name: "chrome_48_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 48,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_48_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_51_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 51,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_51_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_53_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 53,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_53_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_54_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 54,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_54_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_55_macos_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 55,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_55_macos_desktop_3,
    },
    GenEntry {
        name: "chrome_48_macos_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 48,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_48_macos_desktop_3,
    },
    GenEntry {
        name: "chrome_49_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 49,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_49_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_52_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 52,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_52_windows_desktop,
    },
    GenEntry {
        name: "edge_17_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 17,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: edge_17_windows_desktop,
    },
    GenEntry {
        name: "edge_15_windows_desktop_2",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 15,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: edge_15_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_55_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 55,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_55_windows_desktop,
    },
    GenEntry {
        name: "edge_14_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 14,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: edge_14_windows_desktop,
    },
    GenEntry {
        name: "chrome_52_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 52,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: chrome_52_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_116_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 116,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_116_macos_desktop,
    },
    GenEntry {
        name: "chrome_116_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 116,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_116_windows_desktop,
    },
    GenEntry {
        name: "chrome_113_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 113,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_113_macos_desktop,
    },
    GenEntry {
        name: "opera_117_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 117,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_117_windows_desktop,
    },
    GenEntry {
        name: "chrome_114_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 114,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_114_windows_desktop,
    },
    GenEntry {
        name: "chrome_126_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_126_windows_desktop_2,
    },
    GenEntry {
        name: "edge_128_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_128_windows_desktop,
    },
    GenEntry {
        name: "chrome_111_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 111,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_111_windows_desktop,
    },
    GenEntry {
        name: "edge_125_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 125,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_125_windows_desktop,
    },
    GenEntry {
        name: "edge_105_macos_desktop_2",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 105,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_105_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_119_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 119,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_119_macos_desktop,
    },
    GenEntry {
        name: "chrome_122_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 122,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_122_macos_desktop,
    },
    GenEntry {
        name: "chrome_103_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 103,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_103_windows_desktop,
    },
    GenEntry {
        name: "chrome_104_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 104,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_104_windows_desktop,
    },
    GenEntry {
        name: "chrome_105_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 105,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_105_windows_desktop,
    },
    GenEntry {
        name: "chrome_106_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 106,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_106_windows_desktop,
    },
    GenEntry {
        name: "chrome_117_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 117,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_117_windows_desktop,
    },
    GenEntry {
        name: "edge_119_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 119,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_119_windows_desktop,
    },
    GenEntry {
        name: "edge_122_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 122,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_122_windows_desktop,
    },
    GenEntry {
        name: "opera_111_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 111,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_111_windows_desktop,
    },
    GenEntry {
        name: "chrome_37_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 37,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_37_windows_desktop_2,
    },
    GenEntry {
        name: "edge_124_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_124_windows_desktop,
    },
    GenEntry {
        name: "chrome_87_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 87,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_87_macos_desktop,
    },
    GenEntry {
        name: "chrome_110_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 110,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_110_macos_desktop,
    },
    GenEntry {
        name: "opera_109_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 109,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_109_windows_desktop,
    },
    GenEntry {
        name: "edge_127_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_127_windows_desktop,
    },
    GenEntry {
        name: "edge_127_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_127_macos_desktop,
    },
    GenEntry {
        name: "edge_132_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_132_macos_desktop,
    },
    GenEntry {
        name: "chrome_118_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 118,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_118_windows_desktop,
    },
    GenEntry {
        name: "opera_118_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 118,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_118_windows_desktop,
    },
    GenEntry {
        name: "opera_119_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 119,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_119_windows_desktop,
    },
    GenEntry {
        name: "chrome_115_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 115,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_115_macos_desktop_2,
    },
    GenEntry {
        name: "edge_113_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 113,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_113_windows_desktop,
    },
    GenEntry {
        name: "edge_116_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 116,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_116_windows_desktop,
    },
    GenEntry {
        name: "chrome_101_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 101,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_101_windows_desktop,
    },
    GenEntry {
        name: "chrome_106_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 106,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_106_macos_desktop,
    },
    GenEntry {
        name: "chrome_79_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 79,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_79_macos_desktop,
    },
    GenEntry {
        name: "chrome_104_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 104,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_104_macos_desktop,
    },
    GenEntry {
        name: "chrome_109_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 109,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_109_macos_desktop,
    },
    GenEntry {
        name: "edge_124_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_124_macos_desktop,
    },
    GenEntry {
        name: "opera_110_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 110,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_110_macos_desktop,
    },
    GenEntry {
        name: "edge_125_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 125,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_125_macos_desktop,
    },
    GenEntry {
        name: "edge_126_macos_desktop_2",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_126_macos_desktop_2,
    },
    GenEntry {
        name: "opera_112_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 112,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_112_macos_desktop,
    },
    GenEntry {
        name: "opera_113_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 113,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_113_macos_desktop,
    },
    GenEntry {
        name: "opera_114_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 114,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_114_macos_desktop,
    },
    GenEntry {
        name: "opera_115_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 115,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_115_macos_desktop,
    },
    GenEntry {
        name: "opera_116_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 116,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_116_macos_desktop,
    },
    GenEntry {
        name: "opera_117_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 117,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_117_macos_desktop,
    },
    GenEntry {
        name: "opera_118_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 118,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_118_macos_desktop,
    },
    GenEntry {
        name: "opera_119_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 119,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_119_macos_desktop,
    },
    GenEntry {
        name: "chrome_92_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 92,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_92_macos_desktop,
    },
    GenEntry {
        name: "chrome_98_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 98,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_98_macos_desktop,
    },
    GenEntry {
        name: "edge_106_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 106,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_106_windows_desktop,
    },
    GenEntry {
        name: "edge_107_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 107,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_107_windows_desktop,
    },
    GenEntry {
        name: "edge_110_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 110,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_110_windows_desktop,
    },
    GenEntry {
        name: "edge_111_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 111,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_111_windows_desktop,
    },
    GenEntry {
        name: "opera_98_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 98,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_98_windows_desktop,
    },
    GenEntry {
        name: "edge_123_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 123,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_123_windows_desktop,
    },
    GenEntry {
        name: "opera_110_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 110,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_110_windows_desktop,
    },
    GenEntry {
        name: "opera_112_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 112,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_112_windows_desktop,
    },
    GenEntry {
        name: "edge_117_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 117,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_117_windows_desktop,
    },
    GenEntry {
        name: "opera_113_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 113,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_113_windows_desktop,
    },
    GenEntry {
        name: "opera_128_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_128_windows_desktop,
    },
    GenEntry {
        name: "chrome_96_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 96,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_96_windows_desktop,
    },
    GenEntry {
        name: "edge_108_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 108,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_108_windows_desktop,
    },
    GenEntry {
        name: "opera_95_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 95,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: opera_95_windows_desktop,
    },
    GenEntry {
        name: "chrome_132_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_132_windows_desktop,
    },
    GenEntry {
        name: "chrome_102_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 102,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_102_windows_desktop,
    },
    GenEntry {
        name: "chrome_106_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 106,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_106_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_107_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 107,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_107_windows_desktop,
    },
    GenEntry {
        name: "chrome_114_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 114,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_114_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_127_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_127_windows_desktop,
    },
    GenEntry {
        name: "chrome_101_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 101,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_101_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_108_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 108,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_108_windows_desktop,
    },
    GenEntry {
        name: "chrome_83_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 83,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_83_windows_desktop,
    },
    GenEntry {
        name: "chrome_143_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 143,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_143_windows_desktop,
    },
    GenEntry {
        name: "chrome_83_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 83,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_83_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_106_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 106,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_106_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_103_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 103,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: chrome_103_windows_desktop_2,
    },
    GenEntry {
        name: "edge_85_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 85,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: edge_85_windows_desktop,
    },
    GenEntry {
        name: "chrome_146_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 146,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_146_macos_desktop,
    },
    GenEntry {
        name: "chrome_137_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 137,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_137_windows_desktop,
    },
    GenEntry {
        name: "chrome_138_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 138,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_138_windows_desktop,
    },
    GenEntry {
        name: "chrome_131_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 131,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_131_windows_desktop,
    },
    GenEntry {
        name: "chrome_135_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 135,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_135_windows_desktop,
    },
    GenEntry {
        name: "chrome_136_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 136,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_136_windows_desktop,
    },
    GenEntry {
        name: "chrome_146_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 146,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_146_windows_desktop,
    },
    GenEntry {
        name: "chrome_147_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 147,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_147_windows_desktop,
    },
    GenEntry {
        name: "chrome_91_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 91,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_91_windows_desktop,
    },
    GenEntry {
        name: "edge_134_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_134_windows_desktop,
    },
    GenEntry {
        name: "edge_136_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 136,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_136_windows_desktop,
    },
    GenEntry {
        name: "edge_137_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 137,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_137_windows_desktop,
    },
    GenEntry {
        name: "edge_138_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 138,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_138_windows_desktop,
    },
    GenEntry {
        name: "chrome_139_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 139,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_139_macos_desktop,
    },
    GenEntry {
        name: "chrome_143_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 143,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_143_macos_desktop,
    },
    GenEntry {
        name: "chrome_131_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 131,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_131_macos_desktop,
    },
    GenEntry {
        name: "chrome_142_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 142,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_142_macos_desktop,
    },
    GenEntry {
        name: "edge_130_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_130_windows_desktop,
    },
    GenEntry {
        name: "edge_141_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 141,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_141_windows_desktop,
    },
    GenEntry {
        name: "chrome_145_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 145,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_145_windows_desktop,
    },
    GenEntry {
        name: "chrome_125_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 125,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_125_macos_desktop,
    },
    GenEntry {
        name: "chrome_147_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 147,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_147_macos_desktop,
    },
    GenEntry {
        name: "edge_140_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_140_windows_desktop,
    },
    GenEntry {
        name: "edge_146_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 146,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_146_windows_desktop,
    },
    GenEntry {
        name: "chrome_134_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_134_windows_desktop,
    },
    GenEntry {
        name: "chrome_133_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_133_windows_desktop,
    },
    GenEntry {
        name: "chrome_135_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 135,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_135_macos_desktop,
    },
    GenEntry {
        name: "chrome_140_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_140_macos_desktop,
    },
    GenEntry {
        name: "chrome_144_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 144,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_144_macos_desktop,
    },
    GenEntry {
        name: "edge_139_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 139,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_139_windows_desktop,
    },
    GenEntry {
        name: "chrome_142_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 142,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_142_windows_desktop,
    },
    GenEntry {
        name: "edge_142_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 142,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_142_windows_desktop,
    },
    GenEntry {
        name: "chrome_143_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 143,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_143_windows_desktop_2,
    },
    GenEntry {
        name: "edge_143_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 143,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_143_windows_desktop,
    },
    GenEntry {
        name: "edge_144_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 144,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_144_windows_desktop,
    },
    GenEntry {
        name: "chrome_137_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 137,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_137_macos_desktop,
    },
    GenEntry {
        name: "chrome_127_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_127_macos_desktop,
    },
    GenEntry {
        name: "chrome_125_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 125,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_125_windows_desktop,
    },
    GenEntry {
        name: "chrome_107_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 107,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_107_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_115_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 115,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_115_windows_desktop,
    },
    GenEntry {
        name: "chrome_120_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 120,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_120_macos_desktop,
    },
    GenEntry {
        name: "chrome_141_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 141,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_141_windows_desktop,
    },
    GenEntry {
        name: "chrome_129_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_129_windows_desktop,
    },
    GenEntry {
        name: "chrome_132_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_132_macos_desktop,
    },
    GenEntry {
        name: "edge_126_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_126_windows_desktop,
    },
    GenEntry {
        name: "chrome_120_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 120,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_120_windows_desktop,
    },
    GenEntry {
        name: "chrome_128_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_128_windows_desktop,
    },
    GenEntry {
        name: "chrome_148_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 148,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_148_windows_desktop,
    },
    GenEntry {
        name: "chrome_134_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_134_macos_desktop,
    },
    GenEntry {
        name: "edge_147_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 147,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_147_windows_desktop,
    },
    GenEntry {
        name: "chrome_69_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 69,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_69_macos_desktop,
    },
    GenEntry {
        name: "chrome_108_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 108,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_108_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_144_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 144,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_144_windows_desktop,
    },
    GenEntry {
        name: "edge_129_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_129_windows_desktop,
    },
    GenEntry {
        name: "chrome_130_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_130_windows_desktop,
    },
    GenEntry {
        name: "edge_131_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 131,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_131_windows_desktop,
    },
    GenEntry {
        name: "chrome_132_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_132_windows_desktop_2,
    },
    GenEntry {
        name: "edge_135_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 135,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_135_windows_desktop,
    },
    GenEntry {
        name: "chrome_126_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_126_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_128_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_128_macos_desktop,
    },
    GenEntry {
        name: "chrome_129_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_129_macos_desktop,
    },
    GenEntry {
        name: "chrome_130_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_130_macos_desktop,
    },
    GenEntry {
        name: "chrome_133_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_133_macos_desktop,
    },
    GenEntry {
        name: "chrome_136_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 136,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_136_macos_desktop,
    },
    GenEntry {
        name: "chrome_138_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 138,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_138_macos_desktop,
    },
    GenEntry {
        name: "edge_140_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_140_macos_desktop,
    },
    GenEntry {
        name: "chrome_141_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 141,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_141_macos_desktop,
    },
    GenEntry {
        name: "chrome_145_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 145,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_145_macos_desktop,
    },
    GenEntry {
        name: "chrome_127_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_127_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_139_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 139,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_139_windows_desktop,
    },
    GenEntry {
        name: "edge_145_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 145,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_145_windows_desktop,
    },
    GenEntry {
        name: "edge_132_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_132_windows_desktop,
    },
    GenEntry {
        name: "edge_120_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 120,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_120_macos_desktop,
    },
    GenEntry {
        name: "chrome_123_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 123,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_123_macos_desktop,
    },
    GenEntry {
        name: "edge_136_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 136,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_136_macos_desktop,
    },
    GenEntry {
        name: "chrome_100_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 100,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_100_windows_desktop,
    },
    GenEntry {
        name: "chrome_112_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 112,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_112_windows_desktop,
    },
    GenEntry {
        name: "chrome_122_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 122,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_122_windows_desktop,
    },
    GenEntry {
        name: "chrome_123_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 123,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_123_windows_desktop,
    },
    GenEntry {
        name: "chrome_124_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_124_windows_desktop,
    },
    GenEntry {
        name: "edge_133_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_133_windows_desktop,
    },
    GenEntry {
        name: "chrome_140_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_140_windows_desktop,
    },
    GenEntry {
        name: "edge_134_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_134_macos_desktop,
    },
    GenEntry {
        name: "edge_135_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 135,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_135_macos_desktop,
    },
    GenEntry {
        name: "edge_146_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 146,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_146_macos_desktop,
    },
    GenEntry {
        name: "edge_147_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 147,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_147_macos_desktop,
    },
    GenEntry {
        name: "chrome_148_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 148,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_148_macos_desktop,
    },
    GenEntry {
        name: "chrome_114_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 114,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_114_macos_desktop,
    },
    GenEntry {
        name: "chrome_121_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 121,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_121_macos_desktop,
    },
    GenEntry {
        name: "edge_133_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_133_macos_desktop,
    },
    GenEntry {
        name: "chrome_124_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_124_macos_desktop,
    },
    GenEntry {
        name: "opera_120_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 120,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_120_windows_desktop,
    },
    GenEntry {
        name: "opera_122_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 122,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_122_windows_desktop,
    },
    GenEntry {
        name: "opera_123_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 123,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_123_windows_desktop,
    },
    GenEntry {
        name: "opera_124_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_124_windows_desktop,
    },
    GenEntry {
        name: "opera_127_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_127_windows_desktop,
    },
    GenEntry {
        name: "edge_148_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 148,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_148_windows_desktop,
    },
    GenEntry {
        name: "edge_129_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_129_macos_desktop,
    },
    GenEntry {
        name: "edge_130_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_130_macos_desktop,
    },
    GenEntry {
        name: "edge_137_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 137,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_137_macos_desktop,
    },
    GenEntry {
        name: "edge_138_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 138,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_138_macos_desktop,
    },
    GenEntry {
        name: "edge_141_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 141,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_141_macos_desktop,
    },
    GenEntry {
        name: "edge_142_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 142,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_142_macos_desktop,
    },
    GenEntry {
        name: "edge_143_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 143,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_143_macos_desktop,
    },
    GenEntry {
        name: "edge_128_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_128_macos_desktop,
    },
    GenEntry {
        name: "opera_115_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 115,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_115_windows_desktop,
    },
    GenEntry {
        name: "opera_114_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 114,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_114_windows_desktop,
    },
    GenEntry {
        name: "opera_129_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_129_windows_desktop,
    },
    GenEntry {
        name: "chrome_83_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 83,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_83_macos_desktop,
    },
    GenEntry {
        name: "chrome_86_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 86,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_86_windows_desktop,
    },
    GenEntry {
        name: "chrome_108_macos_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 108,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_108_macos_desktop_3,
    },
    GenEntry {
        name: "chrome_91_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 91,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_91_macos_desktop,
    },
    GenEntry {
        name: "chrome_117_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 117,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_117_macos_desktop,
    },
    GenEntry {
        name: "chrome_112_macos_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 112,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_112_macos_desktop_3,
    },
    GenEntry {
        name: "chrome_114_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 114,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_114_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_119_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 119,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_119_windows_desktop,
    },
    GenEntry {
        name: "edge_120_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 120,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_120_windows_desktop,
    },
    GenEntry {
        name: "opera_116_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 116,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_116_windows_desktop,
    },
    GenEntry {
        name: "opera_126_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_126_windows_desktop,
    },
    GenEntry {
        name: "chrome_74_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 74,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_74_windows_desktop,
    },
    GenEntry {
        name: "opera_120_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 120,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_120_macos_desktop,
    },
    GenEntry {
        name: "opera_122_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 122,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_122_macos_desktop,
    },
    GenEntry {
        name: "edge_139_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 139,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_139_macos_desktop,
    },
    GenEntry {
        name: "opera_123_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 123,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_123_macos_desktop,
    },
    GenEntry {
        name: "opera_124_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_124_macos_desktop,
    },
    GenEntry {
        name: "opera_125_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 125,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_125_macos_desktop,
    },
    GenEntry {
        name: "opera_126_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_126_macos_desktop,
    },
    GenEntry {
        name: "opera_127_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_127_macos_desktop,
    },
    GenEntry {
        name: "edge_144_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 144,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_144_macos_desktop,
    },
    GenEntry {
        name: "opera_128_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_128_macos_desktop,
    },
    GenEntry {
        name: "edge_145_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 145,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_145_macos_desktop,
    },
    GenEntry {
        name: "opera_129_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_129_macos_desktop,
    },
    GenEntry {
        name: "opera_130_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_130_macos_desktop,
    },
    GenEntry {
        name: "edge_148_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 148,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_148_macos_desktop,
    },
    GenEntry {
        name: "chrome_149_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 149,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_149_macos_desktop,
    },
    GenEntry {
        name: "edge_129_macos_desktop_2",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_129_macos_desktop_2,
    },
    GenEntry {
        name: "edge_132_macos_desktop_2",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_132_macos_desktop_2,
    },
    GenEntry {
        name: "edge_134_macos_desktop_2",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_134_macos_desktop_2,
    },
    GenEntry {
        name: "edge_135_macos_desktop_2",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 135,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_135_macos_desktop_2,
    },
    GenEntry {
        name: "edge_128_macos_desktop_2",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_128_macos_desktop_2,
    },
    GenEntry {
        name: "edge_129_macos_desktop_3",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_129_macos_desktop_3,
    },
    GenEntry {
        name: "edge_130_macos_desktop_2",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_130_macos_desktop_2,
    },
    GenEntry {
        name: "edge_134_macos_desktop_3",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_134_macos_desktop_3,
    },
    GenEntry {
        name: "edge_135_macos_desktop_3",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 135,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_135_macos_desktop_3,
    },
    GenEntry {
        name: "edge_128_macos_desktop_3",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_128_macos_desktop_3,
    },
    GenEntry {
        name: "edge_129_macos_desktop_4",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_129_macos_desktop_4,
    },
    GenEntry {
        name: "edge_130_macos_desktop_3",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_130_macos_desktop_3,
    },
    GenEntry {
        name: "edge_131_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 131,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_131_macos_desktop,
    },
    GenEntry {
        name: "edge_133_macos_desktop_2",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_133_macos_desktop_2,
    },
    GenEntry {
        name: "edge_134_macos_desktop_4",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_134_macos_desktop_4,
    },
    GenEntry {
        name: "edge_135_macos_desktop_4",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 135,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_135_macos_desktop_4,
    },
    GenEntry {
        name: "chrome_142_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 142,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_142_macos_desktop_2,
    },
    GenEntry {
        name: "opera_121_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 121,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_121_windows_desktop,
    },
    GenEntry {
        name: "opera_125_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 125,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_125_windows_desktop,
    },
    GenEntry {
        name: "opera_130_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_130_windows_desktop,
    },
    GenEntry {
        name: "edge_149_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 149,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: edge_149_windows_desktop,
    },
    GenEntry {
        name: "chrome_619_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 619,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_619_windows_desktop,
    },
    GenEntry {
        name: "chrome_140_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: chrome_140_windows_desktop_2,
    },
    GenEntry {
        name: "opera_115_windows_desktop_2",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 115,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: opera_115_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_109_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 109,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_109_windows_desktop,
    },
    GenEntry {
        name: "edge_116_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 116,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: edge_116_macos_desktop,
    },
    GenEntry {
        name: "chrome_97_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 97,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_97_windows_desktop,
    },
    GenEntry {
        name: "chrome_80_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 80,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_80_macos_desktop,
    },
    GenEntry {
        name: "chrome_110_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 110,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_110_macos_desktop_2,
    },
    GenEntry {
        name: "opera_103_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 103,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: opera_103_macos_desktop,
    },
    GenEntry {
        name: "chrome_79_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 79,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_79_windows_desktop,
    },
    GenEntry {
        name: "edge_92_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 92,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: edge_92_windows_desktop,
    },
    GenEntry {
        name: "edge_99_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 99,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: edge_99_windows_desktop,
    },
    GenEntry {
        name: "chrome_71_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 71,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_71_windows_desktop,
    },
    GenEntry {
        name: "chrome_110_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 110,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: chrome_110_windows_desktop_2,
    },
    GenEntry {
        name: "opera_121_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 121,
        ja4: "t13d1517h2_8daaf6152771_46b8896bec77",
        spec_fn: opera_121_macos_desktop,
    },
    GenEntry {
        name: "opera_95_windows_desktop_2",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 95,
        ja4: "t13d1517h2_8daaf6152771_b1ff8ab2d16f",
        spec_fn: opera_95_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_107_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 107,
        ja4: "t13d1517h2_8daaf6152771_fca9c764716e",
        spec_fn: chrome_107_windows_desktop_3,
    },
    GenEntry {
        name: "edge_111_macos_desktop_2",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 111,
        ja4: "t13d1616h2_e72c3b3287f1_e5627efa2ab1",
        spec_fn: edge_111_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_37_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 37,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_37_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_96_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 96,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_96_macos_desktop,
    },
    GenEntry {
        name: "chrome_85_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 85,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_85_windows_desktop,
    },
    GenEntry {
        name: "chrome_72_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 72,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_72_windows_desktop,
    },
    GenEntry {
        name: "chrome_75_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 75,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_75_windows_desktop,
    },
    GenEntry {
        name: "chrome_52_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 52,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_52_macos_desktop_2,
    },
    GenEntry {
        name: "edge_101_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 101,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: edge_101_windows_desktop,
    },
    GenEntry {
        name: "chrome_102_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 102,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_102_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_66_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 66,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_66_windows_desktop,
    },
    GenEntry {
        name: "chrome_77_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 77,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_77_windows_desktop,
    },
    GenEntry {
        name: "chrome_98_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 98,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_98_windows_desktop,
    },
    GenEntry {
        name: "chrome_61_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 61,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_61_macos_desktop,
    },
    GenEntry {
        name: "chrome_73_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 73,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_73_macos_desktop,
    },
    GenEntry {
        name: "chrome_75_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 75,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_75_macos_desktop,
    },
    GenEntry {
        name: "chrome_74_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 74,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_74_macos_desktop,
    },
    GenEntry {
        name: "edge_77_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 77,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: edge_77_macos_desktop,
    },
    GenEntry {
        name: "chrome_62_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 62,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_62_macos_desktop,
    },
    GenEntry {
        name: "chrome_71_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 71,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_71_macos_desktop,
    },
    GenEntry {
        name: "chrome_76_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 76,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_76_macos_desktop,
    },
    GenEntry {
        name: "chrome_72_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 72,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_72_macos_desktop,
    },
    GenEntry {
        name: "edge_99_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 99,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: edge_99_macos_desktop,
    },
    GenEntry {
        name: "chrome_16_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 16,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_16_macos_desktop,
    },
    GenEntry {
        name: "chrome_19_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 19,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_19_macos_desktop,
    },
    GenEntry {
        name: "chrome_22_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 22,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_22_macos_desktop,
    },
    GenEntry {
        name: "edge_12_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 12,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: edge_12_windows_desktop,
    },
    GenEntry {
        name: "chrome_62_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 62,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_62_windows_desktop,
    },
    GenEntry {
        name: "chrome_68_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 68,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_68_windows_desktop,
    },
    GenEntry {
        name: "opera_62_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 62,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: opera_62_windows_desktop,
    },
    GenEntry {
        name: "chrome_76_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 76,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_76_windows_desktop,
    },
    GenEntry {
        name: "chrome_87_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 87,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_87_windows_desktop,
    },
    GenEntry {
        name: "chrome_55_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 55,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_55_windows_desktop_2,
    },
    GenEntry {
        name: "opera_15_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 15,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: opera_15_windows_desktop,
    },
    GenEntry {
        name: "chrome_49_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 49,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_49_windows_desktop,
    },
    GenEntry {
        name: "chrome_63_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 63,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_63_windows_desktop,
    },
    GenEntry {
        name: "opera_62_windows_desktop_2",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 62,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: opera_62_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_76_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 76,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_76_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_72_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 72,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_72_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_99_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 99,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_99_windows_desktop,
    },
    GenEntry {
        name: "chrome_12_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 12,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_12_windows_desktop,
    },
    GenEntry {
        name: "chrome_20_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 20,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_20_windows_desktop,
    },
    GenEntry {
        name: "chrome_22_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 22,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_22_windows_desktop,
    },
    GenEntry {
        name: "opera_14_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 14,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: opera_14_windows_desktop,
    },
    GenEntry {
        name: "opera_20_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 20,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: opera_20_windows_desktop,
    },
    GenEntry {
        name: "opera_31_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 31,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: opera_31_windows_desktop,
    },
    GenEntry {
        name: "chrome_74_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 74,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_74_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_75_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 75,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: chrome_75_windows_desktop_2,
    },
    GenEntry {
        name: "opera_12_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 12,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: opera_12_windows_desktop,
    },
    GenEntry {
        name: "edge_115_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 115,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: edge_115_windows_desktop,
    },
    GenEntry {
        name: "chrome_54_macos_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 54,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_54_macos_desktop_3,
    },
    GenEntry {
        name: "chrome_101_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 101,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_101_macos_desktop,
    },
    GenEntry {
        name: "chrome_105_macos_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 105,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_105_macos_desktop_3,
    },
    GenEntry {
        name: "chrome_84_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 84,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_84_macos_desktop,
    },
    GenEntry {
        name: "chrome_71_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 71,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_71_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_95_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 95,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_95_macos_desktop,
    },
    GenEntry {
        name: "chrome_65_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 65,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_65_macos_desktop,
    },
    GenEntry {
        name: "chrome_99_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 99,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_99_macos_desktop,
    },
    GenEntry {
        name: "chrome_86_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 86,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_86_macos_desktop,
    },
    GenEntry {
        name: "chrome_63_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 63,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_63_macos_desktop,
    },
    GenEntry {
        name: "chrome_82_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 82,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_82_macos_desktop,
    },
    GenEntry {
        name: "chrome_51_macos_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 51,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_51_macos_desktop_3,
    },
    GenEntry {
        name: "chrome_79_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 79,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_79_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_64_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 64,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_64_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_51_macos_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 51,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_51_macos_desktop_4,
    },
    GenEntry {
        name: "chrome_52_macos_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 52,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_52_macos_desktop_3,
    },
    GenEntry {
        name: "chrome_69_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 69,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_69_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_75_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 75,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_75_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_89_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 89,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_89_macos_desktop,
    },
    GenEntry {
        name: "chrome_100_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 100,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_100_macos_desktop,
    },
    GenEntry {
        name: "chrome_93_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 93,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_93_macos_desktop,
    },
    GenEntry {
        name: "chrome_98_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 98,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_98_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_61_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 61,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_61_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_82_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 82,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_82_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_77_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 77,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_77_macos_desktop,
    },
    GenEntry {
        name: "chrome_65_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 65,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_65_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_92_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 92,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_92_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_95_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 95,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: chrome_95_macos_desktop_2,
    },
    GenEntry {
        name: "edge_94_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 94,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: edge_94_windows_desktop,
    },
    GenEntry {
        name: "edge_98_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 98,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: edge_98_windows_desktop,
    },
    GenEntry {
        name: "edge_8_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 8,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: edge_8_windows_desktop,
    },
    GenEntry {
        name: "chrome_20_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 20,
        ja4: "t13d171100_5b57614c22b0_be53661681a4",
        spec_fn: chrome_20_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_78_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 78,
        ja4: "t13d1711h2_5b57614c22b0_d811adc85aab",
        spec_fn: chrome_78_macos_desktop,
    },
    GenEntry {
        name: "chrome_118_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 118,
        ja4: "t13d1711h2_5b57614c22b0_e7c285222651",
        spec_fn: chrome_118_macos_desktop,
    },
    GenEntry {
        name: "edge_126_macos_desktop_3",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1711h2_5b57614c22b0_e7c285222651",
        spec_fn: edge_126_macos_desktop_3,
    },
    GenEntry {
        name: "edge_126_macos_desktop_4",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1711h2_5b57614c22b0_e7c285222651",
        spec_fn: edge_126_macos_desktop_4,
    },
    GenEntry {
        name: "chrome_39_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 39,
        ja4: "t13d1711ht_ab0a1bf427ad_a29327ec888c",
        spec_fn: chrome_39_macos_desktop,
    },
    GenEntry {
        name: "edge_109_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 109,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: edge_109_windows_desktop,
    },
    GenEntry {
        name: "opera_94_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 94,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: opera_94_windows_desktop,
    },
    GenEntry {
        name: "chrome_68_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 68,
        ja4: "t13d171300_5b57614c22b0_43ade6aba3df",
        spec_fn: chrome_68_macos_desktop,
    },
    GenEntry {
        name: "chrome_51_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 51,
        ja4: "t13d1713ht_5b57614c22b0_eca864cca44a",
        spec_fn: chrome_51_windows_desktop,
    },
    GenEntry {
        name: "edge_114_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 114,
        ja4: "t13d1713ht_ab0a1bf427ad_ecd0401ec68b",
        spec_fn: edge_114_macos_desktop,
    },
    GenEntry {
        name: "edge_114_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 114,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: edge_114_windows_desktop,
    },
    GenEntry {
        name: "edge_109_windows_desktop_2",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 109,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: edge_109_windows_desktop_2,
    },
    GenEntry {
        name: "edge_118_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 118,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: edge_118_windows_desktop,
    },
    GenEntry {
        name: "edge_117_macos_desktop",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 117,
        ja4: "t13d201000_314f1408a5a6_e7c285222651",
        spec_fn: edge_117_macos_desktop,
    },
    GenEntry {
        name: "edge_18_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 18,
        ja4: "t13d201000_314f1408a5a6_e7c285222651",
        spec_fn: edge_18_windows_desktop,
    },
    GenEntry {
        name: "chrome_95_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 95,
        ja4: "t13d201000_314f1408a5a6_e7c285222651",
        spec_fn: chrome_95_windows_desktop,
    },
    GenEntry {
        name: "edge_88_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 88,
        ja4: "t13d201100_314f1408a5a6_ab7e3b40a677",
        spec_fn: edge_88_windows_desktop,
    },
    GenEntry {
        name: "chrome_66_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 66,
        ja4: "t13d201100_314f1408a5a6_ab7e3b40a677",
        spec_fn: chrome_66_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_91_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 91,
        ja4: "t13d201100_314f1408a5a6_e5728521abd4",
        spec_fn: chrome_91_windows_desktop_2,
    },
    GenEntry {
        name: "edge_112_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 112,
        ja4: "t13d2212h2_231e334592e8_36bf25f296df",
        spec_fn: edge_112_windows_desktop,
    },
    GenEntry {
        name: "chrome_118_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 118,
        ja4: "t13d2811h2_a01be8c064b6_1f22a2ca17c4",
        spec_fn: chrome_118_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_44_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 44,
        ja4: "t13d301000_1d37bd780c83_1f22a2ca17c4",
        spec_fn: chrome_44_macos_desktop,
    },
    GenEntry {
        name: "chrome_47_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 47,
        ja4: "t13d301000_1d37bd780c83_1f22a2ca17c4",
        spec_fn: chrome_47_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_114_macos_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 114,
        ja4: "t13d301000_1d37bd780c83_518fb456ca59",
        spec_fn: chrome_114_macos_desktop_3,
    },
    GenEntry {
        name: "edge_114_macos_desktop_2",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 114,
        ja4: "t13d301000_1d37bd780c83_518fb456ca59",
        spec_fn: edge_114_macos_desktop_2,
    },
    GenEntry {
        name: "opera_89_macos_desktop",
        browser: Browser::Opera,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 89,
        ja4: "t13d301000_1d37bd780c83_518fb456ca59",
        spec_fn: opera_89_macos_desktop,
    },
    GenEntry {
        name: "opera_89_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 89,
        ja4: "t13d301000_1d37bd780c83_518fb456ca59",
        spec_fn: opera_89_windows_desktop,
    },
    GenEntry {
        name: "edge_44_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 44,
        ja4: "t13d301000_1d37bd780c83_518fb456ca59",
        spec_fn: edge_44_windows_desktop,
    },
    GenEntry {
        name: "chrome_70_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 70,
        ja4: "t13d301000_1d37bd780c83_5ac7197df9d2",
        spec_fn: chrome_70_macos_desktop,
    },
    GenEntry {
        name: "chrome_36_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 36,
        ja4: "t13d301000_1d37bd780c83_7379471da272",
        spec_fn: chrome_36_macos_desktop,
    },
    GenEntry {
        name: "opera_99_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 99,
        ja4: "t13d301000_1d37bd780c83_7379471da272",
        spec_fn: opera_99_windows_desktop,
    },
    GenEntry {
        name: "chrome_46_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 46,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_46_macos_desktop,
    },
    GenEntry {
        name: "chrome_59_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 59,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_59_macos_desktop,
    },
    GenEntry {
        name: "chrome_25_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 25,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_25_macos_desktop,
    },
    GenEntry {
        name: "chrome_29_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 29,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_29_macos_desktop,
    },
    GenEntry {
        name: "chrome_20_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 20,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_20_macos_desktop,
    },
    GenEntry {
        name: "chrome_58_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 58,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_58_macos_desktop,
    },
    GenEntry {
        name: "chrome_21_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 21,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_21_macos_desktop,
    },
    GenEntry {
        name: "chrome_35_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 35,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_35_windows_desktop,
    },
    GenEntry {
        name: "chrome_26_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 26,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_26_windows_desktop,
    },
    GenEntry {
        name: "chrome_44_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 44,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_44_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_40_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 40,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_40_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_17_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 17,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_17_windows_desktop,
    },
    GenEntry {
        name: "chrome_17_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 17,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: chrome_17_windows_desktop_2,
    },
    GenEntry {
        name: "opera_8_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 8,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: opera_8_windows_desktop,
    },
    GenEntry {
        name: "opera_8_windows_desktop_2",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 8,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: opera_8_windows_desktop_2,
    },
    GenEntry {
        name: "opera_8_windows_desktop_3",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 8,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: opera_8_windows_desktop_3,
    },
    GenEntry {
        name: "opera_8_windows_desktop_4",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 8,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: opera_8_windows_desktop_4,
    },
    GenEntry {
        name: "opera_8_windows_desktop_5",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 8,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: opera_8_windows_desktop_5,
    },
    GenEntry {
        name: "opera_8_windows_desktop_6",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 8,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: opera_8_windows_desktop_6,
    },
    GenEntry {
        name: "opera_9_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 9,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: opera_9_windows_desktop,
    },
    GenEntry {
        name: "opera_9_windows_desktop_2",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 9,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: opera_9_windows_desktop_2,
    },
    GenEntry {
        name: "opera_9_windows_desktop_3",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 9,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: opera_9_windows_desktop_3,
    },
    GenEntry {
        name: "opera_9_windows_desktop_4",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 9,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: opera_9_windows_desktop_4,
    },
    GenEntry {
        name: "chrome_70_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 70,
        ja4: "t13d301100_1d37bd780c83_24695f2957a7",
        spec_fn: chrome_70_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_99_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 99,
        ja4: "t13d301100_1d37bd780c83_d41ae481755e",
        spec_fn: chrome_99_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_84_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 84,
        ja4: "t13d301100_1d37bd780c83_d41ae481755e",
        spec_fn: chrome_84_windows_desktop,
    },
    GenEntry {
        name: "chrome_61_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 61,
        ja4: "t13d301100_1d37bd780c83_d41ae481755e",
        spec_fn: chrome_61_windows_desktop,
    },
    GenEntry {
        name: "chrome_34_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 34,
        ja4: "t13d301100_1d37bd780c83_d41ae481755e",
        spec_fn: chrome_34_macos_desktop,
    },
    GenEntry {
        name: "chrome_103_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 103,
        ja4: "t13d301100_1d37bd780c83_d41ae481755e",
        spec_fn: chrome_103_macos_desktop,
    },
    GenEntry {
        name: "edge_100_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 100,
        ja4: "t13d301100_1d37bd780c83_d41ae481755e",
        spec_fn: edge_100_windows_desktop,
    },
    GenEntry {
        name: "opera_107_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 107,
        ja4: "t13d3011h2_1d37bd780c83_5ac7197df9d2",
        spec_fn: opera_107_windows_desktop,
    },
    GenEntry {
        name: "chrome_111_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 111,
        ja4: "t13d301200_1d37bd780c83_0d7ed806c34c",
        spec_fn: chrome_111_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_50_macos_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 50,
        ja4: "t13d301200_1d37bd780c83_d339722ba4af",
        spec_fn: chrome_50_macos_desktop_3,
    },
    GenEntry {
        name: "chrome_111_macos_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 111,
        ja4: "t13d301200_1d37bd780c83_d339722ba4af",
        spec_fn: chrome_111_macos_desktop_4,
    },
    GenEntry {
        name: "chrome_31_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 31,
        ja4: "t13d301200_1d37bd780c83_ecd0401ec68b",
        spec_fn: chrome_31_windows_desktop,
    },
    GenEntry {
        name: "chrome_109_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 109,
        ja4: "t13d3012h2_1d37bd780c83_b26ce05bbdd6",
        spec_fn: chrome_109_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_121_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 121,
        ja4: "t13d3012h2_1d37bd780c83_b26ce05bbdd6",
        spec_fn: chrome_121_windows_desktop,
    },
    GenEntry {
        name: "edge_131_macos_desktop_2",
        browser: Browser::Edge,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 131,
        ja4: "t13d3012h2_1d37bd780c83_b26ce05bbdd6",
        spec_fn: edge_131_macos_desktop_2,
    },
    GenEntry {
        name: "edge_121_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 121,
        ja4: "t13d3012h2_1d37bd780c83_b26ce05bbdd6",
        spec_fn: edge_121_windows_desktop,
    },
    GenEntry {
        name: "edge_91_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 91,
        ja4: "t13d3012h2_1d37bd780c83_b26ce05bbdd6",
        spec_fn: edge_91_windows_desktop,
    },
    GenEntry {
        name: "chrome_89_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 89,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_89_windows_desktop,
    },
    GenEntry {
        name: "chrome_41_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 41,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_41_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_50_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 50,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_50_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_83_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 83,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_83_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_60_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 60,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_60_windows_desktop,
    },
    GenEntry {
        name: "chrome_86_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 86,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_86_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_67_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 67,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_67_windows_desktop,
    },
    GenEntry {
        name: "chrome_88_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 88,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_88_windows_desktop,
    },
    GenEntry {
        name: "chrome_92_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 92,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_92_windows_desktop,
    },
    GenEntry {
        name: "chrome_54_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 54,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_54_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_50_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 50,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_50_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_93_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 93,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_93_windows_desktop,
    },
    GenEntry {
        name: "chrome_73_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 73,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_73_windows_desktop,
    },
    GenEntry {
        name: "chrome_80_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 80,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_80_windows_desktop,
    },
    GenEntry {
        name: "chrome_81_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 81,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_81_windows_desktop,
    },
    GenEntry {
        name: "chrome_93_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 93,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_93_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_69_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 69,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_69_windows_desktop,
    },
    GenEntry {
        name: "chrome_91_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 91,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_91_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_90_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 90,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_90_windows_desktop,
    },
    GenEntry {
        name: "chrome_14_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 14,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_14_macos_desktop,
    },
    GenEntry {
        name: "chrome_27_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 27,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_27_macos_desktop,
    },
    GenEntry {
        name: "chrome_3_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 3,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_3_windows_desktop,
    },
    GenEntry {
        name: "chrome_81_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 81,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_81_macos_desktop,
    },
    GenEntry {
        name: "chrome_33_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 33,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_33_macos_desktop,
    },
    GenEntry {
        name: "chrome_11_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 11,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_11_macos_desktop,
    },
    GenEntry {
        name: "chrome_4_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 4,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_4_macos_desktop,
    },
    GenEntry {
        name: "chrome_48_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 48,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_48_windows_desktop_2,
    },
    GenEntry {
        name: "edge_83_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 83,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: edge_83_windows_desktop,
    },
    GenEntry {
        name: "opera_69_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 69,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_69_windows_desktop,
    },
    GenEntry {
        name: "opera_73_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 73,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_73_windows_desktop,
    },
    GenEntry {
        name: "opera_74_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 74,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_74_windows_desktop,
    },
    GenEntry {
        name: "edge_90_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 90,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: edge_90_windows_desktop,
    },
    GenEntry {
        name: "opera_77_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 77,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_77_windows_desktop,
    },
    GenEntry {
        name: "opera_78_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 78,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_78_windows_desktop,
    },
    GenEntry {
        name: "opera_34_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 34,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_34_windows_desktop,
    },
    GenEntry {
        name: "opera_53_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 53,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_53_windows_desktop,
    },
    GenEntry {
        name: "opera_54_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 54,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_54_windows_desktop,
    },
    GenEntry {
        name: "chrome_19_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 19,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_19_windows_desktop,
    },
    GenEntry {
        name: "opera_18_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 18,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_18_windows_desktop,
    },
    GenEntry {
        name: "chrome_47_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 47,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_47_windows_desktop_3,
    },
    GenEntry {
        name: "opera_34_windows_desktop_2",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 34,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_34_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_48_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 48,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_48_windows_desktop_3,
    },
    GenEntry {
        name: "opera_35_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 35,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_35_windows_desktop,
    },
    GenEntry {
        name: "chrome_80_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 80,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_80_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_87_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 87,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_87_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_62_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 62,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_62_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_66_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 66,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_66_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_85_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 85,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_85_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_89_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 89,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_89_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_92_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 92,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_92_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_13_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 13,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_13_windows_desktop,
    },
    GenEntry {
        name: "chrome_31_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 31,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_31_windows_desktop_2,
    },
    GenEntry {
        name: "opera_28_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 28,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_28_windows_desktop,
    },
    GenEntry {
        name: "chrome_42_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 42,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_42_windows_desktop_2,
    },
    GenEntry {
        name: "opera_29_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 29,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_29_windows_desktop,
    },
    GenEntry {
        name: "chrome_43_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 43,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_43_windows_desktop,
    },
    GenEntry {
        name: "chrome_44_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 44,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_44_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_46_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 46,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_46_windows_desktop,
    },
    GenEntry {
        name: "chrome_81_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 81,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_81_windows_desktop_2,
    },
    GenEntry {
        name: "edge_81_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 81,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: edge_81_windows_desktop,
    },
    GenEntry {
        name: "chrome_30_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 30,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_30_windows_desktop,
    },
    GenEntry {
        name: "opera_24_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 24,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_24_windows_desktop,
    },
    GenEntry {
        name: "opera_30_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 30,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_30_windows_desktop,
    },
    GenEntry {
        name: "opera_34_windows_desktop_3",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 34,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_34_windows_desktop_3,
    },
    GenEntry {
        name: "opera_35_windows_desktop_2",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 35,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_35_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_49_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 49,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_49_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_39_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 39,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_39_windows_desktop,
    },
    GenEntry {
        name: "chrome_43_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 43,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: chrome_43_windows_desktop_2,
    },
    GenEntry {
        name: "opera_32_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 32,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_32_windows_desktop,
    },
    GenEntry {
        name: "opera_11_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 11,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_11_windows_desktop,
    },
    GenEntry {
        name: "opera_12_windows_desktop_2",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 12,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: opera_12_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_56_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 56,
        ja4: "t13d3013h2_1d37bd780c83_ce5650b735ce",
        spec_fn: chrome_56_macos_desktop,
    },
    GenEntry {
        name: "chrome_59_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 59,
        ja4: "t13d3013h2_1d37bd780c83_ce5650b735ce",
        spec_fn: chrome_59_windows_desktop,
    },
    GenEntry {
        name: "edge_97_windows_desktop",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 97,
        ja4: "t13d3013ht_1d37bd780c83_1b3407e2c936",
        spec_fn: edge_97_windows_desktop,
    },
    GenEntry {
        name: "edge_14_windows_desktop_2",
        browser: Browser::Edge,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 14,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: edge_14_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_42_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 42,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: chrome_42_macos_desktop,
    },
    GenEntry {
        name: "chrome_60_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 60,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: chrome_60_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_42_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 42,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: chrome_42_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_60_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 60,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: chrome_60_macos_desktop,
    },
    GenEntry {
        name: "opera_48_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 48,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: opera_48_windows_desktop,
    },
    GenEntry {
        name: "chrome_61_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 61,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: chrome_61_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_59_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 59,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: chrome_59_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_60_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 60,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: chrome_60_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_61_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 61,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: chrome_61_windows_desktop_3,
    },
    GenEntry {
        name: "opera_12_linux_desktop",
        browser: Browser::Opera,
        os: Some(Os::Linux),
        device: Device::Desktop,
        major: 12,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: opera_12_linux_desktop,
    },
    GenEntry {
        name: "chrome_15_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 15,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_15_macos_desktop,
    },
    GenEntry {
        name: "chrome_132_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_132_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_124_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_124_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_127_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_127_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_124_macos_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_124_macos_desktop_3,
    },
    GenEntry {
        name: "chrome_113_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 113,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_113_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_125_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 125,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_125_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_129_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_129_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_124_macos_desktop_4",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_124_macos_desktop_4,
    },
    GenEntry {
        name: "chrome_130_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_130_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_133_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_133_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_131_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 131,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_131_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_5_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 5,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_5_macos_desktop,
    },
    GenEntry {
        name: "chrome_10_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 10,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_10_macos_desktop,
    },
    GenEntry {
        name: "chrome_6_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 6,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_6_macos_desktop,
    },
    GenEntry {
        name: "chrome_9_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 9,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_9_macos_desktop,
    },
    GenEntry {
        name: "chrome_10_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 10,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_10_windows_desktop,
    },
    GenEntry {
        name: "chrome_4_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 4,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_4_windows_desktop,
    },
    GenEntry {
        name: "chrome_9_linux_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Linux),
        device: Device::Desktop,
        major: 9,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: chrome_9_linux_desktop,
    },
    GenEntry {
        name: "opera_42_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 42,
        ja4: "t13d421000_49900ac2774e_1f22a2ca17c4",
        spec_fn: opera_42_windows_desktop,
    },
    GenEntry {
        name: "chrome_82_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 82,
        ja4: "t13d421000_49900ac2774e_a29327ec888c",
        spec_fn: chrome_82_windows_desktop,
    },
    GenEntry {
        name: "chrome_52_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 52,
        ja4: "t13d421000_49900ac2774e_a29327ec888c",
        spec_fn: chrome_52_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_57_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 57,
        ja4: "t13d421000_49900ac2774e_a29327ec888c",
        spec_fn: chrome_57_windows_desktop,
    },
    GenEntry {
        name: "chrome_93_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 93,
        ja4: "t13d421000_49900ac2774e_a29327ec888c",
        spec_fn: chrome_93_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_69_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 69,
        ja4: "t13d421000_49900ac2774e_a29327ec888c",
        spec_fn: chrome_69_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_70_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 70,
        ja4: "t13d421000_49900ac2774e_a29327ec888c",
        spec_fn: chrome_70_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_80_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 80,
        ja4: "t13d421000_49900ac2774e_a29327ec888c",
        spec_fn: chrome_80_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_58_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 58,
        ja4: "t13d421200_49900ac2774e_d339722ba4af",
        spec_fn: chrome_58_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_78_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 78,
        ja4: "t13d421200_49900ac2774e_d339722ba4af",
        spec_fn: chrome_78_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_36_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 36,
        ja4: "t13d4212ht_49900ac2774e_b26ce05bbdd6",
        spec_fn: chrome_36_windows_desktop,
    },
    GenEntry {
        name: "chrome_57_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 57,
        ja4: "t13d4212ht_49900ac2774e_b26ce05bbdd6",
        spec_fn: chrome_57_macos_desktop,
    },
    GenEntry {
        name: "chrome_57_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 57,
        ja4: "t13d4212ht_49900ac2774e_b26ce05bbdd6",
        spec_fn: chrome_57_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_86_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 86,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_86_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_84_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 84,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_84_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_107_macos_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 107,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_107_macos_desktop_3,
    },
    GenEntry {
        name: "chrome_101_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 101,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_101_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_90_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 90,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_90_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_98_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 98,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_98_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_85_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 85,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_85_macos_desktop,
    },
    GenEntry {
        name: "chrome_95_macos_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 95,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_95_macos_desktop_3,
    },
    GenEntry {
        name: "chrome_88_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 88,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_88_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_79_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 79,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_79_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_102_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 102,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_102_macos_desktop,
    },
    GenEntry {
        name: "chrome_35_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 35,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_35_macos_desktop,
    },
    GenEntry {
        name: "chrome_88_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 88,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_88_macos_desktop,
    },
    GenEntry {
        name: "chrome_101_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 101,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_101_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_27_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 27,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_27_windows_desktop,
    },
    GenEntry {
        name: "chrome_32_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 32,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_32_windows_desktop,
    },
    GenEntry {
        name: "chrome_24_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 24,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_24_windows_desktop,
    },
    GenEntry {
        name: "chrome_96_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 96,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_96_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_77_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 77,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_77_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_94_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 94,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_94_windows_desktop,
    },
    GenEntry {
        name: "chrome_99_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 99,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_99_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_104_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 104,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_104_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_29_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 29,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_29_windows_desktop,
    },
    GenEntry {
        name: "chrome_93_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 93,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_93_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_65_macos_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 65,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_65_macos_desktop_3,
    },
    GenEntry {
        name: "brave_86_macos_desktop",
        browser: Browser::Brave,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 86,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_86_macos_desktop,
    },
    GenEntry {
        name: "chrome_87_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 87,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_87_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_67_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 67,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_67_macos_desktop,
    },
    GenEntry {
        name: "brave_80_macos_desktop",
        browser: Browser::Brave,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 80,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_80_macos_desktop,
    },
    GenEntry {
        name: "brave_84_macos_desktop",
        browser: Browser::Brave,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 84,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_84_macos_desktop,
    },
    GenEntry {
        name: "brave_89_macos_desktop",
        browser: Browser::Brave,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 89,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_89_macos_desktop,
    },
    GenEntry {
        name: "chrome_63_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 63,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_63_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_66_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 66,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_66_macos_desktop,
    },
    GenEntry {
        name: "chrome_90_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 90,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_90_macos_desktop,
    },
    GenEntry {
        name: "chrome_97_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 97,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_97_macos_desktop,
    },
    GenEntry {
        name: "brave_87_macos_desktop",
        browser: Browser::Brave,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 87,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_87_macos_desktop,
    },
    GenEntry {
        name: "brave_88_macos_desktop",
        browser: Browser::Brave,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 88,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_88_macos_desktop,
    },
    GenEntry {
        name: "chrome_89_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 89,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_89_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_94_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 94,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_94_macos_desktop,
    },
    GenEntry {
        name: "chrome_100_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 100,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_100_macos_desktop_2,
    },
    GenEntry {
        name: "brave_78_macos_desktop",
        browser: Browser::Brave,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 78,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_78_macos_desktop,
    },
    GenEntry {
        name: "brave_83_macos_desktop",
        browser: Browser::Brave,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 83,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_83_macos_desktop,
    },
    GenEntry {
        name: "brave_85_macos_desktop",
        browser: Browser::Brave,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 85,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_85_macos_desktop,
    },
    GenEntry {
        name: "brave_90_macos_desktop",
        browser: Browser::Brave,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 90,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_90_macos_desktop,
    },
    GenEntry {
        name: "brave_79_macos_desktop",
        browser: Browser::Brave,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 79,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_79_macos_desktop,
    },
    GenEntry {
        name: "brave_81_macos_desktop",
        browser: Browser::Brave,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 81,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_81_macos_desktop,
    },
    GenEntry {
        name: "chrome_82_macos_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 82,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_82_macos_desktop_3,
    },
    GenEntry {
        name: "chrome_77_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 77,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_77_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_32_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 32,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_32_macos_desktop,
    },
    GenEntry {
        name: "chrome_24_macos_desktop",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 24,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_24_macos_desktop,
    },
    GenEntry {
        name: "brave_88_macos_desktop_2",
        browser: Browser::Brave,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 88,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_88_macos_desktop_2,
    },
    GenEntry {
        name: "brave_87_macos_desktop_2",
        browser: Browser::Brave,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 87,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_87_macos_desktop_2,
    },
    GenEntry {
        name: "brave_89_macos_desktop_2",
        browser: Browser::Brave,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 89,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_89_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_88_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 88,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_88_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_96_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 96,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_96_macos_desktop_2,
    },
    GenEntry {
        name: "chrome_89_macos_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 89,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_89_macos_desktop_3,
    },
    GenEntry {
        name: "brave_78_windows_desktop",
        browser: Browser::Brave,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 78,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_78_windows_desktop,
    },
    GenEntry {
        name: "brave_87_windows_desktop",
        browser: Browser::Brave,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 87,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_87_windows_desktop,
    },
    GenEntry {
        name: "brave_89_windows_desktop",
        browser: Browser::Brave,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 89,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_89_windows_desktop,
    },
    GenEntry {
        name: "chrome_82_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 82,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_82_windows_desktop_2,
    },
    GenEntry {
        name: "brave_77_windows_desktop",
        browser: Browser::Brave,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 77,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_77_windows_desktop,
    },
    GenEntry {
        name: "brave_79_windows_desktop",
        browser: Browser::Brave,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 79,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_79_windows_desktop,
    },
    GenEntry {
        name: "brave_84_windows_desktop",
        browser: Browser::Brave,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 84,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_84_windows_desktop,
    },
    GenEntry {
        name: "brave_86_windows_desktop",
        browser: Browser::Brave,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 86,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_86_windows_desktop,
    },
    GenEntry {
        name: "chrome_67_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 67,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_67_windows_desktop_2,
    },
    GenEntry {
        name: "brave_78_windows_desktop_2",
        browser: Browser::Brave,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 78,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_78_windows_desktop_2,
    },
    GenEntry {
        name: "brave_88_windows_desktop",
        browser: Browser::Brave,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 88,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_88_windows_desktop,
    },
    GenEntry {
        name: "chrome_28_windows_desktop",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 28,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_28_windows_desktop,
    },
    GenEntry {
        name: "chrome_68_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 68,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_68_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_69_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 69,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_69_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_84_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 84,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_84_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_88_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 88,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_88_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_95_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 95,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_95_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_97_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 97,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_97_windows_desktop_2,
    },
    GenEntry {
        name: "brave_80_windows_desktop",
        browser: Browser::Brave,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 80,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_80_windows_desktop,
    },
    GenEntry {
        name: "brave_84_windows_desktop_2",
        browser: Browser::Brave,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 84,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_84_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_103_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 103,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_103_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_73_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 73,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_73_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_75_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 75,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_75_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_78_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 78,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_78_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_81_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 81,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_81_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_24_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 24,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_24_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_100_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 100,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_100_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_29_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 29,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_29_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_94_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 94,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_94_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_28_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 28,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_28_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_87_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 87,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_87_windows_desktop_3,
    },
    GenEntry {
        name: "brave_86_windows_desktop_2",
        browser: Browser::Brave,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 86,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_86_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_27_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 27,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_27_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_94_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 94,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_94_windows_desktop_3,
    },
    GenEntry {
        name: "brave_78_windows_desktop_3",
        browser: Browser::Brave,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 78,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_78_windows_desktop_3,
    },
    GenEntry {
        name: "brave_84_windows_desktop_3",
        browser: Browser::Brave,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 84,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_84_windows_desktop_3,
    },
    GenEntry {
        name: "brave_88_windows_desktop_2",
        browser: Browser::Brave,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 88,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: brave_88_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_100_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 100,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_100_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_105_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 105,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_105_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_108_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 108,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_108_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_74_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 74,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_74_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_84_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 84,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_84_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_85_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 85,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_85_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_89_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 89,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_89_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_98_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 98,
        ja4: "t13d4312ht_36cd39a4fcc1_58ed7828516f",
        spec_fn: chrome_98_windows_desktop_3,
    },
    GenEntry {
        name: "chrome_68_windows_desktop_3",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 68,
        ja4: "t13d481000_c08b26b7ea02_5ac7197df9d2",
        spec_fn: chrome_68_windows_desktop_3,
    },
    GenEntry {
        name: "opera_45_windows_desktop",
        browser: Browser::Opera,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 45,
        ja4: "t13d581000_363f866c7444_1f22a2ca17c4",
        spec_fn: opera_45_windows_desktop,
    },
    GenEntry {
        name: "chrome_63_windows_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 63,
        ja4: "t13d581000_363f866c7444_5ac7197df9d2",
        spec_fn: chrome_63_windows_desktop_2,
    },
    GenEntry {
        name: "chrome_122_macos_desktop_2",
        browser: Browser::Chrome,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 122,
        ja4: "t13d5811ht_363f866c7444_1f22a2ca17c4",
        spec_fn: chrome_122_macos_desktop_2,
    },
];

// ja4=t13d101100_01be160bb49b_36bf25f296df obs=10
#[rustfmt::skip]
spec! {
    edge_79_windows_desktop,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0201, 0x0403, 0x0503, 0x0203, 0x0202, 0x0601, 0x0603],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0031, ""],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    chrome_40_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8728
#[rustfmt::skip]
spec! {
    chrome_70_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8717
#[rustfmt::skip]
spec! {
    chrome_41_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    chrome_41_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8715
#[rustfmt::skip]
spec! {
    chrome_37_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    chrome_40_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8715
#[rustfmt::skip]
spec! {
    chrome_41_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    chrome_64_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    opera_96_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8751
#[rustfmt::skip]
spec! {
    opera_97_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    chrome_112_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8714
#[rustfmt::skip]
spec! {
    edge_104_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8717
#[rustfmt::skip]
spec! {
    chrome_105_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8758
#[rustfmt::skip]
spec! {
    chrome_17_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8722
#[rustfmt::skip]
spec! {
    opera_98_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8728
#[rustfmt::skip]
spec! {
    chrome_111_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8726
#[rustfmt::skip]
spec! {
    chrome_112_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    opera_96_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8722
#[rustfmt::skip]
spec! {
    chrome_115_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8715
#[rustfmt::skip]
spec! {
    edge_111_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8714
#[rustfmt::skip]
spec! {
    edge_112_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8722
#[rustfmt::skip]
spec! {
    opera_97_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8715
#[rustfmt::skip]
spec! {
    chrome_107_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    edge_108_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8717
#[rustfmt::skip]
spec! {
    edge_110_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    chrome_107_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8727
#[rustfmt::skip]
spec! {
    opera_96_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8718
#[rustfmt::skip]
spec! {
    chrome_108_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    chrome_111_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    edge_105_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8730
#[rustfmt::skip]
spec! {
    edge_106_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8714
#[rustfmt::skip]
spec! {
    edge_107_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8716
#[rustfmt::skip]
spec! {
    chrome_111_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8717
#[rustfmt::skip]
spec! {
    chrome_105_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    chrome_37_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    chrome_64_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    chrome_65_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8806
#[rustfmt::skip]
spec! {
    chrome_110_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8714
#[rustfmt::skip]
spec! {
    chrome_113_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8721
#[rustfmt::skip]
spec! {
    chrome_58_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    chrome_42_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8715
#[rustfmt::skip]
spec! {
    chrome_78_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=9299
#[rustfmt::skip]
spec! {
    chrome_126_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8714
#[rustfmt::skip]
spec! {
    chrome_40_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8714
#[rustfmt::skip]
spec! {
    chrome_64_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8715
#[rustfmt::skip]
spec! {
    chrome_65_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8716
#[rustfmt::skip]
spec! {
    opera_97_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    chrome_45_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d131100_f57a46bbacb6_ab7e3b40a677 obs=588
#[rustfmt::skip]
spec! {
    chrome_44_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303], raw[0x0032, ""],
}

// ja4=t13d131100_f57a46bbacb6_e5728521abd4 obs=259
#[rustfmt::skip]
spec! {
    brave_126_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303], raw[0x0032, ""],
}

// ja4=t13d1311h2_f57a46bbacb6_e7c285222651 obs=434
#[rustfmt::skip]
spec! {
    edge_121_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d1311h2_f57a46bbacb6_e7c285222651 obs=435
#[rustfmt::skip]
spec! {
    edge_103_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d1311h2_f57a46bbacb6_e7c285222651 obs=434
#[rustfmt::skip]
spec! {
    chrome_108_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d1311h2_f57a46bbacb6_e7c285222651 obs=434
#[rustfmt::skip]
spec! {
    opera_109_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d1511h2_8daaf6152771_6d021c4c45cd obs=250
#[rustfmt::skip]
spec! {
    chrome_149_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303],
}

// ja4=t13d1511h2_8daaf6152771_6d021c4c45cd obs=248
#[rustfmt::skip]
spec! {
    chrome_150_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303],
}

// ja4=t13d1513h2_8daaf6152771_9249cab70c77 obs=105
#[rustfmt::skip]
spec! {
    chrome_126_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303],
}

// ja4=t13d1513h2_8daaf6152771_9249cab70c77 obs=106
#[rustfmt::skip]
spec! {
    edge_126_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_52_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_49_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_50_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_50_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_54_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    edge_15_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_53_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_53_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    chrome_48_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_4769d65a485e obs=534
#[rustfmt::skip]
spec! {
    edge_9_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_4769d65a485e obs=534
#[rustfmt::skip]
spec! {
    chrome_47_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_55_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1616
#[rustfmt::skip]
spec! {
    chrome_47_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_54_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_53_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_55_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_50_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    edge_16_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_48_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_51_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1616
#[rustfmt::skip]
spec! {
    chrome_48_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_51_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1616
#[rustfmt::skip]
spec! {
    chrome_53_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_54_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_55_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_48_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_49_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_52_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    edge_17_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    edge_15_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_55_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    edge_14_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    chrome_52_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17196
#[rustfmt::skip]
spec! {
    chrome_116_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17175
#[rustfmt::skip]
spec! {
    chrome_116_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17175
#[rustfmt::skip]
spec! {
    chrome_113_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    opera_117_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_114_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17929
#[rustfmt::skip]
spec! {
    chrome_126_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17282
#[rustfmt::skip]
spec! {
    edge_128_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_111_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17284
#[rustfmt::skip]
spec! {
    edge_125_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    edge_105_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17189
#[rustfmt::skip]
spec! {
    chrome_119_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17196
#[rustfmt::skip]
spec! {
    chrome_122_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17175
#[rustfmt::skip]
spec! {
    chrome_103_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17175
#[rustfmt::skip]
spec! {
    chrome_104_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17176
#[rustfmt::skip]
spec! {
    chrome_105_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_106_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17310
#[rustfmt::skip]
spec! {
    chrome_117_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17176
#[rustfmt::skip]
spec! {
    edge_119_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17194
#[rustfmt::skip]
spec! {
    edge_122_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17178
#[rustfmt::skip]
spec! {
    opera_111_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    chrome_37_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17183
#[rustfmt::skip]
spec! {
    edge_124_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_87_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_110_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    opera_109_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17285
#[rustfmt::skip]
spec! {
    edge_127_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17198
#[rustfmt::skip]
spec! {
    edge_127_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17180
#[rustfmt::skip]
spec! {
    edge_132_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17194
#[rustfmt::skip]
spec! {
    chrome_118_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17177
#[rustfmt::skip]
spec! {
    opera_118_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17179
#[rustfmt::skip]
spec! {
    opera_119_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_115_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    edge_113_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    edge_116_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_101_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    chrome_106_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    chrome_79_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17177
#[rustfmt::skip]
spec! {
    chrome_104_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17176
#[rustfmt::skip]
spec! {
    chrome_109_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    edge_124_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17176
#[rustfmt::skip]
spec! {
    opera_110_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17206
#[rustfmt::skip]
spec! {
    edge_125_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17186
#[rustfmt::skip]
spec! {
    edge_126_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17177
#[rustfmt::skip]
spec! {
    opera_112_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    opera_113_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    opera_114_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17176
#[rustfmt::skip]
spec! {
    opera_115_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17181
#[rustfmt::skip]
spec! {
    opera_116_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17175
#[rustfmt::skip]
spec! {
    opera_117_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    opera_118_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    opera_119_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_92_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17176
#[rustfmt::skip]
spec! {
    chrome_98_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    edge_106_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    edge_107_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    edge_110_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    edge_111_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    opera_98_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    edge_123_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17175
#[rustfmt::skip]
spec! {
    opera_110_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    opera_112_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    edge_117_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17177
#[rustfmt::skip]
spec! {
    opera_113_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    opera_128_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    chrome_96_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    edge_108_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    opera_95_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_132_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_102_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_106_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_107_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_114_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_127_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_101_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_108_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_83_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_143_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_83_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_106_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    chrome_103_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    edge_85_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36797
#[rustfmt::skip]
spec! {
    chrome_146_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36388
#[rustfmt::skip]
spec! {
    chrome_137_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36739
#[rustfmt::skip]
spec! {
    chrome_138_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35890
#[rustfmt::skip]
spec! {
    chrome_131_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36334
#[rustfmt::skip]
spec! {
    chrome_135_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36552
#[rustfmt::skip]
spec! {
    chrome_136_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36657
#[rustfmt::skip]
spec! {
    chrome_146_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36603
#[rustfmt::skip]
spec! {
    chrome_147_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_91_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    edge_134_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35998
#[rustfmt::skip]
spec! {
    edge_136_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36025
#[rustfmt::skip]
spec! {
    edge_137_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36115
#[rustfmt::skip]
spec! {
    edge_138_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=37445
#[rustfmt::skip]
spec! {
    chrome_139_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=37328
#[rustfmt::skip]
spec! {
    chrome_143_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35876
#[rustfmt::skip]
spec! {
    chrome_131_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=38206
#[rustfmt::skip]
spec! {
    chrome_142_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_130_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36039
#[rustfmt::skip]
spec! {
    edge_141_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36449
#[rustfmt::skip]
spec! {
    chrome_145_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35862
#[rustfmt::skip]
spec! {
    chrome_125_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36726
#[rustfmt::skip]
spec! {
    chrome_147_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36034
#[rustfmt::skip]
spec! {
    edge_140_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36047
#[rustfmt::skip]
spec! {
    edge_146_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36135
#[rustfmt::skip]
spec! {
    chrome_134_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36172
#[rustfmt::skip]
spec! {
    chrome_133_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36599
#[rustfmt::skip]
spec! {
    chrome_135_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36634
#[rustfmt::skip]
spec! {
    chrome_140_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36767
#[rustfmt::skip]
spec! {
    chrome_144_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36036
#[rustfmt::skip]
spec! {
    edge_139_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36827
#[rustfmt::skip]
spec! {
    chrome_142_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36064
#[rustfmt::skip]
spec! {
    edge_142_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36704
#[rustfmt::skip]
spec! {
    chrome_143_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36071
#[rustfmt::skip]
spec! {
    edge_143_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36083
#[rustfmt::skip]
spec! {
    edge_144_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36667
#[rustfmt::skip]
spec! {
    chrome_137_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_127_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_125_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_107_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_115_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35864
#[rustfmt::skip]
spec! {
    chrome_120_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36528
#[rustfmt::skip]
spec! {
    chrome_141_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    chrome_129_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35986
#[rustfmt::skip]
spec! {
    chrome_132_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_126_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35859
#[rustfmt::skip]
spec! {
    chrome_120_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35853
#[rustfmt::skip]
spec! {
    chrome_128_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35963
#[rustfmt::skip]
spec! {
    chrome_148_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36410
#[rustfmt::skip]
spec! {
    chrome_134_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35990
#[rustfmt::skip]
spec! {
    edge_147_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_69_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_108_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36646
#[rustfmt::skip]
spec! {
    chrome_144_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    edge_129_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35914
#[rustfmt::skip]
spec! {
    chrome_130_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    edge_131_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35985
#[rustfmt::skip]
spec! {
    chrome_132_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35973
#[rustfmt::skip]
spec! {
    edge_135_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_126_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35858
#[rustfmt::skip]
spec! {
    chrome_128_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    chrome_129_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_130_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36709
#[rustfmt::skip]
spec! {
    chrome_133_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36555
#[rustfmt::skip]
spec! {
    chrome_136_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36842
#[rustfmt::skip]
spec! {
    chrome_138_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35865
#[rustfmt::skip]
spec! {
    edge_140_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36598
#[rustfmt::skip]
spec! {
    chrome_141_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36816
#[rustfmt::skip]
spec! {
    chrome_145_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35854
#[rustfmt::skip]
spec! {
    chrome_127_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36530
#[rustfmt::skip]
spec! {
    chrome_139_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36032
#[rustfmt::skip]
spec! {
    edge_145_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    edge_132_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35863
#[rustfmt::skip]
spec! {
    edge_120_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_123_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35894
#[rustfmt::skip]
spec! {
    edge_136_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_100_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35862
#[rustfmt::skip]
spec! {
    chrome_112_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_122_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    chrome_123_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35860
#[rustfmt::skip]
spec! {
    chrome_124_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_133_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=36790
#[rustfmt::skip]
spec! {
    chrome_140_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35853
#[rustfmt::skip]
spec! {
    edge_134_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35873
#[rustfmt::skip]
spec! {
    edge_135_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35883
#[rustfmt::skip]
spec! {
    edge_146_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35893
#[rustfmt::skip]
spec! {
    edge_147_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35945
#[rustfmt::skip]
spec! {
    chrome_148_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_114_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    chrome_121_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    edge_133_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_124_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35870
#[rustfmt::skip]
spec! {
    opera_120_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    opera_122_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35857
#[rustfmt::skip]
spec! {
    opera_123_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35859
#[rustfmt::skip]
spec! {
    opera_124_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35856
#[rustfmt::skip]
spec! {
    opera_127_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35895
#[rustfmt::skip]
spec! {
    edge_148_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_129_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35853
#[rustfmt::skip]
spec! {
    edge_130_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35867
#[rustfmt::skip]
spec! {
    edge_137_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35867
#[rustfmt::skip]
spec! {
    edge_138_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35857
#[rustfmt::skip]
spec! {
    edge_141_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35869
#[rustfmt::skip]
spec! {
    edge_142_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35875
#[rustfmt::skip]
spec! {
    edge_143_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    edge_128_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    opera_115_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    opera_114_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35855
#[rustfmt::skip]
spec! {
    opera_129_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_83_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    chrome_86_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35854
#[rustfmt::skip]
spec! {
    chrome_108_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35869
#[rustfmt::skip]
spec! {
    chrome_91_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_117_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35861
#[rustfmt::skip]
spec! {
    chrome_112_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_114_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_119_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35861
#[rustfmt::skip]
spec! {
    edge_120_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    opera_116_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35856
#[rustfmt::skip]
spec! {
    opera_126_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_74_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    opera_120_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    opera_122_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35871
#[rustfmt::skip]
spec! {
    edge_139_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    opera_123_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35853
#[rustfmt::skip]
spec! {
    opera_124_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    opera_125_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35855
#[rustfmt::skip]
spec! {
    opera_126_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35860
#[rustfmt::skip]
spec! {
    opera_127_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35882
#[rustfmt::skip]
spec! {
    edge_144_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35853
#[rustfmt::skip]
spec! {
    opera_128_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35888
#[rustfmt::skip]
spec! {
    edge_145_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    opera_129_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    opera_130_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35854
#[rustfmt::skip]
spec! {
    edge_148_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    chrome_149_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_129_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_132_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_134_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_135_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_128_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_129_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_130_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_134_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_135_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    edge_128_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_129_macos_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    edge_130_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_131_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_133_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    edge_134_macos_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    edge_135_macos_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35855
#[rustfmt::skip]
spec! {
    chrome_142_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    opera_121_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35858
#[rustfmt::skip]
spec! {
    opera_125_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35870
#[rustfmt::skip]
spec! {
    opera_130_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    edge_149_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    chrome_619_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35855
#[rustfmt::skip]
spec! {
    chrome_140_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    opera_115_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    chrome_109_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], padding,
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2094
#[rustfmt::skip]
spec! {
    edge_116_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], padding,
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2094
#[rustfmt::skip]
spec! {
    chrome_97_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], padding,
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2095
#[rustfmt::skip]
spec! {
    chrome_80_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], padding,
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    chrome_110_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], padding,
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2094
#[rustfmt::skip]
spec! {
    opera_103_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], padding,
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2194
#[rustfmt::skip]
spec! {
    chrome_79_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], padding,
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2096
#[rustfmt::skip]
spec! {
    edge_92_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], padding,
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2096
#[rustfmt::skip]
spec! {
    edge_99_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], padding,
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    chrome_71_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], padding,
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    chrome_110_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], padding,
}

// ja4=t13d1517h2_8daaf6152771_46b8896bec77 obs=7041
#[rustfmt::skip]
spec! {
    opera_121_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""], raw[0x0029, ""],
}

// ja4=t13d1517h2_8daaf6152771_b1ff8ab2d16f obs=542
#[rustfmt::skip]
spec! {
    opera_95_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], raw[0xfe0d, ""], padding,
}

// ja4=t13d1517h2_8daaf6152771_fca9c764716e obs=74
#[rustfmt::skip]
spec! {
    chrome_107_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""], padding,
}

// ja4=t13d1616h2_e72c3b3287f1_e5627efa2ab1 obs=107
#[rustfmt::skip]
spec! {
    edge_111_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x1302,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          appsettings["h2", "http/1.1"], padding,
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_37_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8716
#[rustfmt::skip]
spec! {
    chrome_96_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_85_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_72_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8607
#[rustfmt::skip]
spec! {
    chrome_75_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_52_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    edge_101_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8694
#[rustfmt::skip]
spec! {
    chrome_102_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_66_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_77_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_98_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_61_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8608
#[rustfmt::skip]
spec! {
    chrome_73_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8610
#[rustfmt::skip]
spec! {
    chrome_75_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_74_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    edge_77_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_62_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8607
#[rustfmt::skip]
spec! {
    chrome_71_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8608
#[rustfmt::skip]
spec! {
    chrome_76_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_72_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    edge_99_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_16_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_19_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_22_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    edge_12_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_62_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_68_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    opera_62_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_76_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8660
#[rustfmt::skip]
spec! {
    chrome_87_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_55_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    opera_15_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8623
#[rustfmt::skip]
spec! {
    chrome_49_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8607
#[rustfmt::skip]
spec! {
    chrome_63_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    opera_62_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_76_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_72_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_99_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_12_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_20_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_22_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8607
#[rustfmt::skip]
spec! {
    opera_14_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    opera_20_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    opera_31_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_74_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    chrome_75_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    opera_12_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=806
#[rustfmt::skip]
spec! {
    edge_115_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_54_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_101_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_105_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_84_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_71_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_95_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_65_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_99_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_86_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_63_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_82_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_51_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_79_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_64_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_51_macos_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_52_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_69_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=805
#[rustfmt::skip]
spec! {
    chrome_75_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_89_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_100_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_93_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_98_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_61_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_82_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_77_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_65_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_92_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    chrome_95_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    edge_94_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    edge_98_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    edge_8_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d171100_5b57614c22b0_be53661681a4 obs=6
#[rustfmt::skip]
spec! {
    chrome_20_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303],
}

// ja4=t13d1711h2_5b57614c22b0_d811adc85aab obs=213
#[rustfmt::skip]
spec! {
    chrome_78_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"], status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303], raw[0x3374, ""],
}

// ja4=t13d1711h2_5b57614c22b0_e7c285222651 obs=181
#[rustfmt::skip]
spec! {
    chrome_118_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d1711h2_5b57614c22b0_e7c285222651 obs=181
#[rustfmt::skip]
spec! {
    edge_126_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d1711h2_5b57614c22b0_e7c285222651 obs=181
#[rustfmt::skip]
spec! {
    edge_126_macos_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d1711ht_ab0a1bf427ad_a29327ec888c obs=17
#[rustfmt::skip]
spec! {
    chrome_39_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=761
#[rustfmt::skip]
spec! {
    edge_109_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    opera_94_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d171300_5b57614c22b0_43ade6aba3df obs=17930
#[rustfmt::skip]
spec! {
    chrome_68_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], sct,
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding,
}

// ja4=t13d1713ht_5b57614c22b0_eca864cca44a obs=94
#[rustfmt::skip]
spec! {
    chrome_51_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"], status,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding,
}

// ja4=t13d1713ht_ab0a1bf427ad_ecd0401ec68b obs=220
#[rustfmt::skip]
spec! {
    edge_114_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["http/1.1"],
          sigalgs[0x0905, 0x0906, 0x0904, 0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x081a, 0x081b, 0x081c, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x0016, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=5968
#[rustfmt::skip]
spec! {
    edge_114_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          sct, keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0xfe0d, ""], rslimit[16385], raw[0x0022, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=5967
#[rustfmt::skip]
spec! {
    edge_109_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          sct, keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0xfe0d, ""], rslimit[16385], raw[0x0022, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=5977
#[rustfmt::skip]
spec! {
    edge_118_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"], status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          sct, keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0xfe0d, ""], rslimit[16385], raw[0x0022, ""],
}

// ja4=t13d201000_314f1408a5a6_e7c285222651 obs=171
#[rustfmt::skip]
spec! {
    edge_117_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x003c, 0xc009, 0xc00a, 0xc023, 0xc027,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d201000_314f1408a5a6_e7c285222651 obs=172
#[rustfmt::skip]
spec! {
    edge_18_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x003c, 0xc009, 0xc00a, 0xc023, 0xc027,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d201000_314f1408a5a6_e7c285222651 obs=171
#[rustfmt::skip]
spec! {
    chrome_95_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x003c, 0xc009, 0xc00a, 0xc023, 0xc027,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303],
}

// ja4=t13d201100_314f1408a5a6_ab7e3b40a677 obs=41
#[rustfmt::skip]
spec! {
    edge_88_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x003c, 0xc009, 0xc00a, 0xc023, 0xc027,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303], raw[0x0032, ""],
}

// ja4=t13d201100_314f1408a5a6_ab7e3b40a677 obs=41
#[rustfmt::skip]
spec! {
    chrome_66_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x003c, 0xc009, 0xc00a, 0xc023, 0xc027,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303], raw[0x0032, ""],
}

// ja4=t13d201100_314f1408a5a6_e5728521abd4 obs=82
#[rustfmt::skip]
spec! {
    chrome_91_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x003c, 0xc009, 0xc00a, 0xc023, 0xc027,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, status,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, keyshare[mlkem768, x25519], versions[0x0304, 0x0303], raw[0x0032, ""],
}

// ja4=t13d2212h2_231e334592e8_36bf25f296df obs=7536
#[rustfmt::skip]
spec! {
    edge_112_windows_desktop,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x009c, 0x009d, 0x002f,
             0x0035, 0x003c, 0x003d, 0x009e, 0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"],
          sigalgs[0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0201, 0x0403, 0x0503, 0x0203, 0x0202, 0x0601, 0x0603],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0031, ""],
}

// ja4=t13d2811h2_a01be8c064b6_1f22a2ca17c4 obs=5
#[rustfmt::skip]
spec! {
    chrome_118_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e, 0x009f, 0xc009,
             0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_1f22a2ca17c4 obs=101
#[rustfmt::skip]
spec! {
    chrome_44_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_1f22a2ca17c4 obs=99
#[rustfmt::skip]
spec! {
    chrome_47_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_518fb456ca59 obs=308
#[rustfmt::skip]
spec! {
    chrome_114_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x081a, 0x081b, 0x081c, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_518fb456ca59 obs=301
#[rustfmt::skip]
spec! {
    edge_114_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x081a, 0x081b, 0x081c, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_518fb456ca59 obs=318
#[rustfmt::skip]
spec! {
    opera_89_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x081a, 0x081b, 0x081c, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_518fb456ca59 obs=328
#[rustfmt::skip]
spec! {
    opera_89_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x081a, 0x081b, 0x081c, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_518fb456ca59 obs=308
#[rustfmt::skip]
spec! {
    edge_44_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x081a, 0x081b, 0x081c, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_5ac7197df9d2 obs=644
#[rustfmt::skip]
spec! {
    chrome_70_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_7379471da272 obs=35
#[rustfmt::skip]
spec! {
    chrome_36_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_7379471da272 obs=35
#[rustfmt::skip]
spec! {
    opera_99_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1186
#[rustfmt::skip]
spec! {
    chrome_46_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_59_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_25_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_29_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_20_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1186
#[rustfmt::skip]
spec! {
    chrome_58_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_21_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_35_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_26_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_44_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_40_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_17_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    chrome_17_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1187
#[rustfmt::skip]
spec! {
    opera_8_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1188
#[rustfmt::skip]
spec! {
    opera_8_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1189
#[rustfmt::skip]
spec! {
    opera_8_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    opera_8_windows_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1188
#[rustfmt::skip]
spec! {
    opera_8_windows_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1189
#[rustfmt::skip]
spec! {
    opera_8_windows_desktop_6,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1186
#[rustfmt::skip]
spec! {
    opera_9_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1191
#[rustfmt::skip]
spec! {
    opera_9_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1186
#[rustfmt::skip]
spec! {
    opera_9_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1187
#[rustfmt::skip]
spec! {
    opera_9_windows_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301100_1d37bd780c83_24695f2957a7 obs=219
#[rustfmt::skip]
spec! {
    chrome_70_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301100_1d37bd780c83_d41ae481755e obs=5253
#[rustfmt::skip]
spec! {
    chrome_99_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301100_1d37bd780c83_d41ae481755e obs=5427
#[rustfmt::skip]
spec! {
    chrome_84_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301100_1d37bd780c83_d41ae481755e obs=5256
#[rustfmt::skip]
spec! {
    chrome_61_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301100_1d37bd780c83_d41ae481755e obs=5253
#[rustfmt::skip]
spec! {
    chrome_34_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301100_1d37bd780c83_d41ae481755e obs=5253
#[rustfmt::skip]
spec! {
    chrome_103_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d301100_1d37bd780c83_d41ae481755e obs=5253
#[rustfmt::skip]
spec! {
    edge_100_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d3011h2_1d37bd780c83_5ac7197df9d2 obs=5646
#[rustfmt::skip]
spec! {
    opera_107_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0016, ""],
}

// ja4=t13d301200_1d37bd780c83_0d7ed806c34c obs=8
#[rustfmt::skip]
spec! {
    chrome_111_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d301200_1d37bd780c83_d339722ba4af obs=2383
#[rustfmt::skip]
spec! {
    chrome_50_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d301200_1d37bd780c83_d339722ba4af obs=2381
#[rustfmt::skip]
spec! {
    chrome_111_macos_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d301200_1d37bd780c83_ecd0401ec68b obs=14
#[rustfmt::skip]
spec! {
    chrome_31_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0905, 0x0906, 0x0904, 0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x081a, 0x081b, 0x081c, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], compress[brotli, zstd],
          raw[0x0016, ""],
}

// ja4=t13d3012h2_1d37bd780c83_b26ce05bbdd6 obs=28326
#[rustfmt::skip]
spec! {
    chrome_109_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d3012h2_1d37bd780c83_b26ce05bbdd6 obs=28318
#[rustfmt::skip]
spec! {
    chrome_121_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d3012h2_1d37bd780c83_b26ce05bbdd6 obs=28316
#[rustfmt::skip]
spec! {
    edge_131_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d3012h2_1d37bd780c83_b26ce05bbdd6 obs=28320
#[rustfmt::skip]
spec! {
    edge_121_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d3012h2_1d37bd780c83_b26ce05bbdd6 obs=28330
#[rustfmt::skip]
spec! {
    edge_91_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_89_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_41_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4458
#[rustfmt::skip]
spec! {
    chrome_50_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4472
#[rustfmt::skip]
spec! {
    chrome_83_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_60_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_86_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_67_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4458
#[rustfmt::skip]
spec! {
    chrome_88_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_92_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_54_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4458
#[rustfmt::skip]
spec! {
    chrome_50_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_93_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_73_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_80_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4465
#[rustfmt::skip]
spec! {
    chrome_81_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_93_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_69_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_91_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_90_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4458
#[rustfmt::skip]
spec! {
    chrome_14_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_27_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_3_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_81_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_33_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_11_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_4_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_48_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    edge_83_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    opera_69_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    opera_73_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    opera_74_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    edge_90_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    opera_77_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    opera_78_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    opera_34_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    opera_53_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    opera_54_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_19_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    opera_18_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_47_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4458
#[rustfmt::skip]
spec! {
    opera_34_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_48_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    opera_35_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_80_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_87_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_62_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4460
#[rustfmt::skip]
spec! {
    chrome_66_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4458
#[rustfmt::skip]
spec! {
    chrome_85_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_89_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_92_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_13_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_31_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    opera_28_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_42_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    opera_29_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_43_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_44_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_46_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_81_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4459
#[rustfmt::skip]
spec! {
    edge_81_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    chrome_30_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4458
#[rustfmt::skip]
spec! {
    opera_24_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4458
#[rustfmt::skip]
spec! {
    opera_30_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4458
#[rustfmt::skip]
spec! {
    opera_34_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    opera_35_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_49_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_39_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    chrome_43_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4458
#[rustfmt::skip]
spec! {
    opera_32_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    opera_11_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4458
#[rustfmt::skip]
spec! {
    opera_12_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_ce5650b735ce obs=1222
#[rustfmt::skip]
spec! {
    chrome_56_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_ce5650b735ce obs=1199
#[rustfmt::skip]
spec! {
    chrome_59_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["h2", "http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013ht_1d37bd780c83_1b3407e2c936 obs=155
#[rustfmt::skip]
spec! {
    edge_97_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1189
#[rustfmt::skip]
spec! {
    edge_14_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x009c, 0x009d, 0x002f,
             0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1186
#[rustfmt::skip]
spec! {
    chrome_42_macos_desktop,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x009c, 0x009d, 0x002f,
             0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1191
#[rustfmt::skip]
spec! {
    chrome_60_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x009c, 0x009d, 0x002f,
             0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1187
#[rustfmt::skip]
spec! {
    chrome_42_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x009c, 0x009d, 0x002f,
             0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1191
#[rustfmt::skip]
spec! {
    chrome_60_macos_desktop,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x009c, 0x009d, 0x002f,
             0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1190
#[rustfmt::skip]
spec! {
    opera_48_windows_desktop,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x009c, 0x009d, 0x002f,
             0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1194
#[rustfmt::skip]
spec! {
    chrome_61_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x009c, 0x009d, 0x002f,
             0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1188
#[rustfmt::skip]
spec! {
    chrome_59_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x009c, 0x009d, 0x002f,
             0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1188
#[rustfmt::skip]
spec! {
    chrome_60_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x009c, 0x009d, 0x002f,
             0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1189
#[rustfmt::skip]
spec! {
    chrome_61_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x009c, 0x009d, 0x002f,
             0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1187
#[rustfmt::skip]
spec! {
    opera_12_linux_desktop,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x009c, 0x009d, 0x002f,
             0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    chrome_15_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    chrome_132_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    chrome_124_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    chrome_127_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=463
#[rustfmt::skip]
spec! {
    chrome_124_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    chrome_113_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    chrome_125_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    chrome_129_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    chrome_124_macos_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    chrome_130_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=463
#[rustfmt::skip]
spec! {
    chrome_133_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    chrome_131_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    chrome_5_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=464
#[rustfmt::skip]
spec! {
    chrome_10_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    chrome_6_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    chrome_9_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=463
#[rustfmt::skip]
spec! {
    chrome_10_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    chrome_4_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    chrome_9_linux_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, status,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0011, ""],
          raw[0x0032, ""],
}

// ja4=t13d421000_49900ac2774e_1f22a2ca17c4 obs=54
#[rustfmt::skip]
spec! {
    opera_42_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0016, ""],
}

// ja4=t13d421000_49900ac2774e_a29327ec888c obs=18
#[rustfmt::skip]
spec! {
    chrome_82_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d421000_49900ac2774e_a29327ec888c obs=18
#[rustfmt::skip]
spec! {
    chrome_52_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d421000_49900ac2774e_a29327ec888c obs=18
#[rustfmt::skip]
spec! {
    chrome_57_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d421000_49900ac2774e_a29327ec888c obs=20
#[rustfmt::skip]
spec! {
    chrome_93_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d421000_49900ac2774e_a29327ec888c obs=18
#[rustfmt::skip]
spec! {
    chrome_69_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d421000_49900ac2774e_a29327ec888c obs=18
#[rustfmt::skip]
spec! {
    chrome_70_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d421000_49900ac2774e_a29327ec888c obs=18
#[rustfmt::skip]
spec! {
    chrome_80_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
}

// ja4=t13d421200_49900ac2774e_d339722ba4af obs=46
#[rustfmt::skip]
spec! {
    chrome_58_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d421200_49900ac2774e_d339722ba4af obs=45
#[rustfmt::skip]
spec! {
    chrome_78_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4212ht_49900ac2774e_b26ce05bbdd6 obs=860
#[rustfmt::skip]
spec! {
    chrome_36_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4212ht_49900ac2774e_b26ce05bbdd6 obs=859
#[rustfmt::skip]
spec! {
    chrome_57_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4212ht_49900ac2774e_b26ce05bbdd6 obs=860
#[rustfmt::skip]
spec! {
    chrome_57_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_86_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1568
#[rustfmt::skip]
spec! {
    chrome_84_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1567
#[rustfmt::skip]
spec! {
    chrome_107_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1579
#[rustfmt::skip]
spec! {
    chrome_101_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1569
#[rustfmt::skip]
spec! {
    chrome_90_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1568
#[rustfmt::skip]
spec! {
    chrome_98_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1567
#[rustfmt::skip]
spec! {
    chrome_85_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1573
#[rustfmt::skip]
spec! {
    chrome_95_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1566
#[rustfmt::skip]
spec! {
    chrome_88_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1568
#[rustfmt::skip]
spec! {
    chrome_79_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1569
#[rustfmt::skip]
spec! {
    chrome_102_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1566
#[rustfmt::skip]
spec! {
    chrome_35_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1575
#[rustfmt::skip]
spec! {
    chrome_88_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1568
#[rustfmt::skip]
spec! {
    chrome_101_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1570
#[rustfmt::skip]
spec! {
    chrome_27_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1566
#[rustfmt::skip]
spec! {
    chrome_32_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1567
#[rustfmt::skip]
spec! {
    chrome_24_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1569
#[rustfmt::skip]
spec! {
    chrome_96_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_77_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1567
#[rustfmt::skip]
spec! {
    chrome_94_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1578
#[rustfmt::skip]
spec! {
    chrome_99_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_104_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1566
#[rustfmt::skip]
spec! {
    chrome_29_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1571
#[rustfmt::skip]
spec! {
    chrome_93_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1572
#[rustfmt::skip]
spec! {
    chrome_65_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1576
#[rustfmt::skip]
spec! {
    brave_86_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1575
#[rustfmt::skip]
spec! {
    chrome_87_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1573
#[rustfmt::skip]
spec! {
    chrome_67_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1573
#[rustfmt::skip]
spec! {
    brave_80_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1567
#[rustfmt::skip]
spec! {
    brave_84_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1572
#[rustfmt::skip]
spec! {
    brave_89_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_63_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1570
#[rustfmt::skip]
spec! {
    chrome_66_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1576
#[rustfmt::skip]
spec! {
    chrome_90_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1568
#[rustfmt::skip]
spec! {
    chrome_97_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1573
#[rustfmt::skip]
spec! {
    brave_87_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1566
#[rustfmt::skip]
spec! {
    brave_88_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1580
#[rustfmt::skip]
spec! {
    chrome_89_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1576
#[rustfmt::skip]
spec! {
    chrome_94_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1580
#[rustfmt::skip]
spec! {
    chrome_100_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1570
#[rustfmt::skip]
spec! {
    brave_78_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1569
#[rustfmt::skip]
spec! {
    brave_83_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1570
#[rustfmt::skip]
spec! {
    brave_85_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    brave_90_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    brave_79_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    brave_81_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1566
#[rustfmt::skip]
spec! {
    chrome_82_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_77_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_32_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_24_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1567
#[rustfmt::skip]
spec! {
    brave_88_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1566
#[rustfmt::skip]
spec! {
    brave_87_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1569
#[rustfmt::skip]
spec! {
    brave_89_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1575
#[rustfmt::skip]
spec! {
    chrome_88_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1568
#[rustfmt::skip]
spec! {
    chrome_96_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1567
#[rustfmt::skip]
spec! {
    chrome_89_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1566
#[rustfmt::skip]
spec! {
    brave_78_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1569
#[rustfmt::skip]
spec! {
    brave_87_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1567
#[rustfmt::skip]
spec! {
    brave_89_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_82_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    brave_77_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1566
#[rustfmt::skip]
spec! {
    brave_79_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    brave_84_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    brave_86_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1570
#[rustfmt::skip]
spec! {
    chrome_67_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1566
#[rustfmt::skip]
spec! {
    brave_78_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1567
#[rustfmt::skip]
spec! {
    brave_88_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_28_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_68_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1567
#[rustfmt::skip]
spec! {
    chrome_69_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1573
#[rustfmt::skip]
spec! {
    chrome_84_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1567
#[rustfmt::skip]
spec! {
    chrome_88_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1568
#[rustfmt::skip]
spec! {
    chrome_95_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1566
#[rustfmt::skip]
spec! {
    chrome_97_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1567
#[rustfmt::skip]
spec! {
    brave_80_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1566
#[rustfmt::skip]
spec! {
    brave_84_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1567
#[rustfmt::skip]
spec! {
    chrome_103_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_73_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1567
#[rustfmt::skip]
spec! {
    chrome_75_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_78_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1568
#[rustfmt::skip]
spec! {
    chrome_81_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_24_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1566
#[rustfmt::skip]
spec! {
    chrome_100_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1568
#[rustfmt::skip]
spec! {
    chrome_29_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1566
#[rustfmt::skip]
spec! {
    chrome_94_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1568
#[rustfmt::skip]
spec! {
    chrome_28_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1566
#[rustfmt::skip]
spec! {
    chrome_87_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    brave_86_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1566
#[rustfmt::skip]
spec! {
    chrome_27_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_94_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    brave_78_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1566
#[rustfmt::skip]
spec! {
    brave_84_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    brave_88_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1570
#[rustfmt::skip]
spec! {
    chrome_100_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_105_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1567
#[rustfmt::skip]
spec! {
    chrome_108_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_74_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_84_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_85_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_89_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d4312ht_36cd39a4fcc1_58ed7828516f obs=1565
#[rustfmt::skip]
spec! {
    chrome_98_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009e,
             0x009f, 0x1304, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e,
             0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x0804, 0x080a, 0x0805, 0x080b, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0203, 0x0201],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], padding, raw[0x0016, ""],
          raw[0x0031, ""],
}

// ja4=t13d481000_c08b26b7ea02_5ac7197df9d2 obs=83
#[rustfmt::skip]
spec! {
    chrome_68_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac,
             0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0016, ""],
}

// ja4=t13d581000_363f866c7444_1f22a2ca17c4 obs=445
#[rustfmt::skip]
spec! {
    opera_45_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xc050, 0xc051, 0xc052, 0xc053, 0xc056, 0xc057, 0xc05c, 0xc05d, 0xc060,
             0xc061, 0xc09c, 0xc09d, 0xc09e, 0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad,
             0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0016, ""],
}

// ja4=t13d581000_363f866c7444_5ac7197df9d2 obs=104
#[rustfmt::skip]
spec! {
    chrome_63_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xc050, 0xc051, 0xc052, 0xc053, 0xc056, 0xc057, 0xc05c, 0xc05d, 0xc060,
             0xc061, 0xc09c, 0xc09d, 0xc09e, 0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad,
             0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0016, ""],
}

// ja4=t13d5811ht_363f866c7444_1f22a2ca17c4 obs=332
#[rustfmt::skip]
spec! {
    chrome_122_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x009c, 0x009d, 0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040,
             0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xc050, 0xc051, 0xc052, 0xc053, 0xc056, 0xc057, 0xc05c, 0xc05d, 0xc060,
             0xc061, 0xc09c, 0xc09d, 0xc09e, 0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad,
             0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[mlkem768, x25519, p256, p384], ecpf, ticket, alpn["http/1.1"],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          keyshare[mlkem768, x25519], psk, versions[0x0304, 0x0303], raw[0x0016, ""],
}
