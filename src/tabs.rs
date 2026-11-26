//! Page views (one WebView2 per awake tab) and the native hooks Lurt adds to them:
//! request blocking, fullscreen, URL/history tracking and page snapshots.

use std::{rc::Rc, time::Instant};

use tao::{event_loop::EventLoopProxy, window::Window};
use webview2_com::{
    CapturePreviewCompletedHandler, ContainsFullScreenElementChangedEventHandler,
    HistoryChangedEventHandler, Microsoft::Web::WebView2::Win32::*, SourceChangedEventHandler,
    WebResourceRequestedEventHandler, take_pwstr,
};
use windows::{
    Win32::System::Com::{IStream, STATFLAG_NONAME, STATSTG, STREAM_SEEK_SET},
    Win32::UI::Shell::SHCreateMemStream,
    core::{BOOL, HSTRING, Interface, PWSTR, w},
};
use wry::{
    NewWindowResponse, PageLoadEvent, Rect, Theme, WebContext, WebView, WebViewBuilder,
    WebViewBuilderExtWindows, WebViewExtWindows,
};

use crate::{Ev, blocker::Shield, util};

const CONTENT_JS: &str = include_str!("content.js");

pub struct Tab {
    pub id: u32,
    /// `None` while the tab is a fresh start page or asleep.
    pub view: Option<WebView>,
    pub url: String,
    pub title: String,
    pub fav: String,
    pub loading: bool,
    pub blocked: u32,
    /// What was blocked on the current page: (request URL, resource type). Capped.
    pub blocked_log: Vec<(String, &'static str)>,
    pub back: bool,
    pub fwd: bool,
    pub last_active: Instant,
    /// A fresh page view stays hidden (the UI shows a page skeleton) until it has
    /// content, so you never see a blank white rectangle.
    pub shown: bool,
    /// Reveal anyway at this time, for pages that are slow to render.
    pub reveal_by: Option<Instant>,
}

impl Tab {
    pub fn new(id: u32, url: String, title: String) -> Self {
        Self {
            id,
            view: None,
            url,
            title,
            fav: String::new(),
            loading: false,
            blocked: 0,
            blocked_log: Vec::new(),
            back: false,
            fwd: false,
            last_active: Instant::now(),
            shown: false,
            reveal_by: None,
        }
    }
}

pub struct ViewOpts<'a> {
    pub url: &'a str,
    pub bounds: Rect,
    pub visible: bool,
    pub theme: Theme,
}

pub fn create(
    id: u32,
    opts: ViewOpts,
    ctx: &mut WebContext,
    window: &Window,
    proxy: &EventLoopProxy<Ev>,
    shield: &Rc<Shield>,
) -> Option<WebView> {
    let p = proxy.clone();
    let ipc = move |req: wry::http::Request<String>| {
        let _ = p.send_event(Ev::Page(id, req.into_body()));
    };
    let p = proxy.clone();
    let title = move |t: String| {
        let _ = p.send_event(Ev::Title(id, t));
    };
    let p = proxy.clone();
    let load = move |e: PageLoadEvent, url: String| {
        let _ = p.send_event(Ev::Load(id, matches!(e, PageLoadEvent::Started), url));
    };
    let p = proxy.clone();
    let new_window = move |url: String, f: wry::NewWindowFeatures| {
        // Sized popups (sign-in windows) keep their opener; everything else becomes a tab.
        if f.size.is_some() {
            NewWindowResponse::Allow
        } else {
            let _ = p.send_event(Ev::NewTab(url));
            NewWindowResponse::Deny
        }
    };

    let view = WebViewBuilder::new_with_web_context(ctx)
        .with_url(opts.url)
        .with_bounds(opts.bounds)
        .with_visible(opts.visible)
        .with_focused(opts.visible)
        .with_initialization_script_for_main_only(CONTENT_JS, true)
        .with_ipc_handler(ipc)
        .with_document_title_changed_handler(title)
        .with_on_page_load_handler(load)
        .with_new_window_req_handler(new_window)
        .with_devtools(true)
        .with_theme(opts.theme)
        .with_scroll_bar_style(wry::ScrollBarStyle::FluentOverlay)
        .build_as_child(window)
        .ok()?;

    if let Err(e) = unsafe { hook(&view, id, proxy.clone(), shield.clone()) } {
        eprintln!("lurt: native hooks failed for tab {id}: {e}");
    }
    Some(view)
}

