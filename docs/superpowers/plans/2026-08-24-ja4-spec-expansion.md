# JA4-Driven Fingerprint Expansion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Generate JA4-faithful ClientHelloSpecs for every attributable identity in ja4db-export (998 unique TLS-1.3 fingerprints → filtered roster), behind a compact `spec!` macro, one module per browser family, all offline-verified against our own JA4 oracle.

**Architecture:** Python generator (`gen_specs.py`) parses raw JA4 strings, mirrors the Rust JA4 hash as a self-check, attributes rows via ua-parser joined across both CSVs, synthesizes wire shape from per-family templates, and emits `profiles/generated/<browser>.rs` plus resolver rows. Hand-written transcriptions stay authoritative where identities overlap.

**Tech Stack:** Python 3 + ua-parser[regex] (generator), Rust workspace crate `xray-tui-tls` (macro, resolver, tests).

**Spec:** `docs/superpowers/specs/2026-08-24-ja4-spec-expansion-design.md` — binding fidelity contract, ambiguity policy, tier semantics.

## Global Constraints

- Clippy pedantic+nursery zero warnings; `cargo fmt` clean; `cargo test --workspace` green at every task boundary.
- `thirdparty/*` strictly read-only.
- Tool restrictions: `ast_edit` forbidden; use read/search tools, not bash cat/grep.
- TLS-1.3 rows only (`t13d` prefix); t12/t11 skipped (engine constraint, okhttp precedent).
- Generated entries are JA4-faithful tier; wire-exact tier = the 17 hand-written profiles, untouched.
- Identity collision winner: highest observation_count, verified-first; drops recorded in manifest.
- Resolver merge rule: hand-written row wins on identity/band overlap.
- Error alternatives listing: first 8 matches plus total count.
- Never dispatch subagents from subagents (execution-time constraint).

---

### Task 1: Raw-string parser + JA4 hash mirror

**Files:**
- Create: `crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py`
- Test: inline `pytest`-style asserts runnable via `python3 gen_specs.py --selftest`

**Interfaces:**
- Produces: `parse_raw(s: str) -> RawComponents | None`; dataclass `RawComponents {ja4_a: str, ciphers: list[int], exts_sorted: list[int], sigalgs_ordered: list[int], alpn_first: str}`; `ja4_hash(rc: RawComponents) -> str` (full 36-char FoxIO JA4); `load_raw_rows(path) -> dict[str, RawComponents]` keyed by recomputed JA4.

- [ ] **Step 1: Write parse + hash functions**

```python
JA4_A_RE = re.compile(r"^t13d\d{2}(?:99|\d{2})(?:h2|h1|00|[a-z0-9]{2})$")
SEG_SPLIT = "__"

def parse_raw(s):
    # format: <ja4_a>[_<ciphers>[_<exts>[_<sigalgs>]]]; segments comma-hex
    if not s.startswith("t13d"):
        return None
    parts = s.split("_")
    if len(parts) < 2:
        return None
    ja4_a = parts[0]
    if not JA4_A_RE.match(ja4_a):
        return None
    ciphers = [int(x, 16) for x in parts[1].split(",") if x]
    exts = [int(x, 16) for x in parts[2].split(",")] if len(parts) > 2 else []
    sigalgs = [int(x, 16) for x in parts[3].split(",")] if len(parts) > 3 else []
    return RawComponents(ja4_a, ciphers, exts, sigalgs)
```

Hash mirror must replicate `crypto/fingerprint/ja4.rs` exactly (read that file first): GREASE ids excluded everywhere; SNI 0x0000, ALPN 0x0010, padding 0x0015 counted but excluded from hash2's ext list; hash1 = sha256[:12] of sorted non-GREASE ciphers 4-hex comma-joined; hash2 = sha256[:12] of sorted filtered exts + `_` + sig-algs in hello order; A-part counts clamped at 99. Reuse GREASE ranges: 0x0A0A–0x1F1F pattern `(v & 0x0f0f) == 0x0a0a`.

```python
def is_grease(v): return (v & 0x0F0F) == 0x0A0A
HASH2_EXCLUDED = {0x0000, 0x0010, 0x0015}

def ja4_hash(rc):
    cs = sorted(f"{c:04x}" for c in rc.ciphers if not is_grease(c))
    es = sorted(f"{e:04x}" for e in rc.exts_sorted if not is_grease(e)
                and e not in HASH2_EXCLUDED)
    ss = ",".join(f"{a:04x}" for a in rc.sigalgs_ordered if not is_grease(a))
    h1 = hashlib.sha256(",".join(cs).encode()).hexdigest()[:12]
    h2 = hashlib.sha256(("_".join([",".join(es)] + ([ss] if ss else []))
                         ).encode()).hexdigest()[:12]
    return f"{rc.ja4_a}_{h1}_{h2}"
```

