# Purge reasons: evidence-based permanent moves to Purgatory

Date: `2026-09-17`
Status: `design — awaiting user review`
Scope: `xray-tui-db`, `xray-tui`, `xray-tui-native`, `xray-tui-tls`
Related: `specs/2026-09-11-profiles-page-query-design.md` (the page query),
`specs/2026-09-14-profiles-stored-sort-key-design.md` + `adr/0003` (the ordering
keys), `specs/2026-09-11-write-behind-link-writer-design.md` + `adr/0002` (the
link writers), `adr/0004` (the native real-ping engine), `docs/database.md`,
`docs/database-manual-sql.md`, `AGENTS.md` decisions 4, 16, 20, 21, 22.

## 1. Problem

Purgatory today holds exactly one thing: rows whose newest link is older than
`purgatory.ttl_days` (7), reclaimed by `purge_expired` after
`retention_days` (30). That is a *staleness* test, and it is the only test.

But a real probe can produce evidence that a config can **never** work again —
a REALITY endpoint answering with a real certificate, a proxy whose own upgrade
handshake answers `404`, a CDN answering `530` because the origin is gone. That
evidence is discarded: the failure lands in `profile_stats.error` (a marker the
`error_ttl_hours` sweep deletes) and the row stays in the Active view forever,
competing for attention with configs that work. On the 2026-09-17 corpus, 1,687
links already carry such evidence, and phase 2 was only 27 % done.

The goal: **a link whose probe returned permanent evidence leaves the Active
view immediately and permanently** (until it is proven alive again, or the
retention window reclaims it). The tab then shows effective profiles, and the
"one endpoint, several protocols, only one real" case resolves itself after a
Real run.

## 2. Decisions

| # | Decision | Rationale |
| --- | --- | --- |
| D1 | The unit of purge is the **link** (`(protocol_id, endpoint_id)`), not the endpoint | A purged sibling must not drag a working protocol out of the Active view. The user's case, verbatim: "wrong protocols will be naturally wiped" |
| D2 | Evidence set = **strict + server-answered handshake** (taxonomy in §7) | The strongest available signals; 429/412/5xx and TLS `alert 40` are plausibly transient CDN behaviour, so they are excluded |
| D3 | A purge is reversed by a **manual real test from the Purgatory view**; feed-wide sweeps skip purged links | Phase 2 is the long pole (2.6 results/s). A sweep must not spend it re-proving what it already knows; a user asking about one row must be able to re-prove it |
| D4 | The dead `AppConfig.purgatory.enabled` field is **deleted** | It has no reader and no settings-form row; the feature needs no toggle |
| D5 | The purge is a **typed enum in a new `profile_stats` column**, never a text prefix | The page predicate (SQL) and the ordering law (Rust) must read it; `docs/database-manual-sql.md` §1 forbids a second spelling of a stored fact, and the error marker is swept while the purge must outlive it |
| D6 | Evidence is **typed where it happens**, never parsed from the message text | The repo's standing rule (`ProbeClass` doc comment, `ping.rs`); the status code is available as a value at every site that produces it |
| D7 | `PurgatoryView::Stale` is **renamed `Purgatory`** and its predicate becomes "not confirmed live and recent" | The view now holds two populations (aged links, purged links) under one question, and the old name would be a lie |

## 3. Data model

```rust
// crates/xray-tui-db/src/models_toasty.rs

/// Why a link was moved to Purgatory permanently. Fieldless on purpose: the
/// exact case IS the variant, so the row needs ONE nullable column and one
/// CHECK list — the status code and the human detail stay in `error_text`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed)]
pub enum PurgeReason {
    /// A REALITY handshake was answered by a real certificate: wrong
    /// `pbk`/`sid`, or the host is not a REALITY endpoint at all.
    RealityFallback,
    /// The server's certificate does not cover the configured host/SNI.
    CertificateMismatch,
    /// The server's certificate is expired.
    CertificateExpired,
    /// The port answered in cleartext: it is not the TLS endpoint the config
    /// claims.
    NotTls,
    /// The config as stored cannot dial (missing `pbk`, malformed request
    /// authority/URI, unknown xhttp mode, unusable PSK).
    ConfigInvalid,
    /// The proxy's OWN upgrade handshake was answered with an HTTP status
    /// (400/403/404/405/409/410/3xx).
    TransportRejected,
    /// The CDN answered an origin error (530/521/522/526): the origin behind
    /// the fronting host is gone.
    OriginUnreachable,
}
```

