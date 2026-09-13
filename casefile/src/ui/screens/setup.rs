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
const ROWS: usize = 7;
const SEED_ROW: usize = 5;
const START_ROW: usize = 6;

pub(crate) fn on_key(app: &mut App, key: Key) {
    let themes = crate::theme_names().len();
    match key {
        Key::Esc => app.quit = true,
        Key::Char('?') => app.help = true,
        Key::Up | Key::Char('k') => app.setup_row = app.setup_row.saturating_sub(1),
        Key::Down | Key::Char('j') | Key::Tab => app.setup_row = (app.setup_row + 1) % ROWS,
        Key::Left | Key::Right | Key::Char('h') | Key::Char('l') => {
            let fwd = matches!(key, Key::Right | Key::Char('l'));
            match app.setup_row {
                0 => {
                    let i = LEVELS.iter().position(|&l| l == app.level).unwrap_or(0);
                    app.level = LEVELS[(i + if fwd { 1 } else { 2 }) % 3];
                }
                // None = Random sits before index 0.
                1 => {
                    app.theme = match (app.theme, fwd) {
                        (None, true) => Some(0),
                        (None, false) => themes.checked_sub(1),
                        (Some(i), true) if i + 1 < themes => Some(i + 1),
                        (Some(_), true) => None,
                        (Some(0), false) => None,
                        (Some(i), false) => Some(i - 1),
                    }
                }
                2 => app.adult = !app.adult,
                3 => {
                    let all = Phosphor::ALL;
                    let i = all.iter().position(|&p| p == app.phosphor()).unwrap_or(0);
                    let n = all.len();
                    app.set_phosphor(all[(i + if fwd { 1 } else { n - 1 }) % n]);
                }
                4 => app.fx = !app.fx,
                _ => {}
            }
        }
        Key::Char(c)
            if app.setup_row == SEED_ROW && c.is_ascii_digit() && app.seed_text.len() < 19 =>
        {
            app.seed_text.push(c)
        }
        Key::Backspace if app.setup_row == SEED_ROW => {
            app.seed_text.pop();
        }
        Key::Enter if app.setup_row == START_ROW => app.start(),
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
    let values = [
        format!("< {} >", app.level.name()),
        format!("< {theme} >"),
        format!("< {} >", if app.adult { "on" } else { "off" }),
        format!("< {} >", app.phosphor().name()),
        format!("< {} >", if app.fx { "on" } else { "off" }),
        seed,
        String::new(),
    ];
    let labels = [
        "Level", "Theme", "Adult", "Screen", "Effects", "Seed", "Start",
    ];
    let mut lines = vec![
        Line::from("Open a new case").style(t.bright()),
        Line::from(""),
    ];
    for (i, (label, value)) in labels.iter().zip(values).enumerate() {
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
