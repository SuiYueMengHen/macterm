use std::time::Duration;

use anyhow::Result;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, KeyCode,
    KeyEventKind, KeyModifiers, MouseEventKind, EventStream,
};
use futures::StreamExt;
use log::{info, trace};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Terminal;
use tokio::sync::mpsc;

use crate::app::{App, ConfirmAction, ResizeState};
use crate::widgets::header::{header_area, HeaderBar};
use crate::widgets::pane_grid::PaneGrid;
use crate::widgets::status_bar::{status_bar_area, StatusBar};

/// Run the main TUI event loop
pub async fn run(mut app: App) -> Result<()> {
    color_eyre::install().unwrap_or_default();

    // Setup terminal
    let mut stdout = std::io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Async event stream + tick interval
    let mut events = EventStream::new();
    let tick_rate = Duration::from_millis(16); // ~60fps
    let mut interval = tokio::time::interval(tick_rate);
    interval.tick().await; // consume immediate tick

    // Main event loop — render on every tick, handle events inline
    while app.running {
        tokio::select! {
            _ = interval.tick() => {
                app.tick();
            }
            Some(Ok(event)) = events.next() => {
                handle_event(&mut app, &event)?;
            }
            Some(event) = app.pty_rx.recv() => {
                handle_pty_event(&mut app, &event);
            }
        }

        terminal.draw(|frame| {
            render(&mut app, frame);
        })?;
    }

    // Cleanup
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

/// Process a single PTY event
fn handle_pty_event(app: &mut App, event: &crate::pty::PtyEvent) {
    match event {
        crate::pty::PtyEvent::Output(pane_id, _) => {
            trace!("Output received for pane {}", pane_id);
        }
        crate::pty::PtyEvent::Resized(pane_id, cols, rows) => {
            if let Some(session) = app.sessions.get_mut(pane_id) {
                let _ = session.resize(*cols, *rows);
            }
        }
        crate::pty::PtyEvent::Exited(pane_id, code) => {
            info!("Pane {} exited with code {}", pane_id, code);
            let (symbol, color) = if *code == 0 {
                ('✓', Color::Rgb(80, 220, 100))
            } else {
                ('✗', Color::Rgb(240, 80, 80))
            };
            app.set_status_message_colored(
                format!("Pane {} {} ({})", pane_id, symbol, code),
                color,
            );
        }
    }
}

/// Handle input events
fn handle_event(app: &mut App, event: &Event) -> Result<()> {
    match event {
        Event::Key(key) => {
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }

            // Handle confirmation dialog (E4/E5) — intercept all keys when dialog is open
            if app.confirm_action != ConfirmAction::None {
                match key.code {
                    KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                        match app.confirm_action {
                            ConfirmAction::ClosePane => {
                                app.close_active_pane();
                                app.set_status_message("Closed pane".to_string());
                            }
                            ConfirmAction::Quit => {
                                app.running = false;
                            }
                            ConfirmAction::None => {}
                        }
                        app.confirm_action = ConfirmAction::None;
                    }
                    KeyCode::Esc
                    | KeyCode::Char('n')
                    | KeyCode::Char('N')
                    | KeyCode::Char('q') => {
                        app.confirm_action = ConfirmAction::None;
                        app.set_status_message("Cancelled".to_string());
                    }
                    _ => {}
                }
                return Ok(());
            }

            // Handle help overlay — Esc or Ctrl+H to close
            if app.show_help {
                match key.code {
                    KeyCode::Esc => app.show_help = false,
                    KeyCode::Char('h') if key.modifiers == KeyModifiers::CONTROL => {
                        app.show_help = false;
                    }
                    _ => {}
                }
                return Ok(());
            }

            // Handle command palette input
            if app.show_command_palette {
                match key.code {
                    KeyCode::Esc => {
                        app.show_command_palette = false;
                        app.command_input.clear();
                    }
                    KeyCode::Enter => {
                        app.command_palette_execute();
                    }
                    KeyCode::Backspace => {
                        app.command_palette_backspace();
                    }
                    KeyCode::Char(c) => {
                        app.command_palette_input(c);
                    }
                    _ => {}
                }
                return Ok(());
            }

            // Handle search overlay (E1)
            if app.show_search {
                match key.code {
                    KeyCode::Esc => {
                        app.show_search = false;
                        app.search_query.clear();
                        app.search_matches.clear();
                    }
                    KeyCode::Enter => {
                        app.next_match();
                    }
                    KeyCode::Backspace => {
                        app.search_backspace();
                    }
                    KeyCode::Tab => {
                        if key.modifiers == KeyModifiers::SHIFT {
                            app.prev_match();
                        } else {
                            app.next_match();
                        }
                    }
                    KeyCode::Char(c) => {
                        app.search_input(c);
                    }
                    _ => {}
                }
                return Ok(());
            }

            match key.code {
                // Search (find)
                KeyCode::Char('s') if key.modifiers == KeyModifiers::ALT => {
                    app.show_search = !app.show_search;
                    if app.show_search {
                        app.search_query.clear();
                        app.search_matches.clear();
                        app.search_match_index = 0;
                    }
                }

                // Quit (with confirmation)
                KeyCode::Char('q') if key.modifiers == KeyModifiers::CONTROL => {
                    app.confirm_action = ConfirmAction::Quit;
                }

                // Split pane
                KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
                    app.split_active_pane(macterm_core::SplitDirection::Horizontal);
                    app.set_status_message("Split right".to_string());
                }
                KeyCode::Char('e') if key.modifiers == KeyModifiers::CONTROL => {
                    app.split_active_pane(macterm_core::SplitDirection::Vertical);
                    app.set_status_message("Split down".to_string());
                }

                // Close pane (with confirmation)
                KeyCode::Char('w') if key.modifiers == KeyModifiers::CONTROL => {
                    if app.workspace.active_tab().pane_count() > 1 {
                        app.confirm_action = ConfirmAction::ClosePane;
                    } else {
                        app.close_active_pane();
                        app.set_status_message("Closed pane".to_string());
                    }
                }

                // New tab
                KeyCode::Char('t')
                    if key.modifiers == KeyModifiers::ALT
                        || key.modifiers == KeyModifiers::CONTROL =>
                {
                    let tab_num = app.workspace.tab_count() + 1;
                    app.workspace.add_tab(format!("term-{}", tab_num));

                    // Spawn PTY for the new tab's first pane
                    let pane_id = app.workspace.active_tab().active_pane();
                    let (tx, _rx) = mpsc::unbounded_channel();
                    match crate::pty::PtySession::spawn(pane_id, 80, 24, tx) {
                        Ok(session) => {
                            app.sessions.insert(pane_id, session);
                        }
                        Err(e) => {
                            log::error!("Failed to spawn PTY for new tab: {}", e);
                        }
                    }
                    // Immediately resize to actual pane dimensions
                    app.resize_active_panes();
                    app.ensure_tab_visible();

                    app.set_status_message("New tab".to_string());
                }

                // Switch tabs (with auto-scroll to keep active tab visible)
                KeyCode::Char(c @ '1'..='9') if key.modifiers == KeyModifiers::ALT => {
                    let idx = c.to_digit(10).unwrap_or(1) as usize - 1;
                    app.workspace.switch_to_tab(idx);
                    app.ensure_tab_visible();
                    app.resize_active_panes();
                }

                // Next/prev tab (with auto-scroll)
                KeyCode::Right if key.modifiers == KeyModifiers::ALT => {
                    app.workspace.next_tab();
                    app.ensure_tab_visible();
                    app.resize_active_panes();
                }
                KeyCode::Left if key.modifiers == KeyModifiers::ALT => {
                    app.workspace.prev_tab();
                    app.ensure_tab_visible();
                    app.resize_active_panes();
                }

                // Focus navigation: Ctrl+arrows to move between panes
                KeyCode::Right if key.modifiers == KeyModifiers::CONTROL => {
                    app.focus_next_pane();
                }
                KeyCode::Left if key.modifiers == KeyModifiers::CONTROL => {
                    app.focus_prev_pane();
                }
                KeyCode::Down if key.modifiers == KeyModifiers::CONTROL => {
                    app.focus_next_pane();
                }
                KeyCode::Up if key.modifiers == KeyModifiers::CONTROL => {
                    app.focus_prev_pane();
                }

                // Command palette
                KeyCode::Char('p') if key.modifiers == KeyModifiers::CONTROL => {
                    app.show_command_palette = !app.show_command_palette;
                    app.command_input.clear();
                }

                // Toggle file tree
                KeyCode::Char('f') if key.modifiers == KeyModifiers::CONTROL => {
                    app.show_file_tree = !app.show_file_tree;
                    if app.show_file_tree {
                        app.refresh_file_tree();
                    }
                }

                // Help overlay
                KeyCode::Char('h') if key.modifiers == KeyModifiers::CONTROL => {
                    app.show_help = !app.show_help;
                }

                // Pass through to active pane — handle Ctrl/Alt modifiers correctly
                KeyCode::Char(c) if key.modifiers == KeyModifiers::CONTROL => {
                    let code = (c as u8) & 0x1f;
                    app.write_to_active_pane(&[code]);
                }
                KeyCode::Char(c) if key.modifiers == KeyModifiers::ALT => {
                    let buf = [0x1b, c as u8];
                    app.write_to_active_pane(&buf);
                }
                KeyCode::Char(c) => {
                    // Proper UTF-8 encoding (multi-byte support for Unicode)
                    let mut buf = [0u8; 4];
                    let s = c.encode_utf8(&mut buf);
                    app.write_to_active_pane(s.as_bytes());
                }
                KeyCode::Enter => {
                    app.write_to_active_pane(b"\r");
                }
                KeyCode::Backspace => {
                    app.write_to_active_pane(b"\x7f");
                }
                KeyCode::Tab => {
                    app.write_to_active_pane(b"\t");
                }
                KeyCode::Esc => {
                    app.write_to_active_pane(b"\x1b");
                }
                KeyCode::Up => {
                    app.write_to_active_pane(b"\x1b[A");
                }
                KeyCode::Down => {
                    app.write_to_active_pane(b"\x1b[B");
                }
                KeyCode::Left => {
                    app.write_to_active_pane(b"\x1b[D");
                }
                KeyCode::Right => {
                    app.write_to_active_pane(b"\x1b[C");
                }
                KeyCode::Home => {
                    app.write_to_active_pane(b"\x1b[H");
                }
                KeyCode::End => {
                    app.write_to_active_pane(b"\x1b[F");
                }
                KeyCode::Delete => {
                    app.write_to_active_pane(b"\x1b[3~");
                }
                KeyCode::PageUp => {
                    app.write_to_active_pane(b"\x1b[5~");
                }
                KeyCode::PageDown => {
                    app.write_to_active_pane(b"\x1b[6~");
                }
                _ => {}
            }
        }

        Event::Mouse(mouse) => {
            let click_x = mouse.column;
            let click_y = mouse.row;

            // Compute the content area (below header, above status bar) — must match render()
            let content_area = {
                let status_h = if app.show_status_bar { 1 } else { 0 };
                Rect {
                    x: if app.show_file_tree { 20 } else { 0 },
                    y: header_area(app.area).bottom(),
                    width: app.area.width - if app.show_file_tree { 20 } else { 0 },
                    height: app.area.height.saturating_sub(2 + status_h),
                }
            };

            match mouse.kind {
                MouseEventKind::Down(btn) if btn == crossterm::event::MouseButton::Left => {
                    // First check for border click → start resize
                    let (border_hit, focus_hit) = {
                        let tab = app.workspace.active_tab();
                        let border = find_border_at_position(
                            &tab.root,
                            content_area,
                            click_x,
                            click_y,
                        );
                        let focus = if border.is_some() {
                            None
                        } else {
                            let pane_ids = tab.pane_ids();
                            find_pane_at_position(
                                &tab.root,
                                content_area,
                                click_x,
                                click_y,
                                &pane_ids,
                            )
                        };
                        (border, focus)
                    };

                    if let Some((dir, split_area, ratio, pane_id)) = border_hit {
                        let start_pos = match dir {
                            macterm_core::SplitDirection::Horizontal => click_x,
                            macterm_core::SplitDirection::Vertical => click_y,
                        };
                        app.start_resize_drag(pane_id, dir, split_area, ratio, start_pos);
                    } else if let Some(pane_id) = focus_hit {
                        let tab = app.workspace.active_tab_mut();
                        tab.set_active_pane(pane_id);
                    }
                }
                MouseEventKind::Drag(btn) if btn == crossterm::event::MouseButton::Left => {
                    app.update_resize_drag(click_x, click_y);
                }
                MouseEventKind::Up(btn) if btn == crossterm::event::MouseButton::Left => {
                    app.end_resize_drag();
                }
                _ => {}
            }
        }

        Event::Resize(cols, rows) => {
            app.area = Rect::new(0, 0, *cols, *rows);
            app.resize_active_panes();
        }

        _ => {}
    }

    Ok(())
}

