# Native ping pipeline: real ping on the in-process core

Date: 2026-09-14
Status: implemented (2026-09-14, ADR 0004)
Supersedes (on accept): `docs/native-core-integration.md` §6, D6, work item 7, and the
"Native anywhere in the real-ping path" out-of-scope bullet
Related: decision 15 (batch keying), decision 16 (test-priority tiers + labels),
decision 20 (native core is connect-time only), decision 22 (write-behind RESULT group),
decision 17 (never a blanket TLS skip), decision 4 (a schema-tag bump is a wipe)

## Problem (measured / anchored)

`real ping` is the ONLY remaining binary-core dependency in the test pipeline. Fast ping
is already process-free (`ping/adapters/{tcp,udp,quic}.rs` on raw tokio sockets), so the
premise "reimplement Fast *and* Real" reduces to: keep fast as-is, move real onto the
native engine.

Cost of one real ping today (`ping/real/{mod,pool}.rs`, `xray.rs`, `singbox.rs`):

| Step | Anchor |
| --- | --- |
| `tempfile::TempDir` in `bin_configs_dir` + serialize + write `config.json` | `process.rs:150-160` |
| `find_binary` per probe; miss ⇒ `"Binary not found"` for EVERY real ping | `pool.rs:345-359`, `xray.rs:64-65` |
| `Command::spawn` + stdout/stderr reader tasks (channel 512) + readiness poll (20×500 ms) | `process.rs`, `xray.rs:63-85` |
| local SOCKS5 listener + `wait_for_socks5` (5 s deadline) | `pool.rs:251-267` |
| warm-core reuse behind a `tokio::Mutex` held ACROSS the HTTP probe (single pings serialize) | `pool.rs:135-293` |
| SIGHUP (sing-box) / stop+restart (xray) reload per single ping | `pool.rs:180-249` |
| batch phase 2: up to `real_ping_concurrency` (default 100) live cores at once | `ops/ping.rs:793`, `app_config.rs:570` |

Consequence on record: "with no core binaries installed, real pings fail with
'Binary not found' even for profiles that connect natively (known limitation)"
(`native-core-integration.md:322-325`). A dead end that cannot be fixed inside the
subprocess arm.

The native engine needs none of that: `xray_tui_native::connect(NativeConnectParams) ->
NativeTunnel` (`lib.rs:74`, `chain.rs:88`) is a plain `AsyncRead + AsyncWrite` stream, with
per-step deadlines (`error.rs:34-40`: DIAL/TRANSPORT/SECURITY/PROTOCOL 10 s) and a
capability gate that already knows which rows it can serve (`capability.rs:39-300`).

### Prior decision reversed

`native-core-integration.md` D6 (`~~Real ping for native = ephemeral native server~~ —
SUPERSEDED: real ping always uses subprocess temp cores`) rejected a native probe because it
"would add a second core lifecycle for no observable gain". That premise no longer holds:
the native probe REMOVES a core lifecycle instead of adding one, and it removes the binary
dependency, the port allocator, the readiness poll, the temp config dir, the reader tasks and
the pool lock with it. This spec re-opens the decision deliberately and amends that document;
it is not a silent drift.

## Decisions (user-owned, answered 2026-09-14)

| # | Decision |
| --- | --- |
| D1 | **Fast ping mechanism unchanged** — raw TCP/UDP/QUIC transport probe, no config load, dedup by `(host, port)`. Nothing to replace (it has no binary dependency). |
| D2 | **Real ping = direct native tunnel + HTTP over the tunnel** (the sing-box `urltest` shape: dial the outbound, run the HTTP client on that one connection). No local listener, no SOCKS hop. |
| D3 | **Untestable rows are persisted as `ProfileErr::Real` with a reason text** ("fast-fail with the `[real]` label"). |
| D4 | *(agent decision, flagged)* Visibility of the marker, in three rules: **R1** the fast phase stays untouched and probes every row (D1); **R2** the real phase is gated — an untestable row gets the `[real]` marker instead of a probe, and the plan-time kind gate writes the marker for ANY ping operation, fast-only batches included; **R3** in the Test cell the untestable marker outranks a measured delay, exactly as `[name]` (DNS-unresolved) already does. Rationale: the alternative (suppress the fast probe) throws away a working measurement just to win a rendering rule. |
| D5 | **The subprocess real-ping machinery is deleted, not kept behind a setting** — `CorePool`, `RealPingManager`, `ping/real/{xray,singbox}.rs`, the port allocator, `wait_for_socks5`. |

