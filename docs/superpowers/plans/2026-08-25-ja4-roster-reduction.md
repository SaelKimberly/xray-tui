# JA4 Roster Reduction + Cloudflare Verification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce the TLS fingerprint catalog to ≤3 distinct profiles per `(browser, os, device)` triple (71 total), declare every kept profile via `spec!`, add a Cloudflare-based amiabot verification sweep, and re-verify the kept roster against tls.peet.ws.

**Architecture:** A deterministic selection step in `gen_specs.py` keeps the top-3 observed distinct JA4 clusters per triple (with family sanity floors and two wire-exact hand upgrades). Generated modules regenerate to kept entries only; the two surviving hand profiles are re-declared via `spec!`. The resolver's generated tier changes from strict band containment to next-modern fallback with a cross-triple os-drop fallback. A new amiabot sweep example in `xray-tui-native` drives the engine TLS + hyper HTTP/1.1 & HTTP/2 with per-family headers and h2 settings.

**Tech Stack:** Python (`gen_specs.py`), Rust (`xray-tui-tls`, `xray-tui-native`), hyper 1.11, sha2, serde_json. Reference data: `thirdparty/impit` (headers + h2 settings, data only).

**Spec:** `docs/superpowers/specs/2026-08-25-ja4-roster-reduction-design.md`

## Global Constraints

- Clippy pedantic+nursery **zero warnings**; `cargo fmt` clean; `cargo test --workspace` green at every task boundary.
- `ast_edit` forbidden. Use `read`/`search`/`edit`. `thirdparty/` **read-only** — import impit data as copied constants with attribution, never code.
- Kept roster = 69 generated + 2 hand (`chrome_130`, `edge_106`) = 71; 18 distinct JA4s; **0 PSK** entries (all peet.ws-connectable).
- Family sanity ranges: chrome/edge/opera/brave/firefox `[80,155]`, samsung `[10,40]`, safari `[3,30]`.
- Full 1825 manifest stays as the regeneration source; selection marks `kept`.
- Hand JA4 A-part+hash2 table (from `tests/fingerprints.rs`): chrome_130 `t13d1516h2`/`8daaf6152771`; edge_106 `t13d1516h2`/`8daaf6152771`. Both upgrade their triple's **top-ranked matching cluster slot only** (exclude that generated slot from `kept`).
- peet.ws JA4 match is the hard gate; amiabot is a report-with-flags (soft).
- TLS-1.3 only; keyshare keyspace grease/x25519/mlkem768 (P-curve hybrids compile_error).

---

### Task 1: Selection step in `gen_specs.py`

**Files:**
- Modify: `crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py` (add `--select`, `select_roster()`, selftest)
- Test: selftest inside `gen_specs.py` (`--selftest`)

**Interfaces:**
- Consumes: existing `build_manifest()` output — entries with `browser`, `browser_major`, `os`, `device`, `ja4`, `observation_count`, `low_fidelity`, `name`, `wire.extensions`.
- Produces: `select_roster(entries) -> set[str]` of kept `name`s (the 2 upgraded slots excluded), plus `KEPT_STATS` for the selftest assertion. Later tasks consume the emitted kept `name`s.

- [ ] **Step 1: Write the failing selftest**

Add to `gen_specs.py` a `--select` mode and a `select_roster()` function, plus a selftest asserting the exact kept roster. The kept set (from the design simulation):

