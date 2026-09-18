# Cryptographic dependency hardening: RustCrypto ML-KEM, hazmat sites, and advisory triage

Status: implemented (2026-09-18). Route: `brainstorming` (cross-module
dependency + security-posture change).
Scope of change: dependency direction and security posture of `xray-tui-tls`,
`xray-tui-native`, `xray-tui-proto`, `xray-tui-core`; no wire-format, schema, or
user-facing behavior change.

## 1. Problem

Three classes of finding, all reproduced on this checkout at `7e20498`-era
`Cargo.lock` (818 crates; `cargo audit`: 0 vulnerabilities, 6 warnings).

**P1 — `oqs` (liboqs) is a C build dependency in the crypto core.**
`crates/xray-tui-tls/Cargo.toml:41` pulls
`oqs = { version = "0.11", features = ["ml_kem", "kyber", "vendored"] }`.
`oqs-sys` builds liboqs through `cmake::Config::new("liboqs")`, generates
bindings with `bindgen` (requires libclang), and vendors OpenSSL. Both
`rust-toolchain.toml` and `deny.toml` declare `x86_64-pc-windows-gnu` as a
first-class target, where the toolchain may be absent — the reported failure.
Its sole consumer is `crates/xray-tui-tls/src/crypto/mlkem.rs`; everything
else (TLS `X25519MLKEM768` key shares, REALITY, VLESS `mlkem768x25519plus`)
goes through that wrapper.

**P2 — the `ctr` crate carries a hazmat banner.** `ctr 0.10.1` is used in one
place: `crates/xray-tui-native/src/protocol/vless/encryption/mlkem.rs`
(`Ctr128BE<Aes256>`, lines 37/46/269/479/746/1012). That is xray's `XorConn`
keystream (`thirdparty/Xray-core/proxy/vless/encryption/xor.go`): relay blocks
are XOR-masked in `xorpub`/`random` modes and the 5-byte record headers are
masked in `random` mode, keyed by `blake3.DeriveKey("VLESS", k)` with a 16-byte
big-endian counter. The banner warns about unauthenticated ciphertext; here
integrity comes from the NFS AEAD and the `hash32` relay chain, and no
non-hazmat RustCrypto API exists for raw AES-CTR. The real defect is coverage:
`roundtrip_xorpub_custom_padding` (`mlkem.rs:1216`) runs client against our own
server double, so a wrong mask passes on both ends, and the only true peer
coverage is in `#[ignore]`d e2e.

**P3 — dependency hygiene.** `chacha20 0.10.1` is yanked upstream and reachable
through our direct `chacha20poly1305 0.11.0` (0.10.2 is clean).
`smartstring 1.0.1` is unmaintained (RUSTSEC-2026-0249) and a direct dependency
of `xray-tui-proto` (`Cargo.toml:45`), hidden behind the `TinyText` alias
(`src/urlx/mod.rs:5`, 269 uses). `postcard`'s default `heapless-cas` feature
drags in unmaintained `heapless 0.7`/`atomic-polyfill` although no heapless
type is used. `lru 0.16.4` (unsound, RUSTSEC-2026-0253) is reachable as
`toasty-driver-turso → turso_core → tantivy → lru`; no fixed `0.16.x` exists;
we never call `LruCache::pop`. `bincode 1.3.3` (unmaintained) arrives through
`heed-types` with no in-tree fix. A hand-rolled constant-time comparison sits
in `crates/xray-tui-tls/src/reality/verify.rs:319`.

**P4 — no authority doc and no compile-time guarantee.** The repo documents its
hand-written SQL sites in `docs/database-manual-sql.md`, but the equivalent
crypto surfaces (hand-rolled BLAKE3 `b3.rs`, `ctr`, the hand-rolled TLS 1.3 key
schedule, QUIC header protection) have no recorded cause or retirement trigger,
and nothing prevents a future edit from introducing `unsafe` into the crypto
crates.

## 2. Decisions

- **D1** ML-KEM moves to RustCrypto `ml-kem 0.3.2`; `oqs` is removed rather than
  made optional, so no build path retains the CMake/libclang requirement.
