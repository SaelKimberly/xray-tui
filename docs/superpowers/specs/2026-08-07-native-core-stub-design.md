# Native Core Stub — Design Spec

- Date: 2026-08-07
- Status: approved for implementation (pending user review of this doc)
- Owner: xray-tui workspace
- Scope: new crate `crates/xray-tui-native`

## 1. Purpose

A native, in-process client-side implementation of the proxy protocols that
xray-tui currently obtains by generating JSON configs and spawning xray-core /
sing-box subprocesses. Native connect gives instant availability checks and
better TUI responsiveness, and eventually replaces the temp-core spawning in
the real-ping / speed-test / outbound-IP pipelines.

Session goal: a maintainable placeholder skeleton covering **all 27 protocol
kinds, all transports, all securities**, plus **one fully working protocol —
VLESS over TCP with TLS — proven end-to-end against real xray-core AND
sing-box servers**, tested locally with no external network.

The crate is *client-only*. Server-side implementations are explicitly out of
scope; the e2e harness uses real core binaries as servers.

## 2. Non-goals (this session)

- TUI integration and any change to `RealPingManager` / `CorePool` / ping
  pipeline. The crate is standalone until a later milestone.
- VLESS `xtls-rprx-vision` flow, VLESS UDP/Mux commands, mux in general.
- REALITY implementation (contract is specified here; code is a stub returning
  `NotImplemented`).
- Fingerprint-mimicry TLS engine implementation (seam is specified here;
  standard rustls is the only working provider).
- SSH and WireGuard connect paths (classified, stubbed).
- Server-side core implementations.
- Automated download of xray/sing-box binaries for tests.

## 3. Ground rules and decisions (from the design session)

1. **License hygiene.** Workspace is MIT. Wire logic is modeled on
   `v2ray-core` (MIT — the clean reference). `Xray-core` (MPL-2.0) and
   `mihomo` (GPL-3.0) are specification references only — never copied.
   `shoes` (MIT), `jets` / `ray` (Apache-2.0) are readable prior art. All code
   is written fresh in this crate's own style; protocol contracts are
   attributed, not imported.
2. **Config identity — single source of truth.** The native crate depends on
   `xray-tui-proto` and consumes its typed models directly:
   `ProtocolConfig` (27-kind enum), `EndpointEssentials` (host/port split),
   `TransportConfig`, `SecurityConfig`, `ProtocolKind`. No duplicate config
   model, no `From`-adapter layer, no config drift.
3. **Static enum pipeline, not a registry.** Dispatch by `match` over the
   proto enums. No trait-object protocol registries, no side-effect
   registries (Xray's reflection-based `init()` pattern is an anti-pattern
   here).
4. **Transport OUTERMOST, protocol INNERMOST** (Xray composition order):
   dial → transport upgrade → security wrap → protocol handshake → tunnel.
5. **Stream seams are boxed.** `Box<dyn AsyncRead + AsyncWrite + Unpin +
   Send>` at every layer boundary; concrete types inside modules.
6. **Deadlines on every network step.** `tokio::time::timeout` around dial,
   handshake, and tunnel operations (repo deadline culture).
7. **One match arm per new protocol / transport.** Extensibility = copy a
   neighbor directory, add one match arm, fill vectors.
8. **`crypto/` owns algorithms, layers own decisions.** No I/O policy in
   `crypto/`; packet-level obfuscation lives inside the transport that uses
   it, byte-stream transforms compose as stream wrappers.

## 4. Crate layout

