# VLESS Flow (xtls-rprx-vision) — Design Spec

Date: 2026-08-13. Branch: `native-core-stub`.

## 1. Goal

Implement XTLS Vision flow control (`xtls-rprx-vision`) for the VLESS client in
`xray-tui-native`, interoperable with mainstream xray-core and sing-box servers.
The app's inner TLS session is raw-relayed through the tunnel after a splice
point; the tunnel's outer TLS (fingerprint / REALITY) camouflages the header
phase.

## 2. Non-Goals (scope decisions)

- **UDP**: vision + UDP → client error (xray rejects UDP/443; other UDP needs
  the XUDP mux rewrite — `command=Mux` to `v1.mux.cool:666` — out of scope,
  separate mux plan).
- **`xtls-rprx-vision-udp443`**: not accepted (same wire; only lifts the UDP/443
  rejection — pointless without XUDP mux).
- **Legacy flows** (`xtls-rprx-origin`, `-direct`, `-splice`): removed from
  mainstream xray; not implemented.
- **Mux / XUDP / UoT**: not implemented. Any non-empty flow other than
  `xtls-rprx-vision` keeps the current `NotImplemented` guard.
- **Server-side vision**: native core is a client; no server work.

## 3. Reference Inventory (wire authority)

| Ref | Version | Role | Key files |
|---|---|---|---|
| `/tmp/xray-main-26327` (fetched tag v26.3.27) | 26.3.27 | **wire authority** (matches the e2e binary) | `proxy/vless/encoding/addons.go`, `proxy/vless/outbound/outbound.go`, `proxy/vless/inbound/inbound.go`, `proxy/proxy.go` |
| `thirdparty/Xray-core` | 26.7.28 (fork) | diffed identical for vision | same paths |
| `thirdparty/jets` | 0.4.3 | Rust client-side precedent | `src/proxy/vless/xtls.rs`, `stream.rs`, `transport/tls.rs` |
| `thirdparty/shoes` | — | Rust server-side precedent | `src/vless/vision_{filter,pad,stream,unpad}.rs` |
| `thirdparty/sing-box` + `sing-vmess@0.2.8-…` (go.mod) | 1.13.16 | e2e server #2; wire-compatible vision | `option/vless.go`, `protocol/vless/{inbound,outbound}.go`, `vless/vision.go` |
| `thirdparty/leaf` | — | **partial only** (read-side unpad, no write padding) — not used as reference | — |

Wire contract verified **byte-identical** across all of: mainstream 26.3.27,
fork 26.7.28, jets, shoes, sing-vmess.

## 4. Wire Spec (xtls-rprx-vision)

### 4.1 VLESS request header with flow addon

```
0x00            version (1 byte)
uuid            VLESS user id (16 bytes)
addons_len      addons length (1 byte)
addons          protobuf { string Flow = 1; bytes Seed = 2; }
command         TCP = 0x01, UDP = 0x02, Mux = 0x03, Rvs = 0x04 (1 byte)
port            big-endian (2 bytes)
address         atyp (1) + address (IPv4 4 / IPv6 16 / domain 1+len)
```

For `flow = "xtls-rprx-vision"`: `addons` = protobuf field 1
(`0x0A 0x10` + 16 bytes `xtls-rprx-vision`) = **18 bytes**; `addons_len` = 18.
Client sends no `Seed` (field 2 unset). Empty flow → `addons_len` = 0
(current behavior unchanged).

`EncodeHeaderAddons` (addons.go:31-47) and `DecodeHeaderAddons` (server,
addons.go:50-83, verification switch empty) confirm. The fork's `0x12`-tag
reading in earlier notes was a misread; `addons.proto` declares `Flow = 1`.

### 4.2 Body: padded frames

Every frame, after the optional UUID:

```
[16-byte UUID]      first frame only (the VLESS user id bytes)
[1 byte  cmd]       0x00 Continue | 0x01 End | 0x02 Direct
[2 bytes content_len]  big-endian
[2 bytes pad_len]      big-endian
[content_len bytes]    the app payload
[pad_len bytes]        zero bytes
```

Padding length generation (`XtlsPadding`, proxy/proxy.go:496-534):
- long padding (`content < 900 && longPadding`): `pad = 900 + rand(0..500) - content`
  (xray: `rand.Int(500) + 900 - contentLen`)