On `ProfileStats`:

```rust
    /// Permanent-purge verdict from the last real probe. `None` = live.
    /// Distinct fact from `error` (which is a measurement marker the TTL sweep
    /// clears): this one is only cleared by a data-carrying success.
    pub purge_reason: Option<PurgeReason>,
```

- **Physical shape (verified by probe, 2026-09-17).** toasty's derive emits
  `Option<PurgeReason>` as exactly ONE nullable `TEXT` column — no separate
  flag column — with a `CHECK` list of the variants rendered in snake_case:

  ```sql
  "purge_reason" TEXT CHECK ("purge_reason" IN ('reality_fallback',
    'certificate_mismatch', 'certificate_expired', 'not_tls', 'config_invalid',
    'transport_rejected', 'origin_unreachable'))
  ```

  Probe: a throwaway `toasty::Model` with `reason: Option<ProbeReason>` pushed
  through `push_schema` emitted `"reason" TEXT CHECK ("reason" IN ('alpha',
  'beta_gamma'))`, `Some(BetaGamma)` stored `'beta_gamma'` and `None` stored
  `NULL` (the probe was removed after the run). The hand-written writers must
  spell the variants the same way (`purge_reason_str`, an exhaustive `const fn`
  mirroring `error_kind_str`); a wrong spelling is refused by the `CHECK`, and
  the statement-vs-schema test is what proves the spellings match.
- **Schema tag 11 → 12.** A column cannot be added to an already-pushed table
  here (decision 4), so this is a **wipe**, not a migration.

```text
Data Destruction Guard
- Target Class: persistent-state (the live SQLite file, `~/.config/xray-tui/data.db`)
- Exact Target(s): the `profile_stats` re-create; every endpoint, link, protocol,
  test result and traffic counter on the developer machine
- Environment: local developer database, schema tag 11 (27,142 endpoints /
  21,113 protocols / 56,140 links, re-importable from its subscriptions)
- Why Irreversible: `push_schema` emits CREATE TABLE without IF NOT EXISTS and
  `open` deletes the file on a tag mismatch (decision 4); there is no migration
  machinery and none is planned while the database is re-importable fixture data
- Backup / Rollback Note: the file is 32.8 MB and untouched until the new binary
  opens it; copy `data.db{,-wal,-shm}` before the first run to keep the corpus
- Allowed Read-Only Next Steps: write the spec and plan; read the corpus
- Blocked Destructive Steps: shipping the tag bump without the copy above, and
  re-running the corpus before deciding the taxonomy is final
- Confirmation Required: yes — the plan's first task takes it as a gate
- Status: confirmed by the user 2026-09-17 (bump to 12; corpus wipe accepted)
```

## 4. The ordering law (decision 16 amendment)

`endpoint_rank::RankLink` gains one input, and the law stays in ONE place
(`RankLink::key`, which `EndpointRow::link_test_key` delegates to):

```rust
pub struct RankLink { /* … */ pub purged: bool }
```

- `key()`: `purged` → **tier 6**, below every live tier (0 real-ok … 5 DNS).
  A purged link can therefore never be an endpoint's representative key or
  display link while a live link exists; when every link is purged, the min is
  tier 6 and the endpoint still has a deterministic position for the
  Purgatory/All views.
- `compute_rank`: `newest_seen` is the maximum `seen_secs` over **live** links,
  `NO_SEEN` (`i64::MIN`) when there are none. This is the single fact that makes
  §5's predicate work.
- `display_link_index`: skips purged links (including a `manual_protocol_override`
  that names one — a pin must not resurrect a purged link as the representative);
  falls back to the purged set only when nothing live remains.
- `EndpointRow::select_best_measured_link`: same rule, so an `All`-view row's
  displayed delay never comes from a purged link unless the endpoint has no
  live link at all.

No new rank column and no new index: tier 6 rides `rank_tier`, and the
`rank_newest_seen` column keeps its `endpoint_rank_window` index. The covering
index `endpoint_rank_test` is unchanged.

