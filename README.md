# ⌘ macterm

> A modern multi-terminal TUI multiplexer for macOS — split panes, tabs, animations, built with Rust + Ratatui.

![demo](https://img.shields.io/badge/status-beta-blue)
![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-MIT-green)

---

## Features

- **Multi-tab terminals** — multiple terminal sessions in one window, switch with `Alt+←→`
- **Split panes** — split horizontally (`Alt+D`) or vertically (`Alt+E`) into resizable panes
- **Drag-to-resize** — click and drag split borders to resize panes in real-time
- **Pane navigation** — `Ctrl+↑↓←→` to move focus between panes
- **Brand header** — bold "MACTERMINAL" logo with tab bar
- **Tab scrolling** — `◀▶` arrows when tabs overflow, auto-scroll to active tab
- **Pane title bar** — 1-line header inside each pane with `[N]` label
- **Pane number overlays** — numbered `[1]` `[2]` labels in pane borders
- **Rounded borders** — `╭─╮` style pane borders with `║═╬` double-line separators
- **Search overlay** — `Alt+S` to find text in the active pane, Enter/Tab navigation
- **Confirmation dialogs** — confirm before closing a pane or quitting
- **Help overlay** — `Alt+H` shows all keybindings
- **Command palette** — `Alt+P` for quick commands
- **File tree sidebar** — `Alt+F` to toggle, reads live directory listing (sorted, dirs first)
- **Terminal-default colors** — all UI chrome uses terminal theme colors, no hardcoded palette
- **Status bar** — tab count, pane count, status messages with auto-fade
- **Async event loop** — 60fps rendering via `tokio::select!` — no input lag
- **Proper terminal emulation** — vt100 parser with full ANSI/ECMA-48 support
- **SIGWINCH propagation** — PTY sessions properly resize when the window changes

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

## Keybindings

| Shortcut | Action |
|---|---|
| `Ctrl+Q` | Quit (with confirmation) |
| **Panes** | |
| `Alt+D` | Split pane right (horizontal) |
| `Alt+E` | Split pane down (vertical) |
| `Alt+W` | Close active pane (with confirmation) |
| `Ctrl+↑↓←→` | Focus next/previous pane |
| **Mouse** | |
| Click pane | Focus pane |
| Drag border | Resize split panes |
| **Tabs** | |
| `Alt+T` | New tab |
| `Alt+←→` | Switch tab prev/next (auto-scroll) |
| `Alt+1-9` | Switch to tab by number |
| **Interface** | |
| `Alt+P` | Command palette |
| `Alt+F` | File tree (toggle) |
| `Alt+S` | Search in active pane |
| `Alt+H` | Help overlay |
| **Search** (when open) | |
| `Enter` / `Tab` | Next match |
| `Shift+Tab` | Previous match |
| `Esc` | Close search |
| **Confirm Dialog** (when open) | |
| `Enter` / `Y` | Confirm action |
| `Esc` / `N` / `Q` | Cancel |
| **Shell Input** | |
| `Ctrl+letter` | Standard control codes (EOF, SIGINT, etc.) |
| `Alt+letter` | Alt codes (ESC+letter) |
| Arrow keys, Home, End, etc. | Passthrough to shell |
| ← All Ctrl+letter combos pass through to the shell — no TUI conflicts | |

---

## Changelog

### 0.2.3 — Color Strip, Keyboard Conflict Fix, PTY Size Fix

- **Color stripped from entire app**: all custom `Color::Rgb(...)` removed — header, status bar, pane borders, title bars, overlays, sidebar all use terminal defaults. No hardcoded palette anywhere.
- **Dead code removal**: `ColorAnimation`, `AnimationTimeline`, `animated_border`, `focus_animation` removed.
- **Keyboard conflict fix**: all TUI shortcuts moved from `Ctrl+letter` (which conflicted with shell readline) to `Alt+letter`:
  - `Alt+D` split right, `Alt+E` split down, `Alt+W` close pane
  - `Alt+T` new tab, `Alt+P` command palette, `Alt+F` file tree, `Alt+H` help
  - `Ctrl+Q` quit kept (standard TUI convention)
  - All `Ctrl+letter` combos now pass through to the shell properly
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

## Architecture

```
macterm/
├── src/
│   └── main.rs              # CLI entrypoint (clap)
├── crates/
│   ├── macterm-core/        # Data model
│   │   ├── src/layout.rs    # SplitNode binary tree
│   │   ├── src/pane.rs      # PaneId, SplitDirection
│   │   ├── src/workspace.rs # Workspace→Tab→SplitNode hierarchy
│   │   └── src/lib.rs
│   └── macterm-tui/         # Terminal UI layer
│       ├── src/app.rs       # App state, PTY management
│       ├── src/ui.rs        # Event loop, keyboard/mouse handlers
│       ├── src/pty.rs       # PTY session (portable-pty + vt100)
│       └── src/widgets/
│           ├── pane_grid.rs # Split tree rendering, border drag feedback
│           ├── header.rs    # Gradient MACTERMINAL header + tab bar
│           └── status_bar.rs
```

---

## Building

```bash
# Build
cargo build

# Release build
cargo build --release

# Run
cargo run
```

**Dependencies**: Rust 1.70+, macOS (cross-platform via portable-pty/Ratatui)

---

## License

MIT
