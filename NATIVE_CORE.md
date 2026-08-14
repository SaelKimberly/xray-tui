# NATIVE_CORE.md — Native proxy core + TLS fingerprint engine

Status and roadmap for the two crates that remove the subprocess dependency for
the tunnel itself: **`xray-tui-native`** (in-process client-side protocol
implementations) and **`xray-tui-tls`** (the ring-based TLS 1.3 client that
backs the fingerprint/REALITY security paths).

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

Both crates are client-side only. Server-side protocol behavior remains the
job of xray-core / sing-box; the e2e harness runs real cores as servers to
prove wire compatibility.

**Roadmap: engine-only TLS.** The native client path is engine-only — every
TLS/REALITY connect runs through `xray-tui-tls`. The rustls *client* path was
removed; rustls remains only as the server-side test double (unit tests + the
e2e `tls_echo` dest). The engine is TLS 1.3-only: TLS 1.2 engine support is a
future task, so legacy (TLS 1.2-only) servers are not yet reachable.

## Principles

- **Xray composition order, one layer stack**: `dial → security → transport
  upgrade → protocol → tunnel` — TLS is OUTERMOST (a ws/grpc transport runs
  framing INSIDE the engine TLS session, matching xray/sing-box). Each phase
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
  REALITY must agree twice with the same scalar.
- **Explicit absence beats silent fallback.** Every protocol kind, transport,
  and security has a module and a dispatch arm. Unsupported combinations
  return `NativeError::NotImplemented` naming the missing feature — no stub
  that pretends to work.
- **Real verification, three tiers** (see below). Wire-format claims carry
  byte-level evidence: RFC 8448 vectors, JA3/JA4 goldens, Go VMess vectors,
  live tls.peet.ws grading, real-core e2e.
- **`xray-tui-proto` is never modified by these crates.** If a protocol config
  lacks a field the tunnel needs, that's a proto change evaluated separately.

## Verification tiers

| Tier | Gate | What runs | Evidence |
|------|------|-----------|----------|
| 1 — offline | `cargo test -p xray-tui-tls -p xray-tui-native --features native-e2e --lib` | unit: wire encodings, RFC 8448 key-schedule vectors, GREASE pairing, JA3/JA4 goldens, VMess Go byte-vectors, rustls-server interop (dev-dep), multi-record reassembly, Spider-X fallback, transport framing (httpupgrade header set, xhttp chunk/seq/pacing + padding, v2rayhttp method/authority), vision codec (padded frames, TLS filter, Direct splice) + hermetic fake-vision-server, vless UDP (packet framing, packetaddr codec, PacketConn, hermetic fake-UDP-server) | 304 lib tests (122 tls + 182 native; native incl. 37 vision-module tests (incl. 2 hermetic fake-vision-server) + 21 vless UDP tests (framing, packetaddr codec, PacketConn, 2 hermetic fake-UDP-server)) |
| 2 — live grader | `cargo run -p xray-tui-tls --example grader -- --profile <id>`; `cargo test -p xray-tui-tls --test tls_peet_ws -- --ignored` | ClientHello graded against tls.peet.ws | Chrome130 JA4 `t13d1516h2_8daaf6152771_f37e75b10bcc`; Firefox128ESR JA3 `361e0ca6ef1ca4dbe3a1d987722a1980` + JA4 `t13d1314h2_07be0c029dc8_46701d79520f` |
| 3 — real-core e2e | `XRAY_TUI_CORE_BIN_DIR=<dir> cargo test -p xray-tui-native --features native-e2e --test vless --test vmess` | native client against spawned xray-core (26.3.27) + sing-box (1.13.16) servers, transport + TLS-variant matrix, VLESS vision flow axis, VLESS UDP datagram path | 113 tests = 109 green + 4 documented ignored (vless ws/grpc plain-into-reality-server semantic rows × both cores; single-core rows: xhttp/xray, v2rayhttp/sing-box, ws+httpupgrade reality/sing-box) |

Tier 2 needs network; tier 3 needs the version-pinned core binaries (hard-fail,
not skip, on version mismatch). Tier 1 is hermetic and is the CI gate.

## Crate maps

### xray-tui-native (`crates/xray-tui-native/src/`)

