# Unified TLS / REALITY Client API — Design

Date: 2026-08-12
Branch: `native-core-stub` (HEAD `72185db`)
Status: approved design — pending implementation plan

## Problem

The TLS implementation is "pieces of almost independent code parts": the
fingerprint engine (`xray-tui-tls::handshake::connect`) and the REALITY client
(`xray-tui-tls::reality::connect_reality`) are two separate handshake drivers
that mirror each other (~150 duplicated lines), and the native security phase
(`xray-tui-native::security::wrap`) dispatches three disjoint paths
(standard rustls / fingerprint / REALITY) behind two independent provider
knobs (`TlsProvider` + `reality_provisioner`).

Three concrete gaps:

1. **REALITY cannot mimic any browser**: the `HelloProvisioner` owns its spec
   internally — `FixedChrome133.provision()` hardcodes `profiles::chrome133::spec()`.
2. **`fp` is ignored on REALITY**: `RealityConnector` implements `TlsConnector`
   but drops `TlsParams.fingerprint`.
3. **Two provider concepts, three paths**: `wrap()` has no single config type;
   the native `FingerprintId` (5 coarse ids) is a subset of the engine's 12
   profiles; `RealityConnector`/`FingerprintConnector`/standard-rustls are
   three unrelated connectors.

## Goal

One simple, highly configurable engine-level API that wires fingerprint
mimicry and REALITY together, adopted exactly by `xray-tui-native`. A REALITY
handshake is a *layer of security on top of the standard TLS machinery*,
shaped by any supported browser profile. On a REALITY fallback (real
certificate / transparent proxy), the client adopts xray-core's Spider-X
behavior — keep the established TLS session alive with HTTP/2 GETs to the real
site instead of closing immediately — for DPI-stealth on both sides.

## Decisions (from brainstorming)

- **Engine-level API, native adapts**: the unified config + connect entry live
  in `xray-tui-tls`; `xray-tui-native` becomes a thin translator from the proto
  config. Future engine users (real-ping, tests) get the engine API directly.
- **One entry, config enum dispatch**: `TlsConfig { mode }` where
  `mode: TlsMode::{Plain, Reality}`; `connect()` dispatches internally.
- **Driver unification (Approach B)**: the two mirrored drivers collapse into
  one flight-processing core parameterized by hello source + server auth.
- **Engine-only client**: the standard rustls client path is dropped; TLS 1.2
  support comes to the engine later. Rustls survives only as the server-side
  test double (unit tests + e2e dest).
- **Full `BrowserProfile` enum in the engine config**; the 5-id URL `fp`
  vocabulary stays native-side as a parse layer.
- **`HelloProvisioner` trait kept** as the custom escape hatch; add
  `ProfileProvisioner(BrowserProfile)`; `FixedChrome133` = compat alias.
- **Spider fallback, minimal config**: xray's exact behavior on `!Verified`
  (keep conn, HTTP/2 GETs to the real site, then error to caller); config =
  `paths + max_gets + interval`, xray-shaped defaults for padding cookie /
  referer chain / browser headers as engine constants.
- **Two-servers e2e in this task**: correct / wrong pbk / wrong sid / plain
  probe / reality-into-plain scenarios × xray + sing-box.

## Engine API (`crates/xray-tui-tls`)

### New `client` module

```rust
// client.rs
pub enum TlsMode {
    /// Plain TLS 1.3, fingerprint-shaped hello.
    Plain {
        profile: Option<BrowserProfile>,   // None → Chrome130 (engine default)
        verifier: Arc<dyn ServerVerifier>, // WebPkiVerifier | test seams
    },
    /// REALITY: fingerprint-shaped hello + sealed session id + HMAC/Ed25519
    /// server auth, Spider-X fallback on auth failure.
    Reality {
        provisioner: Arc<dyn HelloProvisioner>, // ProfileProvisioner(profile) default; Custom injected
        public_key: [u8; 32],                   // decoded pbk
        short_id: Vec<u8>,                      // ≤8 bytes
        spider: SpiderConfig,
    },
}

pub struct TlsConfig {
    pub mode: TlsMode,
    pub server_name: String,
    /// Plain: override the spec's ALPN list; Reality: ignored (protocol
    /// forces h2 + http/1.1).
    pub alpn: Option<Vec<Vec<u8>>>,
    pub rng: Arc<dyn SecureRandom>,  // SystemRandom default; tests inject fixed-seed
}

/// Spider-X fallback behavior on a REALITY auth failure.
pub struct SpiderConfig {
    /// Paths from the URL `spx`; default `["/"]`.
    pub paths: Vec<String>,
    /// Bounded spider session (GETs before close); default ~4.
    pub max_gets: usize,
    /// Jittered delay between GETs; default ~0.5–2s.
    pub request_interval: std::time::Duration,
}

pub async fn connect<S: Stream + 'static>(
    stream: S,
    config: &TlsConfig,
) -> Result<TlsStream<S>>;
```

- Constructors `TlsConfig::plain(profile, verifier, sni)` /
  `TlsConfig::reality(provisioner, pbk, sid, sni)` default `rng =
  SystemRandom`.
