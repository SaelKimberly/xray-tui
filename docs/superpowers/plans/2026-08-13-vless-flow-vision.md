# VLESS Flow (xtls-rprx-vision) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement XTLS Vision (`xtls-rprx-vision`) flow for the VLESS client in `xray-tui-native`, interoperable with mainstream xray-core 26.3.27 and sing-box 1.13.16 servers, with the inner TLS session raw-relayed after a splice point.

**Architecture:** The engine `TlsStream` gains per-direction direct mode (`set_write_direct`/`set_read_direct`) — a generic record-layer bypass. A new vision codec in the native crate (`protocol/vless/vision.rs`) ports xray's padding/unpadding/TLS-filter state machine; the VLESS header gains the protobuf flow addon; on the Direct command both directions switch to raw socket reads/writes (the outer TLS is abandoned, one inner TLS session remains on the wire).

**Tech Stack:** tokio, ring (`ring::rand` for padding RNG — no new deps), bytes, xray-tui-tls (engine), xray-tui-native (codec), rstest e2e against real core binaries.

**Spec:** `docs/superpowers/specs/2026-08-13-vless-flow-vision-design.md` (wire authority; every byte claim below quotes it or the xray sources it cites).

## Global Constraints

- xray-tui-proto NEVER modified; xray-tui-tls changes limited to `record/stream.rs` direct-mode methods + tests.
- No new `rand`/`unsafe` dependencies. Padding RNG = `ring::rand::{SystemRandom, SecureRandom}` (existing pattern from xhttp `x_padding`).
- `NativeError`-only error surface in the native crate; no new variants (reuse `NotImplemented`, config/IO variants).
- Clippy workspace pedantic+nursery: 0 warnings. `cargo fmt --check` clean.
- e2e requires feature `native-e2e` AND `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin` (xray 26.3.27 + sing-box 1.13.16).
- Wire contract (spec §4): frames `[16B UUID (first only)][1B cmd][2B BE content_len][2B BE pad_len][content][zero pad]`; cmds `0x00` Continue / `0x01` End / `0x02` Direct; padding long `900+rand(500)-content` when `content<900 && longPadding`, else `rand(256)`, cap `8171-content`; filter budget 8; `EnableXtls` = inner TLS1.3 (`{0x00 0x2b 0x00 0x02 0x03 0x04}` in ServerHello) && cipher ∉ {unknown, `0x1305`}.
- Direct transition writes: the Direct frame is the LAST outer-TLS record; trailing bytes of the same write go raw immediately (sing-vmess model, spec §9.2 — interoperates with both servers).
- Direct transition reads: the reader must have fully consumed the Direct frame; engine record layer is then at a clean boundary (verified no read-ahead, spec §5.2).

---

### Task 1: Engine per-direction direct mode

**Files:**
- Modify: `crates/xray-tui-tls/src/record/stream.rs`

**Interfaces:**
- Produces: `impl<S: AsyncRead + AsyncWrite + Unpin + Send> TlsStream<S> { pub fn set_write_direct(&mut self); pub fn set_read_direct(&mut self); }` — after `set_write_direct()`, `poll_write`/`poll_flush`/`poll_shutdown` forward to `inner` unchanged; after `set_read_direct()`, `poll_read` reads from `inner` directly (no record parsing, no decryption, no NST skipping). Both default false. Caller owns the transition invariants (write: prior record flushed; read: clean boundary, empty `read_buf`).

- [ ] **Step 1: Add the flags and methods**

```rust
// In TlsStream<S> struct (record/stream.rs, near read_buf/rec fields):
    write_direct: bool,
    read_direct: bool,

// In new(): initialize both to false.

impl<S: AsyncRead + AsyncWrite + Unpin + Send> TlsStream<S> {
    /// Switch the write side to direct raw writes to the underlying stream.
    /// Caller must have fully written and flushed the last TLS record first.
    pub fn set_write_direct(&mut self) { self.write_direct = true; }

    /// Switch the read side to direct raw reads from the underlying stream.
    /// Caller must have consumed all decrypted bytes; the record state must
    /// be at a clean record boundary (empty read_buf, rec = Header{filled: 0}).
    pub fn set_read_direct(&mut self) { self.read_direct = true; }
}
```

- [ ] **Step 2: Forward in the trait impls**

```rust
// In impl AsyncRead for TlsStream<S>:
    fn poll_read(...) -> Poll<io::Result<usize>> {
        if self.read_direct {
            return Pin::new(&mut self.inner).poll_read(cx, buf);
        }
        // ... existing record path unchanged
    }

// In impl AsyncWrite for TlsStream<S>:
    fn poll_write(...) -> Poll<io::Result<usize>> {
        if self.write_direct {
            return Pin::new(&mut self.inner).poll_write(cx, buf);
        }
        // ... existing record path unchanged
    }
    fn poll_flush(...) -> Poll<io::Result<()>> {
        if self.write_direct {
            return Pin::new(&mut self.inner).poll_flush(cx);
        }
        // ... existing path
    }
    fn poll_shutdown(...) -> Poll<io::Result<()>> {
        if self.write_direct {
            return Pin::new(&mut self.inner).poll_shutdown(cx);
        }
        // ... existing path
    }
```

Note: `self.inner` is already pinned via the existing code (`Pin::new(&mut self.inner)`); keep the same pinning approach. If the existing impls borrow `self.rec`/`self.read_buf` mutably in poll_read, structure the direct check BEFORE those borrows.

