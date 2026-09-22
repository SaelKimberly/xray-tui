# Native testability — implementation plan

- **Date**: 2026-09-22
- **Parent spec**: `docs/aegis/specs/2026-09-22-native-testability-improvement-design.md`
- **Status**: ready for execution
- **Execution route**: `inline` — the tasks share files (`ops/ping.rs`, `probe.rs`, `security/fingerprint.rs`) and are ordered by the spec's §6 preconditions, so coordination cost exceeds parallelism benefit.

## Goal

Make every persisted real-ping verdict an auditable function of the failure stage it came from, and raise the fraction of feed links that receive a trustworthy verdict — without changing any tuning default and without a schema change.

## Architecture

No new owner, no new artifact class, no new persistence. One derived predicate (approximation) replaces a refusal; one clamp replaces five independent step constants; three mapping edits replace taxonomy growth. The measurement authority is the existing perf lab (`ops/ping/flow_cost.rs`, ADR 0008) extended with the production runner it already has available.

## Tech Stack

Rust 2024, tokio, the in-process native engine (`xray-tui-native`), the TLS engine (`xray-tui-tls`), toasty/turso (untouched), `ops/ping/flow_cost.rs` as the measurement harness.

## Baseline / authority refs

- `docs/aegis/specs/2026-09-22-native-testability-improvement-design.md` (parent; acceptance in §7, sequence in §6)
- `docs/aegis/specs/2026-09-17-purge-reason-design.md` + ADR 0006
- `docs/aegis/specs/2026-09-17-batch-ping-pipeline-design.md` + ADR 0008
- `docs/aegis/specs/2026-09-14-native-ping-pipeline-design.md`
- `NATIVE_CORE.md`
- `plans/2026-09-16-ping-engine-tls-fixes.md`, `plans/2026-09-17-ping-run-analysis.md` — the recorded deferrals T6 re-opens

## Compatibility boundary

- **Unchanged**: the SQLite schema (tag 12), every `PurgeReason` variant, every tuning default, the `profile_stats` column set, the `TransportConfig`/`SecurityConfig` serde shapes.
- **Changed, user-visible**: the probe budget default (a correctness default — the one named exception to "no default change"), the fingerprint capability verdict for 1,165 links, and the class a failure is counted under.
- **Retained deliberately**: `NullRunner` stays in the lab (it is the fast, network-free baseline for flow/DB rows).

## TDD Route

- **Mode**: `auto` (no `~/.config/aegis/config.toml`, no `AEGIS_ACTIVATION_MODE`; no explicit user TDD request).
- **Decision**: `strict` for T3–T9 — each changes behaviour, a persisted verdict, or a wire contract, which is exactly the auto-route strict signal set. `light` for T0–T2 — instrumentation and diagnostics with an obvious focused check and no behaviour change.
- **Authority**: the recorded auto decision above.
- **Test posture**: unit tests next to code (repo convention); the ignored perf lab for measurements, run explicitly.
- **Verification**: per-task commands below; no project-wide suite mid-flight.

## Verification

```bash
cargo test -p xray-tui-native --lib
cargo test -p xray-tui-tls --lib
cargo test -p xray-tui --lib
cargo test -p xray-tui-proto --lib
cargo test -p xray-tui --release --lib -- --ignored --nocapture flow_cost   # with XRAY_TUI_MEASURE_DB
cargo clippy --workspace --all-targets
```

## Tasks

### T0 — M0: real runner in the perf lab, pinned slice, baseline · `light`

**Files**: `crates/xray-tui/src/ops/ping/flow_cost.rs`

**Change**: `batch_params` takes a runner choice; the lab gains an `EngineProbeRunner` row set over a pinned slice of endpoint ids loaded from `XRAY_TUI_MEASURE_DB`. Add one table row reporting median real-probe latency and results/s at a fixed concurrency.

**Slice selection — decided here, not at execution.** The live evidence is thin (22 rows ever recorded a real success, days old, none reproducing), and the two numbers have different sample needs:

- **results/s** counts *attempts*, so mostly-failing endpoints are fine and the slice can be drawn straight from the feed.
- **the p99 of successes** needs actual successes. At the measured ~1.3 % real-ok rate a 50-endpoint slice yields ~0–1 of them — no usable p99.