| Module | Responsibility |
|--------|----------------|
| `lib.rs` | `Stream`/`BoxStream` seam, `connect()` entry, `NativeTunnel` |
| `chain.rs` | `connect_chain`: fold the layer stack |
| `context.rs` | `LinkContext`, `NativeConnectParams` (wraps proto types) |
| `addr.rs` | `TargetAddr` (domain/IP + port) encode/decode |
| `transport/` | `connect` = TCP dial (any transport; framing is an upgrade step); `upgrade` = ws (tokio-tungstenite over the engine stream, v2ray Host/path/headers, Binary framing) + grpc (h2 over the engine stream, gun mode, `Hunk` protobuf + 5-byte gRPC prefix, deferred response headers via spawned task, write-through with flow-control reserve) + httpupgrade (hyper http1 conn + RFC 7230 101 upgrade: `GET {path}`, `Connection: Upgrade` + `Upgrade: websocket` echo validated, ALPN `http/1.1`) + xhttp (splithttp v3, xray-only server: uuid session in path, GET-body download, raw POST uploads with `seq` + 30 ms pacing + `Referer` `x_padding`, ≤1 MB chunks; packet-up + stream-up; h1 when no TLS, h2 over TLS) + v2rayhttp (h2 single full-duplex PUT stream, `:authority` = config host else `www.example.com`; sing-box only). HTTP framing (requests/responses/chunked/101) is hyper 1.11 (`client`+`http1`+`http2`) + hyper-util 0.1.20 (`tokio`) + http-body-util 0.1.5 (`channel`) — we own the byte stream, the dial, and the timeouts |
| `security/` | `wrap()` builds an engine `TlsConfig` and runs `xray_tui_tls::client::connect` (both arms); `fingerprint.rs` (fp-id parser → `BrowserProfile`, `WebPkiVerifier` builder + test CA), `reality.rs` (`HelloProvisionerChoice`, pbk/sid decoders) |
| `protocol/` | 20 protocol modules; only `vless` + `vmess` implemented, rest `NotImplemented`. `vless/vision.rs` = the `xtls-rprx-vision` codec (padded camouflage frames, inner-TLS filter, Direct splice state machine); `vless/header.rs` carries the protobuf flow addon; `vless/udp.rs` + `vless/packet.rs` + `vless/packetaddr.rs` = the UDP command path (cmd 0x02, `[2B len][payload]` framing, `PacketConn` datagram API, packetaddr destination codec) |
| `crypto/` | VMess-adjacent primitives (aead/kdf/legacy_stream/salamander stubs) |
| `shape.rs` | `ConnectShape`: uniform vs divergent connect paths |
| `e2e/` (feature `native-e2e`) | case/config/core/harness/variant — real-core scenarios |

### xray-tui-tls (`crates/xray-tui-tls/src/`)

| Module | Responsibility |
|--------|----------------|
| `spec/` | declarative `ClientHelloSpec`/`ExtensionSpec`/`SessionIdSpec`, RFC 6066/8446 wire encodings, GREASE (RFC 8701) |
| `profiles/` | 12 browser profiles as spec data (`define_profiles!` macro): Chrome119/130/133, ChromeAndroid130, Edge130, Brave167, Opera114, Firefox, Firefox128Esr, Safari17, SafariIos17 (+ `Chrome` = Chrome130 alias) |
| `client/` | unified engine API: `TlsConfig { mode, server_name, alpn, rng }` + `TlsMode::{Plain, Reality}` + one `connect(stream, &TlsConfig)` entry |
| `hello/` | `build_hello`/`to_record` (GREASE pairing, 512-byte record padding), `parse_hello` |
| `crypto/` | key schedule (RFC 8448-verified), AEAD record keys (IV XOR seq), `X25519KeyPair`, `fingerprint/` JA3 + JA4 encoders |
| `record/` | record framing, `read_record`, `TlsStream<S>` (AsyncRead/Write, close_notify→EOF; per-direction direct mode `set_write_direct`/`set_read_direct` — raw record-layer bypass that hands the socket to the tunnel, backing the vision Direct splice) |
| `handshake/` | TLS 1.3 client handshake, `ServerVerifier` seam, multi-record flight reassembly; one shared `drive()` for plain + REALITY |
| `verify/` | `WebPkiVerifier` (roots/CA DER/`insecure`/`pin_sha256`; CV signature always checked) |
| `reality/` | `HelloProvisioner` + `ProfileProvisioner(BrowserProfile)` (any of the 12 profiles) + 9-step wire contract, `FixedChrome133`, auth-key/session-seal/server-auth, `SpiderConfig` + `spider.rs` (Spider-X h2 fallback) |
| `http2/` | minimal h2 layer (tls.peet.ws grading + Spider-X fallback GETs) |
| `error.rs` | `TlsError`/`Result` (thiserror) |

