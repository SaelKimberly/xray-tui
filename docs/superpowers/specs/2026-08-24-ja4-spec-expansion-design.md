# Design: JA4-Driven Fingerprint Expansion

Date: 2026-08-24
Status: Approved (design dialogue 2026-08-24)
Scope: `crates/xray-tui-tls` fingerprint subsystem
References: `docs/superpowers/specs/2026-08-24-tls-fingerprint-design.md` (predecessor), `thirdparty/ja4db-export`, `thirdparty/impersonator`, `thirdparty/curl-impersonate`

## Goal

Expand the TLS mimicry library from 17 hand-transcribed profiles to every identity derivable from the ja4db-export corpus, with machine-verifiable JA4 fidelity, while shrinking per-profile declaration cost.

## Ground Truth and Fidelity Contract

`csv/ja4_fingerprint_string.csv` holds raw (unhashed) JA4 components:
`<ja4_a>_<ciphers-hex,comma>_<exts-hex,comma>_<sigalgs-hex,comma>` plus an optional ALPN segment. 74,486 rows deduplicate to **998 unique TLS-1.3 fingerprints**.

JA4 is lossy by construction. From the corpus we recover exactly:

| Recovered | Lost |
|---|---|
| cipher set | cipher wire order (JA4 sorts) |
| extension id set | extension wire order (JA4 sorts) |
| signature algorithm set + order | per-extension contents (key_share groups, supported_versions, GREASE values, padding, compress_certificate) |
| version letters, ALPN first protocol | full ALPN list |

**Fidelity contract (user decision): two tiers.**

1. **JA4-faithful tier (generated)** — every generated entry provably hashes to its source JA4 through our own `full_ja4` codec; extension order and contents are synthesized from family templates. Documented limitation: JA4/JA3-level DPI sees the expected browser; byte-level inspection beyond JA4 may differ.
2. **Wire-exact tier (transcribed)** — the existing 17 hand-written uTLS-derived profiles remain untouched and authoritative where identities overlap.

## Data Pipeline

New generator `gen_specs.py` alongside gen.py in the generator workspace:

1. Parse raw strings into `{ja4_a, ciphers[], exts_sorted[], sigalgs_ordered[], alpn_first}`.
2. Recompute the JA4 hash from components in Python (mirror of `crypto/fingerprint/ja4.rs`, final FoxIO scheme); hard-fail on any mismatch — the self-check validates both the parser and the mirror.
3. ua-parser (`ua-parser[regex]`) over `user_agent_string` in **both** CSVs → `Browser/browser_major`, `Os/os_major`, `Device`. Keep only rows where all three resolve.
4. Join: computed JA4 ↔ `ja4_fingerprint.csv` hashed keys, enriching attribution in both directions (raw file gives components; hashed file gives labels, observation_count, verified flag).
5. Skip non-`t13d` rows (engine is TLS-1.3-only; precedent: android_11_okhttp ruling).
6. Dedup to one spec per unique raw fingerprint. Identity collision (same Browser+major+Os+major+Device mapping to multiple fingerprints): winner = highest observation_count, verified-first. Losers dropped; drop counts recorded in the manifest.
7. Emit `specs_manifest.json` (committed, reproducible intermediate) + generated Rust.

## Family Templates (Synthesis Layer)

Python template registry keyed by family: `chrome_desktop`, `firefox`, `safari`, `chrome_android`, `safari_ios`, `okhttp`, fallback `ascending`.

Each template supplies what the CSV cannot:
- **Extension wire order**: canonical order for the family, filtered to the row's ext-id set; unknown ids appended ascending at the end.
- **Extension contents**: per-ext-id constructor rules (supported_versions list, key_share groups, GREASE value/position, padding rule, compress_certificate algorithms) — the same rules proven by the 17 transcriptions.
- **Cipher wire order**: family-canonical sequence, filtered to the row's set.
- Compression byte and record-layer defaults.

Fallback template orders everything ascending and emits minimal standard contents; entries built from it are flagged in the manifest as low-fidelity.

## Generated Code Layout

```
crates/xray-tui-tls/src/profiles/generated/
  chrome.rs      # chrome_desktop family
  firefox.rs
  safari.rs
  ...
```

- Each module: `#[rustfmt::skip]`; one entry = 1–2 lines via a new `spec!` declarative macro (ciphers/exts/sig-algs lists inline).
- Per-entry doc comment: source JA4 string + observation count.
- Hand-written profiles stay in `profiles/*.rs` unchanged; `profiles/mod.rs` aggregates both tiers. Module placement is the tier marker (documented in NATIVE_CORE.md).

## Resolution Integration

Strict-error behavior unchanged; the alternatives listing in errors shows the first 8 matches plus a total count.

Merge rule: **hand-written row wins** when generated and transcribed rows claim the same identity/band overlap (wire-exact precedence).

## Verification

- **Offline gate (hard, CI)**: for every generated entry, build the ClientHello through the engine and assert `full_ja4() == source JA4`. Exercises parser → template → spec → hello → codec end-to-end on all identities without network.
- **Live sweep (manual)**: extend `examples/grader.rs` / ignored peet.ws test to iterate the generated roster; results recorded in a committed verification report (`docs/` or SDD notes). Not a merge blocker.
- Determinism rstest tables: sampled per family/version band (not all entries), same GREASE-nondeterminism rules as before.
- Generator unit tests: raw-string parsing, hash-mirror self-check, join logic, ambiguity policy.

## Non-Goals

- TLS-1.2 handshake support (rows skipped; deferred).
- HTTP/2 fingerprint settings (impersonator covers them; out of scope here).
- Wire-exactness for generated entries; enriching generated specs from impersonator/curl-impersonate sources is a future upgrade path, not part of this design.

## Risks

- Template misattribution (library impersonating Chrome labeled as Chrome): mitigated by observation_count weighting; inherent to the tier and documented.
- Error-message bloat from hundreds of alternatives: capped listings.
- Binary size: hundreds of small const structs — negligible.
