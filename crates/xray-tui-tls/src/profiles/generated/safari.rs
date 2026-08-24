//! Safari hellos on macOS and desktop (the `safari` wire template)
//!
//! Emitter output (`gen_specs.py --emit`); do not edit by hand.
//! Regeneration is byte-deterministic (`--selftest` verifies the
//! committed files match a fresh render).

use super::GenEntry;
use crate::fingerprints::{Browser, Device, Os};

#[rustfmt::skip]
pub const GENERATED: &[GenEntry] = &[
    GenEntry {
        name: "safari_15_macos_desktop",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 15,
        ja4: "t13d131000_f57a46bbacb6_e7c285222651",
        spec_fn: safari_15_macos_desktop,
    },
    GenEntry {
        name: "safari_16_macos_desktop",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 16,
        ja4: "t13d1516h2_8daaf6152771_02713d6af862",
        spec_fn: safari_16_macos_desktop,
    },
    GenEntry {
        name: "safari_26_macos_desktop",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 26,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: safari_26_macos_desktop,
    },
    GenEntry {
        name: "safari_17_macos_desktop",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 17,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: safari_17_macos_desktop,
    },
    GenEntry {
        name: "safari_18_macos_desktop",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 18,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: safari_18_macos_desktop,
    },
    GenEntry {
        name: "safari_17_macos_desktop_2",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 17,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: safari_17_macos_desktop_2,
    },
    GenEntry {
        name: "safari_17_macos_desktop_3",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 17,
        ja4: "t13d1516h2_8daaf6152771_d8a2da3f94cd",
        spec_fn: safari_17_macos_desktop_3,
    },
    GenEntry {
        name: "safari_8_macos_desktop",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 8,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: safari_8_macos_desktop,
    },
    GenEntry {
        name: "safari_10_macos_desktop",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 10,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: safari_10_macos_desktop,
    },
    GenEntry {
        name: "safari_12_macos_desktop",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 12,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: safari_12_macos_desktop,
    },
    GenEntry {
        name: "safari_4_macos_desktop",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 4,
        ja4: "t13d170900_5b57614c22b0_97f8aa674fd9",
        spec_fn: safari_4_macos_desktop,
    },
    GenEntry {
        name: "safari_26_macos_desktop_2",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 26,
        ja4: "t13d1710h2_5b57614c22b0_97f8aa674fd9",
        spec_fn: safari_26_macos_desktop_2,
    },
    GenEntry {
        name: "safari_16_macos_desktop_2",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 16,
        ja4: "t13d1714h2_5b57614c22b0_14788d8d241b",
        spec_fn: safari_16_macos_desktop_2,
    },
    GenEntry {
        name: "safari_19_macos_desktop",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 19,
        ja4: "t13d1714h2_5b57614c22b0_d0a99439f9b1",
        spec_fn: safari_19_macos_desktop,
    },
    GenEntry {
        name: "safari_12_windows_desktop",
        browser: Browser::Safari,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 12,
        ja4: "t13d1716h2_5b57614c22b0_eeeea6562960",
        spec_fn: safari_12_windows_desktop,
    },
    GenEntry {
        name: "safari_14_macos_desktop",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 14,
        ja4: "t13d201000_314f1408a5a6_e7c285222651",
        spec_fn: safari_14_macos_desktop,
    },
    GenEntry {
        name: "safari_12_windows_desktop_2",
        browser: Browser::Safari,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 12,
        ja4: "t13d201100_314f1408a5a6_e5728521abd4",
        spec_fn: safari_12_windows_desktop_2,
    },
    GenEntry {
        name: "safari_15_macos_desktop_2",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 15,
        ja4: "t13d301000_1d37bd780c83_518fb456ca59",
        spec_fn: safari_15_macos_desktop_2,
    },
    GenEntry {
        name: "safari_4_windows_desktop",
        browser: Browser::Safari,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 4,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: safari_4_windows_desktop,
    },
    GenEntry {
        name: "safari_5_windows_desktop",
        browser: Browser::Safari,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 5,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: safari_5_windows_desktop,
    },
    GenEntry {
        name: "safari_4_windows_desktop_2",
        browser: Browser::Safari,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 4,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: safari_4_windows_desktop_2,
    },
    GenEntry {
        name: "safari_5_windows_desktop_2",
        browser: Browser::Safari,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 5,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: safari_5_windows_desktop_2,
    },
    GenEntry {
        name: "safari_4_windows_desktop_3",
        browser: Browser::Safari,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 4,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: safari_4_windows_desktop_3,
    },
    GenEntry {
        name: "safari_5_windows_desktop_3",
        browser: Browser::Safari,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 5,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: safari_5_windows_desktop_3,
    },
    GenEntry {
        name: "safari_4_windows_desktop_4",
        browser: Browser::Safari,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 4,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: safari_4_windows_desktop_4,
    },
    GenEntry {
        name: "safari_5_windows_desktop_4",
        browser: Browser::Safari,
        os: Some(Os::Windows),
        device: Device::Desktop,
        major: 5,
        ja4: "t13d301000_1d37bd780c83_a29327ec888c",
        spec_fn: safari_5_windows_desktop_4,
    },
    GenEntry {
        name: "safari_14_macos_desktop_2",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 14,
        ja4: "t13d301100_1d37bd780c83_d41ae481755e",
        spec_fn: safari_14_macos_desktop_2,
    },
    GenEntry {
        name: "safari_9_macos_desktop",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 9,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: safari_9_macos_desktop,
    },
    GenEntry {
        name: "safari_13_macos_desktop",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 13,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: safari_13_macos_desktop,
    },
    GenEntry {
        name: "safari_5_macos_desktop",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 5,
        ja4: "t13d3013h2_1d37bd780c83_1b3407e2c936",
        spec_fn: safari_5_macos_desktop,
    },
    GenEntry {
        name: "safari_7_macos_desktop",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 7,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: safari_7_macos_desktop,
    },
    GenEntry {
        name: "safari_11_macos_desktop",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 11,
        ja4: "t13d320900_47c5e39c651d_9da38b6fd1bc",
        spec_fn: safari_11_macos_desktop,
    },
    GenEntry {
        name: "safari_18_macos_desktop_2",
        browser: Browser::Safari,
        os: Some(Os::MacOs),
        device: Device::Desktop,
        major: 18,
        ja4: "t13d3613h2_c014a34ff1af_aac333855136",
        spec_fn: safari_18_macos_desktop_2,
    },
];