## Security layer capabilities

Every TLS/REALITY connect is engine-only: `security::wrap` builds an engine
`TlsConfig` from the profile's proto security config and runs
`xray_tui_tls::client::connect` (`TlsMode::Plain` | `TlsMode::Reality`). The
rustls client path and the `TlsProvider` plug are gone.

| Path | Trigger | Mechanism | Status |
|------|---------|-----------|--------|
| Plain TLS | `tls` config | engine `TlsMode::Plain`: fingerprint-shaped hello from the `fp` profile (`None` → Chrome130 default), `WebPkiVerifier` via `verifier_for(insecure, pin)`; profile = `parse_fingerprint_id(fp)` (exact ids: `chrome`/`chrome-randomized`/`firefox`/`safari`/`random` → Chrome130/Firefox128Esr/Safari17; unknown → config error) | ✅ |
| REALITY | `reality` config | engine `TlsMode::Reality`: fingerprint-shaped hello with any of the 12 profiles via `ProfileProvisioner(BrowserProfile)` (or a custom `HelloProvisioner`), sealed session id, X25519 auth key + HMAC/Ed25519 server auth (no PKI); Spider-X fallback on auth failure | ✅ |
| Trust modes | `insecure` / `pin_sha256` | `with_insecure()` skips chain walk; `with_pin(sha256(SPKI))` replaces chain+SAN but **never** skips the CertificateVerify signature (a MITM must hold the private key) | ✅ |

The unified API: `TlsConfig { mode: TlsMode, server_name, alpn, rng }` with a
single entry point `connect(stream, &TlsConfig)` — `TlsMode::Plain { profile,
verifier }` and `TlsMode::Reality { provisioner, public_key, short_id,
spider }`. REALITY is a security layer over the same TLS machinery: plain and
REALITY handshakes share one driver (`handshake::drive`).

**Spider-X fallback.** On a REALITY auth failure — the server flight is a real
certificate (a transparent proxy / possible MITM), mirroring xray-core
`reality.go`'s `!Verified` path — the client keeps the established TLS session
alive instead of tearing it down: it walks the real site with bounded HTTP/2
GETs (`SpiderConfig { paths, max_gets, request_interval }`; padding cookie +
Referer chaining) so a DPI observer sees browsing traffic, then reports
`TlsError::RealityFallback` → native `NativeError::Reality("REALITY: received
real certificate (potential MITM or redirection)")`.

## E2E coverage (tier 3)

