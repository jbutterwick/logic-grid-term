use std::io;
use std::path::PathBuf;

use logicgrid::{Difficulty, Puzzle};
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind, MouseButton, MouseEventKind};

use crate::screens::home::HomeScreen;
use crate::screens::play::PlayScreen;
use crate::storage;

/// What a screen asks the [`App`] to do after a key press.
pub enum Transition {
    None,
    Quit,
    Home,
    /// Play `puzzle`, autosaving under `label` (difficulty name, "file", ...).
    Play(Puzzle, String),
}

enum Screen {
    Home(HomeScreen),
    Play(Box<PlayScreen>),
}

pub struct App {
    screen: Screen,
    dir: PathBuf,
}

pub fn diff_name(d: Difficulty) -> String {
    format!("{d:?}").to_lowercase()
}

impl App {
    pub fn new() -> Self {
        let dir = storage::dir();
        App {
            screen: Screen::Home(HomeScreen::new(&dir)),
            dir,
        }
    }

    pub fn with_puzzle(puzzle: Puzzle, label: String) -> Self {
        let dir = storage::dir();
        App {
            screen: Screen::Play(Box::new(PlayScreen::new(puzzle, label, dir.clone()))),
            dir,
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|f| match &mut self.screen {
                Screen::Home(s) => s.render(f.area(), f.buffer_mut()),
                Screen::Play(s) => s.render(f.area(), f.buffer_mut()),
            })?;
            let key = match event::read()? {
                Event::Key(key) => key,
                Event::Mouse(me) if me.kind == MouseEventKind::Down(MouseButton::Left) => {
                    if let Screen::Play(s) = &mut self.screen {
                        s.on_click(me.column, me.row);
                    }
                    continue;
                }
                _ => continue,
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            let t = match &mut self.screen {
                Screen::Home(s) => s.on_key(key),
                Screen::Play(s) => s.on_key(key),
            };
            match t {
                Transition::None => {}
                Transition::Quit => return Ok(()),
                Transition::Home => self.screen = Screen::Home(HomeScreen::new(&self.dir)),
                Transition::Play(p, label) => {
                    self.screen =
                        Screen::Play(Box::new(PlayScreen::new(p, label, self.dir.clone())));
                }
            }
        }
    }
}
