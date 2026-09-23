# WS path canonicalization — implementation

- **Date**: 2026-09-23
- **Spec**: `specs/2026-09-23-ws-path-canonicalization-design.md`
- **Status**: executing

## Tasks

| # | Task | Files |
| --- | --- | --- |
| T1 | `canonicalize_transport_paths` (rule) + the pchar encoder + unit tests (merge, no-false-merge, `?`/`#`, idempotence) | `crates/xray-tui-proto/src/proto_spec/common.rs` (+ its tests) |
| T2 | Apply at the single owner — `protocol_from_parsed`, before `identity_once()` and the stored columns | `crates/xray-tui/src/state.rs` (+ test) |
| T3 | e2e: real xray server with a non-canonical path; client config **parsed from a share URL** then canonicalized | new `crates/xray-tui-native/tests/ws_paths.rs` (self-contained — no `CaseSpec` knob) |
| T4 | Owner-doc amendment: split the `ConfigInvalid` class | `docs/aegis/specs/2026-09-17-purge-reason-design.md` §7 |
| T5 | Verification: `cargo nextest run -p xray-tui-proto -p xray-tui`; e2e ignored row run with binaries | — |
| T6 | Early data: `?ed=NNNN` hoisted into `max_early_data` (path untouched); the xray carrier re-derived when the path has none; `SCHEMA_VERSION` 12 → 13 | `proto_spec/{common,mod}.rs`, `xray-tui-db/src/database.rs` |

## Ordering

T1 → T2 (T2 depends on the rule) → T4/T5. T3 in parallel with T2 (different crates).

## Contracts

- Rule signature (T1): `pub fn canonicalize_transport_paths(&mut self)` on `ProtocolConfig`. Pure, idempotent, scoped to `Ws`/`Http`/`HttpUpgrade`/`XHttp`; `Grpc` untouched.
- Rule output (pinned): rows 2 and 4 of the spec §2 both → `/trTelegram%F0%9F%87%A8%F0%9F%87%B3%20@WangCai2`; `/?ed=2560` byte-identical.
- Coverage: the transport path **and** the vless/vmess/trojan top-level mirror, the mirror gated on a URI-path transport (`canonicalize_paths` returns that bool). `Grpc`'s service name must not move, and neither must a pathless transport's mirror.
- T3 client config: `ProtocolConfig::try_parse_proto(url)` → `canonicalize_transport_paths()` → dial. Never a struct-literal path.

## Deferred / not done here

- grpc service-name path (spec §3).
- The §5.9 default-header A/B (recorded negative).