- `connect` dispatches on `mode`. The `'static` bound on `S` is required by
  the fallback `tokio::spawn` (the stream moves into the spider task);
  `BoxStream` (native) is `'static` — no caller change.

### `reality` module additions

```rust
/// A `HelloProvisioner` shaped by any browser profile.
pub struct ProfileProvisioner(pub BrowserProfile);
impl HelloProvisioner for ProfileProvisioner {
    // profile.spec() + SessionIdSpec::AuthPayload { len: 32 } + build_hello
}
```

`FixedChrome133` stays as a unit struct delegating to
`ProfileProvisioner(BrowserProfile::Chrome133)` — existing call sites
(`&FixedChrome133`) untouched.

### Error variant

```rust
#[error("REALITY: received real certificate (potential MITM or redirection)")]
RealityFallback,
```

Mirrors xray-core's `!Verified` diagnostic. Native maps it to
`NativeError::Reality`.

## Driver unification

`handshake/mod.rs` gains one internal driver; `connect()` and
`connect_reality()` become thin pre-flights.

```rust
pub(crate) enum ServerAuth<'a> {
    Verifier(&'a dyn ServerVerifier),
    Reality { auth_key: &'a [u8; 32] },  // HMAC mismatch → fallback, not error
}

pub(crate) enum AuthOutcome { Ok, RealityFallback }

pub(crate) async fn drive<S>(
    stream: S,
    hello: &[u8],             // full handshake message, sealed if Reality
    offered_session_id: &[u8],
    keypair: X25519KeyPair,   // generated in the entry; pub injected into hello
    auth: ServerAuth<'_>,
) -> Result<(TlsStream<S>, AuthOutcome)>;
```

`drive()` = the shared middle: write hello record → `read_server_hello`
(echo check, CCS skip, HRR reject) → `KeySchedule` + `AeadKey` →
`read_server_hs_messages` → transcript assembly → **auth dispatch** → server
Finished MAC → client Finished → `TlsStream`. The auth dispatch returns
`AuthOutcome`:

- `Verifier` → `VerifyContext` / `verify()`; failure = `Verify` error.
- `Reality { auth_key }` → `verify::verify_server`; on HMAC mismatch return
  `(stream, AuthOutcome::RealityFallback)` — **the TLS session to dest is
  legitimate (full TLS 1.3 handshake completed), so it is completed and
  returned**, not torn down.

Entries:

- `connect()`: keypair gen → `build_hello(spec, …)` → `drive(Verifier)`.
- `connect_reality()`: keypair gen → `provisioner.provision(…)` →
  `client_random` extract → `derive_auth_key` → `seal_and_splice` →
  `drive(Reality { auth_key })`. On `AuthOutcome::RealityFallback`: spawn
  the spider with the returned stream + spider config + SNI, then return
  `Err(TlsError::RealityFallback)` to the caller.

Net: the two mirrored bodies (~150 dup lines) collapse; transcript, HRR,
CCS, flight accumulation live once.

## Spider-X fallback

Adopts xray-core `reality.go:185-243` exactly in shape.

On `AuthOutcome::RealityFallback` (established TLS session to the real site /
dest):

1. `tokio::spawn(spider(stream, spider_cfg, sni))`.
2. Return `Err(TlsError::RealityFallback)` to the caller (xray: error + the
   connection lives on for the spider).

Spider driver (`reality/spider.rs`):

- `http2::Client` — the engine's `http2` module (client preface, SETTINGS,
  HPACK-encoded HEADERS, DATA accumulation, PING/GOAWAY) is single-GET-per-
  connection today. Extend to multi-request on one connection: `new(conn)` =
  preface + SETTINGS, `get(path, host)` = HEADERS on stream id 1, 3, 5, …
  (increment, no re-preface). The existing free `get()` becomes
  `Client::new` + one `get`.
- Loop `max_gets` times: GET `https://<sni><path>` (paths cycled), padding
  cookie of 0–512 zeros (xray `SpiderY[0..1]`), Referer chaining, default
  browser headers (`TryDefaultHeadersWith(…, "nav")` equivalent), jittered
  `request_interval` between GETs. Constants match xray defaults.
- All errors swallowed (background task; the caller already got its error).

`http2::Client` needs `S: 'static` too (spider holds the stream across
GETs) — consistent with the `connect` bound.

## Native adaptation (`crates/xray-tui-native`)

- **Delete**: `security/tls.rs` (rustls client path), `security/tls_provider.rs`
  (`TlsConnector` / `TlsProvider` / `TlsParams`), `FingerprintConnector` and
  `RealityConnector` (the two `TlsConnector` impls). `NativeConnectParams`
  drops `tls_provider`; keeps `reality_provisioner: HelloProvisionerChoice`.
