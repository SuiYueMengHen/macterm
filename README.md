# ⌘ macterm

> A modern multi-terminal TUI multiplexer for macOS — split panes, tabs, animations, built with Rust + Ratatui.

![demo](https://img.shields.io/badge/status-beta-blue)
![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-MIT-green)

---

## Features

- **Multi-tab terminals** — multiple terminal sessions in one window, switch with `Alt+←→`
- **Split panes** — split horizontally (`Ctrl+D`) or vertically (`Ctrl+E`) into resizable panes
- **Drag-to-resize** — click and drag split borders to resize panes in real-time
- **Pane navigation** — `Ctrl+↑↓←→` to move focus between panes
- **Branded header** — gradient "MACTERMINAL" logo with tab bar
- **Help overlay** — `Ctrl+H` shows all keybindings
- **Command palette** — `Ctrl+P` for quick commands
- **File tree sidebar** — `Ctrl+F` to toggle (`Ctrl+P → files` also works)
- **Status bar** — tab count, pane count, status messages
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
| `Ctrl+Q` | Quit |
| **Panes** | |
| `Ctrl+D` | Split pane right (horizontal) |
| `Ctrl+E` | Split pane down (vertical) |
| `Ctrl+W` | Close active pane |
| `Ctrl+↑↓←→` | Focus next/previous pane |
| **Mouse** | |
| Click pane | Focus pane |
| Drag border | Resize split panes |
| **Tabs** | |
| `Ctrl+T` | New tab |
| `Alt+←→` | Switch tab prev/next |
| `Alt+1-9` | Switch to tab by number |
| **Interface** | |
| `Ctrl+P` | Command palette |
| `Ctrl+F` | File tree (toggle) |
| `Ctrl+H` | Help overlay |
| **Shell Input** | |
| `Ctrl+C` / `Ctrl+D` / etc. | Standard control codes sent to shell |
| `Alt+letter` | Alt codes (ESC+letter) |
| Arrow keys, Home, End, etc. | Passthrough to shell |

---

## Changelog

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
│       ├── src/animations.rs # ColorAnimation lerp/easing
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
