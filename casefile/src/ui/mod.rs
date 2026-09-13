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

/// What kind of on-screen keyboard a screen wants, for hosts that have to summon one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextInput {
    /// Free text: the notepad's notes.
    Text,
    /// Digits only: the seed on setup.
    Digits,
}

/// The whole frontend: one game (after setup) plus per-screen cursors.
pub struct App {
    pub(crate) screen: Screen,
    pub(crate) help: bool,
    pub(crate) quit: bool,
    pub(crate) game: Option<Game>,
    /// A saved game the host found, offered as a Resume row on setup until taken.
    pub(crate) saved: Option<Game>,
    /// Palette every screen paints from.
    pub(crate) look: Theme,
    /// Frames drawn so far; seeds the static.
    pub(crate) frame: u64,
    /// The static reseeds every this many frames.
    pub(crate) static_every: u64,
    /// Static on or off.
    pub(crate) fx: bool,
    /// Whether the key legend is drawn along the bottom; off when a host shows soft keys.
    pub(crate) legend: bool,
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
            saved: None,
            look: Theme::default(),
            frame: 0,
            static_every: 6,
            fx: true,
            legend: true,
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

    /// Show or hide the key legend along the bottom. A host with labelled on-screen keys
    /// turns it off and gets the rows back.
    pub fn set_legend(&mut self, on: bool) {
        self.legend = on;
    }

    /// Whether the key legend is drawn.
    pub fn legend(&self) -> bool {
        self.legend
    }

    /// Turn the static on or off. On the web this also drives the CRT glass, so off is
    /// the plain terminal look.
    pub fn set_fx(&mut self, on: bool) {
        self.fx = on;
    }

    /// Whether the static (and, on the web, the glass) is on.
    pub fn fx(&self) -> bool {
        self.fx
    }

    /// Count `frames` drawn. The static reseeds every [`App::set_static_every`] frames
    /// (six by default), so a host that redraws ten times a second passes 6.
    pub fn advance(&mut self, frames: u64) {
        self.frame = self.frame.wrapping_add(frames);
    }

    /// How many frames the static holds still for. A host whose repaints are expensive
    /// (a browser with a CRT filter over the grid) wants this high.
    pub fn set_static_every(&mut self, frames: u64) {
        self.static_every = frames.max(1);
    }

    /// The static's current seed; a host only needs to redraw when this or the input changes.
    pub fn static_seed(&self) -> u64 {
        if self.fx {
            self.frame / self.static_every
        } else {
            0
        }
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
                ("FX", Char('e')),
                ("?", Char('?')),
            ],
            Screen::Thread => vec![
                ("▲", Up),
                ("▼", Down),
                ("COMPOSE", Char('c')),
                ("BACK", Esc),
            ],
            Screen::Compose => vec![("▲", Up), ("▼", Down), ("SEND", Enter), ("BACK", Esc)],
            Screen::Notepad if self.pad_notes => vec![("GRID", Tab), ("BACK", Esc)],
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

    /// Offer a saved game (its JSON, as [`App::snapshot`] gave it) on the setup screen.
    /// Malformed JSON is ignored and reported.
    pub fn set_saved(&mut self, json: &str) -> Result<(), String> {
        let game = Game::from_json(json).map_err(|e| e.to_string())?;
        self.saved = Some(game);
        Ok(())
    }

    /// Whether a saved game is on offer.
    pub fn has_saved(&self) -> bool {
        self.saved.is_some()
    }

    /// Pick up the saved game where it left off, at the inbox.
    pub fn resume(&mut self) -> bool {
        let Some(game) = self.saved.take() else {
            return false;
        };
        self.game = Some(game);
        self.screen = Screen::Inbox;
        self.help = false;
        self.inbox_sel = 0;
        self.scroll = 0;
        true
    }

    /// The whole game as JSON, for a host to keep between runs; `None` before a case opens.
    /// Everything that matters is in it: mail, the notepad grid and notes, tick and rank.
    pub fn snapshot(&self) -> Option<String> {
        self.game.as_ref().map(Game::to_json)
    }

    /// The keyboard the active screen wants, if a host has to bring one up.
    pub fn wants_text_input(&self) -> Option<TextInput> {
        if self.help {
            return None;
        }
        match self.screen {
            Screen::Notepad if self.pad_notes => Some(TextInput::Text),
            Screen::Setup if screens::setup::on_seed_row(self) => Some(TextInput::Digits),
            _ => None,
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