So M0 runs at a **harness-only raised concurrency** (a lab override, not a shipped default, so §4's non-goal is not breached) and is **sized to a stated minimum success count** (≥200) before the p99 is read. That same pass produces the concurrency curve the §7 falsifier needs, so the raise is not extra work. If the minimum is not reached on the pinned slice, M0 reports the shortfall and T4 takes its stated fallback basis — it does not guess.

**Compat**: none — `#[cfg(test)]`, ignored by default.

**Verify**: the harness command completes and prints the new rows. Record the baseline (command, slice ids, raw medians, success count) in the spec's §8.1.

**Stop**: if the production runner cannot be driven from the lab without a live `AppState`/event loop, record the blocker and fall back to the spec's §10 clause (class-2 acceptance becomes "not verified").

### T1 — M1: header A/B control run · `light`

**Files**: `crates/xray-tui/src/ops/ping/flow_cost.rs` (slice only)

**Change**: run T0's harness over the pinned CDN-refused slice with today's wire and record the **probe's failure class and text distribution** — *not* an HTTP status. The CDN 403/409 is a transport-handshake failure surfacing as `ProbeOutcome::Failed { class: Transport, text: "httpupgrade: expected 101, got 403" }`; the probe's own HTTP status is the *target's* (gstatic's 204), so a status distribution would collect the control from the wrong layer and the Q12-C A/B would have no usable control.

**Verify**: the control class/text distribution is recorded in §8.1. It is the only control the Q12-C A/B will have.

### T2 — M3: forced-override proof of the latent build path · `light`

**Files**: none (diagnostic)

**Change**: set one `protocol_core_overrides` entry forcing `xray` on a `type=http` link, attempt a connect, and record the observed failure. Restore the config afterwards.

**Verify**: the recorded observation — today an xray-core load error naming a removed feature; after T8, a named downgrade.

### T3 — fingerprint policy + derived marker · `strict`

**Files**: `crates/xray-tui-native/src/security/fingerprint.rs`, `crates/xray-tui-native/src/capability.rs`, `crates/xray-tui/src/ui/profiles.rs`

**Change**: one predicate — present, not `""`, not `unsafe`, and `parse_fingerprint_id` fails ⇒ approximated. `security_reason` stops refusing those ids; the row is probed with the engine default hello and labelled from the same predicate.

**RED**: a test asserting `("", unsafe, None)` are *not* approximated and `("qq", "android", "hellochrome_120")` are; and that the gate no longer returns `Some(reason)` for them.
**GREEN**: implement the predicate and the gate change.

**Verify**: class-1 static check — refusal count 0, approximated count 923 over the feed.

### T4 — budget authority: steps clamp to the attempt budget · **DEFERRED**

**Deferred 2026-09-22, before implementation.** The observable contract this task exists to provide is **already met and already tested**, so what remained was inner-constant hygiene with an unobservable effect.

`probe.rs:73-96` already wraps the WHOLE attempt in one caller-supplied deadline:

```rust
let attempt = async move {
    let tunnel = crate::connect(params).await?;
    fetch_over(tunnel, req).await
};
let response = Box::pin(tokio::time::timeout(req.timeout, attempt)).await …
```

and its own doc states the engine's per-step limits are *"defence in depth, **not the contract** — without this wrapper a 5 s setting let a probe spend 10 s in the dial alone and report `timeout on tcp dial (limit 10s)`"*. A test already pins it: `probe::tests::a_stalled_handshake_is_bounded_by_the_attempt_budget` (`probe.rs:235-239`), green in every run. M0b's `Timeout` bin pinned at **5,001 ms** is that same wrapper firing — the empirical confirmation.

What T4 would still add is making the inner constants caps rather than independent values. Under the current 5 s budget every clamp resolves to the same value the outer wrapper already imposes, so the change is unobservable — and a *partially* threaded budget is worse than none: some steps clamped and others on bare constants would make the relationship look explicit while being incomplete, and no test distinguishes that from a complete threading.

**Un-defer trigger**: land T4 in its own slice when a budget value **exceeding 10 s** exists — that is the only condition under which the inner clamps have a job. M0b showed no success-based p99 is obtainable on this feed, so that trigger is not currently reachable, and §5.2's rationale is satisfied by the existing wrapper meanwhile.

