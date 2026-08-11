//! Firefox 121 fingerprint profile.
//!
//! Firefox does NOT use GREASE. It has a distinct cipher suite list and
//! extension ordering compared to Chrome. Ported VERBATIM from
//! `thirdparty/tls-fingerprint/src/profiles/firefox.rs`.

use crate::spec::{ClientHelloSpec, ExtensionSpec, KeyShareGroup, SessionIdSpec};

/// Cipher suites used by Firefox 121 (in order).
const CIPHER_SUITES: &[u16] = &[
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
    0x002F, // RSA-AES128-SHA
    0x0035, // RSA-AES256-SHA
];

/// Firefox 121 signature algorithms (distinct ordering from Chrome).
const SIG_ALGOS: &[u16] = &[
    0x0403, // ecdsa_secp256r1_sha256
    0x0503, // ecdsa_secp384r1_sha384
    0x0603, // ecdsa_secp521r1_sha512
    0x0807, // ed25519
    0x0808, // ed448
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
    0x0100, // ffdhe2048
    0x0101, // ffdhe3072
];

/// The Firefox 121 `ClientHello` spec — no GREASE, X25519-only key share,
/// zlib-only `compress_certificate`.
#[must_use]
pub fn spec() -> ClientHelloSpec {
    ClientHelloSpec {
        legacy_version: 0x0303,
        cipher_suites: CIPHER_SUITES.to_vec(),
        compression_methods: vec![0x00],
        session_id: SessionIdSpec::Random32,
        extensions: vec![
            ExtensionSpec::ServerName,
            // extended_master_secret.
            ExtensionSpec::Raw { ty: 0x0017, data: Vec::new() },
            ExtensionSpec::RenegotiationInfo,
            ExtensionSpec::SupportedGroups(NAMED_GROUPS.to_vec()),
            ExtensionSpec::EcPointFormats,
            ExtensionSpec::SessionTicket,
            ExtensionSpec::Alpn(vec!["h2".into(), "http/1.1".into()]),
            ExtensionSpec::StatusRequest,
            ExtensionSpec::KeyShare(vec![KeyShareGroup::X25519]),
            ExtensionSpec::SupportedVersions(vec![0x0304, 0x0303]),
            ExtensionSpec::SignatureAlgorithms(SIG_ALGOS.to_vec()),
            ExtensionSpec::PskKeyExchangeModes,
            // Firefox includes compress_certificate with zlib only.
            ExtensionSpec::CompressCertificate(vec![0x0002]),
            ExtensionSpec::Padding,
        ],
    }
}
