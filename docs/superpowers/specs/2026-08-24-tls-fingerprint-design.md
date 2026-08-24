# Flexible TLS Fingerprint Configuration (JA3/JA4) — Design

Date: 2026-08-24
Crate: `crates/xray-tui-tls`
Status: approved (design dialogue 2026-08-24)

## Goal

Replace the closed `BrowserProfile` enum with a flexible fingerprint
identity that can be selected by **operating system**, **device kind**,
**browser**, and optional **browser version**, built on a compositional
override layer, and backed by real-world JA4 data from
`thirdparty/ja4db-export` (frozen snapshot 2026-05-15).

Non-goals: TUI/settings integration (follow-up), hello-parameter
synthesis from JA4 hashes alone (impossible — JA4 is lossy), runtime
catalog regeneration.

## Decisions locked during brainstorming

1. **Layered model**: compositional builder is the foundation;
   catalog-backed selection is sugar over it.
2. **Strict resolution**: `resolve()` errors on unknown combinations,
   listing available alternatives. No silent nearest-match.
3. **Machinery + preset batch**: build the catalog/builder/oracle AND
   transcribe five new uTLS presets this round.
4. **Single identity**: one `Fingerprint` struct replaces
   `BrowserProfile` everywhere; per-browser constructors are replaced by
   `Fingerprint::default_for(Browser)` plus `Default` implemented as a
   `cfg`-selected platform default.

## Public API

```rust
/// The single fingerprint identity used across the crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fingerprint {
    pub browser: Browser,        // Chrome, Firefox, Safari, Edge, Brave,
                                 // Opera, SamsungInternet (+ new presets)
    pub version: Option<u16>,    // exact major; None = latest known
    pub os: Option<Os>,          // Windows, MacOs, Linux, Android, Ios
    pub device: Option<Device>,  // Desktop, Phone, Tablet
}

impl Fingerprint {
    /// Latest-known version/os/device defaults for a browser.
    pub fn default_for(browser: Browser) -> Self;

    /// Platform-sensible default (cfg-selected):
    /// Windows -> Chrome/Windows, MacOs -> Safari/macOS,
    /// Linux -> Firefox/Linux, Android -> Chrome/Android,
    /// unknown targets fall back to Chrome/Desktop.
    pub fn platform_default() -> Self;

    /// Strict resolution to a concrete hello.
    pub fn resolve(&self) -> Result<Resolved, FingerprintError>;
}

pub struct Resolved {
    pub identity: FingerprintIdentity, // resolved fields + matched catalog evidence
    pub spec: ClientHelloSpec,
}

impl Resolved {
    /// Full JA4 (`t13d…_hash1_hash2`) computed via the oracle.
    pub fn ja4(&self) -> String;
}
```

`BrowserProfile` is deleted. Existing call sites migrate:

- `client.rs`: `TlsMode::Plain { profile: Option<BrowserProfile> }`
  becomes `fingerprint: Option<Fingerprint>`; the default stays
  Chrome130-equivalent.
- `reality.rs`: `ProfileProvisioner(BrowserProfile)` generalizes to
  `SpecProvisioner { spec: ClientHelloSpec }` with
  `From<Fingerprint>` (resolve + expect — REALITY provisioning needs a
  concrete hello). `FixedChromeProvisioner` remains byte-identical; its
  golden test pins that.

Serde derives on `Fingerprint` so the TUI can persist choices later.

## Module layout

```
src/fingerprints/
  mod.rs            // re-exports, docs
  query.rs          // Fingerprint, Browser, Os, Device, default_for/platform_default
  builder.rs        // FingerprintBuilder (foundation layer)
  resolved.rs       // Resolved / FingerprintIdentity
  error.rs          // FingerprintError { Unknown { query, available }, InvalidOverride }
  catalog/
    gen.py          // stdlib-only generator (one-off, rerunnable)
    catalog_data.rs // generated static &[CatalogEntry]
profiles/           // transcription modules stay as-is (spec sources)
```

## Catalog

Generator input: `thirdparty/ja4db-export/csv/all_records.csv`
(~74.5k rows). Cleaning rules:

- Keep rows with a well-formed JA4 (`^t1[0-3]…`) **and** an identifiable
  application: non-empty `application` field OR parseable UA string.
- Parse UA → `(browser, major_version, os, device)` in Python (regex set
  covering Chrome/Edg/Firefox/Safari/Samsung); unparseable rows dropped.
- Dedupe on the full tuple; sum `observation_count`; carry `verified`.
- Expected yield: low thousands of entries — small embedded table, no
  runtime CSV dependency.

The dataset is frozen (upstream API dead); regenerating is a manual
script run when someone refreshes the CSV.

The catalog's runtime role is *evidence* (matched entries attached to
`Resolved.identity`, surfaced by audits) — never hello synthesis.

## Resolution

Hand-maintained table:
`(browser, os, device, version-range) -> transcription module`.
Fallback chain on resolve:

1. exact match (browser+version+os+device)
2. same browser+os+device, nearest lower version
3. same browser+device, any os
4. `Err(Unknown { available })`

Every table row must name an existing transcription module (compile-time
test).

## Builder (foundation layer)

