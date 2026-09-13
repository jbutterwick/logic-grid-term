//! Inbox: thread list beside a preview of the selected thread's latest mail. On a narrow
//! screen the list sits above the preview instead.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget, Wrap};

use crate::ui::chrome::{self, Tab};
use crate::ui::{App, Key, Screen, step};

pub(crate) fn on_key(app: &mut App, key: Key) {
    let n = app.game().case.suspects.len() + 1;
    match key {
        Key::Char('q') => app.quit = true,
        Key::Char('?') => app.help = true,
        Key::Enter => {
            let t = app.thread();
            app.game_mut().mark_read(t);
            app.scroll = 0;
            app.screen = Screen::Thread;
        }
        Key::Char('n') => app.screen = Screen::Notepad,
        Key::Char('a') if !app.game().solved => {
            app.accuse_step = 0;
            app.accuse_suspect = 0;
            app.accuse_item = 0;
            app.screen = Screen::Accuse;
        }
        Key::Char('w') => app.game_mut().wait(),
        Key::Char('e') => app.fx = !app.fx,
        _ => step(&mut app.inbox_sel, key, n),
    }
}

pub(crate) fn render(app: &App, area: Rect, buf: &mut Buffer) {
    let g = app.game();
    let t = &app.look;
    let area = chrome::header(app, Some(Tab::Inbox), area, buf);
    let area = chrome::footer(
        app,
        &[
            ("j/k", "move"),
            ("Enter", "open"),
            ("n", "notepad"),
            ("a", "accuse"),
            ("w", "wait"),
            ("e", "effects"),
            ("?", "help"),
            ("q", "quit"),
        ],
        area,
        buf,
    );
    let [banner, body] =
        Layout::vertical([Constraint::Length(u16::from(g.solved)), Constraint::Fill(1)])
            .areas(area);
    if g.solved {
        Paragraph::new(Line::from(format!(" Case closed: {:?} ", g.rank())).style(t.banner()))
            .render(banner, buf);
    }

    let rows = g.case.suspects.len() as u16 + 1;
    let (list, preview) = if chrome::narrow(body) {
        let [a, b] =
            Layout::vertical([Constraint::Length(rows + 2), Constraint::Fill(1)]).areas(body);
        (a, b)
    } else {
        let [a, b] = Layout::horizontal([Constraint::Length(32), Constraint::Fill(1)]).areas(body);
        (a, b)
    };

    // Thread list: marker, name, badge flush right; the selected row is a solid block.
    let inner = chrome::frame(app, "THREADS", None, true, list, buf);
    let mut entries = vec![(g.case.frame.chief.clone(), g.unread(None), false)];
    entries.extend(
        g.case
            .suspects
            .iter()
            .enumerate()
            .map(|(i, s)| (s.name.clone(), g.unread(Some(i)), s.silenced)),
    );
    let name_w = usize::from(inner.width).saturating_sub(9);
    let lines: Vec<Line> = entries
        .into_iter()
        .enumerate()
        .map(|(i, (name, unread, silent))| {
            let selected = i == app.inbox_sel;
            let name: String = name.chars().take(name_w).collect();
            let pad = name_w - name.chars().count();
            let (badge, badge_style) = if unread > 0 {
                (format!("{unread} new"), t.accent())
            } else if silent {
                ("silent".to_string(), t.muted())
            } else {
                (String::new(), t.text())
            };
            let head = format!(
                "{} {name}{} ",
                if selected { "▸" } else { " " },
                " ".repeat(pad)
            );
            let badge = format!("{badge:>6}");
            if selected {
                Line::from(format!("{head}{badge}")).style(t.inverted())
            } else {
                Line::from(vec![Span::raw(head), Span::styled(badge, badge_style)])
            }
        })
        .collect();
    Paragraph::new(lines).render(inner, buf);
    chrome::noise(app, inner, buf);

    // Preview: the latest mail of the selected thread.
    let (title, mails) = match app.thread() {
        None => (g.case.frame.chief.clone(), &g.chief),
        Some(i) => (g.case.suspects[i].email.clone(), &g.threads[i]),
    };
    let count = format!("{} of {}", mails.len(), mails.len());
    let inner = chrome::frame(
        app,
        &title.to_uppercase(),
        (!mails.is_empty()).then_some(count.as_str()),
        false,
        preview,
        buf,
    );
    let inner = inset(inner);
    let lines = match mails.last() {
        Some(e) => chrome::mail_lines(app, e, inner.width),
        None => vec![Line::from("No mail yet. Open the thread and compose.").style(t.muted())],
    };
    Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .render(inner, buf);
    chrome::noise(app, inner, buf);
}

/// One blank column each side, so text never touches the frame.
pub(crate) fn inset(r: Rect) -> Rect {
    Rect::new(r.x + 1, r.y, r.width.saturating_sub(2), r.height)
}
