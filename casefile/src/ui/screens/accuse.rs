//! Accuse: pick a suspect, pick the guilty-category item, confirm.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};

use super::footer;
use crate::Entity;
use crate::ui::{App, BOLD, HILITE, Key, MUTED, Screen, step};

pub(crate) fn on_key(app: &mut App, key: Key) {
    let g = app.game();
    let (n, m) = (g.case.suspects.len(), g.case.puzzle.n_items());
    match (app.accuse_step, key) {
        (_, Key::Char('?')) => app.help = true,
        (0, Key::Esc) => app.screen = Screen::Inbox,
        (_, Key::Esc) => app.accuse_step -= 1,
        (0, Key::Enter) => app.accuse_step = 1,
        (1, Key::Enter) => app.accuse_step = 2,
        (2, Key::Char('y')) => {
            let fact = Entity {
                cat: g.case.guilty.cat,
                item: app.accuse_item,
            };
            let s = app.accuse_suspect;
            app.game_mut().accuse(s, fact);
            app.inbox_sel = 0;
            app.screen = Screen::Inbox;
        }
        (2, Key::Char('n')) => app.accuse_step = 1,
        (0, _) => step(&mut app.accuse_suspect, key, n),
        (1, _) => step(&mut app.accuse_item, key, m),
        _ => {}
    }
}

pub(crate) fn render(app: &App, area: Rect, buf: &mut Buffer) {
    let g = app.game();
    let c = &g.case;
    let cat = &c.puzzle.categories[c.guilty.cat];
    let hint = match app.accuse_step {
        2 => "y confirm  n back  Esc back",
        _ => "j/k move  Enter next  Esc back",
    };
    let area = footer(hint, area, buf);
    let block = Block::bordered().title(" Accuse ");
    let inner = block.inner(area);
    block.render(area, buf);
    let pick = |items: Vec<String>, sel: usize| -> Vec<Line<'static>> {
        items
            .into_iter()
            .enumerate()
            .map(|(i, s)| {
                let l = Line::from(s);
                if i == sel { l.style(HILITE) } else { l }
            })
            .collect()
    };
    let suspect = &c.suspects[app.accuse_suspect];
    let mut lines = vec![
        Line::from("A wrong accusation costs rank and silences the accused.").style(MUTED),
        Line::from(""),
    ];
    match app.accuse_step {
        0 => {
            lines.push(Line::from(format!("Step 1: which {}?", c.noun())).style(BOLD));
            lines.extend(pick(
                c.suspects.iter().map(|s| s.name.clone()).collect(),
                app.accuse_suspect,
            ));
        }
        1 => {
            lines.push(
                Line::from(format!("Step 2: {} and the {}?", suspect.name, cat.name)).style(BOLD),
            );
            lines.extend(pick(cat.items.clone(), app.accuse_item));
        }
        _ => {
            let fact = cat.phrase.replace("{}", &cat.items[app.accuse_item]);
            lines.push(Line::from("Step 3: send this to the chief?").style(BOLD));
            lines.push(Line::from(format!(
                "I accuse {}: the {} who {fact}.",
                suspect.name,
                c.noun()
            )));
            lines.push(Line::from(""));
            lines.push(Line::from("[y] yes   [n] no").style(MUTED));
        }
    }
    Paragraph::new(lines).render(inner, buf);
}