```
crates/xray-tui-native/
  Cargo.toml
  src/
    lib.rs                  pub API: connect(), connect_chain(), NativeTunnel; re-exports
    context.rs              NativeConnectParams, LinkContext (server, target, resolved IP, per-link state)
    addr.rs                 TargetAddr + addr codec (port-first; v2ray AddressType 1/2/3)
    error.rs                NativeError (thiserror)
    chain.rs                connect_chain fold + duplex unit tests
    shape.rs                ConnectShape taxonomy + per-kind classification fn + test
    transport/
      mod.rs                dispatch by TransportConfig variant
      tcp.rs                real: dial (base=None) / identity upgrade (base=Some)
      ws.rs                 stub
      grpc.rs               stub
      kcp.rs                stub
      quic.rs               stub (future: quinn + crypto/salamander)
      httpupgrade.rs        stub
      xhttp.rs              stub
    security/
      mod.rs                dispatch by SecurityConfig variant
      none.rs               real: identity
      tls.rs                real: rustls 0.23 (ring), root store, ALPN from config
      tls_provider.rs       TlsConnector trait; TlsProvider { Standard, Custom(Arc<dyn TlsConnector>) }
      reality.rs            stub: HelloProvisioner contract + NotImplemented connect
    protocol/
      mod.rs                dispatch by ProtocolKind; route via ConnectShape
      vless/
        mod.rs              connect impl (real)
        header.rs           pure wire codec — v2ray port-first, ported vectors
        addons.rs           protobuf addons — framework only, no flow support yet
      vmess/mod.rs          stub (+ body crypto home referenced)
      trojan/mod.rs         stub
      shadowsocks/mod.rs    stub
      ... (one dir per kind; 27 total)
    crypto/
      mod.rs                algorithm registry notes
      kdf.rs                HKDF/SHA helpers (vmess KDF, reality) — vectors
      aead.rs               AES-128/256-GCM + ChaCha20-Poly1305 body codecs — RFC vectors
      salamander.rs         Hysteria2 packet obfuscation — port spec + vectors
      legacy_stream.rs      rc4-md5 / chacha20-ietf / aes-cfb — stub + doc
      fingerprint/
        ja3.rs              offline JA3 encoder + vectors
        ja4.rs              offline JA4 encoder + vectors
  examples/
    fingerprint_check.rs    tier-2 grader tool (fingerprint engine milestone)
  tests/
    e2e/
      harness.rs            spawn_http_echo, spawn_core_server, cert CA, probe_once
      common.rs             core dir/env handling, skip policy
      vless_xray.rs         e2e: native VLESS+TLS+TCP client ↔ xray-core server
      vless_singbox.rs      e2e: native client ↔ sing-box server
      chain_ignored.rs      e2e: SS→VLESS chain (both binaries; #[ignore])
      fingerprint_grader_ignored.rs  (fingerprint engine milestone)
```

One directory per component; a new protocol = `protocol/<kind>/` + one arm in
`protocol/mod.rs` + classify in `shape.rs` + stub or vectors.

## 5. Public API

```rust
pub struct NativeConnectParams {
    pub protocol: ProtocolConfig,      // typed proto config (owned clone at connect boundary)
    pub server: EndpointEssentials,    // dial address (host/port), host-free config mandate holds
    pub target: TargetAddr,            // final destination for single connect
    pub resolved_ip: Option<SocketAddr>, // optional pre-resolved override (enrichment pipeline)
}

pub struct TargetAddr { pub host: Host, pub port: u16 }  // Host: Ip(std::net::IpAddr) | Domain(String)

pub async fn connect(params: NativeConnectParams) -> Result<NativeTunnel, NativeError>;
pub async fn connect_chain(links: &[NativeConnectParams], target: TargetAddr)
    -> Result<NativeTunnel, NativeError>;

pub struct NativeTunnel { /* AsyncRead+AsyncWrite impls, Drop tears down */ }
```

`connect` is the single-link case of `connect_chain`. `NativeTunnel` is the
byte tunnel; consumers (tests today, real-ping later) read/write raw
HTTP/proxied bytes through it.

## 6. ConnectShape taxonomy

Not every kind is a TCP dial + handshake. `shape.rs` classifies statically
(unit-tested):

```rust
pub enum ConnectShape {
    TcpStream,      // transport (tcp/ws/grpc/...) + security + handshake over stream
                    //   vless, vmess, trojan, shadowsocks, socks, http, shadowtls, anytls, naive
    Quic,           // quic transport (+ packet obfuscation) then protocol over quic stream
                    //   tuic, hysteria1, hysteria2
    DeviceTunnel,   // no dial; device + own socket path
                    //   wireguard, tailscale
    HandshakeOwn,   // own state machine, ignores transport seam
                    //   ssh
    NoStream,       // no connect exists; NotImplemented
                    //   redirect, tproxy, mixed, placeholder kinds
}
```