- [ ] **Step 3: Write the tests (in the existing `mod tests` of record/stream.rs)**

Existing tests already build a TLS client+server pair over `tokio::io::duplex` — reuse that fixture pattern (see tests at lines ~480-620: `client`/`server` TlsStreams over a duplex).

```rust
#[tokio::test]
async fn direct_write_bypasses_record_layer() {
    // Build client+server TlsStream pair over a duplex (reuse existing fixture helper).
    // 1. Encrypted phase: client writes "hello", server reads "hello".
    // 2. client.set_write_direct();
    // 3. Client writes raw b"RAW-BYTES" (NOT a TLS record).
    // 4. Server's TlsStream::poll_read must FAIL to parse it as a record
    //    (invalid header or decrypt error) — proving the bytes were not
    //    encrypted. Read them via the duplex's other side instead:
    //    the raw half of the duplex returns b"RAW-BYTES" verbatim.
    // (Use the duplex handle directly for the raw assertion.)
}

#[tokio::test]
async fn direct_read_bypasses_record_layer() {
    // 1. Encrypted phase: server writes "hello", client reads "hello".
    // 2. client.set_read_direct();
    // 3. Server writes raw b"RAW" (via server.set_write_direct() or the
    //    duplex handle).
    // 4. client reads exactly b"RAW" — no record parsing, no decrypt.
}

#[tokio::test]
async fn direct_transition_at_record_boundary_loses_nothing() {
    // Encrypted phase, then:
    // server.set_write_direct(); server writes a raw payload immediately
    // after its last encrypted record (pipelined).
    // client reads the last encrypted record fully, THEN set_read_direct(),
    // THEN reads the raw payload — the kernel/duplex buffer preserved it
    // (no read-ahead loss). Assert both bytes arrive intact.
}

#[tokio::test]
async fn write_direct_requires_flush_first() {
    // Partial-write path: client writes a large payload that splits across
    // records; flush completes it; then set_write_direct; the raw phase
    // starts cleanly. Assert ordering: server sees the encrypted bytes in
    // order, then the raw bytes.
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p xray-tui-tls record::stream`
Expected: all pass (new 4 + existing).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-tls/src/record/stream.rs
git commit -m "feat(tls): per-direction direct mode on TlsStream (vision splice support)"
```

---

### Task 2: Vision codec

**Files:**
- Create: `crates/xray-tui-native/src/protocol/vless/vision.rs`
- Modify: `crates/xray-tui-native/src/protocol/vless/mod.rs` (declare `mod vision;`)

**Interfaces:**
- Consumes: nothing from Task 1 at compile time (codec is generic over `S: AsyncRead + AsyncWrite + Unpin`; unit tests use `tokio::io::duplex`).
- Produces (consumed by Task 3):

```rust
pub(crate) const FLOW_XTLS_RPRX_VISION: &str = "xtls-rprx-vision";
pub(crate) const CMD_CONTINUE: u8 = 0x00;
pub(crate) const CMD_END: u8 = 0x01;
pub(crate) const CMD_DIRECT: u8 = 0x02;

/// Client-side vision stream: pads uplink writes, unpads downlink reads,
/// sniffs the inner TLS, and switches to raw socket relay on Direct.
pub(crate) struct VisionStream<S> { /* inner: S, state, buffers */ }

impl<S: AsyncRead + AsyncWrite + Unpin> VisionStream<S> {
    /// `user_uuid` = the VLESS user id bytes. The caller MUST have already
    /// written the VLESS header AND the camouflage frame (Task 3) before
    /// wrapping; the writer's UUID is consumed by the camouflage frame.
    pub(crate) fn new(inner: S, user_uuid: [u8; 16], rng: ring::rand::SystemRandom) -> Self;

    /// The header-camouflage frame bytes: one Continue long-padding frame
    /// carrying the UUID. Caller writes it via the raw stream right after
    /// the VLESS header (spec §4.6 step 3, deviation 1).
    pub(crate) fn camouflage_frame(user_uuid: &[u8; 16], rng: &ring::rand::SystemRandom) -> bytes::Bytes;
}

impl<S: AsyncRead + AsyncWrite + Unpin> tokio::io::AsyncRead for VisionStream<S> { /* ... */ }
impl<S: AsyncRead + AsyncWrite + Unpin> tokio::io::AsyncWrite for VisionStream<S> { /* ... */ }
```

- [ ] **Step 1: Write the frame codec + padding (with tests)**

Constants (spec §4): `TLS_APP_DATA_START = [0x17, 0x03, 0x03]`, `TLS_SERVER_HELLO_START = [0x16, 0x03, 0x03]`, `TLS_HANDSHAKE_TYPE_CLIENT_HELLO = 0x01`, `TLS_HANDSHAKE_TYPE_SERVER_HELLO = 0x02`, `TLS13_SUPPORTED_VERSIONS = [0x00, 0x2b, 0x00, 0x02, 0x03, 0x04]`, `MAX_FRAME = 8192` (xray `buf.Size`), `FRAME_OVERHEAD = 21` (16 uuid + 1 cmd + 2 content + 2 pad), `FILTER_BUDGET = 8`.

```rust
struct FrameHeader { cmd: u8, content_len: u16, pad_len: u16 }