- **`wrap()` rewrite** — both arms build the engine config:
  - `TlsConfig::Tls(opts)` → `TlsMode::Plain { profile, verifier }`:
    `fp` → `parse_fingerprint_id` → `profile_for` (5-id URL vocabulary stays
    in `fingerprint.rs`, maps to the full engine enum; absent fp / Random →
    Chrome130). Verifier = test-CA / webpki-roots / `insecure` / `pin_sha256`
    build (logic moves from `fingerprint.rs`, stays native-side, feeds
    `Arc<WebPkiVerifier>` into the engine).
  - `TlsConfig::Reality(opts)` → `TlsMode::Reality { provisioner, public_key,
    short_id, spider }`: provisioner = `fp` present → `Arc::new(
    ProfileProvisioner(profile))`, else `reality_provisioner.provisioner()`.
    `pbk` / `sid` decode (existing `decode_pbk` / `decode_sid`), `spx` →
    `SpiderConfig { paths: vec![spx], ..default }` (proto `RealityOpts.spx` is
    a single path string, e.g. `/?ed=2048`; absent → default `["/"]`).
- **Cargo**: `tokio-rustls` moves from `[dependencies]` to
  `[dev-dependencies]` + `native-e2e` feature deps (server-side test double
  only). The `rustls` workspace pin and `rcgen` stay for test/e2e servers.
- **NATIVE_CORE.md**: roadmap — engine TLS 1.2 support pending (legacy
  server support); rustls client path removed (server double only);
  Spider-X fallback documented.

## Two-servers e2e

`E2eExpect` gains a connect outcome:

```rust
pub enum ConnectExpect { Ok, ErrRealityFallback }
// E2eExpect { connect: ConnectExpect, status: u16, body: String }
```

Error cases assert the `RealityFallback` error **and** that the dest received
the HTTP/2 client preface bytes (proves the spider ran — the connection was
not immediately closed). `tls_echo` (dest) gains post-handshake byte
recording (first ~64 B per connection); the run lifecycle asserts within a
short timeout for error cases and skips the probe step.

| Case | Server | Client | Expect |
|---|---|---|---|
| R1 correct | `vless_reality_inbound` (real key) | Reality, correct pbk/sid | Ok, probe ok (existing `RealityTls` — keep) |
| R2 wrong pbk | same | Reality, wrong pbk (valid 32 B) | `ErrRealityFallback` + dest saw preface |
| R3 plain probe | same | Plain (chrome fp) | Ok — stealth transparent TLS to dest, probe = dest's response |
| R4 wrong sid | same | Reality, wrong sid | `ErrRealityFallback` + dest saw preface |
| R5 reality→plain | `vless_inbound`, standard TLS (harness CA cert) | Reality | `ErrRealityFallback` + dest saw preface |

Each case × xray 26.3.27 + sing-box 1.13.16.

R3 detail: a Plain-mode client through a REALITY server is transparently
proxied to dest; it completes standard TLS to `tls_echo` (SNI `localhost`,
harness-CA cert) and probes — the probe destination is `tls_echo` (its
response), not the tiny_http echo.

## Verification

- **Migrate, don't weaken**: all existing engine goldens (RFC 8448 vectors,
  JA3/JA4, VMess Go vectors) + native 110 tests stay green through the driver
  merge (pure extraction, no behavior change). Existing standard-rustls
  security tests migrate to the engine Plain path (profile None → Chrome130).
- **New unit tests**:
  - `ProfileProvisioner(Chrome133)` hello ≡ `FixedChrome133` golden equality.
  - Reality → rustls test server fallback: `RealityFallback` error returned
    AND the server received the h2 preface bytes (recording rustls server).
  - `http2::Client` multi-request: two GETs on one connection, stream ids
    increment, no re-preface.
  - Deterministic-rng goldens unchanged (fixed-seed `TlsConfig`).
- **Full**: workspace `cargo test`, clippy 0 warnings, fmt, e2e sweep ×2 cores
  (`XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin`).

## Out of scope

- TLS 1.2 engine support (future task; legacy servers break until then).
- Full SpiderY config surface (cookie-padding range, per-iteration request
  range, timing range exposed as config fields) — engine constants now.
- Spider traffic verification against a real HTTP/2 dest (the e2e dest is an
  HTTP/1.1-ish sink; the client-side preface-arrival assertion is the
  contract).
- The `spx` value already flows through `RealityOpts.spx` in the proto —
  no proto changes needed.

## Files touched

Engine (`crates/xray-tui-tls`): `src/client.rs` (new), `src/lib.rs`,
`src/handshake/mod.rs` (driver extraction), `src/reality/mod.rs`
(`ProfileProvisioner`, slim `connect_reality`), `src/reality/spider.rs` (new),
`src/http2/mod.rs` (multi-request `Client`), `src/error.rs`
(`RealityFallback`).

Native (`crates/xray-tui-native`): `src/security/mod.rs` (`wrap` rewrite),
`src/security/fingerprint.rs` (drop `FingerprintConnector`, keep parsers +
verifier build), `src/security/reality.rs` (drop `connect`/`RealityConnector`,
keep choice + decoders), delete `src/security/tls.rs` + `tls_provider.rs`,
`src/context.rs` (drop `tls_provider`), `src/e2e/*` (variant/case/harness/
config), `Cargo.toml` (tokio-rustls → dev/e2e), `NATIVE_CORE.md` (roadmap).
