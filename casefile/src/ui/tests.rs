use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::{App, Key, Phosphor, Screen};
use crate::{Entity, Game, Level, Mark, Settings};

const LEVELS: [Level; 3] = [Level::Easy, Level::Medium, Level::Hard];

fn game(level: Level) -> Game {
    Game::new(Settings {
        seed: 1,
        level,
        theme: None,
        adult: false,
    })
}

fn draw(app: &App, w: u16, h: u16) -> Buffer {
    let area = Rect::new(0, 0, w, h);
    let mut buf = Buffer::empty(area);
    app.render(area, &mut buf);
    buf
}

fn text(buf: &Buffer) -> String {
    let mut s = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            s.push_str(buf[(x, y)].symbol());
        }
        s.push('\n');
    }
    s
}

fn keys(app: &mut App, ks: &[Key]) {
    ks.iter().for_each(|&k| app.on_key(k));
}

#[test]
fn setup_to_inbox() {
    let mut app = App::new(42);
    assert_eq!(app.screen, Screen::Setup);
    draw(&app, 80, 24);
    // Level -> medium, theme -> first, adult on, screen -> amber, seed 7, start.
    keys(
        &mut app,
        &[Key::Right, Key::Down, Key::Right, Key::Down, Key::Right],
    );
    keys(&mut app, &[Key::Down, Key::Right]);
    assert_eq!(app.phosphor(), Phosphor::Amber);
    keys(&mut app, &[Key::Down, Key::Char('7'), Key::Char('x')]);
    draw(&app, 80, 24);
    keys(&mut app, &[Key::Down, Key::Enter]);
    assert_eq!(app.screen, Screen::Inbox);
    let s = &app.game().case.settings;
    assert_eq!(
        (s.seed, s.level, s.theme, s.adult),
        (7, Level::Medium, Some(0), true)
    );
    assert!(!app.quit());
}

#[test]
fn setup_blank_seed_uses_default_and_matches_with_game() {
    let mut app = App::new(1);
    keys(&mut app, &[Key::Down; 5]);
    app.on_key(Key::Enter);
    let direct = App::with_game(game(Level::Easy));
    assert_eq!(app.game().to_json(), direct.game().to_json());
    assert_eq!(direct.screen, Screen::Inbox);
}

#[test]
fn every_screen_renders_at_common_sizes() {
    for level in LEVELS {
        for (w, h) in [(80, 24), (120, 40), (48, 36)] {
            let mut app = App::with_game(game(level));
            let out = text(&draw(&app, w, h));
            assert!(
                out.contains(&app.game().case.frame.title),
                "{level:?} {w}x{h}"
            );
            let (done, total) = app.game().progress();
            assert!(
                out.contains(&format!("{done}/{total}")),
                "gauge missing at {level:?} {w}x{h}"
            );
            app.on_key(Key::Char('?'));
            draw(&app, w, h);
            app.on_key(Key::Enter);
            for k in [Key::Enter, Key::Down, Key::Char('j')] {
                app.on_key(k);
                draw(&app, w, h);
            }
            keys(&mut app, &[Key::Esc, Key::Down, Key::Enter, Key::Char('c')]);
            assert_eq!(app.screen, Screen::Compose);
            draw(&app, w, h);
            keys(&mut app, &[Key::Esc, Key::Esc, Key::Char('n')]);
            draw(&app, w, h);
            keys(
                &mut app,
                &[Key::Tab, Key::Char('h'), Key::Char('i'), Key::Enter],
            );
            draw(&app, w, h);
            keys(&mut app, &[Key::Esc, Key::Char('a')]);
            for _ in 0..3 {
                draw(&app, w, h);
                app.on_key(Key::Enter);
            }
            draw(&app, 5, 3);
        }
    }
}

#[test]
fn compose_sends_and_reply_lands_after_next_action() {
    let mut app = App::with_game(game(Level::Easy));
    keys(
        &mut app,
        &[Key::Down, Key::Enter, Key::Char('c'), Key::Enter],
    );
    assert_eq!(app.screen, Screen::Thread);
    let t = &app.game().threads[0];
    assert_eq!(t.len(), 1);
    assert!(t[0].outgoing);
    keys(&mut app, &[Key::Esc, Key::Char('w')]);
    let t = &app.game().threads[0];
    assert_eq!(t.len(), 2);
    assert!(!t[1].outgoing && !t[1].read);
    assert!(text(&draw(&app, 80, 24)).contains("1 new"));
    // The last menu row sends its label verbatim.
    keys(&mut app, &[Key::Enter, Key::Char('c')]);
    let qs = app.game().questions(0);
    keys(&mut app, &vec![Key::Down; qs.len()]);
    app.on_key(Key::Enter);
    let last = qs.last().unwrap().label(&app.game().case);
    assert_eq!(app.game().threads[0][2].body, last);
}

