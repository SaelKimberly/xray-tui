//! `ClientHello` builder: turns a [`ClientHelloSpec`] into wire bytes.
//!
//! The builder draws GREASE values and random material from a
//! [`SecureRandom`], substitutes GREASE placeholders per the Chrome-family
//! pairing rule (one value across the four slot-A positions, a second for
//! the standalone GREASE extension), and wraps the result in a TLS
//! handshake message and record.

use std::ops::Range;

use crate::error::TlsError;
use crate::spec::grease::GREASE_PLACEHOLDER;
use crate::spec::{ClientHelloSpec, ExtensionSpec, KeyShareGroup, RuntimeValues, SessionIdSpec};

/// Random-byte source for the builder.
///
/// ring's `SecureRandom` is a sealed trait (ring 0.17.13+), so a
/// deterministic test double cannot implement it. This crate-local seam
/// mirrors ring's shape and is blanket-implemented for everything ring's
/// `SecureRandom` accepts: production passes `ring::rand::SystemRandom`,
/// tests pass fixed-seed RNGs.
pub trait SecureRandom {
    /// Fills `dest` with random bytes.
    fn fill(&self, dest: &mut [u8]) -> Result<(), ring::error::Unspecified>;
}

impl<T> SecureRandom for T
where
    T: ring::rand::SecureRandom,
{
    fn fill(&self, dest: &mut [u8]) -> Result<(), ring::error::Unspecified> {
        ring::rand::SecureRandom::fill(self, dest)
    }
}

/// Runtime inputs for a single `ClientHello` build.
pub struct BuildParams<'a> {
    pub server_name: &'a str,
    /// `None` → use the spec's ALPN list.
    pub alpn: Option<&'a [&'a str]>,
    pub x25519_pub: &'a [u8; 32],
    pub rng: &'a dyn SecureRandom,
}

/// The built `ClientHello`.
pub struct BuiltHello {
    /// type `0x01` + 3-byte length + body.
    pub handshake_bytes: Vec<u8>,
    /// handshake record: `0x16 0x0301 <len u16>` + handshake.
    pub record_bytes: Vec<u8>,
    /// Byte range of the session id inside `handshake_bytes`; `Some` only
    /// for [`SessionIdSpec::AuthPayload`] (a REALITY auth payload slot).
    pub session_id_range: Option<Range<usize>>,
}

/// Target total record size for the padding extension. Chrome pads the
/// `ClientHello` handshake message to 512 bytes, i.e. a 517-byte record
/// (`5` record header + `512` handshake).
const PADDING_TARGET: usize = 517;

