//! Backend-agnostic ratatui screens (D2.1). Everything renders into a [`Buffer`] and reacts to a
//! [`Key`], so a terminal or Ratzilla host only has to map its own events and draw.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use crate::{Game, Level, Settings};

mod screens;
#[cfg(test)]
mod tests;

/// A key press, already stripped of backend detail.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(missing_docs)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Esc,
    Backspace,
    Tab,
    Char(char),
}

/// Which screen owns the keys and the frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Screen {
    Setup,
    Inbox,
    Thread,
    Compose,
    Notepad,
    Accuse,
}

/// The whole frontend: one game (after setup) plus per-screen cursors.
pub struct App {
    pub(crate) screen: Screen,
    pub(crate) help: bool,
    pub(crate) quit: bool,
    pub(crate) game: Option<Game>,
    // setup
    pub(crate) seed: u64,
    pub(crate) setup_row: usize,
    pub(crate) level: Level,
    pub(crate) theme: Option<usize>,
    pub(crate) adult: bool,
    pub(crate) seed_text: String,
    // inbox / thread / compose: 0 = chief, i + 1 = suspect i
    pub(crate) inbox_sel: usize,
    pub(crate) scroll: u16,
    pub(crate) compose_sel: usize,
    // notepad
    pub(crate) pad_row: usize,
    pub(crate) pad_col: usize,
    pub(crate) pad_notes: bool,
    // accuse
    pub(crate) accuse_step: usize,
    pub(crate) accuse_suspect: usize,
    pub(crate) accuse_item: usize,
}

pub(crate) const HILITE: Style = Style::new().add_modifier(Modifier::REVERSED);
pub(crate) const MUTED: Style = Style::new().fg(Color::DarkGray);
pub(crate) const BOLD: Style = Style::new().add_modifier(Modifier::BOLD);
pub(crate) const OUTGOING: Style = Style::new().fg(Color::Cyan);

impl App {
    /// Start at the setup screen; `seed` seeds the first case unless the player types one.
    pub fn new(seed: u64) -> App {
        App {
            screen: Screen::Setup,
            help: false,
            quit: false,
            game: None,
            seed,
            setup_row: 0,
            level: Level::Easy,
            theme: None,
            adult: false,
            seed_text: String::new(),
            inbox_sel: 0,
            scroll: 0,
            compose_sel: 0,
            pad_row: 0,
            pad_col: 0,
            pad_notes: false,
            accuse_step: 0,
            accuse_suspect: 0,
            accuse_item: 0,
        }
    }

    /// Skip setup (CLI flags).
    pub fn with_game(game: Game) -> App {
        let mut app = App::new(game.case.settings.seed);
        app.game = Some(game);
        app.screen = Screen::Inbox;
        app
    }

    /// Feed one key press to whichever screen is active.
    pub fn on_key(&mut self, key: Key) {
        if self.help {
            self.help = false;
            return;
        }
        match self.screen {
            Screen::Setup => screens::setup::on_key(self, key),
            Screen::Inbox => screens::inbox::on_key(self, key),
            Screen::Thread => screens::thread::on_key(self, key),
            Screen::Compose => screens::compose::on_key(self, key),
            Screen::Notepad => screens::notepad::on_key(self, key),
            Screen::Accuse => screens::accuse::on_key(self, key),
        }
    }

    /// Draw the active screen (and the help overlay, if open) into `buf`.
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        match self.screen {
            Screen::Setup => screens::setup::render(self, area, buf),
            Screen::Inbox => screens::inbox::render(self, area, buf),
            Screen::Thread => screens::thread::render(self, area, buf),
            Screen::Compose => screens::compose::render(self, area, buf),
            Screen::Notepad => screens::notepad::render(self, area, buf),
            Screen::Accuse => screens::accuse::render(self, area, buf),
        }
        if self.help {
            screens::help::render(self, area, buf);
        }
    }

    /// The player asked to leave.
    pub fn quit(&self) -> bool {
        self.quit
    }

    /// Build the game from the setup rows and open the inbox.
    pub(crate) fn start(&mut self) {
        let seed = self.seed_text.parse().unwrap_or(self.seed);
        self.game = Some(Game::new(Settings {
            seed,
            level: self.level,
            theme: self.theme,
            adult: self.adult,
        }));
        self.screen = Screen::Inbox;
    }

    /// The game; every screen after setup has one.
    pub(crate) fn game(&self) -> &Game {
        self.game.as_ref().expect("game exists after setup")
    }

    pub(crate) fn game_mut(&mut self) -> &mut Game {
        self.game.as_mut().expect("game exists after setup")
    }

    /// Selected inbox thread: `None` = chief.
    pub(crate) fn thread(&self) -> Option<usize> {
        self.inbox_sel.checked_sub(1)
    }
}

/// Move a list cursor by one, clamped to `len`.
pub(crate) fn step(sel: &mut usize, key: Key, len: usize) {
    match key {
        Key::Up | Key::Char('k') => *sel = sel.saturating_sub(1),
        Key::Down | Key::Char('j') => *sel = (*sel + 1).min(len.saturating_sub(1)),
        _ => {}
    }
}