Dispatch rule in `protocol/mod.rs`: match `ConnectShape` first, then
`ProtocolKind`. Each `protocol/<kind>/` owns its `connect()`; stream-family
kinds use the transport seam, `DeviceTunnel` / `HandshakeOwn` ignore it,
`NoStream` returns `NativeError::NotImplemented`. This is documented in
`lib.rs` — the uniform `handshake(stream)` shape is a *family default*, not a
contract.

## 7. Layered pipeline

### Transport phase — `transport/mod.rs`

```rust
impl TransportCall {
    // base: None  -> dial server directly (first hop of a chain)
    // base: Some  -> upgrade the given tunneled stream in place (later hops; e.g. WS handshake)
    async fn connect(ctx: &LinkContext, base: Option<BoxStream>)
        -> Result<BoxStream, NativeError>;
}
```

- `Tcp`: `base=None` → `TcpStream::connect` under timeout; `base=Some` →
  identity (a TCP "transport" over an existing tunnel is raw bytes).
- All other variants: stubs → `NotImplemented` with the module skeleton and
  config plumbing in place.

### Security phase — `security/mod.rs`

Dispatch by `SecurityConfig`:

- `none.rs` — identity.
- `tls.rs` — rustls 0.23 ring `ClientConfig`, root store, ALPN from config;
  no skip-verify.
- `reality.rs` — stub; connect returns `NotImplemented`. Contract in §11;
  provider seam below.

### TLS provider plug — `security/tls_provider.rs`

```rust
pub trait TlsConnector: Send + Sync {
    async fn connect(&self, stream: BoxStream, params: TlsParams)
        -> Result<BoxStream, NativeError>;
}

pub enum TlsProvider { Standard, Custom(Arc<dyn TlsConnector>) }

pub struct TlsParams {
    pub sni: String,
    pub alpn: Vec<String>,
    pub fingerprint: Option<FingerprintId>,
}

pub enum FingerprintId { Chrome, ChromeRandomized, Firefox, Safari, Random, /* … */ }
```

- `Standard` = built-in rustls; ignores `fingerprint` (documented no-op).
- `Custom` = external engine (future: refactored tls-fingerprint, or a rustls
  fork) registered without touching `security/`. The REALITY connector is the
  first `Custom` consumer.
- `TlsParams.fingerprint` derives from the URL `fp` parameter, which remains a
  documented no-op until the engine milestone.

### Protocol phase — `protocol/mod.rs`

Match `ProtocolKind`, route by shape (§6), call the kind module's `connect()`.
Each module owns config plumbing (extract fields from its typed config
variant) and the handshake. Pure wire codecs live in separate files
(`vless/header.rs`) so they are unit-testable against ported vectors without
any I/O.

## 8. Chaining

Client-side chain semantics, parity with xray `proxySettings.socksTag` and
sing-box `detour`: link N's *dial address* is link N+1's server; the traffic
of every later link flows inside the earlier link's tunnel.

```rust
pub async fn connect_chain(links: &[NativeConnectParams], target: TargetAddr) -> Result<NativeTunnel, NativeError>
// links[0] is dialed directly; links[0]'s target = links[1]'s server; …
// last link targets the real destination.
// Client -> SS server -> (SS tunnel) -> VLESS server -> (VLESS tunnel) -> target
//   == connect_chain([ss_params, vless_params], target)
```

Implementation is a ~15-line fold: first hop dials via transport with
`base=None`; each subsequent hop reuses the previous tunnel as `base=Some`.
Unit tests use `tokio::io::duplex` to verify composition order and teardown.
An e2e chain test (SS+VLESS through both real cores) exists as
`tests/e2e/chain_ignored.rs`, `#[ignore]` + feature-gated.

## 9. Crypto