/// Builds a `ClientHello` handshake message and record from a spec.
pub fn build_hello(spec: &ClientHelloSpec, params: &BuildParams) -> Result<BuiltHello, TlsError> {
    // GREASE pairing (Chrome family): the first placeholder in each of
    // {cipher_suites, supported_groups, supported_versions, key_share} is
    // replaced with `grease_a`; the standalone GREASE extension uses
    // `grease_b`. Additional placeholders get fresh values. Specs without
    // placeholders (Firefox family) skip the draws entirely.
    let (grease_a, grease_b) = if spec_has_grease(spec) {
        (draw_grease(params.rng)?, draw_grease(params.rng)?)
    } else {
        (0x0A0A, 0x1A1A)
    };

    let mut rt = RuntimeValues {
        server_name: params.server_name.to_string(),
        alpn: effective_alpn(spec, params.alpn),
        x25519_pub: *params.x25519_pub,
        grease_a,
        grease_b,
        padding_len: 0,
    };

    // Client random (RFC 8446 §4.1.2).
    let mut random = [0u8; 32];
    params.rng
        .fill(&mut random)
        .map_err(|_| TlsError::Crypto("client random failed".to_string()))?;

    // Legacy session id: 32 random bytes (TLS 1.3 default) or the REALITY
    // auth payload slot (len zero bytes; the range is recorded so a later
    // stage can overwrite them with the payload).
    let (session_id, session_id_range) = match &spec.session_id {
        SessionIdSpec::Random32 => {
            let mut sid = [0u8; 32];
            params
                .rng
                .fill(&mut sid)
                .map_err(|_| TlsError::Crypto("session id random failed".to_string()))?;
            (sid.to_vec(), None)
        }
        SessionIdSpec::AuthPayload { len } => {
            // handshake header (4) + legacy_version (2) + random (32) +
            // session id length byte (1).
            let start = 4 + 2 + 32 + 1;
            (vec![0u8; *len], Some(start..start + len))
        }
    };
    let sid_len = u8::try_from(session_id.len())
        .map_err(|_| TlsError::Spec("session id exceeds 255 bytes".to_string()))?;
    let comp_len = u8::try_from(spec.compression_methods.len())
        .map_err(|_| TlsError::Spec("compression methods exceed 255 bytes".to_string()))?;

    let mut body = Vec::with_capacity(
        2 + 32 + 1 + session_id.len() + 2 + spec.cipher_suites.len() * 2 + 1
            + spec.compression_methods.len()
            + 2
            + spec.extensions.len() * 8,
    );
    body.extend_from_slice(&spec.legacy_version.to_be_bytes());
    body.extend_from_slice(&random);
    body.push(sid_len);
    body.extend_from_slice(&session_id);

    // Cipher suites: first placeholder → grease_a, later ones → fresh.
    let cs_len = u16::try_from(spec.cipher_suites.len() * 2)
        .map_err(|_| TlsError::Spec("cipher suites exceed u16 length".to_string()))?;
    body.extend_from_slice(&cs_len.to_be_bytes());
    let mut first_cs_grease = true;
    for &cs in &spec.cipher_suites {
        let v = if cs == GREASE_PLACEHOLDER {
            if first_cs_grease {
                first_cs_grease = false;
                grease_a
            } else {
                draw_grease(params.rng)?
            }
        } else {
            cs
        };
        body.extend_from_slice(&v.to_be_bytes());
    }

    body.push(comp_len);
    body.extend_from_slice(&spec.compression_methods);

    // Extensions in spec order.
    let mut ext_bytes = Vec::with_capacity(spec.extensions.len() * 8);
    for ext in &spec.extensions {
        match ext {
            ExtensionSpec::Alpn(_) => {
                let encoded = match params.alpn {
                    Some(protos) => {
                        let list: Vec<String> = protos.iter().copied().map(str::to_string).collect();
                        ExtensionSpec::Alpn(list).encode_body(&rt)?
                    }
                    None => ext.encode_body(&rt)?,
                };
                ext_bytes.extend_from_slice(&encoded);
            }
            ExtensionSpec::SupportedGroups(groups) => {
                if groups.contains(&GREASE_PLACEHOLDER) {
                    let mut g = groups.clone();
                    fill_grease(&mut g, grease_a, params.rng)?;
                    ext_bytes.extend_from_slice(&ExtensionSpec::SupportedGroups(g).encode_body(&rt)?);
                } else {
                    ext_bytes.extend_from_slice(&ext.encode_body(&rt)?);
                }
            }
            ExtensionSpec::SupportedVersions(versions) => {
                if versions.contains(&GREASE_PLACEHOLDER) {
                    let mut v = versions.clone();
                    fill_grease(&mut v, grease_a, params.rng)?;
                    ext_bytes.extend_from_slice(
                        &ExtensionSpec::SupportedVersions(v).encode_body(&rt)?,
                    );
                } else {
                    ext_bytes.extend_from_slice(&ext.encode_body(&rt)?);
                }
            }
            ExtensionSpec::Padding => {
                // Accumulated RECORD size without the padding extension:
                // record header (5) + handshake header (4) + version (2) +
                // random (32) + sid (1 + len) + cipher suites (2 + bytes) +
                // compression (1 + len) + extension total length field (2)
                // + already-encoded extensions. Chrome pads so the final
                // record reaches PADDING_TARGET; a zero result omits the
                // extension (matches the reference implementation).
                let acc_record = 5 + 4 + 2 + 32 + 1 + usize::from(sid_len) + 2
                    + spec.cipher_suites.len() * 2 + 1 + usize::from(comp_len) + 2
                    + ext_bytes.len();
                let padding_len = PADDING_TARGET.saturating_sub(acc_record + 4);
                if padding_len > 0 {
                    rt.padding_len = padding_len;
                    ext_bytes.extend_from_slice(&ext.encode_body(&rt)?);
                }
            }
            _ => ext_bytes.extend_from_slice(&ext.encode_body(&rt)?),
        }
    }

    let ext_len = u16::try_from(ext_bytes.len())
        .map_err(|_| TlsError::Spec("extensions exceed u16 length".to_string()))?;
    body.extend_from_slice(&ext_len.to_be_bytes());
    body.extend_from_slice(&ext_bytes);

    // Handshake message: type 0x01 + 3-byte length + body.
    let body_len = u32::try_from(body.len())
        .map_err(|_| TlsError::Spec("client hello body exceeds u32 length".to_string()))?;
    if body_len > 0xFF_FFFF {
        return Err(TlsError::Spec(
            "client hello body exceeds 16 MiB".to_string(),
        ));
    }
    let mut handshake_bytes = Vec::with_capacity(4 + body.len());
    handshake_bytes.push(0x01);
    handshake_bytes.extend_from_slice(&body_len.to_be_bytes()[1..]);
    handshake_bytes.extend_from_slice(&body);

    let record_bytes = to_record(&handshake_bytes);
    Ok(BuiltHello {
        handshake_bytes,
        record_bytes,
        session_id_range,
    })
}

