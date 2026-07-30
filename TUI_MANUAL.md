# xray-tui Manual

## Overview

xray-tui is a terminal user interface (TUI) for managing proxy connections through
xray-core and sing-box. It provides profile management, subscription updates, speed
testing, settings configuration, and real-time statistics — all from the terminal.

The dual-backend architecture abstracts over xray-core and sing-box subprocesses.
The TUI writes JSON configs and manages the binary lifetime. Only one backend runs
per connection session; switching profiles between backends stops the current core
and starts the other.

---

## Screens

### Profiles

The main screen. Shows a table of server profiles with the following columns:

| Column   | Width | Content                              |
| -------- | ----- | ------------------------------------ |
| `#`      | 5     | Row number; `*` when multi-selected  |
| `Type`   | 8     | Protocol type (vmess, trojan, etc.)  |
| `Remarks`| 24    | Server name / remark                 |
| `Address`| 30    | Server address or hostname           |
| `Port`   | 6     | Server port                          |
| `Delay`  | 6     | Last ping delay in ms (or `-`)       |
| `Speed`  | 6     | Last speed test result (or `-`)      |
| `Traffic`| 10    | Total traffic through this profile   |
| `Core`   | 8     | Assigned core (xray/sing-box)        |

The Core column is color-coded: blue for xray-core, green for sing-box.

**Filter strip** above the table shows:
- Current group filter (if one is active via Groups overlay)
- Search query when `/` is pressed

**DataGrid features:**
- Alternating row colors (odd rows have alt background)
- Selected row highlighted with bold
- Multi-select indicator: `*` replaces row number
- Scrollable via arrow keys, Home/End

**Search:** Press `/` to focus search field, type to filter profiles by name/address/type.
Search matches are highlighted. `Esc` clears filter and exits search mode.

**Delete confirmation overlay** — centered dialog asking `y/n` when `d` is pressed.
`y` confirms, `n`/`q`/`Esc` cancels.

---

### Settings

A settings panel with a menu of 14 sections. Press `Enter` on a section to open its
configuration form. A separator line visually groups the first 5 "active" sections
from the remaining "deferred" sections.

**Menu items (in order):**

1. Core Settings — binary paths, log level, default core type
2. GUI Settings — language, theme, refresh interval
3. Inbound Settings — ports, listen address, sniffing
4. Routing Rules — add/edit/delete/reorder rules
5. DNS Settings — servers, hosts, query strategy, cache
6. Protocol Core — per-protocol core override (Auto/Xray/SingBox)
7. System Proxy — enable/disable HTTP_PROXY, ports, bypass
8. TUN Mode — enabled, interface name, MTU
9. Mux — multiplexing settings
10. Statistics — enable/disable stats collection
11. Speed Test — ping URL, IP API URL, timeouts, batch concurrency
12. Logging — log level, log-to-file path, log retention
13. Subscriptions — group list, add/edit/delete/update
14. Updates — check and install backend updates

**Form navigation within a section:**
- `Tab` / `Shift+Tab` — move focus between fields
- `Enter` — save and return to menu
- `Esc` — cancel and return to menu

**Routing Rules** (section 4) has a two-level UI:
- Routing list: `↑↓` navigate, `Enter` edit, `a` add, `d` delete, `r` reorder
- Routing rule form: same Tab/S-Tab/Enter/Esc navigation

**Updates form** — shows current version and latest available version for each
backend. `C` triggers a version check, `D` triggers download for all available
updates. `Esc` returns to menu.

---

### Logs

A bordered container displaying live log output from the connected core process.
When no core is running, shows:

```
┌ Logs ───────────────────────────────────────────────────┐
│No logs                                                  │
└──────────────────────────────────────────────────────────┘
```

After connecting to a profile, the core's stdout (xray-core) and stderr (sing-box)
log lines appear in real time. Log levels are color-coded:

- **error/fatal/panic** — red bold (`Theme::ERROR`)
- **warning/warn** — yellow (`Theme::WARNING`)
- **info** — default terminal color
- **debug/trace** — gray (`Theme::HINT`)

The buffer holds the most recent 1000 log lines. Scrolling is bottom-anchored
(0 = newest line visible at bottom of viewport).

