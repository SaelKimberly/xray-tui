# Test-cell truth, resolution ownership, and batch plan scopes

Date: 2026-09-18
Status: approved (implementation in progress)
Related: ADR 0003 (stored ordering keys), ADR 0005 (endpoint addresses), decision 16 (test-priority law), decision 20 (native connect-time core), decision 4 (schema tag = wipe), decision 22 (write-behind link writer), `2026-09-17-batch-ping-pipeline-design.md`, `2026-09-15-endpoint-ip-storage-design.md`

## Problem

Four reports from real use. Root causes below are code-verified (`file:line`); each was traced to its canonical owner.

### R1 — a real success shows no outbound IP, and a `[name]` row shows one

Three independent owners, never reconciled:

1. **Exit IP is best-effort.** `ping_native::real_ping` runs the latency probe, then `fetch_ip_info` (`ops/ping_native.rs:234`, `:305-322`). Any failure returns `None` without failing the probe, so `apply_test_result` writes `Latency::Real { delay, ip: None }` (`ops/events.rs:122-126`) **and** the handler skips enrichment entirely (`ops/events.rs:680`). The row renders `[ 12 ] … —`.
2. **The exit-IP fetch is not retried, and a later fast/udp result erases it.** `TestType::TcpPing | UdpTest => row.latency = Some(Latency::Fast { delay })` (`ops/events.rs:128-129`) replaces the whole `latency`, dropping the stored `ip`.
3. **`[name]` is terminal for a DNS host that a feed-wide batch never resolved.** The only non-forced resolution trigger is the result handler, and it resolves the endpoint **by page membership**:

   ```rust
   // ops/events.rs:551-556
   let (ep_id, ep_host_is_dns) = state.endpoints.iter()
       .find(|r| r.endpoint.id.get() == endpoint_id)
       .map_or((0, false), |r| (r.endpoint.id.get(), r.endpoint.host_type == HostType::Dns));
   ```

   `state.endpoints` is the loaded 200-row page (`ops/profiles.rs:190`). Every endpoint outside the page yields `(0, false)` → **no resolution trigger and no outbound enrich**. The batch still persists `latency_ip` for those links (`ops/ping.rs:1791-1818`); the next page load then seeds the mmdb country from those persisted IPs (`ops/enrich.rs:522-553`). Net: outbound IP + country + `[name]`.

   Nothing else resolves them: `spawn_enrich_ip_hosts` filters `matches!(host_type, Ipv4|Ipv6) || !r.resolved_ips.is_empty()` (`ops/enrich.rs:320-326`). A fresh DNS host has neither, so it gets no `endpoint_info` entry, and no startup or import pass resolves it. `spawn_dns_resolve`'s other triggers are both `force = true` (`ops/connect.rs:183`, `ui/mod.rs:810`).

   This is why a fresh database with a single new subscription shows it: nothing has ever resolved those hosts, and a feed-wide batch structurally cannot.

4. **Latent trap in the merge guard.** The entry is created when `info.outbound_ip.is_some()` (`ops/events.rs:840-845`), producing an entry with empty `resolved_ips` and `resolved_at_secs: None`. The comment two lines above states that must never happen — such an entry "would block the startup seeding pass and make `should_resolve` treat the endpoint as a never-retried IP host" — and the `outbound_ip` arm re-introduces exactly that. Once created, `spawn_enrich_ip_hosts` skips it (`!contains_key`) and `should_resolve` returns `false` (`ops/enrich.rs:35-50`).

5. **`[name]` and the tier-5 band read different sources.** The cell reads in-memory `endpoint_info.resolved_ips` (`ui/profiles.rs:687`); the ordering law reads the page row's persisted `resolved_ips` (`endpoint_rank.rs:145`).

### R2/R3 — one root cause: the sort key and the Test cell are different laws

