//! Shadowsocks: method table + key derivation.
//!
//! Native covers the AEAD set (2017 edition) and the 2022-blake3 family; the
//! legacy stream ciphers (`aes-*-cfb/ctr`, `rc4-md5`, `chacha20-ietf`,
//! `xchacha20`, `none`) stay on sing-box and are deliberately absent here.
//! Crypto: `crypto/aead`, `crypto/kdf`.
//! Reference: v2ray-core `proxy/shadowsocks`, thirdparty/shadowsocks-rust.

pub mod method;
pub mod stream;
pub mod stream2022;
