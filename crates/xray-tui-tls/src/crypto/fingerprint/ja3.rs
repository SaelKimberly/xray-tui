//! JA3 fingerprint encoder (offline).
//!
//! JA3 is the md5 of a canonical string over `ClientHello` fields:
//! `version,ciphers,extensions,curves,point_formats` with decimal version,
//! lowercase-hex extension/cipher/curve ids and decimal point formats.
//! Computing JA3 from raw hello bytes is the [`crate::hello::parse`]
//! parser's job; this codec encodes from structured fields.

use md5::{Digest, Md5};

use crate::hello::parse::ParsedClientHello;

/// JA3 field set extracted from a `ClientHello`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ja3Fields {
    /// TLS legacy version, e.g. `0x0303` → 771.
    pub version: u16,
    pub ciphers: Vec<u16>,
    pub extensions: Vec<u16>,
    pub curves: Vec<u16>,
    pub point_formats: Vec<u8>,
    /// `signature_algorithms` extension entries (used by JA4).
    pub signature_algorithms: Vec<u16>,
    /// ALPN protocols, first entry first (used by JA4).
    pub alpn: Vec<String>,
}

impl From<&ParsedClientHello> for Ja3Fields {
    fn from(hello: &ParsedClientHello) -> Self {
        Self {
            version: hello.legacy_version,
            ciphers: hello.cipher_suites.clone(),
            extensions: hello.extensions.iter().map(|(ty, _)| *ty).collect(),
            curves: extract_u16_list(hello.extension(0x000a)),
            point_formats: extract_u8_list(hello.extension(0x000b)),
            signature_algorithms: extract_u16_list(hello.extension(0x000d)),
            alpn: extract_alpn(hello.extension(0x0010)),
        }
    }
}

/// The canonical pre-hash JA3 string.
#[must_use]
pub fn ja3_string(f: &Ja3Fields) -> String {
    format!(
        "{},{},{},{},{}",
        f.version,
        join_hex(&f.ciphers),
        join_hex(&f.extensions),
        join_hex(&f.curves),
        join_dec(&f.point_formats),
    )
}

