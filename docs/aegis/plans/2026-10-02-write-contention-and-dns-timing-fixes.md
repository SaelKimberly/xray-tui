# Write contention, import integrity, DNS timing, and log fidelity — implementation plan

Date: 2026-10-02
Status: proposed (awaiting user review)
Input: the 2026-10-01 production run — 4 subscriptions (4,027 / 3,850 / 51,087 / 90,688 links),
`Fast + Real Ping` over 4,028 links, `~/.config/xray-tui/logs.lmdb` (1,313 entries) and `dump-4.log`
(575 lines — see T3).

---

## Aegis Visibility

Four independent owners are implicated (link-writer flush cadence, import batch semantics,
`endpoint_ip` per-address writes, `defer_retry` timing), two hold **accepted ADRs with
amendments** (`0002`, `0010`), and two governing specs are **not frozen** — the monitoring spec
pins `LogVisitor`'s capture set with a test, and the band spec is `draft — awaiting user review`.
Sliced per-finding, each slice would silently amend a different frozen contract.

## Goal

Remove the behaviours the 2026-10-01 run exposed — a write path that commits 1,167 transactions
for 4,028 rows and ~410,000 single-row statements per import, unbounded DNS deferral timing, a log
that cannot be trusted as evidence, and a configuration the user cannot change — and leave behind
a durable harness so the next run is measurable against this one.

## Architecture (unchanged)

No new subsystem, no new owner, no schema change, no migration. Every task edits an existing owner.
The two pieces of genuinely new surface are T11 (a config field) and T1 (a session-only field on
`CoreEvent::TuiLog`); both are additive and neither moves a source of truth.

## Tech stack

Rust 2024, toasty 0.11 / turso 0.7.2 (**WAL**), tokio, heed 0.22 (logs), ratatui. Gates:
`cargo clippy --workspace --all-targets --all-features -- -D warnings`,
`cargo nextest run --workspace`, `just quality-gate`.

---

## Baseline / authority refs

| Area | Owner | Ref |
| --- | --- | --- |
| Profiles page query | `specs/2026-09-11-profiles-page-query-design.md`, `adr/0001` | raw SQL, LIMIT/OFFSET, keyset explicitly deferred |
| Link-writer persistence | `specs/2026-09-11-write-behind-link-writer-design.md`, `adr/0002` + 4 amendments | **canonical**: multi-row upsert, `LINK_STATEMENT_ROWS=400`, STOP-on-error, re-stage failed windows |
| Resolved addresses | `specs/2026-09-15-endpoint-ip-storage-design.md`, `adr/0005` | 908 µs/address, ~1.0 s per 1,105-host pass, "never on the UI task", and its own **"no benefit; not shipped"** verdict on MVCC |
| Journal mode / retry boundary | `specs/2026-09-24-turso-mvcc-rollout-design.md`, `plans/2026-09-24-turso-mvcc-production-rollout.md` | opt-in MVCC; **"retry behavior at every production transaction owner" is the minimum safe boundary** |
| Active band | `specs/2026-09-24-profiles-view-band-design.md` (**draft**), `adr/0010` | `(band, rank_host, endpoint_id)`; release A/B at N∈{7.6k, 50k, 200k}, **50k/200k pending** |
| Log capture | `specs/2026-09-24-db-query-monitoring-design.md`, `plans/…-db-query-monitoring.md` Task 5 | capture set pinned to `duration_ms` + `db.statement` **with a pinning test**; `db.params` opt-in via `.log_statement_params(bool)`; non-goal 10: **never persist params to heed** |
| Raw SQL | `docs/database-manual-sql.md` | every hand-written statement needs a recorded cause + measurement |
| Measurement lab | `adr/0008-batch-feed-scaling.md`, `ops/ping/flow_cost.rs` | `cargo test -p xray-tui --release --lib -- --ignored --nocapture flow_cost`; `XRAY_TUI_MEASURE_DB`, `XRAY_TUI_SCALE` |

**Unowned findings.** Import batch semantics (500-URL chunks, `persist_batch`, the 64 MiB budget)
and `purge_expired` have **no owner in `docs/aegis/`**. T5 and T6 create the first record for the
import batch contract and attach to `specs/2026-09-17-batch-ping-pipeline-design.md`; T8 records
`purge_expired` under `specs/2026-09-24-profiles-view-band-design.md`; and T11 needs a short spec
before it implements.

---

## Corrections carried from the analysis

1. **The run was WAL, not MVCC.** `~/.config/xray-tui/data.db` header bytes 18/19 = `2 2`
   (`read_version=2`); MVCC requires 255. No `XRAY_TUI_TURSO_CONCURRENT_WRITES` was in play. This
   is the contention-heavy real-feed observation the MVCC spec's §88 gate is waiting for.
2. **The two contention classes are distinct and must not be merged.**
   - `database snapshot is stale` — `turso_core-0.7.2/storage/wal.rs:3311-3319`, a failed
     read→write **upgrade**. Returns immediately. The engine's own comment: *"Retrying with
     busy_timeout will NEVER HELP."* 88 occurrences, all in t+0–30 s.
   - ~5,000 ms failures — `LimboError::Busy` at `wal.rs:3308`, governed by
     `PRAGMA busy_timeout=5000`. 10 occurrences, 12:20:11–12:22:13, each burning 5 s × 5 attempts.
3. **F1 is a latent defect, not realised data loss.** `stream_import.rs:376-385` swallows a batch
   that exhausts its retries and returns `(0, batch_summary)`, so `links` under-counts and
   `ended_early` stays `None` — a clean group status. **But heed contains zero
   `bulk persist failed` lines**, so no batch exhausted its retries in this run. The 7
   `upsert_endpoints_bulk fails=7` are *individual attempts* absorbed by `persist_batch`'s own
   `retry_on_busy` (a different span, so the db-monitor `retries` field for
   `upsert_endpoints_bulk` reads 0). The defect is real and must be pinned by a test; it is not
   the top severity item and no user data was lost.
4. **The DNS failure window stays armed at report time.** `AGENTS.md` is explicit: *"The window
   is measured from the moment a lookup REPORTS, so it MUST outlast `enrich::DNS_LOOKUP_TIMEOUT`
   (8s) or it can never defer anything."* Arming at request time would also let a 30 s-queued
   failure's window be expired at report, probing a host that just failed. T8 bounds the queue
   instead and resets the retry clock.
5. **The journal mode is not the lever for the write-path defects.** MVCC makes a transaction
   fast; it does not reduce 1,167 transactions to 8, or ~410,000 statements to ~45,000. T3–T6 are
   mode-independent. And the endpoint_ip spec already measured MVCC's reader latency during a
   flush as **"no benefit; not shipped"** (`spec:132`).
6. **T11 (the A/B) is last, on purpose.** The recorded microbenchmark is a **32-row competing
   writer**; this run had ~1,000. Running it before T3/T4/T6 would measure the flush pathology
   under both modes and inflate the MVCC win.

**Project-measured baselines reused below** (verified in the governing specs):

- `turso-mvcc-rollout-design.md:44-45` — WAL *"competing writer hit 5,000 ms busy timeout in
  5/5 samples; 0 successful competing commits"*; MVCC *"completed 0.170–0.197 ms in 5/5 samples;
  0 conflicts"*.
- `turso-mvcc-rollout-design.md:61` — *"This is a production-shaped DB-API workload, not a
  real-feed result. It did not reproduce the reported long wait: WAL completed every window and
  geo flush. Therefore MVCC's contention mechanism is real, but its measured cost is not
  negligible in this mix. Default enablement is not justified by current evidence."*
- `turso-mvcc-rollout-design.md` §DB-API A/B — **total wall +32%, import p50 +47%**.
- `turso-mvcc-rollout-design.md` §Retry/checkpoint — `finish_batch` skips
  `PRAGMA wal_checkpoint(PASSIVE)` for MVCC handles, and the probe shows that call failing.
- `endpoint-ip-storage-design.md:217-219` — **908 µs per address**; *"a full resolution pass over
  1,105 DNS hosts therefore moves from ~0.35 s to ~1.0 s of write work."*
- `endpoint-ip-storage-design.md:113` — **425 µs/insert**, 8.5 s per 20,000. Applied to this run's
  ~410,000 single-row import statements: **~174 s of pure statement time**, consistent with the
  measured 5m24s import.

### Commits landed during planning — anchors re-verified, plan unchanged

