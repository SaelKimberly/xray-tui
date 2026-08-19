# VLESS XHTTP/3 (splithttp over QUIC) Implementation Plan (SP5)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add xhttp-over-QUIC to xray-tui-native: when the xhttp transport's TLS ALPN is exactly `h3`, dial a QUIC connection (quinn — RFC 9000/9001/9002) over UDP, negotiate HTTP/3, and run the existing splithttp v3 protocol (uuid session, GET-body download, POST uploads with seq + pacing + Referer padding) over the h3 transport. 0-RTT early data enabled; fresh connections do a full handshake. Interoperable with xray-core 26.3.27 splithttp + HTTP/3.

**Architecture:** The h3 mode is a DIFFERENT DIAL, not an upgrade step: quinn Endpoint (UDP) → quinn Connection (TLS via rustls with the single `h3` ALPN — the engine TLS fingerprint machinery does not apply to QUIC) → h3 client connection → the splithttp v3 protocol over h3 (reusing the existing xhttp.rs session/GET/POST logic, factored transport-agnostic). Dispatch: xhttp + exactly-one-ALPN `h3` → `xhttp::connect_quic`; reality → h2 (never quic, per `decideHTTPVersion`).

**Tech Stack:** quinn (QUIC, tokio), h3 + h3-quinn (HTTP/3) — the program's first major new deps (user-approved); tokio; the existing xhttp.rs protocol logic.

**Spec:** `docs/superpowers/specs/2026-08-19-vless-xhttp3-design.md` (wire authority: xray 26.3.27 `transport/internet/splithttp/{dialer,client,hub}.go` — `decideHTTPVersion`, QuicParams defaults: 300s idle / 10s keepalive / -1 max incoming streams, `DialEarly`/`ListenEarly` 0-RTT, `http3.Transport`).

## Global Constraints

- xray-tui-proto NEVER modified. The h3 ALPN comes from the existing `TlsOpts::alpn`.
- New deps (user-approved): quinn + h3 + h3-quinn (or the maintained equivalent stack). Version pins: latest minor with a minimum bound per the workspace direct-dep convention.
- The dispatch mirrors xray's `decideHTTPVersion` EXACTLY: reality present → h2; no TLS → h1.1; TLS with exactly one ALPN: "http/1.1" → h1.1, "h3" → QUIC/h3, other → h2; 0 or >1 ALPN → h2.
- The splithttp v3 protocol (uuid session path, GET-body download, raw POST uploads, seq + 30 ms pacing + Referer padding, ≤1 MB chunks, packet-up/stream-up) is unchanged over h3 — reuse the existing xhttp.rs logic; do NOT reimplement the protocol.
- 0-RTT: quinn early data enabled; fresh e2e connections handshake (same as xray without a ticket).
- e2e rows: xray-single-core only (sing-box has no xhttp QUIC); reality-over-QUIC impossible (decideHTTPVersion); count vless 74+4 → 75+4 (or 76+4), vmess 48, total 127 (or 128).
- Gates per spec §7.4; run BOTH `--features native-e2e` builds when the feature-gated e2e module is touched.
- Cores at /tmp/core-bin (xray 26.3.27 + sing-box 1.13.16).

---

### Task 1: Dependency integration + the h3 dispatch rule

- [ ] Add quinn + h3 + h3-quinn (or equivalent) to the native crate; a minimal quinn Endpoint/Connection smoke test (loopback UDP, self-signed, ALPN) proving the stack compiles + connects.
- [ ] The dispatch decision as a pure function mirroring `decideHTTPVersion` (alpn/reality/security → version: "1.1"/"2"/"3"): unit tests for every branch (reality → 2; none → 1.1; exactly-one h3 → 3; exactly-one http/1.1 → 1.1; other/0/2 ALPNs → 2).
- [ ] Wire the xhttp dispatch: `transport_type == "xhttp"` + the decision == "3" → the quic arm (stub returning NotImplemented until T2 lands the client, or land the arm with the client in T2 — pick the cleaner order).

- Consumes: spec §4.1/§5.1/§5.2.
- Produces: the stack + the decision rule.

### Task 2: The h3 client arm (xhttp over QUIC)

- [ ] `xhttp::connect_quic(ctx)`: quinn Endpoint (bind UDP) → connect (server host/port, SNI, ALPN ["h3"]) → h3 client connection.
- [ ] Factor the splithttp v3 protocol (session open, GET-body download, POST uploads with seq/pacing/Referer, packet-up/stream-up) so the h1/h2 hyper framing and the h3 arm share it (the protocol logic stays identical; the transport differs).
- [ ] The h3 request/response handling: GET `/{uuid}`, POST uploads (≤1 MB chunks, 30 ms pacing), server 200s, session lifecycle. 0-RTT: quinn early data enabled (fresh connections handshake).
- [ ] Unit tests: the factored protocol over a mock h3 transport (or the existing xhttp tests unchanged + new h3-specific ones where the framing differs).

- Consumes: Task 1.
- Produces: the h3 client end-to-end.

### Task 3: Hermetic HTTP/3 server double

- [ ] A minimal h3 server double over quinn (loopback UDP, self-signed cert, h3 ALPN): accepts the connection, answers GET `/{uuid}` + the v3 POSTs per the contract (session-open/200 + download/upload echo or the simplified contract), driving the client's full path without real cores. Event-based, bounded assertions (the established hermetic pattern).

- Consumes: Task 2.
- Produces: core-free interop confidence.

### Task 4: e2e rows (xray xhttp/h3)

- [ ] Harness: the xray splithttp server config with the h3 ALPN TLS config (+ quic_settings if the harness emission needs it — verify with the real xray) + the row(s) (xhttp/h3 tls-standard; a second row only if a meaningful variant exists — reality is impossible, no-TLS is impossible (h3 requires TLS)).
- [ ] Sweeps: vless 75+4 (or 76+4) + vmess 48. The e2e module is feature-gated — run BOTH feature sets.

- Consumes: Tasks 2-3.
- Produces: the green row(s).

### Task 5: Docs + full verification

- [ ] NATIVE_CORE.md + AGENTS.md: the xhttp/h3 axis (QUIC dial, ALPN trigger, decideHTTPVersion mirror, quinn/h3 deps, xray-only e2e), counts (unit incl. new tests, vless 75+4 or 76+4, vmess 48, total 127/128).
- [ ] Full gates per spec §7.4: clippy 0 (both feature sets), fmt, unit lib, e2e sweeps.

- Consumes: Tasks 1-4.
- Produces: SP5 closed.
