# Native real-ping pipeline — implementation plan

Goal: real ping (single + batch phase 2) executes on the in-process native engine
(`xray_tui_native::connect`), the subprocess probe arm is deleted, and rows the native engine
cannot serve fast-fail to a persisted `[real]` marker (`ProfileErr::Real` + reason text).

Architecture: `ops/ping.rs` (orchestration) → `ops/ping_native.rs` (probe policy) →
`xray_tui_native::probe` (tunnel + target TLS + one HTTP/1.1 request) →
`xray_tui_native::capability::support_reason` (the single testability authority).

Tech stack: Rust 2024, tokio, hyper 1.11 (`client`+`http1`, already a dep of `xray-tui-native`),
hyper-util `TokioIo`, `http-body-util` (`Limited`, ungated), `xray-tui-tls`
(`client::connect` + `verify::WebPkiVerifier::webpki_roots()`), `reqwest::Url` for URL parsing in
the TUI crate.

Baseline / authority refs: `docs/aegis/specs/2026-09-14-native-ping-pipeline-design.md` (this
plan's spec), decision 15 (result keying), decision 16 (tiers + labels), decision 20 (native is
runtime-only), decision 22 (write-behind RESULT group), decision 4 (schema tag = wipe),
`docs/native-core-integration.md` D3/D6.

Compatibility boundary: no DB/schema change; persisted `core_type` stays `{Xray, SingBox}`;
`(endpoint_id, protocol_id)` keying, tier law, `[name] > [real] > [fast]` precedence,
RESULT/TRAFFIC groups, batch serialization and `dedup_endpoints` semantics all preserved.

TDD Route:
- Mode: off
- Decision: skipped
- Strict authority: not applicable
- Strict signals: none recorded (no explicit user/project strict request)
- Light eligibility: n/a
- TDD-fit exception: n/a
- Test posture: post-change regression (new modules ship with unit tests; the probe's network
  behaviour is proven by the required real-runtime smoke)
- Reason: project default is off and the user did not request TDD
- Verification: focused `cargo test -p <crate> <filter>` per task, workspace suite at the end