## Architecture

```
ops/ping.rs (orchestration: plan, scheduler gate, events, progress)
  └─ ops/ping_native.rs (probe policy: URL, retries, latency span, IP info, error text)
       ├─ xray_tui_native::capability::support_reason(kind, config) -> Option<&'static str>
       └─ xray_tui_native::probe::{fetch, fetch_over}
            ├─ connect(NativeConnectParams) -> NativeTunnel        (proxy dial + handshake)
            ├─ xray_tui_tls::client::connect(tunnel, plain WebPKI)  (target TLS, https only)
            └─ hyper 1.x http1 client conn over TokioIo             (one request, response head)
```

Owner changes:

- **Probe execution** moves from `xray-tui-core::ping::real` (subprocess) to a new native
  primitive + a TUI-side policy module. This keeps D3 of `native-core-integration.md`
  intact: `xray-tui-core` stays the subprocess facade; the native engine is called from the
  layer that already calls it (`crates/xray-tui`, which depends on `xray-tui-native`).
- **Testability authority** stays a single predicate: `capability.rs` grows a reason-returning
  form so the gate and the marker text cannot drift.
- **Display** gains one precedence rule (`ui/profiles.rs` Test cell, R3) — the only UI change;
  no layout, column, key or flow changes.

## Probe execution

Per attempt (attempts run concurrently, `real_ping_retries`, min 1 — parity with today):

1. Parse `speed_test.ping_url` (`reqwest::Url`), take `(scheme, host, port_or_default, path)`.
2. `t0 = Instant::now()`.
3. `NativeConnectParams::new(config.clone(), endpoint_essentials(endpoint), TargetAddr(host, port))`
   → `connect(params)`, wrapped in `tokio::time::timeout(real_ping_timeout_secs)`.
4. `https` only: `xray_tui_tls::client::connect(tunnel, TlsConfig::plain(None,
   Arc::new(WebPkiVerifier::webpki_roots()), host))` with `alpn = ["http/1.1"]`.
   Real WebPKI verification, **no insecure mode** (decision 17: "never a blanket skip"); this
   matches today, where the core verified the target certificate.
5. `probe::fetch` sends `HEAD <path>` (no redirects) and returns as soon as the response head
   arrives. `latency_ms = t0.elapsed()`.
6. Fastest 2xx across attempts wins; otherwise the last error wins (parity with
   `speed_test.rs:174-260`).
7. On success only: a second tunnel + `GET ip_api_url` (10 s cap, body limited to 64 KiB via
   `http_body_util::Limited`, which is NOT feature-gated in 0.1.5 — no manifest change) →
   `query` + `country` → `"<ip> | <country>"`; any failure ⇒ `ip_info = None` (parity).

Notes:

- The probe offers `http/1.1` only, and the ALPN list MUST be set explicitly:
  `TlsConfig.alpn = None` makes the engine offer the *profile's* own list (chrome_130 =
  h2 + http/1.1), the target then negotiates **h2** and a hyper http1 client over that stream
  breaks. Using hyper's http2 client instead was rejected (more surface for no gain). A ping
  URL whose host negotiates nothing but h2 fails the handshake with a clear error; that is
  acceptable for the default URLs (gstatic, ip-api).
- The proxy server's host is resolved by `connect` once per attempt (bounded by the 10 s DIAL
  deadline). Injecting the endpoint's `resolved_as` as `params.resolved_ip` was considered and
  rejected: a stale resolution is a wrong measurement, and the subprocess path resolved per
  attempt too.

