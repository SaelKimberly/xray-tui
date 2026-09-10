# Native Shadowsocks / Shadowsocks-2022 — Design

Date: 2026-09-10. Status: approved (approach A; interview decisions locked).
Scope: `xray-tui-native` only. No `xray-tui-proto` change, no server side.

## Goal

Implement the Shadowsocks client (classic AEAD **and** 2022-blake3) inside the
in-process native core, TCP + UDP, with tier-1 unit tests, tier-3 real-core e2e
rows, and benchmarks. Today both kinds are placeholders
(`protocol/ss/mod.rs`, `protocol/mod.rs` dispatch arm, `capability` keeps both
out of `NATIVE_KINDS`).

## Decisions (locked in the interview)

1. **Cipher scope: AEAD + 2022-blake3 only** — `aes-128-gcm`, `aes-192-gcm`
   (sing-box-only), `aes-256-gcm`, `chacha20-ietf-poly1305`,
   `xchacha20-ietf-poly1305`, `2022-blake3-aes-128-gcm`,
   `2022-blake3-aes-256-gcm`, `2022-blake3-chacha20-poly1305`. No legacy stream
   ciphers (aes-cfb/ctr, rc4-md5, chacha20-ietf, xchacha20, none). Legacy rows
   keep resolving to sing-box, which is exactly what `resolve_core` already
   does.
2. **UDP: implemented** (classic per-datagram salt; 2022 session-based), with a
   new `PacketTunnel::Ss` variant.
3. **SIP003 plugins: gated off** (`plugin`/`plugin_opts` set → capability
   `false` → subprocess). Native plugin support is a *recorded deferral*, not
   part of this work.
4. **E2E breadth: one method per family × both serving cores** (xray + sing-box
   via `#[values]`), plus UDP rows.
5. **Benchmarks: hermetic codec micro-bench + throughput rows.**
6. **2022 multi-user EIH: not implemented.** Single-user PSK only, no proto
   identity field. Documented limitation.

## Dependencies (new; explicit approval obtained)

| Crate | Version | Why | zeroize |
|---|---|---|---|
| `sha1` | 0.11 | HKDF-SHA1 for the classic-AEAD subkey | `features = ["zeroize"]` — OFF by default |
| `hkdf` | 0.13 | classic-AEAD subkey = HKDF-SHA1(psk, salt, `b"ss-subkey"`) | no `zeroize` feature; the derived subkey is *our* buffer → `Zeroizing` |
| `blake3` | 1.8 | 2022 subkey = `blake3::derive_key("shadowsocks 2022 session subkey", psk ‖ salt)` | `features = ["zeroize"]` — OFF by default |

- `blake3` is promoted from dev-dependency to dependency; the dev-deps entry is
  removed (normal deps are visible to unit tests).
- Considered and rejected: `ring::hkdf::HKDF_SHA1_FOR_LEGACY_USE_ONLY` (already
  in tree, would keep the dep count flat) — rejected per the explicit decision
  to prefer the well-verified RustCrypto crates; it also matches vmess, which
  already uses RustCrypto AEAD (not ring AEAD) despite `crypto/aead.rs`'s stale
  "ring backend" note.
- `hkdf 0.13` + `sha1 0.11` + `hmac 0.13` must share digest 0.11. **Step 0 of
  implementation is a compile + RFC 5869 vector test** before any other code.
- Mandatory gates after the manifest edit: `cargo tree -p xray-tui-native -e
  features | grep zeroize` (decision 19), `cargo hakari generate`, and the
  `xray-tui-hakari` machete ignore list (AGENTS.md awk recipe).

## Two BLAKE3 users (do not merge them)

`protocol/vless/encryption/b3.rs` stays as-is: its derive-key needs a **binary**
context, which the `blake3` crate cannot express (`&str` only). The 2022
session subkey uses an **ASCII** context, so it uses the audited crate. Each
file's module doc names the split; b3.rs's cross-validation tests already pin
the crate-equivalent path.

## Module layout

