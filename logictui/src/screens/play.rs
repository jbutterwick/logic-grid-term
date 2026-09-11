//! Play screen: triangular grid + clue panel (B2).

use std::cell::Cell;
use std::collections::HashSet;
use std::path::PathBuf;

use logicgrid::{Cat, Entity, Grid, Mark, Puzzle};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Widget, Wrap};

use crate::app::Transition;
use crate::storage::{self, Progress};
use crate::theme;

/// Width of one grid cell in columns (mark + spacer).
const CW: u16 = 2;
// Longest theme label ("Locker 101"); themes.rs has a test pinning this.
const MAX_LABEL: usize = 10;
/// Breathing room between the terminal edge and the play screen.
const MARGIN: Margin = Margin::new(2, 1);

pub struct PlayScreen {
    puzzle: Puzzle,
    grid: Grid,
    struck: Vec<bool>,
    label: String,
    dir: PathBuf,
    /// Cursor over the full cell lattice: (row, col) in `0..(n-1)*m`.
    cursor: (usize, usize),
    clue_sel: usize,
    bad: Vec<(Entity, Entity)>,
    undo: Vec<(Grid, Auto)>,
    /// Cells auto-marked No by a Yes, keyed by the Yes cell that placed them.
    auto: Auto,
    help: bool,
    /// Intro blurb overlay; shown on open, reopened with `i`.
    intro: bool,
    solved: bool,
    status: String,
    /// (x, y) of the top-left mark cell from the last render, for mouse clicks.
    origin: Cell<(u16, u16)>,
}

/// Row-block categories top to bottom: anchor, then n-1 down to 2.
fn row_cats(n: usize) -> Vec<Cat> {
    std::iter::once(0).chain((2..n).rev()).collect()
}

/// Column-block categories left to right: 1..n.
fn col_cats(n: usize) -> Vec<Cat> {
    (1..n).collect()
}

/// Sub-grid (bi, bj) exists in the staircase iff bi + bj < n - 1.
fn block_exists(n: usize, bi: usize, bj: usize) -> bool {
    bi + bj < n - 1
}

/// Auto-placed No cells, keyed by the Yes cell that caused them.
pub type Auto = HashSet<(Entity, Entity, Entity, Entity)>;

/// Sets a mark. A Yes auto-marks the rest of its sub-grid row/column as No; leaving a Yes
/// reverts only the Nos that Yes placed, so mistaken Yes marks are cheap to undo (B2.3).
pub fn apply(grid: &mut Grid, auto: &mut Auto, p: &Puzzle, a: Entity, b: Entity, m: Mark) {
    if grid.get(a, b) == Mark::Yes && m != Mark::Yes {
        auto.retain(|&(ya, yb, x, y)| {
            let mine = (ya, yb) == (a, b);
            if mine && grid.get(x, y) == Mark::No {
                grid.set(x, y, Mark::Unknown);
            }
            !mine
        });
    }
    grid.set(a, b, m);
    if m == Mark::Yes {
        let mut mark = |x: Entity, y: Entity| {
            if grid.get(x, y) == Mark::Unknown {
                grid.set(x, y, Mark::No);
                auto.insert((a, b, x, y));
            }
        };
        for i in 0..p.n_items() {
            if i != a.item {
                mark(
                    Entity {
                        cat: a.cat,
                        item: i,
                    },
                    b,
                );
            }
            if i != b.item {
                mark(
                    a,
                    Entity {
                        cat: b.cat,
                        item: i,
                    },
                );
            }
        }
    }
}

impl PlayScreen {
    pub fn new(puzzle: Puzzle, label: String, dir: PathBuf) -> Self {
        let key = storage::key(&puzzle, &label);
        let saved = storage::load(&dir, &key).filter(|pr| {
            pr.puzzle.categories == puzzle.categories && pr.puzzle.clues == puzzle.clues
        });
        let (grid, struck, solved) = match saved {
            Some(pr) => (pr.grid, pr.struck, pr.solved),
            None => (Grid::new(&puzzle), vec![false; puzzle.clues.len()], false),
        };
        let intro = !puzzle.intro.is_empty();
        PlayScreen {
            grid,
            struck,
            solved,
            puzzle,
            label,
            dir,
            cursor: (0, 0),
            clue_sel: 0,
            bad: vec![],
            undo: vec![],
            // ponytail: not persisted; after a restore, un-Yes keeps its auto-Nos.
            auto: Auto::new(),
            help: false,
            intro,
            status: String::new(),
            origin: Cell::new((0, 0)),
        }
    }

