//! Thread: every email in one conversation, newest last, scrollable.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget, Wrap};

use super::footer;
use crate::Email;
use crate::ui::{App, BOLD, Key, MUTED, OUTGOING, Screen};

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
            let n = lines(app).len() as u16;
            app.scroll = (app.scroll + 1).min(n.saturating_sub(1));
        }
        _ => {}
    }
}

pub(crate) fn render(app: &App, area: Rect, buf: &mut Buffer) {
    let g = app.game();
    let (name, hint) = match app.thread() {
        None => (g.case.frame.chief.clone(), "j/k scroll  Esc back"),
        Some(i) if g.case.suspects[i].silenced => (
            g.case.suspects[i].email.clone(),
            "j/k scroll  Esc back  (silent)",
        ),
        Some(i) => (
            g.case.suspects[i].email.clone(),
            "j/k scroll  c compose  Esc back",
        ),
    };
    let area = footer(hint, area, buf);
    let block = Block::bordered().title(format!(" {name} "));
    let inner = block.inner(area);
    block.render(area, buf);
    Paragraph::new(lines(app))
        .wrap(Wrap { trim: false })
        .scroll((app.scroll, 0))
        .render(inner, buf);
}

fn lines(app: &App) -> Vec<Line<'static>> {
    let g = app.game();
    let mails = match app.thread() {
        None => &g.chief,
        Some(i) => &g.threads[i],
    };
    if mails.is_empty() {
        return vec![Line::from("No mail yet. Press c to compose.").style(MUTED)];
    }
    mails.iter().flat_map(email_lines).collect()
}

/// Header plus body plus a blank line; outgoing mail is tinted.
pub(crate) fn email_lines(e: &Email) -> Vec<Line<'static>> {
    let tint = if e.outgoing { OUTGOING } else { BOLD };
    let mut out = vec![
        Line::from(Span::styled(format!("From: {}", e.from), tint)),
        Line::from(Span::styled(
            format!("Subject: {}   (tick {})", e.subject, e.tick),
            MUTED,
        )),
    ];
    for l in e.body.lines() {
        let l = Line::from(l.to_string());
        out.push(if e.outgoing { l.style(OUTGOING) } else { l });
    }
    out.push(Line::from(""));
    out
}
