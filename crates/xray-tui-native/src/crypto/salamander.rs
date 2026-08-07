//! Hysteria2 Salamander packet obfuscation.
//!
//! AES-keystream XOR over QUIC packets — a DATAGRAM transform, not a byte
//! stream. Consumed by the future `transport/quic` module (hysteria2 /
//! hysteria1 / tuic), never by the stream seam.
//! Reference: thirdparty/hysteria (Go Salamander impl).
//! Backend: ring (AES-128-ECB keystream). Implemented at milestone M2.