Latency span: proxy dial + protocol handshake + target TCP + target TLS + request → response
head. That is the same span the persisted number covers today minus the loopback SOCKS hop
(sub-millisecond), so stored delays stay comparable across the cutover.

Errors map to the existing `error: Option<String>`: `timeout`, `dial`, `tls`, `reality`,
`protocol`, `unexpected status <code>`, `not testable by the native engine: <reason>`.

## Testability gate and the `[real]` marker

`capability.rs`:

```rust
pub fn support_reason(kind: ProtocolKind, config: &ProtocolConfig) -> Option<&'static str>;
pub fn supported(kind, config) -> bool { support_reason(kind, config).is_none() }
```

The six per-kind checks (`vless_supported`, `vmess_supported`, …, `ss_supported`) become
reason-returning internally; `supported` delegates. `ops/connect.rs` may then use the same
reason in its downgrade warning instead of the coarse `REFUSED_CAPABILITY` text (one
authority, no drift).

Gate placement:

- **Plan time, kind only** (`capability::kind_supported`, in-memory `proto_kind`, no DB read):
  every ping operation — single or batch, fast-only or fast+real — retires a link whose kind
  has no native implementation by emitting the `[real]` marker once and never scheduling a
  `RealPing` task for it. Its fast probe still runs (R1).
- **Probe time, config-aware** (`support_reason(kind, &loaded_config)`, after
  `load_protocol_with_config`): a native-capable kind whose *config* the engine refuses
  (VLESS `mlkem768x25519plus`, legacy VMess ciphers, non-zero `alter_id`, xray-only uTLS
  fingerprint ids, mKCP `seed`/`header_type`, SS SIP003 plugin / legacy cipher / malformed
  PSK) replaces its real probe with the same marker.
  The config check runs only after the kind check, so an unsupported kind never pays a config
  load at all, and the fast phase (which never loads configs) stays exactly as fast as today.

Marker:

- Written through the EXISTING path: `TestTypeUpdate` (arm the dedupe guard) then
  `SpeedTestResult { test_type: RealPing, latency_ms: None, error: Some(text) }`
  (`ops/events.rs:422-598` ⇒ `ProfileErr::Real` via `err_kind_for`, staged in
  `LinkGroups::RESULT`). No new persistence code, no schema change.
- `text = format!("{UNTESTABLE_PREFIX}{reason}")` with
  `pub const UNTESTABLE_PREFIX: &str = "not testable by the native engine: ";`
- **Why reuse `ProfileErr::Real`:** `error_kind` is CHECK-constrained
  (`database.rs:462-471`), so a new variant is a schema-tag bump = data wipe (decision 4).
  Not an option.

Consequences that MUST be handled:

1. **Test-cell precedence (R3).** `test_cell_content`/`compute_test_cell`
   (`ui/profiles.rs:403-488`) must treat an active-link marker whose text starts with
   `UNTESTABLE_PREFIX` as a first-class status that outranks a measured delay — the same slot
   `[name]` already occupies, and for the same reason (a status the user must see).
   Without it the label is invisible on any row that has a stored fast (or older real) delay,
   because a measured delay renders first. Genuine `[real]`/`[fast]` failure markers keep
   today's precedence (delay wins).
2. **Stale measurements.** An untestable row may still carry a previous real latency (measured
   by the subprocess engine before this change, or by a fast probe now). The tier law already
   lets the marker win (`endpoint_rank.rs:59-88` checks `error_kind` before `measured`), and
   R3 makes the display agree: the value is retained in the DB but is neither ranked nor shown
   at row level.
3. **`remove_failed_servers` guard.** `state::link_is_failed` is `error.is_some()`
   (`state.rs:413-415`), and `remove_failed_servers` deletes every endpoint with any failed
   link (`ops/ping.rs:398-432`). Without a guard, "Remove Bad Servers" deletes every
   unsupported profile. Guard: a link counts as failed for removal only when its marker text
   does NOT start with `UNTESTABLE_PREFIX`. One constant, one writer, one reader, one test.
