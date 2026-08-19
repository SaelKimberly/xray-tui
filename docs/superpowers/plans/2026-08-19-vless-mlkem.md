# VLESS ML-KEM (Post-Quantum KEM) Implementation Plan (SP7)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add ML-KEM-768 (Kyber768) support to xray-tui-native via liboqs:
- TLS 1.3 hybrid curves: `X25519MLKEM768`, `SecP256r1MLKEM768`, `SecP384r1MLKEM1024` (ClientHello supported_groups + key share with ML-KEM public key / ciphertext)
- utls PQ fingerprints: `HelloChrome_115_PQ`, `HelloChrome_115_PQ_PSK`, `HelloChrome_120_PQ` (end-to-end)
- VLESS encryption: `mlkem768x25519plus` mode (`native`/`xorpub`/`random` + padding)
- REALITY with `X25519MLKEM768` key share

Interoperable with xray-core 26.3.27+ and sing-box 1.18+.

**Architecture:** New `xray-tui-tls` crypto module wrapping `oqs::kem::Kyber768` (liboqs binding — the program's 2nd major dep after quinn). TLS engine hybrid key exchange (classical || PQ shared secret). VLESS protocol `mlkem768x25519plus` encryption. REALITY X25519MLKEM768 key share. Proto changes for curve names + VLESS encryption.

**Tech Stack:** `oqs` crate (liboqs binding), the existing `xray-tui-tls` handshake/key_schedule, `xray-tui-native` VLESS/REALITY, `xray-tui-proto` curve names + encryption parser.

**Spec:** `docs/superpowers/specs/2026-08-19-vless-mlkem-design.md` (wire authority: xray 26.3.27 `tls/config.go`, `vless.go`, `reality.go`; sing-box `option/tls.go`, `reality_client.go`; native TLS `profiles/chrome133.rs`).

## Global Constraints

- xray-tui-proto **IS MODIFIED** for SP7 (unlike previous SPs): curve name parsing + VLESS `mlkem768x25519plus` encryption parser.
- New dependency: `oqs` (liboqs binding) — user-approved; build needs `liboqs` system library (or `oqs-sys` vendored). Version: latest minor with minimum bound per workspace direct-dep convention.
- The three hybrid curves: `X25519MLKEM768` (primary, 4588), `SecP256r1MLKEM768` (4587), `SecP384r1MLKEM1024` (4589). Primary target: X25519MLKEM768 (used by xray REALITY + sing-box).
- utls PQ fingerprints already exist in `xray-tui-tls/profiles/` (Chrome133 PQ etc.) — they declare the hybrid curves; the implementation must actually generate/process the ML-KEM key shares.
- VLESS `mlkem768x25519plus` format: `mlkem768x25519plus.<mode>.<seconds_from>[-<seconds_to>]s.<padding>` (mirror xray `infra/conf/vless.go`).
- REALITY: X25519MLKEM768 curve ID 4588 (0x11EC) — both client + server.
- e2e rows: xray-core (tls-pq, vless-pq-enc, reality-pq) + sing-box 1.18+ (tls-pq, reality-pq). Count: vless +3, vmess +1 (tls-pq).
- Gates per spec §7.4; run BOTH `--features native-e2e` builds.

---

### Task 1: liboqs integration + ML-KEM crypto primitives

- [ ] Add `oqs` crate to `xray-tui-tls` (or a new `xray-tui-crypto` crate if preferred — keep it in `xray-tui-tls` for now) with feature `kyber768` (or the equivalent). Ensure `liboqs` is available at build time (document the system dep; CI installs `liboqs-dev`).
- [ ] Create `crates/xray-tui-tls/src/crypto/mlkem.rs`: `Mlkem768` struct with `generate_keypair() -> (PublicKey, SecretKey)`, `encapsulate(pk: &PublicKey) -> (Ciphertext, SharedSecret)`, `decapsulate(sk: &SecretKey, ct: &Ciphertext) -> SharedSecret`. Use `oqs::kem::Kyber768` directly.
- [ ] Unit tests: keypair roundtrip, encapsulate/decapsulate match, shared secret equality. Test vectors if available.
- [ ] Build verification: `cargo build -p xray-tui-tls` + `cargo test -p xray-tui-tls` (clippy 0, fmt clean).

- Consumes: spec §5.1/§5.2/§4.1.
- Produces: the ML-KEM primitive the TLS engine + VLESS + REALITY consume.

### Task 2: TLS engine hybrid curve support

- [ ] `handshake/drive.rs`: detect hybrid curve selection (curve ID in {4587, 4588, 4589}); on ClientHello, generate ML-KEM keypair, include public key (1184 bytes) in the key share alongside the classical public key; on ServerHello, decapsulate the ML-KEM ciphertext (1088 bytes) to get the PQ shared secret; combine `classical_secret || pq_secret` as the input to the TLS 1.3 key schedule (extend `key_schedule.rs` to accept the hybrid concatenation).
- [ ] `crypto/key_schedule.rs`: extend the HKDF input to accept `classical || pq` (the existing code path takes a single shared secret — add a hybrid path or make it generic).
- [ ] The utls PQ profiles (`profiles/chrome133.rs` etc.) already declare the hybrid curves in `supported_groups` — verify the key share generation works for the listed curves.
- [ ] Unit tests: hybrid handshake with a fake server double (loopback) that negotiates X25519MLKEM768 and performs the ML-KEM encapsulation/decapsulation; assert the derived traffic keys match on both sides.
- [ ] No proto changes here (curves are parsed from string config).

- Consumes: Task 1.
- Produces: TLS 1.3 hybrid handshake end-to-end.

### Task 3: VLESS `mlkem768x25519plus` encryption

- [ ] Proto change: `xray-tui-proto` — add the `mlkem768x25519plus` parser to the VLESS account config (mirror xray `infra/conf/vless.go`: mode `native`/`xorpub`/`random`, seconds window, base64url padding). The `Encryption` field becomes an enum with this variant.
- [ ] `xray-tui-native/src/protocol/vless/encryption/mlkem.rs`: implement the encryption/decryption per xray's `server.go`. Uses ML-KEM-768 for KEM + X25519 for classical. Modes: `native` (direct), `xorpub` (XOR the ML-KEM public key with the X25519 public key), `random` (random padding). Padding: time-window + base64url decode.
- [ ] Wire the encryption into the VLESS protocol handshake (the `encryption` field on the account config selects this path).
- [ ] Unit tests: encryption/decryption roundtrip for all 3 modes + padding variations; test vectors from xray if available.

- Consumes: Task 1 (ML-KEM primitive).
- Produces: VLESS PQ encryption end-to-end.

### Task 4: REALITY X25519MLKEM768 support

- [ ] `security/reality.rs`: the `HelloProvisioner` / `ProfileProvisioner` must support curve ID 4588 (`X25519MLKEM768`). When this curve is selected, generate the ML-KEM keypair and include the 1184-byte public key in the REALITY key share; process the server's 1088-byte ML-KEM ciphertext.
- [ ] The curve preference comes from the TLS config (the hybrid curve names in the `curves` list). The REALITY handshake reuses the TLS hybrid logic.
- [ ] Unit tests: REALITY handshake with the hybrid curve against a fake server double.

- Consumes: Task 1 + Task 2.
- Produces: REALITY with ML-KEM.

### Task 5: Proto changes (curve names + VLESS encryption)

- [ ] `xray-tui-proto/src/proto_spec/common.rs` (or wherever curve preferences live): extend the curve name parser to accept `x25519mlkem768`, `secp256r1mlkem768`, `secp384r1mlkem1024` (mirror xray's `ParseCurveName`). The curve IDs map to the TLS engine's hybrid curve IDs.
- [ ] VLESS account config: add the `mlkem768x25519plus` encryption variant parser (mirror xray's `infra/conf/vless.go` parser logic).
- [ ] No other proto changes.

- Consumes: Tasks 2-4 (the implementation must exist).
- Produces: config-driven PQ selection.

### Task 6: e2e rows (xray + sing-box PQ)

- [ ] Harness: xray-core 26.3.27+ server configs with hybrid curves (tlsSettings.curves includes `x25519mlkem768`), VLESS `mlkem768x25519plus` encryption, REALITY with X25519MLKEM768. sing-box 1.18+ equivalent.
- [ ] Rows (xray-single-core + sing-box where available): tls-pq (X25519MLKEM768), vless-pq-enc, reality-pq (xray); tls-pq + reality-pq (sing-box). vmess: tls-pq.
- [ ] Sweeps: `XRAY_TUI_CORE_BIN_DIR=/tmp/core-bin cargo test -p xray-tui-native --features native-e2e --test vless` (expect vless +3) + `--test vmess` (vmess +1).
- [ ] Verify the PQ curves are actually negotiated (check logs / handshake dump).

- Consumes: Tasks 1-5.
- Produces: the green PQ rows.

### Task 7: Docs + full verification

- [ ] NATIVE_CORE.md + AGENTS.md: ML-KEM support (hybrid curves, VLESS encryption, REALITY), liboqs dep, PQ fingerprints, counts (unit + e2e).
- [ ] Full gates: clippy 0 (both feature sets), fmt, unit lib, e2e sweeps.
- [ ] Verify no stale claims.

- Consumes: Tasks 1-6.
- Produces: SP7 closed.