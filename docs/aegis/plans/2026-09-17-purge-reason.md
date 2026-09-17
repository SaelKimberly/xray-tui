# 2026-09-17 — Purge reasons: evidence-based permanent moves to Purgatory

Parent spec: `docs/aegis/specs/2026-09-17-purge-reason-design.md` (approved by
the user 2026-09-17; the schema wipe is confirmed).

## Goal

A real probe that returns permanent evidence — a REALITY fallback, a cert the
config's SNI cannot match, a cleartext port, an origin the CDN cannot reach, a
`404` from the proxy's own upgrade — moves that **link** out of the Active view
permanently. Purgatory gains the second population it was designed for, the tab
shows effective profiles only, and a real test proves a purged link back to life.

## Architecture

Five layers, one direction of dependency:

```
xray-tui-tls   typed cert/cleartext causes ─┐
xray-tui-native typed evidence on the error ─┴→ xray-tui ops/purge.rs (policy)
                                                   ↓ ProbeOutcome.evidence
                                          ops/events.rs::apply_test_result
                                                   ↓ LinkGroups::RESULT
                                          xray-tui-db profile_stats.purge_reason
                                                   ↓
                                    endpoint_rank keys → profiles_query predicate
```

- `purge_reason` is a **new nullable column on `profile_stats`** and rides the
  existing RESULT writer group — no new write path, no new owner.
- The ordering law (`RankLink::key`) gains one input and sinks a purged link to
  **tier 6**; `compute_rank`'s `newest_seen` becomes the **live-only** maximum.
  No new rank column, no new index.
- The view predicate becomes `rank_newest_seen < active` for Purgatory — one
  range on the existing `endpoint_rank_window` index.
- The taxonomy (`which evidence purges`) lives in ONE function in the TUI crate.

## Tech Stack

Rust 2024, toasty 0.10 + turso, tokio, ratatui 0.30, thiserror.

## Baseline / Authority Refs

- `AGENTS.md` decisions 4 (schema tag = wipe), 16 (the ordering law), 20 (the
  native probe), 21 (the page query), 22 (write-behind, column groups).
- `docs/database.md` (schema + flows), `docs/database-manual-sql.md` §1/§3/§6
  (the rule, the site inventory, the checklist for a new raw statement).
- `docs/aegis/adr/0002` (column groups), `adr/0003` (stored ordering keys),
  `adr/0004` (the native real-ping engine).
