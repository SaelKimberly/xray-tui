//! Android 11 `OkHttp` fingerprint profile.
//!
//! Transcribed VERBATIM from the uTLS `HelloAndroid_11_OkHttp` preset
//! (`thirdparty/utls/u_parrots.go`). `OkHttp`'s hello is Chromium-derived
//! but distinctly minimal:
//!
//! - **No GREASE anywhere** — no grease cipher slot, no grease extension
//!   entries, no standalone GREASE extensions.
//! - **TLS 1.2-era shape** — no `supported_versions`, no `key_share`,
//!   no `psk_key_exchange_modes`; only TLS ≤ 1.2 cipher suites.
//! - **No ALPN**, no `session_ticket`, no padding.
//!
//! Consequence: this hello cannot negotiate TLS 1.3 and therefore cannot
//! complete a handshake with this crate's TLS 1.3-only engine or with the
//! HTTP/2 peet.ws grader path; it is exercised offline (build → parse →
//! fingerprints) only. See the task report for details.

use crate::spec::{ClientHelloSpec, ExtensionSpec, SessionIdSpec};

/// Android 11 `OkHttp` cipher suites — all TLS ≤ 1.2, preset order.
const CIPHER_SUITES: &[u16] = &[
    0xC02B, // ECDHE-ECDSA-AES128-GCM-SHA256
    0xC02C, // ECDHE-ECDSA-AES256-GCM-SHA384
    0xCCA9, // ECDHE-ECDSA-CHACHA20-POLY1305
    0xC02F, // ECDHE-RSA-AES128-GCM-SHA256
    0xC030, // ECDHE-RSA-AES256-GCM-SHA384
    0xCCA8, // ECDHE-RSA-CHACHA20-POLY1305
    0xC013, // ECDHE-RSA-AES128-SHA
    0xC014, // ECDHE-RSA-AES256-SHA
    0x009C, // RSA-AES128-GCM-SHA256
    0x009D, // RSA-AES256-GCM-SHA384
    0x002F, // RSA-AES128-SHA
    0x0035, // RSA-AES256-SHA
];

/// Android 11 `OkHttp` signature algorithms.
const SIG_ALGOS: &[u16] = &[
    0x0403, // ecdsa_secp256r1_sha256
    0x0804, // rsa_pss_rsae_sha256
    0x0401, // rsa_pkcs1_sha256
    0x0503, // ecdsa_secp384r1_sha384
    0x0805, // rsa_pss_rsae_sha384
    0x0501, // rsa_pkcs1_sha384
    0x0806, // rsa_pss_rsae_sha512
    0x0601, // rsa_pkcs1_sha512
    0x0201, // rsa_pkcs1_sha1
];

/// Android 11 `OkHttp` named groups.
const NAMED_GROUPS: &[u16] = &[
    0x001D, // x25519
    0x0017, // secp256r1
    0x0018, // secp384r1
];

/// The Android 11 `OkHttp` `ClientHello` spec — no ALPN, no key exchange
/// material, no version negotiation: exactly what the preset offers.
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
            ExtensionSpec::Raw {
                ty: 0x0017,
                data: Vec::new(),
            },
            ExtensionSpec::RenegotiationInfo,
            ExtensionSpec::SupportedGroups(NAMED_GROUPS.to_vec()),
            ExtensionSpec::EcPointFormats,
            ExtensionSpec::StatusRequest,
            ExtensionSpec::SignatureAlgorithms(SIG_ALGOS.to_vec()),
            // No session_ticket, no ALPN, no supported_versions/key_share
            // — the preset ends here.
        ],
    }
}
