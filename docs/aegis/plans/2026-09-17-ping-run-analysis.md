# 2026-09-17 — Ping-run analysis: two malformed extension bodies, phase-2 cost, and a truncated import

Trigger: the user ran a Fast + Real ping batch on `7cabbb7` (the commit that
landed the 2026-09-16 engine fixes) and asked for a log analysis, an issue
classification, and an improvement plan. This document is the evidence base and
the disposition; the slices that landed with it are listed in §5.

Headline: **the engine emitted two extension bodies that are invalid on the
wire, and every fingerprint preset the app hands out was affected.** They are
the same class as the 2026-09-16 ALPS bug (a body length the RFC does not
allow), and they were invisible to every existing check because the roster's
only body-level verification is self-referential. The 2026-09-16 fixes
themselves hold: `server presented no certificate` and `record too large` are
**0 rows** (201 and 134 before).

## 1. Evidence base

| source | what it is |
| --- | --- |
| `~/.config/xray-tui/logs.lmdb` | 159,301 entries total; **19 from this run** — the store holds only the startup envelope, the plan line, the interrupted line and the import's decisions (decision 15 makes per-result lines session-only, and the batch was quit mid-phase-2, so the store carries no outcome data) |
| `~/.config/xray-tui/data.db` | the durable witness: `profile_stats` (34,562 rows = the batch plan), `endpoints`, `protocols`, `groups` |
| `updated_at` buckets | phase timing (write-behind flushes continuously, so per-minute write volume times the phases) |
| fingerprint A/B | the same link config (host, SNI, ALPN, transport), only `security.fp` swapped, replayed through `crates/xray-tui-native/examples/probe_replay.rs` (added here) — the batch's own probe policy (`probe::fetch`), not the handshake-only `probe_host` |
| positive control | the one link that succeeded in-batch, replayed 3× → **1 OK, 2 early-EOF**: the harness can reproduce a success, so a 0/N replay is the feed, not the instrument |
| `thirdparty/` | the captures that settled the ECH wire shape (see §2) |

Run shape (`7cabbb7`, 2026-09-17):

| stage | window | volume | outcome |
| --- | --- | --- | --- |
| import | 08:14:07 → 08:15:26 | 3 fetch cycles for 2 groups | 2 cycles ended early; 34,562 links / 18,334 endpoints persisted |
| phase 1 (fast, concurrency 200) | 08:16 → 08:21 | 34,562 results | 17,044 ok (49.3%) · 10,755 timeout · 4,587 dns · 1,760 refused · 379 no-route · 30 unreachable |
| phase 2 (real, concurrency 5) | 08:20 → 08:43 | **3,381 of 17,051 candidates (19.8%)** | **1 ok, 3,380 failed** (2.45 results/s) |

Phase-2 classes, from the persisted `error_text` (the class histogram that only
the summary line carries never existed — the batch was interrupted):

| class | rows | reading |
| --- | --- | --- |
| `timeout on probe attempt (limit 5s)` | 1,296 | 38%; the stage is unknown (dial / tls / transport / protocol / target) |
| `alert 2 40` (handshake_failure) | 684 | the front refuses that SNI — reproduces with openssl; dead/stale config |
| ws `HTTP 403/530/400/302/…` | 600 | the CDN edge answered; origin gone or path/Host stale |
| `REALITY: received real certificate` / `missing pbk` | 224 | not REALITY with that key |
| `not testable by the native engine` | 171 | capability gate (154 are `qq`/`android`/empty fingerprints) |
| `I/O error: early eof` / `reset by peer` / truncated vless header | 167 | tunnel died mid-protocol |
| `v2rayhttp` 4xx/5xx | 75 | origin refused the request |
| `certificate verification failed` | 53 | 49 SAN mismatch, 4 chain — **not** the 2026-09-16 defect |
| **`alert 2 50` (decode_error)** | **47** | **the engine defect (§2): firefox 37, ios 6, safari 4 — no other fingerprint produces it** |
| `httpupgrade` 101-expectation, peer-cleartext, MAC mismatch, rest | ~93 | long tail |

Persistence health: every one of the 34,562 rows carries an outcome, the phase-2
patch kept the phase-1 delay beside every real failure (3,374 of 3,380), no
`link_writer` warning was emitted, and the DB absorbed ~145 row-writes/min
through phase 2 — the write-behind path is intact.

## 2. The engine defect: extension bodies the wire format forbids

### 2.1 `encrypted_client_hello` with an empty body