// ja4=t13d131000_f57a46bbacb6_e7c285222651 obs=8712
#[rustfmt::skip]
spec! {
    safari_15_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521], ecpf, status, sct,
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d1516h2_8daaf6152771_02713d6af862 obs=17175
#[rustfmt::skip]
spec! {
    safari_16_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521], ecpf,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          ticket, appsettings["h2", "http/1.1"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    safari_26_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521], ecpf,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35854
#[rustfmt::skip]
spec! {
    safari_17_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521], ecpf,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    safari_18_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521], ecpf,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35850
#[rustfmt::skip]
spec! {
    safari_17_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521], ecpf,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d1516h2_8daaf6152771_d8a2da3f94cd obs=35851
#[rustfmt::skip]
spec! {
    safari_17_macos_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc014, 0xc013,
             0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521], ecpf,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601], compress[],
          ticket, raw[0x44cd, "000c02683208687474702f312e31"], raw[0xfe0d, ""],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    safari_8_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, reneg, groups[x25519, p256, p384, p521], ecpf, status, sct, versions[0x0304, 0x0303],
          psk,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    safari_10_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, reneg, groups[x25519, p256, p384, p521], ecpf, status, sct, versions[0x0304, 0x0303],
          psk,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    safari_12_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, reneg, groups[x25519, p256, p384, p521], ecpf, status, sct, versions[0x0304, 0x0303],
          psk,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d170900_5b57614c22b0_97f8aa674fd9 obs=8606
#[rustfmt::skip]
spec! {
    safari_4_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, reneg, groups[x25519, p256, p384, p521], ecpf, status, sct, versions[0x0304, 0x0303],
          psk,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d1710h2_5b57614c22b0_97f8aa674fd9 obs=177
#[rustfmt::skip]
spec! {
    safari_26_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, reneg, groups[x25519, p256, p384, p521], ecpf, alpn["h2", "http/1.1"], status, sct,
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d1714h2_5b57614c22b0_14788d8d241b obs=2639
#[rustfmt::skip]
spec! {
    safari_16_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521], ecpf,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0203, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          padding, compress[],
}

// ja4=t13d1714h2_5b57614c22b0_d0a99439f9b1 obs=3
#[rustfmt::skip]
spec! {
    safari_19_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521], ecpf,
          alpn["h2", "http/1.1"], status, sct, keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0805, 0x0501, 0x0806, 0x0601, 0x0201],
          compress[], ticket,
}

// ja4=t13d1716h2_5b57614c22b0_eeeea6562960 obs=1218
#[rustfmt::skip]
spec! {
    safari_12_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc00a, 0xc009,
             0xc014, 0xc013, 0x009d, 0x009c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521], ecpf,
          alpn["h2", "http/1.1"], status, keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0203, 0x0201],
          compress[], rslimit[16385], raw[0x0022, ""], ticket, raw[0xfe0d, ""],
}

// ja4=t13d201000_314f1408a5a6_e7c285222651 obs=171
#[rustfmt::skip]
spec! {
    safari_14_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc023, 0xc027,
             0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521], ecpf, status, sct,
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
}

