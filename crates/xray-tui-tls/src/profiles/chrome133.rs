//! Chrome 133 fingerprint profile.
//!
//! Transcribed from the uTLS `HelloChrome_133` preset
//! (`thirdparty/utls/u_parrots.go`, a tlsfingerprint.io capture):
//! cipher suites, extension order, and GREASE slots. Chrome 131+ moved
//! `signature_algorithms` ahead of the SCT extension, adopted the
//! post-quantum X25519MLKEM768 hybrid group, switched ALPS to the
//! "new" codepoint 0x446D, and dropped the 512-byte padding extension.

use crate::spec::grease::GREASE_PLACEHOLDER;
use crate::spec::{ClientHelloSpec, ExtensionSpec, KeyShareGroup, SessionIdSpec};

/// Cipher suites used by Chrome 133 (in order).
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

/// Signature algorithms used by Chrome 133.
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

/// The Chrome 133 `ClientHello` spec.
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
            // Supported curves: GREASE, X25519MLKEM768, X25519, P256, P384.
            ExtensionSpec::SupportedGroups(vec![
                GREASE_PLACEHOLDER,
                0x11EC, // x25519mlkem768 (post-quantum hybrid)
                0x001D, // x25519
                0x0017, // secp256r1
                0x0018, // secp384r1
            ]),
            ExtensionSpec::EcPointFormats,
            ExtensionSpec::SessionTicket,
            ExtensionSpec::Alpn(vec!["h2".into(), "http/1.1".into()]),
            ExtensionSpec::StatusRequest,
            // Chrome 131+ moved signature_algorithms before SCT.
            ExtensionSpec::SignatureAlgorithms(SIG_ALGOS.to_vec()),
            ExtensionSpec::SignedCertificateTimestamp,
            // uTLS key share: GREASE + X25519MLKEM768 + X25519. The hybrid
            // entry is omitted here (no MLKEM material in this crate); the
            // GREASE + X25519 slots remain. Not fingerprinted by JA3/JA4.
            ExtensionSpec::KeyShare(vec![KeyShareGroup::Grease, KeyShareGroup::X25519]),
            ExtensionSpec::PskKeyExchangeModes,
            ExtensionSpec::SupportedVersions(vec![GREASE_PLACEHOLDER, 0x0304, 0x0303]),
            // compress_certificate: brotli only (Chrome 133).
            ExtensionSpec::CompressCertificate(vec![0x0003]),
            // ALPS "new" (0x446D): ALPN-style u8 per-entry lengths, unlike
            // the 0x4469 draft form. Static body: list_len 0003, "h2".
            ExtensionSpec::Raw {
                ty: 0x446D,
                data: vec![0x00, 0x03, 0x02, 0x68, 0x32],
            },
            // Note: uTLS's Chrome 133 also carries a GREASE ECH (0xFE0D)
            // outer extension. This crate cannot emit a *valid* ECH outer
            // (no HPKE key agreement), and rustls 0.23 rejects even a
            // well-shaped empty ECH outer with alert decode_error — which
            // breaks the REALITY dest handshake (xtls/reality borrows the
            // dest's ServerHello flight). The extension type is not part of
            // JA3, so it is omitted here.
            // Second standalone GREASE extension (Chrome 133 sends two; the
            // builder renders each with a distinct GREASE value).
            ExtensionSpec::Grease,
        ],
    }
}
