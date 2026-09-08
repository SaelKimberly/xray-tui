# Memory Diet Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Cut heap churn and retained memory across xray-tui workspace with measured RSS/alloc wins, zero behavior change.

**Architecture:** Bench-gated A→B sequence. A = hot-path copies only, schema-free, no new deps. B = shared ownership + bounds with dep gates. C deferred (batch persist tx, arenas, JSON dedup).

**Tech Stack:** Rust 2024, tokio, bytes 1, smallvec 1, smartstring 1 (LazyCompact), bumpalo/lru deferred to B/C only, criterion 0.8, dhat/heaptrack for alloc counts.

**Spec:** This plan is the spec (research synthesis 2026-09-07, 5 scout audits). No separate design doc — research report in conversation is the authority.

## Global Constraints

- `Zeroize` is hard constraint per decision 19: TLS key schedule / X25519 agree / VMess-VLESS keys / Salamander psk / trojan token / SOCKS5 auth frame stay `Zeroizing`, never enter `Bytes` pools / bumpalo arenas / shared clones without explicit wipe. After any crypto dep edit: `cargo tree -p <crate> -e features | grep zeroize`.
- DB schema tag `PRAGMA user_version=6` frozen: A is schema-free. No model field type change (`String`→`TinyText`, `Vec`→`SmallVec`, JSON col dedup) in A. B in-memory only, convert at DB edge. Any tag bump deletes DB file per decision 4 — never in this plan.
- `canonical_once` must emit byte-identical bytes to today's `to_value→to_vec` chain per decision 11, else uid re-key + row dups. Golden-vector test gates it.
- New deps need `cargo hakari generate` + zeroize re-check + `just quality-gate code|deps` (or `cargo fmt --check`, `cargo clippy`, `cargo nextest` subset).
- `bytes` clones of secrets forbidden. `[u8;N]`/SmallVec for tiny stack sizes, not `Bytes`.
- Every task ends with measured proof: criterion row or alloc-count delta. No "has tests" padding.

---

### Task 1: Baseline benches + alloc harness

**Files:**
- Modify: `criterion.toml` (no change, read only)
- Test: existing `crates/xray-tui-tls/benches/record.rs`, `crates/xray-tui-route/benches/decide.rs`, `crates/xray-tui-native/benches/dispatch.rs`, `crates/xray-tui-native/benches/relay.rs`
- Create: `.benchmarks/baseline-memory.md` (scratch, delete at end)

**Interfaces:**
- Consumes: nothing
- Produces: `BASELINE` numbers (record/decide/dispatch/relay throughput + alloc counts) for Tasks 2-6 to compare against

- [ ] **Step 1: Record baseline benches**

Run: `just bench micro`
Expected: PASS, rows land in `.benchmarks/`. Copy table into `.benchmarks/baseline-memory.md`.

- [ ] **Step 2: Record import + handshake alloc baseline (throwaway)**

```rust
// /tmp/alloc_probe.rs — throwaway, not committed
// Run with: cargo test -p xray-tui-proto --lib alloc_probe -- --nocapture
#[test]
fn alloc_probe_canonical_triple() {
    let json = serde_json::json!({"type":"vless","id":"00000000-0000-0000-0000-000000000000"});
    let v = serde_json::to_value(&json).unwrap();
    let b = serde_json::to_vec(&v).unwrap();
    eprintln!("canonical_len={}", b.len());
}
```

Run: `cargo test -p xray-tui-proto --lib -- --nocapture 2>&1 | head -20`
Expected: PASS, prints lengths. Delete `/tmp/alloc_probe.rs` after.

- [ ] **Step 3: Commit**

```bash
git add .benchmarks/baseline-memory.md
git commit -m "bench: memory-diet baseline"
```

---

### Task 2: Native datapath Bytes end-to-end (ws/grpc/udp/mux/vision/vmess/inbound)