`src/crypto/` holds pure algorithm codecs only — no I/O policy, no transport
logic. Consumers decide *when* an algorithm applies; `crypto/` decides *how*
bytes transform. Every module ships upstream test vectors.

| module | content | consumers |
|---|---|---|
| `kdf.rs` | HKDF/SHA helpers (VMess KDF, REALITY auth) | `protocol/vmess`, `security/reality`, future HPKE |
| `aead.rs` | AES-128/256-GCM, ChaCha20-Poly1305 body codecs | `protocol/vmess`, `protocol/shadowsocks` |
| `salamander.rs` | Hysteria2 packet obfuscation (AES keystream XOR over QUIC packets) | future `transport/quic` (hy2/hy1/tuic) |
| `legacy_stream.rs` | rc4-md5 / chacha20-ietf / aes-cfb (sing-box legacy SS) | `protocol/shadowsocks` — stub + doc |
| `fingerprint/ja3.rs`, `ja4.rs` | offline fingerprint encoders | fingerprint verification tier 1 |

Layer ownership (explicit, keeps the rule honest):

- VMess / Shadowsocks body ciphers stay inside their protocol modules,
  calling `crypto/*` codecs.
- Packet-level obfuscation (Salamander) is NOT a byte-stream seam — it lives
  inside the QUIC transport's packet handling.
- Traffic shaping (Vision padding, mux, mKCP FEC) stays in the owning layer
  (`protocol/vless/`, future `transport/kcp/`), never in `crypto/`.

## 10. TLS and fingerprint mimicry

- Standard TLS: rustls 0.23 ring-only (workspace standard), real chain
  verification. This is the POC path (VLESS + TLS).
- Custom fingerprint: the `TlsConnector` plug (§7). The first consumer is
  REALITY.
- Key synergy: REALITY authenticates the server via the authKey HMAC — it does
  **not** require PKI chain validation. A hand-rolled fingerprint engine (the
  `tls-fingerprint` experiment) can therefore power the REALITY path before it
  is production-grade for standard TLS. Its current gaps (no cert validation,
  aws-lc-rs backend, single-shot test transport) must be addressed before it
  becomes the standard-TLS provider; the ring migration is a hard requirement
  before either use.

## 11. REALITY adoption contract

Modeled on `shoes` (the only Rust reference implementing REALITY; MIT).
Verification basis: `shoes/src/reality/` — `reality_client_connection.rs`
(lines 150–243), `reality_auth.rs`, `reality_client_verify.rs`,
`reality_tls13_messages.rs`, `reality_records.rs`, `reality_tls13_keys.rs`,
`reality_aead.rs`, `reality_cipher_suite.rs`.

### Wire contract (client side)

1. Generate X25519 keypair; `client_random` = 32 random bytes.
2. `shared` = ECDH(client_priv, server_pub from `pbk`).
3. `auth_key` = HKDF-SHA256(shared, salt = `client_random[0..20]`,
   info = `b"REALITY"`).
4. SessionId plaintext (16 B): version (1,8,0) + pad (1) + timestamp
   u32 big-endian (4) + short_id (8).
5. Build ClientHello: our X25519 keyshare, SNI = steal target, cipher suites
   0x1301/0x1302/0x1303, ALPN `["h2","http/1.1"]` (browser-like defaults;
   overridable from config later).
6. AAD = ClientHello with the SessionId field zeroed; nonce =
   `client_random[20..32]`; encrypted SessionId = AES-256-GCM(auth_key, nonce,
   plaintext, aad) → 32 B (16 ciphertext + 16 tag).
7. Restore the encrypted SessionId into the hello (session-id byte range
   returned by the builder — never hardcoded offsets); send; transcript hashes
   the WIRE hello (with ciphertext SessionId).
8. Server auth: extract cert DER from the TLS 1.3 Certificate message; require
   the embedded signature field == `HMAC-SHA512(auth_key, ed25519_public_key)`;
   verify the Ed25519 CertificateVerify signature over the transcript. No PKI
   chain check — authKey HMAC is the trust root.
