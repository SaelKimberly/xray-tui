# Native testability: trustworthy verdicts on a real feed

- **Date**: 2026-09-22
- **Kind**: design spec (spine) + four in-place amendments
- **Status**: proposed — awaiting user review
- **Depends on**: `2026-09-14-native-ping-pipeline-design.md`, `2026-09-17-purge-reason-design.md` (ADR 0006), `2026-09-17-batch-ping-pipeline-design.md` (ADR 0008), `NATIVE_CORE.md`

## 1. Outcome

Increase the fraction of feed links that receive a **trustworthy** real-ping verdict, and make every persisted verdict an auditable function of the failure stage it came from.

Coverage alone is explicitly **not** the criterion. The chosen fingerprint policy (§5.1) makes coverage free on day one, so a coverage percentage would measure the fallback rather than the work. The spec's acceptance is fidelity, verdict integrity, and one falsifiable speed claim (§7).

## 2. Why this exists

A read-only audit of a real two-subscription feed (8,071 endpoints / 10,264 links, `data.db`; 162,875 heed log entries) produced seven findings. The dominant cause of failure is the feed, not the engine — two oracles established that (§9). But a measurable subset fails for reasons under our control, and several persisted verdicts do not mean what they say.

Audit run analysed: `2026-09-21 17:30:59 → 17:40:55` (quit at 598 s):

```
links=9378 plan=1582 ms untestable=0 queue-full=0 deferred=0
fast ok=3824 hard-fail=5554 [timeout=4712 dns=170 refused=586 no-route=80 unreachable=6]
real ok=9     failed=996     [timeout=535 tls=213 reality=53 transport=22 config=173]
skipped-unreachable=5554 | wall=598230 ms
```

## 3. Scope

| # | item | lands in |
|---|---|---|
| 1 | fingerprint fallback; **derived** approximation marker | this spec; `NATIVE_CORE.md` |
| 2 | mlkem differential → fix → pq-enc row green | this spec; `crates/xray-tui-native/tests/vless.rs` |
| 3 | `ProbeClass` stage attribution (re-opens a recorded deferral) | amend `2026-09-14-native-ping-pipeline-design.md` |
| 4 | purge mapping: config-defect → `config_invalid`; obsolete-transport cause named | amend `2026-09-17-purge-reason-design.md`; ADR 0006 doc note |
| 5 | budget authority: inner steps clamp to one number; value from measurement | amend `2026-09-14-native-ping-pipeline-design.md` |
| 6 | removed-transport emit + connect downgrade + counter split | amend `2026-09-17-batch-ping-pipeline-design.md`; ADR 0008 note |
| 7 | HTTP-transport Host fallback + default headers (ws/httpupgrade/xhttp) — wire-affecting, **measure-first** | this spec; amend `NATIVE_CORE.md` if the wire changes |

**Owner documents are amended in the same task that lands the delta.** Their content is specified here so the amendment cannot drift; they are not edited at spec-write time, because a spec marked `implemented` must not describe unlanded behaviour.

## 4. Non-goals (deliberate, with reasons)

