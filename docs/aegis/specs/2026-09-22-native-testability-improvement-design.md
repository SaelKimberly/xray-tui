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

`security_fp` is already in the page projection (`crates/xray-tui-db/src/profiles_query.rs:616-617`), so the label needs no new column, no projection change, no schema-tag bump and no wipe. One predicate feeds three consumers — `security::wrap` (where the substitution happens), the row label, and the purge rule — and all three read the same **input** too, the `security_fp` column: the batch captures it into `LoadedProtocol` at load precisely so the purge path never has to reach for the config (§5.3). The capability GATE is not among them: after T3 it refuses no fingerprint id at all. The discipline `endpoint_rank::dns_unresolved(row)` already follows — one predicate, read from the row, never the cache.

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
| **any failure from an approximated probe** | a verdict was possible | **no verdict** — the link's `security_fp` is an approximation, so `reason_for` returns `None` |

The last row is expressed as a **rule read from the link's `security_fp` column on every path** — decided by the **same predicate T3 introduced** (`security::fingerprint::resolve_fingerprint`), so the gate, the row label and the purge rule cannot disagree over the predicate *or* over the input.

The column is reachable at all three sites, so there is no second accessor:

| site | where the column comes from |
|---|---|
| the single real ping (`ops/ping.rs`) | its own `Protocol` row, a live local beside the config |
| `stage_result` / `emit_result` | `LoadedProtocol.fp`, captured at the struct's ONE construction site from `row.security.fp` |

`LoadedProtocol` gained the field for exactly this reason: the two `self.protocols` sites would otherwise have had to reach the config (`config.security().fp()`), which is a *different* input, and "one predicate" would then have held over the predicate but not over the input. Capturing the column once at load is cheaper than that traversal and strictly stronger — and it means §5.1's original *"the gate, the label and the purge rule cannot disagree"* holds without qualification.

The projection that produces the column (`state::security_embed`) is still guarded by `the_security_fp_column_agrees_with_the_config_field`, now justified as guarding the rule's **only** input rather than an agreement between two.

**Lazy and fail-closed**, and both live in one function (`BatchShared::purge_for`, extracted because the closure had been duplicated verbatim at two sites, where a test at one would not have covered the other):
- *Lazy*: the lookup runs only inside the `Some(evidence)` branch — a fast probe's evidence is always `None`, and this is the write-behind hot path (10.0 ms per 512-patch window).
- *Fail-closed*: a cache **miss** returns `None`, not a verdict. Failing open would let an approximated probe earn exactly the permanent verdict the rule exists to prevent, and Purgatory is the one output that is not cheaply reversible.

ADR 0006's premise holds — this is a mapping refinement inside `reason_for`, the one function that decides verdicts. No new variant, no CHECK change, no schema bump, no wipe.

Two reasons this beats the evidence-carrier form the earlier draft implied:

- **Blast radius.** An evidence carrier has to travel from where the flag is produced (`security::wrap`'s `resolve_fingerprint(...).1`, per connection) up through `ProbeFailure` → `FailureEvidence` → `reason_for` — an engine-wide threading job on the very surface T4 was just deferred for, which would have made T5 depend on engine work instead of standing alone.
- **Coverage.** Deriving it at purge time applies to failures **already persisted**, not only to ones probed after this change. The carrier would have exempted every historical verdict from the rule it exists to impose.

It is also stable: `fp` participates in identity, so a changed `fp` is a different `Protocol` row — never a silent reclassification of an old verdict. And it costs nothing to read: `security_fp` is a plain `String` column already on every page row (`profiles_query.rs:616-617`).

ADR 0006's premise holds — this is a mapping refinement inside `reason_for`, the one function that decides verdicts. No new variant, no CHECK change, no schema bump, no wipe.

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
- Assert `native.1rtt` and `native.0rtt` behave identically on a fresh dial — **as a STRUCTURAL test, not byte equality** (the first flight is randomised: a per-connection IV and a fresh ML-KEM encapsulation, so two connects can never match). Vehicle: `mlkem::tests::a_fresh_dial_never_takes_the_zero_rtt_path`, which runs the hermetic fake server for a `1rtt` and a `0rtt` config alike and asserts the sealed first-flight length is the 1-RTT pfs form (`PFS_EXCHANGE_LEN - 18`) rather than the ticket form (`32 + TAG_LEN`). That is exactly the finding the reference reading gave, and a future change that lets the mode affect the first flight fails loudly.

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

**Class 4 — pass/fail.**
- *mlkem:* the pq-enc e2e row green against the release peer, version recorded. **Not met, and recorded as such:** the differential localized the fault to our client (§8.1), so the row stays ignored and the gate stays closed. This is §5.8's stated branch, not a dropped criterion.
- *Removed transport:* proven by one forced `protocol_core_overrides` entry on a `type=http` link. **Met at the build level** — the refusal is asserted for all three spellings (`http`/`h2`/`quic`) with no partial config left behind — and the *connect-path* half needed no code: `resolve_runtime_core` already sends native-capable kinds to native before any build, so the refusal is reachable only through an explicit override, which decision 20 answers by warning.

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

**M0 — the real level's network baseline (measured 2026-09-22).** First run of the lab's production-runner pass:

```
cargo test -p xray-tui --release --lib -- --ignored --nocapture flow_cost_network
XRAY_TUI_MEASURE_DB=<copy of data.db>
```

Slice: the whole applicable feed in **ID order** (10,358 links over 8,127 endpoints, 7,340 distinct protocols; page offsets 0..8,200). Harness-only knobs: **concurrency 256**, budget 5 s. Everything else production — `dedup_endpoints` OFF, `real_phase` true.

```
links=10358 plan=20 ms untestable=0 queue-full=0 deferred=0
fast ok=4498 hard-fail=5860 [timeout=4985 dns=196 refused=639 no-route=35 unreachable=5] (11..42404 ms)
real ok=75  failed=4423 [timeout=2172 dial=7 tls=985 reality=825 transport=291 config=143]
skipped-unreachable=5860 (35..56455 ms) | wall=56800 ms flushes=21 staged-left=0
real results 4498 in 56.8 s = 79.19 results/s
```

**The baseline the falsifier reads: 79.19 results/s at concurrency 256**, against **1.81 results/s at concurrency 5** in the user's 09-21 run — a **43.7×** ratio, consistent with the 51× concurrency ratio minus overhead. This is the harness-only concurrency effect, not an engine improvement; the after-run must be compared at the same 256.

**Two findings this run produced that change T4:**

1. **The engine's step deadlines are unreachable under the probe's budget — by construction, not by measurement.** The budget is 5 s; the engine's steps are `DIAL`/`TRANSPORT`/`SECURITY`/`PROTOCOL` at 10 s each plus a 30 s `TUNNEL_READ` (`error.rs:174-181`). The outer budget therefore always fires first, so no engine step deadline is ever reached on this path, and any authorization needing more than 5 s is truncated by the outer budget rather than bounded by the step that should have governed it. (The run's `(35..56,455 ms)` is the real **level's** start..settle span — inside `wall=56800 ms` — and measures no single attempt; the sequential pass's 3.7 s/attempt is an average over mostly fast failures, so neither is cited as an attempt duration.)
2. **The ≥200-success minimum is unreachable on this feed.** One whole-feed pass yields **75** real successes (1.67 %), and a sequential 300 s sample yielded **0 successes in 81 attempts**. So T4's basis 1 cannot be met at any practical sample size, and **basis 2 — the p99 over attempts with failures excluded by class — is the operative one.** That is now a measured fact rather than a contingency.

Fresh class distribution at scale, for reference: timeout 49.1 %, tls 22.3 %, **reality 18.6 %**, transport 6.6 %, config 3.2 %.

**M1 — the transport control (measured 2026-09-22).** 40 pinned protocol ids whose links failed with a transport error on the `http`/`httpupgrade`/`x_http` transports, probed per-attempt on today's wire (the control the Q12-C A/B needs):

