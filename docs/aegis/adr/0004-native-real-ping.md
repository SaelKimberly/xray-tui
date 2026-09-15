# 0004 — The real-ping probe engine is the native core

Date: 2026-09-14
Status: accepted
Supersedes: `docs/native-core-integration.md` D6 (2026-09-03) and its §6 / work item 7 /
out-of-scope entry
Related: decisions 15 (batch keying), 16 (tiers + labels), 17 (never a blanket TLS skip),
20 (native is connect-time + runtime-only), 22 (write-behind RESULT group)
Spec: `docs/aegis/specs/2026-09-14-native-ping-pipeline-design.md`
Plan: `docs/aegis/plans/2026-09-14-native-ping-pipeline.md`

## Decision

Real ping — single (menu) and batch phase 2 — executes on the in-process native engine.
`ops/ping.rs` schedules the same `(endpoint_id, protocol_id)` tasks as before; the probe
itself is `crates/xray-tui/src/ops/ping_native.rs` → `xray_tui_native::probe::fetch`, which
dials the proxy with `connect(NativeConnectParams)` (optional target TLS through the engine,
ALPN pinned to `http/1.1`, verified against the Mozilla root program) and issues one
HTTP/1.1 request over the tunnel. Latency is measured from the start of the dial to the
response head — the same span the subprocess probe covered minus the loopback SOCKS hop.

The whole subprocess probe arm is retired: `CorePool`, `RealPingManager`,
`ping/real/{xray,singbox}.rs`, the temp config directory, the port allocator, the SOCKS
readiness poll, the warm-core reload paths, and `AppState.core_pool`. `find_binary` is no
longer on any test path.

Rows the native engine cannot serve are a first-class result, not a fallback:
`capability::support_reason(kind, config) -> Option<&'static str>` is the single testability
authority (its bool form `supported` delegates), kind-level refusals are gated at plan time
from the in-memory `proto_kind`, config-level refusals inside the real probe, and both
persist a `ProfileErr::Real` marker whose text is `"not testable by the native engine: …"`.

## Why (the reversal)

D6 rejected a native probe because it "would add a second core lifecycle for no observable
gain". Measured against the actual goal — a binary-free test pipeline — that premise
inverts:

- the native probe REMOVES a lifecycle (no process, no temp config, no port allocation, no
  readiness poll, no stdout/stderr reader tasks, no pool `Mutex` held across the HTTP probe);
- a native-only install could not real-ping at all (`"Binary not found"` for every row);
- batch phase 2 legitimately held up to `real_ping_concurrency` (default 100) core processes
  at once;
- the subprocess engine also was not the engine the app connects with by default.

## Alternatives considered

1. **Ephemeral in-process SOCKS5 server (`NativeCoreServer` on 127.0.0.1:0) + the existing
   reqwest-over-SOCKS probe** (the v2rayN shape). Rejected by the user: it keeps a local
   listener, an accept loop and a userspace relay per probe to save ~0.2 ms, and it measures
   through a hop the app does not otherwise have.
2. **Keep the subprocess probe** (D6 as written). Rejected: keeps the binary dependency and
   every cost above.
3. **A new `ProfileErr` variant for "untestable".** Rejected: `error_kind` is CHECK-frozen
   (`database.rs` `error_kind_str`), so a new value is a schema-tag bump = a data wipe
   (decision 4). The marker therefore reuses `ProfileErr::Real` and is discriminated by its
   text prefix.

## Consequences

- No DB change, no schema bump, no migration. Persisted `core_type` stays `{Xray, SingBox}`;
  native stays runtime-only (decision 20).
- An untestable row ranks tier 3 (real-err) and is TTL-swept like any marker; the Test cell
  renders `[real]` for it even when a fast measurement exists (decision 16 amendment).
- `remove_failed_servers` skips untestable markers (`is_removable_failure`) — without that
  guard, "Remove Bad Servers" would delete every non-native profile.
- For a native-capable row that would connect via a subprocess (routing rules or non-default
  DNS present), the probe now tests the native engine; native is the default connect path.
- `real_ping_concurrency` now bounds in-process handshakes (CPU) rather than processes.
- `fast_ping_concurrency` is no longer dead config: it bounds phase 1.
- Verification: hermetic unit tests for the probe HTTP half, the capability reason/support
  parity, and the gate/guard wiring; plus a real-core e2e
  (`crates/xray-tui-native/tests/probe_e2e.rs`, `#[ignore]`d without a core binary) that
  drives one HTTP request through a real xray-core VLESS server.

## Amendment — 2026-09-15: the attempt deadline is the configured budget

`ProbeRequest::timeout` (from `real_ping_timeout_secs`) bounded only the HTTP
request step, while dial / security / transport / protocol ran under the
engine's fixed 10 s `timeouts::*` constants — so a 5 s setting still produced
`timeout on tcp dial (limit 10s)` (192 of 350 real probes in the measured run),
and a pathological attempt could spend ~60 s across its steps. `probe::fetch`
now runs `connect` + `fetch_over` under ONE `tokio::time::timeout(req.timeout)`
reporting `NativeError::Timeout { step: "probe attempt", .. }`; the engine's
per-step limits stay as defence in depth. The same wrapper makes the reported
latency the whole probe span (dial included), matching the subprocess era's
local-SOCKS measurement, and `a_stalled_handshake_is_bounded_by_the_attempt_budget`
pins it against a peer that accepts and stays silent.
