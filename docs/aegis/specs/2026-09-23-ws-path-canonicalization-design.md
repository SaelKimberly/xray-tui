# WS path canonicalization at parse: dialable configs, one uid per logical path

- **Date**: 2026-09-23
- **Kind**: design spec (new owner: the transport-path canonicalizer) + one specified amendment
- **Status**: proposed — awaiting user review
- **Depends on**: `2026-09-17-purge-reason-design.md` (§7 classification — **amended by this spec**, §4.4), `2026-09-22-native-testability-improvement-design.md` (§5.9 HTTP-transport divergence — its default-header A/B is a recorded negative and is **not** retried here), `docs/database.md` (identity change checklist), AGENTS.md decision 4 (schema tag) and decision 11 (identity wire format)

## 1. Outcome

A ws-family config whose stored path is non-canonical becomes **dialable**, and two spellings of the same logical path become **one `Protocol` row** (one uid).

Both are achieved by canonicalizing the path at the parse boundary, so the request builder never receives a path it cannot frame and the identity function sees one value per logical path.

## 2. Why this exists

Measured from the persisted feed (`profile_stats` ⨝ `protocols`), 2026-09-23 snapshot: **30 rows** — 26 `trojan`, 4 `vless` — fail with

```
invalid or unsupported config: ws request: HTTP format error
```

26 of the 30 are **one server** (`Koma-YT.PAGeS.Dev`), carrying **three distinct logical paths in four stored spellings** — which is exactly why canonicalizing must merge some and must not merge others:

| `transport_data.path` as stored | after one more decode | rows | merges with |
| --- | --- | --- | --- |
| `/trTelegram @WangCai2` | `/trTelegram @WangCai2` | 17 | — (no emoji: a different channel name) |
| `/trTelegram🇨🇳 @WangCai2` | same | 5 | **↔ row 4** |
| `%2FtrTelegram%F0%9F%87%A8%F0%9F%87%B3%2B%40WangCai2` | `/trTelegram🇨🇳+@WangCai2` | 2 | — (`+`, not a space) |
| `%2FtrTelegram%F0%9F%87%A8%F0%9F%87%B3%20%40WangCai2` | `/trTelegram🇨🇳 @WangCai2` | 2 | **↔ row 2** |

The merge is rows 2 ↔ 4 (the single-encoded source vs the double-encoded twin of the *same* path); rows 1 and 3 are genuinely different paths and must stay distinct uids. A canonicalizer that merged all four would be wrong in the opposite direction from today's behaviour.

The two facts that shape the design:

1. **`RawUrlX::query()` percent-decodes key and value exactly once** (`crates/xray-tui-proto/src/urlx/split_url.rs:191-195`). A double-encoded source therefore survives one decode as a still-escaped string; a single-encoded source arrives fully decoded (raw space, raw emoji).
2. **The ws path is identity-bearing.** `write_transport` (`crates/xray-tui-proto/src/proto_spec/common.rs:898-901`) writes it: `w.opt_str(TR_PATH, cfg.path.as_deref(), "/")`, elided at its `/` default. A changed stored path therefore changes `sig` → the uid.

The reference tolerates both spellings: `xray`'s ws dialer concatenates `protocol + "://" + host + GetNormalizedPath()` (`thirdparty/Xray-core/transport/internet/websocket/dialer.go:140`) and hands it to Go's lenient `url.Parse`, which accepts a raw space and re-encodes at serialization; `GetNormalizedPath` (`websocket/config.go:11`) prepends `/` when absent. We feed the identical string into the strict `http`-crate `into_client_request`, which rejects it.

## 3. Scope