```
chrome_148_android_desktop, chrome_134_android_desktop, chrome_141_android_desktop_3,
chrome_147_android_tablet, chrome_131_android_tablet, chrome_83_android_tablet,
chrome_144_ios_phone_2, chrome_143_ios_phone, chrome_133_ios_phone,
chrome_148_ios_tablet, chrome_146_ios_tablet, chrome_141_ios_tablet,
chrome_149_macos_desktop, chrome_122_macos_desktop, chrome_115_macos_desktop,
chrome_143_windows_desktop, chrome_93_windows_desktop,   # chrome_148_windows_desktop EXCLUDED (hand-upgraded)
edge_146_android_desktop, edge_134_android_desktop, edge_121_android_desktop,
edge_144_android_tablet, edge_143_ios_phone_3, edge_143_ios_phone_2, edge_121_ios_phone,
edge_131_ios_tablet, edge_148_macos_desktop, edge_132_macos_desktop, edge_112_macos_desktop,
edge_128_windows_desktop, edge_121_windows_desktop,        # edge_149_windows_desktop EXCLUDED (hand-upgraded)
firefox_150_android_desktop, firefox_149_android_desktop_2, firefox_144_android_desktop_5,
firefox_146_ios_phone_2, firefox_138_ios_phone, firefox_137_ios_phone,
firefox_150_macos_desktop, firefox_149_macos_desktop, firefox_148_macos_desktop,
firefox_148_windows_desktop, firefox_139_windows_desktop, firefox_125_windows_desktop,
opera_96_android_desktop, opera_88_android_desktop, opera_80_android_desktop,
opera_130_macos_desktop, opera_119_macos_desktop, opera_98_macos_desktop,
opera_130_windows_desktop, opera_128_windows_desktop, opera_97_windows_desktop,
brave_90_macos_desktop, brave_89_windows_desktop, brave_126_windows_desktop,
safari_26_ios_phone, safari_18_ios_phone, safari_9_ios_phone,
safari_18_ios_tablet, safari_17_ios_tablet, safari_6_ios_tablet,
safari_26_macos_desktop, safari_16_macos_desktop, safari_12_macos_desktop,
safari_12_windows_desktop, safari_12_windows_desktop_2, safari_5_windows_desktop,
samsung_29_android_desktop, samsung_28_android_desktop, samsung_17_android_desktop
```

That is **69 generated names** (2 hand-upgraded generated slots excluded). Selftest asserts `sorted(select_roster(build_manifest())) == sorted(<the 69 names>)`.

- [ ] **Step 2: Run selftest to verify it fails**

Run: `python3 crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py --selftest`
Expected: FAIL — `select_roster` not defined.

- [ ] **Step 3: Implement `select_roster`**

```python
# Family sanity ranges (spec §1). Keyed by manifest `browser`.
FAMILY_RANGES = {
    'chrome': (80, 155), 'edge': (80, 155), 'opera': (80, 155),
    'brave': (80, 155), 'firefox': (80, 155), 'samsung': (10, 40),
    'safari': (3, 30),
}
# Hand profiles that survive as wire-exact upgrades: assigned concrete
# triple -> (A-part, hash2). Their JA4s are absent from the corpus, so they
# can never win by count; they replace the top-ranked cluster slot whose
# A-part+hash2 they share. (Source: tests/fingerprints.rs captured JA4s.)
HAND_UPGRADES = {
    'chrome_130': (('chrome', 'windows', 'desktop'), 't13d1516h2', '8daaf6152771'),
    'edge_106':   (('edge', 'windows', 'desktop'),  't13d1516h2', '8daaf6152771'),
}

def is_psk(e):
    return any(x.get('ty') == 0x0029 for x in e['wire'].get('extensions', []))

def select_roster(entries):
    """Top-3 observed distinct JA4 clusters per triple, most-modern major
    rep, family range filtered, PSK excluded; the top-ranked slot matching
    a hand profile's A+hash2 is removed from the generated keep (the hand
    profile replaces it), one slot per hand profile. Returns the set of
    kept generated `name`s."""
    from collections import defaultdict
    triples = defaultdict(list)
    for e in entries:
        triples[(e['browser'], e['os'], e['device'])].append(e)
    kept = set()
    used_upgrades: set[str] = set()
    for t, es in triples.items():
        flo, cap = FAMILY_RANGES[t[0]]
        es = [e for e in es if flo <= e['browser_major'] <= cap and not is_psk(e)]
        if not es:
            continue
        clusters = defaultdict(list)
        for e in es:
            clusters[e['ja4']].append(e)
        ranked = sorted(clusters.items(),
                        key=lambda kv: (-sum(x['observation_count'] for x in kv[1]), kv[0]))
        for ja4, cl in ranked[:3]:
            rep = max(cl, key=lambda x: x['browser_major'])
            a, h2 = ja4.split('_')[0], ja4.split('_')[1]
            # wire-exact upgrade: first (top-ranked) cluster in this triple
            # whose A+hash2 matches an unused hand profile is replaced.
            upgraded = False
            for hname, (ht, ha, hh2) in HAND_UPGRADES.items():
                if hname not in used_upgrades and ht == t and (ha, hh2) == (a, h2):
                    used_upgrades.add(hname)
                    upgraded = True
                    break
            if not upgraded:
                kept.add(rep['name'])
    return kept
```

