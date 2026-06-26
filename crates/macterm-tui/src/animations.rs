use ratatui::style::Color;
use tachyonfx::{Duration, Effect, Shader};

/// Timeline for managing multiple simultaneous animations
pub struct AnimationTimeline {
    pub effects: Vec<Effect>,
}

impl AnimationTimeline {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    /// Add a new animation effect
    pub fn add(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    /// Process all active effects and return completed ones
    pub fn process(&mut self, _dt: Duration) -> Vec<Effect> {
        let mut completed = Vec::new();
        self.effects.retain(|effect| {
            if effect.done() {
                completed.push(effect.clone());
                false
            } else {
                true
            }
        });
        completed
    }

    /// Check if any animation is running
    pub fn is_running(&self) -> bool {
        !self.effects.is_empty()
    }
}

/// A smooth color interpolation between two colors
#[derive(Debug, Clone)]
pub struct ColorAnimation {
    pub from: Color,
    pub to: Color,
    pub current: Color,
    pub progress: f32,
    pub duration: f32, // in seconds
}

impl ColorAnimation {
    pub fn new(from: Color, to: Color, duration: f32) -> Self {
        Self {
            from,
            to,
            current: from,
            progress: 0.0,
            duration,
        }
    }

    /// Advance the animation by dt seconds
    pub fn advance(&mut self, dt: f32) -> bool {
        self.progress = (self.progress + dt / self.duration).min(1.0);
        self.current = self.lerp_color(self.from, self.to, self.ease_out_cubic(self.progress));
        self.progress >= 1.0
    }

    fn ease_out_cubic(&self, t: f32) -> f32 {
        1.0 - (1.0 - t).powi(3)
    }

    fn lerp_color(&self, a: Color, b: Color, t: f32) -> Color {
        fn to_rgb(c: Color) -> (u8, u8, u8) {
            match c {
                Color::Rgb(r, g, b) => (r, g, b),
                _ => (0, 0, 0),
            }
        }
        let (r1, g1, b1) = to_rgb(a);
        let (r2, g2, b2) = to_rgb(b);
        Color::Rgb(
            (r1 as f32 + (r2 as f32 - r1 as f32) * t) as u8,
            (g1 as f32 + (g2 as f32 - g1 as f32) * t) as u8,
            (b1 as f32 + (b2 as f32 - b1 as f32) * t) as u8,
        )
    }

    pub fn finished(&self) -> bool {
        self.progress >= 1.0
    }
}

/// Renders a pane border with smooth color animation support
pub fn animated_border(
    title: Option<String>,
    is_active: bool,
    animation: Option<&ColorAnimation>,
) -> ratatui::widgets::Block<'static> {
    let border_color = match animation {
        Some(anim) => anim.current,
        None => {
            if is_active {
                Color::Rgb(0, 180, 255) // bright cyan for active
            } else {
                Color::Rgb(80, 80, 80) // dim gray for inactive
            }
        }
    };

    let mut block = ratatui::widgets::Block::default()
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(ratatui::style::Style::default().fg(border_color));

    if let Some(t) = title {
        block = block.title(t);
    }

    block
}