| # | item | lands in |
| --- | --- | --- |
| 1 | transport-path canonicalizer at the parse boundary (`encode(decode(stored))` + `/`-prepend), covering the transport path **and** the vless/vmess/trojan mirror (gated on a URI-path transport) | `crates/xray-tui-proto/src/proto_spec/` |
| 2 | purge-reason §7 class split — path-encoding rows stop being `Config`; invalid-authority rows keep it | amendment to `2026-09-17-purge-reason-design.md` |
| 3 | e2e: real xray server configured with a path needing canonicalization | new `crates/xray-tui-native/tests/ws_paths.rs` — self-contained (hand-written server config; client config parsed from a share URL), **no `CaseSpec` knob** (which would have bypassed the fix — §7.5) |
| 4 | early data: `?ed=NNNN` hoisted into `max_early_data`, the xray carrier re-derived when the path has none, `SCHEMA_VERSION` 12 → 13 | §4.6; `crates/xray-tui-proto/src/proto_spec/common.rs`, `crates/xray-tui-db/src/database.rs` |

### Non-goals (deliberate, with reasons)

- **No `ws.rs` (or any transport-builder) change.** The builder keeps framing whatever it is given; the invariant is established one layer earlier. A build-site fix would leave the two spellings two rows and forfeit the dedup.
- **No table or column change.** §4.6's `SCHEMA_VERSION` bump (12 → 13) is a **wipe** under decision 4, not a schema delta — nothing about the tables changes; the bump exists only to drop the data rather than carry the `ed` re-key's duplicate set.
- **httpupgrade's `ed` — same mechanism, deliberately not in scope.** `HttpUpgradeConfig.ed` exists and is in the identity (`TR_ED`), but no builder emits it and nothing hoists a path query into it; the feed carries no httpupgrade row with `ed`. The mechanism is identical to §4.6 and would take the same two edits, so it is recorded as a known sibling rather than silently passed over.
- **No `IDENTITY_VERSION` bump** — the wire *format* is unchanged (§4.3).
- **grpc's `path` is out.** It is a service name, not a URI path (`common.rs:1308-1311`: "share links carry the service name in `path`"); percent-encoding it would corrupt it. The canonicalizer scopes to the URI-path transports: ws, http, httpupgrade, xhttp.
- **The §5.9 default-header A/B is not retried.** It was measured negative and reverted (2026-09-22 §8.1/T10); this spec adds no request headers.

## 4. Design

### 4.1 The canonicalizer — a request-target, not a path

```
// The stored value is a REQUEST-TARGET: path ["?" query]. Split BEFORE encoding.
// Everything from the first '#' is a fragment: not sent, dropped — which is what
// today's `into_client_request` already does (mirrored, not invented).
(path_part, query_part) = split_at_first(stored, '?')   // '#' handled the same way, dropped

logical = percent_decode(path_part)             // ONE extra decode; RawUrlX decoded once
logical = "/" + logical  if !starts_with('/')   // the GetNormalizedPath rule
wire    = percent_encode_path(logical)          // pchar set only
wire    = wire + "?" + query_part               // query preserved VERBATIM
```

- The **extra decode** is what makes the dedup work. It undoes exactly the double-encoding `RawUrlX` left behind, so the single-encoded source (row 2, `/trTelegram🇨🇳 @WangCai2`) and the double-encoded twin (row 4, `%2FtrTelegram%F0%9F…%20%40…`) land on the same logical path, then on the same `wire` string.
- `%2F` decodes to `/`, so the leading-slash rule falls out for free on the double-encoded spelling; the explicit rule covers the rest.
- `percent_encode_path` encodes, in the **path part only**, everything outside the pchar set (`unreserved` + `!$&'()*+,;=` + `:` `@` `/`) — space, non-ASCII, `"`, `<`, `>`, `#`, and a literal `%` (after the decode, a `%` can only be literal). `?` is **not** encoded: it was already consumed by the split, and a `?` inside the query part is legal there.
- **The query is preserved verbatim, and this is not cosmetic.** The parser stores the early-data form *inside* `path` (`from_type_and_path`: `path.map(TinyText::from)`), so the real, working value `/?ed=2560` is pinned by `vless.rs:932` and used across `common.rs`'s tests. Encoding its `?` would turn a dialing config into `/%3Fed=2560`, change the request-target, and re-key it — falsifying §4.3's re-key bound and §7.2's no-churn test. Splitting first is what keeps it byte-identical.
- **What `ed` is, for a reader who meets it later.** `?ed=NNNN` is the carrier for WebSocket **early data** (`maxEarlyData`): xray's config parser hoists it out of the ws path into `Config.Ed` and deletes it from the path (`thirdparty/Xray-core/infra/conf/transport_method.go:625-632`), and the client then base64s its first tunnel bytes into the `Sec-WebSocket-Protocol` header (`websocket/dialer.go:153`), which the server reads back (`websocket/hub.go:55-59`) — one RTT saved. We keep it **in the path** deliberately: a server that reads it from the query must still see it, and stripping it is a wire change with no correctness upside. Handling it properly (deriving the size for our own early-data support) is **out of scope here** — see §4.6.
- Already-canonical paths (e.g. `/ws`) are **byte-identical** through the function; this is the bound on the re-key (§4.3, §7).

