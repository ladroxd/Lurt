#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Lurt: a light browser. One borderless glass window, one small HTML layer for the
//! browser UI ("chrome"), and one WebView2 per awake tab, all sharing one engine process.

mod blocker;
mod glass;
mod importer;
mod splash;
mod store;
mod tabs;
mod util;

use std::{
    rc::Rc,
    sync::OnceLock,
    time::{Duration, Instant},
};

use base64::Engine as _;
use serde_json::{Value, json};
use tao::{
    dpi::LogicalSize as WinSize,
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    platform::windows::{WindowBuilderExtWindows, WindowExtWindows},
    window::{Fullscreen, ResizeDirection, Window, WindowBuilder},
};
use wry::{
    MemoryUsageLevel, Rect, Theme, WebContext, WebView, WebViewBuilder, WebViewBuilderExtWindows,
    WebViewExtWindows,
    dpi::{LogicalPosition, LogicalSize},
};

use crate::{
    blocker::Shield,
    store::{Session, SessionTab, Settings, Store},
    tabs::{Tab, ViewOpts},
};

/// Everything that reaches the event loop from webviews and worker threads.
pub enum Ev {
    /// Message from the browser UI.
    Chrome(String),
    /// Message from a website's content script.
    Page(u32, String),
    Title(u32, String),
    /// (tab, started?, url)
    Load(u32, bool, String),
    Source(u32, String),
    History(u32),
    NewTab(String),
    Blocked(u32),
    Fullscreen(u32, bool),
    Snapshot(u32, Vec<u8>),
    /// (serialized engine, freshly downloaded?)
    ShieldReady(Vec<u8>, bool),
    ShieldFailed,
    /// Import finished: (bookmarks added, history entries merged) or an error.
    Imported(Result<(usize, usize), String>, String),
}

const UI_HTML: &str = include_str!("../ui/index.html");
const UI_CSS: &str = include_str!("../ui/style.css");
const UI_JS: &str = include_str!("../ui/app.js");

#[derive(Clone, Copy, PartialEq, Default)]
struct Insets {
    t: f64,
    r: f64,
    b: f64,
    l: f64,
}

struct App {
    window: Window,
    chrome: WebView,
    ctx: WebContext,
    proxy: EventLoopProxy<Ev>,
    store: Store,
    settings: Settings,
    backdrop: &'static str,
    shield: Rc<Shield>,
    tabs: Vec<Tab>,
    active: u32,
    next_id: u32,
    insets: Insets,
    closed: Vec<String>,
    fullscreen: bool,
    overlay: bool,
    ui_ready: bool,
    started: Instant,
    quit: bool,
}

/// `LURT_BENCH=1` prints startup timings to stderr, never shows the window,
/// writes nothing to the user's session, and exits once the first page has loaded.
fn bench() -> bool {
    static B: OnceLock<bool> = OnceLock::new();
    *B.get_or_init(|| std::env::var_os("LURT_BENCH").is_some())
}

static T0: OnceLock<Instant> = OnceLock::new();

pub fn mark(label: &str) {
    if bench() {
        let t = T0.get_or_init(Instant::now).elapsed();
        eprintln!("{:>6.1} ms  {label}", t.as_secs_f64() * 1000.0);
    }
}