Every generated roster row of the Firefox, Safari, Safari-on-iOS, Chrome and
Chrome-Android families (`profiles/generated/{firefox,safari,safari_ios,chrome,chrome_android}.rs`,
36 sites) declared `raw[0xfe0d, ""]` — an `encrypted_client_hello` extension
whose body is zero bytes. RFC 9849 §5.1's outer `ECHClientHello` cannot be
empty (`type || cipher_suite || config_id || enc<0..2^16-1> || payload<1..2^16-1>`,
≥38 bytes).

Evidence chain:

1. `alert 2 50` appears **only** on the fingerprint ids that resolve to those
   rows: firefox 37, ios 6, safari 4 — 47 rows; `chrome`, `edge`, `randomized`,
   `random` and no-`fp` never produce it.
2. Same config, only `fp` swapped, against `91.193.58.201:443` (SNI
   `PioHosting.ir`, ALPN `http/1.1`): `chrome`/`edge`/`randomized` complete the
   handshake (the tunnel reaches the ws step and the origin answers 403);
   `firefox`/`ios`/`safari` get `alert 2 50` in 0.1 s. `openssl s_client`
   against the same host/SNI completes with a valid certificate, so the peer is
   fine and the difference is ours.
3. Six independent Cloudflare-fronted hosts from the same failure set: `chrome`
   → `alert 2 40` (parsed, then refused) or ws 403; `firefox` → `alert 2 50`
   **6/6**.
4. ALPN on/off changes nothing (not the ALPS path).
5. Removing just that extension from the resolved Firefox row
   (`firefox_139_windows_desktop`) made the same probe reach the ws step —
   proving the extension was the trigger, not the row's other differences.
6. The fix (a GREASE `ECHClientHello`: outer type, HKDF-SHA256 + AES-128-GCM,
   a random `config_id`, 32 random `enc` bytes, a 144-byte random `payload`)
   makes all six fingerprints behave identically to `chrome` on the same hosts.

Why nothing caught it: `tests/generated_ja4_gate.rs` validates the wire against
**our own** parser and the JA4 id/hash projection (ECH bodies are invisible to
JA4), the tier-2 grader validates against **tls.peet.ws**, a lenient Go peer
that skips an ECH extension it does not implement — and Go's `crypto/tls` omits
ECH entirely. The captures that settled the body shape came from
`thirdparty/impersonator/bctls` (a maintained impersonation library ships a real
captured 186-byte ECH body for MacChrome/MacFirefox) and RFC 9849.

### 2.2 `compress_certificate` with an empty algorithm list

The Safari, Safari-on-iOS and Chrome-Android rows declared `compress[]`
(28 sites). RFC 8879 §4's `CompressionAlgorithm algorithms<2..2^8-2>` carries at
least one algorithm; a zero-length list frames as a 0-byte body. Same evidence
shape: removing that extension from the resolved Safari row
(`safari_26_macos_desktop`) made the probe pass, after which the rows emit
`compress[brotli]` (Safari family) / `compress[brotli, zstd]` (Chrome-Android) —
the algorithms the engine can actually decompress (decision 17).

The same sweep found a third instance of the class: `raw[0x0022, ""]`
(`delegated_credentials`, RFC 9345 §3, whose body is a
`SignatureScheme ...<2..2^16-2>` vector) in two rows, now
`raw[0x0022, "00020403"]`.

### 2.3 What is now enforced

- `ExtensionSpec::EchGrease` with the body framed per RFC 9849, and the encoder
  **refuses** an empty `payload` — the invalid shape is no longer representable.
- `ExtensionSpec::CompressCertificate` refuses an empty list.
- `profiles::tests::every_roster_hello_has_rfc_minimal_extension_bodies` sweeps
  **every** roster hello (hand + generated) and asserts each extension body
  meets its RFC's minimum (SNI, supported_groups, signature_algorithms, ALPN,
  compress_certificate, delegated_credentials, supported_versions, key_share,
  ALPS, ECH) and that no non-GREASE id repeats — the regression that would have
  caught all three.
- `tests/strict_peer_presets.rs` (`#[ignore]`, network) handshakes a
  Cloudflare front with every preset the app resolves — the tier-2 oracle for
  the class of defect a self-parser cannot see. It fails on the pre-fix code
  (firefox/ios/safari answered `alert 2 50`) and passes after.

### 2.4 Related: `server_name` for IP literals

RFC 6066 §3 forbids a literal address in `server_name`, and Go's
`hostnameInSNI` (hence xray-core and sing-box) sends no extension at all when
the only name is an address; the engine sent the literal. Fixed at the encoder
(`is_ip_literal`, bracketed IPv6 included); verification still uses the address,
so IP-SAN chains keep working.

