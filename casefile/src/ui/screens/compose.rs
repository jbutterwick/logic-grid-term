//! Compose: pick a question from the suspect's menu.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};

use super::footer;
use crate::ui::{App, HILITE, Key, MUTED, Screen, step};

pub(crate) fn on_key(app: &mut App, key: Key) {
    let Some(s) = app.thread() else {
        app.screen = Screen::Thread;
        return;
    };
    let qs = app.game().questions(s);
    match key {
        Key::Esc => app.screen = Screen::Thread,
        Key::Char('?') => app.help = true,
        Key::Enter => {
            if let Some(&q) = qs.get(app.compose_sel) {
                app.game_mut().ask(s, q);
            }
            app.screen = Screen::Thread;
        }
        _ => step(&mut app.compose_sel, key, qs.len()),
    }
}

pub(crate) fn render(app: &App, area: Rect, buf: &mut Buffer) {
    let g = app.game();
    let area = footer("j/k move  Enter send  Esc cancel", area, buf);
    let Some(s) = app.thread() else { return };
    let block = Block::bordered().title(format!(" To: {} ", g.case.suspects[s].email));
    let inner = block.inner(area);
    block.render(area, buf);
    let qs = g.questions(s);
    let lines: Vec<Line> = if qs.is_empty() {
        vec![Line::from("They won't talk to you any more.").style(MUTED)]
    } else {
        qs.iter()
            .enumerate()
            .map(|(i, q)| {
                let l = Line::from(q.label(&g.case));
                if i == app.compose_sel {
                    l.style(HILITE)
                } else {
                    l
                }
            })
            .collect()
    };
    Paragraph::new(lines).render(inner, buf);
}
