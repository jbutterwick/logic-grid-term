//! Help overlay: key list; any key closes it.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::widgets::{Clear, Paragraph, Widget};

use crate::ui::App;
use crate::ui::chrome;

const TEXT: &str = "\
Setup    ↑↓ move   ←→ change   digits seed   Enter start
Inbox    j/k move   Enter open   n notepad   a accuse   w wait   q quit
Thread   j/k scroll   c compose (suspects)   Esc back
Compose  j/k move   Enter send   Esc cancel
Notepad  hjkl move   Space cycle   x no   o yes   Backspace clear
         Tab notes / grid   Esc back
Accuse   j/k move   Enter next   y confirm   Esc back
?        this help   (any key closes)";

pub(crate) fn render(app: &App, area: Rect, buf: &mut Buffer) {
    let [v] = Layout::vertical([Constraint::Length(11)])
        .flex(Flex::Center)
        .areas(area);
    let [box_area] = Layout::horizontal([Constraint::Length(74)])
        .flex(Flex::Center)
        .areas(v);
    Clear.render(box_area, buf);
    buf.set_style(box_area, app.look.text());
    let inner = chrome::frame(app, "KEYS", None, true, box_area, buf);
    Paragraph::new(TEXT).render(inner, buf);
}
