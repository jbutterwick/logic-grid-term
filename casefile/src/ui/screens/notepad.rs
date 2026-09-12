//! Notepad: suspects x facts grid on top, free-text notes below.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget, Wrap};

use super::inbox::inset;
use crate::ui::chrome::{self, Tab};
use crate::ui::{App, Key, Screen};
use crate::{Entity, Mark};

/// Every non-anchor entity, in column order.
fn columns(app: &App) -> Vec<Entity> {
    let p = &app.game().case.puzzle;
    (1..p.n_cats())
        .flat_map(|cat| (0..p.n_items()).map(move |item| Entity { cat, item }))
        .collect()
}

pub(crate) fn on_key(app: &mut App, key: Key) {
    if app.pad_notes {
        match key {
            Key::Esc => app.screen = Screen::Inbox,
            Key::Tab => app.pad_notes = false,
            Key::Enter => app.game_mut().notes.push('\n'),
            Key::Backspace => {
                app.game_mut().notes.pop();
            }
            Key::Char(c) => app.game_mut().notes.push(c),
            _ => {}
        }
        return;
    }
    let cols = columns(app);
    let rows = app.game().case.suspects.len();
    let a = Entity {
        cat: 0,
        item: app.pad_row,
    };
    let b = cols[app.pad_col];
    let cur = app.game().grid.get(a, b);
    let set = |app: &mut App, m| app.game_mut().grid.set(a, b, m);
    match key {
        Key::Esc => app.screen = Screen::Inbox,
        Key::Char('?') => app.help = true,
        Key::Tab => app.pad_notes = true,
        Key::Up | Key::Char('k') => app.pad_row = app.pad_row.saturating_sub(1),
        Key::Down | Key::Char('j') => app.pad_row = (app.pad_row + 1).min(rows - 1),
        Key::Left | Key::Char('h') => app.pad_col = app.pad_col.saturating_sub(1),
        Key::Right | Key::Char('l') => app.pad_col = (app.pad_col + 1).min(cols.len() - 1),
        Key::Char(' ') | Key::Enter => set(
            app,
            match cur {
                Mark::Unknown => Mark::No,
                Mark::No => Mark::Yes,
                Mark::Yes => Mark::Unknown,
            },
        ),
        Key::Char('x') => set(app, Mark::No),
        Key::Char('o') => set(app, Mark::Yes),
        Key::Backspace => set(app, Mark::Unknown),
        _ => {}
    }
}

pub(crate) fn render(app: &App, area: Rect, buf: &mut Buffer) {
    let g = app.game();
    let t = &app.look;
    let p = &g.case.puzzle;
    let keys: &[(&str, &str)] = if app.pad_notes {
        &[
            ("type", "to edit"),
            ("Enter", "newline"),
            ("Tab", "grid"),
            ("Esc", "back"),
        ]
    } else {
        &[
            ("hjkl", "move"),
            ("Space", "cycle"),
            ("x", "no"),
            ("o", "yes"),
            ("Bksp", "clear"),
            ("Tab", "notes"),
            ("Esc", "back"),
        ]
    };
    let area = chrome::header(app, Some(Tab::Notepad), area, buf);
    let area = chrome::footer(app, keys, area, buf);
    let m = p.n_items() as u16;
    let [grid_area, notes_area] =
        Layout::vertical([Constraint::Length(m + 5), Constraint::Fill(1)]).areas(area);

    let inner = inset(chrome::frame(
        app,
        "GRID",
        None,
        !app.pad_notes,
        grid_area,
        buf,
    ));
    let cols = columns(app);
    let name_w = g
        .case
        .suspects
        .iter()
        .map(|s| s.name.chars().count())
        .max()
        .unwrap_or(4)
        .min(12) as u16
        + 1;
    // ponytail: fixed cell width from the space left; headers truncate rather than rotate.
    let w = ((inner.width.saturating_sub(name_w)) / cols.len().max(1) as u16).clamp(2, 9);
    let text_w = usize::from(w - 1);
    let x0 = inner.x + name_w;
    let put = |buf: &mut Buffer, x: u16, y: u16, s: &str, style| {
        if y < inner.bottom() && x < inner.right() {
            buf.set_stringn(x, y, s, usize::from(inner.right() - x), style);
        }
    };
    // Header row 1: category names at each group's first column; row 2: item names.
    for (ci, e) in cols.iter().enumerate() {
        let x = x0 + ci as u16 * w;
        if e.item == 0 {
            put(buf, x, inner.y, &p.categories[e.cat].name, t.bright());
        }
        let item: String = p.name(*e).chars().take(text_w).collect();
        put(buf, x, inner.y + 1, &item, t.muted());
    }
    for (r, s) in g.case.suspects.iter().enumerate() {
        let y = inner.y + 2 + r as u16;
        let name: String = s.name.chars().take(usize::from(name_w) - 1).collect();
        put(buf, inner.x, y, &name, t.bright());
        for (ci, e) in cols.iter().enumerate() {
            let x = x0 + ci as u16 * w;
            let sym = match g.grid.get(Entity { cat: 0, item: r }, *e) {
                Mark::Unknown => "·",
                Mark::No => "✗",
                Mark::Yes => "●",
            };
            let hot = !app.pad_notes && r == app.pad_row && ci == app.pad_col;
            let style = if hot {
                t.inverted()
            } else if sym == "·" {
                t.muted()
            } else {
                t.bright()
            };
            put(buf, x, y, sym, style);
        }
    }

    // Legend: the full names behind the (possibly truncated) column at the cursor.
    let cur = cols[app.pad_col];
    let legend = format!(
        "{} × {}: {}",
        g.case.suspects[app.pad_row].name,
        p.categories[cur.cat].name,
        p.name(cur)
    );
    put(buf, inner.x, inner.y + 2 + m, &legend, t.muted());

    let right = if app.pad_notes { Some("editing") } else { None };
    let inner = inset(chrome::frame(
        app,
        "NOTES",
        right,
        app.pad_notes,
        notes_area,
        buf,
    ));
    let mut text = g.notes.clone();
    if app.pad_notes {
        text.push('▏');
    }
    let lines: Vec<Line> = text
        .split('\n')
        .map(|l| Line::from(l.to_string()))
        .collect();
    let n = lines.len() as u16;
    Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((n.saturating_sub(inner.height), 0))
        .render(inner, buf);
    chrome::noise(app, inner, buf);
}
