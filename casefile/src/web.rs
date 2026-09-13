//! Browser host. A DOM grid sized from the real font metrics (one `<span>` per cell), a
//! requestAnimationFrame draw loop, keys read off the document, and an on-screen key bar for
//! touch screens. The page around it (`index.html`) supplies the CRT glass.

use std::cell::{Cell as Flag, RefCell};
use std::io;
use std::rc::Rc;

use ratatui::Terminal;
use ratatui::backend::{Backend, ClearType, WindowSize};
use ratatui::buffer::Cell;
use ratatui::layout::{Position, Size};
use ratatui::style::{Color, Modifier};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, Event, HtmlTextAreaElement, InputEvent, KeyboardEvent};

use crate::ui::{App, Key, Phosphor, TextInput};
use crate::{Game, Level, Settings};

/// The requestAnimationFrame callback, shared with itself so it can ask for the next frame.
type FrameLoop = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

/// A grid of `<span>`s inside `#screen`, one row per `<div>`.
pub struct DomGrid {
    document: Document,
    screen: Element,
    cells: Vec<Element>,
    /// What each span last showed, so unchanged text and style cost no DOM write.
    shown: Vec<(String, String)>,
    size: Size,
    cell_px: (f64, f64),
    bg: Color,
    dirty: Rc<Flag<bool>>,
    _resize: Closure<dyn FnMut(Event)>,
}

impl DomGrid {
    /// Build the grid inside the element with id `screen`; re-measures on every window resize.
    pub fn new(document: Document, screen: Element) -> Result<DomGrid, JsValue> {
        let dirty = Rc::new(Flag::new(true));
        let flag = dirty.clone();
        let resize =
            Closure::wrap(Box::new(move |_: Event| flag.set(true)) as Box<dyn FnMut(Event)>);
        web_sys::window()
            .ok_or("no window")?
            .add_event_listener_with_callback("resize", resize.as_ref().unchecked_ref())?;
        let mut grid = DomGrid {
            document,
            screen,
            cells: Vec::new(),
            shown: Vec::new(),
            size: Size::new(0, 0),
            cell_px: (10.0, 20.0),
            bg: Color::Reset,
            dirty,
            _resize: resize,
        };
        grid.refresh(true);
        Ok(grid)
    }

    /// Re-measure on the next [`DomGrid::refresh`], as after the page around the grid moved.
    pub fn mark_dirty(&self) {
        self.dirty.set(true);
    }

    /// The background every cell shares; cells painted in it carry no background of their own.
    pub fn set_bg(&mut self, bg: Color) {
        self.bg = bg;
    }

    /// Measure one cell, recompute the grid, rebuild the spans if it changed. Cheap when
    /// nothing happened: only a resize, or every `probe`th call, pays for the measurement.
    /// The probe catches what no event announces, like the webfont landing after first paint
    /// or a stylesheet changing the font.
    /// Returns true when the grid was rebuilt and needs a full draw.
    pub fn refresh(&mut self, probe: bool) -> bool {
        if !self.dirty.replace(false) && !probe {
            return false;
        }
        if let Some(px) = self.measure() {
            self.cell_px = px;
        }
        let rect = self.screen.get_bounding_client_rect();
        let size = Size::new(
            (rect.width() / self.cell_px.0).floor().max(1.0) as u16,
            (rect.height() / self.cell_px.1).floor().max(1.0) as u16,
        );
        if size != self.size || self.cells.is_empty() {
            self.size = size;
            self.rebuild();
            return true;
        }
        false
    }

    fn measure(&self) -> Option<(f64, f64)> {
        let row = self.document.create_element("div").ok()?;
        row.set_class_name("row");
        let span = self.document.create_element("span").ok()?;
        span.set_text_content(Some("█"));
        row.append_child(&span).ok()?;
        self.screen.append_child(&row).ok()?;
        let w = span.get_bounding_client_rect().width();
        let h = row.get_bounding_client_rect().height();
        let _ = self.screen.remove_child(&row);
        (w > 0.0 && h > 0.0).then_some((w, h))
    }

