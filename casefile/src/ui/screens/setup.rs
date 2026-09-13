//! Setup: level, theme, adult toggle, screen phosphor, effects, seed, start.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget, Wrap};

use super::inbox::inset;
use crate::Level;
use crate::ui::chrome;
use crate::ui::theme::Phosphor;
use crate::ui::{App, Key};

const LEVELS: [Level; 3] = [Level::Easy, Level::Medium, Level::Hard];

/// The rows, top to bottom. Resume only shows while a saved game is on offer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Row {
    Resume,
    Level,
    Theme,
    Adult,
    Screen,
    Effects,
    Seed,
    Start,
}

fn rows(app: &App) -> Vec<Row> {
    let mut rows = Vec::with_capacity(8);
    if app.saved.is_some() {
        rows.push(Row::Resume);
    }
    rows.extend([
        Row::Level,
        Row::Theme,
        Row::Adult,
        Row::Screen,
        Row::Effects,
        Row::Seed,
        Row::Start,
    ]);
    rows
}

fn current(app: &App) -> Row {
    let rows = rows(app);
    rows[app.setup_row.min(rows.len() - 1)]
}

/// True while the cursor is on the seed, where digits are typed.
pub(crate) fn on_seed_row(app: &App) -> bool {
    current(app) == Row::Seed
}

pub(crate) fn on_key(app: &mut App, key: Key) {
    let themes = crate::theme_names().len();
    let n = rows(app).len();
    let row = current(app);
    match key {
        Key::Esc => app.quit = true,
        Key::Char('?') => app.help = true,
        Key::Up | Key::Char('k') => app.setup_row = app.setup_row.saturating_sub(1),
        Key::Down | Key::Char('j') | Key::Tab => app.setup_row = (app.setup_row + 1) % n,
        Key::Left | Key::Right | Key::Char('h') | Key::Char('l') => {
            let fwd = matches!(key, Key::Right | Key::Char('l'));
            match row {
                Row::Level => {
                    let i = LEVELS.iter().position(|&l| l == app.level).unwrap_or(0);
                    app.level = LEVELS[(i + if fwd { 1 } else { 2 }) % 3];
                }
                // None = Random sits before index 0.
                Row::Theme => {
                    app.theme = match (app.theme, fwd) {
                        (None, true) => Some(0),
                        (None, false) => themes.checked_sub(1),
                        (Some(i), true) if i + 1 < themes => Some(i + 1),
                        (Some(_), true) => None,
                        (Some(0), false) => None,
                        (Some(i), false) => Some(i - 1),
                    }
                }
                Row::Adult => app.adult = !app.adult,
                Row::Screen => {
                    let all = Phosphor::ALL;
                    let i = all.iter().position(|&p| p == app.phosphor()).unwrap_or(0);
                    let n = all.len();
                    app.set_phosphor(all[(i + if fwd { 1 } else { n - 1 }) % n]);
                }
                Row::Effects => app.fx = !app.fx,
                _ => {}
            }
        }
        Key::Char(c) if row == Row::Seed && c.is_ascii_digit() && app.seed_text.len() < 19 => {
            app.seed_text.push(c)
        }
        Key::Backspace if row == Row::Seed => {
            app.seed_text.pop();
        }
        Key::Enter if row == Row::Start => app.start(),
        Key::Enter if row == Row::Resume => {
            app.resume();
        }
        Key::Enter => app.setup_row += 1,
        _ => {}
    }
}

pub(crate) fn render(app: &App, area: Rect, buf: &mut Buffer) {
    let t = &app.look;
    let area = chrome::header(app, None, area, buf);
    let area = chrome::footer(
        app,
        &[
            ("↑↓", "move"),
            ("←→", "change"),
            ("Enter", "start"),
            ("?", "help"),
            ("Esc", "quit"),
        ],
        area,
        buf,
    );
    let inner = inset(chrome::frame(app, "NEW CASE", None, true, area, buf));
    let theme = match app.theme {
        None => "Random".to_string(),
        Some(i) => crate::theme_names()[i].to_string(),
    };
    let seed = if app.seed_text.is_empty() {
        format!("{} (default)", app.seed)
    } else {
        app.seed_text.clone()
    };
    let resume = app.saved.as_ref().map(|g| {
        format!(
            "{} · seed {} · {} · tick {}{}",
            g.case.frame.title,
            g.case.settings.seed,
            g.case.settings.level.name(),
            g.tick,
            if g.solved { " · closed" } else { "" }
        )
    });
    let entries: Vec<(&str, String)> = rows(app)
        .into_iter()
        .map(|row| match row {
            Row::Resume => ("Resume", resume.clone().unwrap_or_default()),
            Row::Level => ("Level", format!("< {} >", app.level.name())),
            Row::Theme => ("Theme", format!("< {theme} >")),
            Row::Adult => (
                "Adult",
                format!("< {} >", if app.adult { "on" } else { "off" }),
            ),
            Row::Screen => ("Screen", format!("< {} >", app.phosphor().name())),
            Row::Effects => (
                "Effects",
                format!("< {} >", if app.fx { "on" } else { "off" }),
            ),
            Row::Seed => ("Seed", seed.clone()),
            Row::Start => ("Start", String::new()),
        })
        .collect();
    let mut lines = vec![
        Line::from("Open a new case").style(t.bright()),
        Line::from(""),
    ];
    for (i, (label, value)) in entries.into_iter().enumerate() {
        let selected = i == app.setup_row;
        let marker = if selected { "▸" } else { " " };
        let label_style = if selected { t.inverted() } else { t.bright() };
        lines.push(Line::from(vec![
            Span::raw(format!("{marker} ")),
            Span::styled(format!(" {label:<7}"), label_style),
            Span::raw("  "),
            Span::raw(value),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(
        Line::from("Adult: profanity, harsher crimes, innuendo; nothing explicit.")
            .style(t.muted()),
    );
    lines.push(Line::from("Screen: the phosphor the case glows in.").style(t.muted()));
    if app.saved.is_some() {
        lines.push(
            Line::from("Resume: the case is saved after every move; Enter picks it up again.")
                .style(t.muted()),
        );
    }
    lines.push(
        Line::from(
            "Effects: static, and in the browser the whole CRT glass. Off is a plain terminal.",
        )
        .style(t.muted()),
    );
    Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .render(inner, buf);
    chrome::noise(app, inner, buf);
}
