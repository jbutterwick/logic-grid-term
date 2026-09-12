//! Thread: every email in one conversation, newest last, scrollable.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget, Wrap};

use super::inbox::inset;
use crate::ui::chrome::{self, Tab};
use crate::ui::{App, Key, Screen};

pub(crate) fn on_key(app: &mut App, key: Key) {
    match key {
        Key::Esc => app.screen = Screen::Inbox,
        Key::Char('?') => app.help = true,
        Key::Char('c') if app.thread().is_some() => {
            app.compose_sel = 0;
            app.screen = Screen::Compose;
        }
        Key::Up | Key::Char('k') => app.scroll = app.scroll.saturating_sub(1),
        Key::Down | Key::Char('j') => {
            // ponytail: clamp to unwrapped line count; wrapped text can scroll a little short.
            let n = lines(app, 80).len() as u16;
            app.scroll = (app.scroll + 1).min(n.saturating_sub(1));
        }
        _ => {}
    }
}

pub(crate) fn render(app: &App, area: Rect, buf: &mut Buffer) {
    let g = app.game();
    let (title, right, keys): (String, String, &[(&str, &str)]) = match app.thread() {
        None => (
            g.case.frame.chief.clone(),
            format!("{} mail", g.chief.len()),
            &[("j/k", "scroll"), ("Esc", "back")],
        ),
        Some(i) if g.case.suspects[i].silenced => (
            g.case.suspects[i].email.clone(),
            "silent".to_string(),
            &[("j/k", "scroll"), ("Esc", "back")],
        ),
        Some(i) => (
            g.case.suspects[i].email.clone(),
            format!("{} mail", g.threads[i].len()),
            &[("j/k", "scroll"), ("c", "compose"), ("Esc", "back")],
        ),
    };
    let area = chrome::header(app, Some(Tab::Inbox), area, buf);
    let area = chrome::footer(app, keys, area, buf);
    let inner = chrome::frame(app, &title.to_uppercase(), Some(&right), true, area, buf);
    let inner = inset(inner);
    let lines = lines(app, inner.width);
    Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((app.scroll, 0))
        .render(inner, buf);
    chrome::noise(app, inner, buf);
}

fn lines(app: &App, width: u16) -> Vec<Line<'static>> {
    let g = app.game();
    let mails = match app.thread() {
        None => &g.chief,
        Some(i) => &g.threads[i],
    };
    if mails.is_empty() {
        return vec![Line::from("No mail yet. Press c to compose.").style(app.look.muted())];
    }
    mails
        .iter()
        .flat_map(|e| chrome::mail_lines(app, e, width))
        .collect()
}
