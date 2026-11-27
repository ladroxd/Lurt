#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Lurt: a light browser. One borderless glass window, one small HTML layer for the
//! browser UI ("chrome"), and one WebView2 per awake tab, all sharing one engine process.

mod account;
mod dialogs;
mod privacy;
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
    store::{Session, SessionTab, Settings, Slot, Store},
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
    /// A page started or stopped playing sound: (tab, playing).
    Audio(u32, bool),
    /// (opener tab, url): a page opened a link in a new tab.
    NewTab(u32, String),
    /// A page was zoomed: (tab, percent).
    Zoom(u32, u32),
    /// Load this URL in the tab instead: a cleaned link, or an HTTPS upgrade or its fallback.
    Redirect(u32, String),
    /// Browsing data deleted on quit.
    Cleared,
    /// The icon found on a site's homepage: (host, icon URL or "").
    FoundIcon(String, String),
    /// A page started a download (the file goes to the Downloads folder).
    /// `ask`: waiting for "Save as". `open`: open the file when it finishes.
    DlStart { tab: u32, dl: u32, path: String, url: String, total: i64, ask: bool, open: bool },
    /// A download of a file type you blocked.
    DlBlocked { tab: u32, name: String, url: String },
    /// (download, bytes received, total bytes or -1)
    DlProgress(u32, i64, i64),
    /// (download, state, why it failed, file path)
    DlState(u32, &'static str, &'static str, String),
    /// Tab id, blocked request URL, resource type.
    Blocked(u32, String, &'static str),
    Fullscreen(u32, bool),
    /// Tab id, JPEG bytes (empty on failure), and whether it is a wake-up preview.
    Snapshot(u32, Vec<u8>, bool),
    /// (serialized engine, freshly downloaded?)
    ShieldReady(Vec<u8>, bool),
    ShieldFailed,
    /// Import finished: (bookmarks added, history entries merged) or an error.
    Imported(Result<(usize, usize), String>, String),
    /// Google accounts signed in to the browser, or why they couldn't be read.
    Accounts(Result<Vec<account::Account>, String>),
    /// A site's favicon for the loading screen color: (host, data: URL or None).
    FavData(String, Option<String>),
    /// New tab shortcuts found while importing a browser profile: (title, url).
    ImportedShortcuts(Vec<(String, String)>),
    SignedOut,
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
    /// When to re-present the window so the Windows 10 blur covers its new size.
    glass_fix_at: Option<Instant>,
    /// When to look up the signed-in Google account again (after Google pages load).
    account_at: Option<Instant>,
    accounts: Vec<account::Account>,
    /// "Maximized": Lurt fills the work area as a normal window, because Windows 10 paints
    /// the blur of a truly maximized window with a light gray fill (the browser looked
    /// brighter maximized). Holds the position and size to restore.
    zoom: Option<(tao::dpi::PhysicalPosition<i32>, tao::dpi::PhysicalSize<u32>)>,
    /// When old history was last auto-deleted (checked hourly).
    pruned_at: Option<Instant>,
    ui_ready: bool,
    started: Instant,
    quit: bool,
    /// Quitting waits until data marked "delete on quit" is gone, or this deadline.
    exit_clear: Option<Instant>,
    exit_cleared: bool,
    /// Per-session seed for fingerprint noise.
    seed: u32,
    downloads: Vec<store::Download>,
    /// Progress updates to the UI are batched until then.
    dl_push_at: Option<Instant>,
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
    let downloads = store.load_downloads();
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
        // No shadow: on Windows 10 it keeps an invisible frame around the window that
        // the blur paints over, showing as a see-through band on three sides.
        .with_undecorated_shadow(false)
        .with_visible(false)
        .with_inner_size(WinSize::new(w, h))
        .with_min_inner_size(WinSize::new(520.0, 380.0));
    if let (Some(x), Some(y)) = (x, y) {
        builder = builder.with_position(tao::dpi::LogicalPosition::new(x.max(0.0), y.max(0.0)));
    }
    let window = builder.build(&event_loop).expect("window");
    glass::set_icon(&window);
    mark("window created");
    let backdrop = glass::apply(&window, &settings.backdrop, is_dark(&settings.theme));
    glass::set_window_opacity(&window, settings.window_opacity);
    mark("backdrop applied");

    // Show the window right away with a skeleton of the UI; the WebView2 engine
    // starts underneath it (0.3 s warm, a few seconds on the first launch after boot).
    let early = !bench() && std::env::var_os("LURT_NO_SPLASH").is_none();
    if early {
        window.set_visible(true);
        let dark = is_dark(&settings.theme);
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
                tabs: if settings.restore_session && !session.tabs.is_empty() { session.tabs.len() } else { 1 },
                show_clock: settings.show_clock,
                bm_bar: settings.bookmarks_bar == "always" || (settings.bookmarks_bar == "start" && blank),
                shape: store.load_shape(),
            },
        );
        mark("skeleton shown");
    }

    tabs::write_engine_prefs(&Store::webview_dir(), &settings);
    let mut ctx = WebContext::new(Some(Store::webview_dir()));
    let page = UI_HTML.replace("/*CSS*/", UI_CSS).replace("/*JS*/", UI_JS).replace("__LURT_VERSION__", env!("CARGO_PKG_VERSION"));
    let p = proxy.clone();
    let size = window.inner_size().to_logical::<f64>(window.scale_factor());
    let args = tabs::ENGINE_ARGS.get_or_init(|| tabs::engine_args(&settings)).clone();
    let chrome = WebViewBuilder::new_with_web_context(&mut ctx)
        .with_additional_browser_args(args)
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
    shield.privacy.set(privacy_of(&settings));
    *shield.downloads.borrow_mut() = download_prefs(&settings);
    blocker::load(store.dir.clone(), false, settings.filter_lists.clone(), proxy.clone());

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
        glass_fix_at: None,
        account_at: Some(Instant::now() + Duration::from_secs(2)),
        accounts: Vec::new(),
        zoom: None,
        pruned_at: None,
        ui_ready: false,
        started: Instant::now(),
        quit: false,
        exit_clear: None,
        exit_cleared: false,
        downloads,
        dl_push_at: None,
        seed: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(7, |d| d.subsec_nanos() ^ d.as_secs() as u32),
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
                    // Win+Up, double-clicking or dragging to the top edge maximize for real;
                    // turn that into Lurt's own maximize (see `zoom`).
                    if app.window.is_maximized() && !app.fullscreen {
                        app.window.set_maximized(false);
                        app.set_zoom(true);
                    } else if app.zoom.is_some() && !app.fullscreen && !app.fills_work_area() {
                        // Snapped or resized some other way: no longer "maximized".
                        app.zoom = None;
                    }
                    // Once resizing settles (maximize, restore, or the end of a drag).
                    if app.backdrop != "none" {
                        app.glass_fix_at = Some(Instant::now() + Duration::from_millis(140));
                    }
                    app.layout();
                    app.push_state();
                }
                _ => {}
            },
            _ => {}
        }
        if app.ui_ready && app.account_at.is_some_and(|t| t <= Instant::now()) {
            app.account_at = None;
            account::refresh(&app.chrome, app.proxy.clone());
        }
        if app.dl_push_at.is_some_and(|t| t <= Instant::now()) {
            app.dl_push_at = None;
            app.send_downloads();
        }
        if app.glass_fix_at.is_some_and(|t| t <= Instant::now()) {
            app.glass_fix_at = None;
            glass::refresh(&app.window);
        }
        let mut wake = app.tabs.iter().filter_map(|t| t.reveal_by).min().map_or(next_tick, |r| r.min(next_tick));
        if let Some(t) = app.glass_fix_at {
            wake = wake.min(t);
        }
        if let Some(t) = app.dl_push_at {
            wake = wake.min(t);
        }
        if let Some(t) = app.account_at.filter(|_| app.ui_ready) {
            wake = wake.min(t);
        }
        *cf = ControlFlow::WaitUntil(wake);
        if app.quit {
            if app.exit_clear.is_none() {
                app.begin_exit_clear();
            }
            match app.exit_clear {
                Some(t) if !app.exit_cleared && t > Instant::now() => *cf = ControlFlow::WaitUntil(t),
                _ => {
                    app.save_session();
                    *cf = ControlFlow::Exit;
                }
            }
        }
    });
}