```
cargo test -p xray-tui --release --lib -- --ignored --nocapture flow_cost_transport_control
XRAY_TUI_MEASURE_DB=<copy> XRAY_TUI_MEASURE_PROTO_IDS=<40 ids> XRAY_TUI_MEASURE_BUDGET_SECS=4
ok=0 of 40 — distribution:
  14  Transport: v2rayhttp: expected 200, got 404 Not Found
   6  Transport: v2rayhttp: expected 200, got 400 Bad Request
   4  Transport: v2rayhttp: expected 200, got 405 Method Not Allowed
   7  Timeout:   timeout on probe attempt (limit 4s)
   2  Transport: httpupgrade: expected 101, got 400 Bad Request
   2  Transport: httpupgrade: expected 101, got 403 Forbidden
   1  Transport: v2rayhttp: expected 200, got 403 Forbidden
   1  Transport: v2rayhttp: expected 200, got 410 Gone
   1  Transport: httpupgrade: expected 101, got 404 Not Found
   1  Dial:      Connection refused
   1  Tls:       I/O error: early eof
```

**The A/B's candidate population is the 31 HTTP-layer refusals**, not the 40: the 7 timeouts, 1 dial refusal and 1 TLS EOF never reached an HTTP response, so no request-header change can move them. `405 Method Not Allowed` (4 rows) is notable — our method defaults to **PUT** (`transport/v2rayhttp.rs:52-56`, matching sing-box), so a 405 on PUT means the server does not accept it at that path; since sing-box also failed on the earlier sample, the method is not the discriminator and item 7's remaining hypothesis is the **default request headers**, which is what the A/B will test.

**M3 — the removed-transport path, proven deterministically (measured 2026-09-22).** No TUI connect needed: the claim is that xray-core *refuses* the config we emit, and that is decidable offline with the pinned binary. Three minimal outbounds, `xray run -c`:

```
network: "http"  -> Failed to start: ... common/errors: The feature HTTP transport (without header
                    padding, etc.) has been removed and migrated to XHTTP stream-one H2 & H3.
network: "h2"    -> same (HTTP transport removed)
network: "quic"  -> Failed to start: ... The feature QUIC transport (without web service, etc.) has
                    been removed and migrated to XHTTP stream-one H3.
```

A **fatal** config-load failure, not a warning: `PrintRemovedFeatureError` returns an error (`thirdparty/Xray-core/common/errors/feature_errors.go:25-27`).

**And our builder emits exactly those strings** — `to_xray_stream_settings` remaps `XHttp` → `"splithttp"` and sends everything else through `transport.type_str()` (`proto_spec/common.rs:935-950`), so `TransportConfig::Http` → `"http"` and `TransportConfig::Quic` → `"quic"`.

**The sharper finding:** the emitted network IS pinned by tests for `ws` (`common.rs:1378`), `httpupgrade` (`:1411`) and `splithttp` (`:1399`, with the comment *"xray-core only recognizes \"splithttp\" as the network name"*) — and **not for `http` or `quic`**, the only two whose emitted name xray-core has removed. The two broken cases are precisely the two without a pin, so nothing failed when xray-core removed them. T8's RED test closes that by asserting the build refuses them.

**Path status: latent, not live.** `config.json` carries no `protocol_core_overrides`, and `routing_rules` and `dns_settings` are empty tables, so decision 20 sends every link to the native attempt first and `capability.rs::transport_reason` calls `TransportConfig::Http(_)` supported (`=> None`) — the xray builder is never reached for a `type=http` link today. One routing rule or one override away from firing.

**M0b — basis 2: the non-hang attempt spans (measured 2026-09-22).** Same slice, `XRAY_TUI_MEASURE_SAMPLE_DEADLINE_SECS=600`, sequential per-attempt probes. Failure spans are binned by class — `ProbeFailure` carries class + text only, so the sample times each attempt itself:

