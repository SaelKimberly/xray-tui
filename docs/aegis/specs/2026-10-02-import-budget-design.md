# Import budget — Design Spec Brief

Date: 2026-10-02
Status: **draft — awaiting user decision on §6 (the defaults)**
Origin: `plans/2026-10-02-write-contention-and-dns-timing-fixes.md` task T11

---

## 1. Why this exists

The import path carries two hard-coded, unconfigurable budgets in
`ops/stream_import.rs::run_streaming_import`:

```rust
const MAX_FEED_BYTES: usize = 64 * 1024 * 1024;   // 64 MiB, decoded
const MAX_FEED_LINKS: usize = 200_000;
```

Neither is in `AppConfig`, neither has a settings row, and neither is documented anywhere except
the source. On 2026-10-01 a **90,688-profile** subscription hit the byte budget and was cut off:

```
Feed exceeded the 67108864-byte budget (67112960 bytes) — stopping with partial results
Subscription update failed: … the feed is incomplete, stored rows were kept: feed over the budget
```

The stored rows were kept (correct) and the group row was written `GroupStatus::Error` (correct),
but the Settings group list renders only the literal word `error` — the reason lives solely in the
activity log. A user with a large-but-legitimate feed has no way to raise the ceiling.

**No document in `docs/aegis/` owns import limits.** This brief creates the first owner.

## 2. Scope

Make the two budgets configurable, and make a truncated import *legible* in the UI. Nothing else.

Explicitly **out of scope**: streaming-vs-buffered import, chunk sizing (`PERSIST_CHUNK`), the
retry ladder, and any change to what happens to already-stored rows.

## 3. The shape

A new `ImportConfig` section on `AppConfig`, both fields `#[serde(default)]`:

| field | type | meaning |
| --- | --- | --- |
| `max_feed_bytes` | `u64` | decoded-feed ceiling; `0` disables the byte budget |
| `max_feed_links` | `usize` | stored-link ceiling; `0` disables the link budget |

`0` disables, because "unbounded" is a legitimate choice for a self-hosted feed and encoding it as
a magic `u64::MAX` makes the config file unreadable.

`run_streaming_import` takes the two values as parameters. **No behaviour change at the current
defaults** — see §6.

## 4. Compatibility boundary

- **No `SCHEMA_VERSION` bump.** Nothing here touches the database.
- **No rename.** These are new keys. `AGENTS.md` records that `ip_api_url` → `ip_provider` was a
  *breaking* change because `AppConfig::load` propagates a serde error and `main.rs` uses `?`; a new
  key with `#[serde(default)]` cannot fail that way, and there is no `deny_unknown_fields` anywhere.
- An existing `config.json` loads with the defaults, unchanged.

## 5. What the UI must show

`partial_import_message` already produces a good sentence. The gap is that the group list renders
only `error`. **Change:** the group list row renders the group's `error_message` (truncated) beneath
the status word, so the reason is visible where the red cell is.

The count semantics must stay honest and are a stated requirement of the message: `links` counts
what was **stored**, not what the feed held. On a budget breach the message already says
"after N link(s) stored — the feed is incomplete".

## 6. Open — the defaults (user decision)

Two questions, and I am not answering them unilaterally because both change what a user
experiences:

1. **Byte ceiling.** Keep **64 MiB** (no behaviour change, the status quo), or raise it?
   The 2026-10-01 feed that overflowed decoded to just over 64 MiB, so 64 MiB is *marginal* for a
   ~90k-profile subscription — 128 MiB would have covered it. A self-hosted feed can be far larger.
2. **Link ceiling.** Keep **200,000**, or raise it? 90,688 links fit, so this did not bind. It is a
   runaway guard, not a real limit today.

My recommendation, offered as a recommendation and not applied: **128 MiB / 200,000**. It keeps the
runaway guard, removes the marginal failure the run actually hit, and 128 MiB is still small enough
that the decoded buffer cannot exhaust memory on a normal machine. Both fields are editable at
runtime through Settings → Subscriptions, so the ceiling is not a one-shot decision.

## 7. Verification

- A config round-trip test: a `config.json` with no `import` section loads with the defaults.
- An override test: `max_feed_bytes = 1024` cuts a feed off early and the outcome carries the
  budget reason through `ended_early` → `GroupStatus::Error` — i.e. it exercises the **existing**
  path (T5) rather than adding one.
- `0` disables: a feed over 0 bytes' worth still imports.
- `just quality-gate` green.

## 8. Retirement

No path is retired. The two function-local `const`s are removed and the values become parameters;
nothing else changes shape.