    fn n(&self) -> usize {
        self.puzzle.n_cats()
    }

    fn m(&self) -> usize {
        self.puzzle.n_items()
    }

    fn valid(&self, (r, c): (usize, usize)) -> bool {
        let m = self.m();
        block_exists(self.n(), r / m, c / m)
    }

    fn entities(&self, (r, c): (usize, usize)) -> (Entity, Entity) {
        let (n, m) = (self.n(), self.m());
        (
            Entity {
                cat: row_cats(n)[r / m],
                item: r % m,
            },
            Entity {
                cat: col_cats(n)[c / m],
                item: c % m,
            },
        )
    }

    fn step(&mut self, dr: isize, dc: isize) {
        let (r, c) = self.cursor;
        let next = (r.checked_add_signed(dr), c.checked_add_signed(dc));
        if let (Some(r), Some(c)) = next
            && r < (self.n() - 1) * self.m()
            && c < (self.n() - 1) * self.m()
            && self.valid((r, c))
        {
            self.cursor = (r, c);
        }
    }

    /// Move the cursor to the mark cell under screen position (x, y), if any.
    pub fn on_click(&mut self, x: u16, y: u16) {
        let (ox, oy) = self.origin.get();
        let (Some(dx), Some(dy)) = (x.checked_sub(ox), y.checked_sub(oy)) else {
            return;
        };
        let (m, bw) = (self.m(), self.m() * CW as usize + 1);
        let (dx, dy) = (dx as usize, dy as usize);
        let (bj, ij) = (dx / bw, (dx % bw) / CW as usize);
        let (bi, ii) = (dy / (m + 1), dy % (m + 1));
        let next = (bi * m + ii, bj * m + ij);
        if ij < m && ii < m && bi + 1 < self.n() && bj + 1 < self.n() && self.valid(next) {
            self.cursor = next;
        }
    }

    fn set_mark(&mut self, m: Mark) {
        let (a, b) = self.entities(self.cursor);
        self.undo.push((self.grid.clone(), self.auto.clone()));
        apply(&mut self.grid, &mut self.auto, &self.puzzle, a, b, m);
        self.after_change();
    }

    fn after_change(&mut self) {
        self.bad.clear();
        self.solved = self.grid.is_solved(&self.puzzle);
        self.status = if self.solved {
            "Solved!".into()
        } else {
            String::new()
        };
        self.save();
    }

    fn save(&self) {
        let pr = Progress {
            puzzle: self.puzzle.clone(),
            grid: self.grid.clone(),
            struck: self.struck.clone(),
            solved: self.solved,
            label: self.label.clone(),
        };
        // ponytail: best-effort autosave; a failed write just loses this change.
        let _ = storage::save(&self.dir, &storage::key(&self.puzzle, &self.label), &pr);
    }