---

### Statistics

When not connected to a server:

```
┌─ Statistics ─────────────────────────────────────────┐
│                                                       │
│               No data — connect to a server            │
│                                                       │
└───────────────────────────────────────────────────────┘
```

When connected: displays traffic stats (total upload/download, speed), system
statistics (CPU/memory/uptime if available), and connection info (API endpoint,
connection status). Data sourced from the backend's gRPC/V2Ray API when available.

---

### Groups Overlay

Modal overlay opened with `g` from the Profiles tab. Lists subscription groups
with columns:

| Column        | Content                           |
| ------------- | --------------------------------- |
| `Name`        | Group name                        |
| `URL`         | Subscription URL (truncated)      |
| `Ena`         | Enabled/Disabled                  |
| `Status`      | Last update status                |
| `Last Updated`| Timestamp of last fetch           |

**Actions** (shown in footer):

| Key        | Action                            |
| ---------- | --------------------------------- |
| `a`        | Add new group                     |
| `e`        | Edit selected group               |
| `d`        | Delete selected group (y/n confirm)|
|| `u`        | Update single group's subscriptions                          |
|| `Shift+U`  | Update all groups                                           |
|| `Enter`    | Filter profiles by selected group                            |
|| `[`        | Cycle to previous group in filter (wraps, skips purgatory)   |
|| `]`        | Cycle to next group in filter (wraps, skips purgatory)       |
|| `Esc`      | Close overlay                                                |

**Group form** — fields: `name`, `subscription_url`, `user_agent`,
`update_interval`, `core_type`. `Tab`/`Shift+Tab` navigates, `Enter` saves,
`Esc` cancels.

---

### Add/Edit Server Forms

Opened with `a` (add) or `e` (edit) from the Profiles tab.

**Protocol picker** (add only) — scrollable list of all supported protocols:

- Xray-core native: VMess, VLESS, Shadowsocks, Shadowsocks-2022, SOCKS, HTTP,
  Trojan, WireGuard, Hysteria v2, Dokodemo-door, Freedom, Blackhole, DNS,
  Loopback, Custom
- Sing-box only: TUIC, Hysteria v1, Naïve, AnyTLS, ShadowTLS, Tor, SSH,
  Tailscale, ShadowsocksR, Redirect

`↑↓` to navigate, `Enter` to select, `Esc` to cancel.

**Parameter form** — after selecting a protocol (or when editing), displays
fields specific to that protocol. Field types: text, number, boolean (toggle
on any key press), and select (cycle with `←`/`→` or any key press).

- `Tab` / `Shift+Tab` or `↑↓` — move focus
- `Enter` — save and return to profile list
- `Ctrl+S` — save (alternative)
- `Esc` — cancel
- `Backspace` — delete character before cursor
- `Space` — type space
- `-` in number fields — negative sign

---

### Import URL Screen

Opened with `Ctrl+V` from the Profiles tab. A bordered input field with title:

```
Import URL — Ctrl+V paste, Enter parse, Esc cancel
```