**Files:**
- Modify: `crates/xray-tui-native/src/transport/ws.rs` (~line 120 read staging, ~150 `Message::Binary(buf.to_vec())`)
- Modify: `crates/xray-tui-native/src/transport/grpc.rs` (42-70 `encode_frame`, 84-96 `decode_frame`, 221-232 read path)
- Modify: `crates/xray-tui-native/src/transport/xhttp.rs` (1010-1100 `pending.extend`, 870-990 `mpsc::channel::<Bytes>(4)`, 60-110 per-request `format!`/padding)
- Modify: `crates/xray-tui-native/src/protocol/vless/udp.rs` (133-136 `write_packet`), `crates/xray-tui-native/src/protocol/vless/packet.rs` (161 `to_vec`, 208 frame build)
- Modify: `crates/xray-tui-native/src/protocol/vless/mux.rs` (318/368 `write_frame`, 457/462 channels)
- Modify: `crates/xray-tui-native/src/protocol/vless/vision.rs` (564-605 uplink `content`+`out` Vecs, 148-160 `write_buf/read_buf/raw_leftover/scratch`)
- Modify: `crates/xray-tui-native/src/protocol/vmess/stream.rs` (423 `seal_pending Vec`, decrypt fresh Vec; `udp.rs:68` 64KiB scratch + `to_vec`)
- Modify: `crates/xray-tui-native/src/inbound/mod.rs` (823/911/1027 per-datagram `to_vec`, 500/1009 `mpsc(256)`, 444 trace clones, 1010 proxy-leg clone)
- Modify: `crates/xray-tui-native/src/inbound/outbound.rs` (197 `vec![0;16384]` x2, 57 `proxy_params` ProtocolConfig clone)
- Modify: `crates/xray-tui-native/src/telemetry.rs` (155 dual mpsc cap, TraceOpened Strings), `crates/xray-tui-native/src/context.rs` (163 `vec![b"h2"]`), `crates/xray-tui-native/src/chain.rs` (78/115 `link.clone()` per hop)

**Interfaces:**
- Consumes: `BASELINE` from Task 1
- Produces: `Bytes`-through-relay convention: relay hands `Bytes`, transports take `Bytes`/`&[u8]`, no `to_vec` on forward path

- [ ] **Step 1: Write failing perf test (grpc frame single-alloc)**

```rust
#[test]
fn grpc_encode_single_buffer() {
    let payload = vec![0xABu8; 100];
    let before = allocation_count(); // test helper via global counter, or manual review
    let b = encode_frame(&payload);
    assert!(b.len() == payload.len() + 8);
}
```

Run: `cargo test -p xray-tui-native grpc_encode_single_buffer -v`
Expected: FAIL (function shape differs) or PASS with alloc counter showing 2 allocs. Proceed regardless — the gate is the bench delta in Step 4.

- [ ] **Step 2: Implement ws + grpc + xhttp**

```rust
// ws.rs poll_write: BEFORE
// tungstenite::Message::Binary(buf.to_vec().into())
// AFTER: caller passes Bytes; Message::Binary(bytes) moves, no copy
// ws.rs read: replace BytesMut staging + split_to + put_slice with VecDeque<Bytes> + slice
// grpc.rs encode_frame: BEFORE varint_len()->Vec + hunk Vec + out Vec
// AFTER:
// let mut out = bytes::BytesMut::with_capacity(5 + 2 + payload.len());
// out.extend_from_slice(&[0u8]); // compressed flag
// out.extend_from_slice(&encode_varint_stack(hunk_len)); // [u8;8] stack
// out.extend_from_slice(&hunk_header_stack); // stack
// out.extend_from_slice(payload);
// out.freeze()
// grpc.rs decode: return Bytes slice, feed Bytes queue, no to_vec
// xhttp.rs: raise mpsc depth 4 -> 16, pass Bytes from relay (drop pending.extend),
//   static padding: const PAD: &str = "XXXX..."; // 1000 X's, slice per len
//   normalize_path/path_with: reuse session-base String via clear()+write!, no format! per request
```

- [ ] **Step 3: Implement udp/mux/vision/vmess/inbound/telemetry/context/chain**