**Invariant (the load-bearing rule):** the emitted request-target must percent-decode, server-side, to the configured path — `hub.go:47` compares the server's decoded `request.URL.Path` against its raw configured path.

**Pinning the input shape.** The measured spellings are the canonicalizer's unit case: rows 2 and 4 must both produce `/trTelegram%F0%9F%87%A8%F0%9F%87%B3%20@WangCai2` (`@` stays bare — it is an RFC 3986 pchar, and Go's `EscapedPath` leaves it too), which must decode back to `/trTelegram🇨🇳 @WangCai2`, and the two configs' uids must be **equal** (that last assertion is the dedup). Rows 1 (`/trTelegram @WangCai2`) and 3 (`/trTelegram🇨🇳+@WangCai2`) must stay **distinct** from both and from each other — the negative half, which a merge-happy canonicalizer would fail.

Encoder choice (as landed): a ~15-line local encoder over the `pchar` set — no new dependency. `urlencoding::encode` is unsuitable (it encodes `/`), and `percent-encoding` was not needed for one function.

**Coverage — the transport path AND its mirror.** vless/vmess/trojan carry the ws path **twice**: the transport's (which `write_transport` hashes into the identity) and a top-level `path` that `write_identity` deliberately excludes but `reconstruct_proto` **emits** (`trojan.rs:313`, `vless.rs:387`, `vmess.rs:385`). Canonicalizing only the transport leaves the stored config, its exported share URL and its uid describing different paths, so the mirror is canonicalized with the same rule — **gated on the transport being a URI-path kind**. For `Grpc` the mirror is the SERVICE NAME (a real link is `?type=grpc&path=svc`, no leading slash): canonicalizing it would prepend `/`, export `/svc`, and reimporting that hands the transport a different grpc path, which `write_transport` hashes into a different uid — a duplicate row. `TransportConfig::canonicalize_paths` **returns** whether it applied, and that bool is the gate, so "which transports have a URI path" is declared exactly once. Pinned by `grpc_service_name_is_not_a_path`, `canonicalize_transport_paths_leaves_a_non_uri_transport_mirror_alone` and `canonicalize_transport_paths_covers_the_top_level_mirror`.

### 4.2 Where it lands — one owner, at the row-build convergence

The rule is one pure function on the typed config (`ProtocolConfig::canonicalize_transport_paths`, in `proto_spec/common.rs`). It is **applied at exactly one place**: `state::protocol_from_parsed` (`crates/xray-tui/src/state.rs:311`), on the config clone, **before** `identity_once()` and before the stored `config`/`transport` columns are built.

**Why there and not at the construction sites.** There are *three* live construction sites, not two, and enumerating them invites the fourth:

| site | how it builds the ws path |
| --- | --- |
| share URL | `from_type_and_path` (`common.rs:99-133`) via each kind's `try_parse` (vless, trojan, vmess) |
| Clash | `clash_to_transport` (`common.rs:634-680`) |
| **Add/Edit form** | `forms.rs::transport_and_path` builds `TransportConfig::Ws(WebSocketConfig { path: opt_text(ss.get("ws.path")), .. })` **directly**, bypassing both |

