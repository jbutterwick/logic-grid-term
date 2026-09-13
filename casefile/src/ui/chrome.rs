//! Shared chrome: the header strip, double-line pane frames, the key legend, and static.
//! Every screen paints the background through [`header`] first, so the terminal's own
//! colours never show.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph, Widget};

use super::App;
use crate::Email;

/// Below this many columns the panes stack and the header takes two rows. Side by side, the
/// thread list takes 32 columns, so this leaves the preview at least 38.
pub(crate) const NARROW: u16 = 70;

/// Which header tab lights up.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Tab {
    Inbox,
    Notepad,
    Accuse,
}

/// True when the area is too narrow for side-by-side panes.
pub(crate) fn narrow(area: Rect) -> bool {
    area.width < NARROW
}

/// One row: `left` flush left, `right` flush right if it fits, else dropped.
fn row(buf: &mut Buffer, area: Rect, left: Line, right: Line) {
    if area.height == 0 {
        return;
    }
    let (lw, rw) = (left.width() as u16, right.width() as u16);
    if rw > 0 && lw + rw <= area.width {
        let r = Rect::new(area.x + area.width - rw, area.y, rw, 1);
        Paragraph::new(right).render(r, buf);
    }
    Paragraph::new(left).render(Rect::new(area.x, area.y, area.width, 1), buf);
}

/// Paint the background and the header strip; returns the body area below a blank row.
pub(crate) fn header(app: &App, tab: Option<Tab>, area: Rect, buf: &mut Buffer) -> Rect {
    let t = &app.look;
    buf.set_style(area, t.text());
    let brand = Span::styled(" CASEFILE ", t.inverted());
    let title = app.game.as_ref().map(|g| g.case.frame.title.clone());
    let tabs = |left: &mut Vec<Span<'static>>| {
        let Some(tab) = tab else {
            left.push(Span::styled("NEW CASE", t.bright()));
            return;
        };
        for (i, (name, which)) in [
            ("INBOX", Tab::Inbox),
            ("NOTEPAD", Tab::Notepad),
            ("ACCUSE", Tab::Accuse),
        ]
        .into_iter()
        .enumerate()
        {
            if i > 0 {
                left.push(Span::raw("  "));
            }
            if which == tab {
                left.push(Span::styled(format!("[{name}]"), t.bright()));
            } else {
                left.push(Span::styled(name, t.muted()));
            }
        }
    };

    let Some(g) = app.game.as_ref() else {
        let mut left = vec![brand, Span::raw("  ")];
        tabs(&mut left);
        row(buf, area, Line::from(left), Line::default());
        return body_after(area, 2);
    };
    let (done, total) = g.progress();
    let filled = if total == 0 {
        0
    } else {
        (10 * done).div_ceil(total).min(10)
    };
    let gauge = |label: bool| {
        let mut v = Vec::new();
        if label {
            v.push(Span::styled("FACTS ", t.muted()));
        }
        v.extend([
            Span::styled("█".repeat(filled), t.bright()),
            Span::styled("░".repeat(10 - filled), t.noise()),
            Span::raw(format!(" {done}/{total}")),
        ]);
        v
    };
    let tick = format!("{:02}", g.tick);
    let rank = format!("{:?}", g.rank()).to_uppercase();

    if narrow(area) {
        let mut left = vec![brand, Span::raw("  ")];
        tabs(&mut left);
        row(buf, area, Line::from(left), Line::default());
        let mut second = vec![Span::raw(" ")];
        second.extend(gauge(true));
        second.extend([Span::styled("  TICK ", t.muted()), Span::raw(tick)]);
        let status = Line::from(vec![Span::raw(title.unwrap_or_default()), Span::raw(" ")]);
        row(
            buf,
            Rect::new(
                area.x,
                area.y + 1,
                area.width,
                area.height.saturating_sub(1),
            ),
            Line::from(second),
            status,
        );
        return body_after(area, 3);
    }

    let mut left = vec![
        brand,
        Span::raw(" "),
        Span::raw(title.unwrap_or_default()),
        Span::raw("   "),
    ];
    tabs(&mut left);
    let left = Line::from(left);
    // Right side, fullest first; the first one that fits beside the tabs wins.
    let full = {
        let mut v = gauge(true);
        v.extend([
            Span::styled("   TICK ", t.muted()),
            Span::raw(tick.clone()),
            Span::styled("   RANK ", t.muted()),
            Span::raw(rank),
            Span::raw(" "),
        ]);
        v
    };
    let no_rank = {
        let mut v = gauge(true);
        v.extend([
            Span::styled("   TICK ", t.muted()),
            Span::raw(tick.clone()),
            Span::raw(" "),
        ]);
        v
    };
    let tight = {
        let mut v = gauge(false);
        v.extend([
            Span::styled("  TICK ", t.muted()),
            Span::raw(tick),
            Span::raw(" "),
        ]);
        v
    };
    let mut bare = gauge(false);
    bare.push(Span::raw(" "));
    let lw = left.width();
    let right = [full, no_rank, tight, bare]
        .into_iter()
        .map(Line::from)
        .find(|r| lw + r.width() <= usize::from(area.width))
        .unwrap_or_default();
    row(buf, area, left, right);
    body_after(area, 2)
}