/// Find which pane is at the given terminal position
fn find_pane_at_position(
    node: &macterm_core::SplitNode,
    area: Rect,
    x: u16,
    y: u16,
    pane_ids: &[macterm_core::PaneId],
) -> Option<macterm_core::PaneId> {
    if !area.contains((x, y).into()) {
        return None;
    }

    match node {
        macterm_core::SplitNode::Leaf(pane_id) => Some(*pane_id),
        macterm_core::SplitNode::Split {
            direction,
            ratio,
            left,
            right,
        } => {
            let (left_area, right_area) = match direction {
                macterm_core::SplitDirection::Horizontal => {
                    let left_w = (area.width as f32 * ratio) as u16;
                    (
                        Rect::new(area.x, area.y, left_w, area.height),
                        Rect::new(area.x + left_w, area.y, area.width - left_w, area.height),
                    )
                }
                macterm_core::SplitDirection::Vertical => {
                    let left_h = (area.height as f32 * ratio) as u16;
                    (
                        Rect::new(area.x, area.y, area.width, left_h),
                        Rect::new(
                            area.x,
                            area.y + left_h,
                            area.width,
                            area.height - left_h,
                        ),
                    )
                }
            };

            find_pane_at_position(left, left_area, x, y, pane_ids)
                .or_else(|| find_pane_at_position(right, right_area, x, y, pane_ids))
        }
    }
}

