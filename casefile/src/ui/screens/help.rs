//! Help overlay: key list; any key closes it.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::widgets::{Block, Clear, Paragraph, Widget};

use crate::ui::App;

const TEXT: &str = "\
Setup    ↑↓ move   ←→ change   digits seed   Enter start
Inbox    j/k move   Enter open   n notepad   a accuse   w wait   q quit
Thread   j/k scroll   c compose (suspects)   Esc back
Compose  j/k move   Enter send   Esc cancel
Notepad  hjkl move   Space cycle   x no   o yes   Backspace clear
         Tab notes / grid   Esc back
Accuse   j/k move   Enter next   y confirm   Esc back
?        this help   (any key closes)";

pub(crate) fn render(_app: &App, area: Rect, buf: &mut Buffer) {
    let [v] = Layout::vertical([Constraint::Length(11)])
        .flex(Flex::Center)
        .areas(area);
    let [box_area] = Layout::horizontal([Constraint::Length(72)])
        .flex(Flex::Center)
        .areas(v);
    Clear.render(box_area, buf);
    let block = Block::bordered().title(" Keys ");
    let inner = block.inner(box_area);
    block.render(box_area, buf);
    Paragraph::new(TEXT).render(inner, buf);
}
