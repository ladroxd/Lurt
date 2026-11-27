//! Page views (one WebView2 per awake tab) and the native hooks Lurt adds to them:
//! request blocking, fullscreen, URL/history tracking and page snapshots.

use std::{rc::Rc, time::Instant};

use tao::{event_loop::EventLoopProxy, window::Window};
use webview2_com::{
    CapturePreviewCompletedHandler, ContainsFullScreenElementChangedEventHandler,
    BytesReceivedChangedEventHandler, ClearBrowsingDataCompletedHandler, DownloadStartingEventHandler,
    HistoryChangedEventHandler, IsDocumentPlayingAudioChangedEventHandler, StateChangedEventHandler, Microsoft::Web::WebView2::Win32::*,
    NavigationCompletedEventHandler, NavigationStartingEventHandler, PermissionRequestedEventHandler,
    FaviconChangedEventHandler, GetFaviconCompletedHandler, SourceChangedEventHandler, WebResourceRequestedEventHandler, ZoomFactorChangedEventHandler, take_pwstr,
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
const VOLUME_JS: &str = include_str!("volume.js");

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
    /// A JPEG of the page taken when you last left the tab, shown blurred while it wakes.
    pub preview: Option<Vec<u8>>,
    pub back: bool,
    pub fwd: bool,
    pub last_active: Instant,
    /// A fresh page view stays hidden (the UI shows a page skeleton) until it has
    /// content, so you never see a blank white rectangle.
    pub shown: bool,
    /// Reveal anyway at this time, for pages that are slow to render.
    pub reveal_by: Option<Instant>,
    /// A private tab: its own cookies (gone when the last private tab closes), no history.
    pub private: bool,
    /// The page is playing sound (even if you muted it).
    pub audible: bool,
    pub muted: bool,
    /// Lurt's volume for this tab, 0..=100, on top of the site's own.
    pub volume: u8,
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
            preview: None,
            back: false,
            fwd: false,
            last_active: Instant::now(),
            shown: false,
            reveal_by: None,
            private: false,
            audible: false,
            muted: false,
            volume: 100,
        }
    }
}

pub struct ViewOpts<'a> {
    pub url: &'a str,
    pub bounds: Rect,
    pub visible: bool,
    pub theme: Theme,
    pub private: bool,
    /// Page zoom in percent.
    pub zoom: u32,
    /// The privacy script for every frame (see `privacy::page_script`).
    pub script: Option<String>,
}

/// WebView2 engine arguments, the same for every view (views sharing a profile must match).
/// Starts with wry's defaults: no mini menu and SmartScreen, autoplay allowed.
pub static ENGINE_ARGS: std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub fn engine_args(s: &crate::store::Settings) -> String {
    let mut a = String::from("--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --autoplay-policy=no-user-gesture-required");
    if s.share_processes {
        // Cross-site iframes stay in their page's renderer instead of a process each
        // ("WebView2 Subframe: stripe.network" and so on): about half the memory.
        a.push_str(" --disable-site-isolation-trials");
    }
    a
}

/// Engine preferences with no API, written to its own files before it starts: Secure DNS
/// (in `Local State`) and how WebRTC picks network interfaces (in the profile's `Preferences`).
pub fn write_engine_prefs(root: &std::path::Path, s: &crate::store::Settings) {
    use serde_json::{Value, json};
    fn edit(path: &std::path::Path, f: impl FnOnce(&mut serde_json::Map<String, Value>) -> bool) {
        let mut v = match std::fs::read_to_string(path) {
            // Never clobber a file we can't read.
            Ok(t) => match serde_json::from_str::<Value>(&t) {
                Ok(v) if v.is_object() => v,
                _ => return,
            },
            Err(_) => json!({}),
        };
        let Some(map) = v.as_object_mut() else { return };
        if f(map) {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(path, v.to_string());
        }
    }
    let dir = root.join("EBWebView");
    let template = match s.doh.as_str() {
        "cloudflare" => "https://cloudflare-dns.com/dns-query",
        "quad9" => "https://dns.quad9.net/dns-query",
        "mullvad" => "https://dns.mullvad.net/dns-query",
        "google" => "https://dns.google/dns-query",
        _ => "",
    };
    edit(&dir.join("Local State"), |m| {
        if template.is_empty() {
            m.remove("dns_over_https").is_some()
        } else {
            let want = json!({ "mode": "secure", "templates": template });
            m.insert("dns_over_https".into(), want.clone()).as_ref() != Some(&want)
        }
    });
    edit(&dir.join("Default").join("Preferences"), |m| {
        let webrtc = m.entry("webrtc").or_insert_with(|| json!({}));
        let Some(w) = webrtc.as_object_mut() else { return false };
        if s.webrtc_protect {
            let want = json!("default_public_interface_only");
            w.insert("ip_handling_policy".into(), want.clone()).as_ref() != Some(&want)
        } else {
            w.remove("ip_handling_policy").is_some()
        }
    });
}