/// Render the entire UI
fn render(app: &mut App, frame: &mut ratatui::Frame) {
    let area = frame.area();
    app.area = area;

    // Background
    let bg_block = Block::default()
        .style(Style::default().bg(Color::Rgb(15, 18, 28)));
    frame.render_widget(bg_block, area);

    // Header bar at top (brand + tabs)
    let head_area = header_area(area);
    frame.render_widget(
        HeaderBar::new(&app.workspace, "0.1.0", app.frame_count, app.tab_scroll_offset),
        head_area,
    );

    // Main content area
    let content_y = head_area.bottom();
    let status_h = if app.show_status_bar { 1 } else { 0 };
    let content_height = area.height.saturating_sub(2 + status_h);

    let content_area = Rect {
        x: if app.show_file_tree { 20 } else { 0 },
        y: content_y,
        width: area.width - if app.show_file_tree { 20 } else { 0 },
        height: content_height,
    };

    // File tree sidebar (D1)
    if app.show_file_tree {
        let sidebar_area = Rect {
            x: 0,
            y: content_y,
            width: 20,
            height: content_height,
        };

        let sidebar_bg = Color::Rgb(18, 22, 33);
        let sidebar_block = Block::default()
            .title(" Files ")
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(Color::Rgb(60, 65, 80)))
            .style(Style::default().bg(sidebar_bg));

        let inner = sidebar_block.inner(sidebar_area);
        frame.render_widget(sidebar_block, sidebar_area);

        // Render file tree entries
        let max_rows = inner.height as usize;
        let entries: Vec<ratatui::text::Span> = app.file_tree_entries
            .iter()
            .skip(app.file_tree_scroll)
            .take(max_rows)
            .map(|(name, is_dir)| {
                let icon = if *is_dir { "📁 " } else { "  " };
                let fg = if *is_dir { Color::Rgb(100, 180, 255) } else { Color::Rgb(180, 185, 200) };
                ratatui::text::Span::styled(
                    format!("{}{}", icon, name),
                    Style::default().fg(fg).bg(sidebar_bg),
                )
            })
            .collect();

        if !entries.is_empty() {
            let lines: Vec<ratatui::text::Line> = entries.into_iter()
                .map(|s| ratatui::text::Line::from(s))
                .collect();
            frame.render_widget(Paragraph::new(ratatui::text::Text::from(lines)), inner);
        }
    }

    // Extract resize_pane from resize state (before borrowing tab)
    let resize_pane = match app.resize_state {
        ResizeState::Dragging { split_pane, .. } => Some(split_pane),
        ResizeState::Idle => None,
    };

    // Pane grid (terminal content)
    let tab = app.workspace.active_tab();

    // Build parsers map from sessions
    let parsers: std::collections::HashMap<
        macterm_core::PaneId,
        std::sync::Arc<std::sync::RwLock<vt100::Parser>>,
    > = app
        .sessions
        .iter()
        .map(|(id, session)| (*id, session.parser.clone()))
        .collect();

    let pane_ids_list = tab.pane_ids();
    let pane_indices: std::collections::HashMap<_, _> = pane_ids_list
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, i + 1))
        .collect();

    let pane_grid = PaneGrid {
        root: &tab.root,
        active_pane: tab.active_pane(),
        parsers: &parsers,
        area: content_area,
        focus_animation: None,
        resize_pane,
        pane_indices: &pane_indices,
        frame_count: app.frame_count,
    };
    frame.render_widget(pane_grid, content_area);

    // Render cursor in the active pane
    {
        let active_pane_id = tab.active_pane();
        if let Some(parser) = app.sessions.get(&active_pane_id).map(|s| s.parser.clone()) {
            if let Ok(guard) = parser.try_read() {
                let (cursor_row, cursor_col) = guard.screen().cursor_position();
                let pane_rects = pane_rects_from_tree(&tab.root, content_area);
                if let Some(pane_area) = pane_rects.get(&active_pane_id) {
                    // render_pane: area → border(+1) → title_bar(+1) → content
                    let inner_x = pane_area.x.saturating_add(1);      // border left
                    let inner_y = pane_area.y.saturating_add(2);      // border top + title bar
                    let cursor_x = inner_x.saturating_add(cursor_col).min(area.right().saturating_sub(1));
                    let cursor_y = inner_y.saturating_add(cursor_row).min(area.bottom().saturating_sub(1));
                    frame.set_cursor_position((cursor_x, cursor_y));
                }
            }
        }
    }

    // Status bar at bottom
    if app.show_status_bar {
        let status_area = status_bar_area(area);
        let tab = app.workspace.active_tab();
        let msg = app.status_message.as_deref();
        frame.render_widget(
            StatusBar {
                tab_count: app.workspace.tab_count(),
                pane_count: tab.pane_count(),
                active_tab: app.workspace.active_tab,
                message: msg,
                message_color: app.status_message_color,
                show_file_tree: app.show_file_tree,
                version: "0.1.0",
            },
            status_area,
        );
    }

    // Command palette overlay
    if app.show_command_palette {
        let palette_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: 3,
        };

        frame.render_widget(Clear, palette_area);

        let palette_block = Block::default()
            .title(" Command Palette ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(70, 100, 140)))
            .style(Style::default().bg(Color::Rgb(22, 26, 38)));

        let input = if app.command_input.is_empty() {
            " Type a command... "
        } else {
            &app.command_input
        };

        let palette_inner = palette_block.inner(palette_area);
        frame.render_widget(palette_block, palette_area);
        frame.render_widget(Paragraph::new(input), palette_inner);
    }

    // Search overlay (E1)
    if app.show_search {
        let search_area = Rect {
            x: area.width / 4,
            y: area.height.saturating_sub(3),
            width: area.width / 2,
            height: 3,
        };

        frame.render_widget(Clear, search_area);

        let match_info = if app.search_matches.is_empty() {
            if app.search_query.is_empty() {
                String::new()
            } else {
                " 0/0 ".to_string()
            }
        } else {
            format!(" {}/{} ", app.search_match_index + 1, app.search_matches.len())
        };

        let search_title = format!(" Find{}", match_info);
        let search_block = Block::default()
            .title(search_title.as_str())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(70, 100, 140)))
            .style(Style::default().bg(Color::Rgb(20, 24, 36)));

        let display = if app.search_query.is_empty() {
            " Type to search... "
        } else {
            &app.search_query
        };

        let inner = search_block.inner(search_area);
        frame.render_widget(search_block, search_area);
        frame.render_widget(Paragraph::new(display), inner);
    }

    // Help overlay
    if app.show_help {
        let hdr = Style::default().fg(Color::Rgb(100, 140, 180));
        let key = Style::default().fg(Color::Rgb(150, 160, 180));
        let desc = Style::default().fg(Color::Rgb(130, 140, 160));
        let note = Style::default().fg(Color::Rgb(90, 100, 120));

        let mut rows: Vec<Line> = Vec::new();
        macro_rules! sec { ($n:expr) => { rows.push(Line::from(Span::styled($n.to_string(), hdr))); }}
        macro_rules! row { ($k:expr, $d:expr, $n:expr) => {{
            let mut s: Vec<Span> = vec![Span::styled($k.to_string(), key)];
            if !$d.is_empty() { s.push(Span::styled($d.to_string(), desc)); }
            if !$n.is_empty() { s.push(Span::styled(format!(" {}", $n), note)); }
            rows.push(Line::from(s));
        }}}

        sec!(" Panes ");
        row!(" Ctrl+D      ", "Split right   ", "(horizontal)");
        row!(" Ctrl+E      ", "Split down    ", "(vertical)");
        row!(" Ctrl+W      ", "Close pane    ", "");
        row!(" Ctrl+↑↓←→  ", "Focus pane    ", "next/prev");
        rows.push(Line::from(""));
        sec!(" Tabs ");
        row!(" Ctrl+T      ", "New tab       ", "");
        row!(" Alt+← →     ", "Switch tab    ", "prev/next");
        row!(" Alt+1-9     ", "Switch tab    ", "by number");
        rows.push(Line::from(""));
        sec!(" Interface ");
        row!(" Ctrl+P      ", "Command palette", "");
        row!(" Ctrl+F      ", "File tree     ", "toggle");
        row!(" Alt+S       ", "Search        ", "find in pane");
        row!(" Ctrl+H      ", "Help          ", "this screen");
        row!(" Ctrl+Q      ", "Quit          ", "");
        rows.push(Line::from(""));
        sec!(" Shell input ");
        row!(" Enter/Tab/Esc", "Sent to shell  ", "");
        row!(" ↑ ↓ ← →     ", "Cursor keys   ", "");
        row!(" Ctrl+letter ", "Control codes ", "Ctrl+C=SIGINT");
        row!(" Alt+letter  ", "Alt codes     ", "ESC+letter");

        let help_h = rows.len() as u16 + 2;
        let help_w = 44u16;
        let ha = Rect {
            x: (area.width.saturating_sub(help_w)) / 2,
            y: (area.height.saturating_sub(help_h)) / 2,
            width: help_w.min(area.width),
            height: help_h.min(area.height),
        };

        frame.render_widget(Clear, ha);
        let block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(70, 100, 140)))
            .style(Style::default().bg(Color::Rgb(18, 22, 33)));
        let inner = block.inner(ha);
        frame.render_widget(block, ha);
        frame.render_widget(Paragraph::new(ratatui::text::Text::from(rows)), inner);
    }

    // Confirm dialog overlay (E4/E5)
    if app.confirm_action != ConfirmAction::None {
        let (title, message) = match app.confirm_action {
            ConfirmAction::ClosePane => (" Close Pane ", " Close this pane?"),
            ConfirmAction::Quit => (" Quit macterm ", " Quit macterm?"),
            ConfirmAction::None => unreachable!(),
        };

        let dlg_w = 36u16;
        let dlg_h = 5u16;
        let da = Rect {
            x: (area.width.saturating_sub(dlg_w)) / 2,
            y: (area.height.saturating_sub(dlg_h)) / 2,
            width: dlg_w.min(area.width),
            height: dlg_h.min(area.height),
        };

        frame.render_widget(Clear, da);
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(100, 120, 140)))
            .style(Style::default().bg(Color::Rgb(22, 25, 32)));
        let inner = block.inner(da);
        frame.render_widget(block, da);

        let text = vec![
            Line::from(Span::styled(
                message,
                Style::default().fg(Color::Rgb(150, 160, 170)),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(" [Y]es  ", Style::default().fg(Color::Rgb(90, 140, 90))),
                Span::styled("[N]o  ", Style::default().fg(Color::Rgb(140, 150, 160))),
                Span::styled("[Esc] ", Style::default().fg(Color::Rgb(140, 90, 90))),
            ]),
        ];
        frame.render_widget(Paragraph::new(ratatui::text::Text::from(text)).alignment(Alignment::Center), inner);
    }
}