/// Encode one padded frame: [uuid?][cmd][clen:2][plen:2][content][zeros].
/// `uuid: Option<&[u8;16]>` — Some only for the first frame of a direction.
/// Returns the frame bytes. Panics if content.len() + 21 + pad_len > 8192
/// (caller must reshape chunks >= 8171 first).
fn encode_frame(uuid: Option<&[u8; 16]>, cmd: u8, content: &[u8], pad_len: usize) -> Vec<u8>;

/// XtlsPadding port (spec §4.2): compute pad_len for the given content.
/// long = content.is_empty() || (traffic.is_tls && content.len() < 900)
fn padding_len(content_len: usize, long_padding: bool, rng: &ring::rand::SystemRandom) -> usize {
    let raw = if long_padding { 900 + rand_u32(rng, 500) - content_len as u32 } else { rand_u32(rng, 256) };
    raw.min((MAX_FRAME - FRAME_OVERHEAD - content_len) as u32) as usize
}

/// rand_u32(rng, bound): rejection-sample [0, bound) from ring::rand fills
/// (no modulo bias; mirror the xhttp x_padding pattern).
fn rand_u32(rng: &ring::rand::SystemRandom, bound: u32) -> u32;
```

Tests (in-file `#[cfg(test)]`):
- `encode_frame` round-trip: exact bytes for a known input (cmd=CONTINUE, content=b"abc", pad=2, uuid Some → assert `[uuid][0][0 3][0 2]abc\x00\x00`).
- `padding_len`: long branch with content 0 → `900..=1399` (rejection-sampled, assert bounds); plain branch → `< 256`; cap: content 8170 → pad 0; content 8000 → pad ≤ 171.
- Content ≥ 8171: `encode_frame` panics (assert `#[should_panic]`) — the reshape guard.

- [ ] **Step 2: Write the unpad state machine (with tests)**

```rust
/// XtlsUnpadding port (spec §4.4). Splits an incoming chunk into unpadded
/// content bytes; the Remaining* state carries partial frames across chunks.
struct Unpadder {
    user_uuid: [u8; 16],
    remaining_command: i32, // 5 = awaiting cmd, 4..1 = awaiting len bytes
    remaining_content: i32,
    remaining_padding: i32,
    current_command: u8,
    within_padding: bool,  // false once End/Direct seen (or UUID gate failed)
    direct: bool,          // true after a Direct frame
    uuid_gate_passed: bool, // false until first >=21-byte chunk matched UUID
    plain_passthrough: bool, // true after a UUID-gate mismatch → never unpad
}

impl Unpadder {
    fn new(user_uuid: [u8; 16]) -> Self; // within_padding=true, remaining* = -1

    /// Feed one decrypted chunk; returns unpadded content (possibly empty).
    /// On `direct`, the caller must switch the stream to raw reads and stop
    /// feeding this unpadder (spec §4.4).
    fn feed(&mut self, chunk: &[u8], out: &mut Vec<u8>);
}
```

Port `XtlsUnpadding` exactly (spec §4.4 / proxy/proxy.go:535-617):
- Initial state (`remaining == (-1,-1,-1)`): if `chunk.len() >= 21 && chunk[0..16] == user_uuid` → consume 16, `remaining_command = 5`, `uuid_gate_passed = true`; else → `plain_passthrough = true`, append the whole chunk to `out` unchanged, return.
- Once `uuid_gate_passed`: loop over the chunk: while `remaining_command > 0` take 1 byte (cmd → content_hi → content_lo → pad_hi → pad_lo, i.e. `current_command` on the first, `remaining_content = hi<<8` etc.); while `remaining_content > 0` move `min(rem, chunk_left)` bytes to `out`; else skip `min(remaining_padding, chunk_left)` bytes. When a block completes (`remaining_command <= 0 && remaining_content <= 0 && remaining_padding <= 0`): `current_command == CMD_CONTINUE` → `remaining_command = 5` (next block); else → reset remaining* to `-1`; if `current_command == CMD_END` → `within_padding = false`; if `current_command == CMD_DIRECT` → `within_padding = false`, `direct = true`; append any leftover chunk bytes to `out` (Go: "shouldn't happen", defensive).

Tests:
- Single complete frame unpads to content.
- Continue chain across chunk boundaries (split a 2-frame chunk mid-frame; feed halves; assert full content).
- UUID gate: chunk without UUID prefix → passthrough verbatim; then `within_padding == false` forever.
- `[0,0]` (server response header) prefix chunk → passthrough (len < 21).
- End frame → content delivered, `within_padding = false`, no `direct`.
- Direct frame → content delivered, `direct = true`.

- [ ] **Step 3: Write the filter (with tests)**

```rust
/// XtlsFilterTls port (spec §4.5): sniffs inner TLS records to set
/// is_tls / is_tls12_or_above / enable_xtls.
struct TlsFilter {
    budget: i32,               // 8
    is_tls: bool,
    is_tls12_or_above: bool,
    remaining_server_hello: i32, // -1 = not awaiting a ServerHello body
    cipher: u16,
    enable_xtls: bool,
}
impl TlsFilter {
    fn new() -> Self; // budget = FILTER_BUDGET, remaining_server_hello = -1
    fn feed(&mut self, chunk: &[u8]);
}
```

