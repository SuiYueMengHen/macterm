use macterm_core::Workspace;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Paragraph, Widget, Wrap};

/// Render the tab bar at the top of the screen
pub struct TabBar<'a> {
    pub workspace: &'a Workspace,
}

impl<'a> TabBar<'a> {
    pub fn new(workspace: &'a Workspace) -> Self {
        Self { workspace }
    }
}

impl Widget for TabBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let tabs = &self.workspace.tabs;
        let active_idx = self.workspace.active_tab;

        // Calculate tab widths
        let tab_count = tabs.len().max(1);
        let tab_width = (area.width as usize / tab_count).max(15).min(30);
        let total_needed = tab_width * tab_count;

        let start_x = if total_needed < area.width as usize {
            area.left() as usize
        } else {
            area.left() as usize
        };

        for (i, tab) in tabs.iter().enumerate() {
            let x = start_x + i * tab_width;
            if x + tab_width > area.right() as usize {
                break;
            }

            let tab_area = Rect::new(x as u16, area.y, tab_width as u16, area.height);
            let is_active = i == active_idx;

            let bg = if is_active {
                Color::Rgb(30, 40, 60)
            } else {
                Color::Rgb(20, 25, 35)
            };

            let fg = if is_active {
                Color::Rgb(0, 180, 255)
            } else {
                Color::Rgb(140, 140, 160)
            };

            let style = Style::default()
                .bg(bg)
                .fg(fg)
                .add_modifier(if is_active {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                });

            let title = if tab.title.len() > tab_width.saturating_sub(3) as usize {
                format!(" {}…", &tab.title[..tab_width.saturating_sub(4)])
            } else {
                format!(" {} ", tab.title)
            };

            let span = Span::styled(title, style);
            let p = Paragraph::new(span).wrap(Wrap { trim: false });

            // Clear the area first
            for y in tab_area.y..tab_area.y + tab_area.height {
                for x in tab_area.x..tab_area.x + tab_area.width {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_bg(bg);
                        cell.set_fg(fg);
                        cell.set_char(' ');
                    }
                }
            }

            p.render(tab_area, buf);

            // Draw separator
            if i < tabs.len() - 1 {
                let sep_x = x + tab_width;
                if sep_x < area.right() as usize {
                    if let Some(cell) = buf.cell_mut((sep_x as u16, area.y)) {
                        cell.set_char('│');
                        cell.set_fg(Color::Rgb(60, 60, 80));
                        cell.set_bg(Color::Rgb(15, 18, 28));
                    }
                }
            }
        }
    }
}

/// Calculate the area for the tab bar
pub fn tab_bar_area(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    }
}
