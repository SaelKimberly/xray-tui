# NATIVE_CORE.md — Native proxy core + TLS fingerprint engine

Status and roadmap for the two crates that remove the subprocess dependency for
the tunnel itself: **`xray-tui-native`** (in-process client-side protocol
implementations) and **`xray-tui-tls`** (the ring-based TLS 1.3 client that
backs the fingerprint/REALITY security paths).

Sibling docs: `AGENTS.md` (agent guide), `ARCHITECTURE.md` (whole-system
architecture), `CONTEXT.md` (domain glossary + decisions), `docs/protocols.md`
(protocol wire specs), `docs/clash.md` (Clash YAML conversion).

## Purpose

The main xray-tui app gets its tunnel by spawning xray-core or sing-box as a
subprocess, writing JSON configs, and talking gRPC. That works, but it is
heavy: two Go binaries to ship, update, and babysit. The native core is the
long-term replacement — a pure-Rust, in-process client for the same protocols,
built on the same `xray-tui-proto` typed configs the subprocess path already
uses. The TLS engine exists because stock rustls ClientHellos are instantly
identifiable: browser-fingerprint mimicry (and REALITY) needs a client hello
we fully control.

Both crates are client-side only. Server-side protocol behavior remains the
job of xray-core / sing-box; the e2e harness runs real cores as servers to
prove wire compatibility.

## Principles

- **Xray composition order, one layer stack**: `dial → transport → security →
  protocol → tunnel`. Each phase consumes the previous phase's byte stream
  (`BoxStream`) and returns the next. A protocol = the same handshake-over-
  stream pipeline unless its `ConnectShape` says otherwise (device tunnels,
  own-handshake protocols, outbound-only kinds).
- **Config source of truth is `xray-tui-proto`.** No config model is defined in
  native; `NativeConnectParams` wraps the typed `ProtocolConfig` +
  `EndpointEssentials`. The host/port is injected from the endpoint at connect
  time.
- **`xray-tui-tls` is ring-only**: no aws-lc-rs, no `rand`, no `unsafe`.
  CSPRNG enters through a crate-local `SecureRandom` seam (ring's trait is
  sealed; a blanket impl covers any `ring::rand::SecureRandom`). One
  documented exception: x25519-dalek (runtime dep) for the REALITY client,
  because ring's `EphemeralPrivateKey` is single-use and cannot serialize —
  REALITY must agree twice with the same scalar.
- **Explicit absence beats silent fallback.** Every protocol kind, transport,
  and security has a module and a dispatch arm. Unsupported combinations
  return `NativeError::NotImplemented` naming the missing feature — no stub
  that pretends to work.
- **Real verification, three tiers** (see below). Wire-format claims carry
  byte-level evidence: RFC 8448 vectors, JA3/JA4 goldens, Go VMess vectors,
  live tls.peet.ws grading, real-core e2e.
- **`xray-tui-proto` is never modified by these crates.** If a protocol config
  lacks a field the tunnel needs, that's a proto change evaluated separately.

## Verification tiers

| Tier | Gate | What runs | Evidence |
|------|------|-----------|----------|
| 1 — offline | `cargo test -p xray-tui-tls -p xray-tui-native` | unit + integration: wire encodings, RFC 8448 key-schedule vectors, GREASE pairing, JA3/JA4 goldens, VMess Go byte-vectors, rustls-server interop (dev-dep), multi-record reassembly | 172 tests (62 tls + 110 native) |
| 2 — live grader | `cargo run -p xray-tui-tls --example grader -- --profile <id>`; `cargo test -p xray-tui-tls --test tls_peet_ws -- --ignored` | ClientHello graded against tls.peet.ws | Chrome130 JA4 `t13d1516h2_8daaf6152771_f37e75b10bcc`; Firefox128ESR JA3 `361e0ca6ef1ca4dbe3a1d987722a1980` + JA4 `t13d1314h2_07be0c029dc8_46701d79520f` |
| 3 — real-core e2e | `XRAY_TUI_CORE_BIN_DIR=<dir> cargo test -p xray-tui-native --features native-e2e --test vless --test vmess` | native client against spawned xray-core (26.3.27) + sing-box (1.13.16) servers | 7 cases × 2 cores green (see e2e cases) |