Port `XtlsFilterTls` exactly (spec §4.5 / proxy/proxy.go:619-671):
- If `budget <= 0` return. `budget -= 1`.
- If `chunk.len() >= 6`:
  - `chunk[0..3] == TLS_SERVER_HELLO_START && chunk[5] == TLS_HANDSHAKE_TYPE_SERVER_HELLO` → `is_tls12_or_above = true; is_tls = true; remaining_server_hello = ((chunk[3] as i32) << 8 | chunk[4] as i32) + 5;` and if `chunk.len() >= 79 && remaining_server_hello >= 79` → `sid_len = chunk[43]`; `cipher = (chunk[43+sid_len+1] as u16) << 8 | chunk[43+sid_len+2] as u16`.
  - else if `chunk[0..2] == [0x16, 0x03] && chunk[5] == TLS_HANDSHAKE_TYPE_CLIENT_HELLO` → `is_tls = true`.
- If `remaining_server_hello > 0`: `end = min(remaining_server_hello, chunk.len() as i32)`; `remaining_server_hello -= chunk.len() as i32`; if `chunk[..end as usize]` contains `TLS13_SUPPORTED_VERSIONS` as a contiguous sub-slice → `enable_xtls = matches!(cipher, 0x1301 | 0x1302 | 0x1303 | 0x1304)` (i.e. known TLS1.3 suite other than CCM_8 0x1305); `budget = 0; return`. Else if `remaining_server_hello <= 0` → `budget = 0; return` (TLS 1.2).

Tests:
- ClientHello chunk → `is_tls`, budget decremented.
- Full TLS1.3 ServerHello (craft a ≥79-byte record: `16 03 03 len(2B) 02` + session_id_len byte + 32-byte session id + cipher bytes + a `supported_versions` extension containing `00 2b 00 02 03 04`): → `is_tls12_or_above`, `enable_xtls = true` (cipher 0x1301), budget 0.
- Same with cipher 0x1305 → `enable_xtls = false`.
- TLS1.2 ServerHello (no supported_versions marker, exhausted) → `enable_xtls = false`, budget 0.
- 8 non-TLS chunks → budget exhausted, no flags.

- [ ] **Step 4: Write the VisionStream (writer + reader) with the shared state**

```rust
pub(crate) struct VisionStream<S> {
    inner: S,
    uuid: [u8; 16],
    rng: ring::rand::SystemRandom,
    writer: WriterState,   // is_padding=true, direct=false, uuid_written=true (camouflage consumed it)
    reader: ReaderState,   // Unpadder + direct flag
    filter: TlsFilter,     // shared — fed by BOTH directions (spec §4.5)
    write_buf: bytes::BytesMut,  // pending frame bytes not yet flushed to inner
    read_buf: bytes::BytesMut,   // unpadded content awaiting the app
    raw_leftover: bytes::BytesMut, // bytes after a Direct frame (defensive)
    accepted: usize,       // app bytes accepted by the in-flight write
}

struct WriterState { is_padding: bool, direct: bool }
struct ReaderState { unpad: Unpadder, direct: bool }
```

