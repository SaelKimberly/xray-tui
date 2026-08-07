//! HKDF / SHA helpers shared by protocol crypto.
//!
//! Consumers: `protocol/vmess` (VMess KDF64/KDF16), `security/reality` (auth
//! key derivation: HKDF-SHA256 with salt = ClientHello.Random[0..20], info
//! `b"REALITY"`), future HPKE (Reality target crypto).
//! Reference: `thirdparty/v2ray-core/proxy/vmess/encoding/aead/kdf.go`,
//! `thirdparty/Xray-core/transport/internet/reality` (spec only).
//!
//! Backend: ring (workspace standard). Implemented at milestone M2/M3.