fn main() {
    T0.get_or_init(Instant::now);
    let event_loop = EventLoopBuilder::<Ev>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let store = Store::open();
    let settings = store.load_settings();
    mark("store + settings loaded");

    // Open at a comfortable size that always fits the screen, centered.
    let (mut w, mut h, mut x, mut y) = (1280.0, 820.0, None, None);
    if let Some(m) = event_loop.primary_monitor() {
        let sf = m.scale_factor();
        let size = m.size().to_logical::<f64>(sf);
        let pos = m.position().to_logical::<f64>(sf);
        w = (size.width * 0.88).min(1480.0);
        h = (size.height * 0.86).min(980.0);
        x = Some(pos.x + (size.width - w) / 2.0);
        y = Some(pos.y + (size.height - h) / 2.0 - size.height * 0.02);
    }
    let mut builder = WindowBuilder::new()
        .with_title("Lurt")
        .with_decorations(false)
        .with_transparent(true)
        .with_undecorated_shadow(true)
        .with_visible(false)
        .with_inner_size(WinSize::new(w, h))
        .with_min_inner_size(WinSize::new(520.0, 380.0));
    if let (Some(x), Some(y)) = (x, y) {
        builder = builder.with_position(tao::dpi::LogicalPosition::new(x.max(0.0), y.max(0.0)));
    }
    let window = builder.build(&event_loop).expect("window");
    mark("window created");
    let backdrop = glass::apply(&window, &settings.backdrop);
    glass::set_window_opacity(&window, settings.window_opacity);
    mark("backdrop applied");

    // Show the window right away with a skeleton of the UI; the WebView2 engine
    // starts underneath it (0.3 s warm, a few seconds on the first launch after boot).
    let early = !bench() && std::env::var_os("LURT_NO_SPLASH").is_none();
    if early {
        window.set_visible(true);
        let dark = match settings.theme.as_str() {
            "dark" => true,
            "light" => false,
            _ => splash::system_dark(),
        };
        let session = store.load_session();
        let has_arg = std::env::args().skip(1).any(|a| !a.starts_with("--"));
        let blank = !has_arg
            && (!settings.restore_session
                || session.tabs.get(session.active).is_none_or(|t| t.url.is_empty()));
        let pos = window.inner_position().unwrap_or_default();
        let size = window.inner_size();
        let tint = if backdrop == "none" { 1.0 } else { f32::from(settings.tint) / 100.0 };
        splash::show(
            windows::Win32::Foundation::HWND(window.hwnd() as _),
            pos.x,
            pos.y,
            size.width as i32,
            size.height as i32,
            &splash::Look {
                dark,
                tint,
                radius: f32::from(settings.radius),
                compact: settings.compact,
                blank,
                scale: window.scale_factor() as f32,
            },
        );
        mark("skeleton shown");
    }

    let mut ctx = WebContext::new(Some(Store::webview_dir()));
    let page = UI_HTML.replace("/*CSS*/", UI_CSS).replace("/*JS*/", UI_JS);
    let p = proxy.clone();
    let size = window.inner_size().to_logical::<f64>(window.scale_factor());
    let chrome = WebViewBuilder::new_with_web_context(&mut ctx)
        .with_html(page)
        .with_transparent(true)
        .with_bounds(Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: LogicalSize::new(size.width, size.height).into(),
        })
        .with_ipc_handler(move |req| {
            let _ = p.send_event(Ev::Chrome(req.into_body()));
        })
        .with_devtools(cfg!(debug_assertions))
        .with_browser_accelerator_keys(false)
        .with_hotkeys_zoom(false)
        .with_theme(theme_of(&settings.theme))
        .with_scroll_bar_style(wry::ScrollBarStyle::FluentOverlay)
        .build_as_child(&window)
        .expect("WebView2 runtime");
    mark("UI webview created (WebView2 engine up)");

    tabs::set_tracking(&chrome, &settings.tracking);

    let shield = Rc::new(Shield::default());
    shield.enabled.set(settings.adblock);
    *shield.allow.borrow_mut() = settings.adblock_allow.clone();
    blocker::load(store.dir.clone(), false, proxy.clone());

    let mut app = App {
        window,
        chrome,
        ctx,
        proxy,
        store,
        settings,
        backdrop,
        shield,
        tabs: Vec::new(),
        active: 0,
        next_id: 1,
        insets: Insets { t: 92.0, r: 8.0, b: 8.0, l: 8.0 },
        closed: Vec::new(),
        fullscreen: false,
        overlay: false,
        ui_ready: false,
        started: Instant::now(),
        quit: false,
    };
    app.restore_session();

    let mut next_tick = Instant::now() + Duration::from_secs(1);
    event_loop.run(move |event, _, cf| {
        match event {
            Event::NewEvents(StartCause::Init) => mark("event loop running"),
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                app.reveal_due();
                if Instant::now() >= next_tick {
                    app.tick();
                    next_tick = Instant::now() + Duration::from_secs(if app.ui_ready { 60 } else { 1 });
                }
            }
            Event::UserEvent(ev) => app.on_event(ev),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => app.quit = true,
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                    // Windows 10's blur stays at the old size/position after a maximize
                    // or restore unless it is applied again.
                    app.backdrop = glass::apply(&app.window, &app.settings.backdrop);
                    app.layout();
                    app.push_state();
                }
                _ => {}
            },
            _ => {}
        }
        let wake = app.tabs.iter().filter_map(|t| t.reveal_by).min().map_or(next_tick, |r| r.min(next_tick));
        *cf = ControlFlow::WaitUntil(wake);
        if app.quit {
            app.save_session();
            *cf = ControlFlow::Exit;
        }
    });
}

fn theme_of(s: &str) -> Theme {
    match s {
        "dark" => Theme::Dark,
        "light" => Theme::Light,
        _ => Theme::Auto,
    }
}

impl App {
    // ---- talking to the UI ------------------------------------------------------

    fn send(&self, v: Value) {
        let _ = self.chrome.evaluate_script(&format!("window.lurt&&lurt.on({v})"));
    }

