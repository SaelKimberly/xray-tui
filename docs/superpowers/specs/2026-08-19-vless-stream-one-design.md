# VLESS XHTTP stream-one + polish — Design Spec (SP6 of VLESS Parity)

Date: 2026-08-19. Branch: `native-core-stub`.
SP6 of the VLESS parity program. SP1-SP5 done (UDP, mux, XUDP+udp443, mKCP,
XHTTP/3). SP6 = the **xhttp `stream-one` mode** (legacy XHTTP v1 — currently
`NotImplemented` in the native core) + the mode auto-selection fix (reality
+ xhttp → stream-one, matching xray) + targeted polish of the accumulated
deferred minors.

## 1. Goal

Implement the xhttp `stream-one` mode in `xray-tui-native`: the single
full-duplex HTTP request tunnel (no session id; the request body carries
client→server, the 200 response body carries server→client), interoperable
with xray-core 26.3.27 splithttp. Fix the mode auto-selection to mirror
xray's (`mode ""`/`auto` → packet-up; **reality present → stream-one**
unless download settings are configured → stream-up). Polish: the cheap,
valuable deferred minors from SP1-SP5 (vmess h3 row, H3Reader chunk copy,
stream-up X-blob drain test, absolute-URI defensive error).

## 2. Non-Goals (scope boundaries for SP6)

- **xmux (x-mux header multiplexing)** — xray's v1 mux over splithttp:
  advanced, out of scope (the plain stream-one/stream-up modes are the
  parity target; the xmux headers ride on the request only when an xmux
  client is configured).
- **DownloadSettings (separate download dial)** — the `download_settings`
  option (a second transport for the download leg): out of scope; the mode
  auto-rule still honors it (present → stream-up) for decision parity.
- **Full polish of every deferred minor** — the final whole-branch review
  triages the rest; SP6 lands only the cheap, valuable ones listed in §6.
- **New dependencies** — none.

## 3. Reference Inventory (wire authority)

- `thirdparty/Xray-core/transport/internet/splithttp/dialer.go` (mode
  selection: `mode ""`/`auto` → packet-up, `realityConfig != nil` →
  stream-one, `DownloadSettings != nil` → stream-up; the stream-one arm:
  `OpenStream(ctx, url, sessionId="", reader, false)` with the request
  `Path` = the normalized path and NO session id), `client.go`
  (`OpenStream`: method = uplink when body non-nil, `FillStreamRequest`,
  non-200 → discard+close, 200 → the response body becomes the conn's
  reader), `config.go` (`FillStreamRequest`: request headers, Referer
  x_padding, `ApplyMetaToRequest`, `Content-Type: application/grpc` when
  body non-nil). Version: 26.3.27.

## 4. Wire Spec

### 4.1 Mode selection (client dial)

Mirror xray's `dialer.go`:
- explicit `mode`: "packet-up" | "stream-up" | "stream-one" — honored.
- `mode` "" or "auto":
  - no reality → **packet-up** (default);
  - reality present → **stream-one**; if `download_settings` configured →
    **stream-up**.
The native currently does packet-up for reality+xhttp (a documented
deviation from the transports plan); after SP6 it follows xray's rule.

### 4.2 stream-one tunnel (legacy XHTTP v1)

- **No session id**: `sessionId = ""` — no uuid in the path, no session
  query; the request URL = `{path}` only (normalized path + query).
- **One full-duplex request**: uplink method (default POST) to `{path}`;
  the request body is the client→server stream; the 200 response body is
  the server→client stream.
- Request headers (`FillStreamRequest`): the configured request headers,
  Referer `x_padding` (query-style padding), `ApplyMetaToRequest` (the
  x-mode/x-seq/x-sid meta headers — pin the exact set against
  `ApplyMetaToRequest`), `Content-Type: application/grpc` (body non-nil —
  the v1 stream framing, same as the native stream-up pipe).
- Response handling: non-200 → discard the body, close, error; 200 → the
  response body becomes the downstream reader (the v1 conn semantics).
- The stream replaces the current `NotImplemented` arm in the native mode
  dispatch (`xhttp.rs`).