    fn strike(&mut self, i: usize) {
        if let Some(s) = self.struck.get_mut(i) {
            *s = !*s;
            self.save();
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> Transition {
        if self.help || self.intro {
            (self.help, self.intro) = (false, false);
            return Transition::None;
        }
        let (a, b) = self.entities(self.cursor);
        let cur = self.grid.get(a, b);
        match key.code {
            KeyCode::Char('q') => return Transition::Quit,
            KeyCode::Esc => return Transition::Home,
            KeyCode::Left | KeyCode::Char('h') => self.step(0, -1),
            KeyCode::Right | KeyCode::Char('l') => self.step(0, 1),
            KeyCode::Up | KeyCode::Char('k') => self.step(-1, 0),
            KeyCode::Down | KeyCode::Char('j') => self.step(1, 0),
            KeyCode::Char(' ') | KeyCode::Enter => self.set_mark(match cur {
                Mark::Unknown => Mark::No,
                Mark::No => Mark::Yes,
                Mark::Yes => Mark::Unknown,
            }),
            KeyCode::Char('x') => self.set_mark(Mark::No),
            KeyCode::Char('o') => self.set_mark(Mark::Yes),
            KeyCode::Backspace | KeyCode::Delete => self.set_mark(Mark::Unknown),
            KeyCode::Char('u') => {
                if let Some((g, auto)) = self.undo.pop() {
                    self.grid = g;
                    self.auto = auto;
                    self.after_change();
                }
            }
            KeyCode::Tab | KeyCode::Char('n') => {
                self.clue_sel = (self.clue_sel + 1) % self.puzzle.clues.len().max(1);
            }
            KeyCode::BackTab | KeyCode::Char('p') => {
                let len = self.puzzle.clues.len().max(1);
                self.clue_sel = (self.clue_sel + len - 1) % len;
            }
            KeyCode::Char('s') => self.strike(self.clue_sel),
            KeyCode::Char(d @ '1'..='9') => self.strike(d as usize - '1' as usize),
            KeyCode::Char('c') => {
                self.bad = self.grid.contradictions(&self.puzzle);
                self.status = match self.bad.len() {
                    0 => "No contradictions".into(),
                    k => format!("{k} contradicting mark(s)"),
                };
            }
            KeyCode::Char('?') => self.help = true,
            KeyCode::Char('i') => self.intro = !self.puzzle.intro.is_empty(),
            KeyCode::Char('w') => {
                let path = format!("{}.json", storage::key(&self.puzzle, &self.label));
                self.status = match std::fs::write(&path, self.puzzle.to_json()) {
                    Ok(()) => format!("wrote {path}"),
                    Err(e) => format!("write failed: {e}"),
                };
            }
            _ => {}
        }
        Transition::None
    }

    // ---- rendering ----

    /// (label width incl. category column, header height) for the grid.
    fn label_dims(&self) -> (u16, u16) {
        let catw = self
            .puzzle
            .categories
            .iter()
            .map(|c| c.name.chars().count())
            .max()
            .unwrap_or(0)
            .min(MAX_LABEL);
        let itemw = self
            .puzzle
            .categories
            .iter()
            .flat_map(|c| c.items.iter())
            .map(|s| s.chars().count())
            .max()
            .unwrap_or(0)
            .min(MAX_LABEL);
        ((catw + 1 + itemw + 1) as u16, itemw as u16)
    }

    /// Columns and rows the grid needs (with full-height headers).
    pub fn grid_size(&self) -> (u16, u16) {
        let (n, m) = (self.n() as u16, self.m() as u16);
        let (left, hh) = self.label_dims();
        (
            left + (n - 1) * m * CW + (n - 2),
            1 + hh + (n - 1) * m + (n - 2),
        )
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let (gw, gh) = self.grid_size();
        // ponytail: drop the vertical margin on short terminals so headers stay full height.
        let vm = u16::from(area.height >= gh + 1 + 2 * MARGIN.vertical);
        let [main, status] = Layout::vertical([Constraint::Min(1), Constraint::Length(1)])
            .areas(area.inner(Margin::new(MARGIN.horizontal, vm)));
        let [grid_area, clue_area] =
            Layout::horizontal([Constraint::Length(gw + 1), Constraint::Min(20)]).areas(main);
        self.render_grid(grid_area, buf);
        let [info, clue_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).areas(clue_area);
        let (a, b) = self.entities(self.cursor);
        let mark = match self.grid.get(a, b) {
            Mark::Unknown => "-",
            Mark::No => "✗",
            Mark::Yes => "●",
        };
        Line::from(format!(
            " {} x {}: {mark}",
            self.puzzle.name(a),
            self.puzzle.name(b)
        ))
        .fg(theme::ACCENT)
        .render(info, buf);
        self.render_clues(clue_area, buf);
        let seed = format!(
            "seed {} · {}x{} {} · {} · ? help · q quit",
            self.puzzle.seed,
            self.n(),
            self.m(),
            self.label,
            self.puzzle.title
        );
        Line::from(vec![
            Span::from(seed).fg(theme::MUTED),
            Span::from("  "),
            Span::from(self.status.as_str()).fg(theme::ACCENT),
        ])
        .render(status, buf);
        if self.solved {
            popup(
                area,
                buf,
                " Solved! ",
                vec![
                    Line::from("You solved it!").fg(theme::WIN).bold(),
                    Line::from(format!("seed {}", self.puzzle.seed)),
                    Line::from("q quit · Esc menu").fg(theme::MUTED),
                ],
            );
        }
        if self.help {
            popup(area, buf, " Help ", HELP.lines().map(Line::from).collect());
        }
        if self.intro {
            let w = (area.width.saturating_sub(6) as usize).clamp(20, 60);
            let mut lines: Vec<Line> = wrap(&self.puzzle.intro, w)
                .into_iter()
                .map(Line::from)
                .collect();
            lines.push(Line::from(""));
            lines.push(Line::from("any key to begin · i reopens").fg(theme::MUTED));
            popup(area, buf, &format!(" {} ", self.puzzle.title), lines);
        }
    }

    fn render_grid(&self, area: Rect, buf: &mut Buffer) {
        let (n, m) = (self.n(), self.m());
        let (left, hh_full) = self.label_dims();
        let body_rows = ((n - 1) * m + (n - 2)) as u16;
        // Shrink vertical column labels if the terminal is short.
        let hh = hh_full
            .min(area.height.saturating_sub(1 + body_rows))
            .max(1);
        let rc = row_cats(n);
        let cc = col_cats(n);
        let bw = m as u16 * CW + 1; // block width incl. gap
        let col_x = |bj: usize, ij: usize| area.x + left + bj as u16 * bw + ij as u16 * CW;
        let row_y = |bi: usize, ii: usize| area.y + 1 + hh + (bi * (m + 1) + ii) as u16;
        self.origin.set((col_x(0, 0), row_y(0, 0)));
        let put = |buf: &mut Buffer, x: u16, y: u16, s: &str, st: Style| {
            if x < area.right() && y < area.bottom() {
                buf.set_stringn(x, y, s, (area.right() - x) as usize, st);
            }
        };
        let (cur_a, cur_b) = self.entities(self.cursor);
        let dim = Style::new().fg(theme::MUTED);
        let bold = Style::new().fg(theme::ACCENT).bold();
        let hi = Style::new().fg(theme::CURSOR_BG).bold();

        for (bj, &c) in cc.iter().enumerate() {
            put(
                buf,
                col_x(bj, 0),
                area.y,
                &self.puzzle.categories[c].name,
                bold,
            );
            for (ij, item) in self.puzzle.categories[c].items.iter().enumerate() {
                let st = if (Entity { cat: c, item: ij }) == cur_b {
                    hi
                } else {
                    dim
                };
                for (k, ch) in item.chars().take(hh as usize).enumerate() {
                    put(
                        buf,
                        col_x(bj, ij),
                        area.y + 1 + k as u16,
                        &ch.to_string(),
                        st,
                    );
                }
            }
        }
        for (bi, &r) in rc.iter().enumerate() {
            put(
                buf,
                area.x,
                row_y(bi, 0),
                &self.puzzle.categories[r].name,
                bold,
            );
            let (catw, _) = self.label_dims();
            let name_x = area.x + catw - 1 - hh_full;
            for (ii, item) in self.puzzle.categories[r].items.iter().enumerate() {
                let label: String = item.chars().take(MAX_LABEL).collect();
                let st = if (Entity { cat: r, item: ii }) == cur_a {
                    hi
                } else {
                    dim
                };
                put(buf, name_x, row_y(bi, ii), &label, st);
                for (bj, &c) in cc.iter().enumerate() {
                    if !block_exists(n, bi, bj) {
                        continue;
                    }
                    for ij in 0..m {
                        let (a, b) = (Entity { cat: r, item: ii }, Entity { cat: c, item: ij });
                        let (ch, mut st) = match self.grid.get(a, b) {
                            Mark::Unknown => ("·", dim),
                            Mark::No => ("✗", Style::new().fg(theme::NO)),
                            Mark::Yes => ("●", Style::new().fg(theme::YES).bold()),
                        };
                        if a == cur_a || b == cur_b {
                            st = st.bg(theme::CROSSHAIR_BG);
                        }
                        if self
                            .bad
                            .iter()
                            .any(|&(x, y)| (x, y) == (a, b) || (y, x) == (a, b))
                        {
                            st = st.bg(theme::BAD_BG).fg(theme::CURSOR_FG);
                        }
                        if (a, b) == (cur_a, cur_b) {
                            st = st.bg(theme::CURSOR_BG).fg(theme::CURSOR_FG);
                        }
                        put(buf, col_x(bj, ij), row_y(bi, ii), ch, st);
                    }
                }
            }
        }
        self.render_answers(area, buf, row_y(n - 2, m - 1) + 2);
    }

    /// Answer table below the grid if there is room (B2.8).
    fn render_answers(&self, area: Rect, buf: &mut Buffer, y: u16) {
        let (n, m) = (self.n(), self.m());
        if y + m as u16 + 1 > area.bottom() {
            return;
        }
        let colw = (MAX_LABEL + 1) as u16;
        if colw * n as u16 > area.width {
            return;
        }
        for c in 0..n {
            let x = area.x + c as u16 * colw;
            buf.set_stringn(
                x,
                y,
                &self.puzzle.categories[c].name,
                MAX_LABEL,
                Style::new().fg(theme::ACCENT).bold(),
            );
            for i in 0..m {
                let a = Entity { cat: 0, item: i };
                let name = if c == 0 {
                    Some(self.puzzle.name(a))
                } else {
                    (0..m)
                        .map(|j| Entity { cat: c, item: j })
                        .find(|&b| self.grid.get(a, b) == Mark::Yes)
                        .map(|b| self.puzzle.name(b))
                };
                let (s, st) = match name {
                    Some(s) => (s, Style::new()),
                    None => ("?", Style::new().fg(theme::MUTED)),
                };
                buf.set_stringn(x, y + 1 + i as u16, s, MAX_LABEL, st);
            }
        }
    }

    fn render_clues(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().title(" Clues (s strike · Tab next) ");
        let inner = block.inner(area);
        let width = inner.width.max(1) as usize;
        let mut lines = vec![];
        let mut sel_line = 0;
        let mut total = 0;
        for (i, clue) in self.puzzle.clues.iter().enumerate() {
            let text = format!("{:>2}. {}", i + 1, clue.text(&self.puzzle));
            let mut st = Style::new();
            if self.struck[i] {
                st = st.fg(theme::MUTED).add_modifier(Modifier::CROSSED_OUT);
            }
            if i == self.clue_sel {
                st = st.bg(theme::CURSOR_BG).fg(theme::CURSOR_FG);
                sel_line = total;
            }
            total += text.chars().count().div_ceil(width);
            lines.push(Line::styled(text, st));
        }
        // ponytail: scroll by estimated wrapped height, keeps the selected clue in view.
        let scroll = sel_line.saturating_sub(inner.height.saturating_sub(1) as usize / 2);
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((scroll as u16, 0))
            .render(area, buf);
    }
}

const HELP: &str = "\
arrows / hjkl   move cursor
Space / Enter   cycle · ✗ ●
x / o           mark No / Yes
Backspace       clear
u               undo
s               strike selected clue
1-9             strike clue N
Tab / n / p     select clue
c               check for contradictions
i               story intro
w               write puzzle JSON to cwd
Esc             back to menu
q               quit
any key closes this";

/// Greedy word wrap to `width` columns.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut out = vec![String::new()];
    for word in text.split_whitespace() {
        let cur = out.last_mut().expect("non-empty");
        if !cur.is_empty() && cur.chars().count() + 1 + word.chars().count() > width {
            out.push(word.to_string());
        } else {
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(word);
        }
    }
    out
}

fn popup(area: Rect, buf: &mut Buffer, title: &str, lines: Vec<Line>) {
    let h = lines.len() as u16 + 2;
    let w = lines.iter().map(|l| l.width()).max().unwrap_or(0) as u16 + 4;
    let r = Rect::new(
        area.x + area.width.saturating_sub(w) / 2,
        area.y + area.height.saturating_sub(h) / 2,
        w.min(area.width),
        h.min(area.height),
    );
    Clear.render(r, buf);
    Paragraph::new(lines)
        .block(Block::bordered().title(title).fg(theme::ACCENT))
        .render(r, buf);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    fn e(cat: usize, item: usize) -> Entity {
        Entity { cat, item }
    }

    #[test]
    fn click_maps_screen_cell_to_cursor() {
        let mut s = PlayScreen::new(fixtures::four(), "t".into(), PathBuf::new());
        let mut buf = Buffer::empty(Rect::new(0, 0, 120, 40));
        s.render(buf.area, &mut buf);
        let (a, b) = s.entities(s.cursor);
        let row0: String = (0..120)
            .map(|x| buf[(x, MARGIN.vertical)].symbol())
            .collect();
        assert!(row0.contains(&format!("{} x {}: -", s.puzzle.name(a), s.puzzle.name(b))));
        let (ox, oy) = s.origin.get();
        let m = s.m() as u16;
        // second block right, third row: skips block gap and spacer columns.
        s.on_click(ox + m * CW + 1 + CW, oy + 2);
        assert_eq!(s.cursor, (2, m as usize + 1));
        // gap column between blocks and empty triangle corner are ignored.
        let before = s.cursor;
        s.on_click(ox + m * CW, oy);
        s.on_click(
            ox + (s.n() as u16 - 2) * (m * CW + 1),
            oy + (s.n() as u16 - 2) * (m + 1),
        );
        assert_eq!(s.cursor, before);
    }

    #[test]
    fn yes_auto_marks_row_and_column_no() {
        let p = fixtures::four();
        let mut g = Grid::new(&p);
        let mut auto = Auto::new();
        // A player-placed No must survive the Yes/un-Yes round trip.
        g.set(e(0, 2), e(2, 3), Mark::No);
        apply(&mut g, &mut auto, &p, e(0, 1), e(2, 3), Mark::Yes);
        assert_eq!(g.get(e(0, 1), e(2, 3)), Mark::Yes);
        for i in 0..4 {
            if i != 1 {
                assert_eq!(g.get(e(0, i), e(2, 3)), Mark::No);
            }
            if i != 3 {
                assert_eq!(g.get(e(0, 1), e(2, i)), Mark::No);
            }
        }
        // Other sub-grids untouched.
        assert_eq!(g.get(e(0, 1), e(1, 0)), Mark::Unknown);
        // Removing the Yes reverts its auto-Nos but keeps the player's No (B2.3).
        apply(&mut g, &mut auto, &p, e(0, 1), e(2, 3), Mark::Unknown);
        assert_eq!(g.get(e(0, 0), e(2, 3)), Mark::Unknown);
        assert_eq!(g.get(e(0, 1), e(2, 0)), Mark::Unknown);
        assert_eq!(g.get(e(0, 2), e(2, 3)), Mark::No);
        assert!(auto.is_empty());
    }

    #[test]
    fn win_detected_on_fixture() {
        let p = fixtures::five();
        let mut g = Grid::new(&p);
        assert!(!g.is_solved(&p));
        for c in 1..p.n_cats() {
            for i in 0..p.n_items() {
                apply(
                    &mut g,
                    &mut Auto::new(),
                    &p,
                    e(0, i),
                    e(c, p.solution.0[c][i]),
                    Mark::Yes,
                );
            }
        }
        assert!(g.is_solved(&p));
        assert!(g.contradictions(&p).is_empty());
    }

    #[test]
    fn intro_shows_once_and_reopens() {
        let mut p = fixtures::four();
        assert!(!PlayScreen::new(p.clone(), "t".into(), PathBuf::new()).intro);
        p.title = "Test".into();
        p.intro = "one two three four five six seven".into();
        let mut s = PlayScreen::new(p, "t".into(), PathBuf::new());
        assert!(s.intro);
        s.on_key(KeyEvent::from(KeyCode::Char('x')));
        assert!(!s.intro && s.grid.get(e(0, 0), e(1, 0)) == Mark::Unknown);
        s.on_key(KeyEvent::from(KeyCode::Char('i')));
        assert!(s.intro);
        assert_eq!(
            wrap("one two three four five six seven", 10),
            ["one two", "three four", "five six", "seven"]
        );
    }

    #[test]
    fn cursor_stays_inside_staircase() {
        let mut s = PlayScreen::new(fixtures::four(), "t".into(), std::env::temp_dir());
        for _ in 0..20 {
            s.step(1, 0);
        }
        assert_eq!(s.cursor, (11, 0));
        for _ in 0..20 {
            s.step(0, 1);
        }
        assert_eq!(s.cursor, (11, 3)); // bottom row block only has one sub-grid
        assert_eq!(s.entities(s.cursor), (e(2, 3), e(1, 3)));
    }

    #[test]
    fn layouts_fit_target_terminals() {
        for (n, w, h) in [(4, 80, 24), (5, 120, 40)] {
            let s = PlayScreen::new(
                fixtures::fixture(n).unwrap(),
                "t".into(),
                std::env::temp_dir(),
            );
            let (gw, gh) = s.grid_size();
            let w2 = w - 2 * MARGIN.horizontal;
            assert!(gw + 20 <= w2 && gh < h, "{n}: grid {gw}x{gh} in {w2}x{h}");
            let mut buf = Buffer::empty(Rect::new(0, 0, w, h));
            s.render(Rect::new(0, 0, w, h), &mut buf); // must not panic
        }
    }
}
