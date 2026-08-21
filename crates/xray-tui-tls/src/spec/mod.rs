//! `ClientHello` specification model and extension wire encoding.
//!
//! [`ClientHelloSpec`] describes a `ClientHello` at the semantic level; the
//! extension arms here encode to the exact RFC 6066/8446 wire format.
//! Browser fingerprint profiles (built by later tasks) are expressed in
//! terms of these types.

pub mod grease;

use crate::error::TlsError;

/// Runtime values injected into a spec at send time.
///
/// Task 3 fills this from real connection state; `Default` values are used
/// by the encoding tests below.
#[derive(Debug, PartialEq, Eq)]
pub struct RuntimeValues {
    pub server_name: String,
    pub alpn: Vec<String>,
    pub x25519_pub: [u8; 32],
    /// ML-KEM-768 encapsulation key for hybrid key shares (1184 bytes;
    /// empty when the spec has no hybrid key-share entry).
    pub mlkem768_pub: Vec<u8>,
    pub grease_a: u16,
    pub grease_b: u16,
    pub padding_len: usize,
}

impl Default for RuntimeValues {
    fn default() -> Self {
        Self {
            server_name: String::new(),
            alpn: Vec::new(),
            x25519_pub: [0; 32],
            mlkem768_pub: Vec::new(),
            grease_a: 0x0A0A,
            grease_b: 0x1A1A,
            padding_len: 0,
        }
    }
}

/// How the `ClientHello` session id is produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionIdSpec {
    /// 32 random bytes, the TLS 1.3 default.
    Random32,
    /// No legacy session id (0 bytes) — the Safari family.
    Empty,
    /// Placeholder for a REALITY authentication payload;
    /// `len` is the full wire length (plaintext + 16-byte tag).
    AuthPayload { len: usize },
}

/// A single TLS extension in the `ClientHello`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionSpec {
    ServerName,
    SupportedGroups(Vec<u16>),
    KeyShare(Vec<KeyShareGroup>),
    SupportedVersions(Vec<u16>),
    SignatureAlgorithms(Vec<u16>),
    Alpn(Vec<String>),
    EcPointFormats,
    SessionTicket,
    PskKeyExchangeModes,
    StatusRequest,
    SignedCertificateTimestamp,
    RenegotiationInfo,
    CompressCertificate(Vec<u16>),
    ApplicationSettings(Vec<String>),
    RecordSizeLimit(u16),
    Padding,
    Grease,
    Raw { ty: u16, data: Vec<u8> },
}

/// One key-share entry in the `key_share` extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyShareGroup {
    /// A GREASE group; the wire value comes from `RuntimeValues::grease_a`.
    Grease,
    X25519,
    /// X25519MLKEM768 hybrid (0x11EC / 4588): the entry key exchange is
    /// `X25519 pub (32) || ML-KEM-768 encapsulation key (1184)` = 1216 bytes.
    X25519Mlkem768,
    /// `SecP256r1MLKEM768` hybrid (0x11EB / 4587). Deferred: the engine has no
    /// P-256 key exchange (xray's primary hybrid is `X25519MLKEM768`).
    Secp256r1Mlkem768,
    /// `SecP384r1MLKEM1024` hybrid (0x11ED / 4589). Deferred: the engine has
    /// no P-384 key exchange (ML-KEM-1024 itself is available via liboqs).
    Secp384r1Mlkem1024,
}

/// The semantic `ClientHello` description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientHelloSpec {
    /// TLS legacy record version, always `0x0303` for TLS 1.3.
    pub legacy_version: u16,
    /// Cipher suites; `GREASE_PLACEHOLDER` is allowed as a slot.
    pub cipher_suites: Vec<u16>,
    /// Compression methods; `[0]` for TLS 1.3.
    pub compression_methods: Vec<u8>,
    pub session_id: SessionIdSpec,
    pub extensions: Vec<ExtensionSpec>,
}

