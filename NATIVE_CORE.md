# NATIVE_CORE.md — Native proxy core + TLS fingerprint engine

Status and roadmap for the two crates that remove the subprocess dependency for
the tunnel itself: **`xray-tui-native`** (in-process client-side protocol
implementations) and **`xray-tui-tls`** (the ring-based TLS client — TLS 1.3,
plus a TLS 1.2 ECDHE+AEAD fallback — that backs the fingerprint/REALITY
security paths).

Sibling docs: `AGENTS.md` (agent guide), `ARCHITECTURE.md` (whole-system
architecture), `CONTEXT.md` (domain glossary + decisions), `docs/protocols.md`
(protocol wire specs), `docs/clash.md` (Clash YAML conversion).

## Purpose

The main xray-tui app gets its tunnel by spawning xray-core or sing-box as a
subprocess, writing JSON configs, and talking gRPC. That works, but it is
heavy: two Go binaries to ship, update, and babysit. The native core is the
long-term replacement — a pure-Rust, in-process client for the same protocols,
built on the same `xray-tui-proto` typed configs the subprocess path already
uses. The TLS engine exists because stock rustls ClientHellos are instantly
identifiable: browser-fingerprint mimicry (and REALITY) needs a client hello
we fully control.

The native core is primarily a client (each `connect` dials a remote proxy);
it also hosts a local **SOCKS5 inbound** (`inbound/`) — accept → route →
outbound — so the crate can stand in for the whole subprocess path: a local
SOCKS5 listener whose routing engine (`xray-tui-route`) forwards to
direct/block/proxy outbounds, where the proxy outbound reuses the native
client tunnel. Remote *server-side* protocol behavior remains the job of
xray-core / sing-box; the e2e harness runs real cores as servers to prove
wire compatibility.

**Roadmap: engine-only TLS.** The native client path is engine-only — every
TLS/REALITY connect runs through `xray-tui-tls`. The rustls *client* path was
removed; rustls client usage is now limited to the server-side test double
(unit tests + the e2e `tls_echo` dest) and quinn's internal QUIC TLS for the
xhttp h3 dial (spec §5.2 — a mandatory native dep) — the engine remains the
only client-facing TLS path. The engine negotiates TLS 1.3 by default and
runs a TLS 1.2 client path (ECDHE + AEAD only — no CBC, no static RSA)
when the ServerHello picks 1.2, so legacy TLS 1.2-only servers are
reachable. REALITY stays 1.3-only: a 1.2 ServerHello surfaces
`TlsError::RealityFallback` and the caller runs Spider-X instead of failing
the dial.

## Principles

- **Xray composition order, one layer stack**: `dial → security → transport
  upgrade → protocol → tunnel` — TLS is OUTERMOST (a ws/grpc transport runs
  framing INSIDE the engine TLS session, matching xray/sing-box). mKCP is the
  dial-end exception: it is a UDP **dial**, not an upgrade — `connect` gains
  the kcp arm (`transport/kcp`, fresh UDP socket + KCP session), `upgrade`
  stays passthrough, and the security layer wraps the KCP stream exactly as
  it wraps TCP. xhttp-h3 is the second dial-end exception: xhttp with a
  single `h3` ALPN makes the chain self-contained (`is_self_contained`) —
  `connect_quic` replaces dial + security + upgrade (quinn/rustls is
  internal to QUIC; the engine TLS never wraps it — spec §5.2). Each phase
  consumes the previous phase's byte stream (`BoxStream`) and returns the
  next. A protocol = the same handshake-over-stream pipeline unless its
  `ConnectShape` says otherwise (device tunnels, own-handshake protocols,
  outbound-only kinds).
- **Config source of truth is `xray-tui-proto`.** No config model is defined in
  native; `NativeConnectParams` wraps the typed `ProtocolConfig` +
  `EndpointEssentials`. The host/port is injected from the endpoint at connect
  time.
- **`xray-tui-tls` is ring-only**: no aws-lc-rs, no `rand`, no `unsafe`.
  CSPRNG enters through a crate-local `SecureRandom` seam (ring's trait is
  sealed; a blanket impl covers any `ring::rand::SecureRandom`). One
  documented exception: x25519-dalek (runtime dep) for the REALITY client,
  because ring's `EphemeralPrivateKey` is single-use and cannot serialize —
  REALITY must agree twice with the same scalar. A second documented
  exception: `oqs` (vendored liboqs build, no system dep) for ML-KEM-768 —
  ring has no post-quantum KEM (SP7). `zeroize` is NOT a third exception:
  it supplies no algorithm and competes with nothing in ring — it is a
  memory-hygiene dep, and both its own feature and the `zeroize` features
  of the cipher crates are a separate contract (next bullet).
- **Secrets are wiped, and the wipe is a dependency contract.** Every owned
  key buffer in both crates is `Zeroizing`/`ZeroizeOnDrop` (key schedules,
  X25519/ML-KEM shared secrets, the REALITY auth key, VMess/VLESS/trojan
  protocol key material). The other half is invisible in the source: the
  `zeroize` FEATURE is OFF by default in every RustCrypto cipher/hasher we
  use and is STRIPPED from `x25519-dalek` by `default-features = false`, so
  both manifests list it explicitly. After any crypto dependency edit,
  re-check `cargo tree -p <crate> -e features | grep zeroize` — losing the
  feature compiles clean and silently stops wiping. What is deliberately NOT
  wiped, and why, is decision 19 in AGENTS.md.
- **Explicit absence beats silent fallback.** Every protocol kind, transport,
  and security has a module and a dispatch arm. Unsupported combinations
  return `NativeError::NotImplemented` naming the missing feature — no stub
  that pretends to work.
- **Real verification, three tiers** (see below). Wire-format claims carry
- **`xray-tui-proto` is changed only deliberately.** If a protocol config
  lacks a field the tunnel needs, that's a proto change evaluated separately
  (SP7's VLESS `mlkem768x25519plus` encryption parser + hybrid curve names
  are the one instance so far).

## Verification tiers

| Tier | Gate | What runs | Evidence |
|------|------|-----------|----------|
| 1 — offline | `cargo test -p xray-tui-tls -p xray-tui-native --features native-e2e --lib` | unit: wire encodings, RFC 8448 key-schedule vectors, GREASE pairing, JA3/JA4 goldens, VMess Go byte-vectors, rustls-server interop (dev-dep), multi-record reassembly, TLS 1.2 client path (ECDHE + AEAD key block, explicit-nonce records, DOWNGRD-sentinel + RFC 7627 EMS guards), Spider-X fallback, transport framing (httpupgrade header set, xhttp chunk/seq/pacing + padding, v2rayhttp method/authority), vision codec (padded frames, TLS filter, Direct splice) + hermetic fake-vision-server, vless UDP (packet framing, packetaddr codec, PacketConn, hermetic fake-UDP-server), v1.mux.cool mux (frame codec, `MuxClient` multiplexer, hermetic fake-mux-server), vless XUDP (mux UDP sessions — per-packet dests + `GlobalID`, `PacketConn` `XUdp` mode, hermetic fake-mux UDP session), trojan UDP (address-framed `PacketConn` over one reused frame buffer, domain-reply → no-address, length-exact payload read, split-flight reassembly, bad-CRLF/unknown-ATYP rejection), vmess UDP (record-boundary datagram view over the AEAD stream — one record per datagram, cancel-safe `write_datagram`, xray 8192-byte chunk cap, header-dest mismatch refusal), hy2 UDP (UDPMessage wire layout, borrowed-view serializer, defrag reassembly — out-of-order, duplicate and stale-packet discard, drop-undeliverable-and-keep-reading, bandwidth-string parser), xhttp h3 (the `decideHTTPVersion` dispatch rule — single `h3` ALPN → QUIC, `http/1.1` → h1, 0/2+ or other → h2; the shared v3 protocol over the `V3Send` seam; hermetic h3 server double over loopback QUIC; ML-KEM-768 primitives (liboqs roundtrips + size pins), TLS hybrid curves (X25519MLKEM768 key-share encode/parse, `pq || classical` key schedule, fake-PQ-server handshake), VLESS `mlkem768x25519plus` encryption (parser, relay/AEAD chain, xor-mode masking, hermetic double), REALITY 4588 hybrid share (hermetic fake REALITY PQ server)) | 659 lib tests (208 tls + 451 native; native incl. 36 vision-module tests (incl. 2 hermetic fake-vision-server) + 24 vless UDP/XUDP-path tests (udp framing, packetaddr codec, PacketConn incl. `XUdp` mode, 2 hermetic fake-UDP-server) + 42 mux tests (v1.mux.cool codec + `MuxClient`/`SessionStream`/`UdpSession` + 3 hermetic fake-mux-server incl. the fake-mux UDP session) + 68 mKCP tests (segment codec, KCP session — RTO/RTT + send/recv windows + retransmit + state machine, hermetic fake-peer over loopback UDP) + 39 non-vless UDP-path tests (19 trojan, 8 vmess udp, 12 hy2 udp))
| 2 — live grader | `cargo run -p xray-tui-tls --example grader -- --profile <id>`; `cargo run -p xray-tui-tls --example grader -- --roster [--family <name>] [--sample]`; `cargo test -p xray-tui-tls --test tls_peet_ws -- --ignored` | ClientHello graded against tls.peet.ws | Chrome130 JA4 `t13d1516h2_8daaf6152771_f37e75b10bcc`; Firefox 128 resolves next-modern to the kept `firefox_139_windows_desktop` (JA3 `fdb1b23bd019c5596f46c8bf59f21968` + JA4 `t13d1516h2_8daaf6152771_02713d6af862`); kept 71-profile roster live sweep (71/71 both runs) + offline JA4 gate + Cloudflare amiabot report (see `docs/tls-fingerprint-roster.md`, `docs/amiabot-roster-report.md`) |
| 3 — real-core e2e | `XRAY_TUI_CORE_BIN_DIR=<dir> cargo test -p xray-tui-native --features native-e2e --test vless --test vmess --test trojan --test hysteria2` | native client against spawned xray-core (26.3.27) + sing-box (1.13.16) servers, transport + TLS-variant matrix, VLESS vision flow axis, VLESS UDP datagram path, VLESS mux axis, VLESS XUDP axis, VLESS mKCP axis, VLESS XHTTP/3 axis, VLESS ML-KEM PQ axis, VMess/Trojan/Hysteria2 UDP datagram rows | 155 tests = 149 green + 6 documented ignored (vless 78+6, vmess 54, trojan 14, hysteria2 3; ignored: the 4 ws/grpc plain-into-reality-server semantic rows × both cores + reality-pq + pq-enc (SP7 — see the ML-KEM axis below); single-core rows run only on the serving core: xhttp + xhttp-h3 + kcp + pq-enc on xray, v2rayhttp + ws/httpupgrade-reality + mux-vision + vision-udp443 on sing-box)

