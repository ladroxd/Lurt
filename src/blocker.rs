//! Ad & tracker blocking with Brave's adblock engine.
//!
//! Filter lists are downloaded once, compiled, and cached as a serialized engine,
//! so later launches only deserialize (a few ms) on a background thread.

use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    fs,
    path::PathBuf,
    thread,
    time::{Duration, SystemTime},
};

use adblock::{Engine, lists::{FilterSet, ParseOptions}};
use tao::event_loop::EventLoopProxy;

use crate::Ev;

/// Filter lists you can turn on in Settings > Privacy, by id.
pub const LISTS: &[(&str, &[&str])] = &[
    ("easylist", &["https://easylist.to/easylist/easylist.txt"]),
    ("easyprivacy", &["https://easylist.to/easylist/easyprivacy.txt"]),
    ("peterlowe", &["https://pgl.yoyo.org/adservers/serverlist.php?hostformat=adblockplus&showintro=0&mimetype=plaintext"]),
    (
        "ublock",
        &[
            "https://ublockorigin.github.io/uAssets/filters/filters.txt",
            "https://ublockorigin.github.io/uAssets/filters/privacy.txt",
            "https://ublockorigin.github.io/uAssets/filters/quick-fixes.txt",
            "https://ublockorigin.github.io/uAssets/filters/unbreak.txt",
        ],
    ),
    ("badware", &["https://ublockorigin.github.io/uAssets/filters/badware.txt"]),
    ("cookies", &["https://secure.fanboy.co.nz/fanboy-cookiemonster.txt"]),
    ("annoyances", &["https://secure.fanboy.co.nz/fanboy-annoyance.txt"]),
    ("miners", &["https://raw.githubusercontent.com/hoshsadiq/adblock-nocoin-list/master/nocoin.txt"]),
];

/// Lists older than this are refreshed in the background.
const MAX_AGE: Duration = Duration::from_secs(60 * 60 * 24 * 4);

/// Shared by every tab's request filter (all on the UI thread).
#[derive(Default)]
pub struct Shield {
    pub engine: RefCell<Option<Engine>>,
    pub enabled: Cell<bool>,
    pub allow: RefCell<Vec<String>>,
    /// Protections that work without filter lists.
    pub privacy: Cell<Privacy>,
    /// Sites that failed over HTTPS this session, so they load over HTTP.
    pub https_skip: RefCell<HashSet<String>>,
}

#[derive(Clone, Copy, Default)]
pub struct Privacy {
    /// Send Global Privacy Control and Do Not Track.
    pub gpc: bool,
    pub strip: bool,
    pub https: bool,
    pub block_pings: bool,
    /// Deny without asking: location, camera, microphone, notifications.
    pub deny: [bool; 4],
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
pub fn load(dir: PathBuf, force: bool, lists: Vec<String>, proxy: EventLoopProxy<Ev>) {
    thread::spawn(move || {
        let cache = cache_path(&dir);
        // Which lists the cache was built from: a different choice means a rebuild.
        let key_path = dir.join("adblock.lists");
        let key = lists.join(",");
        let force = force || fs::read_to_string(&key_path).ok().as_deref() != Some(key.as_str());
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

        match build(&lists) {
            Some(bytes) => {
                let _ = fs::write(&cache, &bytes);
                let _ = fs::write(&key_path, &key);
                let _ = proxy.send_event(Ev::ShieldReady(bytes, true));
            }
            None if cached.is_none() => {
                let _ = proxy.send_event(Ev::ShieldFailed);
            }
            None => {}
        }
    });
}

fn build(lists: &[String]) -> Option<Vec<u8>> {
    let mut set = FilterSet::new(false);
    let mut got = 0;
    let urls = LISTS.iter().filter(|(id, _)| lists.iter().any(|l| l == id)).flat_map(|(_, u)| u.iter());
    for url in urls {
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