unsafe fn hook(
    view: &WebView,
    id: u32,
    proxy: EventLoopProxy<Ev>,
    shield: Rc<Shield>,
) -> windows::core::Result<()> {
    let wv = view.webview();
    let env = view.environment();
    let mut token = Default::default();

    unsafe {
        // --- request blocking ---
        let filter = HSTRING::from("*");
        match wv.cast::<ICoreWebView2_22>() {
            Ok(w22) => w22.AddWebResourceRequestedFilterWithRequestSourceKinds(
                &filter,
                COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                COREWEBVIEW2_WEB_RESOURCE_REQUEST_SOURCE_KINDS_ALL,
            )?,
            Err(_) => wv.AddWebResourceRequestedFilter(&filter, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL)?,
        }
        let p = proxy.clone();
        wv.add_WebResourceRequested(
            &WebResourceRequestedEventHandler::create(Box::new(move |sender, args| {
                let (Some(sender), Some(args)) = (sender, args) else { return Ok(()) };
                if !shield.enabled.get() {
                    return Ok(());
                }
                let engine = shield.engine.borrow();
                let Some(engine) = engine.as_ref() else { return Ok(()) };

                let mut ctx = COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL;
                args.ResourceContext(&mut ctx)?;
                let kind = match ctx {
                    // Never block the page you asked for; ad iframes lose their
                    // scripts and images below, and cosmetic rules hide the frame.
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT => return Ok(()),
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_STYLESHEET => "stylesheet",
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_IMAGE => "image",
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_MEDIA => "media",
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FONT => "font",
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_SCRIPT => "script",
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_XML_HTTP_REQUEST
                    | COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FETCH => "xhr",
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_WEBSOCKET => "websocket",
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_PING => "ping",
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_CSP_VIOLATION_REPORT => "csp_report",
                    _ => "other",
                };

                let req = args.Request()?;
                let mut s = PWSTR::null();
                req.Uri(&mut s)?;
                let uri = take_pwstr(s);
                if !uri.starts_with("http") && !uri.starts_with("ws") {
                    return Ok(());
                }
                let mut s = PWSTR::null();
                sender.Source(&mut s)?;
                let source = take_pwstr(s);
                if shield.allowed(&util::host_of(&source)) {
                    return Ok(());
                }
                let mut s = PWSTR::null();
                req.Method(&mut s)?;
                let method = take_pwstr(s);

                if let Ok(r) = adblock::request::Request::new(&uri, &source, kind, &method) {
                    if engine.check_network_request(&r).should_block() {
                        let resp = env.CreateWebResourceResponse(None, 403, w!("Blocked"), w!(""))?;
                        args.SetResponse(&resp)?;
                        let _ = p.send_event(Ev::Blocked(id, uri.clone(), kind));
                    }
                }
                Ok(())
            })),
            &mut token,
        )?;

        // --- page fullscreen (videos) ---
        let p = proxy.clone();
        wv.add_ContainsFullScreenElementChanged(
            &ContainsFullScreenElementChangedEventHandler::create(Box::new(move |sender, _| {
                if let Some(s) = sender {
                    let mut b = BOOL(0);
                    s.ContainsFullScreenElement(&mut b)?;
                    let _ = p.send_event(Ev::Fullscreen(id, b.as_bool()));
                }
                Ok(())
            })),
            &mut token,
        )?;

        // --- URL changes, including single-page-app navigation ---
        let p = proxy.clone();
        wv.add_SourceChanged(
            &SourceChangedEventHandler::create(Box::new(move |sender, _| {
                if let Some(s) = sender {
                    let mut u = PWSTR::null();
                    s.Source(&mut u)?;
                    let _ = p.send_event(Ev::Source(id, take_pwstr(u)));
                }
                Ok(())
            })),
            &mut token,
        )?;

        // --- back/forward availability ---
        let p = proxy;
        wv.add_HistoryChanged(
            &HistoryChangedEventHandler::create(Box::new(move |_, _| {
                let _ = p.send_event(Ev::History(id));
                Ok(())
            })),
            &mut token,
        )?;
    }
    Ok(())
}

