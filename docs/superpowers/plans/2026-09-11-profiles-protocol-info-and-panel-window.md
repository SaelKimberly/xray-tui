# Profiles: Protocol Info column, roomier row numbers, windowed panels

> **Executed inline by the session owner** (single owner, sequential edits in one screen file; no subagents).

**Goal:** Land the three backlog items on the Profiles screen:

1. Row numbers sized for 100000+ profiles and separated from the next column.
2. The endpoint-level `Type` column moves into the expanded panel (between Last Used and config type); the single row gains a named **Protocol Info** column showing `protocol/transport/security` (e.g. `{ vless/tcp/reality }`).
3. Expanded panels cap at 8 sub-rows and scroll a window that follows the selected sub-row — a 100-protocol endpoint no longer grows taller than the viewport and blanks itself.

**Baseline (read-set):** `crates/xray-tui/src/ui/profiles.rs` (row model, `build_display_rows`, `render_data_grid`, `render_expansion_panel`, `compute_scroll_offset`), `crates/xray-tui/src/ui/widgets/data_table.rs` (clip/offset contract), `crates/xray-tui/src/ops/profiles.rs` (expand/collapse, sub-row nav), `TUI_MANUAL.md` §Profiles.

**Compatibility:** display-only. No DB, schema, config, protocol, or public API change. Persisted data and identity untouched.

**Change Necessity:** user-visible need (backlog items 1–3); no non-code option exists — the defects are in the ratatui row renderer, so the minimum boundary is `ui/profiles.rs` + its docs/tests.

**Existence Check:** not triggered — no new owner, artifact, fallback, or workflow step; one local helper pair (`index_cell`, `panel_window`).

## TDD Route

- Mode: off (no explicit user/project TDD request; Aegis default)
- Decision: skipped
- Strict authority: not applicable
- Test posture: post-change regression (buffer-render + pure-fn tests in the existing module style)
- Reason: display-only layout change with an existing strong render-test convention in `ui/profiles.rs`
- Verification: `cargo test -p xray-tui`, `cargo clippy -p xray-tui`, TestBackend full-screen render test, live TUI launch

## Target layout (16 columns, total 117 cells — unchanged)

| # | Width | Header | Cell |
|---|-------|--------|------|
| 0 | 1 | | tree marker `▶`/`▾` |
| 1 | 2 | | status/test indicator |
| 2 | 7 | `#` | `{:>6}` row number + gap cell (`*` when multi-selected) |
| 3 | 1 | | `[` |
| 4 | 4 | | inbound country flag |
| 5 | 34 | `Address` | ` host:port` |
| 6 | 2 | | `][` |
| 7 | 4 | `Feat` | IP+SNI flags |
| 8 | 4 | | `]=>{` |
| 9 | 24 | `Protocol Info` | `protocol/transport/security`, centered |
| 10 | 3 | | `}=>` |
| 11 | 6 | `Test` | delay / problem label |
| 12 | 1 | | `[` |
| 13 | 16 | `Outbound` | exit IP |
| 14 | 7 | `Country` | exit country flag + ISO |
| 15 | 1 | | `]` |

Panel sub-table (11 columns, 115 of the 115 inner cells, with 1-cell gaps after config and before country): marker 3, id 9, Last Seen 20, Last Used 11, **Protocol Type 11**, config (transport/security) 14, delay 8, speed 7, traffic 10, outbound 16, country 5. The panel draws at most `PANEL_MAX_ROWS = 8` sub-rows; the window start `panel_window(total, selected, 8)` keeps the selected sub-row centered (clamped), and the separator line right-aligns `start-end/total` whenever the list is longer than the window.

## Tasks

### Task 1 — Row number + Protocol Info column (single row)

Files: `crates/xray-tui/src/ui/profiles.rs`

- `DisplayRowData`: drop `type_str`, rename `config_type_str` → `protocol_info_str`.
- `render` match arms: `2` idx, `3` `[`, `4` flag, `5` address, `6` `][`, `7` feat, `8` `]=>{`, `9` protocol info, `10` `}=>`, `11` test, `12` `[`, `13` outbound, `14` country, `15` `]`.
- New pure helpers (unit-testable):