`protocol_from_parsed` is the only function that turns a `ParsedProto` into a `Protocol` row, and **every** flow converges on it — the three above, plus `stream_import` and `subscriptions` (both call it directly, `stream_import.rs:323`, `subscriptions.rs:497`). A future entry point is covered by construction: nothing reaches the `protocols` table without passing here. The rule keeps its single definition; the *application* has one owner.

**Known consequence, stated.** A `ParsedProto` that is parsed and never persisted keeps its raw path. Nothing dials one — the connect path loads the config from the database (decision 20) — and the import preview (`profiles.rs:1016`) itself calls `protocol_from_parsed`, so it renders the canonical form.

**Not changed:** no transport-builder, no `try_parse`, no Clash converter, no form field.

**What the owner canonicalizes — the transport path AND its mirror.** For vless/vmess/trojan the config carries the path **twice**: the transport's (which `write_transport` hashes into the identity) and a top-level `path` that `write_identity` deliberately excludes but `reconstruct_proto` **emits** (`trojan.rs:313`, `vless.rs:387`, `vmess.rs:385`). Both are canonicalized, together, under one rule — `common::canonicalize_config_paths`, whose only match is `TransportConfig::uri_path_mut`. The mirror is a URI path **exactly when the transport is one**, and that is the same match: a mirror canonicalized on its own is how a `Grpc` service name (`?type=grpc&path=svc`, no leading slash) would get a `/` prepended, export as `/svc`, and reimport as a *different* grpc path — which `write_transport` hashes into a different uid. §4.1 records the rule; §8 carries the falsifier.

### 4.3 Identity and the re-key disposition

**Superseded in part by §4.6.** The path fix alone was to be absorbed with **no `IDENTITY_VERSION` bump and no wipe** (decision A, 2026-09-23) — the wire *format* is unchanged, only values move, and a canonical path is byte-identical so the well-formed majority does not re-key. The user then chose a **wipe** for the early-data re-key (§4.6), and a wipe subsumes this one: nothing is carried, so the transient-duplicate window below no longer applies to this spec.

Recorded for the reasoning, not as live policy:

- **The wire format is unchanged** — same tags, same field order, same elision rule (`opt_str(TR_PATH, …, "/")`). `IDENTITY_VERSION` pins the *format*, so it does not move and `identity_format_is_frozen_for_every_kind` stays valid.
- **Scope of a value re-key is bounded by canonicality.** Only a config whose canonical form differs from its stored form re-keys; a canonical path is byte-identical (§4.1). Without the wipe, the affected set would be the 30 rows above plus sibling spellings, and their old links would age through Purgatory (`purgatory_ttl_secs`, 7 d) to `purge_expired` (`purgatory_retention_secs`, 30 d) — the transient duplicate `docs/database.md:453-457` requires to be stated.
- The alternative that keeps uids fixed (canonicalize only at the build/dial site) is rejected — it forfeits the dedup, which is half the point (§4.5).

### 4.4 Class split — reconciling `2026-09-17-purge-reason-design.md` §7

§7 records that a config whose stored host/path cannot compose a request "fails at the request-BUILD site … Those sites move to `Config` … so they carry `ConfigDefect`. Without that, 40-odd structurally broken configs would sit in the Active view forever." That disposition is **right for one half of the class and wrong for the other**, and the spec must not leave the two documents disagreeing over the same rows.

| stored shape | today | after |
| --- | --- | --- |
| path needs canonicalization (space, emoji, missing leading `/`, double-encoded) | `Config` → `config_invalid` | **never reaches the build site** — canonicalized at parse → dialed like xray does |
| host is structurally invalid (`host=/?–v2rayNplus…`, empty host) | `Config` → `config_invalid` | **unchanged** — genuinely cannot dial as stored |

So §7's mechanism is **preserved for the invalid-authority half** and its example list is narrowed. §7's measured `ConfigInvalid 27` figure is not restated as a target — like every count in that spec, it moves with each run.

This amendment is **specified here and applied with the implementation**, per the 2026-09-22 convention (§3): an `implemented` spec must not describe unlanded behaviour. The plan's task list owns the edit to §7 and the `plans/`/`INDEX.md` bookkeeping.

