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
            grease_a: 0x0A0A,
            grease_b: 0x1A1A,
            padding_len: 0,
        }
    }
}

/// How the `ClientHello` session id is produced.
#[derive(Debug, PartialEq, Eq)]
pub enum SessionIdSpec {
    /// 32 random bytes, the TLS 1.3 default.
    Random32,
    /// Placeholder for a REALITY authentication payload;
    /// `len` is the full wire length (plaintext + 16-byte tag).
    AuthPayload { len: usize },
}

/// A single TLS extension in the `ClientHello`.
#[derive(Debug, PartialEq, Eq)]
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
#[derive(Debug, PartialEq, Eq)]
pub enum KeyShareGroup {
    /// A GREASE group; the wire value comes from `RuntimeValues::grease_a`.
    Grease,
    X25519,
}

/// The semantic `ClientHello` description.
#[derive(Debug, PartialEq, Eq)]
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

impl ExtensionSpec {
    /// Encodes the COMPLETE extension: type (u16 BE) + length (u16 BE) + body.
    ///
    /// The length field counts the bytes after itself.
    pub fn encode_body(&self, rt: &RuntimeValues) -> Result<Vec<u8>, TlsError> {
        let (ty, body) = match self {
            Self::ServerName => {
                let host = rt.server_name.as_bytes();
                let host_len = u16::try_from(host.len())
                    .map_err(|_| TlsError::Spec("server_name host exceeds u16 length".to_string()))?;
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
                let byte_len = u16::try_from(groups.len() * 2)
                    .map_err(|_| TlsError::Spec("supported_groups exceeds u16 length".to_string()))?;
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
                    }
                }
                let list_len = u16::try_from(entries.len())
                    .map_err(|_| TlsError::Spec("key_share entries exceed u16 length".to_string()))?;
                let mut body = Vec::with_capacity(2 + entries.len());
                body.extend_from_slice(&list_len.to_be_bytes());
                body.extend_from_slice(&entries);
                (0x0033, body)
            }
            Self::SupportedVersions(versions) => {
                // RFC 8446: 1-byte length counts BYTES (n*2), not versions.
                let byte_len = u8::try_from(versions.len() * 2)
                    .map_err(|_| TlsError::Spec("supported_versions exceeds 255 bytes".to_string()))?;
                let mut body = Vec::with_capacity(1 + versions.len() * 2);
                body.push(byte_len);
                for version in versions {
                    body.extend_from_slice(&version.to_be_bytes());
                }
                (0x002b, body)
            }
            Self::SignatureAlgorithms(schemes) => {
                // RFC 8446 SignatureSchemeList: u16 byte-length + schemes, no count field.
                let byte_len = u16::try_from(schemes.len() * 2)
                    .map_err(|_| TlsError::Spec("signature_algorithms exceeds u16 length".to_string()))?;
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
                let byte_len = u8::try_from(algos.len() * 2)
                    .map_err(|_| TlsError::Spec("compress_certificate exceeds 255 bytes".to_string()))?;
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
        let len = u16::try_from(proto.len())
            .map_err(|_| TlsError::Spec("application_settings protocol exceeds u16 length".to_string()))?;
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(proto.as_bytes());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::grease::is_grease;
    use super::*;

    #[test]
    fn grease_detection() {
        assert!(is_grease(0x0A0A) && is_grease(0xCACA) && is_grease(0xFAFA));
        assert!(!is_grease(0x1301) && !is_grease(0x1516) && !is_grease(0x0000));
    }

    #[test]
    fn server_name_encodes_host() {
        let ext = ExtensionSpec::ServerName;
        let body = ext.encode_body(&RuntimeValues { server_name: "example.com".into(), ..RuntimeValues::default() }).unwrap();
        // "example.com" = 11 bytes.
        // type 00 00 | len 00 10 (2+1+2+11=16) | list_len 00 0e (1+2+11=14) | name_type 00 | host_len 00 0b | host
        assert_eq!(body, vec![
            0x00, 0x00, 0x00, 0x10, 0x00, 0x0e, 0x00, 0x00, 0x0b,
            b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'c', b'o', b'm',
        ]);
    }

    #[test]
    fn supported_groups_encodes_count_and_groups() {
        let ext = ExtensionSpec::SupportedGroups(vec![0x1301, 0x1302, 0x1303]);
        let body = ext.encode_body(&RuntimeValues::default()).unwrap();
        // RFC 8446 NamedGroupList: u16 byte-length + groups, NO count field.
        // type 00 0a | len 00 08 (2+6) | byte_len 00 06 | groups
        assert_eq!(body, vec![0x00, 0x0a, 0x00, 0x08, 0x00, 0x06, 0x13, 0x01, 0x13, 0x02, 0x13, 0x03]);
    }

    #[test]
    fn key_share_encodes_grease_and_x25519() {
        let ext = ExtensionSpec::KeyShare(vec![KeyShareGroup::Grease, KeyShareGroup::X25519]);
        let body = ext.encode_body(&RuntimeValues { grease_a: 0x1A1A, x25519_pub: [0xAB; 32], ..RuntimeValues::default() }).unwrap();
        // RFC 8446 KeyShare: u16 list-length + entries.
        // entries: grease 5 bytes (1a 1a 00 01 00), x25519 36 bytes → list 41 bytes.
        // type 00 33 | len 00 2b (2+41) | list_len 00 29 | grease | x25519
        let mut expected = vec![0x00, 0x33, 0x00, 0x2b, 0x00, 0x29, 0x1a, 0x1a, 0x00, 0x01, 0x00, 0x00, 0x1d, 0x00, 0x20];
        expected.extend_from_slice(&[0xAB; 32]);
        assert_eq!(body, expected);
    }

    #[test]
    fn supported_versions_encodes_count8_and_versions() {
        let ext = ExtensionSpec::SupportedVersions(vec![0x0A0A, 0x0304, 0x0303]);
        let body = ext.encode_body(&RuntimeValues::default()).unwrap();
        // RFC 8446: 1-byte length counts BYTES (n*2), not versions. 3 versions → len 6.
        // type 00 2b | len 00 07 | len8 06 | versions
        assert_eq!(body, vec![0x00, 0x2b, 0x00, 0x07, 0x06, 0x0a, 0x0a, 0x03, 0x04, 0x03, 0x03]);
    }

    #[test]
    fn signature_algorithms_encodes_count_and_schemes() {
        let ext = ExtensionSpec::SignatureAlgorithms(vec![0x0403, 0x0804]);
        let body = ext.encode_body(&RuntimeValues::default()).unwrap();
        // RFC 8446 SignatureSchemeList: u16 byte-length + schemes, NO count field.
        // type 00 0d | len 00 06 (2+4) | byte_len 00 04 | schemes
        assert_eq!(body, vec![0x00, 0x0d, 0x00, 0x06, 0x00, 0x04, 0x04, 0x03, 0x08, 0x04]);
    }

    #[test]
    fn alpn_encodes_list() {
        let ext = ExtensionSpec::Alpn(vec!["h2".into(), "http/1.1".into()]);
        let body = ext.encode_body(&RuntimeValues::default()).unwrap();
        // RFC 7301: per-entry 1-byte length; u16 list-length counts bytes.
        // list = 02 h2 08 http/1.1 (12 bytes).
        // type 00 10 | len 00 0e (2+12) | list_len 00 0c | entries
        assert_eq!(body, vec![
            0x00, 0x10, 0x00, 0x0e, 0x00, 0x0c, 0x02, b'h', b'2', 0x08,
            b'h', b't', b't', b'p', b'/', b'1', b'.', b'1',
        ]);
    }

    #[test]
    fn ec_point_formats_encodes_one_format() {
        let body = ExtensionSpec::EcPointFormats.encode_body(&RuntimeValues::default()).unwrap();
        // type 00 0b | len 00 02 | count 01 | format 00
        assert_eq!(body, vec![0x00, 0x0b, 0x00, 0x02, 0x01, 0x00]);
    }

    #[test]
    fn session_ticket_encodes_empty() {
        assert_eq!(ExtensionSpec::SessionTicket.encode_body(&RuntimeValues::default()).unwrap(), vec![0x00, 0x23, 0x00, 0x00]);
    }

    #[test]
    fn psk_key_exchange_modes_encodes_01_01() {
        let body = ExtensionSpec::PskKeyExchangeModes.encode_body(&RuntimeValues::default()).unwrap();
        // type 00 2d | len 00 02 | modes: count 01, mode 01
        assert_eq!(body, vec![0x00, 0x2d, 0x00, 0x02, 0x01, 0x01]);
    }

    #[test]
    fn status_request_encodes_01_00_00_00_00() {
        let body = ExtensionSpec::StatusRequest.encode_body(&RuntimeValues::default()).unwrap();
        // type 00 05 | len 00 05 | status_type 01 | responder_id_list len 00 00 | request_extensions len 00 00
        assert_eq!(body, vec![0x00, 0x05, 0x00, 0x05, 0x01, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn signed_certificate_timestamp_encodes_empty() {
        assert_eq!(ExtensionSpec::SignedCertificateTimestamp.encode_body(&RuntimeValues::default()).unwrap(), vec![0x00, 0x12, 0x00, 0x00]);
    }

    #[test]
    fn renegotiation_info_encodes_zero() {
        let body = ExtensionSpec::RenegotiationInfo.encode_body(&RuntimeValues::default()).unwrap();
        // type ff 01 | len 00 01 | renegotiated_connection 00
        assert_eq!(body, vec![0xff, 0x01, 0x00, 0x01, 0x00]);
    }

    #[test]
    fn compress_certificate_encodes_count_and_algos() {
        let ext = ExtensionSpec::CompressCertificate(vec![0x0002, 0x0001]);
        let body = ext.encode_body(&RuntimeValues::default()).unwrap();
        // RFC 8871: 1-byte length counts BYTES + algos, NO count field.
        // type 00 1b | len 00 05 | len8 04 | algos
        assert_eq!(body, vec![0x00, 0x1b, 0x00, 0x05, 0x04, 0x00, 0x02, 0x00, 0x01]);
    }

    #[test]
    fn application_settings_encodes_list() {
        let ext = ExtensionSpec::ApplicationSettings(vec!["h2".into()]);
        let body = ext.encode_body(&RuntimeValues::default()).unwrap();
        // ALPS (draft-ietf-tls-alps): 2-byte per-entry lengths, u16 list-length.
        // type 44 69 | len 00 06 | list_len 00 04 | entry 00 02 h2
        assert_eq!(body, vec![0x44, 0x69, 0x00, 0x06, 0x00, 0x04, 0x00, 0x02, b'h', b'2']);
    }

    #[test]
    fn record_size_limit_encodes_limit() {
        let body = ExtensionSpec::RecordSizeLimit(0x00FF).encode_body(&RuntimeValues::default()).unwrap();
        // type 00 1c | len 00 02 | limit 00 ff
        assert_eq!(body, vec![0x00, 0x1c, 0x00, 0x02, 0x00, 0xff]);
    }

    #[test]
    fn padding_encodes_zeroes() {
        let body = ExtensionSpec::Padding.encode_body(&RuntimeValues { padding_len: 4, ..RuntimeValues::default() }).unwrap();
        // type 00 15 | len 00 04 | 00 00 00 00
        assert_eq!(body, vec![0x00, 0x15, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn grease_encodes_grease_b_00_01_00() {
        let body = ExtensionSpec::Grease.encode_body(&RuntimeValues { grease_b: 0x1A1A, ..RuntimeValues::default() }).unwrap();
        // type = grease_b (1a 1a) | len 00 01 | data 00
        assert_eq!(body, vec![0x1a, 0x1a, 0x00, 0x01, 0x00]);
    }

    #[test]
    fn raw_encodes_type_length_data() {
        let ext = ExtensionSpec::Raw { ty: 0x1234, data: vec![0xde, 0xad] };
        let body = ext.encode_body(&RuntimeValues::default()).unwrap();
        assert_eq!(body, vec![0x12, 0x34, 0x00, 0x02, 0xde, 0xad]);
    }
}
