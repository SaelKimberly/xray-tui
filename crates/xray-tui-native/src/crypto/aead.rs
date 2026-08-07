//! AEAD body ciphers: AES-128/256-GCM and ChaCha20-Poly1305 chunk codecs.
//!
//! Consumers: `protocol/vmess` (AEAD body), `protocol/shadowsocks`
//! (AEAD-2022 + classic AEAD methods).
//! References: RFC 8439 (ChaCha20-Poly1305), NIST SP 800-38D (GCM);
//! v2ray-core `proxy/vmess/encoding` KDF-derived chunk keys (MIT).
//!
//! Backend: ring (workspace standard). Implemented at milestone M2.