- Type or paste the share URL (vmess://, ss://, trojan://, etc.)
- `Enter` — parse and add profile
- `Esc` — cancel
- Shows error message in red if parsing fails

### Batch Import Screen

Triggered by pasting multiple share URLs into the Import URL screen and pressing Enter.
Shows a scrollable list of import results with success/failure per URL.

**Keyboard shortcuts:**
- `↑/↓` — scroll results list
- `Enter` — save all successful imports and return to profile list
- `Esc` — cancel all and return to profile list

---

### Speed Test Menu

Opened with `t` from the Profiles tab. Overlay menu centered on screen:

```
┌─ Speed Test ─────────────────────────────────────────┐
│  Fast Ping (Selected)                                  │
│  Real Ping (Selected)                                 │
│  Speed Test (Selected)                                │
│  UDP Test (Selected)                                  │
│  ─────                                                │
│  Fast Ping (All Visible)                                │
│  Fast + Real Ping (All Visible)                         │
│  ─────                                                │
│  Clear All Stats                                      │
│  Sort by Delay                                        │
│  Remove Bad Servers                                   │
│  ─────                                                │
│  Stop Testing                                         │
└───────────────────────────────────────────────────────┘
```

- `↑↓` — navigate (skips separator lines)
- `Enter` — run selected test
- `Esc` — close menu
- `?` — open help overlay
- `s` — stop running tests (works when progress shown in status bar)

Progress is shown in the status bar during batch tests (`Testing: {completed}/{total}` or `Testing...`).

---

### Help Overlay

Opened with `?` from any main screen. Rounded-border centered modal listing
keyboard shortcuts grouped by current context (Profiles, Settings, Statistics,
or Logs). `Esc` or `?` closes.


### Actions Log

A live state information panel showing current system state, connection info,
test results, and recent log lines. Supports two views toggled with `F1`:

**Compact bar** (1 line, default in terminals <20 rows tall):

```
⠋ VMess/my-server 1.2.3.4:443 | TCP:45ms RP:120ms SPD:15Mbps | ⬆1.2GB ⬇3.4GB | Info Xray 1.8.0 started
```

Connection indicators:
- `⠋` yellow — connecting
- `●` green — connected
- `⏹` red — error
- `⏏` yellow — disconnected (profile selected but not connected)
- `○` gray — no profile

Segments (compact): server info, test results (only when available), traffic,
last core log snippet.

**Full panel** (8 rows, default in terminals ≥20 rows tall):

```
┌─ ⚙ Actions Log ───────────────────────────────────────┐
│ ● Connected [xray]                                      │
│ 🖥 VLESS  us.example.com:443                            │
│ ⏱ TCP:45ms  RP:120ms  SPD:15Mbps                       │
│ 📊 ⬆1.2GB  ⬇3.4GB    💾 64.2MB                         │
│ 📋 Core: [Info] Xray 1.8.0 started                      │
│ 📋 TUI:  [Info] Connected [xray] (core)                 │
└────────────────────────────────────────────────────────┘
```

Rows:
1. Connection status (icon + state + backend type)
2. Server info (protocol, remarks, address, port)
3. Test results (TCP ping, real ping, speed — `-` for unavailable)
4. Traffic (upload/download from stats API) and memory usage
5. Last core log line (from core process stderr/log channel)
6. Last TUI log line (from `tracing` events, with target tag)

**Behavior:**
- Default view depends on terminal height: compact if <20 rows, full if ≥20
- `F1` toggles between compact and full at any time
- In terminals <20 rows, full panel replaces content area as an overlay.
  Press `F1` or `Esc` to close the overlay.
- In normal terminals, full panel renders inline below the tab bar.
  Content area adjusts automatically.

Tracing integration (backend logging):
- TUI internals emit `tracing` events via `tracing-subscriber` with a
  custom `TuiLogLayer` that forwards events to the core event channel
- Events carry a `target` field (`core`, `tui`, `speedtest`, `subscription`)
- Viewable in the Actions Log panel and (if stderr is captured) in
  `RUST_LOG=xray_tui=info stderr.log`
- Key operations (connect, disconnect, speed test, subscription updates)
  emit tracing events for real-time visibility
---

## Status Bar

Bottom line of the terminal. Left side shows connection state and mode context;
right side shows contextual key hints.

**Connection states (left side):**

| State                      | Display                                 |
| -------------------------- | --------------------------------------- |
| Disconnected               | ` Disconnected` (red)                   |
| Connecting                  | ` ⠋ Connecting...` (yellow spinner)     |
| Connected (xray)           | ` Connected [xray]` (green)             |
| Connected (sing-box)       | ` Connected [sing-box]` (green)         |
| Error                      | ` Error: <message>` (red bold)          |
| Testing (individual)       | ` Testing...` (yellow)                  |
| Testing (batch)            | ` Testing: 3/7 profiles...` (yellow)    |

**Mode prefix** is shown before the connection state:

- No prefix for List mode
- ` Settings`, ` Settings > Core`, etc. when in Settings
- ` Add Server`, ` Edit Server` when adding/editing
- ` Speed Test` when speed test menu is open
- ` Help` when help overlay is open

**Update indicator** — `[Update: xray]` or `[Update: xray, sing-box]` (yellow)
appended when updates are available for installed backends.
**Contextual hints (right side):**


| Context                 | Hints                                    |
| ----------------------- | ---------------------------------------- |
| Profiles (disconnected) | `[F1] Actions Log  [Ctrl+Enter] Connect  [Tab] Next  [?] Help  [q/Ctrl+C] Quit` |
| Profiles (connecting)   | `[F1] Actions Log  [Tab] Next  [?] Help  [q/Ctrl+C] Quit`   |
 | Profiles (connected)    | `[F1] Actions Log  [Ctrl+D] Disconnect  [Tab] Next  [?] Help  [q/Ctrl+C] Quit` |
| Settings menu           | `[↑↓] Navigate  [Enter] Open  [Esc] Close` |
| Settings form           | `[Tab/Shift+Tab] Focus  [Enter] Save  [Esc] Cancel` |
| Speed test menu         | `[↑↓] Navigate  [Enter] Select  [Esc] Close` |
| Help overlay            | `[Esc] Close help`                       |
| Logs / Statistics       | `[F1] Actions Log  [?] Help  [q/Ctrl+C] Quit`               |

## Keyboard Shortcuts — Full Reference

### Profiles Tab (List mode)

| Key               | Action                              |
| ----------------- | ----------------------------------- |
| `↑` / `↓`        | Navigate profile list               |
| `PgUp` / `PgDn`  | Page up/down in profile list        |
| `Home` / `End`   | Jump to first/last profile          |
| `Enter`          | Set as active server                |
|| `Ctrl+Enter`     | Connect to selected server          |
|| `Ctrl+G`         | Connect fallback (for terminals without Ctrl+Enter) |
|| `Space`          | Toggle multi-select                 |
|| `Ctrl+A`         | Select all / deselect all           |
|| `a`              | Add new server                      |
| `e`              | Edit selected server                |
| `d`              | Delete selected server(s)           |
| `c`              | Clone selected server               |
| `g`              | Manage subscription groups          |
| `t`              | Open speed test menu                |
| `o`              | Cycle sort column (8 columns)       |
| `/`              | Focus search/filter input           |
| `Ctrl+V`         | Import share URL                    |
 | `Ctrl+Shift+S`   | Copy selected server's share URL    |
| `Ctrl+↑`         | Move selected profile up            |
| `Ctrl+↓`         | Move selected profile down          |
| `Tab` / `Shift+Tab` | Cycle through tabs               |
| `?`              | Toggle help overlay                 |
| `q` / `Ctrl+C`   | Quit application                    |
| `F1`             | Toggle actions log compact/full view |
 | `Ctrl+D`   | Disconnect from active server       |

### Settings Tab

| Key               | Action                              |
| ----------------- | ----------------------------------- |
| `↑` / `↓`        | Navigate settings menu              |
| `Enter`          | Open selected section               |
| `Tab` / `Shift+Tab` | Cycle tabs                       |
| `Esc`            | Close settings / return to menu     |
| `?`              | Toggle help overlay                 |
| `q` / `Ctrl+C`   | Quit application                    |

### Settings Forms

| Key               | Action                              |
| ----------------- | ----------------------------------- |
| `Tab` / `Shift+Tab` | Move focus between fields        |
| `Enter`          | Save and return to menu             |
| `Esc`            | Cancel and return to menu           |
| `Ctrl+C`         | Quit application                    |

### Routing List

| Key               | Action                              |
| ----------------- | ----------------------------------- |
| `↑` / `↓`        | Navigate rules                      |
| `Enter`          | Edit selected rule                  |
| `a`              | Add new rule                        |
| `d`              | Delete selected rule                |
| `r`              | Reorder rules                       |

### Updates Form

| Key               | Action                              |
| ----------------- | ----------------------------------- |
| `C`              | Check for updates                   |
| `D`              | Download and install updates        |
| `Esc`            | Return to settings menu             |

### Logs Tab

| Key               | Action                              |
| ----------------- | ----------------------------------- |
| `↑` / `↓`        | Scroll logs                         |
| `PgUp` / `PgDn`  | Page up/down (20 lines)            |
| `Home`           | Jump to oldest log entry            |
| `End`            | Jump to newest log entry            |
| `Tab` / `Shift+Tab` | Cycle tabs                       |
| `?`              | Toggle help overlay                 |
| `q` / `Ctrl+C`   | Quit application                    |

### Statistics Tab

| Key               | Action                              |
| ----------------- | ----------------------------------- |
| `Tab` / `Shift+Tab` | Cycle tabs                       |
| `?`              | Toggle help overlay                 |
| `q` / `Ctrl+C`   | Quit application                    |

### Anywhere (Global)

| Key               | Action                              |
| ----------------- | ----------------------------------- |
| `q` / `Ctrl+C`   | Quit application                    |
| `?`              | Help overlay (main screens only)    |

---

## Common Workflows

### Adding a server manually

1. From Profiles tab, press `a`
2. Scroll through the protocol picker with `↑↓`
3. Press `Enter` on your protocol (e.g. VMess)
4. Fill in form fields: `Tab` to move focus, type values
5. Press `Enter` to save; profile appears in the list

### Importing a subscription URL

1. From Profiles tab, press `Ctrl+V`
2. Type or paste the subscription URL
3. Press `Enter` to parse and import
4. Profiles are added with automatic dedup based on content hash
5. If parsing fails, an error message appears in red below the input

### Running a speed test

1. Select a profile with `↑↓`
2. Press `t` to open the speed test menu
3. Navigate with `↑↓` (skips the separator line)
4. Press `Enter` on the test type:
   - **Fast Ping** — auto-selects TCP or UDP/QUIC adapter based on protocol
   - **Real Ping** — measures response time through the proxy
   - **Speed Test** — downloads a test payload to measure throughput
   - **UDP Test** — tests UDP forwarding
   - **Fast Ping (All Visible)** — batch fast pings all visible profiles
   - **Fast + Real Ping (All Visible)** — batch fast ping followed by real ping for unsupported protocols
   - **Sort by Delay** — re-sorts the table by delay ascending
   - **Remove Bad Servers** — removes profiles with failed tests
5. Results populate the Delay/Speed columns in the table

### Connecting/Disconnecting

**Connect:**
1. Select a profile with `↑↓`
2. Press `Ctrl+Enter`
3. Status bar shows spinner and `Connecting...`
4. On success: status bar shows `Connected [xray]` (or `[sing-box]`)
5. Statistics tab becomes active with traffic data

**Disconnect:**
 1. Press `Ctrl+D`
2. Status bar returns to `Disconnected`

### Subscription group management

1. From Profiles tab, press `g` to open group overlay
2. **Add:** press `a`, fill fields (name, URL, user_agent, update_interval, core_type), `Enter` saves
3. **Edit:** select a group, press `e`, modify fields, `Enter` saves
4. **Update:** select a group, press `u` to fetch latest subscriptions
5. **Update all:** press `Shift+U` to update every group
6. **Filter:** select a group, press `Enter` to filter profiles by that group
7. **Delete:** select a group, press `d`, confirm with `y`, cancel with `n`/`Esc`
8. Press `Esc` to close the overlay and return to the profile list

---

## Visual Theme

Color tokens used throughout the TUI.

### Tab Bar

| Token             | Style                                |
| ----------------- | ------------------------------------ |
| `TAB_SELECTED`    | White text on Cyan background, Bold  |
| `TAB_DESELECTED`  | White text on DarkGray background    |

### Profile Table

| Token               | Style                            |
| ------------------- | -------------------------------- |
| `TABLE_HEADER`      | White on DarkGray, Bold           |
| `TABLE_ROW_SELECTED`| White on `rgb(50,60,90)`, Bold    |
| `TABLE_ROW_ALT`     | White on `rgb(25,25,35)`          |
| `TABLE_ROW_NORMAL`  | White (no background)             |

### Containers / Borders

| Token                | Style                          |
| -------------------- | ------------------------------ |
| `CONTAINER_BORDER`   | Cyan foreground                |
| `CONTAINER_TITLE`    | Cyan foreground, Bold          |

### Status Bar

| Token                | Style                          |
| -------------------- | ------------------------------ |
| `STATUS_BAR_BG`      | Background `rgb(20,30,60)`     |
| `STATUS_BAR_MODE`    | Cyan foreground, Bold          |

### Feedback / Progress

| Token                | Style                          |
| -------------------- | ------------------------------ |
| `PROGRESS_BAR`       | White on DarkGray              |
| `PROGRESS_FILL`      | Cyan on Cyan                   |
| `SPINNER`            | Yellow foreground, Bold        |

### Semantic Colors

| Token       | Style            | Used For                     |
| ----------- | ---------------- | ---------------------------- |
| `ERROR`     | Red, Bold        | Connection errors            |
| `WARNING`   | Yellow           | Testing state, batch progress|
| `SUCCESS`   | Green            | Connected status             |
| `HINT`      | Gray             | Footer hints, placeholder text|

### Core Type Colors

| Core       | Color  |
| ---------- | ------ |
| xray-core  | Blue   |
| sing-box   | Green  |
| Auto       | White  |

---

## Automated Testing with tui-test

The `tui-test` MCP tools drive xray-tui programmatically for automated testing.
All tools operate on a session identified by `session_id` (default `"default"`).

### Tool Reference

#### Session lifecycle

| Tool                      | Purpose                                      |
| ------------------------- | -------------------------------------------- |
| `launch_tui`              | Start `target/release/xray-tui` in buffer mode|
| `close_session`           | Kill subprocess and clean up                 |
| `get_session_info`        | PID, uptime, command, mode                   |
| `list_sessions`           | All active sessions                          |

Launch the TUI:

```
launch_tui(
    command: "target/release/xray-tui",
    mode: "buffer",
    dimensions: "120x36",
    session_id: "smoke-test"
)
```

#### Waiting and synchronizing

Always wait for the screen to stabilize after any action.

| Tool                       | When to use                                  |
| -------------------------- | -------------------------------------------- |
| `wait_for_stable`          | After every key press or state change        |
| `expect_text(pattern)`     | Block until text appears (async operations)  |
| `wait_for_change(ref)`     | Poll until screen differs from a snapshot    |

```
# After launch, wait for the UI to render
wait_for_stable(session_id: "smoke-test")

# Wait for an async subscription download
expect_text(pattern: "Groups", timeout: 15)
```

#### Reading screen state

| Tool                     | When to use                                  |
| ------------------------ | -------------------------------------------- |
| `capture_screen`         | Full terminal text for debugging             |
| `assert_contains(text)`  | Pass/fail assertion for substring            |
| `get_line(row)`          | Single row content (pixel-perfect layout)    |
| `get_char(row, col)`     | Single character at position                 |
| `get_screen_region(...)`  | Rectangular region of the buffer             |
| `assert_at_position(text, row, col)` | Assert text at exact position |

```
# Verify the Profiles tab header
assert_at_position(text: " Profiles ", row: 0, col: 1)

# Verify status bar shows disconnected
assert_contains(text: "Disconnected")

# Check speed test menu rendering
let third_line = get_line(row: 5)
# Expect "  Speed Test (Selected)" or "► Speed Test (Selected)"
```

#### Sending input

| Tool                   | When to use                                  |
| ---------------------- | -------------------------------------------- |
| `send_keys(text)`      | Type text; `\r` = Enter, `\t` = Tab, `\x1b` = Esc |
| `send_special_keys(keys)` | Arrow keys, function keys, Home/End, Page Up/Down |
| `send_ctrl(key)`       | Ctrl+letter (e.g. `"c"` Ctrl+C, `"v"` Ctrl+V) |

Rules:
- Use `send_special_keys(["down"])`, NOT `send_keys("\x1b[B")` for arrows
- Use `\r` for Enter (raw-mode TUI apps require `\r` not `\n`)
- Use `inter_key_delay: 0.02` for reliable character-by-character typing of long strings
- Always call `wait_for_stable` after sending keys

#### Example: Full smoke test

```
# Start
launch_tui(
    command: "target/release/xray-tui",
    mode: "buffer",
    dimensions: "120x36",
    session_id: "smoke"
)

# Wait for initial render
wait_for_stable(session_id: "smoke")

# Verify initial state
assert_contains(text: "Profiles")
assert_contains(text: "Disconnected")

# Cycle through all 4 tabs
send_special_keys(session_id: "smoke", keys: ["tab"])
wait_for_stable(session_id: "smoke")
assert_contains(text: "Settings")

send_special_keys(session_id: "smoke", keys: ["tab"])
wait_for_stable(session_id: "smoke")
assert_contains(text: "Logs")

send_special_keys(session_id: "smoke", keys: ["tab"])
wait_for_stable(session_id: "smoke")
assert_contains(text: "Statistics")

send_special_keys(session_id: "smoke", keys: ["tab"])
wait_for_stable(session_id: "smoke")
assert_contains(text: "Profiles")

# Open help
send_keys(session_id: "smoke", keys: "?")
wait_for_stable(session_id: "smoke")
assert_contains(text: "Keyboard Shortcuts")

# Close help
send_keys(session_id: "smoke", keys: "\x1b")
wait_for_stable(session_id: "smoke")

# Quit
send_ctrl(session_id: "smoke", key: "q")
wait_for_stable(session_id: "smoke")

# Cleanup
close_session(session_id: "smoke")
```

#### Example: Subscription import

```
launch_tui(
    command: "target/release/xray-tui",
    mode: "buffer",
    dimensions: "120x36",
    session_id: "sub-test"
)
wait_for_stable(session_id: "sub-test")

# Open group overlay
send_keys(session_id: "sub-test", keys: "g")
wait_for_stable(session_id: "sub-test")
assert_contains(text: "Groups")

# Add group: press 'a'
send_keys(session_id: "sub-test", keys: "a")
wait_for_stable(session_id: "sub-test")
assert_contains(text: "Group")

# Type group name
send_keys(session_id: "sub-test", keys: "My Group", inter_key_delay: 0.02)
# Tab to URL field
send_special_keys(session_id: "sub-test", keys: ["tab"])
# Type URL
send_keys(session_id: "sub-test", keys: "https://example.com/sub", inter_key_delay: 0.02)
# Save
send_keys(session_id: "sub-test", keys: "\r")
wait_for_stable(session_id: "sub-test")

# Select group and update
send_special_keys(session_id: "sub-test", keys: ["down"])
send_keys(session_id: "sub-test", keys: "u")
expect_text(session_id: "sub-test", pattern: "Profiles", timeout: 15)

# Close overlay
send_keys(session_id: "sub-test", keys: "\x1b")
wait_for_stable(session_id: "sub-test")

close_session(session_id: "sub-test")
```

#### Example: Speed test (with connected core)

```
launch_tui(
    command: "target/release/xray-tui",
    mode: "buffer",
    dimensions: "120x36",
    session_id: "speed-test"
)
wait_for_stable(session_id: "speed-test")

# Connect (assumes a profile exists and is selected)
send_ctrl(session_id: "speed-test", key: "\r")
expect_text(session_id: "speed-test", pattern: "Connected", timeout: 15)

# Open speed test menu
send_keys(session_id: "speed-test", keys: "t")
wait_for_stable(session_id: "speed-test")
assert_contains(text: "Speed Test")
# Select first test (Fast Ping) and run
send_keys(session_id: "speed-test", keys: "\r")
wait_for_stable(session_id: "speed-test")

close_session(session_id: "speed-test")
```

### Important notes

- Always wait for the screen to stabilize (`wait_for_stable`) after every action
- Prefer `send_special_keys` over raw `\x1b` sequences for arrow/function keys —
  raw escape sequences are unreliable in buffer mode
- Use `inter_key_delay: 0.02` for reliable character-by-character typing of
  long strings (URLs, names)
- Always call `close_session` at the end to clean up the subprocess
- Use 10s+ timeout for slow operations (subscription fetch, speed test) via
  `expect_text` or `wait_for_stable(timeout: 10)`
- `send_keys` uses `\r` for Enter (not `\n`)
- For delete confirmation dialogs: `y`/`n` are plain key presses, use
  `send_keys("y")` or `send_keys("n")`
- Check `get_session_info` for quick debugging: PID, uptime, dimensions
