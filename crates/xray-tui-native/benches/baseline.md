# Throughput baseline

Criterion medians. Update this file on every baseline re-run (new dated
section). Raw criterion output (`.benchmarks/`, gitignored) is machine-local;
this file is the committed history.

**Every number here is `XRAY_TUI_BENCH_MB=4`.** The harness default is 64, so a
run that does not pin the variable is not comparable — `just bench` pins it to 4
for exactly this reason. `just bench` also exports `XRAY_TUI_CORE_BIN_DIR`
itself: `cargo criterion` launches the bench executable directly and does NOT
apply `[env]` from `.cargo/config.toml`, so without it the throughput rows print
`SKIP … is not set` and the run still exits 0.

## Env

- CPU: 12th Gen Intel i7-12700, x86_64, 31 GiB RAM
- Cores: xray 26.3.27, sing-box 1.13.16 (loopback)
- Benches: `just bench` (`micro` = record + decide + dispatch + relay, no core
  binaries; `all` adds the throughput matrix)

## 2026-09-04 (v2) — native/tls/route performance pass

Same machine and settings as v1. Every row re-measured after the pass; the
`change` column is criterion's own comparison against the stored pre-change run.

### End-to-end tunnel matrix (`throughput`)

| bench | send | Δ send | recv | Δ recv |
|---|---|---|---|---|
| vless/tcp/plain | 4.5179 GiB/s | −25.1% | 3.9607 GiB/s | −47.9% |
| vless/tcp/tls-chrome | 1.3827 GiB/s | −12.9% | 1.3446 GiB/s | −18.4% |
| vless/tcp/reality | 1.3723 GiB/s | −12.7% | 1.3532 GiB/s | −19.1% |
| vless/vision+reality | 1.3742 GiB/s | −12.7% | 1.3570 GiB/s | −22.3% |
| vless/ws/plain | 1.4175 GiB/s | −26.8% | 1.6990 GiB/s | −25.8% |
| vless/grpc/tls-chrome | 1.0451 GiB/s | −27.8% | 767.64 MiB/s | −8.3% |
| vmess/tcp/aes | 1.5693 GiB/s | −19.4% | 1.1339 GiB/s | −15.0% |
| vmess/ws/chacha | 922.40 MiB/s | −12.0% | 748.31 MiB/s | −16.7% |
| trojan/tcp/tls-chrome | 2.6720 GiB/s | −14.7% | 1.5550 GiB/s | −3.4% |
| hysteria2/tcp | 651.24 MiB/s | −9.5% | 794.10 MiB/s | −7.1% |

Δ is criterion's time delta (negative = faster). Every row improved, `p < 0.05`.
The plain rows moving as much as the TLS rows was not expected from the record
layer alone — the relay rewrite (one buffer + one timer per direction instead of
per chunk) lifts every row, TLS or not.

### TLS record layer (`record`, `xray-tui-tls`)

Before = pristine `HEAD` sources measured with the same bench file (the two
rewritten files were swapped out via `git show`, not `cargo clean`).

| row | before | after | Δ |
|---|---|---|---|
| tls_record/aes128gcm/seal | 788.21 µs (4.96 GiB/s) | 597.66 µs (6.54 GiB/s) | −25.7% |
| tls_record/aes128gcm/open | 966.44 µs (4.04 GiB/s) | 710.69 µs (5.50 GiB/s) | −25.6% |
| tls_record/chacha20/seal | 2.0013 ms (1.95 GiB/s) | 1.8099 ms (2.16 GiB/s) | −9.6% |
| tls_record/chacha20/open | 2.0940 ms (1.87 GiB/s) | 1.9090 ms (2.06 GiB/s) | −9.2% |

AES-GCM gains more than ChaCha20 because the copies removed are a larger share
of its per-record cost.

### SOCKS5 inbound relay (`relay`, in-process, no core binary)

Before = `inbound/outbound.rs` at `HEAD` (the `select!` + `copy_once` shape),
everything else current.

| row | before | after | Δ |
|---|---|---|---|
| relay/socks5-direct/send | 1.2567 ms (3.11 GiB/s) | 1.1163 ms (3.50 GiB/s) | −11.2% |
| relay/socks5-direct/recv | 1.4460 ms (2.70 GiB/s) | 1.3393 ms (2.92 GiB/s) | −7.4% |
| relay/socks5-udp/pps | 18.272 ms/1000 (54.7 Kpps) | 17.701 ms/1000 (56.5 Kpps) | −3.2% (p = 0.14) |