**AsyncWrite::poll_write** (contract: never returns `Ok(0)`; returns the count of accepted app bytes; buffers internally; forwards Pending with the inner's waker):

```rust
fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
    let this = &mut *self;
    // 1. Direct mode: raw forward.
    if this.writer.direct {
        return Pin::new(&mut this.inner).poll_write(cx, buf);
    }
    // 2. Flush a pending frame first (a previous chunk's bytes).
    if !this.write_buf.is_empty() {
        let n = ready!(flush_frame(this, cx))?;   // helper: Pin inner.poll_write until write_buf drained
        if !this.write_buf.is_empty() { return Poll::Pending; } // still flushing
        let accepted = this.accepted;
        this.accepted = 0;
        return Poll::Ready(Ok(accepted));
    }
    // 3. Process the new chunk: filter, then pad per the writer state machine.
    this.filter.feed(buf);
    let (frames, raw_tail, direct) = this.pad_chunk(buf); // Vec<Vec<u8>> frames, Vec<u8> tail, bool
    // 4. Write the frames through the record layer.
    let mut written = 0;
    for f in &frames {
        match Pin::new(&mut this.inner).poll_write(cx, f) {
            Poll::Ready(Ok(n)) => written += n,
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Pending => {
                this.write_buf.extend_from_slice(&f[written..]);
                this.accepted = buf.len();
                return Poll::Pending;
            }
        }
    }
    // 5. After the Direct frame: flush it, then switch the write side raw.
    if direct {
        ready!(flush_all(this, cx))?;
        this.inner.set_write_direct(); // TlsStream method (Task 1)
        this.writer.direct = true;
    }
    // 6. Raw tail (bytes after the Direct frame) written raw immediately.
    if !raw_tail.is_empty() {
        // If direct just switched, inner is already raw.
        let n = ready!(Pin::new(&mut this.inner).poll_write(cx, &raw_tail))?;
        debug_assert_eq!(n, raw_tail.len()); // retry loop if partial (use a write_all helper)
    }
    Poll::Ready(Ok(buf.len()))
}
```

`pad_chunk` (writer state machine, spec §4.3, sing-vmess model — the Direct frame is the last padded frame; trailing bytes raw):

```rust
fn pad_chunk(&mut self, chunk: &[u8]) -> (Vec<Vec<u8>>, Vec<u8>, bool) {
    let mut out = Vec::new();
    let mut raw_tail = Vec::new();
    let mut direct = false;
    if !self.writer.is_padding {
        // is_padding went false via End earlier: chunk passes through the
        // record layer unpadded (outer TLS continues).
        out.push(chunk.to_vec());
        return (out, raw_tail, direct);
    }
    // Reshape chunks >= 8171 at the last 0x17 0x03 0x03 boundary (ReshapeMultiBuffer,
    // spec §4.3) so each piece keeps the frame cap.
    let pieces: Vec<&[u8]> = if chunk.len() >= MAX_FRAME - FRAME_OVERHEAD {
        reshape(chunk)   // split at last TLS_APP_DATA_START occurrence
    } else {
        vec![chunk]
    };
    let long_padding = self.filter.is_tls;
    for (i, piece) in pieces.iter().enumerate() {
        let is_last = i == pieces.len() - 1;
        let is_app_data = piece.len() > 6 && piece.starts_with(&TLS_APP_DATA_START);
        if self.filter.is_tls && is_app_data {
            // Direct (if enabled) or End — this is the last padded frame.
            let (cmd, d) = if self.filter.enable_xtls { (CMD_DIRECT, true) } else { (CMD_END, false) };
            out.push(encode_frame(None, cmd, piece, padding_len(piece.len(), true, &self.rng)));
            self.writer.is_padding = false;
            if d {
                direct = true;
                raw_tail.extend_from_slice(&pieces[i + 1..].concat());
            }
            return (out, raw_tail, direct);
        }
        if !self.filter.is_tls12_or_above && self.filter.budget <= 1 {
            // Non-TLS / TLS1.2 early finish: End frame, rest unpadded via TLS.
            out.push(encode_frame(None, CMD_END, piece, padding_len(piece.len(), long_padding, &self.rng)));
            self.writer.is_padding = false;
            for rest in &pieces[i + 1..] { out.push(rest.to_vec()); }
            return (out, raw_tail, direct);
        }
        let cmd = if is_last && !self.writer.is_padding { CMD_END } else { CMD_CONTINUE };
        // (is_padding is only false after the branches above; this generic
        // branch always runs while is_padding, so cmd is CMD_CONTINUE unless
        // the state flipped mid-chunk, which cannot happen here.)
        out.push(encode_frame(None, cmd, piece, padding_len(piece.len(), long_padding, &self.rng)));
    }
    (out, raw_tail, direct)
}
```

**AsyncRead::poll_read** (spec §4.4):

```rust
fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
    let this = &mut *self;
    if this.reader.direct {
        // Drain defensive leftover first, then raw reads.
        if !this.raw_leftover.is_empty() { copy out min; return Ready; }
        return Pin::new(&mut this.inner).poll_read(cx, buf);
    }
    if !this.read_buf.is_empty() { copy out min(this.read_buf.len(), buf.remaining()); return Ready; }
    // Read a chunk from the inner (record layer), unpad it, retry.
    let mut chunk = [0u8; 16384];
    let mut rb = ReadBuf::new(&mut chunk);
    ready!(Pin::new(&mut this.inner).poll_read(cx, &mut rb))?;
    let n = rb.filled().len();
    if n == 0 { return Poll::Ready(Ok(())); } // EOF
    let data = &chunk[..n];
    if this.reader.unpad.plain_passthrough {
        // Early-out: passthrough mode delivers everything unchanged.
        this.read_buf.extend_from_slice(data);
        copy out min(...); return Ready;
    }
    let mut content = Vec::with_capacity(data.len());
    this.reader.unpad.feed(data, &mut content);
    if this.reader.unpad.direct {
        this.reader.direct = true;
        this.reader.unpad.direct = false;
        // Any bytes the unpadder appended after the Direct frame are the
        // start of the raw stream.
        this.raw_leftover.extend_from_slice(&content);
        // Switch the read side to raw now; TlsStream must be at a clean
        // record boundary (it is: the Direct frame filled the last record,
        // spec §5.2).
        this.inner.set_read_direct();
    }
    this.read_buf.extend_from_slice(&content);
    copy out min(...); return Ready;
}
```

Notes for the implementer:
- `ready!` requires the std `task::ready` macro (Rust 1.97 has it; the crate already uses `std::task::Poll`).
- The filter must be fed on BOTH directions (`writer.pad_chunk` and the reader path both call `this.filter.feed`), sharing one `TlsFilter` — the budget is per-connection, and `EnableXtls` is set by the DOWNLINK ServerHello sniff (spec §4.5-4.6).
- `TlsStream`'s `set_write_direct`/`set_read_direct` are available on the concrete inner type — `VisionStream<S>` calls them only when `S: TlsStream`-compatible. **Design decision:** make `VisionStream<S>` generic and call the direct-mode methods through a small sealed trait:

```rust
pub(crate) trait DirectMode {
    fn set_write_direct(&mut self);
    fn set_read_direct(&mut self);
}
impl<S: AsyncRead + AsyncWrite + Unpin + Send> DirectMode for xray_tui_tls::TlsStream<S> {
    fn set_write_direct(&mut self) { self.set_write_direct(); }
    fn set_read_direct(&mut self) { self.set_read_direct(); }
}
```

(`xray-tui-tls` re-exports `TlsStream`; check the exact path in `crates/xray-tui-tls/src/lib.rs`.) The reader/writer direct paths call `this.inner.set_write_direct()` / `set_read_direct()` via this trait bound `S: DirectMode`. Unit tests use a plain duplex WITHOUT the trait — add a `DirectMode` impl for duplex (or gate the direct paths so the codec still works over non-directable streams: if `direct` fires but the stream can't switch, keep forwarding through `inner` — for tests, the fake server test asserts frames only). **Simplest correct choice:** make the direct transition conditional on a `directable: bool` flag set by the constructor; `VisionStream::new` sets it `true` (real path), a test constructor sets it `false` (codec behavior testable over duplex without the engine). The Direct path then just stops padding (writes forward through `inner` — for the duplex test the "raw" bytes are readable).

- [ ] **Step 5: Run the tests**

Run: `cargo test -p xray-tui-native protocol::vless::vision`
Expected: all vision tests pass; crate still compiles.

- [ ] **Step 6: Commit**

```bash
git add crates/xray-tui-native/src/protocol/vless/vision.rs crates/xray-tui-native/src/protocol/vless/mod.rs
git commit -m "feat(native): VLESS vision codec — padded frames, TLS filter, Direct splice state machine"
```

---

### Task 3: VLESS wiring — header addon, flow guard, camouflage

**Files:**
- Modify: `crates/xray-tui-native/src/protocol/vless/header.rs`
- Modify: `crates/xray-tui-native/src/protocol/vless/mod.rs`
- Modify: `crates/xray-tui-native/src/protocol/vless/stream.rs` (only if composition requires it)

**Interfaces:**
- Consumes: Task 2 (`FLOW_XTLS_RPRX_VISION`, `VisionStream`, `camouflage_frame`); Task 1 (TlsStream direct methods via `DirectMode`).
- Produces: `connect()` in `mod.rs` returns the VLESS stream with vision active for `flow == "xtls-rprx-vision"`.

- [ ] **Step 1: Header addon encoding**

Read `header.rs` — `encode_request(uuid, target, cmd)` currently emits `version | uuid | addon_len=0 | cmd | port | addr` (spec §4.1). Extend:

```rust
/// Protobuf addons bytes for the flow (spec §4.1): field 1 (string Flow),
/// tag 0x0A, length 0x10, 16 bytes "xtls-rprx-vision" → 18 bytes total.
/// Returns None for empty/unknown flows (addon_len stays 0).
pub(crate) fn encode_addons(flow: Option<&str>) -> Option<Vec<u8>> {
    flow.filter(|f| *f == FLOW_XTLS_RPRX_VISION).map(|f| {
        let mut b = Vec::with_capacity(18);
        b.push(0x0A);
        b.push(16);
        b.extend_from_slice(f.as_bytes());
        b
    })
}

// encode_request gains `addons: Option<&[u8]>`:
//   addon_len byte = addons.map_or(0, |a| a.len() as u8)
//   then the addons bytes (if any), then cmd | port | addr as before.
```

Unit tests (in `header.rs`):
- `encode_addons(Some("xtls-rprx-vision"))` == `[0x0A, 0x10] + 16 bytes`.
- `encode_addons(Some("xtls-rprx-splice"))` == None; `encode_addons(None)` == None.
- `encode_request` with addons: header bytes == `00` + uuid + `18` + addons + cmd + port + addr; without: `00` + uuid + `00` + cmd + port + addr (unchanged).

- [ ] **Step 2: Flow guard + vision connect path**

In `mod.rs` `connect()` (the guard today rejects every non-empty flow with `NotImplemented`, lines ~33-41):

```rust
match cfg.flow.as_deref() {
    None | Some("") => connect_plain(...),        // existing path
    Some(FLOW_XTLS_RPRX_VISION) => connect_vision(...),
    Some(other) => return Err(NativeError::NotImplemented(format!(
        "VLESS flow {other:?} is not supported by the native core"
    ))),
}
```

`connect_vision` (read `connect()` first and refactor minimally — it already does dial → security → header-write → return stream):

1. **Security check** — reuse the `has_tls()` helper from `context.rs` (spec §5.4; established in the transports work): if `!has_tls()` → `Err(NativeError::Config("XTLS only supports TLS and REALITY directly for now".into()))` (mirror xray's error text, spec §5.4).
2. **Transport check** — if `ctx.transport_type() != TransportType::Tcp` (or however transport is expressed; see `context.rs`) → same `NativeError::Config` (vision requires the raw TCP socket for the Direct handoff; spec §5.1).
3. **Command check** — the current path targets TCP; if the request command is UDP → `Err(NativeError::NotImplemented("VLESS vision does not support UDP (no XUDP mux)".into()))` (spec §2). (If the codebase has no UDP path at all, assert/handle at the target level — read `mod.rs` and do the minimal correct thing, documented in the commit.)
4. Dial + security wrap exactly as today.
5. Write the request header via `encode_request(..., encode_addons(Some(flow)).as_deref())`.
6. Write the camouflage frame (spec §4.6 step 3, deviation 1):

```rust
let camo = VisionStream::camouflage_frame(&uuid, &rng);
stream.write_all(&camo).await?;
```

(`uuid` is the 16-byte VLESS user id already available in `connect()`; `rng` = `ring::rand::SystemRandom::new()`.)
7. Wrap: `let vision = VisionStream::new(stream, uuid, rng);` and return it where the plain stream is returned today (the caller's stream type: read `mod.rs` — if `connect()` returns `Box<dyn AsyncReadWrite>`, box the vision stream; keep the return type unchanged).

`stream.rs` (`VlessClientStream`) — read it: the lazy response-header peel stays (spec §5.4). If `connect_vision` returns the vision-wrapped stream through the same `VlessClientStream` wrapper, nothing changes there; if the peel is inside `VlessClientStream` and it wraps the tunnel stream, the vision stream is inserted between them (peel reads via the vision stream — the server's `[0,0]` passes the UUID gate, spec §5.4). Verify the composition compiles and the peel still sees the `[0,0]` header.

- [ ] **Step 3: Compile + existing tests**

Run: `cargo test -p xray-tui-native`
Expected: all existing unit tests still pass (no e2e yet — vision rows come in Task 6). Clippy: `cargo clippy -p xray-tui-native --all-targets` → 0.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-native/src/protocol/vless/header.rs crates/xray-tui-native/src/protocol/vless/mod.rs crates/xray-tui-native/src/protocol/vless/stream.rs
git commit -m "feat(native): wire VLESS vision — flow addon header, guard, camouflage frame"
```

---

### Task 4: Hermetic fake-vision-server test

**Files:**
- Modify: `crates/xray-tui-native/src/protocol/vless/vision.rs` (add an integration-style `#[cfg(test)]` module or follow the xhttp hermetic pattern — check `crates/xray-tui-native/src/transport/xhttp.rs` tests for the established raw-TCP fake-server style and mirror it)

**Interfaces:**
- Consumes: Task 3 (`connect()` vision path is the unit under test; or test the codec directly over the raw socket — implementer picks whichever the xhttp pattern supports).

- [ ] **Step 1: Write the hermetic test**

Follow the xhttp hermetic pattern (`transport/xhttp.rs` test module): a raw `TcpListener` fake server speaking the vision wire, driven in a `#[tokio::test]`:

1. Fake server accepts a TCP connection.
2. Performs the outer TLS as the SERVER side of the engine — use the crate's existing rustls server test double (check `xray-tui-tls`'s test server or `xray-tui-native`'s existing hermetic TLS fixtures; the httpupgrade hermetic test already has a TLS-serving fixture — reuse it).
3. Reads the VLESS header: assert `version == 0`, `addons_len == 18`, addons == `0A 10 "xtls-rprx-vision"`, command == TCP, target matches.
4. Reads the camouflage frame: assert UUID prefix == the user uuid, cmd == CONTINUE, content_len == 0.
5. Sends the response header `[0,0]` raw.
6. Reads a padded app frame: assert `[uuid?][cmd][clen][plen]` layout, clen == the sent content, padding within bounds, content matches.
7. Sends a padded frame with the server's UUID + cmd END; asserts the client delivers the content.
8. Optionally: exercises the Direct path with a crafted ServerHello to flip the client's EnableXtls — the client then stops padding. **This is the e2e task's job** (Task 6 proves Direct with real TLS); the hermetic test asserts the frame-level contract (1-7) and the camera/uuid/filter basics. If step 8 is straightforward with a pre-crafted TLS1.3 ServerHello buffer, include it; otherwise defer Direct to Task 6.

Run: `cargo test -p xray-tui-native protocol::vless::vision`
Expected: green.

- [ ] **Step 2: Commit**

```bash
git add crates/xray-tui-native/src/protocol/vless/vision.rs
git commit -m "test(native): hermetic fake-vision-server — header, camouflage, padded frames"
```

---

### Task 5: e2e harness — flow axis

**Files:**
- Modify: `crates/xray-tui-native/src/e2e/case.rs` (CaseSpec)
- Modify: `crates/xray-tui-native/src/e2e/config.rs` (server + client JSON builders)

**Interfaces:**
- Consumes: nothing new from earlier tasks (harness-only).
- Produces: `CaseSpec::flow` field + config emission used by Task 6.

- [ ] **Step 1: CaseSpec flow axis**

Read `case.rs` (CaseSpec fields: `protocol, security, tls, network, xhttp_mode`). Add:

```rust
/// VLESS flow control; only `xtls-rprx-vision` is implemented (None = none).
pub flow: Option<Flow>,
```

with `Flow` (or reuse a string — match the file's conventions; the `xhttp_mode` precedent suggests a small enum):

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow { Vision }
impl Flow { pub fn as_str(&self) -> &'static str { "xtls-rprx-vision" } }
```

Update `label()` to include the flow (e.g. `{proto}/{flow}/{network}/{tls}{sec}` — check the current format and extend it consistently so test names stay unique). Update the `label` expectations in any existing snapshot/assert (read the file).

- [ ] **Step 2: Server config flow emission**

Read `config.rs`: `vless_inbound(...)` + `vless_reality_inbound(...)` build the xray server JSON (`settings.clients[]`) and sing-box server JSON (`users[]`). Add flow emission:
- xray: `"flow": "xtls-rprx-vision"` in `settings.clients[0]` (spec §7.4) — only when the case has a flow.
- sing-box: `"flow": "xtls-rprx-vision"` in `users[0]`.
- Client JSON (`client_params_vless`, ~line 575): emit `"flow": "xtls-rprx-vision"` in the VLESS outbound (the native client parses `VlessConfig.flow` — proto already deserializes it, spec §5.4; verify the JSON key the native client expects — read the client config parsing in `crates/xray-tui-native/src/context.rs`/wherever `VlessConfig` is consumed).

- [ ] **Step 3: Unit-test the config emission**

Read how existing config builders are tested (config.rs has unit tests asserting the generated JSON — follow them): a `flow: Some(Vision)` case emits the flow in xray clients, sing-box users, and the client JSON; a `None` case emits nothing (existing assertions unchanged).

Run: `cargo test -p xray-tui-native e2e`
Expected: green.

- [ ] **Step 4: Commit**

```bash
git add crates/xray-tui-native/src/e2e/case.rs crates/xray-tui-native/src/e2e/config.rs
git commit -m "feat(native): e2e harness flow axis — CaseSpec + server/client JSON emission"
```

---

### Task 6: e2e vision rows + inner-TLS app

**Files:**
- Modify: `crates/xray-tui-native/tests/vless.rs`
- Modify: `crates/xray-tui-native/src/e2e/` (inner-TLS app helper + rustls target; exact file per the harness layout)
- Modify: `crates/xray-tui-native/Cargo.toml` (dev-deps for the e2e feature)

**Interfaces:**
- Consumes: Task 5 (`CaseSpec::flow`, config emission).

- [ ] **Step 1: Inner-TLS app + rustls target**

The e2e harness: native client (NativeConnectParams) → core server (xray/sing-box) → target. Read the harness (`src/e2e/`) to learn how the app side connects (reqwest through a local listener, or a direct connect API) and how the target is started (plain TCP echo).

- **Inner-TLS rows (Direct/splice path, spec §7.4):** the app side establishes a real TLS1.3 session through the tunnel to a rustls echo target:
  - Add a rustls echo **target**: a tokio TLS server with a self-signed cert (`rcgen` for cert gen; `tokio-rustls` for the server) echoing received bytes; SNI/cert CN = a fixed hostname (e.g. `echo.vision.test`).
  - Add the **app-side inner client**: `xray_tui_tls::client::connect(stream, TlsConfig { mode: TlsMode::Plain { profile: chrome130, verifier: WebPkiVerifier::insecure() }, server_name: "echo.vision.test", .. })` — check the exact `xray_tui_tls::client` API (`TlsConfig`, `TlsMode`, verifier constructors) in `crates/xray-tui-tls/src/client.rs` and use the same calls as the native security layer does. The app then sends an HTTP GET over the inner TLS stream and asserts the response.
  - Dev-deps (under the `native-e2e` feature or dev-dependencies — follow the harness's existing dep gating): `tokio-rustls`, `rcgen` (verify versions against the workspace: rustls is already a dependency of xray-tui-tls; use a compatible minor).
- **Plain rows (End path, spec §7.4):** existing app path (plain HTTP) unchanged.

- [ ] **Step 2: Add the 8 vision rows**

In `tests/vless.rs` — follow the existing row macro/case style (`vless_against_cores::case_*` + `vless_single_core` for single-core rows). All vision rows: `protocol = vless`, `network = tcp`, `flow = Some(Flow::Vision)`. Matrix (spec §7.4):

| Row | Core | Security | App type |
|---|---|---|---|
| vision_tls_inner | xray + sing-box | tls | inner-tls |
| vision_tls_plain | xray + sing-box | tls | plain |
| vision_reality_inner | xray + sing-box | reality | inner-tls |
| vision_reality_plain | xray + sing-box | reality | plain |

Both-core rows use the existing two-core pattern; if any row hits a real single-core limitation (e.g. sing-box reality inbound absent), move it to `vless_single_core` with an explicit `#[case]` core arg (documented in the commit).

- [ ] **Step 3: Run the full vless e2e sweep**

Run: `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vless`
Expected: existing 48 green + 4 ignored preserved; 8 vision rows green (56 green + 4 ignored total). If any vision row fails, debug against the wire spec (frames, commands, filter) — do NOT relax assertions.

- [ ] **Step 4: Run the vmess sweep (regression)**

Run: `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vmess`
Expected: 48 green unchanged (vision touches only VLESS).

- [ ] **Step 5: Commit**

```bash
git add crates/xray-tui-native/tests/vless.rs crates/xray-tui-native/src/e2e/ crates/xray-tui-native/Cargo.toml
git commit -m "feat(native): VLESS vision e2e — 8 rows (xray+sing-box × tls+reality × inner-tls+plain)"
```

---

### Task 7: Docs + full verification

**Files:**
- Modify: `NATIVE_CORE.md` (VLESS capability: flow support, vision rows, Direct splice semantics, the 2 documented deviations)
- Modify: `AGENTS.md` (xray-tui-native section: vision implemented; the e2e counts)

- [ ] **Step 1: Update NATIVE_CORE.md**

Add to the VLESS protocol row / protocol roadmap: `xtls-rprx-vision` implemented (TCP only; UDP rejected; requires TLS1.3/REALITY outer; inner TLS1.3 → Direct raw splice; non-1.3 → End, outer TLS continues). Note the deviations (spec §9): no 500 ms camouflage timer, per-direction direct flags, no XUDP mux. Update the tier-3 e2e counts (spec §8: existing 96+4 → 104+4 with 8 vision rows; unit count grows with the codec tests).

- [ ] **Step 2: Update AGENTS.md**

`crates/xray-tui-native/` section: add vision.rs to the module list; VLESS protocol notes: flow support + e2e row counts.

- [ ] **Step 3: Full verification sweep**

```bash
cargo clippy --workspace --all-targets     # 0 warnings
cargo fmt --check                          # clean
cargo test -p xray-tui-tls                 # engine green incl. Task 1 tests
cargo test -p xray-tui-native              # unit green (no e2e feature)
XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vless
XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vmess
```

Expected: clippy 0, fmt clean, unit green (new codec + engine tests), vless e2e 56 green + 4 ignored, vmess 48 green.

- [ ] **Step 4: Commit**

```bash
git add NATIVE_CORE.md AGENTS.md
git commit -m "docs(native): VLESS vision — NATIVE_CORE + AGENTS capability tables + counts"
```