Measured effect on this feed: **none observed** — re-probing 25 of the 218
affected rows (`host_type=ipv4`, no explicit `sni`) still yields `alert 2 40`
(17) / timeout (3) / EOF (2). The fix is reference parity (what the core we are
a client for puts on the wire) and removes a self-inflicted
`unrecognized_name` class, not a measured success-rate win.

## 3. The other findings

### F2 — Phase 2 is cost-bound on a saved `real_ping_concurrency = 5`

2.45 results/s × 17,051 candidates = **1.9 h** for the phase; the run was quit
at 19.8% of it. The code default is 100 and 5 is a persisted override. The
ceiling is not the engine: 200 concurrent fast probes finished 34,562 links in
~5 min, and the replay harness runs 20 concurrent real probes at ~6/s.
Landed: the batch warns once when the effective value is below 50, naming the
default and the measured rate.

### F3 — The fast phase's number is not a usability signal

`17,044 ok` looks like half the feed works. It does not:

- in-batch: of 3,381 attempts drawn from the **best** fast latencies, 1
  succeeded (0.03%);
- replay of 60 **never-attempted** fastest fast-ok links: 59 fail, 1 ok — while
  plain TCP connect to those same 60 addresses succeeds **60/60**;
- 60 links whose in-batch failure was `alert 2 40`: 60/60 fail again;
- 60 in-batch `timeout` links: 60/60 fail again;
- raising the probe budget 5 s → 20 s changes nothing (60/60 still fail).

The fast number is honest about what it measures — a TCP handshake — and that
handshake is typically answered by a **nearby CDN edge** (the 1–49 ms band holds
3,856 links, all fastly/cloudflare-fronted). `best-first` ordering by it was
faithfully applied (mean phase-1 delay per phase-2 minute climbs 11 ms → 40 ms)
and found nothing. The phase still earns its place as a **reachability
prefilter**: it retired 17,511 links before phase 2 ran.

### F4 — Phase-2 candidates are 1.74 links per endpoint, and dedup only fires on success

`dedup_endpoints=true` retires an endpoint's remaining real tasks after the
first **successful** real ping — with a ~0% success rate it never fires: 3,380
probes covered 1,824 endpoints (1.85 links each). The full candidate set is
17,051 links over 9,803 endpoints; a per-endpoint failure verdict would probe
9,803 (−42.5%). A failure on one config is not proof for a sibling config on the
same server: **deliberately not changed** (policy decision, not a free win).

### F5 — An interrupted run kept no classification, and its denominators mixed phases

The class histogram exists only in the batch-end summary line; on quit the store
got `batch interrupted at quit: 3381 of 34562 final-phase probe(s) reported, no
batch summary`. The line is precise about what it counts, but `total` was the
whole plan while `completed` counted final-phase results only, so it read as
"9.8% done" when phase 1 was finished and phase 2 was 19.8% done; the status bar
shared that denominator and sat at `0 / 34,562` through the whole 5-minute phase 1.
Landed: the batch's shared state is published on `AppState`, the quit path
prints the same summary line a completed run writes (plus settled / in-flight),
and the progress pair is restated to the final phase's candidate count when that
set is known (phase 1 has no denominator rather than a wrong one).

### F6 — DNS persistence never ran for a feed-wide batch (0 addresses after a full run)

`endpoint_ip` had **0 rows** and `endpoints.resolved_at` was set on **0 rows**
after a 34,562-link run. `spawn_dns_resolve` fires only on connect, on `x`, and
after a real result **whose endpoint is on the loaded page** (it early-returns
when the row is not in `state.endpoints`); a feed-wide batch is on none of those
paths. Consequences: the address table ADR 0005 introduced started empty every
launch, the flag/country columns stayed blank, and the whitelist pass — which
also only walks the loaded page — had nothing to check. Meanwhile the batch
resolved every host it probed and discarded the addresses. **Still open** (see
§6): the fix is to hand the batch's resolved addresses to the table.

### F7 — `not testable` is persisted as a real failure

171 rows: 143 `qq` + 8 `android` + 3 empty fingerprint ids, 6 hysteria v1 (no
native implementation), 6 VLESS account encryption, 5 Shadowsocks method. They
are correct verdicts (decision 20) but land in `error_kind='real'`, so they
count as probe failures in the results column. Serving `qq`/`android` needs a
roster regeneration — left as §6.

### F8 — The import truncated large bodies, and recorded them as `ok`

Both `sevcator:tr` fetches ended early (`Source ended early: error decoding
response body`) after ~28–31 s, having parsed 29,973 and then 26,547 links. The
run's own log proves the truncation; the differing parse counts of two fetches
of one URL confirm it (the per-run `seen_*` dedup sets are deterministic, so two
complete reads of one body would agree).