Verify exact concatenation/separators against `ja4.rs` lines 56–120 before finalizing — the Rust file is ground truth, this sketch is orientation only.

- [ ] **Step 2: Self-test against the real corpus**

Load `thirdparty/ja4db-export/csv/ja4_fingerprint_string.csv`, recompute hashes, assert ≥99% match rate against the same fingerprints appearing in `csv/ja4_fingerprint.csv`'s `ja4_fingerprint` column. Any systematic mismatch = separator/order bug: fix until 100% of matched keys round-trip. Print unmatched count; investigate any non-zero remainder (may be corpus quirks — record them in the script header).

Run: `python3 crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py --selftest`
Expected: PASS, 100% key round-trip on shared fingerprints.

- [ ] **Step 3: Commit**

```bash
git add crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py
git commit -m "feat(tls): raw JA4 string parser with Rust-mirrored hash self-check"
```

### Task 2: Attribution, join, dedup, manifest

**Files:**
- Modify: `crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py`
- Create: `crates/xray-tui-tls/src/fingerprints/catalog/specs_manifest.json` (generated, committed)

**Interfaces:**
- Consumes: Task 1 `parse_raw`/`ja4_hash`.
- Produces: `build_manifest(csv_dir: str) -> Manifest` writing `specs_manifest.json`: `{"entries": [{name, browser, browser_major, os, os_major, device, ja4, raw: RawComponents, family, observation_count, verified, fallback: bool}], "stats": {rows_in, kept, dropped_no_ua, dropped_t12, collisions_dropped}}`. Entry `name` = `{browser}_{major}[_{os}_{device}]` snake_case, disambiguated with `_2` suffix when two entries share name+band.

- [ ] **Step 1: Implement attribution + join**

Reuse gen.py's `map_browser`/`map_os`/`derive_device`/`parse_ua` (copy them in — gen.py stays untouched). Rules per spec:
- Parse UA from BOTH csvs; keep rows where Browser+major, Os+major, Device all resolve (Os may map to Desktop default only when UA clearly desktop — reuse gen.py logic verbatim).
- Join string-file rows to fingerprint-file labels by computed JA4 == `ja4_fingerprint` column; enrich observation_count (sum both sides), verified flag.
- Skip non-`t13d`.
- Dedup identical (ja4, identity); collisions (same identity, different ja4): winner max observation_count then verified-first then lexicographic ja4 (deterministic tiebreak).

- [ ] **Step 2: Run and inspect stats**

Run: `python3 ...gen_specs.py --manifest`
Expected: manifest written; stats printed. Sanity assertions inside the script: kept ≥ 50; no entry lacks family assignment; names unique.

- [ ] **Step 3: Commit**

```bash
git add crates/xray-tui-tls/src/fingerprints/catalog/specs_manifest.json \
        crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py
git commit -m "feat(tls): attributed+joined fingerprint manifest from ja4db corpus"
```

### Task 3: Family template registry

**Files:**
- Modify: `crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py`

**Interfaces:**
- Consumes: manifest entries.
- Produces: per-entry `wire` block added to manifest: `{cipher_order: [u16], extensions: [{ty, kind, args}], session_id: "random32"|"empty", compression: [0]}` where `kind` ∈ ExtensionSpec variant names (ServerName, SupportedGroups, KeyShare, SupportedVersions, SignatureAlgorithms, Alpn, EcPointFormats, SessionTicket, PskKeyExchangeModes, StatusRequest, SignedCertificateTimestamp, RenegotiationInfo, CompressCertificate, ApplicationSettings, RecordSizeLimit, Padding, Grease, Raw).

- [ ] **Step 1: Template registry**

Templates keyed by family (`chrome_desktop`, `firefox`, `safari`, `chrome_android`, `safari_ios`, `okhttp`, fallback `ascending`). Ground truth for contents rules: read the 17 hand-written profiles (`crates/xray-tui-tls/src/profiles/*.rs`) and transcribe their construction rules into Python:

```python
CHROME_EXT_ORDER = [0x0000(GREASE-first per chrome133.rs), ...]
# per-ext constructor example:
def chrome_ext(ty, row):
    if ty == 0x000b: return {"kind": "EcPointFormats"}
    if ty == 0x000a: return {"kind": "SupportedGroups",
        "args": [[0x001b, 0x001d, 0x0100, 4588, 0x0017, 0x0018]]}  # from profile
    ...
```