**Scope if un-deferred** (measured, so it is not re-discovered): ~23 files — `context.rs`, `probe.rs`, `inbound/*`, `transport/{tcp,quic,httpupgrade,http/conn}.rs`, `security/*` — with the budget carried on `LinkContext` (the per-link policy surface, already in scope at every engine site) rather than threaded by signature. Two sites are deliberate exemptions and a reviewer must confirm both: `inbound/*` (the local server's own handshake, not an attempt) and `outbound.rs::relay`'s `TUNNEL_READ` (*"bounds inactivity of the TUNNEL, not of one direction"*).

**The value decision is settled regardless**: the shipped 5 s default stays (basis 2's p99 is 3,251 ms; no success-based p99 exists to justify a raise).

**Budget basis — stated, because the naive form is unexecutable.** M0 measured it rather than leaving it a contingency: one whole-feed pass yields **75** real successes (1.67 %) and a sequential 300 s sample yielded **0 in 81 attempts**, so a p99 over successes needs the whole feed several times over. Basis 1 is therefore **unreachable on this feed** and the operative basis is:

1. ~~T0's p99 of successes, at ≥200~~ — unreachable: the whole feed yields 75.
2. **T0's p99 over attempts with the failure class excluded** (hangs and hard failures removed by class, not by duration) — the operative basis.
3. A curated known-live set built by a short fresh pass — only if (2) is unavailable.

Whichever applies is recorded in §8.1 with its sample count. **T4 must not land a guessed number**: if no basis is available, T4 ships the clamp structure only (a single authoritative budget with the shipped default left unchanged) and says so.

**Basis 2 is now measured, and it is a FLOOR.** Non-hang attempt spans over 67 samples: median 214 ms, p90 369 ms, **p99 3,251 ms**, max 4,007 ms. The `Timeout` bin (n=116) is pinned at 5,001 ms — the budget itself — which is why it is excluded: a deadline span says nothing about how long work takes.

Two consequences, both settled here rather than at execution:

- **The shipped 5 s default already clears basis 2's p99** with ~1.5× headroom, and no success-based p99 is obtainable on this feed (0 successes in 183 sample attempts; 21–75 over a whole pass). So **T4 lands the clamp structure and leaves the budget value unchanged** — the escape hatch above, not a guess. What T4 fixes is the relationship: the engine's 10 s steps become unreachable clamps instead of independent constants.
- **A success does strictly more work than any failure** (tunnel → target TCP + TLS → HTTP exchange → reverse), so a failure p99 can only under-estimate the span a success needs. The budget's *true* requirement stays unmeasured until a success-based sample exists, and §8.1 records it as such.

**M0's span figures are not an attempt duration.** The run's `(35..56,455 ms)` is the real **level's** start..settle span (inside `wall=56800 ms`), and the sequential pass's 3.7 s/attempt averages mostly fast failures — neither measures a single attempt, so neither is cited as one. T4's justification is code-level and needs no measurement: the budget is 5 s while the engine's steps are 10 s each plus a 30 s read (`error.rs:174-181`), so **no engine step deadline can be reached under the probe budget** — the outer budget always fires first, and a legitimate authorization needing more than 5 s is truncated by it rather than bounded by the step that should have governed. The clamp makes that relationship explicit instead of accidental.

**Verify**: the clamp tests; the budget value recorded in §8.1 with its p99 basis.

### T5 — purge mapping + the approximated rule · `strict` (half landed)

**Files**: `crates/xray-tui-native/src/security/{mod.rs,reality.rs}`, `crates/xray-tui/src/ops/purge.rs`, `crates/xray-tui/src/ops/ping.rs`

**Landed 2026-09-22 — the config-defect mapping.** Four sites created a malformed REALITY config as `NativeError::Reality`, and `evidence()` maps that variant to `FailureEvidence::RealityFallback` (`error.rs:75-76`) — so a broken config was permanently purged as *"the server is not REALITY / a possible MITM"*, a verdict about the peer when the peer was never reached. All four now raise `NativeError::Config` → `ConfigDefect` → `config_invalid`:

- `security/mod.rs` — missing `pbk`
- `security/reality.rs` — non-base64 `pbk`, `pbk` not 32 bytes, malformed `short_id` (two sites)

**`reason_for` needed no change**: `FailureEvidence::ConfigDefect => PurgeReason::ConfigInvalid` already existed and already fired (1 row). The defect was upstream, where the error was *created* — which is why this half is 4 one-line changes and not a taxonomy revision.

Two tests updated/added, both asserting **what the failure proves** (`err.evidence()`), not which variant carries it: `security::reality::tests::malformed_reality_material_proves_a_config_defect_not_a_peer_verdict` and the rewritten `security::tests::reality_connect_rejects_short_pbk`.

**Still open — the approximated rule (T5's second half).** `reason_for(evidence)` is pure and has no link, so the rule "no verdict from an approximated probe" needs the fp at the decision point. Settled design (§5.3): derive it from the link's `security_fp` with T3's predicate — no engine threading. The exact change sites, so it is not re-discovered:

- `reason_for(evidence)` → `reason_for(evidence, fp: Option<&str>)`, first check = approximation → `None`. It stays the module's single verdict owner, which is the property the module doc claims.
- **The fp input is the `security_fp` COLUMN on every path** — the two-accessor split an earlier draft recorded was *not* forced: the single-ping site has its own `Protocol` row as a live local, and `LoadedProtocol` gained an `fp` field captured at its one construction site (`row.security.fp`). So `reason_for` never reads the config, and "one predicate" holds over the input as well as over the predicate.
- **The lookup lives in ONE function, `BatchShared::purge_for`.** It had been duplicated verbatim at two sites, so a test at one would not have covered the other; extracting it also gives the fail-closed arm a home to be tested in.
- **Lazy:** called only inside the `Some(evidence)` branch — `stage_result` runs on the write-behind hot path (10.0 ms per 512-patch window) and a fast probe's evidence is always `None`, so an eager lookup would tax it for a value it never needs.
- **Fail-closed:** a cache miss returns `None`, not a verdict. Failing open would let an approximated probe earn exactly the permanent verdict the rule exists to prevent.
- Three production call sites, all in `ops/ping.rs`: the single real ping, `stage_result`, `emit_result` (the last two via `purge_for`).
- Tests: `an_approximated_probe_earns_no_verdict_at_all` and `no_fingerprint_requested_still_earns_a_verdict` in `purge.rs`; `a_protocol_cache_miss_earns_no_verdict_and_a_cached_fp_decides_it` in `ping.rs` (miss → `None`, cached approximated → `None`, cached honoured → the verdict); `the_security_fp_column_agrees_with_the_config_field` in `state.rs`, guarding the projection that produces the rule's only input.

### T6 — class precision · `strict`

**Files**: `crates/xray-tui-native/src/probe.rs`, `crates/xray-tui/src/ops/ping_native.rs`

**Change**: the class comes from the stage that failed. `probe.rs` stops mapping a tunnel-payload error to `NativeError::Tls`.

**RED**: the VLESS response-header EOF is asserted to land in `ProbeClass::Protocol`; `ProbeClass::Tls` asserted unreachable from a tunnel-payload error.
**GREEN**: implement the mapping change.

**Verify**: those tests. Carries the 151-row measurement as the re-open reason for the recorded deferral.

### T7 — counter split · `strict`

**Files**: `crates/xray-tui/src/ops/ping.rs`

**Change**: one counter for untestability, covering both the plan-time kind gate and the config-level refusal.

**RED**: a summary test asserting a config-level refusal increments `untestable` and does not appear in `real_fail[config]`.
**GREEN**: implement.

**Verify**: that test; the 09-21 shape (`untestable=0 … config=173`) is no longer producible.

### T8 — removed transport: build-time refusal · `strict` (landed)

**Landed 2026-09-22.** `validate_xray_transport` in `proto_spec/common.rs` refuses `Http`/`Quic` with `SupportError::Config`, called from the three `inject_xray` arms that emit a *variable* transport (`vless`, `vmess`, `trojan` — the others hardcode `Tcp`, except hysteria2 which sets `"hysteria"`). Same shape and placement as `validate_xray_reality`, so the build fails with a named reason instead of handing the core a config it refuses to load.

**The second half needed no code, and the reason is structural.** The plan said "the connect path downgrades such a link to native with a named `warn`" — but `capability::supported` already accepts `TransportConfig::Http(_)` for the native engine, and `resolve_runtime_core` (`ops/connect.rs:95-121`) sends every native-capable kind to native *before* any build. So a `type=http` vless/vmess/trojan link already goes native, and the refusal is reachable only when the user explicitly overrides the core to xray — which decision 20 handles by warning that the override was not honoured, not by silently overriding the user back. Adding a downgrade would have contradicted that decision for no gain.

**Tests**: `xray_build_refuses_the_removed_http_and_quic_transports` (all three spellings — `http`, `h2`, `quic` — refuse, the reason names the transport, and no partial config is left behind); `the_removed_transports_are_still_buildable_for_singbox` (the refusal is scoped to the core that removed them, not to the transport); and `the_removed_transports_emit_the_names_xray_refuses` in `common.rs`, which pins the coupling the validator exists for — `ws`/`httpupgrade`/`splithttp` were already pinned, these two were the unpinned pair, which is why nothing failed when xray-core removed them.

### T9 — mlkem differential · `strict` (landed — verdict: the client, not the pin)

**Landed 2026-09-22.** Built the HEAD peer (`thirdparty/Xray-core` → **Xray 26.7.28**, go 1.27.1) and added `vless_pq_enc_against_head` — an ignored test that resolves the SECOND pin and runs the same pq-enc case against it. `XRAY_VERSION` is untouched, so the suite's baseline and its 136 rows are unchanged.

**The differential's verdict: not a version delta.** Both revisions behave identically — `connect()` Ok, probe `status 0 body ""` on all 5 attempts, and no server-side error line at either. So the client's `mlkem768x25519plus` wire (or this case's server config) is wrong against real xray at both 26.3.27 and 26.7.28.

**That is the plan's stated branch, and it is recorded rather than papered over:** the pq-enc row stays ignored (its reason now names the differential), and the gate stays closed. Its acceptance — the row green against the release users run — is **not met**, and a HEAD-only pass would have been the narrower claim §5.8 refused.

**What T9 delivered despite that:** the leading hypothesis (stale pin) is eliminated, so the fix round starts from the client. The spec's §5.8 acceptance and this plan's trigger both now say that, instead of pointing at a re-pin that would have changed nothing.

### T10 — after-numbers and the header decision · **landed (negative result, reverted)**

**M0 reproduces:** three passes at the same harness-only concurrency — **79.19**, **83.94**, **84.55 results/s**, ~7 % spread, the last one *after* T5–T8, so none regressed the real level.

**The ≥1.5× bar has no candidate change in this spec.** The only task targeting per-attempt cost is T4, deferred on measurement; T3 was already in the M0 baseline and T5–T8 change bookkeeping, a class label and a build-time refusal. The bar is not met, and the reason is structural — recorded so it is not read as a tried-and-failed optimisation.

**Item 7's A/B is a measured negative, and the change is reverted.** Implemented xray's `utils.TryDefaultHeadersWith` set behind an env gate (so control and treatment shared one binary) and ran the pinned 40-link control twice:

| | control | treatment |
|---|---|---|
| `ok` | 0 | 0 |
| `v2rayhttp 400` | 6 | 2 |
| timeouts | 5 | 9 |

Not one row turned green, and four got slower (a `400` became a 4 s hang) — so the headers are *seen* but do not make the transport work. Reverted via `git checkout` rather than left behind a gate. Item 7 resolves to **out of scope with a recorded negative**, the spec's stated fallback; the `405` on our PUT (matching sing-box) plus the 404s already pointed at origin answers about a missing path/method rather than edge bot-protection.

**Files**: `crates/xray-tui/src/ops/ping/flow_cost.rs`, `NATIVE_CORE.md` (only if the wire changes)

**Change**: re-run M0/M1; compute the ratio against the T0 baseline. If `≥1.5×`, the speed stage is accepted. If the header A/B moved rows, change the wire; if inconclusive, record the negative result and leave item 7 out of scope.

**Verify**: §8.1 holds before, after, ratio, and the header verdict.

### T11 — owner-doc amendments and index

**Files**: the three amended specs, `NATIVE_CORE.md`, `docs/aegis/INDEX.md`

**Change**: land each amendment in the same task that landed its delta (T3/T5/T6 → native-ping spec; T5 → purge spec + ADR 0006 note; T7/T8 → batch spec + ADR 0008 note). ADR 0006/0008 get **doc notes**, not new variants. The two proposed ADRs stay signals until the work is executed.

## Repair / Retirement tracks

- **Repair**: T5's mapping fix repairs a verdict that mislabels config defects as MITM; T6's repairs a class that mislabels protocol EOFs as TLS.
- **Retirement**: the fingerprint refusal path for `""`/`unsafe`/unrosterable ids is **retired**, not shimmed — no compatibility branch retains it. `NullRunner` is **retained** deliberately (network-free baseline rows); retirement trigger: when a network-free runner is no longer needed to keep the flow/DB rows comparable.
- **Follow-up carried from T3 (not a task here)**: two test literals copy the production column geometry (`vec![1u16, 2, 7, …, 3, 7, 1, 16, 7, 1]`) with a comment claiming it is "real column geometry". Both drifted silently when T3 widened the Test column — the tests passed while asserting a geometry production no longer used. Derive them from the `Column::new` widths or a shared const before the next width change.

## Risks

Per the spec's §12, plus execution risks: T0 may not be drivable in-lab (§10 fallback recorded); T9 may conclude the gate stays closed; T10's header A/B may be inconclusive at ~24 rows.

## Self-review

Scope coverage: all seven spec items map to tasks (1→T3, 2→T9, 3→T6, 4→T5, 5→T4, 6→T8+T7, 7→T1/T10). No placeholders. Owner/type consistency: no new type outside the approximation predicate and the evidence carrier. Minimum-change boundary: one predicate, one clamp, three mappings, one build arm. Compatibility: schema and defaults untouched except the named budget exception. Verification: each task names a command or an assertion. Sequence: T0/T1 precede every engine change, per the spec's §6.
