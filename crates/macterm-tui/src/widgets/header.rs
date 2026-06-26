use macterm_core::Workspace;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

const BRAND: &str = " MACTERMINAL ";

/// Top header bar with brand gradient + tab bar
pub struct HeaderBar<'a> {
    pub workspace: &'a Workspace,
    pub version: &'a str,
    pub frame_count: u64,
    pub tab_scroll_offset: usize,
}

impl HeaderBar<'_> {
    pub fn new<'a>(
        workspace: &'a Workspace,
        version: &'a str,
        frame_count: u64,
        tab_scroll_offset: usize,
    ) -> HeaderBar<'a> {
        HeaderBar { workspace, version, frame_count, tab_scroll_offset }
    }
}

impl Widget for HeaderBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let bg = Color::Rgb(12, 16, 24);
        let brand_line_y = area.y;
        let tab_line_y = area.y + 1;

        // Fill background for both lines
        for y in area.y..area.y + area.height.min(2) {
            for x in area.x..area.x + area.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_bg(bg);
                    cell.set_char(' ');
                }
            }
        }

        // ── Line 1: Brand gradient with flowing wave animation ──
        // Each character's color shifts over time, creating a wave-like flow
        // from cyan (left) through purple (center) and back.
        let base_phase = self.frame_count as f32 * 0.025;
        let brand_chars: Vec<char> = BRAND.chars().collect();

        let mut spans: Vec<Span> = Vec::with_capacity(brand_chars.len());
        for (i, ch) in brand_chars.iter().enumerate() {
            let t = ((base_phase + i as f32 * 0.35).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
            // t=0 → cyan (0, 200, 255), t=1 → purple (180, 60, 255)
            let r = (t * 180.0) as u8;
            let g = (200u16).saturating_sub((t * 160.0) as u16).min(255) as u8;
            let b = 255u8;
            let color = Color::Rgb(r, g, b);
            let style = Style::default()
                .fg(color)
                .bg(bg)
                .add_modifier(Modifier::BOLD);
            spans.push(Span::styled(ch.to_string(), style));
        }

        // Version info (right-aligned)
        let version_text = format!(" v{} ", self.version);
        let version_style = Style::default()
            .fg(Color::Rgb(100, 105, 125))
            .bg(bg);
        let version_width = version_text.len();
        let version_span = Span::styled(version_text, version_style);

        // Build line: brand gradient + filler + version
        let brand_width: usize = brand_chars.len();
        let filler_width = area.width.saturating_sub((brand_width + version_width) as u16) as usize;

        let mut brand_line = Line::default();
        for span in spans {
            brand_line.spans.push(span);
        }
        if filler_width > 0 {
            brand_line.spans.push(Span::styled(
                " ".repeat(filler_width),
                Style::default().bg(bg),
            ));
        }
        brand_line.spans.push(version_span);

        // Render brand line
        let brand_area = Rect::new(area.x, brand_line_y, area.width, 1);
        brand_line.render(brand_area, buf);

        // Decorative corner markers on the brand line
        if area.y + 1 < area.bottom() {
            if let Some(cell) = buf.cell_mut((area.x, brand_line_y)) {
                cell.set_fg(Color::Rgb(0, 200, 255));
                cell.set_bg(bg);
                cell.set_char('█');
            }
            if let Some(cell) = buf.cell_mut((area.x + brand_width as u16 - 1, brand_line_y)) {
                cell.set_fg(Color::Rgb(180, 60, 255));
                cell.set_bg(bg);
                cell.set_char('█');
            }
        }

        // ── Line 2: Tab bar ──
        render_tabs(self.workspace, tab_line_y, area, bg, buf, self.tab_scroll_offset);
    }
}