What the body actually is (fetched directly, 2026-09-17): **26.0 MB, plaintext
(not base64), 172,034 URLs, 26,491 unique `host:port`, 78,256 unique
`(host,port,query)`** — the feed repeats one server under many `#fragment`
tags, and identity ignores remarks, so 172,034 lines collapse legitimately. The
group ended with 14,987 endpoints / 29,915 links. So the loss is a truncated
tail — on the order of a third to a half of that day's bodies — not "83%": the
feed's own redundancy accounts for most of the raw-line gap, and the body was
served in 9–18 s when re-fetched, so the truncation is a **deadline-vs-link-speed
race**, not a fixed size cap.

Cause (prime suspect, not proven): the import client's `reqwest`
`.timeout(30s)` is a **total** deadline covering the response body, and the
streaming loop awaits `persist_batch` between body reads, so the deadline also
billed the consumer's own DB work. `CHUNK_TIMEOUT` (120 s) never fires, so the
30 s cap always wins. Landed: a connect deadline plus a per-read idle deadline
(`connect_timeout` 30 s + `read_timeout` 60 s, below `CHUNK_TIMEOUT`), and a
partial import is now `status='error'` with a message naming the truncation and
the stored count — it used to be indistinguishable from a complete one.
Also latent and fixed: `spawn_auto_update` fetched without the in-flight guard
the manual paths use, so a due group could be fetched twice at once; all fetch
paths now share one exclusion registry.

### F9 — `toasty::query slow query` still carries no label or duration

One occurrence this run, during the bulk upsert. Carried over from 2026-09-15;
upstream to the toasty crate.

## 4. Improvement plan — disposition

| # | Change | Status |
| --- | --- | --- |
| P1 | **Fix the malformed extension bodies** (ECH GREASE, `compress_certificate` list, `delegated_credentials` body) + RFC-minimum sweep test + strict-peer tier-2 test | **landed** (§2) |
| P1b | Omit `server_name` for IP literals (RFC 6066 / `hostnameInSNI` parity) | **landed** (§2.4) |
| P2 | Surface the real-phase cost: warn below 50, log the phase-2 shape at its start | **landed** (F2) |
| P3 | Report an interrupted run: publish the batch handle, print the summary at quit, per-phase denominators | **landed** (F5) |
| P4 | Retire an endpoint's real tasks on the first **failure** under `dedup_endpoints` (−42.5% probes) | deferred — policy (F4) |
| P5 | Persist the resolved address's country in `endpoint_ip`, and read it back so the flag needs no mmdb | **landed** (§5, user-requested) |
| P6 | Import integrity: read/idle deadlines, `status='error'` on a partial import, one in-flight registry for every fetch path | **landed** (F8) |
| P7 | Serve `qq`/`android` fingerprints (needs the ja4db roster subset regenerated) | deferred (§6) |
| P8 | Per-stage attribution inside `NativeError` so `timeout` (38%) stops being one bucket | deferred (§6) |
| P9 | `toasty` statement label + duration on the slow-query warning | deferred (upstream) |
| P10 | Hand the batch's resolved addresses to `endpoint_ip` in the fast phase (fixes F6 at no network cost) | deferred (§6) |

Deliberate non-changes (so they are not re-litigated):

- **The feed's dead links are not ours to fix.** With §2 removed, the remaining
  failures are server-side and reproduce from an independent client (openssl
  handshakes, then the origin answers 403/530 or the front refuses the SNI).
- **`best-first` ordering** stays: free, and F3 changes what the number means,
  not the order.
- **`error_ttl_hours = None`** stays a user choice; the markers it keeps are
  correct.

## 5. Landed slices (files)