- **D2** `ctr` is kept. The banner is accurate but not applicable: the mode is
  used as a keystream mask whose integrity is provided elsewhere, exactly as
  upstream xray does. The gap is closed with pinned vectors, not with a
  replacement.
- **D3** `ring` is retained. Replacing it means a custom rustls `CryptoProvider`
  plus RustCrypto X.509 signature verification that `rustls-webpki` only
  provides via ring/aws-lc-rs — a separate project (user decision, 2026-09-18).
- **D4** `b3.rs` stays. blake3 1.8.7 exposes only `hash_derive_key_context(&str)`
  / `ContextKey` / `new_from_context_key(&[u8;32])`, and its `guts` module is
  deprecated and hash-mode-only (`ChunkState`, `parent_cv`); xray's derive-key
  contexts are binary (16-byte IV, 1216-byte PFS key, record bytes), so no
  upstream API can replace it. It is recorded as a hazmat site with its
  existing crate cross-checks and Go-generated binary-context vectors.
- **D5** `zstd → ruzstd` is deferred, on two grounds. The build-target argument
  does not hold: `zstd 0.13.3` is independently required by
  `tantivy-sstable → tantivy-columnar → tantivy → turso_core`, and
  `libsqlite3-sys`, `lmdb-master-sys` and `ring` already need a C toolchain, so
  the swap would not remove a `cc` invocation. The trust-boundary argument is
  real but inverted in this case: `zstd` is the one decompressor fed
  server-supplied bytes (`handshake::decompress_certificate_message`, RFC 8879
  algorithm 0x0003), and bundled libzstd is a heavily fuzzed, industry-standard
  decoder, whereas `ruzstd` is a young decode-only implementation with a
  documented window-size caveat. Decode is bounded by `decompress_bounded`, and
  the client never encodes, so the exposure is a bounded parse. The alternative
  is recorded in `docs/crypto-dependencies.md` with its revisit trigger
  (ruzstd maturity/fuzzing coverage), and the adoption note that the current
  round-trip test's encoder call (`handshake/mod.rs:1607`) must become a pinned
  compressed fixture — a `zstd` dev-dependency would still build `zstd-sys` and
  defeat the point.
- **D6** Hazmat and accepted-advisory knowledge is recorded in
  `docs/crypto-dependencies.md`, indexed next to `docs/database-manual-sql.md`;
  the accepted posture is recorded as ADR 0007 at closeout.

## 3. Slice 1 — ML-KEM on `ml-kem 0.3.2`

Wrapper: `crates/xray-tui-tls/src/crypto/mlkem.rs`. Consumers keep working
unchanged: `reality/mod.rs:280,294`, `handshake/mod.rs:30,196,249`,
`xray-tui-native/src/protocol/vless/encryption/mlkem.rs`,
`src/e2e/config.rs:84`, `examples/grader.rs`, `tests/tls_peet_ws.rs`,
`tests/generated_ja4_gate.rs`, `reality/messages.rs`.

Dependency:

```toml
ml-kem = { version = "0.3.2", default-features = false, features = ["zeroize"] }
```

`default-features = false` (no `alloc`) keeps `Ciphertext`/`SharedKey` as inline
arrays, preserving the "no avoidable allocation" rule that
`encapsulate_into` exists to serve. `zeroize` enables `module-lattice/zeroize`;
the wrapper's own `Zeroizing` wrappers stay.

Primitive mapping (verified against the crate source):

| current | replacement |
| --- | --- |
| `Mlkem768::generate_keypair()` | `MlKem768::generate_keypair()` (`kem::Kem`) |
| `keypair_from_seed(&[u8; 64])` | `kem::FromSeed::from_seed(&B64)` — the same FIPS 203 `d ‖ z` seed |
| `PublicKey::from_bytes(1184)` | `EncapsulationKey::new(&Key<Self>)`, fallible (`InvalidKey`) |
| `encapsulate_into(pk, ct, ss)` | `Encapsulate::encapsulate_with_rng(&mut RingRng)` into caller buffers |
| `decapsulate(sk, ct)` | `Decapsulate::decapsulate(&ct)` |
| `SecretKey::from_bytes(2400)` / `as_bytes()` | seed form: `DecapsulationKey::from_seed` / `to_seed()` (expanded form is deprecated upstream) |