## 5. Views

| View (`p` cycles Active → Purgatory → All) | Predicate | Links loaded into the page |
| --- | --- | --- |
| `Active` | `k.rank_newest_seen >= :active` | live only |
| `Purgatory` | `k.rank_newest_seen < :active` | all |
| `All` | `1 = 1` | all |

- Purgatory is one range on the **existing** window index, and it needs no
  `OR`: a zero-live-link endpoint has `newest_seen = NO_SEEN < active`, so it
  lands there by construction, together with the aged band.
- `PageRequest.stale_threshold` and the `purgatory_retention_secs` page bound
  are **retired**. The lower bound would make an endpoint invisible in both
  views when its only live links are older than retention while a purged link
  was refreshed today — a state this feature creates deliberately. The
  10-minute retention sweep still owns deletion.
- Hydration: `load_page_projection` and `load_page_rows` (the typed reference the
  parity test pins it against) take `include_purged: bool` and append
  `AND ps.purge_reason IS NULL` when false. Active passes `false`; Purgatory and
  All pass `true`. The batch's `load_feed_plan` passes `false` (D3); the
  selected-endpoint plan reads the already-loaded page, so it keeps them —
  which is exactly the existing `PlanSource::Feed` vs `PlanSource::Links` split.
- `PurgatoryView::Stale` → `Purgatory`; the tab label reads
  `View: Purgatory [P]`.

## 6. Evidence plumbing (typed, never parsed)

| Layer | Today | Change |
| --- | --- | --- |
| `xray-tui-tls/src/error.rs` | `Verify(String)`, `Handshake(String)` | add `CertNotValidForName(String)`, `CertExpired(String)`, `CleartextPeer(String)`; every `Display` keeps today's text |
| `xray-tui-tls/src/verify/mod.rs` | name mismatch (L154) and chain verify (L148) both `Verify` | L154 → `CertNotValidForName`; L148 matches `webpki::Error::CertExpired { .. }` → `CertExpired`, else `Verify` |
| `xray-tui-tls/src/record/mod.rs`, `record/stream.rs` | cleartext diagnosis → `Handshake` | → `CleartextPeer` |
| `xray-tui-native/src/error.rs` | `Tls(String)`, `Transport(String)` | add `TlsRejected { text, evidence }` and `TransportRejected { detail, status: u16 }`; both `Display` byte-identical to the strings they replace (so no log or test churn) |
| `xray-tui-native/src/security/mod.rs::map_tls_err` | flattens every `TlsError` to `Tls(String)` | maps the three typed variants to `TlsRejected`; `RealityFallback` stays `Reality`; everything else stays `Tls` |
| `transport/ws.rs` (2 sites), `httpupgrade.rs`, `v2rayhttp.rs`, `xhttp.rs` (3 sites) | status → `Transport(format!(…))` | → `TransportRejected { detail, status }` with the status from `resp.status()` / tungstenite's `Error::Http` |
| `xray-tui-native/src/error.rs` | — | `pub const fn evidence(&self) -> Option<FailureEvidence>`: ONE exhaustive match, the single owner of "what does this error prove". `Timeout`/`Dial`/`Io`/`Dns`-class/`Protocol`/`NotImplemented` → `None`; `Config(_)` → `ConfigDefect`; `Reality(_)` → `RealityFallback`; `TlsRejected`/`TransportRejected` → their typed kind |
| `xray-tui/src/ops/ping_native.rs` | `ProbeFailure { class, text }` | gains `evidence: Option<FailureEvidence>`, set only by `from_engine`. `local(…)` (probe-URL shape, unexpected status) and `ProbeOutcome::soft_failure(…)` carry `None` — **which is what keeps native-capability refusals out of the taxonomy** |
| `xray-tui/src/ops/ping.rs` | `ProbeOutcome::Failed { text, class, hard }` | gains `evidence` |
| `xray-tui/src/types.rs` `CoreEvent::SpeedTestResult` | `error: Option<String>` | gains `purge: Option<PurgeReason>` for the single-ping path |

The **policy** (evidence → reason) is one new TUI module,
`crates/xray-tui/src/ops/purge.rs`, with one function:

```rust
/// The single owner of "which probe evidence moves a config to Purgatory".
pub fn reason_for(evidence: FailureEvidence) -> Option<PurgeReason>
```

Product policy lives here (which statuses count), while engine truth stays in
`xray-tui-native`. A future taxonomy change touches this function.

## 7. Classification (D2)

| Evidence (typed) | Reason | Corpus 2026-09-17 |
| --- | --- | --- |
| REALITY auth fallback — real certificate | `RealityFallback` | 234 |
| cert not valid for the configured name | `CertificateMismatch` | 110 |
| cert chain verification failed with expiry | `CertificateExpired` | 59 |
| peer answered cleartext | `NotTls` | 37 |
| `NativeError::Config` from the engine (missing `pbk`, malformed authority/URI, unknown xhttp mode, unusable PSK) | `ConfigInvalid` | 53 |
| proxy's own handshake status ∈ {400, 403, 404, 405, 409, 410, 301, 302} | `TransportRejected` | ~1,000 |
| handshake status ∈ {530, 521, 522, 526} | `OriginUnreachable` | ~270 |

**Deliberately excluded** (no purge, marker only — unchanged behaviour):
TLS `alert 40`/`2 112`/`2 80` (1,285), statuses 429/412/500/503 (transient CDN /
rate-limit / bot-management, not config evidence), protocol framing failures
(vless header truncated, http2 error), every timeout (3,079), every fast-phase
failure (timeout / refused / no route / unreachable / DNS — 31,020), and
`not testable by the native engine` (198 — the subprocess core may serve them).

Measured impact of this table on the corpus: **1,687 links purged, 942 endpoints
touched, 721 endpoints leave the Active view** (2.7 % of 27,142), 221 keep a
live sibling. Phase 2 was 27 % done, so the real figure is ~3× larger.

## 8. Write path and lifecycle

- **Purge rides `LinkGroups::RESULT`** — same writer, same probe, one atomic
  outcome: `link_values_sql` gains the column, `link_patch_conflict_sql` gains
  `purge_reason = excluded.purge_reason` in its RESULT set, `upsert_link` (the
  typed full-row writer) round-trips it. `LINK_SOURCE_CONFLICT_SQL` (the import
  path) never writes it — same reasoning that keeps it off `latency`/`error`.
- **Set**: a real probe that fails with `Some(reason)` → `purge_reason = reason`.
  **Cleared** (two owners, both explicit): a successful `RealPing` — a tunnel
  built from this link's own config — or a successful `SpeedTest` — a transfer
  carried by the connected tunnel. **Never touched** by `TcpPing`/`UdpTest`
  (a TCP handshake proves nothing about the config) and never by `Cancelled`.
- `clear_expired_errors` (the `error_ttl_hours` sweep) writes only the error
  columns, so a purge outlives a swept marker — the permanence the user asked
  for.
- **Retention is unchanged**: `purge_expired` still deletes endpoints whose
  newest link (purged or not) is older than the retention window. A subscription
  that keeps re-confirming a purged config keeps it alive in Purgatory; once
  upstream drops it, the window reclaims it. That is the user's "until lifetime
  exceeds (no upstream confirmation)".
- `remove_failed_servers` / `is_removable_failure` are **not** changed: purge is
  not a deletion verdict, and retention owns deletion.

## 9. Surfaces

- View label `View: Purgatory [P]`; the `p` cycle and the `r` (restore/test on
  Purgatory) key handler keep their shape.
- A purged sub-row's Test cell shows the reason label (short forms:
  `[mitm]`, `[cert]`, `[expired]`, `[tls]`, `[cfg]`, `[http]`, `[origin]`)
  instead of a delay or a failure marker — a *status*, exactly as the untestable
  marker already outranks a delay.
- The panel shows purged links only in Purgatory/All; the Active panel is the
  "effective profiles" list. An endpoint row with no live link never appears in
  Active at all.
- Exact label strings and column placement are implementation-owned; the
  precedence rule above is not.

## 10. Non-goals