/// Apply config-driven curve preferences to a spec — the client-side mirror
/// of xray's `CurvePreferences` handling (`transport/internet/tls/config.go`).
///
/// - `supported_groups` is replaced by `curves` verbatim (xray replaces the
///   offered list; the config override intentionally departs from the
///   fingerprint profile's list).
/// - `key_share` keeps the profile's GREASE entries (fingerprint-critical,
///   orthogonal to curve selection) and gains one entry per configured ID
///   that has a key-share group: X25519 and the three ML-KEM hybrids. The
///   classical P-256/P-384/P-521 IDs are advertised in `supported_groups`
///   but produce no key share — the engine implements no P-curve key
///   exchange (see [`KeyShareGroup`]).
///
/// Extensions the spec does not carry are left absent.
#[must_use]
pub fn apply_curve_preferences(spec: &ClientHelloSpec, curves: &[u16]) -> ClientHelloSpec {
    let key_share_group = |id: u16| match id {
        0x001D => Some(KeyShareGroup::X25519),
        0x11EC => Some(KeyShareGroup::X25519Mlkem768),
        0x11EB => Some(KeyShareGroup::Secp256r1Mlkem768),
        0x11ED => Some(KeyShareGroup::Secp384r1Mlkem1024),
        _ => None,
    };
    let mut out = spec.clone();
    for ext in &mut out.extensions {
        match ext {
            ExtensionSpec::SupportedGroups(_) => {
                *ext = ExtensionSpec::SupportedGroups(curves.to_vec());
            }
            ExtensionSpec::KeyShare(groups) => {
                let mut rewritten: Vec<KeyShareGroup> = groups
                    .iter()
                    .filter(|g| matches!(g, KeyShareGroup::Grease))
                    .cloned()
                    .collect();
                rewritten.extend(curves.iter().filter_map(|id| key_share_group(*id)));
                *groups = rewritten;
            }
            _ => {}
        }
    }
    out
}