- **No tuning-default changes** (`real_ping_concurrency`, `dedup_endpoints`, `error_ttl_hours`). The completion win comes from engine capacity at fixed concurrency, so a config tweak cannot masquerade as engine progress.
- **No new `PurgeReason` variant.** The column is a `TEXT CHECK` over 7 variants; adding one requires altering the CHECK, which is a schema-tag bump, and a tag bump makes `Database::open` **delete and recreate the database** (decision 4). A data reset for a label is not a trade this spec makes.
- **`tcp_ping` address-set rework — out.** 4,120 of 4,712 fast timeouts are bare IPv4 literals; ≤115 (2.4 %) could involve a multi-address race. Below measurement noise for a change to the path every link uses. Re-openable with a counter-example.
- **No whole-feed live run as an acceptance vehicle.** `ops/ping/flow_cost.rs` is the measurement authority (ADR 0008's perf lab).
- **No 0-RTT ticket-path work.** See §5.8.

## 5. Design

### 5.1 Fingerprint policy — approximate and mark, never silently substitute

`security/fingerprint.rs` gains the single decision point. Nothing is stored.

| `security_fp` | honest reading | approximation? |
|---|---|---|
| absent (`None`) | no fingerprint requested | no — engine default *is* the requested shape |
| `""` / `unsafe` | no uTLS fingerprint requested | no — same reading |
| `chrome` / `firefox` / `safari` / `edge` / `ios` / `random(ized)` | resolves to a roster row | no |
| `qq` / `android` / `360` / `hellochrome_120` / unknown | requested, no roster row | **yes** — probes with the default hello, marked |

`360` appears in the table for completeness; the measured feed carries none. The subtotals below are the measured populations.

Today's capability-refused population is **1,165 links**, split:

| population | links | disposition after this spec |
|---|---|---|
| `qq` | 875 | approximated (marked) |
| `android` | 46 | approximated (marked) |
| `hellochrome_120` | 2 | approximated (marked) |
| **subtotal** | **923** | marked approximated |
| `""` | 241 | honoured as specified (unmarked) |
| `unsafe` | 1 | honoured as specified (unmarked) |
| **subtotal** | **242** | unmarked |
| **total** | **1,165** | — |

Roster-mapped links (chrome/firefox/safari/edge/ios/random) never carried a refusal and are **outside** this denominator.

**The marker is derived, not stored.** The predicate is *not* "`parse_fingerprint_id` fails" — that would wrongly mark `""`/`unsafe`, which are honoured. It is:

> approximated ⟺ `security_fp` is present, is not `""` and is not `unsafe`, **and** `parse_fingerprint_id` fails.

`security_fp` is already in the page projection (`crates/xray-tui-db/src/profiles_query.rs:616-617`), so the label needs no new column, no projection change, no schema-tag bump and no wipe. One predicate feeds both the gate and the label, so they cannot drift — the discipline `endpoint_rank::dns_unresolved(row)` already follows.

The fallback hello reuses `SpecProvisioner` / `FixedChrome133` (`crates/xray-tui-tls/src/reality/mod.rs:158-201`); no new hello mechanism is introduced.

**The `""` / `unsafe` reading is a fix, not a new feature.** `parse_fingerprint_id` refuses `""` although an absent `fp` is supported and `security.fp()` returns `Some("")` for a present-but-empty field; `unsafe` is v2rayN's "no uTLS". Both mean "no fingerprint requested" and must map to the engine default.

### 5.2 Budget authority — one number, steps clamp to it

`crates/xray-tui-native/src/error.rs:174-181` declares five independent constants:

```rust
pub const DIAL: Duration = Duration::from_secs(10);
pub const TRANSPORT: Duration = Duration::from_secs(10);
pub const SECURITY: Duration = Duration::from_secs(10);
pub const PROTOCOL: Duration = Duration::from_secs(10);
pub const TUNNEL_READ: Duration = Duration::from_secs(30);
```

An attempt traverses dial → security → transport → protocol **sequentially**, so its legitimate window is their sum (~40 s), while the probe budget is **5 s** — smaller than a single one of the engine's own steps. The 5 s is our shipped default, not a user choice (`crates/xray-tui-config/src/app_config.rs`, `default_real_ping_timeout_secs`).

Design: the constants become **clamps** against a caller-supplied attempt deadline — each step takes `min(step_constant, remaining)`. The clamp is a property of the caller's budget, not of the step: a caller that supplies a budget has its steps clamped; a caller that supplies none (the live proxied tunnel) keeps the constants. Under that rule `TUNNEL_READ`'s 30 s becomes **unreachable for the probe** (four clamped steps always consume the budget first), which is the intended outcome rather than an exemption, and a live connection is untouched.

The budget's **value** is not fixed here; it is an output of §6 (above the p99 of *successful* attempts).

### 5.3 Evidence → verdict table

`crates/xray-tui/src/ops/purge.rs::reason_for` keeps its role as the only place verdicts are decided. **No new `PurgeReason` variant; no schema change.**

| evidence | today | after |
|---|---|---|
| server-auth failure against a REALITY peer | `reality_fallback` | unchanged |
| missing `pbk` / malformed `sid` | `reality_fallback` | **`config_invalid`** |
| peer answered cleartext | `not_tls` | unchanged |
| HTTP-layer refusal on a transport the peer's core removed | `transport_rejected` | unchanged verdict; cause named in `error_text` |
| **any failure from an approximated probe** | a verdict was possible | **no verdict** — evidence carries "shape approximated"; `reason_for` returns `None` |

The last row is expressed as **evidence**, not as a special case in the taxonomy: `FailureEvidence` gains the carrier and the decision function declines it. ADR 0006's premise holds — a taxonomy revision still costs one function, not a schema change.

### 5.4 The measurement stage — three numbers, one home

`ops/ping/flow_cost.rs` currently probes through `struct NullRunner`, so the real level's network cost is measured nowhere. It gains a **real** `BatchProbeRunner` over a pinned, bounded slice (endpoint ids recorded in §8), run against a **copy** of `data.db` via `XRAY_TUI_MEASURE_DB` — the harness's existing rule, never the live file. Rows stay medians, so new rows remain comparable to the existing stub rows.

One stage produces all three numbers:

1. **results/s at fixed concurrency** → the §7 class-2 bar.
2. **per-attempt latency distribution** → the p99-of-successes rule for the budget value (§5.2).
3. **the header A/B** on the `403`/`409`/`530` rows → whether item 7 changes the wire at all.

### 5.5 Build-time contract — the removed transport

`to_xray_stream_settings` (`crates/xray-tui-proto/src/proto_spec/common.rs`) remaps `XHttp` → `"splithttp"` and sends everything else through `transport.type_str()`, so `TransportConfig::Http` emits `"network": "http"` and `Quic` emits `"network": "quic"`. Both are **removed features** in xray-core 26 (`thirdparty/Xray-core/infra/conf/transport_internet.go:33-34`), and `PrintRemovedFeatureError` **returns an error** (`common/errors/feature_errors.go:25-27`), so such a config fails to load.

Meanwhile `capability.rs::transport_reason` calls `TransportConfig::Http(_)` **supported** (`=> None`). The two owners disagree about the same transport with no reconciliation point.

Fix: `inject_to` refuses `Http`/`Quic` with a `SupportError` → `BuildError::Support` (the existing reality/cipher precedent), and the connect path downgrades such a link to native (which serves it) with a `warn` naming host + kind + reason (decision 20's shape).

**This path is latent, not live.** `config.json` has no `protocol_core_overrides` key, and `routing_rules` and `dns_settings` are empty tables, so decision 20 sends every link to the native attempt first and the gate accepts `Http` — the xray builder is never reached for a `type=http` link today. Same for `quic`: the gate refuses it, but the feed carries no vless+quic rows. It is one routing rule or one override away from firing.

### 5.6 Class precision

The class must be produced by the **stage that failed**, never by the outermost wrap. `probe.rs:117-123` wraps the tunnel in target TLS and maps to `NativeError::Tls`, so a tunnel-payload error becomes `TlsError::Io` (`crates/xray-tui-tls/src/record/mod.rs:89-90`), then `ProbeClass::Tls` (`ping_native.rs:113-116`). The identical failure on a plain-`http` probe URL would be `Transport` (`probe.rs:147`).

Measured impact: **151 rows** — `vless response header truncated (EOF)` (94) and `early eof` (57) — counted as TLS when the peer closed the tunnel before sending anything. The VLESS peel is correct (xray's inbound writes `version|addon_len|addons` unconditionally, `proxy/vless/inbound/inbound.go:618-620`); only the label is wrong.

This re-opens a recorded deferral: `plans/2026-09-16-ping-engine-tls-fixes.md` closed with *"class precision and toasty labels deferred"*, and `plans/2026-09-17-ping-run-analysis.md` lists P4/P7/P8/P9/P10 deferred. The 151-row measurement is the reason to re-open.

### 5.7 Counter split

`counters.untestable` (`ops/ping.rs:1678`) counts only the plan-time kind gate (`capability::kind_supported`, `ping.rs:1616-1620`), while config-level refusals land in `real_fail[config]` (`ping.rs:1896`, `ProbeOutcome::soft_failure`). One fact, two buckets — which is why the 09-21 run line reads `untestable=0 … config=173` while 173 rows carry a `not testable by the native engine:` marker. One counter, one meaning.

### 5.8 mlkem — differential first

Feed population: **113 protocols / 131 links**, every one `mlkem768x25519plus.<native|random|xorpub>.0rtt.<key>`.

The `0rtt` mode is **inert on a fresh dial**: `client.go:113-121` takes the 0-RTT branch only when `time.Now().Before(i.Expire)`, and `i.Expire` is set only at `client.go:188-193` from a ticket a *previous* connection returned. A probe dials once per attempt, so the 1-RTT path always runs. Omission is also the safe direction: the reference server rejects the 0-RTT form when its own seconds range is 0 (`server.go:197-201`) and answers an unknown ticket with 1279–2279 bytes of noise (`server.go:212-221`).

The shipped e2e pair is `native.1rtt` client against `native.0.` server (`e2e/config.rs:70-94`) — a server that forbids 0-RTT — which is the correct pairing for the 1-RTT path.

So the differential is **base handshake only**, and the `0rtt`/`1rtt` split is not a discriminator:

- Build `thirdparty/Xray-core` HEAD (go 1.27.1 present; `main/main.go` exists).
- Run our client against it **and** against the pinned 26.3.27.
- Assert `native.1rtt` and `native.0rtt` behave **identically** on a fresh dial, pinning the above finding as a test rather than a note.

**Harness shape (decided): a second pinned peer.** `26.3.27` stays the suite's baseline — only the pq-enc row targets HEAD. Re-pinning the shared peer would re-baseline all 136 e2e rows (130 green + 6 ignored) and let an unrelated HEAD behaviour change surface as noise, while silently restating the suite's compatibility claim.

**Acceptance (narrowed):** the pq-enc row green against **the release users run**, with HEAD as a diagnostic only. If HEAD passes and 26.3.27 fails, that means our client implements a revision newer than the release — **not** that the pin is stale. A HEAD-only interop is not a capability claim worth shipping; if the release cannot be satisfied, the gate stays closed and that is the recorded outcome.

### 5.9 HTTP-transport divergence — measure first

Verified divergences from the reference, spanning three transports:

- **Host fallback.** `ws.rs:28` (`cfg.host.as_deref().unwrap_or(server_host)`), `httpupgrade.rs:33-36` and `xhttp.rs:75-79` all fall back to the **server address** (an IP for IP endpoints); xray goes `config.Host → ServerName → dest`. `ws.rs:213-219` pins the current behaviour with a test. Fires only when `host` is absent (2 httpupgrade + 3 xhttp rows measured).
- **Default headers.** xray injects a UA-conditional masquerade set via `utils.TryDefaultHeadersWith` — `websocket/config.go:27` (`"ws"`), `httpupgrade/dialer.go:99` (`"ws"`), `splithttp/config.go:60` (`"fetch"`), plus `reality/reality.go:226` (`"nav"`) and `tls/ech.go:240` (`"fetch"`); semantics at `common/utils/browser.go:266-286`. We call it nowhere.

Adding these headers changes what **1,026 links** (743 ws + 82 httpupgrade + 201 xhttp) put on the wire, against a measured CDN refusal cluster of ~24 rows. The change is therefore **measure-first**: A/B the `403`/`409`/`530` rows with and without the masquerade set (§5.4), and change the wire only for rows the measurement moves. An inconclusive result resolves to out-of-scope with a recorded negative result.

## 6. Ordered preconditions

Pinned at spec level because a plan tends to shuffle sequence, and both the falsifier and the A/B control depend on it.

| stage | must land | because |
|---|---|---|
| **M0** | `flow_cost` real `BatchProbeRunner` + pinned slice + **baseline results/s recorded** (command, slice ids, raw medians) | without a recorded "before", the §7 class-2 bar has no control and the falsifier cannot fire |
| **M1** | header A/B **control** run (today's wire) on the pinned `403`/`409`/`530` rows | a wire change made first destroys the control |
| **M2** | mlkem differential vs the HEAD-built peer and the pinned 26.3.27 | the fix depends on which side is wrong |
| M3 | forced-override proof of the latent build path (§5.5) | diagnostic; order-free |
| **then** | §5.2, §5.5, §5.6, §5.7 changes | — |
| **then** | M0 / M1 re-run for the after-numbers | — |

M0 is the only stage that blocks other scoped work, so it is the natural first plan task.

## 7. Acceptance

**Class 1 — static, offline, no network.** After the change, the count of links whose `security_fp` yields a capability refusal is **0**, and exactly **923** are labelled approximated (242 honoured unmarked). Proven by a unit test over the predicate plus one query, so it cannot be confounded by the dead-feed ratio.

**Class 2 — the falsifiable speed claim.** Median results/s at fixed concurrency improves by **≥1.5×** in `flow_cost` with the real runner, before vs after. **Below 1.5×, the speed stage is dropped** and items 1/3/4/6 still ship. The bar is a **same-machine relative** claim — the lab's rows are medians "on this machine" — and must never be quoted as an absolute.

**Class 3 — correctness, survives a flat curve.**
- *No path outlives the budget*: unit assertion that each clamped step is `≤ remaining` at point of use, plus an integration assertion that no attempt exceeds the budget.
- *No verdict from an approximated probe*: `reason_for` returns `None` for approximated evidence carrying any variant; existing purge tests keep passing for non-approximated evidence.
- *Class mis-attribution = 0*: the VLESS response-header EOF lands in `ProbeClass::Protocol`; `ProbeClass::Tls` is proven unreachable from a tunnel-payload error.
- *`config_invalid` mapping*: missing-pbk / malformed-sid evidence produces `config_invalid`; a genuine server-auth failure still produces `reality_fallback`.

**Class 4 — pass/fail.** The pq-enc e2e row green against the release peer, version recorded; and the removed-transport refusal proven by one forced `protocol_core_overrides` entry on a `type=http` link.

## 8. Verification mechanics

- **Harness**: `cargo test -p xray-tui --release --lib -- --ignored --nocapture flow_cost`, with `XRAY_TUI_MEASURE_DB=<copy of data.db>`. Slice ids and raw medians recorded in the spec's results section when M0/M1 run.
- **Pinned slice**: a fixed list of endpoint ids chosen once from a known-live subset, recorded by id. It decays as public endpoints churn; re-pinning is a recorded maintenance cost.
- **Not verified, stated as such**: the CDN/bot-protection hypothesis (needs live CDN-fronted servers; may be inconclusive at ~24 rows); whether the approximated hello helps or hurts in the field — class 1 proves the label exists, never that the approximation is good; the pinned slice's stability over time.

### 8.1 Results

**T3 — fingerprint policy, class-1 static check.** The predicate applies to every link where a fingerprint can exist: `security_type='none'` protocols carry **zero** `security_fp` rows (verified), so the scanned population is the whole applicable feed.

| snapshot | tls+reality links | approximated | honoured | previously refused | refusals remaining |
|---|---|---|---|---|---|
| 2026-09-22 session start | 9,067 | 923 | 242 | 1,165 | **0** |
| 2026-09-22 13:2x (re-measured) | 9,111 | 924 | 257 | 1,181 | **0** |

**The claim is invariant; the absolutes are snapshot-bound.** The feed grows while the app runs (subscription imports), so a durable figure must name its snapshot. What does not move: **no `security_fp` value reaches a refusal arm**, and every unrosterable id is marked approximated. Produced by the same rule the gate applies — no network, no run.

**Data-safety note, stated rather than left open.** The verification launches ran the TUI against a *copy* of `data.db` under a throwaway `XDG_CONFIG_HOME`; that isolation is **proven for the wrapper-launched session** (the copy's row count moved 7,584 → 7,811 while the wrapper ran). It is **not** proven for the first launch, which the harness ran as a bare binary path with no way to set the environment. The live database was in fact written during this session — endpoints 8,071 → 8,127, protocols 7,268 → 7,340, links 10,264 → 10,358, mtime 13:17:50 — and that pattern (new endpoints *and* protocols *and* links) cannot come from enrichment, which writes only `endpoint_ip`/`resolved_at`/country. It matches the TUI's 60-second subscription auto-update, i.e. an import. The user's feed was therefore mutated (additively, by a refresh) rather than left untouched, and the first launch's config target is unverified.

**Defect found and fixed while doing this:** the column geometry was hardcoded in **two** test literals (`vec![…, 24, 3, 6, 1, …]`, commented *"Real column geometry (117 cells) so the panel draws at its production width"*). Widening Test to 7 made production 118 while the copies still said 117 — and the tests passed, because the literal is a copy, so the assertion had silently stopped describing production. All copies and the `118 cells` comment updated. **Follow-up:** derive the test vectors from the `Column::new` widths (or a shared const) — two copies have already drifted once.

**T3 — the approximation marker, both surfaces (visual confirmation).** Captured from `target/debug/xray-tui` at 200×50 on the isolated copy:

```
 17 [🇯🇵 103.115.18.22:8443 ][    ]=>{ vless/tcp/reality }=>[ 331]~[—   —  ]
 18 [🇨🇿 103.119.19.201:2087][  🏳 ]=>{ vless/tcp/reality }=>[fast]~[—   —  ]
 16 [🇳🇱 103.112.71.83:443  ][    ]=>{ vless/tcp/reality }=>[ 71 ] [—   —  ]
    └ row 17 expanded: │● eb33561a … vless  tcp/reality~  331ms …│
```

The endpoint Test cell and the panel sub-row both carry `~`; rows whose representative link is roster-mapped or carries no fingerprint do not. Column alignment holds across the widened Test column (the Outbound `[` sits in the same cell on every row). Unit coverage: `approximation_marker_follows_the_representative_link` (a sibling's unrosterable `fp` must NOT mark the row) and `approximation_marker_composes_with_the_verdict`.

*Pending M0 / M1*: the baseline results/s, the derived budget value, and the M0/M1 after-numbers are recorded here, with the harness command, the pinned slice ids and the raw medians. This section is the acceptance evidence for §7 class 2; a plan that begins with an engine change has skipped it.

## 9. Evidence base

**The feed, not the engine, dominates.** Two oracles, both on a `curl --socks5-hostname` harness validated against a known-good control (`freedom` outbound → `HTTP 204`).

- **REALITY fallback — 710 rows (38 % of 1,882 real failures).** 16/16 sampled fallback configs (distinct `pbk` each) also fail under a real xray-core client. Our auth is separately e2e-proven interoperable against real xray 26.3.27 / sing-box 1.13.16 (`tests/vless.rs:88-107`, `e2e/variant.rs:223-296,344-370`). → stale/rotated `pbk`/`sid`, or a host that is not REALITY.
- **`type=http` — 144 rows.** The raw subscription links say `type=http` (verified by re-fetching `groups.url`), and xray-core removed that transport. sing-box 1.13.16 fails 8/8 on the same sample. No engine can serve them.
- **Fast timeouts — 4,712.** 4,120 are bare IPv4 literals: dead servers, not a multi-address artefact.
- **Parse layer is faithful** — `raw`→Tcp (715 links), `http|h2|https`→Http, `xhttp|splithttp`→XHttp, malformed `type=` values prefix-recovered (`proto_spec/common.rs:64-133`). No action.

**Feed-wide capability populations.** Fingerprint refusals 913 protocols / 1,165 links; mlkem 113 / 131; `type=http` 569 protocols; `quic` 53 links / 48 protocols (hysteria2 only, so sing-box).

**Throughput.** 1.81 real results/s at `real_ping_concurrency: 5` (code default 100); 1,083 real results in 598 s. 531 of 535 real-timeout rows had a sub-second fast latency, i.e. the server answered TCP instantly and the probe still blew its budget.

**Staleness.** 7,436 rows carry an error, 953 are purged, 22 ever recorded a real success — and `error_ttl_hours` defaults to `None`, so nothing expires.

## 10. Failure and recovery

- **Harness extension infeasible** (a real runner cannot be isolated from the batch's global state): class-2 acceptance drops to "not verified" and is recorded; correctness items still ship.
- **mlkem differential says the client is at fault**: item 2 returns to design; it is not quietly closed.
- **Header A/B inconclusive**: resolve to out-of-scope with a recorded negative result, not to a wire change.
- **Budget value absurd**: the p99 is taken over *successes*, so hangs are excluded by construction. If it still lands above a sane ceiling, the ceiling wins and the discrepancy is recorded.
- **Rollback is clean, and no path can lose data**: the budget value is a user-overridable config default; the approximation marker is derived, so removing the fallback restores today's refusal with no migration; the purge change is one function; the build-time refusal is one match arm. **No schema change anywhere in this spec.**

## 11. ADR signals

| signal | disposition |
|---|---|
| ADR 0006 — purge evidence and verdicts | doc note: two mapping refinements + the approximated-evidence rule. No variant, no CHECK change, no wipe. |
| ADR 0008 — batch-feed scaling | doc note: the counter split, and whether the budget clamp moves the measured flow cost. |
| **new ADR (proposed)** — *unhonourable capability is approximated and marked, never silently substituted* | durable policy with a real alternative (refuse vs approximate) and a trust boundary. Source refs: §5.1, §5.3, `security/fingerprint.rs`, `reality/mod.rs:158-201`. Expected baseline-sync question: which capability gaps are approximated vs refused, and who may change that list? |
| **new ADR (proposed)** — *one authoritative attempt budget; engine step limits are clamps* | runtime-boundary decision replacing five independent constants (`error.rs:174-181`). Source refs: §5.2, `probe.rs`, `ping_native.rs`. |

Both proposed ADRs are **signals only**; they become accepted architecture memory after the work executes.

## 12. Residual risks

1. The 1.5× bar is a same-machine relative claim, never portable.
2. Raising the budget increases hang slot-time ~9× (5 s → 45 s) and puts two tunnels per slot instead of one; it pulls against the bar and the two may partially cancel.
3. Approximation can be wrong in both directions on the CDN leg, contained only by the purge prohibition — a wrong answer stays re-testable but is still wrong.
4. The approximation count has no end date: `qq` is a new capture, not a roster lookup, and every new uTLS id re-opens the gap.
5. "We interoperate with xray" holds for the release only; a HEAD-only pass is a recorded narrower claim (§5.8).
6. The removed-transport fix is verified latent-only — one forced override, not a production path.
7. The pinned slice decays as public endpoints churn; re-pinning is recorded maintenance.
8. Class 1 proves the label exists, never that the approximation helps. Coverage is not quality.
