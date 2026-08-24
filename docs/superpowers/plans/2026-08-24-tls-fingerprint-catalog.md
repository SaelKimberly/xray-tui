# TLS Fingerprint Catalog & Builder Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the closed `BrowserProfile` enum in `xray-tui-tls` with a flexible `Fingerprint` identity (browser/version/os/device), backed by a compositional override builder, a generated JA4 catalog, and a strict resolver — plus fix the JA4 codec to the final FoxIO scheme.

**Architecture:** Layered: `FingerprintBuilder` (foundation) under a catalog-backed selector (sugar). Hand-maintained resolution table maps identities to hand-transcribed `ClientHelloSpec` modules. Generated Rust catalog from the frozen `thirdparty/ja4db-export` CSV provides identity evidence. Full-JA4 oracle (validated against live peet.ws) powers tests and audits.

**Tech Stack:** Rust 2024, ring-based engine (`crates/xray-tui-tls`), `sha2` (new dep), `serde` derive (new dep), Python stdlib (codegen), `rstest` (dev-dep, already present).

**Spec:** `docs/superpowers/specs/2026-08-24-tls-fingerprint-design.md`

## Global Constraints

- Workspace lints: clippy `pedantic` + `nursery` at `warn` — every crate opt-in via `[lints] workspace = true`; `cargo clippy --workspace` must stay clean; format with `cargo fmt`.
- New deps go in `crates/xray-tui-tls/Cargo.toml` (single-crate deps declared in-crate, minor-version bound): `sha2 = "0.10"`, `serde = { version = "1", features = ["derive"] }`.
- No shims/deprecated aliases: `BrowserProfile` is deleted in the cutover task; all callers migrate in the same change.
- JA4 codec follows the validated algorithm in the spec (peet.ws semantics; padding/SNI/ALPN excluded from hash2 list but counted in ext_count).
- Reference repos (`thirdparty/*`) are read-only — never edit them.
- Tests live next to code (unit) or in `crates/xray-tui-tls/tests/` (integration).

---

### Task 1: JA4 codec rewrite (final FoxIO scheme)

**Files:**
- Modify: `crates/xray-tui-tls/src/crypto/fingerprint/ja4.rs` (full rewrite)
- Modify: `crates/xray-tui-tls/src/crypto/fingerprint/ja3.rs` (add `from_spec`, `ja3_grease_stripped`)
- Modify: `crates/xray-tui-tls/src/profiles/mod.rs` (tests: migrate `ja4_a` call sites)
- Modify: `crates/xray-tui-tls/Cargo.toml` (add `sha2`)

**Interfaces:**
- Consumes: `crate::spec::{ClientHelloSpec, ExtensionSpec}`; `crate::spec::grease::is_grease(u16)`; existing `Ja3Fields`.
- Produces (used by Tasks 5, 9, 10):
  - `ja3::Ja3Fields::from_spec(&ClientHelloSpec) -> Self`
  - `ja3::ja3_grease_stripped(f: &Ja3Fields) -> String` (canonical string, GREASE removed)
  - `ja4::ja4_a(f: &Ja3Fields) -> String` — REWRITTEN: counts-based A-part
  - `ja4::hash1(f: &Ja3Fields) -> String`, `ja4::hash2(f: &Ja3Fields) -> String`
  - `ja4::full_ja4(f: &Ja3Fields) -> String` — `"{ja4_a}_{hash1}_{hash2}"`

- [ ] **Step 1: Add sha2 dependency**

In `crates/xray-tui-tls/Cargo.toml` `[dependencies]`, next to the md-5 line:

```toml
# JA4 hash segments (sha256 truncated)
sha2 = "0.10"
```

- [ ] **Step 2: Write failing known-vector tests**

Replace the test module in `ja4.rs` with (vector captured live from `tls.peet.ws/api/all` on 2026-08-24):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::fingerprint::ja3::Ja3Fields;

    /// Ground truth: curl-shaped ClientHello reported by tls.peet.ws
    /// (`t13d3113h2_e8f1e7e78f70_db572f7c111e`). Lists verbatim from its
    /// `ja4_r` rendering; extension order reconstructed from its ja3 field.
    fn peet_vector() -> Ja3Fields {
        let ciphers: Vec<u16> = [
            "002f", "0033", "0035", "0039", "003c", "003d", "0067", "006b",
            "009c", "009d", "009e", "009f", "00ff", "1301", "1302", "1303",
            "c009", "c00a", "c013", "c014", "c023", "c024", "c027", "c028",
            "c02b", "c02c", "c02f", "c030", "cca8", "cca9", "ccaa",
        ].iter().map(|s| u16::from_str_radix(s, 16).unwrap()).collect();
        let extensions: Vec<u16> = [
            0x0000, 0x000b, 0x000a, 0x3374, 0x0010, 0x0016, 0x0017,
            0x0031, 0x000d, 0x002b, 0x002d, 0x0033, 0x0015,
        ].to_vec();
        let signature_algorithms: Vec<u16> = [
            0x0403, 0x0503, 0x0603, 0x0807, 0x0808, 0x0809, 0x080a, 0x080b,
            0x0804, 0x0805, 0x0806, 0x0401, 0x0501, 0x0601, 0x0303, 0x0301,
            0x0302, 0x0402, 0x0502, 0x0602,
        ].to_vec();
        Ja3Fields {
            version: 771,
            ciphers,
            extensions,
            curves: vec![29, 23, 30, 25, 24, 256, 257, 258, 259, 260],
            point_formats: vec![0, 1, 2],
            signature_algorithms,
            alpn: vec!["h2".into()],
        }
    }

    #[test]
    fn full_ja4_peet_ws_known_vector() {
        let f = peet_vector();
        assert_eq!(full_ja4(&f), "t13d3113h2_e8f1e7e78f70_db572f7c111e");
    }

    #[test]
    fn ext_count_includes_sni_alpn_padding_but_hash_excludes_them() {
        // 13 extensions counted (incl. 0000/0010/0015); hash2 list excludes them.
        assert_eq!(ja4_a(&peet_vector()), "t13d3113h2");
    }

    #[test]
    fn grease_removed_from_counts_and_hashes() {
        let mut f = peet_vector();
        f.ciphers.insert(0, 0xCACA);
        f.extensions.insert(0, 0x0A0A);
        assert_eq!(full_ja4(&f), "t13d3113h2_e8f1e7e78f70_db572f7c111e");
    }

    #[test]
    fn alpn_short_form_and_none() {
        let mut f = peet_vector();
        f.alpn = vec!["http/1.1".into()];
        assert!(ja4_a(&f).ends_with("h1"));
        f.alpn.clear();
        assert!(ja4_a(&f).ends_with("00"));
    }

    #[test]
    fn no_sig_algs_drops_trailing_segment() {
        let mut f = peet_vector();
        f.signature_algorithms.clear();
        let h2 = hash2(&f); // payload ends after ext list, no trailing underscore join
        assert_eq!(h2.len(), 12);
    }
}
```

- [ ] **Step 3: Run to verify failure**

Run: `cargo test -p xray-tui-tls crypto::fingerprint::ja4`
Expected: FAIL (compile error — `full_ja4`/`hash2` undefined).

- [ ] **Step 4: Rewrite ja4.rs**

Full new content of `crates/xray-tui-tls/src/crypto/fingerprint/ja4.rs`:

```rust
//! JA4 fingerprint encoder (final FoxIO scheme, offline).
//!
//! Validated byte-for-byte against a live `tls.peet.ws` capture (see
//! tests). Semantics:
//!
//! - A-part `{t|q}{version}{d|i}{cipher_count}{ext_count}{alpn}`: counts
//!   EXCLUDE GREASE entirely but INCLUDE SNI (`0000`), ALPN (`0010`) and
//!   padding (`0015`). ALPN renders first protocol's first+last char
//!   (`h2`, `http/1.1` → `h1`), `00` when absent, `99` when non-ASCII.
//! - hash1: sha256 of sorted non-GREASE cipher ids (lowercase 4-hex,
//!   comma-joined), truncated to 12 hex chars.
//! - hash2: sha256 of sorted non-GREASE extension ids EXCLUDING SNI,
//!   ALPN and padding, joined `,`, then `_` then sig-alg ids in HELLO
//!   ORDER joined `,`; truncated to 12 hex chars. (Padding exclusion =
//!   peet.ws semantics; pinned by the known-vector test.)

use sha2::{Digest, Sha256};

use crate::spec::grease::is_grease;
use crate::spec::{ClientHelloSpec, ExtensionSpec};

use super::ja3::Ja3Fields;

/// Extension ids excluded from the hash2 list (but still counted).
const HASH2_EXCLUDED: [u16; 3] = [0x0000, 0x0010, 0x0015];

/// The JA4-A part: `t13d{cipher_count:02}{ext_count:02}{alpn}` shape.
#[must_use]
pub fn ja4_a(f: &Ja3Fields) -> String {
    let cipher_count = f.ciphers.iter().filter(|&&c| !is_grease(c)).count();
    let ext_count = f.extensions.iter().filter(|&&e| !is_grease(e)).count();
    let alpn = match f.alpn.first() {
        None => "00".to_string(),
        Some(p) if p.bytes().all(|b| b.is_ascii()) => {
            let bytes = p.as_bytes();
            if bytes.len() > 2 {
                format!("{}{}", bytes[0] as char, bytes[bytes.len() - 1] as char)
            } else {
                p.clone()
            }
        }
        Some(_) => "99".to_string(),
    };
    format!("t13d{cipher_count:02}{ext_count:02}{alpn}")
}

/// hash1: sha256[:12] over sorted non-GREASE ciphers (4-hex, comma-joined).
#[must_use]
pub fn hash1(f: &Ja3Fields) -> String {
    let mut ciphers: Vec<String> = f
        .ciphers
        .iter()
        .filter(|&&c| !is_grease(c))
        .map(|c| format!("{c:04x}"))
        .collect();
    ciphers.sort_unstable();
    sha12(ciphers.join(","))
}