impl ExtensionSpec {
    /// Encodes the COMPLETE extension: type (u16 BE) + length (u16 BE) + body.
    ///
    /// The length field counts the bytes after itself.
    pub fn encode_body(&self, rt: &RuntimeValues) -> Result<Vec<u8>, TlsError> {
        let (ty, body) = match self {
            Self::ServerName => {
                let host = rt.server_name.as_bytes();
                let host_len = u16::try_from(host.len()).map_err(|_| {
                    TlsError::Spec("server_name host exceeds u16 length".to_string())
                })?;
                // RFC 6066: ServerNameList { list_length u16, name_type 00, host_name_length u16, host_name }
                let mut body = Vec::with_capacity(3 + host.len());
                body.extend_from_slice(&(1 + 2 + host_len).to_be_bytes());
                body.push(0x00);
                body.extend_from_slice(&host_len.to_be_bytes());
                body.extend_from_slice(host);
                (0x0000, body)
            }
            Self::SupportedGroups(groups) => {
                // RFC 8446 NamedGroupList: u16 byte-length + groups, no count field.
                let byte_len = u16::try_from(groups.len() * 2).map_err(|_| {
                    TlsError::Spec("supported_groups exceeds u16 length".to_string())
                })?;
                let mut body = Vec::with_capacity(2 + groups.len() * 2);
                body.extend_from_slice(&byte_len.to_be_bytes());
                for group in groups {
                    body.extend_from_slice(&group.to_be_bytes());
                }
                (0x000a, body)
            }
            Self::KeyShare(groups) => {
                // RFC 8446 KeyShareClientHello: u16 list-length + entries.
                let mut entries = Vec::with_capacity(groups.len() * 36);
                for group in groups {
                    match group {
                        KeyShareGroup::Grease => {
                            // Entry: group (grease_a), key_exchange_length 00 01, key_exchange 00.
                            entries.extend_from_slice(&rt.grease_a.to_be_bytes());
                            entries.extend_from_slice(&[0x00, 0x01, 0x00]);
                        }
                        KeyShareGroup::X25519 => {
                            // Entry: group 00 1d (x25519), key_exchange_length 00 20, raw public key.
                            entries.extend_from_slice(&[0x00, 0x1d, 0x00, 0x20]);
                            entries.extend_from_slice(&rt.x25519_pub);
                        }
                        KeyShareGroup::X25519Mlkem768 => {
                            if rt.mlkem768_pub.len() != 1184 {
                                return Err(TlsError::Spec(format!(
                                    "X25519MLKEM768 key share requires an ML-KEM-768 encapsulation key of 1184 bytes, got {}",
                                    rt.mlkem768_pub.len()
                                )));
                            }
                            // Entry: group 11 ec, key_exchange_length 04 c0 (1216),
                            // key_exchange = X25519 pub (32) || ML-KEM-768 encap key (1184).
                            entries.extend_from_slice(&[0x11, 0xec, 0x04, 0xc0]);
                            entries.extend_from_slice(&rt.x25519_pub);
                            entries.extend_from_slice(&rt.mlkem768_pub);
                        }
                        KeyShareGroup::Secp256r1Mlkem768 | KeyShareGroup::Secp384r1Mlkem1024 => {
                            return Err(TlsError::Spec(
                                "SecP256r1MLKEM768/SecP384r1MLKEM1024 key shares are not supported: the engine implements no P-256/P-384 key exchange (xray's primary hybrid, X25519MLKEM768, is fully supported)".to_string(),
                            ));
                        }
                    }
                }
                let list_len = u16::try_from(entries.len()).map_err(|_| {
                    TlsError::Spec("key_share entries exceed u16 length".to_string())
                })?;
                let mut body = Vec::with_capacity(2 + entries.len());
                body.extend_from_slice(&list_len.to_be_bytes());
                body.extend_from_slice(&entries);
                (0x0033, body)
            }
            Self::SupportedVersions(versions) => {
                // RFC 8446: 1-byte length counts BYTES (n*2), not versions.
                let byte_len = u8::try_from(versions.len() * 2).map_err(|_| {
                    TlsError::Spec("supported_versions exceeds 255 bytes".to_string())
                })?;
                let mut body = Vec::with_capacity(1 + versions.len() * 2);
                body.push(byte_len);
                for version in versions {
                    body.extend_from_slice(&version.to_be_bytes());
                }
                (0x002b, body)
            }
            Self::SignatureAlgorithms(schemes) => {
                // RFC 8446 SignatureSchemeList: u16 byte-length + schemes, no count field.
                let byte_len = u16::try_from(schemes.len() * 2).map_err(|_| {
                    TlsError::Spec("signature_algorithms exceeds u16 length".to_string())
                })?;
                let mut body = Vec::with_capacity(2 + schemes.len() * 2);
                body.extend_from_slice(&byte_len.to_be_bytes());
                for scheme in schemes {
                    body.extend_from_slice(&scheme.to_be_bytes());
                }
                (0x000d, body)
            }
            Self::Alpn(protos) => (0x0010, prepend_list_len(&encode_alpn_list(protos)?)?),
            Self::EcPointFormats => (0x000b, vec![0x01, 0x00]),
            Self::SessionTicket => (0x0023, Vec::new()),
            Self::PskKeyExchangeModes => (0x002d, vec![0x01, 0x01]),
            Self::StatusRequest => (0x0005, vec![0x01, 0x00, 0x00, 0x00, 0x00]),
            Self::SignedCertificateTimestamp => (0x0012, Vec::new()),
            Self::RenegotiationInfo => (0xff01, vec![0x00]),
            Self::CompressCertificate(algos) => {
                // RFC 8871: 1-byte length counts BYTES + algos, no count field.
                let byte_len = u8::try_from(algos.len() * 2).map_err(|_| {
                    TlsError::Spec("compress_certificate exceeds 255 bytes".to_string())
                })?;
                let mut body = Vec::with_capacity(1 + algos.len() * 2);
                body.push(byte_len);
                for algo in algos {
                    body.extend_from_slice(&algo.to_be_bytes());
                }
                (0x001b, body)
            }
            Self::ApplicationSettings(protos) => {
                // ALPS (draft-ietf-tls-alps): 2-byte per-entry lengths (differs
                // from ALPN), u16 list-length prefix.
                (0x4469, prepend_list_len(&encode_alps_list(protos)?)?)
            }
            Self::RecordSizeLimit(limit) => (0x001c, limit.to_be_bytes().to_vec()),
            Self::Padding => (0x0015, vec![0u8; rt.padding_len]),
            Self::Grease => (rt.grease_b, vec![0x00]),
            Self::Raw { ty, data } => (*ty, data.clone()),
        };
        let len = u16::try_from(body.len())
            .map_err(|_| TlsError::Spec("extension body exceeds u16 length".to_string()))?;
        let mut out = Vec::with_capacity(4 + body.len());
        out.extend_from_slice(&ty.to_be_bytes());
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(&body);
        Ok(out)
    }
}