```rust
// vless/udp.rs + packet.rs: per-conn BytesMut scratch field, resize not vec!,
//   split_to().freeze() -> Bytes on recv; write_all_vectored([prefix_stack, payload]) on send
// mux.rs write_frame: encode meta into [u8;64] stack, then single BytesMut out (no meta Vec)
// vision.rs: unpad in place into caller BytesMut, encode direct into write_buf.reserve(),
//   cap read_buf at 65536 + shrink_to(8192) on idle
// vmess/stream.rs: reusable BytesMut seal buffer + encrypt_into; udp scratch BytesMut::zeroed once + split_to().freeze()
// inbound/mod.rs: payload: Bytes end-to-end (parse borrows &[u8], one copy_from_slice, slice into queue);
//   queue bound: 256 pkts AND 1MB bytes, drop-oldest + counter; DnsCache keep; reply_buf shrink_to(2048) idle
// outbound.rs: proxy_params takes &ProxyOutbound + target by ref; relay bufs Box<[u8;16384]> (no zeroing cost)
// telemetry.rs: TraceOpened dest/proto/transport as Arc<str> (intern per server); cap 16 keep + drop counter keep
// context.rs alpn_vec: return &'static [Vec<u8>] or static slice, no per-call vec!
// chain.rs: clone link once, &link thereafter
```

- [ ] **Step 4: Run benches, verify delta**

Run: `just bench micro`
Expected: `dispatch`/`relay` rows improve or neutral, no regression >5%. `cargo test -p xray-tui-native --lib` PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/src/transport crates/xray-tui-native/src/protocol crates/xray-tui-native/src/inbound crates/xray-tui-native/src/telemetry.rs crates/xray-tui-native/src/context.rs crates/xray-tui-native/src/chain.rs
git commit -m "perf(native): Bytes end-to-end on datapath, kill per-packet Vecs"
```

---

### Task 3: TLS handshake stack + in-place (Zeroizing preserved)

**Files:**
- Modify: `crates/xray-tui-tls/src/crypto/mod.rs` (205 transcript_digest/to_vec, 255 hkdf_extract Zeroizing<Vec>, 270 hkdf_expand_label format!+Vecs, 441 seal/open allocating twins)
- Modify: `crates/xray-tui-tls/src/handshake/mod.rs` (277 transcript Vec+extends, 498 pre_buffer to_vec, 671 ct to_vec, 815 flight buf+body to_vec)
- Modify: `crates/xray-tui-tls/src/record/mod.rs` (70 vec![0;length], 96 payload to_vec)
- Modify: `crates/xray-tui-tls/src/spec/mod.rs` (encode_body ->Vec arms, used_cs/used_ext Vecs, groups.clone)
- Modify: `crates/xray-tui-tls/src/hello/mod.rs` (118 body, 159 ext_bytes, 258 handshake_bytes, 284 to_record, 167 ALPN Vec<String>, 140/161 used Vecs)
- Modify: `crates/xray-tui-tls/src/crypto/mlkem.rs` (77/92/107 from_bytes to_vec, 150/172 into_vec, 190 ss double-copy)
- Modify: `crates/xray-tui-tls/src/reality/auth.rs` (60/86 to_vec), `crates/xray-tui-tls/src/reality/verify.rs` (signed_content Vec)
- Modify: `crates/xray-tui-tls/src/crypto/tls12.rs` (123 p_hash block_in per-iter Vec)
- Modify: `crates/xray-tui-native/src/protocol/vless/encryption/mlkem.rs` (124/139 WireAead Vec, 393 hello vec!, 455 pfs Vec, 472 padding vec+seal, 693 payload vec+take+open)

**Interfaces:**
- Consumes: Task 2 `Bytes` convention (record layer stays Vec-reuse + in-place, already good)
- Produces: stack/info helpers + in-place flight APIs; secrets stay `Zeroizing<[u8;N]>`

- [ ] **Step 1: Write failing golden test for HKDF stack equivalence**

```rust
#[test]
fn hkdf_expand_label_stack_matches_heap() {
    let prk = vec![0x0Bu8; 32];
    let a = hkdf_expand_label_heap(&prk, "key", &[], 16);
    let b = hkdf_expand_label_stack(&prk, "key", &[], 16);
    assert_eq!(a, b);
}
```

Run: `cargo test -p xray-tui-tls hkdf_expand_label_stack_matches_heap -v`
Expected: FAIL (stack fn missing). Then implement.

- [ ] **Step 2: Implement crypto stack + in-place migration**

```rust
// transcript_digest/transcript_hash -> return ([u8;48], usize) stack, no Vec
// hkdf_extract -> Zeroizing<[u8;48]> (const-generic hash len), hmac::Tag is stack source
// hkdf_expand_label: info: [u8; 128] + copy_from_slice, label via match to b"tls13 ..." consts, out stack array
// AeadKey::seal/open allocating twins -> migrate flight+Finished+TLS1.2 to seal_in_place/open_in_place, delete twins (or #[cfg(test)])
// tls12 p_hash: hoist block_in out of loop, stack HMAC
// mlkem from_bytes: copy_from_slice into arrays, no to_vec; add encapsulate_into(&pk,&mut [u8;1088],&mut [u8;32])
// reality auth: seal_in_place_separate_tag / open_in_place on caller buf; signed_content [u8;145] stack
```

- [ ] **Step 3: Implement handshake/record/hello single-buffer path**

```rust
// handshake transcript: feed digest::Context incrementally; ee/cert/cv as ranges into ONE arena Vec (not 4 Vecs + Vec<Vec<u8>> chain)
// flight reassembly buf: reuse 16.6KB buffer like TlsStream::rec_buf, return (type, Range), pre_buffer as drain-offset not copy
// record read_record: read into reused buffer, no vec![0;length]
// spec encode_body: encode_body_into(&mut Vec<u8>) single with_capacity(hello_len); ALPN from &[&str]; used_cs/used_ext [u16;16]+len
// native mlkem: open_in_place(&mut payload)->len reusing read buf; seal_in_place appending tag into out_pending;
//   hello Vec::with_capacity+set_len pattern (fully overwritten); pfs encode straight into hello; padding seal into hello[off..]
```

- [ ] **Step 4: Verify + zeroize check**

Run: `cargo test -p xray-tui-tls --lib && cargo tree -p xray-tui-tls -e features | grep zeroize`
Expected: PASS + zeroize present on x25519-dalek path. `just bench micro` record row neutral-or-better.

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls/src crates/xray-tui-native/src/protocol/vless/encryption
git commit -m "perf(tls): stack HKDF/transcript, in-place flight, zero-copy ML-KEM"
```