- **Sort**: `RankLink::key` (`endpoint_rank.rs:66-87`) lets `error_kind` force tier 3/4 and `latency = i32::MAX`; the endpoint key is the min over its links (`compute_rank:195-238`); SQL orders by `rank_dns, rank_tier, rank_latency, rank_seen DESC, rank_protocol` (`profiles_query.rs:175-182`).
- **Cell**: `compute_test_cell` (`ui/profiles.rs:400-441`) reads `active_link()` → `select_best_measured_link` (`models_toasty.rs:592-620`), which is **error-blind**, and `test_cell_content:500-530` lets the delay win over the marker.
- `apply_test_result` keeps `row.latency` when it writes an error (`ops/events.rs:100-119`) and `stage_result` re-injects the batch's fast latency into the snapshot (`ops/ping.rs:1805-1811`), so a failed real probe persists `latency = Fast` **plus** `error = Real`.

Consequences, both reported:

- **R2**: such a link **shows `[ 12 ]` but sorts at tier 3** — below blank untested rows. Numeric cells interleaved with a band the user cannot see. Secondary: all tiers ≥ 2 store `rank_latency = i32::MAX`, so that whole region orders by `last_seen_at DESC` then `protocol_id`, invisible under a latency column.
- **R3**: after a full fast+real batch every real failure keeps its fast latency and hides its `[real]` marker. The marker is persisted; the UI does not show it. Amplifiers: `dedup_endpoints` defaults **on** (`app_config.rs:565`) so `dispatch_real_probe:1584-1593` retires sibling links with no result, no marker and no counter; stop retires silently (`:1577-1582`); a fast success clears a prior marker (`ops/events.rs:133`).

Note the batch's own semantics already match the intended model — "probe every protocol of an endpoint until 1+ success or all fail": `completed_endpoints` is inserted only on a real `Ok` (`ops/ping.rs:1627-1632`), so a failure never retires siblings. Only the visibility was wrong.

### R4 — no plan scope exists

`PageRequest` carries view / active-threshold / search / group / sort only. Nothing expresses "has a real success", "untested", or "has an error", so a scoped batch cannot be planned.

## Decision

### D1 — the Test cell renders the endpoint's REPRESENTATIVE link

The cell describes the endpoint's representative link: the minimum of the
decision-16 key (`EndpointRow::representative_link_index`), which is the same
link `compute_rank` derives the endpoint's stored `rank_tier` from. That link's
own state is what the cell shows, with a marker on it outranking its own stored
delay:

```
[name] > untestable > purge > failure marker > delay > blank
```

The link **selection** was the real defect, not only the precedence. Selecting by
`active_link()` → `select_best_measured_link` (lowest-delay MEASURED link,
error-blind) picks a link the ordering law does not: for `A = fast-ok 12 + real-err`
(tier 3) beside `B = fast-ok 20 + real pending` (tier 1), the error-blind rule
picks A, so a precedence-only fix rendered `[real]` on a row whose `rank_tier` is
1 — a marker cell sorting inside the measurement band. With the representative
link the cell's content IS the row's tier band, so acceptance 4 holds by
construction:

| tier | cell |
| --- | --- |
| 0 real-ok | the real delay |
| 1 fast-ok | the fast delay (`rank_latency` is that same number) |
| 2 untested | blank |
| 3 / 4 real-err / fast-err | `[real]` / `[fast]` — no link has a clean measurement left |
| 5 dns-unresolved | `[name]` |
| 6 purged | the purge label, which is now reachable only when EVERY link is purged (the representative of an endpoint with a live link is that live link) |

Behavioural consequences (all intended):

| moment | cell |
| --- | --- |
| fast half settled, real pending | fast delay |
| real half succeeded | real delay |
| real half failed, no clean sibling | `[real]` |
| every link failed | `[real]` / `[fast]` |
| retired sibling after a sibling's success | unchanged fast latency; the endpoint's representative is the real-ok link, so the row shows the real success |