fn download_prefs(s: &Settings) -> blocker::DownloadPrefs {
    blocker::DownloadPrefs {
        dir: s.download_dir.clone(),
        ask: s.download_ask,
        rules: s.download_rules.iter().map(|r| (r.ext.clone(), r.action.clone())).collect(),
    }
}

fn now_secs() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
}

fn privacy_of(s: &Settings) -> blocker::Privacy {
    let deny = |v: &str| v == "block";
    blocker::Privacy {
        gpc: s.gpc,
        strip: s.strip_tracking,
        https: s.https_upgrade,
        block_pings: s.block_pings,
        deny: [deny(&s.perm_location), deny(&s.perm_camera), deny(&s.perm_mic), deny(&s.perm_notifications)],
    }
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

    /// While a woken tab loads, the UI shows its last picture, blurred.
    fn send_preview(&self, id: u32) {
        let Some(t) = self.tab(id) else { return };
        if t.url.is_empty() || (t.view.is_some() && t.shown) {
            return;
        }
        let img = t.preview.as_ref().map(|b| {
            format!("data:image/jpeg;base64,{}", base64::engine::general_purpose::STANDARD.encode(b))
        });
        self.send(json!({ "t": "preview", "id": id, "img": img }));
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
                    "private": t.private,
                    "audible": t.audible && t.view.is_some(), "muted": t.muted, "volume": t.volume,
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
            "max": self.maximized(), "fs": self.fullscreen,
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

    /// A Google account appeared: once per account, offer to bring in bookmarks and history
    /// from the browser profiles on this PC, the ones signed in to the same account first.
    /// (Google doesn't let other browsers download Chrome Sync data.)
    fn offer_import(&mut self, new: &[account::Account]) {
        let Some(a) = new.iter().find(|a| !self.settings.import_offered.iter().any(|e| e.eq_ignore_ascii_case(&a.email))) else {
            return;
        };
        let sources = importer::detect();
        let mut s = self.settings.clone();
        s.import_offered.push(a.email.clone());
        self.apply_settings(s);
        if sources.is_empty() {
            return;
        }
        let (mut same, others): (Vec<_>, Vec<_>) = sources.into_iter().partition(|x| x.email.eq_ignore_ascii_case(&a.email));
        same.extend(others);
        self.send(json!({ "t": "importOffer", "account": a, "sources": same }));
    }

    fn maximized(&self) -> bool {
        self.zoom.is_some() || self.window.is_maximized()
    }

    fn fills_work_area(&self) -> bool {
        let Some((x, y, w, h)) = glass::work_area(&self.window) else { return false };
        let pos = self.window.outer_position().unwrap_or_default();
        let size = self.window.outer_size();
        (pos.x - x).abs() <= 2 && (pos.y - y).abs() <= 2 && (size.width as i32 - w).abs() <= 2 && (size.height as i32 - h).abs() <= 2
    }

    /// Fill the monitor's work area (on) or go back to the size before (off).
    fn set_zoom(&mut self, on: bool) {
        if on {
            let Some((x, y, w, h)) = glass::work_area(&self.window) else { return };
            if self.zoom.is_none() {
                let pos = self.window.outer_position().unwrap_or_default();
                self.zoom = Some((pos, self.window.inner_size()));
            }
            self.window.set_outer_position(tao::dpi::PhysicalPosition::new(x, y));
            self.window.set_inner_size(tao::dpi::PhysicalSize::new(w.max(1) as u32, h.max(1) as u32));
        } else if let Some((pos, size)) = self.zoom.take() {
            self.window.set_inner_size(size);
            self.window.set_outer_position(pos);
        }
        self.push_state();
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
        let kept: Vec<&Tab> = self.tabs.iter().filter(|t| !t.private).collect();
        let tabs = kept.iter().map(|t| SessionTab { url: t.url.clone(), title: t.title.clone() }).collect();
        let active = kept.iter().position(|t| t.id == self.active).unwrap_or(0);
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
        let private = self.tabs[idx].private;
        let s = &self.settings;
        let script = privacy::page_script(s.gpc, s.fingerprint, self.seed);
        let view = tabs::create(
            id,
            ViewOpts { url: &url, bounds: vp, visible: false, theme, private, zoom: s.page_zoom, script },
            &mut self.ctx,
            &self.window,
            &self.proxy,
            &self.shield,
        );
        mark("page view created");
        if let Some(v) = &view {
            if private {
                // A private tab has its own profile: nothing saved, same tracking prevention.
                tabs::set_autofill(v, false, false);
                tabs::set_tracking(v, &self.settings.tracking);
            } else {
                tabs::set_autofill(v, self.settings.save_passwords, self.settings.autofill);
            }
        }
        let t = &mut self.tabs[idx];
        t.view = view;
        t.loading = t.view.is_some();
        t.blocked = 0;
        t.blocked_log.clear();
        t.shown = false;
        t.reveal_by = t.view.is_some().then(|| Instant::now() + Duration::from_millis(2500));
        t.audible = false;
        if let Some(v) = &t.view {
            tabs::set_sound(v, t.volume, t.muted);
        }
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
        self.open_tab(url, next_to_active, self.settings.always_private)
    }

    fn open_tab(&mut self, url: Option<String>, next_to_active: bool, private: bool) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let pos = if next_to_active {
            self.tabs.iter().position(|t| t.id == self.active).map_or(self.tabs.len(), |i| i + 1)
        } else {
            self.tabs.len()
        };
        let blank = url.is_none();
        let mut tab = Tab::new(id, url.unwrap_or_default(), String::new());
        tab.private = private;
        self.tabs.insert(pos, tab);
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
        // Remember how the tab you're leaving looks, for when it wakes from sleep.
        if prev != id {
            if let Some(t) = self.tab(prev) {
                if let (Some(v), true, false) = (&t.view, t.shown, t.url.is_empty()) {
                    tabs::snapshot(v, prev, self.proxy.clone(), true);
                }
            }
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
        self.send_preview(id);
        if let Some(v) = self.active_view() {
            let _ = v.set_memory_usage_level(MemoryUsageLevel::Normal);
        }
        self.show_active();
        self.push_state();
    }

    fn close_tab(&mut self, id: u32) {
        let Some(i) = self.tabs.iter().position(|t| t.id == id) else { return };
        let t = self.tabs.remove(i);
        if !t.url.is_empty() && !t.private {
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
            t.audible = false;
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
        if self.pruned_at.is_none_or(|t| t.elapsed() > Duration::from_secs(3600)) {
            self.pruned_at = Some(Instant::now());
            self.store.prune_history(self.settings.history_days);
        }
        let mins = self.settings.sleep_minutes;
        if mins > 0 {
            let limit = Duration::from_secs(u64::from(mins) * 60);
            let mut slept = false;
            for t in self.tabs.iter_mut() {
                if t.id != self.active
                    && !t.private
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
        self.bookmarks_changed();
    }

    /// The whole bookmarks bar, as flat folder and bookmark lists.
    fn send_bar(&self) {
        let (folders, items) = self.store.bar();
        self.send(json!({ "t": "bar", "folders": folders, "items": items }));
    }

    fn bookmarks_changed(&self) {
        self.send_bar();
        self.send_list("bookmarks", "");
        self.push_state();
    }

    /// What the shield blocked on the active page, grouped by domain, busiest first.
    fn send_shield_info(&self) {
        let Some(t) = self.tab(self.active) else { return };
        let mut groups: Vec<(String, u32, Vec<&'static str>)> = Vec::new();
        for (url, kind) in &t.blocked_log {
            let host = util::host_of(url);
            let i = match groups.iter().position(|g| g.0 == host) {
                Some(i) => i,
                None => {
                    groups.push((host, 0, Vec::new()));
                    groups.len() - 1
                }
            };
            let g = &mut groups[i];
            g.1 += 1;
            if !g.2.contains(kind) {
                g.2.push(kind);
            }
        }
        groups.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let items: Vec<_> = groups.iter().map(|(h, n, k)| json!({ "h": h, "n": n, "k": k })).collect();
        self.send(json!({
            "t": "shieldInfo", "id": t.id, "n": t.blocked,
            "host": util::host_of(&t.url), "items": items,
        }));
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
            // Extra candidates so the UI can skip hidden and pinned sites.
            "top" => self.store.top_sites(40),
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
        if old.backdrop != s.backdrop || old.theme != s.theme {
            self.backdrop = glass::apply(&self.window, &s.backdrop, is_dark(&s.theme));
        }
        if old.window_opacity != s.window_opacity {
            glass::set_window_opacity(&self.window, s.window_opacity);
        }
        if old.page_zoom != s.page_zoom {
            for v in self.tabs.iter().filter_map(|t| t.view.as_ref()) {
                tabs::set_zoom(v, s.page_zoom);
            }
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
            for t in self.tabs.iter().filter(|t| t.private) {
                if let Some(v) = &t.view {
                    tabs::set_tracking(v, &s.tracking);
                }
            }
        }
        self.shield.privacy.set(privacy_of(s));
        *self.shield.downloads.borrow_mut() = download_prefs(s);
        if old.filter_lists != s.filter_lists {
            self.toast("Updating filter lists...");
            blocker::load(self.store.dir.clone(), true, s.filter_lists.clone(), self.proxy.clone());
        }
        let restart = old.share_processes != s.share_processes || old.webrtc_protect != s.webrtc_protect || old.doh != s.doh;
        let rescript = old.gpc != s.gpc || old.fingerprint != s.fingerprint;
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
        if old.history_days != s.history_days && s.history_days > 0 {
            let n = self.store.prune_history(s.history_days);
            if n > 0 {
                self.toast(&format!("Deleted {n} old history entries"));
            }
        }
        if rescript {
            self.reload_views();
        }
        if restart {
            self.toast("Restart Lurt to apply this");
        }
        self.store.save_settings(&self.settings);
        self.send_settings();
        self.push_state();
    }

    /// The download folder you chose, or the Windows Downloads folder.
    fn download_dir(&self) -> std::path::PathBuf {
        let d = &self.settings.download_dir;
        if !d.is_empty() && std::path::Path::new(d).is_dir() {
            return d.into();
        }
        dirs::download_dir().unwrap_or_else(|| self.store.dir.clone())
    }

    fn send_downloads(&self) {
        let exists: Vec<bool> = self.downloads.iter().map(|d| d.state != "done" || std::path::Path::new(&d.path).exists()).collect();
        let list: Vec<Value> = self
            .downloads
            .iter()
            .zip(exists)
            .map(|(d, ok)| {
                json!({ "id": d.id, "name": d.name, "path": d.path, "url": d.url, "total": d.total, "recv": d.recv,
                        "state": d.state, "reason": d.reason, "at": d.at, "gone": !ok })
            })
            .collect();
        let dir = self.download_dir().to_string_lossy().into_owned();
        self.send(json!({ "t": "downloads", "list": list, "dir": dir }));
    }

    fn remember_icon(&mut self, host: &str, url: &str) {
        if self.store.set_icon(host, url) {
            self.send(json!({ "t": "icon", "host": host, "url": url }));
        }
    }

    /// Recreate page views (for settings baked into them): background tabs reload when
    /// you switch to them, the current one now. Private tabs keep theirs.
    fn reload_views(&mut self) {
        for t in self.tabs.iter_mut().filter(|t| !t.private && t.view.is_some()) {
            t.view = None;
            t.loading = false;
            t.shown = false;
            t.reveal_by = None;
        }
        self.wake(self.active);
        self.show_active();
    }

    /// On quit: delete what Settings > Privacy says to, then let the loop exit.
    fn begin_exit_clear(&mut self) {
        let s = &self.settings;
        if s.exit_clear_history {
            self.store.clear_history();
        }
        if !s.exit_clear_cookies {
            self.exit_clear = Some(Instant::now());
            return;
        }
        self.window.set_visible(false);
        self.exit_clear = Some(Instant::now() + Duration::from_secs(4));
        let p = self.proxy.clone();
        tabs::clear_data(
            &self.chrome,
            tabs::site_data(),
            move || {
                let _ = p.send_event(Ev::Cleared);
            },
        );
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
            "ctrl+shift+n" => {
                self.open_tab(None, false, true);
            }
            "ctrl+w" => self.close_tab(self.active),
            "ctrl+l" | "alt+d" | "f6" => self.focus_address(),
            "ctrl+tab" => self.cycle(1),
            "ctrl+shift+tab" => self.cycle(-1),
            "ctrl+d" => self.send(json!({ "t": "starPop" })),
            "ctrl+h" => self.send(json!({ "t": "panel", "p": "history" })),
            // Like Chrome: Ctrl+Shift+B shows or hides the bookmarks bar.
            "ctrl+shift+b" => {
                let mut s = self.settings.clone();
                s.bookmarks_bar = if s.bookmarks_bar == "never" { "always" } else { "never" }.into();
                self.apply_settings(s);
            }
            "ctrl+shift+o" => self.send(json!({ "t": "panel", "p": "bookmarks" })),
            "ctrl+comma" => self.send(json!({ "t": "panel", "p": "settings" })),
            "ctrl+j" => self.send(json!({ "t": "panel", "p": "downloads" })),
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
                    // Signing in or out on a Google page: refresh the account button.
                    let host = util::host_of(&url);
                    if host == "google.com" || host.ends_with(".google.com") {
                        self.account_at = Some(Instant::now() + Duration::from_millis(800));
                    }
                }
                let Some(t) = self.tab_mut(id) else { return };
                t.loading = started;
                if started {
                    t.blocked = 0;
                    t.blocked_log.clear();
                } else if !util::is_internal(&url) && !t.private {
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
                    if !t.loading && !util::is_internal(&url) && !t.private {
                        let title = t.title.clone();
                        self.store.add_visit(&url, &title);
                    }
                    self.push_state();
                }
            }
            Ev::Audio(id, on) => {
                if let Some(t) = self.tab_mut(id) {
                    t.audible = on;
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
            Ev::Zoom(id, percent) => {
                // Only the page you're looking at can be zoomed by you; other tabs report
                // the zoom Lurt just gave them.
                let percent = percent.clamp(25, 500);
                if id == self.active && percent != self.settings.page_zoom {
                    let mut s = self.settings.clone();
                    s.page_zoom = percent;
                    self.apply_settings(s);
                    self.toast(&format!("Zoom {percent}%"));
                }
            }
            Ev::NewTab(from, url) => {
                // Links from a private tab stay private.
                let private = self.settings.always_private || self.tab(from).is_some_and(|t| t.private);
                self.open_tab(Some(url), true, private);
            }
            Ev::Redirect(id, url) => {
                if let Some(v) = self.tab(id).and_then(|t| t.view.as_ref()) {
                    let _ = v.load_url(&url);
                }
            }
            Ev::Cleared => self.exit_cleared = true,
            Ev::FoundIcon(host, url) => self.remember_icon(&host, &url),
            Ev::DlStart { tab, dl, mut path, url, total, ask, open } => {
                if ask {
                    match dialogs::save_file(self.window.hwnd() as isize, &path) {
                        Some(p) => {
                            tabs::finish_ask(dl, Some(&p));
                            path = p;
                        }
                        None => {
                            tabs::finish_ask(dl, None);
                            tabs::forget_download(dl);
                            return;
                        }
                    }
                }
                let name = std::path::Path::new(&path).file_name().map_or_else(|| path.clone(), |n| n.to_string_lossy().into_owned());
                let private = self.tab(tab).is_some_and(|t| t.private);
                self.toast(&format!("Downloading {name}"));
                self.downloads.insert(0, store::Download {
                    id: dl, name: name.clone(), path, url, total, recv: 0, state: "progress".into(), reason: String::new(),
                    at: now_secs(), private, open,
                });
                self.send(json!({ "t": "dlStart", "id": dl, "name": name }));
                self.send_downloads();
            }
            Ev::DlBlocked { tab, name, url } => {
                let ext = std::path::Path::new(&name).extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
                self.toast(&format!("Blocked {name}: .{ext} downloads are turned off"));
                let private = self.tab(tab).is_some_and(|t| t.private);
                self.downloads.insert(0, store::Download {
                    id: 1_000_000 + self.downloads.len() as u32 + now_secs() as u32 % 1_000_000, name, path: String::new(), url,
                    state: "failed".into(), reason: format!(".{ext} downloads are turned off"), at: now_secs(), private,
                    ..Default::default()
                });
                self.store.save_downloads(&self.downloads);
                self.send_downloads();
            }
            Ev::DlProgress(dl, recv, total) => {
                if let Some(d) = self.downloads.iter_mut().find(|d| d.id == dl) {
                    d.recv = recv;
                    d.total = total;
                }
                if self.dl_push_at.is_none() {
                    self.dl_push_at = Some(Instant::now() + Duration::from_millis(250));
                }
            }
            Ev::DlState(dl, state, reason, path) => {
                let Some(d) = self.downloads.iter_mut().find(|d| d.id == dl) else { return };
                d.state = state.into();
                d.reason = reason.into();
                if !path.is_empty() {
                    d.path = path;
                }
                if state == "done" && d.total <= 0 {
                    d.total = d.recv;
                }
                if matches!(state, "done" | "failed" | "canceled") {
                    tabs::forget_download(dl);
                    if state == "done" {
                        d.recv = d.total.max(d.recv);
                        if d.open {
                            let _ = std::process::Command::new("explorer").arg(&d.path).spawn();
                        }
                        let name = d.name.clone();
                        self.send(json!({ "t": "dlDone", "id": dl, "name": name }));
                    }
                }
                self.store.save_downloads(&self.downloads);
                self.send_downloads();
            }
            Ev::Blocked(id, url, kind) => {
                if let Some(t) = self.tab_mut(id) {
                    t.blocked += 1;
                    if t.blocked_log.len() < 2000 {
                        t.blocked_log.push((url, kind));
                    }
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
            Ev::Snapshot(id, bytes, true) => {
                if !bytes.is_empty() {
                    if let Some(t) = self.tab_mut(id) {
                        t.preview = Some(bytes);
                    }
                }
            }
            Ev::Snapshot(id, bytes, false) => {
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
            Ev::Accounts(result) => {
                let err = result.as_ref().err().cloned();
                if let Ok(list) = result {
                    let new: Vec<account::Account> =
                        list.iter().filter(|a| !self.accounts.iter().any(|o| o.email == a.email)).cloned().collect();
                    self.accounts = list;
                    self.offer_import(&new);
                }
                self.send(json!({ "t": "account", "accounts": self.accounts, "ok": err.is_none(), "err": err }));
            }
            Ev::SignedOut => {
                self.accounts.clear();
                self.toast("Signed out of Google");
                self.send(json!({ "t": "account", "accounts": self.accounts, "ok": true }));
                // Reload open Google pages so they show you signed out too.
                for t in &self.tabs {
                    if let Some(v) = &t.view {
                        let h = util::host_of(&t.url);
                        if h == "google.com" || h.ends_with(".google.com") || h.ends_with("youtube.com") {
                            let _ = v.reload();
                        }
                    }
                }
                self.account_at = Some(Instant::now() + Duration::from_millis(500));
            }
            Ev::FavData(host, data) => {
                self.send(json!({ "t": "favData", "host": host, "data": data }));
            }
            Ev::ImportedShortcuts(list) => {
                // Add them to the start page favourites, skipping sites already pinned.
                let mut s = self.settings.clone();
                for (title, url) in list {
                    let host = util::host_of(&url);
                    if host.is_empty() || s.shortcuts.iter().any(|x| util::host_of(&x.url) == host && x.url == url) {
                        continue;
                    }
                    s.hidden_tiles.retain(|h| *h != host);
                    let title = if title.trim().is_empty() { host.clone() } else { title };
                    s.shortcuts.push(store::Shortcut { title, url });
                }
                self.apply_settings(s);
            }
            Ev::Imported(result, from) => {
                let text = match result {
                    Ok((b, h)) => format!("Imported from {from}: {b} new bookmarks, {h} history entries"),
                    Err(e) => format!("Import from {from} failed: {e}"),
                };
                self.send(json!({ "t": "imported", "text": text }));
                self.send_list("top", "");
                self.bookmarks_changed();
            }
        }
    }

    fn on_page(&mut self, id: u32, raw: &str) {
        // Websites can post here too, so only a few harmless messages are accepted.
        let Ok(m) = serde_json::from_str::<Value>(raw) else { return };
        match m["t"].as_str().unwrap_or("") {
            // A new document: give it the tab's volume.
            "volumeAsk" => {
                if let Some(t) = self.tab(id).filter(|t| t.volume != 100) {
                    if let Some(v) = &t.view {
                        tabs::set_sound(v, t.volume, t.muted);
                    }
                }
            }
            "key" if id == self.active => {
                let k = m["k"].as_str().unwrap_or("").to_string();
                self.shortcut(&k);
            }
            "down" if id == self.active => self.send(json!({ "t": "pageDown" })),
            "fav" => {
                let u = m["u"].as_str().unwrap_or("");
                if u.len() < 2048 && (u.starts_with("https://") || u.starts_with("http://")) {
                    let u = u.to_string();
                    // Remember icons a page declares itself, for favourites and bookmarks.
                    let private = self.tab(id).is_some_and(|t| t.private);
                    let host = util::host_of(&u);
                    let page_host = self.tab(id).map(|t| util::host_of(&t.url)).unwrap_or_default();
                    if !private && !page_host.is_empty() && !u.ends_with(&format!("{host}/favicon.ico")) {
                        self.remember_icon(&page_host, &u);
                    }
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
                self.send_bar();
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
            // Volume and/or mute for one tab.
            "sound" => {
                if let Some(t) = self.tab_mut(id) {
                    if let Some(v) = m["volume"].as_u64() {
                        t.volume = v.min(100) as u8;
                    }
                    if let Some(on) = m["muted"].as_bool() {
                        t.muted = on;
                    }
                    if let Some(v) = &t.view {
                        tabs::set_sound(v, t.volume, t.muted);
                    }
                    self.push_state();
                }
            }
            "tabOrder" => {
                let ids: Vec<u64> = m["ids"].as_array().map(|a| a.iter().filter_map(Value::as_u64).collect()).unwrap_or_default();
                // Tabs the UI didn't list (opened while dragging) go to the end.
                self.tabs.sort_by_key(|t| ids.iter().position(|&i| i == u64::from(t.id)).unwrap_or(ids.len()));
                self.push_state();
                self.save_session();
            }
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
                "max" => {
                    if self.window.is_maximized() {
                        self.window.set_maximized(false);
                    } else {
                        let on = self.zoom.is_none();
                        self.set_zoom(on);
                    }
                }
                "close" => self.quit = true,
                "drag" => {
                    // Dragging a maximized window restores it under the cursor first.
                    if let Some((_, size)) = self.zoom.take() {
                        let pos = self.window.outer_position().unwrap_or_default();
                        let width = self.window.outer_size().width.max(1);
                        let (cx, _) = glass::cursor();
                        let frac = f64::from(cx - pos.x) / f64::from(width);
                        let x = cx - (frac * f64::from(size.width)) as i32;
                        self.window.set_inner_size(size);
                        self.window.set_outer_position(tao::dpi::PhysicalPosition::new(x, pos.y));
                        self.push_state();
                    }
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
                        // Hiding the page must not take keyboard focus from the address bar.
                        let _ = self.chrome.focus();
                    }
                    return;
                }
                self.overlay = m["open"].as_bool().unwrap_or(false);
                if self.overlay {
                    let ready = self.tab(self.active).is_some_and(|t| t.shown);
                    match self.active_view().filter(|_| ready) {
                        Some(v) => tabs::snapshot(v, self.active, self.proxy.clone(), false),
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
            "shieldInfo" => self.send_shield_info(),
            "winOpacity" => {
                let v = m["v"].as_u64().unwrap_or(100).min(100) as u8;
                glass::set_window_opacity(&self.window, v);
            }
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
            "account" => {
                self.send(json!({ "t": "account", "accounts": self.accounts, "ok": true }));
                self.account_at = Some(Instant::now());
            }
            "signOut" => account::sign_out(&self.chrome, self.proxy.clone()),
            "dataInfo" => {
                let (bookmarks, folders, history) = self.store.counts();
                let size = |p: std::path::PathBuf| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
                self.send(json!({
                    "t": "dataInfo", "bookmarks": bookmarks, "folders": folders, "history": history,
                    "dir": self.store.dir.to_string_lossy(), "dbBytes": size(self.store.db_path()),
                    "webDir": Store::webview_dir().to_string_lossy(),
                }));
            }
            "delBookmark" => {
                self.store.remove_bookmark(&s("url"));
                self.bookmarks_changed();
            }
            "bmAdd" => {
                let url = s("url");
                if !url.is_empty() {
                    self.store.add_bookmark(&url, &s("title"), m["parent"].as_i64().unwrap_or(0));
                    self.bookmarks_changed();
                }
            }
            // From the bookmark card: put the page in a new folder made there.
            "bmAddNew" => {
                let url = s("url");
                let name = s("folder");
                if !url.is_empty() && !name.trim().is_empty() {
                    let id = self.store.add_folder(name.trim(), m["parent"].as_i64().unwrap_or(0));
                    self.store.add_bookmark(&url, &s("title"), id);
                    self.send(json!({ "t": "bmFolderMade", "id": id }));
                    self.bookmarks_changed();
                }
            }
            "bmFolder" => {
                let title = Some(s("title")).filter(|t| !t.trim().is_empty()).unwrap_or_else(|| "New folder".into());
                self.store.add_folder(title.trim(), m["parent"].as_i64().unwrap_or(0));
                self.bookmarks_changed();
            }
            "bmRename" => {
                self.store.rename_folder(m["id"].as_i64().unwrap_or(-1), s("title").trim());
                self.bookmarks_changed();
            }
            "bmDelFolder" => {
                self.store.delete_folder(m["id"].as_i64().unwrap_or(-1));
                self.bookmarks_changed();
            }
            "bmEdit" => {
                let new_url = s("newUrl");
                if !new_url.is_empty() {
                    self.store.edit_bookmark(&s("url"), &new_url, s("title").trim());
                    self.bookmarks_changed();
                }
            }
            "bmOrder" => {
                let slots: Vec<Slot> = m["items"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| match (v["f"].as_i64(), v["u"].as_str()) {
                                (Some(id), _) => Some(Slot::Folder(id)),
                                (_, Some(u)) => Some(Slot::Mark(u.to_string())),
                                _ => None,
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                self.store.reorder(m["parent"].as_i64().unwrap_or(0), &slots);
                self.bookmarks_changed();
            }
            "delHistoryMany" => {
                let urls: Vec<String> = m["urls"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                let n = self.store.delete_history_many(&urls);
                self.send_list("history", &s("q"));
                self.toast(&format!("Deleted {n} {}", if n == 1 { "page" } else { "pages" }));
            }
            "clearHistoryRange" => {
                let n = self.store.clear_history_since(m["secs"].as_i64().unwrap_or(-1).max(0));
                self.send_list("history", &s("q"));
                self.send_list("top", "");
                self.toast(&format!("Deleted {n} history {}", if n == 1 { "entry" } else { "entries" }));
            }
            "clearHistory" => {
                self.store.clear_history();
                self.send_list("history", "");
                self.toast("History cleared");
            }
            "wipeAll" => {
                // Only when signed out of Google: the UI signs you out first.
                if !self.accounts.is_empty() {
                    self.toast("Sign out of Google first");
                    return;
                }
                // Cookies, cache, passwords, autofill and site data of the whole profile.
                let _ = self.chrome.clear_all_browsing_data();
                self.store.wipe();
                self.closed.clear();
                // Close every tab and start over with one new tab.
                let old: Vec<u32> = self.tabs.iter().map(|t| t.id).collect();
                self.new_tab(None, false);
                for id in old {
                    self.close_tab(id);
                }
                self.closed.clear();
                self.apply_settings(Settings::default());
                self.store.save_settings(&self.settings);
                self.bookmarks_changed();
                self.send_list("history", "");
                // The start page's favourites come from bookmarks and history: reload them.
                self.send_list("top", "");
                self.toast("All browser data deleted");
            }
            "clearData" => {
                // Not passwords or autofill: those have their own switch.
                tabs::clear_data(
                    &self.chrome,
                    tabs::site_data(),
                    || {},
                );
                self.toast("Cookies, cache and site data cleared");
            }
            "downloads" => self.send_downloads(),
            "dlPause" | "dlResume" | "dlCancel" => {
                let id = m["id"].as_u64().unwrap_or(0) as u32;
                tabs::download_op(id, match s("t").as_str() { "dlPause" => "pause", "dlResume" => "resume", _ => "cancel" });
                // Pausing doesn't always report a state change: show it right away.
                if let Some(d) = self.downloads.iter_mut().find(|d| d.id == id && (d.state == "progress" || d.state == "paused")) {
                    match s("t").as_str() {
                        "dlPause" => d.state = "paused".into(),
                        "dlResume" => d.state = "progress".into(),
                        _ => {}
                    }
                }
                self.send_downloads();
            }
            "dlOpen" | "dlShow" => {
                let id = m["id"].as_u64().unwrap_or(0) as u32;
                let Some(d) = self.downloads.iter().find(|d| d.id == id) else { return };
                let path = std::path::PathBuf::from(&d.path);
                if !path.exists() {
                    self.toast("The file was moved or deleted");
                    return;
                }
                let mut c = std::process::Command::new("explorer");
                if s("t") == "dlShow" {
                    c.arg("/select,");
                }
                let _ = c.arg(&path).spawn();
            }
            "dlFolder" => {
                let _ = std::process::Command::new("explorer").arg(self.download_dir()).spawn();
            }
            "dlPickDir" => {
                let start = self.download_dir().to_string_lossy().into_owned();
                if let Some(dir) = dialogs::pick_folder(self.window.hwnd() as isize, &start) {
                    let mut s = self.settings.clone();
                    s.download_dir = dir;
                    self.apply_settings(s);
                }
            }
            "dlRetry" => {
                let id = m["id"].as_u64().unwrap_or(0) as u32;
                if let Some(url) = self.downloads.iter().find(|d| d.id == id).map(|d| d.url.clone()) {
                    self.downloads.retain(|d| d.id != id);
                    match self.active_view() {
                        Some(v) => {
                            let _ = v.load_url(&url);
                        }
                        None => {
                            self.new_tab(Some(url), true);
                        }
                    }
                }
                self.store.save_downloads(&self.downloads);
                self.send_downloads();
            }
            "dlRemove" => {
                let id = m["id"].as_u64().unwrap_or(0) as u32;
                tabs::download_op(id, "cancel");
                tabs::forget_download(id);
                self.downloads.retain(|d| d.id != id);
                self.store.save_downloads(&self.downloads);
                self.send_downloads();
            }
            "dlClear" => {
                self.downloads.retain(|d| d.state == "progress" || d.state == "paused");
                self.store.save_downloads(&self.downloads);
                self.send_downloads();
            }
            "icons" => {
                let map: serde_json::Map<String, Value> = self.store.icons().into_iter().map(|(h, u)| (h, Value::String(u))).collect();
                self.send(json!({ "t": "icons", "map": map }));
            }
            "findIcon" => {
                let (host, proxy) = (s("host"), self.proxy.clone());
                if host.is_empty() || host.contains('/') {
                    return;
                }
                std::thread::spawn(move || {
                    let url = util::find_icon(&host).unwrap_or_default();
                    let _ = proxy.send_event(Ev::FoundIcon(host, url));
                });
            }
            "newPrivate" => {
                self.open_tab(None, false, true);
            }
            "settings" => {
                if let Ok(new) = serde_json::from_value::<Settings>(m["s"].clone()) {
                    self.apply_settings(new);
                }
            }
            "updateLists" => {
                self.toast("Updating filter lists...");
                blocker::load(self.store.dir.clone(), true, self.settings.filter_lists.clone(), self.proxy.clone());
            }
            "reloadMods" => {
                self.send(json!({ "t": "mods", "css": self.store.ui_mods() }));
                if let Some(v) = self.active_view() {
                    let _ = v.reload();
                }
                self.toast("Mods reloaded");
            }
            "skel" => {
                if let Ok(shape) = serde_json::from_value::<splash::Shape>(m["shape"].clone()) {
                    self.store.save_shape(&shape);
                }
            }
            "openData" => {
                let _ = std::process::Command::new("explorer").arg(&self.store.dir).spawn();
            }
            "exportSettings" => {
                let dir = dirs::download_dir().unwrap_or_else(|| self.store.dir.clone());
                let path = dir.join("Lurt settings.json");
                let text = serde_json::to_string_pretty(&json!({ "lurt": "settings", "version": 1, "settings": self.settings }));
                match text.map_err(|e| e.to_string()).and_then(|t| std::fs::write(&path, t).map_err(|e| e.to_string())) {
                    Ok(()) => {
                        self.toast("Settings saved to Downloads");
                        let _ = std::process::Command::new("explorer").arg("/select,").arg(&path).spawn();
                    }
                    Err(e) => self.toast(&format!("Couldn't save settings: {e}")),
                }
            }
            "importSettings" => {
                let v: Value = serde_json::from_str(&s("text")).unwrap_or(Value::Null);
                // A file saved by Lurt, or a bare settings.json.
                let raw = if v.get("lurt").and_then(Value::as_str) == Some("settings") { v["settings"].clone() } else { v };
                // Apply the keys Lurt knows on top of the current settings; ignore the rest.
                let mut cur = serde_json::to_value(&self.settings).unwrap_or(Value::Null);
                let mut known = 0;
                if let (Some(src), Some(dst)) = (raw.as_object(), cur.as_object_mut()) {
                    for (k, v) in src {
                        if dst.contains_key(k) {
                            dst.insert(k.clone(), v.clone());
                            known += 1;
                        }
                    }
                }
                match (known > 0).then(|| serde_json::from_value::<Settings>(cur).ok()).flatten() {
                    Some(new) => {
                        self.apply_settings(new);
                        self.toast("Settings loaded");
                    }
                    None => self.toast("That file doesn't contain Lurt settings"),
                }
            }
            "openMods" => {
                let _ = std::process::Command::new("explorer").arg(self.store.mods_dir()).spawn();
            }
            "devtools" => {
                if let Some(v) = self.active_view() {
                    v.open_devtools();
                }
            }
            "favColor" => {
                let (host, url, proxy) = (s("host"), s("url"), self.proxy.clone());
                if host.is_empty() || host.contains('/') {
                    return;
                }
                std::thread::spawn(move || {
                    let data = util::fetch_image(&url).or_else(|| util::fetch_image(&format!("https://{host}/favicon.ico")));
                    let _ = proxy.send_event(Ev::FavData(host, data));
                });
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
                            let batch = importer::read_profile(&src.id, &scratch);
                            if let Ok(b) = &batch {
                                let _ = proxy.send_event(Ev::ImportedShortcuts(b.shortcuts.clone()));
                            }
                            match batch.and_then(|b| importer::save(&db, &b)) {
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
                        let batch = importer::read_profile(&id, &scratch);
                        if let Ok(b) = &batch {
                            let _ = proxy.send_event(Ev::ImportedShortcuts(b.shortcuts.clone()));
                        }
                        batch.and_then(|b| importer::save(&db, &b))
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

/// Whether the theme setting resolves to dark ("system" follows Windows).
fn is_dark(theme: &str) -> bool {
    match theme {
        "dark" => true,
        "light" => false,
        _ => splash::system_dark(),
    }
}