- No purge from fast-phase failures, from gRPC/telemetry, or from a connect.
- No purge for `not testable by the native engine` (subprocess-only capability).
- No auto-revert on a subscription refresh (a re-published config is not proof).
- No purge clearing on connect (`update_last_used` stamps recency only): a connect
  emits no result event, so a data-carrying success is the only reversal signal
  the pipeline actually reports.
- No settings toggle, no settings-form surface, no new config field (D4 deletes
  one instead).
- No manual "purge this row" / "restore this row" action.
- No migration machinery, no `ensure_column`, no second schema-tag meaning.
- No change to `purge_expired`, to retention semantics, or to `Protocol` rows
  (an orphan protocol stays legal).

## 11. Acceptance criteria

1. A real probe whose failure carries evidence sets the link's `purge_reason` to
   that reason, and the value survives `error_ttl_hours` expiry and a restart.
2. An endpoint with zero live links is absent from Active and present in
   Purgatory; a partially purged endpoint stays in Active and its panel lists
   only its live links.
3. A successful real test launched from the Purgatory view clears the reason and
   the link returns to Active on the next page load.
4. A feed-wide batch plans zero purged links; a selected-endpoint real test on a
   purged link does probe it.
5. Stored ordering keys still match the Rust oracle for all 8 sorts × both
   directions with purged links in the fixture, and an endpoint whose best link
   was purged orders by its best live link.
6. `not testable by the native engine` never sets a purge reason, and never
   disappears from the Active view.
7. `rank_newest_seen` equals the live-only maximum: an endpoint whose only fresh
   link is purged appears in Purgatory only.
8. The statement-vs-schema test exercises the new column on every raw path that
   reads or writes it (`PAGE_PROJECTION`, the rank readers, the link writers).

## 12. Verification

- `every_statement_runs_against_a_pushed_schema` (`tests/profiles_query.rs`) —
  must cover the projection and each view.
- `page_order_matches_the_rust_oracle_for_every_sort` + the parity test
  `page_projection_matches_the_orm_rows` with a purged-link fixture.
- New unit tests: `reason_for` over every `FailureEvidence` (including the
  excluded statuses), `NativeError::evidence` over every variant, `RankLink::key`
  tier 6, `compute_rank`'s live-only `newest_seen`, the display-link fallback.
- Behaviour tests on the write path: `apply_link_patches` RESULT group carries the
  column; a TRAFFIC-only patch leaves it alone; `update_scheduler_state` never
  writes it; the import path never writes it; the TTL sweep never clears it.
- Smoke run against the real corpus after the wipe: import, batch, then read the
  Active/Purgatory counts and compare with §7's projection.

## 13. Risks

| Risk | Mitigation |
| --- | --- |
| A `403` is Cloudflare bot-managing our TLS fingerprint, not a dead config | Excluded statuses cover the transient classes; the purge is reversible by one real test |
| The taxonomy turns out too aggressive or too timid | One function (`reason_for`), one column, no schema change to revise it |
| The tag bump destroys the corpus under study | Guard card in §3; copy the file before the first run |
| A purge on a link whose protocol row is shared with a live endpoint | Link-scoped by construction: `purge_reason` lives on the pair, `Protocol` rows are untouched |
| `All`-view rows looking odd (purged link with a delay) | The display-link rules in §4 keep the row's headline numbers on a live link |

## 14. Impact statement and ADR signal

```text
ImpactStatementDraft
- Layers: db schema + raw SQL writers (db), probe policy + event handlers (tui),
  error types (native, tls), UI view semantics (tui/ui)
- Owners: profile_stats (new fact), endpoint_rank (new input), profiles_query
  (predicate + projection), ops/purge.rs (new, the taxonomy's single owner)
- Invariants: decision 16 law has one implementation; SQL never re-derives it;
  one column group per writer; no text parsing of failures
- Compatibility: schema tag 11 → 12 = wipe (decision 4); no migration
- Retired: AppConfig.purgatory.enabled, PurgatoryView::Stale, PageRequest.stale_threshold
- Non-goals: see §10
- ADR signal: yes — ADR 0006 "Purge evidence is persisted per link; Purgatory is
  'not confirmed live and recent'" (owner: db + tui). Durable surfaces: a new
  source-of-truth fact, the view predicate, the ordering law's new input, three
  retirements. Status: proposed, to be written when the plan lands.
```
