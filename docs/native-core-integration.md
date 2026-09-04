# Native-core integration — Spec Brief

Status: IMPLEMENTED 2026-09-03 · Baseline: `b2c1786ecd67be191912f0f1b020fab3f8753eae` (clean tree)

## Implementation deviations from the brief

- **Core resolution (§1 + work item 5 superseded):** the proto `CoreType` stays
  `{Xray, SingBox}` — nothing native-specific is persisted. Native preference is
  decided per connect in `ops/connect.rs::resolve_runtime_core` from the link's
  concrete stamp + `protocol_core_overrides` + the loaded config (see §1).
- **Real ping (item 7 superseded):** no ephemeral native server and no
  `native_ping`. `real_ping` and `CorePool::ping` dispatch on the link's
  concrete `core_type` (`Xray`/`SingBox`), so Phase-2/single real pings always
  run on temp subprocess cores with zero new code. A native real-ping path
  remains possible later; it is not needed for Test-column parity.
- **UDP ASSOCIATE trace:** TCP CONNECT + HTTP CONNECT legs emit full
  open/close/byte trace rows; UDP associations are relayed but untraced
  (byte accounting would need relay-loop surgery — documented gap).
- **Server lifecycle test:** not added as a hermetic test; coverage is the
  inbound trace test (`relayed_leg_emits_trace_events_and_traffic`) plus the
  connect-branch compile + workspace suite.
- **Activity scroll state:** lives in a module-local atomic in
  `ui/native_activity.rs`, not on `AppState` (matches the Logs-tab pattern).
- **Statistics screen:** connection section shows `in-process (event feed)`
  instead of the gRPC endpoint when the native core is connected.
- **Tier-3 status 2026-09-03:** trojan 14/14, vless 78 + 6 ignored, vmess
  54/54, hysteria2 3/3. The vmess UDP row (`case_22_tcp_aes128gcm_udp`,
  cmd 0x02 vs sing-box 1.13.16: server logs `bad packet connection`,
  client sees a truncated response header) fails identically on the clean
  baseline — pre-existing version-specific interop, not a slice regression.
  Row scoped to xray single-core in `tests/vmess.rs` with the exact log line
  in the comment; client-side framing fix tracked separately.

## Goal

Integrate `xray-tui-native` (in-process proxy core) into the TUI as a first-class
backend for the four e2e-verified protocols — VLESS, VMess, Trojan, Hysteria2 —
with the same user-facing capabilities xray-core/sing-box subprocesses have today
(connect/disconnect, per-profile core selection, traffic + sys stats, live logs,
speed-test integration, UI indicators), plus realtime observability of the native
core: per-connection usage, failures, and traffic on a dedicated screen.