### 4.3 Relationship to the existing modes

- packet-up: session id + GET download + per-packet POSTs (implemented).
- stream-up: session id + GET download + one stream-up POST pipe
  (implemented; the server streams keepalive X-blobs into the POST
  response — the native drain handles them generically).
- stream-one: NO session id + one full-duplex POST (this SP).
- The v3 `V3Send` seam extends to the stream-one arm (the tunnel body
  semantics differ from the packet/stream-up seams — a stream-one arm on
  the seam or a direct impl, whichever is cleaner; the protocol logic is
  reused, not reimplemented).

## 5. Architecture (native crate)

### 5.1 The stream-one arm (`transport/xhttp.rs`)

- Mode dispatch: `stream-one` explicit + the auto rules (§4.1) route to the
  new arm; the `NotImplemented` error is replaced.
- The arm: build the request (path, headers, meta, x_padding Referer,
  `application/grpc`), open the stream (h1/h2/h3 per the existing HTTP
  version decision — the stream-one mode works over all three like the
  other modes), wire the bidirectional body (request body ← tunnel writes;
  response body → tunnel reads), non-200 handling.
- The reality auto-default change: after the arm lands, reality+xhttp
  rows exercise stream-one (the server accepts all modes; the change is
  client-side).

### 5.2 Polish items (this SP)

1. **vmess h3 row** — the shared V3Send seam means vmess xhttp/h3 already
   works untested; add the row (vmess h3, xray-single-core) if the harness
   needs no new machinery.
2. **H3Reader per-chunk copy** — `poll_read` copies each recv_data chunk
   via `copy_to_bytes(remaining)`: switch to borrow-slice + partial
   ReadBuf consumption (one alloc per chunk removed).
3. **stream-up X-blob drain test** — the stream-up arm handles X-blob
   keepalives generically but no test exercises the content drain; add a
   hermetic assertion (the fake h3 server streams X-blobs into the upload
   response).
4. **absolute-URI defensive error** — the h3 V3Send rewrite is Host-gated;
   add the else-error (absent Host → Config/InvalidInput instead of a
   doomed path-only URI).

## 6. Testing

### 6.1 Unit (native crate)

- Mode selection: the §4.1 table (explicit modes; auto×{no-reality,
  reality, reality+download_settings}).
- The stream-one arm over the existing fake server machinery (h1/h2
  fixtures) + the hermetic h3 server (T3 of SP5): full-duplex round trip,
  non-200 error, header assertions (Referer padding, meta, grpc
  content-type).
- The polish items' tests (H3Reader borrow test; the X-blob drain
  assertion; the absolute-URI else-error).

### 6.2 e2e (tier-3, `native-e2e` + `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin`)

- stream-one rows: xray-single-core — stream-one + tls (h1 or h2 — the
  existing xhttp server config; verify the mode header reaches the server).
- The reality auto-default: the existing reality+xhttp row now exercises
  stream-one (client-side change; verify it stays green + assert the mode).
- vmess h3 row (polish item 1) if cheap: vmess 48 → 49.
- Counts: vless 75+4 → 76+4 (stream-one row) or 77+4 (+vmess? no — vmess
  count is separate); vmess 48 → 49; total 127 → 129 (or 128).

### 6.3 Gates

- clippy 0 (both feature sets); fmt clean; unit lib green (default +
  native-e2e); vless + vmess sweeps; proto untouched; no new deps.

## 7. Deviations from Upstream

1. **xmux not implemented** — the x-mux header multiplexing layer is out of
   scope (§2); plain stream-one/stream-up interop is the target.
2. **DownloadSettings honored for the mode rule only** — the actual
   separate-download-dial is not implemented; `download_settings` present
   → mode stream-up (matching xray's rule) with the single-dial stream-up
   implementation.

## 8. Verification Order

1. Mode selection unit tests (§4.1).
2. The stream-one arm (unit + hermetic h3).
3. The polish items.
4. e2e rows (stream-one + the reality auto-default + vmess h3).
5. Docs (NATIVE_CORE.md xhttp axis + counts) + full gates.
