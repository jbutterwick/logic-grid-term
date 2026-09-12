//! Phosphor palettes. Every screen paints from one [`Theme`] so a palette swap is one value.

use std::str::FromStr;

use ratatui::style::{Color, Modifier, Style};

/// Which phosphor the screen glows in.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Phosphor {
    /// P1 green, the default.
    #[default]
    Green,
    /// P3 amber.
    Amber,
    /// P4 white.
    White,
}

impl Phosphor {
    /// Every phosphor, in setup-screen order.
    pub const ALL: [Phosphor; 3] = [Phosphor::Green, Phosphor::Amber, Phosphor::White];

    /// Lower-case name, as accepted by [`FromStr`].
    pub fn name(self) -> &'static str {
        match self {
            Phosphor::Green => "green",
            Phosphor::Amber => "amber",
            Phosphor::White => "white",
        }
    }
}

impl FromStr for Phosphor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Phosphor::ALL
            .into_iter()
            .find(|p| p.name().eq_ignore_ascii_case(s))
            .ok_or_else(|| format!("unknown phosphor {s:?}; known: green, amber, white"))
    }
}

/// The colours one phosphor renders in. All explicit RGB, so the terminal's own palette
/// never leaks through.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Theme {
    /// Which phosphor this is.
    pub phosphor: Phosphor,
    /// Screen background.
    pub bg: Color,
    /// Body text.
    pub fg: Color,
    /// Headings and keys: the brightest glow.
    pub bright: Color,
    /// Labels and legends.
    pub dim: Color,
    /// Static in empty space; barely there.
    pub noise: Color,
    /// The one contrasting colour: unread badges, outgoing mail, banners.
    pub accent: Color,
    /// Frame around the pane that owns the keys.
    pub border_focus: Color,
    /// Frame around every other pane.
    pub border: Color,
}

const fn rgb(hex: u32) -> Color {
    Color::Rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

impl Theme {
    /// The palette for a phosphor.
    pub fn new(phosphor: Phosphor) -> Theme {
        match phosphor {
            Phosphor::Green => Theme {
                phosphor,
                bg: rgb(0x050a06),
                fg: rgb(0x7bf3a0),
                bright: rgb(0xdbffe6),
                dim: rgb(0x2f8a4f),
                noise: rgb(0x163a25),
                accent: rgb(0xffb347),
                border_focus: rgb(0x4fc776),
                border: rgb(0x2f8a4f),
            },
            Phosphor::Amber => Theme {
                phosphor,
                bg: rgb(0x0a0704),
                fg: rgb(0xffb347),
                bright: rgb(0xffe0b0),
                dim: rgb(0x9a6a1e),
                noise: rgb(0x3a2a10),
                accent: rgb(0x7bf3a0),
                border_focus: rgb(0xd9922e),
                border: rgb(0x9a6a1e),
            },
            Phosphor::White => Theme {
                phosphor,
                bg: rgb(0x07080a),
                fg: rgb(0xd6dde3),
                bright: rgb(0xffffff),
                dim: rgb(0x6b7680),
                noise: rgb(0x262c33),
                accent: rgb(0xffb347),
                border_focus: rgb(0xa8b3bd),
                border: rgb(0x6b7680),
            },
        }
    }

    /// Plain body text on the screen background.
    pub fn text(&self) -> Style {
        Style::new().fg(self.fg).bg(self.bg)
    }

    /// Headings, names, and key caps.
    pub fn bright(&self) -> Style {
        Style::new().fg(self.bright).add_modifier(Modifier::BOLD)
    }

    /// Labels, legends, and anything secondary.
    pub fn muted(&self) -> Style {
        Style::new().fg(self.dim)
    }

    /// Static.
    pub fn noise(&self) -> Style {
        Style::new().fg(self.noise)
    }

    /// The contrasting colour.
    pub fn accent(&self) -> Style {
        Style::new().fg(self.accent)
    }

    /// Selection: phosphor block with dark text.
    pub fn inverted(&self) -> Style {
        Style::new().fg(self.bg).bg(self.fg)
    }

    /// Banner: accent block with dark text.
    pub fn banner(&self) -> Style {
        Style::new()
            .fg(self.bg)
            .bg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Frame colour for a pane.
    pub fn border(&self, focused: bool) -> Style {
        Style::new().fg(if focused {
            self.border_focus
        } else {
            self.border
        })
    }
}

impl Default for Theme {
    fn default() -> Theme {
        Theme::new(Phosphor::Green)
    }
}