- [ ] **Step 4: Run selftest to verify it passes**

Run: `python3 crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py --selftest`
Expected: PASS — `select_roster` returns exactly the 69 names. Also re-run the existing selftest sections (JA4 hash mirror, manifest stats) — must stay green.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py
git commit -m "feat(tls): deterministic kept-roster selection in gen_specs.py"
```

---

### Task 2: Regenerate modules to kept roster + update offline JA4 gate

**Files:**
- Modify: `crates/xray-tui-tls/src/profiles/generated/*.rs` + `generated/mod.rs` (emitter output, 69 entries)
- Modify: `crates/xray-tui-tls/tests/generated_ja4_gate.rs` (roster-size expectations)
- Modify: `gen_specs.py` emitter (`--emit` uses `select_roster`; writes `kept` flag into the manifest)

**Interfaces:**
- Consumes: `select_roster` from Task 1.
- Produces: `GENERATED` = 69 `GenEntry`s; gate asserts every one passes. Task 5's grader iterates `GENERATED` + hand profiles.

- [ ] **Step 1: Add `kept` flag to emitter + wire `--emit` to selection**

In `gen_specs.py`, after `build_manifest()`, compute `kept = select_roster(entries)`; add `"kept": name in kept` to each manifest entry; the Rust emitter renders only entries with `kept: true` into `generated/*.rs` + `generated/mod.rs`. Verify the two excluded names (`chrome_148_windows_desktop`, `edge_149_windows_desktop`) are absent from emitted output.

- [ ] **Step 2: Regenerate**

Run: `python3 crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py` (full pipeline)
Expected: `generated/*.rs` now hold 69 `GenEntry`s; `GENERATED` registry lists 69; `specs_manifest.json` entries carry `kept`.

- [ ] **Step 3: Update the offline gate**

`tests/generated_ja4_gate.rs` iterates `GENERATED` generically; replace any hardcoded roster-size constant (1825) and any per-classification expected totals with the new kept-roster values. Recompute the classification counts (no-sig / padding-in-hello / padding-omitted / ht) over the 69 entries and assert them exactly. The per-entry assertion logic (full_ja4, corpus-rule hash2, A-part with deltas) is unchanged.

- [ ] **Step 4: Run the gate**

Run: `cargo test -p xray-tui-tls --test generated_ja4_gate`
Expected: PASS, 69/69 verified, 0 failures. Then `cargo test --workspace` green.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(tls): regenerate generated roster to kept 69 entries; update JA4 gate"
```

---

### Task 3: Hand-selected `spec!` conversion + delete dropped hand modules

**Files:**
- Create: `crates/xray-tui-tls/src/profiles/hand_selected.rs`
- Modify: `crates/xray-tui-tls/src/profiles/mod.rs` (module decls, HAND_WRITTEN roster)
- Modify: `crates/xray-tui-tls/src/fingerprints/resolve.rs` (HAND_ROWS → hand_selected, os=Windows, remove 13 rows)
- Delete: `profiles/{chrome,chrome119,chrome133,chrome_android130,edge,brave167,opera114,firefox,firefox120,firefox128esr,safari,safari16,safari_ios17,ios14,android11_okhttp}.rs`

**Interfaces:**
- Consumes: `spec!` macro (proven fully expressive); hand-profile wire bytes currently in `chrome.rs`/`edge106.rs`.
- Produces: `hand_selected::chrome_130()/edge_106()` spec fns referenced by HAND_ROWS; `HAND_ROWS` = 2 rows. Task 4 resolves against them.

- [ ] **Step 1: Write the failing byte-equivalence test**

In `profiles/mod.rs` tests, add (temporarily):

```rust
#[test]
fn hand_selected_matches_original_wire_bytes() {
    assert_eq!(super::hand_selected::chrome_130(), super::chrome::spec());
    assert_eq!(super::hand_selected::edge_106(), super::edge106::spec());
}
```

Run: `cargo test -p xray-tui-tls profiles::tests::hand_selected_matches_original_wire_bytes`
Expected: FAIL — `hand_selected` module missing.

- [ ] **Step 2: Declare the two profiles via `spec!`**

Create `hand_selected.rs`. Transcribe `chrome.rs`'s `spec()` and `edge106.rs`'s `spec()` into `spec!` declarations named `chrome_130` and `edge_106` (same `ciphers`, `session`, `exts` in the same order — the macro is a 1:1 wire encoding; `raw[ty,"hex"]` covers extended_master_secret, delegated-credentials, ALPS bodies; `rslimit[N]` covers record_size_limit). Re-run the equivalence test — PASS.

- [ ] **Step 3: Rewire `HAND_ROWS`**

In `resolve.rs`, replace the 15 `HAND_ROWS` rows with 2 rows (os pinned Windows, both desktop):

```rust
static HAND_ROWS: &[Row] = &[
    Row { name: "chrome_130", browser: Browser::Chrome, os: Some(Os::Windows),
          device: Device::Desktop, max_version: 130, min_version: 0,
          spec: hand_selected::chrome_130 },
    Row { name: "edge_106", browser: Browser::Edge, os: Some(Os::Windows),
          device: Device::Desktop, max_version: 106, min_version: 0,
          spec: hand_selected::edge_106 },
];
```

Update the `use` imports; delete the 15 hand profile module files and their `pub mod` decls in `profiles/mod.rs`; drop the deleted entries from `HAND_WRITTEN` (keep `chrome_130`, `edge_106`).

- [ ] **Step 4: Run tests**

Run: `cargo test -p xray-tui-tls`
Expected: PASS (equivalence test passes, then its comparison targets are gone; if it can't compile after deletion, remove the temporary test in the same commit — its job is done). Update `tests/fingerprints.rs` if any case references a dropped profile (keep the 2 survivors' cases). `cargo test --workspace` green.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(tls): spec!-declare surviving hand profiles; delete dropped hand tier"
```

---

### Task 4: Resolver next-modern + cross-triple os-drop fallback

**Files:**
- Modify: `crates/xray-tui-tls/src/fingerprints/resolve.rs` (`choose_generated`, `resolve`, `matches`, docs)

**Interfaces:**
- Consumes: `HAND_ROWS` (2), `GENERATED` (69).
- Produces: new resolution contract consumed by the grader/amiabot roster iteration and the UI pickers (`available_identities()` = 71).

- [ ] **Step 1: Write the failing resolver tests**

Add rstest cases in `resolve.rs`'s test module:

```rust
/// Next-modern fallback: a version between kept majors resolves to the
/// smallest kept major >= v.
#[rstest::rstest]
#[case::chrome_windows_115_goes_143(fp(Browser::Chrome, 115, Some(Os::Windows), Device::Desktop), "chrome_143_windows_desktop")]
#[case::chrome_windows_93_exact(fp(Browser::Chrome, 93, Some(Os::Windows), Device::Desktop), "chrome_93_windows_desktop")]
/// Hand band (min 0) absorbs old versions within the triple.
#[case::chrome_windows_50_hand_130(fp(Browser::Chrome, 50, Some(Os::Windows), Device::Desktop), "chrome_130")]
/// Above the newest kept major refuses.
#[case::chrome_windows_200_refuses(fp(Browser::Chrome, 200, Some(Os::Windows), Device::Desktop), Err)]
/// Cross-triple os-drop: chrome/linux/desktop falls back to any-os chrome/desktop rows.
#[case::chrome_linux_115_cross_triple(fp(Browser::Chrome, 115, Some(Os::Linux), Device::Desktop), "chrome_143_windows_desktop")]
/// os=None query still resolves (any-os rows), next-modern.
#[case::chrome_osless_115(fp(Browser::Chrome, 115, None, Device::Desktop), "chrome_143_windows_desktop")]
fn resolves_under_reduced_roster(#[case] fp: Fingerprint, #[case] expected: ...) { ... }
```

Run: `cargo test -p xray-tui-tls fingerprints::resolve` — expected FAIL (current strict containment misses).

- [ ] **Step 2: Implement next-modern in `choose_generated`**

Replace the strict `min_version <= v <= max_version` filter with: among os/device-compatible rows of the group, keep rows with `v <= max_version`, pick the smallest `max_version`; refuse if none (v above newest) or if `v <` the group's smallest `max_version` (below oldest). Preserve the first-declared tie-break and the version-unset (`None` → greatest) path.

- [ ] **Step 3: Implement cross-triple os-drop in `resolve()`**

After the primary match, if no row matched the exact triple, retry with `self.os = None` (same browser+device, any-os rows) and apply next-modern across the union. Keep hand-precedence (HAND_ROWS first) in both passes. Update the `resolve()` doc comment to state the new contract.

- [ ] **Step 4: Run tests**

Run: `cargo test -p xray-tui-tls` — all resolver cases green (new + existing); fix any existing case that encoded strict-containment expectations. `cargo test --workspace` green.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(tls): next-modern + cross-triple os-drop resolver fallback"
```

---

### Task 5: Grader roster mode over kept 71 + peet.ws sweep + report

**Files:**
- Modify: `crates/xray-tui-tls/examples/grader.rs` (roster iteration = `GENERATED` + the 2 hand profiles)
- Modify: `crates/xray-tui-tls/tests/tls_peet_ws.rs` (sample covers hand profiles)
- Create: `docs/tls-fingerprint-roster.md` (rewrite for reduced roster)

**Interfaces:**
- Consumes: `GENERATED` (69), `hand_selected::chrome_130/edge_106` + their captured JA4s (`t13d1516h2_8daaf6152771_f37e75b10bcc`).

- [ ] **Step 1: Extend grader roster to include hand profiles**

`roster::main` currently iterates `GENERATED`. Build a combined roster `Vec<(name, expected_ja4, spec_fn)>` = 69 `GenEntry`s + `("chrome_130", CHROME130_JA4, hand_selected::chrome_130)` + `("edge_106", EDGE106_JA4, hand_selected::edge_106)` where `EDGE106_JA4 = "t13d1516h2_8daaf6152771_f37e75b10bcc"`. Update `--roster`, `--sample`, `--family` to walk this list.

- [ ] **Step 2: Run the live sweep**

Run: `cargo run -p xray-tui-tls --example grader -- --roster` (network)
Expected: all 71 entries connect and return their expected JA4 (hand profiles return their captured JA4s). Record per-family pass counts and any transient failures (retry once).

- [ ] **Step 3: Update the sampled live test + docs**

`tests/tls_peet_ws.rs`: add one case per kept family/band (incl. a hand profile). Rewrite `docs/tls-fingerprint-roster.md`: roster 71, methodology, sweep results, the 2 hand-upgraded slots' expected JA4s, the 50 low-fidelity note.

- [ ] **Step 4: Run tests**

Run: `cargo test --workspace`; `cargo clippy --workspace --all-targets`; `cargo fmt --check`. All clean.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "docs(tls): verify reduced 71-profile roster against tls.peet.ws"
```

---

### Task 6: amiabot checker (headers + h2 settings + sweep example + report)

**Files:**
- Create: `crates/xray-tui-native/src/headers.rs` (per-family header templates + per-major overrides; h2 settings)
- Create: `crates/xray-tui-native/examples/amiabot_sweep.rs`
- Create: `docs/amiabot-roster-report.md`
- Modify: `crates/xray-tui-native/src/lib.rs` (module decl) + `Cargo.toml` if `serde_json` missing

**Interfaces:**
- Consumes: `xray_tui_tls::client::connect` (`TlsConfig`); `generated::GENERATED` + `hand_selected` roster (69+2); hyper 1.11 `http1`/`http2` builder; impit header/h2 data (copied constants).
- Produces: `headers::HeadersFor { user_agent, accept, accept_language, sec_ch_ua, sec_fetch_* }` and `headers::h2_settings(browser) -> (u32,u32,u32)`.

- [ ] **Step 1: Write `headers.rs` (data + unit tests)**

```rust
//! Per-family HTTP request header emulation + h2 settings for the
//! amiabot verification sweep. Header vectors and h2 window sizes are
//! transcribed from thirdparty/impit (fingerprint/database/{chrome,firefox}.rs
//! + http_headers) — data only, Apache-2.0; impit's TLS is patched-rustls
//! and unused here.
```

`user_agent(browser, os, device, major)` synthesizes e.g. `Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36`; `sec_ch_ua(browser, major)` builds `"Chromium";v="140", "Not_A Brand";v="24", "Chrome";v="140"`-style brand strings; family defaults for accept/accept-language/sec-fetch-\*; per-major overrides where impit has them (chrome_100..151, firefox 128/133/135/144). `h2_settings` returns `(stream_window, conn_window, max_header_list)`: chrome `(6_291_456, 15_663_105, 262_144)`, firefox `(131_072, 12_517_377, 262_144)`, else hyper defaults.

Unit tests: UA for chrome/140/windows/desktop, sec-ch-ua major interpolation, h2 settings per family. Run `cargo test -p xray-tui-native headers` — green.

- [ ] **Step 2: Write the sweep example**

`examples/amiabot_sweep.rs`: for each of the 71 profiles (69 `GENERATED` + 2 hand):
1. ALPN from profile spec (`h2` → HTTP/2, else HTTP/1.1); `TlsConfig { mode: Plain{ fingerprint: None, verifier: WebPkiVerifier }, server_name: "amiabot.app", alpn: Some([proto]), curves: None, rng: SystemRandom }`.
2. `connect(tcp_stream, &config)` → `TlsStream`; `TokioIo::new(stream)`; `http2::Builder::new(TokioExecutor::new()).initial_stream_window_size(..).initial_connection_window_size(..).max_header_list_size(..).serve_connection` or `http1::Builder::new().serve_connection` (mirror `transport/http/conn.rs` h1_client/h2_client).
3. Send `GET /api/check` with `headers::` vector; read body; `serde_json` parse `verdict.{score,classification,reasons[].id}`, `cloudflareBotManagement.score`, `server.{headers,httpProtocol}`.
4. Concurrency ~4, per-request 15s timeout, one retry. Print a row per profile.

- [ ] **Step 3: Run a smoke check**

Run: `cargo run -p xray-tui-native --example amiabot_sweep -- --sample chrome_130`
Expected: one row; verdict JSON parsed; header echo shows our User-Agent. (Live; the full 71-row sweep is an on-demand run.)

- [ ] **Step 4: Write the report + flags**

Run the full sweep; write `docs/amiabot-roster-report.md` (date, methodology, IP-pollution caveat — our datacenter/VPN IP adds ~48 absolute points; compare relative + CF score). Per-profile rows + flags: `library_user_agent` reason, `cloudflareBotManagement.score >= 99`, handshake failure, header-echo mismatch.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(tls): amiabot Cloudflare verification sweep with per-family headers"
```

---

### Task 7: Docs + final whole-branch review

**Files:**
- Modify: `NATIVE_CORE.md` (tier model: 2 wire-exact + 69 JA4-faithful; roster 1825→71; `--select` usage; fidelity contract)

- [ ] **Step 1: Update NATIVE_CORE.md**

Tier model section: hand tier = 2 wire-exact profiles (chrome_130, edge_106, `spec!`-declared in `hand_selected.rs`); generated tier = 69 JA4-faithful entries (kept subset of the 1825 manifest); resolver next-modern + cross-triple fallback; amiabot verification reference.

- [ ] **Step 2: Full verification**

Run: `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `cargo fmt --check`, `python3 crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py --selftest`. All green/clean.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "docs(tls): NATIVE_CORE.md reduced-roster tier model"
```

- [ ] **Step 4: Whole-branch review**

Dispatch a reviewer over the full `native-core-stub` branch diff for this feature. Triage any findings (fix-before-merge vs defer), ledger in the SDD progress file, fix Critical/Important, re-review clean.

---

## Self-Review

**Spec coverage:**
- §1 selection (floors, cluster ranking, upgrade, kept flag) → Task 1–2 ✓
- §2 hand conversion (spec!, equivalence, deletions) → Task 3 ✓
- §3 resolver (next-modern, below/above refuse, cross-triple os-drop, os=Windows rows, available_identities) → Task 4 ✓
- §4 amiabot (headers, h2 settings, sweep, report, flags, IP caveat) → Task 6 ✓
- §5 peet.ws re-verify + gate + docs → Tasks 2, 5, 7 ✓
- 0 PSK kept, 71 total, floors table → Task 1 selftest pins exact 69-name set ✓

**Placeholders:** none — every code step carries real code or exact references.

**Type consistency:** `select_roster -> set[str]` (Task 1) consumed by emitter (Task 2); `hand_selected::chrome_130/edge_106` (Task 3) consumed by resolver (Task 4), grader (Task 5), amiabot (Task 6); `headers::h2_settings -> (u32,u32,u32)` and `HeadersFor` (Task 6) used only within Task 6.
