//! The Google account signed in to the browser, found the way Chrome finds it for its
//! avatar: Google's ListAccounts endpoint, called with the browser's own Google cookies.
//! Nothing is stored; signing out removes Google's sign-in cookies from the profile.

use serde::Serialize;
use serde_json::Value;
use tao::event_loop::EventLoopProxy;
use webview2_com::{
    GetCookiesCompletedHandler,
    Microsoft::Web::WebView2::Win32::{ICoreWebView2_2, ICoreWebView2CookieList, ICoreWebView2CookieManager},
    take_pwstr,
};
use windows::core::{HSTRING, Interface, PWSTR};
use wry::{WebView, WebViewExtWindows};

use crate::Ev;

#[derive(Serialize, Clone, Default, PartialEq)]
pub struct Account {
    pub name: String,
    pub email: String,
    pub photo: String,
}

const LIST_URL: &str = "https://accounts.google.com/ListAccounts?gpsia=1&source=ChromiumBrowser&json=standard";
/// Where Google keeps its sign-in cookies.
const SIGN_IN_URLS: &[&str] = &["https://accounts.google.com/", "https://www.google.com/", "https://myaccount.google.com/"];

fn manager(view: &WebView) -> Option<ICoreWebView2CookieManager> {
    unsafe { view.webview().cast::<ICoreWebView2_2>().ok()?.CookieManager().ok() }
}

fn for_each_cookie(list: &ICoreWebView2CookieList, mut f: impl FnMut(&webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Cookie, String, String)) {
    unsafe {
        let mut n = 0u32;
        let _ = list.Count(&mut n);
        for i in 0..n {
            if let Ok(c) = list.GetValueAtIndex(i) {
                let (mut a, mut b) = (PWSTR::null(), PWSTR::null());
                let _ = c.Name(&mut a);
                let _ = c.Value(&mut b);
                f(&c, take_pwstr(a), take_pwstr(b));
            }
        }
    }
}

/// Look up the signed-in accounts; the answer arrives as `Ev::Accounts`.
pub fn refresh(view: &WebView, proxy: EventLoopProxy<Ev>) {
    let Some(mgr) = manager(view) else { return };
    unsafe {
        let _ = mgr.GetCookies(
            &HSTRING::from(LIST_URL),
            &GetCookiesCompletedHandler::create(Box::new(move |res, list| {
                let mut header = String::new();
                if let (Ok(()), Some(list)) = (res, list) {
                    for_each_cookie(&list, |_, name, value| {
                        if !header.is_empty() {
                            header.push_str("; ");
                        }
                        header.push_str(&format!("{name}={value}"));
                    });
                }
                // The network call runs off the UI thread.
                std::thread::spawn(move || {
                    let _ = proxy.send_event(Ev::Accounts(lookup(&header)));
                });
                Ok(())
            })),
        );
    }
}

fn lookup(cookies: &str) -> Result<Vec<Account>, String> {
    if !cookies.contains("SID=") {
        return Ok(Vec::new());
    }
    let body = ureq::get(LIST_URL)
        .header("Cookie", cookies)
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    Ok(parse(&body))
}

/// `["gaia.l.a.r", [["gaia.l.a", 1, "Name", "email", "photo", ..., [9] valid, [10] id, ..., [14] signed out], ...]]`,
/// read the same way as Chromium's `ParseListAccountsData`.
fn parse(body: &str) -> Vec<Account> {
    let Ok(v) = serde_json::from_str::<Value>(body) else { return Vec::new() };
    let Some(list) = v.get(1).and_then(Value::as_array) else { return Vec::new() };
    list.iter()
        // Field 9: the session is valid (1). Field 14: the account was signed out (1).
        .filter(|a| a.get(9).and_then(Value::as_i64).unwrap_or(1) != 0)
        .filter(|a| a.get(14).and_then(Value::as_i64).unwrap_or(0) == 0)
        .filter_map(|a| {
            let email = a.get(3)?.as_str()?.to_string();
            Some(Account {
                name: a.get(2).and_then(Value::as_str).unwrap_or("").to_string(),
                photo: a.get(4).and_then(Value::as_str).unwrap_or("").to_string(),
                email,
            })
        })
        .collect()
}

/// Sign out of Google in this browser: drop Google's sign-in cookies, then look again.
pub fn sign_out(view: &WebView, proxy: EventLoopProxy<Ev>) {
    let Some(mgr) = manager(view) else { return };
    let left = std::rc::Rc::new(std::cell::Cell::new(SIGN_IN_URLS.len()));
    for url in SIGN_IN_URLS {
        let (m, p, left) = (mgr.clone(), proxy.clone(), left.clone());
        unsafe {
            let _ = mgr.GetCookies(
                &HSTRING::from(*url),
                &GetCookiesCompletedHandler::create(Box::new(move |res, list| {
                    if let (Ok(()), Some(list)) = (res, list) {
                        for_each_cookie(&list, |c, _, _| {
                            let _ = m.DeleteCookie(c);
                        });
                    }
                    left.set(left.get() - 1);
                    if left.get() == 0 {
                        let _ = p.send_event(Ev::SignedOut);
                    }
                    Ok(())
                })),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_list_accounts() {
        // As Google sends it: a signed-in account, one whose session expired (field 9 = 0)
        // and one that was signed out (field 14 = 1).
        let body = r#"["gaia.l.a.r",[["gaia.l.a",1,"Ada L","ada@example.com","https://lh3.googleusercontent.com/a/x=s48",1,1,0,null,1,"123",null,null,null,0,1],["gaia.l.a",1,"Expired","exp@example.com","",1,0,0,null,0,"456"],["gaia.l.a",1,"Old","old@example.com","",1,0,0,null,1,"789",null,null,null,1,1]]]"#;
        let a = super::parse(body);
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].email, "ada@example.com");
        assert_eq!(a[0].name, "Ada L");
    }
}
