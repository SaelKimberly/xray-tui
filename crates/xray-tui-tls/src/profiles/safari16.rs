//! Safari 16 fingerprint profile.
//!
//! Transcribed from the uTLS `HelloSafari_16_0` preset
//! (`thirdparty/utls/u_parrots.go`). Unlike the tls-fingerprint Safari 17
//! model (`profiles/safari.rs`), the uTLS Safari 16 preset carries GREASE:
//! a GREASE cipher slot, GREASE entries in `supported_groups`,
//! `supported_versions` and `key_share`, plus standalone GREASE extensions
//! — all preserved verbatim.

use crate::spec::grease::GREASE_PLACEHOLDER;
use crate::spec::{ClientHelloSpec, ExtensionSpec, KeyShareGroup, SessionIdSpec};

/// Cipher suites used by Safari 16 (in order), including legacy 3DES.
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
    0xC00A, // ECDHE-ECDSA-AES256-SHA
    0xC009, // ECDHE-ECDSA-AES128-SHA
    0xC014, // ECDHE-RSA-AES256-SHA
    0xC013, // ECDHE-RSA-AES128-SHA
    0x009D, // RSA-AES256-GCM-SHA384
    0x009C, // RSA-AES128-GCM-SHA256
    0x0035, // RSA-AES256-SHA
    0x002F, // RSA-AES128-SHA
    0xC008, // ECDHE-ECDSA-3DES-EDE-SHA (uTLS FAKE_ placeholder for the real IANA id)
    0xC012, // ECDHE-RSA-3DES-EDE-SHA
    0x000A, // RSA-3DES-EDE-SHA (legacy)
];

/// Safari 16 signature algorithms — the uTLS preset repeats PSS-SHA384;
/// kept verbatim.
const SIG_ALGOS: &[u16] = &[
    0x0403, // ecdsa_secp256r1_sha256
    0x0804, // rsa_pss_rsae_sha256
    0x0401, // rsa_pkcs1_sha256
    0x0503, // ecdsa_secp384r1_sha384
    0x0203, // ecdsa_sha1
    0x0805, // rsa_pss_rsae_sha384
    0x0805, // rsa_pss_rsae_sha384 (duplicated in the preset)
    0x0501, // rsa_pkcs1_sha384
    0x0806, // rsa_pss_rsae_sha512
    0x0601, // rsa_pkcs1_sha512
    0x0201, // rsa_pkcs1_sha1
];

/// The Safari 16 `ClientHello` spec — GREASE-carrying, empty session id,
/// zlib-only `compress_certificate`, no `session_ticket`.
#[must_use]
pub fn spec() -> ClientHelloSpec {
    ClientHelloSpec {
        legacy_version: 0x0303,
        cipher_suites: CIPHER_SUITES.to_vec(),
        compression_methods: vec![0x00],
        session_id: SessionIdSpec::Empty,
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
                0x0304,
                0x0303,
                0x0302,
                0x0301,
            ]),
            ExtensionSpec::CompressCertificate(vec![0x0002]), // zlib only
            ExtensionSpec::Grease,
            ExtensionSpec::Padding,
        ],
    }
}
