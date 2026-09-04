# Throughput baseline (v1)

Date: 2026-09-04. Commit: `4292c47`. Criterion medians, `XRAY_TUI_BENCH_MB=4`.

Update this file on every baseline re-run (new row section per date).
Raw criterion output (`.benchmarks/`, gitignored) is machine-local;
this file is the committed history.

## Env

- CPU: 12th Gen Intel i7-12700, x86_64, 31 GiB RAM
- Cores: xray 26.3.27, sing-box 1.13.16 (loopback)
- Bench: `cargo criterion -p xray-tui-native --features native-e2e --bench throughput`

## 2026-09-04 (v1)

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