`850abb7 feat(db): static config weight as an ordering prior` and
`cfcb248 docs: add note about quic mode for v2ray plugin` (both **2026-10-02 11:36**, i.e. *after*
the 2026-10-01 run, so the run's binary was a working-tree build of this code) touch
`endpoint_rank.rs` (+515), `models_toasty.rs` (+85), `profiles_query.rs` (+19), `ping.rs` (+131)
and add `proto_spec/weight.rs`. Re-checked against `HEAD`:

- **Every line anchor in this plan still holds.** `ping.rs` `defer_retry` `:1968`, budget `:1983`,
  first sleep `:1987`, warn `:2015-2020`, 250 ms poll `:2033`, `defer_delay` `:2567`;
  `enrich.rs` permit `:310` / timeout `:342`; `endpoint_rank.rs` `endpoint_rank_test` `:378`,
  in-place-edit trap `:410`, `endpoint_rank_test_v2` `:415`, band indexes `:431`/`:435`;
  `profiles_query.rs` `PageSort::Test` terms `:239-247`. The +131 `ping.rs` lines landed **outside**
  `defer_retry`.
- **The other eight files this plan anchors to are untouched**: `main.rs`, `enrich.rs`,
  `stream_import.rs`, `link_writer.rs`, `retry.rs`, `flow_cost.rs`, `state.rs`, `ui/logs.rs`.
- **`SCHEMA_VERSION` is still 14** — the weight used a nullable-add + backfill
  (`WEIGHT_COLUMN`, `endpoint_rank.rs:406`), not a tag bump, so no wipe and the Compatibility
  boundary's "no schema bump" claim stands.
- **T10's premise is restated, not demoted.** The static weight is now the **third term** of the
  Test sort, a DESC over an 8-byte BLOB. The weight spec's §Index shows `endpoint_rank_test_v2`
  served as a covering scan — but on a **4,000-endpoint synthetic feed at an unrecorded offset**,
  against a live DB of **74,014 endpoints (18.5×)** whose 1,004 ms came from *consecutive*
  deep-offset page-walk statements. Scale **and** offset both differ, so **no super-linearity ratio
  is claimed**: the Test sort is simply **not yet measured at this scale at a known offset**, and the
  lab's offset sweep is what attributes the cost. T10 stays; its DDL stays conditional on that.

---

## Context that is NOT a workstream

**The real-ping success rate is a feed property, not a defect.** Measured across four independent
runs: `1 ok / 3,381` (2026-09-17), `9 / 996` = 0.9% (2026-09-22), `75 / 4,498` = 1.7% (2026-09-22),
`61 / 3,075` = 2.0% (2026-10-01). The actionable real-level finding is **wall time and
concurrency** (T12), not success. `plans/2026-09-17-ping-run-analysis.md:174` already records the
mechanical consequence: `dedup_endpoints` retires sibling links only on a *success*, so at ~2% it
essentially never fires and the real level pays for every sibling. Recorded here so the next
reader does not open a workstream for it.

---

## Compatibility boundary

- **No `SCHEMA_VERSION` bump.** No task adds, drops, or renames a column. Bumping the tag wipes
  SQLite (decision 4) and none of these require it.
- **`LogMessage` (heed) is not modified.** Non-goal 10 of the monitoring spec forbids persisting
  monitoring detail. T1 enriches the **session-only** `CoreEvent::TuiLog` channel, which already
  carries a `persisted: bool` distinguishing the two deliveries.
- **New config field only (T12).** No `deny_unknown_fields` anywhere, so an existing `config.json`
  loads with the default. Nothing is renamed.
- **New index name only (T10).** `CREATE INDEX IF NOT EXISTS` makes a changed column list a silent
  no-op on an existing database — the trap recorded at `endpoint_rank.rs:409-414`. Never edit
  `endpoint_rank_test_v2` in place.
- **Retry is never removed.** The MVCC plan's minimum safe boundary is *"retry behavior at every
  production transaction owner"*. T4 adds jitter; T5 changes only the post-exhaustion path. Neither
  deletes a retry.
- **The import failure window is not re-based.** `AGENTS.md`'s report-time rule stands (correction 4).

---

## Change Necessity

No change / docs-only is insufficient for every task: T3 and T4 burn write transactions and
statements that no configuration can bound; T8 is arithmetic inside `defer_retry`; T1 loses fields
before the `LogMessage` is built; T2 loses entries at shutdown; T5 hides a dropped batch behind a
clean status. Minimum boundary: **15 tasks, all edits to existing owners, one new config field.**

## Ripple Signal Triage

- **Producer/consumer:** T1 (tracing → `LogMessage` + `TuiLog` → panel) and T2 (heed → `log_cache`
  → export) cross a producer/consumer seam. Canonical owners: `main.rs` (capture),
  `log_heed.rs` (storage). No duplicate owner is retained.
- **Contract amendment:** T1 amends `specs/2026-09-24-db-query-monitoring-design.md` **and its
  pinning test**; T3 amends `specs/2026-09-11-write-behind-link-writer-design.md`'s stated trigger.
  Both are recorded as spec amendments in the task, not applied silently.
- **Draft spec:** T10 extends `specs/2026-09-24-profiles-view-band-design.md`, which is `draft —
  awaiting user review`. It is **not** treated as frozen; if the user has pending edits there, T10
  waits.
- **Fallback:** none is introduced. T1/T2/T5 replace a lossy path rather than adding a second one.

## TDD Route

Mode **auto** (no project-wide TDD setting; no explicit user TDD request). Recorded per task:

| Task | Decision | Reason |
| --- | --- | --- |
| T1, T2, T4, T5, T6, T7, T8, T11 | **strict** | bugfix / contract / persistence / producer-consumer / new surface |
| T0, T3, T9, T10, T13, T14 | **light** | single owner, additive or no behaviour change, obvious focused check |
| T12 | **light** | a decision recorded from `flow_cost_contention`; no product code beyond a log line |

T11 is **strict**, not light: it introduces a new `AppConfig` surface and amends the import
budget's contract, which this route classifies as contract work. T2's route was
**diagnose-first** and resolved to **strict** — the mechanism is proven
(`main.rs:254-257`), not assumed.


---

## Verification

**Every task:** `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
`cargo nextest run --workspace`.

### Measurement database — re-copy first (correction to the first draft of this plan)

`~/.config/xray-tui/data_copy.db` is from **2026-09-17** and holds a different, smaller feed
(27,142 endpoints / 56,140 links). Using it would measure a third scale against the baseline
below. **T0 re-copies** the live DB and names the artifact:

```
sqlite3 ~/.config/xray-tui/data.db ".backup '/tmp/xraytui-measure-2026-10-01.db'"
```

Live scale at the time of this plan (`sqlite3 … "select count(*) from endpoints"`):
**74,014 endpoints · 145,268 profile_stats links · 73,666 rows with `band = 0` (99.50%)**.

### Harness commands

```bash
# page-path sweep — already sweeps (Address|Port|Test) x offsets {0, total/2, N-200}
# over PurgatoryView::Active, 5 reps, with the band=0 seek A/B immediately after
XRAY_TUI_MEASURE_DB=/tmp/xraytui-measure-2026-10-01.db XRAY_TUI_SCALE=1 \
  cargo test -p xray-tui --release --lib -- --ignored --nocapture flow_cost

# NEW in T0 — import+geo contention mix, mode-switchable, WAL vs MVCC
XRAY_TUI_MEASURE_DB=/tmp/xraytui-measure-2026-10-01.db \
  cargo test -p xray-tui --release --lib -- --ignored --nocapture flow_cost_contention
```

### Baseline to beat (2026-10-01, WAL, 74,014 endpoints / 145,268 links)

| Metric | Baseline |
| --- | --- |
| `profiles_page` (Test sort) p99 | 1,004.3 ms |
| `flushes` per 4,028-row batch | 1,167 commits (3.45 rows/tx) |
| `set_endpoint_ip_country` | 790 inv → 8,608 stmts, p99 1,051.1 ms, **88 aborts** |
| `country persist failed` | 88 (dump showed 28 — export truncation, see T2) |
| Statements at `busy_timeout` | 10 (7 endpoints INSERT + 3 endpoint_ip INSERT) |
| `bulk persist failed` | **0** — F1 did not fire (correction 3) |
| `purge_expired` slowest | 3,098 ms |
| Single-row import upserts | ~410,000 statements (105,147 + 99,728 + 204,592) |
| `deferred` / booked unprobed | 1,577 / 12 |
| `query failed` / `slow query` | 126 / 40 |
| heed entries per session | 1,313 (79% `tui::ops::enrich`) |


### T0 lab baseline — 2026-10-02, WAL, `/tmp/xraytui-measure-2026-10-01.db`

74,014 endpoints · 145,268 links. `reps=1`, `FANIN=16`, `GEO_SINGULAR_ROWS=320`,
`GEO_FANIN_TASKS=32`, trickle 4,028 @ 12/s. **136 write failures.**

> **History — an earlier run's two import rows (2,450 ms and 41,489 ms) are DISCARDED.** That
> run's slice was unbounded: the page loop's condition was `links < want || geo < want_ips`, and
> stored countries are sparse, so the geo requirement drove extra paging until the slice hit
> 43,991 links / 22,000 endpoints / 44,179 protocols / 43,991 group links. One "import
> transaction" then wrote ~154,000 rows, ~62× a production batch, so those figures described that
> shape and not the import path. A second defect compounded it: `links_per_tx` only sized the
> slice, because every transaction replayed the whole slice, so no transaction ever matched a
> production batch. Both are fixed — the slice is bounded to `want_links`, the four families
> accumulate as whole per-endpoint units so truncation cannot orphan a row, and `split_batches`
> builds `import_tx` real ~`links_per_tx`-link transactions. The table below is the corrected
> shape, measured at `links_per_tx=500` to match production.

| Row | Measured | n | Valid | Read as |
| --- | --- | --- | --- | --- |
| seq import tx (4 families) | ~~32.03 ms~~ | 8 | **✗** | **every transaction FAILED** — see below |
| seq geo flush **BATCHED** (100 rows) | 3.11 ms | 2 | ✓ | **31.1 µs/row** — T7's *fix* |
| seq geo **PER-ADDRESS** (pre-fix) | 0.108 ms | 144 | ✓ | **108 µs/address** — T7's *before* |
| **fan-in geo PER-ADDRESS / 32 writers** | **10.5 ms** | 1 | ✓ | the concurrent single-row-writer shape |
| fan-in import+geo / 16 writers | ~~3,971 ms~~ | 1 | **✗** | same — nothing was written |
| flush trickle 4,028 @ 12/s | 340 s → **1,669 flushes, 2.41 rows/flush** | 1,669 | ✓ | **T3's baseline** |

Per-arm failure split, from the run where the instrument could finally attribute them:
`seq import tx` **FAILED ON ALL 8 SAMPLES — row discarded** · `fan-in import+geo` **NO SAMPLES —
row discarded** · `seq geo flush BATCHED` 0/2 · `seq geo PER-ADDRESS` 0/144 · `fan-in geo
PER-ADDRESS` 0/144. The three surviving rows are the only numbers quoted.

**What these numbers say, precisely:**

- **T3 is validated and reproducible, three times.** Independent runs produced **1,668 / 1,668 /
  1,669 flushes for the same 4,028 rows** at 12/s — against production's 1,167. The pathology is stable
  and the measurement is not noise. **Achieved: 32 flushes / 125.9 rows-per-commit — a 52×
  reduction from 1,669.** The plan's original `commits ≤ 20` target is **amended to ~32**: ~16
  commits needs a 256-row floor, which is ~21 s of staging at the observed rate, and that is a
  durability decision rather than a tuning one.
- **T7's isolated win is 3.5×, and it is not where production's cost was.** 31.1 µs/row batched vs
  108 µs/address per-address, **with zero contention**. Production's p99 was **1,051 ms per
  address** — ~9,700× the isolated per-write cost. The per-address writer is only mildly
  expensive; the damage was the **contention multiplier**, so T7 must be judged on the concurrent
  row, not the sequential one.

**BLOCKER, resolved in the code: every import transaction failed, and the 136 were the import
attempts.** 8 sequential + 16×8 fan-in = **136**, identical across two runs whose geo arms
carried different row counts (1,600 then 144) and contributed **zero** failures. Cause:
`upsert_protocols_bulk` (`database.rs:1768-1776`) refuses a `Protocol` whose deferred carriers
are unloaded, and `load_page_rows` returns exactly that — the page path deliberately leaves
`config`/`transport.data`/`security.data` unloaded. So every `write_import_once` aborted at the
second of four families, and **32.03 ms and 3,971 ms measure a failed guard check, not a write.**
The "0.97 wall/serial, clean serialization" reading is an artifact and is withdrawn.

**Consequence: T6 has no valid before, and the import arms are not buildable from the measure
database.** A loaded `Protocol` requires `.include(Protocol::fields().config())` through a toasty
query; there is no public loader. Production's import rows come from
`state::protocol_from_parsed` — built in memory from the parse boundary, hence loaded — so the
import path cannot be replayed from DB rows at all. T6 therefore needs its own probe (or leans on
the spec's recorded 360 ms vs 44.3 ms per 2,000 links). This is **not** a defect in the import
code; it is a limit on what a database-resident lab can measure.

**Instrument hardening, so this class of failure cannot recur silently.** Every arm now owns an
`ArmFailures` counter and prints its own tally; an arm that failed on **every** sample, or took
**no** samples, is named and its table row is **dropped rather than averaged in**. The shared
counter is what let a run in which every transaction failed report a plausible millisecond
figure twice.

**T12's premise is UNSATISFIABLE with this harness — decide the fallback now, not at T12.**
Per-arm reporting resolved the 136: they were **8 + 16×8 = 136 import attempts, 100%**, with the
geo arms contributing **0/2, 0/144 and 0/144 failures**. So this lab reproduces production's
**cost** shape, not its **failure** shape: 32 concurrent single-row writers on a 144-row slice
complete cleanly, and 8/16 writers never collide at all.

Therefore T12's "reach the failure regime, then A/B the modes" **cannot be met here**, and
**"no mode shows contention" is a real, expected outcome** — not a failure of the benchmark.
Decided in advance:

- **If neither mode fails at the lab's scale, T12 does not promote MVCC to default.** The spec's
  §88 gate asks whether avoided waits outweigh the tax; with no waits observed there is nothing
  to outweigh, so the gate is **not cleared** and WAL stays default.
- The recorded outcome is then: *the contention observed on 2026-10-01 is not reproducible below
  ~1,000 concurrent single-row transactions, which this harness cannot reach from DB rows.*
  That is a finding about the **feed's** concurrency, and it points at the real fix — reduce
  transaction count (T6/T7), not change the journal mode.
- T12 therefore needs a **purpose-built concurrent probe** (many tasks, each a single-row write,
  on a scratch DB) to reach the production regime. That is new work and is scoped at T12, not
  assumed here.

**Whole-plan acceptance (after T10):** re-import the same four subscriptions and re-run
`Fast + Real Ping (All Profiles)` on the same machine, then diff the `d` key's db-monitor dump and
the batch summary against the table above. **T2 must land first or the evidence is partial again.**

---

## Tasks

### WS-0 — Make the instrument trustworthy (do first; unblocks verification of everything else)

#### T0 — promote the disposable import+geo probe into a durable, mode-switchable lab
**Keystone. Light.** `flow_cost` covers the page path and the network path. The MVCC spec's
import+geo probe (8×800-link import transactions + 16×100-row country flushes, 5 repetitions,
WAL-vs-MVCC) was explicitly **disposable** and is now gone — which is why the open "production
workload benchmark" has no harness and why T3/T4/T6/T7 have no before/after. That probe is
exactly this run's workload.
- **Minimum change:** a new ignored `flow_cost_contention` row in
  `crates/xray-tui/src/ops/ping/flow_cost.rs` with **four** rows: sequential import tx, sequential
  geo flush, a **concurrent fan-in** (import and geo OVERLAPPING — the only shape that reproduces
  the `snapshot is stale` aborts; every other row in that module is sequential), and a **link-writer
  flush trickle** on a wall-clock *arrival* schedule reporting `flush_count()`. Rows-per-flush is
  set by arrival rate, not row count: a tight 4,028-row loop trips the 512-row size trigger and
  yields ~8 flushes, while the production trickle produced 1,167.
- **The import transaction replays all FOUR bulk families.** `upsert_links_bulk` is the one already
  chunked at `LINK_STATEMENT_ROWS`; the three per-row writers are T6's subject. Replaying only the
  links would measure the writer that is *not* broken and make the row useless as T6's before/after.
- **Non-mutating by construction:** rows are replayed with the values they already hold, and an
  address whose country is `None` is **skipped, never given a placeholder** — `set_endpoint_ip_countries`
  takes a concrete `String`, so a fake ISO code would land in the very column the WAL-vs-MVCC
  comparison reads. Addresses are seeded from `endpoint_resolutions` (`endpoint_ip`), not the page
  row's `resolved_ips`, so the replay hits the UPDATE-miss / INSERT-race path the aborts came from.
  Slice clones are hoisted above the timed regions; cloning them per writer would rival the
  collision the row exists to quantify.
- **The MVCC arm cannot use this path.** An existing file stays WAL (`read_version=2`); MVCC needs a
  fresh file under `XRAY_TUI_TURSO_CONCURRENT_WRITES=1`. T12 reuses the synthetic-feed builder in
  `measure_page_scale` for that arm — never a second copy of the same WAL file.
- **Why first:** without it, T3/T4/T6/T7 are unmeasurable and T12's A/B has nothing to run.
- **Verify:** the row runs green against the fresh measure DB, prints the journal mode, and its
  numbers are recorded in the baseline table **before any write-path task starts** (sequencing rule
  at the end of this document).


**Operational hazard — the measure copy is destroyed, not rejected, on a bad tag.**
`Database::open` reads `PRAGMA user_version`; on a mismatch it calls `push_schema()`, and if
that call errors it **drops the file and rebuilds** (`database.rs:274-288`). A copy whose tag
reads as anything but `SCHEMA_VERSION` is therefore *destroyed* rather than refused. Observed on
2026-10-02: two lab runs sharing one `/tmp` copy left a **114.6 MB → 4 KB** file with zero rows,
and the second run correctly reported `slice: 0 links` and skipped rather than writing into
nothing. Three rules for T0, T10 and T12:

1. **One process at a time** on a measure copy. Never two lab runs, and never a lab run while a
   second process has the file open.
2. **A fresh copy per run**, because the lab writes.
3. **Verify `user_version` after the backup and before any `Database::open`** — the backup must be
   taken from a quiescent database (an empty `-wal`, no app running).

`sqlite3 "file:…?mode=ro" ".backup …"` then
`sqlite3 "file:…?mode=ro" "select * from pragma_user_version"` is the check. The live `data.db` was
verified intact throughout (74,014 endpoints / 145,268 links, `user_version=14`).


#### T1 — `LogVisitor` drops every field but two, and the loss is permanent
**F6. Strict.** `main.rs:31-68` keeps only `message`, `duration_ms`, `db.statement`; everything else
hits `_ => {}`. `ping.rs:2015` emits `endpoint_id`, `half`, `waited_ms` for the stuck-deferral
warn — gone **before** `LogMessage` is built, so the 12 rows in heed carry no ids and no
store-side change recovers them. `db.statement` also carries no bind values, so the page query's
`OFFSET` is invisible and T10's mechanism question cannot be answered from a log.
- **Minimum change, use the sanctioned mechanism — do not hand-roll bind parsing:**
  1. Wire toasty's existing `.log_statement_params(true)` on the three `Db::builder()` sites
     (`database.rs:352`, `:418`, `:2324`). It is named in the monitoring spec (`:21`, `:23`) and is
     **referenced nowhere in this tree** — the output lands in the `db.params` field the visitor
     then reads. Enablement is global, so no bespoke `profiles_query` parsing and no second
     opinion about what a bind means.
  2. Add `detail: Option<String>` to `CoreEvent::TuiLog` (`types.rs:494`) and `LogLine`
     (`types.rs:241`); render the visitor's remaining scalar fields plus `db.params` into `detail`
     in `main.rs`; send `detail` on the **session-only** `TuiLog` channel and **never** into
     `LogMessage`.
- **Contract amendment, in this task or it gets reverted:** `plans/2026-09-24-db-query-monitoring.md`
  Task 5 pins the capture set, and `specs/2026-09-24-db-query-monitoring-design.md` owns it **with
  a pinning test**. Amend the spec text **and** that test in the same commit, recording why.
- **Non-goal 10 (`spec:43`, `:80`):** monitoring detail is never persisted to Turso or heed. The
  `detail` field rides the session channel only.
- **Repair track:** a test asserting an event with `endpoint_id` yields a `detail` carrying it, and
  that `LogMessage.message` stays bare.
- **Retirement track:** the silent `_ => {}` drop is retired. No fallback.
- **Verify:** the test; a stuck-deferral warn shows its id in the Logs tab.

#### T2 — the export copies the view, and shutdown drops the in-flight batch
**F12. Strict. Mechanism proven, not assumed.**
- **Export is the Logs-tab `Y` key** (`ui/logs.rs` `copy_all_filtered`), which serializes
  `state.log_cache` only; `clipboard_line` (`:644-657`) is byte-identical in shape to the
  `dump-4.log` line format. `config.json` has `log_to_file=false`, so the JSON file writer is off;
  `dump-1.log`/`dump-2.log` being exactly 10,000 lines is the `log_cache` cap
  (`ops/events.rs:474-475`). There is no separate dump path.
- **The missing 738 entries are the OLDEST, not the middle.** `load_initial_logs`
  (`state.rs:666-697`) seeds `log_cache` with the **500 newest** (`:671`) and then advances the
  watermark to the newest of those (`:684-686`), so everything older sits behind a watermark the
  forward poll (`ui/logs.rs:463`) can never cross. Corroboration: heed holds
  `06:52:01.357 "batch: planned 4028 link(s) over 14 page(s)"` and the concurrency warning; the
  dump's first line is `06:52:13.88`. The dump's oldest line is not the store's oldest
  (`06:51:50.946`).
- **Second, independent defect:** the writer loop's shutdown branch (`main.rs:254-257`) `return`s
  **without flushing the in-flight batch**; only the `Disconnected` branch (`:263-270`) flushes.
  A quit inside a 500 ms batch loses up to 100 lines.
- **Third defect, and it compounds the other two:** `enrich.rs:349` logs every DNS resolution at
  `info` — **933 of the 1,313 stored entries (79%)** this session were that one line. The
  information is already durable in `endpoint_ip` and `endpoints.resolved_at`, and `ping.rs` made
  its per-result lines session-only `debug` for exactly this reason. The same flood is what fills
  the 10,000-row `log_cache` cap that truncates the export above, so the retention fix and the
  export fix are one task, not two.
- **Two checks to run first, and record the result:**
  - `dropped_logs` (`main.rs:129`) is the bounded `try_send` drop counter. The channel is
    `sync_channel(4096)` (`main.rs:188`) — **`AGENTS.md` is stale in calling it unbounded**. With
    a 933-line DNS burst in 30 s this is unlikely to drop, but assert it is 0; a non-zero value
    means 1,313 is itself a floor.
  - Confirm the artifact's provenance per the export path above rather than assuming it.
- **Minimum change:** an export that pages heed to exhaustion (`read_older_than_async`, `:393`)
  instead of copying `log_cache`; surface `log_has_older` (`state.rs:673`) so a truncated export
  is never silent; flush the in-flight batch on the shutdown branch; demote `enrich.rs:349`
  `info!` to `log_activity` (session-only) or `debug`; add a `read_all`-style API to `log_heed.rs`
  (none exists — the three readers are `:227`/`:249`/`:273`, all `take(limit)`).
- **Verify:** after a 1,300-entry session an export yields 1,300 lines, not 575; a regression test
  seeds > 500 entries and asserts the export length; a quit mid-batch loses nothing; a batch run
  stores < 5% of entries under `tui::ops::enrich`.

#### T3 — the flush timer commits on a single staged row
**F2. Strict.** `link_writer.rs:256-260`: `select!{ wake | sleep(200ms) }` → flush if
`staged_len() > 0`. No floor on the timer path. **1,167 commits for 4,028 rows** (3.45 rows/tx
against a 512-row design target), 3.57 commits/s through the whole 327 s batch.
- **Minimum change:** a row floor on the timer path plus idle backoff. The size-triggered `wake`
  path and the `flush_interval` value are unchanged.
- **Contract:** the error path is untouched — `adr/0002`'s STOP-on-error, multi-row upsert,
  `LINK_STATEMENT_ROWS=400`, and the MVCC spec's "re-stages failed windows" invariant all hold.
  `specs/2026-09-11-write-behind-link-writer-design.md` states `flush_interval` as the trigger and
  is **amended** by this task; record the amendment.
- **Repair track:** two tests, both **scaled down** — the production shape (4,028 rows at 12/s) is
  minutes of wall clock, and `flush_rows` drives *both* the size trigger and the floor, so the
  numbers must be chosen for the floor to sit strictly inside the row count:
  - `a_trickle_coalesces_at_the_floor_not_every_tick` — `FLUSH_ROWS=128` → floor 32, `ROWS=64`,
    5 ms interval. Asserts `flush_count()` in **2..=3**, so a regression that ignores the floor and
    writes per tick (~64) fails loudly. Each stage must use a **distinct `endpoint_id`**:
    `StageKey` is `((protocol_id, endpoint_id), group)`, so re-staging one link coalesces into a
    single entry and the floor is unreachable — that defect made an earlier version of this test
    measure the *deadline* while claiming it measured the floor.
  - `a_trickle_below_the_floor_is_still_written_by_the_deadline` — `ROWS=8` under the floor, so the
    deadline is the only trigger; asserts nothing is written until `max_staged_age` elapses, then
    that it is.

  This is the **only** case where `flush_rows` is large enough for `TIMER_FLOOR_DIVISOR` to matter —
  the pre-existing tests pass `flush_rows = 1`, which floors to 1 and would not notice a regression.
- **Second consequence, beyond durability: the ordering keys go stale for the same window.**
  `endpoint_rank::refresh` runs **inside `apply_link_patches`, after the commit**
  (`database.rs:1182-1190`) — that is, at **flush** time, not at `stage`. So deferring the
  flush by up to `max_staged_age` defers the *stored* decision-16 keys by the same amount, and a
  Test-sorted page refetch during a batch (`filter_cache_valid = false`, `events.rs:770-772`)
  places each row by its **stale rank** for up to ~10.7 s, where it previously lagged ~200 ms.
  The visible Test cell and the expanded sub-table order are **unaffected** — `row
  .sort_links_by_test_priority` is patched in memory at event time (`events.rs:729-749`) — so
  this is only **a row's position in the window**. `adr/0008` §3 sized that refetch around UI
  cost and never considered staleness, so the interaction is undocumented there. Judgement: ~10.7 s
  of window-position lag during a live batch is acceptable for a 52× cut in write transactions; a
  refresh at `stage` time instead would reintroduce per-row writes, which is what ADR 0003
  retired.
- **Retirement track:** the bare-timer flush is retired; the size trigger is retained as the only
  unbounded-latency path. No fallback timer.
- **Single (non-batch) pings: the floor must not hold them.**
  Every non-test `flush()` site is connect-disconnect (`connect.rs:653`), `finish_batch`
  (`ping.rs:1444`), reload
  (`profiles.rs:217`) or quit (`ui/mod.rs:237`) — **nothing flushed when a manual TCP/Real ping
  finished**, and a manual ping stages 1-3 column groups, far below the 128-row floor. So a
  user-initiated result would have sat until `max_staged_age` (15 s), with its `endpoint_rank`
  refresh deferred too, while the Test cell *looked* correct (in-memory patch) until a reload or
  restart. Fixed by `LinkWriter::flush_soon()` — a `wake.notify_one()` at
  `testing_profiles.is_empty() && batch_progress.is_none()` (`events.rs:789`) — **not** by a
  `flush().await` there: `draining_results_performs_no_commit_on_the_ui_task` is a standing guard
  that draining a result must not commit on the UI task, and the loop's size-trigger arm already
  writes unconditionally, so a wake buys the latency without violating it. The `batch_progress`
  guard keeps this out of the batch path, where coalescing is the point.
- **Verify:** the test; a re-run shows `flushes` in the tens.

#### T4 — un-jittered backoff across ~1,000 concurrent writers
**F3b. Strict.** `retry.rs:34` sleeps `20 << attempt.min(6)` ms — 20/40/80/160/320, 620 ms total,
**no jitter**. Every conflicted writer retries in lockstep, so the DB never drains; this is what
turns one contention event into 88 aborts. The MVCC plan explicitly permits adding jitter
(minimum safe boundary forbids only *removing* retries).
- **Minimum change:** full jitter, preserving the 5-attempt count and the ~620 ms ceiling. Derive the
  jitter from an existing process-local source rather than adding a dependency.
- **Repair track:** a test asserting N concurrent conflicting calls produce ≥ N distinct sleep
  buckets.
- **Verify:** the test; `retries`/`fails` in the db-monitor dump fall while throughput holds.

### WS-1 — Import integrity

#### T5 — a dropped 500-URL batch reports a clean success (latent, did not fire this run)
**F1. Strict.** `stream_import.rs:376-385`: on final failure it logs
`bulk persist failed for a {}-URL batch` and returns `(0, batch_summary)`. `links` then
under-counts, `ended_early` stays `None`, and `record_import_result` sets `GroupStatus::Ok` — up
to 500 URLs gone behind `Subscription updated: N profiles`. **Zero occurrences this run**
(correction 3), so this is a latent path that must be pinned, not a live loss.
- **Minimum change — reuse the existing channel, do not add an error surface:** have `persist_batch`
  return a dropped-URL count (it already returns `(links, summary)`; make it
  `(links, dropped, summary)`) and fold it into `ImportOutcome.ended_early` in
  `run_streaming_import`. `partial_import_message` → `record_import_result` then makes the group
  red through the path that already exists.
- **Count semantics (must be stated in the message):** on a budget breach `links` is the number
  **stored**, not the number present in the feed. The message must not imply the feed's size.
- **Repair track:** a test that forces `persist_batch` to fail after retries and asserts
  `status == Error`, that `stored + dropped == attempted`, and that the success line is not
  emitted.
- **Retirement track:** the swallow is retired.
- **Verify:** the test.

#### T6 — ~410,000 single-row upserts on the import path
**F9. Light (additive SQL), largest diff.** `upsert_endpoints_bulk`, `upsert_protocols_bulk`,
`upsert_endpoint_group_links_bulk` loop one typed upsert per row. Only the link writer chunks
(`LINK_STATEMENT_ROWS=400`, `database.rs:499`). Measured 105,147 + 99,728 + 204,592 statements
across 12:20–12:25 and 29 `slow query` at 1.0–4.7 s. `AGENTS.md` decision 22 **already records**
the measurement — *"the import path's per-row typed upserts 360 ms per 2,000 links against
44.3 ms"* — and the fix was never applied to this path.
- **Minimum change:** reuse the existing `exec_link_upsert` multi-row shape for the three writers.
  No new chunk constant, no new owner.
- **Contract:** `docs/database-manual-sql.md` requires a recorded cause + measurement + checklist
  per hand-written statement. The cause is pre-recorded; add this run's numbers.
- **Verify:** statement count for one 90k-link import drops from ~410k to ~45k, and the
  spec's recorded **360 ms vs 44.3 ms per 2,000 links** is the cost reference. **`flow_cost_contention`
  CANNOT be T6's instrument** — the import arms are unbuildable from DB rows (deferred `Protocol`
  carriers are unloaded on the page path), so T6 needs its own statement-count probe or leans on
  that recorded figure. Do not go looking for a harness here.

### WS-2 — Per-address write volume

#### T7 — enrichment writes one transaction per address, and per-host batching will NOT fix it
**F3a. Strict. Premise corrected by the numbers — read this before implementing.**
`enrich.rs:444-452` already loops `waiters`, so "collect the per-host rows and call
`set_endpoint_ip_countries` once" collapses transactions **only when `waiters.len() > 1** — and it
is ~1. The arithmetic:

- `deferred = 1,577` counts **half-dispatches**, not links: it is incremented in BOTH
  `dispatch_fast_link` (`ping.rs:1805`) and `dispatch_real_probe` (`ping.rs:1922`) → **~788 distinct
  links**.
- `set_endpoint_ip_country` ran **790** times (db-monitor window 1).

**790 transactions for ~788 links ⇒ `waiters.len() ≈ 1`.** Per-host batching is very close to a
no-op, and it cannot be the fix. (It is also a wasted-write bug when waiters *do* exceed 1: the loop
passes the SAME `(ip, iso)` to every waiter, so N waiters write one identical row N times.)
- **The real cost is contention, not write count.** Isolated cost is **108 µs/address**; production
  p99 was **1,051 ms** — a ~9,700× multiplier produced by *concurrent* single-row write
  transactions, not by 790 cheap writes.
- **Minimum change:** **cross-host accumulation** — a bounded buffer that a dedicated task drains
  into one `set_endpoint_ip_countries` per window, the shape the page seed already uses
  (`enrich.rs:592-596`). The resolution task pushes `(endpoint, ip, iso)` and returns; it must not
  await. A failed drain must re-queue the rows, **not** `break` — the current `break` abandons the
  remaining waiters, which is the 88 aborts.
- **Sizing:** the buffer bounds how long a country can sit unwritten; the drain interval is the
  durability window this introduces, and must be recorded the way T3's was.
- **Retirement track:** the singular call at the enrichment site is retired. `set_endpoint_ip_country`
  is retained **only** if a second caller survives; if not, delete it, and record which.
- **Verify:** `flow_cost_contention` before/after — the `fan-in geo PER-ADDRESS / 32 writers` row is
  the one that must change, since it is the concurrency measurement; `country persist failed` ≤ 1.

#### T8 — `purge_expired` holds a write transaction across a 3.1 s scan
**F5. Strict.** `database.rs:1440-1499` opens the transaction **before** the `NOT EXISTS`
full-table scan, so a 3.1 s read-only scan is a 3.1 s write-lock stall every 10 minutes on the
retention task.
- **Minimum change:** run the candidate-id scan outside the transaction; open the write
  transaction only for the cascade delete + commit, re-checking `last_seen_at` inside it so the
  race stays correct.
- **Record:** the statement in `docs/database-manual-sql.md` with this run's measurement — the
  band spec owns the rewrite, this plan records the number.
- **Verify:** a test asserting the write transaction's duration is bounded by the delete count, not
  the scan.

### WS-3 — DNS deferral timing

#### T9 — the first retry sleep is the whole window, and the budget is shorter than the queue
**F7 + F8. Strict. One task, because it is one product.**
- **Current shape:** `ping.rs:2567` sets `defer_delay = dns_defer_secs` = 15 s, so
  `defer_retry` (`:1968`) sleeps the full window before its **first** re-check; only the second
  poll drops to 250 ms (`:2033`). heed: 900 of 933 lookups drained by t+30.6 s (~40/s), so 15 s is
  ~2× the real wait. 1,577 of 4,028 links deferred → a 15 s floor on 39% of the fast level.
- **Why the two halves cannot be split:** the poll interval and the budget are one product. A flat
  250 ms poll against a 30 s budget is **60 `sched.schedule()` gate acquisitions per deferred
  half** — 1,577 halves ≈ 95k spurious dispatches per batch, which is the very contention the
  full-window sleep was avoiding.
- **Why the 12 were booked unprobed:** the budget is `2 × dns_defer_secs` = 30 s from the **first
  deferral** (`:1984`), but `mark_dns_failure` arms a fresh 15 s window at **report** time, and
  report time is unbounded because the `RESOLVE_SEM` permit is acquired at `enrich.rs:310` —
  **outside** the `DNS_LOOKUP_TIMEOUT` wrap at `:342`. The 12 warns fired at t+38.4–40.2 s = exactly
  30 s after a deferral starting t≈8–10 s; the six hosts whose failures reported after t≈23–25 s
  open their window after the budget expires. Nothing is stuck — the warn's "resolution state is
  stuck" is a misdiagnosis.
- **Minimum change, three parts:**
  1. **Bound the queue, keep report-time arming** (`AGENTS.md`'s rule stands — correction 4):
     acquire the permit **inside** the `timeout` in `enrich.rs`, so request→report ≤
     `DNS_LOOKUP_TIMEOUT`.
  2. **Pinned backoff**, replacing the flat full-window first sleep: `250 ms → 1 s → 2 s → 4 s → …`
     capped at the window. `defer_delay` **may no longer equal** `dns_defer_secs`; the window is
     the maximum, never the poll interval, and the cap is what bounds gate acquisitions to ~6 per
     half rather than 60.
  3. **Reset `waited` when the deferral state's own timestamp advances**, so a re-armed failure
     window extends the budget instead of being cut off by a clock that started at the first
     deferral. This handles the general case, which a fixed budget formula would not.
  Plus: reword the warn to name a slow-or-failed lookup, not a stuck marker.
- **Repair track:** a test with a deliberately slow permit queue asserts no half is booked
  unprobed; a test asserting gate acquisitions per half are bounded.
- **Verify:** `deferred` still ~1,577 but booked-unprobed = 0.

**T9 acceptance is STRUCTURAL — the plan's original repair track was not achievable.**
The plan asked for "a test with a deliberately slow permit queue asserting no half is booked
unprobed" and for bounded gate acquisitions. Only the second is testable, and
`the_deferral_poll_schedule_is_capped_at_the_window` covers it (≤10 polls per budget; a flat poll
would be ~60). The first is **not discriminating by construction**: the budget is
`window + DNS_LOOKUP_TIMEOUT + 1 s`, so it always exceeds the window by ~9 s and the link is
released at window expiry long before the budget could expire. A batch-level test written against
it **passes with the reset disabled** — confirmed by falsification, not assumed.

What is tested instead is the signal the reset depends on:
`the_dns_state_stamp_moves_exactly_when_the_state_is_re_armed` — the stamp appears on a failure,
advances on a lookup, and **holds still** when nothing re-arms (a stamp that moved every poll
would reset the budget forever). It falsifies: deleting the `dns_pending` branch from
`dns_state_stamp` makes it fail.

**Known limitation, accepted:** the stamp is **second-granular**, matching `is_dns_unresolved`'s
own arithmetic, so a re-arm inside the same second as the previous state is invisible and the
budget does not restart for it. Harmless while the budget outlasts the window by ~9 s; it would
matter if those two ever came within a second of each other.

### WS-4 — Page query at scale

#### T10 — the band index covers Address, not Test
**F10. Light. The Test sort has never been measured at *this* scale *at a known offset*, and the
one in-repo datapoint differs on both axes. Measure first; the DDL is conditional on the lab.**
`adr/0010`'s acceptance measured and validated the **Address** sort
(`endpoint_rank_band_host(band, rank_host, endpoint_id)`). The 1,004 ms statements are the **Test**
sort — `ORDER BY k.rank_dns, k.rank_tier, k.rank_weight DESC, k.rank_latency, k.rank_seen DESC,
k.rank_protocol, k.endpoint_id`, exactly the seven terms `profiles_query.rs:239-247` emits and
`specs/2026-10-01-static-config-weight-design.md` §Index mandates.
- **What the weight spec settles, and what it does not.** It measured `PageSort::Test` at
  **4.43 ms** with plan `SCAN endpoint_rank AS k USING COVERING INDEX endpoint_rank_test_v2`, so
  the "~240 ms filesort from a missing index" case **did not appear at 4,000 endpoints**. That
  closes the *filesort* reading — not the *cost* reading — and it says nothing about 74k: whether
  the planner still serves the covering index at this size is itself part of what the lab
  re-establishes. A T10 proposal that adds DDL to fix a *missing index* would be answering a closed
  question; one that adds DDL to fix an **unexplained cost curve** would not.
- **Why 4.43 ms does not answer T10, and why T10 therefore stays.** That figure is a **synthetic
  4,000-endpoint / 12,000-link feed** (`weight-design.md:165`) **with its offset unrecorded**, and
  the spec's own next line refuses to extrapolate: *"Both scale linearly; re-measure on the 7.7k
  reference feed before trusting either number at scale."* The live database is **74,014 endpoints /
  145,268 links — 18.5× the endpoints, 12.1× the links.**
- **The two datapoints differ in scale *and* in offset, so no ratio is claimed.** The production
  1,004 ms came from four *consecutive* slow statements at 12:24:45–51 — the deep-offset page-walk
  shape — while the 4.43 ms records no offset at all. Dividing one by a linear extrapolation of the
  other is **not** evidence of a super-linear cost curve. Normalize on **endpoints** (18.5×) for
  any scale statement; the link ratio (12.1×) is a different quantity and must not be substituted
  for it. **Neither offset-depth nor index degradation is established** — the lab's offset sweep
  (`{0, total/2, N−200}` × 5 reps, against the `Address` sort at the same offsets) is exactly what
  separates them. T10 is a measurement task that must run, not one that may be deferred.
- **Second owner, reconciled here.** The index shape is owned by
  `specs/2026-10-01-static-config-weight-design.md` §Index (*"New name — never edit
  `COVERING_INDEX` in place"*), not only by `adr/0010`. A new partial index is a **sibling** of a
  spec-owned index and must be proposed in that spec as well. **Note that spec's own line anchors
  are already stale** — it cites `endpoint_rank.rs:266` and `:326-333`, while the constants now sit
  at `:415` (`WEIGHT_COVERING_INDEX`) and the DDL loop at `:478-486`. Reconcile the anchors rather
  than trusting either set.
- **Step 1 — measure, do not build.** `flow_cost.rs:1128-1160` already sweeps
  `[(Address, "Address(filesort)"), (Port, "Port(filesort)"), (Test, "Test(index)")]` ×
  `offsets {0, active_total/2, active_total − 200}` under `PurgatoryView::Active`, 5 reps each,
  with the `band = 0` seek A/B immediately after (`:1162`). Run it against the fresh measure DB.
  The `Active Test(index) offset=N-200` row **is** the observation, and the `Test` vs
  `Address(filesort)` comparison at the same offsets says whether the cost is the index or the
  offset. This simultaneously **closes `adr/0010:79`'s pending N∈{50k, 200k} gate**, because the
  DB is 74k — squarely in the unvalidated range. No new harness.
- **Step 2 — only if the lab falsifies the index.** Add a **band-partial** index over the Test
  order terms (`… WHERE band = 0`), as a **new name**, never an in-place edit to
  `endpoint_rank_test_v2` (the trap at `endpoint_rank.rs:409-414`).
- **Sizing rationale, stated correctly:** `band = 0` is **73,666 of 74,014 rows — 99.5%**. A
  partial index here is ~the whole table and saves ~0.5% of it. The win, if there is one, is that
  the predicate leaves the row-lookup path, **not** that the index is small. Therefore the gate is
  `EXPLAIN QUERY PLAN` showing a `band = 0` seek with no `USE TEMP B-TREE` and no table lookup —
  not an implied size win.
- **Spec touch — two specs, not one.** Extend
  `specs/2026-09-24-profiles-view-band-design.md` §6 acceptance from Address to the Test sort at
  N∈{50k, 200k}, **and** reconcile the new index with
  `specs/2026-10-01-static-config-weight-design.md` §Index, which owns `endpoint_rank_test_v2` and
  mandates its exact column list. **The band spec is `draft — awaiting user review`; if the user has
  pending edits there, this task waits.**
- **Document sequencing (commit `850abb7` conflict):** that commit just edited all three docs T6
  and T10 must write to — `docs/database-manual-sql.md` (+65), `docs/database.md` (+17),
  `adr/0008-batch-feed-scaling.md` (+12). **T6 lands before T10**, and both re-read their target
  doc at task start; the new static-weight DDL entries are the context, not something to overwrite.
- **Retirement:** if the partial index fully serves the sort, `endpoint_rank_test_v2` becomes
  write-only. Record as a **candidate** with a trigger (measure write cost first); do not drop it
  here.
- **Side effect of T1:** with `.log_statement_params(true)` wired, the next slow-query line carries
  the `OFFSET` bind, so the mechanism is visible in production logs too.

### WS-5 — New config surface

#### T11 — the 64 MiB import budget is a hard-coded const
**F4. Light, but new surface — needs a short spec first.**
`stream_import.rs:174-175` `MAX_FEED_BYTES = 64 MiB`, `MAX_FEED_LINKS = 200_000`; no config knob.
It truncated a 90,688-profile feed, set the group to `GroupStatus::Error`, and
`ui/settings.rs` renders only the literal word `error` — the detailed message exists solely in the
activity log. **No document in `docs/aegis/` owns import batch limits.**
- **Gate:** write a short Spec Brief for the import budget (owner, default, and what the group
  status means on truncation) before implementing. The one task that escapes "bounded".
- **Minimum change:** an `import` section in `AppConfig` with `max_feed_bytes` and
  `max_feed_links`, both serde-defaulted (an existing `config.json` loads unchanged; nothing is
  renamed); surface `error_message` in the group list.
- **Verify:** a config round-trip test; a run at a deliberately low budget yields the documented
  status and a visible message.

### WS-6 — Decide the journal-mode gate

#### T12 — the A/B is a measured trade with a one-way cost, and it is last
**Light.** This is the decision `specs/2026-09-24-turso-mvcc-rollout-design.md:88` is waiting for:
*"Keep WAL default unless a contention-heavy real-feed benchmark shows that avoided waits outweigh
the measured import/geo tax."* `plans/2026-09-24-turso-mvcc-production-rollout.md` leaves it open.
- **The gate has two sides and the 2026-10-01 run supplies one.** *Avoided waits* — supplied:
  88 `snapshot is stale` aborts, 10 `busy_timeout` drops, a 327 s batch. *Measured tax* — measured
  at `spec:61` against a workload that "did not reproduce the reported long wait", so it must be
  **re-measured under the real feed**, and T3/T4/T6/T7 change it.
- **It is not a config flip.** §Rollout: *"Do not convert an existing WAL database in place. To move
  an existing feed to MVCC, perform the project's existing explicit destructive schema
  reset/reimport path."* On a 74,014-endpoint / 145,268-link database that is a **full wipe and
  reimport of every subscription** — a named one-way cost of the experiment, paid before any
  measurement exists.
- **The tax side is already quantified and is not small:** **total wall +32%**, **import p50 +47%**
  (§DB-API A/B), plus `finish_batch` skipping `PRAGMA wal_checkpoint(PASSIVE)` for MVCC handles
  with the probe showing that call failing (§Retry/checkpoint). And
  `endpoint-ip-storage-design.md:132` measured MVCC's reader latency during a flush as **"no
  benefit; not shipped"** (p50 27.0 → 27.6 ms).
- **Sequencing (correction 6):** run only after T3/T4/T6/T7, or the 32-row microbenchmark is
  re-measured against this run's ~1,000-writer fan-out and a pathological flush cadence — it would
  measure the pathology, not the modes.
- **Harness:** T0's `flow_cost_contention`, mode-switchable, so the A/B is one command. Re-copy the
  DB (MVCC requires 255 **and** a fresh file).
- **Output, exactly two:** promote MVCC to default with the measurement attached, or record WAL as
  deliberate with the T3/T4/T6/T7 numbers as the mitigation. Either closes
  `specs/2026-09-24-turso-mvcc-rollout-design.md` §Unverified production claim and updates the
  rollout plan's status. **"Needs more data" is not an allowed output** — the gate was written to
  be decidable by one real-feed run.


### WS-7 — Minor

#### T13 — a 1-link subscription fails 4/4 with no diagnostic
**F11. Light.** `1 links, 1 errors` at 11:49, 11:55, 12:20 ×2 — deterministic, reproducible, never
investigated. Add the failing entry's reason to the validation log line so the cause is stated
once. No behaviour change.

#### T14 — `AGENTS.md` is stale about the log channel
**Docs, folded here because T2 touches the same code.** `main.rs:188` is a
`std::sync::mpsc::sync_channel(4096)` — **bounded**. `AGENTS.md`'s log-subsystem section says
"UNBOUNDED channel, never blocks under tracing lock". Both are true (it does not block the
producer; it drops), but the drop behaviour is undocumented and `dropped_logs` has no reporting
path. Correct the text and surface the counter.

---

## Risks

| Risk | Mitigation |
| --- | --- |
| **T3 changes how long a result sits unpersisted — 200 ms → ~10.7 s at the observed trickle** | **Measured and accepted, not incidental.** `TIMER_FLOOR_DIVISOR = 4` puts the floor at 128 of 512, and `MAX_STAGED_AGE_TICKS = 75` puts a hard **15 s ceiling** on it. Nothing is *lost* — batch end, quit and reload flush explicitly — but results are *delayed*, and that delay is what buys **1,669 → 32 commits** (52×). Reaching the original `≤ 20` would need a 256-row floor and ~21 s of staging; that is a durability call for the user, not a knob to turn silently. Both constants carry the trade table in their doc comments. |
| T9's budget change lengthens the batch's worst case | Only for halves whose lookup is genuinely failing; the 12 booked-unprobed become 12 probed halves. Bounded by `DNS_LOOKUP_TIMEOUT` plus the backoff cap. |
| T9's backoff must not become the contention it replaces | The pinned `250 ms → 1 s → 2 s → …` cap bounds gate acquisitions to ~6 per half (~9.5k per batch) against 1,577 today. |
| T10 depends on a **draft** spec | Task explicitly waits if the user has pending edits. |
| T6 is the largest diff and touches `docs/database-manual-sql.md` | Reuses the existing `exec_link_upsert` shape; no new chunk constant. **Not** measured by T0 — the import arms are unbuildable from DB rows; verified by statement count plus the spec's 360 ms vs 44.3 ms per 2,000 links. |
| T12's outcome may be "WAL stays" | A valid, recorded outcome, not a failure. The write-path fixes stand on their own evidence. |
| T12's A/B costs a full wipe + reimport | Named as a one-way cost up front, and gated behind T0's harness so the experiment is at least reproducible. |
| T10 might attribute the cost wrongly — the two datapoints differ in scale *and* offset, so neither super-linearity nor offset-depth is established | T10 claims no ratio and normalizes on **endpoints** (18.5×), never links (12.1×). The lab's `{0, total/2, N−200}` sweep against the `Address` sort at the same offsets is the gate, and the sizing note forbids claiming a size win. |
| T6 and T10 both write docs `850abb7` just edited | T6 before T10; both re-read the target doc at task start (T10's sequencing note). |

## Retirement summary

| Retired | Replacement | Trigger/notes |
| --- | --- | --- |
| `persist_batch` silent swallow | `dropped` folded into the existing `ended_early` | T5, immediate |
| bare-timer flush | row floor + idle backoff | T3, immediate |
| per-address country write at the enrich site | `set_endpoint_ip_countries` | T7; singular fn kept only if a second caller survives |
| un-jittered backoff | full jitter, same 5 attempts | T4, immediate |
| full-window first sleep | pinned `250 ms → … → window` backoff + `waited` reset on state advance | T9, immediate |
| `_ => {}` field drop; unwired `log_statement_params` | sanctioned driver knob + session-only `detail` | T1, immediate |
| `log_cache`-based export; shutdown no-flush; `enrich.rs:349` at `info` | heed-paged export; flushed shutdown; session-only resolution log | T2, immediate |
| disposable import+geo probe | durable `flow_cost_contention` | T0, immediate |
| `endpoint_rank_test_v2` | *candidate only* | T10 — measure write cost first; **not** dropped here |

## Execution route

**inline**, in the WS order above. T3/T4/T6/T7 share the contention cluster and T12 depends on all
of them landing, so splitting across agents would create edit conflicts for no parallelism gain.
`User confirmation required: yes` on one boundary — **T11's new config field** (new surface, needs
a Spec Brief and your sign-off on the default). T10 additionally waits on the band spec's pending
review. Everything else proceeds.

**Finding coverage** (all 13 from the 2026-10-01 report, no orphans): F1 → T5 · F2 → T3 ·
F3a → T7 · F3b → T4 · F4 → T11 · F5 → T8 · F6 → T1 · F7 → T9 (merged) · F8 → T9 (merged) ·
F9 → T6 · F10 → T10 · F11 → T13 · F12 → T2 · **F13 → T2 (folded)**. T0, T12 and T14 are
supporting tasks with no finding of their own.

### Sequencing rule — T0's "before" is captured once, before T3

`flow_cost_contention` measures the **unfixed** code, and the "before" for a given arm is gone
the moment that arm's writer changes. The rule is therefore **per-arm, not per-workstream**:

- **T3 is free to start** — the trickle baseline (**1,668 / 1,669 flushes, 2.41 rows/flush**)
  is recorded and stable across three runs, and T3 does not touch the import or geo writers.
- **T7 is free to start** — the geo arms (31–36 µs/row batched vs 108–121 µs/address
  per-address, and the 32-writer fan-in) are recorded, and it is the only valid reader of them.
- **T6 has no before at all** and is blocked on a purpose-built statement-count probe.
- **T12** is blocked on a purpose-built concurrent probe (see the T12 premise note above).

The earlier over-broad form of this rule — "no task in WS-1 or WS-2 may start" — was wrong and
would have stalled T3 behind work it does not depend on.

### Measurement boundary introduced by T1 — read before comparing T0 numbers

T1 wires toasty's `.log_statement_params(true)` on the ONE production `Db::builder()` site
(`try_open_db`). toasty renders params at statement-construction time for **every** statement when
the flag is on (it early-returns only when the event is disabled, and `toasty::query` DEBUG is
enabled by our filter) — so it now formats params for the 400-row link-writer windows and the
import bulk upserts too, not just for the page query that motivated it.

**The knob is per-`Db`, not per-statement, so it cannot be scoped.** The choice was: keep it on and
know the cost, or keep it off and lose the `OFFSET` bind that T10 needs to attribute the 1,004 ms
page query. Kept on.

**Consequence, and it matters:** the **T0 baseline in this document was captured with the flag
OFF**. Any T6/T7 re-measurement happens with it ON. Write-path wall-clock numbers are therefore
**not comparable across that boundary** — re-baseline T0 before judging T6 or T7 on time, or judge
them on statement count and `retries` (which is what this plan already says for T6 and T7).

---

## T10 Step 1 — MEASURED 2026-10-02 (real feed, 74,014 endpoints, WAL, idle copy, median of 5)

`flow_cost_report` gained a real-feed sweep of `Address` and `Test` across offsets
`{0, total/2, N−200}` — the synthetic `XRAY_TUI_SCALE` lab already did this, but only on seeded
feeds, so the 74k question was unmeasured. Run against a `sqlite3 .backup` copy; the live file was
not opened.

| query | offset 0 | offset 37,007 | offset 73,814 |
| --- | --- | --- | --- |
| All view, `Address` | 20.7 ms | 20.3 ms | 21.0 ms |
| All view, `Test` | 4.3 ms | 5.9 ms | 7.8 ms |
| **Active (`band = 0`), `Test`** | **102.6 ms** | — | — |
| Active, `Address` | 4.8 ms | — | — |
| Active, `LastSeen` | 66.8 ms | — | — |
| `load_page_projection` (hydration) | 3.3 ms | — | — |

**1. `band = 0` is the driver.** Adding it to the `Test` order costs **~20×**
(4.3 ms → 102.6 ms). `Address` barely moves, because `endpoint_rank_band_host` *leads* with `band`.

**2. This is UI-visible.** `load_page_projection` is 3.3 ms, so a page refetch is dominated by
`profiles_page`: **~100 ms on the render task** at 74k, against a 16 ms tick.

---

### CORRECTION (same day, later measurement) — the dominant cause is **missing planner statistics**

The measurement above ran against a copy that had been `ANALYZE`d at some point in the session. On a
**fresh** copy of the live database — and the live database has **no `sqlite_stat1` table at all** —
the plan for `Active + Test` is:

```
SEARCH k USING INDEX endpoint_rank_band_window (band=?)
USE TEMP B-TREE FOR ORDER BY
```

**Every Active-view Test-sorted page sorts all ~73,666 matching rows to satisfy the ORDER BY, then
keeps 200** — and with `OFFSET` it sorts them all and *then* discards the skipped ones. Raw SQL,
median of 5, on a copy of the live 74,014-endpoint database:

| variant | offset 0 | offset 37,007 | offset 73,814 |
| --- | --- | --- | --- |
| **A. as production is** (no statistics) | 20.4 ms | 58.2 ms | 64.9 ms |
| **B. A + band-partial index** (still no statistics) | 20.0 ms | 61.4 ms | 65.0 ms |
| **C. A + `ANALYZE`** | 0.1 ms | 9.8 ms | 19.4 ms |
| **D. C + band-partial index** | 0.0 ms | **0.6 ms** | **1.2 ms** |

**The findings that matter, in order of cost:**

1. **B is the headline: adding the partial index alone does NOTHING.** The planner never chooses it,
   because with no statistics it does not know the index exists is selective. A DDL-only fix for T10
   would have shipped a no-op.
2. **`ANALYZE` (C) is a 4–16× win for the price of one statement.** It moves the plan from a
   filesort to `endpoint_rank_test_v2`.
3. **The partial index only pays ON TOP of statistics (D)**, a further ~16× — because
   `endpoint_rank_test_v2` lacks `band`, so every skipped index entry costs a **table lookup** to test
   the predicate, while the partial index is implicitly `band = 0` and covers the query. Total A→D is
   **~54× at the deep offset**.
4. So the size argument is irrelevant and the covering argument is the real one — and it is
   *conditional on statistics existing*.

> **⚠ SUPERSEDED by the "T10 FINAL" section below.** This paragraph concluded that
> `ANALYZE`-at-open was "the first move". That conclusion was **wrong and the change was
> reverted** — see T10 FINAL. Do NOT re-add `ANALYZE`-at-open and do NOT write it into the band
> spec's acceptance as a first move. Retained only as the record of what was measured here.

This measurement closed `adr/0010:79`'s pending N∈{50k, 200k} gap.

### T10 FINAL — both candidate fixes measured, **neither works**; the cost is toasty's execution layer

Raw SQL on a copy of the live 74,014-endpoint feed (median of 5), with and without planner
statistics, and with/without the band-partial index:

| variant | offset 0 | offset 37,007 | offset 73,814 |
| --- | --- | --- | --- |
| no statistics (as production ships) | 20.4 ms | 58.2 ms | 64.9 ms |
| + band-partial index, still no statistics | 20.0 ms | 61.4 ms | 65.0 ms |
| + `ANALYZE` | 0.1 ms | 9.8 ms | 19.4 ms |
| + `ANALYZE` + band-partial index | 0.0 ms | 0.6 ms | 1.2 ms |

But **through the app's own `profiles_page`, with statistics present and the partial index
installed, the Active+Test page is still ~98 ms — flat in offset**:

| `profiles_page` Active/Test, statistics + partial index | offset 0 | offset 36,833 | offset 73,466 |
| --- | --- | --- | --- |
| through toasty | 98.0 ms | 103.8 ms | 106.7 ms |

And the *same work* in raw SQL on the same file, statistics and index present:

| | ms |
| --- | --- |
| `COUNT(*)` over `band = 0` (what every page pays first) | 1.11 ms |
| the page itself, offset 0 | 0.04 ms |
| **count + page** | **~1.2 ms** |

**So the page query's ~100 ms is toasty's execution layer, not SQLite.** The plan is clean
(`SCAN k USING INDEX endpoint_rank_test_v2`, no temp b-tree), the index is covering, and raw
execution is ~1.2 ms — roughly **80× less** than the same statement through toasty. The cost is
flat in offset, which rules out deep-offset walking; it is flat in plan, which rules out index
selection.

**Both fixes were therefore reverted.** `ANALYZE`-at-open (77 ms once, plus per import) removes a
real filesort, but it was introduced to fix *this* query and demonstrably does not: shipping it
would be a speculative cost against a disproven rationale.

**What T10 actually is:** a toasty row-materialisation cost, not a page-query design problem.
The escalation is the one ADR 0001 and the band spec already name — **keyset/cursor pagination**,
which stops asking for a deep `OFFSET` at all — or an investigation of toasty's
materialisation path. It is **not** an index, and **not** a statistics problem.

This also settles the earlier `profiles_page` p50-vs-p99 split from the 2026-10-01 run: the plan
was never the cost, so the p50 (1.2 ms, a `PageMeta.total` that returned early) versus the p99
(1,004 ms, a full materialisation) difference is the toasty path, not contention.