**Preserved**: the panel keeps per-link state; `select_best_measured_link` and
`active_link()` keep driving the row's exit IP, country, speed and traffic; a
link with both a marker and a measurement still contributes its measurement to
the panel. **Changed deliberately**: the cell no longer follows
`selected_protocol`/`manual_protocol_override` — the ordering law's tier ignores
the override, and a cell that honoured it could not report the endpoint's band.

### D2 — one resolution source per fact, and resolution driven by database state

1. **Outbound/Country render only when the endpoint is resolved** (IP hosts count as resolved). The outbound columns are a fact about a *working tunnel*, so a row that is not resolved must not claim one.
2. **One resolution accessor.** The cell's `resolved` is `!endpoint_dns_unresolved(row)` — the ROW, i.e. the very predicate `rank_dns` is computed from. The cell previously read the session's `endpoint_info` cache while the ordering law read the row, so `[name]` and the tier-5 band could answer differently for the same endpoint; the cache stays a cache and is never a resolution source. (`endpoint_dns_unresolved` itself is unchanged — its doc states that its callers must not consult the cache, and the fix belongs at the caller.)
3. **Resolution is driven by database state, not page membership.** The batch carries the endpoint facts with its request — `CoreEvent::DnsResolveRequest { endpoint_id, host, host_type, sni }`, emitted once per DNS endpoint per planned page and deduped within the page (a fresh entry is invisible to the handler's TTL gate until its resolution completes, so per-link requests would all spawn their own lookup). Two bounds make the fan-out safe: the batch gates the request on the endpoint's PERSISTED `resolved_at` against `dns_cache_ttl_secs` (the handler's gate only sees the session's entries, which are seeded for the loaded page, so without this every off-page DNS endpoint is re-resolved every run — and a transient failure anywhere in that fan-out calls `mark_dns_failure`, DNS-deferring a healthy endpoint inside the batch that asked for it), and every spawned lookup holds one permit of a process-wide `Semaphore` (64) acquired inside the task, never in the drain — except a FORCED lookup (the `x` key, the connect path), which bypasses the bound: the semaphore is fair, so a user-triggered resolve would otherwise queue behind an entire feed's fan-out and look dead, and one request cannot flood anything. `enrich::spawn_dns_resolve_host` is the one worker behind the page-scoped trigger, the forced `x`/connect trigger, and this one, so the TTL gate is evaluated in a single place. `[name]` becomes a transient, not a terminal state.
4. **An outbound-only event may no longer poison the resolution gate.** The guard's `outbound_ip` arm stays (the exit-IP fallback is worth keeping), and instead both readers that treated "entry exists" as "resolved" are fixed: `should_resolve` resolves an entry with an empty address set AND no attempt timestamp, and the startup seed no longer skips such an entry (preserving its outbound fields on the write). The invariant the old comment asserted becomes non-load-bearing rather than something three call sites must remember.
5. **The exit-IP fetch retries once, spaced, and only on a transient miss.** `fetch_ip_info_once` returns `Answer | Refused(reason) | Transport(e)`: a non-2xx status or a body that is not the provider's object is a REFUSAL — deterministic, and usually the provider's rate limit, which every concurrent probe shares — so it is logged at `debug` (the Actions panel) and the retry is skipped; only a tunnel/request failure is retried, after 250 ms. Each attempt carries `IP_INFO_TIMEOUT`.
6. **The default exit-IP endpoint is fixed, and its ceiling is documented.** `speed_test.ip_api_url` defaulted to `https://ip-api.com/json/`, which answers `403 {"status":"fail","message":"SSL unavailable for this endpoint,…"}` on the free tier; `parse_ip_info` needs `query`, so a fresh install persisted `Latency::Real { ip: None }` on EVERY real ping. The default is now the provider's working free endpoint (`http://ip-api.com/json/`, fetched through the probe's own tunnel).
   **What this does NOT fix**: ip-api's free tier is published at ~45 requests/minute, and one request per real probe means a feed-wide real level (2.6 results/s ≈ 156/min on the 2026-09-16 measurement) runs several times over that ceiling. A whole-feed fill of exit IPs is therefore not achievable on the free tier at any retry policy — it needs a key/provider that permits the rate, which is exactly what `ip_api_url` is for. The honest expectation for the free tier is "the first probes of a run get IPs, the rest refuse", and each refusal now says so in the Actions Log.