Tier 2 needs network; tier 3 needs the version-pinned core binaries (hard-fail,
not skip, on version mismatch). Tier 1 is hermetic and is the CI gate.

## Crate maps

### xray-tui-native (`crates/xray-tui-native/src/`)

| Module | Responsibility |
|--------|----------------|
| `lib.rs` | `Stream`/`BoxStream` seam, `connect()` entry, `NativeTunnel` |
| `chain.rs` | `connect_chain`: fold the layer stack |
| `context.rs` | `LinkContext`, `NativeConnectParams` (wraps proto types) |
| `addr.rs` | `TargetAddr` (domain/IP + port) encode/decode |
| `transport/` | TCP dial (only transport implemented) |
| `security/` | `wrap()` dispatch: `tls.rs` (tokio-rustls), `fingerprint.rs` (`FingerprintConnector`), `reality.rs` (`RealityConnector`), `tls_provider.rs` (`TlsProvider` plug, `TlsParams`) |
| `protocol/` | 20 protocol modules; only `vless` + `vmess` implemented, rest `NotImplemented` |
| `crypto/` | VMess-adjacent primitives (aead/kdf/legacy_stream/salamander stubs) |
| `shape.rs` | `ConnectShape`: uniform vs divergent connect paths |
| `e2e/` (feature `native-e2e`) | case/config/core/harness/variant — real-core scenarios |

### xray-tui-tls (`crates/xray-tui-tls/src/`)

| Module | Responsibility |
|--------|----------------|
| `spec/` | declarative `ClientHelloSpec`/`ExtensionSpec`/`SessionIdSpec`, RFC 6066/8446 wire encodings, GREASE (RFC 8701) |
| `profiles/` | 12 browser profiles as spec data (`define_profiles!` macro): Chrome119/130/133, ChromeAndroid130, Edge130, Brave167, Opera114, Firefox, Firefox128Esr, Safari17, SafariIos17 (+ `Chrome` = Chrome130 alias) |
| `hello/` | `build_hello`/`to_record` (GREASE pairing, 512-byte record padding), `parse_hello` |
| `crypto/` | key schedule (RFC 8448-verified), AEAD record keys (IV XOR seq), `X25519KeyPair`, `fingerprint/` JA3 + JA4 encoders |
| `record/` | record framing, `read_record`, `TlsStream<S>` (AsyncRead/Write, close_notify→EOF) |
| `handshake/` | TLS 1.3 client handshake, `ServerVerifier` seam, multi-record flight reassembly |
| `verify/` | `WebPkiVerifier` (roots/CA DER/`insecure`/`pin_sha256`; CV signature always checked) |
| `reality/` | `HelloProvisioner` + 9-step wire contract, `FixedChrome133`, auth-key/session-seal/server-auth |
| `http2/` | minimal h2 layer (tls.peet.ws grading only) |
| `error.rs` | `TlsError`/`Result` (thiserror) |

## Security layer capabilities