`SecretKey` wraps the typed `DecapsulationKey<MlKem768>` (built by
`from_seed`, serialized by `to_seed`) so `decapsulate` never re-expands a key;
the wrapper keeps its hand-written length-only `Debug`. Its only non-test
consumer is `HandshakeParams.mlkem_sk` in the server-side test double. The
2400-byte round-trip test is replaced by a seed round-trip test.

Randomness: a ~15-line `RingRng` adapter implements `rand_core 0.10`
`TryRng`/`CryptoRng` over `ring::rand::SystemRandom`, matching the crate's
existing `.expect("ring CSPRNG fills")` convention, so key material keeps
coming from the workspace's single RNG provider and the fallible path is a
`Result` rather than a panic. `ml-kem`'s `getrandom` feature is the documented
fallback if the trait shape proves unstable.

**Migration proof (the load-bearing test).** Before removing `oqs`, capture a
KAT from the current implementation with a throwaway harness: a fixed 64-byte
seed → the **encapsulation key** bytes. The assertion is pinned on the ek, not
the decapsulation key: liboqs' `SecretKey` is the 2400-byte expanded form while
`ml-kem` serializes the dk as the 64-byte seed, so dk bytes are not comparable
across the swap, whereas the ek is the byte string a VLESS client publishes and
a Go/circl peer must accept.

Two oracles, one assertion — the vectors came out byte-identical, so a second
test asserting the same constant would be padding:

1. The ek vector captured from the retired liboqs implementation.
2. The same ek reproduced independently by the installed Go 1.27.1
   `crypto/mlkem`, so the invariant does not rest on liboqs ≡ RustCrypto
   agreeing with each other.

`mlkem768_seed_derivation_matches_liboqs_and_go_stdlib` pins both (two seeds:
`[0u8; 64]` and `0..=63`), with the generator recorded in the test comment.

Both crates implement FIPS 203 `KeyGen_internal(d, z)`, so equality is the
expectation; if it fails, FIPS 203 semantics win and the divergence is
investigated before proceeding — the `pq-enc` e2e row that would otherwise catch
it is `#[ignore]`d (P2/§4). Encapsulation/decapsulation agreement with a real
peer is covered by the green hybrid-TLS e2e rows (§10), which is why this KAT is
about key derivation and not about round-trip.

Removals: the `oqs` dependency line and its comment; `cargo hakari generate`;
`Cargo.lock` refresh. Proof of removal: `cargo tree -i oqs-sys` finds nothing
and no manifest mentions `oqs`.

## 4. Slice 2 — VLESS XOR mask: keep `ctr`, pin the wire

No code change to the mask itself. Three additions to
`crates/xray-tui-native/src/protocol/vless/encryption/mlkem.rs`:

1. A call-site rationale on `new_ctr` stating what the crate banner means and
   why it does not apply here (keystream mask; integrity from the NFS AEAD and
   the `hash32` chain; `Ctr128BE` ≡ Go `cipher.NewCTR` with a 16-byte counter
   block).
2. A NIST SP 800-38A F.5.5 (AES-256-CTR) vector pinning the raw counter-mode
   semantics of `Ctr128BE<Aes256>` with a 16-byte IV — the property that makes
   the mask wire-compatible with Go.
