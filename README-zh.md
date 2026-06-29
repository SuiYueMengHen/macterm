# ⌘ macterm

[🇺🇸 English](README.md) · [🇨🇳 中文](README-zh.md)

> 一款适用于 macOS 的现代化多终端 TUI 复用器 —— 支持分屏、标签页、动画，基于 Rust + Ratatui 构建。

![status](https://img.shields.io/badge/状态-beta-blue)
![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)
![License](https://img.shields.io/badge/许可证-MIT-green)

---

## 功能特性

- **多标签终端** — 一个窗口中管理多个终端会话，使用 `Alt+←→` 切换
- **分屏面板** — 水平分屏（`Ctrl+D`）或垂直分屏（`Ctrl+E`），面板可自由调整大小
- **拖拽调整大小** — 点击并拖拽分割边框，实时调整面板比例
- **面板导航** — `Ctrl+↑↓←→` 在面板之间移动焦点
- **品牌头部** — 醒目的 "MACTERMINAL" 标识与标签栏
- **标签滚动** — 标签溢出时显示 `◀▶` 箭头，自动滚动到当前活动标签
- **面板标题栏** — 每个面板内部显示带有 `[N]` 标签的 1 行标题
- **面板编号覆盖层** — 面板边框上显示编号 `[1]` `[2]` 标签
- **圆角边框** — 使用 `╭─╮` 风格的面板边框搭配 `║═╬` 双线分隔符
- **搜索覆盖层** — `Ctrl+S` 在当前面板中查找文本，Enter/Tab 导航
- **确认对话框** — 关闭面板或退出前进行确认
- **帮助覆盖层** — `Ctrl+H` 显示所有快捷键
- **命令面板** — `Ctrl+P` 快速执行命令
- **文件树侧边栏** — `Ctrl+F` 切换显示，实时读取目录列表（排序，目录优先）
- **终端默认颜色** — 所有 UI 元素使用终端主题颜色，无硬编码调色板
- **状态栏** — 标签计数、面板计数、状态消息（自动淡出）
- **异步事件循环** — 通过 `tokio::select!` 实现 60fps 渲染，无输入延迟
- **完善的终端模拟** — 基于 vt100 解析器，完整支持 ANSI/ECMA-48
- **SIGWINCH 信号传播** — PTY 会话在窗口大小变化时正确调整

---

## 快速开始

```bash
# 直接运行
cargo run

# 启动时显示文件树侧边栏
cargo run -- -f

# 安装到系统
cargo build --release
./target/release/macterm
```

### 命令行选项

```
用法: macterm [选项]

选项:
  -f, --file-tree     启动时显示文件树侧边栏
  -d, --dir <DIR>     指定工作目录启动
  -n, --panes <N>     初始打开的终端面板数量 [默认: 1]
  -h, --help          打印帮助信息
  -V, --version       打印版本信息
```

---

## 快捷键

| 快捷键 | 操作 |
|---|---|
| `Ctrl+Q` | 退出（带确认） |
| **面板操作** | |
| `Ctrl+D` | 向右分屏（水平） |
| `Ctrl+E` | 向下分屏（垂直） |
| `Ctrl+W` | 关闭当前面板（带确认） |
| `Ctrl+Z` | 缩放/取消缩放当前面板 |
| `Ctrl+↑↓←→` | 切换面板焦点 |
| `PgUp` / `PgDn` | 向后/向前滚动（一页） |
| `Ctrl+Space` | 快速面板跳转（显示编号） |
| `Ctrl+Tab` | 全屏面板循环（下一个） |
| `Ctrl+Shift+Tab` | 全屏面板循环（上一个） |
| `Ctrl+G` | 切换全屏面板模式 |
| **鼠标操作** | |
| 点击面板 | 切换焦点至该面板 |
| 拖拽边框 | 调整面板大小 |
| 滚轮 | 向后/向前滚动 |
| 拖拽选择 | 选择文本（自动复制到剪贴板） |
| **标签页操作** | |
| `Ctrl+T` | 新建标签页 |
| `Ctrl+Shift+W` | 关闭当前标签页 |
| `Alt+←→` | 切换标签页上一个/下一个（自动滚动） |
| `Alt+1-9` | 按编号切换标签页 |
| **界面操作** | |
| `Ctrl+P` | 命令面板 |
| `Ctrl+F` | 文件树（切换） |
| `Ctrl+S` | 搜索（整个回滚缓冲区） |
| `Ctrl+Shift+V` | 从剪贴板粘贴 |
| `Ctrl+H` | 帮助覆盖层 |
| **搜索模式**（打开后） | |
| `Enter` / `Tab` | 下一个匹配（自动滚动） |
| `Shift+Tab` | 上一个匹配 |
| `Esc` | 关闭搜索 |
| **面板跳转**（打开后） | |
| `1-9` | 跳转到编号 N 的面板 |
| `Esc` | 取消 |
| **确认对话框**（打开后） | |
| `Enter` / `Y` | 确认操作 |
| `Esc` / `N` / `Q` | 取消 |
| **Shell 输入** | |
| `Ctrl+字母` | 标准控制码（EOF、SIGINT 等） |
| `Alt+字母` | Alt 码（ESC+字母） |
| 方向键、Home、End 等 | 透传给 Shell |

---

## 更新日志

### 0.2.6 — 安全增强、性能优化、温度修复

- **缓存初始化修复**：`cached_parsers`/`pane_indices` 在启动时即填充——修复首次渲染面板为空的问题。切换标签页后也刷新缓存，确保面板编号正确。
- **温度传感器修复**：在 Apple Silicon 上正确选择 CPU die 传感器（`PMU tdie*`）替代热敏校准传感器（`PMU tcal`）。显示 "Tdie 43°C"，带三级优先级回退。
- **死代码清理**：删除 `tab_bar.rs`（114 行）、`animations.rs`；移除 `tachyonfx`、`tokio-stream`、`color-eyre` 和可选的 `serde` 依赖。
- **依赖精简**：tokio 从 `"full"` 缩减为最小功能集；ratatui 从 `"all-widgets"` 缩减为默认功能。
- **安全修复**：4 处不安全的 `unwrap()` 替换为安全回退模式。
- **线程安全**：`PtySession` 的 `Drop` 实现在关闭面板时终止后台读取线程。
- **内存缓存**：解析器和面板索引的 HashMap 现在缓存使用，仅在会话/标签变更时重建，而非每帧重建。
- **搜索高亮**：匹配项在面板内容中显示为亮黄底深灰背景高亮。
- **滚动计数器**：面板标题显示 `↑ N` 滚动行数而非仅箭头。
- **分屏区域统一**：4 处分屏计算合并为 `compute_split_areas()` 函数——修复渲染/点击/调整大小时的截断与舍入不一致问题。
- **模块提取**：`SysStats` → `stats.rs`，`ConfirmAction` → `confirmation.rs`（`app.rs` 减少约 80 行）。
- **语义化状态颜色**：`set_status_success()`（绿色）、`set_status_error()`（红色）。
- **Clippy 清理**：修复 13 个警告（`collapsible_if`、`redundant_closure`、`manual_clamp`、`unnecessary_cast` 等）。

### 0.2.5 — 复制/粘贴、全屏面板、面板跳转、全回滚搜索、配置文件

- **复制/粘贴**：鼠标拖拽选择文本（反色高亮显示），松开后自动复制到系统剪贴板。`Ctrl+Shift+V` 将剪贴板内容粘贴到活动面板。
- **全屏面板循环**：`Ctrl+G` 切换全屏面板模式 —— 每个面板占据整个终端区域。`Ctrl+Tab` / `Ctrl+Shift+Tab` 循环切换面板。状态栏显示 `[FULL]` 指示器。
- **快速面板跳转**：`Ctrl+Space` 在所有面板上覆盖编号标签，按数字键直接跳转到对应面板（类似 tmux display-panes）。
- **全回滚搜索**：`Ctrl+S` 现在搜索整个回滚缓冲区，而不仅仅是可见屏幕。匹配项自动滚动视口到对应位置。
- **配置文件**：`~/.config/macterm/config.toml` 支持 `scrollback_lines`、`default_panes`、`shell` 路径和自定义 `keybindings`。配置文件中的设置覆盖硬编码默认值。
- **CLI 增强**：`macterm -e /bin/bash` 指定 Shell，`macterm -n 4` 多面板启动，`macterm -f` 显示文件树，`macterm -d ~/project` 指定工作目录。CLI 参数覆盖配置文件。
- **可配置回滚**：`vt100::Parser::new(rows, cols, scrollback_lines)` —— 缓冲区大小可通过配置文件配置，默认为 10,000 行。
- **状态栏增强**：显示模式指示器（`[ZOOM]`、`[FULL]`），更清晰的布局，显示 `^Pcmd`。
- **帮助覆盖层**：更新了所有新快捷键（全屏、面板跳转、粘贴、搜索）。

### 0.2.4 — 回滚、缩放、关闭标签页、鼠标滚轮

- **回滚支持**：`PgUp`/`PgDn` 将活动面板向上滚动一页。滚动时面板标题栏显示 `[↑]` 指示器。任意按键输入返回底部。
- **鼠标滚轮滚动**：`ScrollUp`/`ScrollDown` 事件将活动面板向后/向前滚动（每次滚动一页）。
- **面板缩放**：`Ctrl+Z` 切换活动面板的全屏缩放，隐藏其他分屏。再次按 `Ctrl+Z` 恢复布局。
- **关闭标签页**：`Ctrl+Shift+W` 关闭当前标签页及其所有面板。
- **自动滚动重置**：向面板写入内容（键盘输入）自动将回滚重置到底部。

### 0.2.3 — 移除颜色、键盘冲突修复、PTY 尺寸修复

- **全局移除颜色**：所有自定义 `Color::Rgb(...)` 已移除 —— 头部、状态栏、面板边框、标题栏、覆盖层、侧边栏均使用终端默认颜色。任何地方均无硬编码调色板。
- **死代码清理**：移除了 `ColorAnimation`、`AnimationTimeline`、`animated_border`、`focus_animation`。
- **键盘快捷键修复**：将所有 Alt/Option 快捷键恢复为 Ctrl+字母 —— macOS 上的 Option 键默认发送 Unicode 字符而非修饰事件。所有 `Ctrl+字母` 组合键在 macOS 终端上均可靠工作。
- **PTY 尺寸修复**：`resize_active_panes()` 现在考虑了边框（2 列、2 行）和标题栏（1 行）—— Shell 输出不再溢出或换行错误。
- **初始窗口尺寸修复**：PTY 在启动时调整为终端尺寸，不再默认为 80×24。
- **版本显示修复**：头部版本号不再截断；现在从 `CARGO_PKG_VERSION` 读取。

### 0.2.2 — README 与发布打包

- 全面更新 README：功能特性、快捷键和更新日志与所有 v0.2.x 变更同步
- 快捷键表格：添加了搜索覆盖层、确认对话框、标签滚动、关闭面板/退出确认等章节
- 修复了表格分隔符渲染错误（列数不正确）
- 发布构建：`cargo build --release` 生成 `target/release/macterm`（3.5M arm64 Mach-O）
- `dist/` 中的分发包：
  - `macterm` — 独立的 arm64 二进制文件
  - `install.sh` — 一键安装脚本（`install -m 755` 到 `/usr/local/bin`）
  - `macterm-aarch64-macos.tar.gz` — 1.4MB 压缩分发包

### 0.2.1 — 搜索覆盖层

- **搜索覆盖层 (E1)**：`Alt+S` 在活动面板中打开不区分大小写的搜索
  - 输入时实时匹配，显示匹配计数（`3/42`）
  - Enter/Tab 向前循环，Shift+Tab 向后
  - Esc 关闭
- 帮助覆盖层更新了新的快捷键参考

### 0.2.0 — UI 美化与动画

#### 动画（第二阶段）
- **波浪渐变头部 (A1)**："MACTERMINAL" 品牌标识逐字符流动色彩波浪
- **发光边框 (B2)**：活动面板边框的正弦脉冲效果（青色亮度振荡）
- **焦点呼吸 (C2)**：活动面板内容单元格的 ±8 亮度调制

#### UI 增强（第一阶段）
- **圆角边框**：面板边框使用 `╭╮╰╯` 圆角字形（`symbols::border::ROUNDED`）
- **双线分隔符**：`║` 垂直和 `═` 水平分割线，带 `╬` 交叉点检测
- **面板编号覆盖层**：每个面板边框标题中显示序号 `[N]` 标签
- **标签指示器**：`●`/`○` 圆点、`▏` 分隔符、`▔` 活动标签下划线
- **面板标题栏 (B5)**：每个面板内部显示带 `[N]` 标签的彩色 1 行标题

#### 确认对话框 (E4/E5)
- **关闭面板确认**：在多个面板时按 `Ctrl+W` 显示确认对话框
- **退出确认**：`Ctrl+Q` 现在显示确认后才退出
- 带有 `[Y]es  [N]o  [Esc]` 按钮的样式化覆盖层
- Enter/Y 确认，Esc/N/Q 取消

#### 命令退出通知 (E3)
- 彩色退出代码显示：退出码 0 显示 `✓ 绿色`，非零显示 `✗ 红色`
- 约 2 秒后自动淡出
- `StatusBar.message_color` 基础设施支持按消息着色

#### Tokyo Night 色彩校准 (F2)
- 通过 OSC 10/11 序列设置 vt100 解析器的默认前景色 `#a9b1d6` 和背景色 `#1a1b26`
- 完整 ANSI 调色板（颜色 0–15）校准为 Tokyo Night 值
- `render_screen()` 将 `Color::Default` 解析为 Tokyo Night 颜色而非 `Color::Reset`
- 所有 UI 元素和终端内容统一呈现深色主题

#### 标签滚动 (A4)
- `tab_scroll_offset` 状态跟踪当前滚动位置
- 切换标签时自动滚动（`Alt+←→`、`Alt+1-9`、`Ctrl+T`）
- 标签超出屏幕时显示 `◀▶` 箭头指示器
- 与标签栏比例宽度布局协同工作

#### 文件树改进 (D1)
- 通过 `std::fs::read_dir` 实时读取当前工作目录列表
- 目录优先排序，然后按字母顺序
- 目录显示 `📁` 图标，文件显示空白间距
- 支持长列表滚动

### 0.1.0（初始版本）

#### 核心
- Rust 工作空间包含 3 个 crate：`macterm`（二进制）、`macterm-core`（数据模型）、`macterm-tui`（终端 UI）
- 分叉/二叉树数据模型 —— 包含 `Leaf` 和 `Split` 变体的 `SplitNode`
- 使用 `portable-pty` + `vt100` 解析器 + tokio mpsc 事件通道的 PTY 封装
- 基于 `tokio::select!` 的异步事件循环 —— 多路复用键盘、PTY 事件和帧时钟

#### UI 与渲染
- 基于 Ratatui 的终端 UI，60fps 渲染
- 每个面板的块边框，活动面板高亮（青色边框）
- vt100 屏幕渲染，支持颜色、粗体、斜体、下划线
- 从 vt100 的 `Screen::cursor_position()` 获取光标位置
- 渐变 "MACTERMINAL" 双行品牌头部（逐字符青→紫渐变）
- 带有标签/面板计数和消息的状态栏
- 带有样式化章节标题和键/描述/备注列的帮助覆盖层

#### 分屏面板
- 水平（`Ctrl+D`）和垂直（`Ctrl+E`）面板分屏，比例为 50/50
- 面板关闭（`Ctrl+W`）并自动重新平衡树结构
- `pane_rects_from_tree()` —— 从分叉树计算每个面板精确矩形的递归算法
- 通过 `Ctrl+↑↓←→` 进行焦点导航

#### 拖拽调整大小（v0.1.0 功能）
- 点击并拖拽分割边框，实时调整面板比例
- `find_border_at_position()` —— 具有 1 单元格容差的递归树遍历
- 正在拖拽的边框显示青色高亮
- 基于增量的比例更新，限制在 0.1–0.9 之间

#### 标签页
- 使用 `Ctrl+T` 支持多标签页
- 通过 `Alt+←→` 和 `Alt+1-9` 切换标签页
- 切换标签页时：PTY 会话调整到其分叉树尺寸

#### PTY 与终端
- 环境中设置 `TERM=xterm-256color`
- 支持多字节 UTF-8 输入
- Ctrl/Alt 修饰键处理（`Ctrl+C → 0x03`、`Alt+X → ESC+X`）
- 使用 `try_read()` 的 `RwLock<vt100::Parser>` —— 渲染永远不会因解析器争用而阻塞
- **SIGWINCH 传播** —— `PtySession::resize()` 调用 `master.resize()` 通知内核，使 Shell 以正确的终端尺寸重新绘制
- 内容区域计算考虑了文件树侧边栏（修复了文件树打开时输出错位的问题）

#### Bug 修复
- 输入吞没：通过使用 `tokio::select!` 异步事件循环 + `RwLock` 替代 `Mutex` 解决
- 延迟：通过渲染路径中的 `try_read()` + 独立的读取器线程解决
- 重叠填充循环：将 3 个循环合并为 1 个单次渲染
- 标签切换面板尺寸：在切换标签时添加了 `resize_active_panes()` 调用
- 分割比例完整性：比例限制在 `[0.1, 0.9]` 以防止面板折叠
- 窗口调整大小：从分叉树正确调整每个面板的 PTY 大小（而非硬编码的初始尺寸）
- 鼠标点击区域：内容区域 y 坐标从 1 修正为 2（位于 2 行头部下方）

---

## 架构

```
macterm/
├── src/
│   └── main.rs              # CLI 入口点 (clap)
├── crates/
│   ├── macterm-core/        # 数据模型
│   │   ├── src/layout.rs    # SplitNode 二叉树
│   │   ├── src/pane.rs      # PaneId, SplitDirection
│   │   ├── src/workspace.rs # Workspace→Tab→SplitNode 层次结构
│   │   └── src/lib.rs
│   └── macterm-tui/         # 终端 UI 层
│       ├── src/app.rs       # 应用状态、PTY 管理
│       ├── src/ui.rs        # 事件循环、键盘/鼠标处理
│       ├── src/pty.rs       # PTY 会话 (portable-pty + vt100)
│       └── src/widgets/
│           ├── pane_grid.rs # 分叉树渲染、边框拖拽反馈
│           ├── header.rs    # 渐变 MACTERMINAL 头部 + 标签栏
│           └── status_bar.rs
```

---

## 构建

```bash
# 构建
cargo build

# 发布构建
cargo build --release

# 运行
cargo run
```

**依赖要求**：Rust 1.70+、macOS（通过 portable-pty/Ratatui 支持跨平台）

---

## 许可证

MIT