### 4.5 What is not changed

- No transport-builder change (`ws.rs`, `httpupgrade.rs`, `xhttp.rs` untouched).
- No new header, no method change, no `Host` fallback (that is §5.9 item 7, closed).
- No **identity format** change and no column: the writer's tags, order and elision rule are untouched, so no `IDENTITY_VERSION` bump. §4.6 does carry a `SCHEMA_VERSION` bump (12 → 13) — a wipe, not a column change.

### 4.6 Early data (`?ed=NNNN`) — hoisted into the typed field

`?ed=NNNN` is the carrier for WebSocket **early data**: the client base64s its
first tunnel bytes into the `Sec-WebSocket-Protocol` header
(`websocket/dialer.go:24,153`) so the server can feed them to the tunnel before
its first socket read (`websocket/hub.go:55-59`) — one RTT saved.

**The reference decides the shape, and it is asymmetric.**

| core | carrier |
| --- | --- |
| xray | the `ed=NNNN` **path query**, and nothing else. Its JSON `WebSocketConfig` has exactly five keys — `host`, `path`, `headers`, `acceptProxyProtocol`, `heartbeatPeriod` (`infra/conf/transport_method.go:611-617`) — so `Build()` (622-651) reads `Ed` from the query, and there is no `maxEarlyData` key to accept |
| sing-box | the typed `max_early_data` / `early_data_header_name` (`option/v2ray_transport.go:82-83`), with no path hoisting |

So the design keeps **both**, one owner per direction:

1. **The path is xray's carrier and stays byte-identical.** Stripping `ed` from
   it would silently drop early data for the xray subprocess — the opposite of
   the goal — and no `maxEarlyData` key exists to restore it.
