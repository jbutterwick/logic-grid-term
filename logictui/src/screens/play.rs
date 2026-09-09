//! Play screen: triangular grid + clue panel (B2).

use std::path::PathBuf;

use logicgrid::{Cat, Entity, Grid, Mark, Puzzle};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Widget, Wrap};

use crate::app::Transition;
use crate::storage::{self, Progress};
use crate::theme;

/// Width of one grid cell in columns (mark + spacer).
const CW: u16 = 2;
const MAX_LABEL: usize = 8;

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
    undo: Vec<Grid>,
    help: bool,
    solved: bool,
    status: String,
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

/// Sets a mark, auto-marking the rest of the sub-grid row/column as No on Yes (B2.3).
pub fn apply(grid: &mut Grid, p: &Puzzle, a: Entity, b: Entity, m: Mark) {
    grid.set(a, b, m);
    if m == Mark::Yes {
        for i in 0..p.n_items() {
            if i != a.item {
                grid.set(
                    Entity {
                        cat: a.cat,
                        item: i,
                    },
                    b,
                    Mark::No,
                );
            }
            if i != b.item {
                grid.set(
                    a,
                    Entity {
                        cat: b.cat,
                        item: i,
                    },
                    Mark::No,
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
            help: false,
            status: String::new(),
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

    fn set_mark(&mut self, m: Mark) {
        let (a, b) = self.entities(self.cursor);
        self.undo.push(self.grid.clone());
        apply(&mut self.grid, &self.puzzle, a, b, m);
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
        if self.help {
            self.help = false;
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
                if let Some(g) = self.undo.pop() {
                    self.grid = g;
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
        let (gw, _) = self.grid_size();
        let [main, status] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(area);
        let [grid_area, clue_area] =
            Layout::horizontal([Constraint::Length(gw + 1), Constraint::Min(20)]).areas(main);
        self.render_grid(grid_area, buf);
        self.render_clues(clue_area, buf);
        let seed = format!(
            "seed {} · {}x{} {} · ? help · q quit",
            self.puzzle.seed,
            self.n(),
            self.m(),
            self.label
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
        let put = |buf: &mut Buffer, x: u16, y: u16, s: &str, st: Style| {
            if x < area.right() && y < area.bottom() {
                buf.set_stringn(x, y, s, (area.right() - x) as usize, st);
            }
        };
        let (cur_a, cur_b) = self.entities(self.cursor);
        let dim = Style::new().fg(theme::MUTED);
        let bold = Style::new().fg(theme::ACCENT).bold();

        for (bj, &c) in cc.iter().enumerate() {
            put(
                buf,
                col_x(bj, 0),
                area.y,
                &self.puzzle.categories[c].name,
                bold,
            );
            for (ij, item) in self.puzzle.categories[c].items.iter().enumerate() {
                for (k, ch) in item.chars().take(hh as usize).enumerate() {
                    put(
                        buf,
                        col_x(bj, ij),
                        area.y + 1 + k as u16,
                        &ch.to_string(),
                        dim,
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
                put(buf, name_x, row_y(bi, ii), &label, dim);
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
w               write puzzle JSON to cwd
Esc             back to menu
q               quit
any key closes this";

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
    fn yes_auto_marks_row_and_column_no() {
        let p = fixtures::four();
        let mut g = Grid::new(&p);
        apply(&mut g, &p, e(0, 1), e(2, 3), Mark::Yes);
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
        // Removing the Yes keeps the auto-Nos (B2.3).
        apply(&mut g, &p, e(0, 1), e(2, 3), Mark::Unknown);
        assert_eq!(g.get(e(0, 0), e(2, 3)), Mark::No);
    }

    #[test]
    fn win_detected_on_fixture() {
        let p = fixtures::five();
        let mut g = Grid::new(&p);
        assert!(!g.is_solved(&p));
        for c in 1..p.n_cats() {
            for i in 0..p.n_items() {
                apply(&mut g, &p, e(0, i), e(c, p.solution.0[c][i]), Mark::Yes);
            }
        }
        assert!(g.is_solved(&p));
        assert!(g.contradictions(&p).is_empty());
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
            assert!(gw + 20 <= w && gh < h, "{n}: grid {gw}x{gh} in {w}x{h}");
            let mut buf = Buffer::empty(Rect::new(0, 0, w, h));
            s.render(Rect::new(0, 0, w, h), &mut buf); // must not panic
        }
    }
}
