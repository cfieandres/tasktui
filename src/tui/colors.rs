use ratatui::style::{Color, Modifier, Style};

/// Dark/Yellow color theme
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,      // Gold/Yellow
    pub secondary: Color,    // Amber
    pub accent: Color,       // Orange
    pub text_dim: Color,
    pub border: Color,
    pub border_focused: Color,
}

pub const THEME: Theme = Theme {
    background: Color::Rgb(10, 10, 15),      // #0A0A0F
    foreground: Color::Rgb(220, 220, 220),   // Light gray
    primary: Color::Rgb(255, 215, 0),        // #FFD700 Gold
    secondary: Color::Rgb(255, 191, 0),      // #FFBF00 Amber
    accent: Color::Rgb(255, 140, 0),         // #FF8C00 Orange
    text_dim: Color::Rgb(128, 128, 128),     // Gray
    border: Color::Rgb(64, 64, 64),          // Dark gray
    border_focused: Color::Rgb(255, 215, 0), // Gold
};

impl Theme {
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn normal_style(&self) -> Style {
        Style::default().fg(self.foreground)
    }

    pub fn dim_style(&self) -> Style {
        Style::default().fg(self.text_dim)
    }

    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(self.background)
            .bg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn border_focused_style(&self) -> Style {
        Style::default().fg(self.border_focused)
    }

    pub fn accent_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn tag_style(&self) -> Style {
        Style::default().fg(self.secondary)
    }
}
