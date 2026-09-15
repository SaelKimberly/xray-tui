# Ping-pipeline persistence and observability — implementation plan

Goal: a Fast+Real batch persists every result it emits, an import refresh cannot
destroy test results or traffic counters, phase 2 does not re-probe hosts phase 1
already proved dead, the configured real-ping timeout actually bounds the probe,
and one batch leaves one summary line instead of thousands of per-result lines.

Evidence base: `~/.config/xray-tui/logs.lmdb` (120,869 entries, 09-09 → 09-15,
session 09-15 08:11:59–08:25:00 on commit `6a605d6`) + `data.db` cross-check.
Findings: 172/446 emitted results (39%) never persisted with identical-looking
logs; the import's `upsert_links_bulk` writes the RESULT/TRAFFIC columns
(proven by a throwaway test); result persistence required the link to be in the
loaded 200-row page; 253/350 phase-2 probes (72%) re-tested hosts phase 1 had
already failed; `real_ping_timeout_secs=5s` was ignored (engine steps use fixed
10 s limits); engine-suspect TLS failures did not reproduce from the same host
(60/60 concurrent engine handshakes OK) and are therefore NOT in scope as engine
defects.

Architecture: `ops/ping.rs` (batch orchestration) → `ops/link_writer.rs`
(write-behind) → `xray_tui_db::{apply_link_patches, upsert_links_bulk}`
(`profile_stats`) with `LinkGroups` disjointness as the writer contract;
`ops/events.rs` (UI-side result handler) keeps the in-memory page in step.

Tech stack: Rust 2024, tokio, toasty 0.10 + turso, tracing.

Baseline / authority refs: decision 4 (schema tag = wipe; no schema change here),
decision 15 (result keying), decision 16 (tier law + labels), decision 20 (native
is runtime-only), decision 22 (write-behind, groups), ADR 0002 (+ its two
amendments), ADR 0003 (stored ordering keys — every link write refreshes them),
`docs/aegis/specs/2026-09-11-write-behind-link-writer-design.md` §4.1 (the group
table this plan restores), `docs/aegis/adr/0004-native-real-ping.md`,
`docs/aegis/specs/2026-09-14-native-ping-pipeline-design.md`.

Compatibility boundary: no schema change, no uid/identity change, no change to
the tier law, the `[name] > [real] > [fast]` label precedence, batch
serialization, `dedup_endpoints`, or `DnsDeferred`/`QueueFull` semantics. New
observable behaviour (only intended): a phase-2 skip for phase-1-hard-failed
links leaves the row at its `[fast]` marker instead of overwriting it with a
`[real]` marker of the same cause; per-result success lines move to `debug`.

TDD Route:
- Mode: off
- Decision: skipped
- Strict authority: not applicable
- Strict signals: none recorded (no explicit user/project strict request)
- Light eligibility: n/a
- TDD-fit exception: n/a
- Test posture: post-change regression — each slice ships the test that fails
  before it and passes after (the DB group contract, the flush remainder, the
  phase-1→phase-2 skip, the probe deadline, the TLS 1.2 P-256 handshake)
- Reason: project default is off and the user did not request TDD
- Verification: focused `cargo test -p <crate> <filter>` per slice, then
  `just quality-gate code` (fmt + clippy + nextest) + the crate suites

Verification: `cargo test -p xray-tui-db`, `cargo test -p xray-tui`,
`cargo test -p xray-tui-native`, `cargo test -p xray-tui-tls`,
`cargo clippy --workspace --all-targets`, `cargo fmt --check`,
`just quality-gate code`.

Change Necessity:
- User-visible need: results from a batch must survive; a subscription refresh
  must not erase measurements or traffic; a real phase must not spend 9 minutes
  re-probing hosts that are down; the configured timeout must be the timeout.
- No-change / non-code option: none — each defect is in code paths that write or
  schedule.
- Why code change is necessary: persistence, scheduling and deadline ownership
  are code.
- Minimum change boundary: `crates/xray-tui-db/src/database.rs` (groups +
  import write shape), `crates/xray-tui/src/ops/{link_writer,ping,events}.rs`,
  `crates/xray-tui-native/src/{probe,error}.rs`, `crates/xray-tui-tls`
  (TLS 1.2 curve), plus docs.
- Decision: code-change

Existence Check:
- Proposed new surface: `LinkGroups::SOURCE` (the group the spec already
  describes but the code lost), one probe-attempt deadline, a `probe_host`
  example in the TLS crate.
- Existing owner / reuse candidate: `LinkGroups` + `link_patch_update_sql` +
  `existing_link_keys` already own "which columns a writer may touch" and
  "does the row exist"; `timeouts::DIAL/SECURITY/PROTOCOL` already own the
  engine's step limits; `examples/grader.rs` is the existing tier-2 tool shape.
