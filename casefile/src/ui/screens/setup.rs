//! Setup: level, theme, adult toggle, seed, start.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};

use super::footer;
use crate::Level;
use crate::ui::{App, BOLD, HILITE, Key, MUTED};

const LEVELS: [Level; 3] = [Level::Easy, Level::Medium, Level::Hard];
const ROWS: usize = 5;

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
                _ => {}
            }
        }
        Key::Char(c) if app.setup_row == 3 && c.is_ascii_digit() && app.seed_text.len() < 19 => {
            app.seed_text.push(c)
        }
        Key::Backspace if app.setup_row == 3 => {
            app.seed_text.pop();
        }
        Key::Enter if app.setup_row == 4 => app.start(),
        Key::Enter => app.setup_row += 1,
        _ => {}
    }
}

pub(crate) fn render(app: &App, area: Rect, buf: &mut Buffer) {
    let area = footer(
        "↑↓ move  ←→ change  Enter start  ? help  Esc quit",
        area,
        buf,
    );
    let block = Block::bordered().title(" casefile ");
    let inner = block.inner(area);
    block.render(area, buf);
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
        seed,
        String::new(),
    ];
    let labels = ["Level", "Theme", "Adult", "Seed", "Start"];
    let mut lines = vec![Line::from("Open a new case").style(BOLD), Line::from("")];
    for (i, (label, value)) in labels.iter().zip(values).enumerate() {
        let style = if i == app.setup_row { HILITE } else { BOLD };
        lines.push(Line::from(vec![
            Span::styled(format!("{label:<7}"), style),
            Span::raw(" "),
            Span::raw(value),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(
        Line::from("Adult: profanity, harsher crimes, innuendo; nothing explicit.").style(MUTED),
    );
    Paragraph::new(lines).render(inner, buf);
}