// Helper for pane rects

/// Find which split border (if any) is at the given terminal position.
/// Returns `(direction, split_area, ratio, child_pane_id)` where `child_pane_id`
/// identifies the split node for updating the ratio.
fn find_border_at_position(
    node: &macterm_core::SplitNode,
    area: Rect,
    x: u16,
    y: u16,
) -> Option<(macterm_core::SplitDirection, Rect, f32, macterm_core::PaneId)> {
    match node {
        macterm_core::SplitNode::Leaf(_) => None,
        macterm_core::SplitNode::Split {
            direction,
            ratio,
            left,
            right,
        } => {
            let (left_area, right_area) = match direction {
                macterm_core::SplitDirection::Horizontal => {
                    let left_w = (area.width as f32 * ratio).round() as u16;
                    (
                        Rect::new(area.x, area.y, left_w.min(area.width), area.height),
                        Rect::new(
                            area.x.saturating_add(left_w),
                            area.y,
                            area.width.saturating_sub(left_w),
                            area.height,
                        ),
                    )
                }
                macterm_core::SplitDirection::Vertical => {
                    let left_h = (area.height as f32 * ratio).round() as u16;
                    (
                        Rect::new(area.x, area.y, area.width, left_h.min(area.height)),
                        Rect::new(
                            area.x,
                            area.y.saturating_add(left_h),
                            area.width,
                            area.height.saturating_sub(left_h),
                        ),
                    )
                }
            };

            // Check if click is on this split's border (tolerance of 1 cell)
            let on_border = match direction {
                macterm_core::SplitDirection::Horizontal => {
                    let border_col = left_area.right();
                    (x == border_col || (x > 0 && x.saturating_sub(1) == border_col))
                        && y >= area.y
                        && y < area.y.saturating_add(area.height)
                }
                macterm_core::SplitDirection::Vertical => {
                    let border_row = left_area.bottom();
                    (y == border_row || (y > 0 && y.saturating_sub(1) == border_row))
                        && x >= area.x
                        && x < area.x.saturating_add(area.width)
                }
            };

            if on_border {
                // Return a pane ID from one child to identify this split
                let child_id = left
                    .pane_ids()
                    .first()
                    .copied()
                    .or_else(|| right.pane_ids().first().copied())?;
                Some((*direction, area, *ratio, child_id))
            } else if left_area.contains((x, y).into()) {
                find_border_at_position(left, left_area, x, y)
            } else if right_area.contains((x, y).into()) {
                find_border_at_position(right, right_area, x, y)
            } else {
                None
            }
        }
    }
}