// Downloads in progress, by Lurt's download id (all on the UI thread).
thread_local! {
    static DOWNLOADS: std::cell::RefCell<std::collections::HashMap<u32, ICoreWebView2DownloadOperation>> =
        Default::default();
    /// Downloads waiting for "Save as": their start event and its deferral.
    static ASKING: std::cell::RefCell<std::collections::HashMap<u32, (ICoreWebView2DownloadStartingEventArgs, ICoreWebView2Deferral)>> =
        Default::default();
}

/// Finish a "Save as": save to `path`, or cancel the download if None.
pub fn finish_ask(id: u32, path: Option<&str>) {
    let Some((args, deferral)) = ASKING.with(|a| a.borrow_mut().remove(&id)) else { return };
    unsafe {
        match path {
            Some(p) => {
                let _ = args.SetResultFilePath(&HSTRING::from(p));
            }
            None => {
                let _ = args.SetCancel(true);
            }
        }
        let _ = deferral.Complete();
    }
}
static NEXT_DOWNLOAD: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

/// Pause, resume or cancel a download in progress.
pub fn download_op(id: u32, action: &str) {
    let Some(op) = DOWNLOADS.with(|d| d.borrow().get(&id).cloned()) else { return };
    unsafe {
        let _ = match action {
            "pause" => op.Pause(),
            "resume" => op.Resume(),
            _ => op.Cancel(),
        };
    }
}
pub fn forget_download(id: u32) {
    DOWNLOADS.with(|d| d.borrow_mut().remove(&id));
}

/// Why a download stopped, in words.
fn interrupt_text(r: COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON) -> &'static str {
    match r {
        COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_NETWORK_DISCONNECTED => "No internet connection",
        COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_NETWORK_TIMEOUT => "The connection timed out",
        COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_NETWORK_FAILED
        | COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_NETWORK_SERVER_DOWN
        | COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_NETWORK_INVALID_REQUEST => "Network error",
        COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_FILE_NO_SPACE => "Not enough disk space",
        COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_FILE_ACCESS_DENIED => "Can't save to the Downloads folder",
        COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_FILE_NAME_TOO_LONG => "The file name is too long",
        COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_FILE_TOO_LARGE => "The file is too large",
        COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_FILE_MALICIOUS
        | COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_FILE_SECURITY_CHECK_FAILED
        | COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_FILE_BLOCKED_BY_POLICY => "Blocked by Windows security",
        COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_SERVER_UNAUTHORIZED
        | COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_SERVER_FORBIDDEN => "The site refused the download",
        COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_SERVER_BAD_CONTENT => "The file isn't on the server",
        COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_USER_SHUTDOWN => "Lurt was closed",
        COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_DOWNLOAD_PROCESS_CRASHED => "The download crashed",
        _ => "Failed",
    }
}

/// Cookies, site storage and cache (not passwords, autofill or settings).
pub fn site_data() -> COREWEBVIEW2_BROWSING_DATA_KINDS {
    COREWEBVIEW2_BROWSING_DATA_KINDS_ALL_SITE | COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE
}

