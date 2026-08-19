# VLESS ML-KEM (Post-Quantum KEM) — Design Spec (SP7 of VLESS Parity)

Date: 2026-08-19. Branch: `native-core-stub`.
SP7 of the VLESS parity program. SP1-SP6 done (UDP, mux, XUDP, mKCP, xhttp/3, stream-one). SP7 = **ML-KEM (Kyber) post-quantum key encapsulation** integration for TLS 1.3 hybrid curves + VLESS encryption + REALITY.

## 1. Goal

Add ML-KEM-768 (Kyber768) support to `xray-tui-native` via `liboqs`:
- **TLS 1.3 hybrid curves**: `X25519MLKEM768`, `SecP256r1MLKEM768`, `SecP384r1MLKEM1024` — the ClientHello `supported_groups` extension advertises them; the native TLS engine (`xray-tui-tls`) performs the hybrid key exchange (X25519 + ML-KEM-768, etc.)
- **utls PQ fingerprints**: `HelloChrome_115_PQ`, `HelloChrome_115_PQ_PSK`, `HelloChrome_120_PQ` — the profile specs exist in `xray-tui-tls`; they now work end-to-end (the `supported_groups` include the hybrid curves + the key share carries the ML-KEM ciphertext)
- **VLESS encryption**: `mlkem768x25519plus` mode (`native`/`xorpub`/`random` + padding) — the native VLESS protocol implements the encryption/decryption using ML-KEM-768 + X25519
- **REALITY with ML-KEM**: the REALITY handshake can use `X25519MLKEM768` as the key share (both client + server — the server check in sing-box expects this curve)

Interoperable with xray-core 26.3.27+ and sing-box 1.18+ (both support ML-KEM).

## 2. Non-Goals (scope boundaries for SP7)

- **Full PQ TLS 1.3 spec compliance beyond hybrid curves** — only the three hybrid curves used by xray/sing-box are implemented.
- **ML-KEM-1024 beyond SecP384r1MLKEM1024** — xray only uses ML-KEM-768 (Kyber768) in the hybrid curves.
- **Key encapsulation as a standalone service** — ML-KEM is only used inside TLS hybrid curves + VLESS encryption + REALITY.
- **Post-quantum signatures (ML-DSA/Falcon)** — out of scope; only KEM.
- **sing-box feature parity beyond the curves used** — sing-box 1.18+ supports `X25519MLKEM768` in TLS + REALITY; that's the target.

## 3. Reference Inventory (wire authority)

- **xray-core 26.3.27**: `transport/internet/tls/config.go` (ParseCurveName → hybrid curve IDs), `transport/internet/tls/tls.go` (PQ utls profiles), `infra/conf/vless.go` (mlkem768x25519plus encryption parser), `proxy/vless/encryption/server.go` (ML-KEM encryption impl using `crypto/mlkem`), `transport/internet/reality/reality.go` (X25519MLKEM768 check).
- **sing-box 1.18+**: `option/tls.go` (CurvePreference X25519MLKEM768 = 4588), `common/tls/reality_client.go` (REALITY X25519MLKEM768 check), `common/sniff/quic_blacklist.go` (X25519Kyber768Draft00 = 0x11EC).
- **xray-tui-tls** (native): `profiles/chrome133.rs` (PQ fingerprint specs with X25519MLKEM768 key share — spec exists, material missing), `crypto/x25519.rs` (X25519 impl), `handshake/drive.rs` (the key share processing).
- **liboqs / oqs crate**: `oqs::kem::Kyber768` (ML-KEM-768 / FIPS 203) — the Rust binding to liboqs.

## 4. Wire Spec

### 4.1 TLS 1.3 Hybrid Curves (RFC 9180 / draft-ietf-tls-hybrid-design)

The hybrid curves combine a classical KEM (X25519 / P256 / P384) with ML-KEM-768:

| Curve ID (tls/utls) | Name | Classical | PQ |
|---|---|---|---|
| 0x11EC (4588) | X25519MLKEM768 | X25519 | ML-KEM-768 |
| 0x11EB (4587) | SecP256r1MLKEM768 | P-256 | ML-KEM-768 |
| 0x11ED (4589) | SecP384r1MLKEM1024 | P-384 | ML-KEM-1024 |

**ClientHello**: `supported_groups` extension includes these IDs (in preference order).
**KeyShare**: for the chosen hybrid curve, the key share contains:
- classical public key (32 bytes for X25519, 65 for P-256, 97 for P-384)
- ML-KEM-768 public key (1184 bytes) — the **encapsulation key** (the client generates a keypair, sends the public key; the server encapsulates to it)

**ServerHello**: selects a hybrid curve; its key share contains:
- classical public key
- ML-KEM-768 **ciphertext** (1088 bytes) — the encapsulation of the shared secret to the client's public key

**Shared secret derivation**: `HKDF-Extract(salt, classical_shared_secret || mlkem_shared_secret)` — the two shared secrets are concatenated and fed to the TLS 1.3 key schedule.

### 4.2 utls PQ Fingerprints

The profiles in `xray-tui-tls/src/profiles/` already declare the `supported_groups` with the hybrid curves (Chrome133: GREASE, X25519MLKEM768, X25519, P256, P384). The missing piece is **generating the ML-KEM key share** in the ClientHello and **processing the server's ML-KEM ciphertext** in the ServerHello.

### 4.3 VLESS Encryption: `mlkem768x25519plus`

Format (from `infra/conf/vless.go`):
```
mlkem768x25519plus.<mode>.<seconds_from>[-<seconds_to>]s.<padding>
```
- `mode`: `native` | `xorpub` | `random` (XorMode 0/1/2)
- `seconds_from[-seconds_to]`: time window for key rotation (padding timing)
- `padding`: base64url-encoded 32 or 64 byte padding strings

The encryption uses ML-KEM-768 to encapsulate a secret to the peer's X25519 public key (or vice versa) — see `proxy/vless/encryption/server.go` which imports `crypto/mlkem`.

### 4.4 REALITY with ML-KEM

The REALITY handshake's key share can be `X25519MLKEM768`. xray's `reality.go:79` checks `ServerShare.Group == utls.X25519MLKEM768`. sing-box's `reality_client.go:136` expects the curve ID 4588. The native REALITY handshake must support generating/processing this hybrid key share.

## 5. Architecture (native crate)

### 5.1 New Dependency (user-approved for SP7)

- **`oqs`** (Rust binding to liboqs) — provides `oqs::kem::Kyber768` (ML-KEM-768 / FIPS 203). Version: latest minor with minimum bound. liboqs is a C library — the build needs `liboqs` installed (system package or vendored via `oqs-sys`). This is the program's **second major new dependency** (after quinn for SP5).

### 5.2 TLS Engine (`xray-tui-tls`)

- `crypto/mlkem.rs` (new): wrapper around `oqs::kem::Kyber768` — `generate_keypair()`, `encapsulate(pk)`, `decapsulate(sk, ct)`. Returns (public_key, secret_key) and (ciphertext, shared_secret).
- `handshake/drive.rs`: when the selected curve is a hybrid curve, generate the ML-KEM keypair, include the public key in the ClientHello key share; on ServerHello, decapsulate the ML-KEM ciphertext to get the PQ shared secret; combine with the classical shared secret for the key schedule.
- `profiles/*.rs`: the PQ fingerprints (Chrome133 PQ etc.) already list the hybrid curves in `supported_groups` — no profile changes needed.
- `crypto/key_schedule.rs`: the TLS 1.3 key schedule takes `classical_secret || pq_secret` as the input keying material (the existing code expects a single shared secret — extend to accept the hybrid concatenation).

### 5.3 VLESS Protocol (`xray-tui-native/src/protocol/vless/`)