| File | Change |
|---|---|
| `crypto/kdf.rs` | replace stub note: HKDF-SHA1 helper (salt-only extract/expand over `hkdf` + `sha1`) |
| `crypto/aead.rs` | replace stub: `SsAead` enum (`Aes128Gcm`, `Aes192Gcm`, `Aes256Gcm`, `ChaCha20Poly1305`, `XChaCha20Poly1305`) with explicit-nonce seal/open |
| `protocol/ss/method.rs` | method string → `(SsAead, key_len, family)`; aliases (`aead_aes_*`, `chacha20-poly1305`, `xchacha20-poly1305`, `aead_chacha20_poly1305`); password → key (classic: MD5 `EVP_BytesToKey`, **unconditional**; 2022: base64 PSK, 16/32 B) |
| `protocol/ss/stream.rs` | classic AEAD TCP codec |
| `protocol/ss/stream2022.rs` | 2022 TCP codec |
| `protocol/ss/udp.rs` | `PacketConn` + `PacketReader`/`PacketWriter` for both families |
| `protocol/ss/mod.rs` | replace placeholder doc: `connect` + `connect_udp` dispatch |
| `protocol/mod.rs` | `Ss` arms in `connect`/`connect_udp`; `PacketTunnel::Ss` + reader/writer variants |
| `capability.rs` | `NATIVE_KINDS` += `Shadowsocks`, `Shadowsocks2022`; `ss_supported`; UDP doc note update |
| `crypto/legacy_stream.rs` | stub stays; doc → "deferred: legacy ciphers stay on sing-box" |
| `Cargo.toml` | 3 deps; `[[bench]] name = "ss_codec"` (`harness = false`, no feature gate) |
| `src/e2e/case.rs` | `ProtocolKind::Shadowsocks` + method field + `CaseSpec::shadowsocks(method)` |
| `src/e2e/config.rs` | `ss_inbound` (both dialects) + `client_params_ss` |
| `tests/shadowsocks.rs` | NEW e2e matrix |
| `benches/ss_codec.rs` | NEW hermetic codec bench |
| `benches/throughput.rs` | SS rows |

`chain.rs`, `shape.rs`, `lib.rs` are untouched (both kinds are already
`ConnectShape::TcpStream`).

## Wire formats

### Classic AEAD — TCP (`relay/tcprelay/aead.rs`, 0x3FFF cap)

```
[salt][ [2B BE len][16B tag] [ct(payload)][16B tag] ]*
```
Two AEAD seals per chunk — the 2-byte length, then the payload — with the
counter advanced by each seal (shadowsocks-rust `tcprelay/aead.rs` header
diagram; the 2022 edition spec notes it inherits the same
length-chunk/payload-chunk model); `len ≤ 0x3FFF`.
- `salt` = `max(16, key_len)` random bytes: 16 for aes-128-gcm, 24 for
  aes-192-gcm, 32 for aes-256-gcm / chacha20-ietf-poly1305 /
  xchacha20-ietf-poly1305.
- `subkey = HKDF-SHA1(psk, salt, "ss-subkey")`, output = key_len.
- nonce = little-endian counter of the cipher's nonce size (12 B for AES-GCM /
  ChaCha20-Poly1305, 24 B for XChaCha20-Poly1305), incremented per seal/open,
  AAD empty.
- Request and response streams each have their **own** salt, subkey and
  counter; the client reads the server's salt first.
- First chunk's plaintext = `[ATYP|addr|port]` (`addr::encode_addr`) `‖` data.
- Zero-length chunk is an error (`ErrZeroChunk`).
- Key: password → **MD5 `EVP_BytesToKey`, always** — no raw-password shortcut
  (xray `proxy/shadowsocks/config.go::passwordToCipherKey`, mihomo
  `core/cipher.go::PickCipher`, ss-rust `make_derived_key`).

### Classic AEAD — UDP (`shadowaead/packet.go`)

```
[salt][ seal(zerononce, [ATYP|addr|port] ‖ payload) ]     one packet, fresh salt
```
Separate HKDF-SHA1 subkey per datagram; nonce is the cipher's nonce size of
all-zero bytes.

### 2022-blake3 — TCP (spec §3.1, `relay/tcprelay/aead_2022.rs`)

```
subkey = blake3::derive_key("shadowsocks 2022 session subkey", psk ‖ salt)
nonce  = 12-byte LE counter, advanced by EVERY seal/open (spec §3.1.1) —
         so a payload chunk costs +2 (length seal, payload seal), and the
         standalone header chunks cost +1 each

request : [salt][seal(11B: type=0 | ts u64be | len u16be)]
                [seal(varlen: ATYP|addr|port | pad_len u16be | padding | initial payload)]
                [seal(2B len)][seal(payload)] ...      len ≤ 0xFFFF
response: [salt][seal(27/43B: type=1 | ts u64be | request_salt | len u16be)]
                [seal(payload)][seal(2B len)][seal(payload)] ...
```
- Client MUST verify `request_salt` in the response header equals its own salt.
- Salt + both header chunks MUST go out in **one** `write_all` (anti-DPI).
- Request header MUST carry payload or non-zero random padding; simplest
  conforming client: always random padding 1..=900 (spec §3.1.3).