/// Delete browsing data of a view's profile; `done` runs when it's finished.
pub fn clear_data(view: &WebView, kinds: COREWEBVIEW2_BROWSING_DATA_KINDS, done: impl FnOnce() + 'static) {
    unsafe {
        let r = view
            .webview()
            .cast::<ICoreWebView2_13>()
            .and_then(|w| w.Profile())
            .and_then(|p| p.cast::<ICoreWebView2Profile2>())
            .and_then(|p| {
                p.ClearBrowsingData(
                    kinds,
                    &ClearBrowsingDataCompletedHandler::create(Box::new(move |_| {
                        done();
                        Ok(())
                    })),
                )
            });
        if let Err(e) = r {
            eprintln!("lurt: clear browsing data: {e}");
        }
    }
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
            let _ = p.send_event(Ev::NewTab(id, url));
            NewWindowResponse::Deny
        }
    };

    let view = WebViewBuilder::new_with_web_context(ctx)
        .with_additional_browser_args(ENGINE_ARGS.get().cloned().unwrap_or_default())
        .with_incognito(opts.private)
        .with_bounds(opts.bounds)
        .with_visible(opts.visible)
        .with_focused(opts.visible)
        .with_initialization_script_for_main_only(CONTENT_JS, true)
        .with_initialization_script_for_main_only(VOLUME_JS, false)
        .with_initialization_script_for_main_only(opts.script.unwrap_or_default(), false)
        .with_ipc_handler(ipc)
        .with_document_title_changed_handler(title)
        .with_on_page_load_handler(load)
        .with_new_window_req_handler(new_window)
        .with_devtools(true)
        // Ctrl + scroll, Ctrl +/-/0 and touchpad pinch.
        .with_hotkeys_zoom(true)
        .with_theme(opts.theme)
        .with_scroll_bar_style(wry::ScrollBarStyle::FluentOverlay)
        .build_as_child(window)
        .ok()?;

    if let Err(e) = unsafe { hook(&view, id, proxy.clone(), shield.clone()) } {
        eprintln!("lurt: native hooks failed for tab {id}: {e}");
    }
    set_zoom(&view, opts.zoom);
    // Load only now, so the first page also gets clean links, HTTPS and privacy headers.
    let _ = view.load_url(opts.url);
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
        let (p, shield_req, env) = (proxy.clone(), shield.clone(), env.clone());
        wv.add_WebResourceRequested(
            &WebResourceRequestedEventHandler::create(Box::new(move |sender, args| {
                let shield = &shield_req;
                let (Some(sender), Some(args)) = (sender, args) else { return Ok(()) };
                let pv = shield.privacy.get();
                let mut ctx = COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL;
                args.ResourceContext(&mut ctx)?;
                if pv.gpc {
                    if let Ok(h) = args.Request().and_then(|r| r.Headers()) {
                        let _ = h.SetHeader(w!("Sec-GPC"), w!("1"));
                        let _ = h.SetHeader(w!("DNT"), w!("1"));
                    }
                }
                // Hyperlink auditing: <a ping> tells a site which links you click.
                if pv.block_pings && ctx == COREWEBVIEW2_WEB_RESOURCE_CONTEXT_PING {
                    let resp = env.CreateWebResourceResponse(None, 403, w!("Blocked"), w!(""))?;
                    args.SetResponse(&resp)?;
                    let mut s = PWSTR::null();
                    args.Request()?.Uri(&mut s)?;
                    let _ = p.send_event(Ev::Blocked(id, take_pwstr(s), "ping"));
                    return Ok(());
                }
                if !shield.enabled.get() {
                    return Ok(());
                }
                let engine = shield.engine.borrow();
                let Some(engine) = engine.as_ref() else { return Ok(()) };

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

        // --- clean links and HTTPS upgrades, before a page loads ---
        // The host last upgraded to HTTPS, to fall back to HTTP if that fails.
        let upgraded: Rc<std::cell::RefCell<Option<String>>> = Rc::default();
        let (p, sh, up) = (proxy.clone(), shield.clone(), upgraded.clone());
        wv.add_NavigationStarting(
            &NavigationStartingEventHandler::create(Box::new(move |_, args| {
                let Some(args) = args else { return Ok(()) };
                let pv = sh.privacy.get();
                // The page request itself (sub-requests get it in WebResourceRequested).
                if pv.gpc {
                    if let Ok(h) = args.RequestHeaders() {
                        let _ = h.SetHeader(w!("Sec-GPC"), w!("1"));
                        let _ = h.SetHeader(w!("DNT"), w!("1"));
                    }
                }
                if !pv.strip && !pv.https {
                    return Ok(());
                }
                let mut s = PWSTR::null();
                args.Uri(&mut s)?;
                let uri = take_pwstr(s);
                // Server redirects aren't upgraded: a site sending you back to HTTP would loop.
                let mut redirected = BOOL(0);
                args.IsRedirected(&mut redirected)?;
                let skip = sh.https_skip.borrow();
                if let Some(clean) = crate::privacy::clean_url(&uri, pv.strip, pv.https && !redirected.as_bool(), &skip) {
                    if uri.starts_with("http://") && clean.starts_with("https://") {
                        *up.borrow_mut() = Some(util::host_of(&clean));
                    }
                    args.SetCancel(true)?;
                    let _ = p.send_event(Ev::Redirect(id, clean));
                }
                Ok(())
            })),
            &mut token,
        )?;
        let (p, sh, up) = (proxy.clone(), shield.clone(), upgraded);
        wv.add_NavigationCompleted(
            &NavigationCompletedEventHandler::create(Box::new(move |sender, args| {
                let (Some(sender), Some(args)) = (sender, args) else { return Ok(()) };
                let mut ok = BOOL(0);
                args.IsSuccess(&mut ok)?;
                let mut err = COREWEBVIEW2_WEB_ERROR_STATUS_UNKNOWN;
                args.WebErrorStatus(&mut err)?;
                // Our own cancel of the HTTP request lands here too.
                if err == COREWEBVIEW2_WEB_ERROR_STATUS_OPERATION_CANCELED {
                    return Ok(());
                }
                let Some(host) = up.borrow_mut().take() else { return Ok(()) };
                let broken = matches!(
                    err,
                    COREWEBVIEW2_WEB_ERROR_STATUS_CERTIFICATE_COMMON_NAME_IS_INCORRECT
                        | COREWEBVIEW2_WEB_ERROR_STATUS_CERTIFICATE_EXPIRED
                        | COREWEBVIEW2_WEB_ERROR_STATUS_CLIENT_CERTIFICATE_CONTAINS_ERRORS
                        | COREWEBVIEW2_WEB_ERROR_STATUS_CERTIFICATE_REVOKED
                        | COREWEBVIEW2_WEB_ERROR_STATUS_CERTIFICATE_IS_INVALID
                        | COREWEBVIEW2_WEB_ERROR_STATUS_SERVER_UNREACHABLE
                        | COREWEBVIEW2_WEB_ERROR_STATUS_TIMEOUT
                        | COREWEBVIEW2_WEB_ERROR_STATUS_ERROR_HTTP_INVALID_SERVER_RESPONSE
                        | COREWEBVIEW2_WEB_ERROR_STATUS_CONNECTION_ABORTED
                        | COREWEBVIEW2_WEB_ERROR_STATUS_CONNECTION_RESET
                        | COREWEBVIEW2_WEB_ERROR_STATUS_DISCONNECTED
                        | COREWEBVIEW2_WEB_ERROR_STATUS_CANNOT_CONNECT
                        | COREWEBVIEW2_WEB_ERROR_STATUS_UNKNOWN
                );
                if !ok.as_bool() && broken {
                    let mut s = PWSTR::null();
                    sender.Source(&mut s)?;
                    let src = take_pwstr(s);
                    if util::host_of(&src) == host {
                        if let Some(rest) = src.strip_prefix("https://") {
                            sh.https_skip.borrow_mut().insert(host);
                            let _ = p.send_event(Ev::Redirect(id, format!("http://{rest}")));
                        }
                    }
                }
                Ok(())
            })),
            &mut token,
        )?;

        // --- site permissions you chose to always block ---
        let sh = shield.clone();
        wv.add_PermissionRequested(
            &PermissionRequestedEventHandler::create(Box::new(move |_, args| {
                let Some(args) = args else { return Ok(()) };
                let mut k = COREWEBVIEW2_PERMISSION_KIND_UNKNOWN_PERMISSION;
                args.PermissionKind(&mut k)?;
                let i = match k {
                    COREWEBVIEW2_PERMISSION_KIND_GEOLOCATION => 0,
                    COREWEBVIEW2_PERMISSION_KIND_CAMERA => 1,
                    COREWEBVIEW2_PERMISSION_KIND_MICROPHONE => 2,
                    COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS => 3,
                    _ => return Ok(()),
                };
                if sh.privacy.get().deny[i] {
                    args.SetState(COREWEBVIEW2_PERMISSION_STATE_DENY)?;
                }
                Ok(())
            })),
            &mut token,
        )?;

        // --- downloads: Lurt shows them in its own panel instead of the engine's pop-up ---
        if let Ok(w4) = wv.cast::<ICoreWebView2_4>() {
            let (p, sh) = (proxy.clone(), shield.clone());
            w4.add_DownloadStarting(
                &DownloadStartingEventHandler::create(Box::new(move |_, args| {
                    let Some(args) = args else { return Ok(()) };
                    args.SetHandled(true)?;
                    let op = args.DownloadOperation()?;
                    let dl = NEXT_DOWNLOAD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let mut s = PWSTR::null();
                    args.ResultFilePath(&mut s)?;
                    let mut path = take_pwstr(s);
                    let mut s = PWSTR::null();
                    op.Uri(&mut s)?;
                    let url = take_pwstr(s);
                    let mut total = 0i64;
                    op.TotalBytesToReceive(&mut total)?;

                    // Your download settings: blocked types, another folder, "Save as".
                    let prefs = sh.downloads.borrow().clone();
                    let name = std::path::Path::new(&path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                    let action = prefs.action(&name).to_string();
                    if action == "block" {
                        args.SetCancel(true)?;
                        let _ = p.send_event(Ev::DlBlocked { tab: id, name, url });
                        return Ok(());
                    }
                    if !prefs.dir.is_empty() && std::path::Path::new(&prefs.dir).is_dir() {
                        path = crate::dialogs::unique_path(std::path::Path::new(&prefs.dir), &name).to_string_lossy().into_owned();
                        args.SetResultFilePath(&HSTRING::from(path.as_str()))?;
                    }
                    let ask = action == "ask";
                    if ask {
                        let deferral = args.GetDeferral()?;
                        ASKING.with(|a| a.borrow_mut().insert(dl, (args.clone(), deferral)));
                    }
                    let _ = p.send_event(Ev::DlStart { tab: id, dl, path, url, total, ask, open: action == "open" });

                    let mut tok = 0i64;
                    let p2 = p.clone();
                    op.add_BytesReceivedChanged(
                        &BytesReceivedChangedEventHandler::create(Box::new(move |op, _| {
                            if let Some(op) = op {
                                let (mut r, mut t) = (0i64, 0i64);
                                op.BytesReceived(&mut r)?;
                                op.TotalBytesToReceive(&mut t)?;
                                let _ = p2.send_event(Ev::DlProgress(dl, r, t));
                            }
                            Ok(())
                        })),
                        &mut tok,
                    )?;
                    let p3 = p.clone();
                    op.add_StateChanged(
                        &StateChangedEventHandler::create(Box::new(move |op, _| {
                            let Some(op) = op else { return Ok(()) };
                            let mut st = COREWEBVIEW2_DOWNLOAD_STATE_IN_PROGRESS;
                            op.State(&mut st)?;
                            let mut why = COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_NONE;
                            op.InterruptReason(&mut why)?;
                            let mut s = PWSTR::null();
                            op.ResultFilePath(&mut s)?;
                            let (state, reason) = match st {
                                COREWEBVIEW2_DOWNLOAD_STATE_COMPLETED => ("done", ""),
                                COREWEBVIEW2_DOWNLOAD_STATE_INTERRUPTED => match why {
                                    COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_USER_CANCELED => ("canceled", ""),
                                    COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_USER_PAUSED => ("paused", ""),
                                    r => ("failed", interrupt_text(r)),
                                },
                                _ => ("progress", ""),
                            };
                            let _ = p3.send_event(Ev::DlState(dl, state, reason, take_pwstr(s)));
                            Ok(())
                        })),
                        &mut tok,
                    )?;
                    DOWNLOADS.with(|d| d.borrow_mut().insert(dl, op));
                    Ok(())
                })),
                &mut token,
            )?;
        }

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

        // --- the page's icon, as the engine loaded it ---
        // Asking for the file again from the toolbar fails on sites that only serve it to
        // their own pages (Discord), so take the picture the engine already has.
        if let Ok(w15) = wv.cast::<ICoreWebView2_15>() {
            let p = proxy.clone();
            w15.add_FaviconChanged(
                &FaviconChangedEventHandler::create(Box::new(move |sender, _| {
                    let Some(w) = sender.and_then(|s| s.cast::<ICoreWebView2_15>().ok()) else { return Ok(()) };
                    let mut uri = PWSTR::null();
                    w.FaviconUri(&mut uri)?;
                    if take_pwstr(uri).is_empty() {
                        return Ok(());
                    }
                    let mut u = PWSTR::null();
                    w.Source(&mut u)?;
                    let (p, page) = (p.clone(), take_pwstr(u));
                    w.GetFavicon(
                        COREWEBVIEW2_FAVICON_IMAGE_FORMAT_PNG,
                        &GetFaviconCompletedHandler::create(Box::new(move |res, stream| {
                            if let (Ok(()), Some(stream)) = (res, stream) {
                                let _ = p.send_event(Ev::Favicon(id, page, read_stream(&stream)));
                            }
                            Ok(())
                        })),
                    )
                })),
                &mut token,
            )?;
        }

        // --- sound ---
        if let Ok(w8) = wv.cast::<ICoreWebView2_8>() {
            let p = proxy.clone();
            w8.add_IsDocumentPlayingAudioChanged(
                &IsDocumentPlayingAudioChangedEventHandler::create(Box::new(move |sender, _| {
                    if let Some(w) = sender.and_then(|s| s.cast::<ICoreWebView2_8>().ok()) {
                        let mut on = BOOL(0);
                        w.IsDocumentPlayingAudio(&mut on)?;
                        let _ = p.send_event(Ev::Audio(id, on.as_bool()));
                    }
                    Ok(())
                })),
                &mut token,
            )?;
        }

        // --- zoom (Ctrl + scroll and friends) ---
        let p = proxy.clone();
        view.controller().add_ZoomFactorChanged(
            &ZoomFactorChangedEventHandler::create(Box::new(move |sender, _| {
                if let Some(c) = sender {
                    let mut f = 1.0;
                    c.ZoomFactor(&mut f)?;
                    let _ = p.send_event(Ev::Zoom(id, (f * 100.0).round() as u32));
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

/// Show a page at this many percent of its normal size.
pub fn set_zoom(view: &WebView, percent: u32) {
    let _ = view.zoom(f64::from(percent.clamp(25, 500)) / 100.0);
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

/// Apply a tab's volume (0..=100) and mute to its page. Silence uses the engine's own mute,
/// which also covers anything the volume script can't reach.
pub fn set_sound(view: &WebView, volume: u8, muted: bool) {
    unsafe {
        if let Ok(w) = view.webview().cast::<ICoreWebView2_8>() {
            let _ = w.SetIsMuted(muted || volume == 0);
        }
    }
    let factor = f64::from(volume.min(100)) / 100.0;
    let _ = view.evaluate_script(&format!("window.__lurtVolume&&window.__lurtVolume({factor})"));
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

/// Take a JPEG of the page; it arrives as `Ev::Snapshot`. `preview` marks a picture
/// kept for when the tab wakes up, rather than one for the address bar overlay.
pub fn snapshot(view: &WebView, id: u32, proxy: EventLoopProxy<Ev>, preview: bool) {
    unsafe {
        let Some(stream) = SHCreateMemStream(None) else {
            let _ = proxy.send_event(Ev::Snapshot(id, Vec::new(), preview));
            return;
        };
        let s2 = stream.clone();
        let p = proxy.clone();
        let r = view.webview().CapturePreview(
            COREWEBVIEW2_CAPTURE_PREVIEW_IMAGE_FORMAT_JPEG,
            &stream,
            &CapturePreviewCompletedHandler::create(Box::new(move |res| {
                let bytes = if res.is_ok() { read_stream(&s2) } else { Vec::new() };
                let _ = p.send_event(Ev::Snapshot(id, bytes, preview));
                Ok(())
            })),
        );
        if r.is_err() {
            let _ = proxy.send_event(Ev::Snapshot(id, Vec::new(), preview));
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
