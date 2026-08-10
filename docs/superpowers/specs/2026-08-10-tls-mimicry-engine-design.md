# TLS Fingerprint Mimicry Engine — Design Spec

Date: 2026-08-10
Status: Approved (design review)
Branch: `native-core-stub`

## Goal

Build a lightweight, extensible TLS library with browser-fingerprint mimicry
as an enhancement to the partially implemented `xray-tui-native` subcrate:
a custom TLS engine that can emit a mimicked `ClientHello` (JA3/JA4 matching
real browsers), carry VLESS/VMESS traffic to real xray-core / sing-box
servers, and pass tls.peet.ws fingerprint grading.

## Context (evidence)

- `xray-tui-native` (branch `native-core-stub`, clean build) implements the
  client-side VLESS and VMESS protocol phases (headers, AEAD request sealing,
  response-header peel, aes-128-gcm + chacha20-poly1305 payload security) and
  an e2e harness (`src/e2e/`, feature `native-e2e`, env
  `XRAY_TUI_CORE_BIN_DIR`) that runs real xray 26.3.27 and sing-box 1.13.16
  servers with local TLS (rcgen certs, thread-local test trust store).
- The TLS seam for fingerprint mimicry exists but is unwired:
  `security/tls_provider.rs` defines `TlsProvider::Standard | Custom(Arc<dyn
  TlsConnector>)`, `FingerprintId` (chrome / chrome-randomized / firefox /
  safari / random), `TlsParams { sni, alpn, fingerprint }`. `security/mod.rs
  ::wrap()` only dispatches to rustls. `security/reality.rs` defines the
  `HelloProvisioner` trait + `FixedChrome133` (stub, M3) with the full wire
  contract documented in its module doc.
- `TlsOpts.fp` (the URL `fingerprint=` param) is already parsed by
  `xray-tui-proto` (`common.rs`, `SecurityConfig::fp()`).
- `xray-tui-native/src/crypto/fingerprint/` has offline JA3/JA4 encoders
  (tier-1 verification); the tier-2 tls.peet.ws grader was planned for the
  fingerprint engine milestone.
- `thirdparty/tls-fingerprint/` (user side project, MIT-style self-owned) is a
  complete hand-rolled TLS 1.3 stack on aws-lc-rs: configurable `ClientHello`
  (typed `Extension` builders), X25519 + HKDF + AEAD key schedule, record
  layer, HTTP/1.1 + HTTP/2 response framing, a tls.peet.ws example, and 10
  hand-coded browser profiles. Weaknesses: profiles are copy-paste builder
  functions (no data model), no certificate verification, TLS 1.3 only,
  `TcpStream`-bound I/O (no generic stream seam, no `AsyncRead`/`AsyncWrite`
  record stream for tunnels).
- `thirdparty/utls/` — reference for the declarative spec model:
  `ClientHelloSpec` (cipher suites + ordered `TLSExtension` list + version
  bounds) as data, `ApplyPreset` filling GREASE/keyshare/session-id at
  connect time, presets from tlsfingerprint.io captures. GREASE is randomized
  per connection with a seedable PRNG.
- `thirdparty/wreq-util/` — reference for extendability ergonomics:
  `define_enum!` macro generates the profile enum + dispatch; each browser
  version is a module with one `emulation()` fn; ~120 profiles via "one
  module + one macro line".
- `thirdparty/shoes/src/reality/` — MIT-licensed REALITY client implementation
  (aws-lc-rs, rustls-compatible API, slide-buffer record handling) to adopt;
  `thirdparty/REALITY/` — Go reference (fork of crypto/tls).
- Baseline fingerprints captured with the current `tls-fingerprint` example
  against tls.peet.ws (2026-08-10):
  - Chrome/130: JA3 `2b916ec56aedf4a5ecbeb5804f60c242` (GREASE-randomized
    per run), JA4 `t13d1516h2_8daaf6152771_f37e75b10bcc`, HTTP/2.
  - Firefox/128 ESR: JA3 `361e0ca6ef1ca4dbe3a1d987722a1980` (stable — no
    GREASE), JA4 `t13d1314h2_07be0c029dc8_46701d79520f`, HTTP/2.

## Decisions (user-approved)

1. **Crypto backend: ring.** The workspace/native standard (rustls ring
   backend, `ring` CSPRNG, Reality stub already commits to ring). No
   aws-lc-rs in the new crate; port tls-fingerprint crypto to ring.
2. **Extensibility: hybrid — uTLS-style declarative spec data + wreq-util-style
   `define_profiles!` macro dispatch.** Profile = one `spec()` fn returning
   `ClientHelloSpec` data + one macro line. No engine changes to add a
   profile.
3. **Certificate verification: implemented.** rustls-webpki chain + SAN
   verification, CertificateVerify signature via ring, honoring
   `TlsOpts.insecure` (skip) and `TlsOpts.pin_sha256` (leaf SPKI hash
   override). Default: verify (keeps the native crate invariant).