```
per-attempt sample: 0 successes, 183 failures
Timeout  (hang):     n=116  median 5001 ms  p90 5001 ms  max 5002 ms
Dial     (counts):   n= 26  median   32 ms  p90   99 ms  max 4007 ms
Tls      (counts):   n= 18  median  289 ms  p90  970 ms  max 3251 ms
Reality  (counts):   n= 21  median  222 ms  p90  231 ms  max  405 ms
Transport(counts):   n=  2  median  122 ms              max  214 ms
basis 2 (non-hang, n=67): median 214 ms | p90 369 ms | p99 3251 ms | max 4007 ms
```

Three readings:

1. **The `Timeout` bin is pinned at the budget** — n=116, median = p90 = max ≈ 5001 ms. So the outer 5 s budget *is* enforced, and the hang class is purely deadline-shaped: it carries no information about how long work takes, which is why it is excluded.
2. **Non-hang work is fast**: median 214 ms, p90 369 ms, p99 3,251 ms. Real work that fails completes in a few hundred ms, with a ~3–4 s tail.
3. **Basis 2 is a FLOOR, not a ceiling.** A *successful* attempt does strictly more work than any failure — it completes the tunnel, then the target's TCP + TLS, then the HTTP exchange, then the reverse. With **0 successes** in the sample, the span a success needs is still unmeasured; the failure p99 can only under-estimate it.