- Why existing surface is insufficient: the import path bypasses the group
  machinery entirely (whole-row typed upsert); no caller bounds a whole probe
  attempt; the grader only speaks to tls.peet.ws.
- Creation proof: `SOURCE` is one bit + one match arm reusing the existing
  narrow-UPDATE statement builder; no new dependency anywhere.
- Entropy / retirement impact: net negative — after this plan a `profile_stats`
  writer that skips the group machinery is the exception, not the norm (the
  single-row `upsert_link` keeps whole-row semantics for the connect-time stats
  flush only).
- Decision: add-with-proof

Plan-Time Complexity Check:
- Target files: `crates/xray-tui/src/ops/ping.rs` (≈1.7k lines) and
  `crates/xray-tui-db/src/database.rs` (≈2.3k lines) — both already over budget.
- Existing size / shape signals: `ops/ping.rs` mixes plan, gate driving, probe
  execution and event emission; `database.rs` mixes schema, queries, patch
  writers and bulk writers.
- Owner fit: no new responsibility lands in either — `SOURCE` extends an
  existing group enum, the flush retry extends an existing writer, the phase-2
  skip is a predicate inside the existing phase-2 loop, the batch summary is one
  function next to `finish_batch`.
- Add-in-place risk: low for every slice (no new owner, no new module).
- Better file boundary: not required by any slice; the TLS 1.2 curve work stays
  inside the existing TLS 1.2 driver.
- Recommendation: edit in place; no extraction task in this plan.

## Slice 1 — persistence integrity (P0)

### T1 — `LinkGroups::SOURCE`: an import refresh cannot write test state

Files: `crates/xray-tui-db/src/database.rs` (modify).

Why: today the import writes `latency/latency_delay/latency_ip/speed_bps/error/
error_kind/error_text/traffic_*` from a freshly parsed snapshot, so every
refresh wipes measurements, markers and traffic. The spec's §4.1 group table
requires the import be restricted to `core_type`, `config_type`, `last_seen_at`
(+ insert); the group was lost when the TASK group died.

Change Necessity: the user-visible need is "an import must not destroy test
state"; the code-shape fix is a group-value write, not a config change.

Change (as executed — the original shape added a `LinkGroups::SOURCE` variant
and a chunked existence probe; toasty's branch-specific upsert does both in one
statement, so the variant was dropped rather than added unused):
1. `upsert_links_bulk`: shared setters for the SOURCE columns
   (`core_type`/`config_type`/`last_seen_at`/`updated_at`), and
   `on_create(...)` carrying `latency`/`speed_bps`/`error`/`traffic` — on an
   UPDATE nothing touches the RESULT/TRAFFIC columns, on an INSERT the whole
   snapshot lands (there is nothing to clobber).
2. Doc comment states the contract and the 2026-09-15 evidence.
3. The spec's §4.1 table is amended to match (the group vocabulary stays
   `RESULT`/`TRAFFIC`; `SOURCE` is the import's update surface, `TASK` is marked
   removed).

Verification: `cargo test -p xray-tui-db --lib` — new
`import_refresh_preserves_result_and_traffic` (fails before, passes after) plus
the existing `upsert_link_*` tests.

### T2 — a failed flush window loses only what it attempted

Files: `crates/xray-tui/src/ops/link_writer.rs` (modify).

Why: `flush` drains every staged entry, then returns `Err` at the first failing
chunk; chunks after it are dropped from the local `Vec` and are gone.

Change: on a chunk error, re-stage that chunk AND every later chunk (newer
staged entries keep winning), then return `Err`. Keep the existing log line.