| slice | files |
| --- | --- |
| Emitter (`gen_specs.py`) — the roster's `*.rs` files are its output, so the bodies are synthesized there and the files regenerated (`--manifest`, `--emit`, `--selftest` PASS): ECH names the engine's `EchGrease` variant, `delegated_credentials`/`signature_algorithms_cert` get their sig-alg vectors, an empty `compress_certificate` template becomes `brotli`, and `_ext_token` refuses to emit a bodyless extension whose id is in `EMPTY_BODY_ILLEGAL` (`pre_shared_key` included) | `crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py`, `specs_manifest.json`, `profiles/generated/*` |
| ECH GREASE extension + `compress_certificate`/`delegated_credentials` bodies + IP-literal SNI skip | `xray-tui-tls/src/spec/mod.rs`, `hello/mod.rs`, `crypto/fingerprint/ja3.rs`, `fingerprints/builder.rs`, `profiles/mod.rs`, `profiles/generated/{chrome,chrome_android,firefox,safari,safari_ios}.rs`, `tests/strict_peer_presets.rs` |
| `endpoint_ip.country` (user-requested) + schema tag 11 | `xray-tui-db/src/{models_toasty,endpoint_ip,database}.rs`, `xray-tui/src/ops/enrich.rs` |
| Real-phase cost warning + phase-2 shape line | `xray-tui/src/ops/ping.rs` |
| Interrupted-run summary + per-phase denominators | `xray-tui/src/{ops/ping,ops/events,state,ui/mod}.rs` |
| Import deadlines + partial-import status + in-flight registry | `xray-tui/src/ops/{stream_import,subscriptions}.rs` |
| Replay harness (diagnostic, kept) | `xray-tui-native/examples/probe_replay.rs` |

**Schema tag 11 wipes the database on the next launch** (decision 4: the tag is
a reset, not a migration) — the column cannot be added by `push_schema` to an
already-pushed table. Verified escape hatch for keeping the current data (the
rest of the schema is unchanged by this bump):

```sql
ALTER TABLE endpoint_ip ADD COLUMN country TEXT;
PRAGMA user_version = 11;
```

`sqlite3 .schema endpoint_ip` after that ALTER is identical to a freshly pushed
tag-11 table (`CREATE TABLE … ("endpoint_id" BIGINT NOT NULL, "ip_key" BLOB NOT
NULL, "country" TEXT, PRIMARY KEY ("endpoint_id","ip_key"))` + the same covering
index, the only difference being the column-name quoting), and a live run
against such a file wrote 162 countries and rendered the flags — so the ALTER is
equivalence, not a guess. Without it, the next launch deletes the file and the
subscriptions re-import.

## 6. Follow-ups

1. **F6/P10** — the batch already resolves every host it probes; persisting those
   addresses for the endpoints it touched fills `endpoint_ip` (and the flags)
   for a feed-wide run at no additional network cost.
2. **P8** — per-stage `NativeError` variants: with `timeout` at 38% of phase 2,
   the next run's dominant class is still a bucket with no stage.
3. **P4** — decide the per-endpoint failure-dedup policy.
4. **P7** — regenerate the kept roster subset from the ja4db manifest to serve
   `qq`/`android` (and any future preset) instead of refusing them.
5. Subscription import: resume instead of re-fetching a 26 MB body.
6. `toasty` slow-query label + duration.

## 7. Verification

- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean.
- `cargo nextest run --workspace`: see the run recorded in the commit that
  carries this document.
- New tests: `spec::tests::{ech_grease_frames_a_parseable_outer_client_hello,
  ech_grease_refuses_an_empty_payload, compress_certificate_refuses_an_empty_algorithm_list,
  server_name_is_omitted_for_ip_literals}`,
  `profiles::tests::every_roster_hello_has_rfc_minimal_extension_bodies`,
  `xray-tui-tls::strict_peer_presets` (ignored, network),
  `database::tests::{stored_country_survives_re_resolution,
  country_writes_before_the_address_row_exists}`,
  `ops::enrich::tests::stored_country_reaches_the_ui_without_the_mmdb`,
  plus the batch-observability, phase-denominator and import-integrity tests
  named in the slice reports.
- Live: `probe_replay` A/B over the recorded failure set (firefox/safari/ios now
  behave as chrome against the same hosts), the positive control, and
  `cargo test -p xray-tui-tls --test strict_peer_presets -- --ignored`.
- **Tier 3** (`XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native
  --features native-e2e --test vless --test vmess --test trojan --test shadowsocks`,
  pinned xray-core 26.3.27 + sing-box 1.13.16): 173 green, 6 ignored — no interop
  regression, and the changed bodies are now covered: the pre-existing matrix
  used `fp("chrome")` everywhere, which resolves to the hand-declared, unchanged
  `chrome_130`, so this slice added `tcp_firefox`/`tcp_safari`/`tcp_ios` rows
  (both cores, 6 tests) — the three generated-tier presets whose hellos carried
  the malformed bodies, now handshaking against real cores.
  Two caveats, stated rather than implied: the e2e configs use an explicit
  `localhost` SNI, so the IP-literal `server_name` skip is exercised only by the
  live A/B replay (§2.4) and `server_name_is_omitted_for_ip_literals`; and REALITY
  rows provision `FixedChrome133`, i.e. they never exercise a generated-tier
  hello either.
- The emitter's own gate: `gen_specs.py --selftest` → PASS (round-trip,
  selection, emit determinism).
