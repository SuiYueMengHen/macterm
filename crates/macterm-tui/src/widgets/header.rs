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
}

impl HeaderBar<'_> {
    pub fn new<'a>(workspace: &'a Workspace, version: &'a str) -> HeaderBar<'a> {
        HeaderBar { workspace, version }
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

        // ── Line 1: Brand gradient ──
        let gradient_colors = [
            Color::Rgb(0, 180, 255),   // cyan
            Color::Rgb(0, 170, 245),
            Color::Rgb(30, 150, 250),
            Color::Rgb(60, 130, 255),
            Color::Rgb(80, 115, 255),
            Color::Rgb(95, 100, 255),
            Color::Rgb(110, 85, 255),
            Color::Rgb(120, 75, 250),
            Color::Rgb(130, 65, 245),
            Color::Rgb(140, 55, 240),
            Color::Rgb(150, 45, 235),
        ];

        let brand_chars: Vec<char> = BRAND.chars().collect();
        let max_colors = gradient_colors.len().min(brand_chars.len());

        let mut spans: Vec<Span> = Vec::with_capacity(brand_chars.len());
        for (i, ch) in brand_chars.iter().enumerate() {
            let color = if i < max_colors {
                gradient_colors[i]
            } else {
                gradient_colors[max_colors - 1]
            };
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

        // Draw a subtle separator below the brand line
        if area.y + 1 < area.bottom() {
            if let Some(cell) = buf.cell_mut((area.x, brand_line_y)) {
                cell.set_fg(gradient_colors[0]);
                cell.set_bg(bg);
                cell.set_char('█');
            }
            if let Some(cell) = buf.cell_mut((area.x + brand_width as u16 - 1, brand_line_y)) {
                cell.set_fg(gradient_colors[max_colors - 1]);
                cell.set_bg(bg);
                cell.set_char('█');
            }
        }

        // ── Line 2: Tab bar ──
        render_tabs(self.workspace, tab_line_y, area, bg, buf);
    }
}

fn render_tabs(workspace: &Workspace, y: u16, area: Rect, bg: Color, buf: &mut Buffer) {
    let tabs = &workspace.tabs;
    let active_idx = workspace.active_tab;
    let tab_count = tabs.len().max(1);

    let tab_width = (area.width as usize / tab_count).max(12).min(30);
    let start_x = area.x as usize;

    for (i, tab) in tabs.iter().enumerate() {
        let x = start_x + i * tab_width;
        if x + tab_width > area.right() as usize {
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

        // Tab title
        let title = if tab.title.len() > tab_width.saturating_sub(3) {
            format!(" {}…", &tab.title[..tab_width.saturating_sub(4)])
        } else {
            format!(" {} ", tab.title)
        };

        let title_style = Style::default()
            .bg(tab_bg)
            .fg(tab_fg)
            .add_modifier(if is_active {
                Modifier::BOLD | Modifier::UNDERLINED
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

        // Separator between tabs
        if i < tabs.len() - 1 {
            let sep_x = x + tab_width;
            if sep_x < area.right() as usize {
                if let Some(cell) = buf.cell_mut((sep_x as u16, y)) {
                    cell.set_char('│');
                    cell.set_fg(Color::Rgb(50, 55, 70));
                    cell.set_bg(bg);
                }
            }
        }
    }

    // Bottom separator line for tab area
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
