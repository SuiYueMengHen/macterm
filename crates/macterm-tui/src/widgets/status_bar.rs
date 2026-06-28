use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::text::Span;
use ratatui::widgets::{Paragraph, Widget};

pub struct StatusBar<'a> {
    pub tab_count: usize,
    pub pane_count: usize,
    pub active_tab: usize,
    pub message: Option<&'a str>,
    pub message_color: Color,
    pub show_file_tree: bool,
    pub version: &'a str,
    pub fullscreen_pane_mode: bool,
    pub zoom_mode: bool,
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let mode_indicator = if self.zoom_mode {
            " [ZOOM] "
        } else if self.fullscreen_pane_mode {
            " [FULL] "
        } else {
            ""
        };

        let left_text = format!(
            " Tab {} | {} panes{} ",
            self.active_tab + 1,
            self.pane_count,
            mode_indicator,
        );
        let left_span = Span::raw(left_text.clone());
        let left_area = Rect::new(area.x, area.y, left_text.len() as u16, area.height);
        Paragraph::new(left_span).render(left_area, buf);

        if let Some(msg) = self.message {
            let msg_span = Span::raw(format!(" {} ", msg));
            let msg_area = Rect::new(
                area.x + left_area.width + 1,
                area.y,
                msg.len() as u16 + 2,
                area.height,
            );
            Paragraph::new(msg_span).render(msg_area, buf);
        }

        let right_parts = vec![
            Span::raw(" ^D↓ "),
            Span::raw(" ^E→ "),
            Span::raw(" ^Ttab "),
            Span::raw(" ^Pcmd "),
            Span::raw(" ^Qquit "),
        ];

        let right_text: String = right_parts.iter().map(|s| s.content.as_ref()).collect();
        let right_x = area.right().saturating_sub(right_text.len() as u16);

        let mut x_offset = 0;
        for part in &right_parts {
            let part_area = Rect::new(right_x + x_offset, area.y, part.content.len() as u16, 1);
            Paragraph::new(part.clone()).render(part_area, buf);
            x_offset += part.content.len() as u16;
        }
    }
}

pub fn status_bar_area(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.bottom().saturating_sub(1),
        width: area.width,
        height: 1,
    }
}