### D3 — four flat menu items, planned by the materialized tier (no schema change)

Menu items, after `Fast + Real Ping (All Profiles)`:

- `Fast + Real Ping (Successful)`
- `Fast + Real Ping (New)`
- `Fast + Real Ping (Successful + New)`
- `Fast + Real Ping (Failed)`

The scope selects **endpoints**; every link of a selected endpoint is planned, so `dispatch_page` is unchanged. A scoped walk does freeze its endpoint set first (see below).

The predicate needs no new storage: the endpoint's min-tier is already materialized and indexed as `endpoint_rank.rank_tier`.

| scope | predicate | derivation |
| --- | --- | --- |
| Successful | `rank_tier = 0` | tier 0 ⇒ a live, resolved link with a real measurement and no marker |
| New | `rank_tier = 2` | tier 2 ⇒ no link has a clean measurement and ≥1 link is untested |
| Successful And New | `rank_tier IN (0, 2)` | exactly `Successful ∪ New` |
| Failed | `rank_tier IN (3, 4, 5)` | tiers 3/4/5 ⇒ no live link has a clean measurement or is untested |

Index-driven by the existing covering index `(rank_dns, rank_tier, rank_latency, rank_seen DESC, rank_protocol, endpoint_id)` (`endpoint_rank.rs:249-250`).

**A scoped walk freezes its endpoint set before the first probe is dispatched.** The predicate reads derived state this very batch mutates, so paging it with `LIMIT/OFFSET` would silently skip rows: a `Failed` run moves its own endpoints to tier 0 as they succeed, the filtered set shrinks, and the next page's OFFSET lands past endpoints it never visited — roughly half the run. The walk therefore reads the matching endpoint ids ONCE (`profiles_page` with the scope, before any write) and serves `load_page_projection` in page-size chunks from that list; the streaming shape and the memory profile (ids only) are unchanged. `PlanScope::All` keeps streaming one page at a time — its membership ("has a link") and its `PageSort::Address` order are both independent of probe results, which is the property that does NOT carry over to a tier predicate. `a_scoped_walk_freezes_its_set_before_probing` pins it by removing endpoints from the scope mid-walk.

**Recorded boundary**: an endpoint that has both an untested link and a failed link reports `rank_tier = 2`, so it is selected by *Successful And New*, not by *Failed*, even though its cell shows a marker under D1. That is deliberate — it still has a link worth probing, and it lands in *Failed* once that link is probed and fails. Closing the gap exactly would need a stored per-endpoint "has a marker" bit; that is not worth a new owned surface for a state a full batch resolves by construction.

## Invariants

- Identity, the `uid = sig ^ cred_hash` wire format, and every stored ordering key's *values* are unchanged by D1/D2/D3. D1 changes the cell only; D3 adds a `WHERE` predicate only.
- `profile_stats` columns and semantics are unchanged. No schema tag bump; no `ALTER TABLE`; no new table.
- `RankLink::key` remains the single implementation of the test-priority law; the parity golden (8 sorts × both directions vs the Rust oracle) stays green.
- The native connect-time core choice (decision 20) is untouched: the kind gate stays at plan time, the config gate inside the real probe.
- A `Cancelled` result still persists nothing.

## Non-goals

