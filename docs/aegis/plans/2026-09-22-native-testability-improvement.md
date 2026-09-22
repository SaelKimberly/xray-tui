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

### T4 — budget authority: steps clamp to the attempt budget · `strict`

**Files**: `crates/xray-tui-native/src/error.rs`, `crates/xray-tui-native/src/probe.rs`, `crates/xray-tui/src/ops/ping_native.rs`

**Change**: `timeouts` constants become clamps against a caller-supplied deadline; the probe passes its budget down. Callers without a budget (the live tunnel) keep the constants.

**RED**: a test asserting every clamped step is `≤ remaining`, and that an attempt cannot exceed its budget.
**GREEN**: implement the clamp; set the budget default from T0's p99 of *successful* attempts.

**Budget basis — stated, because the naive form is unexecutable.** A p99 over successes cannot be read off a bounded slice at the measured ~1.3 % real-ok rate without order 10³ successes, which is the hours-long whole-feed run §4 excludes. So the basis is one of, in order:

1. **T0's p99 of successes**, when M0 reached its stated minimum success count (≥200) on the pinned slice at the harness-only concurrency — the intended path.
2. **T0's p99 over attempts with the failure class excluded** (hangs and hard failures removed by class, not by duration) — used when the success count falls short.
3. A curated known-live set built by a short fresh pass — used only if both above are unavailable.

Whichever applies is recorded in §8.1 with its sample count. **T4 must not land a guessed number**: if no basis is available, T4 ships the clamp structure only (a single authoritative budget with the shipped default left unchanged) and says so.

**Verify**: the clamp tests; the budget value recorded in §8.1 with its p99 basis.

### T5 — purge mapping + approximated evidence · `strict`

**Files**: `crates/xray-tui/src/ops/purge.rs`, `crates/xray-tui-native/src/error.rs`

**Change**: missing-pbk / malformed-sid evidence maps to `config_invalid`; `FailureEvidence` gains the approximated carrier; `reason_for` returns `None` for approximated evidence. No new `PurgeReason` variant.

**RED**: `reason_for` tests — config-defect evidence ⇒ `config_invalid`; genuine server-auth failure ⇒ `reality_fallback`; approximated evidence ⇒ `None` for every variant.
**GREEN**: implement the mapping and the carrier.

**Verify**: those tests, plus the existing purge tests unchanged.

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

### T8 — removed transport: build-time refusal + downgrade · `strict`

**Files**: `crates/xray-tui-proto/src/proto_spec/common.rs`, `crates/xray-tui-native/src/capability.rs`, `crates/xray-tui/src/ops/connect.rs`

**Change**: `inject_to` refuses `Http`/`Quic` for xray with a `SupportError`; the connect path downgrades such a link to native with a named `warn`.

**RED**: a build test asserting an xray build of an `Http`/`Quic` transport returns `BuildError::Support`, not a config xray-core would reject.
**GREEN**: implement the refusal and the downgrade.

**Verify**: that test plus T2's re-run showing a named downgrade.

### T9 — mlkem differential · `strict`

**Files**: `crates/xray-tui-native/tests/vless.rs`, `crates/xray-tui-native/src/e2e/config.rs`

**Change**: build `thirdparty/Xray-core` HEAD; run the pq-enc row against it and against the pinned 26.3.27; assert `native.1rtt` and `native.0rtt` behave identically on a fresh dial. Fix whatever the differential localizes.

**Verify**: the pq-enc row green against the release peer, version recorded. If only HEAD passes, the gate stays closed and that is the recorded outcome.

### T10 — after-numbers and the header decision

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