Tier 2 needs network; tier 3 needs the version-pinned core binaries (hard-fail,
not skip, on version mismatch). Tier 1 is hermetic and is the CI gate.

### Fingerprint roster (two tiers)

The TLS engine's fingerprint corpus is a **two-tier roster of 71 profiles**:

- **Hand tier — 2 wire-exact profiles** (`profiles/hand_selected.rs`,
  `spec!`-declared): `chrome_130` and `edge_106`, both Windows desktop.
  Transcribed byte-for-byte from the original hand modules (deleted; the
  equality was pinned by the removed equivalence test). These are the only
  profiles whose byte-level wire shape — extension order, signature-algorithm
  ordering, GREASE placement (Edge 106 carries two GREASE extensions to
  Chrome 130's one) — is captured from a real browser, not synthesized.
- **Generated tier — 69 JA4-faithful entries** (`profiles/generated/`,
  emitted by `gen_specs.py --emit`; chrome 19, firefox 9, safari 6,
  chrome_android 16, safari_ios 19): the deterministic `select_roster` kept
  subset of the 1825-entry ja4db manifest (top-3 distinct-JA4 clusters per
  browser/os/device triple, family sanity floors/caps, PSK excluded, 2
  hand-upgraded slots removed). Every entry reproduces its registered source
  JA4 exactly, but byte-level shape may be synthesized (48 of 69 flagged
  `low_fidelity` in the manifest — the JA4-faithful contract).

Generator (`crates/xray-tui-tls/src/fingerprints/catalog/gen_specs.py`):
`--manifest` rebuilds `specs_manifest.json` from the ja4db export (writing
the `kept` flags), `--select` prints the deterministic kept roster (names),
`--emit` re-renders the Rust files (byte-deterministic), `--selftest`
verifies the committed files match a fresh render of the kept subset.

Fidelity contract: every entry's built ClientHello must reproduce its
registered source JA4 — pinned offline by `tests/generated_ja4_gate.rs`
(69/69 generated entries, 0 failures; the 2 hand profiles are pinned by
`tests/tls_peet_ws.rs::local_fingerprints_match_locked_constants`) and
confirmed live against tls.peet.ws by the grader's `--roster` sweep
(71/71 both runs — every entry connected and returned its expected JA4;
0 `pre_shared_key` entries by construction). Cloudflare-side verification
is the amiabot sweep (`docs/amiabot-roster-report.md`): 35/71 profiles got a
verdict (the remaining 36 are HRR-limited against Cloudflare — the engine
has no HRR retry and rejects the HRR a hybrid-unaware server sends), every
successful row `likely_human` under an IP-pollution caveat.

Resolution over the two tiers: **next-modern** (a query version `v`
resolves to the smallest kept major `>= v` within the os/device-compatible
identity group; above the group's newest row or below its oldest kept major
refuses) with **cross-triple os-drop** fallback (an exact
`(browser, os, device)` triple miss retries with the os dropped — desktop
hellos are OS-independent within a family). See `docs/tls-fingerprint-roster.md`.


## Crate maps

### xray-tui-native (`crates/xray-tui-native/src/`)

| Module | Responsibility |
|--------|----------------|
| `lib.rs` | `Stream`/`BoxStream` seam, `connect()` entry, `NativeTunnel` |
| `chain.rs` | `connect_chain`: fold the layer stack |
| `context.rs` | `LinkContext`, `NativeConnectParams` (wraps proto types) |
| `addr.rs` | `TargetAddr` (domain/IP + port) encode/decode |
| `inbound/` | local SOCKS5 server (`Socks5Inbound`, RFC 1928 + RFC 1929 auth): accept → `xray-tui-route` `Engine` → tagged outbound (`Outbound`/`OutboundKind` — Direct / Block / Proxy, the proxy reusing `crate::connect`). TCP CONNECT **plus UDP ASSOCIATE** (`Socks5InboundConfig::udp`, default on): per-datagram routing, per-family direct sockets, and a proxy leg task owning a split `PacketTunnel` (a stream carrier's `recv` must never be raced in a `select!`). Association pinned to the control peer, expiring without a first datagram, ending on control-TCP EOF; per-datagram failures drop the datagram only. BIND refused `0x07`, `HijackDns` drops the datagram (TCP `0x02`). Hermetic tier-1: codec units (greeting, command discriminants, UDP header round-trip + malformed inputs) + full-flow integration (echo relay, block/reject rules, domain routing, `override_addr` rewrite, auth, unknown tag, UDP echo, fragment drop, source pinning, multi-destination fan-out, teardown/port release) |
| `inbound/http.rs` | local HTTP CONNECT proxy (`HttpInbound`, same `Engine` → tagged-outbound path as SOCKS5). **CONNECT-only v1**: absolute-form requests are not forwarded (`501`), CONNECT without `host:port` `400`, head over 16 KiB `431`, missing/malformed/wrong Basic credential `407` + `Proxy-Authenticate: Basic realm="xray-tui"`, Block/Reject `403`, unknown outbound tag or failed dial `502`, success a bare `200 Connection Established` (refusals framed with Content-Type/Content-Length/`Connection: close` + a one-line reason). Optional Basic auth (`HttpInboundConfig::with_auth`) — the TUI session has no credential source and relies on the loopback bind; both inbounds `warn` once at bind time on a non-loopback bind without auth. Post-head pipelined bytes are replayed into the tunnel; a failed `accept` retries instead of killing the listener |
| `transport/` | `connect` = TCP dial (ws/grpc/httpupgrade/xhttp/v2rayhttp; framing is an upgrade step) **or fresh-UDP mKCP dial** (`kcp/` — wire codec + session + stream: KCP segments over UDP, one segment per datagram, conv from a process-global counter; xray-only — sing-box has no kcp) **or the QUIC dial** (`quic.rs` — the shared quinn endpoint + rustls/TLS-verify + 0-RTT helpers used by BOTH the xhttp h3 arm and the hysteria2 client; xhttp + exactly-one `h3` ALPN → `connect_quic` and hysteria2 are `is_self_contained` — the dial REPLACES dial + security + upgrade, quinn/rustls TLS is internal, webpki-roots default verify with the harness-CA override in test/e2e builds); `upgrade` = ws (tokio-tungstenite over the engine stream, v2ray Host/path/headers, Binary framing) + grpc (h2 over the engine stream, gun mode, `Hunk` protobuf + 5-byte gRPC prefix, deferred response headers via spawned task, write-through with flow-control reserve) + httpupgrade (hyper http1 conn + RFC 7230 101 upgrade: `GET {path}`, `Connection: Upgrade` + `Upgrade: websocket` echo validated, ALPN `http/1.1`) + xhttp (splithttp v3, xray-only server: uuid session in path, GET-body download, raw POST uploads with `seq` + 30 ms pacing + `Referer` `x_padding`, ≤1 MB chunks; packet-up + stream-up; h1 when no TLS, h2 over TLS — the h3 mode is the `connect` QUIC dial above, not an upgrade step; the v3 protocol (session open, GET download, POST uploads, pacing) is written once over the `V3Send` seam shared by h1/h2/h3, and h3 requests use absolute-URI form (`:scheme`/`:authority` per RFC 9114 §4.3.1 — the interop fix)) + v2rayhttp (h2 single full-duplex PUT stream, `:authority` = config host else `www.example.com`; sing-box only). HTTP framing (requests/responses/chunked/101) is hyper 1.11 (`client`+`http1`+`http2`) + hyper-util 0.1.20 (`tokio`) + http-body-util 0.1.5 (`channel`) — we own the byte stream, the dial, and the timeouts. QUIC/HTTP-3 is quinn 0.11 (rustls-ring) + h3 0.0.8 + h3-quinn 0.0.10 + webpki-roots (the h3 arm's default trust store); rustls (ring) is a mandatory native dep (was native-e2e-gated optional) — the h3 arm's quinn TLS config + the unit/e2e server double |
| `security/` | `wrap()` builds an engine `TlsConfig` and runs `xray_tui_tls::client::connect` (both arms); `fingerprint.rs` (fp-id parser → `Fingerprint`, `WebPkiVerifier` builder + test CA), `reality.rs` (`HelloProvisionerChoice`, pbk/sid decoders) |
| `protocol/` | 20 protocol modules; `vless` + `vmess` + `trojan` + `hysteria2` implemented, rest `NotImplemented`. `vless/vision.rs` = the `xtls-rprx-vision` codec (padded camouflage frames, inner-TLS filter, Direct splice state machine); `vless/header.rs` carries the protobuf flow addon (the udp443 variant truncated to the first 16 bytes on the wire — xray `requestAddons.Flow[:16]`) + the command byte (0x03 Mux carries NO destination bytes); `vless/mux.rs` = the v1.mux.cool frame codec + `MuxClient` multiplexer (`[2B meta_len][metadata][2B data_len][payload]` frames, eager New, event-driven Keep/End + tunnel KeepAlive, 8 KiB chunks, concurrent TCP sessions + XUDP datagram sessions (`UdpSession` — network=UDP New frames carrying the tunnel's random 8-byte `GlobalID`, per-packet dests on Keep) over one `cmd 0x03` tunnel); `vless/udp.rs` + `vless/packet.rs` + `vless/packetaddr.rs` = the UDP path (cmd 0x02 raw tunnel with `[2B len][payload]` framing; `PacketConn` datagram API in `Raw`/`PacketAddr`/`XUdp` modes; packetaddr destination codec); `vless/encryption/` = the `mlkem768x25519plus` payload encryption (SP7 — ML-KEM-768 + X25519 PFS handshake, sealed record tunnel, native/xorpub/random modes, xor-mode masking per xray `xor.go`, ChaCha-only client sealing; 0-RTT resume omitted — 0rtt accounts run full 1-RTT) |
| `crypto/` | VMess-adjacent primitives (aead/kdf/legacy_stream) + `salamander.rs` (Hysteria2 Salamander packet obfuscation: salt + BLAKE2b-256 keyed-XOR, datagram transform) |
| `shape.rs` | `ConnectShape`: uniform vs divergent connect paths |
| `e2e/` (feature `native-e2e`) | case/config/core/harness/variant — real-core scenarios |

### xray-tui-tls (`crates/xray-tui-tls/src/`)

| Module | Responsibility |
|--------|----------------|
| `spec/` | declarative `ClientHelloSpec`/`ExtensionSpec`/`SessionIdSpec`, RFC 6066/8446 wire encodings, GREASE (RFC 8701), hybrid key-share encoding (`X25519MLKEM768`: client share = ML-KEM ek(1184) ‖ X25519 pub(32)) |
| `profiles/` | two-tier roster: the **hand tier** (`hand_selected.rs`, `spec!`-declared) = 2 wire-exact profiles, `chrome_130` + `edge_106` (transcribed byte-for-byte from the deleted hand modules — the only byte-level-wire-faithful profiles) + the **generated tier** (`generated/`, emitted by `gen_specs.py --emit`) = 69 JA4-faithful entries, the `select_roster` kept subset of the 1825-entry manifest; the resolver merges both tiers (hand bands win on identity overlap) — offline gate + live peet.ws sweep pin the fidelity contract (`tests/generated_ja4_gate.rs`, `docs/tls-fingerprint-roster.md`) |
| `fingerprints/` | identity selector: `Fingerprint { browser, version?, os?, device? }` → **next-modern** table resolution over the 71-row two-tier table (smallest kept major `>= v` within the os/device-compatible group; above-newest / below-oldest refuse) with **cross-triple os-drop** fallback (exact triple miss → retry os-dropped — desktop hellos are OS-independent within a family); never a different browser, never older than requested; unknown combos error listing what IS resolvable; `FingerprintBuilder` overrides (ciphers/extensions/curves/ALPN/signature-algs, `GreasePolicy::Keep|Strip`); generated JA4 catalog (`catalog/catalog_data.rs`, from the frozen ja4db-export snapshot 2026-05-15 via ua-parser — rerun `gen.py`, never hand-edit) as evidence (`Resolved::in_catalog`); full-JA4 oracle in `crypto/fingerprint/ja4.rs` (final FoxIO scheme, peet.ws-validated) |
| `client/` | unified engine API: `TlsConfig { mode, server_name, alpn, rng }` + `TlsMode::{Plain, Reality}` + one `connect(stream, &TlsConfig)` entry |
| `hello/` | `build_hello`/`to_record` (GREASE pairing, 512-byte record padding), `parse_hello` |
| `crypto/` | key schedule (RFC 8448-verified; hybrid input = `pq ‖ classical` shared secrets), AEAD record keys (IV XOR seq), TLS 1.2 key block (`tls12.rs` — X25519 ECDHE + AES-GCM/ChaCha20-Poly1305 explicit-nonce AEAD via `seal_with_nonce`/`open_with_nonce`), `X25519KeyPair` (low-order peer points refused per RFC 7748 §6.1), `mlkem.rs` ML-KEM-768 primitives via liboqs (`oqs`, vendored — pk 1184 / sk 2400 / ct 1088 / ss 32), `fingerprint/` JA3 + JA4 encoders |
| `record/` | record framing, `read_record`, `TlsStream<S>` (AsyncRead/Write, close_notify→EOF; per-direction direct mode `set_write_direct`/`set_read_direct` — raw record-layer bypass that hands the socket to the tunnel, backing the vision Direct splice). TLS 1.2 record protection (`aead_aad_12` plaintext-length AAD, explicit nonce, one write counter advanced only after a successful seal, `AppKeys` deliberately not `Clone`; a plaintext CCS after the handshake is `unexpected_message`, not a skip) |
| `handshake/` | TLS 1.3 client handshake + TLS 1.2 fallback driver (`handshake/tls12.rs`, ECDHE + AEAD — reached from the shared `drive()` on a 1.2 ServerHello; REALITY over 1.2 surfaces `RealityFallback`), `ServerVerifier` seam, multi-record flight reassembly; one shared `drive()` for plain + REALITY; hybrid-curve key exchange (curve 4588 selected → decapsulate the ServerHello's 1088-B ML-KEM ciphertext, feed `pq ‖ classical` to the key schedule) |
| `verify/` | `WebPkiVerifier` (roots/CA DER/`insecure`/`pin_sha256`; CV signature always checked — the server sigalg is checked against the ClientHello offer on both versions; TLS 1.2 ECDSA schemes name no curve (RFC 8422 §5.1.3), so one candidate per curve is tried) |
| `reality/` | `HelloProvisioner` + `SpecProvisioner` (`From<&Fingerprint>` — any resolvable identity over the 71-row two-tier table) + 9-step wire contract, `FixedChrome133` (the surviving wire-exact chrome_130 spec + the X25519MLKEM768 hybrid share — the Chrome-133 hand profile was dropped in the roster reduction), auth-key/session-seal/server-auth, REALITY over the `X25519MLKEM768` share (curve 4588 — xray `reality.go:79` / sing-box `reality_client.go:136`), `SpiderConfig` + `spider.rs` (Spider-X h2 fallback) |
| `http2/` | minimal h2 layer (tls.peet.ws grading + Spider-X fallback GETs) |
| `error.rs` | `TlsError`/`Result` (thiserror) |

## Security layer capabilities

Every TLS/REALITY connect is engine-only: `security::wrap` builds an engine
`TlsConfig` from the profile's proto security config and runs
`xray_tui_tls::client::connect` (`TlsMode::Plain` | `TlsMode::Reality`). The
rustls client path and the `TlsProvider` plug are gone.

| Path | Trigger | Mechanism | Status |
|------|---------|-----------|--------|
| Plain TLS | `tls` config | engine `TlsMode::Plain`: fingerprint-shaped hello from the `fp` profile (`None` → `Fingerprint::default_for(Browser::Chrome)` = the newest kept chrome row), `WebPkiVerifier` via `verifier_for(insecure, pin)`; identity = `parse_fingerprint_id(fp)` + `profile_for` (exact ids: `chrome`/`chrome-randomized`/`random` → Chrome v130 — the hand `chrome_130` band; `firefox` → Firefox v128 (Linux); `safari` → Safari v17 (macOS); unknown → config error) | ✅ |
| REALITY | `reality` config | engine `TlsMode::Reality`: fingerprint-shaped hello with any resolvable identity via `SpecProvisioner::from(&Fingerprint)` (or a custom `HelloProvisioner`), sealed session id, X25519 auth key + HMAC/Ed25519 server auth (no PKI); Spider-X fallback on auth failure | ✅ |
| Trust modes | `insecure` / `pin_sha256` | `with_insecure()` skips chain walk; `with_pin(sha256(SPKI))` replaces chain+SAN but **never** skips the CertificateVerify signature (a MITM must hold the private key) | ✅ |

The unified API: `TlsConfig { mode: TlsMode, server_name, alpn, rng }` with a
single entry point `connect(stream, &TlsConfig)` — `TlsMode::Plain {
fingerprint, verifier }` and `TlsMode::Reality { provisioner, public_key,
short_id, spider }`. REALITY is a security layer over the same TLS machinery:
plain and REALITY handshakes share one driver (`handshake::drive`).

**Spider-X fallback.** On a REALITY auth failure — the server flight is a real
certificate (a transparent proxy / possible MITM), mirroring xray-core
`reality.go`'s `!Verified` path — the client keeps the established TLS session
alive instead of tearing it down: it walks the real site with bounded HTTP/2
GETs (`SpiderConfig { paths, max_gets, request_interval }`; padding cookie +
Referer chaining) so a DPI observer sees browsing traffic, then reports
`TlsError::RealityFallback` → native `NativeError::Reality("REALITY: received
real certificate (potential MITM or redirection)")`.

**Post-quantum key exchange (SP7).** The engine speaks the `X25519MLKEM768`
hybrid group (curve 4588): the ClientHello key share carries the ML-KEM-768
encapsulation key (1184 B) beside the X25519 public key; on a hybrid
ServerHello the client decapsulates the server's 1088-B ciphertext and feeds
`pq ‖ classical` to the TLS 1.3 key schedule — the Go wire order (xray
`handshake_client_tls13.go`/`handshake_server_tls13.go`; an earlier
classical-first draft was wrong and is fixed everywhere: encode, parse,
IKM order, fakes, goldens). ML-KEM-768 primitives come from liboqs via the
`oqs` crate (vendored build — no system liboqs; FIPS 203, wire-compatible
with Go's `crypto/mlkem`). `SecP256r1MLKEM768` (4587) /
`SecP384r1MLKEM1024` (4589) are parsed but rejected at handshake time — the
engine has no P-256/P-384 ECDH (explicit error, not a silent classical
fallback). REALITY accepts the 4588 share per xray `reality.go:79` /
sing-box `reality_client.go:136`. The fixed-chrome REALITY provisioner's PQ
fingerprint (`fixed_chrome_spec` = the chrome_130 hand spec + the
X25519MLKEM768 share: supported_groups GREASE + X25519MLKEM768 +
X25519/P256/P384, hybrid key share) now works end-to-end.

## E2E coverage (tier 3)

The suite has eleven subsections. **Transport matrix** (`tests/vless.rs` + `tests/vmess.rs`): every
VLESS/VMess case × TCP/WS/gRPC/HTTPUpgrade/XHTTP/h2/KCP/XHTTP-h3(QUIC) × serving core(s) — 136
tests = 130 green + 6 documented ignored (vless 78+6, vmess 52; ignored: the 4 ws/grpc
plain-into-reality-server semantic rows × both cores + reality-pq + pq-enc (ML-KEM axis,
below); single-core rows run only on the serving core: xhttp + xhttp-h3 + kcp + pq-enc on
xray, v2rayhttp + ws/httpupgrade-reality + mux-vision + vision-udp443 on sing-box). **TLS-variant cases**, each run against both cores (xray
26.3.27, sing-box 1.13.16), each
spawning a real server inbound + dialing it with the native client. The
two-servers scenarios are rows 4-6: the REALITY server's `dest` is a second
local server (`tls_echo`), so a fallback or a transparently-proxied plain
probe terminates there and the recording server observes the client's bytes
(spider h2 preface / plain-TLS ClientHello). Row 7 (`plain-server-reality-client`)
is a single-server scenario — the fallback terminates at the plain TLS server
itself, so the spider's bytes never reach a dest (`spider_reaches_dest()`
false). Rows 1-3, 6 and 8-11 probe HTTP through an established tunnel; the
two fallback cases (rows 4-5, wrong pbk / wrong sid) expect `connect()` to fail
with the fallback error and skip the probe.

**Trojan axis** (`tests/trojan.rs`): 12 rows — the TCP-stream trojan (password header + raw relay) ×
network {tcp, ws, grpc} × TLS {tls-standard, tls-chrome fingerprint} × core {xray, sing-box}. The
wire protocol is identical on either core; the transport + security ride the uniform pipeline the
vless/vmess rows exercise.

**Hysteria2 axis** (`tests/hysteria2.rs`): 2 rows — the QUIC-family hysteria2 (\`ConnectShape::Quic\`,
fresh quinn dial + ALPN \`h3\` + HTTP/3 auth) against a sing-box hysteria2 inbound (xray-core has no
hysteria2 server): {default cert-TLS} × {salamander-obfs}. QUIC TLS is rustls-internal, so no
fingerprint row exists. TCP tunnels use raw QUIC streams + the \`TCPRequest\`/\`TCPResponse\` frames;
hysteria2 UDP rows (SP1 gap-closure): the QUIC DATAGRAM relay (UDPMessage framing + fragmentation) against a sing-box inbound.

**VLESS vision flow axis** (spec §7.4): 8 rows — `flow = xtls-rprx-vision`,
tcp network, × core {xray, sing-box} × outer {tls, reality} × app {plain,
inner-tls}. The inner-tls rows drive the Direct-splice path (the app
establishes a real TLS 1.3 session through the tunnel to the rustls echo
target, then the vision filter switches both directions to a raw relay after
the Direct frame); the plain rows exercise the End (non-TLS inner traffic —
padding stops, outer TLS continues) path. UDP under the vision flows is
rejected on the RAW path only: the flow guard refuses UDP traffic under
`xtls-rprx-vision` over the direct `cmd 0x02` tunnel (mirrors xray's
UDP/443 rejection); UDP rides the mux tunnel instead (XUDP, SP3 — below).

**VLESS UDP path** (spec §4): 5 rows — `cmd 0x02` UDP command, `[2B BE
len][payload]` datagram framing in both directions over the tunnel stream
(`protocol/vless/udp.rs`), the `PacketConn` datagram API with the shared
response-header peel (`packet.rs`). 4 both-cores rows: Raw (header-dest)
mode × {tls-standard, reality} × {xray, sing-box}; 1 sing-box single-core
row: packetaddr mode over tls (xray-core's VLESS has no packetaddr
registration, verified against thirdparty/Xray-core). packetaddr mode
(sing-box-style): the header dest is the magic fqdn
`sp.packet-addr.v2fly.arpa` and each datagram's frame carries a per-packet
address header `atyp|addr|port` — no magic inside the frame
(`packetaddr.rs`, verified against sing-box 1.13.16).

**VLESS mux axis** (spec §5): 5 rows — `cmd 0x03` Mux command to the fixed
`v1.mux.cool:9527` tunnel destination (the header carries NO destination
bytes — the no-addr rule, mirroring xray's `EncodeRequestHeader`), then
the `MuxClient` multiplexes concurrent TCP sessions over the one tunnel
(v1.mux.cool frames `[2B meta_len][metadata][2B data_len][payload]`, eager
`New`, event-driven `Keep`/`End` frames, tunnel-level `KeepAlive`, 8 KiB chunks). 4
both-cores rows: {tls, reality} × {xray, sing-box}, each tunnel carrying
4 concurrent sessions to the echo target with every response asserted;
1 sing-box single-core row: vision+mux (`xtls-rprx-vision` flow addon +
camouflage frame, mux frames riding the vision-padded stream — the
response-header peel sits INSIDE the vision codec, mirroring xray
`outbound.go`'s getResponse-before-VisionReader composition). xray-core
rejects vision+mux TCP by server design (its vision+mux path is the XUDP
route — `AllowedNetwork = UDP`), so the vision+mux row is sing-box only;
the row comment documents the xray semantics.

**VLESS XUDP axis** (spec §7.3): 5 rows — UDP over the mux tunnel
(`connect_udp` dispatches XUDP on `params.mux`): one `cmd 0x03` tunnel +
one UDP session (`UdpSession`, `open_udp_session`) carrying a fresh random
8-byte `GlobalID` per tunnel (written on the New frame only — xray
`frame.go` `WriteTo`), each datagram a `Keep` frame with its own
per-packet destination (network byte UDP + port-first addr), wrapped in
the `PacketMode::XUdp` `PacketConn`; echoes matched by payload,
order-independent. All 5 rows run both cores: {tls, reality} × {xray,
sing-box} for plain XUDP, plus the `xtls-rprx-vision-udp443` flow — the
client config carries the full name (selects the XUDP path client-side:
mux-forced, guard lifted), the wire addon truncates to `xtls-rprx-vision`
(`requestAddons.Flow[:16]`) and the server config emits that truncated
name (`Flow::server_str` — both cores validate the request addon against
it). xray serves vision+mux+UDP by design (mux under flow XRV is the XUDP
path: `AllowedNetwork = UDP` — inbound.go; the SP2 vision+mux TCP
rejection does not apply to UDP), and sing-box terminates the same XUDP
session — the xray arm was verified in the final-review fix wave.

**VLESS mKCP axis** (spec §7.3): 2 rows — the mKCP transport (xray's KCP
fork over UDP; `transport/kcp/` — wire codec `{wire,session,stream}.rs`:
Data `[conv u16][cmd][opt][ts u32][sn u32][una u32][len u16][payload]`
18B overhead, Ack, CmdOnly Ping/Terminate; one segment per UDP datagram;
conv from a process-global counter, xray `dice.RollUint16()` equivalent).
xray-single-core rows (sing-box has no kcp): `kcp_plain` — the suite's
first genuinely no-TLS row (server `streamSettings` without `tlsSettings`,
client security none) — and `kcp_chrome` (tls, chrome fingerprint engine);
the two composition arms of `dial kcp → security wrap → protocol`. The
kcp+reality arm is dropped (spec deviation 5): xray-core 26.3.27's
`StreamConfig.Build()` rejects `security: "reality"` over any protocol but
tcp/splithttp/grpc — "REALITY only supports RAW, XHTTP and gRPC for now."
— so no core can serve reality-over-kcp (the native client's
reality-over-kcp dial stays implemented, unreachable server-side exactly
like xray's own client).

**VLESS ML-KEM PQ axis** (SP7, spec §7.3): 3 vless rows + 1 vmess row.
`tcp_tls_pq` (both cores — sing-box 1.13.16 accepts `curve_preferences`,
verified empirically; the spec's "sing-box 1.18+" note is stale): the
hybrid curve is pinned on BOTH ends — the client offers only the
X25519MLKEM768 key share (`curves: "x25519mlkem768"`) and the server's
`tlsSettings.curvePreferences` accepts nothing else, so a green row is a
negotiated ML-KEM-768 exchange, never a classical fallback; the runner
additionally asserts the engine's `negotiated_hybrid()` flag (set from the
ServerHello ML-KEM ciphertext). `tcp_reality_pq` IGNORED — blocked by the
harness dest, not the client: xray REALITY replays the DEST's ServerHello
flight as camouflage (`xtls/reality tls.go s2cSaved`, group check :359),
and the rustls `tls_echo` dest has no PQ, so the tunnel is structurally
classical regardless of the client's hybrid offer; a PQ reality row needs a
PQ-capable dest (Go 1.24+ / OpenSSL 3.5 echo) — the client's REALITY 4588
support is proven hermetically (T4 fake REALITY PQ server with real
encapsulation + HMAC-stamped Ed25519 cert + encrypted echo).
`tcp_pq_enc` IGNORED — the VLESS `mlkem768x25519plus` account encryption
end to end (client outbound `encryption` = server's PUBLIC halves
(X25519 pub + ML-KEM ek); xray inbound `settings.decryption` = PRIVATE
halves (X25519 priv + 64-B seed), bridged by `keypair_from_seed` (FIPS 203
d‖z derand)): against real xray 26.3.27 the client's PQ handshake COMPLETES
(connect Ok, request sealed + written) but the tunnel EOFs before the
response header with zero server logs — an unresolved interop divergence
(the native impl mirrors xray `encryption/client.go` wire and passes the
T3 hermetic double); left ignored with the harness wired (`XRAY_TUI_CORE_LOG`
gate) for a dedicated fix round. pq-enc is xray-single-core in any case
(sing-box has no VLESS account encryption). The vmess
`tcp_aes128gcm_tls_pq` row mirrors `tcp_tls_pq` on both cores.

**VLESS XHTTP/3 axis** (spec §7.3): 1 row — `xhttp_h3_tls`
(`vless("xhttp3")`, xray single-core — sing-box has no xhttp-over-QUIC): a
single exactly-one `h3` ALPN flips the xray splithttp listener to QUIC +
HTTP/3 (`hub.go` `isH3`), and the client's `connect_quic` (quinn Endpoint
over UDP, ALPN `h3`) runs the splithttp v3 protocol over HTTP/3 — the v3
session/GET/POST logic shared with the h1/h2 arms via the `V3Send` seam,
h3 requests in absolute-URI form (`:scheme`/`:authority`, RFC 9114 §4.3.1 —
the interop fix), webpki-roots default verify with the harness CA as trust
anchor (`StandardTls::client_trust` + `set_test_ca`). One row: reality is
impossible (`decideHTTPVersion` returns h2 when a reality config is
present), h3 requires TLS (no plain row), and the engine TLS is not
involved in QUIC (quinn/rustls internal — no fingerprint variant, the row
is tls-standard).

| Case | Payload security | TLS variant |
|------|------------------|-------------|
| VLESS | — | tls-standard |
| VLESS | — | tls-chrome (fingerprint engine) |
| VLESS | — | reality |
| VLESS | — | reality-wrong-pbk (client holds a wrong pbk → server proxies → Spider-X → fallback) |
| VLESS | — | reality-wrong-sid (client sid mismatch → server proxies → Spider-X → fallback) |
| VLESS | — | reality-server-plain-client (plain fingerprint probe through a REALITY server → transparently proxied to dest, stealth) |
| VLESS | — | plain-server-reality-client (REALITY client against a cert-TLS server → Spider-X → fallback) |
| VMess | aes-128-gcm | tls-standard |
| VMess | chacha20-poly1305 | tls-standard |
| VMess | aes-128-gcm | tls-firefox (fingerprint engine) |
| VMess | aes-128-gcm | reality |

## Protocol roadmap

Legend (emoji = phase/decision per column):

| Mark | Meaning |
|------|---------|
| ✅ | done, verified |
| 🚧 | in progress |
| 📋 | todo (planned) |
| ⏸ | deferred (planned but not scheduled) |
| ⛔ | won't implement (decision) |
| 🔒 | blocked on a prerequisite |

Capability columns compare what the mainstream repositories (xray-core,
sing-box — `thirdparty/`) provide for each protocol. "Native client" is the
xray-tui-native tunnel; "TLS engine" is the xray-tui-tls path — engine-only
now, every TLS/REALITY connect routes through `xray_tui_tls::client::connect`
(the column marks whether the protocol routes through it); "REALITY" marks
REALITY compatibility (any resolvable identity via
`SpecProvisioner`); "e2e" is tier-3 proof.

### Overview matrix

| Protocol | xray-core | sing-box | Native client | TLS engine | REALITY | e2e |
|----------|:---------:|:--------:|:-------------:|:----------:|:-------:|:---:|
| VLESS | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| VMess | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Trojan | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Shadowsocks | ✅ | ✅ | 📋 | — | — | 📋 |
| ShadowsocksR | ⛔ | ✅ | 📋 | — | — | 📋 |
| SOCKS | ✅ | ✅ | 📋 | — | — | 📋 |
| HTTP | ✅ | ✅ | 📋 | — | — | 📋 |
| WireGuard | ✅ | ✅ | 📋 | — | — | 📋 |
| Hysteria2 | ✅ | ✅ | ✅ | 🔒 QUIC | — | ✅ |
| Hysteria1 | ⛔ | ✅ | 📋 | 🔒 QUIC | — | 📋 |
| TUIC | ⛔ | ✅ | 📋 | 🔒 QUIC | — | 📋 |
| Naïve | ⛔ | ✅ | 📋 | ✅ | — | 📋 |
| AnyTLS | ⛔ | ✅ | 📋 | ✅ | — | 📋 |
| ShadowTLS | ⛔ | ✅ | 📋 | ✅ | — | 📋 |
| Tor | ⛔ | ✅ | 📋 | — | — | 📋 |
| SSH | ⛔ | ✅ | 📋 | — | — | 📋 |
| Tailscale | ⛔ | ✅ | 📋 | — | — | 📋 |
| Redirect | ⛔ | ✅ | 📋 | — | — | 📋 |
| TProxy | ⛔ | ✅ | 📋 | — | — | 📋 |
| Mixed | ⛔ | ✅ | 📋 | — | — | 📋 |

Notes on the matrix:
- **TLS engine / REALITY columns** are "✅" only for the TCP-stream family
  where the security phase applies. QUIC-family protocols (Hysteria1/2, TUIC)
  carry TLS inside QUIC, so the engine's security phase never runs — 🔒. That
  is orthogonal to the native column: Hysteria2 is native-complete because its
  quinn dial owns TLS internally; Hysteria1 + TUIC still need a QUIC client.
- **TLS 1.2**: the engine has a TLS 1.2 fallback path (ECDHE + AEAD only),
  reached on a 1.2 ServerHello — legacy TLS 1.2-only servers are reachable.
  REALITY stays 1.3-only (a 1.2 ServerHello → `RealityFallback` → Spider-X).
- **Shadowsocks** gets no TLS column (plain TCP + AEAD; obfuscation comes from
  plugins, which are 📋 transport work). `ss` method whitelists live in
  `proto_spec/core_mapping.rs`.
- **Redirect/TProxy/Mixed** are outbound-only kinds (kernel/port forwarding):
  no client handshake of their own — the native `connect` is trivially a dial,
  but the `ConnectShape` path must exist.
- **Tor** is a full onion-routing stack (circuits, cell crypto, directory
  protocol); realistically ⛔ for the in-process core — the subprocess path
  stays the answer.
- **SSH/Tailscale/WireGuard** are device-tunnel or own-handshake shapes —
  divergent `connect()` paths, not the uniform pipeline.

### Per-protocol detail

**VLESS** — ✅ native
| Capability | Detail |
|------------|--------|
| Encryption | none (identity = UUID; optional `xtls-rprx-vision` flow control, TLS 1.3 framing). Account encryption `mlkem768x25519plus` ✅ (SP7): `mlkem768x25519plus.<native|xorpub|random>.<seconds>[s].<padding>` (proto parser mirrors xray `infra/conf/vless.go`), ML-KEM-768 + X25519 PFS handshake wrapping the session before the request header, sealed-record tunnel (`protocol/vless/encryption/`); client always seals ChaCha20-Poly1305 (server accepts either); 0-RTT ticket resume omitted |
| Auth | UUID (command bytes in header) |
| Obfuscation | none at protocol level; REALITY supplies traffic camouflage |
| Transports | TCP, WS, gRPC, h2, QUIC (xray-core); TCP/WS/gRPC/h2/QUIC (sing-box) — native: TCP/WS/gRPC/HTTPUpgrade/XHTTP/h2/KCP ✅ + XHTTP-h3 (QUIC) ✅ (SP5 — the xhttp h3 dial, see Transport roadmap; XHTTP stream-one ✅ (SP6 — legacy v1 dialect, single full-duplex request, no session id; reality auto-default selects stream-one)). KCP = mKCP over UDP, xray-only (sing-box has no kcp); e2e plain + tls rows. Vision requires raw TCP — the Direct handoff needs the socket (ws/grpc/xhttp framing is incompatible).
| UDP | `cmd 0x02` UDP command path ✅ — `[2B BE len][payload]` datagram framing both directions over the tunnel stream (`protocol/vless/udp.rs`), `PacketConn` datagram API with the response-header peel (`packet.rs`), packetaddr mode (`packetaddr.rs`; sing-box-style: header dest = magic fqdn `sp.packet-addr.v2fly.arpa`, per-packet frame header `atyp|addr|port`, no magic in the frame). XUDP ✅ (SP3) — UDP over the mux tunnel (`PacketConn::xudp` over `UdpSession`, per-packet dests + the tunnel's 8-byte `GlobalID`; see Mux). e2e 10 rows: Raw × {tls-standard, reality} × both cores + packetaddr/tls/sing-box (xray has no packetaddr registration) + XUDP × {tls-standard, reality} × both cores + vision-udp443/sing-box. Vision+UDP rejected on the RAW path only — the flow guard refuses UDP under the vision flows over the direct cmd-0x02 tunnel (mirrors xray's UDP/443 rejection); under the mux path the rejection lifts (XUDP). |
| Flow | `xtls-rprx-vision` ✅ — padded camouflage frames + inner-TLS filter + Direct splice state machine (`protocol/vless/vision.rs`, protobuf flow addon in the request header). TCP transport only; UDP app traffic under the vision flows rides the mux tunnel (XUDP, SP3) — the raw-path guard rejects UDP under vision (mirrors xray's UDP/443 rejection). `xtls-rprx-vision-udp443` ✅ (SP3) — the client config carries the full name (selects the XUDP path: mux-forced, guard lifted), the wire addon truncates to the first 16 bytes (`xtls-rprx-vision`, xray `requestAddons.Flow[:16]`), and the server validates against that truncated name. Requires outer TLS1.3/REALITY over raw TCP (guards in `connect_vision` mirror xray's rejection). Inner TLS1.3 → `Direct` raw splice — both directions abandon the outer TLS after the Direct frame (the Direct frame is the last outer-TLS record); non-1.3 inner traffic → `End`, padding stops, outer TLS continues. Deviations (spec §9): no 500 ms camouflage timer (the empty Continue long-padding frame is emitted immediately after the header — same wire bytes, deterministic), per-direction direct flags (`TlsStream::set_write_direct`/`set_read_direct`) instead of Go's unsafe `tls.Conn` reflection. |
| Mux | `cmd 0x03` v1.mux.cool multiplexer ✅ — one tunnel (fixed `v1.mux.cool:9527` header destination, NO destination bytes on the wire — the no-addr rule) carrying concurrent TCP sessions + XUDP datagram sessions: `MuxClient` (demux + writer + keepalive tasks, eager `New`, event-driven `Keep`/`End` frames, tunnel-level `KeepAlive` every 10 s, 8 KiB chunks) + `SessionStream` app streams + `UdpSession` (XUDP, SP3: `open_udp_session` with a fresh random 8-byte `GlobalID` — network=UDP New frame — then `Keep` frames carrying each packet's own destination; wrapped by `PacketConn::xudp`) (`protocol/vless/mux.rs`). Vision+mux composition ✅ — peel-inside (response header peeled before the vision codec, mirroring xray `outbound.go`), mux frames ride the vision-padded stream; sing-box server only (xray rejects vision+mux TCP by server design — its vision+mux is the XUDP path). e2e 10 rows: mux-tls + mux-reality × both cores, mux-vision/sing-box, xudp-tls + xudp-reality × both cores, vision-udp443/sing-box. |
| Status | Native client complete + e2e (tls-standard, tls-chrome, reality, vision-tls, vision-reality, udp-raw, udp-packetaddr, mux-tls, mux-reality, mux-vision, xudp-tls, xudp-reality, vision-udp443, kcp-plain, kcp-chrome, xhttp-h3-tls, xhttp-stream-one-tls, xhttp-stream-one-reality) × both cores where the transport allows (kcp + xhttp-h3 rows xray-only), full transport matrix e2e (136-test sweep = 130 green + 6 documented ignored: vless 78+6, vmess 52). Deferred: HTTPUpgrade `ed` early-data, h2 PING keepalive, xmux (mux v2) / connection-reuse pooling, browser-masquerade header set, and the general QUIC client transport for Hysteria1/2 + TUIC (quinn landed for xhttp h3, SP5).

**VMess** — ✅ native
| Capability | Detail |
|------------|--------|
| Encryption | payload: `aes-128-gcm`, `chacha20-poly1305` (native ✅); legacy `aes-128-cfb` / `none` (xray-only, ⛔ legacy disabled in modern cores) |
| Auth | AEAD request header (mandatory), MD5/HMAC-SHA256 KDF chain, FNV-1a + CRC-32 frame checks |
| Obfuscation | none at protocol level (transport-level ws/grpc/http in xray) |
| Transports | TCP, WS, gRPC, h2, QUIC (xray-core); TCP/WS/gRPC/h2 (sing-box) — native: TCP/WS/gRPC/HTTPUpgrade/XHTTP/h2/KCP ✅ + XHTTP-h3 via the shared xhttp h3 dial (e2e covered by the vless row; XHTTP stream-one ✅ (SP6))
| UDP | `cmd 0x02` AEAD-record datagram tunnel ✅ (`protocol/vmess/udp.rs`) — the request header carries command 0x02, each datagram is one AEAD record (one `write` = one record = one datagram, sent through the cancel-safe `VmessClientStream::write_datagram` — never the byte-stream `write_all`, whose dropped-future state would truncate the next datagram). No per-packet address: the header destination is the session target, so `send(None, …)` is the normal call and a `Some(dest)` that differs from it is refused (`InvalidInput`). Send payloads are capped at xray's chunk size (`8192 - 2 - 16`; above it the server's reader re-splits the chunk and the datagram boundary is lost), reads accept the full u16 record. e2e 1 row: `tcp_aes128gcm_udp` × both cores.
| Status | Native client complete + e2e (aes-128-gcm, chacha20-poly1305, tls-firefox, reality, tls-pq, udp) × both cores, full transport matrix e2e (kcp rows run via the shared transport; e2e coverage is the vless kcp rows; the SP7 tls-pq row pins X25519MLKEM768 on both ends and asserts negotiated-hybrid). Legacy ciphers ⛔. Deferred: HTTPUpgrade `ed` early-data, h2 PING keepalive, xmux/reuse pooling, browser-masquerade header set, v2rayhttp no-TLS h1 arm, the general QUIC client transport (quinn landed for xhttp h3, SP5). |

**Trojan** — ✅ native
| Capability | Detail |
|------------|--------|
| Encryption | none — plaintext after TLS; security is the TLS layer |
| Auth | password: 56-byte lowercase `hex(sha224(password))` in the request header (`protocol/trojan/auth_key`, xray `hexSha224`) |
| Obfuscation | none (REALITY can carry it via the engine) |
| Transports | TCP, WS, gRPC, h2, QUIC (xray-core); TCP/WS/gRPC/h2/QUIC (sing-box) — native: TCP/WS/gRPC/HTTPUpgrade ride the uniform pipeline ✅ |
| Wire | request `key(56) || CRLF || command(1) || ATYP+addr+port-BE2 (port-LAST — trojan's `NewAddressParser` has no `PortFirst`, sing-box's `SocksaddrSerializer` is port-last too) || CRLF`; **no server response header** (xray + sing-box relay raw after the request) — the tunnel is pure passthrough |
| UDP | command 3 (`cmd 0x03`) per-datagram `[addr][2B len][CRLF][payload]` ✅ (`protocol/trojan/mod.rs` `PacketConn`) — the request header carries command 3 + the session target (port-last); each datagram is an address-prefixed frame (`ATYP|addr|port || len || CRLF || payload`), the per-packet address defaulting to the session target (xray `PacketWriter`/sing-box `WritePacket`). Send payload capped at 8192 (xray `maxLength`; sing-box reads the full u16, so we follow the permissive peer on read). A domain-addressed reply frame — sing-box's default NAT domain-unmapping for FQDN destinations — is delivered with no per-packet address instead of failing the session. e2e 1 row: `udp` × both cores.
| Status | Native client complete + e2e (`tests/trojan.rs`): {tcp, ws, grpc} × {tls-standard, tls-chrome} × both cores + the UDP row. TLS engine path reuses the existing security stack exactly. |

**Shadowsocks** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | AEAD: `aes-128/256-gcm`, `chacha20-ietf-poly1305`, `xchacha20`, `2022-blake3-*` (both cores); legacy stream: `aes-*-cfb/ctr`, `rc4-md5`, `chacha20-ietf`, `none` (sing-box only) |
| Auth | none beyond AEAD key from password (2022 adds user lists) |
| Obfuscation | plugin: `simple-obfs`, `v2ray-plugin` (via URL `plugin` param) 📋 |
| Transports | TCP + UDP (same key); native UDP path 📋 |
| Status | Crypto primitives exist in `protocol/ss` + `crypto/` scaffolding; AEAD framing (2-byte length + tag) straightforward. Cipher-validity rules already in `core_mapping.rs`. |

**ShadowsocksR** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | stream ciphers (`aes-*-cfb/ctr`, `rc4-md5`, `chacha20`, …) |
| Auth | protocol plugins: `auth_chain_a/b`, `auth_aes128_*` |
| Obfuscation | obfs plugins: `http_simple`, `tls1.2_ticket_auth`, `plain` |
| Transports | TCP |
| Status | sing-box-only; obfs/auth plugin matrix is the bulk. 📋. |

**SOCKS** — ✅ native (client TCP) / ✅ native (inbound TCP + UDP)
| Capability | Detail |
|------------|--------|
| Encryption | none (SOCKS5) |
| Auth | optional username/password (RFC 1929), no-auth |
| Obfuscation | none |
| Transports | TCP; UDP ASSOCIATE served by the INBOUND, not the client |
| Status | Client handshake done (`protocol/socks`: greeting → method → RFC 1929 → CONNECT → reply, every step under `timeouts::PROTOCOL`); an over-long credential is an error, never a panic. The client's UDP side is deliberately absent: SOCKS5 datagrams ride a raw UDP socket to the proxy, not the (possibly TLS-wrapped) `BoxStream` this layer sits on. The local SOCKS5 inbound serves both TCP CONNECT and UDP ASSOCIATE. |

**HTTP** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | none (HTTP CONNECT) |
| Auth | Basic auth (optional) |
| Obfuscation | none |
| Transports | TCP |
| Status | Trivial CONNECT proxy. |

**WireGuard** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | Noise IK handshake (x25519 + ChaCha20Poly1305), data = ChaCha20Poly1305 |
| Auth | static/temporary public keys, PSK optional |
| Obfuscation | none |
| Transports | UDP device tunnel |
| Status | 🔒 needs TUN/device abstraction or userspace UDP socket tunnel. `ConnectShape` divergent. |

**Hysteria2** — ✅ native
| Capability | Detail |
|------------|--------|
| Encryption | TLS 1.3 inside QUIC (quinn/rustls internal — spec §5.2; ALPN `h3`, `insecure` honored, webpki-roots default, harness-CA override in test/e2e) |
| Auth | HTTP/3 POST `https://hysteria/auth` with `hysteria-auth` header (auth token); status **233** = OK (`protocol/hysteria2` `runtime_auth`) |
| Obfuscation | `salamander` obfs ✅ (`crypto/salamander.rs`: 8-byte random salt + payload XOR `blake2b-256(psk||salt)`; a `quinn::AsyncUdpSocket` wrapper (`SalamanderSocket`) transforms every QUIC datagram) |
| Transports | QUIC (`ConnectShape::Quic` — a fresh quinn dial replaces dial + security + upgrade; `protocol/hysteria2/quic.rs` shared endpoint/TLS/0-RTT helpers via `transport/quic.rs`). TCP tunnel = a RAW quinn bidi stream with `TCPRequest` frame (`varint 0x401 | addrLen | addr | padLen | padding`) + `TCPResponse` (status 0 = OK), then the `H2Stream` relay. **The h3 session must outlive the tunnel**: the server runs the h3 `StreamDispatcher` (quic-go `http3.Server`) over the SAME quinn connection that carried the auth, so dropping the last `SendRequest` clone after auth closes the session and tears the connection down (the `H3Keepalive` RAII guard in `H2Stream` holds it) |
| UDP | QUIC DATAGRAM relay ✅ (`protocol/hysteria2/udp.rs` `UdpConn`) — the auth response's `Hysteria-UDP` header gates it (server-disabled → connect error); each datagram is a `UDPMessage` (`sessionID u32 BE | packetID u16 BE | fragID u8 | fragCount u8 | addrLen varint | addr | payload`), fragmentation on `DatagramTooLarge` + per-session defrag (hysteria `frag.Defragger`). `send(None, …)` carries the session destination (every `UDPMessage` has an address on the wire); an undeliverable datagram — foreign session, malformed, incomplete fragment — is dropped and the loop keeps reading, as upstream does. The client sends its max receive bandwidth in the auth `hysteria-cc-rx` header (bytes/sec from the config `down` string — hysteria `AuthRequest.Rx`, sing-box `ReceiveBPS`). e2e 1 row: `udp` (sing-box single-core).
| Status | Native client complete + e2e (`tests/hysteria2.rs`, sing-box single-core — xray-core has no hysteria2 inbound): {default-tls, salamander-obfs} + the UDP row. No fingerprint row (QUIC ClientHellos are rustls-internal). |

**Hysteria1** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | none (protocol-level); TLS optional via QUIC |
| Auth | password |
| Obfuscation | `salamander` |
| Transports | QUIC + bandwidth control |
| Status | sing-box-only; 🔒 QUIC. |

**TUIC** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | TLS inside QUIC (v5: TLS 1.3, UUID auth in QUIC handshake) |
| Auth | UUID |
| Obfuscation | none |
| Transports | QUIC |
| Status | sing-box-only; 🔒 QUIC. |

**Naïve** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | TLS 1.3 (HTTP/2) — Chromium networking stack fingerprint required |
| Auth | Basic auth over H2 |
| Obfuscation | the entire point: looks like Chrome H2 to a CDN |
| Transports | TCP (h2) |
| Status | TLS engine ✅ (fingerprints cover Chrome); needs real HTTP/2 client (current `http2/` is grader-minimal, ⛔ not a client) → 🔒 on a full h2 implementation. |

**AnyTLS** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | none beyond TLS; padding obfuscation of the inner handshake |
| Auth | password (scrambled with the handshake) |
| Obfuscation | TLS-like padding/random extension injection |
| Transports | TCP |
| Status | TLS engine ✅; protocol layer simple once h2/record work exists. |

**ShadowTLS** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | none — shadowsocks-style relay over camouflaged TLS |
| Auth | password picked by SNI-specific key derivation (v3) |
| Obfuscation | TLS-in-TLS: outer TLS to a real site, inner plaintext relay |
| Transports | TCP |
| Status | TLS engine ✅ (outer hello mimicry is exactly what profiles do). |

**Tor** — ⛔ native
| Capability | Detail |
|------------|--------|
| Encryption | onion circuits: AES-128-CTR cells, RSA/Ed25519 handshakes, per-hop keys |
| Auth | circuit-level (no server password) |
| Obfuscation | pluggable transports (obfs4, …) |
| Transports | TCP + directory protocol |
| Status | Full Tor stack is a project of its own — keep the sing-box subprocess path. ⛔ in-process. |

**SSH** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | chacha20-poly1305 / aes-gcm / aes-ctr, host-key auth (ed25519/rsa) |
| Auth | username + password/key |
| Obfuscation | none |
| Transports | TCP (own handshake — `ConnectShape` divergent) |
| Status | sing-box uses SSH as proxy transport. A Rust SSH client crate could back it; otherwise 📋. |

**Tailscale** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | WireGuard-derived (x25519 + ChaCha20Poly1305), DERP relay fallback |
| Auth | tailnet key (coordination server) |
| Obfuscation | none |
| Transports | UDP device tunnel + DERP |
| Status | Needs the coordination protocol; realistically the tailscale-client crate or ⛔. 📋. |

**Redirect / TProxy / Mixed** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | none (kernel-level / port forwarding) |
| Auth | none |
| Obfuscation | none |
| Transports | n/a (outbound-only kinds) |
| Status | `PlaceholderConfig` in proto; native arm exists (NotImplemented). A redirect dial is ~20 lines once shapes land. |

## Transport roadmap

| Transport | Status | Notes |
|-----------|:------:|-------|
| TCP | ✅ | `transport/tcp.rs`; all e2e cases run over it |
| WS | ✅ | `transport/ws.rs`; tokio-tungstenite framing over the engine stream; v2ray Host/path/headers; e2e vs xray + sing-box (standard + chrome fp) |
| gRPC | ✅ | `transport/grpc.rs`; h2 framing over the engine stream, gun mode; `Hunk` protobuf (`0x0A` + varint) inside the 5-byte gRPC prefix; e2e vs xray + sing-box (standard + chrome fp) |
| HTTPUpgrade | ✅ | `transport/httpupgrade.rs`; hyper http1 conn + RFC 7230 101 upgrade over the engine stream, `Connection: Upgrade`/`Upgrade: websocket` echo validated; ALPN `http/1.1`; e2e vs xray + sing-box |
| XHTTP (splithttp) | ✅ | `transport/xhttp.rs`; v3 dialect (xray-only server): uuid session in path, GET-body download, raw POST uploads (seq + 30 ms pacing + `Referer` `x_padding`, ≤1 MB chunks); **packet-up + stream-up + stream-one**; h1 when no TLS, h2 over TLS, **h3 over QUIC when the TLS ALPN is exactly `h3`** (mirror of xray `decideHTTPVersion`: reality → h2, no TLS → h1.1, single `h3` ALPN → QUIC, 0/2+ ALPNs → h2). **Mode auto-rule (mirrors xray): `""`/`auto` → packet-up; reality present → stream-one; reality + download_settings → stream-up.** The h3 dial **replaces** dial+security+upgrade (`connect_quic` — quinn/rustls internal, webpki-roots default verify; the engine TLS is not involved in QUIC); the v3 protocol is shared across h1/h2/h3 via the `V3Send` seam, and h3 requests use absolute-URI form (`:scheme`/`:authority`, RFC 9114 §4.3.1 — the interop fix). e2e vs xray (single-core: xhttp-h3/tls, xhttp-stream-one/tls, xhttp-stream-one/reality).
| h2 (v2rayhttp) | ✅ | `transport/v2rayhttp.rs`; h2 single full-duplex PUT stream, `:authority` = config host else `www.example.com`; sing-box only (xray removed the h2 transport in 26.x); e2e vs sing-box (single-core) |
| QUIC | 🚧 | quinn + h3 landed (SP5) as the xhttp h3 dial **and** the hysteria2 client dial (`connect_quic`; shared endpoint/TLS/0-RTT helpers in `transport/quic.rs`); a general QUIC client transport for the Hysteria1 + TUIC clients remains 🔒 |
| KCP (mKCP) | ✅ | `transport/kcp/` — xray's KCP fork over UDP: wire codec + session (windows/retransmit/RTO/congestion) + stream; one segment per datagram, conv from a process-global counter; xray-only (sing-box has no kcp); e2e vs xray (plain no-TLS + chrome tls) |
| obfs plugins (SS) | 📋 | plugin URL param already parsed by proto |

### Transport deferrals (documented, no stub)

- **quic client transport** — the quinn+h3 stack landed (SP5) as the xhttp
  h3 dial, and hysteria2 rides the same helpers for its own self-contained
  dial (`protocol/hysteria2`, complete + e2e); a general QUIC client transport
  gating the Hysteria1 + TUIC clients is still deferred.
- **HTTPUpgrade `ed` early-data** — xray-only client flag; server sees a
  normal 101 exchange.
- **h2 PING keepalive** — `idle_timeout`/`ping_timeout` in `HttpConfig` not
  wired.
- **xmux (mux v2) / connection-reuse pooling** — the classic v1.mux.cool
  multiplexer is implemented (SP2, `protocol/vless/mux.rs`); xray's newer
  `xmux` dialect and cross-tunnel reuse pooling remain deferred.
- **Browser-masquerade default header set** — servers validate
  Host/path/padding only; functional correctness first, masquerade is a later
  DPI-polish step.
- **v2rayhttp no-TLS (h1 early-data) arm** — sing-box hijacks raw bytes with
  no Content-Length; hyper always frames request bodies, so chunked bytes
  would pollute the hijacked stream (not reproducible with hyper). Real-world
  use is TLS.

## Where this is headed

1. **Finish the TCP-stream protocol family** (Trojan, Shadowsocks, SOCKS, HTTP,
   AnyTLS, ShadowTLS, Naïve) — the TLS engine is done, the shapes are uniform.
2. **Non-TCP transports** — the TCP-stream transport set is done
   (WS/gRPC/HTTPUpgrade/XHTTP/h2), mKCP landed (SP4, UDP), and the quinn
   QUIC stack landed for the xhttp h3 dial (SP5) plus the hysteria2 client
   dial (complete + e2e); next is a general QUIC client transport, which
   unlocks Hysteria1 + TUIC.
3. **Wire in the TUI** ✅ (2026-09-03, `docs/native-core-integration.md`): no
   per-profile toggle and nothing native persisted — the connect-time gate
   prefers the in-process core for the four e2e-verified protocols
   (vless/vmess/trojan/hysteria2) via `capability::supported`, with loud
   downgrade to xray-core for deferred configs. `NativeCoreServer` (SOCKS5 +
   HTTP CONNECT inbounds, proxy-all engine, watch-shutdown) replaces the
   subprocess arm in `ops/connect.rs`; telemetry (log/traffic/trace) feeds the
   existing stats/logs screens plus the Native Activity tab.
4. **Routing + outbound-only kinds** via native — the SOCKS5 inbound
   (`inbound/`, accept → route → direct/block/proxy outbound) is done; the
   remaining piece is the outbound-only kinds (redirect dial for
   split-tunnel rules).
5. **TLS 1.2 CBC / static-RSA suites** — the engine's TLS 1.2 path is ECDHE +
   AEAD only; servers that require CBC encryption or RSA key exchange remain
   unreachable.