Verification: `cargo test`, `cargo clippy --workspace --all-targets`, `cargo fmt --check`,
`just quality-gate code`, plus the real-runtime smoke (`pgrep -c xray` / `-c sing-box` = 0 during
a real batch, latency + exit IP observed, an unsupported row marked `[real]`, "Remove Bad
Servers" keeps it).

Change Necessity:
- User-visible need: real ping must work without core binaries and must not spawn processes;
  rows the native engine cannot serve must say so.
- No-change / non-code option: none — the subprocess dependency is code, not config.
- Why code change is necessary: the probe engine itself is being replaced.
- Minimum change boundary: `xray-tui-native` (capability reasons + probe primitive),
  `crates/xray-tui/src/ops/{ping.rs,ping_native.rs}`, `ui/profiles.rs`, and the deletion set.
- Decision: code-change

Existence Check:
- Proposed new surface: `xray_tui_native::probe` (fetch a URL through a tunnel).
- Existing owner / reuse candidate: `transport/xhttp.rs` already owns hyper-over-stream, but it
  is xray's protocol framing, not a general request helper; `speed_test::real_ping` needs a local
  SOCKS proxy that no longer exists.
- Why existing surface is insufficient: nothing in the tree issues an HTTP request over an
  arbitrary tunnel with target TLS.
- Creation proof: one module, no new dependency (hyper/hyper-util/http-body-util/tls/tokio are
  present), used by both the ping probe and (later) any other through-tunnel fetch.
- Entropy / retirement impact: net negative — ~1.5k lines of subprocess probe code deleted.
- Decision: add-with-proof

Plan-Time Complexity Check:
- Target files: `ops/ping.rs` (1816 lines) — the only at-risk file.
- Existing size / shape signals: already over budget; every batch concern lives there.
- Owner fit: orchestration stays; the probe policy leaves to `ops/ping_native.rs`.
- Add-in-place risk: high if the probe code is inlined.
- Better file boundary: new `ops/ping_native.rs` + new `xray-tui-native/src/probe.rs`.
- Recommendation: extract helper + add owner file; `ops/ping.rs` shrinks.

## Task batch A (independent, delegated)

### T1 — `capability::support_reason` (native crate)

Files: `crates/xray-tui-native/src/capability.rs` (modify).

Why: the connect downgrade, the ping gate and the persisted marker must agree; today the
predicate returns a bare `bool` and every reason is lost.

Change Necessity: the marker text needs the reason; a bool cannot carry it.

Change:
1. Add `pub fn support_reason(kind: ProtocolKind, config: &ProtocolConfig) -> Option<&'static str>`
   and make `pub fn supported(kind, config) -> bool { support_reason(kind, config).is_none() }`.
2. Refactor each existing early-out (read the bodies at `capability.rs:87-330` first) to return
   the reason at that point. Keep every predicate byte-identical in meaning; the existing
   `#[cfg(test)] mod tests` verdicts must stay green unchanged.
3. Reason strings (exact):
   - kind: `"no native implementation for this protocol kind"`
   - kind/config mismatch: `"protocol kind and config type do not match"`
   - fingerprint: `"unparseable TLS fingerprint id"`
   - VLESS encryption: `"vless account encryption is not implemented"`
   - VLESS flow: `"vless flow is not implemented"`
   - VMess security: `"legacy vmess payload security is not implemented"`
   - VMess alter_id: `"vmess alter_id is not implemented"`
   - transport: `"transport is not implemented"`
   - mKCP: `"mKCP seed or header_type is not implemented"`
   - SS method: `"shadowsocks method is not implemented"`
   - SS family: `"shadowsocks method family does not match the protocol kind"`
   - SS plugin: `"shadowsocks SIP003 plugin is not implemented"`
   - SS PSK: `"shadowsocks 2022 password key is malformed"`
4. New test: over a table of configs (one per refusal class + one accepted row per kind),
   assert `support_reason(kind, cfg).is_none() == supported(kind, cfg)` (anti-drift).

Verification: `cargo test -p xray-tui-native capability` (all pre-existing + the new parity test).

### T2 — `probe` primitive (native crate)

Files: `crates/xray-tui-native/src/probe.rs` (create), `crates/xray-tui-native/src/lib.rs`
(add `pub mod probe;` after `pub mod protocol;`).

Why: the real-ping probe needs "one HTTP request through a tunnel" with target TLS.

Change Necessity: no such helper exists (see Existence Check).

Initialization: none (leaks handle themselves).

Verification: `cargo test -p xray-tui-native --lib probe`.

## Task batch B (coupled, inline, single file + UI)

### T3 — `ops/ping_native.rs` (TUI crate)

Files: `crates/xray-tui/src/ops/ping_native.rs` (create), `crates/xray-tui/src/ops/mod.rs`
(register the module).

Why: probe policy (URL, retries, latency span, exit IP) must not live in the batch file.

Verification: `cargo test -p xray-tui ping_native`.

### T4 — `ops/ping.rs`: gate, marker, runner, phase-1 bound

Files: `crates/xray-tui/src/ops/ping.rs` (modify), `crates/xray-tui/src/state.rs` (drop
`core_pool`).

Why: the probe engine changes and untestable rows must fail fast.

Verification: `cargo test -p xray-tui ping` (the existing batch tests plus the new gate tests).

### T5 — `[real]` precedence + removal guard

Files: `crates/xray-tui/src/ui/profiles.rs` (modify), `crates/xray-tui/src/ops/ping.rs`
(guard), `crates/xray-tui/src/ops/profiles.rs` if the sweep is invoked there.

Why: without precedence the marker is invisible; without the guard "Remove Bad Servers"
deletes every unsupported profile.

Verification: `cargo test -p xray-tui profiles` + `cargo test -p xray-tui ping`.

### T6 — retire the subprocess probe arm

Files: delete `crates/xray-tui-core/src/ping/real/{mod,pool,xray,singbox}.rs`;
`crates/xray-tui-core/src/ping/mod.rs` (drop `mod real;` + its re-exports);
`crates/xray-tui-core/src/lib.rs` (drop the ping re-export list entries);
`crates/xray-tui-core/src/speed_test.rs` (delete `real_ping`, `wait_for_socks5`,
`RealPingResult`, and the client-cache flag only it used);
`crates/xray-tui-core/src/config_builder/mod.rs` + `xray.rs`/`singbox.rs` (`build_multi`);
`crates/xray-tui/src/ops/ping.rs` (delete `get_or_create_pool`, the pool imports);
`crates/xray-tui/src/state.rs` (the `core_pool` field, its init and the `CorePool` import).

Verification: `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets`.

### T7 — docs

Files: `AGENTS.md` (decisions 15/16/20), `TUI_MANUAL.md` (test-label section),
`docs/native-core-integration.md` (§6, D6, work item 7, out-of-scope bullet),
`docs/aegis/adr/0004-native-real-ping.md` (create), `docs/aegis/INDEX.md`.

Verification: read-back of the amended sections (no stale "always subprocess" claim left).

### T8 — end-to-end verification

Files: none.

Verification: the spec's §Verification list, including the required real-runtime smoke.

## Risks / retirement

- Divergence for native-capable rows that would connect via a subprocess (routing/DNS present):
  documented in the spec; native is the default connect path.
- Untestable rows: tier 3 (below untested) and TTL-swept — documented.
- Retirement: the whole subprocess probe arm; no fallback path is kept (spec D5).
- ADR 0004 records the D6 reversal after the work lands.