- Timestamp: `std::time::SystemTime` (no new dep).
- AES methods use plain AES-GCM with the counter nonce (spec §3.1.1); the
  chacha method substitutes ChaCha20-Poly1305 (spec §4).

### 2022-blake3 — UDP (spec §3.2 + §4.1) — the largest single piece

AES methods (AES-128/256-GCM):
```
packet = [ AES-ECB(psk, [client_session_id u64be ‖ packet_id u64be]) ]   (16 B, one block)
       ‖ seal( nonce = separate_header[4..16],
                [type=0 | ts u64be | pad_len u16be | padding | addr | port | payload] )
subkey = blake3::derive_key("shadowsocks 2022 session subkey", psk ‖ client_session_id[0..8])
```
Server → client is a **different, larger** shape: its own
`[server_session_id ‖ server_packet_id]` header block, and a body whose main
header is `[type=1 | ts | client_session_id 8B | pad_len | padding | addr | port | payload]`.
The client keeps a client-session ↔ server-session map (≥1 old association per
spec §3.2.4) and a packet-id sliding-window filter, checked after header
validation.

The slice `separate_header[4..16]` above is the **plaintext** block (the
ciphertext is `encrypted_separate_header`): the nonce is
`session_id[4..8] ‖ packet_id` — never bytes read off the wire. The replay
window is **per relay session**, not global: a restarted server issues a new
server session whose packet ids restart at 0, and a shared window would reject
every one of its replies. Each session slot therefore carries its own window
(and its own body subkey).

ChaCha method: `[24B random nonce][XChaCha20-Poly1305(psk directly) over body]`
with the session id + packet id **merged into the main header** (no separate
header block).

## UDP carrier contract

`PacketTunnel::Ss` follows the trojan precedent: the wire always carries a
per-packet address, so

- `send(Some(d))` — carries `d`;
- `send(None)` — the session destination, i.e. `params.target` (every variant
  accepts `None`);
- `recv` returns the address parsed out of the reply header;
- `PacketMode::PacketAddr` / `XUdp` stay refused by the existing
  `reject_vless_only_mode`; `PacketMode::Raw` is accepted and inert.
  **The guard is not modified** — no carved exemption, no divergent e2e row
  shape (mirrors trojan/hysteria2 rows).