fn body_after(area: Rect, rows: u16) -> Rect {
    let rows = rows.min(area.height);
    Rect::new(area.x, area.y + rows, area.width, area.height - rows)
}

/// The key legend on the bottom row, a blank row above it; returns the area above both.
/// With the legend off (soft keys on screen) the whole area comes back untouched.
pub(crate) fn footer(app: &App, keys: &[(&str, &str)], area: Rect, buf: &mut Buffer) -> Rect {
    if !app.legend {
        return area;
    }
    if area.height < 2 {
        return Rect::new(area.x, area.y, area.width, 0);
    }
    let t = &app.look;
    let mut spans = vec![Span::raw(" ")];
    for (i, (k, label)) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(format!("[{k}]"), t.bright()));
        spans.push(Span::styled(format!(" {label}"), t.muted()));
    }
    let right = match app.game.as_ref() {
        Some(g) => Line::from(vec![
            Span::raw(g.case.frame.title.clone()),
            Span::styled(
                format!(
                    " · SEED {} · {} ",
                    g.case.settings.seed,
                    g.case.settings.level.name().to_uppercase()
                ),
                t.muted(),
            ),
        ]),
        None => Line::default(),
    };
    let bottom = Rect::new(area.x, area.bottom() - 1, area.width, 1);
    row(buf, bottom, Line::from(spans), right);
    Rect::new(area.x, area.y, area.width, area.height - 2)
}

/// A double-line frame with a title in the top rule; returns the inner area.
pub(crate) fn frame(
    app: &App,
    title: &str,
    right: Option<&str>,
    focused: bool,
    area: Rect,
    buf: &mut Buffer,
) -> Rect {
    let t = &app.look;
    let border = t.border(focused);
    let mut block = Block::bordered()
        .border_type(BorderType::Double)
        .border_style(border)
        .title(Line::from(vec![
            Span::styled("═", border),
            Span::styled(format!(" {title} "), t.bright()),
        ]));
    if let Some(r) = right {
        block = block.title_top(
            Line::from(vec![
                Span::styled(format!(" {r} "), t.muted()),
                Span::styled("═", border),
            ])
            .right_aligned(),
        );
    }
    let inner = block.inner(area);
    block.render(area, buf);
    inner
}

/// Sparse static over the empty rows at the bottom of `area`: every row after the last one
/// holding any text. Reseeds every few frames so it shimmers.
pub(crate) fn noise(app: &App, area: Rect, buf: &mut Buffer) {
    if !app.fx || area.is_empty() {
        return;
    }
    let blank = |buf: &Buffer, y: u16| {
        (area.left()..area.right()).all(|x| buf.cell((x, y)).is_none_or(|c| c.symbol() == " "))
    };
    let mut top = area.bottom();
    while top > area.top() && blank(buf, top - 1) {
        top -= 1;
    }
    let seed = app.static_seed();
    let style = app.look.noise();
    for y in top..area.bottom() {
        for x in area.left()..area.right() {
            let r = hash(seed, x, y) as f64 / u32::MAX as f64;
            let glyph = if r < 0.03 {
                "░"
            } else if r < 0.04 {
                "▒"
            } else if r < 0.075 {
                "·"
            } else {
                continue;
            };
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_symbol(glyph).set_style(style);
            }
        }
    }
}

fn hash(seed: u64, x: u16, y: u16) -> u32 {
    let mut h = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ u64::from(x).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ u64::from(y).wrapping_mul(0x1656_67B1_9E37_79F9);
    h ^= h >> 31;
    h = h.wrapping_mul(0x9FB2_1C65_1E98_DF25);
    h ^= h >> 28;
    (h >> 32) as u32
}

/// One email as rows: FROM, SUBJECT with the tick flush right, a rule, the body, a blank.
/// `width` is the column count the rows are laid out for.
pub(crate) fn mail_lines(app: &App, e: &Email, width: u16) -> Vec<Line<'static>> {
    let t = &app.look;
    let name = if e.outgoing { t.accent() } else { t.bright() };
    let tick = format!("tick {:02}", e.tick);
    let subject_w = usize::from(width).saturating_sub(9 + tick.len() + 1);
    let subject: String = e.subject.chars().take(subject_w).collect();
    let gap = subject_w.saturating_sub(subject.chars().count()) + 1;
    let mut out = vec![
        Line::from(vec![
            Span::styled("FROM     ", t.muted()),
            Span::styled(e.from.clone(), name),
        ]),
        Line::from(vec![
            Span::styled("SUBJECT  ", t.muted()),
            Span::raw(subject),
            Span::raw(" ".repeat(gap)),
            Span::styled(tick, t.muted()),
        ]),
        Line::from(Span::styled("─".repeat(usize::from(width)), t.muted())),
    ];
    for l in e.body.lines() {
        let l = Line::from(l.to_string());
        out.push(if e.outgoing { l.style(t.accent()) } else { l });
    }
    out.push(Line::from(""));
    out
}