#[test]
fn silenced_suspect_has_no_compose() {
    let mut app = App::with_game(game(Level::Easy));
    let culprit = app.game().case.culprit;
    let wrong = (culprit + 1) % app.game().case.suspects.len();
    keys(&mut app, &[Key::Char('a')]);
    keys(&mut app, &vec![Key::Down; wrong]);
    keys(&mut app, &[Key::Enter, Key::Enter, Key::Char('y')]);
    assert_eq!(app.game().wrong, 1);
    assert!(text(&draw(&app, 80, 24)).contains("silent"));
    keys(&mut app, &vec![Key::Down; wrong + 1]);
    keys(&mut app, &[Key::Enter, Key::Char('c')]);
    let before = app.game().threads[wrong].len();
    app.on_key(Key::Enter);
    assert_eq!(app.game().threads[wrong].len(), before);
}

#[test]
fn notepad_cycles_grid_and_edits_notes() {
    let mut app = App::with_game(game(Level::Medium));
    app.on_key(Key::Char('n'));
    let a = Entity { cat: 0, item: 1 };
    let b = Entity { cat: 1, item: 2 };
    keys(&mut app, &[Key::Down, Key::Right, Key::Right]);
    let get = |app: &App| app.game().grid.get(a, b);
    app.on_key(Key::Char(' '));
    assert_eq!(get(&app), Mark::No);
    app.on_key(Key::Enter);
    assert_eq!(get(&app), Mark::Yes);
    app.on_key(Key::Char(' '));
    assert_eq!(get(&app), Mark::Unknown);
    app.on_key(Key::Char('o'));
    assert_eq!(get(&app), Mark::Yes);
    app.on_key(Key::Char('x'));
    assert_eq!(get(&app), Mark::No);
    app.on_key(Key::Backspace);
    assert_eq!(get(&app), Mark::Unknown);
    assert!(text(&draw(&app, 80, 24)).contains('·'));
    keys(
        &mut app,
        &[
            Key::Tab,
            Key::Char('a'),
            Key::Enter,
            Key::Char('b'),
            Key::Backspace,
        ],
    );
    assert_eq!(app.game().notes, "a\n");
    assert!(text(&draw(&app, 120, 40)).contains("editing"));
    keys(&mut app, &[Key::Esc]);
    assert_eq!(app.screen, Screen::Inbox);
}

#[test]
fn accuse_culprit_solves_and_shows_banner() {
    for level in LEVELS {
        let mut app = App::with_game(game(level));
        let (culprit, guilty) = (app.game().case.culprit, app.game().case.guilty);
        app.on_key(Key::Char('a'));
        keys(&mut app, &vec![Key::Down; culprit]);
        app.on_key(Key::Enter);
        keys(&mut app, &vec![Key::Down; guilty.item]);
        app.on_key(Key::Enter);
        assert!(text(&draw(&app, 80, 24)).contains("Step 3"));
        app.on_key(Key::Char('y'));
        assert_eq!(app.screen, Screen::Inbox);
        assert!(app.game().solved);
        let out = text(&draw(&app, 80, 24));
        assert!(out.contains("Case closed: Inspector"), "{level:?}\n{out}");
        let reply = app.game().chief.last().unwrap();
        assert!(!reply.outgoing && !reply.read);
    }
}

#[test]
fn quit_and_help() {
    let mut app = App::with_game(game(Level::Easy));
    app.on_key(Key::Char('?'));
    assert!(text(&draw(&app, 80, 24)).contains("any key closes"));
    app.on_key(Key::Char('q'));
    assert!(!app.quit(), "first key only closes help");
    app.on_key(Key::Char('q'));
    assert!(app.quit());
    let mut app = App::new(3);
    app.on_key(Key::Esc);
    assert!(app.quit());
}
