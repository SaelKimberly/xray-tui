//! iOS 14 fingerprint profile.
//!
//! Transcribed VERBATIM from the uTLS `HelloIOS_14` preset
//! (`thirdparty/utls/u_parrots.go`). The preset carries GREASE everywhere
//! (leading cipher slot, `supported_groups`/`supported_versions`/
//! `key_share` entries, standalone GREASE extensions first and
//! second-to-last), a duplicated PSS-SHA384 signature algorithm, legacy
//! 3DES suites, and a trailing Boring-style padding extension.

use crate::spec::grease::GREASE_PLACEHOLDER;
use crate::spec::{ClientHelloSpec, ExtensionSpec, KeyShareGroup, SessionIdSpec};

/// iOS 14 cipher suites — GREASE slot + TLS 1.3 trio + full legacy list,
/// including the 3DES trio (`0xC008`, `0xC012`, `0x000A`). The uTLS
/// `DISABLED_*` names are the ordinary IANA values (disabled only in Go's
/// default config, not on the wire).
const CIPHER_SUITES: &[u16] = &[
    GREASE_PLACEHOLDER,
    0x1301, // TLS_AES_128_GCM_SHA256
    0x1302, // TLS_AES_256_GCM_SHA384
    0x1303, // TLS_CHACHA20_POLY1305_SHA256
    0xC02C, // ECDHE-ECDSA-AES256-GCM-SHA384
    0xC02B, // ECDHE-ECDSA-AES128-GCM-SHA256
    0xCCA9, // ECDHE-ECDSA-CHACHA20-POLY1305
    0xC030, // ECDHE-RSA-AES256-GCM-SHA384
    0xC02F, // ECDHE-RSA-AES128-GCM-SHA256
    0xCCA8, // ECDHE-RSA-CHACHA20-POLY1305
    0xC024, // ECDHE-ECDSA-AES256-SHA384
    0xC023, // ECDHE-ECDSA-AES128-SHA256
    0xC00A, // ECDHE-ECDSA-AES256-SHA
    0xC009, // ECDHE-ECDSA-AES128-SHA
    0xC028, // ECDHE-RSA-AES256-SHA384
    0xC027, // ECDHE-RSA-AES128-SHA256
    0xC014, // ECDHE-RSA-AES256-SHA
    0xC013, // ECDHE-RSA-AES128-SHA
    0x009D, // RSA-AES256-GCM-SHA384
    0x009C, // RSA-AES128-GCM-SHA256
    0x003D, // RSA-AES256-SHA256
    0x003C, // RSA-AES128-SHA256
    0x0035, // RSA-AES256-SHA
    0x002F, // RSA-AES128-SHA
    0xC008, // ECDHE-ECDSA-3DES-EDE-CBC
    0xC012, // ECDHE-RSA-3DES-EDE-CBC
    0x000A, // RSA-3DES-EDE-CBC
];

/// iOS 14 signature algorithms — preset-verbatim, including the
/// duplicated PSS-SHA384 (`0x0805`) entry and `ecdsa_sha1` (`0x0203`).
const SIG_ALGOS: &[u16] = &[
    0x0403, // ecdsa_secp256r1_sha256
    0x0804, // rsa_pss_rsae_sha256
    0x0401, // rsa_pkcs1_sha256
    0x0503, // ecdsa_secp384r1_sha384
    0x0203, // ecdsa_sha1
    0x0805, // rsa_pss_rsae_sha384 (duplicated in the preset)
    0x0805, // rsa_pss_rsae_sha384
    0x0501, // rsa_pkcs1_sha384
    0x0806, // rsa_pss_rsae_sha512
    0x0601, // rsa_pkcs1_sha512
    0x0201, // rsa_pkcs1_sha1
];

/// The iOS 14 `ClientHello` spec.
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
            ExtensionSpec::Raw {
                ty: 0x0017,
                data: Vec::new(),
            },
            ExtensionSpec::RenegotiationInfo,
            ExtensionSpec::SupportedGroups(vec![
                GREASE_PLACEHOLDER,
                0x001D, // x25519
                0x0017, // secp256r1
                0x0018, // secp384r1
                0x0019, // secp521r1
            ]),
            ExtensionSpec::EcPointFormats,
            ExtensionSpec::Alpn(vec!["h2".into(), "http/1.1".into()]),
            ExtensionSpec::StatusRequest,
            ExtensionSpec::SignatureAlgorithms(SIG_ALGOS.to_vec()),
            ExtensionSpec::SignedCertificateTimestamp,
            ExtensionSpec::KeyShare(vec![KeyShareGroup::Grease, KeyShareGroup::X25519]),
            ExtensionSpec::PskKeyExchangeModes,
            ExtensionSpec::SupportedVersions(vec![
                GREASE_PLACEHOLDER,
                0x0304, // TLS 1.3
                0x0303, // TLS 1.2
                0x0302, // TLS 1.1
                0x0301, // TLS 1.0
            ]),
            ExtensionSpec::Grease,
            // BoringPaddingStyle in uTLS; the builder computes the length.
            ExtensionSpec::Padding,
        ],
    }
}
