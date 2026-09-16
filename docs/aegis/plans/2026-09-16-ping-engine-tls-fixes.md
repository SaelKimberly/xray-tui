# 2026-09-16 — Ping-run log analysis: engine TLS fixes and batch observability

Trigger: the user ran a Fast + Real ping batch over the feed on `7e20498` and
asked for a log analysis, a classification of the issues, and an improvement
plan. The analysis (evidence below) found **two engine defects** that produced
23% of the phase-2 failures, plus a set of policy and observability gaps.

## 1. Evidence base

LMDB `~/.config/xray-tui/logs.lmdb` (159,282 entries), cross-checked against
`data.db` (`profile_stats`), plus live re-probing of every recorded failing
host with `crates/xray-tui-tls/examples/probe_host`.

Run `2026-09-16 11:56:18 → 12:12:12` (interrupted by the user's quit):

| phase | volume | outcome |
| --- | --- | --- |
| import | 2 groups, cumulative 4,552 → 26,547 links | 3× `Source ended early` → whole-list re-upsert per retry |
| phase 1 (fast) | 31,043 results over **7,516 unique (host,port)** (4.1 links/probe dedup), 242 s | 16,109 ok · 8,924 timeout · 4,119 dns · 1,514 refused · 364 no-route · 13 unreachable |
| phase 2 (real) | **1,001 of ~16,095 candidates**, 390 s (2.57 results/s) | 1 ok, 1,000 failures |

Phase-2 classes (1,001 results): timeout 338, no-certificate 201,
record-too-large 134, alert-2-40 120, unparseable-fp 35, reality-real-cert 29,
MAC-mismatch 28, early-eof 18, alert-2-50 17, v2rayhttp 17, name-mismatch 16,
vless-header-EOF 15, rest ≤4 each.

Persistence held (all 31,043 rows carry an outcome; 996 keep the fast delay
beside a real failure; no `link_writer` warnings), and the phase-2 skip of the
14,934 hard fast-failures saved ~97 min of phase-2 time.

## 2. Root causes (both verified by implementing the fix, then reverted)

**A1 — RFC 8879 `CompressedCertificate` (type 25) was ignored.**
`read_server_hs_messages` had no arm for it, so the transcript lost the
Certificate message: `server presented no certificate` under verification,
`server Finished MAC mismatch` without it (same host, two texts = the insecure
flag). 229 of 1,001 phase-2 results. Wire form is
`algorithm(2) || uncompressed_length(3) || compressed_certificate_message<1..2^24-1>`
— the vector bound is a **third 3-byte length** (the first patch missed it and
the payload then only decompressed from offset 3), the payload is the
Certificate **body** (rustls parses it the same way), and the **raw type-25
message** belongs in the transcript (a rebuilt `Certificate` fails the server's
Finished MAC).

**A2 — the ALPS (`application_settings`, 0x4469) body was malformed.**
`write_alps_entries` wrote u16 per-entry lengths; draft-vvv-tls-alps encodes
`ProtocolName` (RFC 7301) with a **u8** length. Google and Yandex answered
`alert 2 50 (decode_error)` and closed on every ALPS-bearing profile
(`chrome_130`, `edge_106`, the generated roster). Fix: `4469 0005 0003 026832`.

Verified live with `probe_host --verify` after the fixes: `www.google.com`,
`yandex.ru`, `cloudflare.com`, `discord.com`, `stackoverflow.com` and the three
hosts the run failed on (`dl-am1.steamdownload.top`, `139.59.183.33:2053`,
`162.159.153.8`) all complete **with certificate verification on**; re-probing
the full recorded failing set (78 unique targets across the `no certificate`,
`MAC mismatch` and `alert 2 50` classes) gives **55 OK, 20 still failing
(server-side: `alert 2 40` SNI refusals, name mismatches, chain/leaf errors),
3 dead**.

Not defects (recorded so they are not re-litigated):

- `record too large: 20527 bytes` = 0x502F = `'P' '/'`, i.e. a **cleartext HTTP
  answer** read as a record header. 8 rows are genuinely wrong-port links
  (80/8080/2052), the rest were transient (three sampled hosts handshook
  seconds later). `alert 2 40` (120 rows) reproduces with openssl too — the
  front refuses that SNI (dead/stale config).
- The GREASE design (one value for the slots, a second for the standalone
  extension) mirrors BoringSSL's `ssl_grease_extension1/2`; forcing a single
  value changed nothing, so it was left alone.

## 3. Executed slices

| # | Change | Files |
| --- | --- | --- |
| T1 | ALPS `ProtocolName` u8 lengths + re-pinned goldens (derived, not copied from the new output) | `xray-tui-tls/src/spec/mod.rs`, `hello/mod.rs` |
| T2 | RFC 8879 type-25 arm (`decompress_certificate_message`), raw-message transcript, per-entry decompression path deleted (it modelled no RFC wire form) | `handshake/mod.rs`, `record/mod.rs` |
| T3 | Hermetic tests: brotli **and** zstd paths, unoffered algorithm, declared-length lie, truncated vector, transcript-carries-type-25, bomb bound | `handshake/mod.rs`, `record/mod.rs` |
| T4 | Cleartext peer named instead of a bogus record size (`peer does not speak TLS: … ("HTTP/")`) | `record/mod.rs`, `record/stream.rs` |
| T5 | `ProbeClass` vocabulary produced from the typed error (`NativeError` / `PingError`) at the failure site; per-phase class histograms in the batch summary; per-result lines routed to the actions panel only (`AppState::log_activity`), never the store | `ops/ping_native.rs`, `ops/ping.rs`, `ops/events.rs`, `state.rs` |
| T6 | Phase 2 dispatches best-first (ascending phase-1 latency, unmeasured last) — the dispatch order is an explicit insertion-order vector, since `per_endpoint` is keyed by a hash | `ops/ping.rs` |
| T7 | Interrupted runs report at quit (`batch interrupted at quit: N of M final-phase probe(s) reported, no batch summary; staged writes flushed, S left staged`) | `ui/mod.rs` |
| T8 | `edge`/`ios`/`randomized` fingerprint presets served (roster rows `edge_106`, Safari-on-iOS, randomized-Chrome alias); `360`/`qq`/`android`/`randomizednoalpn`/`unsafe` stay refused | `xray-tui-native/src/security/fingerprint.rs`, `capability.rs` |
| T9 | Validation warnings separated from errors (`total_errors` excludes `insecure=true` profiles; the two log lines say errors vs warnings) | `xray-tui-config/src/{subscription,import_export}.rs`, `ops/events.rs`, `ops/subscriptions.rs` |
| T10 | Dead settings removed: `real_ping_window`, `batch_page_size` (both defined, displayed, persisted, read by nothing — the 2026-09-14 design doc had already recommended their deletion) | `app_config.rs`, `ops/settings.rs`, `ui/settings.rs` |
| T11 | `probe_host --verify` (the harness was `insecure` by default, i.e. structurally blind to every verify-class failure it is used to re-probe) | `examples/probe_host.rs`, `NATIVE_CORE.md` |

## 4. Deliberate non-changes

- **`real_ping_concurrency` default is already 100** (the run's `5` was a saved
  override), so the throughput recommendation becomes "order + surface the
  cost", not "raise the default".
- **Environmental classes** (`alert 2 40`, REALITY `received real certificate`,
  the transient `record too large`, the fast-phase timeout/DNS majority) get
  diagnostics, not code: openssl reproduces the first, the second is a config
  or auth mismatch, and per-unique-address the feed is majority-dead
  (52.5% ok / 26.8% timeout / 17.0% non-resolving / 3.0% refused).
- **Subscription retry re-fetches and re-upserts the whole list** (cumulative
  4,552 → 23,574 → 24,048 → 26,547 on three early-ended bodies). Correct
  (upserts are idempotent and SOURCE-scoped) but wasteful; resume-or-skip
  needs its own design, so it is a follow-up, not a drive-by change.
- **`probe_host` on a fresh config dir** logs `failed to write DNSCrypt
  resolver cache` twice and self-heals (the file lands on the next attempt) —
  observed, benign, unfixed.

## 5. Verification

- `cargo fmt`, `cargo clippy --workspace --all-targets` (clean),
  `cargo nextest run --workspace`: **1943 passed, 8 skipped**.
- New tests: ALPS spec vector + golden hello/record re-pin, compressed-cert
  decompression (brotli/zstd), unoffered algorithm, declared-length mismatch,
  truncated vector, raw-transcript flight, decompression bound, cleartext-peer
  detection (record + stream), `ProbeClass` mapping per `NativeError` variant,
  class histogram in the summary line, best-first phase-2 order, fingerprint
  preset mapping + roster-row resolution.
- Live: `probe_host --verify` across the recorded failing set (55/78 now OK,
  listed above).
- TUI smoke run (isolated `XDG_CONFIG_HOME`, 4 profiles, `tui-test`):
  `batch: planned 4 link(s) over 4 endpoint(s) in the feed`; the summary line
  rendered `phase1 ok=3 hard-fail=1 soft-fail=0 [timeout=1] …` and
  `phase2 ok=0 failed=3 [tls=3] skipped-unreachable=1`; the `[fast]` marker
  appeared on the timed-out row; a real failure kept the fast delay on the
  table; per-result lines showed in the Actions panel and **stopped reaching
  the log store**; quitting mid-batch produced the interrupted-run line with
  the phase-2 semantics documented in its text.

## 6. Baseline amendments

1. `xray-tui-tls` speaks RFC 8879 certificate compression and encodes ALPS per
   draft-vvv-tls-alps; the transcript carries the message as sent.
2. `ping_native::ProbeClass` is the probe-failure vocabulary; a batch summary
   counts by class, and per-result chatter is session-only (the store keeps
   diagnostics, the DB keeps results).
3. The native capability gate serves `edge`/`ios`/`randomized` and still
   refuses the presets with no engine hello.
4. `ValidationSummary::total_errors` excludes `security_warning_count`.

## 7. Follow-ups

- Subscription retry: resume or skip an unchanged batch (no re-download of the
  26k-link body on `Source ended early`).
- Class precision: a post-handshake EOF on the protocol step surfaces as
  `NativeError::Tls` and therefore counts as `tls`; if that matters, the
  tunnel's I/O errors need a distinct variant before the class can say
  `protocol`.
- `toasty::query slow query` still has no statement label or duration (carried
  over from 2026-09-15).
- A batch-wide ETA/progress surface (the counters exist; the phase-2 rate is
  measurable per window).
