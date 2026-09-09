//! Home screen: pick size and difficulty (B1.1), resume recent puzzles (B3.3).

use std::path::Path;

use logicgrid::{Difficulty, Size};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};

use crate::app::{Transition, diff_name};
use crate::storage::{self, Progress};
use crate::theme;

const SIZES: [(usize, usize); 5] = [(3, 3), (3, 4), (4, 4), (4, 5), (5, 5)];
const DIFFS: [Difficulty; 3] = [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];

pub struct HomeScreen {
    size: usize,
    diff: usize,
    /// 0 = size row, 1 = difficulty row, 2.. = recent list.
    row: usize,
    recent: Vec<(String, Progress)>,
}

impl HomeScreen {
    pub fn new(dir: &Path) -> Self {
        HomeScreen {
            size: 2,
            diff: 1,
            row: 0,
            recent: storage::list(dir),
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> Transition {
        let rows = 2 + self.recent.len();
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Transition::Quit,
            KeyCode::Up | KeyCode::Char('k') => self.row = self.row.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => self.row = (self.row + 1).min(rows - 1),
            KeyCode::Left | KeyCode::Char('h') => match self.row {
                0 => self.size = (self.size + SIZES.len() - 1) % SIZES.len(),
                1 => self.diff = (self.diff + DIFFS.len() - 1) % DIFFS.len(),
                _ => {}
            },
            KeyCode::Right | KeyCode::Char('l') => match self.row {
                0 => self.size = (self.size + 1) % SIZES.len(),
                1 => self.diff = (self.diff + 1) % DIFFS.len(),
                _ => {}
            },
            KeyCode::Enter => {
                if self.row < 2 {
                    let (cats, items) = SIZES[self.size];
                    let d = DIFFS[self.diff];
                    let p = logicgrid::generate(Size { cats, items }, d, crate::random_seed());
                    return Transition::Play(p, diff_name(d));
                }
                let (_, pr) = &self.recent[self.row - 2];
                return Transition::Play(pr.puzzle.clone(), pr.label.clone());
            }
            _ => {}
        }
        Transition::None
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let sel = |r: usize| {
            if r == self.row {
                Style::new().fg(theme::CURSOR_FG).bg(theme::CURSOR_BG)
            } else {
                Style::new()
            }
        };
        let (c, i) = SIZES[self.size];
        let mut lines = vec![
            Line::from("logictui").fg(theme::ACCENT).bold(),
            Line::from(""),
            Line::styled(format!("  Size:        < {c}x{i} >"), sel(0)),
            Line::styled(
                format!("  Difficulty:  < {} >", diff_name(DIFFS[self.diff])),
                sel(1),
            ),
            Line::from(""),
            Line::from("  j/k pick row · h/l change · Enter play · q quit").fg(theme::MUTED),
            Line::from(""),
        ];
        if !self.recent.is_empty() {
            lines.push(Line::from("Recent").fg(theme::ACCENT).bold());
            for (idx, (key, pr)) in self.recent.iter().enumerate() {
                let status = if pr.solved { "solved" } else { "in progress" };
                lines.push(Line::styled(format!("  {key}  ({status})"), sel(idx + 2)));
            }
        }
        Paragraph::new(lines)
            .block(Block::bordered().title(" Logic Grid "))
            .render(area, buf);
    }
}
