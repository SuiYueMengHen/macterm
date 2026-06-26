use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use macterm_core::*;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Widget};

use crate::animations::ColorAnimation;

pub struct PaneGrid<'a> {
    pub root: &'a SplitNode,
    pub active_pane: PaneId,
    pub parsers: &'a HashMap<PaneId, Arc<RwLock<vt100::Parser>>>,
    pub area: Rect,
    pub focus_animation: Option<&'a ColorAnimation>,
    /// If set, highlight the split border being drag-resized (pane ID identifies which split)
    pub resize_pane: Option<PaneId>,
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
                            Rect::new(
                                area.x,
                                area.y + left_h,
                                area.width,
                                area.height - left_h,
                            ),
                        )
                    }
                };

                let is_resizing = self
                    .resize_pane
                    .is_some_and(|p| left.contains(&p) || right.contains(&p));
                let sep_fg = if is_resizing {
                    Color::Rgb(0, 220, 255)
                } else {
                    Color::Rgb(50, 55, 70)
                };

                match direction {
                    SplitDirection::Horizontal => {
                        let sep_x = left_area.right();
                        for y in area.y..area.y + area.height {
                            if let Some(cell) = buf.cell_mut((sep_x, y)) {
                                cell.set_char('│');
                                cell.set_fg(sep_fg);
                                cell.set_bg(Color::Rgb(15, 18, 28));
                            }
                        }
                    }
                    SplitDirection::Vertical => {
                        let sep_y = left_area.bottom();
                        for x in area.x..area.x + area.width {
                            if let Some(cell) = buf.cell_mut((x, sep_y)) {
                                cell.set_char('─');
                                cell.set_fg(sep_fg);
                                cell.set_bg(Color::Rgb(15, 18, 28));
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
        if area.width < 3 || area.height < 3 {
            return;
        }

        let is_active = pane_id == self.active_pane;
        let border_color = if is_active {
            Color::Rgb(0, 180, 255)
        } else {
            Color::Rgb(60, 65, 80)
        };

        let id_str = pane_id.to_string();
        let short_id = format!(" {}", &id_str[..8.min(id_str.len())]);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(border_color)
                    .add_modifier(if is_active { Modifier::BOLD } else { Modifier::empty() }),
            )
            .title(Span::styled(
                short_id,
                Style::default()
                    .fg(border_color)
                    .add_modifier(if is_active { Modifier::BOLD } else { Modifier::empty() }),
            ));

        let inner = block.inner(area);
        block.render(area, buf);

        if let Some(parser) = self.parsers.get(&pane_id) {
            // Try to read the screen — never block render on parser contention
            let screen = parser.try_read().ok().map(|p| p.screen().clone());
            if let Some(ref scr) = screen {
                self.render_screen(scr, inner, buf);
            }
        } else {
            for y in inner.y..inner.bottom() {
                for x in inner.x..inner.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_char(' ');
                        cell.set_style(Style::default().bg(Color::Rgb(20, 25, 35)));
                    }
                }
            }
        }
    }

    fn render_screen(&self, screen: &vt100::Screen, area: Rect, buf: &mut Buffer) {
        let (screen_rows, screen_cols) = screen.size();
        let max_rows = screen_rows.min(area.height);
        let max_cols = screen_cols.min(area.width);

        // Single pass: iterate every cell in the area once
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
                            let fg = vt100_color_to_ratatui(vt_cell.fgcolor());
                            let bg = vt100_color_to_ratatui(vt_cell.bgcolor());
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
                            cell.set_style(Style::default().bg(Color::Rgb(20, 25, 35)));
                        }
                    } else {
                        cell.set_char(' ');
                        cell.set_style(Style::default().bg(Color::Rgb(20, 25, 35)));
                    }
                }
            }
        }
    }
}

fn vt100_color_to_ratatui(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(idx) => Color::Indexed(idx),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}
