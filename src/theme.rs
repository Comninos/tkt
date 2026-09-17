use ratatui::layout::Alignment;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, BorderType};

/// Palette family + light/dark variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Palette {
    Catppuccin,
    Ayu,
}

impl Palette {
    pub fn name(self) -> &'static str {
        match self {
            Palette::Catppuccin => "catppuccin",
            Palette::Ayu => "ayu",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "catppuccin" | "cappuccin" | "cappuchin" => Some(Palette::Catppuccin),
            "ayu" => Some(Palette::Ayu),
            _ => None,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Palette::Catppuccin => Palette::Ayu,
            Palette::Ayu => Palette::Catppuccin,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeId {
    pub palette: Palette,
    pub dark: bool,
}

impl ThemeId {
    pub fn label(self) -> String {
        let variant = if self.dark { "dark" } else { "light" };
        format!("{} {}", self.palette.name(), variant)
    }

    pub fn toggle_light_dark(self) -> Self {
        Self {
            dark: !self.dark,
            ..self
        }
    }

    pub fn cycle_palette(self) -> Self {
        Self {
            palette: self.palette.next(),
            ..self
        }
    }

    pub fn theme(self) -> Theme {
        match (self.palette, self.dark) {
            (Palette::Catppuccin, true) => Theme::catppuccin_mocha(),
            (Palette::Catppuccin, false) => Theme::catppuccin_latte(),
            (Palette::Ayu, true) => Theme::ayu_dark(),
            (Palette::Ayu, false) => Theme::ayu_light(),
        }
    }

    pub fn to_storage(self) -> (String, String) {
        (
            self.palette.name().to_string(),
            if self.dark {
                "dark".into()
            } else {
                "light".into()
            },
        )
    }

    pub fn from_storage(palette: &str, variant: &str) -> Self {
        let palette = Palette::parse(palette).unwrap_or(Palette::Catppuccin);
        let dark = !matches!(variant.trim().to_ascii_lowercase().as_str(), "light" | "latte");
        Self { palette, dark }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub bg: Color,
    pub surface: Color,
    pub text: Color,
    pub muted: Color,
    pub border: Color,
    pub border_focus: Color,
    pub title: Color,
    pub accent: Color,
    pub metric: Color,
    pub highlight_fg: Color,
    pub highlight_bg: Color,
    pub danger: Color,
    pub warn: Color,
    pub good: Color,
}

impl Theme {
    pub fn panel(&self, title: impl AsRef<str>, focused: bool) -> Block<'static> {
        self.panel_with_status(title, focused, None)
    }

    pub fn panel_with_status(
        &self,
        title: impl AsRef<str>,
        focused: bool,
        status: Option<&str>,
    ) -> Block<'static> {
        let border = if focused {
            self.border_focus
        } else {
            self.border
        };
        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border))
            .style(Style::default().bg(self.surface).fg(self.text))
            .title(
                Line::from(Span::styled(
                    format!(" {} ", title.as_ref().trim()),
                    Style::default()
                        .fg(if focused { self.title } else { self.muted })
                        .add_modifier(Modifier::BOLD),
                ))
                .left_aligned(),
            );

        if let Some(status) = status.map(str::trim).filter(|s| !s.is_empty()) {
            block = block.title(
                Line::from(Span::styled(
                    format!(" {status} "),
                    Style::default()
                        .fg(self.accent)
                        .add_modifier(Modifier::BOLD),
                ))
                .alignment(Alignment::Right),
            );
        }

        block
    }

    pub fn root_style(&self) -> Style {
        Style::default().bg(self.bg).fg(self.text)
    }

    pub fn help_style(&self) -> Style {
        Style::default().bg(self.bg).fg(self.muted)
    }

    pub fn key_style(&self) -> Style {
        Style::default().fg(self.accent).add_modifier(Modifier::BOLD)
    }

    fn catppuccin_mocha() -> Self {
        Self {
            bg: rgb(0x1e, 0x1e, 0x2e),
            surface: rgb(0x18, 0x18, 0x25),
            text: rgb(0xcd, 0xd6, 0xf4),
            muted: rgb(0x6c, 0x70, 0x86),
            border: rgb(0x45, 0x47, 0x5a),
            border_focus: rgb(0xcb, 0xa6, 0xf7),
            title: rgb(0xcb, 0xa6, 0xf7),
            accent: rgb(0xcb, 0xa6, 0xf7),
            metric: rgb(0x89, 0xb4, 0xfa),
            highlight_fg: rgb(0x1e, 0x1e, 0x2e),
            highlight_bg: rgb(0xcb, 0xa6, 0xf7),
            danger: rgb(0xf3, 0x8b, 0xa8),
            warn: rgb(0xf9, 0xe2, 0xaf),
            good: rgb(0xa6, 0xe3, 0xa1),
        }
    }

    fn catppuccin_latte() -> Self {
        Self {
            bg: rgb(0xef, 0xf1, 0xf5),
            surface: rgb(0xe6, 0xe9, 0xef),
            text: rgb(0x4c, 0x4f, 0x69),
            muted: rgb(0x9c, 0xa0, 0xb0),
            border: rgb(0xbc, 0xc0, 0xcc),
            border_focus: rgb(0x88, 0x39, 0xef),
            title: rgb(0x88, 0x39, 0xef),
            accent: rgb(0x88, 0x39, 0xef),
            metric: rgb(0x1e, 0x66, 0xf5),
            highlight_fg: rgb(0xef, 0xf1, 0xf5),
            highlight_bg: rgb(0x88, 0x39, 0xef),
            danger: rgb(0xd2, 0x0f, 0x39),
            warn: rgb(0xdf, 0x8e, 0x1d),
            good: rgb(0x40, 0xa0, 0x2b),
        }
    }

    fn ayu_dark() -> Self {
        Self {
            bg: rgb(0x0b, 0x0e, 0x14),
            surface: rgb(0x0f, 0x13, 0x1a),
            text: rgb(0xbf, 0xbd, 0xb6),
            muted: rgb(0x63, 0x6a, 0x72),
            border: rgb(0x1c, 0x23, 0x2b),
            border_focus: rgb(0xff, 0x8f, 0x40),
            title: rgb(0xff, 0x8f, 0x40),
            accent: rgb(0xff, 0x8f, 0x40),
            metric: rgb(0x59, 0xc2, 0xff),
            highlight_fg: rgb(0x0b, 0x0e, 0x14),
            highlight_bg: rgb(0xff, 0x8f, 0x40),
            danger: rgb(0xf0, 0x71, 0x78),
            warn: rgb(0xff, 0xb4, 0x54),
            good: rgb(0xaa, 0xd9, 0x4c),
        }
    }

    fn ayu_light() -> Self {
        Self {
            bg: rgb(0xf8, 0xf9, 0xfa),
            surface: rgb(0xfc, 0xfc, 0xfc),
            text: rgb(0x5c, 0x61, 0x66),
            muted: rgb(0x8a, 0x91, 0x99),
            border: rgb(0xce, 0xd2, 0xd6),
            border_focus: rgb(0xfa, 0x8d, 0x3e),
            title: rgb(0xfa, 0x8d, 0x3e),
            accent: rgb(0xfa, 0x8d, 0x3e),
            metric: rgb(0x39, 0x9e, 0xe6),
            highlight_fg: rgb(0xf8, 0xf9, 0xfa),
            highlight_bg: rgb(0xfa, 0x8d, 0x3e),
            danger: rgb(0xf0, 0x71, 0x78),
            warn: rgb(0xf2, 0xae, 0x49),
            good: rgb(0x86, 0xb3, 0x00),
        }
    }
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}
