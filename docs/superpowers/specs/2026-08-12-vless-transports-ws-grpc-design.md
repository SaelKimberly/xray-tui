# VLESS Transports: WebSocket + gRPC over the Engine TLS Stream

Date: 2026-08-12
Branch: `native-core-stub`
Status: Approved (user: "proceed with this task")

## Purpose

Prove the transport architecture decision: transport framing runs OVER the
engine's TLS/REALITY stream (TLS outer, transport inner), not via rustls
integration inside transport crates. Implement WebSocket and gRPC transports
for the native VLESS client, e2e-verified against real xray-core + sing-box
inbounds.

Cross-repo evidence (thirdparty/): ray (VLESS server) runs WS/XHTTP-H2 over
TLS via `S: AsyncRead+AsyncWrite+Unpin` + h2 crate; leaf's WS transport
receives a pre-TLS `AnyStream` and calls tokio-tungstenite
`client_async_with_config(req, stream, ws_config)`; jets stacks
Tcp→Tls→Ws with `Box<dyn ProxyStream>`; shoes wraps `Box<dyn AsyncStream>`.
All four: TLS outermost, transport framing over the established stream,
framing crates never own TLS.

## Architecture

Per link, the fold becomes (TLS outer for ws/grpc):

```
dial (tcp) → security (engine TLS/REALITY) → transport upgrade (ws/grpc) → protocol (VLESS)
```

Current native fold is `transport → security → protocol` (chain.rs:37-38)
with transport = tcp dial only. The rework: keep `transport::connect(ctx,
base)` for dialing, run `security::wrap`, then run a new
`transport::upgrade(ctx, stream)` that applies ws/grpc framing over the
secured stream (tcp = passthrough). Later chain hops (base = Some) already
use the same upgrade shape.

## Components

### 1. `crates/xray-tui-native/src/transport/ws.rs` — WebSocket transport

- tokio-tungstenite over the engine stream (BoxStream → `WebSocketStream<BoxStream>`).
- Request: GET `<path>` (default `/`), `Host: <config.host or server host>`,
  extra headers from `WebSocketConfig.headers`, standard Upgrade headers
  (Connection: Upgrade, Upgrade: websocket, Sec-WebSocket-Key,
  Sec-WebSocket-Version: 13). No subprotocol unless v2rayN-style
  `Sec-WebSocket-Protocol` is requested (xray client sends none by default).
- Verify server 101 + path echo.
- Wrapper `WsStream`: AsyncWrite buffers bytes → Binary messages (flushed on
  poll_write_ready/poll_flush); AsyncRead drives tungstenite read
  (auto pong), buffers Binary payloads into an internal read buffer, errors
  on Text/Close.
- Handshake timeout: `timeouts::TRANSPORT` (new, 15s); errors → `NativeError::Transport`.
- Early data (max_early_data/early_data_header_name), v2ray_http_upgrade:
  out of scope (documented unsupported for this proof).

### 2. `crates/xray-tui-native/src/transport/grpc.rs` — gRPC transport

- h2 crate `h2::client::handshake(stream)` over the engine stream.
- One stream (gun mode, matches xray/sing-box default): HEADERS
  `:method: POST`, `:scheme: https`, `:authority: <authority or server host>`,
  `:path: /<service_name>/Tun`, `content-type: application/grpc`,
  `te: trailers`, `grpc-accept-encoding: identity`, optional `user-agent`
  from config. Service name default "gun" (verify against
  thirdparty/Xray-core/transport/internet/grpc/ at impl time; sing-box
  service_name default "" which maps to "gun").
- Framing: VLESS bytes → one gRPC message: 5-byte prefix (1 flag byte 0x00 +
  4-byte big-endian length) + payload. Read: accumulate DATA → parse prefix →
  emit payload bytes to VLESS. (Ray skips the prefix — do NOT follow; xray
  interop requires it.)
- Wrapper `GrpcStream { send: SendStream<Bytes>, recv: RecvStream, write_buf,
  read_buf }`: AsyncWrite prefixes + writes DATA; AsyncRead parses.
- h2 flow control, window updates, trailers: handled by the h2 crate.
- Mode "multi", ping_interval, authority-less variants: out of scope.

### 3. Chain rework — `crates/xray-tui-native/src/chain.rs`

```
let dialed = transport::connect(&ctx, base).await?;        // tcp dial / reuse
let secured = security::wrap(&ctx, dialed).await?;         // engine TLS (passthrough when none)
let upgraded = transport::upgrade(&ctx, secured).await?;   // ws/grpc over TLS (tcp = passthrough)
base = Some(protocol::connect(&ctx, upgraded).await?);
```

`transport::upgrade(ctx, stream)` dispatches on `ctx.transport_type()`:
`ws` → ws::connect, `grpc` → grpc::connect, `tcp`/None → passthrough. The
existing `transport::connect(ctx, base)` stays as the dialer for the first
hop (and reuse for later hops).

### 4. ALPN per transport (native-side, no engine change)

- grpc: force ALPN `["h2"]` on the TLS config (from `LinkContext.alpn_vec()`:
  transport grpc → override to h2 when user didn't set one; Reality forces
  h2+http/1.1 already).
- ws: force `["http/1.1"]` (the ws upgrade is an HTTP/1.1 exchange; over
  Reality both are offered, server picks http/1.1).
- Engine negotiated-ALPN accessor: NOT needed for ws/grpc (config-time
  forcing suffices). Deferred to the XHTTP-auto task.

### 5. E2E proof — `crates/xray-tui-native/src/e2e/`

- `vless_inbound` gains a `network` parameter; emits xray/sing-box
  `streamSettings` with `network: "ws"/"grpc"` + wsSettings/grpcSettings
  per core dialect.
- New cases (variant.rs): `VlessWsPlain`, `VlessWsTls`, `VlessGrpcPlain`,
  `VlessGrpcTls` — each × xray + sing-box (8 new runs in the sweep).
  REALITY stays tcp-only (both cores: reality only over tcp; already proven).
- Client side: `NativeConnectParams.protocol.transport` = Ws/Grpc config;
  TLS variant on top (plain/tls/reality-unsupported-combination excluded).
- Assert: connect + HTTP proxy round-trip through the core (existing
  `spawn_core` + `probe_target` machinery).

## Global Constraints

- ring-only in xray-tui-tls; native adds tokio-tungstenite + h2 as
  non-optional deps (they are transport framing, not TLS).
- Clippy pedantic+nursery ZERO warnings; edition 2024; rustfmt.
- xray-tui-proto UNTOUCHED (configs already model Ws/Grpc transports).
- Behavior-preserving: tcp-only paths byte-identical (tls/vless e2e green).
- e2e: feature `native-e2e` + `XRAY_TUI_CORE_BIN_DIR` (hard-fail absent),
  xray 26.3.27 + sing-box 1.13.16 at /tmp/core-bin.

## Out of Scope (future tasks)

XHTTP, httpupgrade, splithttp, kcp, quic, ws early data, grpc multi-mode,
negotiated-ALPN accessor, Trojan transports (same transport module, later).

## Verification

- `cargo test -p xray-tui-native --lib` — unit tests (5-byte prefix codec,
  ws request headers, chain order).
- e2e sweep: existing 11 cases + 8 new (ws/grpc × plain/tls × 2 cores) all
  green; workspace tests + clippy zero warnings.
