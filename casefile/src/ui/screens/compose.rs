//! Compose: pick a question from the suspect's menu.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use super::inbox::inset;
use crate::ui::chrome::{self, Tab};
use crate::ui::{App, Key, Screen, step};

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
    let t = &app.look;
    let area = chrome::header(app, Some(Tab::Inbox), area, buf);
    let area = chrome::footer(
        app,
        &[("j/k", "move"), ("Enter", "send"), ("Esc", "cancel")],
        area,
        buf,
    );
    let Some(s) = app.thread() else { return };
    let title = format!("TO: {}", g.case.suspects[s].email);
    let inner = inset(chrome::frame(app, &title, Some("compose"), true, area, buf));
    let qs = g.questions(s);
    let lines: Vec<Line> = if qs.is_empty() {
        vec![Line::from("They won't talk to you any more.").style(t.muted())]
    } else {
        qs.iter()
            .enumerate()
            .map(|(i, q)| {
                let label = q.label(&g.case);
                if i == app.compose_sel {
                    Line::from(format!(
                        "▸ {label:<w$}",
                        w = usize::from(inner.width).saturating_sub(2)
                    ))
                    .style(t.inverted())
                } else {
                    Line::from(format!("  {label}"))
                }
            })
            .collect()
    };
    Paragraph::new(lines).render(inner, buf);
    chrome::noise(app, inner, buf);
}
