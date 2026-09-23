# 0007 — Cryptographic dependency posture: RustCrypto ML-KEM, `ctr` and ring retained, hazmat sites governed

Date: `2026-09-18`
Status: `accepted`
Supersedes: nothing
Related: `specs/2026-09-18-crypto-dependency-hardening-design.md`,
`plans/2026-09-18-crypto-dependency-hardening.md`, `docs/crypto-dependencies.md`,
`NATIVE_CORE.md`, decision 17 (TLS engine), decision 19 (secret wiping),
decision 20 (native core).

## Context

The crypto core of `xray-tui-tls` + `xray-tui-native` had five open problems at
once:

1. **ML-KEM came from liboqs** (`oqs` → `oqs-sys`), whose vendored build needs
   CMake, libclang (bindgen) and a C/C++ toolchain. `x86_64-pc-windows-gnu` is a
   declared target (`rust-toolchain.toml`, `deny.toml`), and the toolchain may
   simply not be there. The only consumer was `crypto/mlkem.rs`.
2. **`ctr` carries a hazmat banner** ("ciphertexts are not authenticated"). Its
   single use is xray's `XorConn` keystream: AES-256-CTR keyed by
   `blake3.DeriveKey("VLESS", key)`, masking relay blocks (`xorpub`/`random`) and
   the 5-byte record headers (`random`).
3. **The VLESS mask had no peer-independent test.** The existing round-trip and
   structural tests compare this implementation against its own mirror, so a
   wrong mask passes on both ends; the only true peer coverage sits in an
   `#[ignore]`d e2e row.
4. **`b3.rs` is a hand-rolled BLAKE3** derive-key mode (481 lines) in a
   production path, because the `blake3` crate's derive-key API takes `&str`
   contexts while xray's are binary.
5. **Dependency hygiene and triage**: `chacha20 0.10.1` was yanked upstream,
   `smartstring` was an unmaintained *direct* dependency, `postcard`'s default
   feature dragged in `heapless 0.7`/`atomic-polyfill`, `lru 0.16.4` is
   unsound (reachable via `turso_core → tantivy`), `bincode` is unmaintained
   through `heed-types`, a hand-rolled constant-time compare sat in
   `reality/verify.rs`, and no document said which hand-rolled crypto sites are
   intentional.

## Decision

1. **ML-KEM is RustCrypto `ml-kem 0.3.2`.** `oqs` is deleted, not
   feature-gated, so no build path retains the CMake/libclang requirement.
   `default-features = false, features = ["zeroize"]` keeps ciphertexts and
   shared secrets inline (the allocation-free `encapsulate_into` contract) and
   wipes the decapsulation key. The swap is pinned by
   `mlkem768_seed_derivation_matches_liboqs_and_go_stdlib`: the ek for a fixed
   64-byte seed was captured from the liboqs implementation before removal and
   independently reproduced by Go 1.27.1's stdlib `crypto/mlkem` — identical.
   The assertion is on the **encapsulation key**, not the decapsulation key:
   liboqs serialized a 2400-byte expanded key while `ml-kem` serializes the
   64-byte FIPS 203 seed, so dk bytes are not comparable across the swap.
2. **One RNG provider.** A `rand_core` 0.10 adapter (`RingRng`) over ring's
   `SystemRandom` feeds `encapsulate_with_rng`; ml-kem's `getrandom` feature
   stays off, so ML-KEM key material comes from the same provider as every
   other key byte in the crate.
3. **`ctr` is retained, documented and pinned.** The banner describes
   unauthenticated *encryption*; this site is a keystream mask whose integrity
   comes from the NFS AEAD plus the `hash32` relay chain, and no RustCrypto API
   offers raw AES-CTR otherwise. The wire is now pinned against Go (NIST
   SP 800-38A F.5.5 for the counter-mode semantics, `new_ctr`'s composition, and
   the relay-chain continuation offset across the `hash32` region), because a
   self-consistent round-trip is not evidence.
4. **ring stays.** rustls (ring backend), quinn (`rustls-ring`), hickory and
   reqwest (`rustls-no-provider` with ring installed in code) all land on it;
   replacing it means a custom rustls `CryptoProvider` plus RustCrypto X.509
   signature verification (RSA/ECDSA/EdDSA that `rustls-webpki` provides only
   through ring/aws-lc-rs). That is a TLS-provider migration — a separate epic,
   not a dependency swap.
5. **`b3.rs` stays**, recorded as a hazmat site. blake3 1.8.7 exposes
   `hash_derive_key_context(&str)` / `ContextKey` / `new_from_context_key(&[u8; 32])`
   only, and its `guts` module is deprecated with a hash-mode-only `ChunkState`
   — no upstream API computes a DERIVE_KEY_CONTEXT hash over binary bytes.
