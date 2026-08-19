# VLESS XHTTP stream-one + polish Implementation Plan (SP6)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the xhttp `stream-one` mode (legacy XHTTP v1 — the single full-duplex HTTP request tunnel with no session id) in xray-tui-native, fix the mode auto-selection to mirror xray's (reality + xhttp → stream-one), and land the cheap, valuable polish items from the SP1-SP5 deferred lists (vmess h3 row, H3Reader chunk copy, stream-up X-blob drain test, absolute-URI defensive error).

**Architecture:** The stream-one arm in `transport/xhttp.rs` replaces the `NotImplemented` mode dispatch: a single uplink-method request to `{path}` (no session id), Referer x_padding + meta headers + `Content-Type: application/grpc`, request body = client→server, 200 response body = server→client, non-200 → error. The mode selection mirrors `dialer.go` (`""`/`auto` → packet-up; reality → stream-one; download_settings → stream-up). The v3 logic is reused, not reimplemented.

**Tech Stack:** tokio, hyper/h3 (existing), the existing xhttp.rs machinery, no new deps.

**Spec:** `docs/superpowers/specs/2026-08-19-vless-stream-one-design.md` (wire authority: xray 26.3.27 `transport/internet/splithttp/{dialer,client,config}.go`).

## Global Constraints

- xray-tui-proto NEVER modified. No new dependencies.
- Mode selection mirrors xray EXACTLY: explicit modes honored; `""`/`auto` → packet-up; reality → stream-one; reality + download_settings → stream-up.
- stream-one: NO session id; request URL = `{path}`; uplink method (default POST); Referer x_padding; meta headers per `ApplyMetaToRequest` (pin the exact set against the Go source); `Content-Type: application/grpc`; non-200 → discard+close+error; 200 → response body = the downstream reader.
- The v3 protocol logic is REUSED (the V3Send seam extends to stream-one; no protocol reimplementation).
- Polish items: (1) vmess h3 row if the harness needs no new machinery; (2) H3Reader borrow-slice + partial ReadBuf consumption (one alloc per chunk removed); (3) stream-up X-blob drain hermetic assertion; (4) absolute-URI else-error (absent Host → error instead of a doomed path-only URI).
- The reality auto-default changes the existing reality+xhttp rows' mode (now stream-one, client-side; the server accepts all modes — verify the rows stay green).
- e2e: xray-single-core rows; counts vless 75+4 → 76+4, vmess 48 → 49, total 127 → 129 (or as landed).
- Gates per spec §6.3; run BOTH `--features native-e2e` builds when the feature-gated e2e module is touched. Cores at /tmp/core-bin.

---

### Task 1: Mode selection + the stream-one arm

- [ ] The §4.1 mode-selection function (explicit modes; auto×{no-reality, reality, reality+download_settings}) + unit tests for the full table.
- [ ] The stream-one arm in xhttp.rs replacing the `NotImplemented` dispatch: the single full-duplex request (path, headers, x_padding Referer, meta headers per ApplyMetaToRequest, application/grpc), the bidirectional body wiring (request body ← tunnel writes; 200 response body → tunnel reads), non-200 handling.
- [ ] The arm over the existing h1/h2 fixtures + unit tests (full-duplex round trip, non-200, header assertions).

- Consumes: spec §4.1/§4.2/§5.1.
- Produces: stream-one over h1/h2.

### Task 2: stream-one over h3 + the reality auto-default

- [ ] The stream-one arm over the hermetic h3 server (the SP5 fake): full-duplex round trip, non-200, header assertions.
- [ ] The reality auto-default change (reality + xhttp → stream-one): update the client mode selection; verify the existing reality+xhttp rows stay green (they now exercise stream-one).

- Consumes: Task 1.
- Produces: stream-one everywhere + the xray-matching default.

### Task 3: Polish items

- [ ] H3Reader borrow-slice + partial ReadBuf consumption (one alloc per chunk removed) + test.
- [ ] Stream-up X-blob drain: hermetic assertion that the fake h3 server streams X-blobs into the upload response and the client drains them generically.
- [ ] Absolute-URI defensive else-error (absent Host → error instead of the doomed path-only URI).
- [ ] vmess h3 row if the harness needs no new machinery (vmess 48 → 49).

- Consumes: Tasks 1-2 (the h3 machinery).
- Produces: the polish.

### Task 4: e2e rows + docs + full verification

- [ ] The stream-one e2e row (xray-single-core) + the reality auto-default verification in the existing rows; sweeps: vless 76+4, vmess 49.
- [ ] NATIVE_CORE.md + AGENTS.md: the stream-one mode + the reality auto-default + counts; no stale claims.
- [ ] Full gates per spec §6.3: clippy 0 (both feature sets), fmt, unit lib, e2e sweeps.

- Consumes: Tasks 1-3.
- Produces: SP6 closed.
