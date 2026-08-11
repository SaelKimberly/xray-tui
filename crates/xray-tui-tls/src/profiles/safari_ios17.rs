//! Safari on iOS 17 fingerprint profile.
//!
//! iOS Safari 17 differs from macOS Safari 17 in several ways:
//! - No `ec_point_formats` extension
//! - No `session_ticket` extension
//! - No `signed_certificate_timestamp` extension
//! - No 3DES cipher suite (0x000A)
//! - Shorter named groups list (no secp521r1)
//! - Empty legacy session ID (like macOS Safari)
//!
//! Ported VERBATIM from `thirdparty/tls-fingerprint/src/profiles/safari_ios17.rs`.

use crate::spec::{ClientHelloSpec, ExtensionSpec, KeyShareGroup, SessionIdSpec};

/// iOS Safari 17 cipher suites — no 3DES (0x000A) unlike macOS Safari.
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
];

/// iOS Safari 17 signature algorithms.
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

/// iOS Safari named groups — no secp521r1 unlike macOS Safari.
const NAMED_GROUPS: &[u16] = &[
    0x001D, // x25519
    0x0017, // secp256r1
    0x0018, // secp384r1
];

/// The iOS Safari 17 `ClientHello` spec — no `ec_point_formats`,
/// `session_ticket`, or `signed_certificate_timestamp`; EMPTY session id.
#[must_use]
pub fn spec() -> ClientHelloSpec {
    ClientHelloSpec {
        legacy_version: 0x0303,
        cipher_suites: CIPHER_SUITES.to_vec(),
        compression_methods: vec![0x00],
        // iOS Safari: empty session ID.
        session_id: SessionIdSpec::Empty,
        extensions: vec![
            ExtensionSpec::ServerName,
            // extended_master_secret.
            ExtensionSpec::Raw { ty: 0x0017, data: Vec::new() },
            ExtensionSpec::RenegotiationInfo,
            ExtensionSpec::SupportedGroups(NAMED_GROUPS.to_vec()),
            // No ec_point_formats (iOS Safari dropped this).
            // No session_ticket.
            ExtensionSpec::Alpn(vec!["h2".into(), "http/1.1".into()]),
            ExtensionSpec::StatusRequest,
            // No signed_certificate_timestamp.
            ExtensionSpec::KeyShare(vec![KeyShareGroup::X25519]),
            ExtensionSpec::SupportedVersions(vec![0x0304, 0x0303]),
            ExtensionSpec::PskKeyExchangeModes,
            ExtensionSpec::SignatureAlgorithms(SIG_ALGOS.to_vec()),
        ],
    }
}
