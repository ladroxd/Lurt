//! Ad & tracker blocking with Brave's adblock engine.
//!
//! Filter lists are downloaded once, compiled, and cached as a serialized engine,
//! so later launches only deserialize (a few ms) on a background thread.

use std::{
    cell::{Cell, RefCell},
    fs,
    path::PathBuf,
    thread,
    time::{Duration, SystemTime},
};

use adblock::{Engine, lists::{FilterSet, ParseOptions}};
use tao::event_loop::EventLoopProxy;

use crate::Ev;

const LISTS: &[&str] = &[
    "https://easylist.to/easylist/easylist.txt",
    "https://easylist.to/easylist/easyprivacy.txt",
    "https://pgl.yoyo.org/adservers/serverlist.php?hostformat=adblockplus&showintro=0&mimetype=plaintext",
];

/// Lists older than this are refreshed in the background.
const MAX_AGE: Duration = Duration::from_secs(60 * 60 * 24 * 4);

/// Shared by every tab's request filter (all on the UI thread).
#[derive(Default)]
pub struct Shield {
    pub engine: RefCell<Option<Engine>>,
    pub enabled: Cell<bool>,
    pub allow: RefCell<Vec<String>>,
}

impl Shield {
    pub fn install(&self, bytes: &[u8]) -> bool {
        let mut e = Engine::default();
        if e.deserialize(bytes).is_ok() {
            *self.engine.borrow_mut() = Some(e);
            true
        } else {
            false
        }
    }

    pub fn allowed(&self, host: &str) -> bool {
        !host.is_empty() && self.allow.borrow().iter().any(|a| host == a || host.ends_with(&format!(".{a}")))
    }
}

fn cache_path(dir: &PathBuf) -> PathBuf {
    dir.join("adblock.dat")
}

/// Load the cached engine, or download and build it. Result arrives as an `Ev`.
pub fn load(dir: PathBuf, force: bool, proxy: EventLoopProxy<Ev>) {
    thread::spawn(move || {
        let cache = cache_path(&dir);
        let age = fs::metadata(&cache)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| SystemTime::now().duration_since(t).ok());
        let cached = fs::read(&cache).ok();

        if let Some(bytes) = &cached {
            // Use the cache right away, even if stale, so blocking starts instantly.
            let _ = proxy.send_event(Ev::ShieldReady(bytes.clone(), false));
            if !force && age.is_some_and(|a| a < MAX_AGE) {
                return;
            }
        }

        match build() {
            Some(bytes) => {
                let _ = fs::write(&cache, &bytes);
                let _ = proxy.send_event(Ev::ShieldReady(bytes, true));
            }
            None if cached.is_none() => {
                let _ = proxy.send_event(Ev::ShieldFailed);
            }
            None => {}
        }
    });
}

fn build() -> Option<Vec<u8>> {
    let mut set = FilterSet::new(false);
    let mut got = 0;
    for url in LISTS {
        let text = ureq::get(*url)
            .call()
            .ok()
            .and_then(|mut r| r.body_mut().with_config().limit(32 * 1024 * 1024).read_to_string().ok());
        if let Some(text) = text {
            set.add_filter_list(text, ParseOptions::default());
            got += 1;
        }
    }
    if got == 0 {
        return None;
    }
    Some(Engine::new_with_filter_set(set).serialize())
}
