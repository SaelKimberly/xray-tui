# 0009 — Unhonourable TLS fingerprints are approximated and marked, never refused

Date: 2026-09-22
Status: accepted
Related: decisions 11 (identity), 13 (enrichment), 16 (tiers + labels), 20 (native core chosen at
connect time); ADR 0006 (purge evidence and verdicts)
Spec: `docs/aegis/specs/2026-09-22-native-testability-improvement-design.md` (§5.1, §5.3, §7 class 1)
Evidence: `crates/xray-tui-native/src/security/fingerprint.rs` (`resolve_fingerprint`),
`crates/xray-tui-native/src/security/mod.rs` (the dial), `crates/xray-tui/src/ui/profiles.rs` (the
marker), `crates/xray-tui/src/ops/purge.rs` (`reason_for`), `crates/xray-tui/src/state.rs` (the
projection guard)

## Context

A subscription's `fp` parameter names a uTLS preset. The engine models a subset of them, so an id
like `qq`, `android`, `360` or `hellochrome_120` had no hello, and neither did the empty string —
although an *absent* `fp` was supported and `security.fp()` returns `Some("")` for a
present-but-empty field.

The old capability gate refused all of them, which deferred the link to the subprocess. Measured on
a two-subscription feed: **1,165 links** carried that refusal (`qq` 875, `""` 241, `android` 46,
`hellochrome_120` 2, `unsafe` 1). A refusal is a *testability* loss, not a safety property: the
engine can dial all of them with a different hello, and it is the only core that serves some of the
transports those links carry.

Two further facts shaped the decision. REALITY fallback was the largest real-failure cluster (710
rows, 38 % of real failures), so a substitution on that path had to be *visible* rather than silent.
And `purge_reason` is a `TEXT CHECK` over 7 variants, so any new persisted label is a schema-tag
bump — and a tag bump makes `Database::open` delete the file.

## Decision

**One predicate decides, and it returns the identity plus whether the request was honoured.**
`security::fingerprint::resolve_fingerprint(Option<&str>) -> (Option<FingerprintId>, bool)`:

| `fp` | reading | approximated |
|---|---|---|
| absent / `""` / `unsafe` | no fingerprint requested — the engine default IS the shape asked for | no |
| `chrome` / `chrome-randomly` / `randomized` / `firefox` / `safari` / `edge` / `ios` / `random` | resolves to a roster row | no |
| `qq` / `android` / `360` / `hellochrome_120` / any future id | requested, no roster row | **yes** |

**An unrosterable id is probed with the engine default and MARKED; it is never refused and never
silently substituted.** The capability gate's fingerprint refusal is **retired** — the function was
deleted, not left returning `None`, so no caller can re-introduce a divergence between the gate and
the dial.

**The marker is derived, not stored.** approximated ⟺ the `security_fp` column is present, is not
`""`, is not `unsafe`, and does not resolve to a roster row. It renders as a trailing `~` on the
Test cell and on the panel's per-link shape cell, scoped to the **representative link's own
protocol** — a `Protocol` row is shared across endpoints, so an endpoint-level predicate would mark
a verdict that came from a roster-mapped sibling.

**An approximated probe earns no purge verdict.** `reason_for(evidence, fp)` returns `None` when
that link's fingerprint is an approximation: the shape dialled is not the shape the link asked for,
so the failure is evidence about a config that was never tried. A protocol-cache miss also returns
`None` — it fails **closed**, because failing open would let an approximated probe earn exactly the
permanent Purgatory verdict the rule exists to prevent, and Purgatory is the one output that is not
cheaply reversible.

## Consequences

**No persistence, no schema change.** The label is a pure function of a column that is already in
the page projection (`profiles_query.rs`), so no new column, no projection change, no tag bump and
no wipe. This is why the marker could be added at all: every alternative that persisted it required
deleting the database.

**One predicate, TWO inputs — and the agreement is asserted.** The predicate is shared, but its
input differs by path: `security::wrap` (the dial, where the substitution happens) resolves
`opts.fp` off the loaded **config**; the row label and `reason_for` read the **`security_fp`
column**. They agree only because `state::security_embed` projects the column from that same
`tls.fp`, and `the_security_fp_column_agrees_with_the_config_field` is the guard on that projection.
The purge path reads the column on every path (the single-ping site has its own `Protocol` row; the
two batch sites read `LoadedProtocol.fp`, captured at that struct's one construction site) — but
"one predicate" was never the same claim as "one input", and treating them as the same is what made
an earlier draft of the spec wrong.

**Blast radius, measured.** 1,165 links stopped carrying a capability refusal: **924 approximated,
257 honoured, 0 refusals remaining.**

**Alternatives rejected.**
- *Keep refusing* — costs testable links for a config the engine can dial, and the refusal is not a
  safety property.
- *Persist the marker* (a column, an enum, a purge variant) — each is a schema-tag bump, and a bump
  deletes the database. A label is not worth a data reset.
- *Thread "approximated" from the engine up through `ProbeFailure` → `FailureEvidence` →
  `reason_for`* — an engine-wide threading job on the surface T4 was deferred over, and it would
  apply only to failures probed *after* the change, exempting every row already persisted. Deriving
  it at purge time has neither property.

**Cost, stated.** An approximation can be wrong in both directions — it may pass a CDN check the
requested shape would fail, or be blocked where the requested shape would pass. That is contained,
not eliminated, by the no-verdict rule: a wrong answer stays re-testable instead of becoming a
verdict.

**Baseline-sync question this ADR creates, and does not answer:** *which capability gaps are
approximated vs refused, and who may change that list?* The predicate is the single decision point;
the roster is what it consults, so a new uTLS id appearing in the wild re-opens the same question.

**Retirement trigger.** The approximation count trends to zero as roster rows land — with the
qualification that `qq` is a *new capture*, not a roster lookup, so the gap has no end date and each
new uTLS id re-opens it. `""`/`unsafe` are not part of that count: they are honoured, not
approximated.

## Evidence

Class-1 static acceptance, re-measured on the live feed (`security_fp` over the `tls`+`reality`
population, no network): **924 approximated / 257 honoured / 0 refusals**, across two snapshots
(9,067 and 9,111 links) — the claim is invariant, the absolutes snapshot-bound. Unit coverage:
`resolve_marks_only_unrosterable_ids_as_approximated`,
`approximated_and_absent_resolve_to_the_same_identity`,
`approximation_marker_follows_the_representative_link`,
`approximation_marker_composes_with_the_verdict`,
`a_protocol_cache_miss_earns_no_verdict_and_a_cached_fp_decides_it`,
`an_approximated_probe_earns_no_verdict_at_all`. Visual confirmation on the real TUI (isolated
`XDG_CONFIG_HOME`, a copy of the database).