**Transport shape (dial-end, not a stream carrier).** Shadowsocks' UDP relay
is reached by sending UDP datagrams to the server's own port — not by framing
UDP inside a TCP tunnel the way VLESS/VMess/trojan do (xray
`proxy/shadowsocks/client.go` UDP dispatch; sing-box's ss outbound likewise).
`connect_chain_udp` therefore treats an SS last link like the QUIC arm: the
dial (`bind` a UDP socket to `params.server`) REPLACES dial + security +
transport + upgrade, no chain is possible (an SS UDP link must be the only
link — the guard mirrors `quic_guard` and refuses any non-last position), and
each SS datagram is exactly one UDP datagram on the wire (no carrier length
prefix). `PacketTunnel::Ss` consequently owns an `Arc<UdpSocket>`, and its
split halves are two independent states over that socket (classic: salt/subkey
per datagram, stateless; 2022: writer state = client session + packet counter,
reader state = server-session map + sliding window), so no shared lock is
needed. SS remains `ConnectShape::TcpStream` for the TCP path — the UDP shape
is a UDP-path-only dispatch arm, documented next to `is_quic_link`.

## Capability gate

`kind_supported` gains both kinds. `supported(kind, config)` → `false` when:

- method is not in the native AEAD/2022 table (legacy → sing-box);
- `method` family disagrees with the kind (`2022-blake3-*` ⇔
  `Shadowsocks2022`, everything else ⇔ `Shadowsocks`);
- `plugin` or `plugin_opts` is set (SIP003, deferral #3);
- 2022 password is not valid base64 of exactly 16/32 bytes (fail-closed,
  per the file's own contract), or the method is unknown;
- `security_supported` / `transport_supported` reject the row (SS-over-TLS/ws
  works and is *allowed* — the chain applies security/transport outside the
  protocol for free).

The `supported()` doc note about the SOCKS5 proxy leg never setting
`params.udp` stays true and keeps its wording (SS-datagram support here is the
`connect_udp` path, same as the other carriers).

## Verification

**Tier 1 (`cargo test -p xray-tui-native --features native-e2e --lib`)**

- classic: HKDF-SHA1 vs RFC 5869 vectors + a pinned `ss-subkey` vector; MD5
  `EVP_BytesToKey` vector (from xray/mihomo); salt size rule; byte-exact golden
  of the first request chunk (`[2B len][tag][ct][tag]`, two seals, counter +2);
  chunk round-trip; 0x3FFF boundary (0 / 1 / 0x3FFF / 0x4000); tamper → AEAD
  error; zero chunk rejected; LE counter increment + wrap; UDP pack/unpack +
  short packet.
- 2022: `blake3::derive_key` vector pinned from a Go/reference run; PSK
  base64/length validation; request fixed-header golden (type/ts/len);
  response `request_salt` mismatch → error; 0xFFFF payload boundary; single
  write-call assertion for salt + headers (recording stream); padding rule.
- 2022 UDP: AES-ECB separate-header round-trip; nonce slice `[4..16]`; subkey
  from `psk ‖ session_id[0..8]`; client→server and server→client header
  layouts; chacha 24-byte-nonce shape; sliding-window filter (duplicate /
  out-of-window rejected); session map.
- capability truth table (legacy, plugin, kind/method mismatch, bad 2022 key,
  classic ok, 2022 ok).
- dispatch: `connect`/`connect_udp` arm selection; mode guard unchanged.

**Tier 3 (`XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin`, both cores present locally)**

`tests/shadowsocks.rs`, new `CaseSpec::shadowsocks(method)` ×
`#[values(CoreKind::Xray, CoreKind::SingBox)]`:

- classic: `aes-128-gcm`, `chacha20-ietf-poly1305`, `xchacha20-ietf-poly1305`
  (the last one exercises the 24-byte-nonce counter path the other two never
  touch); `aes-192-gcm` is deliberately absent — it shares the AES-GCM path
  with 128/256 and only moves the key length;
- 2022: `2022-blake3-aes-256-gcm`, `2022-blake3-chacha20-poly1305`;
- UDP: one classic + one 2022 row;
- one `ws` (or plain-TLS) transport row to prove the chain composes.

**Benches**

- NEW `benches/ss_codec.rs` (hermetic, no core, no network — CI-runnable):
  seal/open MB/s per classic cipher, 2022 TCP chunk seal/open, 2022 UDP AES vs
  XChaCha packet seal/open, HKDF-SHA1 subkey cost, blake3 `derive_key` cost.
- `throughput.rs` rows: `ss/tcp/aead-aes-128-gcm` (xray),
  `ss/tcp/aead-chacha20-ietf-poly1305` (xray),
  `ss/tcp/2022-blake3-aes-256-gcm` (xray),
  `ss/tcp/2022-blake3-chacha20-poly1305` (sing-box).

## Docs to update (owner docs only)

- `NATIVE_CORE.md`: `protocol/` + `crypto/` crate-map rows, capability notes,
  tier counts, an SS axis paragraph, and the deferral list (plugins, legacy
  ciphers, 2022 EIH multi-user).
- `AGENTS.md`: `NATIVE_KINDS` mentions, native crate-map line, decision 20's
  "vless/vmess/trojan/hysteria2" text.
- No `docs/protocols.md` edit (proto and URL surface unchanged).

## Risks / sequencing

- **Dep-major compatibility** (`hkdf 0.13` × `sha1 0.11` × `hmac 0.13`): step 0
  compile + RFC 5869 test before any protocol code.
- **2022 UDP is the largest piece** (two constructions × two directions +
  session state + replay window). Implemented last; if it must shrink, that is
  an explicit re-decision, never a silent drop.
- **2022 AES-GCM TCP construction** is pinned from spec §3.1.1 + reference
  implementations and proven by interop rows against both cores.
- Client-side salt uniqueness depends on the CSPRNG: salts come from
  `rand::fill_nonsecret` (wire-visible filler, correct home per `rand.rs`);
  every derived subkey is `Zeroizing`.
- Anti-probe requirements (single write for salt+headers, payload-or-padding)
  are test-asserted, not just commented.

## Non-goals

Server-side implementations; SIP003 plugins (deferred, gated); legacy stream
ciphers (deferred to sing-box); 2022 multi-user EIH; ShadowsocksR; a
`PacketMode` addition; any `xray-tui-proto` change.
