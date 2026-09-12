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
use web_sys::{Document, Element, Event, KeyboardEvent};

use crate::ui::{App, Key, Phosphor};
use crate::{Game, Level, Settings};

/// The requestAnimationFrame callback, shared with itself so it can ask for the next frame.
type FrameLoop = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

/// A grid of `<span>`s inside `#screen`, one row per `<div>`.
pub struct DomGrid {
    document: Document,
    screen: Element,
    cells: Vec<Element>,
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
            size: Size::new(0, 0),
            cell_px: (10.0, 20.0),
            bg: Color::Reset,
            dirty,
            _resize: resize,
        };
        grid.refresh(true);
        Ok(grid)
    }

    /// The background every cell shares; cells painted in it carry no background of their own.
    pub fn set_bg(&mut self, bg: Color) {
        self.bg = bg;
    }

    /// Measure one cell, recompute the grid, rebuild the spans if it changed. Cheap when
    /// nothing happened: only a resize, or every `probe`th call, pays for the measurement.
    /// The probe catches what no event announces, like the webfont landing after first paint
    /// or a stylesheet changing the font.
    pub fn refresh(&mut self, probe: bool) {
        if !self.dirty.replace(false) && !probe {
            return;
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
        }
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
            }
            let _ = self.screen.append_child(&row);
        }
    }

    fn css(&self, cell: &Cell) -> String {
        let mut css = String::new();
        if let Color::Rgb(r, g, b) = cell.fg {
            css.push_str(&format!("color:rgb({r},{g},{b});"));
        }
        if let Color::Rgb(r, g, b) = cell.bg
            && cell.bg != self.bg
        {
            css.push_str(&format!("background:rgb({r},{g},{b});"));
        }
        if cell.modifier.contains(Modifier::BOLD) {
            css.push_str("font-weight:bold;");
        }
        css
    }
}

impl Backend for DomGrid {
    type Error = io::Error;

    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        let w = usize::from(self.size.width);
        for (x, y, cell) in content {
            let Some(el) = self.cells.get(usize::from(y) * w + usize::from(x)) else {
                continue;
            };
            el.set_text_content(Some(cell.symbol()));
            let _ = el.set_attribute("style", &self.css(cell));
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
        for el in &self.cells {
            el.set_text_content(Some(" "));
            let _ = el.remove_attribute("style");
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

/// Run the game in the page: `#screen` holds the grid, `#keys` the soft-key bar.
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
            app = App::with_game(Game::new(Settings {
                seed: seed.unwrap_or_else(|| js_sys::Date::now() as u64),
                level: level.unwrap_or(Level::Easy),
                theme,
                adult,
            }));
        }
        if let Some(p) = params
            .get("phosphor")
            .and_then(|p| p.parse::<Phosphor>().ok())
        {
            app.set_phosphor(p);
        }
        let mut classes = Vec::new();
        if params.get("fx").as_deref() == Some("off") {
            app.set_fx(false);
            classes.push("plain");
        }
        match params.get("keys").as_deref() {
            Some("on") => classes.push("keys"),
            Some("off") => classes.push("nokeys"),
            _ => {}
        }
        if let Some(body) = document.body() {
            body.set_class_name(&classes.join(" "));
        }
    }

    let app = Rc::new(RefCell::new(app));
    let mut terminal = Terminal::new(DomGrid::new(document.clone(), screen)?)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Physical keyboard: anything the browser itself does not own.
    let keydown = Closure::wrap(Box::new({
        let app = app.clone();
        move |e: KeyboardEvent| {
            if e.ctrl_key() || e.meta_key() || e.alt_key() {
                return;
            }
            let Some(key) = key_from_name(&e.key()) else {
                return;
            };
            e.prevent_default();
            app.borrow_mut().on_key(key);
        }
    }) as Box<dyn FnMut(KeyboardEvent)>);
    document.add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())?;
    keydown.forget();

    // Soft keys: one listener on the bar, buttons carry `data-key`.
    let click = Closure::wrap(Box::new({
        let app = app.clone();
        move |e: Event| {
            let key = e
                .target()
                .and_then(|t| t.dyn_into::<Element>().ok())
                .and_then(|el| el.closest("[data-key]").ok().flatten())
                .and_then(|el| el.get_attribute("data-key"))
                .and_then(|name| key_from_name(&name));
            if let Some(key) = key {
                app.borrow_mut().on_key(key);
            }
        }
    }) as Box<dyn FnMut(Event)>);
    keys.add_event_listener_with_callback("click", click.as_ref().unchecked_ref())?;
    click.forget();

    // Draw loop: 60 Hz, redrawing only what changed.
    let mut last_keys: Vec<&'static str> = Vec::new();
    let mut last_phosphor: Option<Phosphor> = None;
    let root = document.document_element().ok_or("no root")?;
    let root: web_sys::HtmlElement = root.dyn_into()?;
    let frame: FrameLoop = Rc::new(RefCell::new(None));
    let next = frame.clone();
    *frame.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        {
            let mut app = app.borrow_mut();
            app.advance(1);
            if last_phosphor != Some(app.phosphor()) {
                last_phosphor = Some(app.phosphor());
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
            let labels: Vec<&'static str> = soft.iter().map(|(l, _)| *l).collect();
            if labels != last_keys {
                last_keys = labels;
                let html: String = soft
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
                keys.set_inner_html(&html);
            }
        }
        let probe = app.borrow().frame().is_multiple_of(30);
        terminal.backend_mut().refresh(probe);
        let app = app.borrow();
        let _ = terminal.draw(|f| app.render(f.area(), f.buffer_mut()));
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