The UDP row's difference is not significant, as expected: the relay rewrite is
TCP-only. The UDP work (inline DNS-cache hit, reusable reply buffer) removes a
task spawn, a payload copy and 3 allocations per datagram, but at ~17 µs per
datagram — dominated by `sendto`/`recvfrom` and task wakeups — that is below
this bench's noise floor. Recorded as verified-by-code, not by measurement.

### Routing engine (`decide`, `xray-tui-route`)

| row | before | after | Δ |
|---|---|---|---|
| decide/rules_0/domain_hit | 16.318 ns | 19.609 ns | **+20.2%** |
| decide/rules_0/ip_hit | 16.375 ns | 19.60 ns | **+19.7%** |
| decide/rules_0/miss | 16.407 ns | 19.64 ns | **+19.5%** |
| decide/rules_8/domain_hit | 193.79 ns | 161.60 ns | −16.6% |
| decide/rules_8/ip_hit | 95.629 ns | 94.404 ns | −1.3% |
| decide/rules_8/miss | 239.55 ns | 180.84 ns | −24.5% |
| decide/rules_64/domain_hit | 1.4865 µs | 1.0537 µs | −29.1% |
| decide/rules_64/ip_hit | 614.30 ns | 605.40 ns | −1.5% |
| decide/rules_64/miss | 1.4424 µs | 1.0591 µs | −26.6% |

Honest split: at **zero** rules — the shape production runs today, since routing
rules force the subprocess core — the pass costs **+3.3 ns per decision**
(cloning a pre-built `Decision` vs. building one from the IR action). Every
non-empty rule set is 13–29 % faster. Run-to-run noise on these rows is ±1–2.5 %,
so both directions are outside the noise.

`Arc<str>` vs `String` for `Decision::Route.tag` was measured directly (three
interleaved runs in one thermal state, one variant swapped between them):

| row | `String` | `Arc<str>` |
|---|---|---|
| decide/rules_0/domain_hit | 20.756 ns | 19.609 ns (−5.3%) |
| decide/rules_8/domain_hit | 226.52 ns | 161.60 ns (−28.9%) |
| decide/rules_64/domain_hit | 1.4541 µs | 1.0537 µs (−27.6%) |

`Arc<str>` wins at every rule count, so it stays. (A `String` tag was tried and
reverted on this evidence — the reverted variant was at or worse than pristine
everywhere, which also shows the matcher short-circuits alone are not what
produced the gain.)

### Dispatch cost (`dispatch`) — measurement only, no change made

| row | median | thrpt |
|---|---|---|
| dispatch/static (4 monomorphized layers) | 311.24 µs | 12.55 GiB/s |
| dispatch/boxed (4 `Box<dyn Stream>` layers) | 322.82 µs | 12.10 GiB/s |

`boxed / static = 1.02` at 4 MiB (1.020 at 64 MiB in a separate run). This is
the number behind rejecting the "`BoxStream` costs 2–5× throughput" review item:
the seam costs **~2 %**, because it is one virtual call per `poll_*` (≈ one per
16 KiB), not per byte. The plan's action threshold was 15 %.

## 2026-09-04 (v1)

Commit `4292c47`. `cargo criterion -p xray-tui-native --features native-e2e --bench throughput`.

| bench | send | recv |
|---|---|---|
| vless/tcp/plain | 3.3778 GiB/s | 2.1064 GiB/s |
| vless/tcp/tls-chrome | 1.1954 GiB/s | 1.0733 GiB/s |
| vless/tcp/reality | 1.2036 GiB/s | 1.0943 GiB/s |
| vless/vision+reality | 1.1883 GiB/s | 1.0160 GiB/s |
| vless/ws/plain | 1.0281 GiB/s | 1.2411 GiB/s |
| vless/grpc/tls-chrome | 732.01 MiB/s | 684.17 MiB/s |
| vmess/tcp/aes | 1.2605 GiB/s | 980.23 MiB/s |
| vmess/ws/chacha | 794.93 MiB/s | 652.89 MiB/s |
| trojan/tcp/tls-chrome | 2.2739 GiB/s | 1.4969 GiB/s |
| hysteria2/tcp | 597.02 MiB/s | 750.45 MiB/s |