Rules (from the transcription sources, verified against profiles/*.rs):
- GREASE ext id present in row → emit Grease slot at the family's canonical position.
- supported_versions: always `[0x0304, 0x0303]` (engine is 1.3-only; t12 rows already skipped).
- key_share groups: family canonical list intersected with SupportedGroups ids; X25519MLKEM768 (4588) maps to hybrid entry when present.
- Extension order: family canonical order filtered to row's ext-id set; unknown ids appended ascending with `Raw {ty, data: []}` and flagged low-fidelity in the manifest.
- Cipher order: family canonical sequence filtered to row's set; leftovers appended ascending.
- Fallback template: everything ascending, minimal standard contents, `fallback: true`.

- [ ] **Step 2: Cross-check against oracle**

For every templated entry, reconstruct Ja3Fields-equivalent lists and verify the A-part letters/count segments equal `raw.ja4_a` (hash equality was Task 1's job; this checks the wire synthesis didn't drop/add anything).

Run: `python3 ...gen_specs.py --selftest` (extended)
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git commit -am "feat(tls): family templates synthesize wire shape for manifest entries"
```

### Task 4: `spec!` declaration macro

**Files:**
- Modify: `crates/xray-tui-tls/src/profiles/mod.rs`
- Test: same file `#[cfg(test)] mod macro_tests`

**Interfaces:**
- Produces: `pub(crate) fn spec_from_parts(cipher_suites: Vec<u16>, extensions: Vec<ExtensionSpec>, session_id: SessionIdSpec) -> ClientHelloSpec` and macro:

```rust
spec! {
    chrome_gen_137,
    ciphers: GREASE, 1301, 1302, 1303, 0xc02b,
    session: random32,
    exts: grease, sni, groups[0x001b, x25519, mlkem768], versions[0x0304],
         sigalgs[0x0403, 0x0804, ...], alpn["h2", "http/1.1"], psk, padding
}
```

expanding to `fn chrome_gen_137() -> ClientHelloSpec` (name → fn, matching `ALL_SPECS` entry type `(&'static str, fn() -> ClientHelloSpec)`). Token grammar: bare ids map to unit variants (`sni`→ServerName, `psk`→PskKeyExchangeModes, …), `name[args]` to tuple variants (`groups[..u16]`, `keyshare[grease,x25519,mlkem768,p256,p384,p521]`, `versions[..u16]`, `sigalgs[..u16]`, `compress[zlib,brotli,zstd]`, `appsettings[..str]`, `rslimit[N]`, `raw[ty,data-hex]`). `legacy_version` always 0x0303, compression `[0]`.

- [ ] **Step 1: Write failing equivalence test**

Rebuild `chrome133::spec` body via the macro in the test module; assert `spec_from_macro == chrome133::spec()` field-for-field.

- [ ] **Step 2:** Run, expect FAIL (macro undefined). **Step 3:** implement macro + helper. **Step 4:** run, PASS. Also extend the existing ALL_SPECS uniqueness/format test to cover macro-generated fns.

- [ ] **Step 5: Commit**

```bash
git commit -am "feat(tls): declarative spec! macro for compact profile declarations"
```

### Task 5: Generator emits generated/<browser>.rs + ALL_SPECS wiring

**Files:**
- Modify: `crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py` (emitter)
- Create: `crates/xray-tui-tls/src/profiles/generated/{chrome,firefox,safari,chrome_android,safari_ios,okhttp,fallback}.rs` (generated, committed) + `mod.rs`
- Modify: `crates/xray-tui-tls/src/profiles/mod.rs` (aggregate)

**Interfaces:**
- Consumes: Task 3 manifest `wire` blocks; Task 4 macro grammar.
- Produces: per-family module with `#[rustfmt::skip] pub(crate) fn <name>() -> ClientHelloSpec` per entry (via `spec!`), doc comment `/// ja4=<full> obs=<n>`; `pub(crate) const GENERATED: &[GenEntry]` where `struct GenEntry { name, browser: Browser, os: Option<Os>, device: Device, major: u16, ja4: &'static str }` — the source-JA4 registry used by Task 7's gate. `Browser/Os/Device` enums may need new variants (e.g. `Browser::Vivaldi`…) — extend `query.rs` enums only with families present in the manifest; unknown UA families are dropped at Task 2 (already enforced by map_browser returning None).

Emitter renders Rust literals from manifest JSON deterministically (sorted keys, stable formatting); running the generator twice produces byte-identical output (assert in --selftest). Regenerated files are committed.

- [ ] **Step 1: Emitter** — render modules; wire `mod generated;` + merge `GENERATED` entries into `ALL_SPECS` iteration points (keep hand-written list separate: `HAND_WRITTEN` + `GENERATED`, concatenated where `ALL_SPECS` is consumed today).
- [ ] **Step 2:** `cargo build -p xray-tui-tls` clean; existing 17-profile tests still pass unchanged.
- [ ] **Step 3: Commit** — `feat(tls): generated JA4-faithful profile roster per browser family`

### Task 6: Resolver merge + capped errors

**Files:**
- Modify: `crates/xray-tui-tls/src/fingerprints/resolve.rs`
- Test: `resolve.rs` test module

**Interfaces:**
- Consumes: `profiles::generated::GENERATED`.
- Produces: `static TABLE` now = `HAND_ROWS.iter().chain(GENERATED_ROWS)` where `GENERATED_ROWS` is derived from `GENERATED` (Row{name: concat identity, min/max band from contiguous major runs}). Resolution lookup checks HAND first (precedence). `FingerprintError` alternatives listing: first 8 + `… and N more`.

- [ ] **Step 1: Failing tests** — (a) an identity covered by both tiers resolves to the hand-written spec name; (b) a generated-only identity (pick one from the roster, pin its name) resolves and `Resolved::ja4()` equals its registered source JA4; (c) error message with >8 alternatives shows truncation suffix.
- [ ] **Steps 2–5:** implement, pass, full `cargo test -p xray-tui-tls`, commit — `feat(tls): resolver merges generated roster under hand-written precedence`

### Task 7: Offline JA4 gate (all entries)

**Files:**
- Create: `crates/xray-tui-tls/tests/generated_ja4_gate.rs`
- Interfaces: consumes `GENERATED` registry + `hello::build_hello` + `crypto::fingerprint::full_ja4`.

- [ ] **Step 1: Failing-or-green gate test**

```rust
#[test]
fn every_generated_entry_hashes_to_source_ja4() {
    for entry in generated::GENERATED {
        let rng = FixedRandom { bytes: vec![0x5A; 512] }; // same fixture style as profiles/mod.rs tests
        let params = BuildParams { server_name: "example.org".into(), ..defaults };
        let hello = build_hello(&(entry.spec_fn)(), &params, &rng);
        let fields = Ja3Fields::from_hello(&hello);
        assert_eq!(full_ja4(&fields), entry.ja4, "entry {}", entry.name);
    }
}
```

(Adapt fixture construction to whatever `profiles/mod.rs:104-110` uses — copy it.) Note `full_ja4` is deterministic here: GREASE/SNI/padding are excluded or fixed, so RNG choice cannot flip the hash.

- [ ] **Step 2:** Run over the whole roster (expect hundreds of iterations, instant). Fix template/emitter bugs until green — this is THE deliverable gate.
- [ ] **Step 3: Commit** — `test(tls): offline JA4 oracle gate over full generated roster`

### Task 8: Live sweep + docs

**Files:**
- Modify: `examples/grader.rs` (accept roster mode: iterate GENERATED, print peet.ws JA4/JA3 + match verdict vs registered source JA4)
- Modify: ignored `tests/tls_peet_ws.rs` (sampled rstest: one entry per family/band)
- Modify: `NATIVE_CORE.md` (tier model, generator usage, roster scale), `crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py` header docs
- Create: `docs/tls-fingerprint-roster.md` (verification report: date, roster size, live results summary)

- [ ] **Step 1:** grader roster mode + sampled ignored tests.
- [ ] **Step 2:** Run live sweep against tls.peet.ws (network); record per-family pass/fail into `docs/tls-fingerprint-roster.md`. Investigate mismatches: template bug → fix + regenerate; corpus mislabel → note in report.
- [ ] **Step 3:** Docs; `cargo fmt/clippy/test --workspace` gates.
- [ ] **Step 4: Commit** — `docs(tls): live peet.ws sweep report + tier documentation`

---

## Verification (whole feature)

1. `cargo test -p xray-tui-tls` — includes the all-roster JA4 gate (Task 7).
2. `python3 gen_specs.py --selftest` — corpus-level round-trip + emitter determinism.
3. `cargo clippy --workspace --all-targets` zero warnings; `cargo fmt --all` clean.
4. Manual live sweep report committed (Task 8), sampled families verified against peet.ws.
