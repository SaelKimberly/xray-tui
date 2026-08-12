# Unified TLS / REALITY Client API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire fingerprint-mimicry TLS and REALITY into one engine-level `TlsConfig`/`connect()` API (REALITY = a security layer over the shared TLS driver), adopted by `xray-tui-native`, with xray-core's Spider-X fallback and a dest/target two-servers e2e.

**Architecture:** Collapse the two mirrored handshake drivers (`handshake::connect` + `reality::connect_reality`) into one `drive()` core parameterized by hello source + a `ServerAuth` enum. Add `ProfileProvisioner(BrowserProfile)` so REALITY mimics any of the 12 browser profiles. On REALITY auth failure (real certificate / transparent proxy), keep the established TLS session alive with Spider-X HTTP/2 GETs (xray `reality.go:185` semantics) and return a specific `RealityFallback` error. Expose `TlsConfig { mode: Plain|Reality }` in a new `client` module; native drops the rustls client path and maps the proto config onto it.

**Tech Stack:** Rust 2024, ring 0.17.14 (+ x25519-dalek 3 for REALITY's dual agreement), tokio, xray-tui-tls engine, xray-tui-native, e2e vs xray 26.3.27 + sing-box 1.13.16.

## Global Constraints

- ring only in `xray-tui-tls`: no aws-lc-rs, no `rand`, no `unsafe`. Crate-local `SecureRandom` seam is the ONLY random interface — the public entries take `&dyn SecureRandom` (crate trait), never ring's sealed trait object.
- Workspace lints clippy pedantic+nursery `warn` — **ZERO warnings** enforced. Edition 2024, rustfmt, thiserror.
- Every task compiles (`cargo check -p xray-tui-tls` / `-p xray-tui-native`) and its own tests pass before commit. The binary crate is untouched.
- Driver extraction is **behavior-preserving** for the success paths: RFC 8448 vectors, JA3/JA4 goldens, and VMess Go vectors must stay green unchanged.
- No implicit network in usual tests (e2e is feature + env-gated `XRAY_TUI_CORE_BIN_DIR`, hard-fail absent).
- `xray-tui-proto` is untouched.

---

### Task 1: `TlsError::RealityFallback` + `ProfileProvisioner`

**Files:**
- Modify: `crates/xray-tui-tls/src/error.rs`
- Modify: `crates/xray-tui-tls/src/reality/mod.rs`
- Test: `crates/xray-tui-tls/src/reality/mod.rs` (`mod tests`)

**Interfaces:**
- Consumes: `crate::profiles::BrowserProfile` (already `pub`, has `spec() -> ClientHelloSpec`); `crate::hello::{build_hello, BuildParams}`; `crate::spec::SessionIdSpec`; `crate::SecureRandom`.
- Produces:
  - `TlsError::RealityFallback` (variant) with Display `"REALITY: received real certificate (potential MITM or redirection)"`.
  - `pub struct ProfileProvisioner(pub BrowserProfile)` implementing `HelloProvisioner`.
  - `FixedChrome133` now delegates to `ProfileProvisioner(BrowserProfile::Chrome133)` (same public shape — `struct FixedChrome133;` usable as `&FixedChrome133`).

- [ ] **Step 1: Add the error variant**

In `crates/xray-tui-tls/src/error.rs`, add to the `TlsError` enum:

```rust
/// REALITY was expected but the server flight was not REALITY-authenticated
/// — a real certificate (transparent proxy / possible MITM or redirection).
#[error("REALITY: received real certificate (potential MITM or redirection)")]
RealityFallback,
```

- [ ] **Step 2: Add `ProfileProvisioner` and re-point `FixedChrome133`**

In `crates/xray-tui-tls/src/reality/mod.rs`, after the `FixedChrome133` struct, add:

```rust
/// A `HelloProvisioner` shaped by any browser profile: the profile's
/// `ClientHello` spec with the REALITY `AuthPayload` session id slot.
pub struct ProfileProvisioner(pub BrowserProfile);

impl HelloProvisioner for ProfileProvisioner {
    fn provision(&self, params: &HelloProvisionParams<'_>) -> Result<ProvisionedHello> {
        let mut spec = self.0.spec();
        spec.session_id = SessionIdSpec::AuthPayload { len: 32 };
        let built = build_hello(
            &spec,
            &BuildParams {
                server_name: params.server_name,
                alpn: params.alpn,
                x25519_pub: params.x25519_pub,
                rng: params.rng,
            },
        )?;
        let session_id_range = built.session_id_range.ok_or_else(|| {
            TlsError::Spec("profile spec must use SessionIdSpec::AuthPayload".into())
        })?;
        Ok(ProvisionedHello {
            handshake_bytes: built.handshake_bytes,
            session_id_range,
        })
    }
}
```

Replace the body of `impl HelloProvisioner for FixedChrome133` so it delegates:

```rust
impl HelloProvisioner for FixedChrome133 {
    fn provision(&self, params: &HelloProvisionParams<'_>) -> Result<ProvisionedHello> {
        ProfileProvisioner(BrowserProfile::Chrome133).provision(params)
    }
}
```

`crate::profiles::BrowserProfile` must be imported in `reality/mod.rs` (add `use crate::profiles::BrowserProfile;`).

- [ ] **Step 3: Write the golden-equality test**

In `crates/xray-tui-tls/src/reality/mod.rs` `mod tests`, add:

```rust
#[test]
fn profile_provisioner_chrome133_matches_fixed_chrome133() {
    let rng = ring::rand::SystemRandom::new();
    let params = || HelloProvisionParams {
        server_name: "www.microsoft.com",
        alpn: Some(REALITY_ALPN),
        x25519_pub: &[0xAB; 32],
        rng: &rng,
    };
    let fixed = FixedChrome133.provision(&params()).unwrap();
    let profile = ProfileProvisioner(BrowserProfile::Chrome133)
        .provision(&params())
        .unwrap();
    assert_eq!(fixed.handshake_bytes, profile.handshake_bytes);
    assert_eq!(fixed.session_id_range, profile.session_id_range);
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p xray-tui-tls reality::tests`
Expected: PASS (new test + all existing reality tests, including `fixed_chrome133_provisioner_builds_auth_payload_hello`).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls/src/error.rs crates/xray-tui-tls/src/reality/mod.rs
git commit -m "feat(tls): add ProfileProvisioner + TlsError::RealityFallback"
```

---

### Task 2: `http2::Client` multi-request + extra headers

**Files:**
- Modify: `crates/xray-tui-tls/src/http2/mod.rs`
- Test: `crates/xray-tui-tls/src/http2/mod.rs` (`mod tests`)

**Interfaces:**
- Consumes: existing `make_frame`, `read_frame`, `hpack_string`, `encode_get_headers`, `PREFACE`, `FRAME_*`/`FLAG_*` consts.
- Produces:
  - `pub struct Client { next_stream: u32 }` with `pub fn new() -> Self` and
    `pub async fn get<S: AsyncRead + AsyncWrite + Unpin + Send>(&mut self, conn: &mut TlsStream<S>, path: &str, host: &str, extra_headers: &[(&str, &str)]) -> Result<String>`.
  - The existing free `pub async fn get<S>(conn, path, host) -> Result<String>` stays, delegating to `Client::new().get(conn, path, host, &[])`.

- [ ] **Step 1: Add a multi-request `Client`**

Restructure `crates/xray-tui-tls/src/http2/mod.rs`. The current free `get` sends the preface + SETTINGS + HEADERS on stream 1 and reads to `END_STREAM`. Extract a `Client` that owns the stream id and only sends the preface on the first request:

```rust
/// A minimal HTTP/2 client capable of several GETs on one connection.
///
/// The connection preface + SETTINGS exchange run once (first `get`);
/// subsequent GETs reuse the connection on stream ids 1, 3, 5, … (RFC 7540
/// §5.1.1 — client-initiated streams are odd and monotonically increasing).
pub struct Client {
    next_stream: u32,
}

impl Client {
    #[must_use]
    pub fn new() -> Self {
        Self { next_stream: 1 }
    }

    pub async fn get<S: AsyncRead + AsyncWrite + Unpin + Send>(
        &mut self,
        conn: &mut TlsStream<S>,
        path: &str,
        host: &str,
        extra_headers: &[(&str, &str)],
    ) -> Result<String> {
        if self.next_stream == 1 {
            // First request: client preface + empty SETTINGS (RFC 7540 §3.5).
            conn.write_all(PREFACE).await?;
            conn.write_all(&make_frame(FRAME_SETTINGS, 0, 0, &[])).await?;
        }
        let mut hpack = encode_get_headers(path, host);
        for (name, value) in extra_headers {
            encode_literal_header(&mut hpack, name, value.as_bytes());
        }
        let stream_id = self.next_stream;
        self.next_stream += 2;
        conn.write_all(&make_frame(FRAME_HEADERS, FLAG_END_HEADERS | FLAG_END_STREAM, stream_id, &hpack)).await?;
        receive_response(conn, stream_id).await
    }
}
```

Add a literal-with-new-name HPACK encoder (RFC 7541 §6.2.2 — literal header field without indexing, new name): prefix `0x00` + name length + name + value length + value (raw, non-Huffman, matching `hpack_string`):

```rust
/// Encodes a literal header field *without indexing, new name* (RFC 7541
/// §6.2.2): `0x00` prefix, then name and value as raw length-prefixed
/// strings. Used for `cookie` and `referer` in the spider's extra headers.
fn encode_literal_header(buf: &mut Vec<u8>, name: &str, value: &[u8]) {
    buf.push(0x00);
    hpack_string(buf, name.as_bytes());
    hpack_string(buf, value);
}
```

The shared response reader must take the stream id (currently hardcodes stream 1):

```rust
async fn receive_response<S: AsyncRead + AsyncWrite + Unpin + Send>(
    conn: &mut TlsStream<S>,
    stream_id: u32,
) -> Result<Vec<u8>>
```

Inside, `DATA`/`HEADERS` handling checks `frame.2 == stream_id` instead of `== 1`. Keep the free `get` for the grader:

```rust
pub async fn get<S: AsyncRead + AsyncWrite + Unpin + Send>(
    conn: &mut TlsStream<S>,
    path: &str,
    host: &str,
) -> Result<String> {
    Client::new().get(conn, path, host, &[]).await
}
```

`receive_response` is currently `async fn` (private) — make it take the stream id param. `encode_get_headers` already emits the static-table `:path`/`:authority` pseudo-headers; the extra headers append after them.

- [ ] **Step 2: Write the multi-request test**

In `mod tests` add a test against a recording rustls TLS server (mirror the existing http2 test server if present; otherwise build one: a `rustls::ServerConfig` with a generated cert that loops reading raw bytes from each accepted TLS conn and stores them in an `Arc<Mutex<Vec<u8>>>`, appended on read). The test asserts the wire has the preface exactly once and two HEADERS frames on stream ids 1 and 3:

```rust
#[tokio::test]
async fn client_reuses_connection_across_gets() {
    // recording rustls server (see Task 2 note below); ALPN "h2" optional.
    let recorded: Arc<Mutex<Vec<u8>>> = Default::default();
    let server = spawn_recording_tls_server(recorded.clone()).await;
    // client: xray-tui-tls handshake::connect with a chrome spec + AcceptAll
    // verifier against the server's cert (insecure), then:
    let mut conn = /* TlsStream from handshake::connect */;
    let mut client = http2::Client::new();
    let _ = client.get(&mut conn, "/a", "localhost", &[("cookie", "padding=0")]).await;
    let _ = client.get(&mut conn, "/b", "localhost", &[]).await;

    let bytes = recorded.lock().unwrap().clone();
    let preface_count = bytes.windows(PREFACE.len()).filter(|w| *w == PREFACE).count();
    assert_eq!(preface_count, 1, "preface must be sent exactly once");
    // two HEADERS frames (type 0x01), stream ids 1 and 3
    assert!(bytes.windows(9).any(|w| w[0] == FRAME_HEADERS && u32::from_be_bytes(w[5..9].try_into().unwrap()) == 1));
    assert!(bytes.windows(9).any(|w| w[0] == FRAME_HEADERS && u32::from_be_bytes(w[5..9].try_into().unwrap()) == 3));
}
```

Test helper `spawn_recording_tls_server`: reuse the rcgen CA/server pattern from `crates/xray-tui-tls/src/verify/mod.rs` tests or `crates/xray-tui-native/src/e2e/harness.rs` (`tls_server_config` + accept loop). The rustls server must be a dev-dependency of `xray-tui-tls` if not already present (check `crates/xray-tui-tls/Cargo.toml`; add `rustls = { workspace = true, features = ["ring"] }` + `rcgen` under `[dev-dependencies]` if absent).

- [ ] **Step 3: Run tests**

Run: `cargo test -p xray-tui-tls http2`
Expected: PASS (new multi-request test + existing http2 tests).

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-tls/src/http2/mod.rs crates/xray-tui-tls/Cargo.toml
git commit -m "feat(tls): multi-request http2 Client for the spider"
```

---

### Task 3: Driver unification — one `drive()`, both callers, `AuthOutcome`

**Files:**
- Modify: `crates/xray-tui-tls/src/handshake/mod.rs`
- Modify: `crates/xray-tui-tls/src/reality/mod.rs`
- Modify: `crates/xray-tui-tls/src/crypto/mod.rs` (`X25519KeyPair::generate` rng seam)
- Test: `crates/xray-tui-tls/src/handshake/mod.rs`, `src/reality/mod.rs`, `src/verify/mod.rs` (existing goldens)

**Interfaces:**
- Consumes: existing `read_server_hello`, `read_server_hs_messages`, `KeySchedule`, `AeadKey`, `make_hs_msg`, `to_record`, `VerifyContext`, `ServerVerifier`, `reality::verify::verify_server`, `X25519KeyPair`, `CipherSuiteId::digest`.
- Produces:
  - `pub(crate) enum ServerAuth<'a> { Verifier(&'a dyn ServerVerifier), Reality { auth_key: &'a [u8; 32] } }`
  - `pub(crate) enum AuthOutcome { Ok, RealityFallback }`
  - `pub(crate) async fn drive<S: AsyncRead + AsyncWrite + Unpin + Send>(stream: S, hello: &[u8], offered_session_id: &[u8], keypair: X25519KeyPair, server_name: &str, auth: ServerAuth<'_>) -> Result<(TlsStream<S>, AuthOutcome)>`
  - `handshake::connect` and `reality::connect_reality` now take `rng: &dyn crate::SecureRandom` (the crate seam, NOT ring's sealed trait).
  - `reality::RealityParams` drops `rng`'s ring-trait object type → `pub rng: &'a dyn crate::SecureRandom`; gains `pub spider: &'a crate::reality::SpiderConfig` (introduced here with a `Default` impl).

- [ ] **Step 1: Switch `X25519KeyPair::generate` to the crate seam**

In `crates/xray-tui-tls/src/crypto/mod.rs`, change `X25519KeyPair::generate` to take `&dyn crate::SecureRandom` (it draws 32 random bytes):

```rust
pub fn generate(rng: &dyn crate::SecureRandom) -> Result<Self> {
    let mut seed = [0u8; 32];
    rng.fill(&mut seed)?;   // ring::error::Unspecified -> TlsError::Crypto
    // ...existing StaticSecret::from(seed) construction...
}
```

Keep `public_key()` and `agree()` unchanged.

- [ ] **Step 2: Extract `drive()` in `handshake/mod.rs`**

Add the two enums and the driver. The driver is the shared middle of the current `connect` and `connect_reality` bodies (steps: write hello record → read server hello → agree → key schedule → flight → transcript → **auth dispatch** → server Finished MAC → client Finished → app keys → `TlsStream`). The `transcript` Vec is only consumed by the auth step; do not extend it after auth (the key schedule uses `ks.add_transcript`, not `transcript`).

```rust
pub(crate) enum ServerAuth<'a> {
    Verifier(&'a dyn ServerVerifier),
    Reality { auth_key: &'a [u8; 32] },
}

pub(crate) enum AuthOutcome { Ok, RealityFallback }

pub(crate) async fn drive<S: AsyncRead + AsyncWrite + Unpin + Send>(
    stream: S,
    hello: &[u8],
    offered_session_id: &[u8],
    keypair: X25519KeyPair,
    server_name: &str,
    auth: ServerAuth<'_>,
) -> Result<(TlsStream<S>, AuthOutcome)> {
    let mut stream = stream;
    stream.write_all(&to_record(hello)).await?;

    let server_hello = read_server_hello(&mut stream, offered_session_id).await?;
    let shared = keypair.agree(&server_hello.peer_key)?;

    let mut ks = KeySchedule::new(server_hello.suite);
    ks.add_transcript(hello);
    ks.add_transcript(&server_hello.raw);
    let hs_secret = ks.handshake_secret(&shared)?;
    let (client_hs_ts, server_hs_ts) = ks.handshake_traffic_secrets(&hs_secret)?;
    let server_hs_key = AeadKey::new(server_hello.suite, &server_hs_ts)?;
    let client_hs_key = AeadKey::new(server_hello.suite, &client_hs_ts)?;

    let flight = read_server_hs_messages(&mut stream, &server_hs_key).await?;

    let mut transcript = Vec::with_capacity(
        hello.len() + server_hello.raw.len() + flight.ee_raw.len() + flight.cert_raw.len(),
    );
    transcript.extend_from_slice(hello);
    transcript.extend_from_slice(&server_hello.raw);
    transcript.extend_from_slice(&flight.ee_raw);
    transcript.extend_from_slice(&flight.cert_raw);

    let outcome = match auth {
        ServerAuth::Verifier(verifier) => {
            verifier.verify(&VerifyContext {
                chain: &flight.chain,
                sni: server_name,
                signature_scheme: flight.signature_scheme,
                cert_verify_body: &flight.cert_verify_body,
                transcript: &transcript,
                suite: server_hello.suite,
            })?;
            AuthOutcome::Ok
        }
        ServerAuth::Reality { auth_key } => {
            let Some(cert_der) = flight.chain.first() else {
                return Err(TlsError::Verify("REALITY server sent no certificate".into()));
            };
            match crate::reality::verify::verify_server(
                cert_der,
                &flight.cv_raw,
                auth_key,
                &transcript,
                server_hello.suite.digest(),
            ) {
                Ok(()) => AuthOutcome::Ok,
                // Any REALITY auth failure = a real certificate (transparent
                // proxy / possible MITM or redirection), matching xray's
                // `!Verified` handling.
                Err(_) => AuthOutcome::RealityFallback,
            }
        }
    };

    ks.add_transcript(&flight.ee_raw);
    ks.add_transcript(&flight.cert_raw);
    ks.add_transcript(&flight.cv_raw);
    let server_finished_key = ks.finished_key(&server_hs_ts)?;
    if ks.finished_mac(&server_finished_key) != flight.sf_verify_data {
        return Err(TlsError::Handshake(
            "server Finished MAC mismatch — possible MITM or wrong key".into(),
        ));
    }

    let sf_raw = make_hs_msg(HS_FINISHED, &flight.sf_verify_data);
    ks.add_transcript(&sf_raw);
    let client_finished_key = ks.finished_key(&client_hs_ts)?;
    let client_finished_mac = ks.finished_mac(&client_finished_key);
    let cf_hs_msg = make_hs_msg(HS_FINISHED, &client_finished_mac);
    let mut cf_inner = cf_hs_msg.clone();
    cf_inner.push(CONTENT_HANDSHAKE);
    let cf_ciphertext = client_hs_key.seal(0, &aead_aad(cf_inner.len() + AEAD_TAG_LEN), &cf_inner)?;
    stream.write_all(&make_app_data_record(&cf_ciphertext)).await?;

    let master = ks.master_secret(&hs_secret)?;
    let (client_app_ts, server_app_ts) = ks.app_traffic_secrets(&master)?;
    let client_app_key = AeadKey::new(server_hello.suite, &client_app_ts)?;
    let server_app_key = AeadKey::new(server_hello.suite, &server_app_ts)?;
    ks.add_transcript(&cf_hs_msg);

    let tls = TlsStream::new(
        stream,
        AppKeys {
            read_key: server_app_key,
            write_key: client_app_key,
            read_seq: 0,
            write_seq: 0,
        },
    );
    Ok((tls, outcome))
}
```

- [ ] **Step 3: Slim `handshake::connect` onto `drive()`**

Replace the body of `connect` (keep the `HandshakeParams` struct; change `rng` to `&'a dyn crate::SecureRandom`):

```rust
pub async fn connect<S: AsyncRead + AsyncWrite + Unpin + Send>(
    stream: S,
    params: HandshakeParams<'_>,
) -> Result<TlsStream<S>> {
    let keypair = X25519KeyPair::generate(params.rng)?;
    let x25519_pub = keypair.public_key();
    let hello = build_hello(
        params.spec,
        &BuildParams {
            server_name: params.server_name,
            alpn: params.alpn,
            x25519_pub: &x25519_pub,
            rng: params.rng,
        },
    )?;
    let (tls, _) = drive(
        stream,
        &hello.handshake_bytes,
        hello.session_id(),
        keypair,
        params.server_name,
        ServerAuth::Verifier(params.verifier),
    )
    .await?;
    Ok(tls)
}
```

Delete the now-unused `RingRng` adapter (build_hello now receives the crate-seam rng directly) and the `to_record`-vs-`record_bytes` special case (the driver writes `to_record(hello)`, identical to `hello.record_bytes`). Update `HandshakeParams.rng`'s doc comment to reference the crate seam.

- [ ] **Step 4: Slim `reality::connect_reality` onto `drive()`**

In `reality/mod.rs`, change `RealityParams.rng` to `&'a dyn crate::SecureRandom` and add `pub spider: &'a SpiderConfig` with a `Default`:

```rust
#[derive(Clone)]
pub struct SpiderConfig {
    /// Paths from the URL `spx`; default `["/"]`.
    pub paths: Vec<String>,
    /// Bounded spider session (GETs before close); default 4.
    pub max_gets: usize,
    /// Delay between GETs; default 1s.
    pub request_interval: std::time::Duration,
}

impl Default for SpiderConfig {
    fn default() -> Self {
        Self { paths: vec!["/".to_string()], max_gets: 4, request_interval: std::time::Duration::from_secs(1) }
    }
}
```

Replace `connect_reality`'s body after the seal step:

```rust
let (tls, outcome) = drive(
    stream,
    &hello.handshake_bytes,
    hello.session_id(),
    keypair,
    params.server_name,
    ServerAuth::Reality { auth_key: &auth_key },
)
.await?;
match outcome {
    AuthOutcome::Ok => Ok(tls),
    AuthOutcome::RealityFallback => Err(TlsError::RealityFallback),
}
```

(the pre-flight: keypair gen, provision, `extract_client_random`, `build_session_id_plaintext`, `derive_auth_key`, `seal_and_splice` stays exactly as-is; delete the duplicated post-seal driver steps 7–13.)

Remove the now-unused `RingRng` use in `connect_reality` (pass `params.rng` directly to the provisioner). Update the native caller in `crates/xray-tui-native/src/security/reality.rs` `run_handshake` minimally so the crate compiles: pass `&SpiderConfig::default()` in `RealityParams` (spx wiring lands in Task 6):

```rust
RealityParams {
    server_name,
    public_key,
    short_id,
    provisioner,
    rng: &rng,          // `ring::rand::SystemRandom` coerces to &dyn crate::SecureRandom
    spider: &SpiderConfig::default(),
}
```

`crate::security::reality::SpiderConfig` re-export: native's `reality.rs` has `pub use xray_tui_tls::reality::{...}` — add `SpiderConfig` to that re-export list.

- [ ] **Step 5: Run all engine + native tests (goldens must stay green)**

Run: `cargo test -p xray-tui-tls -p xray-tui-native`
Expected: ALL PASS — RFC 8448 vectors, JA3/JA4 goldens, VMess Go vectors, reality success tests unchanged. The `RealityFallback` path is new behavior: existing `reality_handshake_errors_are_reality_errors` (native) now surfaces `RealityFallback` (still a Reality-family error — that test must keep passing; if it asserted a specific message, update the expected variant to `RealityFallback`).

- [ ] **Step 6: Add a fallback-detection unit test**

In `crates/xray-tui-tls/src/reality/mod.rs` tests, add a test proving a REALITY-mode client against a plain rustls server (which echoes the client's session id per RFC 8446 §4.1.3 and returns a real cert) yields `TlsError::RealityFallback`:

```rust
#[tokio::test]
async fn reality_against_plain_tls_server_is_fallback() {
    // recording rustls server (reuse Task 2 helper or a local one) with a
    // generated cert; no REALITY on the server side.
    let (server, addr) = spawn_plain_tls_server().await;
    // client: connect_reality with FixedChrome133 provisioner + a 32-byte
    // public_key + empty short_id + SpiderConfig::default() + SystemRandom.
    let tcp = tokio::net::TcpStream::connect(addr).await.unwrap();
    let err = connect_reality(tcp, RealityParams { /* ... */ spider: &SpiderConfig::default() })
        .await
        .unwrap_err();
    assert!(matches!(err, TlsError::RealityFallback));
    drop(server);
}
```

`spawn_plain_tls_server`: a rustls server (rcgen cert, SAN localhost) that completes the handshake then reads/drains bytes. The client's sealed session id is echoed (rustls echoes legacy session id) so the handshake proceeds to the CertificateVerify, where the HMAC check fails → `RealityFallback`.

- [ ] **Step 7: Commit**

```bash
git add crates/xray-tui-tls/src/handshake/mod.rs crates/xray-tui-tls/src/reality/mod.rs crates/xray-tui-tls/src/crypto/mod.rs crates/xray-tui-native/src/security/reality.rs
git commit -m "refactor(tls): unify handshake drivers behind drive() + AuthOutcome"
```

---

### Task 4: Spider-X fallback — keep the connection alive with h2 GETs

**Files:**
- Create: `crates/xray-tui-tls/src/reality/spider.rs`
- Modify: `crates/xray-tui-tls/src/reality/mod.rs` (`connect_reality` fallback arm; re-export `SpiderConfig`/`Spider`); `crates/xray-tui-tls/src/lib.rs` (`pub mod` wiring)
- Test: `crates/xray-tui-tls/src/reality/mod.rs` (extend the Task 3 fallback test)

**Interfaces:**
- Consumes: `http2::Client` (Task 2), `TlsStream<S>`, `crate::Stream`.
- Produces:
  - `pub(crate) async fn run<S: Stream + 'static>(conn: TlsStream<S>, spider: SpiderConfig, sni: String)` — the bounded spider session.
  - `connect_reality` gains `S: 'static` and, on `AuthOutcome::RealityFallback`, spawns `tokio::spawn(run(tls, spider.clone(), server_name.to_string()))` then returns `Err(TlsError::RealityFallback)`.

- [ ] **Step 1: Write the spider driver**

Create `crates/xray-tui-tls/src/reality/spider.rs`:

```rust
//! Spider-X: after a REALITY fallback (real certificate / transparent
//! proxy), keep the established TLS session alive with HTTP/2 GETs to the
//! real site — xray-core `reality.go` `!Verified` behavior — so a DPI
//! observer sees a browsing session instead of an abrupt close.

use std::time::Duration;

use crate::http2;
use crate::record::stream::TlsStream;
use crate::reality::SpiderConfig;
use crate::Stream;

/// Padding cookie zeros (xray `SpiderY[0..1]`).
const PADDING_MAX: usize = 512;

/// Bounded Spider-X session: `max_gets` HTTP/2 GETs to `https://<sni><path>`
/// with a padding cookie, Referer chaining, and jittered delays. All errors
/// are swallowed — the caller already received its `RealityFallback` error
/// and this task owns the connection.
pub(crate) async fn run<S: Stream + 'static>(
    conn: TlsStream<S>,
    spider: SpiderConfig,
    sni: String,
) {
    let mut conn = conn;
    let mut client = http2::Client::new();
    let mut idx = 0usize;
    let rng = ring::rand::SystemRandom::new();
    let mut prev_path: Option<String> = None;
    for _ in 0..spider.max_gets {
        let path = &spider.paths[idx % spider.paths.len()];
        idx += 1;
        let mut extra: Vec<(&str, String)> = Vec::new();
        // Padding cookie: `padding=0…0` (xray SpiderY, 0..=512 zeros).
        let pad = usize::try_from(crate::crypto::fingerprint::rand_u64(&rng) % (PADDING_MAX as u64 + 1)).unwrap_or(0);
        extra.push(("cookie", format!("padding={}", "0".repeat(pad))));
        // Referer chain: each later request refers to the previous path.
        if let Some(prev) = &prev_path {
            extra.push(("referer", format!("https://{sni}{prev}")));
        }
        let extra_refs: Vec<(&str, &str)> = extra.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let result = client
            .get(&mut conn, path, &sni, &extra_refs)
            .await;
        prev_path = Some(path.clone());
        if result.is_err() {
            break;
        }
        tokio::time::sleep(spider.request_interval).await;
    }
}
```

Add `pub(crate) fn rand_u64(rng: &dyn crate::SecureRandom) -> u64` to `crates/xray-tui-tls/src/crypto/fingerprint/mod.rs` (draw 8 bytes, little-endian) if it does not already exist — check first; a `rand_*` helper may already be there. If it exists, reuse it.

- [ ] **Step 2: Wire the fallback arm + `'static` bound**

In `reality/mod.rs`:

```rust
pub async fn connect_reality<S: AsyncRead + AsyncWrite + Unpin + Send + 'static>(
    stream: S,
    params: RealityParams<'_>,
) -> Result<TlsStream<S>> {
    // ...pre-flight unchanged...
    let (tls, outcome) = drive(/* ... */).await?;
    match outcome {
        AuthOutcome::Ok => Ok(tls),
        AuthOutcome::RealityFallback => {
            // The TLS session to the real site is established and usable —
            // hand it to the Spider-X task so the connection looks like a
            // browsing session, then report the fallback to the caller.
            let spider = params.spider.clone();
            let sni = params.server_name.to_string();
            tokio::spawn(spider::run(tls, spider, sni));
            Err(TlsError::RealityFallback)
        }
    }
}
```

`params.spider` is `&SpiderConfig` — `SpiderConfig` must be `Clone` (already derived in Task 3 Step 4). Add `mod spider;` + `pub(crate) use`/`pub use` in `reality/mod.rs` and `crates/xray-tui-tls/src/lib.rs` (`pub mod reality;` already exists — add `pub mod spider;` or keep it `pub(crate)`; it can be private since only `connect_reality` uses it).

- [ ] **Step 3: Extend the fallback test to assert the spider ran**

Extend the Task 3 fallback test's server to record post-handshake bytes (reuse the Task 2 recording server). After `connect_reality` returns `RealityFallback`, poll the recorded bytes (up to 5s) for the HTTP/2 preface:

```rust
// after the unwrap_err in the Task 3 test:
let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
loop {
    let bytes = recorded.lock().unwrap();
    if bytes.windows(http2::PREFACE.len()).any(|w| w == http2::PREFACE) {
        break;
    }
    drop(bytes);
    if std::time::Instant::now() > deadline {
        panic!("spider never sent the h2 preface to the dest");
    }
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p xray-tui-tls reality`
Expected: PASS (fallback test asserts `RealityFallback` + preface arrival; all other reality tests green).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls/src/reality/spider.rs crates/xray-tui-tls/src/reality/mod.rs crates/xray-tui-tls/src/lib.rs crates/xray-tui-tls/src/crypto/fingerprint/mod.rs
git commit -m "feat(tls): Spider-X h2 fallback on REALITY auth failure"
```

---

### Task 5: `client` module — the unified public API

**Files:**
- Create: `crates/xray-tui-tls/src/client.rs`
- Modify: `crates/xray-tui-tls/src/lib.rs` (`pub mod client;`)
- Test: `crates/xray-tui-tls/src/client.rs` (`mod tests`)

**Interfaces:**
- Consumes: `handshake::{connect, HandshakeParams, ServerVerifier}`, `reality::{connect_reality, RealityParams, HelloProvisioner, SpiderConfig}`, `profiles::BrowserProfile`, `crate::SecureRandom`.
- Produces (the public API native + future engine users consume):
  - `pub enum TlsMode { Plain { profile: Option<BrowserProfile>, verifier: Arc<dyn ServerVerifier> }, Reality { provisioner: Arc<dyn HelloProvisioner>, public_key: [u8; 32], short_id: Vec<u8>, spider: SpiderConfig } }`
  - `pub struct TlsConfig { pub mode: TlsMode, pub server_name: String, pub alpn: Option<Vec<Vec<u8>>>, pub rng: Arc<dyn SecureRandom> }`
  - `impl TlsConfig { pub fn plain(profile: Option<BrowserProfile>, verifier: Arc<dyn ServerVerifier>, server_name: impl Into<String>) -> Self; pub fn reality(provisioner: Arc<dyn HelloProvisioner>, public_key: [u8; 32], short_id: Vec<u8>, spider: SpiderConfig, server_name: impl Into<String>) -> Self }`
  - `pub async fn connect<S: Stream + 'static>(stream: S, config: &TlsConfig) -> Result<TlsStream<S>>`

- [ ] **Step 1: Write the module**

Create `crates/xray-tui-tls/src/client.rs`:

```rust
//! Unified client API: one `TlsConfig` + `connect` covering plain TLS (any
//! browser profile, verifier seam) and REALITY (any profile via
//! `ProfileProvisioner`, Spider-X fallback). The single entry the native
//! layer and external engine users call.

use std::sync::Arc;

use crate::error::Result;
use crate::handshake::{self, HandshakeParams, ServerVerifier};
use crate::profiles::BrowserProfile;
use crate::reality::{HelloProvisioner, RealityParams, SpiderConfig};
use crate::record::stream::TlsStream;
use crate::{SecureRandom, Stream};

/// The security mode of a connect.
pub enum TlsMode {
    /// Plain TLS 1.3, fingerprint-shaped hello.
    Plain {
        /// Fingerprint profile; `None` → `BrowserProfile::Chrome130`.
        profile: Option<BrowserProfile>,
        /// Server-authentication seam (WebPkiVerifier or a test verifier).
        verifier: Arc<dyn ServerVerifier>,
    },
    /// REALITY: fingerprint-shaped hello + sealed session id + HMAC/Ed25519
    /// server auth; Spider-X fallback on auth failure.
    Reality {
        /// `ProfileProvisioner(profile)` for a browser shape, or any custom
        /// `HelloProvisioner`.
        provisioner: Arc<dyn HelloProvisioner>,
        /// The server's static X25519 public key (decoded `pbk`).
        public_key: [u8; 32],
        /// The REALITY short id (≤8 bytes).
        short_id: Vec<u8>,
        /// Spider-X fallback behavior on auth failure.
        spider: SpiderConfig,
    },
}

/// A fully specified TLS/REALITY connect.
pub struct TlsConfig {
    pub mode: TlsMode,
    /// SNI (the `server_name` extension and verifier host).
    pub server_name: String,
    /// Plain: override the spec's ALPN list. Reality: ignored (protocol
    /// forces h2 + http/1.1).
    pub alpn: Option<Vec<Vec<u8>>>,
    /// Random source; `SystemRandom` default.
    pub rng: Arc<dyn SecureRandom>,
}

impl TlsConfig {
    #[must_use]
    pub fn plain(
        profile: Option<BrowserProfile>,
        verifier: Arc<dyn ServerVerifier>,
        server_name: impl Into<String>,
    ) -> Self {
        Self {
            mode: TlsMode::Plain { profile, verifier },
            server_name: server_name.into(),
            alpn: None,
            rng: Arc::new(ring::rand::SystemRandom::new()),
        }
    }

    #[must_use]
    pub fn reality(
        provisioner: Arc<dyn HelloProvisioner>,
        public_key: [u8; 32],
        short_id: Vec<u8>,
        spider: SpiderConfig,
        server_name: impl Into<String>,
    ) -> Self {
        Self {
            mode: TlsMode::Reality { provisioner, public_key, short_id, spider },
            server_name: server_name.into(),
            alpn: None,
            rng: Arc::new(ring::rand::SystemRandom::new()),
        }
    }
}

/// Run a TLS/REALITY connect over `stream`, dispatching on `config.mode`.
pub async fn connect<S: Stream + 'static>(
    stream: S,
    config: &TlsConfig,
) -> Result<TlsStream<S>> {
    match &config.mode {
        TlsMode::Plain { profile, verifier } => {
            let profile = profile.unwrap_or(BrowserProfile::Chrome130);
            let spec = profile.spec();
            let alpn: Option<Vec<&str>> = config
                .alpn
                .as_ref()
                .map(|list| {
                    list.iter()
                        .map(|p| {
                            std::str::from_utf8(p).map_err(|e| {
                                crate::error::TlsError::Handshake(format!(
                                    "invalid ALPN bytes: {e}"
                                ))
                            })
                        })
                        .collect::<Result<Vec<&str>>>()
                })
                .transpose()?;
            let tls = handshake::connect(
                stream,
                HandshakeParams {
                    spec: &spec,
                    server_name: &config.server_name,
                    alpn: alpn.as_deref(),
                    verifier: verifier.as_ref(),
                    rng: &config.rng,
                },
            )
            .await?;
            Ok(tls)
        }
        TlsMode::Reality { provisioner, public_key, short_id, spider } => {
            reality::connect_reality(
                stream,
                RealityParams {
                    server_name: &config.server_name,
                    public_key,
                    short_id,
                    provisioner: provisioner.as_ref(),
                    rng: &config.rng,
                    spider,
                },
            )
            .await
        }
    }
}
```

`RealityParams.public_key` is `&[u8; 32]` — the `TlsMode::Reality.public_key` field is `[u8; 32]`; `&config.mode` borrows it so pass `public_key` (a `&[u8; 32]`) directly. `spider` is `&SpiderConfig` from the borrowed mode. Add `use crate::reality as reality;` (or import `connect_reality`). Wire `pub mod client;` in `lib.rs`.

- [ ] **Step 2: Write API tests**

In `client.rs` `mod tests`: a plain-mode connect against the recording rustls server (Task 2 helper) proves the new entry works and defaults to Chrome130 when `profile: None` (assert via the server-received hello — or simpler, reuse the existing handshake goldens by asserting `TlsConfig::plain(None, verifier, sni)` produces a `TlsStream` that round-trips bytes). A reality-mode connect against the plain rustls server returns `RealityFallback` (proves the Reality arm + spider path through the public entry):

```rust
#[tokio::test]
async fn plain_default_profile_connects() {
    // recording rustls server + rcgen cert (Task 2 helper); verifier =
    // WebPkiVerifier::insecure() (AcceptAll-like) or the generated CA.
    let (server, addr) = spawn_recording_tls_server(...).await;
    let tcp = tokio::net::TcpStream::connect(addr).await.unwrap();
    let verifier: Arc<dyn ServerVerifier> = Arc::new(WebPkiVerifier::insecure());
    let mut conn = connect(tcp, &TlsConfig::plain(None, verifier, "localhost")).await.unwrap();
    conn.write_all(b"ping").await.unwrap();
    // server saw the request bytes; drop(server)
}
```

Use `WebPkiVerifier::webpki_roots().with_insecure(true)` (the engine's
insecure trust mode — see `verify/mod.rs`; `with_insecure`/`from_ca_der`/
`webpki_roots`/`with_pin` are the constructors, as used in native's
`fingerprint.rs::verifier_for`).

- [ ] **Step 3: Run tests**

Run: `cargo test -p xray-tui-tls client`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-tls/src/client.rs crates/xray-tui-tls/src/lib.rs
git commit -m "feat(tls): unified client module — TlsConfig/TlsMode/connect"
```

---

### Task 6: Native — drop rustls client, engine-only `wrap()`

**Files:**
- Delete: `crates/xray-tui-native/src/security/tls.rs`, `crates/xray-tui-native/src/security/tls_provider.rs`
- Modify: `crates/xray-tui-native/src/security/mod.rs` (`wrap` rewrite), `crates/xray-tui-native/src/security/fingerprint.rs` (drop `FingerprintConnector`, keep parsers + verifier build), `crates/xray-tui-native/src/security/reality.rs` (drop `connect`/`run_handshake`/`RealityConnector`, keep choice + decoders), `crates/xray-tui-native/src/context.rs` (drop `tls_provider` field), `crates/xray-tui-native/src/lib.rs` (module wiring), `crates/xray-tui-native/Cargo.toml` (deps), `crates/xray-tui-native/src/e2e/variant.rs` (`StandardTls.client_trust` now installs the engine test CA)
- Test: `crates/xray-tui-native/src/security/mod.rs`, `src/security/fingerprint.rs`, `src/security/reality.rs` (migrate/update)

**Interfaces:**
- Consumes: `xray_tui_tls::client::{TlsConfig, TlsMode, connect}`, `xray_tui_tls::verify::WebPkiVerifier`, `xray_tui_tls::profiles::BrowserProfile`, `xray_tui_tls::reality::{ProfileProvisioner, SpiderConfig}`.
- Produces:
  - `wrap()` builds the engine config from the proto security config; `TlsProvider`/`TlsConnector`/`TlsParams`/`FingerprintConnector`/`RealityConnector` are gone.
  - `NativeConnectParams` drops `tls_provider`; keeps `reality_provisioner`.
  - `fingerprint.rs` keeps `parse_fingerprint_id`, `profile_for`, `decode_pin_sha256`, `set_test_ca`, and a `verifier_for(sni/insecure/pin)` helper returning `WebPkiVerifier`.

- [ ] **Step 1: Rewrite `wrap()`**

In `crates/xray-tui-native/src/security/mod.rs`, replace the `wrap()` body so both arms build the engine config and call `xray_tui_tls::client::connect`:

```rust
pub async fn wrap(ctx: &LinkContext, stream: BoxStream) -> Result<BoxStream, NativeError> {
    let Some(sec) = ctx.security() else { return Ok(stream); };
    if sec.is_empty() { return Ok(stream); }
    let rng: Arc<dyn SecureRandom> = Arc::new(ring::rand::SystemRandom::new());
    match &sec.tls {
        Some(TlsConfig::Tls(opts)) => {
            let profile = opts
                .fp
                .as_ref()
                .map(|fp| fingerprint::parse_fingerprint_id(fp).and_then(fingerprint::profile_for))
                .transpose()?;
            let verifier: Arc<dyn ServerVerifier> = Arc::new(fingerprint::verifier_for(
                opts.insecure.unwrap_or(false),
                fingerprint::decode_pin_sha256(opts.pin_sha256.as_deref())?,
            ));
            let config = TlsConfig {
                mode: TlsMode::Plain { profile, verifier },
                server_name: ctx.sni(),
                alpn: (!ctx.alpn_vec().is_empty()).then(|| ctx.alpn_vec()),
                rng,
            };
            Ok(Box::new(client_connect(stream, &config).await?))
        }
        Some(TlsConfig::Reality(opts)) => {
            let pbk = opts.pbk().ok_or_else(|| NativeError::Reality("reality config missing pbk".into()))?;
            let provisioner: Arc<dyn HelloProvisioner> = match &ctx.params.reality_provisioner {
                HelloProvisionerChoice::Custom(p) => p.clone(),
                HelloProvisionerChoice::FixedChrome133 => match &opts.fp {
                    Some(fp) => Arc::new(ProfileProvisioner(
                        fingerprint::parse_fingerprint_id(fp).and_then(fingerprint::profile_for)?,
                    )),
                    None => Arc::new(ProfileProvisioner(BrowserProfile::Chrome133)),
                },
            };
            let spider = SpiderConfig {
                paths: opts.spx().map_or_else(|| vec!["/".to_string()], |s| vec![s.to_string()]),
                ..SpiderConfig::default()
            };
            let config = TlsConfig {
                mode: TlsMode::Reality {
                    provisioner,
                    public_key: decode_pbk(pbk)?,
                    short_id: decode_sid(opts.sid().unwrap_or_default())?,
                    spider,
                },
                server_name: ctx.sni(),
                alpn: None,
                rng,
            };
            Ok(Box::new(client_connect(stream, &config).await?))
        }
        None => Ok(stream),
    }
}
```

Notes:
- Import `xray_tui_tls::client::{TlsConfig as EngineTlsConfig, TlsMode, connect as client_connect}` with an alias because `proto_spec::TlsConfig` is also in scope (the `match &sec.tls` uses the proto type). Alias the proto import if needed.
- `rng` type: `Arc<dyn xray_tui_tls::SecureRandom>`. The engine's `TlsConfig.rng` is `Arc<dyn SecureRandom>` (crate seam) — `SystemRandom` implements it via the blanket impl.
- Map the engine error to `NativeError`: on `TlsError::RealityFallback` → `NativeError::Reality("REALITY: received real certificate (potential MITM or redirection)".into())`; other engine errors → `NativeError::Tls(...)`. Add a small `fn map_tls_err(e: TlsError) -> NativeError` helper.
- `sec.pbk()`, `sec.sid()`, `sec.spx()`, `opts.fp`, `opts.insecure`, `opts.pin_sha256`, `ctx.sni()`, `ctx.alpn_vec()` — confirm the accessor names against the proto `SecurityConfig`/`RealityOpts` (`spx()` may be `RealityOpts.spx` — check `crates/xray-tui-proto` and use the field path that compiles; `pbk()`/`sid()` are used by the old `reality.rs::connect`).

- [ ] **Step 2: Delete the rustls client + provider plug**

- Delete `crates/xray-tui-native/src/security/tls.rs` and `tls_provider.rs`.
- Remove `pub mod tls;` and `pub mod tls_provider;` from `security/mod.rs`.
- In `context.rs`, drop the `tls_provider: TlsProvider` field from `NativeConnectParams` and its init in `new()`; remove the `TlsProvider` import.
- In `fingerprint.rs`: delete `FingerprintConnector` + its `TlsConnector` impl + the `tls_provider` import; keep `parse_fingerprint_id`, `profile_for`, `decode_pin_sha256`, `set_test_ca`, `TEST_CA`. Change `verifier_for` to take `(insecure: bool, pin: Option<[u8;32]>)` and return `WebPkiVerifier` (the `TlsParams` struct is gone).
- In `reality.rs` (native): delete `connect`, `run_handshake`, `RealityConnector`; keep `HelloProvisionerChoice`, `decode_pbk`, `decode_sid`; update the `pub use xray_tui_tls::reality::{...}` re-export list to add `SpiderConfig` and drop `ProvisionedHello`/`HelloProvisionParams` if now unused (keep whatever compiles — unused-import warnings are fatal under clippy, so trim).
- In `lib.rs` (native): remove `security::tls` / `security::tls_provider` module references.

- [ ] **Step 3: Cargo dependency cleanup**

In `crates/xray-tui-native/Cargo.toml`:
- Remove `tokio-rustls` (deleted client path) and `webpki-roots` (the engine owns the root store now — `WebPkiVerifier::webpki_roots`).
- Make `rustls` optional (`rustls = { workspace = true, features = ["ring"], optional = true }`) and add it to the `native-e2e` feature (`"dep:rustls"`); ALSO add `rustls` to `[dev-dependencies]` (unit tests in `src` use the server double under `cfg(test)` without the e2e feature). Keep `ring`, `base64`, `xray-tui-proto`, `xray-tui-tls`, `tokio`, `thiserror`, `uuid`, `x25519-dalek` (optional), `serde_json`/`tempfile`/`rcgen`/`tiny_http` (optional, e2e).

- [ ] **Step 4: Migrate tests**

In `security/mod.rs` tests:
- `tls_wraps_and_passes_bytes` (standard rustls client) → now uses the engine Plain path: build `TlsConfig::plain(None, verifier, sni)` with a verifier trusting the rcgen harness CA (`WebPkiVerifier::from_ca_der`), against the rustls server. The `set_test_config`/`test_client_config` helpers in `tls.rs` are deleted — the fingerprint `set_test_ca`/`verifier_for` path replaces them.
- `fingerprint_tls_wraps_and_passes_bytes` — unchanged behavior, still passes (engine Plain arm with a profile).
- `reality_connect_rejects_short_pbk`, `reality_handshake_errors_are_reality_errors` — update to expect the engine `RealityFallback` mapping (`NativeError::Reality`).
- e2e `variant.rs` `StandardTls.client_trust`: it currently calls `crate::security::tls::set_test_config(...)` — replace with `crate::security::fingerprint::set_test_ca(&certs.ca_der)` (the engine verifier path now handles stock TLS too). `FingerprintTls.client_trust` unchanged.

Run: `cargo test -p xray-tui-native` (unit, no e2e) — PASS. Then `cargo check -p xray-tui-native --features native-e2e` — compiles (e2e module adapted in Task 7).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/security crates/xray-tui-native/src/context.rs crates/xray-tui-native/src/lib.rs crates/xray-tui-native/Cargo.toml
git commit -m "refactor(native): engine-only TLS client — drop rustls + provider plug"
```

---

### Task 7: E2E — two-servers scenarios + spider assertion

**Files:**
- Modify: `crates/xray-tui-native/src/e2e/mod.rs` (`E2eExpect` + `run()` error handling + `E2eCase` probe-target hook), `crates/xray-tui-native/src/e2e/harness.rs` (`TlsEchoServer` byte recording), `crates/xray-tui-native/src/e2e/variant.rs` (new variants), `crates/xray-tui-native/src/e2e/case.rs` (`CaseSpec` connect-expect), `crates/xray-tui-native/src/e2e/config.rs` (wrong-key client params), `crates/xray-tui-native/tests/vless.rs` + `tests/vmess.rs` (new test fns)
- Test: the new e2e fns (feature + env gated)

**Interfaces:**
- Consumes: existing `E2eCase`, `CaseSpec`, `TlsVariant`, `SecurityVariant`, `ServerEnv`, `config::*`, `harness::{TlsEchoServer, generate_certs}`.
- Produces:
  - `pub enum ConnectExpect { Ok, ErrRealityFallback }`; `E2eExpect { pub connect: ConnectExpect, pub status: u16, pub body: String }` with a `Default` (`Ok, 200, "hello native core"`).
  - `E2eCase` gains `fn probe_target(&self, _env: &ServerEnv) -> SocketAddr { _env.echo }` (default).
  - `TlsEchoServer` records post-handshake bytes (`Arc<Mutex<Vec<u8>>>`, first 64 B) with `pub fn saw_h2_preface(&self) -> bool`.
  - New variants: `RealityWrongPbkTls`, `RealityWrongSidTls`, `RealityServerPlainClientTls`, `PlainServerRealityClientTls`.

- [ ] **Step 1: Extend `E2eExpect` + `run()` + `E2eCase`**

In `crates/xray-tui-native/src/e2e/mod.rs`:

```rust
pub enum ConnectExpect { Ok, ErrRealityFallback }

pub struct E2eExpect {
    pub connect: ConnectExpect,
    pub status: u16,
    pub body: String,
}

impl Default for E2eExpect {
    fn default() -> Self {
        Self { connect: ConnectExpect::Ok, status: 200, body: crate::e2e::config::BODY.to_string() }
    }
}
```

In the `E2eCase` trait, add `fn probe_target(&self, _env: &ServerEnv) -> SocketAddr { _env.echo }`. In `run()`:
- `let expect = case.expected();`
- If `matches!(expect.connect, ConnectExpect::ErrRealityFallback)`: after `case.client_trust(&certs)`, call `case.client_params(port, case.probe_target(&env))`, then `match crate::connect(params).await { Err(_) => { /* assert spider: wait up to 5s for tls_echo.saw_h2_preface() */ Ok(()) }, Ok(_) => return Err(format!("{}: expected RealityFallback, got a connection", case.label())) }` — the wait polls `tls_echo.saw_h2_preface()` with 100ms sleeps; if the deadline passes, return Err. Skip the probe.
- Else (Ok): the existing connect → probe → status/body compare flow, but use `case.client_params(port, case.probe_target(&env))` instead of the hardcoded `echo.addr` target.

- [ ] **Step 2: Record bytes in `TlsEchoServer`**

In `harness.rs`, `TlsEchoServer` gains a `recorded: Arc<Mutex<Vec<u8>>>`. In the accept loop, after the rustls handshake completes, `read` from the conn and append up to the first 64 bytes to `recorded` (bounded; keep reading so the spider's GET doesn't wedge the loop). Add:

```rust
impl TlsEchoServer {
    /// True if any connection received the HTTP/2 client preface
    /// (`PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n`) as the first app data.
    pub fn saw_h2_preface(&self) -> bool {
        let b = self.recorded.lock().unwrap();
        b.windows(crate::security::reality::H2_PREFACE.len())
            .any(|w| w == crate::security::reality::H2_PREFACE)
    }
}
```

Export `pub const H2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";` from `xray-tui-native/src/security/reality.rs` (or import from the engine — `xray_tui_tls::http2::PREFACE` is `pub`; use that to avoid duplication: `xray_tui_tls::http2::PREFACE`).

- [ ] **Step 3: Add the wrong-key / probe variants**

In `variant.rs`, add:

```rust
/// REALITY server with the real keypair but the client given a *wrong*
/// public key (valid 32-byte key, not the server's). Server transparently
/// proxies → client's auth fails → RealityFallback.
pub struct RealityWrongPbkTls(RealityTls);

impl RealityWrongPbkTls {
    #[must_use]
    pub fn fresh() -> Self {
        let inner = RealityTls::fresh();
        // A wrong but well-formed pbk: the same keypair generation, kept
        // only on the client side (never the server's private key).
        let (wrong_private, wrong_pbk) = config::reality_keypair();
        let _ = wrong_private;
        Self(RealityTls { pbk: wrong_pbk, ..inner })
    }
}

impl TlsVariant for RealityWrongPbkTls {
    fn name(&self) -> &'static str { "reality-wrong-pbk" }
    fn sni(&self) -> &'static str { "localhost" }
    fn reality_private_key(&self) -> Option<&str> { self.0.reality_private_key() }
    fn reality_pbk(&self) -> Option<&str> { Some(&self.0.pbk) }          // wrong pbk
    fn reality_sid(&self) -> Option<&str> { self.0.reality_sid() }
}

/// REALITY with the server's short id *different* from the client's.
/// Server decrypts the session id (right pbk) but the sid inside doesn't
/// match its own → transparent proxy → fallback.
pub struct RealityWrongSidTls(RealityTls, String); // (real server, wrong client sid)

impl RealityWrongSidTls {
    #[must_use]
    pub fn fresh() -> Self {
        Self(RealityTls::fresh(), config::reality_sid())
    }
}

impl TlsVariant for RealityWrongSidTls {
    fn name(&self) -> &'static str { "reality-wrong-sid" }
    fn sni(&self) -> &'static str { "localhost" }
    fn reality_private_key(&self) -> Option<&str> { self.0.reality_private_key() }
    fn reality_pbk(&self) -> Option<&str> { Some(&self.0.pbk) }
    fn reality_sid(&self) -> Option<&str> { self.0.reality_sid() }
    fn reality_client_sid(&self) -> Option<&str> { Some(&self.1) }        // wrong client sid
}

/// REALITY server (real key) but a plain-fingerprint client — the "active
/// probing without REALITY" case: the probe client is transparently proxied
/// and completes standard TLS to the dest (stealth). Probe target = tls_echo.
pub struct RealityServerPlainClientTls(RealityTls);

impl TlsVariant for RealityServerPlainClientTls {
    fn name(&self) -> &'static str { "reality-server-plain-client" }
    fn sni(&self) -> &'static str { "localhost" }
    fn fingerprint(&self) -> Option<&'static str> { Some("chrome") }
    fn reality_private_key(&self) -> Option<&str> { self.0.reality_private_key() }
    fn reality_pbk(&self) -> Option<&str> { Some(&self.0.pbk) }
    fn reality_sid(&self) -> Option<&str> { self.0.reality_sid() }
    fn client_trust(&self, certs: &Certs) { crate::security::fingerprint::set_test_ca(&certs.ca_der); }
}

/// Plain-TLS VLESS server (harness-CA cert) but a REALITY client — the
/// client's auth fails against a non-REALITY server → RealityFallback.
pub struct PlainServerRealityClientTls(RealityTls);

impl TlsVariant for PlainServerRealityClientTls {
    fn name(&self) -> &'static str { "plain-server-reality-client" }
    fn sni(&self) -> &'static str { "localhost" }
    fn reality_private_key(&self) -> Option<&str> { None }               // server = cert TLS
    fn reality_pbk(&self) -> Option<&str> { Some(&self.0.pbk) }
    fn reality_sid(&self) -> Option<&str> { self.0.reality_sid() }
}
```

Add `fn reality_client_sid(&self) -> Option<&str> { None }` to the `TlsVariant` trait (default), and in `config::reality_client_security`, use `tls.reality_client_sid().or(tls.reality_sid())` for the client-side `sid`. For `RealityWrongSidTls`, `reality_client_sid` returns the wrong sid.

Note: `RealityTls` fields are `pub pbk`, `pub sid`, `private_key` (private). `RealityWrongPbkTls` constructs via `RealityTls { pbk: wrong_pbk, ..inner }` — the private `private_key` field is fine via struct-update from `inner`. Keep `RealityWrongPbkTls` in the same module as `RealityTls` so the private field is accessible.

- [ ] **Step 4: Add the error-case expectation + cases**

`E2eExpect` for error cases: `E2eExpect { connect: ConnectExpect::ErrRealityFallback, ..Default::default() }`. The `CaseSpec`/`E2eCase` impl derives expectations from the variant — add a method on the `TlsVariant` trait or a `CaseSpec` flag. Simplest: add `fn expect_fallback(&self) -> bool { false }` to `TlsVariant` (override `true` in `RealityWrongPbkTls`, `RealityWrongSidTls`, `PlainServerRealityClientTls`), and have `CaseSpec`'s `E2eCase::expected()` return `E2eExpect { connect: if tls.expect_fallback() { ConnectExpect::ErrRealityFallback } else { ConnectExpect::Ok }, ..Default::default() }`.

Add `fn probe_target` override for the `RealityServerPlainClientTls` case: `CaseSpec` can't know the variant type — instead have the variant expose it: add `fn probe_dest(&self) -> bool { false }` to `TlsVariant` (true for `RealityServerPlainClientTls`), and `CaseSpec::probe_target(env)` returns `env.tls_echo` when `tls.probe_dest()` else `env.echo`.

- [ ] **Step 5: Add the test fns**

In `crates/xray-tui-native/tests/vless.rs`, add:

```rust
#[tokio::test]
async fn vless_reality_wrong_pbk_is_fallback() {
    run_against_cores(&CaseSpec::vless().with_tls(Box::new(RealityWrongPbkTls::fresh())))
        .await
        .expect("vless reality wrong-pbk e2e failed");
}

#[tokio::test]
async fn vless_reality_wrong_sid_is_fallback() {
    run_against_cores(&CaseSpec::vless().with_tls(Box::new(RealityWrongSidTls::fresh())))
        .await
        .expect("vless reality wrong-sid e2e failed");
}

#[tokio::test]
async fn vless_plain_client_through_reality_server_is_stealth() {
    run_against_cores(&CaseSpec::vless().with_tls(Box::new(RealityServerPlainClientTls::fresh())))
        .await
        .expect("vless plain-probe e2e failed");
}

#[tokio::test]
async fn vless_reality_client_into_plain_server_is_fallback() {
    run_against_cores(&CaseSpec::vless().with_tls(Box::new(PlainServerRealityClientTls::fresh())))
        .await
        .expect("vless reality-into-plain e2e failed");
}
```

Import the new variants in `tests/vless.rs`. The `RealityServerPlainClientTls` case relies on `run()` using `probe_target(env)` = `tls_echo` and the probe flowing through the transparent proxy to the dest.

- [ ] **Step 6: Run the e2e sweep**

Run: `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vless --test vmess`
Expected: ALL PASS — the 7 existing cases (now via the engine Plain path) + the 4 new cases × 2 cores. The `RealityServerPlainClientTls` (stealth) case proves a plain client gets a working TLS session to the dest through the REALITY server.

- [ ] **Step 7: Commit**

```bash
git add crates/xray-tui-native/src/e2e crates/xray-tui-native/tests/vless.rs crates/xray-tui-native/tests/vmess.rs
git commit -m "test(native): two-servers e2e — wrong keys, plain probe, spider assertion"
```

---

### Task 8: NATIVE_CORE.md roadmap + doc sync

**Files:**
- Modify: `NATIVE_CORE.md`, `AGENTS.md` (if the native/tls sections reference the removed rustls path)

- [ ] **Step 1: Update NATIVE_CORE.md**

- Add a roadmap note: the native client path is **engine-only** (`xray-tui-tls`); the rustls client path was removed — rustls remains only as the server-side test double. TLS 1.2 engine support is a future task; legacy (TLS 1.2-only) servers are not yet reachable.
- Document the unified `TlsConfig { Plain | Reality }` API + Spider-X fallback behavior (on a REALITY auth failure — real certificate / transparent proxy — the client keeps the session alive with HTTP/2 GETs to the real site, then reports `REALITY: received real certificate (potential MITM or redirection)`).
- Update the security-layer capability table + protocol roadmap emoji columns to reflect engine-only TLS + per-profile REALITY.

- [ ] **Step 2: Update AGENTS.md pointers if stale**

Check `AGENTS.md`'s `xray-tui-native`/`xray-tui-tls` bullets still name the right modules (they mention `security/mod.rs` `wrap()` and the `TlsProvider` plug — update the plug wording to the engine config).

- [ ] **Step 3: Full verification**

Run: `cargo test --workspace` (1002+ tests, now including the new engine/native tests), `cargo clippy --workspace --all-targets` (0 warnings), `cargo fmt --check` (on touched crates). Then the e2e sweep from Task 7 Step 6.

- [ ] **Step 4: Commit**

```bash
git add NATIVE_CORE.md AGENTS.md
git commit -m "docs: engine-only TLS client + Spider-X fallback roadmap"
```

---

## Self-Review

- **Spec coverage:** `client` module + `connect()` (T5), `ProfileProvisioner` (T1), driver unification (T3), Spider-X (T4), engine-only native + `wrap()` rewrite + Cargo (T6), two-servers e2e + `E2eExpect` + byte recording + R2/R3/R4/R5 (T7), TLS-1.2 roadmap + docs (T8), `TlsError::RealityFallback` (T1), full `BrowserProfile` in the engine config + 5-id native parser (T6). All spec sections covered.
- **Placeholders:** every step has concrete code or exact existing-symbol references; `spawn_recording_tls_server`/`spawn_plain_tls_server` are named test helpers whose construction is specified (rustls + rcgen, mirroring `e2e/harness.rs`). Proto accessor names (`spx()`, `pbk()`, `sid()`, `fp`, `insecure`, `pin_sha256`) are flagged to confirm against `xray-tui-proto` at implementation time.
- **Type consistency:** `ServerAuth`/`AuthOutcome`/`drive` defined in T3, used in T3+T4+T5; `SpiderConfig` defined in T3 (Default/Clone), used in T4 (spider) + T5 (TlsMode) + T6 (native) + T7 (no direct). `RealityParams.spider` added in T3, consumed in T4+T5. `TlsConfig`/`TlsMode` defined in T5, consumed in T6. `ConnectExpect`/`probe_target`/`reality_client_sid`/`expect_fallback`/`probe_dest` defined in T7. `http2::Client` in T2, used in T4. `rand_u64` in T4, produced in `crypto/fingerprint/mod.rs`.