- else: `pad = rand(0..256)`
- cap: `pad = min(pad, 8192 - 21 - content)` → max frame 8171 + 21 bytes,
  so one frame always fits one outer TLS record (16 KiB limit).

The UUID is written once per direction (first frame); the reader gates the
whole vision mode on a UUID match in the first ≥21-byte chunk
(`XtlsUnpadding` initial state, proxy/proxy.go:535-564): mismatch → plain
passthrough (no unpadding).

### 4.3 Padding state machine (client uplink writer)

Shared `TrafficState` (proxy/proxy.go:104-175): `NumberOfPacketToFilter = 8`,
`EnableXtls = false`, `IsTLS = false`, `IsTLS12orAbove = false`,
`RemainingServerHello = -1`, `Cipher = 0`; both directions start
`WithinPaddingBuffers = true`, `IsPadding = true`.

`VisionWriter.WriteMultiBuffer` (proxy/proxy.go:322-405):
1. Filter the chunk (see 4.5).
2. If `isPadding`:
   - **nil chunk** (camouflage, no app data): one long-padding Continue frame
     (`XtlsPadding(nil, Continue, uuid, longPadding=true)`).
   - else compute `isComplete = IsCompleteRecord(chunk)` (whole chunk is
     well-formed `17 03 03`-framed records); `reshape` chunks ≥ `8192-21`
     (split at the last `0x17 0x03 0x03` boundary; `ReshapeMultiBuffer`).
   - per buffer: if `IsTLS && starts with 0x17 0x03 0x03 && isComplete` →
     EnableXtls already true ⇒ `switchToDirectCopy = true`; command =
     Continue, or **End**/**Direct** when it is the last block of the chunk;
     `isPadding = false`.
     else if `!IsTLS12orAbove && NumberOfPacketToFilter <= 1` → End frame,
     `isPadding = false`, stop (early finish for non-1.3 traffic).
     else → Continue frame (End/Direct if last block and already `!isPadding`).
3. After `switchToDirectCopy`: the writer writes to the **raw** underlying
   stream (Go: `UnwrapRawConn`). The last padded frame is the final outer-TLS
   record; subsequent bytes are inner records written raw.

Direct fires **only** when: inner TLS 1.3 detected (`EnableXtls` — set from
the *downlink* ServerHello sniff) AND an inner app-data record (`0x17 0x03 0x03`)
completes a write. End fires for non-TLS / TLS 1.2 inner traffic → padding
stops but the outer TLS **continues** (normal relay, no splice).

### 4.4 Padding state machine (client downlink reader)

`VisionReader.ReadMultiBuffer` (proxy/proxy.go:203-288) with
`XtlsUnpadding` (proxy/proxy.go:535-617): per chunk, consume frames using
`RemainingCommand/Content/Padding` + `CurrentCommand`; after each block:
- `currentCommand == 0` (Continue) → remain in padded mode, parse next block
- `== 1` (End) → `WithinPaddingBuffers = false` (stay in outer TLS)
- `== 2` (Direct) → `WithinPaddingBuffers = false`, `switchToDirectCopy = true`
  → subsequent reads come **raw** from the underlying stream.
- Unknown command → log + continue (Go behavior).

### 4.5 TLS filter (both directions, shared state)

`XtlsFilterTls` (proxy/proxy.go:619-671), budget 8 chunks:
- `0x16 0x03 0x03 … 0x02` at `[0..3],[5]` → ServerHello: set
  `IsTLS12orAbove`, `IsTLS`; `RemainingServerHello = (len_hi<<8|len_lo) + 5`;
  cipher = `[43 + session_id_len + 1 .. +3]` (needs ≥ 79 bytes).
- `0x16 0x03 … 0x01` at `[0..2],[5]` → ClientHello: `IsTLS = true`.
- While `RemainingServerHello > 0`: if `{0x00 0x2b 0x00 0x02 0x03 0x04}`
  (supported_versions TLS 1.3) appears → `EnableXtls = true` unless
  cipher == `0x1305` (TLS_AES_128_CCM_8_SHA256); budget = 0, stop.
- TLS 1.2 (ServerHello exhausted without 1.3) → budget = 0, stop.

### 4.6 Session trace (client, inner TLS 1.3)

1. Dial TCP → outer TLS handshake (engine: `TlsMode::Plain` or `Reality`).
2. Write VLESS header (addons = `0x0A 0x10 xtls-rprx-vision`).
3. Write empty long-padding Continue frame (camouflage; xray uses a 500 ms
   timer — we emit it immediately, same wire effect, see §9).
4. App's inner ClientHello arrives → padded Continue frame (filter: IsTLS).
5. Server relays ClientHello to the target; target's ServerHello returns →
   server's downlink writer pads it (Continue); client's downlink reader
   unpads → filter: IsTLS12orAbove + cipher + TLS 1.3 → `EnableXtls = true`.
6. Client's next uplink write (inner TLS 1.3 Finished / app data, a complete
   `0x17 0x03 0x03` record) → Direct frame as the last padded write →
   **write side switches to raw** (no more outer TLS records).
7. Server's read side sees Direct → switches to raw reads; server's downlink
   writer (EnableXtls from *its* filter) Direct-frames the target's next
   app-data write → **client's read side sees Direct → switches to raw**.
8. Both directions raw: inner TLS records relayed end-to-end; the tunnel's
   outer TLS is abandoned (no close_notify) — one TLS session on the wire.

Server: response header `[0,0]` (ver + addons_len 0) is NOT written as its
own TLS record — `EncodeResponseHeader` buffers it and `SetFlushNext`
(inbound.go:619-623; common/buf/writer.go:165-167,215) flushes it TOGETHER
with the first padded frame in ONE outer-TLS record:
`[0,0][uuid][cmd][clen][plen][content][pad]`. The client handles this: the
codec's Unpadder skips a leading `[0,0]` before the UUID gate
(vision.rs:663-672), and the outer peel (stream.rs:46-78) is lenient (hands
the first non-0x00 byte back as payload). Go peels the header from the raw
conn BEFORE wrapping the VisionReader (outbound.go getResponse) — our peel
sits outside the codec, hence the codec-side skip. Flow must equal the
account flow; XRV inbound rejects UDP (inbound.go).

## 5. Architecture

### 5.1 Layer stack (client connection)

```
app stream
  └─ vision codec (native protocol/vless/vision.rs)      NEW
       ├─ writer: pad frames → Direct → raw-write
       ├─ reader: unpad frames → Direct → raw-read
       └─ shared TrafficState + filter
  └─ engine TlsStream (xray-tui-tls)                     + direct mode (NEW)
  └─ transport::connect (tcp)                            existing
```

Composition order preserved (`chain.rs`): dial → security → (transports are
tcp-only for vision; ws/grpc/httpupgrade/xhttp/h2 are incompatible with the
Direct socket handoff) → protocol. **Vision rows use network=tcp only**; a
non-tcp transport with vision is a build error (xray: "XTLS only supports TLS
and REALITY directly for now").

### 5.2 Engine change (xray-tui-tls) — per-direction direct mode

`TlsStream<S>` gains:

```rust
pub fn set_write_direct(&mut self);  // subsequent writes bypass record layer → inner
pub fn set_read_direct(&mut self);   // subsequent reads bypass record layer → inner
```

- **Why per-direction, not `into_inner`**: the write side goes raw when the
  client's filter triggers (step 6); the read side stays on the outer TLS
  until the server's Direct frame arrives (step 7). `into_inner` would kill
  the still-padded direction. This mirrors Go's `input`/`rawInput` reach-ins
  (outbound.go:270-296) without unsafe.
- **Write transition invariant**: caller (vision codec) must have fully
  written + flushed the Direct frame through the record layer before calling
  `set_write_direct`. Subsequent `poll_write`/`poll_flush` forward to `inner`.
- **Read transition invariant**: caller must have consumed the Direct frame
  completely; the record layer is then at a clean record boundary with an
  empty `read_buf`. Verified: `poll_record` (record/stream.rs:96-172) reads
  exactly the 5-byte header + `len` payload per poll — no read-ahead — so
  raw bytes after the last outer-TLS record remain in the socket kernel
  buffer and are recovered by raw reads (lossless).
- Engine stays VLESS-agnostic: the flags are generic record-layer bypasses.
- `into_inner` is NOT added (not needed with per-direction flags).
- New unit tests: duplex round-trip (encrypted phase → write-direct → raw
  writes read by the peer as plaintext; read-direct after a padded phase),
  boundary invariants, no-read-ahead losslessness.

### 5.3 Vision codec (native `protocol/vless/vision.rs`)

Pure, unit-testable port of §4.2-§4.5:

- `VisionFrame { cmd, content: Bytes, pad_len }` codec
  (`encode`/`decode` with the `Remaining*` split-frame state).
- `TrafficState` (shared by reader + writer halves; mirrors xray).
- `VisionWriter` — `AsyncWrite` adapter wrapping the TlsStream:
  - per `poll_write` chunk: filter → pad (one frame per chunk) → forward to
    the TlsStream; buffer partial writes; return the accepted app-byte count.
  - chunks ≥ 8171: reshape at the last `0x17 0x03 0x03` boundary.
  - camouflage: empty Continue long-padding frame emitted once right after
    the VLESS header.
  - Direct: after the Direct frame is flushed → `set_write_direct()`; raw
    writes forward directly.
- `VisionReader` — `AsyncRead` adapter wrapping the TlsStream:
  - unpad frames across chunk boundaries; deliver content; `End` → pass
    through; `Direct` → `set_read_direct()`; raw reads forward directly.
  - initial-state UUID gate: first ≥21-byte chunk must start with the user
    UUID, else permanent passthrough.
- `filter_tls` — budget 8, per §4.5.

The codec is **generic over the stream** (holds the TlsStream) and lives in
the native crate; the engine has no vision knowledge.

### 5.4 VLESS wiring (native `protocol/vless/`)

- `header.rs`: `encode_request` gains the flow addon: for
  `flow == "xtls-rprx-vision"`, `addons_len = 18`, bytes
  `0x0A 0x10 "xtls-rprx-vision"`. (protobuf field-1 string; no Seed.)
- `mod.rs` (`connect`): replace the flow guard:
  - `flow == "xtls-rprx-vision"` → vision path:
    - require outer security (TLS 1.3 or REALITY) — engine is TLS 1.3-only,
      so `security().is_some()` + non-empty `type_str` (the existing
      `has_tls()` helper) is the check; else `NativeError` "XTLS only supports
      TLS and REALITY directly for now".
    - require network == tcp; else `NativeError`.
    - command TCP only; UDP → `NativeError` (reject; no XUDP mux).
    - write header + camouflage frame, then wrap the stream in the codec.
  - any other non-empty flow → `NotImplemented` (unchanged guard).
- `stream.rs`: lazy response-header peel unchanged — the server's `[0,0]`
  does NOT arrive as its own TLS record: it is coalesced with the first
  padded frame into ONE outer-TLS record (§4.6 step 8); the codec's Unpadder
  skips the leading `[0,0]` before its UUID gate (vision.rs:663-672), so the
  peel never sees it — the lenient peel (stream.rs:46-78) hands the first
  non-0x00 byte back as payload and finishes. The peel happens *outside* the
  codec (wraps the codec), preserving the existing non-vless fallback.
- Flow value source: `VlessConfig.flow` (proto already parses/stores/hashes
  it; no proto changes).

## 6. Error Handling

- Unknown/unsupported flow: `NotImplemented` (current behavior).
- Vision without TLS/REALITY: clear `NativeError` (mirror xray's message).
- Vision + UDP: `NativeError` (no mux).
- Vision + non-tcp transport: `NativeError` at build/connect.
- Codec errors (malformed frames, oversized records, I/O): propagate as
  `NativeError` variants; no panics. Malformed frame handling follows Go:
  unparseable chunks pass through unmodified (never dropped mid-stream on
  the read side; the UUID gate is the trust boundary).
- Direct-mode misuse (write-direct before flush): documented invariant, the
  caller owns ordering; tests enforce the boundary.

## 7. Testing

### 7.1 Unit (native crate)

- Frame encode/decode round-trips incl. split-frame reads (Remaining*).
- Padding lengths: long/plain branches, cap (8171 − content), determinism via
  injected RNG (no new `rand` dep — use the existing `ring::rand` pattern from
  the xhttp padding work).
- UUID gate: match → unpad; mismatch → passthrough; `[0,0]` prefix passthrough.
- Commands: Continue chain, End, Direct transitions on both reader+writer.
- Filter: ClientHello-only, ServerHello with/without TLS 1.3 marker, cipher
  `0x1305` (no EnableXtls), 0x1301 (EnableXtls), budget exhaustion.
- Camouflage frame emission (one Continue long-padding frame after header).
- Addon encode: `0x0A 0x10 "xtls-rprx-vision"` + `addons_len = 18`.

### 7.2 Engine unit

- `set_write_direct` / `set_read_direct` round-trip over a duplex pair.
- Boundary invariants (clean record boundary, empty read_buf at Direct).
- No-read-ahead: raw bytes after the last record are recoverable.

### 7.3 Hermetic (native, raw-TCP fake vision server)

Pattern from the xhttp/httpupgrade hermetic tests: a raw TCP peer that
performs the outer TLS as the *server* side of the engine (rustls test
double), reads the VLESS header, then drives the vision frames: padded echo
with Continue/End/Direct; assert client frame bytes (UUID, commands, padding
bounds) and the raw-relay switch after Direct.

### 7.4 e2e (tier-3, feature `native-e2e` + `XRAY_TUI_CORE_BIN_DIR`)

Servers: xray 26.3.27 + sing-box 1.13.16 (`/tmp/core-bin`). Server config:
xray `settings.clients[].flow = "xtls-rprx-vision"`; sing-box
`users[].flow`. Client JSON: `flow: "xtls-rprx-vision"` (proto already
parses). CaseSpec gains a flow axis.

**Matrix — 8 vision rows** (protocol=vless, network=tcp, flow=vision):

| Core | Security | App type | Path exercised |
|---|---|---|---|
| xray | tls | inner-tls | Direct + raw splice |
| xray | tls | plain | End (padding stops, outer TLS continues) |
| xray | reality | inner-tls | Direct over REALITY |
| xray | reality | plain | End over REALITY |
| sing-box | tls | inner-tls | Direct (sing-vmess server) |
| sing-box | tls | plain | End |
| sing-box | reality | inner-tls | Direct over REALITY |
| sing-box | reality | plain | End |

- **inner-tls rows**: the app side = engine client (`xray_tui_tls::client::connect`,
  a browser profile, `insecure` verifier) establishing a real TLS 1.3 session
  to a rustls echo target **through the tunnel** — exercises ClientHello
  sniff → ServerHello → EnableXtls → Direct → raw splice end-to-end. The
  engine client as the inner app is deliberate dogfood.
- **plain rows**: raw HTTP text through the tunnel — filter budget exhausts,
  End frame, padding stops, outer TLS persists. Proves the non-splice
  fallback stays interoperable.
- Existing rows untouched (counts preserved); vision rows use the
  `vless_single_core`-style explicit-core pattern where needed.

## 8. Gates

- clippy workspace pedantic+nursery: 0 warnings.
- `cargo fmt --check` clean.
- unit: existing 215 + new (engine + codec) green.
- e2e: existing 96 green + 4 ignored preserved; 8 vision rows green.
- `xray-tui-proto` NOT modified.
- No new `rand`/unsafe deps; RNG via `ring::rand` (existing pattern).
- `NativeError`-only error surface.

## 9. Documented Deviations from xray

1. **No 500 ms camouflage timer** — the empty long-padding Continue frame is
   emitted immediately after the VLESS header. Same wire bytes; deterministic
   (xray's timer only delays the empty frame until app data or 500 ms).
2. **Per-direction direct flags instead of unsafe reflection** — Go reaches
   into `tls.Conn` `input`/`rawInput` (outbound.go:270-296); the engine's
   record-aligned reads make the flags equivalent and lossless.
3. **UDP + vision rejected** (xray rejects only UDP/443 and XUDP-rewrites the
   rest) — no mux in scope.
4. **`xtls-rprx-vision-udp443` rejected** — requires the UDP path (3).

## 10. Verification Order

1. Engine: direct-mode unit tests green.
2. Codec: frame/padding/filter/transition unit tests green.
3. Hermetic: fake-vision-server test green.
4. e2e sweep: `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vless` (all rows incl. 8 vision).
5. Gates: clippy + fmt + full unit suite.