`security::wrap` dispatch (from the profile's `TlsConfig`):

| Path | Trigger | Mechanism | Status |
|------|---------|-----------|--------|
| Standard TLS | `tls` config, no `fp`, provider `Standard` | tokio-rustls (workspace rustls, ring backend) | ✅ |
| Fingerprint TLS | `tls` config with any `fp` value, or `TlsProvider::Custom` | `FingerprintConnector` → xray-tui-tls handshake + `WebPkiVerifier`; profile = `parse_fingerprint_id(fp)` (exact ids: `chrome`/`chrome-randomized`/`firefox`/`safari`/`random` → Chrome130/Firefox128Esr/Safari17; unknown → config error) | ✅ |
| REALITY | `reality` config | `RealityConnector` → xray-tui-tls REALITY client (`HelloProvisioner`); server authenticated by auth key, not PKI | ✅ |
| Trust modes | `insecure` / `pin_sha256` | `with_insecure()` skips chain walk; `with_pin(sha256(SPKI))` replaces chain+SAN but **never** skips the CertificateVerify signature (a MITM must hold the private key) | ✅ |

`TlsParams` carries sni/alpn/fingerprint/insecure/pin_sha256. `fp.is_some() ||
Custom` is the exact routing condition — a non-empty `fp` always means the
fingerprint engine, never stock rustls.

## E2E coverage (tier 3)

7 cases, each run against both cores (xray 26.3.27, sing-box 1.13.16), each
spawning a real server inbound + dialing it with the native client and probing
HTTP through the tunnel:

| Case | Payload security | TLS variant |
|------|------------------|-------------|
| VLESS | — | tls-standard |
| VLESS | — | tls-chrome (fingerprint engine) |
| VLESS | — | reality |
| VMess | aes-128-gcm | tls-standard |
| VMess | chacha20-poly1305 | tls-standard |
| VMess | aes-128-gcm | tls-firefox (fingerprint engine) |
| VMess | aes-128-gcm | reality |

## Protocol roadmap

Legend (emoji = phase/decision per column):

| Mark | Meaning |
|------|---------|
| ✅ | done, verified |
| 🚧 | in progress |
| 📋 | todo (planned) |
| ⏸ | deferred (planned but not scheduled) |
| ⛔ | won't implement (decision) |
| 🔒 | blocked on a prerequisite |

Capability columns compare what the mainstream repositories (xray-core,
sing-box — `thirdparty/`) provide for each protocol. "Native client" is the
xray-tui-native tunnel; "TLS engine" is the xray-tui-tls path that backs it
(the engine is fully built — the column marks whether the protocol routes
through it); "REALITY" marks REALITY compatibility; "e2e" is tier-3 proof.

### Overview matrix

| Protocol | xray-core | sing-box | Native client | TLS engine | REALITY | e2e |
|----------|:---------:|:--------:|:-------------:|:----------:|:-------:|:---:|
| VLESS | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| VMess | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Trojan | ✅ | ✅ | 📋 | ✅ | ✅ | 📋 |
| Shadowsocks | ✅ | ✅ | 📋 | — | — | 📋 |
| ShadowsocksR | ⛔ | ✅ | 📋 | — | — | 📋 |
| SOCKS | ✅ | ✅ | 📋 | — | — | 📋 |
| HTTP | ✅ | ✅ | 📋 | — | — | 📋 |
| WireGuard | ✅ | ✅ | 📋 | — | — | 📋 |
| Hysteria2 | ✅ | ✅ | 📋 | 🔒 QUIC | — | 📋 |
| Hysteria1 | ⛔ | ✅ | 📋 | 🔒 QUIC | — | 📋 |
| TUIC | ⛔ | ✅ | 📋 | 🔒 QUIC | — | 📋 |
| Naïve | ⛔ | ✅ | 📋 | ✅ | — | 📋 |
| AnyTLS | ⛔ | ✅ | 📋 | ✅ | — | 📋 |
| ShadowTLS | ⛔ | ✅ | 📋 | ✅ | — | 📋 |
| Tor | ⛔ | ✅ | 📋 | — | — | 📋 |
| SSH | ⛔ | ✅ | 📋 | — | — | 📋 |
| Tailscale | ⛔ | ✅ | 📋 | — | — | 📋 |
| Redirect | ⛔ | ✅ | 📋 | — | — | 📋 |
| TProxy | ⛔ | ✅ | 📋 | — | — | 📋 |
| Mixed | ⛔ | ✅ | 📋 | — | — | 📋 |

Notes on the matrix:
- **TLS engine / REALITY columns** are "✅" only for the TCP-stream family
  where the security phase applies. QUIC-family protocols (Hysteria1/2, TUIC)
  carry TLS inside QUIC and need a QUIC transport first — 🔒.
- **Shadowsocks** gets no TLS column (plain TCP + AEAD; obfuscation comes from
  plugins, which are 📋 transport work). `ss` method whitelists live in
  `proto_spec/core_mapping.rs`.
- **Redirect/TProxy/Mixed** are outbound-only kinds (kernel/port forwarding):
  no client handshake of their own — the native `connect` is trivially a dial,
  but the `ConnectShape` path must exist.
- **Tor** is a full onion-routing stack (circuits, cell crypto, directory
  protocol); realistically ⛔ for the in-process core — the subprocess path
  stays the answer.
- **SSH/Tailscale/WireGuard** are device-tunnel or own-handshake shapes —
  divergent `connect()` paths, not the uniform pipeline.

### Per-protocol detail

**VLESS** — ✅ native
| Capability | Detail |
|------------|--------|
| Encryption | none (identity = UUID; optional `xtls-rprx-vision` flow control, TLS 1.3 framing) |
| Auth | UUID (command bytes in header) |
| Obfuscation | none at protocol level; REALITY supplies traffic camouflage |
| Transports | TCP, WS, gRPC, h2, QUIC (xray-core); TCP/WS/gRPC/h2/QUIC (sing-box) — native: TCP only 📋 |
| Status | Native client complete + e2e (tls-standard, tls-chrome, reality) × both cores. Vision flow control 📋. Non-TCP transports 📋. |

**VMess** — ✅ native
| Capability | Detail |
|------------|--------|
| Encryption | payload: `aes-128-gcm`, `chacha20-poly1305` (native ✅); legacy `aes-128-cfb` / `none` (xray-only, ⛔ legacy disabled in modern cores) |
| Auth | AEAD request header (mandatory), MD5/HMAC-SHA256 KDF chain, FNV-1a + CRC-32 frame checks |
| Obfuscation | none at protocol level (transport-level ws/grpc/http in xray) |
| Transports | TCP, WS, gRPC, h2, QUIC (xray-core); TCP/WS/gRPC/h2 (sing-box) — native: TCP only 📋 |
| Status | Native client complete + e2e (aes-128-gcm, chacha20-poly1305, tls-firefox, reality) × both cores. Legacy ciphers ⛔. Non-TCP transports 📋. |

**Trojan** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | none — plaintext after TLS; security is the TLS layer |
| Auth | password (SHA-224-derived key in the header) |
| Obfuscation | none (REALITY can carry it) |
| Transports | TCP, WS, gRPC, h2, QUIC (xray-core); TCP/WS/gRPC/h2/QUIC (sing-box) — native: TCP only 📋 |
| Status | Simple: TLS + password header + raw tunnel. TLS engine ✅. |

**Shadowsocks** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | AEAD: `aes-128/256-gcm`, `chacha20-ietf-poly1305`, `xchacha20`, `2022-blake3-*` (both cores); legacy stream: `aes-*-cfb/ctr`, `rc4-md5`, `chacha20-ietf`, `none` (sing-box only) |
| Auth | none beyond AEAD key from password (2022 adds user lists) |
| Obfuscation | plugin: `simple-obfs`, `v2ray-plugin` (via URL `plugin` param) 📋 |
| Transports | TCP + UDP (same key); native UDP path 📋 |
| Status | Crypto primitives exist in `protocol/ss` + `crypto/` scaffolding; AEAD framing (2-byte length + tag) straightforward. Cipher-validity rules already in `core_mapping.rs`. |

**ShadowsocksR** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | stream ciphers (`aes-*-cfb/ctr`, `rc4-md5`, `chacha20`, …) |
| Auth | protocol plugins: `auth_chain_a/b`, `auth_aes128_*` |
| Obfuscation | obfs plugins: `http_simple`, `tls1.2_ticket_auth`, `plain` |
| Transports | TCP |
| Status | sing-box-only; obfs/auth plugin matrix is the bulk. 📋. |

**SOCKS** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | none (SOCKS5) |
| Auth | optional username/password (RFC 1929), no-auth |
| Obfuscation | none |
| Transports | TCP (+ UDP ASSOCIATE 📋) |
| Status | Simple handshake; low priority (client rarely proxies to a bare SOCKS server). |

**HTTP** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | none (HTTP CONNECT) |
| Auth | Basic auth (optional) |
| Obfuscation | none |
| Transports | TCP |
| Status | Trivial CONNECT proxy. |

**WireGuard** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | Noise IK handshake (x25519 + ChaCha20Poly1305), data = ChaCha20Poly1305 |
| Auth | static/temporary public keys, PSK optional |
| Obfuscation | none |
| Transports | UDP device tunnel |
| Status | 🔒 needs TUN/device abstraction or userspace UDP socket tunnel. `ConnectShape` divergent. |

**Hysteria2** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | TLS 1.3 inside QUIC (standard + fingerprint TLS ✅ via engine) |
| Auth | password in QUIC 0-RTT |
| Obfuscation | `salamander` obfs (crypto/ `salamander.rs` scaffolding) |
| Transports | QUIC (UDP) — 🔒 no QUIC transport yet |
| Status | Client needs a QUIC stack; out of scope until `quic` transport 📋. |

**Hysteria1** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | none (protocol-level); TLS optional via QUIC |
| Auth | password |
| Obfuscation | `salamander` |
| Transports | QUIC + bandwidth control |
| Status | sing-box-only; 🔒 QUIC. |

**TUIC** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | TLS inside QUIC (v5: TLS 1.3, UUID auth in QUIC handshake) |
| Auth | UUID |
| Obfuscation | none |
| Transports | QUIC |
| Status | sing-box-only; 🔒 QUIC. |

**Naïve** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | TLS 1.3 (HTTP/2) — Chromium networking stack fingerprint required |
| Auth | Basic auth over H2 |
| Obfuscation | the entire point: looks like Chrome H2 to a CDN |
| Transports | TCP (h2) |
| Status | TLS engine ✅ (fingerprints cover Chrome); needs real HTTP/2 client (current `http2/` is grader-minimal, ⛔ not a client) → 🔒 on a full h2 implementation. |

**AnyTLS** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | none beyond TLS; padding obfuscation of the inner handshake |
| Auth | password (scrambled with the handshake) |
| Obfuscation | TLS-like padding/random extension injection |
| Transports | TCP |
| Status | TLS engine ✅; protocol layer simple once h2/record work exists. |

**ShadowTLS** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | none — shadowsocks-style relay over camouflaged TLS |
| Auth | password picked by SNI-specific key derivation (v3) |
| Obfuscation | TLS-in-TLS: outer TLS to a real site, inner plaintext relay |
| Transports | TCP |
| Status | TLS engine ✅ (outer hello mimicry is exactly what profiles do). |

**Tor** — ⛔ native
| Capability | Detail |
|------------|--------|
| Encryption | onion circuits: AES-128-CTR cells, RSA/Ed25519 handshakes, per-hop keys |
| Auth | circuit-level (no server password) |
| Obfuscation | pluggable transports (obfs4, …) |
| Transports | TCP + directory protocol |
| Status | Full Tor stack is a project of its own — keep the sing-box subprocess path. ⛔ in-process. |

**SSH** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | chacha20-poly1305 / aes-gcm / aes-ctr, host-key auth (ed25519/rsa) |
| Auth | username + password/key |
| Obfuscation | none |
| Transports | TCP (own handshake — `ConnectShape` divergent) |
| Status | sing-box uses SSH as proxy transport. A Rust SSH client crate could back it; otherwise 📋. |

**Tailscale** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | WireGuard-derived (x25519 + ChaCha20Poly1305), DERP relay fallback |
| Auth | tailnet key (coordination server) |
| Obfuscation | none |
| Transports | UDP device tunnel + DERP |
| Status | Needs the coordination protocol; realistically the tailscale-client crate or ⛔. 📋. |

**Redirect / TProxy / Mixed** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | none (kernel-level / port forwarding) |
| Auth | none |
| Obfuscation | none |
| Transports | n/a (outbound-only kinds) |
| Status | `PlaceholderConfig` in proto; native arm exists (NotImplemented). A redirect dial is ~20 lines once shapes land. |

## Transport roadmap

| Transport | Status | Notes |
|-----------|:------:|-------|
| TCP | ✅ | `transport/tcp.rs`; all e2e cases run over it |
| WS / gRPC / h2 / HTTPUpgrade / XHTTP | 📋 | config already parses transport fields in proto; no native upgrade layer |
| QUIC | 🔒 | prerequisite for Hysteria1/2 + TUIC clients |
| KCP | 📋 | xray-core only; niche |
| obfs plugins (SS) | 📋 | plugin URL param already parsed by proto |

## Where this is headed

1. **Finish the TCP-stream protocol family** (Trojan, Shadowsocks, SOCKS, HTTP,
   AnyTLS, ShadowTLS, Naïve) — the TLS engine is done, the shapes are uniform.
2. **Non-TCP transports** (WS first — most common real-world flavor).
3. **QUIC transport** unlocks Hysteria1/2 + TUIC.
4. **Wire in the TUI**: a per-profile "native" toggle (or auto-fallback when a
   core binary is missing) — the `TlsProvider` seam already proves the
   integration point.
5. **Outbound-only kinds + routing** via native (redirect dial for
   split-tunnel rules).
