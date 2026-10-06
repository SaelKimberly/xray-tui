# Checkpoint — DB rewamp

## Snapshot
- root `/home/user/oss/xray-tui`; branch `native-core-stub`; HEAD `8097cb9`.
- Tree **clean**; the workstream is the last 5 commits.

## Commits landed (each green: nextest + workspace check)
| commit | slice |
| --- | --- |
| `544d441` | S1 — spec rev.2, plan, baseline, harness guard, turso planner gate |
| `068bbb6` | drop write-only `transport_data`/`security_data` (−13.6% file) |
| `80111de` | drop `core_type` (per-pair + group) + the form override (D3) |
| `c41e63f` | drop `ConfigType` from identity + schema (D9), `IDENTITY_VERSION` 2→3, golden re-pinned; `rank_config` removed |
| `1f89de5` | lock the Profiles order — delete the sort UI (D1) |
| `8097cb9` | `psl2` dep + `xray_tui_config::domain::split` (the DNS-split owner, D2/D6) |

## Todo map
- **Done:** T0 baseline · T1 harness guard · T2 turso gate · T3 psl2 helper
  (meta row still to wire) · T6 core_type · T7 ConfigType+JSON · T10 sort UI
- **Remaining:** T3-meta (PSL version row) · T4 identity/host model ·
  T5 validation + counted skip · T8 binned law + one index · T9 page order/search ·
  T11 tag-15 wipe · T12 FK cascade · T13 direct reader · T14 gated WITHOUT ROWID ·
  T15 docs/ADR/AGENTS

## Verification at this HEAD
- `cargo check --workspace --all-targets` → 0 errors.
- `cargo nextest`: proto/config/core **732**, db **170**, tui **261** — all green.

## Blockers
- none technical. **Budget**: the remaining slices (psl2 identity, binned law,
  page/search, sort UI, wipe, FK, reader, docs) are each large; the session did
  not have room to finish them.

## Next step (exact resume point)
1. **T4 — endpoint identity/host model.** Drop `endpoints.host`/`host_type`; add
   `domain`/`sub_domain`; identity = `stable_hash(ascii_name, port)` using
   `xray_tui_config::domain::split(host).ascii` for dns, the literal for ip,
   `("undefined", config_uid)` for exotic; write an IP host's literal as its
   `endpoint_ip` row; derive host-kind (`domain` non-empty → dns; address row →
   ip; else undefined).
   **Consumer surface (must all move in ONE commit — it cannot compile until
   done):**
   - `xray-tui-db`: `models_toasty::Endpoint`, `database.rs`
     (`upsert_endpoints_bulk` SQL + seeds), `profiles_query.rs`
     (`PAGE_PROJECTION` `e.host`/`e.host_type`, search predicate),
     `export.rs` (SELECT + decode), `endpoint_rank.rs`
     (`dns_unresolved_endpoint`, `weight_from_discriminators`, `rank_host`).
   - `xray-tui-core`: `config_builder/mod.rs::endpoint_essentials(&Endpoint)`
     — **needs the addresses** (an IP host's dial host now lives only in
     `endpoint_ip`), so its signature must take them/`EndpointRow`.
   - `xray-tui`: `state.rs::endpoint_from_essentials`, `ops/native_connect.rs`
     (its own `endpoint_essentials` copy + `:85`), `ops/ping_native.rs:224`,
     `ui/mod.rs:902`, `ui/profiles.rs` (Address column), `ops/export.rs:136`.
   - validation: `import_export::validate_host` DNS branch calls `domain::split`
     and STOPS normalizing via `url::Host` (one normalizer).
   - The **parse boundary** `xray-tui-proto::EndpointEssentials.host/host_type`
     STAYS (the split is the DB layer's job).
   Bump `SCHEMA_VERSION` to 16 again (the model changes).
2. **T5** validation counted-skip (form == import) · **T8/T9** binned law + one
   index + page order/search (needs T4's `domain`/`sub_domain`/`addr`) ·
   **T12** FK cascade · **T13** direct reader · **T14** gated WR ·
   **T15** docs/ADR/AGENTS.

## Verification at this HEAD
- `cargo check --workspace --all-targets` → 0 errors (2 pre-existing native doc warnings).
- nextest: proto/config/core **732**, db **170**, tui **259** — all green.
- Clippy: MY files clean; the workspace gate is red from PRE-EXISTING lints on
  this WIP branch (native/context.rs, tls/spec, route/compiler, proto mod.rs
  len_zero) — untouched by this work.
