# ─── Quality gate ─────────────────────────────────────────────────
# Docs: https://github.com/casey/just
#
# `just quality-gate`            — every check, verbose report, all checks run even on failure
# `just quality-gate code`       — source checks only (fmt-check, clippy, nextest)
# `just quality-gate deps`       — dependency checks only (hakari-check, deny, machete, outdated, audit)
# `just quality-gate-ci`         — every check, minimal output, stops at first failure (exit 0/1, for CI)
# `just quality-gate-ci code|deps` — the same subsets in CI mode

# Run the quality gate (verbose report). `target` selects the group:
# code | deps | all (default). Exit 0 = all passed, 1 = any failed.
quality-gate target='all':
    @just _gate report "{{target}}"

# Run the quality gate for CI (minimal output, stop at first failure). Exit 0/1.
quality-gate-ci target='all':
    @just _gate ci "{{target}}"

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

# Outdated direct dependencies. Informational by design (`--exit-code 0`):
# direct deps are kept at latest (semver-major tracks toasty 0.10, base64 0.23,
# brotli 8, sha2 0.11 are applied with breakage fixes). The residual entries are
# graph-inherent dual-major pins in the generated xray-tui-hakari (e.g.
# base64 0.22 via dns-stamp-parser, hashbrown 0.16 via yaml-rust2,
# compact_str 0.9 via ratatui, syn 2, windows-sys platform pins) that only
# upstream bumps can remove, so a hard fail would keep the gate red indefinitely.
outdated mode='report':
    @cargo outdated --workspace --root-deps-only --exit-code 0 {{ if mode == "ci" { "--quiet" } else { "" } }}

# cargo-audit vulnerability scan (.cargo/audit.toml)
audit mode='report':
    @cargo audit {{ if mode == "ci" { "--quiet" } else { "" } }}

# ─── Gate runner (private) ────────────────────────────────────────

_gate mode target:
    #!/usr/bin/env bash
    set -uo pipefail
    mode="{{mode}}"
    target="{{target}}"
    case "$target" in
        code) tools=(fmt-check clippy nextest) ;;
        deps) tools=(hakari-check deny machete outdated audit) ;;
        all)  tools=(fmt-check clippy nextest hakari-check deny machete outdated audit) ;;
        *)
            echo "unknown gate target: $target (expected code | deps | all)" >&2
            exit 2
            ;;
    esac
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
