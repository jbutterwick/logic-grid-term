//! One file per screen: `on_key(app, key)` and `render(app, area, buf)`.

pub(crate) mod accuse;
pub(crate) mod compose;
pub(crate) mod help;
pub(crate) mod inbox;
pub(crate) mod notepad;
pub(crate) mod setup;
pub(crate) mod thread;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use super::MUTED;

/// Render a muted single-line key legend at the bottom of `area`; returns the rest.
pub(crate) fn footer(text: &str, area: Rect, buf: &mut Buffer) -> Rect {
    if area.height == 0 {
        return area;
    }
    let bottom = Rect::new(area.x, area.bottom() - 1, area.width, 1);
    Paragraph::new(Line::from(text).style(MUTED)).render(bottom, buf);
    Rect::new(area.x, area.y, area.width, area.height - 1)
}
