# Design: native transports — XHTTP, HTTPUpgrade, h2/v2rayhttp

**Implemented 2026-08-13** — see plan `docs/superpowers/plans/2026-08-13-transports-xhttp-httpupgrade-v2rayhttp.md` and the T1-T7 commits (98bcf3c + T7 docs commit).

Date: 2026-08-13. Branch: `native-core-stub`.

Extends `xray-tui-native`'s transport phase from tcp/ws/grpc to the full
TCP-stream transport set supported by the two e2e cores, for **both VLESS and
VMESS**. Headline is **XHTTP** (v3 dialect, DPI-resistant). Backed by a
battle-tested HTTP client (hyper) so we don't own request/response/chunked
parsing.

## Goal

VLESS + VMESS usable over every transport the e2e cores serve over TCP, with
e2e proof per transport. Concretely, three new transports:

| Transport | xray 26.3.27 | sing-box 1.13.16 | e2e rows |
|-----------|:---:|:---:|---|
| `httpupgrade` | ✅ | ✅ | both cores |
| `xhttp` (splithttp, packet-up + stream-up) | ✅ | ❌ | xray only (single-core) |
| `h2` (v2rayhttp) | ❌ (removed in 26.x) | ✅ | sing-box only (single-core) |

`kcp`/`quic` are UDP transports (mKCP FEC/session; QUIC stack) — explicitly
out of scope, separate plan. `stream-one` (legacy XHTTP v1) deferred. This is
the complete TCP-stream transport matrix for the two pinned cores.

## Non-goals / deferred (documented, no stub)

- kcp, quic (UDP stacks — separate project).
- XHTTP `stream-one` mode (legacy; `auto` selects packet-up; only reachable
  via the REALITY auto-default, which we override with explicit packet-up).
- HTTPUpgrade `ed` early-data flag (xray-only client flag; skip, document).
- h2 PING keepalive (`idle_timeout`/`ping_timeout` in `HttpConfig`).
- xmux / connection-reuse pooling.
- Browser-masquerade default header set. Servers validate Host/path/padding
  only; functional correctness first, masquerade is a later DPI-polish step.
- v2rayhttp no-TLS (h1 early-data) arm — see T4 rationale.
- grpc.rs stays on the h2 crate directly (working, Hunk-codec-verified against
  both cores) — no churn of verified code.

## HTTP client choice: hyper 1.x (minimal features)

Rationale (user decision): maintainability over hand-rolled HTTP. hyper owns
HTTP/1.1 + HTTP/2 framing, chunked, Content-Length, and the RFC 7230 upgrade
(101) handshake. We keep ownership of: the byte stream (`BoxStream`), the dial,
and timeouts.

```toml
# crates/xray-tui-native/Cargo.toml — latest minor versions (verified 2026-08-13)
hyper = { version = "1.11", default-features = false, features = ["client", "http1", "http2"] }
hyper-util = { version = "0.1.20", default-features = false, features = ["tokio"] }
http-body = "1"
http-body-util = { version = "0.1.5", features = ["channel"] }
```

- `default-features = false`: hyper 1.x default features are empty anyway; this
  pins it. We enable **only** `client` (required by hyper's module layout —
  `hyper::client::conn` is gated behind it in 1.11; adds want/pin-project-lite/
  smallvec) + `http1` + `http2`. No `server`, `runtime`, `full`, `ffi`, `capi`.
- hyper-util `default-features = false, features = ["tokio"]`: the standard
  tokio bridge (`TokioIo` wraps tokio `AsyncRead`/`AsyncWrite` into hyper's
  `rt::Read`/`rt::Write`; `TokioExecutor` drives the h2 conn task). hyper 1.x
  implements its rt traits for no tokio types, so this bridge is mandatory —
  hand-rolling it would own unsafe cursor/init code. No connectors, no
  `hyper-util::client` — we keep the stream + dial + timeouts.
- hyper **1.11.0** + http-body-util **0.1.5** (latest minor releases, verified via
  `cargo search` 2026-08-13; per workspace dep rule: minor-version bound pinned
  to the latest release). http-body-util 0.1.5 fetches on first build (0.1.4
  cached). hyper 1.11 uses h2 0.4.x internally (same major as grpc's h2 dep —
  no duplicate).
- http-body-util `Channel::new(buffer) -> (Sender, Channel)` provides the
  upload pipe bodies (stream-up POST, v2rayhttp PUT); `Empty`/`Full` cover the
  discrete POST bodies. bytes + http already direct deps.