6. **Hygiene**: `chacha20` → 0.10.2 (un-yanked); `smartstring` → `compact_str`
   0.9 behind the existing `TinyText` alias (maintained, already in the graph,
   no second major); `postcard` `default-features = false` (drops
   `heapless`/`atomic-polyfill`); `subtle::ConstantTimeEq` replaces the
   hand-rolled compare. `lru`, `bincode`, `rustls-pemfile` and `rsa` are
   recorded as accepted with their reachability and re-evaluation triggers, kept
   **visible** in `cargo audit` rather than silenced in `ignore`.
7. **Governance**: `#![forbid(unsafe_code)]` on both crypto crates (verified
   zero `unsafe` before enabling; a future genuine need moves code out rather
   than adding an exception), and `docs/crypto-dependencies.md` becomes the
   authority for every non-default crypto site — cause, independent oracle,
   wire-compatibility statement, retirement trigger — mirroring
   `docs/database-manual-sql.md`.
8. **`zstd → ruzstd` deferred.** The build-toolchain argument is void
   (`zstd-sys` also arrives via `turso_core → tantivy-sstable → zstd`, and
   `libsqlite3-sys`/`lmdb-master-sys`/ring need `cc` anyway). The
   trust-boundary argument is recorded with its trigger: bundled libzstd is
   heavily fuzzed, `ruzstd` is young, and the decode is bounded by
   `decompress_bounded` + `MAX_CERT_BYTES`.

## Alternatives rejected

| Alternative | Why not |
| --- | --- |
| Keep liboqs behind a feature/native-vs-C switch | Two ML-KEM implementations is the duplicate-owner shape this project retires, and the C toolchain would stay on the default path for the target that cannot assume it |
| Hand-roll AES-CTR over `aes` to drop the hazmat-labelled `ctr` | Trades a maintained RustCrypto mode crate for in-repo counter mode, which `docs/crypto-dependencies.md` exists to discourage; the banner is about authenticating ciphertexts, not about this mask |
| Replace ring with RustCrypto | Large TLS-provider migration (custom rustls `CryptoProvider` + X.509 signature verification); no security win at this threat model |
| `aws-lc-rs` as the workspace provider | C/asm build, worse for `x86_64-pc-windows-gnu`, no win over ring here |
| Fork or patch `blake3` for byte-context derive-key | Maintaining a fork of a security crate is a larger liability than a 481-line, Go-vector-pinned implementation |
| `ruzstd` today | Younger and less fuzzed than the bundled libzstd, and it removes no C toolchain |
| Keep the hand-rolled `ct_eq` | `subtle` was already in the resolved graph and is the audited primitive |
| Keep `smartstring` / ignore the yank | Unmaintained direct dependency with a maintained, already-resolved replacement reachable through one type alias |

## Consequences

- The ML-KEM path itself no longer needs CMake, libclang or a C++ compiler, and
  `cmake` has left the resolved graph. What remains — stated so the claim is not
  oversold — is `bindgen`/`clang-sys` as `turso_sdk_kit`'s build-dependency
  (whose build script does not call them) and `cc` for `zstd-sys`,
  `lmdb-master-sys`, `libsqlite3-sys` and ring's asm. `cargo hakari`
  regenerated (liboqs' `clang-sys` entry left the workspace-hack).
- ML-KEM secret keys are seed-shaped (64 bytes): a 2400-byte expanded key no
  longer exists in the API. Nothing in-tree used that form.
- A regression in the VLESS mask, the ML-KEM seed derivation, or the REALITY
  HMAC comparison now fails a pinned vector or the audited `subtle` path instead
  of agreeing with a mirror.
- The VLESS account-encryption path's real-peer e2e coverage is now GREEN
  (`tcp_pq_enc`, 2026-09-23): the row was `#[ignore]`d on a mis-diagnosis — the
  PQ wire was byte-correct and the defect was the record layer raising
  `UnexpectedEof` on a CLEAN end of stream at a field boundary (so a completed
  response was discarded). `CommonConn::poll_read` mirrors Go's `io.ReadFull`
  EOF rules, pinned hermetically by `eof_at_a_record_boundary_is_clean`. The
  KATs + Go vectors still cover the mask and the seed derivation.
- Accepted advisories remain visible; their re-evaluation triggers are the
  turso/tantivy bump (`lru`), the heed bump (`bincode`), and the driver-chain
  removal (`rsa`, `rustls-pemfile`).
- Recorded retirement triggers: an upstream byte-context derive-key API
  (`b3.rs`), a pure-Rust rustls provider (`ring`), `ruzstd` maturity (`zstd`).