4. **TLS 1.3 only.** JA3 hello still carries TLS 1.2 suites (fingerprint
   fidelity); actual negotiation is 1.3. HelloRetryRequest → explicit
   `TlsError::HelloRetryRequest`, not retried (rare on modern servers;
   xray/sing-box don't force it).
5. **REALITY in scope.** Adopt shoes reality client (client side), port to
   ring, reusing the crate's record layer + key schedule. The
   `HelloProvisioner` trait + `FixedChrome133` become real.
6. **New workspace crate `xray-tui-tls`.** Standalone; `xray-tui-native`
   depends on it. Matches the proto/dns/geoip/host-features crate split.
7. **Verification targets: Chrome 130 + Firefox 128 ESR**, expected JA3/JA4
   hardcoded from captured real-browser values, asserted in tier-2 grader
   (example binary + `#[ignore]`d tests). No implicit network in usual tests.

## Architecture

```
xray-tui-tls
├── spec/        ClientHelloSpec + ExtensionSpec + SessionIdSpec + Grease
├── profiles/    profile spec data per browser/version + define_profiles! macro
├── hello/       ClientHello build (host, keyshare, grease, rng → bytes) + parse
├── crypto/      ring port: X25519, HKDF key schedule, AEAD, transcript,
│                JA3/JA4 encoders
├── record/      TlsRecord read/write + TlsStream<S> (AsyncRead/AsyncWrite)
├── handshake/   TLS 1.3 client handshake — plain-TLS + REALITY personalities
├── verify/      cert chain (rustls-webpki + webpki-roots), SAN, pin, sig verify
├── reality/     REALITY client: auth key, session-id seal, server auth
├── http2/       HTTP/2 client framing (for the grader)
└── error.rs
```

- Layering matches Xray composition order in `xray-tui-native`:
  dial → transport → security → protocol. The engine is the security layer's
  custom provider.
- I/O seam: everything generic over `S: AsyncRead + AsyncWrite + Unpin +
  Send` (tokio traits). The native `TlsConnector::connect(BoxStream,
  TlsParams) -> BoxStream` contract stays; the engine returns a
  record-framed `TlsStream<BoxStream>`.
- `xray-tui-native` side changes: real `security/fingerprint.rs` connector
  (implements `TlsConnector`); `security/reality.rs` becomes wiring over
  `xray-tui-tls::reality`; `HelloProvisioner` moves to `xray-tui-tls`;
  `crypto/fingerprint/` (JA3/JA4 encoders) moves to `xray-tui-tls` and native
  re-imports.

## Component details

### spec + profiles (extensibility model)

```rust
pub struct ClientHelloSpec {
    pub legacy_version: u16,          // 0x0303
    pub cipher_suites: Vec<u16>,      // Grease::Slot allowed
    pub compression_methods: Vec<u8>, // [0]
    pub session_id: SessionIdSpec,    // Random32 | AuthPayload (REALITY)
    pub extensions: Vec<ExtensionSpec>,
}

pub enum ExtensionSpec {
    ServerName, SupportedGroups(Vec<u16>), KeyShareX25519,
    SupportedVersions(Vec<u16>), SignatureAlgorithms(Vec<u16>),
    Alpn(Vec<String>), EcPointFormats, SessionTicket, PskKeyExchangeModes,
    StatusRequest, SignedCertificateTimestamp, RenegotiationInfo,
    CompressCertificate(Vec<u16>), ApplicationSettings(Vec<String>),
    RecordSizeLimit(u16), Padding, Grease, Raw(u16, Vec<u8>),
}
```

- Spec = static data; `build()` injects runtime values: SNI (from params),
  keyshare public key, GREASE values (seeded RNG), session id, padding.
- GREASE pairing (Chrome family): one random value shared across
  cipher_suites[0], supported_groups[0], supported_versions[0], key_share[0]
  (same group id), plus a second value for the standalone GREASE extension.
  Firefox family: no GREASE.
- `define_profiles!` macro generates `BrowserProfile` enum +
  `name()`/`spec()`/`all()` dispatch. Existing 10 tls-fingerprint profiles are
  ported to spec data verbatim (extension lists already captured from real
  browsers).

### record + handshake (TLS 1.3 client)

1. Build hello from spec → send as record version 0x0301.
2. Read ServerHello (skip ChangeCipherSpec). HRR → `TlsError::HelloRetryRequest`.
3. Key schedule (HKDF, transcript hash), decrypt EncryptedExtensions /
   Certificate / CertificateVerify / Finished.
4. Verify: rustls-webpki `EndEntityCert::verify_for_usage` (chain + SAN vs
   SNI), CertificateVerify signature via ring. `insecure` skips, `pin_sha256`
   checks leaf SPKI hash. ALPN mismatch tolerated (no rejection).
5. Client Finished; derive app keys; return `TlsStream<S>`.

`TlsStream<S>`: buffered record reader (fragmentation, inner content type,
close_notify → EOF), 16 KB-split write encryption, `AsyncRead/AsyncWrite/
Unpin/Send` impls.

### reality (REALITY client)

Adopt shoes (client side), port to ring; reuse our record layer + key
schedule (shoes duplicates this machinery for its rustls-compatible API):

1. X25519 keypair + client_random.
2. `shared = ECDH(client_priv, server_pub from pbk)`.
3. `auth_key = HKDF-SHA256(shared, salt = random[0..20], info "REALITY")`.
4. SessionId plaintext (16 B) = version(1,8,0) + pad(1) + timestamp u32 BE +
   short_id(8).
5. ClientHello: Chrome-133-shaped (FixedChrome133 provisioner), SNI = steal
   target, suites 0x1301-03, ALPN h2+http/1.1.
6. AAD = hello with SessionId zeroed; nonce = random[20..32]; SessionId =
   AES-256-GCM(auth_key, nonce, plaintext, aad).
7. Restore SessionId (builder-returned byte range).
8. Server auth: cert signature field == HMAC-SHA512(auth_key, ed25519_pub);
   Ed25519 CertificateVerify over transcript. No PKI chain.
9. TLS 1.3 key schedule + Finished, then app records.

`HelloProvisioner` trait moves to `xray-tui-tls`; `FixedChrome133` real
impl. Native `reality.rs` keeps only the connect wiring.

## Integration (xray-tui-native)

- `security/wrap()` dispatch: `TlsConfig::Tls(opts)` → fingerprint engine
  when `opts.fp.is_some()` OR a `Custom` provider is configured, else rustls.
  `TlsConfig::Reality` → reality connector (real now).
- `FingerprintId` parse from `fp` string: exact ids (`chrome`,
  `chrome-randomized`, `firefox`, `safari`, `random`); unknown →
  `NativeError::Config`. Mapping: Chrome → Chrome130, Firefox → Firefox128Esr,
  Safari → Safari17, `random`/`chrome-randomized` → Chrome130 shape + random
  GREASE (uTLS full randomization deferred, documented).
- ALPN: profile default (h2, http/1.1) overridden by config `alpn`; Reality
  fixed h2, http/1.1.
- Timeouts: `timeouts::SECURITY` (10 s) around handshake, same as rustls path.
  Errors mapped to `NativeError::Tls` / `NativeError::Reality`.
- Trust store: thread-local harness CA hook (mirror of `tls.rs` `TEST_CFG`) so
  e2e works with rcgen certs.

## Verification

- **Tier 1 (offline, always run)**: fixed-seed build → parse own hello → JA3/
  JA4 assert against locked constants; ClientHello golden-bytes test; handshake
  against a rustls server (rcgen certs) for both personalities.
- **Tier 2 (network, opt-in)**: `grader` example (port of
  `fingerprint_test`) + `#[ignore]`d tests hitting tls.peet.ws `/api/all`;
  assert JA3/JA4 == expected Chrome130 / Firefox128ESR values (cross-checked
  against tlsfingerprint.io real-browser captures during implementation).
- **Tier 3 (real cores, `native-e2e` + `XRAY_TUI_CORE_BIN_DIR`)**: new cases —
  vless-tls-chrome, vmess-tls-firefox, vless-reality, vmess-reality — against
  xray 26.3.27 AND sing-box 1.13.16. Reality inbound: generate x25519 keypair
  client-side, derive `pbk`, server gets privateKey.
- JA3/JA4 encoders move to `xray-tui-tls` (compute from our own hello
  fields); native drops `crypto/fingerprint`, re-imports.

## Error handling & conventions

- `xray-tui-tls` error enum (thiserror): `Spec`, `Handshake`, `Verify`,
  `Crypto`, `Io` — mapped at the native boundary.
- ring only; no aws-lc-rs, no `rand` crate. Seedable RNG via
  `ring::rand::SecureRandom` trait (SystemRandom production, FixedSeed tests).
- Edition 2024, workspace lints, rustfmt. No `unsafe`.

## Risks

- GREASE pairing subtleties and tls.peet.ws JA4 semantics — empirical; build
  the grader early.
- Shoes Reality is aws-lc-based; the port is the largest chunk (~4.5K lines
  client-side, with our records/keys reused).
- rustls-webpki signature-scheme mapping (medium).
- HRR intentionally unsupported (documented error, not silent failure).

## Non-goals

- TLS 1.2 handshake, HRR retry, session resumption, uTLS-style randomized
  spec generation (HelloRandomized), QUIC, ECH.
- Protocol phases beyond the existing native crate scope (WS/gRPC transports,
  flows like xtls-rprx-vision).
