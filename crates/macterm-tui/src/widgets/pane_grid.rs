use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use macterm_core::*;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Widget};
use ratatui::symbols::border;

const SEP_BG: Color = Color::Reset;

/// ── Separator characters ────────────────────────────────
const SEP_V: char = '║';
const SEP_H: char = '═';
const CROSS: char = '╬';

/// ── PaneGrid widget ─────────────────────────────────────
pub struct PaneGrid<'a> {
    pub root: &'a SplitNode,
    pub active_pane: PaneId,
    pub parsers: &'a HashMap<PaneId, Arc<RwLock<vt100::Parser>>>,
    pub area: Rect,
    /// If set, highlight the split border being drag-resized (pane ID identifies which split)
    pub resize_pane: Option<PaneId>,
    /// Sequential index for each pane (for number overlay)
    pub pane_indices: &'a HashMap<PaneId, usize>,
    /// Current frame count for animations (subtle glow pulsing)
    pub frame_count: u64,
}

impl Widget for PaneGrid<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        self.render_node(self.root, area, buf);
    }
}

impl PaneGrid<'_> {
    fn render_node(&self, node: &SplitNode, area: Rect, buf: &mut Buffer) {
        match node {
            SplitNode::Leaf(pane_id) => {
                self.render_pane(*pane_id, area, buf);
            }
            SplitNode::Split {
                direction,
                ratio,
                left,
                right,
            } => {
                let (left_area, right_area) = match direction {
                    SplitDirection::Horizontal => {
                        let left_w = (area.width as f32 * ratio) as u16;
                        (
                            Rect::new(area.x, area.y, left_w, area.height),
                            Rect::new(area.x + left_w, area.y, area.width - left_w, area.height),
                        )
                    }
                    SplitDirection::Vertical => {
                        let left_h = (area.height as f32 * ratio) as u16;
                        (
                            Rect::new(area.x, area.y, area.width, left_h),
                            Rect::new(area.x, area.y + left_h, area.width, area.height - left_h),
                        )
                    }
                };

                let _is_resizing = self
                    .resize_pane
                    .is_some_and(|p| left.contains(&p) || right.contains(&p));
                let sep_fg = Color::Reset;

                match direction {
                    SplitDirection::Horizontal => {
                        let sep_x = left_area.right();
                        for y in area.y..area.y + area.height {
                            if let Some(cell) = buf.cell_mut((sep_x, y)) {
                                let existing = cell.symbol().chars().next().unwrap_or(' ');
                                let ch = crossing_char(existing, SEP_V, SEP_H, CROSS);
                                cell.set_char(ch);
                                cell.set_fg(sep_fg);
                                cell.set_bg(SEP_BG);
                            }
                        }
                    }
                    SplitDirection::Vertical => {
                        let sep_y = left_area.bottom();
                        for x in area.x..area.x + area.width {
                            if let Some(cell) = buf.cell_mut((x, sep_y)) {
                                let existing = cell.symbol().chars().next().unwrap_or(' ');
                                let ch = crossing_char(existing, SEP_H, SEP_V, CROSS);
                                cell.set_char(ch);
                                cell.set_fg(sep_fg);
                                cell.set_bg(SEP_BG);
                            }
                        }
                    }
                }

                self.render_node(left, left_area, buf);
                self.render_node(right, right_area, buf);
            }
        }
    }

    fn render_pane(&self, pane_id: PaneId, area: Rect, buf: &mut Buffer) {
        if area.width < 4 || area.height < 3 {
            return;
        }

        let border_color = Color::Reset;

        // Build title with pane number
        let pane_num = self.pane_indices.get(&pane_id).copied().unwrap_or(0);
        let id_str = pane_id.to_string();
        let short_id = &id_str[..8.min(id_str.len())];
        let title = if pane_num > 0 {
            format!(" {} [{}] ", short_id, pane_num)
        } else {
            format!(" {} ", short_id)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                title.as_str(),
                Style::default().fg(border_color),
            ));

        let inner = block.inner(area);
        block.render(area, buf);

        let bar_y = inner.y;
        let bar_h = if inner.height > 3 { 1u16 } else { 0u16 };
        if bar_h > 0 {
            let bar_label = format!("  [{}]  ", pane_num);
            for bx in inner.x..inner.right() {
                if let Some(cell) = buf.cell_mut((bx, bar_y)) {
                    cell.set_char(' ');
                }
            }
            for (ci, ch) in bar_label.chars().enumerate() {
                let cx = inner.x + ci as u16;
                if cx < inner.right() {
                    if let Some(cell) = buf.cell_mut((cx, bar_y)) {
                        cell.set_char(ch);
                    }
                }
            }
        }

        // Adjust content area to skip the title bar
        let content_inner = Rect {
            x: inner.x,
            y: inner.y + bar_h,
            width: inner.width,
            height: inner.height.saturating_sub(bar_h),
        };

        if let Some(parser) = self.parsers.get(&pane_id) {
            if let Ok(guard) = parser.try_read() {
                self.render_screen(guard.screen(), content_inner, buf);
            }
        } else {
            for y in content_inner.y..content_inner.bottom() {
                for x in content_inner.x..content_inner.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_char(' ');
                        cell.set_style(Style::default());
                    }
                }
            }
        }
    }

    fn render_screen(&self, screen: &vt100::Screen, area: Rect, buf: &mut Buffer) {
        let (screen_rows, screen_cols) = screen.size();
        let max_rows = screen_rows.min(area.height);
        let max_cols = screen_cols.min(area.width);

        // Fill area beyond screen content with blank cells
        if area.height > max_rows {
            for y in (area.y + max_rows)..area.bottom() {
                for x in area.x..area.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_char(' ');
                        cell.set_style(Style::default());
                    }
                }
            }
        }
        if area.width > max_cols {
            for y in area.y..(area.y + max_rows) {
                for x in (area.x + max_cols)..area.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_char(' ');
                        cell.set_style(Style::default());
                    }
                }
            }
        }

        // Render screen cells — only iterate over screen dimensions
        for row in 0..max_rows {
            for col in 0..max_cols {
                let buf_x = area.x + col;
                let buf_y = area.y + row;
                if let Some(cell) = buf.cell_mut((buf_x, buf_y)) {
                    if let Some(vt_cell) = screen.cell(row, col) {
                        let fg = vt100_color_to_ratatui(vt_cell.fgcolor(), Color::Reset);
                        let bg = vt100_color_to_ratatui(vt_cell.bgcolor(), Color::Reset);
                        let mut style = Style::default().fg(fg).bg(bg);
                        if vt_cell.bold() {
                            style = style.add_modifier(Modifier::BOLD);
                        }
                        if vt_cell.italic() {
                            style = style.add_modifier(Modifier::ITALIC);
                        }
                        if vt_cell.underline() {
                            style = style.add_modifier(Modifier::UNDERLINED);
                        }
                        cell.set_style(style);
                        let ch = vt_cell.contents().chars().next().unwrap_or(' ');
                        cell.set_char(ch);
                    } else {
                        cell.set_char(' ');
                        cell.set_style(Style::default());
                    }
                }
            }
        }
    }
}

/// Pick the correct crossing character when two separator lines meet.
/// `my_char` is the primary char for this direction, `perp_char` is the char from the
/// perpendicular direction (already drawn), `cross` is used when both meet.
fn crossing_char(existing: char, my_char: char, perp_char: char, cross: char) -> char {
    if existing == perp_char || existing == '│' || existing == '─'
        || existing == '║' || existing == '═'
    {
        if existing != ' ' {
            return cross;
        }
    }
    my_char
}

fn vt100_color_to_ratatui(color: vt100::Color, default_color: Color) -> Color {
    match color {
        vt100::Color::Default => default_color,
        vt100::Color::Idx(idx) => Color::Indexed(idx),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}


