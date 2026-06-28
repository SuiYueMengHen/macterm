use macterm_core::Workspace;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::app::SysStats;

const BRAND: &str = " MACTERMINAL ";

pub struct HeaderBar<'a> {
    pub workspace: &'a Workspace,
    pub version: &'a str,
    pub frame_count: u64,
    pub tab_scroll_offset: usize,
    pub stats: &'a SysStats,
}

impl HeaderBar<'_> {
    pub fn new<'a>(
        workspace: &'a Workspace,
        version: &'a str,
        frame_count: u64,
        tab_scroll_offset: usize,
        stats: &'a SysStats,
    ) -> HeaderBar<'a> {
        HeaderBar { workspace, version, frame_count, tab_scroll_offset, stats }
    }
}

impl Widget for HeaderBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let version_str = format!(" v{} ", self.version);
        let filler_len = area.width.saturating_sub(BRAND.len() as u16 + version_str.len() as u16);
        let brand_line = Line::default()
            .spans(vec![
                Span::styled(BRAND.to_string(), Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(" ".repeat(filler_len as usize), Style::default()),
                Span::styled(version_str, Style::default()),
            ]);
        brand_line.render(Rect::new(area.x, area.y, area.width, 1), buf);

        // Stats line
        let s = &self.stats;
        let mem_str = format!("{:.1}/{:.0}G", s.mem_used_gb, s.mem_total_gb);
        let stats_text = format!(
            " CPU {}%  MEM {}  {} ",
            s.cpu_pct as u8, mem_str, s.cpu_brand,
        );
        let stats_filler = area.width.saturating_sub(stats_text.len() as u16);
        let stats_line = Line::default().spans(vec![
            Span::styled(format!(" {} ", stats_text.trim()), Style::default()),
            Span::styled(" ".repeat(stats_filler as usize), Style::default()),
        ]);
        stats_line.render(Rect::new(area.x, area.y + 1, area.width, 1), buf);

        render_tabs(self.workspace, area.y + 2, area, buf, self.tab_scroll_offset);
    }
}

fn render_tabs(workspace: &Workspace, y: u16, area: Rect, buf: &mut Buffer, scroll_offset: usize) {
    let tabs = &workspace.tabs;
    let active_idx = workspace.active_tab;
    let tab_count = tabs.len().max(1);

    let tab_width = (area.width as usize / tab_count).max(14).min(32);
    let start_x = area.x as usize;

    let max_visible = (area.width as usize) / tab_width;
    let has_left = scroll_offset > 0;
    let has_right = scroll_offset + max_visible < tabs.len();

    let left_reserve: usize = if has_left { 2 } else { 0 };
    let right_reserve: usize = if has_right { 2 } else { 0 };

    for (i, tab) in tabs.iter().enumerate().skip(scroll_offset) {
        let visual_idx = i - scroll_offset;
        let x = start_x + visual_idx * tab_width + left_reserve;
        if x + tab_width > area.right() as usize - right_reserve {
            break;
        }

        let is_active = i == active_idx;

        let raw_title = &tab.title;
        let max_title_len = tab_width.saturating_sub(3) as usize;
        let short_title = if raw_title.len() > max_title_len {
            format!("{}…", &raw_title[..max_title_len.saturating_sub(1)])
        } else {
            raw_title.clone()
        };
        let title = format!(" {} ", short_title);

        let title_style = if is_active {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        for (ci, ch) in title.chars().enumerate() {
            let cx = x + ci;
            if cx < area.right() as usize {
                if let Some(cell) = buf.cell_mut((cx as u16, y)) {
                    cell.set_char(ch);
                    cell.set_style(title_style);
                }
            }
        }

        if i < tabs.len() - 1 {
            let sep_x = x + tab_width;
            if sep_x < area.right() as usize {
                if let Some(cell) = buf.cell_mut((sep_x as u16, y)) {
                    cell.set_char('│');
                }
            }
        }
    }

    if has_left {
        if let Some(cell) = buf.cell_mut((area.x, y)) { cell.set_char('<'); }
        if let Some(cell) = buf.cell_mut((area.x + 1, y)) { cell.set_char(' '); }
    }
    if has_right && area.right() >= 2 {
        if let Some(cell) = buf.cell_mut((area.right().saturating_sub(2), y)) { cell.set_char(' '); }
        if let Some(cell) = buf.cell_mut((area.right().saturating_sub(1), y)) { cell.set_char('>'); }
    }
}

pub fn header_area(area: Rect) -> Rect {
    Rect { x: area.x, y: area.y, width: area.width, height: 3 }
}
