//! Chrome 130 fingerprint profile.
//!
//! Cipher suites, extension ordering, and GREASE placement based on
//! captured Chrome 130 (Chromium) `ClientHello` handshakes. Ported VERBATIM
//! from `thirdparty/tls-fingerprint/src/profiles/chrome.rs`.

use crate::spec::grease::GREASE_PLACEHOLDER;
use crate::spec::{ClientHelloSpec, ExtensionSpec, KeyShareGroup, SessionIdSpec};

/// Cipher suites used by Chrome 130 (in order).
const CIPHER_SUITES: &[u16] = &[
    0xCACA, // GREASE (placeholder — replaced at build time)
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

/// Signature algorithms used by Chrome 130.
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

/// Certificate compression algorithms (brotli + zlib).
const COMPRESS_CERT: &[u16] = &[
    0x0002, // zlib
    0x0003, // brotli
];

/// The Chrome 130 `ClientHello` spec.
#[must_use]
pub fn spec() -> ClientHelloSpec {
    ClientHelloSpec {
        legacy_version: 0x0303,
        cipher_suites: CIPHER_SUITES.to_vec(),
        compression_methods: vec![0x00],
        session_id: SessionIdSpec::Random32,
        extensions: vec![
            // GREASE extension (first).
            ExtensionSpec::Grease,
            // SNI.
            ExtensionSpec::ServerName,
            // extended_master_secret (no dedicated variant).
            ExtensionSpec::Raw { ty: 0x0017, data: Vec::new() },
            // renegotiation_info.
            ExtensionSpec::RenegotiationInfo,
            // supported_groups: GREASE first (the reference prepends a
            // freshly drawn GREASE group to NAMED_GROUPS_BASE).
            ExtensionSpec::SupportedGroups(vec![
                GREASE_PLACEHOLDER,
                0x001D, // x25519
                0x0017, // secp256r1
                0x0018, // secp384r1
            ]),
            // ec_point_formats.
            ExtensionSpec::EcPointFormats,
            // session_ticket.
            ExtensionSpec::SessionTicket,
            // ALPN.
            ExtensionSpec::Alpn(vec!["h2".into(), "http/1.1".into()]),
            // status_request (OCSP).
            ExtensionSpec::StatusRequest,
            // signed_certificate_timestamp.
            ExtensionSpec::SignedCertificateTimestamp,
            // key_share (X25519 with GREASE entry).
            ExtensionSpec::KeyShare(vec![KeyShareGroup::Grease, KeyShareGroup::X25519]),
            // psk_key_exchange_modes.
            ExtensionSpec::PskKeyExchangeModes,
            // supported_versions (with GREASE).
            ExtensionSpec::SupportedVersions(vec![GREASE_PLACEHOLDER, 0x0304, 0x0303]),
            // compress_certificate.
            ExtensionSpec::CompressCertificate(COMPRESS_CERT.to_vec()),
            // application_settings / ALPS (Chrome style).
            ExtensionSpec::ApplicationSettings(vec!["h2".into()]),
            // signature_algorithms (Chrome puts this near the end).
            ExtensionSpec::SignatureAlgorithms(SIG_ALGOS.to_vec()),
            // Padding: the builder pads the record to exactly 512 bytes
            // (the reference computes `512 - unpadded_record - 4`).
            ExtensionSpec::Padding,
        ],
    }
}
