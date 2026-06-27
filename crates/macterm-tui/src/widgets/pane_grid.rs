use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use macterm_core::*;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Widget};
use ratatui::symbols::border;

use crate::animations::ColorAnimation;

/// ── Colors ──────────────────────────────────────────────
const BG_DARK: Color = Color::Rgb(15, 18, 28);
const BG_PANE: Color = Color::Rgb(20, 25, 35);

const BORDER_INACTIVE: Color = Color::Rgb(50, 55, 70);
const BORDER_RESIZE: Color = Color::Rgb(0, 235, 255);

const SEP_DIM: Color = Color::Rgb(50, 55, 70);
const SEP_BRIGHT: Color = Color::Rgb(0, 220, 255);
const SEP_BG: Color = BG_DARK;

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
    pub focus_animation: Option<&'a ColorAnimation>,
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

                let is_resizing = self
                    .resize_pane
                    .is_some_and(|p| left.contains(&p) || right.contains(&p));
                let sep_fg = if is_resizing { SEP_BRIGHT } else { SEP_DIM };

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

        let is_active = pane_id == self.active_pane;
        let is_resizing = self.resize_pane.is_some_and(|p| p == pane_id);

        // Breathing removed: per-frame sin() on all cells caused flicker.
        let breathe_amount: i16 = 0;

        let border_color = if is_resizing {
            BORDER_RESIZE
        } else if is_active {
            // Quantized glow: 4 discrete steps → no visible flicker
            let phase = (self.frame_count as f32 * 0.01) % 4.0;
            let boost = match phase as u8 {
                0 => 0u8,
                1 => 15u8,
                2 => 0u8,
                3 | _ => 0u8,
            };
            Color::Rgb(
                0,
                (180u16 + boost as u16).min(255) as u8,
                (240u16 + boost as u16).min(255) as u8,
            )
        } else {
            BORDER_INACTIVE
        };

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
            .border_style(
                Style::default()
                    .fg(border_color)
                    .add_modifier(if is_active { Modifier::BOLD } else { Modifier::empty() }),
            )
            .title(Span::styled(
                title.as_str(),
                Style::default()
                    .fg(border_color)
                    .add_modifier(if is_active { Modifier::BOLD } else { Modifier::empty() }),
            ));

        let inner = block.inner(area);
        block.render(area, buf);

        // ── Pane title bar (B5) ──
        let bar_bg = if is_active { Color::Rgb(28, 38, 58) } else { Color::Rgb(22, 27, 38) };
        let bar_fg = if is_active { Color::Rgb(130, 190, 255) } else { Color::Rgb(100, 110, 130) };
        let bar_y = inner.y;
        let bar_h = if inner.height > 3 { 1u16 } else { 0u16 };
        if bar_h > 0 {
            let bar_label = format!("  [{}]  ", pane_num);
            for bx in inner.x..inner.right() {
                if let Some(cell) = buf.cell_mut((bx, bar_y)) {
                    cell.set_bg(bar_bg);
                    cell.set_fg(bar_fg);
                    cell.set_char(' ');
                }
            }
            for (ci, ch) in bar_label.chars().enumerate() {
                let cx = inner.x + ci as u16;
                if cx < inner.right() {
                    if let Some(cell) = buf.cell_mut((cx, bar_y)) {
                        cell.set_char(ch);
                        cell.set_bg(bar_bg);
                        cell.set_fg(bar_fg);
                        if is_active { cell.set_style(Style::default().add_modifier(Modifier::BOLD)); }
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
            let screen = parser.try_read().ok().map(|p| p.screen().clone());
            if let Some(ref scr) = screen {
                self.render_screen(scr, content_inner, buf, breathe_amount);
            }
        } else {
            let bg = if breathe_amount != 0 { breathe_color(BG_PANE, breathe_amount) } else { BG_PANE };
            for y in content_inner.y..content_inner.bottom() {
                for x in content_inner.x..content_inner.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_char(' ');
                        cell.set_style(Style::default().bg(bg));
                    }
                }
            }
        }
    }

    fn render_screen(&self, screen: &vt100::Screen, area: Rect, buf: &mut Buffer, breathe: i16) {
        let (screen_rows, screen_cols) = screen.size();
        let max_rows = screen_rows.min(area.height);
        let max_cols = screen_cols.min(area.width);

        for row in 0..area.height {
            for col in 0..area.width {
                let buf_x = area.x + col;
                let buf_y = area.y + row;
                if buf_x >= area.right() || buf_y >= area.bottom() {
                    continue;
                }
                if let Some(cell) = buf.cell_mut((buf_x, buf_y)) {
                    if row < max_rows && col < max_cols {
                        if let Some(vt_cell) = screen.cell(row, col) {
                            // Tokyo Night default colors for unset/default terminal colors
                            const TOKYO_FG: Color = Color::Rgb(169, 177, 214); // #a9b1d6
                            const TOKYO_BG: Color = Color::Rgb(26, 27, 38);   // #1a1b26
                            let fg = vt100_color_to_ratatui(vt_cell.fgcolor(), TOKYO_FG);
                            let bg = vt100_color_to_ratatui(vt_cell.bgcolor(), TOKYO_BG);
                            let fg = if breathe != 0 { breathe_color(fg, breathe) } else { fg };
                            let bg = if breathe != 0 { breathe_color(bg, breathe) } else { bg };
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
                            let bg = if breathe != 0 { breathe_color(BG_PANE, breathe) } else { BG_PANE };
                            cell.set_char(' ');
                            cell.set_style(Style::default().bg(bg));
                        }
                    } else {
                        cell.set_char(' ');
                        cell.set_style(Style::default().bg(BG_PANE));
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

/// Apply a subtle brightness shift for the focus-breathing effect.
/// Only modulates explicit RGB colors; passes Reset/Indexed through unchanged.
fn breathe_color(color: Color, amount: i16) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            let r = (r as i16 + amount).clamp(0, 255) as u8;
            let g = (g as i16 + amount).clamp(0, 255) as u8;
            let b = (b as i16 + amount).clamp(0, 255) as u8;
            Color::Rgb(r, g, b)
        }
        other => other,
    }
}