3. Go-generated end-to-end vectors for the actual construction: the relay-block
   mask keyed by `blake3.DeriveKey("VLESS", server_key)`, the chained `hash32`
   region unmasked by the previous hop's shared secret, and the `random`-mode
   5-byte record-header stream (including the 5-byte-per-record keystream
   advance). Generated with Go 1.27.1 (`crypto/aes` + `lukechampine.com/blake3`,
   xray's own dependency) and pinned with the generator command in the test
   comment, mirroring the existing Go-vector precedent in `b3.rs`.

This is the test class `roundtrip_xorpub_custom_padding` cannot provide:
an independent peer, so a mask bug fails instead of agreeing with itself.

Coverage boundary, stated so it is not mistaken for full peer verification:
the `mlkem768x25519plus` account-encryption path is the one that runs this mask,
and its real-peer e2e row (`pq-enc`) is `#[ignore]`d. The wire check therefore
rests on these Go-generated vectors plus the seed path the e2e harness uses
(`e2e/config.rs::mlkem_enc_pair`, seed → ek via the wrapper of Slice 1) — the
same derivation both the client string and the server config exercise.

## 5. Slice 3 — Dependency hygiene

- `cargo update -p chacha20 --precise 0.10.2` (yanked 0.10.1 → clean 0.10.2;
  `chacha20poly1305 0.11.0` requires `^0.10`).
- `smartstring` → `compact_str`: `crates/xray-tui-proto/Cargo.toml:45` becomes
  `compact_str = { version = "0.9", features = ["serde"] }` and
  `src/urlx/mod.rs:5` becomes
  `pub type TinyText = compact_str::CompactString;`. `0.9` matches the version
  already in the lock via `ratatui-core`, so no second major appears. The alias
  is the only seam (269 uses resolve through it); the two types share the
  surface this code depends on (`Deref<Target = str>`, `From<&str>`,
  `From<String>`, `Hash`, `Ord`, serde as a string on the wire).
- `crates/xray-tui-core/Cargo.toml:63`: `postcard = { version = "1",
  default-features = false, features = ["alloc"] }` — drops `heapless 0.7` and
  `atomic-polyfill` (no heapless type is used).
- `.cargo/audit.toml`: record the triage for `lru 0.16.4` and `bincode 1.3.3`
  as comments beside the existing `rsa` entry, stating the reachable path, why
  it is not actionable in this repo (`0.16.4` is the newest `0.16.x` and
  `tantivy` pins `^0.16.3`, so the unsound advisory is unpatchable from our side
  without a turso/tantivy bump or a `[patch]`; `heed-types` owns `bincode`), and
  the re-evaluation trigger. They stay visible as `informational_warnings` —
  deliberately not added to `ignore`.
- `crates/xray-tui-tls/src/reality/verify.rs:319`: replace the hand-rolled
  `ct_eq` with `subtle::ConstantTimeEq` (`subtle 2.6.1` is already in the
  resolved graph) and add `subtle` as a direct dependency of `xray-tui-tls`.

## 6. Slice 4 — Governance

- `#![forbid(unsafe_code)]` at the root of `xray-tui-tls` and
  `xray-tui-native`. Both crates contain zero `unsafe` occurrences today
  (`unsafe impl|fn|extern|{` across `src/`, `tests/`, `benches/`, `examples/`),
  so this is free now and makes hand-rolled unsafe unrepresentable in the
  crypto and protocol core later. Crates that legitimately need FFI
  (`xray-tui-core` libc/heed, `xray-tui-db`, `xray-tui-config`) are out of
  scope.
- `docs/crypto-dependencies.md` — the authority for hazmat and hand-rolled
  crypto sites, in the shape of `docs/database-manual-sql.md`: for each site
  the cause, the wire/protocol constraint that forces it, the evidence that
  pins it, and the retirement trigger. Initial inventory: `b3.rs`
  (binary-context BLAKE3 derive-key), `ctr` (xray `XorConn` keystream), the
  C decoder on the handshake path (`zstd`/`zstd-sys` for RFC 8879 algorithm
  0x0003 — bounded decode guard, `ruzstd` alternative and its maturity
  trade-off recorded), the
  hand-rolled TLS 1.3 key schedule and transcript, the TLS 1.2 PRF path, QUIC
  header protection in `xray-tui-route/src/sniff.rs`, protocol-mandated legacy
  primitives (VMess MD5/AES-CFB-era KDFs, `EVP_BytesToKey`, classic-AEAD
  HKDF-SHA1), and the accepted advisories. Cross-referenced from `AGENTS.md`
  and `NATIVE_CORE.md` next to the existing manual-SQL pointer.

## 7. Slice 5 — Docs and gate sync

- `NATIVE_CORE.md` and `AGENTS.md` (decisions 17/19 mention liboqs/`oqs`
  vendored) updated to the RustCrypto ML-KEM backend; `Cargo.toml` comments
  corrected where they describe the oqs build.
- `cargo hakari generate` after the lock change. The acceptance gate is the
  **full** `just quality-gate` (`code` = fmt-check/clippy/nextest; `deps` =
  hakari-check/deny/machete/outdated/audit) — a manifest + `Cargo.lock` change
  is exactly what the `deps` half catches: a stale workspace-hack, a now-unused
  direct dependency (`cargo machete`), and the yanked/advisory state (the
  `audit` recipe, not `deny`, owns advisories). `docs/aegis/INDEX.md` gets the
  entry for this spec and, at closeout, ADR 0007.

## 8. Non-goals

- Replacing `ctr` with hand-rolled counter mode (D2).
- Removing `ring` (D3).
- Retiring `b3.rs` (D4, no upstream API exists).
- `zstd → ruzstd` (D5, no build-target gain).
- Fixing the `lru`/`bincode`/`atomic-polyfill` advisories in-tree (upstream
  pins own them).
- Any change to wire formats, protocol behavior, DB schema, or UI.
- Fuzzing infrastructure, cargo-vet/cargo-supply-chain adoption, and the
  `xray-tui-route`/`xray-tui-config` safe-code postures.

## 9. Acceptance criteria

1. `oqs` is gone: no manifest mentions it, `cargo tree -i oqs-sys` is empty,
   and `cmake` (whose only consumer was liboqs) has left the resolved graph.
   Not claimed: `bindgen`/`clang-sys` still resolve through `turso_sdk_kit`'s
   build-dependency (its build script does not call them), and `cc` remains for
   `zstd-sys`/`lmdb-master-sys`/`libsqlite3-sys`/ring — so a full build still
   compiles those; what the swap removes is liboqs' own CMake + C/C++ +
   bindgen build.
2. `mlkem768_seed_derivation_matches_liboqs_and_go_stdlib` passes — the ek from
   a fixed 64-byte seed is byte-identical across liboqs, RustCrypto and Go
   1.27.1 `crypto/mlkem` (one test, two oracles — see §3).
3. The `crypto::mlkem` public API keeps its existing shape apart from the
   documented `SecretKey` seed change; every consumer compiles unchanged.
4. New pinned vectors pass: the NIST AES-256-CTR vector and the Go-generated
   relay/header mask vectors.
5. `chacha20 0.10.1` and `smartstring` no longer resolve; `atomic-polyfill`
   leaves the graph; `compact_str` has a single version in the lock.
6. `cargo audit` reports no yanked crate, and the `lru`/`bincode` entries carry
   recorded rationale.
7. Both crypto crates compile with `#![forbid(unsafe_code)]`, tests included.
8. `docs/crypto-dependencies.md` exists, is complete for the inventory in §6,
   and is referenced from `AGENTS.md` and `NATIVE_CORE.md`.
9. The full `just quality-gate` is green (both halves, including
   `hakari-check`, `deny`, `machete`, `outdated`, `audit`), and the e2e rows
   that exercise ML-KEM (`tls-pq` hybrid rows, both cores) stay green.

## 10. Verification

- `cargo nextest run -p xray-tui-tls -p xray-tui-native -p xray-tui-proto
  -p xray-tui-core` — unit and integration suites for every touched crate.
- `cargo tree -i oqs-sys`, `cargo tree -i atomic-polyfill`,
  `cargo tree -i chacha20`, `cargo tree -d` — dependency facts (counts and
  absence are the evidence).
- `cargo audit` — advisory state (the `audit` recipe owns it; the gate's
  `deny` recipe covers bans/licenses/sources).
- e2e: the ML-KEM hybrid TLS rows against real xray-core and sing-box
  (`native-e2e` feature), which are the interop proof for ek/ct/ss against Go.
  The `pq-enc` row stays `#[ignore]`d; its coverage is the KATs plus the
  Go-generated mask vectors (§3, §4).
- The full `just quality-gate` (`code` + `deps`) as the acceptance gate.
- The throwaway KAT harness is deleted after capture; the pinned vector is the
  artifact.

## 11. Risks

| Risk | Handling |
| --- | --- |
| `ml-kem` without `alloc` does not compile for a path we need | enable `alloc` (boxed returns), still copying into caller buffers; no behavior change |
| liboqs and RustCrypto disagree on the seed→key derivation | the KAT fails loudly; FIPS 203 semantics (RustCrypto) win, and the divergence is investigated before proceeding |
| RustCrypto's seed-only `DecapsulationKey` breaks a consumer | no in-repo consumer needs the expanded 2400-byte form; the deprecated expanded codec remains available as an escape hatch |
| RNG source changes (liboqs OpenSSL RNG → ring CSPRNG) | KEM randomness is internal to encapsulation; no wire impact; the `RingRng` adapter keeps one RNG provider |
| The Go-generated mask vectors contradict the current implementation | that is the defect the vectors exist to find; the fix is a mask correction, wire-compatible with xray |
| The VLESS account-encryption path has no real-peer e2e coverage (`pq-enc` is `#[ignore]`d), so a KEM or mask regression there would not be caught by a core run | ek KATs pin key derivation, the Go vectors pin the keystream, the in-repo client/server double covers the full handshake, and the gap is recorded in `docs/crypto-dependencies.md` instead of being assumed closed |
| `chacha20 0.10.2` or `compact_str` swap changes behavior | both are API-compatible for our usage; the touched suites cover the Shadowsocks codecs and the proto URL/identity paths |
| `forbid(unsafe_code)` breaks a macro expansion | verified zero `unsafe` today; a local `#[allow]` is not possible under `forbid`, so a genuine need would move that code out of the crypto crate instead |

## 12. Impact statement and ADR signal

Layers touched: TLS engine (`xray-tui-tls`), native protocol core
(`xray-tui-native` tests/rationale), protocol types (`xray-tui-proto`
dependency + alias), core (`xray-tui-core` dependency), build metadata
(`Cargo.toml` files, `Cargo.lock`, `xray-tui-hakari`, `.cargo/audit.toml`),
documentation (`NATIVE_CORE.md`, `AGENTS.md`, `docs/crypto-dependencies.md`,
this spec, ADR 0007, `docs/aegis/INDEX.md`).

Invariants preserved: FIPS 203 key bytes and the ML-KEM wire encoding, the
VLESS `xorpub`/`random` keystream, the TLS fingerprint profiles, the RFC 8879
decompression path, and the `TinyText` on-the-wire string shape.

ADR signal: yes — ADR 0007 records the accepted cryptographic dependency
posture (RustCrypto ML-KEM in place of liboqs, `ctr` and `ring` retained with
their reasons, hazmat sites governed by `docs/crypto-dependencies.md`), with the
real alternatives (`aws-lc-rs`, hand-rolled CTR, byte-context BLAKE3 via a
forked crate) and the retirement trigger (an upstream byte-context derive-key
API, a rustls RustCrypto provider, a fixed `lru 0.16.x`).

## Appendix — evidence commands

```bash
cargo audit                                      # 0 vulnerabilities, 6 warnings
cargo tree -i oqs-sys                            # oqs-sys 0.11.0+liboqs-0.13.0
cargo tree -i zstd --depth 2                     # tantivy-sstable <- tantivy-columnar
cargo tree -i lru@0.16.4                         # tantivy <- turso_core
cargo tree -i atomic-polyfill                    # heapless 0.7.17 <- postcard
rg -n "use ctr|Ctr128" crates/                   # the single ctr call site
rg -n "unsafe (impl|fn|extern|\{)" crates/xray-tui-tls crates/xray-tui-native  # 0 hits
```
