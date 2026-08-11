//! Safari 17 (macOS Sonoma) fingerprint profile.
//!
//! Safari uses a unique cipher suite list with RSA and older suites, no
//! GREASE, an EMPTY legacy session id, and a specific extension set
//! distinct from Chrome/Firefox. Ported VERBATIM from
//! `thirdparty/tls-fingerprint/src/profiles/safari.rs`.

use crate::spec::{ClientHelloSpec, ExtensionSpec, KeyShareGroup, SessionIdSpec};

/// Safari 17 cipher suites — note CHACHA20 placement and older TLS 1.2
/// suites (including legacy 3DES).
const CIPHER_SUITES: &[u16] = &[
    0x1301, // TLS_AES_128_GCM_SHA256
    0x1302, // TLS_AES_256_GCM_SHA384
    0x1303, // TLS_CHACHA20_POLY1305_SHA256
    0xC02C, // ECDHE-ECDSA-AES256-GCM-SHA384
    0xC02B, // ECDHE-ECDSA-AES128-GCM-SHA256
    0xC030, // ECDHE-RSA-AES256-GCM-SHA384
    0xC02F, // ECDHE-RSA-AES128-GCM-SHA256
    0xCCA9, // ECDHE-ECDSA-CHACHA20-POLY1305
    0xCCA8, // ECDHE-RSA-CHACHA20-POLY1305
    0xC024, // ECDHE-ECDSA-AES256-SHA384
    0xC023, // ECDHE-ECDSA-AES128-SHA256
    0xC028, // ECDHE-RSA-AES256-SHA384
    0xC027, // ECDHE-RSA-AES128-SHA256
    0xC00A, // ECDHE-ECDSA-AES256-SHA
    0xC009, // ECDHE-ECDSA-AES128-SHA
    0xC014, // ECDHE-RSA-AES256-SHA
    0xC013, // ECDHE-RSA-AES128-SHA
    0x009D, // RSA-AES256-GCM-SHA384
    0x009C, // RSA-AES128-GCM-SHA256
    0x003D, // RSA-AES256-SHA256
    0x003C, // RSA-AES128-SHA256
    0x0035, // RSA-AES256-SHA
    0x002F, // RSA-AES128-SHA
    0x000A, // RSA-3DES-EDE-SHA (legacy)
];

/// Safari 17 signature algorithms.
const SIG_ALGOS: &[u16] = &[
    0x0403, // ecdsa_secp256r1_sha256
    0x0503, // ecdsa_secp384r1_sha384
    0x0603, // ecdsa_secp521r1_sha512
    0x0804, // rsa_pss_rsae_sha256
    0x0805, // rsa_pss_rsae_sha384
    0x0806, // rsa_pss_rsae_sha512
    0x0401, // rsa_pkcs1_sha256
    0x0501, // rsa_pkcs1_sha384
    0x0601, // rsa_pkcs1_sha512
    0x0201, // rsa_pkcs1_sha1
    0x0203, // ecdsa_sha1
];

/// Named groups (`supported_groups`).
const NAMED_GROUPS: &[u16] = &[
    0x001D, // x25519
    0x0017, // secp256r1
    0x0018, // secp384r1
    0x0019, // secp521r1
];

/// The Safari 17 `ClientHello` spec — no GREASE, EMPTY session id, no
/// `session_ticket`, no `compress_certificate`.
#[must_use]
pub fn spec() -> ClientHelloSpec {
    ClientHelloSpec {
        legacy_version: 0x0303,
        cipher_suites: CIPHER_SUITES.to_vec(),
        compression_methods: vec![0x00],
        // Safari sends an empty session ID (unlike Chrome/Firefox).
        session_id: SessionIdSpec::Empty,
        extensions: vec![
            ExtensionSpec::ServerName,
            // extended_master_secret.
            ExtensionSpec::Raw {
                ty: 0x0017,
                data: Vec::new(),
            },
            ExtensionSpec::RenegotiationInfo,
            ExtensionSpec::SupportedGroups(NAMED_GROUPS.to_vec()),
            ExtensionSpec::EcPointFormats,
            ExtensionSpec::Alpn(vec!["h2".into(), "http/1.1".into()]),
            ExtensionSpec::StatusRequest,
            ExtensionSpec::SignedCertificateTimestamp,
            ExtensionSpec::KeyShare(vec![KeyShareGroup::X25519]),
            ExtensionSpec::SupportedVersions(vec![0x0304, 0x0303]),
            ExtensionSpec::PskKeyExchangeModes,
            ExtensionSpec::SignatureAlgorithms(SIG_ALGOS.to_vec()),
            // Safari does not send session_ticket or compress_certificate.
        ],
    }
}