4. **Rank tier.** Untestable rows are tier 3 (real-err) by the existing law
   (`endpoint_rank.rs:59-88`), i.e. below untested (2) in Test order. Accepted and documented:
   the marker text says why, and no schema change can express a distinct tier.
5. **Error TTL.** `error_ttl_hours` sweeps untestable markers like any other
   (`ops/profiles.rs:34-77`); the next ping operation re-writes them. Accepted.
6. **`protocol_core_overrides` does not make a row testable.** Forcing `xray`/`sing-box` for
   a native-incapable row still leaves it untestable (the probe engine is native-only by
   decision D5).

## Batch pipeline deltas

- `EngineProbeRunner` loses `pool` and `batch_active`; its `real` calls the native probe.
- **Phase 1 gets its missing concurrency bound.** `fast_ping_concurrency` (default 200) is
  defined, displayed and saved but never read (`app_config.rs:517-523`, `ops/settings.rs:267`,
  `ui/settings.rs:574`); the fast phase today spawns one future per link with no global cap.
  Wire it as the phase-1 semaphore. It is now the ONLY global bound on the fast phase (the
  kind gate no longer removes links from it, R1).
- **Phase 2**: keep one scheduled `RealPing` task per link, the per-endpoint grouping and
  the `dedup_endpoints` retirement (`cancel_queued` + `complete`, no marker written); the
  semaphore is now `real_ping_concurrency` in-process probes instead of processes. Untestable
  links are simply absent from this phase (retired at plan time or, for config-level refusals,
  settled by their own marker instead of a probe).
- **Dead keys in the same settings section.** `real_ping_window` (20) and `batch_page_size`
  (1000) are defined, displayed and persisted but read by NO code path
  (`app_config.rs:515,521`, `ops/settings.rs:251,263`, `ui/settings.rs:570,573`).
  Recommendation: delete both (config field + form row) in this cutover — a knob that does
  nothing is worse than a missing one. Flagged for the reviewer; drop this item on request.
- Unchanged: batches serialized, `DnsDeferred` re-scheduling, `QueueFull` warn, progress
  atomics, the write-behind RESULT/TRAFFIC split, flush + TTL sweep + `wal_checkpoint` at
  batch end, `(endpoint_id, protocol_id)` keying everywhere.

## What is deleted

| Path | Note |
| --- | --- |
| `crates/xray-tui-core/src/ping/real/{mod,pool,xray,singbox}.rs` | whole subprocess arm (~1.5k lines with tests) |
| `crates/xray-tui-core/src/speed_test.rs`: `real_ping`, `wait_for_socks5`, `RealPingResult` | replaced by the native probe; `tcp_ping`/`udp_ping`/`speed_test`/`udp_test` stay |
| SOCKS client cache policy bit only used by `real_ping` | prune to the remaining callers' shape |
| `AppState.core_pool`, `ops/ping.rs::get_or_create_pool` | pool no longer exists |
| re-exports of `RealPingManager`/`CorePool` (`ping/mod.rs`, `core/lib.rs`) | deleted with the module |
| `xray-tui-core/src/ping/mod.rs`: `PingResult`, `ProfileKey` (real-path types) | `PingError::NotSupported` stays — the fast manager returns it |
| `ConfigBuilder::build_multi` (`config_builder/mod.rs` + `xray.rs`/`singbox.rs` + `MultiInboundItem`) | **DEFERRED** — genuinely dead (zero callers since it was written), but it is a separate dead-code sweep in three files, not part of this cutover; recorded in the plan status. |

`ConfigBuilder::build` STAYS: `connect.rs` still builds subprocess configs for the
subprocess connect path.

## Verification

