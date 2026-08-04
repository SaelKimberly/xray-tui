# ProtoIdentity: Trait Extraction + Identity Container

**Date:** 2026-08-03
**Branch:** feature/proto-identity
**Status:** Approved

## Problem

Every protocol config type in `crates/xray-tui-proto/src/proto_spec/` carries two
cache fields (`sig_cache: OnceLock<NonZeroU64>`, `cred_hash_cache: OnceLock<u64>`),
a private inherent `compute_sig()`, and an `impl ProtoSpec` that repeats the
same caching/serialization rules via the `impl_sig_cache!` macro. The rules
(sig never zero, OneShot caching, `uid = sig ^ cred_hash`) are enforced by a
macro, not by the language, and the cache fields are duplicated across 21
structs.

## Design

### 1. `ProtoIdentity` trait (crate-private, sealed)

Plain-private in `proto_spec/mod.rs` — private items are visible to descendant
modules, so every protocol file reaches it via `super::`; the crate root never
re-exports it. Exactly two required functions, both pure computation:

```rust
trait ProtoIdentity {
    fn compute_sig(&self) -> u64;
    fn compute_cred_hash(&self) -> u64;
}
```

No caching, no state. Each protocol's `compute_sig` moves verbatim from its
inherent impl; `compute_cred_hash` is the body of today's `cred_hash()` (the
`utils::compute_cred_hash(&[("uuid", ..), ..])` calls). `PlaceholderConfig`
implements it with the whole-body rapidhash and `0` respectively.

### 2. `Identity` struct + `Proto` container

```rust
struct Identity {
    sig: NonZeroU64,
    cred_hash: u64,
}

pub struct Proto {
    config: ProtocolConfig,
    identity: OnceLock<Identity>,   // empty = deferred; first access materializes
}
```

Derives: `#[derive(Debug, Clone, Copy)]` on `Identity`; `#[derive(Debug)]` on
`Proto` (`OnceLock<T>` is `Debug` when `T: Debug`). Do **not** derive `Clone`
on `Proto` — `OnceLock` isn't `Clone` and no clone sites exist; none needed.

- `Proto` is the new primary public type. `sig()`, `cred_hash()`, `uid()` are
  inherent members. First call materializes atomically:
  `self.identity.get_or_init(|| Identity { sig: NonZeroU64::new(self.config.compute_sig()).unwrap_or(NonZeroU64::MIN), cred_hash: self.config.compute_cred_hash() })`
  — race-safe by construction, never blocks, no partial states. The empty
  `OnceLock` IS the deferred state (an explicit `Defer` enum variant was
  considered and rejected: `OnceLock::set` fails on any initialized cell, so a
  materialized Defer could never transition; the empty cell encodes the same
  state with no dead variant).
- `set_identity` exists only as a `#[cfg(test)]` private method (zero production
  callers; seeded `Identity` lets tests assert no recompute).
- Accessors: `new(config)`, `config(&self)`, `into_config(self)`.

### 3. `ProtoSpec` trait (sealed)

```rust
#[allow(private_bounds)] // edition 2024 denies private bounds; deliberate seal
pub trait ProtoSpec: ProtoIdentity { /* behavior methods only */ }
```

- Behavior methods unchanged: `try_parse`, `reconstruct`, `schema`, `host`,
  `port`, `remarks`, `security`, `transport_type`, `security_type`,
  `country_flags`, `to_json_config`, `to_clash`, `try_from_clash`.
- `sig()`/`cred_hash()`/`uid()`/`set_sig_cache()`/`set_cred_hash_cache()` are
  **removed** from `ProtoSpec` — identity is `Proto`'s job now (clean cutover,
  every caller migrates).
- Protocol configs keep `impl ProtoSpec` (behavior) and gain `impl ProtoIdentity`
  (2 fns). `ProtocolConfig` gets `impl ProtoIdentity` via the existing
  `dispatch!` macro. `Proto` gets `impl ProtoIdentity` (delegates `compute_*` to
  `self.config`) and `impl ProtoSpec` (behavior dispatches to `self.config`).
- `impl_sig_cache!` macro: deleted.

### 4. Serialization

Manual `Serialize`/`Deserialize` for `Proto`, delegating to `config`
(`#[serde(transparent)]` with a second `#[serde(skip)]` field is a
compile-error risk; manual impl is deterministic):

```rust
impl Serialize for Proto { /* self.config.serialize(s) */ }
impl<'de> Deserialize<'de> for Proto { /* Self::new(ProtocolConfig::deserialize(d)?) */ }
```

spec_blob bytes stay **identical** — no DB migration. Deserialized `Proto`
starts `Defer`.

### 5. Workspace ripple

`ProtocolConfig` → `Proto` at every construction/deserialization/identity site:

- `xray-tui-db`: `database.rs` (`serde_json::from_slice::<ProtocolConfig>` →
  `::<Proto>`; variant matches on config → match `proto.config()`)
- `xray-tui-config`: `import_export.rs` (`from_legacy_parse`, `try_parse`,
  `from_slice`, `typed.uid()/sig()/cred_hash()`)
- `xray-tui-core`: `lib.rs` (spec_blob parse + variant match),
  `config_builder/mod.rs` `parse_settings`
- `xray-tui`: TUI connect/profiles paths
- `ProtocolConfig` stays `pub` (cross-crate variant matching still needed);
  `Proto` is primary.

Compiler finds every site — no manual hunting.

## Rules normalized at language level

- sig never zero: `NonZeroU64::new(compute_sig()).unwrap_or(NonZeroU64::MIN)`
  — single site in `Proto::materialize`
- `uid == sig ^ cred_hash` — single site in `Proto::uid`
- One-shot caching — `OnceLock<Identity>` in one struct, not 21 copies

## Testing

- Existing proto suite (164 tests) stays green, adapted — removing
  `sig()/cred_hash()/uid()` from `ProtoSpec` breaks every call site in the
  proto crate itself, not just external crates:
  - `test_helpers::check_roundtrip` drops its `parsed.sig()` warmup line
    (configs no longer expose `sig()`)
  - per-protocol sig/uid determinism assertions (T6b tests across all 21
    protocol files) migrate to `Proto::new(parsed).sig()` — "behavior
    identical" means workspace tests migrate to
    `Proto::new(config).sig()/.cred_hash()/.uid()`
  - any doc tests calling `.sig()` on configs updated the same way
- New tests:
  - `Proto` JSON roundtrip byte-identical to `ProtocolConfig`; identity Defer
    after deserialize
  - materialization: `sig()`/`cred_hash()`/`uid()` consistent, `uid == sig ^
    cred_hash`, sig never zero
  - `#[cfg(test)] set_identity(Cache)` → `sig()` reads seeded value
  - placeholder body-hash determinism (existing test, adapted to `Proto`)
- Full workspace suite green; clippy clean on touched lines.

## Out of scope

- No change to `compute_sig`/`compute_cred_hash` semantics per protocol
- No DB migration (bytes identical)
- `L4` subscription_url_split behavior (unchanged)