```rust
const PROTOCOL_INFO_WIDTH: usize = 24;

fn index_cell(n: usize, is_multi: bool) -> String {
    let number = if is_multi { String::new() } else { n.to_string() };
    let suffix = if is_multi { '*' } else { ' ' };
    format!("{number:>6}{suffix}")
}

fn protocol_info_cell(row: &EndpointRow) -> String {
    let info = row.active_protocol().map_or_else(
        || "-".to_string(),
        |(_, p)| {
            format!(
                "{}/{}/{}",
                p.proto_kind,
                p.transport.r#type.as_str(),
                p.security.r#type.as_str()
            )
        },
    );
    center_pad(&info, PROTOCOL_INFO_WIDTH)
}
```

- `render_data_grid`: 16 columns per the table above; sort map `ConfigType => Some(9)`, `Address | Port => Some(5)`, `Test => Some(11)`.
- Tests: `index_cell` fits 6 digits (`100000 `), keeps the `*` marker; `protocol_info_cell` over `test_support::fake_row` yields `vless/tcp/none` centered; a buffer-render assertion that the cell at the Protocol Info column carries the merged value.

### Task 2 — Panel: Protocol Type column + windowed cap

Files: `crates/xray-tui/src/ui/profiles.rs`

- `PanelRow` gains `protocol_type` (kind string) between `last_used` and `config_type`.
- `height()`: `1 + self.panel_rows.len().min(PANEL_MAX_ROWS) as u16 + 4 + 1` (max 14 lines).
- `render_expansion_panel`: windowed loop over `panel_rows[win .. win + visible]`; selection style keyed on the global index (`panel_selected == Some(win + n)`); separator range label when `total > visible`.
- New pure helper:

```rust
fn panel_window(total: usize, selected: Option<usize>, max_visible: usize) -> usize {
    if total <= max_visible || max_visible == 0 {
        return 0;
    }
    let sel = selected.unwrap_or(0).min(total - 1);
    sel.saturating_sub(max_visible / 2).min(total - max_visible)
}
```

- Tests: window math cases; `height()` capped at 14 for 20 rows; buffer render with 23 panel rows and `panel_selected = Some(9)` draws `panel_rows[5..13]`, the range label `6-13/23`, and nothing past `clip_bottom`.

### Task 3 — Docs + verification

Files: `TUI_MANUAL.md`, `AGENTS.md`, `CONTEXT.md`

- `TUI_MANUAL.md`: replace the Profiles column table rows (index 7, `Protocol Info` 24, no `Type`) and the panel paragraph (Protocol Type column; 8-row window with the range indicator).
- `AGENTS.md` decision 14: 16 fixed columns, capped panel, windowed sub-table.
- `CONTEXT.md` display table: 16-column description.

Verification: `cargo fmt`, `cargo test -p xray-tui`, `cargo clippy -p xray-tui`, then a live launch (`XDG_CONFIG_HOME` isolated) driven through the tui-test harness: header shows `Protocol Info`, a multi-protocol endpoint expands, `↓` scrolls the window, the range indicator appears past 8 variants.

## Risks

- Column-width drift: total must stay 117 (terminal ≥ 120 shows every column); the plan's table sums to 117.
- Sub-row index vs window: selection styling must use the global sub-row index, not the window-local one.
- Panel height/`compute_scroll_offset` agreement: both derive from `min(len, 8)`; the tall-row regression tests stay green.

## Verification outcome (2026-09-11)

- `cargo test -p xray-tui` — 144 lib + 2 bin tests pass; new regressions: `index_cell_fits_six_digit_rows`, `protocol_info_cell_merges_kind_transport_security`, `panel_window_follows_selected_sub_row`, `panel_height_is_capped_at_eight_sub_rows`, `windowed_panel_renders_selected_window_and_range`, `panel_config_and_delay_keep_a_gap`.
- `cargo clippy -p xray-tui --all-targets` — clean.
- Live launch (isolated `XDG_CONFIG_HOME`, 140x40, ten variants on one endpoint incl. `httpupgrade`, tui-test harness): header shows `Protocol Info`; the row reads `{ vless/httpupgrade/tls }`; `→` expands to exactly 8 sub-rows with `1-8/10` on the separator; `↓`×8 scrolls the window to `3-10/10`. The pre-fix blanking case (row taller than the viewport) is gone.
- Review follow-up: panel column offsets were documentation-only (the render loop accumulated widths), so the documented gap after config never rendered — offsets are authoritative now, and the live row shows the clipped `httpupgrade/tl` followed by a gap before the delay cell.

## Retirement

No code retired beyond `DisplayRowData::type_str` (replaced by `protocol_info_str`).
