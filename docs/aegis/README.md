# Aegis Workspace

Design and decision records for `xray-tui`. This directory holds the
project's durable Aegis artifacts; it is not user documentation.

## Structure

| Path | Purpose |
| --- | --- |
| `INDEX.md` | Registry of specs, plans, and ADRs |
| `BASELINE-GOVERNANCE.md` | Project constitution: baseline roles, defect/drift rules, review dimensions |
| `baseline/` | Point-in-time baseline snapshots (evidence, not authority) |
| `specs/` | Design specs and spec briefs |
| `plans/` | Implementation plans derived from approved specs |
| `adr/` | Accepted architecture decisions |
| `work/` | Session-level drafts, not project documents |

Promotion rule: a draft stays in `work/` until the Doc Necessity Gate passes
(durable, irreversible, or approval-gated surface), then it is promoted to
`specs/`, `plans/`, or `adr/`.

This workspace was initialized on 2026-09-11 alongside the first design spec.
Workspace support scripts are not installed on this host, so index entries are
maintained by hand.
