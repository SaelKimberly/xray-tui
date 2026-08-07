//! JA3 fingerprint encoder (offline).
//!
//! JA3 is the md5 of a canonical string over ClientHello fields:
//! `version,ciphers,extensions,curves,point_formats` with decimal version,
//! lowercase-hex extension/cipher/curve ids and decimal point formats.
//! Computing JA3 from raw hello bytes requires a ClientHello parser (M2,
//! with the fingerprint engine); this codec encodes from structured fields.

/// JA3 field set extracted (later) from a ClientHello.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ja3Fields {
    /// TLS record/layout version, e.g. 0x0303 → 771.
    pub version: u16,
    pub ciphers: Vec<u16>,
    pub extensions: Vec<u16>,
    pub curves: Vec<u16>,
    pub point_formats: Vec<u8>,
}

impl Ja3Fields {
    /// The canonical pre-hash JA3 string.
    #[must_use]
    pub fn canonical_string(&self) -> String {
        format!(
            "{},{},{},{},{}",
            self.version,
            join_hex(&self.ciphers),
            join_hex(&self.extensions),
            join_hex(&self.curves),
            join_dec(&self.point_formats),
        )
    }

    /// The JA3 hash (md5 of the canonical string).
    #[must_use]
    pub fn hash(&self) -> String {
        use md5::{Digest, Md5};
        let mut hasher = Md5::new();
        hasher.update(self.canonical_string().as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

fn join_hex(values: &[u16]) -> String {
    values
        .iter()
        .map(|v| format!("{v:x}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn join_dec(values: &[u8]) -> String {
    values
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_string_exact() {
        let f = Ja3Fields {
            version: 771,
            ciphers: vec![0x1301, 0x1302, 0x1303],
            extensions: vec![0x0000, 0x001a, 0x0a0a],
            curves: vec![0x001d, 0x0017],
            point_formats: vec![0],
        };
        assert_eq!(f.canonical_string(), "771,1301,1302,1303,0,1a,a0a,1d,17,0");
    }

    #[test]
    fn hash_matches_md5_of_canonical() {
        use md5::{Digest, Md5};
        let f = Ja3Fields {
            version: 771,
            ciphers: vec![0x1301],
            extensions: vec![],
            curves: vec![0x001d],
            point_formats: vec![0],
        };
        assert_eq!(f.hash(), format!("{:x}", Md5::digest(b"771,1301,,1d,0")));
    }

    #[test]
    fn defaults_produce_empty_tail() {
        let f = Ja3Fields::default();
        assert_eq!(f.canonical_string(), "0,,,,");
    }
}