/// Wraps a handshake message in a TLS handshake record:
/// `0x16 0x0301 <len u16 BE> <handshake>`.
#[must_use]
pub fn to_record(handshake_bytes: &[u8]) -> Vec<u8> {
    let mut record = Vec::with_capacity(5 + handshake_bytes.len());
    record.push(0x16); // ContentType: handshake
    record.extend_from_slice(&0x0301u16.to_be_bytes()); // legacy record version
    // A ClientHello never approaches the u16 record length limit; saturate
    // defensively rather than panic on a pathological spec.
    let len = u16::try_from(handshake_bytes.len()).unwrap_or(u16::MAX);
    record.extend_from_slice(&len.to_be_bytes());
    record.extend_from_slice(handshake_bytes);
    record
}

/// Returns `true` if the spec contains any GREASE placeholder slot.
fn spec_has_grease(spec: &ClientHelloSpec) -> bool {
    spec.cipher_suites.contains(&GREASE_PLACEHOLDER)
        || spec.extensions.iter().any(|e| match e {
            ExtensionSpec::SupportedGroups(groups) => groups.contains(&GREASE_PLACEHOLDER),
            ExtensionSpec::SupportedVersions(versions) => versions.contains(&GREASE_PLACEHOLDER),
            ExtensionSpec::KeyShare(groups) => {
                groups.iter().any(|g| matches!(g, KeyShareGroup::Grease))
            }
            ExtensionSpec::Grease => true,
            _ => false,
        })
}

/// Replaces `GREASE_PLACEHOLDER` slots in `values`: the first becomes
/// `grease_a`, later ones get freshly drawn GREASE values.
fn fill_grease(
    values: &mut [u16],
    grease_a: u16,
    rng: &dyn SecureRandom,
) -> Result<(), TlsError> {
    let mut first = true;
    for v in values.iter_mut() {
        if *v == GREASE_PLACEHOLDER {
            *v = if first {
                first = false;
                grease_a
            } else {
                draw_grease(rng)?
            };
        }
    }
    Ok(())
}

/// Draws a GREASE value uniformly from the 16 RFC 8701 values.
///
/// Mirrors `spec::grease::random_grease`, which is bounded to ring's sealed
/// `SecureRandom` and therefore unusable through the crate-local seam.
fn draw_grease(rng: &dyn SecureRandom) -> Result<u16, TlsError> {
    let mut byte = [0u8; 1];
    rng.fill(&mut byte)
        .map_err(|_| TlsError::Crypto("random grease failed".to_string()))?;
    // The low nibble indexes the 16 GREASE byte values 0x0A..0xFA.
    let b = 0x0A + (byte[0] & 0x0F) * 0x10;
    Ok((u16::from(b) << 8) | u16::from(b))
}