/// Compute the actual pixel Rect for each pane in a split tree
pub fn pane_rects_from_tree(node: &macterm_core::SplitNode, area: Rect) -> std::collections::HashMap<macterm_core::PaneId, Rect> {
    let mut rects = std::collections::HashMap::new();
    collect_rects(node, area, &mut rects);
    rects
}

fn collect_rects(node: &macterm_core::SplitNode, area: Rect, rects: &mut std::collections::HashMap<macterm_core::PaneId, Rect>) {
    match node {
        macterm_core::SplitNode::Leaf(pane_id) => {
            rects.insert(*pane_id, area);
        }
        macterm_core::SplitNode::Split { direction, ratio, left, right } => {
            let (left_area, right_area) = match direction {
                macterm_core::SplitDirection::Horizontal => {
                    let left_w = (area.width as f32 * ratio).round() as u16;
                    (
                        Rect::new(area.x, area.y, left_w.min(area.width), area.height),
                        Rect::new(area.x.saturating_add(left_w), area.y, area.width.saturating_sub(left_w), area.height),
                    )
                }
                macterm_core::SplitDirection::Vertical => {
                    let left_h = (area.height as f32 * ratio).round() as u16;
                    (
                        Rect::new(area.x, area.y, area.width, left_h.min(area.height)),
                        Rect::new(area.x, area.y.saturating_add(left_h), area.width, area.height.saturating_sub(left_h)),
                    )
                }
            };
            collect_rects(left, left_area, rects);
            collect_rects(right, right_area, rects);
        }
    }
}
