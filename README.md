# ⌘ macterm

[🇺🇸 English](README.md) · [🇨🇳 中文](README-zh.md)

> A modern multi-terminal TUI multiplexer for macOS — split panes, tabs, animations, built with Rust + Ratatui.

![status](https://img.shields.io/badge/status-beta-blue)
![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-MIT-green)
[![CI](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/your/repo)
![macOS](https://img.shields.io/badge/platform-macOS-lightgrey)

---

## 📸 Screenshot

<!-- TODO: Add screenshot/GIF of macterm in action -->

---

## 🚀 Features

### 🖥️ Terminal Management

- **Multi-tab terminals** — multiple terminal sessions in one window, switch with `Alt+←→`
- **Split panes** — split horizontally (`Ctrl+D`) or vertically (`Ctrl+E`) into resizable panes
- **Pane navigation** — `Ctrl+↑↓←→` to move focus between panes
- **Tab scrolling** — `◀▶` arrows when tabs overflow, auto-scroll to active tab
- **File tree sidebar** — `Ctrl+F` to toggle, reads live directory listing (sorted, dirs first)

### 🎮 Controls

- **Drag-to-resize** — click and drag split borders to resize panes in real-time
- **Search overlay** — `Ctrl+S` to find text in the active pane, Enter/Tab navigation
- **Confirmation dialogs** — confirm before closing a pane or quitting
- **Help overlay** — `Ctrl+H` shows all keybindings
- **Command palette** — `Ctrl+P` for quick commands
- **SIGWINCH propagation** — PTY sessions properly resize when the window changes

### 🎨 Interface

- **Brand header** — bold "MACTERMINAL" logo with tab bar
- **Pane title bar** — 1-line header inside each pane with `[N]` label
- **Pane number overlays** — numbered `[1]` `[2]` labels in pane borders
- **Rounded borders** — `╭─╮` style pane borders with `║═╬` double-line separators
- **Terminal-default colors** — all UI chrome uses terminal theme colors, no hardcoded palette
- **Status bar** — tab count, pane count, status messages with auto-fade

### ⚡ Performance

- **Async event loop** — 60fps rendering via `tokio::select!` — no input lag
- **Proper terminal emulation** — vt100 parser with full ANSI/ECMA-48 support

---

## Quick Start

```bash
# Run directly
cargo run

# Start with file tree sidebar
cargo run -- -f

# Install
cargo build --release
./target/release/macterm
```

### CLI Options

```
Usage: macterm [OPTIONS]

Options:
  -f, --file-tree     Show file tree sidebar on startup
  -d, --dir <DIR>     Start in a specific workspace directory
  -n, --panes <N>     Number of terminal panes to open initially [default: 1]
  -h, --help          Print help
  -V, --version       Print version
```

---

## ⌨️ Keybindings

| Shortcut | Action |
|---|---|
| `Ctrl+Q` | Quit (with confirmation) |
| **Panes** | |
| `Ctrl+D` | Split pane right (horizontal) |
| `Ctrl+E` | Split pane down (vertical) |
| `Ctrl+W` | Close active pane (with confirmation) |
| `Ctrl+Z` | Zoom / unzoom active pane |
| `Ctrl+↑↓←→` | Focus next/previous pane |
| `PgUp` / `PgDn` | Scroll back / forward (1 page) |
| `Ctrl+Space` | Quick pane jump (display numbers) |
| `Ctrl+Tab` | Fullscreen pane cycle (next) |
| `Ctrl+Shift+Tab` | Fullscreen pane cycle (previous) |
| `Ctrl+G` | Toggle fullscreen pane mode |
| **Mouse** | |
| Click pane | Focus pane |
| Drag border | Resize split panes |
| Scroll wheel | Scroll back / forward |
| Drag select | Select text (auto-copy to clipboard) |
| **Tabs** | |
| `Ctrl+T` | New tab |
| `Ctrl+Shift+W` | Close active tab |
| `Alt+←→` | Switch tab prev/next (auto-scroll) |
| `Alt+1-9` | Switch to tab by number |
| **Interface** | |
| `Ctrl+P` | Command palette |
| `Ctrl+F` | File tree (toggle) |
| `Ctrl+S` | Search (full scrollback) |
| `Ctrl+Shift+V` | Paste from clipboard |
| `Ctrl+H` | Help overlay |
| **Search** (when open) | |
| `Enter` / `Tab` | Next match (auto-scroll) |
| `Shift+Tab` | Previous match |
| `Esc` | Close search |
| **Pane Jump** (when open) | |
| `1-9` | Jump to pane N |
| `Esc` | Cancel |
| **Confirm Dialog** (when open) | |
| `Enter` / `Y` | Confirm action |
| `Esc` / `N` / `Q` | Cancel |
| **Shell Input** | |
| `Ctrl+letter` | Standard control codes (EOF, SIGINT, etc.) |
| `Alt+letter` | Alt codes (ESC+letter) |
| Arrow keys, Home, End, etc. | Passthrough to shell |

---

## 📋 Changelog

### 0.2.5 — Copy/Paste, Fullscreen Panes, Pane Jump, Full Scrollback Search, Config

- **Copy/paste**: mouse drag to select text (inverse video highlight), auto-copies to system clipboard on release. `Ctrl+Shift+V` pastes clipboard content into active pane.
- **Fullscreen pane cycling**: `Ctrl+G` toggles fullscreen pane mode — each pane takes the full terminal area. `Ctrl+Tab` / `Ctrl+Shift+Tab` cycles through panes. Status bar shows `[FULL]` indicator.
- **Quick pane jump**: `Ctrl+Space` overlays numbered labels on all panes, press a digit to jump directly to that pane (tmux display-panes style).
- **Full scrollback search**: `Ctrl+S` now searches the entire scrollback buffer, not just the visible screen. Matches auto-scroll viewport to position.
- **Config file**: `~/.config/macterm/config.toml` supports `scrollback_lines`, `default_panes`, `shell` path, and custom `keybindings`. Config-loaded settings override hardcoded defaults.
- **CLI enhancements**: `macterm -e /bin/bash` to specify shell, `macterm -n 4` for multi-pane startup, `macterm -f` for file tree, `macterm -d ~/project` for working directory. CLI flags override config file.
- **Configurable scrollback**: `vt100::Parser::new(rows, cols, scrollback_lines)` — buffer size configurable via config file, defaults to 10,000 rows.
- **Status bar enhancements**: shows mode indicators (`[ZOOM]`, `[FULL]`), cleaner layout with `^Pcmd` display.
- **Help overlay**: updated with all new keybindings (fullscreen, pane jump, paste, search).

### 0.2.4 — Scrollback, Zoom, Tab Close, Mouse Wheel

- **Scrollback support**: `PgUp`/`PgDn` scroll the active pane back one page. Scroll indicator `[↑]` appears in the pane title bar when scrolled. Any key input returns to bottom.
- **Mouse wheel scroll**: `ScrollUp`/`ScrollDown` events scroll the active pane back/forward (1 page per tick).
- **Pane zoom**: `Ctrl+Z` toggles full-screen zoom on the active pane, hiding other splits. `Ctrl+Z` again restores the layout.
- **Tab close**: `Ctrl+Shift+W` closes the active tab and all its panes.
- **Auto-scroll reset**: writing to a pane (keyboard input) automatically resets scrollback to bottom.

### 0.2.3 — Color Strip, Keyboard Conflict Fix, PTY Size Fix

- **Color stripped from entire app**: all custom `Color::Rgb(...)` removed — header, status bar, pane borders, title bars, overlays, sidebar all use terminal defaults. No hardcoded palette anywhere.
- **Dead code removal**: `ColorAnimation`, `AnimationTimeline`, `animated_border`, `focus_animation` removed.
- **Keyboard shortcut fix**: reverted all Alt/Option shortcuts back to Ctrl+letter — Option key on macOS sends Unicode characters by default, not modifier events. All `Ctrl+letter` combos work reliably on macOS Terminal.
- **PTY size fix**: `resize_active_panes()` now accounts for border (2 cols, 2 rows) and title bar (1 row) — shell output no longer overflows or wraps incorrectly
- **Initial window sizing fix**: PTY is resized to terminal dimensions on startup, not left at 80×24
- **Version display fix**: version number no longer truncated in header; now reads from `CARGO_PKG_VERSION`

### 0.2.2 — README & Release Packaging

- Full README overhaul: Features, Keybindings, and Changelog synchronized with all v0.2.x changes
- Keybindings table: added Search overlay, Confirm Dialog, tab scrolling, pane close/quit confirm sections
- Fixed table separator rendering bug (incorrect column count)
- Release build: `cargo build --release` produces `target/release/macterm` (3.5M arm64 Mach-O)
- Distribution package in `dist/`:
  - `macterm` — standalone arm64 binary
  - `install.sh` — one-command installer (`install -m 755` to `/usr/local/bin`)
  - `macterm-aarch64-macos.tar.gz` — 1.4MB compressed distribution archive

### 0.2.1 — Search Overlay

- **Search overlay (E1)**: `Alt+S` opens case-insensitive search in the active pane
  - Real-time matching as you type with match counter (`3/42`)
  - Enter/Tab to cycle forward, Shift+Tab to go backward
  - Esc to close
- Help overlay updated with new keybinding reference

### 0.2.0 — UI Polish & Animations

#### Animations (Phase 2)
- **Wave gradient header (A1)**: per-character flowing color wave on "MACTERMINAL" brand
- **Glowing border (B2)**: sinusoidal pulse on the active pane's border (cyan brightness oscillation)
- **Focus breathing (C2)**: ±8 brightness modulation on the active pane's content cells

#### UI Enhancements (Phase 1)
- **Rounded borders**: pane borders use `╭╮╰╯` rounded corner glyphs (`symbols::border::ROUNDED`)
- **Double-line separators**: `║` vertical and `═` horizontal split lines with `╬` cross-point detection
- **Pane number overlays**: sequential `[N]` labels in each pane's border title
- **Tab indicators**: `●`/`○` bullets, `▏` separators, `▔` underline on active tab
- **Pane title bar (B5)**: colored 1-line header inside each pane showing `[N]` label

#### Confirmation Dialogs (E4/E5)
- **Close pane confirm**: when pressing `Ctrl+W` with multiple panes, shows confirmation dialog
- **Quit confirm**: `Ctrl+Q` now shows confirmation before quitting
- Styled overlay with `[Y]es  [N]o  [Esc]` buttons
- Enter/Y to confirm, Esc/N/Q to cancel

#### Command Exit Notifications (E3)
- Colored exit code display: `✓ green` for exit code 0, `✗ red` for non-zero
- Auto-fades after ~2 seconds
- `StatusBar.message_color` infrastructure for per-message coloring

#### Tokyo Night Color Calibration (F2)
- OSC 10/11 sequences fed to vt100 parser set default fg `#a9b1d6` / bg `#1a1b26`
- Full ANSI palette (colors 0–15) calibrated to Tokyo Night values
- `render_screen()` resolves `Color::Default` to Tokyo Night colors instead of `Color::Reset`
- Cohesive dark theme across all UI chrome and terminal content

#### Tab Scrolling (A4)
- `tab_scroll_offset` state tracks current scroll position
- Auto-scroll on tab switch (`Alt+←→`, `Alt+1-9`, `Ctrl+T`)
- `◀▶` arrow indicators when tabs are hidden off-screen
- Works with the tab bar's proportional-width layout

#### File Tree Improvements (D1)
- Live `cwd` directory listing via `std::fs::read_dir`
- Directories sorted first, then alphabetical
- `📁` icon for directories, plain spacing for files
- Scrolling support for long listings

### 0.1.0 (Initial Release)

#### Core
- Rust workspace with 3 crates: `macterm` (binary), `macterm-core` (data model), `macterm-tui` (terminal UI)
- Split/binary tree data model — `SplitNode` with `Leaf` and `Split` variants
- PTY wrapper using `portable-pty` + `vt100` parser + tokio mpsc event channel
- Async event loop with `tokio::select!` — multiplexes keyboard, PTY events, and frame ticks

#### UI & Rendering
- Ratatui-based terminal UI with 60fps rendering
- Per-pane block borders with active pane highlighting (cyan border)
- vt100 screen rendering with color, bold, italic, underline support
- Cursor positioning from vt100's `Screen::cursor_position()`
- Gradient "MACTERMINAL" 2-line brand header (cyan→purple per-character)
- Status bar with tab/pane counts, messages
- Help overlay with styled section headers and key/desc/note columns

#### Split Panes
- Horizontal (`Ctrl+D`) and vertical (`Ctrl+E`) pane splitting at 50/50 ratio
- Pane close (`Ctrl+W`) with automatic tree rebalancing
- `pane_rects_from_tree()` — recursive algorithm computing exact per-pane Rect from split tree
- Focus navigation via `Ctrl+↑↓←→`

#### Drag-to-Resize (v0.1.0 feature)
- Click and drag split borders to adjust pane ratio in real-time
- `find_border_at_position()` — recursive tree walk with 1-cell tolerance
- Cyan highlight on the border being dragged
- Delta-based ratio update, clamped 0.1–0.9

#### Tabs
- Multi-tab support with `Ctrl+T`
- Tab switching via `Alt+←→` and `Alt+1-9`
- On tab switch: PTY sessions resize to their split-tree dimensions

#### PTY & Terminal
- `TERM=xterm-256color` set in environment
- Multi-byte UTF-8 input support
- Ctrl/Alt modifier handling (`Ctrl+C → 0x03`, `Alt+X → ESC+X`)
- `RwLock<vt100::Parser>` with `try_read()` — render never blocks on parser contention
- **SIGWINCH propagation** — `PtySession::resize()` calls `master.resize()` to inform the kernel, so the shell redraws at the correct terminal size
- Content area calculation accounts for file tree sidebar (fixes misaligned output with file tree open)

#### Bug Fixes
- Input swallowing: fixed by using `tokio::select!` async event loop + `RwLock` instead of `Mutex`
- Lag: fixed by `try_read()` in render path + separate reader thread
- Overlapping fill loops: merged 3 loops into 1 single-pass render
- Tab switch pane sizes: added `resize_active_panes()` call on tab switch
- Split ratio integrity: ratio clamped to `[0.1, 0.9]` to prevent collapsed panes
- Window resize: proper per-pane PTY resize from split tree (not hardcoded initial size)
- Mouse click area: content area y-position corrected from 1 to 2 (below 2-line header)

---

## 🏗️ Architecture

```
macterm/
├── src/
│   └── main.rs              # CLI entrypoint (clap)
├── crates/
│   ├── macterm-core/        # 📦 Data model
│   │   ├── src/layout.rs    # SplitNode binary tree
│   │   ├── src/pane.rs      # PaneId, SplitDirection
│   │   ├── src/workspace.rs # Workspace→Tab→SplitNode hierarchy
│   │   └── src/lib.rs
│   └── macterm-tui/         # 🎨 Terminal UI layer
│       ├── src/app.rs       # App state, PTY management
│       ├── src/ui.rs        # Event loop, keyboard/mouse handlers
│       ├── src/pty.rs       # PTY session (portable-pty + vt100)
│       └── src/widgets/
│           ├── pane_grid.rs # Split tree rendering, border drag feedback
│           ├── header.rs    # Gradient MACTERMINAL header + tab bar
│           └── status_bar.rs
```

| Crate | Description |
|---|---|
| `macterm` (binary) | CLI entrypoint, argument parsing via `clap` |
| `macterm-core` | Data model: `SplitNode` tree, `PaneId`, workspace→tab→split hierarchy |
| `macterm-tui` | Terminal UI: app state, event loop, PTY sessions, widgets |

---

## 🔧 Building

```bash
# Build
cargo build

# Release build
cargo build --release

# Run
cargo run
```

**Dependencies**: Rust 1.70+, macOS (cross-platform via portable-pty / Ratatui)

---

## 🤝 Contributing

Contributions, issues, and feature requests are welcome!  
Feel free to open an [issue](https://github.com/your/repo/issues) or submit a pull request.

---

## 📄 License

MIT