9. Normal TLS 1.3 key schedule, Finished both ways, then app data through
   record encrypt/decrypt.

### Module map (shoes → ours)

| shoes | ours | change |
|---|---|---|
| `reality_auth.rs` | `security/reality/auth.rs` | ring port: X25519, HKDF-SHA256, AES-256-GCM — 1:1 |
| `reality_client_connection.rs` | `security/reality/connection.rs` | sans-I/O `HandshakeState` + `feed` shape kept |
| `reality_tls13_messages.rs` | `security/reality/messages.rs` | behind `HelloProvisioner`; computed offsets |
| `reality_records/keys/aead.rs` | `security/reality/{records,keys,inner_aead}.rs` | ring-backed; suites 0x1301–03 all ring-supported |
| `reality_client_verify.rs` | `security/reality/verify.rs` | x509-parser + ring Ed25519 + HMAC-SHA512 |
| `reality_cipher_suite.rs` | `security/reality/cipher_suite.rs` | bundle design kept |
| server modules, `reality_certificate.rs` | — | not ported (client-only) |
| SlideBuffer / IoState / ReaderWriter | internal buffered reader | pattern kept |

Ring coverage verified: X25519 agreement, HKDF-SHA256, AES-128/256-GCM,
ChaCha20-Poly1305, SHA-256/384/512 + HMAC, Ed25519 verify. The port needs no
aws-lc-rs.

### Driving requirement: DPI/CDN bypass via fingerprint mimicry

Real-world proxy servers sit behind CDN/DPI (Cloudflare and friends) that
fingerprint ClientHellos and throttle or block non-browser shapes. A stock
rustls hello is detectable, so the native core is only useful against real
servers when its TLS layer can mimic a browser fingerprint. **Fingerprint
mimicry is a core requirement of the crate, not an optional enhancement.**
Consequences:

- The `Custom` (fingerprint-capable) provider is the primary TLS path;
  `Standard` without mimicry is a temporary fallback for local/trusted paths
  (tests, loops) only, and is always the last resort when no engine is
  installed.
- The fingerprint engine must serve **both** standard TLS (with certificate
  validation) and REALITY (authKey auth, no PKI). That makes cert validation
  + full suite coverage mandatory engine requirements, per the limitation
  section below.
- DPI bypass is verified empirically: the engine's acceptance is the tier-2
  grader pass (§12), not just byte-shape conformance.

### rustls ClientHello limitation (hard constraint)

Stock rustls 0.23 (our workspace pin) exposes **no ClientHello modification
API**: no custom extension injection, no extension order/values control, no
fingerprint emission. The `dangerous()` verifier hook affects certificate
validation only. Consequences, adopted into the design:

1. **Standard TLS cannot do fingerprint mimicry.** A profile with `fp`
   set (e.g. `vless://…&fp=chrome`) on the `Standard` provider connects
   undisguised (functional — most servers do not require a specific
   fingerprint; only DPI/CDN inspection cares), and the `fp` parameter stays a
   documented no-op until the engine milestone.
2. **The `Custom` provider is the ONLY fingerprint-capable path** — for both
   REALITY and fingerprinted standard TLS. Its engines therefore need
   certificate validation + full cipher-suite coverage before they may serve
   standard-TLS traffic (REALITY path needs neither, per §11).
3. **No rustls fork in this project.** Forking rustls would diverge the
   workspace-wide pin and add permanent maintenance; the hand-rolled engine
   (tls-fingerprint refactor) already shares the REALITY path's machinery, so
   it is the chosen vehicle.

```rust
pub trait HelloProvisioner: Send + Sync {
    // fingerprint engine lays extensions/GREASE/ALPS/padding per profile;
    // reality connector injects keyshare, SNI, random, encrypted SessionId.
    fn provision(&self, params: &HelloProvisionParams)
        -> Result<ProvisionedHello, NativeError>;
}
pub enum HelloProvisionerChoice { FixedChrome133, Custom(Arc<dyn HelloProvisioner>) }
```

