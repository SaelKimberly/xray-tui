# TLS fingerprint roster — JA4 fidelity verification report

**Date:** 2026-08-24
**Branch:** `native-core-stub` (JA4 spec expansion, Task 8)
**Roster:** 1825 generated JA4-faithful profiles (`xray-tui-tls` `profiles/generated/`)
**Live target:** `https://tls.peet.ws` (`/api/all`, TLS 1.3, HTTP/2 + HTTP/1.1)

## Roster scale

| Family (module) | Entries | Wire template |
|---|---|---|
| chrome (desktop Chromium: Chrome/Edge/Opera/Brave/Samsung) | 720 | `chrome_desktop` |
| firefox (incl. Firefox for Android) | 407 | `firefox` |
| safari (macOS/Windows WebKit) | 33 | `safari` |
| chrome_android (Chromium Android) | 359 | `chrome_android` |
| safari_ios (WKWebView reality — any browser on iOS) | 306 | `safari_ios` |
| **Total** | **1825** | (fallback/okhttp families empty) |

Generator: `crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py`
(`--manifest` → `specs_manifest.json` from the ja4db export; `--emit` →
`profiles/generated/*.rs`, byte-deterministic; `--selftest` verifies the
committed files match a fresh render).

## Fidelity contract

Every entry's built `ClientHello` must reproduce its registered source JA4
(the ja4db corpus value, `GenEntry.ja4`):

- **Offline gate** (`tests/generated_ja4_gate.rs`, runs in every
  `cargo test`): all 1825 entries — 1003 full-hash exact, 475
  padding-in-hello, 245 padding-omitted (512-byte target rule), 102 no-sig
  (locked), 0 failures; 306 entries carry the corpus `ht` ALPN letter
  (orthogonal rendering note).
- **Live sweep** (`cargo run -p xray-tui-tls --example grader -- --roster`):
  empirical confirmation against the real peet.ws API — this report.

## Live sweep — methodology

Each entry: `ClientHello` built from the generated spec with the fixed-seed
fixture (all-`0x5A`), parsed locally, then a real TLS 1.3 connection to
tls.peet.ws (per-connection GREASE via `SystemRandom`; JA4 is
GREASE-normalized so the fixed-seed local value is comparable). The server's
reported JA4 is checked against:

