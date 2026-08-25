# JA4 Roster Reduction + Cloudflare Verification

Date: 2026-08-25. Branch: `native-core-stub`. Supersedes the roster scale of
`2026-08-24-ja4-spec-expansion-design.md` (1825 generated entries).

## Purpose

Ship a **small, DPI-aware fingerprint catalog**: keep only the most-modern and
most-observed profiles per `(browser, os, device)` triple, declare every kept
profile with the `spec!` macro, and verify the kept roster against a
Cloudflare-based detector (amiabot.app) in addition to the existing tls.peet.ws
JA4 gate.

## Goals

1. Reduce the full roster (1825 generated + 17 hand-written) to **≤3 distinct
   fingerprints per `(browser, os, device)` triple**.
2. Every kept profile declared via `spec!` (codebase shrink; generated modules
   regenerate to kept entries only).
3. Verify every kept profile against tls.peet.ws (must return its expected
   JA4) — the **hard gate**, unchanged in spirit.
4. Add an **amiabot.app** sweep (proper HTTP/1.1 or HTTP/2 request with browser
   headers over the engine TLS stream) producing a committed report with
   relative flags. Not a merge gate.

## Non-goals

- No byte-level wire synthesis beyond the existing `ClientHelloSpec` shapes
  (JA4-faithful tier keeps its documented byte-level divergence allowance).
- No h2 pseudo-header reordering (impit uses a patched h2 crate; out of scope).
  We set hyper's h2 window sizes only.
- No Cloudflare JS-challenge solving; we record the verdict, never bypass.
- No preservation of dropped profiles beyond git history and the full manifest.

## Data reality (constraints the design responds to)

Measured from `crates/xray-tui-tls/src/fingerprints/catalog/specs_manifest.json`
(1825 entries, stats: rows_in 148992, kept 1825, collisions_dropped 4719).

- The corpus collapses to **106 distinct JA4s** across 1825 entries. Counts are
  dominated by ~20 clusters shared across browsers/OSes; the #1 cluster
  (`t13d1516h2_8daaf6152771_d8a2da3f94cd`) alone backs 43+ kept entries.
- Version labels on dominant clusters are noise: Chrome 9/linux (462 obs),
  **Chrome 619** (spoofed-UA artifact, 35,850 obs), Firefox 2/3, Brave 1.
- **All 17 hand-written JA4s are absent from the corpus** (uTLS-derived; the
  corpus is observed scanner/proxy traffic). Under pure count-wins they score 0.
- Hand chrome130's JA4 (`t13d1516h2_8daaf6152771_f37e75b10bcc`) shares
  A-part + hash2 with the #1 cluster — differs only in sigalgs order (hash3).
- Pure count-wins (top-3 majors by summed obs) yields 92 entries but only
  18 distinct JA4s (43 wire-identical), 74 low-fidelity, and ancient majors.
- 111 entries carry `raw[0x0029]` pre_shared_key — live-infeasible on
  tls.peet.ws (Go server rejects). Excluded from the kept roster by construction.

## Selection algorithm (`gen_specs.py --select`)

Deterministic; byte-deterministic regeneration asserted by `--selftest`.

1. For each `(browser, os, device)` triple, drop entries whose `browser_major`
   is outside the family sanity range **or** that carry PSK (`raw[0x0029]`).
2. Group survivors by full JA4 (clusters). Rank clusters by summed
   `observation_count` (desc), tie-break by JA4 string.
3. Keep the **top-3 clusters**.
4. Per kept cluster, representative = the entry with the **greatest
   `browser_major`** (most-modern).
5. **Wire-exact upgrade**: a hand-written profile whose JA4 shares
   A-part + hash2 with a kept cluster **and whose triple equals the cluster's
   triple** replaces the **top-ranked matching slot** (one hand profile fills
   one slot). Exactly two qualify:
   - `chrome_130` → chrome/windows/desktop (replaces `chrome_148_windows_desktop`)
   - `edge_106` → edge/windows/desktop (replaces `edge_149_windows_desktop`)
   - A second matching cluster slot keeps its generated entry (e.g.
     `chrome_143_windows_desktop`) — a distinct JA4 variant, never dropped.