- `FixedChrome133` = shoes' builder shape ported; hello/session-id range
  computed and unit-tested (`session_id range` invariant test).
- `Custom` = future tls-fingerprint profile engine; `TlsParams.fingerprint`
  flows into `HelloProvisionParams`.
- Staging: contract + vector tests now; implementation at the reality
  milestone.

### Known weaknesses in the source material (addressed in ours)

- shoes hardcodes the SessionId byte offset 39 in the handshake message →
  ours computes it.
- shoes fixes ALPN to `["h2","http/1.1"]` → ours reads ALPN from config with
  browser-like defaults.
- shoes ships one Chrome-133-shaped hello → ours has the provisioner seam.
- shoes is aws-lc-rs → ours is ring.
- shoes' fingerprint fidelity is unproven against CDN filtering (its tests
  only compare signature-algorithm bytes) → ours adds the two-tier
  verification of §12.

## 12. Testing strategy

### Unit (offline, no network, always run)

- `addr.rs`: TargetAddr + VLESS addr codec roundtrips (IPv4/domain/IPv6,
  port-first), vectors ported from `v2ray-core`
  `proxy/vless/encoding/encoding_test.go`.
- `protocol/vless/header.rs`: header encode/decode vectors (uuid, command,
  addr types, addons).
- `crypto/kdf.rs`, `crypto/aead.rs`: RFC 8439 / GCM vectors; VMess
  KDF/authid vectors from v2ray `proxy/vmess/encoding/aead/*_test.go`.
- `crypto/salamander.rs`: hysteria port vectors.
- `crypto/fingerprint/ja3.rs` / `ja4.rs`: encode our built hello, compare to
  reference profile values (tier-1 fingerprint tripwire).
- `shape.rs`: classification table test (every `ProtocolKind` → expected
  `ConnectShape`).
- `chain.rs`: fold order + teardown over `tokio::io::duplex`.
- `security/reality/`: auth/message/verify vector tests ported from shoes'
  inline unit tests.
- Error/timeout behavior per module.

### E2E (feature `native-e2e`, needs real core binaries, no external network)

Harness (`tests/e2e/harness.rs`):

- `spawn_http_echo() -> (url, Guard)`: `tiny_http` responder in a dedicated
  thread (tiny_http is synchronous), returns bound 127.0.0.1 address, fixed
  body, HTTP/1.0 tolerated.
- `spawn_core_server(json: &str, core: CoreType) -> (addr, Guard)`: writes
  hand-written inbound JSON to a temp dir, spawns the real binary, waits for
  readiness, kills on drop.
- `probe_once(params) -> StatusAndBody`: connect via native tunnel, write raw
  `GET / HTTP/1.0` bytes, read response.
- Binary sourcing: `XRAY_TUI_CORE_BIN_DIR` env override of
  `dirs::config_dir()`-based lookup (`xray-tui-core::bin_manager::find_binary`);
  binaries absent → tests skip cleanly (warn, not fail). Never auto-download.

Server configs: hand-written inbound JSON constants per (core, protocol) in
`common.rs` — built for this harness, not via `ConfigBuilder` (which is
outbound-shaped). TLS certs: rcgen-generated CA; rustls client trusts the CA
via root store. **No skip-verify on the proxy path.**

Matrix this session: `vless_xray.rs` (xray-core server, VLESS+TLS+TCP inbound)
and `vless_singbox.rs` (sing-box server, same shape). ALPN matched explicitly
on both sides in the config constants. `chain_ignored.rs` (SS→VLESS, both
binaries) and `fingerprint_grader_ignored.rs` (tier 2) are `#[ignore]`.

### Fingerprint verification (two tiers)

- Tier 1 (now): offline JA3/JA4 encoders + vectors against the built hello
  (§12 unit). Catches extension drift on every commit; shared-codebase
  limitation documented.
- Tier 2 (fingerprint engine milestone): `examples/fingerprint_check.rs`
  connects to tls.peet.ws `/api/all` with the provisioned ClientHello and
  prints reported vs expected JA3/JA4/Akamai per profile; gated
  `tests/e2e/fingerprint_grader_ignored.rs` does the same in CI-op fashion
  (retry + graceful skip; grader APIs are third-party, rate-limited, not
  network-free — hence `#[ignore]`).