- Changing the tier law's ordering *values* or the decision-16 tier bands.
- Reordering the failure band's tiebreak (`last_seen_at DESC`, `protocol_id`). Under D1 the band is label-homogeneous, so it reads as grouped; recorded as an open observation, not a defect.
- Making `display_link_index` and `select_best_measured_link` one function. They remain two rules for two purposes (the display link for the other sorts and for the row's exit IP/speed/traffic vs the representative link the Test cell and the tier read); D1 removes the divergence that mattered.
- Persisting the whitelist features or the outbound country.

## Acceptance

1. A real success whose exit-IP fetch fails once still ends with an IP after the spaced retry; if it fails twice, the row shows `—` and a real delay.
2. A row whose DNS host is unresolved renders `[name]` **and no outbound IP/country**.
3. After a full `Fast + Real Ping (All Profiles)` batch, every Active row shows either a real delay or a marker — no row shows a bare fast delay without a reason, except endpoints whose links were legitimately not probed (stopped batch, purged links, untestable rows).
4. Sorting by Test: a row displaying a delay sorts in a measurement band (tier 0/1); a row displaying a marker sorts in a failure band (tier 3/4/5). This holds by construction — the cell's link IS the rank key's argmin — and is unit-pinned for single-link, mixed-link and override-pinned endpoints, including the mid-batch state `fast-ok + real-err` beside `fast-ok + real pending`. The parity golden (8 sorts × both directions) stays green.
5. The scoped variants visit EVERY endpoint that matched when the plan was built, even though their own results change the predicate mid-run (`a_scoped_walk_freezes_its_set_before_probing`).
6. The three new menu items plan only the selected scope; the scope respects paging (`profiles_count` matches the plan size), and each variant's plan is bounded by the feed walk.
7. A scoped batch asks for a DNS resolution only for endpoints whose persisted attempt is absent or past the TTL, and at most 64 lookups are in flight at once.
8. `cargo fmt`, `cargo clippy`, `cargo nextest` green on the touched crates; the rank parity golden and the batch pipeline tests unchanged.

## Verification record

- `501 passed` (xray-tui-db + xray-tui + xray-tui-config), `cargo fmt --check` clean, `clippy` clean, workspace `cargo check --all-targets` clean.
- Live TUI on an isolated `XDG_CONFIG_HOME` (the user's database untouched): the menu renders every item, navigation lands on each new one, and a scoped batch dispatches, plans and returns cleanly.
- The exit-IP default's defect was measured directly: `https://ip-api.com/json/` → `403 {"status":"fail",…}`, `http://ip-api.com/json/` → `200 {"status":"success",…}`.
- The maintainer's own `~/.config/xray-tui/config.json` was read: its stored `ip_api_url` is ALREADY the plain-HTTP form, so the HTTPS default was a fresh-install defect rather than the cause of the reported rows. Their `—` cells are the free tier's rate ceiling, which D2.6 does not and cannot fix.
- **Not verified live**: a full feed-wide fast+real run (needs the real cores, network and hours). Each seam is covered instead — the SQL predicate, the count and the walk (db tests), the scope→plan path (walk test), the freeze (mid-walk mutation test), and the menu→scope→dispatch path (live TUI).

## Owner-document amendments

| Surface | Owner | Amendment |
| --- | --- | --- |
| Test-cell precedence (decision 16's display half) | `adr/0003-stored-profile-ordering-keys.md` | record that a marker-carrying link contributes no displayable delay, so cell and tier agree |
| Resolution state ownership | `specs/2026-09-15-endpoint-ip-storage-design.md` | one accessor; resolution driven by persisted state, not page membership |
| Plan scopes | `specs/2026-09-17-batch-ping-pipeline-design.md` | the three scope variants and the `rank_tier` predicate |
| Manual-SQL inventory | `docs/database-manual-sql.md` | record the new scope predicate's site (parameterised predicate only, no new DDL) |

## ADR signal

D1 changes a durable contract: **what the Test column asserts about a link**. It is a decision-16 amendment, not a new decision, so it belongs in ADR 0003 rather than a new ADR. D2's accessor is a source-of-truth statement for ADR 0005's surface. D3 adds no durable architecture surface (predicate only), so it needs no ADR.