6. Mark `kept: true` on kept entries in the manifest (full 1825 data remains
   the regeneration source). The emitter renders **kept entries only** into
   `profiles/generated/*.rs` + the `GENERATED` registry.

### Family sanity ranges

| family | floor | cap |
|---|---|---|
| chrome | 80 | 155 |
| edge | 80 | 155 |
| opera | 80 | 155 |
| brave | 80 | 155 |
| firefox | 80 | 155 |
| samsung | 10 | 40 |
| safari | 3 | 30 |

Kills Chrome 619, Chrome 9/linux, Firefox 2/3, Brave 1, Safari 5-era junk.

### Kept roster (result of simulation)

**71 profiles**: 69 generated + 2 wire-exact hand (`chrome_130`, `edge_106`).
18 distinct JA4s, 0 PSK, 50 low-fidelity. Triples with survivors: 26.
Triples with no survivors after floors: chrome/linux, firefox/linux,
firefox/android-tablet, opera/android-tablet, brave/ios(phone/tablet),
opera/linux. These are covered by the cross-triple fallback (§3), not
dropped from resolution.

## Hand-tier conversion

- `chrome130` and `edge106` re-declared via `spec!` in a consolidated module
  (`profiles/hand_selected.rs` or equivalent). The macro is proven fully
  expressive (Task 4 equivalence test; `raw[ty,"hex"]` catch-all covers
  extended_master_secret, delegated-credentials, ALPS bodies).
- **Byte-equivalence assert before deletion**: in the conversion commit, a
  temporary test compares macro-rebuilt `chrome_130`/`edge_106` specs to the
  hand-written `ClientHelloSpec`s field-for-field; run green, then originals
  deleted.
- **Delete**: chrome119, chrome133, brave167, opera114, firefox120,
  firefox128esr, safari, safari16, safari_ios17, ios14, chrome_android130,
  android11_okhttp, edge130, firefox(121, test-only) modules + their
  `HAND_ROWS` rows + `HAND_WRITTEN`/`all_specs` roster tests.
- `tests/fingerprints.rs` JA4 ground-truth cases retained for the 2 survivors
  (`chrome_130` = `t13d1516h2_8daaf6152771_f37e75b10bcc`,
  `edge_106` = same A-part+hash2, hash3 `f37e75b10bcc`).

## Resolver contract

- **Next-modern fallback** (replaces strict band containment in
  `choose_generated`): query `v` → smallest kept major ≥ `v` within the
  matching identity group. Below the group's oldest kept major → refuse (era
  mismatch). Above the group's newest kept major → refuse.
- **Cross-triple fallback**: os is the least fingerprint-relevant axis (desktop
  Chrome sends the same hello on Windows/macOS/Linux). If a query's exact
  triple has no rows, retry with os dropped (same browser+device, any-os rows,
  next-modern across the union). Restores coverage for chrome/linux,
  firefox/linux, firefox+opera/android-tablet, brave/ios, opera/linux via their
  family's windows/macos/android-desktop rows.
- Upgraded hand rows become `os: Some(Windows)` concrete (must not serve
  macos/linux triples directly). Hand precedence over generated (HAND_ROWS
  first) unchanged.
- `available_identities()` shrinks to 71 → UI pickers list only kept.
- `resolve()`/`choose_generated` docs updated; capped error listing unchanged;
  tests updated (next-modern, below/above refuse, cross-triple, os=None query).

## amiabot checker (§4)

**Location**: `examples/amiabot_sweep.rs` in `xray-tui-native` (already has
hyper + engine-TLS deps).