Verification: `cargo test -p xray-tui link_writer` — new
`flush_failure_restages_the_whole_remainder` (3 chunks: make chunk 2 fail.

### T3 — batch results persist without the UI page

Files: `crates/xray-tui/src/ops/ping.rs` (modify), `crates/xray-tui/src/ops/events.rs`
(modify: page-miss path logs at `debug` with the reason, and the handler keeps
its staging for single pings).

Why: `events.rs` resolves the row by scanning the loaded 200-row page; a miss
logs and stages nothing, so ~39% of the run's results vanished while the log
looked identical. `reload_profiles_preserving_selection` replaces that page
wholesale, and an import moves every imported endpoint's ordering keys.

Change:
1. `BatchShared::stage_result(link, test_type, outcome)`: build the RESULT
   snapshot (latency/error exactly as the handler does today) and
   `writer.stage(..., LinkGroups::RESULT)` — called from `emit_result` and from
   `emit_untestable_markers` before the event is sent.
2. `events.rs`: keep the existing handler behaviour (single pings still rely on
   it) but make the not-found branch explicit — `debug` log naming
   `(endpoint_id, protocol_id)` and no behavior change for single pings.

Verification: `cargo test -p xray-tui ping` — new
`batch_stages_every_result_even_when_the_page_lost_the_row` (stub runner + an
empty `state.endpoints`; assert the writer holds a RESULT patch per emitted
result).

### T4 — a busy database retries the batch flush

Files: `crates/xray-tui-db/src/database.rs` (modify).

Why: today's run logged 14 `flush failed: transaction serialization failure`
while a subscription import wrote concurrently; `apply_link_patches` has no
retry while the import path does (`retry_on_busy`).

Change (as executed — a retry inside `apply_link_patches` was rejected: the
driver's `busy_timeout` is 5 s per attempt and the background flush task already
re-attempts every 200 ms, so stacking `retry_on_busy` multiplies a 5 s stall by
six): `finish_batch` retries its final flush `FINAL_FLUSH_ATTEMPTS` (3) times
with a short backoff, and the summary reports `flushes` / `staged-left`.

Verification: `cargo test -p xray-tui-db --lib retry` +
`cargo test -p xray-tui link_writer`.

## Slice 2 — batch efficiency (P1)

### T5 — phase 2 skips links phase 1 already proved unreachable

Files: `crates/xray-tui/src/ops/ping.rs` (modify).

Why: 253/350 phase-2 probes (72%) re-tested hosts whose fast probe had already
failed with the same hard cause; ~226 s of the 555 s phase.

Change: record every phase-1 outcome in `BatchShared`; when phase 2 plans a
link whose fast outcome is a hard failure (timeout / refused / DNS / no route —
the classes a real probe cannot pass), do not schedule it and count it in the
summary. Soft/config failures keep the real probe (a TLS-level fast error is a
different question than reachability).

Verification: `cargo test -p xray-tui ping` — new
`phase_two_skips_links_whose_fast_probe_hard_failed` (stub runner asserts the
real probe was never called for the hard-failed link and was called for the
fast-OK link).

### T6 — the configured real-ping timeout bounds a whole probe attempt

Files: `crates/xray-tui-native/src/probe.rs` (modify), `crates/xray-tui/src/ops/ping_native.rs`
(comment/doc only if needed).

Why: `ProbeRequest::timeout` (from `real_ping_timeout_secs`) bounds only the HTTP
request step; dial/TLS/transport/protocol use the fixed 10 s `timeouts::*`
constants, so a 5 s setting produced `timeout on tcp dial (limit 10s)`.

Change: in `probe::fetch`, run `connect` + `fetch_over` under ONE
`tokio::time::timeout(req.timeout)`, surfacing
`NativeError::Timeout { step: "probe attempt", limit: req.timeout }`. The
engine's inner limits stay as defence in depth.

Verification: `cargo test -p xray-tui-native --lib probe` — new
`probe_attempt_is_bounded_by_the_request_timeout` against a listener that
accepts and never speaks (assert the error is the attempt timeout, elapsed <
timeout + slack).

## Slice 3 — engine gap (P2)

### T7 — TLS 1.2 fallback supports secp256r1

Files: `crates/xray-tui-tls/src/handshake/` (the TLS 1.2 driver) + whatever
holds the curve table (`crypto/`), modify.

Why: deterministic gap reproduced 3/3 against a live server
(`unsupported named curve 0x0017` while OpenSSL completes
`TLSv1.2 ECDHE-RSA-AES128-GCM`).

Change: accept `secp256r1` (0x0017) ServerKeyExchange ECDHE and derive the
premaster secret with ring's P-256 agreement (ring is the workspace crypto
standard); keep X25519 as-is. Reject with a capability-style message only for
curves we still cannot do (e.g. 0x0018 P-384 if it stays out of scope — decide
from the existing curve table, do not invent a new policy).

Verification: `cargo test -p xray-tui-tls` — a hermetic TLS 1.2 handshake test
whose server offers P-256 (the crate's test double), plus the existing TLS 1.2
suite.

### T8 — a live host-probe harness for the transient-failure class

Files: `crates/xray-tui-tls/examples/probe_host.rs` (create), `NATIVE_CORE.md`
(one paragraph in the verification-tiers section).

Why: the batch-time TLS failures (record too large / Finished MAC mismatch /
no certificate) did not reproduce afterwards (60/60 concurrent handshakes OK),
so any "fix" for them would be unverifiable. The harness is what makes the next
occurrence decidable.

Change: an example taking `host port sni [attempts] [alpn]`, dialling with the
engine's plain-TLS path and printing the outcome per attempt (concurrency
included). No behaviour change to the crate.

Verification: run it against a reachable TLS host (10 attempts) and against one
of the recorded suspect hosts; both must print per-attempt outcomes and exit 0.

## Slice 4 — observability (P3)

### T9 — per-result success lines leave INFO, the batch gets one summary

Files: `crates/xray-tui/src/ops/events.rs` (modify: success line → `debug`,
failure stays `warn`), `crates/xray-tui/src/ops/ping.rs` (modify: summary).

