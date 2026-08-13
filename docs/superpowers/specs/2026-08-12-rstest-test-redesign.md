# Design: rstest 0.26.1 test redesign for xray-tui-tls + xray-tui-native

Date: 2026-08-12
Branch: `native-core-stub`
Scope: `crates/xray-tui-tls` + `crates/xray-tui-native` only (research seed for other crates)

## Problem

Testing protocol/transport/security variants in the two new crates is copy-paste
driven:

- `crates/xray-tui-native/tests/vless.rs` + `tests/vmess.rs`: 15 hand-written
  `#[tokio::test]` fns, each an identical `run_against_cores(&CaseSpec::…builder…)`
  shape. The variant matrix (protocol × network × tls × core) is implicit —
  adding a variant means copy-pasting another fn (this happened when ws/grpc
  joined).
- `run_against_cores` runs both cores inside ONE test and short-circuits: a
  sing-box failure is hidden when xray fails first.
- Unit modules repeat assertion tables: grpc codec (6 fns over payload shapes),
  `spec/mod.rs` wire-encoding tables (19), crypto JA3/JA4 goldens (13), vmess
  keys/header (7+5).

## Decision

Adopt `rstest` 0.26.1 (latest stable, crates.io) as the test framework in both
crates: `#[case]` for named scenario tables, `#[values]` for cartesian axes,
`#[fixture]` (incl. `#[once]`, sync) for shared e2e resources. Keep the
existing e2e engine (CaseSpec + trait-object variants, 7-step `run()` lifecycle,
expectations) — the test LAYER is what changes, not the engine.

### E2E matrix — 2 test fns, 78 generated tests

`#[rstest]` fn per protocol; `#[case::name(CaseSpec)]` rows; cores via
`#[values(CoreKind::Xray, CoreKind::SingBox)]` arg → rstest generates the
cartesian, one test per (case, core). Core name lands in the generated test
name (`vless_against_cores::case_2_tcp_chrome::core_2_SingBox`); failures
isolate per core. Trait-object variants stay; `RealityTls::fresh()` is a
runtime expression so it works inside `#[case]`.

| fn | axes | rows | tests |
|---|---|---|---|
| `vless_against_cores` | network {tcp, ws, grpc} × tls {standard, chrome, reality, wrong-pbk, wrong-sid, plain→reality-server, reality→plain-server} | 21 | 42 |
| `vmess_against_cores` | security {aes128gcm, chacha20} × network {tcp, ws, grpc} × tls {standard, chrome, reality} | 18 | 36 |

Full cartesian = new coverage (reality/fallback over ws/grpc framing; vmess over
ws/grpc). A combo that legitimately fails server-side gets a documented skip,
not a delete.

`run_against_cores` is replaced by a per-(case, core) runner `run_against(case,
core, &certs, &echo, &tls_echo)` — same 7-step lifecycle, 3 attempts, same
`E2eExpect` assertions.

### E2E fixtures

- `#[fixture] #[once] fn cores() -> (CoreUnderTest, CoreUnderTest)` — env
  resolve (hard-fail when `XRAY_TUI_CORE_BIN_DIR` missing) once per test
  binary; test picks by `CoreKind` case arg.
- `#[fixture] #[once] fn certs() -> Certs` — rcgen CA once; 78 rows would
  regenerate per run otherwise.
- `#[fixture] fn echo() -> EchoServer` — shared per-test, NOT once: safe
  because tiny_http is stateless (GET / → 200 BODY), but per-test avoids any
  cross-test coupling.
- `#[fixture] fn tls_echo(certs: &Certs) -> TlsEchoServer` — per-test, NOT
  once: `reset_recording`/`saw_h2_preface` is fallback-case mutable state;
  sharing across parallel tests races.

### Unit conversions (same assertions, table-driven — only where the SAME assertion repeats over inputs)

- `transport/grpc.rs`: `frame_roundtrip(#[case] payload)` (short / varint-2-byte
  / empty), `frame_parse(#[case] payload, #[values] split)`, multi-message as a
  case row; `service_name`/`path` stay one-offs.
- `xray-tui-tls` `spec/mod.rs`: the 18 extension `encode_body` tests
  (ServerName … Raw) are one repeating assertion (encode → exact wire bytes) →
  ONE rstest fn with 18 named `#[case]` rows; `grease_detection` stays.

Explicitly NOT converted (verified during survey — each is a distinct Go- or
RFC-verified golden or logic test, no repeating assertion; conversion would be
churn without clarity): `crypto/mod.rs` aead/key-schedule goldens, `ja3.rs`/
`ja4.rs` logic tests, `profiles/mod.rs` (JA4 prefix property loop), `vmess/
keys.rs` + `header.rs` Go-verified wire goldens, `transport/ws.rs` (2 one-offs).

## Non-goals

- **No `#[timeout]` on e2e rows**: rstest timeout aborts the test future; a
  mid-flight abort skips `CoreGuard` Drop → orphan core subprocesses. The
  probe's internal 15 s × 3-step deadlines already bound each row (~90 s worst).
- **No `#[trace]`**: `CaseSpec` holds `Box<dyn TlsVariant>` — not Debug. Named
  `#[case::name]` rows self-document instead.
- **No enum refactor** of the e2e engine; trait-object variants stay.
- **No `rstest_reuse`** this round — the two matrices are one fn each; templates
  become worthwhile when Trojan/SS/other protocols join.
- No changes to tests outside the two crates; no changes to non-test code.

## Risks

- Full-cartesian e2e run time: 78 rows × spawn+probe (~2–6 s each) — expect the
  `native-e2e` sweep to grow from ~15 s to a few minutes. Acceptable for the
  coverage; tests run in parallel.
- rstest macro errors on complex `#[case]` expressions are verbose — keep case
  expressions single-line builders; move repetition into a `fn case_*` helper
  if a row overgrows.
- `#[once]` fixtures are per-test-binary (both `tests/vless.rs` and
  `tests/vmess.rs` get their own instances) — correct, just not shared across
  binaries.

## Verification

- `cargo test -p xray-tui-tls` — unit tables green; count ≈ unchanged (same
  assertions, fewer fns).
- `cargo test -p xray-tui-native --lib` — codec/unit tables green.
- `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features
  native-e2e --test vless --test vmess` — 78 tests, all green; `-- --list`
  shows per-core names.
- `cargo clippy -p xray-tui-tls -p xray-tui-native --all-targets` — zero
  warnings; `cargo fmt --check -p …` clean.