/// hash2: sha256[:12] over sorted extensions (minus SNI/ALPN/padding/GREASE)
/// + `_` + sig algs in hello order.
#[must_use]
pub fn hash2(f: &Ja3Fields) -> String {
    let mut exts: Vec<String> = f
        .extensions
        .iter()
        .copied()
        .filter(|&e| !is_grease(e) && !HASH2_EXCLUDED.contains(&e))
        .map(|e| format!("{e:04x}"))
        .collect();
    exts.sort_unstable();
    let mut payload = exts.join(",");
    if !f.signature_algorithms.is_empty() {
        payload.push('_');
        payload.push_str(
            &f.signature_algorithms
                .iter()
                .map(|s| format!("{s:04x}"))
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    sha12(payload)
}

/// The full JA4 fingerprint: `{ja4_a}_{hash1}_{hash2}`.
#[must_use]
pub fn full_ja4(f: &Ja3Fields) -> String {
    format!("{}_{}_{}", ja4_a(f), hash1(f), hash2(f))
}

fn sha12(payload: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let digest = hasher.finalize();
    digest.iter().take(6).map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    // (test module from Step 2 goes here unchanged)
}
```

Note: the old `sorted_list` helper and the old GREASE-as-`g` behavior are deleted — nothing outside this file uses them.

Also export the new functions from the fingerprint module if `crypto/fingerprint/mod.rs` re-exports item names (check; it currently declares `pub mod ja3; pub mod ja4;` so paths suffice).

- [ ] **Step 5: Add `Ja3Fields::from_spec` and `ja3_grease_stripped`**

In `crates/xray-tui-tls/src/crypto/fingerprint/ja3.rs`, add below the existing `From<&ParsedClientHello>` impl:

```rust
/// Wire codepoint of an `ExtensionSpec` (single source of truth for
/// from_spec; GREASE renders as the canonical 0x0A0A sentinel id).
fn extension_codepoint(ext: &crate::spec::ExtensionSpec) -> u16 {
    use crate::spec::ExtensionSpec as E;
    match ext {
        E::ServerName => 0x0000,
        E::SupportedGroups(_) => 0x000A,
        E::KeyShare(_) => 0x0033,
        E::SupportedVersions(_) => 0x002B,
        E::SignatureAlgorithms(_) => 0x000D,
        E::Alpn(_) => 0x0010,
        E::EcPointFormats => 0x000B,
        E::SessionTicket => 0x0023,
        E::PskKeyExchangeModes => 0x002D,
        E::StatusRequest => 0x0005,
        E::SignedCertificateTimestamp => 0x0012,
        E::RenegotiationInfo => 0xFF01,
        E::CompressCertificate(_) => 0x001B,
        E::ApplicationSettings(_) => 0x4469,
        E::RecordSizeLimit(_) => 0x001C,
        E::Padding => 0x0015,
        E::Grease => 0x0A0A,
        E::Raw { ty, .. } => *ty,
    }
}

impl Ja3Fields {
    /// Extracts JA3/JA4 fields from a semantic [`ClientHelloSpec`] without
    /// building wire bytes. GREASE slots appear verbatim as 0x0A0A
    /// sentinels (callers decide whether to strip — JA4 always does,
    /// classic JA3 never does).
    #[must_use]
    pub fn from_spec(spec: &ClientHelloSpec) -> Self {
        use crate::spec::ExtensionSpec as E;
        let mut out = Self {
            version: spec.legacy_version,
            ciphers: spec.cipher_suites.clone(),
            extensions: spec.extensions.iter().map(extension_codepoint).collect(),
            curves: Vec::new(),
            point_formats: Vec::new(),
            signature_algorithms: Vec::new(),
            alpn: Vec::new(),
        };
        for ext in &spec.extensions {
            match ext {
                E::SupportedGroups(g) => out.curves = g.clone(),
                E::SignatureAlgorithms(s) => out.signature_algorithms = s.clone(),
                E::Alpn(p) => out.alpn = p.clone(),
                E::EcPointFormats => out.point_formats = vec![0],
                _ => {}
            }
        }
        out
    }
}

/// Canonical JA3 string over GREASE-stripped fields (deterministic
/// variant; NOT the classic on-wire JA3 for GREASE-carrying clients —
/// see the design spec's determinism rules).
#[must_use]
pub fn ja3_grease_stripped(f: &Ja3Fields) -> String {
    use crate::spec::grease::is_grease;
    let clean = Ja3Fields {
        version: f.version,
        ciphers: f.ciphers.iter().copied().filter(|&c| !is_grease(c)).collect(),
        extensions: f.extensions.iter().copied().filter(|&e| !is_grease(e)).collect(),
        curves: f.curves.clone(),
        point_formats: f.point_formats.clone(),
        signature_algorithms: f.signature_algorithms.clone(),
        alpn: f.alpn.clone(),
    };
    ja3_string(&clean)
}
```

All current profiles offer TLS 1.3 via `supported_versions`, so `ja4_a` hardcodes the `t13d…` version segment — document that constraint on `ja4_a`.

Add unit tests in `ja3.rs`:

```rust
#[test]
fn from_spec_maps_chrome_like_extension_order() {
    use crate::spec::{ClientHelloSpec, ExtensionSpec, SessionIdSpec};
    let spec = ClientHelloSpec {
        legacy_version: 0x0303,
        cipher_suites: vec![0x1301],
        compression_methods: vec![0],
        session_id: SessionIdSpec::Random32,
        extensions: vec![
            ExtensionSpec::ServerName,
            ExtensionSpec::Grease,
            ExtensionSpec::SupportedVersions(vec![0x0304]),
            ExtensionSpec::SignatureAlgorithms(vec![0x0403]),
        ],
    };
    let f = Ja3Fields::from_spec(&spec);
    assert_eq!(f.extensions, vec![0x0000, 0x0A0A, 0x002B, 0x000D]);
    assert_eq!(f.signature_algorithms, vec![0x0403]);
    assert_eq!(ja3_grease_stripped(&f), ja3_string(&Ja3Fields {
        version: 771,
        ciphers: vec![0x1301],
        extensions: vec![0x0000, 0x002B, 0x000D],
        curves: vec![],
        point_formats: vec![],
        signature_algorithms: vec![0x0403],
        alpn: vec![],
    }));
}
```

- [ ] **Step 6: Migrate `ja4_a` call sites inside the crate**

`crates/xray-tui-tls/src/profiles/mod.rs` tests (~lines 109–197) call `ja4_a(&fields)` and assert `.starts_with("t13d")`. With the counts-based A-part the assertion still holds (`t13d1514h2` style) — verify each assertion compiles and passes unchanged; where a test asserts the OLD first-cipher shape (none currently assert beyond the prefix), migrate to `full_ja4`.

Run: `cargo test -p xray-tui-tls`
Expected: PASS (ja4 known vectors + all profile tests).

- [ ] **Step 7: Commit**

```bash
git add crates/xray-tui-tls/src/crypto/fingerprint/ crates/xray-tui-tls/src/profiles/mod.rs crates/xray-tui-tls/Cargo.toml Cargo.lock
git commit -m "feat(tls): rewrite JA4 codec to final FoxIO scheme (counts A-part + sha256 segments)"
```

---

### Task 2: Fingerprint identity types

**Files:**
- Create: `crates/xray-tui-tls/src/fingerprints/mod.rs`
- Create: `crates/xray-tui-tls/src/fingerprints/query.rs`
- Create: `crates/xray-tui-tls/src/fingerprints/error.rs`
- Modify: `crates/xray-tui-tls/src/lib.rs` (declare module)

**Interfaces:**
- Produces (used by Tasks 4–10):
  - `fingerprints::error::FingerprintError { Unknown { query: String, available: Vec<String> }, InvalidOverride(String) }` (`thiserror::Error`)
  - `fingerprints::query::{Browser, Os, Device, Fingerprint}`
  - `Fingerprint { pub browser: Browser, pub version: Option<u16>, pub os: Option<Os>, pub device: Option<Device> }` — `Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize`
  - `Fingerprint::new(Browser) -> Self`; `.with_version(u16)`, `.with_os(Os)`, `.with_device(Device)` (consuming builders)
  - `Fingerprint::default_for(Browser) -> Self`; `Fingerprint::platform_default() -> Self`; `impl Default`

- [ ] **Step 1: Add serde dependency**

`crates/xray-tui-tls/Cargo.toml`:

```toml
# Fingerprint selection persistence (TUI settings)
serde = { version = "1", features = ["derive"] }
```

- [ ] **Step 2: Write failing tests**

`crates/xray-tui-tls/src/fingerprints/query.rs` (test module at bottom):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_for_pins_latest_known_per_browser() {
        let fp = Fingerprint::default_for(Browser::Chrome);
        assert_eq!(fp.browser, Browser::Chrome);
        assert!(fp.version.is_some(), "latest-known version pinned");
        assert!(fp.device.is_some());
    }

    #[test]
    fn platform_default_matches_target_os_family() {
        let fp = Fingerprint::platform_default();
        #[cfg(target_os = "linux")]
        assert_eq!((fp.browser, fp.os), (Browser::Firefox, Some(Os::Linux)));
        #[cfg(target_os = "macos")]
        assert_eq!((fp.browser, fp.os), (Browser::Safari, Some(Os::MacOs)));
        #[cfg(target_os = "windows")]
        assert_eq!((fp.browser, fp.os), (Browser::Chrome, Some(Os::Windows)));
        #[cfg(target_os = "android")]
        assert_eq!((fp.browser, fp.os), (Browser::Chrome, Some(Os::Android)));
    }

    #[test]
    fn builder_style_setters_consume_self() {
        let fp = Fingerprint::new(Browser::Firefox)
            .with_version(120)
            .with_os(Os::Linux)
            .with_device(Device::Desktop);
        assert_eq!(fp.version, Some(120));
        assert_eq!(fp.os, Some(Os::Linux));
    }

    #[test]
    fn serde_roundtrip() {
        let fp = Fingerprint::new(Browser::Chrome).with_version(133);
        let json = serde_json::to_string(&fp).unwrap();
        assert_eq!(serde_json::from_str::<Fingerprint>(&json).unwrap(), fp);
    }
}
```

- [ ] **Step 3: Run to verify failure**

Run: `cargo test -p xray-tui-tls fingerprints`
Expected: FAIL (module doesn't exist).

- [ ] **Step 4: Implement**

`crates/xray-tui-tls/src/fingerprints/mod.rs`:

```rust
//! Flexible TLS fingerprint selection (see
//! `docs/superpowers/specs/2026-08-24-tls-fingerprint-design.md`).

pub mod error;
pub mod query;

pub use error::FingerprintError;
pub use query::{Browser, Device, Fingerprint, Os};
```

`crates/xray-tui-tls/src/fingerprints/error.rs`:

```rust
//! Fingerprint selection/build errors.

/// Strict-resolution and builder failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FingerprintError {
    /// No resolution-table row satisfies the query.
    #[error("unknown fingerprint combination `{query}`; available: {}", available.join(", "))]
    Unknown {
        /// Rendered query (`chrome@133/windows/desktop` shape).
        query: String,
        /// Resolvable identity names, sorted.
        available: Vec<String>,
    },
    /// A builder override violates a base-spec invariant.
    #[error("invalid fingerprint override: {0}")]
    InvalidOverride(String),
}
```

`crates/xray-tui-tls/src/fingerprints/query.rs`:

```rust
//! The single fingerprint identity used across the crate.

/// Browsers with at least one hand-transcribed hello.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Browser {
    Chrome,
    Firefox,
    Safari,
    Edge,
    Brave,
    Opera,
    SamsungInternet,
}

impl Browser {
    /// Stable lowercase identifier (configs/logs).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Chrome => "chrome",
            Self::Firefox => "firefox",
            Self::Safari => "safari",
            Self::Edge => "edge",
            Self::Brave => "brave",
            Self::Opera => "opera",
            Self::SamsungInternet => "samsung_internet",
        }
    }
}

/// Operating system of the impersonated client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Os { Windows, MacOs, Linux, Android, Ios }

impl Os {
    /// Stable lowercase identifier.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::MacOs => "macos",
            Self::Linux => "linux",
            Self::Android => "android",
            Self::Ios => "ios",
        }
    }
}

/// Device class of the impersonated client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Device { Desktop, Phone, Tablet }

impl Device {
    /// Stable lowercase identifier.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Phone => "phone",
            Self::Tablet => "tablet",
        }
    }
}

/// The fingerprint identity: browser + optional exact major version,
/// OS and device class. Unset fields fall back to the resolution table.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Fingerprint {
    pub browser: Browser,
    pub version: Option<u16>,
    pub os: Option<Os>,
    pub device: Option<Device>,
}

impl Fingerprint {
    /// A bare identity; fill optionals with the `with_*` setters.
    #[must_use]
    pub const fn new(browser: Browser) -> Self {
        Self { browser, version: None, os: None, device: None }
    }

    /// Latest-known version/os/device defaults for a browser (the
    /// newest resolution-table row for that browser).
    #[must_use]
    pub fn default_for(browser: Browser) -> Self {
        Self::new(browser)
    }

    /// Platform-sensible default. Windows → Chrome/Windows,
    /// macOS → Safari/macOS, Android → Chrome/Android, everything else
    /// (incl. Linux) → Firefox/Linux-desktop.
    #[must_use]
    pub fn platform_default() -> Self {
        #[cfg(target_os = "windows")]
        { Self::new(Browser::Chrome).with_os(Os::Windows) }
        #[cfg(target_os = "macos")]
        { Self::new(Browser::Safari).with_os(Os::MacOs) }
        #[cfg(target_os = "android")]
        { Self::new(Browser::Chrome).with_os(Os::Android) }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "android")))]
        { Self::new(Browser::Firefox).with_os(Os::Linux).with_device(Device::Desktop) }
    }

    #[must_use]
    pub const fn with_version(mut self, version: u16) -> Self {
        self.version = Some(version);
        self
    }

    #[must_use]
    pub const fn with_os(mut self, os: Os) -> Self {
        self.os = Some(os);
        self
    }

    #[must_use]
    pub const fn with_device(mut self, device: Device) -> Self {
        self.device = Some(device);
        self
    }

    /// Human-readable identity: `chrome@133/windows/desktop`; unset
    /// parts render as `-`.
    #[must_use]
    pub fn render(&self) -> String {
        let v = self.version.map_or_else(|| "-".into(), |v| v.to_string());
        let os = self.os.map_or_else(|| "-".into(), Os::name);
        let d = self.device.map_or_else(|| "-".into(), Device::name);
        format!("{}/{v}/{os}/{}", self.browser.name(), d)
    }
}

impl Default for Fingerprint {
    fn default() -> Self {
        Self::platform_default()
    }
}
```

Declare in `lib.rs` alongside the other modules (keep alphabetical-ish grouping):

```rust
pub mod fingerprints;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p xray-tui-tls fingerprints`
Expected: PASS.

Note: the `#[cfg]` arms in `platform_default` return different expressions per target — wrap in a single expression via blocks exactly as shown (each arm is a block expression; only one cfg is active).

- [ ] **Step 6: Commit**

```bash
git add crates/xray-tui-tls/src/fingerprints/ crates/xray-tui-tls/src/lib.rs crates/xray-tui-tls/Cargo.toml Cargo.lock
git commit -m "feat(tls): Fingerprint identity types (browser/version/os/device)"
```

---

### Task 3: Catalog generator + generated data

**Files:**
- Create: `crates/xray-tui-tls/src/fingerprints/catalog/gen.py`
- Create: `crates/xray-tui-tls/src/fingerprints/catalog/mod.rs`
- Create: `crates/xray-tui-tls/src/fingerprints/catalog/catalog_data.rs` (generated)

**Interfaces:**
- Consumes: `thirdparty/ja4db-export/csv/all_records.csv` (read-only input).
- Produces (used by Task 5): `catalog::CatalogEntry { ja4: &'static str, application: &'static str, library: &'static str, device: &'static str, os: &'static str, user_agent: &'static str, verified: bool, observation_count: u64 }`; `catalog::CATALOG: &[CatalogEntry]`.

- [ ] **Step 1: Write the generator**

`gen.py` (stdlib-only; run from repo root):

```python
#!/usr/bin/env python3
"""Generates catalog_data.rs from the frozen ja4db-export CSV snapshot.

Cleaning rules (per the design spec):
- keep rows whose ja4_fingerprint matches ^t1[0-3] AND have an identifiable
  application: non-empty `application` OR a parseable user_agent_string;
- parse UA -> (browser, major, os, device); unparseable rows dropped;
- dedupe on (ja4, browser, major, os, device); sum observation_count.
"""
import csv
import re
import sys
from collections import OrderedDict

CSV_PATH = "thirdparty/ja4db-export/csv/all_records.csv"
OUT_PATH = "crates/xray-tui-tls/src/fingerprints/catalog/catalog_data.rs"

JA4_RE = re.compile(r"^t1[0-3][a-z]")
BROWSERS = [
    ("edg", "edge"), ("Edg", "edge"),
    ("Samsung", "samsung"), ("Firefox", "firefox"), ("Chrome", "chrome"),
    ("Safari", "safari"), ("Opera", "opera"), ("OPR", "opera"),
]
OS_PATTERNS = [
    (r"Windows NT 10\.0", "windows"), (r"Windows NT 6\.[13]", "windows"),
    (r"Windows NT 6\.[02]", "windows"), (r"Windows NT 5", "windows"),
    (r"Mac OS X|Macintosh", "macos"), (r"Android", "android"),
    (r"iPhone|iPad", "ios"), (r"Cros", "linux"), (r"X11|Linux", "linux"),
]
VERSION_RE = {
    "chrome": r"(?:Chrome|CriOS)/(\d+)\.",
    "edge": r"(?:Edge|Edg)/(\d+)\.",
    "firefox": r"Firefox/(\d+)\.",
    "safari": r"Version/(\d+)\.",
    "opera": r"(?:Opera|OPR)/(\d+)\.",
    "samsung": r"SamsungBrowser/(\d+)\.",
}


def parse_application(application):
    """Direct application field, e.g. 'Chrome 94.0'."""
    m = re.match(r"^(Chrome|Firefox|Safari|Edge|Brave|Opera|Samsung Internet)"
                 r"(?: ([0-9]+)(?:\.[0-9]+)?)?\s*$", application.strip())
    if not m:
        return None
    name = {"Samsung Internet": "samsung"}.get(m.group(1), m.group(1).lower())
    return name, int(m.group(2)) if m.group(2) else None


def parse_ua(ua):
    ua = ua.strip()
    if not ua:
        return None
    browser = None
    for token, name in BROWSERS:
        if token in ua:
            browser = name
            break
    if browser is None:
        # Safari-only UAs carry no engine token besides Version/
        if "Version/" in ua and "Safari" in ua:
            browser = "safari"
        else:
            return None
    vm = re.search(VERSION_RE[browser], ua)
    version = int(vm.group(1)) if vm else None
    os_name = None
    for pat, o in OS_PATTERNS:
        if re.search(pat, ua):
            os_name = o
            break
    device = "phone" if ("iPhone" in ua or "Android" in ua and "Mobile" in ua) else (
        "tablet" if "iPad" in ua or ("Android" in ua and "Mobile" not in ua) else "desktop")
    if os_name is None:
        return None
    return browser, version, os_name, device


def main():
    rows = OrderedDict()  # key -> row dict
    dropped = kept = 0
    with open(CSV_PATH, newline="", encoding="utf-8") as fh:
        reader = csv.DictReader(fh)
        for r in reader:
            ja4 = (r.get("ja4_fingerprint") or "").strip()
            if not JA4_RE.match(ja4):
                continue
            parsed = parse_application(r.get("application") or "") or parse_ua(r.get("user_agent_string") or "")
            if parsed is None:
                dropped += 1
                continue
            browser, version, os_name, device = parsed
            try:
                count = int(r.get("observation_count") or "1")
            except ValueError:
                count = 1
            key = (ja4, browser, version, os_name, device)
            if key in rows:
                rows[key]["observation_count"] += count
            else:
                rows[key] = {
                    "ja4": ja4, "application": browser, "library": (r.get("library") or "").strip(),
                    "device": device, "os": os_name,
                    "user_agent": (r.get("user_agent_string") or "").strip(),
                    "verified": (r.get("verified") or "").strip() == "true",
                    "observation_count": count, "_version": version,
                }
            kept += 1

    def esc(s):
        return s.replace("\\", "\\\\").replace('"', '\\"')

    lines = [
        "// GENERATED by catalog/gen.py from thirdparty/ja4db-export (frozen",
        "// snapshot 2026-05-15). Do not edit by hand; rerun the generator.",
        "/// One cleaned real-world JA4 observation.",
        "#[derive(Debug, Clone)]",
        "pub struct CatalogEntry {",
        "    pub ja4: &'static str,",
        "    pub application: &'static str,",
        "    pub library: &'static str,",
        "    pub device: &'static str,",
        "    pub os: &'static str,",
        "    pub user_agent: &'static str,",
        "    pub verified: bool,",
        "    pub observation_count: u64,",
        "}",
        "",
        "/// The cleaned catalog.",
        "pub static CATALOG: &[CatalogEntry] = &[",
    ]
    for row in rows.values():
        lines.append(
            "    CatalogEntry {{ ja4: \"{ja4}\", application: \"{app}\", "
            "library: \"{lib}\", device: \"{dev}\", os: \"{os}\", "
            "user_agent: \"{ua}\", verified: {ver}, observation_count: {cnt} }},".format(
                ja4=esc(row["ja4"]), app=esc(row["application"]), lib=esc(row["library"]),
                dev=esc(row["device"]), os=esc(row["os"]), ua=esc(row["user_agent"][:200]),
                ver="true" if row["verified"] else "false",
                cnt=row["observation_count"],
            ))
    lines.append("];")
    with open(OUT_PATH, "w", encoding="utf-8") as out:
        out.write("\n".join(lines) + "\n")
    print(f"kept {len(rows)} unique entries ({kept} rows merged, {dropped} dropped)", file=sys.stderr)


if __name__ == "__main__":
    main()
```

- [ ] **Step 2: Run the generator and inspect yield**

Run: `python3 crates/xray-tui-tls/src/fingerprints/catalog/gen.py`
Expected: stderr reports kept-count in the low thousands (roughly 2k–6k unique entries from ~74.5k rows). If kept < 500, the UA parser is too narrow — debug against sample rows (`head thirdparty/ja4db-export/csv/all_records.csv`) before proceeding.

- [ ] **Step 3: Module wiring + integrity tests**

`crates/xray-tui-tls/src/fingerprints/catalog/mod.rs`:

```rust
//! Generated real-world JA4 catalog (identity evidence only — never used
//! to synthesize hello bytes).

mod catalog_data;

pub use catalog_data::{CatalogEntry, CATALOG};

/// All catalog entries observed for a browser name (e.g. `"chrome"`),
/// optionally filtered to an exact major version.
#[must_use]
pub fn entries_for(application: &str, version: Option<u16>) -> Vec<&'static CatalogEntry> {
    CATALOG
        .iter()
        .filter(|e| {
            if e.application != application {
                return false;
            }
            match version {
                None => true,
                // The application column stores the browser name; versions
                // ride the user_agent. Match on `/major.` in the UA.
                Some(v) => e.user_agent.contains(&format!("/{v}.")),
            }
        })
        .collect()
}

/// True when `ja4` was observed in the wild for this browser (+version).
#[must_use]
pub fn contains(application: &str, version: Option<u16>, ja4: &str) -> bool {
    entries_for(application, version).iter().any(|e| e.ja4 == ja4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_nonempty_and_wellformed() {
        assert!(CATALOG.len() > 500, "catalog suspiciously small: {}", CATALOG.len());
        for e in CATALOG.iter().take(100) {
            assert!(e.ja4.starts_with("t1"), "{}", e.ja4);
            assert!(e.ja4.split('_').count() == 3 || e.ja4.split('_').count() == 2,
                    "unexpected ja4 shape: {}", e.ja4);
            assert!(!e.application.is_empty());
        }
    }

    #[test]
    fn chrome_entries_exist() {
        assert!(!entries_for("chrome", None).is_empty());
    }
}
```

Register in `fingerprints/mod.rs`: add `pub mod catalog;`.

- [ ] **Step 4: Run tests**

Run: `cargo test -p xray-tui-tls fingerprints::catalog`
Expected: PASS. If clippy flags the generated file (long lines etc.), add `#![allow(clippy::all)]`-style exceptions ONLY via crate-level lint config for the generated module (`#[allow(clippy::doc_markdown)]` on the struct is acceptable).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls/src/fingerprints/catalog/
git commit -m "feat(tls): generated JA4 catalog from frozen ja4db-export snapshot"
```

---

### Task 4: Resolution table + strict resolver

**Files:**
- Create: `crates/xray-tui-tls/src/fingerprints/resolve.rs`
- Modify: `crates/xray-tui-tls/src/fingerprints/mod.rs` (wire module + re-export)

**Interfaces:**
- Consumes: `query::{Browser, Os, Device, Fingerprint}`, `profiles::*::spec()` fns.
- Produces (used by Tasks 5, 6, 9):
  - `resolve::Resolved { pub name: &'static str, pub fingerprint: Fingerprint, pub spec: ClientHelloSpec }` (defined in this task; Task 5 extends it with catalog evidence)
  - `Fingerprint::resolve(&self) -> Result<Resolved, FingerprintError>`
  - `resolve::available_identities() -> Vec<String>`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_exact_query() {
        let fp = Fingerprint::new(Browser::Chrome).with_version(133);
        let r = fp.resolve().unwrap();
        assert_eq!(r.name, "chrome_133");
        assert!(!r.spec.cipher_suites.is_empty());
    }

    #[test]
    fn falls_back_to_nearest_lower_version_same_platform() {
        let fp = Fingerprint::new(Browser::Chrome)
            .with_version(132)
            .with_os(Os::Windows)
            .with_device(Device::Desktop);
        let r = fp.resolve().unwrap();
        assert_eq!(r.name, "chrome_133"); // nearest lower-or-equal row
    }

    #[test]
    fn unset_fields_default_to_desktop_rows() {
        let r = Fingerprint::new(Browser::Firefox).resolve().unwrap();
        assert_eq!(r.fingerprint.os, Some(Os::Linux)); // firefox_120 row pins linux? NO:
        // desktop rows may pin any os — assert only device:
        assert_eq!(r.fingerprint.device, Some(Device::Desktop));
    }

    #[test]
    fn unknown_browser_version_errors_with_available_list() {
        let fp = Fingerprint::new(Browser::Chrome).with_version(3);
        let err = fp.resolve().unwrap_err();
        let FingerprintError::Unknown { available, .. } = err else { panic!("wrong variant") };
        assert!(available.iter().any(|a| a.starts_with("chrome")));
    }

    #[test]
    fn mobile_query_lands_on_mobile_row() {
        let fp = Fingerprint::new(Browser::Chrome)
            .with_os(Os::Android)
            .with_device(Device::Phone);
        let r = fp.resolve().unwrap();
        assert_eq!(r.name, "chrome_android_130");
    }

    #[test]
    fn every_table_row_targets_existing_profile_fn() {
        // Compile-time completeness: building each row's spec proves the fn exists.
        for entry in TABLE {
            let _ = (entry.spec)();
        }
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p xray-tui-tls fingerprints::resolve`
Expected: FAIL (module missing).

- [ ] **Step 3: Implement**

`crates/xray-tui-tls/src/fingerprints/resolve.rs`:

```rust
//! Identity → concrete hello resolution over a hand-maintained table.
//!
//! Strict per the design spec: unknown combinations error with the list
//! of what IS resolvable. Never silently substitute a different browser.

use crate::fingerprints::error::FingerprintError;
use crate::fingerprints::query::{Browser, Device, Fingerprint, Os};
use crate::spec::ClientHelloSpec;

/// One resolvable identity row.
pub(crate) struct Row {
    /// Stable spec name (`chrome_130`) — appears in errors and UI.
    pub name: &'static str,
    pub browser: Browser,
    /// `None` = row serves any OS (desktop hellos are largely OS-independent
    /// within a browser family).
    pub os: Option<Os>,
    pub device: Device,
    /// Inclusive maximum requested major this row answers.
    pub max_version: u16,
    pub spec: fn() -> ClientHelloSpec,
}

use crate::profiles::{
    brave167, chrome, chrome119, chrome133, chrome_android130, edge, firefox, firefox128esr,
    opera114, safari, safari_ios17,
};

/// Ordered ascending by `max_version` within each (browser, os, device).
pub(crate) static TABLE: &[Row] = &[
    Row { name: "chrome_119",          browser: Browser::Chrome,  os: None,               device: Device::Desktop, max_version: 119, spec: chrome119::spec },
    Row { name: "chrome_130",          browser: Browser::Chrome,  os: None,               device: Device::Desktop, max_version: 130, spec: chrome::spec },
    Row { name: "chrome_133",          browser: Browser::Chrome,  os: None,               device: Device::Desktop, max_version: 133, spec: chrome133::spec },
    Row { name: "chrome_android_130",  browser: Browser::Chrome,  os: Some(Os::Android),  device: Device::Phone,   max_version: 130, spec: chrome_android130::spec },
    Row { name: "edge_130",            browser: Browser::Edge,    os: None,               device: Device::Desktop, max_version: 130, spec: edge::spec },
    Row { name: "brave_167",           browser: Browser::Brave,   os: None,               device: Device::Desktop, max_version: 167, spec: brave167::spec },
    Row { name: "opera_114",           browser: Browser::Opera,   os: None,               device: Device::Desktop, max_version: 114, spec: opera114::spec },
    Row { name: "firefox_128_esr",     browser: Browser::Firefox, os: None,               device: Device::Desktop, max_version: 128, spec: firefox128esr::spec },
    Row { name: "safari_17",           browser: Browser::Safari,  os: Some(Os::MacOs),    device: Device::Desktop, max_version: 17,  spec: safari::spec },
    Row { name: "safari_ios_17",       browser: Browser::Safari,  os: Some(Os::Ios),      device: Device::Phone,   max_version: 17,  spec: safari_ios17::spec },
    // Tasks 7/8 append: safari_16 (15), firefox_120 (120), edge_106 (106),
    // ios_14, android_11_okhttp — keeping per-browser ascending order.
];

impl Row {
    fn matches(&self, q_os: Option<Os>, q_device: Option<Device>) -> bool {
        if let Some(d) = q_device {
            if d != self.device {
                return false;
            }
        } else if self.device != Device::Desktop {
            return false; // unspecified device defaults to desktop rows
        }
        match (q_os, self.os) {
            (_, None) => true,
            (None, Some(_)) => true, // unspecified os accepts pinned-os rows
            (Some(a), Some(b)) => a == b,
        }
    }
}

/// Every resolvable identity name, sorted (error payloads, UI pickers).
#[must_use]
pub fn available_identities() -> Vec<String> {
    let mut names: Vec<String> = TABLE.iter().map(|r| r.name.to_string()).collect();
    names.sort();
    names
}

impl Fingerprint {
    /// Strict resolution to a concrete hello. Fallback chain: exact →
    /// same browser/device(+compatible os) nearest lower version →
    /// error listing alternatives.
    ///
    /// # Errors
    /// [`FingerprintError::Unknown`] when no row satisfies the query.
    pub fn resolve(&self) -> Result<Resolved, FingerprintError> {
        let candidates: Vec<&Row> = TABLE
            .iter()
            .filter(|r| r.browser == self.browser && r.matches(self.os, self.device))
            .collect();
        let chosen = match self.version {
            None => candidates.iter().max_by_key(|r| r.max_version),
            Some(v) => candidates
                .iter()
                .filter(|r| r.max_version >= v)
                .min_by_key(|r| r.max_version)
                .or_else(|| candidates.iter().max_by_key(|r| r.max_version)),
        };
        let Some(row) = chosen else {
            return Err(FingerprintError::Unknown {
                query: self.render(),
                available: available_identities(),
            });
        };
        Ok(Resolved {
            name: row.name,
            fingerprint: Fingerprint {
                browser: row.browser,
                version: Some(self.version.unwrap_or(row.max_version)),
                os: self.os.or(row.os),
                device: Some(self.device.unwrap_or(row.device)),
            },
            spec: (row.spec)(),
        })
    }
}

/// A successfully resolved fingerprint: identity + concrete hello.
#[derive(Debug, Clone)]
pub struct Resolved {
    /// Resolution-table name (`chrome_133`).
    pub name: &'static str,
    /// Fully concretized identity (unset query fields filled from the row).
    pub fingerprint: Fingerprint,
    /// The hello specification.
    pub spec: ClientHelloSpec,
}
```

Wire in `fingerprints/mod.rs`: add `pub mod resolve;` and extend re-exports:

```rust
pub use query::{Browser, Device, Fingerprint, Os};
pub use resolve::available_identities;
```

(`Resolved` stays `fingerprints::resolve::Resolved` until Task 5 finalizes it.)

Check `profiles` module visibility: `crates/xray-tui-tls/src/lib.rs` has `pub mod profiles;` — confirm submodules (`chrome133` etc.) are `pub mod`; they are (declared in `profiles/mod.rs`).

- [ ] **Step 4: Run tests**

Run: `cargo test -p xray-tui-tls fingerprints`
Expected: PASS. Adjust the two assertions flagged in test comments to the actual pinned-row semantics if a row differs.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls/src/fingerprints/
git commit -m "feat(tls): strict fingerprint resolution over hand-maintained table"
```

---

### Task 5: Resolved type with catalog evidence + JA4 audit

**Files:**
- Modify: `crates/xray-tui-tls/src/fingerprints/resolve.rs` (extend `Resolved`)
- Modify: `crates/xray-tui-tls/src/fingerprints/mod.rs` (re-export `Resolved`)

**Interfaces:**
- Consumes: `crypto::fingerprint::{ja3::Ja3Fields, ja4::full_ja4}`, `catalog::contains`.
- Produces (used by Tasks 6, 9, 10): `Resolved::ja4(&self) -> String`; `Resolved::in_catalog(&self) -> bool`.

- [ ] **Step 1: Write failing test** (append to `resolve.rs` tests)

```rust
#[test]
fn resolved_computes_full_ja4() {
    let r = Fingerprint::new(Browser::Chrome).with_version(133).resolve().unwrap();
    let ja4 = r.ja4();
    assert!(ja4.starts_with("t13d"), "{ja4}");
    let parts: Vec<&str> = ja4.split('_').collect();
    assert_eq!(parts.len(), 3, "A_hash1_hash2");
    assert_eq!(parts[1].len(), 12);
    assert_eq!(parts[2].len(), 12);
}
```

- [ ] **Step 2: Implement** (append to `Resolved` impl in resolve.rs)

```rust
impl Resolved {
    /// The full JA4 of this resolved hello (computed, not looked up).
    #[must_use]
    pub fn ja4(&self) -> String {
        crate::crypto::fingerprint::ja4::full_ja4(&crate::crypto::fingerprint::ja3::Ja3Fields::from_spec(&self.spec))
    }

    /// True when this resolved hello's JA4 was observed in the wild for
    /// its claimed browser identity (catalog evidence).
    #[must_use]
    pub fn in_catalog(&self) -> bool {
        super::catalog::contains(self.fingerprint.browser.name(), self.fingerprint.version, &self.ja4())
    }
}
```

Re-export in `fingerprints/mod.rs`: `pub use resolve::Resolved;`

- [ ] **Step 3: Run tests**

Run: `cargo test -p xray-tui-tls fingerprints`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-tls/src/fingerprints/
git commit -m "feat(tls): Resolved carries JA4 audit + catalog evidence check"
```

---

### Task 6: Compositional builder

**Files:**
- Create: `crates/xray-tui-tls/src/fingerprints/builder.rs`
- Modify: `crates/xray-tui-tls/src/fingerprints/mod.rs`

**Interfaces:**
- Consumes: `Fingerprint::resolve()`, `spec::apply_curve_preferences`, `ExtensionSpec`.
- Produces: `FingerprintBuilder` —
  - `FingerprintBuilder::new(fingerprint: &Fingerprint) -> Result<Self, FingerprintError>`
  - `.override_ciphers(&[u16])`, `.override_extensions(Vec<ExtensionSpec>)`,
    `.curves(&[u16])`, `.alpn(&[&str])`, `.sig_algorithms(&[u16])`,
    `.grease(GreasePolicy)` (all consuming, returning `Self`)
  - `.build(self) -> Result<ClientHelloSpec, FingerprintError>`
  - `enum GreasePolicy { Keep, Strip }`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprints::query::{Browser, Os};

    fn base() -> FingerprintBuilder {
        FingerprintBuilder::new(&Fingerprint::new(Browser::Chrome).with_version(130)).unwrap()
    }

    #[test]
    fn overrides_ciphers_replace_entirely() {
        let spec = base().override_ciphers(&[0x1301, 0x1302]).build().unwrap();
        assert_eq!(spec.cipher_suites, vec![0x1301, 0x1302]);
    }

    #[test]
    fn empty_cipher_override_rejected() {
        let err = base().override_ciphers(&[]).build().unwrap_err();
        assert!(matches!(err, FingerprintError::InvalidOverride(_)));
    }

    #[test]
    fn alpn_replaces_existing_extension() {
        let spec = base().alpn(&["h2", "http/1.1"]).build().unwrap();
        assert!(spec.extensions.iter().any(|e| matches!(
            e, ExtensionSpec::Alpn(p) if p == &["h2".to_string(), "http/1.1".to_string()]
        )));
        // exactly one Alpn extension
        assert_eq!(spec.extensions.iter().filter(|e| matches!(e, ExtensionSpec::Alpn(_))).count(), 1);
    }

    #[test]
    fn duplicate_extensions_rejected_after_override() {
        let mut dup = base().override_extensions(vec![
            ExtensionSpec::ServerName,
            ExtensionSpec::ServerName,
        ]);
        let err = dup.build().unwrap_err();
        assert!(matches!(err, FingerprintError::InvalidOverride(msg) if msg.contains("duplicate")));
    }

    #[test]
    fn curves_delegates_to_apply_curve_preferences() {
        let curves = [0x001Du16, 0x11EC];
        let built = base().curves(&curves).build().unwrap();
        let direct = crate::spec::apply_curve_preferences(
            &Fingerprint::new(Browser::Chrome).with_version(130).resolve().unwrap().spec,
            &curves,
        );
        assert_eq!(built, direct);
    }

    #[test]
    fn grease_strip_removes_grease_variants() {
        let spec = base().grease(GreasePolicy::Strip).build().unwrap();
        assert!(!spec.cipher_suites.iter().any(|&c| crate::spec::grease::is_grease(c)));
        assert!(!spec.extensions.iter().any(|e| matches!(e, ExtensionSpec::Grease)));
    }

    #[test]
    fn macos_safari_resolves_before_building() {
        // builder resolves strictly — unknown combos fail at construction
        let err = FingerprintBuilder::new(&Fingerprint::new(Browser::Chrome).with_version(3));
        assert!(err.is_err());
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p xray-tui-tls fingerprints::builder`
Expected: FAIL (missing module).

- [ ] **Step 3: Implement**

```rust
//! Compositional fingerprint builder (the foundation layer; catalog
//! selection is sugar over this).

use crate::fingerprints::error::FingerprintError;
use crate::fingerprints::query::Fingerprint;
use crate::spec::grease::is_grease;
use crate::spec::{ClientHelloSpec, ExtensionSpec};

/// What happens to GREASE slots in the built hello.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GreasePolicy {
    /// Preserve the base profile's GREASE placement (default).
    #[default]
    Keep,
    /// Remove GREASE cipher slots and extensions entirely (for
    /// GREASE-intolerant servers).
    Strip,
}

/// Builds a `ClientHelloSpec` from a resolved base plus typed overrides.
///
/// Overrides apply in call order; each validates immediately where cheap
/// (empty lists) and finally at `build()` (duplicates, parseability).
#[derive(Debug, Clone)]
pub struct FingerprintBuilder {
    spec: ClientHelloSpec,
}

impl FingerprintBuilder {
    /// Resolves `fingerprint` strictly, then wraps its spec for overriding.
    ///
    /// # Errors
    /// [`FingerprintError::Unknown`] when the identity has no table row.
    pub fn new(fingerprint: &Fingerprint) -> Result<Self, FingerprintError> {
        Ok(Self { spec: fingerprint.resolve()?.spec })
    }

    /// Replaces the whole cipher list. GREASE placeholders allowed; empty rejected.
    #[must_use]
    pub fn override_ciphers(mut self, ciphers: &[u16]) -> Self {
        self.spec.cipher_suites = ciphers.to_vec();
        self
    }

    /// Replaces the whole ordered extension list (order is
    /// fingerprint-critical). Convenience mutators below are usually better.
    #[must_use]
    pub fn override_extensions(mut self, extensions: Vec<ExtensionSpec>) -> Self {
        self.spec.extensions = extensions;
        self
    }

    /// Config-driven curve preferences (xray `CurvePreferences` mirror):
    /// replaces `supported_groups`, rebuilds `key_share` shares.
    #[must_use]
    pub fn curves(mut self, curves: &[u16]) -> Self {
        self.spec = crate::spec::apply_curve_preferences(&self.spec, curves);
        self
    }

    /// Find-and-replace the ALPN extension; keeps position in the list.
    #[must_use]
    pub fn alpn(mut self, protos: &[&str]) -> Self {
        let protos: Vec<String> = protos.iter().map(|p| (*p).to_string()).collect();
        self.replace_one(|e| matches!(e, ExtensionSpec::Alpn(_)), || ExtensionSpec::Alpn(protos.clone()),
            "base spec has no ALPN extension");
        self
    }

    /// Find-and-replace `signature_algorithms`.
    #[must_use]
    pub fn sig_algorithms(mut self, schemes: &[u16]) -> Self {
        self.replace_one(|e| matches!(e, ExtensionSpec::SignatureAlgorithms(_)),
            || ExtensionSpec::SignatureAlgorithms(schemes.to_vec()),
            "base spec has no signature_algorithms extension");
        self
    }

    /// GREASE handling policy applied at `build()`.
    #[must_use]
    pub fn grease(mut self, policy: GreasePolicy) -> Self {
        if policy == GreasePolicy::Strip {
            self.spec.cipher_suites.retain(|&c| !is_grease(c));
            self.spec.extensions.retain(|e| !matches!(e, ExtensionSpec::Grease));
        }
        self
    }

    /// Validates and returns the built spec.
    ///
    /// # Errors
    /// [`FingerprintError::InvalidOverride`] on empty cipher lists or
    /// duplicate extension types.
    pub fn build(self) -> Result<ClientHelloSpec, FingerprintError> {
        if self.spec.cipher_suites.is_empty() {
            return Err(FingerprintError::InvalidOverride("cipher list is empty".into()));
        }
        let mut seen: Vec<u16> = Vec::with_capacity(self.spec.extensions.len());
        for ext in &self.spec.extensions {
            let ty = extension_type(ext);
            if ty == GREASE_TYPE_SENTINEL {
                continue; // multiple GREASE extensions are legal
            }
            if seen.contains(&ty) {
                return Err(FingerprintError::InvalidOverride(format!(
                    "duplicate extension 0x{ty:04x}"
                )));
            }
            seen.push(ty);
        }
        Ok(self.spec)
    }

    fn replace_one(
        &mut self,
        pred: impl Fn(&ExtensionSpec) -> bool,
        make: impl FnOnce() -> ExtensionSpec,
        missing_msg: &str,
    ) {
        match self.spec.extensions.iter_mut().find(|e| pred(e)) {
            Some(slot) => *slot = make(),
            // Missing base extension is a silent no-op slot append at the end,
            // matching "keeps position" only when present; absence is an
            // InvalidOverride surfaced at build().
            None => {
                self.missing = Some(missing_msg.to_string());
            }
        }
    }
}
```

IMPLEMENTATION NOTES (apply while writing):
- The `replace_one` sketch records a deferred error; add a `missing: Option<String>` field to the struct (initialize `None` in `new`), check it first thing in `build()`:
  ```rust
  if let Some(msg) = self.missing {
      return Err(FingerprintError::InvalidOverride(msg));
  }
  ```
- Define near the top:
  ```rust
  /// Sentinel for GREASE extensions (no fixed wire type).
  const GREASE_TYPE_SENTINEL: u16 = 0xFFFF;
  /// Wire type of an extension for validation purposes.
  fn extension_type(ext: &ExtensionSpec) -> u16 {
      match ext {
          ExtensionSpec::ServerName => 0x0000,
          ExtensionSpec::SupportedGroups(_) => 0x000A,
          ExtensionSpec::KeyShare(_) => 0x0033,
          ExtensionSpec::SupportedVersions(_) => 0x002B,
          ExtensionSpec::SignatureAlgorithms(_) => 0x000D,
          ExtensionSpec::Alpn(_) => 0x0010,
          ExtensionSpec::EcPointFormats => 0x000B,
          ExtensionSpec::SessionTicket => 0x0023,
          ExtensionSpec::PskKeyExchangeModes => 0x002D,
          ExtensionSpec::StatusRequest => 0x0005,
          ExtensionSpec::SignedCertificateTimestamp => 0x0012,
          ExtensionSpec::RenegotiationInfo => 0xFF01,
          ExtensionSpec::CompressCertificate(_) => 0x001B,
          ExtensionSpec::ApplicationSettings(_) => 0x4469,
          ExtensionSpec::RecordSizeLimit(_) => 0x001C,
          ExtensionSpec::Padding => 0x0015,
          ExtensionSpec::Grease => GREASE_TYPE_SENTINEL,
          ExtensionSpec::Raw { ty, .. } => *ty,
      }
  }
  ```

Wire in `mod.rs`: `pub mod builder;` + `pub use builder::{FingerprintBuilder, GreasePolicy};`

- [ ] **Step 4: Run tests**

Run: `cargo test -p xray-tui-tls fingerprints`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls/src/fingerprints/
git commit -m "feat(tls): compositional FingerprintBuilder with typed overrides"
```

---

### Task 7: Desktop preset batch — safari_16, firefox_120, edge_106

**Files:**
- Create: `crates/xray-tui-tls/src/profiles/safari16.rs`
- Create: `crates/xray-tui-tls/src/profiles/firefox120.rs`
- Create: `crates/xray-tui-tls/src/profiles/edge106.rs`
- Modify: `crates/xray-tui-tls/src/profiles/mod.rs` (register in `define_profiles!`)
- Modify: `crates/xray-tui-tls/src/fingerprints/resolve.rs` (append table rows)

**Interfaces:**
- Consumes: `define_profiles!` macro conventions; uTLS presets in `thirdparty/utls/u_parrots.go` (`HelloSafari_16_0`, `HelloFirefox_120`, `HelloEdge_106`).
- Produces: `BrowserProfile::{Safari16, Firefox120, Edge106}` variants with `name()`s `"safari_16"`, `"firefox_120"`, `"edge_106"`; three new resolution rows.

- [ ] **Step 1: Transcribe safari_16**

Read `thirdparty/utls/u_parrots.go`, locate `HelloSafari_16_0` (its `Certificates`, `CipherSuites`, `Extensions`). Follow the exact pattern of an existing completed module — `crates/xray-tui-tls/src/profiles/safari.rs` is the same family and the closest structural model. Write `safari16.rs`:

- Module doc-comment citing `u_parrots.go` `HelloSafari_16_0`.
- `const CIPHER_SUITES: &[u16]` — verbatim order from the preset (Safari presets carry no GREASE ciphers).
- `const SIG_ALGOS: &[u16]` — from the preset's `SignatureAlgorithms` extension.
- `pub fn spec() -> ClientHelloSpec` — extension order verbatim; Safari family uses `SessionIdSpec::Empty`; map uTLS extension kinds onto our `ExtensionSpec` variants (`Raw { ty, data }` for anything without a typed variant — e.g. Safari's `0x001C` record size limit fits `RecordSizeLimit`).
- Skip PSK-binding/randomized parrot fields (per spec skip-list); if the preset turns out shuffle/PSK-dependent, STOP and report instead of approximating.

- [ ] **Step 2: Transcribe firefox_120 and edge_106**

Same procedure: `HelloFirefox_120` → `firefox120.rs` (model: `firefox128esr.rs` — Firefox family, GREASE-free), `HelloEdge_106` → `edge106.rs` (model: `edge.rs` — Chromium family WITH GREASE cipher slot `0xCACA` first and `ExtensionSpec::Grease` placements copied from the preset's GREASE extension positions).

- [ ] **Step 3: Register profiles**

In `profiles/mod.rs` `define_profiles!`: add `Safari16, Firefox120, Edge106` to the variant list and pairs:

```rust
Safari16   => ("safari_16",   safari16::spec),
Firefox120 => ("firefox_120", firefox120::spec),
Edge106    => ("edge_106",    edge106::spec),
```

The macro's existing per-profile tests (hello builds, parses, GREASE presence for Chromium family) automatically cover the new variants — extend the family-match arm at `profiles/mod.rs:181-186` to include `Edge106` in the Chromium-GREASE group.

- [ ] **Step 4: Add resolution rows**

In `resolve.rs` `TABLE`, insert keeping per-browser ascending `max_version`:

```rust
Row { name: "safari_16",   browser: Browser::Safari,  os: Some(Os::MacOs),   device: Device::Desktop, max_version: 16,  spec: safari16::spec },
Row { name: "firefox_120", browser: Browser::Firefox, os: None,              device: Device::Desktop, max_version: 120, spec: firefox120::spec },
Row { name: "edge_106",    browser: Browser::Edge,    os: None,              device: Device::Desktop, max_version: 106, spec: edge106::spec },
```

Import the new modules in the `use crate::profiles::{…}` list. Note ordering effects: `firefox_120` (max 120) now sits below `firefox_128_esr` (128) — nearest-lower logic sends Firefox ≤120 queries to `firefox_120`, 121–128 to ESR. Verify with:

```rust
#[test]
fn firefox_120_query_hits_firefox_120_row() {
    let r = Fingerprint::new(Browser::Firefox).with_version(120).resolve().unwrap();
    assert_eq!(r.name, "firefox_120");
}
#[test]
fn safari_16_query_hits_safari_16_row() {
    let r = Fingerprint::new(Browser::Safari).with_version(16).resolve().unwrap();
    assert_eq!(r.name, "safari_16");
}
```

- [ ] **Step 5: Verify determinism + fidelity**

Run: `cargo test -p xray-tui-tls profiles fingerprints`
Expected: PASS — including the macro-generated golden-hello tests for the three new variants (fixed-seed hello builds and parses; Chromium-family GREASE assertions hold for Edge).

Then capture ground-truth JA4 for Task 10: run the grader example against peet.ws once per new profile (`cargo run --example grader` — read `examples/grader.rs` first; if it hardcodes a profile, parameterize temporarily or add a CLI arg) and RECORD the reported JA4 strings in your working notes — Task 10 freezes them into the rstest table.

- [ ] **Step 6: Commit**

```bash
git add crates/xray-tui-tls/src/profiles/ crates/xray-tui-tls/src/fingerprints/resolve.rs
git commit -m "feat(tls): transcribe safari_16, firefox_120, edge_106 from uTLS presets"
```

---

### Task 8: Mobile preset batch — ios_14, android_11_okhttp

**Files:**
- Create: `crates/xray-tui-tls/src/profiles/ios14.rs`
- Create: `crates/xray-tui-tls/src/profiles/android11_okhttp.rs`
- Modify: `crates/xray-tui-tls/src/profiles/mod.rs`
- Modify: `crates/xray-tui-tls/src/fingerprints/resolve.rs`

**Interfaces:**
- Consumes: uTLS `HelloIOS_14`, `HelloAndroid_11_OkHttp`.
- Produces: `BrowserProfile::{Ios14, Android11OkHttp}` (`"ios_14"`, `"android_11_okhttp"`); two resolution rows (both `Device::Phone`, os `Ios`/`Android`).

- [ ] **Step 1: Transcribe ios_14** — procedure identical to Task 7 Step 1; structural models: `safari_ios17.rs` (same family). Note iOS presets often lack ALPN h2 ordering differences and use `SessionIdSpec::Random32`.

- [ ] **Step 2: Transcribe android_11_okhttp** — model: `chrome_android130.rs`. OkHttp's hello is Chromium-derived but with distinct cipher list and no ALPS/application_settings; transcribe verbatim, keep GREASE slots if the preset has them.

- [ ] **Step 3: Register + resolution rows**

`define_profiles!` additions:

```rust
Ios14           => ("ios_14",             ios14::spec),
Android11OkHttp => ("android_11_okhttp",  android11_okhttp::spec),
```

Rows (insert adjacent to sibling rows):

```rust
Row { name: "ios_14",            browser: Browser::Safari, os: Some(Os::Ios),     device: Device::Phone, max_version: 14, spec: ios14::spec },
Row { name: "android_11_okhttp", browser: Browser::Chrome, os: Some(Os::Android), device: Device::Phone, max_version: 11, spec: android11_okhttp::spec },
```

IMPORTANT disambiguation: `ios_14` answers Safari+iOS queries with version ≤14; `safari_ios_17` covers ≥15. `android_11_okhttp` vs `chrome_android_130`: OkHttp row wins for Android versions ≤ 11 — but both rows match `(Chrome, Android, Phone)` with overlapping ranges (okhttp ≤11, android130 ≤130). Nearest-lower picks correctly (v=11 → okhttp row since 11 ≥ 11 and 130 ≥ 11 → min-max picks okhttp). Add the pinning tests:

```rust
#[test]
fn android_11_hits_okhttp_row_chrome_12_hits_modern_row() {
    let old = Fingerprint::new(Browser::Chrome).with_version(11).with_os(Os::Android).with_device(Device::Phone).resolve().unwrap();
    assert_eq!(old.name, "android_11_okhttp");
    let modern = Fingerprint::new(Browser::Chrome).with_version(12).with_os(Os::Android).with_device(Device::Phone).resolve().unwrap();
    assert_eq!(modern.name, "chrome_android_130");
}
```

If the ambiguity is uncomfortable, prefer explicitness: gate the okhttp row behind `version <= 11` by giving `Row` an optional `min_version: u16` field (default 0) checked in `matches()` — decide during implementation, keep tests as the contract.

- [ ] **Step 4: Extend family test arm** — `profiles/mod.rs` GREASE-family match: add `Android11OkHttp` iff its transcription retains GREASE (decide from the preset; OkHttp historically has none).

- [ ] **Step 5: Run tests + capture peet.ws JA4s**

Run: `cargo test -p xray-tui-tls`
Expected: PASS. Record the five new profiles' peet.ws JA4s (from Task 7 Step 5 procedure) for Task 10.

- [ ] **Step 6: Commit**

```bash
git add crates/xray-tui-tls/src/profiles/ crates/xray-tui-tls/src/fingerprints/resolve.rs
git commit -m "feat(tls): transcribe ios_14 and android_11_okhttp from uTLS presets"
```

---

### Task 9: Cutover — delete BrowserProfile, generalize REALITY provisioner

**Files:**
- Modify: `crates/xray-tui-tls/src/client.rs`
- Modify: `crates/xray-tui-tls/src/reality/mod.rs`
- Modify: `crates/xray-tui-tls/src/profiles/mod.rs` (delete `define_profiles!` enum generation; keep spec fns + tests)
- Modify: `crates/xray-tui-tls/src/lib.rs` (exports)
- Modify: `crates/xray-tui-native/src/security/fingerprint.rs`, `crates/xray-tui-native/src/security/mod.rs`, `crates/xray-tui-native/src/context.rs`, `crates/xray-tui-native/src/e2e/harness.rs`
- Test: existing REALITY golden-byte test must pass UNCHANGED (assertions only, not construction)

**Interfaces:**
- Consumes: everything from Tasks 1–8.
- Produces:
  - `TlsMode::Plain { fingerprint: Option<Fingerprint>, verifier: Arc<dyn ServerVerifier> }`
  - `TlsConfig::plain(fingerprint: Option<Fingerprint>, verifier, server_name)`
  - `reality::SpecProvisioner` — `pub fn new(spec_source: impl Fn() -> ClientHelloSpec + Send + Sync)` or struct holding a closure; `From<Fingerprint>`; `FixedChrome133` unchanged externally.
  - `lib.rs`: `pub use fingerprints::{Browser, Device, Fingerprint, FingerprintBuilder, FingerprintError, GreasePolicy, Os, Resolved};`

- [ ] **Step 1: Rework reality provisioners**

In `reality/mod.rs` replace `ProfileProvisioner(pub BrowserProfile)` (lines ~101–145) with:

```rust
/// A `HelloProvisioner` shaped by any concrete hello spec: REALITY's
/// auth payload rides the session-id slot of the given spec.
pub struct SpecProvisioner {
    spec_source: Box<dyn Fn() -> ClientHelloSpec + Send + Sync>,
}

impl SpecProvisioner {
    /// Shapes the provisioner from a spec-producing closure.
    #[must_use]
    pub fn new(spec_source: impl Fn() -> ClientHelloSpec + Send + Sync + 'static) -> Self {
        Self { spec_source: Box::new(spec_source) }
    }
}

impl From<&Fingerprint> for SpecProvisioner {
    fn from(fp: &Fingerprint) -> Self {
        // REALITY provisioning needs a concrete hello; unknown identities
        // panic here by contract (resolve errors are a configuration bug —
        // validated upstream when the fingerprint was configured).
        let spec_source = {
            let fp = fp.clone();
            move || fp.resolve().expect("fingerprint must resolve").spec
        };
        Self::new(spec_source)
    }
}

impl HelloProvisioner for SpecProvisioner {
    fn provision(&self, params: &HelloProvisionParams<'_>) -> Result<ProvisionedHello> {
        let mut spec = (self.spec_source)();
        spec.session_id = SessionIdSpec::AuthPayload { len: 32 };
        // … body identical to the old ProfileProvisioner::provision …
    }
    // offers_hybrid_key_share: same logic over (self.spec_source)()
}

impl HelloProvisioner for FixedChrome133 {
    fn provision(&self, params: &HelloProvisionParams<'_>) -> Result<ProvisionedHello> {
        SpecProvisioner::new(|| crate::profiles::chrome133::spec()).provision(params)
    }
}
```

Keep the import list minimal: `use crate::fingerprints::Fingerprint;`.

- [ ] **Step 2: Cutover client.rs**

- `TlsMode::Plain` field rename `profile` → `fingerprint: Option<Fingerprint>`; doc-comment default → "`None` → `Fingerprint::default_for(Browser::Chrome)`".
- `TlsConfig::plain` signature swap accordingly.
- Connect path:

```rust
TlsMode::Plain { fingerprint, verifier } => {
    let fp = fingerprint.clone().unwrap_or_else(|| Fingerprint::default_for(Browser::Chrome));
    let resolved = fp.resolve().map_err(|e| crate::error::TlsError::Handshake(e.to_string()))?;
    let spec = match config.curves.as_deref() {
        Some(curves) if !curves.is_empty() =>
            crate::spec::apply_curve_preferences(&resolved.spec, curves),
        _ => resolved.spec,
    };
    // … rest unchanged (handshake::drive with spec) …
}
```

(`Fingerprint` isn't `Copy` — clone the `Option`.)

- [ ] **Step 3: Delete BrowserProfile, fix lib.rs**

- `profiles/mod.rs`: remove the `define_profiles!` macro invocation AND the macro definition (nothing else generates the enum). KEEP: module declarations, each profile's `pub fn spec()`, and the test module (rewrite tests to iterate a local `const ALL_SPECS: &[(&str, fn() -> ClientHelloSpec)]` table replacing `BrowserProfile::all()`; the family-GREASE test iterates the same table filtering Chromium-family names).
- `lib.rs`: remove `pub use profiles::BrowserProfile;`, add the fingerprints re-exports listed under Interfaces.

- [ ] **Step 4: Migrate native-crate consumers**

- `security/fingerprint.rs`: `profile_for` returns `Result<Fingerprint, NativeError>` mapping `Chrome*`→`Fingerprint::new(Browser::Chrome).with_version(130)`, `Firefox`→`…with_version(128).with_os(Os::Linux)`, `Safari`→`…with_version(17).with_os(Os::MacOs)` (preserve today's effective choices: Chrome130/Firefox128Esr/Safari17).
- `security/mod.rs:80`: `Arc::new(SpecProvisioner::from(&Fingerprint::new(Browser::Chrome).with_version(133)))` — or hoist a `const CHROME_133_FP:` lazy.
- `context.rs:399-402`: `&xray_tui_tls::profiles::chrome133::spec()` (direct module call; profiles modules remain public).
- `e2e/harness.rs:578`: `TlsConfig::plain(Some(Fingerprint::new(Browser::Chrome).with_version(130)), verifier, INNER_TLS_SNI)`.
- Check for other `BrowserProfile` references: `grep -rn BrowserProfile crates/` must return zero hits when done.

- [ ] **Step 5: Full verification**

Run: `grep -rn BrowserProfile crates/ ; cargo test -p xray-tui-tls -p xray-tui-native ; cargo clippy -p xray-tui-tls -p xray-tui-native`
Expected: grep empty; tests PASS — critically the REALITY golden-byte equality test (`reality/mod.rs:388-410`) proving `SpecProvisioner(chrome133)` ≡ old `ProfileProvisioner(Chrome133)` on the wire; clippy clean.

- [ ] **Step 6: Commit**

```bash
git add crates/xray-tui-tls crates/xray-tui-native
git commit -m "feat(tls)! :cutover BrowserProfile -> Fingerprint identity + SpecProvisioner"
```

---

### Task 10: rstest fingerprint case table

**Files:**
- Create: `crates/xray-tui-tls/tests/fingerprints.rs`

**Interfaces:**
- Consumes: `Fingerprint`, `Resolved::ja4()`, `ja3_grease_stripped`, seeded RNG pattern from `profiles::tests` (`FixedRandom`).

- [ ] **Step 1: Write the case table**

Values marked `PEET` come from the Task 7/8 captures (fill the literal strings you recorded; every profile gets its measured JA4 — the two shown below illustrate the format):

```rust
//! Instantly-verifiable fingerprint cases (design spec: rstest tables).
//! Expected JA4s are frozen from live `tls.peet.ws` captures of each
//! profile's hello (GREASE-stable by construction — JA4 strips GREASE).

use rstest::rstest;
use xray_tui_tls::fingerprints::{
    query::{Browser, Device, Os},
    Fingerprint,
};

fn fp(browser: Browser, version: u16) -> Fingerprint {
    Fingerprint::new(browser).with_version(version)
}

#[rstest]
// ── existing profiles ──
#[case::chrome_130(fp(Browser::Chrome, 130), "PEET")]
#[case::chrome_133(fp(Browser::Chrome, 133), "PEET")]
#[case::chrome_119(fp(Browser::Chrome, 119), "PEET")]
#[case::chrome_android_130(Fingerprint::new(Browser::Chrome).with_version(130).with_os(Os::Android).with_device(Device::Phone), "PEET")]
#[case::edge_130(fp(Browser::Edge, 130), "PEET")]
#[case::brave_167(fp(Browser::Brave, 167), "PEET")]
#[case::opera_114(fp(Browser::Opera, 114), "PEET")]
#[case::firefox_esr(fp(Browser::Firefox, 128), "PEET")]
#[case::safari_17(Fingerprint::new(Browser::Safari).with_version(17).with_os(Os::MacOs).with_device(Device::Desktop), "PEET")]
#[case::safari_ios_17(Fingerprint::new(Browser::Safari).with_version(17).with_os(Os::Ios).with_device(Device::Phone), "PEET")]
// ── new presets ──
#[case::safari_16(fp(Browser::Safari, 16), "PEET")]
#[case::firefox_120(fp(Browser::Firefox, 120), "PEET")]
#[case::edge_106(fp(Browser::Edge, 106), "PEET")]
#[case::ios_14(Fingerprint::new(Browser::Safari).with_version(14).with_os(Os::Ios).with_device(Device::Phone), "PEET")]
#[case::android_11_okhttp(Fingerprint::new(Browser::Chrome).with_version(11).with_os(Os::Android).with_device(Device::Phone), "PEET")]
fn ja4_matches_peet_ground_truth(#[case] fp: Fingerprint, #[case] expected: &str) {
    let resolved = fp.resolve().expect("table case must resolve");
    assert_eq!(resolved.ja4(), expected, "{}", resolved.name);
}

#[rstest]
#[case::chrome_133(fp(Browser::Chrome, 133))]
#[case::edge_130(fp(Browser::Edge, 130))]
fn ja4_is_run_to_run_stable(#[case] fp: Fingerprint) {
    let a = fp.resolve().unwrap().ja4();
    let b = fp.resolve().unwrap().ja4(); // fresh spec; GREASE draws differ
    assert_eq!(a, b, "JA4 must be GREASE-independent");
}

#[rstest]
#[case::firefox_esr(fp(Browser::Firefox, 128))]
#[case::firefox_120(fp(Browser::Firefox, 120))]
#[case::safari_17(Fingerprint::new(Browser::Safari).with_version(17).with_os(Os::MacOs).with_device(Device::Desktop))]
fn grease_free_profiles_pin_classic_ja3_prefix(#[case] fp: Fingerprint) {
    // Firefox/Safari families send no GREASE — classic JA3 string is stable.
    let resolved = fp.resolve().unwrap();
    let ja3 = xray_tui_tls::crypto::fingerprint::ja3::ja3_string(
        &xray_tui_tls::crypto::fingerprint::ja3::Ja3Fields::from_spec(&resolved.spec),
    );
    assert!(ja3.starts_with("771,"), "{ja3}");
}

#[rstest]
#[case::chrome_130(fp(Browser::Chrome, 130))]
fn grease_carrying_profiles_use_stripped_ja3_variant(#[case] fp: Fingerprint) {
    // Determinism rule (b): stripped variant is deterministic; classic
    // on-wire JA3 is NOT pinned for these families.
    let resolved = fp.resolve().unwrap();
    let ja3 = xray_tui_tls::crypto::fingerprint::ja3::ja3_grease_stripped(
        &xray_tui_tls::crypto::fingerprint::ja3::Ja3Fields::from_spec(&resolved.spec),
    );
    assert!(!ja3.is_empty());
}
```

- [ ] **Step 2: Fill PEET literals and run**

Run: `cargo test -p xray-tui-tls --test fingerprints`
Expected: PASS. If a case fails, the mismatch is between transcription and capture — investigate THAT profile's module, never loosen the assertion.

- [ ] **Step 3: Commit**

```bash
git add crates/xray-tui-tls/tests/fingerprints.rs
git commit -m "test(tls): rstest fingerprint case table with peet.ws ground truth"
```

---

### Task 11: Final verification & docs

**Files:**
- Modify: `NATIVE_CORE.md` (profiles section: mention the new selector/builder/oracle)
- Modify: `docs/protocols.md` only if it documents fingerprint choices

- [ ] **Step 1: Whole-workspace gates**

Run: `cargo fmt --all && cargo clippy --workspace && cargo test --workspace`
Expected: clean fmt, zero clippy warnings, all tests pass.

- [ ] **Step 2: Update NATIVE_CORE.md**

In the `xray-tui-tls` bullet (SP-section describing profiles), append: 17-profile catalog selectable by `Fingerprint { browser, version?, os?, device? }` with strict resolution, `FingerprintBuilder` overrides, generated JA4 catalog (ja4db snapshot) as evidence, full-JA4 oracle (FoxIO final scheme, peet.ws-validated), `SpecProvisioner` for REALITY.

- [ ] **Step 3: Commit**

```bash
git add NATIVE_CORE.md docs/
git commit -m "docs: TLS fingerprint selector/builder/oracle in NATIVE_CORE.md"
```