```rust
let spec = FingerprintBuilder::from(fingerprint)   // resolves first
    .override_ciphers(&[...])        // full replacement; GREASE slot ok; empty rejected
    .override_extensions(&[...])     // full ordered replacement (order is fingerprint-critical)
    .curves(&[X25519, X25519Mlkem768]) // delegates to spec::apply_curve_preferences
    .alpn(&["h2", "http/1.1"])       // find-and-replace; errors if base lacks it
    .sig_algorithms(&[...])
    .grease(GreasePolicy::Keep)      // Keep (default) | Strip
    .build()?;                       // -> ClientHelloSpec
```

Build-time validation: duplicate extensions rejected; result re-parsed
via `hello::parse::parse_hello`; computed JA4 must be defined.

Curve overrides compose with REALITY keyshare injection because
`apply_curve_preferences` preserves GREASE keyshare slots
(`spec/mod.rs:138-145`).

## JA4 oracle

**Codec divergence discovered (2026-08-24, must fix first).** The existing
`crypto/fingerprint/ja4.rs` implements the obsolete September-2023 JA4
variant: first-cipher A-part, GREASE rendered as `g` and still counted,
no hash segments. Real-world JA4s (ja4db dataset, live `tls.peet.ws`)
use the final FoxIO scheme below. `ja4_a` is replaced outright — no
compatibility shims; its call sites (profile tests) migrate to the new
API in the same change.

The validated algorithm (reproduced byte-for-byte against a live
`tls.peet.ws` capture, `t13d3113h2_e8f1e7e78f70_db572f7c111e`):

- A-part: `{t|q}{tls_version:2}{d|i}{cipher_count:02}{ext_count:02}{alpn}`
  — counts EXCLUDE GREASE values entirely but INCLUDE SNI (`0000`),
  ALPN (`0010`) and padding (`0015`); ALPN rendered as first+last char of
  the first protocol (or `00`; non-ASCII → `99`).
- hash1 = sha256(`","`.join(sorted cipher ids, lowercase 4-hex,
  GREASE removed))[:12].
- hash2 = sha256(`","`.join(extension ids sorted, GREASE **and** SNI,
  ALPN, padding excluded) + `"_"` + signature-algorithm ids in hello
  order)[:12]. The padding exclusion follows peet.ws semantics (the
  de-facto grader standard); it is pinned by the known-vector test.

New API: `Ja3Fields::from_spec(&ClientHelloSpec)` + `full_ja4(fields)`
(and the raw segment strings for audits). `Resolved::ja4()` delegates.

## Preset batch (this round)

Transcribed by hand from `thirdparty/utls/u_parrots.go`:

| New profile | Fills gap |
|---|---|
| `safari_16` (macOS) | modern Safari desktop |
| `firefox_120` | current-gen Firefox desktop |
| `edge_106` | Edge beyond the 130 alias set |
| `ios_14` (Safari/iOS) | older iPhone hello shape |
| `android_11_okhttp` | Android app traffic (OkHttp ≠ Chrome mobile) |

Deliberately skipped: PSK/shuffle parrot variants (randomized
per-connection fields outside the declarative model), pre-2019 Chrome
versions (no camouflage value).

## Testing & verification (tier-1)

1. Existing per-profile tests keep passing after the cutover.
2. Resolution-table completeness (every row names a real module) +
   strict-error paths.
3. Oracle known vectors; then catalog cross-check: for each resolvable
   fingerprint whose identity exists in the catalog, computed JA4 ∈
   catalog entries for that identity.
4. Builder composition, duplicate-extension rejection, `.curves()` parity
   with `apply_curve_preferences`, GREASE strip policy.
5. REALITY golden-byte equality proves provisioner generalization is
   wire-identical.

Tier-2 (`examples/grader.rs`, peet.ws) unchanged; optionally assert
reported JA4 equals locally computed full JA4.

### rstest fingerprint cases

Fingerprint tests use `rstest` (workspace dep, conventions from
`xray-tui-native` e2e: `#[case::name(...)]` tables). A dedicated
`crates/xray-tui-tls/tests/fingerprints.rs` holds a case table of
`(Device, Os, Browser, Version)` combos with their expected JA4
(and JA3 where known), so common fingerprints are instantly verifiable:

```rust
#[rstest]
#[case::chrome_130_windows(Fingerprint::chrome130_windows(), "t13d1514h2_8daaf6152771_…")]
#[case::firefox_120_linux(  Fingerprint::firefox120_linux(),   "…")]
// ...
fn ja4_matches_expected(#[case] fp: Fingerprint, #[case] expected: &str) { … }
```

**Determinism rules** (GREASE is drawn randomly per hello):

- Fixed expected values are **JA4 only** — the JA4 spec mandates
  GREASE removal before hashing, so full-JA4 is deterministic across
  runs for every profile.
- **JA3 is not stable for GREASE-carrying profiles** (Chrome/Edge/Brave/
  Opera families): classic JA3 hashes the wire bytes including GREASE.
  Where a case pins JA3, the fixture must use one of:
  (a) a seeded `FixedRandom`-style RNG exactly like the existing
  golden-hello tests (byte-reproducible hello ⇒ byte-reproducible JA3),
  or (b) a documented `ja3_grease_stripped()` variant that removes
  GREASE ids before encoding (deterministic, but *not* the on-wire JA3 —
  named accordingly so nobody mistakes it for classic JA3).
- GREASE-free profiles (Firefox/Safari families) may pin plain JA3
  directly.

## Risks

- Catalog rows ≠ specs we can emit; the two connect only through the
  hand-maintained resolution table. Mitigation: compile-time completeness
  test + honest strict-error UX.
- UA-parsing coverage in gen.py determines catalog yield; rows that fail
  parsing are dropped, never guessed.