- Per profile: `xray_tui_tls::client::connect` with
  `TlsConfig { mode: Plain, server_name: "amiabot.app", alpn: <profile ALPN>, rng }`
  → hyper client (http1 or http2 by negotiated ALPN) over the `TlsStream` →
  GET `https://amiabot.app/api/check`.
- ALPN per profile from the kept spec (h2 → HTTP/2; http/1.1 → HTTP/1.1;
  no-ALPN → h1 default).
- **Headers** (`headers.rs` in native crate): per-family templates + per-major
  overrides, data imported from
  `thirdparty/impit/impit/src/fingerprint/database/{chrome,firefox}.rs` +
  `http_headers` (constants copied with attribution; impit's TLS is
  patched-rustls, unusable). UA synthesized from (family, os, device, major);
  sec-ch-ua with brand versions from major. Coverage gaps (safari/edge/opera/
  brave/samsung/ios) → family-level defaults.
- **HTTP/2 settings**: hyper h2 builder
  `http2_initial_stream_window_size` / `http2_initial_connection_window_size` /
  `http2_max_header_list_size` per family (chrome 6,291,456 / 15,663,105;
  firefox 131,072 / 12,517,377).
- **Sweep**: 71 profiles, concurrency ~4, per-request timeout + 1 retry; parse
  `verdict.score`/`classification`/`reasons`, `cloudflareBotManagement.score`,
  `server.headers` echo (verify headers landed), `tlsClientCiphersSha1`/
  `tlsClientExtensionsSha1`.
- **Report**: committed `docs/amiabot-roster-report.md` — per-profile rows +
  flags (library-UA reason, CF score ≥ 99, handshake failure, header-echo
  mismatch). **IP-pollution caveat**: our IP is datacenter/VPN → ~48 absolute
  points regardless of TLS/header quality; compare relative, trust CF score.
  Not a merge gate.
- Offline unit tests for `headers.rs` (UA synthesis, sec-ch-ua per major); a
  sampled ignored network test mirrors `tls_peet_ws.rs`.

## peet.ws re-verification + gates + docs (§5)

- grader `--roster` over kept 71; every entry returns its expected JA4.
  Hand-upgraded slots expect the hand profile's captured JA4
  (`chrome_130`/`edge_106` = `t13d1516h2_8daaf6152771_f37e75b10bcc`).
- Offline gate `tests/generated_ja4_gate.rs` regenerates over kept 71: assert
  `full_ja4()` == expected for every kept entry; survivors via
  `fingerprints.rs` ground-truth cases.
- `gen_specs.py --selftest` asserts kept-set regeneration byte-deterministic.
- Sampled live test keeps the PSK filter (harmless; roster is PSK-free).
- Docs: NATIVE_CORE.md tier model (2 wire-exact + 69 JA4-faithful), roster
  1825→71, `--select` usage, fidelity contract. `docs/tls-fingerprint-roster.md`
  updated with the reduced-roster sweep. New `docs/amiabot-roster-report.md`.

## Verification strategy

- `cargo test --workspace` green; clippy pedantic+nursery zero warnings;
  `cargo fmt` clean at every task boundary.
- `gen_specs.py --selftest`: regenerate == committed (kept set).
- Offline JA4 gate over kept 71 (hard).
- Live peet.ws sweep over kept 71 (recorded report; hard gate on expected JA4).
- amiabot sweep (recorded report + flags; soft).

## Risks / notes

- IP pollution makes amiabot absolute scores unreliable; mitigated by relative
  comparison + CF score + header-echo verification.
- Dropped triples rely on cross-triple fallback (os-drop); a linux-desktop
  chrome query serves a windows/macos chrome fingerprint — DPI-safe (same
  browser hello), different OS label. Documented in the report.
- 50 kept entries are low_fidelity (JA4-faithful tier); their byte-level shape
  may diverge from a real browser. The amiabot report flags any that a
  detector treats as bot-like.
- The 2 wire-exact survivors differ from their cluster only in sigalgs order;
  their expected JA4 on peet.ws is the hand-captured value, not the cluster's.