2. **The path's query is authoritative; the typed field is its mirror.**
   `hoist_early_data` reads `ed` out of the ws path into `max_early_data`
   (mirroring xray's own `Build()`), so a form that sets both cannot make them
   disagree, and the field is populated for every share-URL config. The hoist
   does **not** modify the path, so it is order-independent with the
   canonicalization below it, and the top-level mirror stays equal to the
   transport path — query included.
3. **The field is what sing-box gets** (its builder already emits it when set)
   and what the native ws will read.
4. **The xray emitter re-derives the query only when the path has none**
   (`ws_path_for_xray`) — the Clash-import and Add/Edit-form cases, which xray's
   JSON has no other way to express. A share-URL config's path is emitted
   untouched.

This closes a live defect: `max_early_data` is in the identity
(`write_transport` → `TR_MAX_EARLY_DATA`), is emitted to sing-box and is
populated from Clash — but the xray builder dropped it, so a Clash-imported
early-data setting was silently absent from the xray config while the uid claimed
it. `early_data_header_name` has the **same** gap and no fix: xray's JSON has no
key for it either, and its client always uses `Sec-WebSocket-Protocol`
(`dialer.go:153`), so a non-default header name is expressible to sing-box only.
That is a reference limitation, not a missing branch here.

**Native early data itself stays deferred.** The native ws still has no
early-data path, so every native ws real-ping carries one extra RTT versus the
reference — a Test-column fidelity gap and a connection-setup cost, not a
correctness one. It is a perf feature and wants its own before/after in
`flow_cost`.

**The header route is pinned, and why.** `ed` is xray's convention and xray
delivers the payload in `Sec-WebSocket-Protocol` (`websocket/dialer.go:153`),
never by rewriting the path. sing-box's default is different and worse: with
`early_data_header_name` unset it **appends** the base64 payload to the request
PATH (`sing-box/transport/v2raywebsocket/conn.go:170-175`) — a convention private
to a sing-box server configured the same way (`server.go:75-96`), so every other
peer sees a mutated path and 404s. So `hoist_early_data` pins the name to
`Sec-WebSocket-Protocol`, which sends sing-box down its header branch
(`conn.go:176-179`, moved into the WS subprotocol at `client.go:92-96`) —
byte-identical to what xray's hub reads (`hub.go:55-58`), with the path intact.

**Residual, not measured.** Early data removes the first bytes from the socket by
design — both references do it (`dialer.go:203-205` returns `len(ed), nil`) — so
it is only safe when the peer honours the convention. Pinning the header makes
the sing-box config match the convention the link's `ed` implies, but a sing-box
server configured for the *path* mode would not find its suffix. Exposure needs a
ws link carrying `ed=`, **forced to sing-box** (decision 2 sends ws to xray by
default), against such a server. **Nothing in the suite can see it:** every
`ws_paths.rs` and matrix row carries a path with no `ed=`, and the assertions
here are JSON-shape only. The measurement that settles it is one real sing-box
link carrying `ed=` against (a) an early-data-capable server and (b) a plain ws
server — the `2026-09-22` §5.9 item-7 measure-first rule.

**Schema.** `max_early_data` is in the identity stream, so hoisting it re-keys
every ws config carrying `ed`. By the user's call (2026-09-23) that is answered
with a `SCHEMA_VERSION` bump (**12 → 13**) — a wipe, not a migration (decision 4)
— rather than the transient duplicate set that would otherwise age out through
Purgatory. **No `IDENTITY_VERSION` bump:** the wire *format* is unchanged
(`docs/database.md` §3), so the goldens are untouched.

## 5. Blast radius

- `crates/xray-tui-proto/src/proto_spec/common.rs` — the canonicalizer rule; `mod.rs` its entry point.
- `crates/xray-tui-proto` tests — the two-spelling unit case and the idempotence case.
- `crates/xray-tui-native/tests/ws_paths.rs` (new, self-contained — the `e2e/` harness module is untouched).
- `docs/aegis/specs/2026-09-17-purge-reason-design.md` §7 — amendment (with the implementation).
- `docs/aegis/INDEX.md` — this spec's row.

## 6. Compatibility and durability boundary

- **A `SCHEMA_VERSION` bump (12 → 13) wipes the database** (decision 4: a tag mismatch makes `open` delete and recreate the file). No table or column change and no `IDENTITY_VERSION` bump; the bump is the user's call (2026-09-23) to re-import clean rather than carry the `ed` re-key's duplicate set.
- **The canonicalization applies when a config is WRITTEN** (import or refresh). A row already stored with a non-canonical path keeps it — and keeps its `config_invalid` verdict — until its next import. That is §4.3's re-key window seen from the other side: the fix is not retroactive, and a feed that is never re-imported keeps its rows exactly as they are.
- The re-key affects only configs whose canonical form differs from the stored form; a canonical path is byte-identical.
- Transient duplicate window: none — the wipe replaces it (§4.3/§4.6).
- No rollback hazard: the canonicalizer is one pure function; reverting it restores today's stored values on the next import (and today's `ConfigInvalid` verdicts for those rows).

## 7. Verification

1. **Canonicalizer, unit (hermetic, no cores)** — rows 2 and 4 (§2) produce one string, which decodes back to the logical path; their uids are equal (dedup). Rows 1 and 3 stay distinct from each other and from the merged pair (**no false merge**). Inputs pinned verbatim from §2.
2. **Idempotence / no-churn** — canonical input (`/ws`, `/`, absent) is byte-identical through the function, and its uid is unchanged; this is the bound on the re-key stated as a test, not a claim.
3. **Leading-slash** — `trTelegram x` → `/trTelegram%20x`; a `%2F`-prefixed spelling gets its `/` from the decode.
4. **Class split** — an invalid-authority host still yields `Config`/`config_invalid`; a path-encoding config yields no config error at all.
5. **e2e, real reference server (`ws_paths.rs`, `#[ignore]`d — needs binaries)** — a real xray-core server configured with `path=/a b`, the emoji path, and a no-leading-slash path; the native client completes the ws upgrade and an echo round-trip. This pins the encoding against Go's decode and the server's `hub.go:47` compare, not against a self-consistent unit test. Rows run on `trojan` and `vless` (the two observed kinds).

   **The row MUST build its client config by parsing a share URL** (`ProtocolConfig::try_parse_proto` on a URL carrying the non-canonical path) **and routing the result through the same canonicalizer** — never by assigning `WebSocketConfig.path` on a typed struct. The harness's `CaseSpec::client_params` builds configs through `config::client_params_*` (`case.rs:504-513`) and reaches no parser, so a ws-path knob that sets the field directly would bypass the fix entirely: the unchanged `ws_request` still rejects a raw space, so such a row stays red for the wrong reason — or, on a canonical-looking input, passes trivially. `xray-tui-native` depends on `xray-tui-proto` directly, so the parser and the canonicalizer are both reachable from the test binary.