## 13. Errors and timeouts

Single `NativeError` (thiserror):

```rust
pub enum NativeError {
    Config(String),            // bad/unsupported config combination
    Dial(String),              // server unreachable
    Tls(String),               // rustls / handshake
    Reality(String),          // future
    Protocol { kind: ProtocolKind, detail: String },
    NotImplemented { feature: String },   // placeholder stubs
    Io(std::io::Error),
    Timeout { step: &'static str, limit: Duration },
}
```

Deadlines (constants in `error.rs` or `context.rs`, named): dial 10 s,
transport upgrade 10 s, security handshake 10 s, protocol handshake 10 s,
tunnel idle read 30 s (POC values; tuned later). Every async network call is
wrapped in `tokio::time::timeout`; timeouts map to `NativeError::Timeout`.

## 14. Dependencies

- Runtime: `xray-tui-proto` (path, workspace), `tokio`, `futures-util`,
  `thiserror`, `rustls` 0.23 (workspace pin), `serde` (config plumbing),
  `uuid` (workspace). None of these are new to the workspace.
- Dev: `tiny_http` (harness HTTP responder), `rcgen` (test CA/certs).
- Future (only when their protocols go real): `ring` direct (crypto codecs)
  , `blake3` (SS-2022), `x509-parser` + ed25519 via ring (reality),
  `quinn` (QUIC transports), aead/chacha20 legacy cipher crates (legacy SS).
- No new crypto backend today. POC (VLESS+TLS) needs none beyond workspace
  deps.

## 15. POC acceptance criteria (this session)

1. `cargo build` and `cargo test` green across the workspace, with
   `xray-tui-native` compiling all 27 protocol stubs, all transport stubs,
   all security stubs.
2. Unit vectors pass (addr, vless header, JA3/JA4 encoders, ConnectShape
   table, chain fold).
3. With `native-e2e` + `XRAY_TUI_CORE_BIN_DIR` pointing at real binaries:
   `vless_xray` and `vless_singbox` e2e tests assert HTTP 200 + expected body
   through the native VLESS tunnel.
4. Without binaries: e2e tests skip cleanly; everything else still green.
5. No TUI edits, no ping-pipeline edits, no `ConfigBuilder` reuse for server
   configs, MIT-clean (no GPL/MPL-derived code).

## 16. Milestones

- M0 (this doc): design approved, spec committed.
- M1 (this session): skeleton + ConnectShape + addr/error + VLESS real +
  unit vectors + e2e harness + xray/sing-box e2e + chain unit tests.
- M2: **fingerprint engine** (DPI/CDN priority): tls-fingerprint refactor —
  ring migration, certificate validation, full suite coverage, integration as
  the `Custom` TlsConnector for standard TLS; tier-2 grader acceptance
  (§12/§10 driving requirement). First-class deliverable, not deferrable.
- M3: REALITY ring port + engine-backed provisioner (`FixedChrome133` first)
  + reality e2e against core reality inbounds.
- M4: crypto codecs (aead/kdf/salamander), VMess/SS bodies, legacy stubs.
- M5: chaining e2e (needs real SS client from M4), TUI wiring
  (`RealPingManager`/`CorePool` swap, instant availability checks),
  SSH/WireGuard connect paths.

## 17. Risks

- Hand-rolled TLS 1.3 surface (REALITY only) is security-sensitive; it is
  constrained to the reality path, standard TLS stays on rustls, and every
  crypto module is vector-tested.
- Wire drift across core versions: mitigated by ported vectors + e2e matrix.
- REALITY timestamp auth fails under clock skew — client timestamps are
  server-checked; keep system-clock sanity in diagnostics.
- Grader API flakiness (tier 2): `#[ignore]`, retry, graceful skip.
- `tiny_http` is sync — thread + join guard pattern in harness; revisit if
  transport tests need HTTP/2.