- 101 upgrade: `hyper::upgrade::on(response) -> OnUpgrade` yields the upgraded
  stream for httpupgrade.
- Timeouts: existing `timeouts::*` seams stay — each hyper read/write of the
  resulting bodies/streams is wrapped in `tokio::time::timeout`, and
  `http1::Builder::header_read_timeout` bounds the response-head wait.
- hyper 1.11.0 verified in registry: default `[]`, `http1` = httparse +
  futures-channel/core + itoa, `http2` = h2 + futures-*. conn modules carry no
  extra feature requirement. `client::conn::http1::handshake`, http2
  handshake, and `upgrade::on` all present.

## Transport implementations

### T1. HTTP layer (foundation)

No new source module of our own beyond thin wrappers — hyper is the layer.
Shared helpers where a transport needs the same shape:
- handshake a hyper http1/http2 `SendRequest` over a `BoxStream`, spawn the
  connection driver, keep the driver's lifetime tied to the tunnel.
- `Channel`-backed request body wrapper (for stream-up POST / v2rayhttp PUT).
- body-reader adapter from `hyper::body::Incoming` → `BoxStream` (AsyncRead),
  chunked/Content-Length/EOF handled by hyper.

### T2. `transport/httpupgrade.rs` — both cores

- Hyper http1 conn on the (engine or raw TCP) stream.
- `GET {path}`; `Host` = config host > endpoint host; config headers;
  `Connection: Upgrade` + `Upgrade: websocket`.
- Response must be `101`; then `hyper::upgrade::on(response)` → raw duplex
  byte stream (our `BoxStream`). `Connection: upgrade` + `Upgrade: websocket`
  echo headers validated case-insensitively (sing-box contract).
- ALPN: forced `["http/1.1"]` (xray forces it; sing-box sets it when empty).
- `ed` early-data flag: not implemented (deferred); server sees a normal 101
  exchange.

### T3. `transport/xhttp.rs` — xray-core only (v3 dialect)

Wire spec (verified against xray-core 26.3.27 source + mihomo client, which
interops): **no PDU framing, no X-TP-\* headers**. Session correlated by a
uuid v4 in the URL path; uploads are discrete POSTs of raw body bytes with a
decimal `seq` in the path; download is a long-lived GET response body.

**Modes** (config `mode`): `packet-up` (default for `auto`), `stream-up`.
`stream-one` deferred.

**packet-up client flow** (mirrors xray `dialer.go`):
1. `session_id = uuid v4`.
2. Download GET `{path}/{session_id}` → 200, body = read side of the tunnel.
3. Uploader task: buffer app writes; flush a chunk every `sc_min_posts_interval_ms`
   (default 30 ms) or when it reaches `sc_max_each_post_bytes` (default
   1,000,000); POST `{path}/{session_id}/{seq}` (seq 0,1,2…), raw body with
   Content-Length, one POST in flight (wait for its 200) — server 413s over
   `scMaxEachPostBytes`; pace ≥30 ms (server buffering + anti-burst).
4. Padding on every request: `Referer: {url}?x_padding={100-1000 X's}`.
   **X-repeats are universally valid**: xray's server validator checks either
   raw length (`PaddingMethod("")`/repeat-x) or HPACK-huffman length
   (tokenish) against the 100-1000 range, and X/Z are 8-bit huffman codes so
   both lengths equal the byte count. No base62 tokenish generator needed.
5. Close = close the GET (server ends the session; uploader stops).

**HTTP version** (same rule as xray `decideHTTPVersion`): no TLS → HTTP/1.1;
TLS → HTTP/2 (ALPN h2).
- **h1 arm**: GET on the primary stream; upload POSTs on a **second raw TCP
  dial** (`ctx.server_socket()`) with sequential request/response (Go h1
  serializes responses in request order — a long-lived GET body blocks other
  responses on its conn). Two separate hyper http1 conns.
- **h2 arm**: one hyper http2 conn on the engine stream; GET stream + POST
  streams (multiplexed; POSTs still sequential + paced).

**stream-up client flow**:
1. Download GET `{path}/{session_id}` → 200, read side.
2. Upload POST `{path}/{session_id}` with `Channel` pipe body,
   `Content-Type: application/grpc`; response body drained in a spawned task
   (server writes keepalive X-blob blobs into it when a Referer is present —
   must be drained or the stream window stalls).

### T4. `transport/v2rayhttp.rs` — sing-box only (h2 transport)