- `encryption/mlkem.rs` (new): implement the `mlkem768x25519plus` encryption/decryption per xray's `server.go`. Uses ML-KEM-768 for the KEM operations + X25519 for the classical part. Modes: `native` (direct), `xorpub` (XOR the public key), `random` (random padding). Padding handling per the time-window + base64url spec.

### 5.4 REALITY (`xray-tui-native/src/security/reality.rs`)

- The `HelloProvisioner` for REALITY must support the `X25519MLKEM768` curve (4588). When the peer selects this curve, generate the ML-KEM keypair and include the public key in the key share; process the server's ML-KEM ciphertext.

### 5.5 Config / Proto (`xray-tui-proto`)

- TLS config: the curve preference list (`alpn`/`curves` in `TlsOpts`) gains the hybrid curve names (`x25519mlkem768`, `secp256r1mlkem768`, `secp384r1mlkem1024`). The proto already carries curve preferences as strings — parse them via the existing `ParseCurveName` logic (mirror xray's `ParseCurveName`).
- VLESS encryption: the `encryption` field in the VLESS account config gains the `mlkem768x25519plus` parser (mirror xray's parser in `infra/conf/vless.go`). This is a proto change — **xray-tui-proto IS modified for SP7** (unlike previous SPs).

## 6. Error Handling

- liboqs initialization failure → `NativeError::Config` (missing system liboqs).
- ML-KEM keypair generation / encapsulation / decapsulation failure → `NativeError::Crypto` (internal error).
- TLS hybrid curve negotiation failure (peer doesn't support any hybrid) → falls back to classical curves per the preference list (standard TLS behavior).
- VLESS encryption parsing failure → `NativeError::Config` (invalid format).

## 7. Testing

### 7.1 Unit (native + tls crates)

- ML-KEM keypair/encapsulate/decapsulate roundtrip (liboqs).
- Hybrid shared secret derivation (classical || pq → HKDF).
- ClientHello key share generation for hybrid curves (includes ML-KEM public key).
- ServerHello ML-KEM ciphertext processing (decapsulate).
- VLESS `mlkem768x25519plus` encryption/decryption roundtrip (all 3 modes + padding).
- REALITY X25519MLKEM768 key share generation + processing.

### 7.2 Hermetic

- A fake TLS 1.3 server double (loopback) that negotiates a hybrid curve and performs the ML-KEM encapsulation/decapsulation, driving the native client's handshake through the PQ path without real cores.

### 7.3 e2e (tier-3, `native-e2e` + `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin`)

- xray-core 26.3.27+ rows: TLS with hybrid curves (X25519MLKEM768), VLESS `mlkem768x25519plus` encryption, REALITY with X25519MLKEM768. sing-box 1.18+ rows: same.
- Count: vless +3 (tls-pq, vless-pq-enc, reality-pq), vmess +1 (tls-pq) — total ~134.

### 7.4 Gates

- clippy 0 (both feature sets); fmt clean; unit lib; e2e sweeps; proto changes reviewed.

## 8. Deviations from Upstream

1. **liboqs vs Go's `crypto/mlkem`** — xray uses Go's stdlib ML-KEM; native uses liboqs via `oqs` crate. Wire-compatible (both FIPS 203 ML-KEM-768).
2. **SecP384r1MLKEM1024** — xray supports it; native implements ML-KEM-768 only (Kyber768). The SecP384r1MLKEM1024 curve uses ML-KEM-1024 which liboqs also provides (`Kyber1024`) — implement if the curve appears in e2e; otherwise defer (xray's primary is X25519MLKEM768).
3. **No PQ signatures** — only KEM; xray/sing-box don't use PQ signatures in the parity scope.

## 9. Verification Order

1. liboqs integration + ML-KEM unit tests (crypto crate).
2. TLS engine hybrid curve support (handshake + key schedule).
3. VLESS encryption impl + unit tests.
4. REALITY X25519MLKEM768 support.
5. Proto changes (curve names + VLESS encryption).
6. e2e rows (xray + sing-box where available).
7. Docs (NATIVE_CORE.md + AGENTS.md) + full gates.