fn render_tabs(workspace: &Workspace, y: u16, area: Rect, bg: Color, buf: &mut Buffer, scroll_offset: usize) {
    let tabs = &workspace.tabs;
    let active_idx = workspace.active_tab;
    let tab_count = tabs.len().max(1);

    let tab_width = (area.width as usize / tab_count).max(14).min(32);
    let start_x = area.x as usize;

    // Draw scroll indicators if tabs are scrolled
    let max_visible = (area.width as usize) / tab_width;
    let has_left = scroll_offset > 0;
    let has_right = scroll_offset + max_visible < tabs.len();

    // Reserve space for scroll arrows (2 chars each side)
    let left_reserve: usize = if has_left { 2 } else { 0 };
    let right_reserve: usize = if has_right { 2 } else { 0 };

    for (i, tab) in tabs.iter().enumerate().skip(scroll_offset) {
        let visual_idx = i - scroll_offset;
        let x = start_x + visual_idx * tab_width + left_reserve;
        if x + tab_width > area.right() as usize - right_reserve {
            break;
        }

        let is_active = i == active_idx;

        let tab_bg = if is_active {
            Color::Rgb(25, 35, 55)
        } else {
            bg
        };
        let tab_fg = if is_active {
            Color::Rgb(0, 200, 255)
        } else {
            Color::Rgb(130, 135, 155)
        };

        // Fill tab background
        for cx in x..(x + tab_width).min(area.right() as usize) {
            if let Some(cell) = buf.cell_mut((cx as u16, y)) {
                cell.set_bg(tab_bg);
                cell.set_fg(tab_fg);
                cell.set_char(' ');
            }
        }

        // Tab indicator bullet + title
        let indicator = if is_active { "● " } else { "○ " };
        let raw_title = &tab.title;
        let max_title_len = tab_width.saturating_sub(4) as usize;
        let short_title = if raw_title.len() > max_title_len {
            format!("{}…", &raw_title[..max_title_len.saturating_sub(1)])
        } else {
            raw_title.clone()
        };
        let title = format!("{}{}", indicator, short_title);

        let title_style = Style::default()
            .bg(tab_bg)
            .fg(tab_fg)
            .add_modifier(if is_active {
                Modifier::BOLD
            } else {
                Modifier::empty()
            });

        for (ci, ch) in title.chars().enumerate() {
            let cx = x + ci;
            if cx < area.right() as usize {
                if let Some(cell) = buf.cell_mut((cx as u16, y)) {
                    cell.set_char(ch);
                    cell.set_style(title_style);
                }
            }
        }

        // Active tab — draw a small underline triangle at the bottom edge
        if is_active {
            let underline_x = x + title.len() / 2;
            if underline_x < area.right() as usize {
                if let Some(cell) = buf.cell_mut((underline_x as u16, y)) {
                    cell.set_char('▔');
                    cell.set_fg(Color::Rgb(0, 200, 255));
                    cell.set_bg(bg);
                }
            }
        }

        // Separator between tabs: thin vertical bar
        if i < tabs.len() - 1 {
            let sep_x = x + tab_width;
            if sep_x < area.right() as usize {
                if let Some(cell) = buf.cell_mut((sep_x as u16, y)) {
                    cell.set_char('▏');
                    cell.set_fg(Color::Rgb(45, 50, 65));
                    cell.set_bg(bg);
                }
            }
        }
    }

    // Draw scroll indicator arrows
    let arrow_style = Style::default().fg(Color::Rgb(100, 180, 255)).bg(bg);
    if has_left {
        if let Some(cell) = buf.cell_mut((area.x, y)) {
            cell.set_char('◀');
            cell.set_style(arrow_style);
        }
        if let Some(cell) = buf.cell_mut((area.x + 1, y)) {
            cell.set_char(' ');
            cell.set_style(arrow_style);
        }
    }
    if has_right {
        let right_x = area.right().saturating_sub(2);
        if let Some(cell) = buf.cell_mut((right_x, y)) {
            cell.set_char(' ');
            cell.set_style(arrow_style);
        }
        if let Some(cell) = buf.cell_mut((right_x + 1, y)) {
            cell.set_char('▶');
            cell.set_style(arrow_style);
        }
    }

    // Bottom separator line for tab area (thin dim line)
    let sep_y = y + 1;
    if sep_y < area.bottom() {
        for x in area.x..area.right() {
            if let Some(cell) = buf.cell_mut((x, sep_y - 1)) {
                cell.set_fg(Color::Rgb(30, 35, 50));
                cell.set_bg(Color::Rgb(12, 16, 24));
                cell.set_char('─');
            }
        }
    }
}

/// Calculate the area for the full header (brand + tabs)
pub fn header_area(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 2,
    }
}
