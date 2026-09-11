//! Fixed palette. ponytail: no TOML themes (B4.2 SHOULD); add crosstui-style loader if wanted.

use ratatui::style::Color;

pub const ACCENT: Color = Color::Blue;
pub const MUTED: Color = Color::DarkGray;
pub const YES: Color = Color::Green;
pub const NO: Color = Color::Red;
pub const CURSOR_BG: Color = Color::Yellow;
pub const CURSOR_FG: Color = Color::Black;
/// Row/column of the cursor cell.
pub const CROSSHAIR_BG: Color = Color::Rgb(45, 45, 60);
pub const BAD_BG: Color = Color::Red;
pub const WIN: Color = Color::Magenta;
