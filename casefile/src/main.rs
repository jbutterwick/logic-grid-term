//! Terminal entry point. All screen logic lives in `casefile::ui`; this file only maps
//! crossterm events to `ui::Key` and drives the draw loop.

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use std::time::{SystemTime, UNIX_EPOCH};

    use casefile::ui::{App, Key};
    use casefile::{Game, Level, Settings};
    use clap::Parser;
    use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

    #[derive(Parser)]
    #[command(version, about)]
    struct Cli {
        /// Case seed; nanoseconds since the epoch if omitted.
        #[arg(long)]
        seed: Option<u64>,
        /// easy | medium | hard
        #[arg(long)]
        level: Option<Level>,
        /// Theme name (case-insensitive); random if omitted.
        #[arg(long)]
        theme: Option<String>,
        /// Profanity, harsher crimes, innuendo; nothing explicit.
        #[arg(long)]
        adult: bool,
    }

    let cli = Cli::parse();
    let seed = cli.seed.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1)
    });
    let skip_setup = cli.seed.is_some() || cli.level.is_some() || cli.theme.is_some() || cli.adult;
    let theme = cli.theme.as_deref().map(|name| {
        casefile::theme_names()
            .iter()
            .position(|t| t.eq_ignore_ascii_case(name))
            .unwrap_or_else(|| {
                eprintln!(
                    "unknown theme {name:?}; known: {}",
                    casefile::theme_names().join(", ")
                );
                std::process::exit(2)
            })
    });
    let mut app = if skip_setup {
        App::with_game(Game::new(Settings {
            seed,
            level: cli.level.unwrap_or(Level::Easy),
            theme,
            adult: cli.adult,
        }))
    } else {
        App::new(seed)
    };

    // ratatui::init installs a panic hook that restores the terminal.
    let mut terminal = ratatui::init();
    while !app.quit() {
        terminal
            .draw(|f| app.render(f.area(), f.buffer_mut()))
            .expect("draw");
        let Ok(Event::Key(k)) = event::read() else {
            continue;
        };
        if k.kind == KeyEventKind::Release {
            continue;
        }
        if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('c') {
            break;
        }
        let key = match k.code {
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Enter => Key::Enter,
            KeyCode::Esc => Key::Esc,
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Tab => Key::Tab,
            KeyCode::Char(c) => Key::Char(c),
            _ => continue,
        };
        app.on_key(key);
    }
    ratatui::restore();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use std::cell::RefCell;
    use std::rc::Rc;

    use casefile::ui::{App, Key};
    use ratzilla::event::KeyCode;
    use ratzilla::{DomBackend, WebRenderer};

    let app = Rc::new(RefCell::new(App::new(js_sys::Date::now() as u64)));
    let backend = DomBackend::new().expect("dom backend");
    let mut terminal = ratatui::Terminal::new(backend).expect("terminal");
    terminal
        .on_key_event({
            let app = app.clone();
            move |k| {
                let key = match k.code {
                    KeyCode::Up => Key::Up,
                    KeyCode::Down => Key::Down,
                    KeyCode::Left => Key::Left,
                    KeyCode::Right => Key::Right,
                    KeyCode::Enter => Key::Enter,
                    KeyCode::Esc => Key::Esc,
                    KeyCode::Backspace => Key::Backspace,
                    KeyCode::Tab => Key::Tab,
                    KeyCode::Char(c) => Key::Char(c),
                    _ => return,
                };
                app.borrow_mut().on_key(key);
            }
        })
        .expect("key handler");
    // ponytail: quit() is ignored on the web; there is no terminal to give back.
    terminal.draw_web(move |f| app.borrow().render(f.area(), f.buffer_mut()));
}
