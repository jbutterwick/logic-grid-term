//! Backend-agnostic ratatui screens (D2.1). Everything renders into a [`Buffer`] and reacts to a
//! [`Key`], so a terminal or Ratzilla host only has to map its own events and draw.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::{Game, Level, Settings};

mod chrome;
mod screens;
#[cfg(test)]
mod tests;
mod theme;

pub use theme::{Phosphor, Theme};

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
    /// Palette every screen paints from.
    pub(crate) look: Theme,
    /// Frames drawn so far; seeds the static.
    pub(crate) frame: u64,
    /// Static on or off.
    pub(crate) fx: bool,
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

impl App {
    /// Start at the setup screen; `seed` seeds the first case unless the player types one.
    pub fn new(seed: u64) -> App {
        App {
            screen: Screen::Setup,
            help: false,
            quit: false,
            game: None,
            look: Theme::default(),
            frame: 0,
            fx: true,
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

    /// Swap the phosphor; takes effect on the next draw.
    pub fn set_phosphor(&mut self, phosphor: Phosphor) {
        self.look = Theme::new(phosphor);
    }

    /// The phosphor in use.
    pub fn phosphor(&self) -> Phosphor {
        self.look.phosphor
    }

    /// The palette in use, for a host that paints outside the grid.
    pub fn theme(&self) -> &Theme {
        &self.look
    }

    /// Turn the static on or off.
    pub fn set_fx(&mut self, on: bool) {
        self.fx = on;
    }

    /// Count `frames` drawn; the static reseeds every sixth frame, so a host that redraws
    /// ten times a second should pass 6 and one that redraws at 60 Hz should pass 1.
    pub fn advance(&mut self, frames: u64) {
        self.frame = self.frame.wrapping_add(frames);
    }

    /// Frames drawn so far.
    pub fn frame(&self) -> u64 {
        self.frame
    }

    /// On-screen buttons for a host without a keyboard: label and the key each one sends.
    pub fn soft_keys(&self) -> Vec<(&'static str, Key)> {
        use Key::*;
        if self.help {
            return vec![("CLOSE", Esc)];
        }
        match self.screen {
            Screen::Setup => vec![
                ("▲", Up),
                ("▼", Down),
                ("◀", Left),
                ("▶", Right),
                ("START", Enter),
            ],
            Screen::Inbox => vec![
                ("▲", Up),
                ("▼", Down),
                ("OPEN", Enter),
                ("WAIT", Char('w')),
                ("NOTEPAD", Char('n')),
                ("ACCUSE", Char('a')),
                ("?", Char('?')),
            ],
            Screen::Thread => vec![
                ("▲", Up),
                ("▼", Down),
                ("COMPOSE", Char('c')),
                ("BACK", Esc),
            ],
            Screen::Compose => vec![("▲", Up), ("▼", Down), ("SEND", Enter), ("BACK", Esc)],
            Screen::Notepad => vec![
                ("▲", Up),
                ("▼", Down),
                ("◀", Left),
                ("▶", Right),
                ("MARK", Char(' ')),
                ("NOTES", Tab),
                ("BACK", Esc),
            ],
            Screen::Accuse => vec![
                ("▲", Up),
                ("▼", Down),
                ("NEXT", Enter),
                ("YES", Char('y')),
                ("NO", Char('n')),
                ("BACK", Esc),
            ],
        }
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