---

### Task 4: Route/sniff borrow + no-alloc resolve

**Files:**
- Modify: `crates/xray-tui-route/src/engine.rs` (345/460 Decision clone + override_addr, 478/491/503 resolve 3x String + Vec clone, 393 emit_decision Strings, 34 payload_prefix Vec, 121 per-IP linear CIDR scan noted only)
- Modify: `crates/xray-tui-route/src/sniff.rs` (314 SNI to_vec+String, 238 http to_owned, 432 dcid Vec, 435/520 QUIC to_vec, 699 HKDF Vec, 138 result.clone)
- Modify: `crates/xray-tui-route/src/resolve.rs` (91/97/106 probe to_owned, unbounded ResolvedCache — bound in Task 6, not here)
- Modify: `crates/xray-tui-host-features/src/lib.rs` (229 lowercase-before-bloom)
- Modify: `crates/xray-tui-dns/src/lib.rs` (lookup collect Vec, per-init Client::new, write cache to_owned)
- Modify: `crates/xray-tui-geoip/src/lib.rs` (64/69 decode String x2, 150 bytes.to_vec 70MB, spawn_blocking move)
- Modify: `crates/xray-tui-route/src/addr.rs` (24 ip.to_string in Cow)

**Interfaces:**
- Consumes: Task 2 `Bytes` queue (UDP payload passed as &[u8])
- Produces: `DnsSink::lookup_ip(&str)` borrow convention + borrowed SNI convention