/// Prefixes a protocol list with its u16 BE byte-length (the RFC vector
/// shape shared by ALPN and ALPS).
fn prepend_list_len(list: &[u8]) -> Result<Vec<u8>, TlsError> {
    let list_len = u16::try_from(list.len())
        .map_err(|_| TlsError::Spec("protocol list exceeds u16 length".to_string()))?;
    let mut out = Vec::with_capacity(2 + list.len());
    out.extend_from_slice(&list_len.to_be_bytes());
    out.extend_from_slice(list);
    Ok(out)
}

/// Encodes an ALPN protocol list (RFC 7301): per entry, a u8 BE length
/// followed by the raw protocol bytes. Returns just the entries; the caller
/// prepends the list-length field.
fn encode_alpn_list(protos: &[String]) -> Result<Vec<u8>, TlsError> {
    let mut out = Vec::new();
    for proto in protos {
        let len = u8::try_from(proto.len())
            .map_err(|_| TlsError::Spec("alpn protocol exceeds 255 bytes".to_string()))?;
        out.push(len);
        out.extend_from_slice(proto.as_bytes());
    }
    Ok(out)
}

/// Encodes an ALPS protocol list (draft-ietf-tls-alps): per entry, a u16 BE
/// length followed by the raw protocol bytes — 2-byte entries, unlike ALPN.
fn encode_alps_list(protos: &[String]) -> Result<Vec<u8>, TlsError> {
    let mut out = Vec::new();
    for proto in protos {
        let len = u16::try_from(proto.len()).map_err(|_| {
            TlsError::Spec("application_settings protocol exceeds u16 length".to_string())
        })?;
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(proto.as_bytes());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::grease::is_grease;
    use super::*;
    use rstest::rstest;

    #[test]
    fn grease_detection() {
        assert!(is_grease(0x0A0A) && is_grease(0xCACA) && is_grease(0xFAFA));
        assert!(!is_grease(0x1301) && !is_grease(0x1516) && !is_grease(0x0000));
    }

    /// A minimal spec carrying the two curve-bearing extensions.
    fn curve_spec() -> ClientHelloSpec {
        ClientHelloSpec {
            legacy_version: 0x0303,
            cipher_suites: vec![0x1301],
            compression_methods: vec![0],
            session_id: SessionIdSpec::Random32,
            extensions: vec![
                ExtensionSpec::SupportedGroups(vec![0x11EC, 0x001D, 0x0017]),
                ExtensionSpec::KeyShare(vec![
                    KeyShareGroup::Grease,
                    KeyShareGroup::X25519Mlkem768,
                    KeyShareGroup::X25519,
                ]),
            ],
        }
    }

    #[test]
    fn curve_preferences_replace_supported_groups_and_key_share() {
        let out = apply_curve_preferences(&curve_spec(), &[0x001D, 0x11EC]);
        assert_eq!(
            out.extensions[0],
            ExtensionSpec::SupportedGroups(vec![0x001D, 0x11EC])
        );
        // GREASE kept, entries re-derived from the configured IDs in order.
        assert_eq!(
            out.extensions[1],
            ExtensionSpec::KeyShare(vec![
                KeyShareGroup::Grease,
                KeyShareGroup::X25519,
                KeyShareGroup::X25519Mlkem768
            ])
        );
    }

    #[test]
    fn curve_preferences_map_all_hybrid_and_classical_ids() {
        // P-384/P-521 advertise but yield no key share (no P-curve KEX);
        // the three ML-KEM hybrids map to their key-share groups.
        let out = apply_curve_preferences(&curve_spec(), &[0x0018, 0x0019, 0x11EB, 0x11EC, 0x11ED]);
        assert_eq!(
            out.extensions[1],
            ExtensionSpec::KeyShare(vec![
                KeyShareGroup::Grease,
                KeyShareGroup::Secp256r1Mlkem768,
                KeyShareGroup::X25519Mlkem768,
                KeyShareGroup::Secp384r1Mlkem1024,
            ])
        );
    }

    #[test]
    fn curve_preferences_leave_absent_extensions_absent() {
        let mut spec = curve_spec();
        spec.extensions.retain(|e| {
            !matches!(
                e,
                ExtensionSpec::SupportedGroups(_) | ExtensionSpec::KeyShare(_)
            )
        });
        let out = apply_curve_preferences(&spec, &[0x001D]);
        assert!(out.extensions.is_empty());
        // The input spec is untouched (borrow → clone semantics).
        assert_eq!(
            curve_spec().extensions[0],
            ExtensionSpec::SupportedGroups(vec![0x11EC, 0x001D, 0x0017])
        );
    }

    #[rstest]
    #[case::server_name(
        ExtensionSpec::ServerName,
        RuntimeValues { server_name: "example.com".into(), ..RuntimeValues::default() },
        vec![0x00, 0x00, 0x00, 0x10, 0x00, 0x0e, 0x00, 0x00, 0x0b, b'e', b'x', b'a', b'm',
             b'p', b'l', b'e', b'.', b'c', b'o', b'm']
    )]
    #[case::supported_groups(
        ExtensionSpec::SupportedGroups(vec![0x1301, 0x1302, 0x1303]),
        RuntimeValues::default(),
        vec![0x00, 0x0a, 0x00, 0x08, 0x00, 0x06, 0x13, 0x01, 0x13, 0x02, 0x13, 0x03]
    )]
    #[case::key_share(
        ExtensionSpec::KeyShare(vec![KeyShareGroup::Grease, KeyShareGroup::X25519]),
        RuntimeValues { grease_a: 0x1A1A, x25519_pub: [0xAB; 32], ..RuntimeValues::default() },
        {
            let mut v = vec![0x00, 0x33, 0x00, 0x2b, 0x00, 0x29, 0x1a, 0x1a, 0x00, 0x01, 0x00,
                             0x00, 0x1d, 0x00, 0x20];
            v.extend_from_slice(&[0xAB; 32]);
            v
        }
    )]
    #[case::supported_versions(
        ExtensionSpec::SupportedVersions(vec![0x0A0A, 0x0304, 0x0303]),
        RuntimeValues::default(),
        vec![0x00, 0x2b, 0x00, 0x07, 0x06, 0x0a, 0x0a, 0x03, 0x04, 0x03, 0x03]
    )]
    #[case::signature_algorithms(
        ExtensionSpec::SignatureAlgorithms(vec![0x0403, 0x0804]),
        RuntimeValues::default(),
        vec![0x00, 0x0d, 0x00, 0x06, 0x00, 0x04, 0x04, 0x03, 0x08, 0x04]
    )]
    #[case::alpn(
        ExtensionSpec::Alpn(vec!["h2".into(), "http/1.1".into()]),
        RuntimeValues::default(),
        vec![0x00, 0x10, 0x00, 0x0e, 0x00, 0x0c, 0x02, b'h', b'2', 0x08, b'h', b't', b't', b'p',
             b'/', b'1', b'.', b'1']
    )]
    #[case::ec_point_formats(
        ExtensionSpec::EcPointFormats,
        RuntimeValues::default(),
        vec![0x00, 0x0b, 0x00, 0x02, 0x01, 0x00]
    )]
    #[case::session_ticket(
        ExtensionSpec::SessionTicket,
        RuntimeValues::default(),
        vec![0x00, 0x23, 0x00, 0x00]
    )]
    #[case::psk_key_exchange_modes(
        ExtensionSpec::PskKeyExchangeModes,
        RuntimeValues::default(),
        vec![0x00, 0x2d, 0x00, 0x02, 0x01, 0x01]
    )]
    #[case::status_request(
        ExtensionSpec::StatusRequest,
        RuntimeValues::default(),
        vec![0x00, 0x05, 0x00, 0x05, 0x01, 0x00, 0x00, 0x00, 0x00]
    )]
    #[case::signed_certificate_timestamp(
        ExtensionSpec::SignedCertificateTimestamp,
        RuntimeValues::default(),
        vec![0x00, 0x12, 0x00, 0x00]
    )]
    #[case::renegotiation_info(
        ExtensionSpec::RenegotiationInfo,
        RuntimeValues::default(),
        vec![0xff, 0x01, 0x00, 0x01, 0x00]
    )]
    #[case::compress_certificate(
        ExtensionSpec::CompressCertificate(vec![0x0002, 0x0001]),
        RuntimeValues::default(),
        vec![0x00, 0x1b, 0x00, 0x05, 0x04, 0x00, 0x02, 0x00, 0x01]
    )]
    #[case::application_settings(
        ExtensionSpec::ApplicationSettings(vec!["h2".into()]),
        RuntimeValues::default(),
        vec![0x44, 0x69, 0x00, 0x06, 0x00, 0x04, 0x00, 0x02, b'h', b'2']
    )]
    #[case::record_size_limit(
        ExtensionSpec::RecordSizeLimit(0x00FF),
        RuntimeValues::default(),
        vec![0x00, 0x1c, 0x00, 0x02, 0x00, 0xff]
    )]
    #[case::padding(
        ExtensionSpec::Padding,
        RuntimeValues { padding_len: 4, ..RuntimeValues::default() },
        vec![0x00, 0x15, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00]
    )]
    #[case::grease(
        ExtensionSpec::Grease,
        RuntimeValues { grease_b: 0x1A1A, ..RuntimeValues::default() },
        vec![0x1a, 0x1a, 0x00, 0x01, 0x00]
    )]
    #[case::raw(
        ExtensionSpec::Raw { ty: 0x1234, data: vec![0xde, 0xad] },
        RuntimeValues::default(),
        vec![0x12, 0x34, 0x00, 0x02, 0xde, 0xad]
    )]
    fn extension_wire_encoding(
        #[case] ext: ExtensionSpec,
        #[case] rt: RuntimeValues,
        #[case] expected: Vec<u8>,
    ) {
        assert_eq!(ext.encode_body(&rt).unwrap(), expected);
    }

    #[test]
    fn x25519mlkem768_key_share_entry_encodes_hybrid_public_keys() {
        let rt = RuntimeValues {
            x25519_pub: [0xAB; 32],
            mlkem768_pub: vec![0xCD; 1184],
            ..RuntimeValues::default()
        };
        let ext = ExtensionSpec::KeyShare(vec![KeyShareGroup::X25519Mlkem768])
            .encode_body(&rt)
            .unwrap();
        // type 0033 + ext_len + list_len + entry(group 11ec, kx_len 04c0, 1216 bytes).
        assert_eq!(
            &ext[..10],
            &[0x00, 0x33, 0x04, 0xc6, 0x04, 0xc4, 0x11, 0xec, 0x04, 0xc0]
        );
        assert_eq!(ext.len(), 4 + 2 + 4 + 1216);
        assert_eq!(&ext[10..42], &[0xAB; 32]);
        assert_eq!(&ext[42..], &[0xCD; 1184]);
    }

    #[test]
    fn hybrid_key_share_requires_mlkem_material() {
        let rt = RuntimeValues::default();
        assert!(
            ExtensionSpec::KeyShare(vec![KeyShareGroup::X25519Mlkem768])
                .encode_body(&rt)
                .is_err()
        );
    }

    #[test]
    fn p256_p384_hybrid_key_shares_are_deferred() {
        let rt = RuntimeValues {
            mlkem768_pub: vec![0xCD; 1184],
            ..RuntimeValues::default()
        };
        for group in [
            KeyShareGroup::Secp256r1Mlkem768,
            KeyShareGroup::Secp384r1Mlkem1024,
        ] {
            assert!(
                ExtensionSpec::KeyShare(vec![group])
                    .encode_body(&rt)
                    .is_err()
            );
        }
    }
}
