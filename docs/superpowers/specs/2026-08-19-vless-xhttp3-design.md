# VLESS XHTTP/3 (splithttp over QUIC) — Design Spec (SP5 of VLESS Parity)

Date: 2026-08-19. Branch: `native-core-stub`.
SP5 of the VLESS parity program. SP1-SP4 done (UDP path, mux, XUDP+udp443,
mKCP). SP5 = the **xhttp (splithttp v3) transport over QUIC** — the native
xhttp transport gains a QUIC dial mode speaking HTTP/3, interoperable with
xray-core 26.3.27 splithttp listeners configured for HTTP/3.

## 1. Goal

Add xhttp-over-QUIC to `xray-tui-native`: when the xhttp transport's TLS
ALPN is exactly `h3`, dial a QUIC connection (quinn — RFC 9000/9001/9002)
over UDP, negotiate HTTP/3 via the `h3` ALPN, and run the existing splithttp
v3 protocol (uuid session path, GET-body download, raw POST uploads with seq
+ pacing + Referer padding) over the HTTP/3 transport. 0-RTT early data
enabled (session resumption); fresh connections do a full handshake, exactly
like xray's `DialEarly` without a session ticket. Interoperable with
xray-core 26.3.27 (`splithttp` + `quic_settings`).

## 2. Non-Goals (scope boundaries for SP5)

- **Standalone `network: "quic"` transport** — removed from xray 26.3.27
  (no `transport/internet/quic`); the old v2ray quic transport is gone.
  xray's QUIC surface is splithttp's HTTP/3 mode only.
- **QUIC with REALITY** — xray's `decideHTTPVersion` returns "2" when a
  reality config is present (reality is h2-only in splithttp); no
  reality-over-QUIC row.
- **Congestion control choice** — quinn's built-in (CUBIC/NewReno, or BBR
  when the feature is available) vs xray's quic-go BBR/reno/brutal: a
  performance concern only; the wire interoperates regardless (default
  quinn congestion).