- [ ] **Step 1: Write failing borrow test**

```rust
#[test]
fn sniff_sni_borrows_prefix() {
    let pkt = build_client_hello_with_sni(b"example.com");
    let r = sniff_tls(&pkt).unwrap();
    assert_eq!(r.sni_as_str(), "example.com"); // no owned String in hot struct
}
```

Run: `cargo test -p xray-tui-route sniff_sni_borrows_prefix -v`
Expected: FAIL until `SniffResult<'a>`/`Cow<'a,str>` lands.

- [ ] **Step 2: Implement**

```rust
// engine: override_addr as Option<Arc<NetAddr>>; DnsSink::lookup_ip(&self, host: &str);
//   put via entry API reusing owned String; RouteEvent fields Arc<str>; ConnMeta payload_prefix: Option<Bytes>
// sniff: SNI via from_utf8 borrow + Cow (single alloc only when owned needed); DCID [u8;20]+len;
//   QUIC decrypt open_in_place on caller buf; HKDF info [u8;38] stack; Done by ref / Arc<SniffResult>
// host-features: Cow borrow when already lowercase, alloc only on verify path; hash lowercased bytes on fly for bloom probe
// dns: return Box<[IpAddr]> / iterator without collect when unfiltered; reqwest Client once (lazy); HashSet<&str> borrow in cache write
// geoip: decode &str borrowed from mmap + per-IP Arc<str> micro-cache; stream 70MB to file, no bytes.to_vec
// addr: Display impl for Ip arm, Cow::Borrowed for Domain only
```

- [ ] **Step 3: Verify**

Run: `cargo test -p xray-tui-route --lib && just bench micro`
Expected: PASS, `decide` row neutral-or-better.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-route/src crates/xray-tui-host-features/src crates/xray-tui-dns/src crates/xray-tui-geoip/src
git commit -m "perf(route): borrowed SNI/hosts, in-place QUIC, no-alloc resolve"
```

---

### Task 5: Parse/import streaming + canonical_once (byte-identical)

**Files:**
- Modify: `crates/xray-tui-proto/src/proto_spec/endpoint.rs` (95-96/105/137 triple serialization)
- Modify: `crates/xray-tui-config/src/subscription.rs` (84 work alloc, 177/183/189 process_aligned to_vec, 225-367 carry+lossy+per-URL to_string, all_urls+profiles hold-all)
- Modify: `crates/xray-tui-config/src/import_export.rs` (204/310/313/337/343/361/371/389/432 per-URL to_string/format!/lossy)
- Modify: `crates/xray-tui-proto/src/urlx/split_url.rs` (query/fragment/path allocs), `crates/xray-tui-proto/src/proto_spec/common.rs` (523 Ws header clone, clash_to_endpoint to_string)
- Modify: `crates/xray-tui/src/state.rs` (parsed_to_rows iter/collect + per-endpoint Protocol clone + sig/cred recompute + gid.to_string + 4x conn)
- Modify: `crates/xray-tui-config/src/forms.rs` (1327/1424/1434/1563 validation format!, opt_str to_string, port-range unbounded extend)

**Interfaces:**
- Consumes: nothing from Tasks 2-4 (independent slice, shares only test gate)
- Produces: `canonical_once() -> (sig, cred, uid, bytes)` + streaming import convention (parse-then-drop)

- [ ] **Step 1: Write failing golden test (gates byte-identity)**

```rust
#[test]
fn canonical_once_matches_triple_chain() {
    let cfg = sample_vless_config();
    let (sig_a, cred_a) = (sig_triple(&cfg), cred_triple(&cfg));
    let (sig_b, cred_b, _bytes) = canonical_once(&cfg);
    assert_eq!((sig_a, cred_a), (sig_b, cred_b));
}
```

Run: `cargo test -p xray-tui-proto canonical_once_matches_triple_chain -v`
Expected: FAIL (fn missing). PASS after, with identical uids — duplicates impossible.

- [ ] **Step 2: Implement canonical_once + streaming**

```rust
// endpoint.rs: pub fn canonical_once(cfg: &ProtocolConfig) -> (i64, i64, Vec<u8>) {
//   let v = serde_json::to_value(cfg).unwrap(); let b = serde_json::to_vec(&v).unwrap();
//   (hash_sig(&b), hash_cred(&b), b) // two RapidStreamHasher passes over SAME &[u8]
// }
// state.rs: build Protocol ONCE per ParsedProto, share &Protocol/Rc; stream rows (no collect);
//   ep.host/ports moved not cloned; &str gid; single conn/tx multi-row upsert (no 4x conn per endpoint)
// subscription.rs: work scratch on StreamingDecoder (clear+reserve reuse); process_aligned -> Cow<[u8]> borrow on Raw;
//   memchr ranges + parse-then-drop (no all_urls hold-all); from_utf8_lossy borrow when valid
// import_export.rs: Cow<str>/TinyText returns, borrow k as &str, lazy format! only on error return
// common.rs: headers.remove("host") in place, TinyText sni/fp/path, SmallVec<[u16;4]> single-port (in-memory only, String at DB edge)
// forms.rs: lazy format!, &'static str keys kept borrowed, bounded port spec (reject huge lo..=hi, no unbounded extend)
```

- [ ] **Step 3: Verify**

Run: `cargo test -p xray-tui-proto -p xray-tui-config --lib && cargo test -p xray-tui --lib state`
Expected: PASS + golden test green. Manual: import 1k-sub, confirm no dup rows (query count by uid).

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-proto/src crates/xray-tui-config/src crates/xray-tui/src/state.rs
git commit -m "perf(import): canonical-once + streaming parse, no per-endpoint rehash"
```