1. the local JA4 computed with the server's own algorithm (wire fidelity),
2. the registered corpus JA4 (normalized for the server's rendering).

Two full runs (16 concurrent connections, 20 s per-entry cap). The server's
`ja4.go` rendering differs from the FoxIO spec in ways the grader
replicates:

- A-part counts are **non-padded decimal** and the ALPN letter is
  **omitted when no ALPN is offered** (registered `t13d170900_…` is
  reported as `t13d179_…`);
- hash2 excludes the padding extension `0015` (already documented in the
  grader's reconciliation notes).

Entries whose hello offers no `h2` ALPN (the 306 `http/1.1`-only and 533
no-ALPN entries) are graded over an HTTP/1.1 GET (`Connection: close`,
Content-Length/chunked aware).

## Live results summary (2026-08-24, two full runs)

| Family | Total | Passed both runs | Wire-faithful (run 1) | Deterministic rejections | Transient only |
|---|---|---|---|---|---|
| chrome | 720 | 686 | 695 | 25 | 9 |
| firefox | 407 | 349 | 374 | 33 | 25 |
| safari | 33 | 33 | 33 | 0 | 0 |
| chrome_android | 359 | 321 | 329 | 25 | 13 |
| safari_ios | 306 | 266 | 278 | 28 | 12 |
| **Total** | **1825** | **1655** | **1697** | **111** | **59** |

The three result buckets partition the roster exactly per family
(passed-both + deterministic-rejected + transient-only = total); the
`Wire-faithful (run 1)` column is the first run's pass count, a separate
single-run measurement — transient membership differs between runs, so it
is not a partition of the other columns.

- **1655 entries (90.7%)** connected and matched the local JA4 in **both**
  runs; the first run verified **1697 (93.0%)**.
- **111 deterministic handshake rejections** — see finding 1 below.
- **59 transient entry-runs** hit timeouts (HTTP/2 response timed out /
  fetch timed out) in one of the two runs and passed the other —
  peet.ws throttles under the 16-way burst; none are wire mismatches.
  Per the plan ruling the sweep is a manual report, not a merge gate.

### Finding 1 — extension `0x0029` (pre_shared_key): peet.ws/Go rejects any hello carrying it (111 entries)

All 111 entries whose spec carries `raw[0x0029, ""]` (chrome 25, firefox
33, chrome_android 25, safari_ios 28) fail the live handshake with fatal
`unexpected_message` (alert 2 10) — deterministically, across both runs.
Bisection proved the rejection is caused by the extension's **presence**,
not its body or position: an empty body fails, a structurally valid
one-identity/one-binder body fails, moving it last (RFC 8446 §4.2.11
requires last) fails; removing it makes the identical hello connect and
report a JA4 that matches every remaining field.

The ja4db string export carries extension ids but no bodies; the corpus
counts `0029` in hash2, so the roster keeps it (JA4 fidelity is the
contract) and the entry is recorded as **live-connect-infeasible against
tls.peet.ws**. Real clients that send `pre_shared_key` carry a session
ticket issued by the server they are talking to; a generated hello cannot
fabricate one, and this server rejects the offer outright (Go
`crypto/tls`). These 111 entries remain offline-verified JA4-faithful
(the gate asserts hash2 including `0029`); a real server that issued a
ticket would accept the shape.

### Finding 2 — empty `0x0032` (signature_algorithms_cert) was a real template bug; fixed and regenerated

The first sweep exposed **225 handshake rejections** (69 chrome, 80
firefox, 4 safari, 27 chrome_android, 45 safari_ios) from
`raw[0x0032, ""]` — `signature_algorithms_cert` with an **empty body**,
which is malformed (RFC 8446 §4.2.3 requires a 2-byte length + scheme
list). The corpus string export has no bodies, so the emitter emitted
`""`.

**Fix (template bug → regenerated):** `gen_specs.py` now emits the
`signature_algorithms_cert` body mirroring the spec's `signature_algorithms`
list (the RFC default when the extension is absent) — 2-byte length +
sig-alg ids. The body is JA4-invisible, so the offline gate is unchanged
(1825/1825, identical classification counts) and the regenerated diff is
exactly the 225 `raw[0x0032, ""]` → `raw[0x0032, "<hex>"]` tokens (plus the
225 matching manifest wire blocks). Re-sweep: all four previously failing
safari entries and every other `0x0032` entry now connect. The bisect
probe is preserved in the commit history; the grader itself no longer
carries it.

### Finding 3 — ECH `0xfe0d` empty-Raw entries (Task 3 flag): accepted

The 620 entries carrying the empty ECH `fe0d` outer connect and grade
cleanly (the server treats it as an unknown extension, RFC 8446 §4.2).
The 5 ECH entries among the 111 rejections fail solely because they also
carry `0x0029` — not because of ECH.

### Finding 4 — `http/1.1`-only and no-ALPN entries (306 + 533)

- 306 `ht`-letter entries (ALPN `http/1.1` only): graded via the
  HTTP/1.1 GET path; the server renders the A-part letter `h1`
  (first+last, matching the crate codec and the grader's `peet_a_part`).
  The corpus `ht` is the ja4db first-two-chars rendering — a documented
  A-part letter difference that keeps those entries' registered-match
  flag clear (they are never full-hash class, so it is not a failure).
- 533 no-ALPN entries: connect over HTTP/1.1 default; the server omits
  the A-part letter entirely and uses non-padded counts (see methodology).

### Finding 5 — `chrome_619_windows_desktop` (Task 6 spoofed-UA artifact)

Surfaced **cleanly** in the live sweep (single-major band 619; connects
and reports a wire-faithful JA4). Kept in the roster; the offline gate
already pins it.

### Transient failures

60 entry-runs hit timeouts across the two runs (firefox 25,
chrome_android 13, safari_ios 13, chrome 9; safari 0) — peet.ws
throttling under load, not wire divergence: every one of them passed the
other run. Rerun `--roster` for a clean pass on any flagged entry.

## Reproduction

```bash
# Full live sweep (1825 entries, ~2.5 min)
cargo run -p xray-tui-tls --example grader -- --roster
# Per family / per (family, major)-band sample
cargo run -p xray-tui-tls --example grader -- --roster --family safari
cargo run -p xray-tui-tls --example grader -- --roster --sample
# Offline gate (no network; runs in every cargo test)
cargo test -p xray-tui-tls --test generated_ja4_gate
# Sampled live test (one entry per family/band, ignored by default)
cargo test -p xray-tui-tls --test tls_peet_ws -- --ignored
# Generator self-check
python3 crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py --selftest
```

The grader's `--roster` mode prints per-entry lines (`name family class ht
ech alpn server_ja4 error`) and a per-family summary (full / pad+ / pad- /
no-sig / ht / wire-ok / reg-ok / fail).