6. **Owner wiring (TUI crate, hermetic)** — `protocol_from_parsed` on a `ParsedProto` with a non-canonical path yields a `Protocol` whose stored `config` carries the canonical path and whose `id` equals the id of the canonical config (§4.2's single owner, asserted rather than assumed).
7. **No-churn regression** — the existing `vless.rs`/`trojan.rs`/`vmess.rs` ws rows stay green, and `/?ed=2560` (§4.1) is byte-identical through the canonicalizer (the early-data form the fleet actually uses).
8. **Mirror coherence + the grpc gate** — a URI-path transport canonicalizes the top-level mirror to the same value it gives the transport path; a `Grpc` transport (whose mirror is a service name) and a pathless transport's mirror come back byte-identical, with the export shape pinned through `reconstruct_proto` (`path=svc`, never `path=%2Fsvc`).

## 8. Risks

| Risk | Treatment |
| --- | --- |
| The extra decode over-decodes a path that legitimately contains a literal `%XX` (`%2F` → `/`) | Named counterexample. The feeds in evidence use double-encoding as the *defect* (§2), so collapsing wins; the falsifier is a server configured with a literal-`%2F` path, which the e2e form can express. |
| A literal `?` or `#` that was *meant* as a path byte | Split-first is the rule (§4.1): `?` starts the query (mirroring Go's `url.Parse` inside xray's dialer), `#` is a fragment and is dropped — which is what `into_client_request` already does today, so this invents no new loss. Falsifier: a real server configured with a literal-`?`/`#` path. |
| Canonicalizing a mirror that is not a URI path (grpc service name, a pathless transport's legacy `host`-as-path) | Prevented by the gate: the mirror is canonicalized only when `canonicalize_paths` reports a URI-path transport (§4.1 coverage). Falsifier: `canonicalize_transport_paths_leaves_a_non_uri_transport_mirror_alone`. |
| Early data against a peer that does not honour the convention | Both references remove the first bytes from the socket by design, so a mismatched peer loses the first flight. `hoist_early_data` pins `Sec-WebSocket-Protocol` (§4.6), the convention the link's `ed` implies and the one xray's hub reads; a sing-box server in *path* mode remains exposed. **Unmeasured** — no fixture carries `ed=`. Falsifier: one real sing-box link with `ed=` against an early-data-capable and a plain ws server. |
| Re-key produces a duplicate pair for the retention window | Stated in §4.3/§6 as the disposition's cost; bounded by the purge TTL. |
| `identity_format_is_frozen_for_every_kind` trips on a fixture with a non-canonical path | Expected if it fires — re-pin that fixture and record why; the format itself is unchanged. |
| Canonicalizing grpc's service name | Prevented by the **runtime gate**, not by scope: `common::canonicalize_config_paths` canonicalizes the config's `path` mirror only through `TransportConfig::uri_path_mut`, which returns `None` for `Grpc`. Falsifier: `grpc_service_name_is_not_a_path` + the export pinned via `reconstruct_proto`. |
| A path the canonicalizer still cannot frame (e.g. control bytes) | Falls through to today's `Config`/`config_invalid` path — fail-closed, unchanged. |

## 9. Retirement

| Item | Status | Trigger |
| --- | --- | --- |
| `Config`-classification for path-encoding rows (§7) | retired by this change | canonicalizer lands |
| `Config`-classification for invalid-authority rows | **kept** | never — genuinely undialable |
| A build-site path fix | never introduced | — |