1. Hermetic unit (`xray-tui-native`): `probe::fetch_over` (the stream-level HTTP/TLS step) over
   a local `tiny_http`/TLS double — status, body limit, timeout, no-redirect.
2. Hermetic unit (`xray-tui`): gate reasons per refusal class; marker text + prefix; the
   `remove_failed_servers` guard keeps untestable endpoints; the plan-time kind gate marks an
   unsupported row in a fast-only batch and still schedules its fast probe; the Test cell
   shows `[real]` for an untestable marker over a stored delay, and still shows the delay over
   a genuine `[real]`/`[fast]` failure marker; URL parse (`http`/`https`, explicit and default
   port, path).
3. Probe-outcome mapping with a stub `fetch` (success / timeout / non-2xx / TLS error),
   including "fastest 2xx wins" and "IP-info failure does not fail the probe".
4. Smoke (required, real runtime): launch the TUI against a real profile —
   real ping from the menu shows latency + exit IP; a visible-list batch completes with
   `pgrep -c xray` = 0 and `pgrep -c sing-box` = 0 throughout; an unsupported kind row
   (TUIC/AnyTLS/…) shows `[real]` after any ping operation while its fast delay stays visible
   in the expanded panel; "Remove Bad Servers" keeps it.
5. Measurement on the reference feed: batch real-ping wall clock, peak RSS and process count
   (expect: processes 100 → 0); single real-ping latency vs the subprocess path (expect
   parity ±1 ms).
6. Existing suites: workspace `cargo test`, `cargo clippy` 0 warnings, `just quality-gate code`,
   and `cargo hakari` regeneration if any manifest changes.

## Compatibility

- **No DB change, no schema-tag bump, no migration.** Column shapes, the tier law, the label
  rendering and the RESULT/TRAFFIC groups are untouched.
- Persisted `core_type` stays `{Xray, SingBox}` and native stays runtime-only (decision 20).
- Rows that were real-pingable before stay real-pingable iff the native engine can serve them;
  the rest become visibly untestable instead of silently "Binary not found".
- Rollback: the deleted subprocess arm is recoverable from git; nothing in the DB depends on
  which engine produced a persisted result.

## Risks

| Risk | Treatment |
| --- | --- |
| The probe now tests the native engine, not the subprocess stack, for native-capable rows that would connect via a subprocess (routing rules / non-default DNS present). | Documented divergence; native is the default connect path, and the alternative (spawning a core) is exactly what D5 removes. |
| Untestable rows sink in Test order (tier 3) and are TTL-swept. | Documented; marker text explains; re-marked by the next batch. |
| The removal guard depends on a text prefix. | Single constant, one writer, one reader, pinned by a test in each place. |
| In-process TLS/REALITY handshakes are CPU-bound; `real_ping_concurrency` now bounds CPU, not processes. | Keep the knob and its default; measure before changing it. |
| Target TLS verification (WebPKI roots) fails on intercepting networks ⇒ probe error. | Matches today's behavior (the core verified the target certificate); no insecure fallback (decision 17). |

## Non-goals

- Fast ping mechanism (D1), speed test, UDP test — `start_speed_test`/`start_udp_test` ride
  the CONNECTED core's local SOCKS port and already work for native sessions.
- Routing/DNS-aware native probing (native v1 is proxy-all).
- A new `ProfileErr` variant, any schema change, any UI work beyond the R3 Test-cell
  precedence.
- Reviving `ConfigBuilder::build_multi`.

## ADR signal

ADR 0004 (recorded at implementation time, per the Aegis rule that accepted architecture
memory is not created from unexecuted ideas): *the real-ping probe engine is the native
core; the subprocess probe arm is retired*, with the D6 reversal rationale and the
alternatives considered (in-process SOCKS stub / keep subprocess). Document amendments due on
accept: `docs/native-core-integration.md` §6 + D6 + work item 7 + the out-of-scope bullet,
`AGENTS.md` decisions 15 and 20, `TUI_MANUAL.md` test-label section,
`docs/aegis/INDEX.md`.
