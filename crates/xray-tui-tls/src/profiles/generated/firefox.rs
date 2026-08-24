//! Firefox hellos (the `firefox` wire template)
//!
//! Emitter output (`gen_specs.py --emit`); do not edit by hand.
//! Regeneration is byte-deterministic (`--selftest` verifies the
//! committed files match a fresh render).

use super::GenEntry;
use crate::fingerprints::{Browser, Device, Os};

#[rustfmt::skip]
pub const GENERATED: &[GenEntry] = &[
    GenEntry {
        name: "firefox_99_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 99,
        ja4: "t13d071000_c7abf191d1e4_e7c285222651",
        spec_fn: firefox_99_windows_desktop,
    },
    GenEntry {
        name: "firefox_115_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 115,
        ja4: "t13d1113h2_47af8f603342_f81080dfc557",
        spec_fn: firefox_115_android_desktop,
    },
    GenEntry {
        name: "firefox_139_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 139,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_139_macos_desktop,
    },
    GenEntry {
        name: "firefox_103_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 103,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_103_macos_desktop,
    },
    GenEntry {
        name: "firefox_121_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 121,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_121_macos_desktop,
    },
    GenEntry {
        name: "firefox_126_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_126_macos_desktop,
    },
    GenEntry {
        name: "firefox_150_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 150,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_150_macos_desktop,
    },
    GenEntry {
        name: "firefox_35_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 35,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_35_macos_desktop,
    },
    GenEntry {
        name: "firefox_40_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 40,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_40_macos_desktop,
    },
    GenEntry {
        name: "firefox_48_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 48,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_48_macos_desktop,
    },
    GenEntry {
        name: "firefox_52_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 52,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_52_macos_desktop,
    },
    GenEntry {
        name: "firefox_56_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 56,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_56_macos_desktop,
    },
    GenEntry {
        name: "firefox_57_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 57,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_57_macos_desktop,
    },
    GenEntry {
        name: "firefox_58_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 58,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_58_macos_desktop,
    },
    GenEntry {
        name: "firefox_104_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 104,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_104_macos_desktop,
    },
    GenEntry {
        name: "firefox_102_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 102,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_102_macos_desktop,
    },
    GenEntry {
        name: "firefox_103_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 103,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_103_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_111_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 111,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_111_macos_desktop,
    },
    GenEntry {
        name: "firefox_112_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 112,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_112_macos_desktop,
    },
    GenEntry {
        name: "firefox_107_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 107,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_107_macos_desktop,
    },
    GenEntry {
        name: "firefox_108_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 108,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_108_macos_desktop,
    },
    GenEntry {
        name: "firefox_108_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 108,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_108_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_103_macos_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 103,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_103_macos_desktop_3,
    },
    GenEntry {
        name: "firefox_111_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 111,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_111_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_35_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 35,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_35_windows_desktop,
    },
    GenEntry {
        name: "firefox_40_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 40,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_40_windows_desktop,
    },
    GenEntry {
        name: "firefox_48_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 48,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_48_windows_desktop,
    },
    GenEntry {
        name: "firefox_52_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 52,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_52_windows_desktop,
    },
    GenEntry {
        name: "firefox_57_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 57,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_57_windows_desktop,
    },
    GenEntry {
        name: "firefox_58_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 58,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_58_windows_desktop,
    },
    GenEntry {
        name: "firefox_112_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 112,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_112_windows_desktop,
    },
    GenEntry {
        name: "firefox_35_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 35,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_35_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_40_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 40,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_40_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_47_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 47,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_47_windows_desktop,
    },
    GenEntry {
        name: "firefox_48_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 48,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_48_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_52_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 52,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_52_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_57_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 57,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_57_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_105_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 105,
        ja4: "t13d131100_f57a46bbacb6_ab7e3b40a677",
        spec_fn: firefox_105_macos_desktop,
    },
    GenEntry {
        name: "firefox_122_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 122,
        ja4: "t13d131100_f57a46bbacb6_e5728521abd4",
        spec_fn: firefox_122_macos_desktop,
    },
    GenEntry {
        name: "firefox_117_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 117,
        ja4: "t13d1311h2_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_117_android_desktop,
    },
    GenEntry {
        name: "firefox_106_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 106,
        ja4: "t13d1311h2_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_106_macos_desktop,
    },
    GenEntry {
        name: "firefox_102_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 102,
        ja4: "t13d1311h2_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_102_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_104_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 104,
        ja4: "t13d1311h2_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_104_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_106_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 106,
        ja4: "t13d1311h2_f57a46bbacb6_e7c285222651",
        spec_fn: firefox_106_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_124_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d1313h2_07be0c029dc8_28652fe741a1",
        spec_fn: firefox_124_macos_desktop,
    },
    GenEntry {
        name: "firefox_128_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d1513h2_8daaf6152771_748f4c70de1c",
        spec_fn: firefox_128_android_desktop,
    },
    GenEntry {
        name: "firefox_124_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d1513h2_8daaf6152771_8b8ad545d541",
        spec_fn: firefox_124_android_desktop,
    },
    GenEntry {
        name: "firefox_127_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d1513h2_8daaf6152771_8b8ad545d541",
        spec_fn: firefox_127_android_desktop,
    },
    GenEntry {
        name: "firefox_72_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 72,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: firefox_72_macos_desktop,
    },
    GenEntry {
        name: "firefox_46_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 46,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: firefox_46_macos_desktop,
    },
    GenEntry {
        name: "firefox_64_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 64,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: firefox_64_macos_desktop,
    },
    GenEntry {
        name: "firefox_51_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 51,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: firefox_51_macos_desktop,
    },
    GenEntry {
        name: "firefox_55_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 55,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: firefox_55_macos_desktop,
    },
    GenEntry {
        name: "firefox_50_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 50,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: firefox_50_macos_desktop,
    },
    GenEntry {
        name: "firefox_46_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 46,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: firefox_46_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_54_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 54,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: firefox_54_windows_desktop,
    },
    GenEntry {
        name: "firefox_60_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 60,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: firefox_60_windows_desktop,
    },
    GenEntry {
        name: "firefox_58_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 58,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: firefox_58_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_46_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 46,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: firefox_46_windows_desktop,
    },
    GenEntry {
        name: "firefox_60_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 60,
        ja4: "t13d1514ht_8daaf6152771_a5b99884f7f5",
        spec_fn: firefox_60_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_120_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 120,
        ja4: "t13d1515h2_8daaf6152771_6a09c78d0dc2",
        spec_fn: firefox_120_android_desktop,
    },
    GenEntry {
        name: "firefox_144_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 144,
        ja4: "t13d1515h2_8daaf6152771_a54fffd0eb61",
        spec_fn: firefox_144_android_desktop,
    },
    GenEntry {
        name: "firefox_148_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 148,
        ja4: "t13d1515h2_8daaf6152771_a54fffd0eb61",
        spec_fn: firefox_148_android_desktop,
    },
    GenEntry {
        name: "firefox_64_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 64,
        ja4: "t13d1515ht_8daaf6152771_4769d65a485e",
        spec_fn: firefox_64_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_53_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 53,
        ja4: "t13d1515ht_8daaf6152771_4769d65a485e",
        spec_fn: firefox_53_windows_desktop,
    },
    GenEntry {
        name: "firefox_59_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 59,
        ja4: "t13d1515ht_8daaf6152771_4769d65a485e",
        spec_fn: firefox_59_windows_desktop,
    },
    GenEntry {
        name: "firefox_72_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 72,
        ja4: "t13d1515ht_8daaf6152771_4769d65a485e",
        spec_fn: firefox_72_windows_desktop,
    },
    GenEntry {
        name: "firefox_71_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 71,
        ja4: "t13d1515ht_8daaf6152771_4769d65a485e",
        spec_fn: firefox_71_windows_desktop,
    },
    GenEntry {
        name: "firefox_70_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 70,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_70_macos_desktop,
    },
    GenEntry {
        name: "firefox_60_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 60,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_60_macos_desktop,
    },
    GenEntry {
        name: "firefox_62_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 62,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_62_macos_desktop,
    },
    GenEntry {
        name: "firefox_61_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 61,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_61_macos_desktop,
    },
    GenEntry {
        name: "firefox_53_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 53,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_53_macos_desktop,
    },
    GenEntry {
        name: "firefox_59_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 59,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_59_macos_desktop,
    },
    GenEntry {
        name: "firefox_59_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 59,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_59_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_54_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 54,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_54_macos_desktop,
    },
    GenEntry {
        name: "firefox_66_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 66,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_66_macos_desktop,
    },
    GenEntry {
        name: "firefox_73_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 73,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_73_macos_desktop,
    },
    GenEntry {
        name: "firefox_65_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 65,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_65_macos_desktop,
    },
    GenEntry {
        name: "firefox_71_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 71,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_71_macos_desktop,
    },
    GenEntry {
        name: "firefox_51_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 51,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_51_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_74_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 74,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_74_macos_desktop,
    },
    GenEntry {
        name: "firefox_54_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 54,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_54_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_52_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 52,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_52_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_48_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 48,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_48_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_72_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 72,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_72_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_55_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 55,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_55_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_56_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 56,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_56_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_71_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 71,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_71_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_59_macos_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 59,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_59_macos_desktop_3,
    },
    GenEntry {
        name: "firefox_51_macos_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 51,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_51_macos_desktop_3,
    },
    GenEntry {
        name: "firefox_67_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 67,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_67_windows_desktop,
    },
    GenEntry {
        name: "firefox_64_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 64,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_64_windows_desktop,
    },
    GenEntry {
        name: "firefox_46_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 46,
        ja4: "t13d1515ht_8daaf6152771_de216e0ff992",
        spec_fn: firefox_46_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_127_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: firefox_127_windows_desktop,
    },
    GenEntry {
        name: "firefox_133_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: firefox_133_windows_desktop,
    },
    GenEntry {
        name: "firefox_102_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 102,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: firefox_102_windows_desktop,
    },
    GenEntry {
        name: "firefox_115_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 115,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: firefox_115_windows_desktop,
    },
    GenEntry {
        name: "firefox_128_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: firefox_128_windows_desktop,
    },
    GenEntry {
        name: "firefox_132_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: firefox_132_windows_desktop,
    },
    GenEntry {
        name: "firefox_136_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 136,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: firefox_136_windows_desktop,
    },
    GenEntry {
        name: "firefox_139_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 139,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: firefox_139_windows_desktop,
    },
    GenEntry {
        name: "firefox_137_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 137,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: firefox_137_windows_desktop,
    },
    GenEntry {
        name: "firefox_126_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: firefox_126_windows_desktop,
    },
    GenEntry {
        name: "firefox_129_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: firefox_129_windows_desktop,
    },
    GenEntry {
        name: "firefox_130_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: firefox_130_windows_desktop,
    },
    GenEntry {
        name: "firefox_117_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 117,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: firefox_117_windows_desktop,
    },
    GenEntry {
        name: "firefox_118_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 118,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: firefox_118_windows_desktop,
    },
    GenEntry {
        name: "firefox_122_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 122,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: firefox_122_windows_desktop,
    },
    GenEntry {
        name: "firefox_115_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 115,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: firefox_115_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_134_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: firefox_134_windows_desktop,
    },
    GenEntry {
        name: "firefox_135_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 135,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: firefox_135_windows_desktop,
    },
    GenEntry {
        name: "firefox_144_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 144,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: firefox_144_windows_desktop,
    },
    GenEntry {
        name: "firefox_148_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 148,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: firefox_148_macos_desktop,
    },
    GenEntry {
        name: "firefox_148_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 148,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: firefox_148_windows_desktop,
    },
    GenEntry {
        name: "firefox_143_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 143,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: firefox_143_windows_desktop,
    },
    GenEntry {
        name: "firefox_141_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 141,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: firefox_141_windows_desktop,
    },
    GenEntry {
        name: "firefox_119_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 119,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: firefox_119_windows_desktop,
    },
    GenEntry {
        name: "firefox_109_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 109,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: firefox_109_windows_desktop,
    },
    GenEntry {
        name: "firefox_113_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 113,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: firefox_113_windows_desktop,
    },
    GenEntry {
        name: "firefox_145_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 145,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: firefox_145_windows_desktop,
    },
    GenEntry {
        name: "firefox_59_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 59,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: firefox_59_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_135_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 135,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: firefox_135_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_131_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 131,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: firefox_131_windows_desktop,
    },
    GenEntry {
        name: "firefox_127_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d1516h2_8daaf6152771_e5627efa2ab1",
        spec_fn: firefox_127_android_desktop_2,
    },
    GenEntry {
        name: "firefox_140_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d1517h2_8daaf6152771_68c5a8c2958d",
        spec_fn: firefox_140_android_desktop,
    },
    GenEntry {
        name: "firefox_151_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 151,
        ja4: "t13d1517h2_8daaf6152771_68c5a8c2958d",
        spec_fn: firefox_151_macos_desktop,
    },
    GenEntry {
        name: "firefox_152_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 152,
        ja4: "t13d1517h2_8daaf6152771_68c5a8c2958d",
        spec_fn: firefox_152_macos_desktop,
    },
    GenEntry {
        name: "firefox_151_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 151,
        ja4: "t13d1617h2_86a278354501_3cbfd9057e0d",
        spec_fn: firefox_151_windows_desktop,
    },
    GenEntry {
        name: "firefox_3_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 3,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: firefox_3_macos_desktop,
    },
    GenEntry {
        name: "firefox_20_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 20,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: firefox_20_macos_desktop,
    },
    GenEntry {
        name: "firefox_56_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 56,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: firefox_56_windows_desktop,
    },
    GenEntry {
        name: "firefox_10_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 10,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: firefox_10_windows_desktop,
    },
    GenEntry {
        name: "firefox_3_linux_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Linux),
        device: Device::Desktop,
        major: 3,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: firefox_3_linux_desktop,
    },
    GenEntry {
        name: "firefox_2_linux_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Linux),
        device: Device::Desktop,
        major: 2,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: firefox_2_linux_desktop,
    },
    GenEntry {
        name: "firefox_48_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 48,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: firefox_48_android_desktop,
    },
    GenEntry {
        name: "firefox_98_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 98,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: firefox_98_macos_desktop,
    },
    GenEntry {
        name: "firefox_86_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 86,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: firefox_86_windows_desktop,
    },
    GenEntry {
        name: "firefox_110_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 110,
        ja4: "t13d171000_5b57614c22b0_e7c285222651",
        spec_fn: firefox_110_windows_desktop,
    },
    GenEntry {
        name: "firefox_65_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 65,
        ja4: "t13d1710h2_5b57614c22b0_97f8aa674fd9",
        spec_fn: firefox_65_windows_desktop,
    },
    GenEntry {
        name: "firefox_116_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 116,
        ja4: "t13d171100_ab0a1bf427ad_d41ae481755e",
        spec_fn: firefox_116_windows_desktop,
    },
    GenEntry {
        name: "firefox_91_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 91,
        ja4: "t13d171100_ab0a1bf427ad_d41ae481755e",
        spec_fn: firefox_91_windows_desktop,
    },
    GenEntry {
        name: "firefox_138_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 138,
        ja4: "t13d1712h2_5b57614c22b0_ef7df7f74e48",
        spec_fn: firefox_138_windows_desktop,
    },
    GenEntry {
        name: "firefox_137_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 137,
        ja4: "t13d1712h2_5b57614c22b0_ef7df7f74e48",
        spec_fn: firefox_137_macos_desktop,
    },
    GenEntry {
        name: "firefox_58_windows_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 58,
        ja4: "t13d1712ht_95e1cefdbe28_d41ae481755e",
        spec_fn: firefox_58_windows_desktop_3,
    },
    GenEntry {
        name: "firefox_115_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 115,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: firefox_115_android_desktop_2,
    },
    GenEntry {
        name: "firefox_128_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: firefox_128_android_desktop_2,
    },
    GenEntry {
        name: "firefox_108_macos_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 108,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: firefox_108_macos_desktop_3,
    },
    GenEntry {
        name: "firefox_129_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: firefox_129_macos_desktop,
    },
    GenEntry {
        name: "firefox_110_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 110,
        ja4: "t13d1712ht_ab0a1bf427ad_b26ce05bbdd6",
        spec_fn: firefox_110_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_89_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 89,
        ja4: "t13d1712ht_ab0a1bf427ad_d41ae481755e",
        spec_fn: firefox_89_android_desktop,
    },
    GenEntry {
        name: "firefox_145_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 145,
        ja4: "t13d171300_5b57614c22b0_43ade6aba3df",
        spec_fn: firefox_145_macos_desktop,
    },
    GenEntry {
        name: "firefox_129_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1713h2_5b57614c22b0_748f4c70de1c",
        spec_fn: firefox_129_android_desktop,
    },
    GenEntry {
        name: "firefox_130_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d1713h2_5b57614c22b0_748f4c70de1c",
        spec_fn: firefox_130_android_desktop,
    },
    GenEntry {
        name: "firefox_102_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 102,
        ja4: "t13d1713h2_5b57614c22b0_f81080dfc557",
        spec_fn: firefox_102_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_145_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 145,
        ja4: "t13d1714h2_5b57614c22b0_3dd24b5ebec4",
        spec_fn: firefox_145_android_desktop,
    },
    GenEntry {
        name: "firefox_137_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 137,
        ja4: "t13d1714h2_5b57614c22b0_3dd24b5ebec4",
        spec_fn: firefox_137_android_desktop,
    },
    GenEntry {
        name: "firefox_140_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d1714h2_5b57614c22b0_3dd24b5ebec4",
        spec_fn: firefox_140_android_desktop_2,
    },
    GenEntry {
        name: "firefox_147_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 147,
        ja4: "t13d1714h2_5b57614c22b0_3dd24b5ebec4",
        spec_fn: firefox_147_android_desktop,
    },
    GenEntry {
        name: "firefox_137_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 137,
        ja4: "t13d1714h2_5b57614c22b0_3dd24b5ebec4",
        spec_fn: firefox_137_android_desktop_2,
    },
    GenEntry {
        name: "firefox_138_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 138,
        ja4: "t13d1714h2_5b57614c22b0_3dd24b5ebec4",
        spec_fn: firefox_138_android_desktop,
    },
    GenEntry {
        name: "firefox_134_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1714h2_5b57614c22b0_3dd24b5ebec4",
        spec_fn: firefox_134_android_desktop,
    },
    GenEntry {
        name: "firefox_139_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 139,
        ja4: "t13d1714h2_5b57614c22b0_3dd24b5ebec4",
        spec_fn: firefox_139_android_desktop,
    },
    GenEntry {
        name: "firefox_148_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 148,
        ja4: "t13d1714h2_5b57614c22b0_3dd24b5ebec4",
        spec_fn: firefox_148_android_desktop_2,
    },
    GenEntry {
        name: "firefox_149_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 149,
        ja4: "t13d1714h2_5b57614c22b0_3dd24b5ebec4",
        spec_fn: firefox_149_android_desktop,
    },
    GenEntry {
        name: "firefox_139_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 139,
        ja4: "t13d1714h2_5b57614c22b0_3dd24b5ebec4",
        spec_fn: firefox_139_android_desktop_2,
    },
    GenEntry {
        name: "firefox_132_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1714h2_5b57614c22b0_3dd24b5ebec4",
        spec_fn: firefox_132_macos_desktop,
    },
    GenEntry {
        name: "firefox_109_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 109,
        ja4: "t13d1715h2_5b57614c22b0_3d5424432f57",
        spec_fn: firefox_109_macos_desktop,
    },
    GenEntry {
        name: "firefox_78_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 78,
        ja4: "t13d1715h2_5b57614c22b0_3d5424432f57",
        spec_fn: firefox_78_windows_desktop,
    },
    GenEntry {
        name: "firefox_115_windows_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 115,
        ja4: "t13d1715h2_5b57614c22b0_3d5424432f57",
        spec_fn: firefox_115_windows_desktop_3,
    },
    GenEntry {
        name: "firefox_116_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 116,
        ja4: "t13d1715h2_5b57614c22b0_3d5424432f57",
        spec_fn: firefox_116_android_desktop,
    },
    GenEntry {
        name: "firefox_80_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 80,
        ja4: "t13d1715h2_5b57614c22b0_3d5424432f57",
        spec_fn: firefox_80_android_desktop,
    },
    GenEntry {
        name: "firefox_107_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 107,
        ja4: "t13d1715h2_5b57614c22b0_3d5424432f57",
        spec_fn: firefox_107_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_108_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 108,
        ja4: "t13d1715h2_5b57614c22b0_3d5424432f57",
        spec_fn: firefox_108_windows_desktop,
    },
    GenEntry {
        name: "firefox_95_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 95,
        ja4: "t13d1715h2_5b57614c22b0_3d5424432f57",
        spec_fn: firefox_95_windows_desktop,
    },
    GenEntry {
        name: "firefox_102_windows_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 102,
        ja4: "t13d1715h2_5b57614c22b0_3d5424432f57",
        spec_fn: firefox_102_windows_desktop_3,
    },
    GenEntry {
        name: "firefox_127_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_127_macos_desktop,
    },
    GenEntry {
        name: "firefox_129_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_129_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_130_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_130_android_desktop_2,
    },
    GenEntry {
        name: "firefox_128_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_128_android_desktop_3,
    },
    GenEntry {
        name: "firefox_131_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 131,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_131_android_desktop,
    },
    GenEntry {
        name: "firefox_130_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_130_macos_desktop,
    },
    GenEntry {
        name: "firefox_131_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 131,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_131_macos_desktop,
    },
    GenEntry {
        name: "firefox_149_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 149,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_149_android_desktop_2,
    },
    GenEntry {
        name: "firefox_132_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_132_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_126_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_126_android_desktop,
    },
    GenEntry {
        name: "firefox_130_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_130_android_desktop_3,
    },
    GenEntry {
        name: "firefox_126_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_126_android_desktop_2,
    },
    GenEntry {
        name: "firefox_127_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_127_android_desktop_3,
    },
    GenEntry {
        name: "firefox_129_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_129_android_desktop_2,
    },
    GenEntry {
        name: "firefox_124_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_124_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_126_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_126_android_desktop_3,
    },
    GenEntry {
        name: "firefox_129_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_129_android_desktop_3,
    },
    GenEntry {
        name: "firefox_121_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 121,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_121_android_desktop,
    },
    GenEntry {
        name: "firefox_126_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_126_android_desktop_4,
    },
    GenEntry {
        name: "firefox_130_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 130,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_130_android_desktop_4,
    },
    GenEntry {
        name: "firefox_127_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_127_android_desktop_4,
    },
    GenEntry {
        name: "firefox_129_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 129,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_129_android_desktop_4,
    },
    GenEntry {
        name: "firefox_124_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_124_android_desktop_2,
    },
    GenEntry {
        name: "firefox_125_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 125,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_125_android_desktop,
    },
    GenEntry {
        name: "firefox_126_android_desktop_5",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_126_android_desktop_5,
    },
    GenEntry {
        name: "firefox_131_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 131,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_131_android_desktop_2,
    },
    GenEntry {
        name: "firefox_125_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 125,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_125_android_desktop_2,
    },
    GenEntry {
        name: "firefox_126_android_desktop_6",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_126_android_desktop_6,
    },
    GenEntry {
        name: "firefox_120_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 120,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_120_macos_desktop,
    },
    GenEntry {
        name: "firefox_125_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 125,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_125_macos_desktop,
    },
    GenEntry {
        name: "firefox_120_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 120,
        ja4: "t13d1715h2_5b57614c22b0_5c2c66f702b0",
        spec_fn: firefox_120_windows_desktop,
    },
    GenEntry {
        name: "firefox_86_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 86,
        ja4: "t13d1715h2_5b57614c22b0_a54fffd0eb61",
        spec_fn: firefox_86_macos_desktop,
    },
    GenEntry {
        name: "firefox_139_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 139,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_139_android_desktop_3,
    },
    GenEntry {
        name: "firefox_132_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_132_android_desktop,
    },
    GenEntry {
        name: "firefox_136_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 136,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_136_android_desktop,
    },
    GenEntry {
        name: "firefox_132_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_132_android_desktop_2,
    },
    GenEntry {
        name: "firefox_134_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_134_android_desktop_2,
    },
    GenEntry {
        name: "firefox_138_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 138,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_138_android_desktop_2,
    },
    GenEntry {
        name: "firefox_139_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 139,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_139_android_desktop_4,
    },
    GenEntry {
        name: "firefox_143_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 143,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_143_android_desktop,
    },
    GenEntry {
        name: "firefox_150_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 150,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_150_android_desktop,
    },
    GenEntry {
        name: "firefox_150_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 150,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_150_android_desktop_2,
    },
    GenEntry {
        name: "firefox_147_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 147,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_147_android_desktop_2,
    },
    GenEntry {
        name: "firefox_134_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_134_android_desktop_3,
    },
    GenEntry {
        name: "firefox_132_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_132_android_desktop_3,
    },
    GenEntry {
        name: "firefox_133_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_133_android_desktop,
    },
    GenEntry {
        name: "firefox_137_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 137,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_137_android_desktop_3,
    },
    GenEntry {
        name: "firefox_142_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 142,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_142_android_desktop,
    },
    GenEntry {
        name: "firefox_145_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 145,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_145_android_desktop_2,
    },
    GenEntry {
        name: "firefox_133_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_133_android_desktop_2,
    },
    GenEntry {
        name: "firefox_135_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 135,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_135_android_desktop,
    },
    GenEntry {
        name: "firefox_136_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 136,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_136_android_desktop_2,
    },
    GenEntry {
        name: "firefox_141_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 141,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_141_android_desktop,
    },
    GenEntry {
        name: "firefox_135_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 135,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_135_android_desktop_2,
    },
    GenEntry {
        name: "firefox_142_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 142,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_142_android_desktop_2,
    },
    GenEntry {
        name: "firefox_148_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 148,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_148_android_desktop_3,
    },
    GenEntry {
        name: "firefox_138_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 138,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_138_android_desktop_3,
    },
    GenEntry {
        name: "firefox_144_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 144,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_144_android_desktop_2,
    },
    GenEntry {
        name: "firefox_146_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 146,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_146_android_desktop,
    },
    GenEntry {
        name: "firefox_148_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 148,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_148_android_desktop_4,
    },
    GenEntry {
        name: "firefox_133_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_133_android_desktop_3,
    },
    GenEntry {
        name: "firefox_135_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 135,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_135_android_desktop_3,
    },
    GenEntry {
        name: "firefox_147_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 147,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_147_android_desktop_3,
    },
    GenEntry {
        name: "firefox_134_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_134_android_desktop_4,
    },
    GenEntry {
        name: "firefox_134_android_desktop_5",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_134_android_desktop_5,
    },
    GenEntry {
        name: "firefox_149_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 149,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_149_android_desktop_3,
    },
    GenEntry {
        name: "firefox_133_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_133_android_desktop_4,
    },
    GenEntry {
        name: "firefox_133_android_desktop_5",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_133_android_desktop_5,
    },
    GenEntry {
        name: "firefox_140_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_140_android_desktop_3,
    },
    GenEntry {
        name: "firefox_147_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 147,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_147_android_desktop_4,
    },
    GenEntry {
        name: "firefox_135_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 135,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_135_android_desktop_4,
    },
    GenEntry {
        name: "firefox_138_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 138,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_138_android_desktop_4,
    },
    GenEntry {
        name: "firefox_141_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 141,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_141_android_desktop_2,
    },
    GenEntry {
        name: "firefox_144_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 144,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_144_android_desktop_3,
    },
    GenEntry {
        name: "firefox_137_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 137,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_137_android_desktop_4,
    },
    GenEntry {
        name: "firefox_142_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 142,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_142_android_desktop_3,
    },
    GenEntry {
        name: "firefox_143_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 143,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_143_android_desktop_2,
    },
    GenEntry {
        name: "firefox_145_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 145,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_145_android_desktop_3,
    },
    GenEntry {
        name: "firefox_146_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 146,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_146_android_desktop_2,
    },
    GenEntry {
        name: "firefox_147_android_desktop_5",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 147,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_147_android_desktop_5,
    },
    GenEntry {
        name: "firefox_132_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_132_android_desktop_4,
    },
    GenEntry {
        name: "firefox_133_android_desktop_6",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_133_android_desktop_6,
    },
    GenEntry {
        name: "firefox_137_android_desktop_5",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 137,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_137_android_desktop_5,
    },
    GenEntry {
        name: "firefox_141_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 141,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_141_android_desktop_3,
    },
    GenEntry {
        name: "firefox_143_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 143,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_143_android_desktop_3,
    },
    GenEntry {
        name: "firefox_144_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 144,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_144_android_desktop_4,
    },
    GenEntry {
        name: "firefox_145_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 145,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_145_android_desktop_4,
    },
    GenEntry {
        name: "firefox_146_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 146,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_146_android_desktop_3,
    },
    GenEntry {
        name: "firefox_150_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 150,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_150_android_desktop_3,
    },
    GenEntry {
        name: "firefox_140_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_140_android_desktop_4,
    },
    GenEntry {
        name: "firefox_141_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 141,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_141_android_desktop_4,
    },
    GenEntry {
        name: "firefox_145_android_desktop_5",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 145,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_145_android_desktop_5,
    },
    GenEntry {
        name: "firefox_132_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_132_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_132_windows_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: firefox_132_windows_desktop_3,
    },
    GenEntry {
        name: "firefox_134_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_134_macos_desktop,
    },
    GenEntry {
        name: "firefox_142_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 142,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_142_macos_desktop,
    },
    GenEntry {
        name: "firefox_144_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 144,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_144_macos_desktop,
    },
    GenEntry {
        name: "firefox_138_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 138,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_138_macos_desktop,
    },
    GenEntry {
        name: "firefox_142_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 142,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_142_windows_desktop,
    },
    GenEntry {
        name: "firefox_150_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 150,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_150_windows_desktop,
    },
    GenEntry {
        name: "firefox_133_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_133_macos_desktop,
    },
    GenEntry {
        name: "firefox_128_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 128,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_128_macos_desktop,
    },
    GenEntry {
        name: "firefox_143_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 143,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_143_macos_desktop,
    },
    GenEntry {
        name: "firefox_140_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_140_windows_desktop,
    },
    GenEntry {
        name: "firefox_146_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 146,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_146_windows_desktop,
    },
    GenEntry {
        name: "firefox_147_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 147,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_147_windows_desktop,
    },
    GenEntry {
        name: "firefox_135_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 135,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_135_macos_desktop,
    },
    GenEntry {
        name: "firefox_141_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 141,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_141_macos_desktop,
    },
    GenEntry {
        name: "firefox_146_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 146,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_146_macos_desktop,
    },
    GenEntry {
        name: "firefox_147_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 147,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_147_macos_desktop,
    },
    GenEntry {
        name: "firefox_149_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 149,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_149_windows_desktop,
    },
    GenEntry {
        name: "firefox_140_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_140_macos_desktop,
    },
    GenEntry {
        name: "firefox_136_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 136,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_136_macos_desktop,
    },
    GenEntry {
        name: "firefox_149_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 149,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_149_macos_desktop,
    },
    GenEntry {
        name: "firefox_124_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 124,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_124_windows_desktop,
    },
    GenEntry {
        name: "firefox_136_android_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 136,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_136_android_desktop_3,
    },
    GenEntry {
        name: "firefox_140_android_desktop_5",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 140,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_140_android_desktop_5,
    },
    GenEntry {
        name: "firefox_142_android_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 142,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_142_android_desktop_4,
    },
    GenEntry {
        name: "firefox_144_android_desktop_5",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 144,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_144_android_desktop_5,
    },
    GenEntry {
        name: "firefox_134_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d1717h2_5b57614c22b0_3cbfd9057e0d",
        spec_fn: firefox_134_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_97_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 97,
        ja4: "t13d201000_314f1408a5a6_e7c285222651",
        spec_fn: firefox_97_windows_desktop,
    },
    GenEntry {
        name: "firefox_137_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 137,
        ja4: "t13d201000_314f1408a5a6_e7c285222651",
        spec_fn: firefox_137_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_80_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 80,
        ja4: "t13d201100_314f1408a5a6_ab7e3b40a677",
        spec_fn: firefox_80_windows_desktop,
    },
    GenEntry {
        name: "firefox_82_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 82,
        ja4: "t13d201100_314f1408a5a6_ab7e3b40a677",
        spec_fn: firefox_82_windows_desktop,
    },
    GenEntry {
        name: "firefox_70_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 70,
        ja4: "t13d201100_314f1408a5a6_ab7e3b40a677",
        spec_fn: firefox_70_windows_desktop,
    },
    GenEntry {
        name: "firefox_64_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 64,
        ja4: "t13d201100_314f1408a5a6_e5728521abd4",
        spec_fn: firefox_64_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_73_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 73,
        ja4: "t13d201100_314f1408a5a6_e5728521abd4",
        spec_fn: firefox_73_windows_desktop,
    },
    GenEntry {
        name: "firefox_112_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 112,
        ja4: "t13d201100_314f1408a5a6_e5728521abd4",
        spec_fn: firefox_112_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_81_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 81,
        ja4: "t13d221000_231e334592e8_29829a46703f",
        spec_fn: firefox_81_windows_desktop,
    },
    GenEntry {
        name: "firefox_33_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 33,
        ja4: "t13d301000_1d37bd780c83_1f22a2ca17c4",
        spec_fn: firefox_33_windows_desktop,
    },
    GenEntry {
        name: "firefox_114_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 114,
        ja4: "t13d301000_1d37bd780c83_518fb456ca59",
        spec_fn: firefox_114_macos_desktop,
    },
    GenEntry {
        name: "firefox_114_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 114,
        ja4: "t13d301000_1d37bd780c83_518fb456ca59",
        spec_fn: firefox_114_windows_desktop,
    },
    GenEntry {
        name: "firefox_62_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 62,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_62_android_desktop,
    },
    GenEntry {
        name: "firefox_57_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 57,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_57_android_desktop,
    },
    GenEntry {
        name: "firefox_60_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 60,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_60_android_desktop,
    },
    GenEntry {
        name: "firefox_22_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 22,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_22_android_desktop,
    },
    GenEntry {
        name: "firefox_19_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 19,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_19_android_desktop,
    },
    GenEntry {
        name: "firefox_11_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 11,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_11_android_desktop,
    },
    GenEntry {
        name: "firefox_18_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 18,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_18_android_desktop,
    },
    GenEntry {
        name: "firefox_8_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 8,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_8_android_desktop,
    },
    GenEntry {
        name: "firefox_66_android_tablet",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 66,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_66_android_tablet,
    },
    GenEntry {
        name: "firefox_65_android_tablet",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 65,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_65_android_tablet,
    },
    GenEntry {
        name: "firefox_36_android_tablet",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Tablet,
        major: 36,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_36_android_tablet,
    },
    GenEntry {
        name: "firefox_17_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 17,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_17_android_desktop,
    },
    GenEntry {
        name: "firefox_41_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 41,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_41_android_desktop,
    },
    GenEntry {
        name: "firefox_33_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 33,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_33_android_desktop,
    },
    GenEntry {
        name: "firefox_43_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 43,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_43_android_desktop,
    },
    GenEntry {
        name: "firefox_8_android_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 8,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_8_android_desktop_2,
    },
    GenEntry {
        name: "firefox_31_android_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Android),
        device: Device::Desktop,
        major: 31,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_31_android_desktop,
    },
    GenEntry {
        name: "firefox_5_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 5,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_5_macos_desktop,
    },
    GenEntry {
        name: "firefox_11_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 11,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_11_macos_desktop,
    },
    GenEntry {
        name: "firefox_4_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 4,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_4_macos_desktop,
    },
    GenEntry {
        name: "firefox_13_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 13,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_13_macos_desktop,
    },
    GenEntry {
        name: "firefox_3_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 3,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_3_windows_desktop,
    },
    GenEntry {
        name: "firefox_14_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 14,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_14_windows_desktop,
    },
    GenEntry {
        name: "firefox_3_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 3,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_3_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_7_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 7,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_7_windows_desktop,
    },
    GenEntry {
        name: "firefox_3_windows_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 3,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_3_windows_desktop_3,
    },
    GenEntry {
        name: "firefox_10_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 10,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: firefox_10_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_100_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 100,
        ja4: "t13d301000_1d37bd780c83_c3976d268853",
        spec_fn: firefox_100_windows_desktop,
    },
    GenEntry {
        name: "firefox_115_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 115,
        ja4: "t13d301100_1d37bd780c83_d41ae481755e",
        spec_fn: firefox_115_macos_desktop,
    },
    GenEntry {
        name: "firefox_49_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 49,
        ja4: "t13d301100_1d37bd780c83_d41ae481755e",
        spec_fn: firefox_49_macos_desktop,
    },
    GenEntry {
        name: "firefox_41_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 41,
        ja4: "t13d301100_1d37bd780c83_d41ae481755e",
        spec_fn: firefox_41_windows_desktop,
    },
    GenEntry {
        name: "firefox_67_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 67,
        ja4: "t13d301200_1d37bd780c83_d339722ba4af",
        spec_fn: firefox_67_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_50_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 50,
        ja4: "t13d301200_1d37bd780c83_d339722ba4af",
        spec_fn: firefox_50_windows_desktop,
    },
    GenEntry {
        name: "firefox_39_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 39,
        ja4: "t13d301200_1d37bd780c83_ecd0401ec68b",
        spec_fn: firefox_39_macos_desktop,
    },
    GenEntry {
        name: "firefox_121_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 121,
        ja4: "t13d3012h2_1d37bd780c83_b26ce05bbdd6",
        spec_fn: firefox_121_windows_desktop,
    },
    GenEntry {
        name: "firefox_125_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 125,
        ja4: "t13d3012h2_1d37bd780c83_b26ce05bbdd6",
        spec_fn: firefox_125_windows_desktop,
    },
    GenEntry {
        name: "firefox_123_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 123,
        ja4: "t13d3012h2_1d37bd780c83_b26ce05bbdd6",
        spec_fn: firefox_123_macos_desktop,
    },
    GenEntry {
        name: "firefox_123_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 123,
        ja4: "t13d3012h2_1d37bd780c83_b26ce05bbdd6",
        spec_fn: firefox_123_windows_desktop,
    },
    GenEntry {
        name: "firefox_96_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 96,
        ja4: "t13d3012h2_1d37bd780c83_b26ce05bbdd6",
        spec_fn: firefox_96_windows_desktop,
    },
    GenEntry {
        name: "firefox_57_windows_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 57,
        ja4: "t13d3012h2_1d37bd780c83_b26ce05bbdd6",
        spec_fn: firefox_57_windows_desktop_3,
    },
    GenEntry {
        name: "firefox_89_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 89,
        ja4: "t13d3012ht_1d37bd780c83_d41ae481755e",
        spec_fn: firefox_89_windows_desktop,
    },
    GenEntry {
        name: "firefox_59_macos_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 59,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_59_macos_desktop_4,
    },
    GenEntry {
        name: "firefox_88_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 88,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_88_windows_desktop,
    },
    GenEntry {
        name: "firefox_3_windows_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 3,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_3_windows_desktop_4,
    },
    GenEntry {
        name: "firefox_59_windows_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 59,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_59_windows_desktop_3,
    },
    GenEntry {
        name: "firefox_25_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 25,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_25_macos_desktop,
    },
    GenEntry {
        name: "firefox_29_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 29,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_29_macos_desktop,
    },
    GenEntry {
        name: "firefox_66_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 66,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_66_windows_desktop,
    },
    GenEntry {
        name: "firefox_79_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 79,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_79_windows_desktop,
    },
    GenEntry {
        name: "firefox_45_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 45,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_45_windows_desktop,
    },
    GenEntry {
        name: "firefox_25_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 25,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_25_windows_desktop,
    },
    GenEntry {
        name: "firefox_84_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 84,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_84_windows_desktop,
    },
    GenEntry {
        name: "firefox_85_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 85,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_85_windows_desktop,
    },
    GenEntry {
        name: "firefox_44_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 44,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_44_windows_desktop,
    },
    GenEntry {
        name: "firefox_62_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 62,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_62_windows_desktop,
    },
    GenEntry {
        name: "firefox_81_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 81,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_81_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_91_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 91,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_91_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_14_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 14,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_14_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_36_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 36,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_36_windows_desktop,
    },
    GenEntry {
        name: "firefox_79_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 79,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_79_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_40_windows_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 40,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_40_windows_desktop_3,
    },
    GenEntry {
        name: "firefox_3_linux_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Linux),
        device: Device::Desktop,
        major: 3,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_3_linux_desktop_2,
    },
    GenEntry {
        name: "firefox_84_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 84,
        ja4: "t13d3013h2_1d37bd780c83_ce5650b735ce",
        spec_fn: firefox_84_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_84_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 84,
        ja4: "t13d3013h2_1d37bd780c83_ce5650b735ce",
        spec_fn: firefox_84_macos_desktop,
    },
    GenEntry {
        name: "firefox_111_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 111,
        ja4: "t13d3013ht_1d37bd780c83_1b3407e2c936",
        spec_fn: firefox_111_windows_desktop,
    },
    GenEntry {
        name: "firefox_33_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 33,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: firefox_33_macos_desktop,
    },
    GenEntry {
        name: "firefox_55_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 55,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: firefox_55_windows_desktop,
    },
    GenEntry {
        name: "firefox_55_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 55,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: firefox_55_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_56_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 56,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: firefox_56_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_29_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 29,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: firefox_29_windows_desktop,
    },
    GenEntry {
        name: "firefox_31_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 31,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: firefox_31_windows_desktop,
    },
    GenEntry {
        name: "firefox_37_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 37,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: firefox_37_windows_desktop,
    },
    GenEntry {
        name: "firefox_37_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 37,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: firefox_37_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_55_windows_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 55,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: firefox_55_windows_desktop_3,
    },
    GenEntry {
        name: "firefox_56_windows_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 56,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: firefox_56_windows_desktop_3,
    },
    GenEntry {
        name: "firefox_126_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 126,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_126_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_133_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 133,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_133_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_136_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 136,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_136_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_136_macos_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 136,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_136_macos_desktop_3,
    },
    GenEntry {
        name: "firefox_125_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 125,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_125_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_132_macos_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 132,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_132_macos_desktop_3,
    },
    GenEntry {
        name: "firefox_125_macos_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 125,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_125_macos_desktop_3,
    },
    GenEntry {
        name: "firefox_131_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 131,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_131_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_127_macos_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_127_macos_desktop_2,
    },
    GenEntry {
        name: "firefox_127_macos_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 127,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_127_macos_desktop_3,
    },
    GenEntry {
        name: "firefox_63_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 63,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_63_windows_desktop,
    },
    GenEntry {
        name: "firefox_6_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 6,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_6_windows_desktop,
    },
    GenEntry {
        name: "firefox_4_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 4,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_4_windows_desktop,
    },
    GenEntry {
        name: "firefox_21_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 21,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_21_windows_desktop,
    },
    GenEntry {
        name: "firefox_27_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 27,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_27_windows_desktop,
    },
    GenEntry {
        name: "firefox_19_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 19,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_19_windows_desktop,
    },
    GenEntry {
        name: "firefox_3_linux_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::Linux),
        device: Device::Desktop,
        major: 3,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_3_linux_desktop_3,
    },
    GenEntry {
        name: "firefox_3_linux_desktop_4",
        browser: Browser::Firefox,
        os: Some(Os::Linux),
        device: Device::Desktop,
        major: 3,
        ja4: "t13d361200_c014a34ff1af_7c76daad20ec",
        spec_fn: firefox_3_linux_desktop_4,
    },
    GenEntry {
        name: "firefox_134_macos_desktop_3",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 134,
        ja4: "t13d361300_c014a34ff1af_588fa7aed259",
        spec_fn: firefox_134_macos_desktop_3,
    },
    GenEntry {
        name: "firefox_21_windows_desktop_2",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 21,
        ja4: "t13d421000_49900ac2774e_1f22a2ca17c4",
        spec_fn: firefox_21_windows_desktop_2,
    },
    GenEntry {
        name: "firefox_77_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 77,
        ja4: "t13d421200_49900ac2774e_d339722ba4af",
        spec_fn: firefox_77_macos_desktop,
    },
    GenEntry {
        name: "firefox_78_macos_desktop",
        browser: Browser::Firefox,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 78,
        ja4: "t13d4212ht_49900ac2774e_b26ce05bbdd6",
        spec_fn: firefox_78_macos_desktop,
    },
    GenEntry {
        name: "firefox_34_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 34,
        ja4: "t13d581000_363f866c7444_1f22a2ca17c4",
        spec_fn: firefox_34_windows_desktop,
    },
    GenEntry {
        name: "firefox_24_windows_desktop",
        browser: Browser::Firefox,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 24,
        ja4: "t13d741100_a97353c36de0_d41ae481755e",
        spec_fn: firefox_24_windows_desktop,
    },
];