- Hyper http2 conn (TLS, ALPN `["h2"]`).
- Single full-duplex stream: `PUT {path}` (config `method`, default PUT;
  `:authority` = config host else `www.example.com` — v2fly-compatible
  default), request body = `Channel` pipe (upload), response body (200
  required) = download.
- **h2-only.** The sing-box no-TLS arm (raw request head + early payload
  inline, no Content-Length, server hijacks raw bytes) is not reproducible
  with hyper — hyper always frames request bodies (content-length/chunked)
  and chunked bytes would pollute the hijacked stream. Deferred/documented.
  Real-world use is TLS.

## ALPN in `context.rs`

Transport-implied ALPN extended (explicit user ALPN wins):
| transport | implied ALPN |
|-----------|--------------|
| grpc | `h2` (existing) |
| httpupgrade | `http/1.1` |
| xhttp | `h2` (h1 when no TLS — no ALPN) |
| v2rayhttp | `h2` |

## E2E harness + matrices

`src/e2e/config.rs`: `network` arms for httpupgrade/xhttp/h2 in both
`client_params_*` and `*_inbound` (xray `streamSettings`/sing-box
`transport`). **VMess plumbing** (parked follow-up): `client_params_vmess` +
`vmess_inbound` gain real ws/grpc + the new transports (today vmess rows are
tcp-nominal for ws/grpc).

Single-core rows use the `*_single_core` fns (dedicated `#[case]` core arg)
added this session for ws_reality. New rows:

**vless** (+12): httpupgrade plain/chrome/reality × 2 cores (6); xhttp
packet-up plain/chrome/reality (3) + stream-up plain/chrome (2) (xray-only
single-core; e2e rows all carry TLS → h2 arms — the h1 arms are
hermetic-covered only); v2rayhttp chrome (1, sing-box-only single-core).
Reality on new transports only where the core serves it (xray reality+xhttp
is legal; verified empirically in T-tests).

**vmess** (existing 18 rows are ws/grpc-labeled but run tcp-nominal today —
plumbing makes them real, does not add rows): plumb real ws/grpc + the new
transports through `client_params_vmess` + `vmess_inbound`; re-gate the two
`ws_*_reality` rows to sing-box single-core (xray rejects reality+ws, mirror
of the vless `ws_reality` case); `grpc_*_reality` stays both-core (both cores
serve reality+grpc, proven by the vless matrix). +16 new rows: httpupgrade
plain/chrome × 2 cores (4); ws-reality ×2 sing-box single-core; xhttp
packet-up plain/chrome ×2 sec variants + stream-up chrome (5, xray
single-core); v2rayhttp tls chrome (1, sing-box single-core). Actual final:
**48 vmess tests, all green** (36 baseline − 4 ws-reality re-gate + 4
httpupgrade both-core + 8 single-core rows).

Totals: **100 e2e tests = 96 green + 4 documented ignored** (vless 52 = 48
green + 4 ignored; vmess 48 green). The 4 ignored (vless ws/grpc
plain-into-reality-server semantic-mismatch rows) unchanged.

## Verification gates

- `cargo clippy --workspace --all-targets` → 0 warnings (pedantic+nursery).
- `cargo fmt --check`.
- Unit tests: transport framing (packet-up chunk/seq/pacing logic, padding
  construction, httpupgrade header set, v2rayhttp method/authority) —
  hermetic, no cores.
- Tier 3: `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native
  --features native-e2e --test vless --test vmess` → 91 green + 4 ignored.
  Cores: xray 26.3.27, sing-box 1.13.16.
- NATIVE_CORE.md: crate-map transport row, per-protocol Transports rows
  (native now TCP/WS/gRPC/HTTPUpgrade/XHTTP/h2), e2e count, deferred list.
  AGENTS.md transport/ + xray-tui-native lines.

## Risks

- **hyper http1 conn on a long-lived streaming GET**: standard (hyper streams
  Incoming bodies); verified API shape only — implementation proves it.
- **xray server 413 / session TTL**: client paces POSTs ≥30 ms, keeps GET
  open (session alive), chunks ≤1 MB. Empirically checked in e2e.
- **Reality + xhttp packet-up**: xray's client auto-defaults to stream-one
  under REALITY; we force packet-up explicitly. If the 26.3.27 server rejects
  it, fall back to stream-up for the reality row (documented decision in the
  plan's test task).
- **hyper vs grpc h2**: two h2 consumers (hyper internally for new
  transports, h2 crate for grpc) — same h2 0.4 major, no duplicate, no
  behavior change to grpc.
