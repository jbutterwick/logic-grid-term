//! Inbox: thread list on the left, preview of the selected thread on the right.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Gauge, Paragraph, Widget, Wrap};

use super::{footer, thread};
use crate::ui::{App, BOLD, HILITE, Key, MUTED, Screen, step};

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
        _ => step(&mut app.inbox_sel, key, n),
    }
}

pub(crate) fn render(app: &App, area: Rect, buf: &mut Buffer) {
    let g = app.game();
    let area = footer(
        "j/k move  Enter open  n notepad  a accuse  w wait for mail  ? help  q quit",
        area,
        buf,
    );
    let [top, banner, body] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(u16::from(g.solved)),
        Constraint::Fill(1),
    ])
    .areas(area);

    // Top line: title, gauge, tick and rank.
    let (done, total) = g.progress();
    let [title, gauge, status] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(20),
        Constraint::Length(26),
    ])
    .areas(top);
    Paragraph::new(Line::from(g.case.frame.title.clone()).style(BOLD)).render(title, buf);
    Gauge::default()
        .ratio(if total == 0 {
            0.0
        } else {
            done as f64 / total as f64
        })
        .label(format!("{done}/{total} facts"))
        .gauge_style(Style::new().fg(Color::Green).bg(Color::DarkGray))
        .render(gauge, buf);
    Paragraph::new(Line::from(format!("  tick {}  {:?}", g.tick, g.rank())).style(MUTED))
        .render(status, buf);
    if g.solved {
        Paragraph::new(
            Line::from(format!(" Case closed: {:?} ", g.rank()))
                .style(Style::new().fg(Color::Black).bg(Color::Green)),
        )
        .render(banner, buf);
    }

    let [left, right] =
        Layout::horizontal([Constraint::Length(28), Constraint::Fill(1)]).areas(body);

    // Thread list.
    let mut rows = vec![row(&g.case.frame.chief, g.unread(None), false)];
    rows.extend(
        g.case
            .suspects
            .iter()
            .enumerate()
            .map(|(i, s)| row(&s.name, g.unread(Some(i)), s.silenced)),
    );
    let lines: Vec<Line> = rows
        .into_iter()
        .enumerate()
        .map(|(i, l)| {
            if i == app.inbox_sel {
                l.style(HILITE)
            } else {
                l
            }
        })
        .collect();
    let block = Block::bordered().title(" Threads ");
    let inner = block.inner(left);
    block.render(left, buf);
    Paragraph::new(lines).render(inner, buf);

    // Preview: the latest email of the selected thread.
    let t = app.thread();
    let name = match t {
        None => g.case.frame.chief.clone(),
        Some(i) => g.case.suspects[i].email.clone(),
    };
    let block = Block::bordered().title(format!(" {name} "));
    let inner = block.inner(right);
    block.render(right, buf);
    let mails = match t {
        None => &g.chief,
        Some(i) => &g.threads[i],
    };
    let lines = match mails.last() {
        Some(e) => thread::email_lines(e),
        None => vec![Line::from("No mail yet. Open the thread and compose.").style(MUTED)],
    };
    Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .render(inner, buf);
}

fn row(name: &str, unread: usize, silent: bool) -> Line<'static> {
    let mut spans = vec![Span::raw(format!("{name:<14}"))];
    if unread > 0 {
        spans.push(Span::styled(format!(" {unread} new"), BOLD));
    }
    if silent {
        spans.push(Span::styled(" silent", MUTED));
    }
    Line::from(spans)
}