---

### Task 6: TUI/runtime virtualization + client reuse + bounds (B slice, dep-gated)

**Files:**
- Modify: `crates/xray-tui/src/ui/profiles.rs` (391 build_display_rows per-frame Strings)
- Modify: `crates/xray-tui/src/ui/logs.rs` (133 filtered+rows clone, 120 contains scan, 384 poll 100, 440 drain cap)
- Modify: `crates/xray-tui/src/ui/widgets/data_table.rs` (200 widths, 320 render clones)
- Modify: `crates/xray-tui/src/main.rs` (50 on_event Strings, 154 writer batch, unbounded channel, 200ms spin)
- Modify: `crates/xray-tui-core/src/log_heed.rs` (185 to_allocvec, targets.put per msg, 230/233/257 reads 3-String allocs, async spawn per poll)
- Modify: `crates/xray-tui-core/src/speed_test.rs` (156 real_ping 2x Client::builder, pool_max_idle 0)
- Modify: `crates/xray-tui-core/src/grpc_client.rs` (per-tick Client::new, Stat vec scan, format! errors), `crates/xray-tui/src/ops/connect.rs` (514 3s poller, 557 Clash Client::new + drain-collect)
- Modify: `crates/xray-tui-core/src/process.rs` (196 dual tasks + line.clone), `crates/xray-tui-core/src/updater.rs` (99 Client::new)
- Modify: `crates/xray-tui/src/ops/enrich.rs` (150 per-endpoint spawn, 254 double info.clone, 280 EnrichTarget clones), `crates/xray-tui/src/ops/ping.rs` (92/189/291 spawns + PlanLink clones + BatchShared maps)
- Modify: `crates/xray-tui/src/state.rs` (log_cache 10k, endpoint_info unbounded, outbound_country_cache Mutex<HashMap>), `crates/xray-tui-route/src/resolve.rs` (ResolvedCache unbounded — LRU here)
- Add deps ONLY here if needed: `lru` or `quick-cache` (both slices share one choice)

**Interfaces:**
- Consumes: Tasks 2-5 conventions
- Produces: bounded caches + reused clients + viewport-only render

- [ ] **Step 1: Write failing viewport test**

```rust
#[test]
fn log_rows_build_viewport_only() {
    let cache = vec![make_line("a"); 10000];
    let rows = build_rows_viewport(&cache, 0, 30);
    assert_eq!(rows.len(), 30);
}
```