- **udphop / udpmask** — xray's port-hopping + packet-mask wrappers around
  the QUIC socket: non-default, out of scope (same as SP4's udpmask).
- **sing-box parity** — sing-box's xhttp has no QUIC variant; e2e rows are
  xray-single-core.
- **Hand-rolled QUIC** — quinn + h3 are the implementation (user-approved;
  the program's first major new dependency).

## 3. Reference Inventory (wire authority)

- `thirdparty/Xray-core/transport/internet/splithttp/dialer.go` (the
  `httpVersion == "3"` path: `decideHTTPVersion`, QuicParams → quic.Config,
  `http3.Transport`, `quic.DialEarly`), `client.go` (the http3 client
  carrying the splithttp v3 protocol), `hub.go` (server: `quic.ListenEarly`
  + h3 server), `transport/internet/config.proto` (`QuicParams`),
  `common/net/net.go` (`ConnIdleTimeout`=300s, `QuicgoH3KeepAlivePeriod`=10s).
  Version: 26.3.27.

## 4. Wire Spec

### 4.1 The HTTP/3 trigger (what makes xhttp use QUIC)

`decideHTTPVersion(tlsConfig, realityConfig)`:
- reality config present → `"2"` (h2 over the engine TLS; QUIC never used).
- no TLS config → `"1.1"` (h1).
- TLS with exactly one ALPN (`len(NextProtocol) == 1`):
  - `"http/1.1"` → `"1.1"`;
  - `"h3"` → `"3"` (**QUIC path**);
  - anything else → `"2"`.
- TLS with 0 or >1 ALPN → `"2"`.

So: **xhttp + TLS + a single `h3` ALPN = HTTP/3 over QUIC**. This is a pure
client-side decision (the server negotiates h3 via its ALPN config).

### 4.2 QUIC connection (client)

- Dial a UDP socket to the server; `quic.DialEarly` (0-RTT capable) with a
  `quic.Config` built from `QuicParams` (defaults when unset):
  - `MaxIdleTimeout` = 300s (`ConnIdleTimeout`);
  - `KeepAlivePeriod` = 10s (`QuicgoH3KeepAlivePeriod`);
  - `MaxIncomingStreams` = -1 (unlimited — the hardcoded quic-go/http3
    default);
  - receive windows: quic-go defaults when 0.
- TLS client config: the stream settings' TLS config **plus the `h3` ALPN**
  (the single `NextProtocol` entry that triggered the path).
- `http3.Transport` round-tripper: the splithttp v3 protocol expressed as
  HTTP/3 requests over the QUIC connection's bidirectional streams.
- Rust mapping (quinn): `quinn::Endpoint` (client, 0-RTT via
  `enable_early_data` + session resumption; fresh e2e connections do a full
  handshake — same as xray without a ticket), `quinn::Connection` with the
  h3 ALPN (quinn's TLS = rustls with the server name + ALPN; the engine TLS
  fingerprint machinery does NOT apply to QUIC — xray's h3 server accepts
  any TLS 1.3 client, no fingerprint gate), `h3` + `h3-quinn` (or the
  equivalent maintained http3 client) for HTTP/3 framing.

### 4.3 The splithttp v3 protocol over HTTP/3

Unchanged semantics, h3 transport: the client opens a session via the uuid
path (`/{uuid}`), downloads the body via GET, uploads via raw POST requests
with `seq` + 30 ms pacing + `Referer` padding, ≤1 MB chunks; packet-up and
stream-up modes. HTTP methods/paths/headers are identical over h3 — only
the transport (QUIC streams + h3 framing) differs. The existing native
`transport/xhttp.rs` protocol logic is reused; the h3 mode adds an
HTTP/3 client arm.

### 4.4 Server (e2e only)

xray inbound: splithttp listener with the h3 ALPN TLS config (+ optional
`quic_settings`); `quic.ListenEarly(conn, tlsConfig, quicConfig)` + the
http3 server. The e2e harness emits this config; the client needs no
server-specific knowledge beyond the h3 ALPN.

## 5. Architecture (native crate)

### 5.1 New dependencies (user-approved)

- `quinn` (QUIC — RFC 9000/9001/9002, tokio), the `h3` HTTP/3 crate +
  `h3-quinn` (or the maintained equivalent) — the standard Rust HTTP/3
  client stack. Version pins: latest minor with a minimum bound, matching
  the workspace convention for direct deps.

### 5.2 Transport dispatch (`transport/mod.rs` + `transport/xhttp.rs`)

- The existing xhttp connect path currently runs h1/h2 over the engine TLS
  stream (`upgrade` framing). The h3 mode is a **different dial**: a QUIC
  connection (UDP + quinn) whose h3 stream carries the xhttp protocol —
  it replaces the dial + security + upgrade chain, not an upgrade step.
- Dispatch: `transport_type == "xhttp"` + the TLS ALPN is exactly `h3` →
  `xhttp::connect_quic(ctx)` (new arm); else the existing path. The ALPN
  lives in the transport/security config (`TlsOpts::alpn` — the proto
  already carries it).
- `xhttp::connect_quic`: quinn Endpoint (bind UDP), connect to the server
  (host/port from the ctx; SNI = server name; ALPN `["h3"]`), then the h3
  client handshake, then the splithttp v3 protocol over h3 (reuse the
  session/GET/POST logic factored to be transport-agnostic — the current
  xhttp.rs hyper-based framing stays for h1/h2; the h3 arm speaks h3).
- 0-RTT: quinn early data enabled (`EndpointConfig`/`ConnectOptions`
  early-data settings + a session-cache); fresh connections handshake.

### 5.3 The h3 client arm (`transport/xhttp/h3.rs` or in xhttp.rs)

- `h3::client::Connection` over the quinn connection; requests: GET
  `/{uuid}` (download body), POST uploads with the v3 headers (`x-seq`,
  Referer padding, chunk sizing ≤1 MB, 30 ms pacing); stream-up / packet-up
  modes; server responses (session open, 200s) handled per the v3 contract
  already implemented for h1/h2.
- The engine TLS + fingerprint profile do not apply (QUIC's TLS is
  quinn/rustls internal); the h3 ALPN is the only requirement.

## 6. Error Handling

- QUIC handshake failure, h3 connection errors, and the v3 protocol errors
  surface through `NativeError` (io/NotImplemented/Config arms as the
  existing xhttp path uses); every network step bounded by the existing
  timeout helpers (`timeouts` module).
- 0-RTT rejection (server sends Retry or rejects early data): quinn falls
  back to a full handshake automatically — no client action.

## 7. Testing

### 7.1 Unit (native crate)

- The dispatch decision: alpn h3 → quic arm; alpn http/1.1 → h1; alpn h2 /
  none / reality → h2 (mirror `decideHTTPVersion`); no-TLS → h1.1.
- The factored v3 protocol logic over a mock h3 transport (if the framing
  is factored to a trait) or the existing xhttp unit tests unchanged.

### 7.2 Hermetic

- A minimal HTTP/3 server double over quinn (loopback UDP): accepts the
  quinn connection with the h3 ALPN, answers the GET `/{uuid}` + POSTs per
  the v3 contract (or a simplified echo), driving the client's
  session-open/download/upload paths without real cores. Bounded, event-
  based assertions (the established hermetic pattern).

### 7.3 e2e (tier-3, `native-e2e` + `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin`)

- xray-single-core rows (sing-box has no xhttp-over-QUIC): xhttp/h3 with
  the tls-standard variant (+ a fingerprint variant IF the h3 arm can carry
  one — likely not, quinn's TLS is rustls-internal; document the row as
  tls-standard). Reality-over-QUIC is impossible by `decideHTTPVersion`.
- Count: vless 74+4 → 75+4 (1 row) or 76+4 (2 rows); vmess unchanged; total
  127 or 128.

### 7.4 Gates

- clippy 0 (both feature sets); fmt clean; unit lib green (default +
  native-e2e); vless 75+4 (or 76+4) + vmess 48 sweeps; proto untouched
  (no new fields needed — the ALPN comes from the existing TlsOpts).

## 8. Deviations from Upstream

1. **Congestion control** — xray defaults to quic-go BBR; quinn's default
   (CUBIC/NewReno, or BBR when available) is used; wire-compatible.
2. **TLS stack** — xray uses Go crypto/tls (quic-go); native uses
   quinn/rustls. The h3 server accepts any TLS 1.3 client; no fingerprint
   gate (engine TLS not involved in QUIC).
3. **0-RTT on fresh connections** — xray's `DialEarly` without a session
   ticket performs a full handshake; native same (quinn early data is
   enabled; tickets come from prior sessions).
4. **udphop/udpmask wrappers** — skipped (non-default; §2).

## 9. Verification Order

1. quinn + h3 dependency integration + the dispatch rule (unit).
2. The h3 client arm + the factored v3 protocol (unit + hermetic h3 peer).
3. e2e rows (xray xhttp/h3).
4. Docs (NATIVE_CORE.md xhttp axis + counts) + full gates.