    fn rebuild(&mut self) {
        self.screen.set_inner_html("");
        self.cells.clear();
        self.shown.clear();
        for _ in 0..self.size.height {
            let Ok(row) = self.document.create_element("div") else {
                return;
            };
            row.set_class_name("row");
            for _ in 0..self.size.width {
                let Ok(span) = self.document.create_element("span") else {
                    return;
                };
                span.set_text_content(Some(" "));
                let _ = row.append_child(&span);
                self.cells.push(span);
                self.shown.push((" ".into(), String::new()));
            }
            let _ = self.screen.append_child(&row);
        }
    }
}

/// Inline style for a cell; empty for plain text in the shared background.
fn css(cell: &Cell, shared_bg: Color) -> String {
    let mut css = String::new();
    if let Color::Rgb(r, g, b) = cell.fg {
        css.push_str(&format!("color:rgb({r},{g},{b});"));
    }
    if let Color::Rgb(r, g, b) = cell.bg
        && cell.bg != shared_bg
    {
        css.push_str(&format!("background:rgb({r},{g},{b});"));
    }
    if cell.modifier.contains(Modifier::BOLD) {
        css.push_str("font-weight:bold;");
    }
    css
}

impl Backend for DomGrid {
    type Error = io::Error;

    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        let w = usize::from(self.size.width);
        for (x, y, cell) in content {
            let i = usize::from(y) * w + usize::from(x);
            let (Some(el), Some(shown)) = (self.cells.get(i), self.shown.get_mut(i)) else {
                continue;
            };
            if shown.0 != cell.symbol() {
                el.set_text_content(Some(cell.symbol()));
                shown.0 = cell.symbol().to_string();
            }
            let css = css(cell, self.bg);
            if shown.1 != css {
                if css.is_empty() {
                    let _ = el.remove_attribute("style");
                } else {
                    let _ = el.set_attribute("style", &css);
                }
                // Selection blocks glow as one piece; the class is cheaper than a style query.
                let _ = if css.contains("background") {
                    el.set_attribute("class", "inv")
                } else {
                    el.remove_attribute("class")
                };
                shown.1 = css;
            }
        }
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        Ok(Position::ORIGIN)
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, _: P) -> io::Result<()> {
        Ok(())
    }

    fn clear(&mut self) -> io::Result<()> {
        for (el, shown) in self.cells.iter().zip(&mut self.shown) {
            el.set_text_content(Some(" "));
            let _ = el.remove_attribute("style");
            let _ = el.remove_attribute("class");
            *shown = (" ".into(), String::new());
        }
        Ok(())
    }

    fn clear_region(&mut self, clear_type: ClearType) -> io::Result<()> {
        if clear_type == ClearType::All {
            self.clear()?;
        }
        Ok(())
    }

    fn size(&self) -> io::Result<Size> {
        Ok(self.size)
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        Ok(WindowSize {
            columns_rows: self.size,
            pixels: Size::new(
                (f64::from(self.size.width) * self.cell_px.0) as u16,
                (f64::from(self.size.height) * self.cell_px.1) as u16,
            ),
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A browser key name (`event.key`, or a soft key's `data-key`) as a [`Key`].
fn key_from_name(name: &str) -> Option<Key> {
    Some(match name {
        "ArrowUp" | "Up" => Key::Up,
        "ArrowDown" | "Down" => Key::Down,
        "ArrowLeft" | "Left" => Key::Left,
        "ArrowRight" | "Right" => Key::Right,
        "Enter" => Key::Enter,
        "Escape" | "Esc" => Key::Esc,
        "Backspace" => Key::Backspace,
        "Tab" => Key::Tab,
        _ => Key::Char(name.chars().next().filter(|_| name.chars().count() == 1)?),
    })
}

/// The `data-key` name for a soft key.
fn key_name(key: Key) -> String {
    match key {
        Key::Up => "Up".into(),
        Key::Down => "Down".into(),
        Key::Left => "Left".into(),
        Key::Right => "Right".into(),
        Key::Enter => "Enter".into(),
        Key::Esc => "Esc".into(),
        Key::Backspace => "Backspace".into(),
        Key::Tab => "Tab".into(),
        Key::Char(c) => c.to_string(),
    }
}

fn hex(c: Color) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        _ => "inherit".into(),
    }
}

/// Zero-width space kept in the keyboard textarea so a backspace has something to delete
/// and therefore fires an input event.
const SENTINEL: &str = "\u{200b}";

/// Focus or release the keyboard textarea to match what the screen wants. Called from
/// input handlers, which count as user gestures, the only place a phone will open its
/// keyboard from.
fn sync_keyboard(app: &App, kbd: &HtmlTextAreaElement, document: &Document) {
    let active = document.active_element().is_some_and(|el| el.id() == "kbd");
    match app.wants_text_input() {
        Some(kind) => {
            let mode = match kind {
                TextInput::Text => "text",
                TextInput::Digits => "numeric",
            };
            let _ = kbd.set_attribute("inputmode", mode);
            if !active {
                kbd.set_value(SENTINEL);
                let _ = kbd.focus();
            }
        }
        None if active => {
            let _ = kbd.blur();
        }
        None => {}
    }
}

/// Run the game in the page: `#screen` holds the grid, `#keys` the soft-key bar, `#kbd`
/// a hidden textarea that brings up a phone's keyboard when the notes or the seed take
/// typing.
///
/// Query parameters mirror the command line. `seed`, `level`, `theme` and `adult` skip the
/// setup screen and open that case (a shareable link); `phosphor=green|amber|white` picks
/// the palette; `fx=off` stills the glass; `keys=on|off` forces the soft-key bar.
pub fn run(mut app: App) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let screen = document.get_element_by_id("screen").ok_or("no #screen")?;
    let keys = document.get_element_by_id("keys").ok_or("no #keys")?;
    let kbd: HtmlTextAreaElement = document
        .get_element_by_id("kbd")
        .ok_or("no #kbd")?
        .dyn_into()?;

    // Remembered look, then the link's own parameters on top.
    let storage = window.local_storage().ok().flatten();
    let stored = |key: &str| {
        storage
            .as_ref()
            .and_then(|s| s.get_item(key).ok().flatten())
    };
    if let Some(p) = stored("casefile.phosphor").and_then(|p| p.parse::<Phosphor>().ok()) {
        app.set_phosphor(p);
    }
    if stored("casefile.fx").as_deref() == Some("off") {
        app.set_fx(false);
    }
    // The autosave: offered as Resume on setup, or taken straight away with ?resume.
    let mut last_saved = stored("casefile.save");
    if let Some(json) = &last_saved
        && app.set_saved(json).is_err()
    {
        last_saved = None;
    }
    let mut classes: Vec<&str> = Vec::new();
    if let Ok(params) = web_sys::UrlSearchParams::new_with_str(&window.location().search()?) {
        let seed = params.get("seed").and_then(|s| s.parse::<u64>().ok());
        let level = params.get("level").and_then(|l| l.parse::<Level>().ok());
        let theme = params.get("theme").and_then(|name| {
            crate::theme_names()
                .iter()
                .position(|t| t.eq_ignore_ascii_case(&name))
        });
        let adult = params.get("adult").is_some_and(|a| a != "off" && a != "0");
        if seed.is_some() || level.is_some() || theme.is_some() || adult {
            let (phosphor, fx) = (app.phosphor(), app.fx());
            app = App::with_game(Game::new(Settings {
                seed: seed.unwrap_or_else(|| js_sys::Date::now() as u64),
                level: level.unwrap_or(Level::Easy),
                theme,
                adult,
            }));
            app.set_phosphor(phosphor);
            app.set_fx(fx);
        }
        if let Some(p) = params
            .get("phosphor")
            .and_then(|p| p.parse::<Phosphor>().ok())
        {
            app.set_phosphor(p);
        }
        match params.get("fx").as_deref() {
            Some("off") => app.set_fx(false),
            Some("on") => app.set_fx(true),
            _ => {}
        }
        if params.get("resume").is_some() {
            app.resume();
        }
        match params.get("keys").as_deref() {
            Some("on") => classes.push("keys"),
            Some("off") => classes.push("nokeys"),
            _ => {}
        }
    }
    let body = document.body().ok_or("no body")?;
    let base_classes = classes.join(" ");

    // A browser repaint runs the CRT filter over the whole tube, so hold the static longer
    // than the terminal does: four reseeds a second at 60 Hz.
    app.set_static_every(15);
    let app = Rc::new(RefCell::new(app));
    let input = Rc::new(Flag::new(true));
    let mut terminal = Terminal::new(DomGrid::new(document.clone(), screen)?)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Physical keyboard: anything the browser itself does not own.
    let keydown = Closure::wrap(Box::new({
        let app = app.clone();
        let input = input.clone();
        let kbd = kbd.clone();
        let document = document.clone();
        move |e: KeyboardEvent| {
            if e.ctrl_key() || e.meta_key() || e.alt_key() {
                return;
            }
            let Some(key) = key_from_name(&e.key()) else {
                return;
            };
            e.prevent_default();
            let mut app = app.borrow_mut();
            app.on_key(key);
            sync_keyboard(&app, &kbd, &document);
            input.set(true);
        }
    }) as Box<dyn FnMut(KeyboardEvent)>);
    document.add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())?;
    keydown.forget();

    // Soft keys: one listener on the bar, buttons carry `data-key`.
    let click = Closure::wrap(Box::new({
        let app = app.clone();
        let input = input.clone();
        let kbd = kbd.clone();
        let document = document.clone();
        move |e: Event| {
            let name = e
                .target()
                .and_then(|t| t.dyn_into::<Element>().ok())
                .and_then(|el| el.closest("[data-key]").ok().flatten())
                .and_then(|el| el.get_attribute("data-key"));
            let Some(name) = name else {
                return;
            };
            let mut app = app.borrow_mut();
            // "kbd" is the host's own button: bring the keyboard back after it was dismissed.
            if name != "kbd"
                && let Some(key) = key_from_name(&name)
            {
                app.on_key(key);
                input.set(true);
            }
            sync_keyboard(&app, &kbd, &document);
        }
    }) as Box<dyn FnMut(Event)>);
    keys.add_event_listener_with_callback("click", click.as_ref().unchecked_ref())?;
    click.forget();

    // Typing into the hidden textarea, including phone keyboards that compose words:
    // whatever changed since the last event becomes backspaces and characters.
    let typed = Closure::wrap(Box::new({
        let app = app.clone();
        let input = input.clone();
        let kbd = kbd.clone();
        let mut tracked = SENTINEL.to_string();
        move |e: Event| {
            let now = kbd.value();
            let common = tracked
                .chars()
                .zip(now.chars())
                .take_while(|(a, b)| a == b)
                .count();
            let removed = tracked.chars().count() - common;
            let added: String = now.chars().skip(common).collect();
            let mut app = app.borrow_mut();
            for _ in 0..removed {
                app.on_key(Key::Backspace);
            }
            for c in added.chars() {
                app.on_key(if c == '\n' || c == '\r' {
                    Key::Enter
                } else {
                    Key::Char(c)
                });
            }
            input.set(true);
            let composing = e
                .dyn_ref::<InputEvent>()
                .is_some_and(InputEvent::is_composing);
            if composing {
                tracked = now;
            } else {
                kbd.set_value(SENTINEL);
                tracked = SENTINEL.to_string();
            }
        }
    }) as Box<dyn FnMut(Event)>);
    kbd.add_event_listener_with_callback("input", typed.as_ref().unchecked_ref())?;
    typed.forget();

    // Draw loop: 60 Hz, but the grid is only re-rendered after input, a static reseed, a
    // palette change, or a resize; an idle frame costs a few comparisons.
    let mut last_keys: (Vec<&'static str>, bool) = (Vec::new(), false);
    let mut last_phosphor: Option<Phosphor> = None;
    let mut last_fx: Option<bool> = None;
    let mut last_seed: Option<u64> = None;
    let mut bar_h: f64 = -1.0;
    let root = document.document_element().ok_or("no root")?;
    let root: web_sys::HtmlElement = root.dyn_into()?;
    let frame: FrameLoop = Rc::new(RefCell::new(None));
    let next = frame.clone();
    *frame.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut redraw = input.replace(false);
        {
            let mut app = app.borrow_mut();
            app.advance(1);
            if last_seed != Some(app.static_seed()) {
                last_seed = Some(app.static_seed());
                redraw = true;
            }
            if last_fx != Some(app.fx()) {
                last_fx = Some(app.fx());
                redraw = true;
                // Off is the terminal look: no glass, no filter, no glow.
                let plain = if app.fx() { "" } else { " plain" };
                body.set_class_name(&format!("{base_classes}{plain}"));
                if let Some(s) = &storage {
                    let _ = s.set_item("casefile.fx", if app.fx() { "on" } else { "off" });
                }
            }
            if last_phosphor != Some(app.phosphor()) {
                last_phosphor = Some(app.phosphor());
                redraw = true;
                if let Some(s) = &storage {
                    let _ = s.set_item("casefile.phosphor", app.phosphor().name());
                }
                let t = app.theme();
                let style = root.style();
                for (name, color) in [
                    ("--bg", t.bg),
                    ("--fg", t.fg),
                    ("--dim", t.dim),
                    ("--bright", t.bright),
                ] {
                    let _ = style.set_property(name, &hex(color));
                }
                terminal.backend_mut().set_bg(t.bg);
            }
            let soft = app.soft_keys();
            let typing = app.wants_text_input().is_some();
            let labels: Vec<&'static str> = soft.iter().map(|(l, _)| *l).collect();
            if (labels.as_slice(), typing) != (last_keys.0.as_slice(), last_keys.1) {
                last_keys = (labels, typing);
                let mut html: String = soft
                    .iter()
                    .map(|(label, key)| {
                        let class = if label.chars().count() == 1 {
                            " class=\"arrow\""
                        } else {
                            ""
                        };
                        format!(
                            "<button type=\"button\"{class} data-key=\"{}\">{label}</button>",
                            key_name(*key)
                        )
                    })
                    .collect();
                if typing {
                    html.push_str("<button type=\"button\" data-key=\"kbd\">KEYBOARD</button>");
                }
                keys.set_inner_html(&html);
                bar_h = -1.0;
            }
            // Autosave after anything changed the game.
            if redraw
                && let Some(json) = app.snapshot()
                && last_saved.as_deref() != Some(json.as_str())
            {
                if let Some(s) = &storage {
                    let _ = s.set_item("casefile.save", &json);
                }
                last_saved = Some(json);
            }
        }
        let probe = app.borrow().frame().is_multiple_of(30);
        if terminal.backend_mut().refresh(probe) {
            redraw = true;
        }
        if probe || bar_h < 0.0 {
            // The bar wraps to however many rows it needs, and it comes and goes with the
            // media query; give the tube whatever is left, and drop the legend while the
            // labelled keys are on screen.
            let h = keys.get_bounding_client_rect().height();
            if h != bar_h {
                bar_h = h;
                let _ = root.style().set_property("--keys-h", &format!("{h}px"));
                terminal.backend_mut().mark_dirty();
                if terminal.backend_mut().refresh(true) {
                    redraw = true;
                }
            }
            let mut app = app.borrow_mut();
            let soft_visible = h > 0.0;
            if app.legend() == soft_visible {
                app.set_legend(!soft_visible);
                redraw = true;
            }
        }
        if redraw {
            let app = app.borrow();
            let _ = terminal.draw(|f| app.render(f.area(), f.buffer_mut()));
        }
        if let Some(cb) = next.borrow().as_ref() {
            let _ =
                web_sys::window().map(|w| w.request_animation_frame(cb.as_ref().unchecked_ref()));
        }
    }) as Box<dyn FnMut()>));
    if let Some(cb) = frame.borrow().as_ref() {
        window.request_animation_frame(cb.as_ref().unchecked_ref())?;
    }
    Ok(())
}
