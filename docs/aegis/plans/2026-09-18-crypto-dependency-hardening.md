# 2026-09-18 — Cryptographic dependency hardening: ML-KEM on RustCrypto, hazmat governance

Parent spec: `docs/aegis/specs/2026-09-18-crypto-dependency-hardening-design.md`
(approved 2026-09-18, including the `ctr`-kept / ring-retained / zstd-deferred
scope decisions).

Aegis visibility: this is a manifest + `Cargo.lock` change touching a wire
contract (ML-KEM key derivation, xray's XOR mask), so the plan's value is in the
ordering — the characterization vectors and the ek KAT must exist **before**
`oqs` is removed, and the acceptance gate must be the full `quality-gate`
because the `deps` half is what catches a stale workspace-hack or a now-unused
direct dependency.

## Goal

Replace the liboqs-backed ML-KEM-768 primitive with RustCrypto `ml-kem 0.3.2`
(removing the CMake/libclang/C-toolchain requirement from `xray-tui-tls`), keep
the `ctr` hazmat site with a documented rationale and pinned Go-generated wire
vectors, clear the actionable dependency-hygiene findings, and make the crypto
surface self-governing (`forbid(unsafe_code)` + `docs/crypto-dependencies.md`).
No wire-format, schema, or user-facing behavior changes.

## Architecture

```
crates/xray-tui-tls/src/crypto/mlkem.rs   ← the ONLY oqs consumer (Slice 1)
        ├── reality/mod.rs, handshake/mod.rs           (hybrid TLS key share)
        ├── xray-tui-native/.../vless/encryption/mlkem.rs  (account encryption + ctr mask)
        └── src/e2e/config.rs::mlkem_enc_pair          (seed → ek)
crates/xray-tui-proto/...urlx::TinyText   ← the only smartstring seam (Slice 3)
crates/xray-tui-core/Cargo.toml:63        ← postcard default feature (Slice 3)
docs/crypto-dependencies.md               ← new authority for hazmat sites (Slice 4)
```

Dependency direction is unchanged: protocol crates → TLS engine → primitives.
No new crate enters the workspace except `ml-kem` (replacing `oqs`) and
`compact_str`/`subtle` (already resolved transitively).

## Tech Stack

Rust 2024 (workspace `rust-version = 1.98`), `ml-kem 0.3.2` (no_std-capable,
`unsafe_code = "deny"`), `rand_core 0.10.1`, ring 0.17, RustCrypto (aes/chacha/
hmac/sha2/hkdf), `subtle 2.6.1`, `compact_str 0.9`, tokio, cargo-hakari, Go
1.27.1 (vector generation only, never a build dependency).

## Baseline / Authority Refs

- `docs/aegis/specs/2026-09-18-crypto-dependency-hardening-design.md` — scope,
  decisions D1–D6, acceptance criteria.
- `AGENTS.md` — key source files list, decisions 17/19 (liboqs mentions),
  verification section (`just quality-gate` subsets).
- `NATIVE_CORE.md` — native/TLS crate purpose and verification tiers.
- `docs/database-manual-sql.md` — the authority-doc shape
  `docs/crypto-dependencies.md` mirrors.
- `thirdparty/Xray-core/proxy/vless/encryption/{xor.go,client.go,server.go}` —
  the mask semantics the vectors pin.

## Compatibility Boundary

- **Preserved**: FIPS 203 key/ciphertext/shared-secret encodings and the ek
  bytes derived from a seed; the VLESS `native`/`xorpub`/`random` keystream;
  every TLS fingerprint profile; the RFC 8879 decompression path; `TinyText`'s
  on-the-wire (serde) shape.
- **Changed**: `crypto::mlkem::SecretKey` becomes seed-based (2400-byte
  expanded form dropped; only the in-crate server test double uses either form).
- **Retired**: the `oqs` dependency and its build requirements.

## TDD Route

Mode `auto` (no `~/.config/aegis/config.toml`; no explicit strict request).
Decision: **strict** for T1, T2, T4 (wire-critical producer/consumer: the KAT
and the Go mask vectors are authored and run **before** the implementation
change and must stay green after it; a mask mismatch is a legitimate RED).
**Light/skipped** for T3, T5, T6, T7 (no behavior change — the compiler, the
existing suites, and the full gate are the verification; T5's `ct_eq` swap gets
one focused behavior test because its contract is a security property).
Authority: the approved spec's §9–§11. Verification: per-task commands below.

## Plan Basis

- Requirement Ready Check: ready — the approved spec fixes scope, non-goals,
  acceptance criteria (§9) and verification (§10); no open decision-changing
  unknown remains (zstd deferred by user, ctr/ring retained by user).
- Change Necessity: a dependency swap cannot be expressed by docs/config alone —
  the primitive implementation, its key types, and the manifest must change;
  minimum boundary = `crypto/mlkem.rs` + the two manifests + one test module
  per wire pin.
- Ripple Signal Triage: fires (producer/consumer wire contract + dependency
  direction + build toolchain). Canonical owner of ML-KEM: `crypto::mlkem`;
  consumers listed in Architecture. Source-of-truth: the wrapper's public API
  stays the single seam; no fallback or compat branch is added (`oqs` is
  deleted outright, per D1).
- Plan Pressure Test: owner fit ok (no new owner; `ml-kem` replaces `oqs`
  inside the existing wrapper); no higher-level simplification available;
  verification is exact (vectors + counts + e2e); tasks are executable as
  written. Proceed.
- Execution context: current branch/workspace (no worktree; no concurrent
  checkout needed).

## Files

| File | Task | Change |
| --- | --- | --- |
| `crates/xray-tui-tls/src/crypto/mlkem.rs` | T1, T2 | KAT tests; primitive rewrite on `ml-kem`; `RingRng` |
| `crates/xray-tui-tls/Cargo.toml` | T2, T4 | drop `oqs`, add `ml-kem` + `rand_core` (T2 — `RingRng` names `rand_core::TryRng`/`CryptoRng`, so it must be a direct dep; 0.10.1 is already in the lock); add `subtle` (T4) |
| `crates/xray-tui-tls/src/handshake/mod.rs` | T2 | `mlkem_sk` type follows the `SecretKey` change |
| `crates/xray-tui-tls/src/reality/verify.rs` | T4 | `ct_eq` → `subtle::ConstantTimeEq` |
| `crates/xray-tui-tls/src/lib.rs` | T6 | `#![forbid(unsafe_code)]` |
| `crates/xray-tui-native/src/protocol/vless/encryption/mlkem.rs` | T3 | rationale + NIST/Go vectors |
| `crates/xray-tui-native/src/lib.rs` | T6 | `#![forbid(unsafe_code)]` |
| `crates/xray-tui-proto/Cargo.toml`, `src/urlx/mod.rs` | T5 | `smartstring` → `compact_str` |
| `crates/xray-tui-core/Cargo.toml` | T5 | `postcard` default-features off |
| `.cargo/audit.toml` | T5 | `lru`/`bincode` rationale comments |
| `docs/crypto-dependencies.md` | T6 | new authority doc |
| `AGENTS.md`, `NATIVE_CORE.md` | T6, T7 | cross-refs; liboqs → RustCrypto ML-KEM |
| `Cargo.lock`, `crates/xray-tui-hakari/Cargo.toml` | T7 | regeneration |
| `docs/aegis/INDEX.md`, `docs/aegis/adr/0007-*.md` | T9 | records |

## Sequencing

T1 → T2 → T3 → T4 → T5 → T6 → T7 → T8 → T9. Manifest/lock edits (T2, T5) are
serialized on one owner because `Cargo.lock` and `crates/xray-tui-hakari` are
shared global state; `cargo hakari generate` runs once, in T7, after the last
manifest edit. T3 touches only the native crate and is order-independent, but
stays sequential because the acceptance gate is centralized.

## Tasks

### T1 — Capture both ML-KEM KAT oracles and pin them (tls) — strict

Purpose: fix the invariant before the implementation moves.

1. Throwaway harness (temp file outside git or a `#[test]` deleted in T2, e.g.
   `crates/xray-tui-tls/src/crypto/mlkem.rs` test module + a `println!` run):
   with the current liboqs implementation, print the hex of
   `Mlkem768::keypair_from_seed(&SEED).0.as_bytes()` for
   `SEED = [0u8; 64]` and for a second non-trivial seed
   (`SEED2[i] = i as u8`).
2. Go oracle (independent of both): a `/tmp` Go module using the stdlib
   `crypto/mlkem` (`NewDecapsulationKey768(seed)`, `EncapsulationKey()`,
   `Bytes()`), printing the ek hex for the same two seeds. Record the exact
   program text in the test comment as the generator.
3. Add `mlkem768_seed_derivation_matches_liboqs_and_go_stdlib` (ONE test: the
   two oracles produced byte-identical vectors, so a second assertion on the
   same constant would be padding) with the captured vectors and the generator
   in its comment; run it now (green baseline) — the liboqs vector is the
   pre-swap anchor.

Proof: `cargo nextest run -p xray-tui-tls mlkem768_seed_derivation` → 2 passed.
Do **not** assert on decapsulation-key bytes (expanded 2400 vs seed 64).

### T2 — Swap to `ml-kem 0.3.2` and delete `oqs` (tls) — strict

Files: `crates/xray-tui-tls/src/crypto/mlkem.rs`,
`crates/xray-tui-tls/Cargo.toml`, `crates/xray-tui-tls/src/handshake/mod.rs`.

Change:
- Manifest: `ml-kem = { version = "0.3.2", default-features = false, features =
  ["zeroize"] }`; delete the `oqs` line and its comment.
- `PublicKey`/`Ciphertext`/`SharedSecret` keep their byte-array shape;
  `SecretKey` holds `DecapsulationKey<MlKem768>` with `from_seed`/`to_seed` and
  the existing redacting `Debug`.
- `generate_keypair` → `MlKem768::generate_keypair()`; `keypair_from_seed` →
  `kem::FromSeed::from_seed(&B64::from(*seed))`; `encapsulate_into` →
  `EncapsulationKey::new(&Key::<_>::from(pk.0))` +
  `Encapsulate::encapsulate_with_rng(&mut RingRng)` written into the caller
  buffers (no allocation); `decapsulate` → `Decapsulate::decapsulate`.
- `RingRng`: private adapter implementing `rand_core::TryRng` (`type Error =
  core::convert::Infallible`, `try_fill_bytes` over `ring::rand::SystemRandom`,
  `expect("ring CSPRNG fills")` as elsewhere) plus `CryptoRng`.
- `MlkemError` keeps its variants; map `InvalidKey` to the existing
  length/validation errors.
- If `default-features = false` fails to build a needed path, enable `alloc`
  (recorded in the task note) — behavior unchanged either way.

Proof: `cargo nextest run -p xray-tui-tls` green (includes both T1 KATs);
`cargo tree -i oqs-sys` empty; `rg -n "oqs" crates/*/Cargo.toml` empty;
delete the throwaway harness from T1.

### T3 — VLESS mask: rationale + pinned wire vectors (native) — strict

Files: `crates/xray-tui-native/src/protocol/vless/encryption/mlkem.rs`.

Change:
- Rationale block on `new_ctr` (and on the `XorConn` state fields) stating: the
  `ctr` banner concerns unauthenticated ciphertext; this is xray's
  `crypto/cipher.NewCTR` mask over `blake3.DeriveKey("VLESS", key)`, integrity
  provided by the NFS AEAD + `hash32` chain; `Ctr128BE` ≡ Go's 16-byte
  counter block.
- Tests (Go 1.27.1 generator, module cache already holds
  `lukechampine.com/blake3@v1.3.0/1.4.1`, so it works offline). As implemented
  (the plan's third vector folded into the first two: the header stream is
  `new_ctr` output consumed at 5-byte steps, which `random_mode_masks_headers_only`
  already pins structurally, so composing it with the Go-pinned `new_ctr` vector
  covers it without a third constant):
  1. `nist_sp800_38a_aes256_ctr_matches_the_published_vector` — pins
     `Ctr128BE<Aes256>` against NIST F.5.5 (the counter semantics that make the
     mask wire-compatible).
  2. `xray_new_ctr_keystream_matches_go` — the composition
     (`blake3.DeriveKey("VLESS", k)` then AES-256-CTR) against Go bytes.
  3. `xray_relay_chain_masking_matches_go` — the `hash32` region plus the
     CONTINUING keystream offset into the next block, against Go bytes.
- The rationale block lives on `new_ctr` (the single construction point); the
  `CommonConn` header-mask call site points at it and at
  `docs/crypto-dependencies.md`.

Proof: `cargo nextest run -p xray-tui-native -E 'test(nist_sp800_38a) | test(xray_new_ctr) | test(xray_relay_chain)'`
→ green (nextest takes one filter positional; the `-E` expression is the OR form).
A RED result is a real defect: fix the mask, keep it wire-compatible, and record
the finding in the task note.

### T4 — `ct_eq` → `subtle::ConstantTimeEq` (tls) — light

Files: `crates/xray-tui-tls/src/reality/verify.rs`,
`crates/xray-tui-tls/Cargo.toml`.

Change: delete the local `ct_eq`; compare with
`subtle::ConstantTimeEq::ct_eq`; add `subtle = "2"` to the manifest. No new test:
the behavior is already pinned by `verify_certificate_hmac_with_real_cert`,
`…_invalid_signature` and `…_invalid_der` (a positive, a forged tag and a
malformed input), and a test of `subtle`'s own slice semantics would be padding.

Proof: `cargo nextest run -p xray-tui-tls reality` green.

### T5 — Dependency hygiene (proto, core, tls, audit config) — skipped TDD

Change:
- `cargo update -p chacha20 --precise 0.10.2` (yanked 0.10.1 → clean).
- `crates/xray-tui-proto`: `compact_str = { version = "0.9", features =
  ["serde"] }` in place of `smartstring`; `src/urlx/mod.rs:5` →
  `pub type TinyText = compact_str::CompactString;`. `0.9` matches the version
  already in the lock via `ratatui-core` (no second major). If any call site
  needs a `SmartString`-only method, fix the call site (none expected — the
  alias is the only seam).
- `crates/xray-tui-core/Cargo.toml:63`: `postcard = { version = "1",
  default-features = false, features = ["alloc"] }` (we only use
  `to_allocvec`/`from_bytes`).
- `.cargo/audit.toml`: comments recording the `lru 0.16.4` triage (reachable via
  `turso_core → tantivy`, no `0.16.x` fix exists, `tantivy` pins `^0.16.3`,
  never called by us) and `bincode 1.3.3` (via `heed-types`); both stay in
  `informational_warnings`, not `ignore`.

Proof: `cargo nextest run -p xray-tui-proto -p xray-tui-core` green;
`cargo tree -i atomic-polyfill` empty; `awk` over `Cargo.lock` shows one
`compact_str` and `chacha20 0.10.2`; `cargo machete --with-metadata
--skip-target-dir` clean.

### T6 — Governance: safe-code lint + hazmat authority doc — skipped TDD

Files: `crates/xray-tui-tls/src/lib.rs`, `crates/xray-tui-native/src/lib.rs`,
new `docs/crypto-dependencies.md`, `AGENTS.md`, `NATIVE_CORE.md`.

Change:
- `#![forbid(unsafe_code)]` at the top of both crate roots.
- `docs/crypto-dependencies.md`: for each site — cause, the wire/protocol
  constraint that forces it, the evidence that pins it, the retirement trigger,
  and the accepted-advisory list. Inventory per spec §6 (b3.rs, `ctr`, the zstd
  C decoder with its bounded-decode guard and the `ruzstd` trade-off, the
  hand-rolled TLS 1.3 key schedule/transcript, the TLS 1.2 PRF path, QUIC
  header protection, protocol-mandated legacy primitives).
- `AGENTS.md` (Key Source Files) and `NATIVE_CORE.md` reference the new doc the
  way `docs/database-manual-sql.md` is referenced.

Proof: both crates build with the lint (`cargo check -p xray-tui-tls -p
xray-tui-native --all-targets`); the doc lists every inventory item.

### T7 — Docs sync, hakari regeneration, full gate — skipped TDD

Change: `NATIVE_CORE.md` + `AGENTS.md` decisions 17/19 and any `Cargo.toml`
comment describing the oqs build move to the RustCrypto backend;
`cargo hakari generate`; review `deny.toml`'s windows-gnu note.

Proof: `just quality-gate` (both halves) green.

### T8 — Live verification with real cores — no source edits

Run the ML-KEM e2e rows against the local cores
(`XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin`, `--features native-e2e`):
`tcp_tls_pq` (vless), `tcp_aes128gcm_tls_pq` (vmess) — the interop proof for
ek/ct/ss against Go. Record the pass/ignore counts (the `reality-pq` and
`pq-enc` rows are expected `#[ignore]`d). If a core binary is missing, state it
instead of claiming the row passed.

### T9 — Closeout records

`docs/aegis/adr/0007-cryptographic-dependency-posture.md` (RustCrypto ML-KEM in
place of liboqs; `ctr` and `ring` retained with reasons; hazmat governance;
real alternatives and retirement triggers) and `docs/aegis/INDEX.md` entries for
this plan and the ADR; flip the spec's status line to implemented.

## Risks

| Risk | Handling |
| --- | --- |
| `ml-kem` without `alloc` does not build a needed path | enable `alloc`; behavior unchanged; recorded in T2 |
| liboqs ≠ RustCrypto seed derivation | two KATs (liboqs + Go stdlib) fail loudly; FIPS 203 semantics win; investigate before proceeding |
| Go vector generation unavailable | Go 1.27.1 + cached `lukechampine.com/blake3` verified present; fallback is a compose-from-primitives vector, recorded as weaker |
| Mask vectors go RED | that is a real defect; fix wire-compatibly and record it |
| Manifest/lock drift after edits | `cargo hakari generate` + full `just quality-gate` in T7 is the gate |
| pq-enc path has no real-peer coverage | KATs + Go mask vectors + in-repo client/server double; the gap is recorded in `docs/crypto-dependencies.md` |

## Retirement

`oqs`/`oqs-sys`/liboqs is deleted in T2 (no compat branch, no feature flag).
`smartstring` is deleted in T5. Both are verified by absence in the manifests
and the resolved graph, not by a deprecation path. `ctr`, `zstd`, `ring`, and
`b3.rs` are deliberately retained with recorded causes and revisit triggers in
`docs/crypto-dependencies.md`.

## Execution Route

`inline` — the manifest and `Cargo.lock` ownership is global and single-owner,
each task is small, and one centralized acceptance gate (`just quality-gate` +
e2e) covers all of them; delegating would add coordination cost without
parallelism gain. Fallback: subagent-driven if an independent slice (T3) turns
out to need a separate checkout.
User confirmation required: no.
