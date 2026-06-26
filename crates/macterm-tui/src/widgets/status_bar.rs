use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Paragraph, Widget};

/// Bottom status bar showing keybindings and session info
pub struct StatusBar<'a> {
    pub tab_count: usize,
    pub pane_count: usize,
    pub active_tab: usize,
    pub message: Option<&'a str>,
    pub message_color: Color,
    pub show_file_tree: bool,
    pub version: &'a str,
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // Background fill
        let bg = Color::Rgb(15, 18, 28);
        let fg = Color::Rgb(160, 160, 180);

        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_bg(bg);
                    cell.set_fg(fg);
                    cell.set_char(' ');
                }
            }
        }

        // Left section: session info
        let left_text = format!(
            " Tab {} | {} panes ",
            self.active_tab + 1,
            self.pane_count
        );
        let left_style = Style::default().bg(bg).fg(Color::Rgb(0, 180, 255));
        let left_len = left_text.len();
        let left_span = Span::styled(left_text, left_style);
        let left = Paragraph::new(left_span);
        let left_area = Rect::new(area.x, area.y, left_len as u16, area.height);
        left.render(left_area, buf);

        // Center section: status message (uses message_color from App)
        if let Some(msg) = self.message {
            let msg_span = Span::styled(
                format!(" {} ", msg),
                Style::default()
                    .bg(bg)
                    .fg(self.message_color)
                    .add_modifier(Modifier::ITALIC),
            );
            let msg_area = Rect::new(
                area.x + left_area.width + 1,
                area.y,
                msg.len() as u16,
                area.height,
            );
            Paragraph::new(msg_span).render(msg_area, buf);
        }

        // Right section: keybindings
        let right_parts = vec![
            Span::styled(
                " ^D↓ ",
                Style::default().bg(bg).fg(Color::Rgb(100, 200, 100)),
            ),
            Span::styled(
                " ^E→ ",
                Style::default().bg(bg).fg(Color::Rgb(100, 200, 100)),
            ),
            Span::styled(
                " ^Ttab ",
                Style::default().bg(bg).fg(Color::Rgb(200, 150, 100)),
            ),
            Span::styled(
                " Alt1-9 ",
                Style::default().bg(bg).fg(Color::Rgb(150, 150, 220)),
            ),
            Span::styled(
                " ^Qquit ",
                Style::default().bg(bg).fg(Color::Rgb(200, 100, 100)),
            ),
        ];

        let right_text: String = right_parts.iter().map(|s| s.content.as_ref()).collect();
        let right_x = area.right().saturating_sub(right_text.len() as u16);
        let _right_area = Rect::new(right_x, area.y, right_text.len() as u16, area.height);
        _ = _right_area;

        let mut x_offset = 0;
        for part in &right_parts {
            let part_area = Rect::new(right_x + x_offset, area.y, part.content.len() as u16, 1);
            Paragraph::new(part.clone()).render(part_area, buf);
            x_offset += part.content.len() as u16;
        }

        // Separator line at top
        if area.y > 0 {
            for x in area.x..area.x + area.width {
                if let Some(cell) = buf.cell_mut((x, area.y - 1)) {
                    cell.set_fg(Color::Rgb(40, 45, 60));
                    cell.set_char('─');
                }
            }
        }
    }
}

/// Calculate the area for the status bar
pub fn status_bar_area(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.bottom().saturating_sub(1),
        width: area.width,
        height: 1,
    }
}