    fn tab(&self, id: u32) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == id)
    }

    fn tab_mut(&mut self, id: u32) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|t| t.id == id)
    }

    fn active_view(&self) -> Option<&WebView> {
        self.tab(self.active).and_then(|t| t.view.as_ref())
    }

    fn push_state(&self) {
        let tabs: Vec<Value> = self
            .tabs
            .iter()
            .map(|t| {
                json!({
                    "id": t.id, "title": t.title, "url": t.url, "fav": t.fav,
                    "loading": t.loading, "blocked": t.blocked,
                    "asleep": t.view.is_none() && !t.url.is_empty(),
                    "shown": t.view.is_some() && t.shown,
                })
            })
            .collect();
        let a = self.tab(self.active);
        let url = a.map(|t| t.url.as_str()).unwrap_or("");
        self.send(json!({
            "t": "state", "tabs": tabs, "active": self.active,
            "back": a.is_some_and(|t| t.back), "fwd": a.is_some_and(|t| t.fwd),
            "star": !url.is_empty() && self.store.is_bookmarked(url),
            "allowed": self.shield.allowed(&util::host_of(url)),
            "max": self.window.is_maximized(), "fs": self.fullscreen,
        }));
    }

    fn send_settings(&self) {
        self.send(json!({
            "t": "settings", "s": self.settings, "backdrop": self.backdrop,
            "shield": self.shield.engine.borrow().is_some(),
            "modsDir": self.store.mods_dir().to_string_lossy(),
        }));
    }

    fn toast(&self, text: &str) {
        self.send(json!({ "t": "toast", "text": text }));
    }

    fn focus_address(&self) {
        let _ = self.chrome.focus();
        self.send(json!({ "t": "focus" }));
    }

    // ---- layout -----------------------------------------------------------------

    fn logical_size(&self) -> (f64, f64) {
        let s = self.window.inner_size().to_logical::<f64>(self.window.scale_factor());
        (s.width, s.height)
    }

    fn viewport(&self) -> Rect {
        let (w, h) = self.logical_size();
        let i = if self.fullscreen { Insets::default() } else { self.insets };
        Rect {
            position: LogicalPosition::new(i.l, i.t).into(),
            size: LogicalSize::new((w - i.l - i.r).max(1.0), (h - i.t - i.b).max(1.0)).into(),
        }
    }

    fn layout(&self) {
        let (w, h) = self.logical_size();
        let _ = self.chrome.set_bounds(Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: LogicalSize::new(w, h).into(),
        });
        self.layout_tabs();
    }

    fn layout_tabs(&self) {
        let vp = self.viewport();
        let sf = self.window.scale_factor();
        let px = vp.size.to_physical::<i32>(sf);
        let radius = if self.settings.rounded_pages && !self.fullscreen {
            (f64::from(self.settings.radius) * 0.7 * sf).round() as i32
        } else {
            0
        };
        for t in &self.tabs {
            if let Some(v) = &t.view {
                let _ = v.set_bounds(vp);
                glass::round(v.hwnd(), px.width, px.height, radius);
            }
        }
    }

    // ---- tabs -------------------------------------------------------------------

    fn restore_session(&mut self) {
        let session = if self.settings.restore_session {
            self.store.load_session()
        } else {
            Session::default()
        };
        for t in session.tabs {
            let id = self.next_id;
            self.next_id += 1;
            self.tabs.push(Tab::new(id, t.url, t.title));
        }
        if self.tabs.is_empty() {
            self.tabs.push(Tab::new(self.next_id, String::new(), String::new()));
            self.next_id += 1;
        }
        // Only the active tab gets a page view; the rest stay asleep until opened.
        self.active = self.tabs[session.active.min(self.tabs.len() - 1)].id;

        // `lurt.exe <url or search>` opens it in a new tab (e.g. as the default browser).
        let args: Vec<String> = std::env::args().skip(1).filter(|a| !a.starts_with("--")).collect();
        if !args.is_empty() {
            let url = util::resolve_input(&args.join(" "), &self.settings.search);
            // Reuse a blank start tab rather than leaving it behind.
            let target = match self.tab(self.active) {
                Some(t) if t.url.is_empty() => t.id,
                _ => {
                    let id = self.next_id;
                    self.next_id += 1;
                    self.tabs.push(Tab::new(id, String::new(), String::new()));
                    id
                }
            };
            if let Some(t) = self.tab_mut(target) {
                t.url = url;
            }
            self.active = target;
        }
    }

    fn save_session(&self) {
        if bench() {
            return;
        }
        let tabs = self
            .tabs
            .iter()
            .map(|t| SessionTab { url: t.url.clone(), title: t.title.clone() })
            .collect();
        let active = self.tabs.iter().position(|t| t.id == self.active).unwrap_or(0);
        self.store.save_session(&Session { tabs, active });
    }

    /// Give a tab a page view if it has a URL but none yet (new or asleep).
    fn wake(&mut self, id: u32) {
        let vp = self.viewport();
        let theme = theme_of(&self.settings.theme);
        let Some(idx) = self.tabs.iter().position(|t| t.id == id) else { return };
        if self.tabs[idx].view.is_some() || self.tabs[idx].url.is_empty() {
            return;
        }
        let url = self.tabs[idx].url.clone();
        let view = tabs::create(
            id,
            ViewOpts { url: &url, bounds: vp, visible: false, theme },
            &mut self.ctx,
            &self.window,
            &self.proxy,
            &self.shield,
        );
        mark("page view created");
        if let Some(v) = &view {
            tabs::set_autofill(v, self.settings.save_passwords, self.settings.autofill);
        }
        let t = &mut self.tabs[idx];
        t.view = view;
        t.loading = t.view.is_some();
        t.blocked = 0;
        t.shown = false;
        t.reveal_by = t.view.is_some().then(|| Instant::now() + Duration::from_millis(2500));
        self.layout_tabs();
    }

    /// The page has content: swap the skeleton for the real view.
    fn reveal(&mut self, id: u32) {
        let active = id == self.active && !self.overlay;
        let Some(t) = self.tab_mut(id) else { return };
        t.reveal_by = None;
        if t.shown {
            return;
        }
        t.shown = true;
        if active {
            if let Some(v) = &t.view {
                let _ = v.set_visible(true);
                let _ = v.focus();
            }
        }
        self.push_state();
    }

    fn reveal_due(&mut self) {
        let now = Instant::now();
        let due: Vec<u32> = self.tabs.iter().filter(|t| t.reveal_by.is_some_and(|r| r <= now)).map(|t| t.id).collect();
        for id in due {
            self.reveal(id);
        }
    }

    /// Show the active tab's view if it is ready to be seen.
    fn show_active(&self) {
        if let Some(t) = self.tab(self.active) {
            if let (Some(v), true) = (&t.view, t.shown) {
                let _ = v.set_visible(true);
                let _ = v.focus();
            }
        }
    }

    fn new_tab(&mut self, url: Option<String>, next_to_active: bool) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let pos = if next_to_active {
            self.tabs.iter().position(|t| t.id == self.active).map_or(self.tabs.len(), |i| i + 1)
        } else {
            self.tabs.len()
        };
        let blank = url.is_none();
        self.tabs.insert(pos, Tab::new(id, url.unwrap_or_default(), String::new()));
        self.activate(id);
        if blank {
            self.focus_address();
        }
        id
    }

    fn activate(&mut self, id: u32) {
        if self.tab(id).is_none() {
            return;
        }
        let now = Instant::now();
        let prev = self.active;
        if let Some(t) = self.tab_mut(prev) {
            t.last_active = now;
        }
        self.active = id;
        self.overlay = false;
        for t in &self.tabs {
            if t.id != id {
                if let Some(v) = &t.view {
                    let _ = v.set_visible(false);
                    let _ = v.set_memory_usage_level(MemoryUsageLevel::Low);
                }
            }
        }
        self.wake(id);
        if let Some(t) = self.tab_mut(id) {
            t.last_active = now;
        }
        if let Some(v) = self.active_view() {
            let _ = v.set_memory_usage_level(MemoryUsageLevel::Normal);
        }
        self.show_active();
        self.push_state();
    }

    fn close_tab(&mut self, id: u32) {
        let Some(i) = self.tabs.iter().position(|t| t.id == id) else { return };
        let t = self.tabs.remove(i);
        if !t.url.is_empty() {
            self.closed.push(t.url.clone());
            if self.closed.len() > 25 {
                self.closed.remove(0);
            }
        }
        drop(t);
        if self.tabs.is_empty() {
            self.new_tab(None, false);
        } else if id == self.active {
            let next = self.tabs[i.min(self.tabs.len() - 1)].id;
            self.activate(next);
        } else {
            self.push_state();
        }
    }

    fn cycle(&mut self, step: isize) {
        let n = self.tabs.len() as isize;
        if let Some(i) = self.tabs.iter().position(|t| t.id == self.active) {
            let next = self.tabs[((i as isize + step).rem_euclid(n)) as usize].id;
            self.activate(next);
        }
    }

    fn navigate(&mut self, url: String) {
        if url.is_empty() {
            return;
        }
        let id = self.active;
        self.overlay = false;
        let Some(t) = self.tab_mut(id) else { return };
        match &t.view {
            Some(v) => {
                let _ = v.load_url(&url);
                t.loading = true;
                self.show_active();
            }
            None => {
                t.url = url;
                self.wake(id);
                if let Some(v) = self.active_view() {
                    let _ = v.focus();
                }
            }
        }
        self.push_state();
    }

    fn go_home(&mut self) {
        if !self.settings.homepage.is_empty() {
            let url = util::resolve_input(&self.settings.homepage.clone(), &self.settings.search);
            self.navigate(url);
            return;
        }
        // The start page needs no page view at all, so drop it to free memory.
        let id = self.active;
        if let Some(t) = self.tab_mut(id) {
            t.view = None;
            t.shown = false;
            t.reveal_by = None;
            t.url.clear();
            t.title.clear();
            t.fav.clear();
            t.loading = false;
            t.back = false;
            t.fwd = false;
        }
        self.push_state();
        self.focus_address();
    }

    /// Put background tabs to sleep after the configured idle time.
    fn tick(&mut self) {
        if !self.ui_ready && self.started.elapsed() > Duration::from_secs(3) {
            // The UI never reported in; show the window anyway rather than stay invisible.
            if !bench() {
                self.window.set_visible(true);
            }
            splash::dismiss();
            self.ui_ready = true;
        }
        let mins = self.settings.sleep_minutes;
        if mins > 0 {
            let limit = Duration::from_secs(u64::from(mins) * 60);
            let mut slept = false;
            for t in self.tabs.iter_mut() {
                if t.id != self.active
                    && t.last_active.elapsed() > limit
                    && t.view.as_ref().is_some_and(|v| !tabs::is_playing_audio(v))
                {
                    t.view = None;
                    t.loading = false;
                    t.shown = false;
                    t.reveal_by = None;
                    slept = true;
                }
            }
            if slept {
                self.push_state();
            }
        }
        self.save_session();
    }

    // ---- actions ----------------------------------------------------------------

    fn toggle_star(&mut self) {
        let Some(t) = self.tab(self.active) else { return };
        if t.url.is_empty() {
            return;
        }
        let on = self.store.toggle_bookmark(&t.url, &t.title);
        self.toast(if on { "Bookmarked" } else { "Bookmark removed" });
        self.push_state();
        self.send_list("bookmarks", "");
    }

    fn toggle_shield_for_site(&mut self) {
        let host = self.tab(self.active).map(|t| util::host_of(&t.url)).unwrap_or_default();
        if host.is_empty() {
            return;
        }
        let mut s = self.settings.clone();
        if let Some(i) = s.adblock_allow.iter().position(|h| *h == host) {
            s.adblock_allow.remove(i);
            self.toast(&format!("Blocking on {host}"));
        } else {
            s.adblock_allow.push(host.clone());
            self.toast(&format!("Blocking paused on {host}"));
        }
        self.apply_settings(s);
        if let Some(v) = self.active_view() {
            let _ = v.reload();
        }
    }

    fn send_list(&self, kind: &str, q: &str) {
        let items = match kind {
            "history" => self.store.history(q, 300),
            "top" => self.store.top_sites(8),
            _ => self.store.bookmarks(),
        };
        self.send(json!({ "t": "list", "kind": kind, "items": items }));
    }

    fn apply_settings(&mut self, new: Settings) {
        if new == self.settings {
            return;
        }
        let old = std::mem::replace(&mut self.settings, new);
        let s = &self.settings;
        if old.backdrop != s.backdrop {
            self.backdrop = glass::apply(&self.window, &s.backdrop);
        }
        if old.window_opacity != s.window_opacity {
            glass::set_window_opacity(&self.window, s.window_opacity);
        }
        if old.theme != s.theme {
            let theme = theme_of(&s.theme);
            let _ = self.chrome.set_theme(theme);
            for t in &self.tabs {
                if let Some(v) = &t.view {
                    let _ = v.set_theme(theme);
                }
            }
        }
        if old.tracking != s.tracking {
            tabs::set_tracking(&self.chrome, &s.tracking);
        }
        if old.save_passwords != s.save_passwords || old.autofill != s.autofill {
            for t in &self.tabs {
                if let Some(v) = &t.view {
                    tabs::set_autofill(v, s.save_passwords, s.autofill);
                }
            }
        }
        self.shield.enabled.set(s.adblock);
        *self.shield.allow.borrow_mut() = s.adblock_allow.clone();
        if old.rounded_pages != s.rounded_pages || old.radius != s.radius {
            self.layout_tabs();
        }
        self.store.save_settings(&self.settings);
        self.send_settings();
        self.push_state();
    }

    /// Inject cosmetic ad filters and the user's site mods once a page's DOM is ready.
    fn inject_page(&self, id: u32) {
        let Some(view) = self.tab(id).and_then(|t| t.view.as_ref()) else { return };
        let url = view.url().unwrap_or_default();
        let host = util::host_of(&url);
        let mut css = String::new();
        let mut js = String::new();
        if self.shield.enabled.get() && !self.shield.allowed(&host) {
            if let Some(e) = self.shield.engine.borrow().as_ref() {
                let r = e.url_cosmetic_resources(&url);
                // One rule per selector, so one bad selector can't void the rest.
                for sel in r.hide_selectors {
                    css.push_str(&sel);
                    css.push_str("{display:none!important}\n");
                }
                js.push_str(&r.injected_script);
            }
        }
        let (mod_css, mod_js) = self.store.site_mods(&host);
        css.push_str(&mod_css);
        js.push_str(&mod_js);
        if css.is_empty() && js.is_empty() {
            return;
        }
        let script = format!(
            "(()=>{{let s=document.getElementById('lurt-style');if(!s){{s=document.createElement('style');\
             s.id='lurt-style';(document.head||document.documentElement).appendChild(s)}}s.textContent={};}})();\n{}",
            json!(css),
            js
        );
        let _ = view.evaluate_script(&script);
    }

    fn shortcut(&mut self, k: &str) {
        match k {
            "ctrl+t" => {
                self.new_tab(None, false);
            }
            "ctrl+w" => self.close_tab(self.active),
            "ctrl+l" | "alt+d" | "f6" => self.focus_address(),
            "ctrl+tab" => self.cycle(1),
            "ctrl+shift+tab" => self.cycle(-1),
            "ctrl+d" => self.toggle_star(),
            "ctrl+h" => self.send(json!({ "t": "panel", "p": "history" })),
            "ctrl+shift+b" => self.send(json!({ "t": "panel", "p": "bookmarks" })),
            "ctrl+comma" => self.send(json!({ "t": "panel", "p": "settings" })),
            "ctrl+shift+t" => {
                if let Some(url) = self.closed.pop() {
                    self.new_tab(Some(url), false);
                }
            }
            _ => {
                if let Some(n) = k.strip_prefix("ctrl+").and_then(|d| d.parse::<usize>().ok()) {
                    let idx = if n == 9 { self.tabs.len() - 1 } else { n - 1 };
                    if let Some(t) = self.tabs.get(idx) {
                        let id = t.id;
                        self.activate(id);
                    }
                }
            }
        }
    }

    // ---- events -----------------------------------------------------------------

    fn on_event(&mut self, ev: Ev) {
        match ev {
            Ev::Chrome(raw) => self.on_chrome(&raw),
            Ev::Page(id, raw) => self.on_page(id, &raw),
            Ev::Title(id, title) => {
                if let Some(t) = self.tab_mut(id) {
                    t.title = title;
                    let (url, title) = (t.url.clone(), t.title.clone());
                    if !util::is_internal(&url) {
                        self.store.set_title(&url, &title);
                    }
                    self.push_state();
                }
            }
            Ev::Load(id, started, url) => {
                mark(if started { "page: content loading" } else { "page: load finished" });
                if bench() && !started {
                    self.quit = true;
                }
                if !started {
                    self.reveal(id);
                }
                let Some(t) = self.tab_mut(id) else { return };
                t.loading = started;
                if started {
                    t.blocked = 0;
                } else if !util::is_internal(&url) {
                    let title = t.title.clone();
                    self.store.add_visit(&url, &title);
                }
                self.push_state();
            }
            Ev::Source(id, url) => {
                let Some(t) = self.tab_mut(id) else { return };
                if t.url != url {
                    if util::host_of(&t.url) != util::host_of(&url) {
                        t.fav.clear();
                    }
                    t.url = url.clone();
                    // Single-page-app navigation never fires a page load.
                    if !t.loading && !util::is_internal(&url) {
                        let title = t.title.clone();
                        self.store.add_visit(&url, &title);
                    }
                    self.push_state();
                }
            }
            Ev::History(id) => {
                let Some(t) = self.tab_mut(id) else { return };
                if let Some(v) = &t.view {
                    t.back = v.can_go_back().unwrap_or(false);
                    t.fwd = v.can_go_forward().unwrap_or(false);
                }
                self.push_state();
            }
            Ev::NewTab(url) => {
                self.new_tab(Some(url), true);
            }
            Ev::Blocked(id) => {
                if let Some(t) = self.tab_mut(id) {
                    t.blocked += 1;
                    let n = t.blocked;
                    self.send(json!({ "t": "blocked", "id": id, "n": n }));
                }
            }
            Ev::Fullscreen(id, on) => {
                if id == self.active {
                    self.fullscreen = on;
                    self.window.set_fullscreen(on.then_some(Fullscreen::Borderless(None)));
                    self.layout();
                    self.push_state();
                }
            }
            Ev::Snapshot(id, bytes) => {
                if self.overlay && id == self.active {
                    let snap = if bytes.is_empty() {
                        Value::Null
                    } else {
                        json!(format!(
                            "data:image/jpeg;base64,{}",
                            base64::engine::general_purpose::STANDARD.encode(bytes)
                        ))
                    };
                    self.send(json!({ "t": "overlay", "snap": snap }));
                }
            }
            Ev::ShieldReady(bytes, fresh) => {
                if self.shield.install(&bytes) {
                    if fresh {
                        self.toast("Filter lists updated");
                    }
                    self.send_settings();
                }
            }
            Ev::ShieldFailed => self.toast("Couldn't download filter lists. Blocking is off until it can."),
            Ev::Imported(result, from) => {
                let text = match result {
                    Ok((b, h)) => format!("Imported from {from}: {b} new bookmarks, {h} history entries"),
                    Err(e) => format!("Import from {from} failed: {e}"),
                };
                self.send(json!({ "t": "imported", "text": text }));
                self.send_list("top", "");
                self.send_list("bookmarks", "");
            }
        }
    }

    fn on_page(&mut self, id: u32, raw: &str) {
        // Websites can post here too, so only a few harmless messages are accepted.
        let Ok(m) = serde_json::from_str::<Value>(raw) else { return };
        match m["t"].as_str().unwrap_or("") {
            "key" if id == self.active => {
                let k = m["k"].as_str().unwrap_or("").to_string();
                self.shortcut(&k);
            }
            "fav" => {
                let u = m["u"].as_str().unwrap_or("");
                if u.len() < 2048 && (u.starts_with("https://") || u.starts_with("http://")) {
                    let u = u.to_string();
                    if let Some(t) = self.tab_mut(id) {
                        t.fav = u;
                    }
                    self.push_state();
                }
            }
            "dom" => {
                mark("page: DOM ready");
                self.inject_page(id);
                self.reveal(id);
            }
            _ => {}
        }
    }

    fn on_chrome(&mut self, raw: &str) {
        let Ok(m) = serde_json::from_str::<Value>(raw) else { return };
        let s = |k: &str| m[k].as_str().unwrap_or("").to_string();
        let id = m["id"].as_u64().map(|v| v as u32).unwrap_or(self.active);
        match s("t").as_str() {
            "ready" => {
                self.ui_ready = true;
                self.send_settings();
                self.send(json!({ "t": "mods", "css": self.store.ui_mods() }));
                self.push_state();
                mark("UI ready (first paint done)");
                // Safety net: never leave the skeleton up if the "painted" signal is lost.
                let p = self.proxy.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(1500));
                    let _ = p.send_event(Ev::Chrome(r#"{"t":"painted"}"#.into()));
                });
                if !bench() && !self.window.is_visible() {
                    self.window.set_visible(true);
                }
                let active = self.active;
                if self.tab(active).is_some_and(|t| t.url.is_empty()) {
                    self.focus_address();
                } else {
                    self.activate(active);
                }
            }
            "insets" => {
                let f = |k: &str| m[k].as_f64().unwrap_or(0.0);
                let ins = Insets { t: f("top"), r: f("right"), b: f("bottom"), l: f("left") };
                if ins != self.insets {
                    self.insets = ins;
                    self.layout_tabs();
                }
            }
            "new" => {
                let url = Some(s("url")).filter(|u| !u.is_empty());
                self.new_tab(url, false);
            }
            "close" => self.close_tab(id),
            "activate" => self.activate(id),
            "go" => {
                let url = util::resolve_input(&s("input"), &self.settings.search);
                if m["newTab"].as_bool() == Some(true) {
                    self.new_tab(Some(url), true);
                } else {
                    self.navigate(url);
                }
            }
            "open" => {
                let url = s("url");
                if m["newTab"].as_bool() == Some(true) {
                    self.new_tab(Some(url), true);
                } else {
                    self.navigate(url);
                }
            }
            "nav" => match s("a").as_str() {
                "back" => {
                    if let Some(v) = self.active_view() {
                        let _ = v.go_back();
                    }
                }
                "forward" => {
                    if let Some(v) = self.active_view() {
                        let _ = v.go_forward();
                    }
                }
                "reload" => match self.active_view() {
                    Some(v) => {
                        let _ = v.reload();
                    }
                    None => self.wake(self.active),
                },
                "home" => self.go_home(),
                _ => {}
            },
            "win" => match s("a").as_str() {
                "min" => self.window.set_minimized(true),
                "max" => self.window.set_maximized(!self.window.is_maximized()),
                "close" => self.quit = true,
                "drag" => {
                    let _ = self.window.drag_window();
                }
                "resize" => {
                    let d = match s("d").as_str() {
                        "n" => ResizeDirection::North,
                        "s" => ResizeDirection::South,
                        "e" => ResizeDirection::East,
                        "w" => ResizeDirection::West,
                        "ne" => ResizeDirection::NorthEast,
                        "nw" => ResizeDirection::NorthWest,
                        "se" => ResizeDirection::SouthEast,
                        _ => ResizeDirection::SouthWest,
                    };
                    let _ = self.window.drag_resize_window(d);
                }
                _ => {}
            },
            "overlay" => {
                if m["shown"].as_bool() == Some(true) {
                    // The UI is now showing the snapshot, so the live page can hide.
                    if self.overlay {
                        if let Some(v) = self.active_view() {
                            let _ = v.set_visible(false);
                        }
                    }
                    return;
                }
                self.overlay = m["open"].as_bool().unwrap_or(false);
                if self.overlay {
                    let ready = self.tab(self.active).is_some_and(|t| t.shown);
                    match self.active_view().filter(|_| ready) {
                        Some(v) => tabs::snapshot(v, self.active, self.proxy.clone()),
                        None => self.send(json!({ "t": "overlay", "snap": null })),
                    }
                } else if let Some(t) = self.tab(self.active) {
                    if let (Some(v), true) = (&t.view, t.shown) {
                        let _ = v.set_visible(true);
                        if m["focusPage"].as_bool() == Some(true) {
                            let _ = v.focus();
                        }
                    }
                }
            }
            "star" => self.toggle_star(),
            "shield" => self.toggle_shield_for_site(),
            "list" => self.send_list(&s("kind"), &s("q")),
            "suggest" => {
                let q = s("q");
                let items = self.store.suggest(&q);
                self.send(json!({ "t": "suggest", "q": q, "items": items }));
            }
            "delHistory" => {
                self.store.delete_history(&s("url"));
                self.send_list("history", &s("q"));
            }
            "delBookmark" => {
                self.store.remove_bookmark(&s("url"));
                self.send_list("bookmarks", "");
                self.push_state();
            }
            "clearHistory" => {
                self.store.clear_history();
                self.send_list("history", "");
                self.toast("History cleared");
            }
            "clearData" => {
                let _ = self.chrome.clear_all_browsing_data();
                self.toast("Cookies, cache and site data cleared");
            }
            "settings" => {
                if let Ok(new) = serde_json::from_value::<Settings>(m["s"].clone()) {
                    self.apply_settings(new);
                }
            }
            "updateLists" => {
                self.toast("Updating filter lists...");
                blocker::load(self.store.dir.clone(), true, self.proxy.clone());
            }
            "reloadMods" => {
                self.send(json!({ "t": "mods", "css": self.store.ui_mods() }));
                if let Some(v) = self.active_view() {
                    let _ = v.reload();
                }
                self.toast("Mods reloaded");
            }
            "openMods" => {
                let _ = std::process::Command::new("explorer").arg(self.store.mods_dir()).spawn();
            }
            "devtools" => {
                if let Some(v) = self.active_view() {
                    v.open_devtools();
                }
            }
            "importSources" => {
                self.send(json!({ "t": "sources", "items": importer::detect() }));
            }
            "importProfile" | "importFile" => {
                let (db, scratch, proxy) = (self.store.db_path(), self.store.dir.clone(), self.proxy.clone());
                let profile = s("t") == "importProfile";
                let (id, label, name, text) = (s("id"), s("label"), s("name"), s("text"));
                std::thread::spawn(move || {
                    let result = if profile && id == "*" {
                        // Every profile of every browser; report the totals.
                        let mut total = (0, 0);
                        let mut err = None;
                        for src in importer::detect() {
                            match importer::read_profile(&src.id, &scratch).and_then(|b| importer::save(&db, &b)) {
                                Ok((b, h)) => {
                                    total.0 += b;
                                    total.1 += h;
                                }
                                Err(e) => err = Some(format!("{}: {e}", src.browser)),
                            }
                        }
                        match err {
                            Some(e) if total == (0, 0) => Err(e),
                            _ => Ok(total),
                        }
                    } else if profile {
                        importer::read_profile(&id, &scratch).and_then(|b| importer::save(&db, &b))
                    } else {
                        importer::read_file(&name, &text).and_then(|b| importer::save(&db, &b))
                    };
                    let _ = proxy.send_event(Ev::Imported(result, if profile { label } else { name }));
                });
            }
            "painted" => {
                mark("UI painted, skeleton fading out");
                splash::dismiss();
            }
            "focusPage" => {
                if let Some(v) = self.active_view() {
                    let _ = v.focus();
                }
            }
            "key" => self.shortcut(&s("k")),
            _ => {}
        }
    }
}