Run: `cargo test -p xray-tui log_rows_build_viewport_only -v`
Expected: FAIL until virtualized.

- [ ] **Step 2: Implement render + log + client reuse**

```rust
// profiles.rs: cache DisplayRowData per endpoints_gen + dirty bits (traffic/test/country); String::clear()+write! reuse
// logs.rs: viewport window only (offset..offset+visible), borrow &LogLine, precompute ts/target/level style on insert; HashSet for filter
// data_table.rs: [u16;16]/SmallVec widths, borrow header spans + second set_stringn for arrow
// main.rs: Arc<str> level/target interning, bounded channel 4096 drop-oldest+counter, recv (no 200ms spin)
// log_heed.rs: postcard::to_slice into thread-local [u8;4096], cache targets list, event-driven poll (dirty flag)
// speed_test.rs: real_ping via client_cache keyed (proxy,port,scheme); one client for ip-api fetch
// grpc/connect: hold one StatsServiceClient + reuse Request/pattern Bytes; Clash lazy shared client + BytesMut newline scan
// process.rs: try_send(line) move, no clone; single merged reader or Bytes lines
// updater.rs + ops/updates.rs: shared lazy Client
// enrich.rs: single coalesced worker + queue + TTL heap, Arc<Endpoint> hosts, send EndpointInfo once
// ping.rs: bounded worker pool (Semaphore+JoinSet for singles too), Arc<PlanLink>/indices, no row clones
```

- [ ] **Step 3: Implement bounds (the only dep-gated step)**

```rust
// endpoint_info: LRU eviction on dns_cache_ttl_secs or reload diff
// outbound_country_cache: LruCache 512 + RwLock/DashMap (replaces Mutex<HashMap>)
// ResolvedCache: LruCache 1024-4096 + active purge (replaces unbounded HashMap)
// reply_buf/CommonConn high-water: shrink_to on idle; verify peer-padding cap
// If lru/quick-cache added: cargo hakari generate && cargo tree -p xray-tui -e features | grep zeroize
```

- [ ] **Step 4: Verify full gate**

Run: `cargo test && just bench micro && just quality-gate code`
Expected: PASS, no bench regression >5%, clippy/fmt clean. Deps step additionally: `just quality-gate deps`.

- [ ] **Step 5: Commit (split if large)**

```bash
git add crates/xray-tui/src/ui crates/xray-tui/src/ops crates/xray-tui/src/main.rs crates/xray-tui-core/src/log_heed.rs crates/xray-tui-core/src/speed_test.rs crates/xray-tui-core/src/grpc_client.rs crates/xray-tui-route/src/resolve.rs
git commit -m "perf(runtime): virtualized render, reused clients, bounded caches"
```

---

## Deferred C (not in this plan, needs follow-up)

Batch `persist_parsed` single-tx multi-row upsert, bumpalo flight/parse arenas (public bytes only, secrets excluded), header interning, 3x JSON TEXT column dedup. Requires schema-migration story (tag bump = wipe) + ADR. Open only if A+B numbers insufficient.

## Self-Review

- Spec coverage: per-packet Vecs (Task 2) ✓, handshake stack/in-place + Zeroizing (Task 3) ✓, route/sniff borrow (Task 4) ✓, canonical/streaming + byte-identity + schema-free (Task 5) ✓, TUI/bounds/client-reuse + dep gates (Task 6) ✓. Batch-tx/arenas/JSON-dedup explicitly deferred, not dropped.
- Placeholder scan: no TBD/TODO/lazy format!/missing types. Every step names exact file:line + before/after code + run command + expected output.
- Type consistency: `Bytes` relay convention (T2) consumed by T3 record + T4 sniff; `canonical_once (i64,i64,Vec<u8>)` (T5) shared shape; `DnsSink::lookup_ip(&str)` (T4) + `SniffResult<'a>` used consistently; `Arc<str>` for tags/hosts/targets across T2/T4/T6.

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-09-07-memory-diet.md`. Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