**Consequence for T4.** The shipped 5 s default already clears basis 2's p99 (3,251 ms) with ~1.5× headroom, and no success-based p99 is obtainable on this feed. So T4 lands the **clamp structure and leaves the budget value unchanged**, recording that the value awaits a success-based sample — the plan's stated escape hatch rather than a guess. What T4 fixes is the relationship (the engine's 10 s steps become unreachable clamps instead of independent constants), not the number.

**Run-to-run variance, for reading the falsifier:** two identical batch passes gave **79.19** and **83.94 results/s** (≈6 % apart) but **75** and **21** real successes — the rate is stable because failures dominate; the success *count* is not, so no acceptance may rest on it.

**T9 — the mlkem differential (measured 2026-09-22).**

*First pass — our client against both servers.* Built a SECOND peer from `thirdparty/Xray-core` HEAD (**Xray 26.7.28**, go 1.27.1) and ran the pq-enc case against both:

| server | our client |
|---|---|
| pinned **26.3.27** | `connect()` Ok, probe reads nothing, no server-side error |
| HEAD **26.7.28** | `connect()` Ok, `probe status 0 body ""` 5/5, **no server-side error** |

Identical at both revisions — so the version-delta hypothesis was dead. But that leaves two readings: our client is broken, or the pairing cannot work at all.

*Second pass — the reference client against the reference server (the sharper A/B).* Both ends are real xray, and the keypair comes from the reference's own generator (`xray vlessenc`), so nothing in the experiment is ours:

| client | server | result |
|---|---|---|
| pinned 26.3.27 | pinned 26.3.27 | **HTTP 200** in 86 ms (control — validates the harness) |
| **HEAD 26.7.28** | pinned 26.3.27 | **HTTP 200** in 107 ms |

**The reference client works in every pairing; ours works in none.** So the older server *does* accept the newer client wire, the wire is not the problem, and our `mlkem768x25519plus` implementation is the broken side — an interop bug in our sealing/padding/framing, not a pin and not a version gap.

*Third pass — the hedge's concrete candidate, eliminated.* The verdict above still allowed one alternative: *our client's wire **or this case's server config***, and that alternative had a concrete form — the case's keypair comes from `e2e/config.rs::mlkem_enc_pair`, which derives the ML-KEM keypair from a 64-byte seed via OUR `Mlkem768::keypair_from_seed`, while the server's decryption string carries that seed for Go's `mlkem.NewDecapsulationKey768` to expand. If our expansion differed, the client's `ek` would target a key the server never holds, and the symptom would be exactly this one.

So the row was run again with a pair the **reference itself generated** (`xray vlessenc`), supplied through a new `CaseSpec::with_pq_enc_pair`:

| client | pair | server | result |
|---|---|---|---|
| pinned 26.3.27 | `vlessenc` | pinned 26.3.27 | **HTTP 200** (pass 2's control) |
| HEAD 26.7.28 | `vlessenc` | pinned 26.3.27 | **HTTP 200** (pass 2) |
| ours | `vlessenc` | pinned 26.3.27 | **fails** — `probe status 0 body ""`, 5/5 |

The pair and the server are therefore both known-good, and our client fails with them anyway. **The fault is our client's wire alone** — not the test's key generator, not the pin, not the version delta. (Useful by-product: our parser *accepts* the reference's shape, which is one 1184-byte `ek` segment with no X25519 public half — so the divergence is in the sealing/framing, not in the parse.) That is the bounded target the fix round needed: capture the reference client's first flight for a fixed pair and diff it against ours.

**Acceptance, per §5.8's stated branch:** the pq-enc row **stays ignored**, its reason now naming both passes; the gate stays closed. A HEAD-only pass would have been a narrower claim than the suite's, and this is the other branch — *"both fail → the client is at fault"* — now sharpened to *"the reference works everywhere, so the fault is ours"*.

**Harness shape, as §5.8 decided:** a **second** pin (`XRAY_HEAD_VERSION = "26.7.28"` + `XRAY_TUI_CORE_HEAD_BIN_DIR`), reached through `CoreUnderTest::resolve_from(kind, version, dir_env)`. `XRAY_VERSION` is untouched, so the suite's 136 rows keep their baseline. The sharper A/B itself was a throwaway script (`/tmp/xr_logs/ab_mlkem.py`, two configs + curl through a socks inbound); its method is recorded here because the in-suite row cannot express it.

**T10 — the after-run, and item 7's A/B (measured 2026-09-22).**

**M0 reproduces.** Three passes of the same slice at the same harness-only concurrency: **79.19**, **83.94**, **84.55 results/s** — a ~7 % spread across three runs, and the last one *after* T5/T6/T7/T8 landed, so none of them regressed the real level. That stability is what the falsifier needs; the samples are the baseline.

**The ≥1.5× bar has no candidate change in this spec.** The only task that targeted per-attempt cost is T4, and it was deferred on measurement (the outer budget already binds, so its observable effect is nil). T3 was already in the M0 baseline; T5–T8 change bookkeeping, a class label, and a build-time refusal. So the bar is **not met**, and the reason is structural rather than a failed optimisation — recorded so a later reader does not read it as the engine having been tried and found wanting.

**Item 7 — the header A/B: a measured NEGATIVE, and the change is reverted.** Implemented xray's `utils.TryDefaultHeadersWith` set (chrome identity + `Sec-Fetch-*` + `Cache-Control`/`Pragma`/`Accept`, per variant `ws`/`fetch`) behind an env gate so control and treatment shared one binary, then ran the pinned 40-link control twice:

| | control (no headers) | treatment (headers) |
|---|---|---|
| `ok` | **0** | **0** |
| `v2rayhttp 400 Bad Request` | 6 | **2** |
| timeouts | 5 | **9** |
| total HTTP-layer refusals | 33 | 29 |

**Not one row turned green**, and four got *slower* — a `400` became a 4 s hang, so the headers ARE seen (the server's answer changed) but they do not make the transport work. The hypothesis "the CDN refuses us for lacking browser defaults" is falsified as a fix. The change was reverted (`git checkout`), not left in behind a gate: a wire change that fixes nothing and slows four rows is not worth carrying.

**Item 7 resolves to out-of-scope with a recorded negative**, which is the spec's stated fallback for an inconclusive-or-negative A/B. The `405 Method Not Allowed` on our PUT (matching sing-box) plus the 404s already pointed the same way: these look like ORIGIN answers about a path/method that does not exist there, not edge bot-protection — so no request-header set would help.

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
