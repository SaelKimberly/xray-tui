# 0006 — Purge evidence is persisted per link; Purgatory is "not confirmed live and recent"

Date: `2026-09-17`
Status: `accepted`
Supersedes: nothing
Related: `specs/2026-09-17-purge-reason-design.md`,
`plans/2026-09-17-purge-reason.md`, ADR 0002 (column groups), ADR 0003 (stored
ordering keys), decision 4 (the schema tag), decision 16 (the ordering law),
decision 20 (the native real probe).

## Context

A real probe can return evidence that a config can **never** work again: a
REALITY endpoint answering with a real certificate, a certificate the
configured SNI cannot match, a port answering cleartext, a proxy whose own
upgrade handshake answers `404`, a CDN answering `530` because the origin is
gone. Before this decision that evidence was discarded into
`profile_stats.error`, a marker the `error_ttl_hours` sweep deletes, so a
permanently dead config sat in the Active view forever, competing for attention
with configs that work. On the reference corpus 1,687 links of 56,140 carried
such evidence at the time of the design (739 links over 362 endpoints on the
snapshot taken mid-re-run — the number moves with every probe run).

## Decision

1. **The unit of purge is the LINK** (`protocol_id`, `endpoint_id`), not the
   endpoint: an endpoint with several protocols keeps its working ones.
2. **A new `profile_stats.purge_reason` column** holds the verdict —
   `PurgeReason`, seven fieldless variants, one nullable `TEXT` column with a
   `CHECK` list (schema tag 11 → 12, a WIPE under decision 4).
3. **The verdict is written by its own column group** (`LinkGroups::PURGE`,
   `0b010`). Not riding RESULT: `link_patch_conflict_sql` writes a FIXED column
   set from each patch's *snapshot*, so riding RESULT would let a phase-1 fast
   half — whose snapshot is the plan-time row — rewrite a verdict it never
   classified. That is exactly the clobber shape ADR 0002's groups exist to
   prevent, and the classifier becomes structurally the only producer.
4. **The evidence is typed where it failed, never parsed from the message.**
   `TlsError` splits name-mismatch/expiry/cleartext; `NativeError` gains those
   three plus `TransportRejected { detail, status }` at the five sites that
   render the peer's own HTTP status; `NativeError::evidence()` is one
   exhaustive match. The probe's TARGET leg (the probe URL's own TLS) reports
   plain `Tls` and carries no evidence — a target certificate says nothing
   about the config, and `target_leg_tls_failure_carries_no_evidence` pins it.
5. **The taxonomy is one function in the TUI crate** (`ops::purge::reason_for`):
   521/522/526/530 → `OriginUnreachable`; 301/302/400/403/404/405/409/410 →
   `TransportRejected`; the three TLS causes, `ConfigDefect` and REALITY
   fallback map one-to-one. Deliberately NOT purging: every timeout, dial/DNS
   failure, TLS alert, framing failure, `412`/`429`/`500`/`502`/`503`/`520`/`525`,
   and `not testable by the native engine` (a capability gap, not a verdict —
   the subprocess core may serve it).
6. **The ordering law gains one band**: `RankLink.purged` → tier 6, below every
   live band, so a purged link never represents the endpoint while a live link
   exists (`display_link_index`'s gate covers the `manual_protocol_override`
   branch, which the display rank alone could not); `compute_rank`'s
   `newest_seen` becomes the **live-only** maximum.
7. **Purgatory is one range**: `rank_newest_seen < now - ttl_days`, which
   covers the aged band AND endpoints with no live link (they report
   `NO_SEEN`). No `OR`, no second predicate column, no new index.
8. **A verdict is cleared by a data-carrying success only** — a real ping or a
   speed test. A TCP/UDP handshake cannot clear it, a feed-wide sweep never
   plans a purged link (phase 2 is the long pole at 2.6 results/s), and the
   selected-endpoint entry points keep them so a Purgatory row stays testable
   by hand. The `error_ttl_hours` sweep never touches it; `purge_expired` still
   deletes the profile once its newest link of any kind ages past
   `purgatory.retention_days`.

## Alternatives rejected

| Alternative | Why not |
| --- | --- |
| Encode the reason as an `error_text` prefix (the `UNTESTABLE_PREFIX` pattern) | The `error_ttl_hours` sweep deletes `error_text`, so a "permanent" verdict would vanish; the page predicate and the rank law would have to parse a string, which `docs/database-manual-sql.md` §1 forbids ("never a second owner of a fact") |
| Purge the whole endpoint on any verdict | Destroys the case the feature exists for: one endpoint, several protocols, only one real |
| Ride `LinkGroups::RESULT` (one less group bit) | See decision 3 — a silent clobber, currently only *incidentally* safe because plan/page snapshots happen to be fresh |
| A second predicate column (`rank_live`) plus an `OR` | Would defeat the window index and duplicate a fact the live-only `newest_seen` already carries |
| An `EXISTS (SELECT 1 FROM profile_stats WHERE purge_reason IS NOT NULL)` correlation in the page predicate | The correlated-subquery shape ADR 0001/0003 removed: the page would stop being a single index scan |
| Purge from the fast phase's hard failures (timeout/refused/no-route) | A transient network condition, not a statement by the server; 31,020 links carried one on the reference run |
| Purge `not testable by the native engine` | It is a statement about THIS engine, not about the config — the subprocess core can serve those rows |
| A migration for the new column (additive `ALTER TABLE`) | The project has no migration machinery and the database is re-importable fixture data (decision 4); an additive alter would also weaken what the tag means |

## Consequences

- **A schema bump wipes the database.** Accepted: the verdicts are re-derived
  by the next Real run, and the reference classification was extracted to
  `/tmp/xray-tui-tag11-backup/purge-classification.csv` before the bump.
- The Active view sheds dead configs automatically, and the tab's Test column
  says *why* for the rows that remain visible in Purgatory.
- A wrong verdict costs one manual real test to undo; a too-timid taxonomy
  costs one function's edit (`reason_for`) with no schema change.
- The three retirements this implies: `PurgatoryView::Stale`,
  `PageRequest.stale_threshold`, and the never-read `PurgatoryConfig.enabled`.
- ADR 0002's group contract is now three groups wide, and
  `apply_link_patches_isolates_column_groups` pins the new one.