/// The JA3 hash: md5 hex of [`ja3_string`].
#[must_use]
pub fn ja3_hash(f: &Ja3Fields) -> String {
    let mut hasher = Md5::new();
    hasher.update(ja3_string(f).as_bytes());
    format!("{:x}", hasher.finalize())
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

/// Decodes a u16-BE list behind a 2-byte length prefix (the RFC 8446 vector
/// shape shared by `supported_groups` and `signature_algorithms`). Returns
/// an empty list for a missing or malformed body.
fn extract_u16_list(body: Option<&[u8]>) -> Vec<u16> {
    let Some(body) = body else {
        return Vec::new();
    };
    let Some(rest) = body.get(2..) else {
        return Vec::new();
    };
    let len = usize::from(u16::from_be_bytes([body[0], body[1]]));
    rest[..len.min(rest.len())]
        .chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect()
}

/// Decodes a u8 list behind a 1-byte length prefix (RFC 8422
/// `ec_point_formats`). Returns an empty list for a missing or malformed
/// body.
fn extract_u8_list(body: Option<&[u8]>) -> Vec<u8> {
    let Some(body) = body else {
        return Vec::new();
    };
    let Some(rest) = body.get(1..) else {
        return Vec::new();
    };
    let len = usize::from(body[0]);
    rest[..len.min(rest.len())].to_vec()
}

/// Decodes an ALPN protocol list (RFC 7301): u16 list-length prefix, then
/// per entry a u8 length and the raw protocol bytes. Returns an empty list
/// for a missing or malformed body.
fn extract_alpn(body: Option<&[u8]>) -> Vec<String> {
    let Some(body) = body else {
        return Vec::new();
    };
    let Some(rest) = body.get(2..) else {
        return Vec::new();
    };
    let mut protos = Vec::new();
    let mut off = 0;
    while off < rest.len() {
        let len = usize::from(rest[off]);
        let Some(proto) = rest.get(off + 1..off + 1 + len) else {
            break;
        };
        protos.push(String::from_utf8_lossy(proto).into_owned());
        off += 1 + len;
    }
    protos
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
            ..Default::default()
        };
        assert_eq!(ja3_string(&f), "771,1301,1302,1303,0,1a,a0a,1d,17,0");
    }

    #[test]
    fn ja3_hash_known_value() {
        let f = Ja3Fields {
            version: 771,
            ciphers: vec![0x1301, 0x1302, 0x1303],
            extensions: vec![0x0000, 0x001a, 0x0a0a],
            curves: vec![0x001d, 0x0017],
            point_formats: vec![0],
            ..Default::default()
        };
        // md5("771,1301,1302,1303,0,1a,a0a,1d,17,0")
        assert_eq!(ja3_hash(&f), "efdb5b559e9ffcfe422009f348919377");
    }

    #[test]
    fn defaults_produce_empty_tail() {
        assert_eq!(ja3_string(&Ja3Fields::default()), "0,,,,");
    }

    #[test]
    fn from_parsed_hello_extracts_fields() {
        use crate::hello::parse::{parse_hello, ParsedClientHello};

        // Hand-build a hello with supported_groups, ec_point_formats,
        // signature_algorithms and ALPN extensions so the From impl has
        // something to extract.
        let mut body = Vec::new();
        body.extend_from_slice(&0x0303u16.to_be_bytes());
        body.extend_from_slice(&[0x11; 32]);
        body.push(0); // empty session id
        body.extend_from_slice(&(2 * 2u16).to_be_bytes());
        body.extend_from_slice(&0x1301u16.to_be_bytes());
        body.extend_from_slice(&0xc02bu16.to_be_bytes());
        body.push(1);
        body.push(0x00);

        let mut exts = Vec::new();
        // supported_groups: 0x001d, 0x0017
        let mut groups = Vec::new();
        groups.extend_from_slice(&0x000au16.to_be_bytes());
        groups.extend_from_slice(&0x0006u16.to_be_bytes()); // ext body length
        groups.extend_from_slice(&0x0004u16.to_be_bytes()); // inner byte-length
        groups.extend_from_slice(&0x001du16.to_be_bytes());
        groups.extend_from_slice(&0x0017u16.to_be_bytes());
        exts.extend_from_slice(&groups);
        // ec_point_formats: [0]
        exts.extend_from_slice(&[0x00, 0x0b, 0x00, 0x02, 0x01, 0x00]);
        // signature_algorithms: 0x0403, 0x0804
        let mut sigs = Vec::new();
        sigs.extend_from_slice(&0x000du16.to_be_bytes());
        sigs.extend_from_slice(&0x0006u16.to_be_bytes()); // ext body length
        sigs.extend_from_slice(&0x0004u16.to_be_bytes()); // inner byte-length
        sigs.extend_from_slice(&0x0403u16.to_be_bytes());
        sigs.extend_from_slice(&0x0804u16.to_be_bytes());
        exts.extend_from_slice(&sigs);
        // ALPN: ["h2", "http/1.1"]
        let mut alpn = Vec::new();
        alpn.extend_from_slice(&0x0010u16.to_be_bytes());
        let entries = [2u8, b'h', b'2', 8u8, b'h', b't', b't', b'p', b'/', b'1', b'.', b'1'];
        let list_len = u16::try_from(entries.len()).expect("tiny list");
        alpn.extend_from_slice(&(2 + list_len).to_be_bytes());
        alpn.extend_from_slice(&list_len.to_be_bytes());
        alpn.extend_from_slice(&entries);
        exts.extend_from_slice(&alpn);

        body.extend_from_slice(&u16::try_from(exts.len()).expect("tiny exts").to_be_bytes());
        body.extend_from_slice(&exts);

        let mut hs = Vec::with_capacity(4 + body.len());
        hs.push(0x01);
        let len = u32::try_from(body.len()).expect("tiny body");
        hs.extend_from_slice(&len.to_be_bytes()[1..]);
        hs.extend_from_slice(&body);

        let parsed: ParsedClientHello = parse_hello(&hs).unwrap();
        let f = Ja3Fields::from(&parsed);
        assert_eq!(f.version, 771);
        assert_eq!(f.ciphers, vec![0x1301, 0xc02b]);
        assert_eq!(f.extensions, vec![0x000a, 0x000b, 0x000d, 0x0010]);
        assert_eq!(f.curves, vec![0x001d, 0x0017]);
        assert_eq!(f.point_formats, vec![0]);
        assert_eq!(f.signature_algorithms, vec![0x0403, 0x0804]);
        assert_eq!(f.alpn, vec!["h2".to_string(), "http/1.1".to_string()]);
    }
}