User decisions (2026-09-03):
1. Native-selectable set = the four e2e-verified protocols only.
2. **Native preferred** — a profile whose protocol+config native supports runs
   the in-process core; subprocess only via explicit override. (Shipped as a
   connect-time decision on xray-stamped links — §1; the original "resolves to
   Auto" mechanism is superseded.)
3. Observability = parity counters + realtime per-connection trace + dedicated
   "Native activity" screen.
4. Native honors both `socks_port` and `http_port` inbounds. Routing rules /
   sniffing / TUN are out of v1 (warn, connection still works via proxy-all).

## Current seams (evidence)

- Two disjoint `CoreType` enums: `xray_tui_core::CoreType {Xray, SingBox, Auto}`
  (`crates/xray-tui-core/src/core_type.rs`, Display `xray`/`sing-box`/`auto`,
  kebab-case serde) and `xray_tui_proto::proto_spec::CoreType {Xray, SingBox}`
  (`crates/xray-tui-proto/src/proto_spec/mod.rs:95`, toasty::Embed, custom string
  serde `"xray"`/`"sing-box"`, `from_str` rejects `"auto"`). The proto enum is the
  persisted per-pair override: `ProfileStats.core_type`
  (`crates/xray-tui-db/src/models_toasty.rs:244`). No Native variant anywhere in
  TUI/config/db/proto.
- Parse-time stamping: `persist_parsed` stores `parsed.protocol.core_type`
  (concrete, resolved via `resolve_core`) on every link (`state.rs:273`); a vless
  link therefore stores `Xray` and `resolved_core` (`ops/profiles.rs:256`) always
  honors it first. "Auto" exists only in the core-facing enum and config defaults.
- Connect flow (`crates/xray-tui/src/ops/connect.rs:37`): builds `BuildParams`
  (socks_port, http_port, sniffing, mux JSON, clash_mixin, log_level) →
  `ConfigBuilder::build` → typed `BackendConfig{Xray|SingBox}` → `find_binary` →
  `RealCoreManager::start` (writes JSON, spawns `run -c {0}`, log lines via
  `log_tx`) → TCP port probe on `socks_port` → `CoreEvent::Connected(core_type)` →
  core-type branch: xray = gRPC `StatsProvider` 3 s poll; sing-box = Clash HTTP
  `/traffic` stream → both emit `CoreEvent::StatsUpdate` deltas →
  `apply_stats_delta` accumulates onto `ProfileStats.traffic` (today/total, day
  reset) and `SysStatsUpdate` → `system_stats`.
- `CoreManager` trait (`xray-tui-core/src/process.rs:12`, object-safe) is used as
  `Box<dyn CoreManager>` only by the real-ping `CorePool`; connect.rs constructs
  `RealCoreManager` concretely.
- Statistics tab (`ui/statistics.rs`) is a pure render of `active_link.traffic` +
  `state.system_stats` + connected core label. Logs: subprocess lines →
  `parse_core_log_line` → heed `LogMessage` + `CoreEvent::LogLine`; native has no
  stdout/stderr to read — it must emit log events itself.
- Native crate (`crates/xray-tui-native`): `connect`/`connect_udp`/`connect_mux`
  over typed proto configs; `inbound/` SOCKS5 (TCP CONNECT + UDP ASSOCIATE) with
  `xray-tui-route` Engine → tagged outbounds, Proxy leg reusing `crate::connect`.
  No HTTP inbound, no stats/sys metrics, no event emission, no log output. The
  TUI crate does not depend on xray-tui-native yet.

## Architecture

### 1. Core resolution — native preferred at connect time

Add `Native` to the core-facing enum (`core_type.rs`) for runtime identity only
(`connected_core`, display, sort, statistics label). Updater/`get_core_info`
never see it (defensive: `RealCoreManager::start(Native)` → `Startup` error).

**SUPERSEDED (2026-09-03) — the proto enum keeps `{Xray, SingBox}`.** Adding
`Auto` to `proto_spec::CoreType` and stamping it at parse time was implemented,
reviewed and reverted for two reasons:

- The persisted string `"auto"` violates the frozen CHECK constraint
  (`"core_type" IN ('xray','sing_box')`) carried by every pre-patch database
  file — the write fails at runtime. Bumping the schema tag is NOT a migration:
  a `PRAGMA user_version` mismatch re-pushes the schema and, when that push
  fails, DELETES the database file and recreates it
  (`xray-tui-db/src/database.rs:70-96`).
- `core_type` lives inside the identity-hashed `ProtocolEssentials`, so
  re-stamping a link changes its uid — a subscription refresh would orphan the
  row's stats/history instead of updating it.

Shipped design — nothing new is persisted:

- Parse-time stamping is unchanged: `persist_parsed` stores `resolve_core`'s
  concrete answer, and `resolve_core` returns only `Xray`/`SingBox`.
- The runtime decision is per connect, in `crates/xray-tui/src/ops/connect.rs`:
  ```
  resolve_runtime_core(link_core: CoreType, kind: ProtocolKind,
                       forced: Option<CoreType>, config: Option<&ProtocolConfig>,
                       proxy_all_blocked: bool) -> (CoreType, Option<&'static str>)
  ```
  Native is *asked for* by an xray-stamped link on a native kind, or by a
  `protocol_core_overrides` value of `native`/`auto` (the Settings → Protocol
  Core selector offers only `Auto`/`Xray`/`SingBox`, so `native` is a
  hand-edited config value); an `xray`/`sing-box` override vetoes it. It is
  *granted* only when `capability::kind_supported`
  passes, no routing rules / non-default DNS servers or hosts exist
  (`proxy_all_blocked`), the config is loaded, and
  `capability::supported(kind, config)` passes. Each refusal returns a reason
  string logged as a `warn` naming host, kind, rule/DNS counts and the core that
  runs instead; a refused *explicit* native override earns a second
  "override NOT honored" warn.
- `link_core` stays the only legal subprocess answer: `ConfigBuilder::build`
  dispatches on the stamp, so a runtime core that disagreed with it would feed
  one core's JSON to the other's binary.
- Display/sort (`ops/profiles.rs::resolved_core`) shows the stamp and reports
  `Native` only when `protocol_core_overrides` names it. Connect-time
  downgrades are invisible in the list but loud in the logs.
- No new form surface: the core selector keeps `Auto`/`Xray`/`SingBox`, where
  `Auto` still means "no override" — and, for a native kind, now also "prefer
  native at connect time". No re-stamp, no data migration, no user action.

### 2. Native capability predicate

`xray_tui_native::capability` — explicit, config-level, no stubs:

```
kind_supported(kind: ProtocolKind) -> bool                     // config-blind
supported(kind: ProtocolKind, config: &ProtocolConfig) -> bool // runtime gate
```

There is no `params`/`CapabilityInput` argument, and the predicate consults no
inbound or app-level state: TUN, sniffing, mux settings and routing rules never
reach it. Native is proxy-all, and that half of the decision lives in
`ops/connect.rs` instead — a profile with routing rules or non-default DNS
servers/hosts is refused there with a loud warn ("native core is proxy-all:
routing rules / DNS settings need xray-core") and runs the subprocess core.
Sniffing/mux/TUN gate nothing: those flags simply never reach the in-process
core.

Shipped gates (`capability.rs` — unknown values fail CLOSED, so a field or id
native cannot parse defers to the subprocess instead of dying mid-dial):

- Kind ∉ {Vless, Vmess, Trojan, Hysteria2}, or a `ProtocolConfig` variant that
  does not match `kind` → false.
- VLESS: any non-empty account `encryption` other than `none` → false — this is
  what excludes `mlkem768x25519plus.*` (native diverges from real xray:
  NATIVE_CORE.md SP7 pq-enc, native fails where xray works). Flow must be
  empty, `xtls-rprx-vision` or `xtls-rprx-vision-udp443`; any other non-empty
  flow → false.
- VMess: payload security absent/`auto`/`aes-128-gcm`/`chacha20-poly1305` only
  (legacy `none`/`zero`/`aes-128-cfb`/bare `chacha20` → false); a non-zero
  `alter_id` (the pre-AEAD session scheme) → false.
- Transport — a positive match on the arms native dispatches:
  tcp/ws/grpc/httpupgrade/xhttp/v2rayhttp → true; mKCP only when it carries no
  `seed` (in `KcpConfig::seed` *or* the row's own `path`) and no `header_type`
  camouflage beyond `none`, both being wire format that would silently drop
  every datagram. Bare `TransportConfig::Quic` has no native arm → false. The
  match is exhaustive: a new variant breaks compilation instead of inheriting
  `true`.
- Security: `none`/`tls`/`reality` are all implemented (vision, v1.mux,
  packetaddr, XUDP included) — only the fingerprint id gates. `security.fp()`
  must parse with the same `parse_fingerprint_id` the dial uses (`chrome`,
  `chrome-randomized`, `firefox`, `safari`, `random`); xray-only ids
  (`randomized`, `ios`, `android`, `edge`, `360`, `qq`, …) → false. No `fp` is
  supported (engine default, fixed chrome spec for REALITY).
- Hysteria2 → always true: the dial is a self-contained quinn/QUIC one
  (`protocol::connect_quic`, rustls internal), so there is no transport matrix
  and no fingerprint gate — the QUIC TLS never reads `fp`. Nothing in the typed
  config requests xray-only semantics either: the fields native ignores (`up`,
  `hop_interval`, `pin_sha256`) still interoperate, so the predicate stays true.
  A future gate returning `false` would be a real downgrade, not a kill —
  xray-core builds hysteria2 as the unified `protocol: "hysteria"` outbound with
  `version: 2` (`Hysteria2Config::inject_xray`; the e2e header's "no hysteria2"
  note is about the harness SERVER side).
- The verdict is TCP-truthful: SOCKS5 UDP ASSOCIATE through the native *proxy*
  outbound is not implemented (`proxy_params` never sets `params.udp`), so a
  native session drops the proxy UDP leg regardless of the answer — gating the
  UDP-capable shapes off that gap would only cost them their working TCP path.

### 3. NativeCoreServer (xray-tui-native crate)

New high-level module (in `xray-tui-native`, e.g. `server/`), TUI-facing, no
dependency on xray-tui-core (avoids dragging tonic/heed/LMDB into native; native
already wraps the proto types). Owns one "session" = the connected profile:

```
NativeCoreServer::start(params: ServerParams, events: mpsc::Sender<NativeEvent>) -> JoinHandle-ish
ServerParams {
    socks_port, http_port: Option<u16>, listen: IpAddr,
    route: proxy-all (v1),        // engine default → Proxy outbound
    outbound: NativeConnectParams // from the loaded Protocol row (config + endpoint essentials)
}
```

- Binds the SOCKS5 inbound (existing) on `socks_port` and an HTTP CONNECT
  inbound on `http_port` when set (`inbound/http.rs`). **CONNECT-only v1:**
  absolute-form requests are not forwarded (`501 Not Implemented`); a CONNECT
  without `host:port` is `400`, a head over 16 KiB (`MAX_HEAD_BYTES`) is `431`,
  a Block/Reject routing decision is `403`, an unknown outbound tag or a failed
  dial is `502`, and success is a bare `200 Connection Established` — no framing
  headers on a tunnel, whereas every refusal carries
  Content-Type/Content-Length/`Connection: close` plus a one-line plain-text
  reason. Bytes the client pipelines behind the head are replayed into the
  tunnel, and a failed `accept` never kills the listener (per-connection faults
  retry immediately, resource exhaustion after a 500 ms backoff).
- Proxy auth: `HttpInboundConfig::with_auth(user, pass)` gates the tunnel with
  Basic — missing, malformed and wrong credentials all answer the same `407` +
  `Proxy-Authenticate: Basic realm="xray-tui", charset="UTF-8"`. The native
  session has no credential source in the profile model, so it leaves auth
  unset: in-TUI native HTTP **and** SOCKS5 are unauthenticated and rely on the
  loopback bind, and both inbounds emit exactly one `warn` at bind time when
  they bind a non-loopback address without credentials. No UI/profile field for
  proxy credentials exists.
- Every accepted connection (TCP CONNECT + UDP ASSOCIATE + HTTP CONNECT) resolves
  the outbound and dials via `crate::connect`/`connect_udp`/`connect_mux`
  (mux/vision/flow/transport honored from the proto config).
- Stop = drop inbound listeners + abort in-flight task set; idempotent.
- Failure isolation: per-connection errors never take the server down; they emit
  a Failed event + log line and the connection closes.

### 4. Observability contract (native → TUI)

New `telemetry` module in xray-tui-native. Two levels, one channel:

```
enum NativeEvent {
    Log { level, target, message },                 // structlog-ish strings, no ts (TUI stamps)
    Traffic { up: u64, down: u64 },                 // poll-window delta (read-and-reset atomics)
    Sys { rss_bytes: u64, uptime_secs: u64 },       // process-level, cheap Linux /proc read + fallback
    Trace(TraceEvent),                             // per-connection realtime
}
enum TraceEvent {
    Opened { conn_id, kind: Tcp|UdpAssoc|Http, dest, protocol, transport, tls: Plain|Tls|Reality },
    Bytes   { conn_id, up, down },                  // periodic while open (1 s)
    Closed  { conn_id, up, down, duration_ms, error: Option<NativeErrorInfo> },
}
```

- Byte accounting: counting wrappers on each direction of the relayed stream; a
  per-session task aggregates into both the shared atomic traffic counters
  (delta polls) and the per-conn trace stream.
- TUI adapter task (inside connect.rs's spawn, replacing the core-type stats
  branch for native):
  - `Traffic` → `CoreEvent::StatsUpdate` (same shape as the xray poller — the
    `apply_stats_delta` path, day-reset + upsert, is reused unchanged);
  - `Sys` → `CoreEvent::SysStatsUpdate`;
  - `Log` → `CoreEvent::LogLine` (existing heed + actions-log path — native log
    lines flow into the same Logs tab; no `parse_core_log_line` needed, native
    lines are already structured);
  - `Trace` → new `CoreEvent::NativeTrace(TraceEvent)`.
- Real-time: poller cadence 1 s for trace `Bytes`, 3 s for `Traffic` deltas
  (matches xray gRPC cadence so statistics numbers feel identical).

### 5. TUI wiring (crates/xray-tui)

- Add dependency on `xray-tui-native` (+ `xray-tui-tls` if needed for tests only —
  native re-exports what the server needs).
- `connect.rs`: branch on `resolve_runtime_core` — native path skips
  `ConfigBuilder`/`find_binary`/`RealCoreManager` entirely; starts
  `NativeCoreServer` with the loaded protocol config + params, probes
  `socks_port` readiness the same way, emits `Connected(Native)` on bind.
  The existing xray/sing-box arms are untouched.
- Disconnect: same stop-channel shape; abort task, drop server, flush pending
  traffic deltas (existing `drain_pending_stats_updates`).
- Status/statistics/actions-log: `CoreType::Native` label "native"; statistics
  screen's hardcoded API_ENDPOINT line hidden/neutral for native (no gRPC/Clash
  API exists — native sys/traffic come from events).
- New `Tab::NativeActivity` + render in `ui/native_activity.rs` (naming per
  existing screen modules): live connection table (conn id, opened time, dest,
  protocol/transport/TLS badge, up/down live, state open/closed/error), failure
  strip (recent `Failed` events with layer + message + ts), session totals.
  Backed by a ring buffer in `AppState` fed by the `NativeTrace` handler in
  `poll_core_events`; capped (e.g. 2000 rows), survives only the session
  (no persistence v1). Hotkeys + tab-bar entry mirror existing tabs
  (`types.rs::Tab`, `ui/mod.rs` routing, `TUI_MANUAL.md` update).
- Logs tab: native lines arrive via existing `LogLine` path — zero changes beyond
  `parse_core_log_line` not being applied to native lines.

### 6. Speed-test integration

- Fast ping (Phase 1, in-process adapters) + single TCP ping: unaffected —
  already no subprocess.
- Real ping (Phase 2 + single real ping from menu): **native is never used.**
  `RealPingManager::real_ping` and `CorePool::ping` dispatch on the concrete
  `core_type` (an exhaustive `Xray`/`SingBox` match on the proto enum) and spawn
  temp subprocess cores with a `BackendConfig`, exactly as before this slice.
  The planned ephemeral-`NativeCoreServer` probe (`native_ping`) does not exist
  — see D6. Consequence: with no core binaries installed, real pings fail with
  "Binary not found" even for profiles that connect natively (known
  limitation). Test column semantics unchanged.

## Decisions & rationale

| # | Decision | Why |
|---|----------|-----|
| D1 | ~~Proto `CoreType` gains `Auto`~~ — **SUPERSEDED**: the proto enum stays `{Xray, SingBox}`; nothing native-specific is persisted | Persisting `"auto"` breaks the frozen CHECK constraint on existing DBs (and a schema-tag bump is not a migration — it can delete the file) and re-keys the identity hash that contains `core_type`. Runtime authority moved to connect time, where the config is already loaded. |
| D2 | Native resolution is config-aware at connect only; display shows the stamp | Capability must see `ProtocolConfig` (pq-enc etc.) and only connect loads it. Ping paths take the link's concrete core (no native ping — §6). Downgrade path logs loudly. |
| D3 | `NativeCoreServer` lives in xray-tui-native, not xray-tui-core; events not the `CoreManager` trait | Layering: core is the subprocess facade (tonic/heed/LMDB). Native implements the server + telemetry against proto types only; the TUI adapts events to `CoreEvent`. No new cross-crate trait; the branch in connect.rs mirrors the existing core-type branches. |
| D4 | Telemetry reuses `CoreEvent::StatsUpdate/SysStatsUpdate/LogLine` shapes | Statistics/logs screens, day-reset, and DB accumulation stay untouched; the trace is the only new event family. |
| D5 | Capability predicate excludes native-worse configs, not native-equal | pq-enc diverges (native fails, xray works) → excluded. Equal-failure combos (e.g. reality-over-kcp) route to native. |
| D6 | ~~Real ping for native = ephemeral native server~~ — **SUPERSEDED**: real ping always uses subprocess temp cores | `real_ping`/`CorePool::ping` already dispatch on the link's concrete `core_type`, so batch semantics and the Test column are identical without new code; a native probe path would add a second core lifecycle for no observable gain. |

## Out of scope (v1)

- Protocols beyond the four (SOCKS/HTTP client kinds etc. keep subprocess).
- Routing-rule parity (custom rules + sniffing) through the native route engine.
- TUN/http inbound beyond CONNECT; outbound-only kinds (Redirect/TProxy/Mixed).
- Trace persistence across sessions; per-conn logs beyond failures (debug frames).
- Native anywhere in the real-ping path — real pings always spawn subprocess
  temp cores (§6).

## Risks

- Native becomes the default runtime for the four most common protocols
  (decision 2): regressions surface on real traffic first, e2e suite second.
  Mitigation: capability gate, downgrade-with-log, and the config-level
  `protocol_core_overrides` veto (`xray`/`sing-box`, per protocol kind — note a
  *profile*-level `xray` stamp does not veto native, it is what asks for it);
  e2e suite (149 green rows) is the contract.
- Display/run mismatch: the list shows the stamp (`xray`) while a config that
  passes the gate runs native (D2) — cosmetic; the connect log names the
  runtime core.
- HTTP CONNECT inbound: covered by hermetic tier-1 tests in `inbound/http.rs`
  (status-code surface, auth gate, 16 KiB head cap, pipelined-byte replay), but
  no tier-3 e2e row yet.
- Native inbounds are unauthenticated in the TUI session: safety rests on the
  loopback bind plus the one-shot open-relay warn at bind time. A non-loopback
  `inbound.listen` therefore exposes an open proxy for the connected profile.
- Sys metrics are process-level approximations, not xray's Go runtime numbers —
  statistics screen shows fewer fields for native (uptime/RSS), documented.
- Native is reached with no user action and no migration: every xray-stamped row
  on a native kind goes through the connect-time gate, so a native regression
  surfaces on existing profiles immediately.

## Work items (order)

1. `xray-tui-native`: `capability` predicate + unit tests (incl. pq-enc false).
2. `xray-tui-native`: `telemetry` module (counters, trace events, wrappers).
3. `xray-tui-native`: HTTP CONNECT inbound + tests.
4. `xray-tui-native`: `NativeCoreServer` (socks + http inbounds, proxy-all,
   telemetry feed, graceful stop) + hermetic tier-1 tests.
5. ~~Proto: `CoreType::Auto` variant; parse-time stamping stores Auto; forms
   surface Auto~~ — **SUPERSEDED** (§1): the proto enum keeps
   `{Xray, SingBox}`, stamping is unchanged, and no form gains a new entry.
6. Core: `CoreType::Native` variant (runtime identity only) + defensive guards
   (updater/bin_manager/RealCoreManager). **No core-crate resolver:** the
   `resolve_runtime_core` that shipped is a private connect-time helper in
   `crates/xray-tui/src/ops/connect.rs` (§1).
7. ~~Core ping: native real-ping entry~~ — **SUPERSEDED** (§6): no
   `native_ping`; real pings always use subprocess temp cores.
8. TUI: connect.rs native branch + adapter task (events → CoreEvent);
   disconnect; drain flush.
9. TUI: status/statistics/actions labels for native; updater panel note
   ("in-process — no binary").
10. TUI: `NativeActivity` tab + ring buffer + `TUI_MANUAL.md`.
11. e2e: native-session rows through the TUI harness (connect → traffic →
    disconnect; http inbound; trace events flow), plus existing suite green.
12. Docs: AGENTS.md decision entries + NATIVE_CORE.md status flip.

## Verification

- `cargo test -p xray-tui-native --features native-e2e --lib` (tier 1 hermetic).
- Tier-3 e2e rows for the four protocols (existing 149-green contract) +
  new server-mode loopback rows (socks + http inbounds, telemetry events).
- `cargo test` workspace + `just quality-gate code`.
- Manual: run TUI, import vless/vmess/trojan/hysteria2 links, connect → native
  (no core selection needed: the stamp is `xray` and the connect-time gate
  picks native); Statistics shows live traffic; NativeActivity shows per-conn
  rows; kill network mid-connection → Failed events; http_port proxy works;
  `protocol_core_overrides` xray entry → subprocess path unchanged; pq-enc
  vless → downgrade warn.
- Architecture review of the CoreType/resolution surface (Aegis gate).
