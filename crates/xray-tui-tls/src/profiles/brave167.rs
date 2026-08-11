//! Brave 1.67 fingerprint profile.
//!
//! Brave is Chromium-based and uses GREASE, but differs from Chrome 130 by
//! omitting ALPS (0x4469) and `compress_certificate` (0x001B). The result is
//! a distinct JA3 hash despite sharing the same cipher suite list as
//! Chrome. Ported VERBATIM from
//! `thirdparty/tls-fingerprint/src/profiles/brave167.rs`.

use crate::spec::grease::GREASE_PLACEHOLDER;
use crate::spec::{ClientHelloSpec, ExtensionSpec, KeyShareGroup, SessionIdSpec};

/// Cipher suites used by Brave 1.67 (in order).
const CIPHER_SUITES: &[u16] = &[
    0xCACA, // GREASE placeholder
    0x1301, // TLS_AES_128_GCM_SHA256
    0x1302, // TLS_AES_256_GCM_SHA384
    0x1303, // TLS_CHACHA20_POLY1305_SHA256
    0xC02B, // ECDHE-ECDSA-AES128-GCM-SHA256
    0xC02F, // ECDHE-RSA-AES128-GCM-SHA256
    0xC02C, // ECDHE-ECDSA-AES256-GCM-SHA384
    0xC030, // ECDHE-RSA-AES256-GCM-SHA384
    0xCCA9, // ECDHE-ECDSA-CHACHA20-POLY1305
    0xCCA8, // ECDHE-RSA-CHACHA20-POLY1305
    0xC013, // ECDHE-RSA-AES128-SHA
    0xC014, // ECDHE-RSA-AES256-SHA
    0x009C, // RSA-AES128-GCM-SHA256
    0x009D, // RSA-AES256-GCM-SHA384
    0x002F, // RSA-AES128-SHA
    0x0035, // RSA-AES256-SHA
];

/// Signature algorithms used by Brave 1.67.
const SIG_ALGOS: &[u16] = &[
    0x0403, // ecdsa_secp256r1_sha256
    0x0804, // rsa_pss_rsae_sha256
    0x0401, // rsa_pkcs1_sha256
    0x0503, // ecdsa_secp384r1_sha384
    0x0805, // rsa_pss_rsae_sha384
    0x0501, // rsa_pkcs1_sha384
    0x0806, // rsa_pss_rsae_sha512
    0x0601, // rsa_pkcs1_sha512
];

/// The Brave 1.67 `ClientHello` spec — no ALPS, no `compress_certificate`.
#[must_use]
pub fn spec() -> ClientHelloSpec {
    ClientHelloSpec {
        legacy_version: 0x0303,
        cipher_suites: CIPHER_SUITES.to_vec(),
        compression_methods: vec![0x00],
        session_id: SessionIdSpec::Random32,
        extensions: vec![
            ExtensionSpec::Grease,
            ExtensionSpec::ServerName,
            // extended_master_secret.
            ExtensionSpec::Raw { ty: 0x0017, data: Vec::new() },
            ExtensionSpec::RenegotiationInfo,
            ExtensionSpec::SupportedGroups(vec![
                GREASE_PLACEHOLDER,
                0x001D, // x25519
                0x0017, // secp256r1
                0x0018, // secp384r1
            ]),
            ExtensionSpec::EcPointFormats,
            ExtensionSpec::SessionTicket,
            ExtensionSpec::Alpn(vec!["h2".into(), "http/1.1".into()]),
            ExtensionSpec::StatusRequest,
            ExtensionSpec::SignedCertificateTimestamp,
            ExtensionSpec::KeyShare(vec![KeyShareGroup::Grease, KeyShareGroup::X25519]),
            ExtensionSpec::PskKeyExchangeModes,
            ExtensionSpec::SupportedVersions(vec![GREASE_PLACEHOLDER, 0x0304, 0x0303]),
            // No compress_certificate (0x001B), no ALPS (0x4469).
            ExtensionSpec::SignatureAlgorithms(SIG_ALGOS.to_vec()),
            ExtensionSpec::Padding,
        ],
    }
}