// ja4=t13d071000_c7abf191d1e4_e7c285222651 obs=100
#[rustfmt::skip]
spec! {
    firefox_99_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc013, 0xc014, 0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d1113h2_47af8f603342_f81080dfc557 obs=44
#[rustfmt::skip]
spec! {
    firefox_115_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          padding, rslimit[16385], raw[0x0022, ""],
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    firefox_139_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8728
#[rustfmt::skip]
spec! {
    firefox_103_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8714
#[rustfmt::skip]
spec! {
    firefox_121_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    firefox_126_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    firefox_150_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    firefox_35_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    firefox_40_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    firefox_48_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    firefox_52_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    firefox_56_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    firefox_57_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    firefox_58_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    firefox_104_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    firefox_102_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    firefox_103_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8716
#[rustfmt::skip]
spec! {
    firefox_111_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8717
#[rustfmt::skip]
spec! {
    firefox_112_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8716
#[rustfmt::skip]
spec! {
    firefox_107_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    firefox_108_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8718
#[rustfmt::skip]
spec! {
    firefox_108_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    firefox_103_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8717
#[rustfmt::skip]
spec! {
    firefox_111_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    firefox_35_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    firefox_40_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    firefox_48_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8714
#[rustfmt::skip]
spec! {
    firefox_52_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    firefox_57_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    firefox_58_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8714
#[rustfmt::skip]
spec! {
    firefox_112_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8715
#[rustfmt::skip]
spec! {
    firefox_35_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8714
#[rustfmt::skip]
spec! {
    firefox_40_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8719
#[rustfmt::skip]
spec! {
    firefox_47_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8713
#[rustfmt::skip]
spec! {
    firefox_48_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8714
#[rustfmt::skip]
spec! {
    firefox_52_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    firefox_57_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d131100_f57a46bbacb6_ab7e3b40a677 obs=588
#[rustfmt::skip]
spec! {
    firefox_105_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603],
          sct, raw[0x0032, ""],
}

// ja4=t13d131100_f57a46bbacb6_e5728521abd4 obs=259
#[rustfmt::skip]
spec! {
    firefox_122_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, raw[0x0032, ""],
}

// ja4=t13d1311h2_f57a46bbacb6_e7c285222651 obs=435
#[rustfmt::skip]
spec! {
    firefox_117_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d1311h2_f57a46bbacb6_e7c285222651 obs=434
#[rustfmt::skip]
spec! {
    firefox_106_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d1311h2_f57a46bbacb6_e7c285222651 obs=434
#[rustfmt::skip]
spec! {
    firefox_102_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d1311h2_f57a46bbacb6_e7c285222651 obs=434
#[rustfmt::skip]
spec! {
    firefox_104_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d1311h2_f57a46bbacb6_e7c285222651 obs=434
#[rustfmt::skip]
spec! {
    firefox_106_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d1313h2_07be0c029dc8_28652fe741a1 obs=4
#[rustfmt::skip]
spec! {
    firefox_124_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, rslimit[16385], raw[0x0029, ""],
}

// ja4=t13d1513h2_8daaf6152771_748f4c70de1c obs=108
#[rustfmt::skip]
spec! {
    firefox_128_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1513h2_8daaf6152771_8b8ad545d541 obs=6
#[rustfmt::skip]
spec! {
    firefox_124_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201],
          rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1513h2_8daaf6152771_8b8ad545d541 obs=6
#[rustfmt::skip]
spec! {
    firefox_127_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201],
          rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    firefox_72_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    firefox_46_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    firefox_64_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    firefox_51_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    firefox_55_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    firefox_50_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    firefox_46_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=460
#[rustfmt::skip]
spec! {
    firefox_54_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=460
#[rustfmt::skip]
spec! {
    firefox_60_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    firefox_58_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    firefox_46_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d1514ht_8daaf6152771_a5b99884f7f5 obs=459
#[rustfmt::skip]
spec! {
    firefox_60_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d1515h2_8daaf6152771_6a09c78d0dc2 obs=16
#[rustfmt::skip]
spec! {
    firefox_120_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1515h2_8daaf6152771_a54fffd0eb61 obs=8
#[rustfmt::skip]
spec! {
    firefox_144_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1515h2_8daaf6152771_a54fffd0eb61 obs=10
#[rustfmt::skip]
spec! {
    firefox_148_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1515ht_8daaf6152771_4769d65a485e obs=534
#[rustfmt::skip]
spec! {
    firefox_64_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_4769d65a485e obs=534
#[rustfmt::skip]
spec! {
    firefox_53_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_4769d65a485e obs=535
#[rustfmt::skip]
spec! {
    firefox_59_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_4769d65a485e obs=534
#[rustfmt::skip]
spec! {
    firefox_72_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_4769d65a485e obs=534
#[rustfmt::skip]
spec! {
    firefox_71_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_70_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_60_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_62_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_61_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1616
#[rustfmt::skip]
spec! {
    firefox_53_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_59_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_59_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_54_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1616
#[rustfmt::skip]
spec! {
    firefox_66_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_73_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_65_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_71_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_51_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_74_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_54_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_52_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_48_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_72_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_55_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_56_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_71_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_59_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_51_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_67_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_64_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1515ht_8daaf6152771_de216e0ff992 obs=1615
#[rustfmt::skip]
spec! {
    firefox_46_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    firefox_127_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17184
#[rustfmt::skip]
spec! {
    firefox_133_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    firefox_102_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    firefox_115_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17175
#[rustfmt::skip]
spec! {
    firefox_128_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    firefox_132_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17178
#[rustfmt::skip]
spec! {
    firefox_136_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17179
#[rustfmt::skip]
spec! {
    firefox_139_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    firefox_137_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    firefox_126_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    firefox_129_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17175
#[rustfmt::skip]
spec! {
    firefox_130_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17174
#[rustfmt::skip]
spec! {
    firefox_117_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    firefox_118_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17173
#[rustfmt::skip]
spec! {
    firefox_122_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17175
#[rustfmt::skip]
spec! {
    firefox_115_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    firefox_134_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    firefox_135_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    firefox_144_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    firefox_148_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    firefox_148_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    firefox_143_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    firefox_141_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    firefox_119_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    firefox_109_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    firefox_113_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35852
#[rustfmt::skip]
spec! {
    firefox_145_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    firefox_59_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    firefox_135_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], sct, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    firefox_131_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], padding, sct, appsettings["h2", "http/1.1"],
}

// ja4=t13d1516h2_8daaf6152771_e5627efa2ab1 obs=2093
#[rustfmt::skip]
spec! {
    firefox_127_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], psk,
          compress[brotli], padding, sct, appsettings["h2", "http/1.1"],
}

// ja4=t13d1517h2_8daaf6152771_68c5a8c2958d obs=184
#[rustfmt::skip]
spec! {
    firefox_140_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1517h2_8daaf6152771_68c5a8c2958d obs=195
#[rustfmt::skip]
spec! {
    firefox_151_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1517h2_8daaf6152771_68c5a8c2958d obs=187
#[rustfmt::skip]
spec! {
    firefox_152_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1617h2_86a278354501_3cbfd9057e0d obs=114
#[rustfmt::skip]
spec! {
    firefox_151_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8608
#[rustfmt::skip]
spec! {
    firefox_3_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, status,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    firefox_20_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, status,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    firefox_56_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, status,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    firefox_10_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, status,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8608
#[rustfmt::skip]
spec! {
    firefox_3_linux_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, status,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    firefox_2_linux_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, status,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    firefox_48_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    firefox_98_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    firefox_86_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d171000_5b57614c22b0_e7c285222651 obs=804
#[rustfmt::skip]
spec! {
    firefox_110_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d1710h2_5b57614c22b0_97f8aa674fd9 obs=172
#[rustfmt::skip]
spec! {
    firefox_65_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, alpn["h2", "http/1.1"],
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d171100_ab0a1bf427ad_d41ae481755e obs=576
#[rustfmt::skip]
spec! {
    firefox_116_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d171100_ab0a1bf427ad_d41ae481755e obs=581
#[rustfmt::skip]
spec! {
    firefox_91_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d1712h2_5b57614c22b0_ef7df7f74e48 obs=11369
#[rustfmt::skip]
spec! {
    firefox_138_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], psk,
          padding,
}

// ja4=t13d1712h2_5b57614c22b0_ef7df7f74e48 obs=11369
#[rustfmt::skip]
spec! {
    firefox_137_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], psk,
          padding,
}

// ja4=t13d1712ht_95e1cefdbe28_d41ae481755e obs=2
#[rustfmt::skip]
spec! {
    firefox_58_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          alpn["http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    firefox_115_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    firefox_128_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=760
#[rustfmt::skip]
spec! {
    firefox_108_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=761
#[rustfmt::skip]
spec! {
    firefox_129_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_b26ce05bbdd6 obs=761
#[rustfmt::skip]
spec! {
    firefox_110_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d1712ht_ab0a1bf427ad_d41ae481755e obs=2332
#[rustfmt::skip]
spec! {
    firefox_89_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0x0067, 0x006b,
             0x009e, 0x009f, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          alpn["http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d171300_5b57614c22b0_43ade6aba3df obs=17930
#[rustfmt::skip]
spec! {
    firefox_145_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201], psk,
          padding, sct,
}

// ja4=t13d1713h2_5b57614c22b0_748f4c70de1c obs=240
#[rustfmt::skip]
spec! {
    firefox_129_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1713h2_5b57614c22b0_748f4c70de1c obs=241
#[rustfmt::skip]
spec! {
    firefox_130_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1713h2_5b57614c22b0_f81080dfc557 obs=27
#[rustfmt::skip]
spec! {
    firefox_102_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          padding, rslimit[16385], raw[0x0022, ""],
}

// ja4=t13d1714h2_5b57614c22b0_3dd24b5ebec4 obs=232
#[rustfmt::skip]
spec! {
    firefox_145_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1714h2_5b57614c22b0_3dd24b5ebec4 obs=233
#[rustfmt::skip]
spec! {
    firefox_137_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1714h2_5b57614c22b0_3dd24b5ebec4 obs=232
#[rustfmt::skip]
spec! {
    firefox_140_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1714h2_5b57614c22b0_3dd24b5ebec4 obs=233
#[rustfmt::skip]
spec! {
    firefox_147_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1714h2_5b57614c22b0_3dd24b5ebec4 obs=232
#[rustfmt::skip]
spec! {
    firefox_137_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1714h2_5b57614c22b0_3dd24b5ebec4 obs=232
#[rustfmt::skip]
spec! {
    firefox_138_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1714h2_5b57614c22b0_3dd24b5ebec4 obs=234
#[rustfmt::skip]
spec! {
    firefox_134_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1714h2_5b57614c22b0_3dd24b5ebec4 obs=232
#[rustfmt::skip]
spec! {
    firefox_139_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1714h2_5b57614c22b0_3dd24b5ebec4 obs=232
#[rustfmt::skip]
spec! {
    firefox_148_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1714h2_5b57614c22b0_3dd24b5ebec4 obs=235
#[rustfmt::skip]
spec! {
    firefox_149_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1714h2_5b57614c22b0_3dd24b5ebec4 obs=233
#[rustfmt::skip]
spec! {
    firefox_139_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1714h2_5b57614c22b0_3dd24b5ebec4 obs=232
#[rustfmt::skip]
spec! {
    firefox_132_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_3d5424432f57 obs=680
#[rustfmt::skip]
spec! {
    firefox_109_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, padding, rslimit[16385], raw[0x0022, ""],
}

// ja4=t13d1715h2_5b57614c22b0_3d5424432f57 obs=672
#[rustfmt::skip]
spec! {
    firefox_78_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, padding, rslimit[16385], raw[0x0022, ""],
}

// ja4=t13d1715h2_5b57614c22b0_3d5424432f57 obs=673
#[rustfmt::skip]
spec! {
    firefox_115_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, padding, rslimit[16385], raw[0x0022, ""],
}

// ja4=t13d1715h2_5b57614c22b0_3d5424432f57 obs=673
#[rustfmt::skip]
spec! {
    firefox_116_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, padding, rslimit[16385], raw[0x0022, ""],
}

// ja4=t13d1715h2_5b57614c22b0_3d5424432f57 obs=674
#[rustfmt::skip]
spec! {
    firefox_80_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, padding, rslimit[16385], raw[0x0022, ""],
}

// ja4=t13d1715h2_5b57614c22b0_3d5424432f57 obs=672
#[rustfmt::skip]
spec! {
    firefox_107_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, padding, rslimit[16385], raw[0x0022, ""],
}

// ja4=t13d1715h2_5b57614c22b0_3d5424432f57 obs=672
#[rustfmt::skip]
spec! {
    firefox_108_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, padding, rslimit[16385], raw[0x0022, ""],
}

// ja4=t13d1715h2_5b57614c22b0_3d5424432f57 obs=672
#[rustfmt::skip]
spec! {
    firefox_95_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, padding, rslimit[16385], raw[0x0022, ""],
}

// ja4=t13d1715h2_5b57614c22b0_3d5424432f57 obs=673
#[rustfmt::skip]
spec! {
    firefox_102_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, padding, rslimit[16385], raw[0x0022, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1997
#[rustfmt::skip]
spec! {
    firefox_127_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=2040
#[rustfmt::skip]
spec! {
    firefox_129_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1954
#[rustfmt::skip]
spec! {
    firefox_130_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1949
#[rustfmt::skip]
spec! {
    firefox_128_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1950
#[rustfmt::skip]
spec! {
    firefox_131_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=2011
#[rustfmt::skip]
spec! {
    firefox_130_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=2022
#[rustfmt::skip]
spec! {
    firefox_131_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1951
#[rustfmt::skip]
spec! {
    firefox_149_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1947
#[rustfmt::skip]
spec! {
    firefox_132_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1948
#[rustfmt::skip]
spec! {
    firefox_126_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1948
#[rustfmt::skip]
spec! {
    firefox_130_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1970
#[rustfmt::skip]
spec! {
    firefox_126_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1959
#[rustfmt::skip]
spec! {
    firefox_127_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1952
#[rustfmt::skip]
spec! {
    firefox_129_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1947
#[rustfmt::skip]
spec! {
    firefox_124_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1948
#[rustfmt::skip]
spec! {
    firefox_126_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1947
#[rustfmt::skip]
spec! {
    firefox_129_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1948
#[rustfmt::skip]
spec! {
    firefox_121_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1947
#[rustfmt::skip]
spec! {
    firefox_126_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1947
#[rustfmt::skip]
spec! {
    firefox_130_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1948
#[rustfmt::skip]
spec! {
    firefox_127_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1949
#[rustfmt::skip]
spec! {
    firefox_129_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1948
#[rustfmt::skip]
spec! {
    firefox_124_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1949
#[rustfmt::skip]
spec! {
    firefox_125_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1954
#[rustfmt::skip]
spec! {
    firefox_126_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1947
#[rustfmt::skip]
spec! {
    firefox_131_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1947
#[rustfmt::skip]
spec! {
    firefox_125_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1947
#[rustfmt::skip]
spec! {
    firefox_126_android_desktop_6,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1947
#[rustfmt::skip]
spec! {
    firefox_120_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1952
#[rustfmt::skip]
spec! {
    firefox_125_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_5c2c66f702b0 obs=1950
#[rustfmt::skip]
spec! {
    firefox_120_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1715h2_5b57614c22b0_a54fffd0eb61 obs=793
#[rustfmt::skip]
spec! {
    firefox_86_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_139_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_132_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1223
#[rustfmt::skip]
spec! {
    firefox_136_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_132_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1221
#[rustfmt::skip]
spec! {
    firefox_134_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1219
#[rustfmt::skip]
spec! {
    firefox_138_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1223
#[rustfmt::skip]
spec! {
    firefox_139_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1221
#[rustfmt::skip]
spec! {
    firefox_143_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_150_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1227
#[rustfmt::skip]
spec! {
    firefox_150_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1236
#[rustfmt::skip]
spec! {
    firefox_147_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_134_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_132_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1219
#[rustfmt::skip]
spec! {
    firefox_133_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1219
#[rustfmt::skip]
spec! {
    firefox_137_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_142_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_145_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1221
#[rustfmt::skip]
spec! {
    firefox_133_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1222
#[rustfmt::skip]
spec! {
    firefox_135_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_136_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1219
#[rustfmt::skip]
spec! {
    firefox_141_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1220
#[rustfmt::skip]
spec! {
    firefox_135_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1225
#[rustfmt::skip]
spec! {
    firefox_142_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1219
#[rustfmt::skip]
spec! {
    firefox_148_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_138_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1221
#[rustfmt::skip]
spec! {
    firefox_144_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1221
#[rustfmt::skip]
spec! {
    firefox_146_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1230
#[rustfmt::skip]
spec! {
    firefox_148_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1219
#[rustfmt::skip]
spec! {
    firefox_133_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1219
#[rustfmt::skip]
spec! {
    firefox_135_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1219
#[rustfmt::skip]
spec! {
    firefox_147_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1223
#[rustfmt::skip]
spec! {
    firefox_134_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_134_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_149_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_133_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1219
#[rustfmt::skip]
spec! {
    firefox_133_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_140_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1219
#[rustfmt::skip]
spec! {
    firefox_147_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_135_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1219
#[rustfmt::skip]
spec! {
    firefox_138_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_141_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_144_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_137_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1220
#[rustfmt::skip]
spec! {
    firefox_142_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_143_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_145_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_146_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_147_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1219
#[rustfmt::skip]
spec! {
    firefox_132_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1220
#[rustfmt::skip]
spec! {
    firefox_133_android_desktop_6,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1226
#[rustfmt::skip]
spec! {
    firefox_137_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1222
#[rustfmt::skip]
spec! {
    firefox_141_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_143_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1219
#[rustfmt::skip]
spec! {
    firefox_144_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1222
#[rustfmt::skip]
spec! {
    firefox_145_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1219
#[rustfmt::skip]
spec! {
    firefox_146_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1219
#[rustfmt::skip]
spec! {
    firefox_150_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1222
#[rustfmt::skip]
spec! {
    firefox_140_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    firefox_141_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1220
#[rustfmt::skip]
spec! {
    firefox_145_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1219
#[rustfmt::skip]
spec! {
    firefox_132_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1222
#[rustfmt::skip]
spec! {
    firefox_132_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=5972
#[rustfmt::skip]
spec! {
    firefox_134_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=6068
#[rustfmt::skip]
spec! {
    firefox_142_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=6087
#[rustfmt::skip]
spec! {
    firefox_144_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=6085
#[rustfmt::skip]
spec! {
    firefox_138_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=6105
#[rustfmt::skip]
spec! {
    firefox_142_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=6007
#[rustfmt::skip]
spec! {
    firefox_150_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=5969
#[rustfmt::skip]
spec! {
    firefox_133_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=5967
#[rustfmt::skip]
spec! {
    firefox_128_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=6052
#[rustfmt::skip]
spec! {
    firefox_143_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=6125
#[rustfmt::skip]
spec! {
    firefox_140_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=6083
#[rustfmt::skip]
spec! {
    firefox_146_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=6191
#[rustfmt::skip]
spec! {
    firefox_147_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=6047
#[rustfmt::skip]
spec! {
    firefox_135_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=6062
#[rustfmt::skip]
spec! {
    firefox_141_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=6115
#[rustfmt::skip]
spec! {
    firefox_146_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=6180
#[rustfmt::skip]
spec! {
    firefox_147_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=6078
#[rustfmt::skip]
spec! {
    firefox_149_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=6053
#[rustfmt::skip]
spec! {
    firefox_140_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=6026
#[rustfmt::skip]
spec! {
    firefox_136_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=6092
#[rustfmt::skip]
spec! {
    firefox_149_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=5967
#[rustfmt::skip]
spec! {
    firefox_124_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=5967
#[rustfmt::skip]
spec! {
    firefox_136_android_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=5967
#[rustfmt::skip]
spec! {
    firefox_140_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=5968
#[rustfmt::skip]
spec! {
    firefox_142_android_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=5968
#[rustfmt::skip]
spec! {
    firefox_144_android_desktop_5,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d1717h2_5b57614c22b0_3cbfd9057e0d obs=5967
#[rustfmt::skip]
spec! {
    firefox_134_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x009c, 0x009d, 0xc009, 0xc00a,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          psk, compress[brotli], sct, rslimit[16385], raw[0x0022, ""], raw[0xfe0d, ""],
}

// ja4=t13d201000_314f1408a5a6_e7c285222651 obs=171
#[rustfmt::skip]
spec! {
    firefox_97_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x003c, 0x009c, 0x009d, 0xc009, 0xc00a, 0xc023, 0xc027,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d201000_314f1408a5a6_e7c285222651 obs=171
#[rustfmt::skip]
spec! {
    firefox_137_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x003c, 0x009c, 0x009d, 0xc009, 0xc00a, 0xc023, 0xc027,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct,
}

// ja4=t13d201100_314f1408a5a6_ab7e3b40a677 obs=41
#[rustfmt::skip]
spec! {
    firefox_80_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x003c, 0x009c, 0x009d, 0xc009, 0xc00a, 0xc023, 0xc027,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603],
          sct, raw[0x0032, ""],
}

// ja4=t13d201100_314f1408a5a6_ab7e3b40a677 obs=41
#[rustfmt::skip]
spec! {
    firefox_82_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x003c, 0x009c, 0x009d, 0xc009, 0xc00a, 0xc023, 0xc027,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603],
          sct, raw[0x0032, ""],
}

// ja4=t13d201100_314f1408a5a6_ab7e3b40a677 obs=41
#[rustfmt::skip]
spec! {
    firefox_70_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x003c, 0x009c, 0x009d, 0xc009, 0xc00a, 0xc023, 0xc027,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603],
          sct, raw[0x0032, ""],
}

// ja4=t13d201100_314f1408a5a6_e5728521abd4 obs=82
#[rustfmt::skip]
spec! {
    firefox_64_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x003c, 0x009c, 0x009d, 0xc009, 0xc00a, 0xc023, 0xc027,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, raw[0x0032, ""],
}

// ja4=t13d201100_314f1408a5a6_e5728521abd4 obs=82
#[rustfmt::skip]
spec! {
    firefox_73_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x003c, 0x009c, 0x009d, 0xc009, 0xc00a, 0xc023, 0xc027,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, raw[0x0032, ""],
}

// ja4=t13d201100_314f1408a5a6_e5728521abd4 obs=82
#[rustfmt::skip]
spec! {
    firefox_112_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x003c, 0x009c, 0x009d, 0xc009, 0xc00a, 0xc023, 0xc027,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          sct, raw[0x0032, ""],
}

// ja4=t13d221000_231e334592e8_29829a46703f obs=25
#[rustfmt::skip]
spec! {
    firefox_81_windows_desktop,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x002f, 0x0035, 0x003c,
             0x003d, 0x009c, 0x009d, 0x009e, 0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ticket,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0201, 0x0403, 0x0503, 0x0203, 0x0202, 0x0601, 0x0603],
          psk, raw[0x0031, ""],
}

// ja4=t13d301000_1d37bd780c83_1f22a2ca17c4 obs=98
#[rustfmt::skip]
spec! {
    firefox_33_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_518fb456ca59 obs=310
#[rustfmt::skip]
spec! {
    firefox_114_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x081a, 0x081b, 0x081c, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_518fb456ca59 obs=300
#[rustfmt::skip]
spec! {
    firefox_114_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x081a, 0x081b, 0x081c, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_62_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_57_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_60_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_22_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_19_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_11_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_18_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_8_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_66_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_65_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_36_android_tablet,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_17_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_41_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_33_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_43_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_8_android_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_31_android_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_5_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_11_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1186
#[rustfmt::skip]
spec! {
    firefox_4_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1186
#[rustfmt::skip]
spec! {
    firefox_13_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_3_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_14_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1186
#[rustfmt::skip]
spec! {
    firefox_3_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_7_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1186
#[rustfmt::skip]
spec! {
    firefox_3_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    firefox_10_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_c3976d268853 obs=43
#[rustfmt::skip]
spec! {
    firefox_100_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0905, 0x0906, 0x0904, 0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x081a, 0x081b, 0x081c, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, raw[0x0016, ""],
}

// ja4=t13d301100_1d37bd780c83_d41ae481755e obs=5253
#[rustfmt::skip]
spec! {
    firefox_115_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301100_1d37bd780c83_d41ae481755e obs=5258
#[rustfmt::skip]
spec! {
    firefox_49_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301100_1d37bd780c83_d41ae481755e obs=5260
#[rustfmt::skip]
spec! {
    firefox_41_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d301200_1d37bd780c83_d339722ba4af obs=2396
#[rustfmt::skip]
spec! {
    firefox_67_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d301200_1d37bd780c83_d339722ba4af obs=2379
#[rustfmt::skip]
spec! {
    firefox_50_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d301200_1d37bd780c83_ecd0401ec68b obs=14
#[rustfmt::skip]
spec! {
    firefox_39_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          ticket, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0905, 0x0906, 0x0904, 0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x081a, 0x081b, 0x081c, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, compress[brotli], raw[0x0016, ""],
}

// ja4=t13d3012h2_1d37bd780c83_b26ce05bbdd6 obs=28316
#[rustfmt::skip]
spec! {
    firefox_121_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d3012h2_1d37bd780c83_b26ce05bbdd6 obs=28316
#[rustfmt::skip]
spec! {
    firefox_125_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d3012h2_1d37bd780c83_b26ce05bbdd6 obs=28316
#[rustfmt::skip]
spec! {
    firefox_123_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d3012h2_1d37bd780c83_b26ce05bbdd6 obs=28316
#[rustfmt::skip]
spec! {
    firefox_123_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d3012h2_1d37bd780c83_b26ce05bbdd6 obs=28316
#[rustfmt::skip]
spec! {
    firefox_96_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d3012h2_1d37bd780c83_b26ce05bbdd6 obs=28316
#[rustfmt::skip]
spec! {
    firefox_57_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d3012ht_1d37bd780c83_d41ae481755e obs=86
#[rustfmt::skip]
spec! {
    firefox_89_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          alpn["http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    firefox_59_macos_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    firefox_88_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    firefox_3_windows_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    firefox_59_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    firefox_25_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    firefox_29_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    firefox_66_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    firefox_79_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    firefox_45_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    firefox_25_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    firefox_84_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    firefox_85_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    firefox_44_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    firefox_62_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    firefox_81_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    firefox_91_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    firefox_14_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    firefox_36_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    firefox_79_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    firefox_40_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    firefox_3_linux_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_ce5650b735ce obs=1199
#[rustfmt::skip]
spec! {
    firefox_84_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_ce5650b735ce obs=1199
#[rustfmt::skip]
spec! {
    firefox_84_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["h2", "http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013ht_1d37bd780c83_1b3407e2c936 obs=138
#[rustfmt::skip]
spec! {
    firefox_111_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1188
#[rustfmt::skip]
spec! {
    firefox_33_macos_desktop,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x002f, 0x0035, 0x0032,
             0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b, 0x009c, 0x009d,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1190
#[rustfmt::skip]
spec! {
    firefox_55_windows_desktop,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x002f, 0x0035, 0x0032,
             0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b, 0x009c, 0x009d,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1190
#[rustfmt::skip]
spec! {
    firefox_55_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x002f, 0x0035, 0x0032,
             0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b, 0x009c, 0x009d,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1192
#[rustfmt::skip]
spec! {
    firefox_56_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x002f, 0x0035, 0x0032,
             0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b, 0x009c, 0x009d,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1186
#[rustfmt::skip]
spec! {
    firefox_29_windows_desktop,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x002f, 0x0035, 0x0032,
             0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b, 0x009c, 0x009d,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1186
#[rustfmt::skip]
spec! {
    firefox_31_windows_desktop,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x002f, 0x0035, 0x0032,
             0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b, 0x009c, 0x009d,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1187
#[rustfmt::skip]
spec! {
    firefox_37_windows_desktop,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x002f, 0x0035, 0x0032,
             0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b, 0x009c, 0x009d,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1190
#[rustfmt::skip]
spec! {
    firefox_37_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x002f, 0x0035, 0x0032,
             0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b, 0x009c, 0x009d,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1189
#[rustfmt::skip]
spec! {
    firefox_55_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x002f, 0x0035, 0x0032,
             0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b, 0x009c, 0x009d,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1186
#[rustfmt::skip]
spec! {
    firefox_56_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xc013, 0xc014, 0x002f, 0x0035, 0x0032,
             0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a, 0x006b, 0x009c, 0x009d,
             0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=463
#[rustfmt::skip]
spec! {
    firefox_126_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    firefox_133_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    firefox_136_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=463
#[rustfmt::skip]
spec! {
    firefox_136_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    firefox_125_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    firefox_132_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    firefox_125_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    firefox_131_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    firefox_127_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    firefox_127_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=466
#[rustfmt::skip]
spec! {
    firefox_63_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=463
#[rustfmt::skip]
spec! {
    firefox_6_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=463
#[rustfmt::skip]
spec! {
    firefox_4_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    firefox_21_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    firefox_27_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=462
#[rustfmt::skip]
spec! {
    firefox_19_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=467
#[rustfmt::skip]
spec! {
    firefox_3_linux_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361200_c014a34ff1af_7c76daad20ec obs=463
#[rustfmt::skip]
spec! {
    firefox_3_linux_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          psk, raw[0x0011, ""], raw[0x0032, ""],
}

// ja4=t13d361300_c014a34ff1af_588fa7aed259 obs=347
#[rustfmt::skip]
spec! {
    firefox_134_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          status, keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0201, 0x0203],
          psk, raw[0x0011, ""], raw[0x0029, ""], raw[0x0032, ""],
}

// ja4=t13d421000_49900ac2774e_1f22a2ca17c4 obs=45
#[rustfmt::skip]
spec! {
    firefox_21_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, raw[0x0016, ""],
}

// ja4=t13d421200_49900ac2774e_d339722ba4af obs=45
#[rustfmt::skip]
spec! {
    firefox_77_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d4212ht_49900ac2774e_b26ce05bbdd6 obs=859
#[rustfmt::skip]
spec! {
    firefox_78_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0033, 0x0039, 0x003c, 0x003d, 0x0067, 0x006b, 0x009c, 0x009d, 0x009e,
             0x009f, 0xc009, 0xc00a, 0xc023, 0xc024, 0xc027, 0xc028, 0xc09c, 0xc09d, 0xc09e, 0xc09f,
             0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf,
          alpn["http/1.1"], keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""], raw[0x0031, ""],
}

// ja4=t13d581000_363f866c7444_1f22a2ca17c4 obs=445
#[rustfmt::skip]
spec! {
    firefox_34_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0067, 0x006a,
             0x006b, 0x009c, 0x009d, 0x009e, 0x009f, 0x00a2, 0x00a3, 0xc009, 0xc00a, 0xc023, 0xc024,
             0xc027, 0xc028, 0xc050, 0xc051, 0xc052, 0xc053, 0xc056, 0xc057, 0xc05c, 0xc05d, 0xc060,
             0xc061, 0xc09c, 0xc09d, 0xc09e, 0xc09f, 0xc0a0, 0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad,
             0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, raw[0x0016, ""],
}

// ja4=t13d741100_a97353c36de0_d41ae481755e obs=48
#[rustfmt::skip]
spec! {
    firefox_24_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc02c, 0xc030, 0xcca9, 0xcca8, 0xc013, 0xc014,
             0x002f, 0x0035, 0x0032, 0x0033, 0x0038, 0x0039, 0x003c, 0x003d, 0x0040, 0x0041, 0x0044,
             0x0045, 0x0067, 0x006a, 0x006b, 0x0084, 0x0087, 0x0088, 0x009c, 0x009d, 0x009e, 0x009f,
             0x00a2, 0x00a3, 0x00ba, 0x00bd, 0x00be, 0x00c0, 0x00c3, 0x00c4, 0xc009, 0xc00a, 0xc023,
             0xc024, 0xc027, 0xc028, 0xc050, 0xc051, 0xc052, 0xc053, 0xc056, 0xc057, 0xc05c, 0xc05d,
             0xc060, 0xc061, 0xc072, 0xc073, 0xc076, 0xc077, 0xc09c, 0xc09d, 0xc09e, 0xc09f, 0xc0a0,
             0xc0a1, 0xc0a2, 0xc0a3, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0xccaa,
    session: random32,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521, 0x0100, 0x0101], ecpf, ticket,
          keyshare[x25519], versions[0x0304, 0x0303],
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          psk, padding, raw[0x0016, ""],
}