Why: one 5-minute batch produced 32,035 log lines, almost all per-result; the
run could not be reconstructed without a manual decode.

Change:
1. `emit_result` / the handler's success logging at `debug`; failures stay
   `warn` (they are the actionable class). Do not touch the core log channel.
2. `finish_batch` logs ONE `tui::ops::ping` INFO summary: planned links, phase-1
   ok/hard-fail, phase-2 scheduled/skipped-by-fast/ok/fail/untestable,
   deferred, queue-full, stopped, and both phase durations.
3. Startup envelope: one INFO line with version, loaded page size and the
   effective speed-test config (timeouts, retries, concurrency, dedup) — the
   values the analysis had to read out of `config.json`.

Verification: `cargo test -p xray-tui ping` — `the_batch_summary_reports_every_counter`
pins the record's fields through the pure `summary_line`, and
`log_startup_envelope` is exercised on the real startup path. (A
subscriber-capturing version of the summary test was written and removed: a
thread-local `tracing` default is not visible to the batch reliably under a
parallel test run, and a flaky test is worse than a formatter test.)

### T10 — parser noise and third-party query noise

Files: `crates/xray-tui-proto/src/proto_spec/common.rs` (modify), `AGENTS.md`.

Why: `Recovered transport type` fires once per parsed row (739 today) and
`toasty::query` slow/failed lines (815) carry no context we own.

Change: demote the transport-recovery warn to `debug` (the recovered value is
already in the message); record in AGENTS.md that the `toasty::query`
slow/failed lines are third-party and that our own write paths log their own
context (`link_writer` flush errors, import per-chunk outcomes).

Verification: `cargo test -p xray-tui-proto` + `cargo clippy`.

## Docs (T11)

Files: `docs/aegis/INDEX.md`, `docs/aegis/plans/…` (this file),
`docs/aegis/specs/2026-09-11-write-behind-link-writer-design.md` (group table:
restore SOURCE, drop the dead TASK row), `docs/aegis/adr/0002-write-behind-link-writer.md`
(amendment 3: the import's group, the flush retry), `docs/aegis/adr/0004-native-real-ping.md`
(amendment: the probe-attempt deadline), `AGENTS.md` (decision 22: SOURCE import
rule, page-independent persistence, phase-2 skip, summary line, TLS 1.2 P-256).

Verification: read-back of each amended section; no stale claim that the import
may write the whole row.

## Risks / retirement

- The phase-2 skip changes what a batch *reports* for unreachable links (stays
  `[fast]`), so a fast probe that fails for a non-reachability reason must keep
  its real probe — the skip predicate is by failure class, not by "failed".
- The SOURCE group makes the import narrower than `upsert_link`'s whole-row
  semantic; `upsert_link` remains for insert-style callers (connect-time stats
  flush) and is now documented as such.
- The probe-attempt deadline can turn a legitimately slow (>config) tunnel into
  a timeout: that is the configured contract; the setting is user-visible.
- Not in scope (evidence says not ours): the transient TLS failures, the FD
  exhaustion at 30k-link scale (bounded by code now, needs a load test), and
  the toasty internal warnings.

## Execution notes (drift from the plan, recorded)

- **T1**: no `LinkGroups::SOURCE` variant — toasty's `on_create`/`on_update`
  branches express the same contract in one statement (no existence probe, no
  second narrow-UPDATE builder). The regression test is
  `import_refresh_preserves_result_and_traffic` (+ `import_insert_writes_the_whole_snapshot`).
- **T4**: bounded retry at the batch boundary instead of `retry_on_busy` inside
  the patch writer (see the ADR 0002 amendment 3, item 3). No dedicated
  failure-injection test: every injected failure costs the driver's full 5 s busy
  wait (measured in the T2 test), and the retry decides only how many times the
  same single call repeats.
- **T9**: the summary test asserts the pure `summary_line` output; a capture
  subscriber proved flaky under parallel execution.
- **T3.3 (from the analysis)**: no transient-vs-deterministic classifier was
  added for real-ping failures. The evidence (60/60 concurrent engine
  handshakes OK against the recorded suspects after the run) says the losses were
  path-level, and phase 2 no longer spends its budget re-probing hard failures —
  a classifier without a repro harness would invent policy. The harness (T8)
  is what makes the next occurrence decidable.
- **P2-1 fixed live**: `xray-tui-tls` TLS 1.2 secp256r1 ECDHE, verified against
  the host that produced `unsupported named curve 0x0017` (3/3 OK afterwards via
  `probe_host`).
- **Not verified**: the FD-exhaustion class (absent on the 09-15 run, bounded by
  `fast_ping_concurrency` + page-scoped enrichment) still needs a 30k-link load
  test with `/proc/<pid>/fd` sampling.
