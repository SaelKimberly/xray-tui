# ─── Quality gate ─────────────────────────────────────────────────
# Docs: https://github.com/casey/just
#
# `just quality-gate`    — every check, verbose report, all checks run even on failure
# `just quality-gate-ci` — every check, minimal output, stops at first failure (exit 0/1, for CI)

# Run the full quality gate (verbose report). Exit 0 = all passed, 1 = any failed.
quality-gate:
    @just _gate report

# Run the full quality gate for CI (minimal output, stop at first failure). Exit 0/1.
quality-gate-ci:
    @just _gate ci

# ─── Individual checks ────────────────────────────────────────────

# Formatting check (rustfmt.toml pins edition/max_width/newline_style)
fmt-check mode='report':
    @cargo fmt --all --check

# cargo-hakari workspace-hack verification (.config/hakari.toml)
hakari-check mode='report':
    @cargo hakari verify

# Clippy, all workspace targets and features, warnings denied
clippy mode='report':
    @cargo clippy --workspace --all-targets --all-features -- -D warnings

# Tests via cargo-nextest (.config/nextest.toml; `ci` mode uses the `ci` profile)
nextest mode='report':
    @cargo nextest run {{ if mode == "ci" { "--profile ci" } else { "" } }}

# cargo-deny: bans, licenses, sources (deny.toml); advisories are owned by `audit`
deny mode='report':
    @cargo deny check bans licenses sources

# Unused dependencies (ignores live in `[package/workspace.metadata.cargo-machete]`)
machete mode='report':
    @cargo machete --with-metadata --skip-target-dir

# direct deps are kept at latest (semver-major tracks toasty 0.10, base64 0.23,
# brotli 8, sha2 0.11 are applied with breakage fixes). The residual entries are
# graph-inherent dual-major pins in the generated xray-tui-hakari (e.g.
# crypto-common 0.1 via turso's aes-gcm, hashbrown 0.16 via yaml-rust2,
# compact_str 0.9 via ratatui, windows-sys platform pins) that only upstream
# bumps can remove, so a hard fail would keep the gate red indefinitely.
outdated mode='report':
    @cargo outdated --workspace --root-deps-only --exit-code 0 {{ if mode == "ci" { "--quiet" } else { "" } }}

# cargo-audit vulnerability scan (.cargo/audit.toml)
audit mode='report':
    @cargo audit {{ if mode == "ci" { "--quiet" } else { "" } }}

# ─── Gate runner (private) ────────────────────────────────────────

_gate mode:
    #!/usr/bin/env bash
    set -uo pipefail
    mode="{{mode}}"
    tools=(fmt-check hakari-check clippy nextest deny machete outdated audit)
    total=${#tools[@]}
    passed=0
    failed=()
    for tool in "${tools[@]}"; do
        echo
        echo "===== $tool ====="
        if just "$tool" "$mode" 2>&1; then
            echo "PASS: $tool"
            passed=$((passed + 1))
        else
            echo "FAIL: $tool"
            failed+=("$tool")
            if [ "$mode" = "ci" ]; then
                exit 1
            fi
        fi
    done
    echo
    echo "===== summary: $passed/$total passed ====="
    if [ "${#failed[@]}" -gt 0 ]; then
        printf 'failed: %s\n' "${failed[@]}"
        exit 1
    fi