- `docs/aegis/specs/2026-09-17-purge-reason-design.md` (this work's contract).

## Compatibility Boundary

- **Persisted schema: tag 11 → 12 = WIPE** (decision 4). Confirmed by the user;
  the guard card in spec §3 is `confirmed`. First step of T1 is the backup note.
- **No** protocol-identity, wire-format, or config-key change that alters
  existing behavior; one **dead** config key is removed (`purgatory.enabled`) and
  serde ignores it harmlessly (`AppConfig` has no `deny_unknown_fields`).
- **Text change, deliberate and listed**: `NativeError::Tls` currently renders a
  doubled prefix (`"TLS error: TLS error: certificate verification failed: …"`).
  T4 removes the accidental duplication (`"TLS error: certificate verification
  failed: …"`). No test or doc pins the doubled form (checked).
- The batch plan/`PageRequest` shapes change (T3 + T6); the in-flight
  batch-ping-pipeline plan also touches them — see **Sequencing**.

## TDD Route

```text
TDD Route:
- Mode: off (no activation config present; project has no TDD mandate — same
  record as plans/2026-09-17-batch-ping-pipeline.md)
- Decision: skipped
- Strict authority: not applicable
- Strict signals: recorded for completeness — persistence (new column), schema
  tag bump, cross-crate contract (NativeError shape), shared core (the ordering
  law), producer/consumer (probe → event → writer → rank → page)
- Light eligibility: n/a
- TDD-fit exception: n/a
- Test posture: post-change regression on the seams the change moves — the
  classifier over every FailureEvidence, the rank law's tier-6 sink and
  live-only newest_seen, the raw writers' column groups, the page predicate and
  the projection↔typed parity, plus the existing suite
- Reason: the risk is in restructuring an already-tested pipeline and in the
  *policy* the classifier encodes; the spec's §11 acceptance list is expressed
  as assertions on those seams, so focused regressions replace RED/GREEN
  ceremony
- Verification: `cargo nextest run -p xray-tui-db -p xray-tui-native
  -p xray-tui-tls -p xray-tui`, `cargo clippy --workspace --all-targets
  --all-features -- -D warnings`, `cargo fmt --all --check`, then the live run
  in T8
```

## Plan Basis

```text
Plan Basis:
- Spec: docs/aegis/specs/2026-09-17-purge-reason-design.md (approved)
- Evidence: the live corpus at ~/.config/xray-tui/data.db (schema tag 11,
  27,142 endpoints / 56,140 links) plus the recorded batch summary; the toasty
  column-shape probe of 2026-09-17 (one nullable TEXT column, snake_case CHECK)
- Requirement Ready Check: ready — spec §7 (taxonomy) and §11 (acceptance) are
  the contract; D1-D7 are user-approved
- BaselineUsageDraft: required = AGENTS.md 4/16/21/22, database-manual-sql §1/§6,
  ADR 0002/0003; cited = the column-group contract (T1), the rank law (T2), the
  page predicate (T3), the evidence rule (T4); missing = none; decision = continue
- Change Necessity: user-visible need = the tab keeps dead configs that a real
  probe has already judged; non-code option = none (a text prefix on `error_text`
  is swept by the error TTL and would be a second spelling of a stored fact,
  database-manual-sql §1); minimum boundary = the 15 files in the Files table
- Existence Check: one new surface (`ops/purge.rs`) — a 20-line pure function;
  rejected alternatives are "parse the message" (forbidden by the ProbeClass
  contract) and "encode in error_text" (swept + second spelling). Every other
  surface reuses an existing owner (spec §14)
- Architecture Integrity Lens: higher-level path TAKEN — Purgatory is expressed
  as "not confirmed live and recent" over the existing rank keys, so no new
  predicate column, no index, and no `EXISTS` subquery over profile_stats
- Plan Pressure Test: proceed (owner/contract/retirement per task; verification
  is the existing suite + the drift test + a live run)
```

```text
Complexity Budget:
- Artifact class: maintained source (db crate + tui crate)
- Target files / artifacts: xray-tui-db/src/{models_toasty,database,profiles_query,endpoint_rank}.rs,
  xray-tui/src/ops/{purge,ping,ping_native,events}.rs
- Current pressure: database.rs is 2,994 lines (+131 in flight) and already
  carries three raw-SQL groups (link writers, PRAGMAs, spellings); ops/ping.rs is
  2,977 lines (+21.9k plan in flight)
- Projected post-change pressure: database.rs +~40 lines (one column in two
  constants, one spelling fn, one test); ping.rs +~15; a NEW 45-line file
- Budget result: at-risk for ping.rs (the in-flight plan grows it further),
  within-budget for the rest
- Planned governance: the new decision logic does NOT go into ping.rs — it is
  extracted to ops/purge.rs (one function, one file); the db additions stay in
  the existing writer group they belong to rather than a new module
```

```text
Plan-Time Complexity Check:
- Target files: the Files table
- Existing size / shape signals: database.rs's writer group is cohesive (prefix,
  values, conflict, spellings); ping.rs's BatchShared is being restructured by
  another plan
- Owner fit: the classifier is policy (TUI), the engine's evidence is truth
  (native), the column is state (db) — each lands in its own crate
- Add-in-place risk: adding the taxonomy to `ops/ping.rs` would bury a product
  decision inside a 3k-line orchestration file and collide with the in-flight plan
- Better file boundary: `ops/purge.rs` (new) for the taxonomy; `database.rs`'s
  existing writer constants for the column
- Recommendation: add owner file (ops/purge.rs) + edit-in-place elsewhere
```

## Files

| Task | Create | Modify |
| --- | --- | --- |
| T1 | — | `crates/xray-tui-db/src/{models_toasty,database}.rs`, `crates/xray-tui-db/tests/integration.rs`, `crates/xray-tui-core/src/config_builder/mod.rs`, `crates/xray-tui/src/{state.rs,ops/{link_writer,profiles,scheduler}.rs}` (literal sweep) |
| T2 | — | `crates/xray-tui-db/src/{endpoint_rank,models_toasty}.rs` |
| T3 | — | `crates/xray-tui-db/src/{profiles_query,models_toasty}.rs`, `crates/xray-tui-db/tests/{integration,profiles_query}.rs`, `crates/xray-tui/src/ops/profiles.rs` |
| T4 | — | `crates/xray-tui-tls/src/{error,verify/mod,record/mod,record/stream}.rs`, `crates/xray-tui-native/src/{error,security/mod,transport/{ws,httpupgrade,v2rayhttp,xhttp},inbound/mod}.rs` |
| T5 | `crates/xray-tui/src/ops/purge.rs` | `ops/{mod,ping_native,ping,events}.rs`, `types.rs` |
| T6 | — | `crates/xray-tui/src/ui/{profiles,mod}.rs`, `crates/xray-tui-config/src/app_config.rs` |
| T7 | — | `AGENTS.md`, `docs/database.md`, `docs/database-manual-sql.md`, `CONTEXT.md`, `TUI_MANUAL.md`, `docs/aegis/INDEX.md`, `docs/aegis/adr/0006-purge-evidence.md` (new) |
| T8 | — | live run only (no source edits) |

## Sequencing

The in-flight `plans/2026-09-17-batch-ping-pipeline.md` (status: executing) also
rewrites `PageRequest` construction in `ops/ping.rs` and adds
`profiles_walk_page`. Two rules:

1. **Land that plan first, or rebase this one.** T3 removes
   `PageRequest.stale_threshold`, which its `load_feed_plan` constructs; T5/T6
   touch `ProbeOutcome`/`stage_result`, which it restructures. Do not run both
   concurrently in one worktree.
2. **The schema tag 12 wipe must be the only schema-affecting change in flight**
   — it deletes the worktree-independent `~/.config/xray-tui/data.db`.

Order inside this plan: T1 → T2 → T3 → T4 → T5 → T6 → T7 → T8. T4 is
independent of T1-T3 and may be delegated in parallel (it touches `xray-tui-tls`
and `xray-tui-native` only).

## Tasks

### T1 — The column and its writers (db)

**Files**: modify `crates/xray-tui-db/src/models_toasty.rs`,
`crates/xray-tui-db/src/database.rs`, and (literal sweep)
`crates/xray-tui-core/src/config_builder/mod.rs`,
`crates/xray-tui/src/{state.rs,ops/link_writer.rs,ops/profiles.rs,ops/scheduler.rs}`,
`crates/xray-tui-db/{src/profiles_query.rs,src/models_toasty.rs,tests/integration.rs,tests/profiles_query.rs}`.

**Why**: the purge verdict must be a durable, queryable, typed fact that the
error-TTL sweep cannot delete.

**Change Necessity**: `code-change`. No-config-only option exists: the page
predicate and the rank law are SQL/Rust, and a text carrier would be a second
spelling of a stored fact (`docs/database-manual-sql.md` §1).

**Steps**

1. **Before touching code, copy the live database** (the tag bump wipes it):
   `cp -a ~/.config/xray-tui/data.db{,-wal,-shm} /tmp/xray-tui-tag11-backup/ 2>/dev/null; mkdir -p /tmp/xray-tui-tag11-backup && cp -a ~/.config/xray-tui/data.db* /tmp/xray-tui-tag11-backup/`.

2. `models_toasty.rs` — add after `ErrorInfo`:

```rust
/// Why a link was moved to Purgatory permanently (spec
/// `2026-09-17-purge-reason-design.md` §7).
///
/// Fieldless on purpose: the exact case IS the variant, so the row needs ONE
/// nullable column and one CHECK list — the HTTP status and the human detail
/// stay in `error_text` beside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, toasty::Embed)]
pub enum PurgeReason {
    /// A REALITY handshake was answered by a real certificate: wrong
    /// `pbk`/`sid`, or the host is not a REALITY endpoint.
    RealityFallback,
    /// The server's certificate does not cover the configured host/SNI.
    CertificateMismatch,
    /// The server's certificate is expired (or not yet valid).
    CertificateExpired,
    /// The port answered in cleartext: not the TLS endpoint the config claims.
    NotTls,
    /// The config as stored cannot dial (missing `pbk`, malformed request
    /// authority/URI, unknown xhttp mode, unusable PSK).
    ConfigInvalid,
    /// The proxy's OWN upgrade handshake was answered with an HTTP status.
    TransportRejected,
    /// The CDN answered an origin error (530/521/522/526).
    OriginUnreachable,
}
```

3. Same file — add the field to `ProfileStats`, after `error`:

```rust
    /// Permanent-purge verdict from the last real probe.
    ///
    /// A distinct fact from `error` (a measurement marker the error-TTL sweep
    /// clears): only a data-carrying success clears this one. Not indexed: the
    /// page predicate reads the derived rank keys, and the rank refresh reads
    /// this column per endpoint.
    pub purge_reason: Option<PurgeReason>,
```

4. `database.rs` — bump the tag and record it (after the `11 =` comment block):

```rust
        // 12 = `profile_stats.purge_reason` (spec 2026-09-17-purge-reason):
        // the typed, permanent verdict a real probe's evidence writes and only
        // a data-carrying success clears. A v11 file is WIPED (a column cannot
        // be added to a pushed table here; the verdicts are re-derived by the
        // next Real run).
        const SCHEMA_VERSION: i64 = 12;
```

5. Same file — `LINK_UPSERT_PREFIX` gains the column before the traffic group:

```rust
const LINK_UPSERT_PREFIX: &str = "INSERT INTO profile_stats (protocol_id, endpoint_id, core_type, \
     config_type, last_used_at, last_seen_at, latency, latency_delay, latency_ip, speed_bps, \
     error, error_kind, error_text, purge_reason, traffic_today_up, traffic_today_down, \
     traffic_total_up, traffic_total_down, created_at, updated_at, version) VALUES ";
```

6. Same file — `link_values_sql`, right after the `error` match (before the
   traffic tuple):

```rust
    let _ = write!(
        sql,
        ", {}",
        sql_opt_lit(link.purge_reason.map(purge_reason_str))
    );
```

7. Same file — the spellings helper, beside `error_kind_str`:

```rust
/// The `CHECK`-constrained storage text of a [`PurgeReason`] variant, spelled
/// the way toasty's derive renders it (snake_case; verified by the column-shape
/// probe of 2026-09-17). A wrong spelling is refused by the column CHECK, and
/// `purge_reason_rides_the_result_group_only` executes this statement.
const fn purge_reason_str(reason: crate::models_toasty::PurgeReason) -> &'static str {
    use crate::models_toasty::PurgeReason;
    match reason {
        PurgeReason::RealityFallback => "reality_fallback",
        PurgeReason::CertificateMismatch => "certificate_mismatch",
        PurgeReason::CertificateExpired => "certificate_expired",
        PurgeReason::NotTls => "not_tls",
        PurgeReason::ConfigInvalid => "config_invalid",
        PurgeReason::TransportRejected => "transport_rejected",
        PurgeReason::OriginUnreachable => "origin_unreachable",
    }
}
```

8. Same file — the **PURGE group**. `LinkGroups`:

```rust
    /// `latency` + `speed_bps` + `error` (ping results, error TTL sweeps).
    pub const RESULT: Self = Self(0b001);
    /// `purge_reason` (the real probe's verdict). Its own bit because
    /// `link_patch_conflict_sql` writes a FIXED column set from each patch's
    /// snapshot: riding RESULT would let a phase-1 fast half — whose snapshot
    /// is the plan-time row — rewrite the verdict it never classified, which is
    /// the clobber shape ADR 0002's groups exist to prevent.
    pub const PURGE: Self = Self(0b010);
    /// `traffic_*` (the gRPC stats poller).
    pub const TRAFFIC: Self = Self(0b100);
    /// Every group.
    pub const ALL: Self = Self(0b111);
```

9. Same file — `link_patch_conflict_sql` takes the third flag:

```rust
fn link_patch_conflict_sql(has_result: bool, has_traffic: bool, has_purge: bool) -> String {
    let mut sets: Vec<&str> = Vec::with_capacity(12);
    if has_result {
        sets.extend_from_slice(&[
            "latency = excluded.latency",
            "latency_delay = excluded.latency_delay",
            "latency_ip = excluded.latency_ip",
            "speed_bps = excluded.speed_bps",
            "error = excluded.error",
            "error_kind = excluded.error_kind",
            "error_text = excluded.error_text",
        ]);
    }
    if has_purge {
        sets.push("purge_reason = excluded.purge_reason");
    }
    if has_traffic {
        sets.extend_from_slice(&[
            "traffic_today_up = excluded.traffic_today_up",
            "traffic_today_down = excluded.traffic_today_down",
            "traffic_total_up = excluded.traffic_total_up",
            "traffic_total_down = excluded.traffic_total_down",
        ]);
    }
    /* … unchanged tail: DO NOTHING when `sets` is empty, else updated_at +
       version = version + 1 … */
}
```

   and `apply_link_patches`'s bucket loop becomes the eight shapes:

```rust
        for (has_result, has_purge, has_traffic) in [
            (true, true, true),
            (true, true, false),
            (true, false, true),
            (true, false, false),
            (false, true, true),
            (false, true, false),
            (false, false, true),
            (false, false, false),
        ] {
            let shape: Vec<&LinkPatch> = patches
                .iter()
                .filter(|p| {
                    p.groups.contains(LinkGroups::RESULT) == has_result
                        && p.groups.contains(LinkGroups::PURGE) == has_purge
                        && p.groups.contains(LinkGroups::TRAFFIC) == has_traffic
                })
                .collect();
            if shape.is_empty() {
                continue;
            }
            let conflict = link_patch_conflict_sql(has_result, has_traffic, has_purge);
            /* … unchanged chunk/write … */
        }
```

10. Same file — `upsert_link` (the typed full-row writer): add after `.error(...)`:

```rust
            .purge_reason(s.purge_reason)
```

11. `LinkWriter` (`ops/link_writer.rs`) — three groups instead of two:

```rust
    pub fn stage(&self, link: &ProfileStats, groups: LinkGroups) {
        for flag in [LinkGroups::RESULT, LinkGroups::PURGE, LinkGroups::TRAFFIC] {
            if groups.contains(flag) {
                self.pending
                    .insert(((link.protocol_id, link.endpoint_id), flag), link.clone());
            }
        }
        if self.pending.len() >= self.flush_rows {
            self.wake.notify_one();
        }
    }
```

   and `merge_group`:

```rust
fn merge_group(base: &mut ProfileStats, flag: LinkGroups, staged: &ProfileStats) {
    if flag == LinkGroups::RESULT {
        base.latency.clone_from(&staged.latency);
        base.speed_bps = staged.speed_bps;
        base.error.clone_from(&staged.error);
    } else if flag == LinkGroups::PURGE {
        base.purge_reason = staged.purge_reason;
    } else {
        base.traffic = staged.traffic;
    }
}
```

12. **Literal sweep** — the compiler lists every site: add `purge_reason: None`
    to each `ProfileStats { … }` literal (28 sites; the integration/unit test
    helpers among them). Run
    `cargo check --workspace --all-targets` until clean.

13. **Delivered as**: the group-discipline proof is an EXTENSION of
    `apply_link_patches_isolates_column_groups` in `tests/integration.rs`, not a
    new `database.rs` unit test — it already seeds every group
    (`LinkGroups::ALL`), and it runs the real multi-row statement rather than a
    second hand-built patch. It now seeds a verdict, then asserts a stale
    RESULT-only patch (`purge_reason: None` in the snapshot) leaves it intact,
    then that a PURGE-only patch moves it while latency and traffic stay put.
    `docs/database-manual-sql.md` §3/§6 name that test as the statement's pin.

14. Extend `bulk_upserts_are_idempotent_and_preserve_owned_fields` in
    `crates/xray-tui-db/tests/integration.rs`: before the refresh block, stamp a
    verdict through a PURGE patch; after the refresh block's assertions, add

```rust
    assert_eq!(
        row[0].links[0].purge_reason,
        Some(xray_tui_db::models::PurgeReason::CertificateExpired),
        "a refresh snapshot (SOURCE only) does not clear a purge verdict"
    );
```

**Impact**: the RESULT group's column set grows by one; the import path and
`update_scheduler_state` are untouched, so no writer gains authority over it.

**Verification**: `cargo nextest run -p xray-tui-db` and
`cargo check --workspace --all-targets`.

### T2 — The ordering law (db)

**Files**: modify `crates/xray-tui-db/src/endpoint_rank.rs` (and the
`RankLink` literal sweep inside it).

**Why**: without this, a purged link is still an endpoint's representative key
and display link, so a purged-only endpoint keeps its old position and its
headline delay.

**Change Necessity**: `code-change`. Config/docs cannot express it: the keys are
derived state computed in Rust and stored in SQL, and leaving purged links in
the input would make the stored keys disagree with what the tab must show. The
minimum boundary is `endpoint_rank.rs` plus the field on `RankLink` — no new
column, no new index.

**Steps**

1. `RankLink` gains `pub purged: bool` (doc: "the link carries a purge verdict —
   it can no longer represent the endpoint while a live link exists").

2. `RankLink::key` — the purge band sits BELOW every live band:

```rust
    pub const fn key(&self, dns_unresolved: bool) -> (u8, i32, i64, i64) {
        let tier = if self.purged {
            // 6: purged. Below every live tier, so an endpoint's representative
            // key is a live link whenever one exists, and an endpoint whose
            // links are all purged still has a deterministic position.
            6
        } else if dns_unresolved {
            5
        } else if let Some(kind) = &self.error_kind {
            match kind {
                ProfileErr::Real | ProfileErr::Name => 3,
                ProfileErr::Fast => 4,
            }
        } else {
            match self.measured {
                Some(true) => 0,
                Some(false) => 1,
                None => 2,
            }
        };
        let latency = if tier <= 1 { self.delay } else { i32::MAX };
        (tier, latency, -self.seen_secs, self.protocol_id)
    }
```

3. `display_link_index`: live links own the display while any exists. `display_rank`
   itself is NOT changed — the eligibility gate lives in this function, and it
   must cover the `manual_protocol_override` branch too (`display_rank -> None`
   would not: the override branch early-returns on the protocol id before ever
   consulting the rank, so a pinned purged link would still source
   `rank_display_seen`/`rank_speed`/`rank_traffic`/`rank_config` — i.e. the
   Active view's Speed/Traffic/ConfigType/LastSeen sorts would read a purged
   link). Allocation-free, two passes:

```rust
    /// Index of the endpoint's display link.
    ///
    /// The rule the retired SQL used (manual override, else the best measured
    /// link), plus the purge gate: a purged link never sources the endpoint's
    /// displayed values while a live link exists — the tier-6 sink applied to
    /// the display preference, including a manual override that names one. An
    /// endpoint whose links are ALL purged falls back to them, so a Purgatory
    /// row still shows the values it has.
    #[must_use]
    pub fn display_link_index(links: &[RankLink], override_protocol: Option<i64>) -> Option<usize> {
        let live = links.iter().any(|l| !l.purged);
        let eligible = |l: &RankLink| !live || !l.purged;
        if let Some(pid) = override_protocol
            && let Some(index) = links
                .iter()
                .position(|l| l.protocol_id == pid && eligible(l))
        {
            return Some(index);
        }
        links
            .iter()
            .enumerate()
            .filter(|(_, l)| eligible(l))
            .filter_map(|(i, l)| {
                l.display_rank()
                    .map(|rank| (i, rank, l.delay, l.protocol_id))
            })
            .min_by_key(|&(_, rank, delay, protocol)| (rank, delay, protocol))
            .map(|(i, _, _, _)| i)
    }
```

   `EndpointRow::select_best_measured_link` takes the same `live` gate, so the
   row's headline delay follows the same rule.

4. `impl From<&ProfileStats> for RankLink`: `purged: link.purge_reason.is_some()`.

5. `compute_rank`: `newest_seen` over **live** links only:

```rust
    // The view windows ask whether any LIVE link falls in the band. A
    // purged-only endpoint therefore reports NO_SEEN, which is below every
    // window bound — that is exactly "it belongs to Purgatory" (spec §5).
    let newest_seen = links
        .iter()
        .filter(|l| !l.purged)
        .map(|l| l.seen_secs)
        .max()
        .unwrap_or(NO_SEEN);
```

6. `refresh`'s raw link read: append `purge_reason` to the SELECT list (last, at
   index 10) and map it — a NULL column is a live link, any text is a verdict:

```rust
        "SELECT endpoint_id, protocol_id, error_kind, latency, latency_delay, \
         last_seen_at, speed_bps, traffic_total_up, traffic_total_down, config_type, \
         purge_reason \
         FROM profile_stats WHERE endpoint_id IN ({id_list})"
```

```rust
            purged: field(10).and_then(as_text).is_some(),
```

   (`as_text` already returns `Option<String>`, so a NULL column yields `None`
   and any stored spelling yields `Some` — the value itself is not read here,
   only its presence.)

7. Tests (extend `endpoint_rank.rs`'s module and
   `models_toasty.rs`'s `EndpointRow` tests):
   - a purged link with the best measurement does NOT become the representative
     key or the display link while a live sibling exists;
   - **a `manual_protocol_override` naming a purged link is not the display link
     while a live sibling exists** (the branch that `display_rank -> None` could
     not cover);
   - an all-purged endpoint: tier 6, `newest_seen == NO_SEEN`, and its display
     link IS one of its purged links (the fallback);
   - `sort_links_by_test_priority` sinks purged links below a fast-error sibling;
   - `select_best_measured_link` ignores a purged link's delay while a live link
     exists, and takes it when none does.

**Verification**: `cargo nextest run -p xray-tui-db` (the existing
`page_order_matches_the_rust_oracle_for_every_sort` must stay green — it pins the
law, not the SQL).

### T3 — The view (db + tui)

**Files**: modify `crates/xray-tui-db/src/profiles_query.rs`,
`crates/xray-tui-db/src/models_toasty.rs`,
`crates/xray-tui-db/tests/{integration,profiles_query}.rs`,
`crates/xray-tui/src/ops/profiles.rs`.

**Why**: this is the user-visible behavior — the Active view stops listing
purged links, and Purgatory lists both populations.

**Change Necessity**: `code-change`. The predicate is SQL and the panel's link
set is decided by the loader; neither is a configuration surface. The minimum
boundary is the two loaders, the predicate, and `PageRequest` — the retirement
of `stale_threshold` is the direct consequence, not extra scope.

**Steps**

1. `models_toasty.rs`: `PurgatoryView::Stale` → `Purgatory` (+ doc: "links whose
   newest LIVE link is older than the TTL, plus endpoints with no live link").

2. `profiles_query.rs` — `view_predicate`:

```rust
fn view_predicate(sql: &mut Sql, req: &PageRequest) {
    // "Some link falls in the band" is the same question as "the newest LIVE
    // link does" (`rank_newest_seen` is the live-only maximum, spec §5): an
    // endpoint with no live link reports NO_SEEN and lands in Purgatory by
    // construction, which is why one range serves both populations and no
    // `OR` (which would defeat the window index) is needed.
    match req.view {
        PurgatoryView::All => sql.push("1 = 1"),
        PurgatoryView::Active => {
            let ts = sql.bind(req.active_threshold);
            sql.push(&format!("k.rank_newest_seen >= {ts}"));
        }
        PurgatoryView::Purgatory => {
            let ts = sql.bind(req.active_threshold);
            sql.push(&format!("k.rank_newest_seen < {ts}"));
        }
    }
}
```

3. `profiles_query.rs` — delete `PageRequest.stale_threshold` (the field, its
   doc, and the `page_request` construction in `ops/profiles.rs`; the retention
   bound is not a page concern). Update the 13 call sites the compiler lists.

4. `profiles_query.rs` — the page loaders take the view's purge policy:

```rust
    /// One page's rows. `include_purged` is the Active view's filter: the
    /// Active panel is the effective-profiles list, so a purged sibling of a
    /// live link is hidden there and shown in Purgatory/All (spec §5).
    pub async fn load_page_projection(
        &self,
        ids: &[EndpointId],
        include_purged: bool,
    ) -> Result<Vec<EndpointRow>>
```

   Inside the statement, push `AND ps.purge_reason IS NULL` when
   `!include_purged`. Apply the same flag in `Database::load_page_rows` (the
   typed reference) by filtering the loaded links:

```rust
        let mut rows = self.load_endpoint_rows(endpoints, &mut conn).await?;
        if !include_purged {
            for row in &mut rows {
                row.links.retain(|l| l.purge_reason.is_none());
            }
        }
```

   (`load_endpoint_rows` itself stays unfiltered: `get_endpoint` /
   `get_endpoint_by_protocol_id` must still see a purged link — that is how it
   gets tested back to life.)

5. `ops/profiles.rs` — `load_profiles_rows` / `load_profiles_page_only` pass
   `matches!(load.view, PurgatoryView::Purgatory | PurgatoryView::All)`;
   `page_request` drops `stale_threshold`; `ProfilesLoad` drops
   `purgatory_retention_secs` (keep `AppState.purgatory_retention_secs` — the
   retention task in `main.rs` still owns the sweep) and
   `cycle_purgatory_view` reads Purgatory.

6. Tests:
   - `ops/profiles.rs`: `stale_shows_the_aging_band_only` → a Purgatory fixture
     with two populations: one endpoint whose newest live link is older than the
     TTL, one whose only link carries a purge verdict (assert both are listed in
     Purgatory and neither in Active);
   - `tests/integration.rs` + `tests/profiles_query.rs`: `page_req` loses the
     `stale` argument; add a purged-link fixture asserting Active hides the
     endpoint and Purgatory lists it, and that a partially purged endpoint's
     Active row has one link while its Purgatory row has two;
   - `every_statement_runs_against_a_pushed_schema` must exercise all three views
     through `load_page_projection` with both `include_purged` values.

**Verification**: `cargo nextest run -p xray-tui-db -p xray-tui`.

### T4 — Typed evidence (native + tls)

**Files**: modify `crates/xray-tui-tls/src/{error,verify/mod,record/mod,record/stream}.rs`,
`crates/xray-tui-native/src/{error,security/mod,inbound/mod}.rs`,
`crates/xray-tui-native/src/transport/{ws,httpupgrade,v2rayhttp,xhttp}.rs`.

**Why**: the taxonomy must read typed causes, never the rendered message — the
same rule `ProbeClass` already follows. This task is independent of T1-T3 and
may run in parallel.

**Change Necessity**: `code-change`. The evidence is a runtime fact known only
inside the engine's error values; no config or doc can carry it, and the
alternative — parsing the rendered message — is exactly what the repo's
`ProbeClass` contract forbids. The minimum boundary is the three TLS variants,
the two `NativeError` variants, one accessor, and the status-bearing sites.

**Steps**

1. `xray-tui-tls/src/error.rs` — three variants beside `RealityFallback`, each
   rendering the same text `Verify`/`Handshake` produced for that case:

```rust
    /// The server's certificate does not cover the configured server name.
    #[error("certificate verification failed: server name mismatch: {0}")]
    CertNotValidForName(String),
    /// Chain verification failed because a certificate is expired (or not yet
    /// valid).
    #[error("certificate verification failed: chain verification failed: {0}")]
    CertExpired(String),
    /// The peer answered in cleartext instead of a TLS record.
    #[error("handshake error: peer does not speak TLS: it answered in cleartext \
             ({0}) — the endpoint's port or its `security` setting is wrong")]
    CleartextPeer(String),
```

2. `verify/mod.rs` — the chain-verify arm matches the typed `webpki::Error`
   instead of stringifying it:

```rust
            .map_err(|e| match e {
                // The chain walk is where expiry and name mismatch surface; the
                // SAN check below repeats the name case.
                webpki::Error::CertExpired { .. } | webpki::Error::CertNotValidYet { .. } => {
                    TlsError::CertExpired(e.to_string())
                }
                webpki::Error::CertNotValidForName(_) => {
                    TlsError::CertNotValidForName(e.to_string())
                }
                other => TlsError::Verify(format!("chain verification failed: {other}")),
            })?;
```

   and line ~154: `.map_err(|e| TlsError::CertNotValidForName(e.to_string()))?`.

3. `record/mod.rs` — the `does not speak TLS` site in `read_record_into`:
   `TlsError::Handshake(format!("peer does not speak TLS: …"))` →
   `TlsError::CleartextPeer(format!("{cleartext:?}"))`. This is the path the
   probe takes (it reads the server flight through `read_record_into`); the
   `TlsStream::poll_read` twin in `record/stream.rs` stays an `io::Error` — an
   `AsyncRead` impl cannot return a `TlsError`, and its text is unchanged, so
   that arm carries no evidence.

4. `xray-tui-native/src/error.rs`:

```rust
/// What a failure PROVES about the endpoint, when it points at a permanent
/// defect instead of a transient one (spec `2026-09-17-purge-reason-design.md`
/// §6). Produced from the typed error where the failure happened, never parsed
/// out of a message. The TUI's purge policy is the only consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureEvidence {
    /// A REALITY handshake was answered by a real certificate.
    RealityFallback,
    /// The peer's certificate does not cover the configured server name.
    CertNotValidForName,
    /// The peer's certificate is expired.
    CertExpired,
    /// The peer answered in cleartext.
    CleartextPeer,
    /// The config as stored cannot dial.
    ConfigDefect,
    /// The peer's own answer to our handshake, when it is an HTTP status.
    HttpRejected(u16),
}
```

   three new variants, plus the accessor:

```rust
    #[error("TLS error: {0}")]
    CertNotValidForName(String),
    #[error("TLS error: {0}")]
    CertExpired(String),
    #[error("TLS error: {0}")]
    CleartextPeer(String),
    /// The peer answered our own transport handshake with an HTTP status.
    /// `detail` is the message the `Transport(String)` arm used to carry.
    #[error("transport error: {detail}")]
    TransportRejected { detail: String, status: u16 },
```

```rust
impl NativeError {
    /// What this failure proves, when it proves anything permanent. `None` is
    /// the default and the honest answer for a timeout, a dial failure, an I/O
    /// error, a framing error, or a client-side capability gap.
    #[must_use]
    pub const fn evidence(&self) -> Option<FailureEvidence> {
        match self {
            Self::Config(_) => Some(FailureEvidence::ConfigDefect),
            Self::Reality(_) => Some(FailureEvidence::RealityFallback),
            Self::CertNotValidForName(_) => Some(FailureEvidence::CertNotValidForName),
            Self::CertExpired(_) => Some(FailureEvidence::CertExpired),
            Self::CleartextPeer(_) => Some(FailureEvidence::CleartextPeer),
            Self::TransportRejected { status, .. } => {
                Some(FailureEvidence::HttpRejected(*status))
            }
            Self::Dial(_)
            | Self::Tls(_)
            | Self::Transport(_)
            | Self::Protocol { .. }
            | Self::NotImplemented { .. }
            | Self::Io(_)
            | Self::Timeout { .. } => None,
        }
    }
}
```

5. `security/mod.rs::map_tls_err` — map the typed causes and drop the accidental
   doubled prefix that `format!("TLS error: {other}")` produced:

```rust
fn map_tls_err(e: TlsError) -> NativeError {
    match e {
        TlsError::RealityFallback => NativeError::Reality(
            "REALITY: received real certificate (potential MITM or redirection)".into(),
        ),
        TlsError::CertNotValidForName(detail) => NativeError::CertNotValidForName(detail),
        TlsError::CertExpired(detail) => NativeError::CertExpired(detail),
        TlsError::CleartextPeer(detail) => NativeError::CleartextPeer(detail),
        // The variant's own Display already carries the `TLS error: ` prefix
        // (the old payload pre-pended a second one).
        other => NativeError::Tls(other.to_string()),
    }
}
```

6. The five status sites become `TransportRejected`, with the detail string
   unchanged:
   - `transport/ws.rs` (the `client_async` error): match
     `tokio_tungstenite::tungstenite::Error::Http(resp)` →
     `NativeError::TransportRejected { detail: format!("ws handshake: HTTP error: {}", resp.status()), status: resp.status().as_u16() }`,
     everything else stays `Transport(format!("ws handshake: {other}"))`;
   - the following `if resp.status() != SWITCHING_PROTOCOLS` guard → same shape
     with `format!("ws upgrade rejected: {}", resp.status())`;
   - `transport/httpupgrade.rs` (the `expected 101` site);
   - `transport/v2rayhttp.rs` (the `expected 200` site);
   - `transport/xhttp.rs`'s `send_200` (the `expected 200` site).

7. `inbound/mod.rs`'s exhaustive `NativeError` match (the SOCKS reply-code
   mapping) and the tests that match the old variants
   (`xhttp.rs:1388`, `xhttp.rs:2312`, `httpupgrade.rs:198`) move to the new
   variant — the compiler lists every site.

7b. **Honesty pass the taxonomy forces** (done in the follow-up commit, with
   T4): the request-BUILD sites report `NativeError::Transport` for a config
   that cannot compose a request — `ws.rs` (`ws request:`, `ws host:`),
   `v2rayhttp.rs` (`uri build`, `request build`), `httpupgrade.rs` (request
   build), `xhttp.rs` (`xhttp request build`). Those move to `Config`: the row
   cannot dial as stored, which is what `Config` now means, and the approved
   taxonomy (D2) includes malformed Host/URI configs. Without it, ~40
   structurally broken configs would sit in the Active view forever.
   `now_unix_secs` moves the other way (`Config` → `Io`) — a broken clock is not
   a row defect.

8. Tests: `xray-tui-tls` — a name-mismatch handshake yields
   `TlsError::CertNotValidForName`, an expired chain yields `CertExpired`, a
   cleartext peer yields `CleartextPeer` (extend the existing verify/record
   suites, which already build those fixtures).
   `xray-tui-native` — `NativeError::evidence()` over every variant, including
   the `None` set (`Timeout`, `Dial`, `Io`, `Protocol`, `NotImplemented`,
   `Tls`, `Transport`). **Plus the target-leg guard**: a probe-target TLS failure
   (the `gstatic` leg inside `probe.rs::fetch_over`, which reports plain
   `NativeError::Tls`) must yield `evidence() == None` — otherwise an
   implementer "unifying" that mapping with `map_tls_err` would purge a healthy
   config for the probe target's certificate.

**Impact**: no behavior change; the only visible difference is the removed
doubled prefix on TLS messages (listed in Compatibility Boundary).

**Verification**: `cargo nextest run -p xray-tui-tls -p xray-tui-native`.

### T5 — The policy and the write path (tui)

**Files**: create `crates/xray-tui/src/ops/purge.rs`; modify
`crates/xray-tui/src/ops/mod.rs`, `ops/ping_native.rs`, `ops/ping.rs`,
`ops/events.rs`, `types.rs`.

**Why**: the evidence must reach `profile_stats` through the one existing result
writer, and the "which evidence purges" decision must have a single owner.

**Change Necessity**: `code-change`. The evidence exists only as a runtime value
on the probe path; the minimum boundary is one new pure function plus the
parameter that carries its input to the existing writer — no new write path, no
new event, no second mapping.

**Steps**

1. `ops/mod.rs`: `pub mod purge;`.

2. `ops/purge.rs` (new, the whole file):

```rust
//! The purge policy: which probe evidence moves a config to Purgatory.
//!
//! This module is the ONLY owner of that decision (spec
//! `2026-09-17-purge-reason-design.md` §6): the engine reports typed evidence,
//! and the taxonomy — which classes count, and which are plausibly transient —
//! is product policy that lives here, in one function.

use xray_tui_db::models::PurgeReason;
use xray_tui_native::error::FailureEvidence;

/// The verdict a failed real probe's evidence earns, or `None` for
/// "inconclusive: probe it again later".
///
/// Deliberately NOT purging: timeouts, dial/DNS/refusal failures, TLS alerts,
/// protocol framing failures, and `429`/`412`/`500`/`503` — each is
/// indistinguishable from a transient network or CDN condition, or names a
/// client-side condition the server never judged (spec §7).
#[must_use]
pub const fn reason_for(evidence: FailureEvidence) -> Option<PurgeReason> {
    match evidence {
        FailureEvidence::RealityFallback => Some(PurgeReason::RealityFallback),
        FailureEvidence::CertNotValidForName => Some(PurgeReason::CertificateMismatch),
        FailureEvidence::CertExpired => Some(PurgeReason::CertificateExpired),
        FailureEvidence::CleartextPeer => Some(PurgeReason::NotTls),
        FailureEvidence::ConfigDefect => Some(PurgeReason::ConfigInvalid),
        FailureEvidence::HttpRejected(status) => match status {
            // The CDN could not reach the origin: the config's backend is gone.
            521 | 522 | 526 | 530 => Some(PurgeReason::OriginUnreachable),
            // The proxy's own answer to our exact request: the path/host is
            // wrong or the server refuses this config.
            301 | 302 | 400 | 403 | 404 | 405 | 409 | 410 => {
                Some(PurgeReason::TransportRejected)
            }
            _ => None,
        },
    }
}
```

3. `ops/ping_native.rs` — `ProbeFailure` carries the evidence:

```rust
pub struct ProbeFailure {
    pub class: ProbeClass,
    pub text: String,
    /// The typed proof, when the engine reported one (spec §7). `None` for a
    /// failure this module raised itself — including the probe-URL shape and
    /// the "unexpected status" of the probe's own target request.
    pub evidence: Option<FailureEvidence>,
}
```

   `from_engine` sets `evidence: err.evidence()`; `local(...)` sets `None`.

4. `ops/ping.rs` — `ProbeOutcome::Failed` gains
   `evidence: Option<FailureEvidence>`; the fast runner sets `None`, the real
   runner forwards `e.evidence`. `soft_failure(...)` keeps `None` — that is what
   keeps `not testable by the native engine` out of the taxonomy.

5. `ops/ping.rs` — `BatchShared::stage_result`: pass the verdict through the one
   shared mapping:

```rust
        let purge = match outcome {
            ProbeOutcome::Failed { evidence, .. } => {
                evidence.and_then(crate::ops::purge::reason_for)
            }
            ProbeOutcome::Ok { .. } => None,
        };
        if crate::ops::events::apply_test_result(
            &mut row, test_type, latency_ms, None, ip_info, error, purge,
        ) {
            self.writer.stage(&row, LinkGroups::RESULT);
        }
```

6. `types.rs` — `CoreEvent::SpeedTestResult` gains
   `purge: Option<xray_tui_db::models::PurgeReason>`; every construction site
   passes `None` except the real-ping failure path, which passes
   `e.evidence.and_then(purge::reason_for)`.

7. `ops/events.rs` — `apply_test_result` gains the parameter and the two rules:

```rust
pub(crate) fn apply_test_result(
    row: &mut ProfileStats,
    test_type: TestType,
    latency_ms: Option<u64>,
    speed_bps: Option<u64>,
    ip_info: Option<&str>,
    error: Option<&str>,
    purge: Option<PurgeReason>,
) -> bool {
    if let Some(err) = error {
        if err == "Cancelled" {
            return false;
        }
        row.error = Some(ErrorInfo {
            kind: err_kind_for(test_type),
            text: err.to_string(),
        });
        // Only a real probe carries evidence; a fast result can never set a
        // verdict (it proves nothing about the config).
        if matches!(test_type, TestType::RealPing) && let Some(reason) = purge {
            row.purge_reason = Some(reason);
        }
        return true;
    }
    match test_type {
        TestType::RealPing => {
            row.latency = latency_ms.map(|ms| Latency::Real {
                delay: ms as i32,
                ip: exit_ip(ip_info),
            });
        }
        TestType::TcpPing | TestType::UdpTest => {
            row.latency = latency_ms.map(|ms| Latency::Fast { delay: ms as i32 });
        }
        TestType::SpeedTest => row.speed_bps = speed_bps.map(|v| v as i64),
    }
    row.error = None;
    // A data-carrying success THROUGH the tunnel proves the config works: a
    // real probe built the tunnel from this link's own config, a speed test
    // carried bytes over it. A TCP handshake proves neither (spec §8).
    if matches!(test_type, TestType::RealPing | TestType::SpeedTest) {
        row.purge_reason = None;
    }
    true
}
```

   The `SpeedTestResult` handler in `poll_core_events` forwards the event's
   `purge` into the mapping.

8. Tests:
   - `ops/purge.rs`: every `FailureEvidence` maps as the table says, including
     the excluded statuses (`429`, `412`, `500`, `503`, and `530`'s neighbours
     `520`/`521`/`522`/`526` pinned explicitly);
   - `ops/events.rs`: a real failure with a verdict sets it; a fast failure with
     a verdict does not; a `RealPing` success clears it; a `SpeedTest` success
     clears it; a `TcpPing` success does not; `Cancelled` changes nothing;
   - `ops/ping.rs`: an untestable marker (`soft_failure`) never sets a verdict.

**Verification**: `cargo nextest run -p xray-tui`.

### T6 — Sweep skip, labels, and the dead config (tui + config)

**Files**: modify `crates/xray-tui/src/ui/profiles.rs`,
`crates/xray-tui/src/ui/mod.rs`, `crates/xray-tui/src/ops/ping.rs`,
`crates/xray-tui-config/src/app_config.rs`.

**Why**: D3 (a feed sweep must not spend phase 2 on purged links) and §9 (the
user must see WHY a link is in Purgatory).

**Change Necessity**: `code-change`. D3 is a plan-scoping decision inside the
batch loader, and the label is a render rule; the only config-shaped part of
this task is the *deletion* of a dead key, which is narrower than the code it
accompanies.

**Steps**

1. `ops/ping.rs::load_feed_plan`: `db.load_page_projection(&meta.ids, false)` —
   the feed sweep skips purged links. `plan_selected_endpoint` needs no change:
   it reads the loaded page, which keeps them outside the Active view.

2. `ui/profiles.rs` — the label helper beside `compute_test_cell`:

```rust
/// The short name a purged link shows in place of its Test cell content
/// (spec §9). A purge is a STATUS about the row, so it outranks the delay the
/// link still carries from phase 1 — exactly as the untestable marker does.
#[must_use]
const fn purge_label(reason: xray_tui_db::models::PurgeReason) -> &'static str {
    use xray_tui_db::models::PurgeReason;
    match reason {
        PurgeReason::RealityFallback => "mitm",
        PurgeReason::CertificateMismatch => "cert",
        PurgeReason::CertificateExpired => "expired",
        PurgeReason::NotTls => "notls",
        PurgeReason::ConfigInvalid => "cfg",
        PurgeReason::TransportRejected => "http",
        PurgeReason::OriginUnreachable => "origin",
    }
}
```

3. `compute_test_cell`: after the untestable-marker branch,
   `if let Some(reason) = row.active_link().and_then(|l| l.purge_reason) { return (format!("[{}]", center_cell(purge_label(reason), 4)), bad); }`.

4. `build_display_rows`'s panel row (the `delay` binding at ~line 732): a purged
   link renders `format!("[{}]", purge_label(reason))` instead of its delay.

5. `ui/profiles.rs`'s view label and `ui/mod.rs`'s `r` key guard: `Stale` →
   `Purgatory`, label `View: Purgatory [P]`.

6. `app_config.rs`: delete `PurgatoryConfig.enabled`, `default_purgatory_enabled`,
   and its `Default` line (D4; no reader, no settings-form row, and serde drops
   the key harmlessly).

**Verification**: `cargo nextest run -p xray-tui -p xray-tui-config`, plus a TUI
smoke check of the label and the `p` cycle (T8).

### T7 — Records and baseline sync (docs)

**Files**: modify `AGENTS.md`, `docs/database.md`,
`docs/database-manual-sql.md`, `CONTEXT.md`, `TUI_MANUAL.md`,
`docs/aegis/INDEX.md`; create `docs/aegis/adr/0006-purge-evidence.md`.

**Why**: the schema tag, the column, the view and three retirements are durable
architecture facts; the next reader must find them where they live.

**Steps**

1. `AGENTS.md`: decision 4's tag list gains `12 = profile_stats.purge_reason`;
   decision 16 gains tier 6 and the live-only `newest_seen`; decision 21's
   predicate note becomes the Purgatory range; the file-list entries for
   `endpoint_rank.rs`, `profiles_query.rs`, `models_toasty.rs` and
   `ops/ping.rs` mention the new input; add `ops/purge.rs` to the key-files list.

2. `docs/database.md`: the `profile_stats` mermaid entity gains
   `TEXT purge_reason "reality_fallback|…, NULL = live"`; the flow section gains
   the verdict's set/clear rules; the derived-state invariants note that
   `rank_newest_seen` is the live-only maximum.

3. `docs/database-manual-sql.md` §3: the link-writer row's description gains
   `purge_reason` in the RESULT set, and §6 records that the new column's
   statement is pinned by `purge_reason_rides_the_result_group_only`.

4. `CONTEXT.md`: the Purgatory glossary entry gains the second population; add a
   `PurgeReason` term.

5. `TUI_MANUAL.md`: the `[P]` view is `Purgatory`, and the panel's Test column
   shows the reason labels.

6. `docs/aegis/adr/0006-purge-evidence.md` (new, accepted): the decision, the
   alternatives rejected (text prefix; endpoint-level purge; a second predicate
   column; `EXISTS` over `profile_stats`), the compatibility boundary (tag 12
   wipe), and the retirements. Append both the spec and this ADR to
   `docs/aegis/INDEX.md` and set the spec's status to `implemented`.

**Verification**: `python <aegis-workspace-helper> check --root .` when the
helper is present; otherwise a manual read of the index rows.

### T8 — Live verification (no source edits)

**Files**: none.

**Why**: the policy's value is measured, not argued — spec §11's acceptance on
real data and spec §7's projection.

**Steps**

1. Launch the new binary once against the (about to be wiped) database and let
   it recreate the schema; confirm `PRAGMA user_version` reads 12 and
   `SELECT sql FROM sqlite_master WHERE name = 'profile_stats'` carries the
   `purge_reason` CHECK list.
2. Re-import the subscriptions; run Fast + Real over the feed (the same corpus
   as the 2026-09-17 run).
3. Read the comparison:

```sql
-- the projection: how much the active view sheds
SELECT COUNT(*) FROM profile_stats WHERE purge_reason IS NOT NULL;
SELECT purge_reason, COUNT(*) FROM profile_stats GROUP BY 1 ORDER BY 2 DESC;
SELECT COUNT(*) FROM endpoint_rank WHERE rank_tier = 6;
```

   Compare against **spec §7's measured snapshot**, not against a restated
   projection: the spec holds the rule and one measured `SELECT` (739 links over
   362 endpoints on the validated 2026-09-17 snapshot, with
   `transport_rejected` 408, `reality_fallback` 163, `origin_unreachable` 91,
   `config_invalid` 27, `not_tls` 24, `certificate_mismatch` 23,
   `certificate_expired` 3). Every number moves with every probe run — a later
   batch overwrites an earlier run's markers — so what matters is that the
   reasons are populated and that the classes match the code, not that a count
   reproduces.

4. Verify the acceptance list by hand: the Active tab shows no purged-only
   endpoint; the Purgatory view lists them with reason labels; a real test on
   one of them that succeeds removes the label and returns the row to Active.
5. Record the numbers in the spec §7 table (replace the projection with the
   measurement) and note any taxonomy change the data argues for.

**Verification**: the SQL reads above plus the on-screen checks; the numbers go
into the spec, and any mismatch with the acceptance list is a defect, not a
documentation edit.

## Risks

| Risk | Handling |
| --- | --- |
| The in-flight batch-ping plan rewrites `PageRequest`/`ProbeOutcome` under this one | Sequencing §1-2: land it first or rebase; never run both concurrently |
| The tag-12 wipe deletes the corpus under study | Backed up in T1 step 1; confirmed by the user |
| A `403` is CF bot-managing our fingerprint rather than a dead config | The reversible path (T6 step 1 keeps purged links testable) is the mitigation; the excluded statuses (`429`/`412`/`5xx`) cover the transient classes |
| The `profile_stats` literal sweep is wide (28 sites) | Compiler-driven and mechanical; no behavior in it |
| A purged link hiding from the Active panel could confuse ("where did it go") | The Purgatory label (`[http]`, `[mitm]`, …) and the `p` cycle are the answer; TUI_MANUAL.md documents it |
| The 2026-09-17 anomaly (3 real-ok of 6,835 attempts) means the evidence set is skewed toward "proxy never answered" | The taxonomy's evidence classes are all *server answers*, which is precisely why they are trustworthy even when the corpus is dead |

## Retirement

| Retired | By | Trigger/evidence |
| --- | --- | --- |
| `AppConfig.purgatory.enabled` + `default_purgatory_enabled` | T6 | No reader in `crates/` (never read since it was added); no settings-form row |
| `PurgatoryView::Stale` (name) | T3 | The view's meaning changed: it now holds two populations |
| `PageRequest.stale_threshold` | T3 | With a live-only `newest_seen` the lower bound hides a state this feature creates deliberately |
| `ProfilesLoad.purgatory_retention_secs` | T3 | The retention bound is not a page concern; the sweep keeps `AppState.purgatory_retention_secs` |
| The doubled `TLS error: TLS error:` prefix | T4 | Accidental duplication in `map_tls_err`; no test or doc pins it |

## Execution Readiness View

```text
Execution Readiness View:
- Intent Lock: purged links leave the Active view permanently and reversibly
- Scope Fence: the 15 files in the Files table; no manual purge/restore action,
  no settings toggle, no purge_expired change, no batch-summary counter (deferred)
- Baseline Lock: spec 2026-09-17-purge-reason-design (approved) + AGENTS.md 4/16/21/22
- Approved Behavior: spec §11's eight acceptance criteria
- Owner / Contract Constraints: purge_reason rides LinkGroups::RESULT; the law
  stays in RankLink::key; SQL never re-derives it; one classifier module
- Compatibility Boundary: schema tag 11 → 12 = WIPE (confirmed); one dead config
  key removed; one cosmetic TLS message de-duplicated
- Retirement Boundary: the five rows above, all inside this plan
- Task Batches: T1 → T2 → T3 (db/page), T4 (native/tls, parallelizable),
  T5 → T6 (tui), T7 (docs), T8 (live run)
- Test Obligations: per-task regression list; the drift test and the ordering
  oracle must stay green
- Review Gates: after T1 (schema + writers), after T3 (view semantics), after T6
  (user-visible labels) — each on the diff, before the next batch
- Drift / Rewind Rules: if the in-flight batch-ping plan lands first, rebase
  T3/T5/T6 instead of reverting either; a schema change beyond this one column
  returns to the spec
- Evidence Required Before Completion: the four SQL reads of T8 with the
  acceptance list checked by hand; a green `just quality-gate code`
- Advisory Boundary: method-pack execution guidance only; not GateDecision,
  PolicySnapshot, or completion authority
```

## Execution Route

```text
Execution Route:
- Decision: inline
- Evidence: the tasks share files (models_toasty.rs, database.rs, ops/ping.rs)
  and a half-applied change does not compile — db enum/field, native error
  variants, and the TUI call sites are a single compile unit; T4 is the only
  cleanly separable slice
- Fallback: if a subagent is available, T4 alone may be delegated (xray-tui-tls +
  xray-tui-native, no shared files with T1-T3); everything else stays inline
- User confirmation required: no
```