// ja4=t13d201100_314f1408a5a6_e5728521abd4 obs=82
#[rustfmt::skip]
spec! {
    safari_12_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc023, 0xc027,
             0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003c, 0x0035, 0x002f,
    session: empty,
    exts: sni, raw[0x0017, ""], reneg, groups[x25519, p256, p384, p521], ecpf, status, sct,
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0804, 0x0403, 0x0807, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0503, 0x0603, 0x0201, 0x0203],
          raw[0x0032, ""],
}

// ja4=t13d301000_1d37bd780c83_518fb456ca59 obs=314
#[rustfmt::skip]
spec! {
    safari_15_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521], ecpf, keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x081a, 0x081b, 0x081c, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          raw[0x0016, ""], ticket,
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1186
#[rustfmt::skip]
spec! {
    safari_4_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521], ecpf, keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    safari_5_windows_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521], ecpf, keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    safari_4_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521], ecpf, keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1188
#[rustfmt::skip]
spec! {
    safari_5_windows_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521], ecpf, keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1187
#[rustfmt::skip]
spec! {
    safari_4_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521], ecpf, keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1186
#[rustfmt::skip]
spec! {
    safari_5_windows_desktop_3,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521], ecpf, keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1185
#[rustfmt::skip]
spec! {
    safari_4_windows_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521], ecpf, keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301000_1d37bd780c83_a29327ec888c obs=1186
#[rustfmt::skip]
spec! {
    safari_5_windows_desktop_4,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521], ecpf, keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""],
}

// ja4=t13d301100_1d37bd780c83_d41ae481755e obs=5255
#[rustfmt::skip]
spec! {
    safari_14_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521], ecpf, keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301, 0x0302, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], ticket,
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4458
#[rustfmt::skip]
spec! {
    safari_9_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4457
#[rustfmt::skip]
spec! {
    safari_13_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d3013h2_1d37bd780c83_1b3407e2c936 obs=4456
#[rustfmt::skip]
spec! {
    safari_5_macos_desktop,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0033, 0x0039, 0x0067, 0x006b, 0x009e, 0x009f, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521], ecpf, alpn["h2", "http/1.1"],
          keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b, 0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0203, 0x0301, 0x0201, 0x0302, 0x0202, 0x0402, 0x0502, 0x0602],
          padding, raw[0x0016, ""], raw[0x0031, ""], raw[0x3374, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1186
#[rustfmt::skip]
spec! {
    safari_7_macos_desktop,
    ciphers: 0x1301, 0x1302, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xc024, 0xc023, 0xc028, 0xc027, 0xc00a,
             0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035, 0x002f, 0x0032, 0x0033,
             0x0038, 0x0039, 0x0040, 0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521], ecpf, keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          raw[0x0032, ""],
}

// ja4=t13d320900_47c5e39c651d_9da38b6fd1bc obs=1193
#[rustfmt::skip]
spec! {
    safari_11_macos_desktop,
    ciphers: 0x1301, 0x1302, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xc024, 0xc023, 0xc028, 0xc027, 0xc00a,
             0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035, 0x002f, 0x0032, 0x0033,
             0x0038, 0x0039, 0x0040, 0x0067, 0x006a, 0x006b, 0x009e, 0x009f, 0x00a2, 0x00a3,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521], ecpf, keyshare[x25519],
          versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202],
          raw[0x0032, ""],
}

// ja4=t13d3613h2_c014a34ff1af_aac333855136 obs=28
#[rustfmt::skip]
spec! {
    safari_18_macos_desktop_2,
    ciphers: 0x1301, 0x1302, 0x1303, 0xc02c, 0xc02b, 0xc030, 0xc02f, 0xcca9, 0xcca8, 0xc024, 0xc023,
             0xc028, 0xc027, 0xc00a, 0xc009, 0xc014, 0xc013, 0x009d, 0x009c, 0x003d, 0x003c, 0x0035,
             0x002f, 0x0032, 0x0033, 0x0038, 0x0039, 0x0040, 0x0067, 0x006a, 0x006b, 0x009e, 0x009f,
             0x00a2, 0x00a3, 0xccaa,
    session: empty,
    exts: sni, raw[0x0017, ""], groups[x25519, p256, p384, p521], ecpf, alpn["h2", "http/1.1"],
          status, keyshare[x25519], versions[0x0304, 0x0303], psk,
          sigalgs[0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0804, 0x0805, 0x0806, 0x0809, 0x080a, 0x080b, 0x0401, 0x0501, 0x0601, 0x0402, 0x0303, 0x0301, 0x0302, 0x0203, 0x0201, 0x0202, 0x0101],
          raw[0x0011, ""], ticket, raw[0x0032, ""],
}