Two axes. **Transport matrix** (`tests/vless.rs` + `tests/vmess.rs`): every
VLESS/VMess case × TCP/WS/gRPC/HTTPUpgrade/XHTTP/h2 × serving core(s) — 113
tests = 109 green + 4 documented ignored (vless ws/grpc
plain-into-reality-server semantic rows × both cores; single-core rows run
only on the serving core: xhttp on xray, v2rayhttp + ws/httpupgrade-reality
on sing-box). **TLS-variant cases**, each run against both cores (xray
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

**VLESS vision flow axis** (spec §7.4): 8 rows — `flow = xtls-rprx-vision`,
tcp network, × core {xray, sing-box} × outer {tls, reality} × app {plain,
inner-tls}. The inner-tls rows drive the Direct-splice path (the app
establishes a real TLS 1.3 session through the tunnel to the rustls echo
target, then the vision filter switches both directions to a raw relay after
the Direct frame); the plain rows exercise the End (non-TLS inner traffic —
padding stops, outer TLS continues) path. Vision is UDP-rejected: the flow
guard refuses UDP traffic under `xtls-rprx-vision` (mirrors xray's
UDP/443 rejection; no XUDP mux — SP3), even though the plain UDP command
path (below) is implemented.

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
REALITY compatibility (any of the 12 browser profiles via
`ProfileProvisioner`); "e2e" is tier-3 proof.

### Overview matrix

| Protocol | xray-core | sing-box | Native client | TLS engine | REALITY | e2e |
|----------|:---------:|:--------:|:-------------:|:----------:|:-------:|:---:|
| VLESS | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| VMess | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Trojan | ✅ | ✅ | 📋 | ✅ | ✅ | 📋 |
| Shadowsocks | ✅ | ✅ | 📋 | — | — | 📋 |
| ShadowsocksR | ⛔ | ✅ | 📋 | — | — | 📋 |
| SOCKS | ✅ | ✅ | 📋 | — | — | 📋 |
| HTTP | ✅ | ✅ | 📋 | — | — | 📋 |
| WireGuard | ✅ | ✅ | 📋 | — | — | 📋 |
| Hysteria2 | ✅ | ✅ | 📋 | 🔒 QUIC | — | 📋 |
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
  carry TLS inside QUIC and need a QUIC transport first — 🔒.
- **TLS 1.2**: the engine is TLS 1.3-only. TLS 1.2 engine support is a future
  task — legacy (TLS 1.2-only) servers are not yet reachable.
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
| Encryption | none (identity = UUID; optional `xtls-rprx-vision` flow control, TLS 1.3 framing) |
| Auth | UUID (command bytes in header) |
| Obfuscation | none at protocol level; REALITY supplies traffic camouflage |
| Transports | TCP, WS, gRPC, h2, QUIC (xray-core); TCP/WS/gRPC/h2/QUIC (sing-box) — native: TCP/WS/gRPC/HTTPUpgrade/XHTTP/h2 ✅ (kcp/quic UDP stacks + XHTTP stream-one deferred). Vision requires raw TCP — the Direct handoff needs the socket (ws/grpc/xhttp framing is incompatible). |
| UDP | `cmd 0x02` UDP command path ✅ — `[2B BE len][payload]` datagram framing both directions over the tunnel stream (`protocol/vless/udp.rs`), `PacketConn` datagram API with the response-header peel (`packet.rs`), packetaddr mode (`packetaddr.rs`; sing-box-style: header dest = magic fqdn `sp.packet-addr.v2fly.arpa`, per-packet frame header `atyp|addr|port`, no magic in the frame). e2e 5 rows: Raw × {tls-standard, reality} × both cores + packetaddr/tls/sing-box (xray has no packetaddr registration). Vision+UDP rejected — the flow guard refuses UDP under `xtls-rprx-vision` (mirrors xray's UDP/443 rejection; no XUDP mux — SP3). |
| Flow | `xtls-rprx-vision` ✅ — padded camouflage frames + inner-TLS filter + Direct splice state machine (`protocol/vless/vision.rs`, protobuf flow addon in the request header). TCP only; vision+UDP rejected (the plain UDP command path exists, but the flow guard rejects UDP traffic under vision, mirroring xray; XUDP mux = SP3). Requires outer TLS1.3/REALITY over raw TCP (guards in `connect_vision` mirror xray's rejection). Inner TLS1.3 → `Direct` raw splice — both directions abandon the outer TLS after the Direct frame (the Direct frame is the last outer-TLS record); non-1.3 inner traffic → `End`, padding stops, outer TLS continues. Deviations (spec §9): no 500 ms camouflage timer (the empty Continue long-padding frame is emitted immediately after the header — same wire bytes, deterministic), per-direction direct flags (`TlsStream::set_write_direct`/`set_read_direct`) instead of Go's unsafe `tls.Conn` reflection, no XUDP mux. |
| Status | Native client complete + e2e (tls-standard, tls-chrome, reality, vision-tls, vision-reality, udp-raw, udp-packetaddr) × both cores, full TCP-stream transport matrix e2e (113-test sweep = 109 green + 4 documented ignored). Deferred: kcp/quic (UDP stacks), XHTTP `stream-one`, HTTPUpgrade `ed` early-data, h2 PING keepalive, xmux/reuse pooling, browser-masquerade header set, vision UDP (`xtls-rprx-vision-udp443` — flow guard rejects UDP under vision; XUDP mux is SP3). |

**VMess** — ✅ native
| Capability | Detail |
|------------|--------|
| Encryption | payload: `aes-128-gcm`, `chacha20-poly1305` (native ✅); legacy `aes-128-cfb` / `none` (xray-only, ⛔ legacy disabled in modern cores) |
| Auth | AEAD request header (mandatory), MD5/HMAC-SHA256 KDF chain, FNV-1a + CRC-32 frame checks |
| Obfuscation | none at protocol level (transport-level ws/grpc/http in xray) |
| Transports | TCP, WS, gRPC, h2, QUIC (xray-core); TCP/WS/gRPC/h2 (sing-box) — native: TCP/WS/gRPC/HTTPUpgrade/XHTTP/h2 ✅ (kcp/quic UDP stacks + XHTTP stream-one deferred) |
| Status | Native client complete + e2e (aes-128-gcm, chacha20-poly1305, tls-firefox, reality) × both cores, full TCP-stream transport matrix e2e. Legacy ciphers ⛔. Deferred: kcp/quic (UDP stacks), XHTTP `stream-one`, HTTPUpgrade `ed` early-data, h2 PING keepalive, xmux/reuse pooling, browser-masquerade header set, v2rayhttp no-TLS h1 arm. |

**Trojan** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | none — plaintext after TLS; security is the TLS layer |
| Auth | password (SHA-224-derived key in the header) |
| Obfuscation | none (REALITY can carry it) |
| Transports | TCP, WS, gRPC, h2, QUIC (xray-core); TCP/WS/gRPC/h2/QUIC (sing-box) — native: TCP only 📋 |
| Status | Simple: TLS + password header + raw tunnel. TLS engine ✅. |

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

**SOCKS** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | none (SOCKS5) |
| Auth | optional username/password (RFC 1929), no-auth |
| Obfuscation | none |
| Transports | TCP (+ UDP ASSOCIATE 📋) |
| Status | Simple handshake; low priority (client rarely proxies to a bare SOCKS server). |

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

**Hysteria2** — 📋 native
| Capability | Detail |
|------------|--------|
| Encryption | TLS 1.3 inside QUIC (standard + fingerprint TLS ✅ via engine) |
| Auth | password in QUIC 0-RTT |
| Obfuscation | `salamander` obfs (crypto/ `salamander.rs` scaffolding) |
| Transports | QUIC (UDP) — 🔒 no QUIC transport yet |
| Status | Client needs a QUIC stack; out of scope until `quic` transport 📋. |

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
| XHTTP (splithttp) | ✅ | `transport/xhttp.rs`; v3 dialect (xray-only server): uuid session in path, GET-body download, raw POST uploads (seq + 30 ms pacing + `Referer` `x_padding`, ≤1 MB chunks); packet-up + stream-up; h1 when no TLS, h2 over TLS; e2e vs xray (single-core) |
| h2 (v2rayhttp) | ✅ | `transport/v2rayhttp.rs`; h2 single full-duplex PUT stream, `:authority` = config host else `www.example.com`; sing-box only (xray removed the h2 transport in 26.x); e2e vs sing-box (single-core) |
| QUIC | 🔒 | prerequisite for Hysteria1/2 + TUIC clients |
| KCP | 📋 | xray-core only; UDP (separate UDP-transport project, deferred) |
| obfs plugins (SS) | 📋 | plugin URL param already parsed by proto |

### Transport deferrals (documented, no stub)

- **kcp / quic** — UDP transports (mKCP FEC/session; QUIC stack). Separate
  project; gates Hysteria1/2 + TUIC clients.
- **XHTTP `stream-one`** — legacy v1 dialect (`auto` selects packet-up; only
  reachable via the REALITY auto-default, which we override explicitly).
- **HTTPUpgrade `ed` early-data** — xray-only client flag; server sees a
  normal 101 exchange.
- **h2 PING keepalive** — `idle_timeout`/`ping_timeout` in `HttpConfig` not
  wired.
- **xmux / connection-reuse pooling** — one session per tunnel today.
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
2. **Non-TCP transports** — the full TCP-stream transport set is done
   (WS/gRPC/HTTPUpgrade/XHTTP/h2); next is the UDP side (QUIC/KCP), which
   unlocks Hysteria1/2 + TUIC.
3. **Wire in the TUI**: a per-profile "native" toggle (or auto-fallback when a
   core binary is missing) — the unified `xray_tui_tls::client::connect` /
   `TlsConfig` engine API already proves the integration point.
4. **Outbound-only kinds + routing** via native (redirect dial for
   split-tunnel rules).
5. **TLS 1.2 engine support** — the engine is TLS 1.3-only today; legacy
   (TLS 1.2-only) servers become reachable only after the engine learns
   TLS 1.2.