/// The ALPN list to inject: `params.alpn` wins, else the spec's first ALPN
/// extension, else empty.
fn effective_alpn(spec: &ClientHelloSpec, override_alpn: Option<&[&str]>) -> Vec<String> {
    override_alpn.map_or_else(
        || {
            spec.extensions
                .iter()
                .find_map(|e| match e {
                    ExtensionSpec::Alpn(list) => Some(list.clone()),
                    _ => None,
                })
                .unwrap_or_default()
        },
        |protos| protos.iter().copied().map(str::to_string).collect(),
    )
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use super::{build_hello, to_record, BuildParams, SecureRandom};
    use crate::spec::grease::GREASE_PLACEHOLDER;
    use crate::spec::{ClientHelloSpec, ExtensionSpec, KeyShareGroup, SessionIdSpec};

    /// Deterministic RNG feeding back a fixed byte sequence.
    struct FixedRandom {
        bytes: Vec<u8>,
        pos: Cell<usize>,
    }

    impl SecureRandom for FixedRandom {
        fn fill(&self, dest: &mut [u8]) -> Result<(), ring::error::Unspecified> {
            let mut pos = self.pos.get();
            for b in dest.iter_mut() {
                *b = *self.bytes.get(pos).ok_or(ring::error::Unspecified)?;
                pos += 1;
            }
            self.pos.set(pos);
            Ok(())
        }
    }

    /// Decodes a hex digit into its value (test helper).
    fn hex_val(b: u8) -> u8 {
        match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => panic!("invalid hex digit: {b:#x}"),
        }
    }

    /// Decodes a hex string into bytes (test helper).
    fn decode_hex(s: &str) -> Vec<u8> {
        assert_eq!(s.len() % 2, 0, "hex string must have even length");
        s.as_bytes()
            .chunks_exact(2)
            .map(|c| (hex_val(c[0]) << 4) | hex_val(c[1]))
            .collect()
    }

    /// Golden Chrome 130 `ClientHello` captured from the reference
    /// implementation (`tls-fingerprint` crate, same profile + fixed seed):
    /// RNG bytes all `0x42` (so every GREASE value is `0x2A2A`), X25519
    /// public key `[0xAB; 32]`, no padding extension (kept zero for this
    /// test).
    const EXPECTED_HELLO_HEX: &str = "0100012b0303424242424242424242424242424242424242424242424242424242424242424220424242424242424242424242424242424242424242424242424242424242424200202a2a130113021303c02bc02fc02cc030cca9cca8c013c014009c009d002f0035010000c22a2a00010000000010000e00000b746c732e706565742e777300170000ff01000100000a000a00082a2a001d00170018000b00020100002300000010000e000c02683208687474702f312e31000500050100000000001200000033002b00292a2a000100001d0020abababababababababababababababababababababababababababababababab002d00020101002b0007062a2a03040303001b0005040002000344690006000400026832000d0012001004030804040105030805050108060601";

    /// Same hello wrapped in a handshake record (`0x16 0x0301` + length).
    const EXPECTED_RECORD_HEX: &str = "160301012f0100012b0303424242424242424242424242424242424242424242424242424242424242424220424242424242424242424242424242424242424242424242424242424242424200202a2a130113021303c02bc02fc02cc030cca9cca8c013c014009c009d002f0035010000c22a2a00010000000010000e00000b746c732e706565742e777300170000ff01000100000a000a00082a2a001d00170018000b00020100002300000010000e000c02683208687474702f312e31000500050100000000001200000033002b00292a2a000100001d0020abababababababababababababababababababababababababababababababab002d00020101002b0007062a2a03040303001b0005040002000344690006000400026832000d0012001004030804040105030805050108060601";

    /// Chrome 130 spec assembled from the constants in
    /// `thirdparty/tls-fingerprint/src/profiles/chrome.rs`. Placeholder for
    /// the real `profiles::chrome::v130::spec()` (Task 10).
    fn test_spec() -> ClientHelloSpec {
        ClientHelloSpec {
            legacy_version: 0x0303,
            cipher_suites: vec![
                GREASE_PLACEHOLDER,
                0x1301,
                0x1302,
                0x1303,
                0xC02B,
                0xC02F,
                0xC02C,
                0xC030,
                0xCCA9,
                0xCCA8,
                0xC013,
                0xC014,
                0x009C,
                0x009D,
                0x002F,
                0x0035,
            ],
            compression_methods: vec![0x00],
            session_id: SessionIdSpec::Random32,
            extensions: vec![
                ExtensionSpec::Grease,
                ExtensionSpec::ServerName,
                // extended_master_secret (no dedicated variant).
                ExtensionSpec::Raw { ty: 0x0017, data: Vec::new() },
                ExtensionSpec::RenegotiationInfo,
                ExtensionSpec::SupportedGroups(vec![GREASE_PLACEHOLDER, 0x001D, 0x0017, 0x0018]),
                ExtensionSpec::EcPointFormats,
                ExtensionSpec::SessionTicket,
                ExtensionSpec::Alpn(vec!["h2".into(), "http/1.1".into()]),
                ExtensionSpec::StatusRequest,
                ExtensionSpec::SignedCertificateTimestamp,
                ExtensionSpec::KeyShare(vec![KeyShareGroup::Grease, KeyShareGroup::X25519]),
                ExtensionSpec::PskKeyExchangeModes,
                ExtensionSpec::SupportedVersions(vec![GREASE_PLACEHOLDER, 0x0304, 0x0303]),
                ExtensionSpec::CompressCertificate(vec![0x0002, 0x0003]),
                ExtensionSpec::ApplicationSettings(vec!["h2".into()]),
                ExtensionSpec::SignatureAlgorithms(vec![
                    0x0403, 0x0804, 0x0401, 0x0503, 0x0805, 0x0501, 0x0806, 0x0601,
                ]),
                // No Padding — the golden test keeps padding zero.
            ],
        }
    }

    #[test]
    fn chrome130_golden_hello_with_fixed_seed() {
        // Reference capture (run the CURRENT tls-fingerprint example with
        // the same fixed seed): RNG all 0x42 → every GREASE value 0x2A2A,
        // random and session id all 0x42, X25519 public key [0xAB; 32], no
        // padding. The ported builder must produce byte-identical output.
        let spec = test_spec();
        // 2 GREASE bytes + 32 random + 32 session id = 66 bytes needed; give
        // the fixed RNG headroom so exhaustion is never the failure mode.
        let rng = FixedRandom { bytes: vec![0x42; 128], pos: Cell::new(0) };
        let hello = build_hello(
            &spec,
            &BuildParams {
                server_name: "tls.peet.ws",
                alpn: Some(&["h2", "http/1.1"]),
                x25519_pub: &[0xAB; 32],
                rng: &rng,
            },
        )
        .unwrap();

        assert_eq!(hello.handshake_bytes, decode_hex(EXPECTED_HELLO_HEX));
        assert_eq!(hello.record_bytes, decode_hex(EXPECTED_RECORD_HEX));
        assert_eq!(hello.record_bytes, to_record(&hello.handshake_bytes));
        assert!(hello.session_id_range.is_none());
        // grease_a + grease_b + random + session id all came from the RNG.
        assert_eq!(rng.pos.get(), 66);
    }

    #[test]
    fn grease_pairing_uses_a_for_slots_and_b_for_standalone_ext() {
        // Byte 0x00 → GREASE 0x0A0A (slot A), byte 0x01 → GREASE 0x1A1A
        // (slot B, standalone extension type). Distinct values prove the
        // pairing rule, not just equal-value GREASE everywhere.
        let spec = ClientHelloSpec {
            legacy_version: 0x0303,
            cipher_suites: vec![GREASE_PLACEHOLDER, 0x1301, 0x1302],
            compression_methods: vec![0x00],
            session_id: SessionIdSpec::Random32,
            extensions: vec![
                ExtensionSpec::Grease,
                ExtensionSpec::SupportedGroups(vec![GREASE_PLACEHOLDER, 0x001D]),
                ExtensionSpec::KeyShare(vec![KeyShareGroup::Grease, KeyShareGroup::X25519]),
                ExtensionSpec::SupportedVersions(vec![GREASE_PLACEHOLDER, 0x0304]),
            ],
        };
        let mut bytes = vec![0x00, 0x01];
        bytes.extend_from_slice(&[0x42; 64]);
        let rng = FixedRandom { bytes, pos: Cell::new(0) };
        let hello = build_hello(
            &spec,
            &BuildParams {
                server_name: "example.com",
                alpn: None,
                x25519_pub: &[0xAB; 32],
                rng: &rng,
            },
        )
        .unwrap();

        let parsed = parse_hello(&hello.handshake_bytes);
        // cipher_suites[0] is the first slot-A placeholder.
        assert_eq!(parsed.cipher_suites[0], 0x0A0A);
        assert_eq!(parsed.cipher_suites[1], 0x1301);
        // The standalone GREASE extension carries grease_b as its type.
        assert!(parsed.extensions.iter().any(|(ty, _)| *ty == 0x1A1A));
        // supported_groups[0] → grease_a (skip the u16 byte-length prefix).
        let groups = parsed.extension(0x000A).expect("supported_groups");
        let groups: Vec<u16> = groups[2..]
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        assert_eq!(groups[0], 0x0A0A);
        // key_share first entry group → grease_a (skip the u16 list-length
        // prefix).
        let key_share = parsed.extension(0x0033).expect("key_share");
        let ks_groups: Vec<u16> = key_share[2..]
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        assert_eq!(ks_groups[0], 0x0A0A);
        // supported_versions[0] → grease_a.
        let versions = parsed.extension(0x002B).expect("supported_versions");
        assert_eq!(u16::from_be_bytes([versions[1], versions[2]]), 0x0A0A);
    }

    #[test]
    fn additional_placeholders_get_fresh_grease_values() {
        // The second cipher-suite placeholder must not reuse grease_a: the
        // fresh draw (rng byte 66, all 0x42 → 0x2A2A) lands in its slot.
        let spec = ClientHelloSpec {
            legacy_version: 0x0303,
            cipher_suites: vec![GREASE_PLACEHOLDER, GREASE_PLACEHOLDER, 0x1301],
            compression_methods: vec![0x00],
            session_id: SessionIdSpec::Random32,
            extensions: vec![],
        };
        let mut bytes = vec![0x00, 0x01];
        bytes.extend_from_slice(&[0x42; 66]);
        let rng = FixedRandom { bytes, pos: Cell::new(0) };
        let hello = build_hello(
            &spec,
            &BuildParams {
                server_name: "example.com",
                alpn: None,
                x25519_pub: &[0xAB; 32],
                rng: &rng,
            },
        )
        .unwrap();

        let parsed = parse_hello(&hello.handshake_bytes);
        assert_eq!(parsed.cipher_suites[0], 0x0A0A);
        assert_eq!(parsed.cipher_suites[1], 0x2A2A);
        assert_eq!(parsed.cipher_suites[2], 0x1301);
    }

    #[test]
    fn padding_extension_pads_record_to_517_bytes() {
        // Full Chrome 130 layout + padding: Chrome pads the ClientHello
        // handshake to 512 bytes (record 517 total).
        let mut spec = test_spec();
        spec.extensions.push(ExtensionSpec::Padding);
        let rng = FixedRandom { bytes: vec![0x42; 128], pos: Cell::new(0) };
        let hello = build_hello(
            &spec,
            &BuildParams {
                server_name: "tls.peet.ws",
                alpn: Some(&["h2", "http/1.1"]),
                x25519_pub: &[0xAB; 32],
                rng: &rng,
            },
        )
        .unwrap();

        assert_eq!(hello.handshake_bytes.len(), 512);
        assert_eq!(hello.record_bytes.len(), 517);
        let parsed = parse_hello(&hello.handshake_bytes);
        let padding = parsed.extension(0x0015).expect("padding extension");
        assert_eq!(padding.len(), 205);
        assert!(padding.iter().all(|&b| b == 0));
    }

    #[test]
    fn auth_payload_session_id_is_zeroed_and_ranged() {
        let spec = ClientHelloSpec {
            legacy_version: 0x0303,
            cipher_suites: vec![0x1301],
            compression_methods: vec![0x00],
            session_id: SessionIdSpec::AuthPayload { len: 42 },
            extensions: vec![],
        };
        let rng = FixedRandom { bytes: vec![0x42; 64], pos: Cell::new(0) };
        let hello = build_hello(
            &spec,
            &BuildParams {
                server_name: "example.com",
                alpn: None,
                x25519_pub: &[0xAB; 32],
                rng: &rng,
            },
        )
        .unwrap();

        let range = hello.session_id_range.expect("auth payload range");
        assert_eq!(range, 39..81);
        let sid = &hello.handshake_bytes[range];
        assert_eq!(sid.len(), 42);
        assert!(sid.iter().all(|&b| b == 0));
        // No GREASE in this spec → the RNG only fed the 32-byte random.
        assert_eq!(rng.pos.get(), 32);
    }

    #[test]
    fn alpn_param_overrides_spec_list() {
        let spec = ClientHelloSpec {
            legacy_version: 0x0303,
            cipher_suites: vec![0x1301],
            compression_methods: vec![0x00],
            session_id: SessionIdSpec::Random32,
            extensions: vec![ExtensionSpec::Alpn(vec!["spec-only".into()])],
        };
        let build = |alpn: Option<&[&str]>| {
            let rng = FixedRandom { bytes: vec![0x42; 64], pos: Cell::new(0) };
            build_hello(
                &spec,
                &BuildParams {
                    server_name: "example.com",
                    alpn,
                    x25519_pub: &[0xAB; 32],
                    rng: &rng,
                },
            )
            .unwrap()
        };

        // Some(...) replaces the spec's ALPN list.
        let overridden = build(Some(&["h2"]));
        let parsed = parse_hello(&overridden.handshake_bytes);
        let alpn = parsed.extension(0x0010).expect("alpn");
        assert!(alpn.windows(2).any(|w| w == b"h2".as_slice()));
        assert!(!alpn.windows(2).any(|w| w == b"sp".as_slice()));

        // None keeps the spec's list.
        let spec_alpn = build(None);
        let parsed = parse_hello(&spec_alpn.handshake_bytes);
        let alpn = parsed.extension(0x0010).expect("alpn");
        assert!(alpn.windows(9).any(|w| w == b"spec-only".as_slice()));
    }

    #[test]
    fn no_grease_spec_does_not_consume_grease_bytes() {
        // Firefox-family specs contain no placeholders: the RNG must only
        // feed random + session id (64 bytes).
        let spec = ClientHelloSpec {
            legacy_version: 0x0303,
            cipher_suites: vec![0x1301, 0x1302],
            compression_methods: vec![0x00],
            session_id: SessionIdSpec::Random32,
            extensions: vec![ExtensionSpec::SupportedGroups(vec![0x001D])],
        };
        let rng = FixedRandom { bytes: vec![0x42; 64], pos: Cell::new(0) };
        let hello = build_hello(
            &spec,
            &BuildParams {
                server_name: "example.com",
                alpn: None,
                x25519_pub: &[0xAB; 32],
                rng: &rng,
            },
        )
        .unwrap();
        assert_eq!(rng.pos.get(), 64);
        assert!(hello.session_id_range.is_none());
    }

    /// Parsed view of a `ClientHello` handshake message (test helper).
    struct ParsedHello {
        cipher_suites: Vec<u16>,
        extensions: Vec<(u16, Vec<u8>)>,
    }

    impl ParsedHello {
        /// Returns the body of the extension with the given type.
        fn extension(&self, ty: u16) -> Option<&[u8]> {
            self.extensions
                .iter()
                .find(|(t, _)| *t == ty)
                .map(|(_, data)| data.as_slice())
        }
    }

    /// Parses a `ClientHello` handshake message into its top-level fields.
    fn parse_hello(hs: &[u8]) -> ParsedHello {
        assert_eq!(hs[0], 0x01, "handshake type");
        let body_len = (usize::from(hs[1]) << 16) | (usize::from(hs[2]) << 8) | usize::from(hs[3]);
        assert_eq!(body_len, hs.len() - 4, "handshake body length");
        let mut off = 4;
        off += 2; // legacy_version
        off += 32; // random
        let sid_len = usize::from(hs[off]);
        off += 1 + sid_len; // session id
        let cs_len = usize::from(u16::from_be_bytes([hs[off], hs[off + 1]]));
        off += 2;
        let mut cipher_suites = Vec::with_capacity(cs_len / 2);
        for _ in 0..cs_len / 2 {
            cipher_suites.push(u16::from_be_bytes([hs[off], hs[off + 1]]));
            off += 2;
        }
        let comp_len = usize::from(hs[off]);
        off += 1 + comp_len; // compression methods
        let ext_total = usize::from(u16::from_be_bytes([hs[off], hs[off + 1]]));
        off += 2;
        let ext_end = off + ext_total;
        let mut extensions = Vec::new();
        while off < ext_end {
            let ty = u16::from_be_bytes([hs[off], hs[off + 1]]);
            let len = usize::from(u16::from_be_bytes([hs[off + 2], hs[off + 3]]));
            extensions.push((ty, hs[off + 4..off + 4 + len].to_vec()));
            off += 4 + len;
        }
        assert_eq!(off, ext_end, "extensions consumed exactly");
        ParsedHello { cipher_suites, extensions }
    }
}