/// Password saving and form autofill (WebView2's built-in, stored in the Lurt profile).
pub fn set_autofill(view: &WebView, passwords: bool, general: bool) {
    unsafe {
        if let Ok(s) = view.webview().Settings().and_then(|s| s.cast::<ICoreWebView2Settings4>()) {
            let _ = s.SetIsPasswordAutosaveEnabled(passwords);
            let _ = s.SetIsGeneralAutofillEnabled(general);
        }
    }
}

/// Browser-wide tracking prevention, built into WebView2. Applies to the shared profile.
pub fn set_tracking(view: &WebView, level: &str) {
    let level = match level {
        "strict" => COREWEBVIEW2_TRACKING_PREVENTION_LEVEL_STRICT,
        "basic" => COREWEBVIEW2_TRACKING_PREVENTION_LEVEL_BASIC,
        "off" => COREWEBVIEW2_TRACKING_PREVENTION_LEVEL_NONE,
        _ => COREWEBVIEW2_TRACKING_PREVENTION_LEVEL_BALANCED,
    };
    unsafe {
        let r = view
            .webview()
            .cast::<ICoreWebView2_13>()
            .and_then(|w| w.Profile())
            .and_then(|p| p.cast::<ICoreWebView2Profile3>())
            .and_then(|p| p.SetPreferredTrackingPreventionLevel(level));
        if let Err(e) = r {
            eprintln!("lurt: tracking prevention: {e}");
        }
    }
}

pub fn is_playing_audio(view: &WebView) -> bool {
    unsafe {
        let mut b = BOOL(0);
        view.webview()
            .cast::<ICoreWebView2_8>()
            .and_then(|w| w.IsDocumentPlayingAudio(&mut b))
            .is_ok()
            && b.as_bool()
    }
}

/// Take a JPEG of the page; it arrives as `Ev::Snapshot`.
pub fn snapshot(view: &WebView, id: u32, proxy: EventLoopProxy<Ev>) {
    unsafe {
        let Some(stream) = SHCreateMemStream(None) else {
            let _ = proxy.send_event(Ev::Snapshot(id, Vec::new()));
            return;
        };
        let s2 = stream.clone();
        let p = proxy.clone();
        let r = view.webview().CapturePreview(
            COREWEBVIEW2_CAPTURE_PREVIEW_IMAGE_FORMAT_JPEG,
            &stream,
            &CapturePreviewCompletedHandler::create(Box::new(move |res| {
                let bytes = if res.is_ok() { read_stream(&s2) } else { Vec::new() };
                let _ = p.send_event(Ev::Snapshot(id, bytes));
                Ok(())
            })),
        );
        if r.is_err() {
            let _ = proxy.send_event(Ev::Snapshot(id, Vec::new()));
        }
    }
}

fn read_stream(s: &IStream) -> Vec<u8> {
    unsafe {
        let mut stat = STATSTG::default();
        if s.Stat(&mut stat, STATFLAG_NONAME).is_err() {
            return Vec::new();
        }
        let len = stat.cbSize as usize;
        if s.Seek(0, STREAM_SEEK_SET, None).is_err() {
            return Vec::new();
        }
        let mut buf = vec![0u8; len];
        let mut read = 0u32;
        let _ = s.Read(buf.as_mut_ptr().cast(), len as u32, Some(&mut read));
        buf.truncate(read as usize);
        buf
    }
}
