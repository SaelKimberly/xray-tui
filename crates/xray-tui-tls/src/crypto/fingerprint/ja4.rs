//! JA4 fingerprint encoder (offline, no hash).
//!
//! JA4-A per the original `FoxIO` spec (the `t13d` + first-cipher variant
//! published at the September 2023 launch): `t13d` + first cipher (4 hex,
//! GREASE skipped) + `d` + extension count (2 digits) + signature-algorithm
//! count (2 digits) + ALPN (or `00`) + `_` + sorted extension ids
//! (GREASE → `g`) + `_` + sorted signature algorithms (GREASE → `g`).

use crate::spec::grease::is_grease;

use super::ja3::Ja3Fields;

/// The full JA4-A fingerprint string.
#[must_use]
pub fn ja4_a(f: &Ja3Fields) -> String {
    let first_cipher = f
        .ciphers
        .iter()
        .copied()
        .find(|&c| !is_grease(c))
        .unwrap_or(0);
    let alpn = f.alpn.first().map_or("00", String::as_str);
    let exts = sorted_list(&f.extensions);
    let mut out = format!(
        "t13d{first_cipher:04x}d{ext_count:02}{sig_count:02}{alpn}_{exts}",
        ext_count = f.extensions.len(),
        sig_count = f.signature_algorithms.len(),
    );
    // Per the FoxIO spec: with no signature algorithms the fingerprint
    // ends without a trailing underscore.
    let sigs = sorted_list(&f.signature_algorithms);
    if !sigs.is_empty() {
        out.push('_');
        out.push_str(&sigs);
    }
    out
}

/// Sorted comma-joined ids, GREASE values rendered as `g`.
fn sorted_list(ids: &[u16]) -> String {
    let mut ids = ids.to_vec();
    ids.sort_unstable();
    ids.iter()
        .map(|&id| {
            if is_grease(id) {
                "g".to_string()
            } else {
                format!("{id:x}")
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ja4_normalizes_grease_and_sorts() {
        let f = Ja3Fields {
            version: 771,
            ciphers: vec![0x0A0A, 0x1301, 0x1302],
            extensions: vec![0x001b, 0x0a0a, 0x002b, 0x0017],
            curves: vec![],
            point_formats: vec![0],
            signature_algorithms: vec![0x0403, 0x0804],
            alpn: vec!["h2".into()],
        };
        // t13d + first non-grease cipher 1301 + d + ext_count 04 + sig_count 02 + h2
        assert_eq!(ja4_a(&f), "t13d1301d0402h2_17,1b,2b,g_403,804");
    }

    #[test]
    fn ja4_no_alpn_is_00() {
        let f = Ja3Fields {
            version: 771,
            ciphers: vec![0x1301],
            extensions: vec![0x0000],
            curves: vec![],
            point_formats: vec![0],
            signature_algorithms: vec![],
            alpn: vec![],
        };
        assert_eq!(ja4_a(&f), "t13d1301d010000_0");
    }

    #[test]
    fn ja4_all_grease_ciphers_falls_back_to_0000() {
        let f = Ja3Fields {
            version: 771,
            ciphers: vec![0x0A0A, 0x1A1A],
            extensions: vec![],
            curves: vec![],
            point_formats: vec![],
            signature_algorithms: vec![0x0A0A, 0x0403],
            alpn: vec![],
        };
        assert_eq!(ja4_a(&f), "t13d0000d000200__403,g");
    }
}
