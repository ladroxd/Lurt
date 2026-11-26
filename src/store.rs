//! Everything Lurt keeps on disk: settings (JSON), history + bookmarks (SQLite),
//! the last session, and the user's mods folder.

use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct Settings {
    /// "system" | "light" | "dark"
    pub theme: String,
    /// "blur" | "acrylic" | "mica" | "none"
    pub backdrop: String,
    /// Opacity of the browser's background over the glass, 0 (clear) ..= 100 (solid).
    pub tint: u8,
    /// Opacity of the whole window, pages included, 10..=100.
    pub window_opacity: u8,
    pub accent: String,
    /// Corner radius of glass surfaces, in px.
    pub radius: u8,
    pub compact: bool,
    pub reduce_motion: bool,
    pub rounded_pages: bool,
    pub show_clock: bool,
    /// "duckduckgo" | "google" | "bing" | "brave" | "startpage" | "ecosia" | a URL with %s
    pub search: String,
    /// Empty means the built-in start page.
    pub homepage: String,
    pub restore_session: bool,
    pub adblock: bool,
    /// Sites where blocking is paused.
    pub adblock_allow: Vec<String>,
    /// "strict" | "balanced" | "basic" | "off"
    pub tracking: String,
    /// Unload background tabs after this many minutes. 0 = never.
    pub sleep_minutes: u32,
    /// Extra CSS for the browser UI, edited in Settings.
    pub ui_css: String,
    /// Offer to save passwords you type into sites.
    pub save_passwords: bool,
    /// Autofill addresses and other form data.
    pub autofill: bool,
    /// Favourites pinned to the start page, in order.
    pub shortcuts: Vec<Shortcut>,
    /// Sites (hosts) removed from the start page's most-visited tiles.
    pub hidden_tiles: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct Shortcut {
    pub title: String,
    pub url: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            backdrop: "blur".into(),
            tint: 55,
            window_opacity: 100,
            accent: "#8ab4ff".into(),
            radius: 14,
            compact: false,
            reduce_motion: false,
            rounded_pages: true,
            show_clock: true,
            search: "google".into(),
            homepage: String::new(),
            restore_session: true,
            adblock: true,
            adblock_allow: Vec::new(),
            tracking: "balanced".into(),
            sleep_minutes: 30,
            ui_css: String::new(),
            save_passwords: true,
            autofill: true,
            shortcuts: Vec::new(),
            hidden_tiles: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Session {
    pub tabs: Vec<SessionTab>,
    pub active: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SessionTab {
    pub url: String,
    pub title: String,
}

#[derive(Serialize)]
pub struct Entry {
    pub url: String,
    pub title: String,
    pub time: i64,
}

pub struct Store {
    db: Connection,
    pub dir: PathBuf,
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Escape LIKE wildcards so a user's `%` or `_` is matched literally.
fn like(q: &str) -> String {
    let esc = q.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
    format!("%{esc}%")
}

impl Store {
    pub fn open() -> Self {
        // LURT_DATA points the profile elsewhere (benchmarks, portable installs).
        let dir = std::env::var_os("LURT_DATA").map(PathBuf::from).unwrap_or_else(|| {
            dirs::data_dir().unwrap_or_else(std::env::temp_dir).join("Lurt")
        });
        let _ = fs::create_dir_all(dir.join("mods").join("ui"));
        let _ = fs::create_dir_all(dir.join("mods").join("sites"));
        write_mods_readme(&dir.join("mods"));

        let db = Connection::open(dir.join("lurt.db"))
            .or_else(|_| Connection::open_in_memory())
            .expect("sqlite");
        let _ = db.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             CREATE TABLE IF NOT EXISTS history(
                url TEXT PRIMARY KEY, title TEXT NOT NULL DEFAULT '',
                visits INTEGER NOT NULL DEFAULT 1, last INTEGER NOT NULL);
             CREATE INDEX IF NOT EXISTS history_last ON history(last DESC);
             CREATE TABLE IF NOT EXISTS bookmarks(
                url TEXT PRIMARY KEY, title TEXT NOT NULL DEFAULT '', created INTEGER NOT NULL);",
        );
        Self { db, dir }
    }

    pub fn db_path(&self) -> PathBuf {
        self.dir.join("lurt.db")
    }

    pub fn webview_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("Lurt")
            .join("WebView")
    }

    // ---- settings & session -------------------------------------------------

    pub fn load_settings(&self) -> Settings {
        fs::read_to_string(self.dir.join("settings.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save_settings(&self, s: &Settings) {
        if let Ok(json) = serde_json::to_string_pretty(s) {
            let _ = fs::write(self.dir.join("settings.json"), json);
        }
    }

    pub fn load_session(&self) -> Session {
        fs::read_to_string(self.dir.join("session.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save_session(&self, s: &Session) {
        if let Ok(json) = serde_json::to_string(s) {
            let _ = fs::write(self.dir.join("session.json"), json);
        }
    }

    // ---- history --------------------------------------------------------------

    pub fn add_visit(&self, url: &str, title: &str) {
        let _ = self.db.execute(
            "INSERT INTO history(url, title, visits, last) VALUES(?1, ?2, 1, ?3)
             ON CONFLICT(url) DO UPDATE SET visits = visits + 1, last = ?3,
               title = CASE WHEN ?2 = '' THEN title ELSE ?2 END",
            params![url, title, now()],
        );
    }

    pub fn set_title(&self, url: &str, title: &str) {
        let _ = self.db.execute(
            "UPDATE history SET title = ?2 WHERE url = ?1",
            params![url, title],
        );
        let _ = self.db.execute(
            "UPDATE bookmarks SET title = ?2 WHERE url = ?1 AND title = ''",
            params![url, title],
        );
    }

    pub fn history(&self, q: &str, limit: u32) -> Vec<Entry> {
        self.query(
            "SELECT url, title, last FROM history
             WHERE url LIKE ?1 ESCAPE '\\' OR title LIKE ?1 ESCAPE '\\'
             ORDER BY last DESC LIMIT ?2",
            params![like(q), limit],
        )
    }

    /// Start-page tiles: bookmarks first, topped up with the most-visited sites.
    pub fn top_sites(&self, n: usize) -> Vec<Entry> {
        let mut out = self.bookmarks();
        out.truncate(n);
        if out.len() < n {
            let hosts: Vec<String> = out.iter().map(|e| crate::util::host_of(&e.url)).collect();
            let mut seen = hosts;
            for e in self.query(
                "SELECT url, title, last FROM history ORDER BY visits DESC, last DESC LIMIT ?2",
                params![like(""), 200],
            ) {
                let h = crate::util::host_of(&e.url);
                if h.is_empty() || seen.contains(&h) {
                    continue;
                }
                seen.push(h);
                out.push(e);
                if out.len() >= n {
                    break;
                }
            }
        }
        out
    }

    pub fn delete_history(&self, url: &str) {
        let _ = self.db.execute("DELETE FROM history WHERE url = ?1", params![url]);
    }

    pub fn clear_history(&self) {
        let _ = self.db.execute_batch("DELETE FROM history; VACUUM;");
    }

    // ---- bookmarks -------------------------------------------------------------

    pub fn bookmarks(&self) -> Vec<Entry> {
        self.query(
            "SELECT url, title, created FROM bookmarks ORDER BY created DESC LIMIT ?2",
            params![like(""), 500],
        )
    }

    pub fn is_bookmarked(&self, url: &str) -> bool {
        self.db
            .query_row("SELECT 1 FROM bookmarks WHERE url = ?1", params![url], |_| Ok(()))
            .is_ok()
    }

    /// Returns the new state: true if the page is now bookmarked.
    pub fn toggle_bookmark(&self, url: &str, title: &str) -> bool {
        if self.is_bookmarked(url) {
            self.remove_bookmark(url);
            false
        } else {
            let _ = self.db.execute(
                "INSERT OR REPLACE INTO bookmarks(url, title, created) VALUES(?1, ?2, ?3)",
                params![url, title, now()],
            );
            true
        }
    }

    pub fn remove_bookmark(&self, url: &str) {
        let _ = self.db.execute("DELETE FROM bookmarks WHERE url = ?1", params![url]);
    }

    /// Address-bar suggestions: bookmarks first, then the most-visited history.
    pub fn suggest(&self, q: &str) -> Vec<Entry> {
        if q.trim().is_empty() {
            return Vec::new();
        }
        let mut out = self.query(
            "SELECT url, title, created FROM bookmarks
             WHERE url LIKE ?1 ESCAPE '\\' OR title LIKE ?1 ESCAPE '\\'
             ORDER BY created DESC LIMIT ?2",
            params![like(q), 3],
        );
        for e in self.query(
            "SELECT url, title, last FROM history
             WHERE url LIKE ?1 ESCAPE '\\' OR title LIKE ?1 ESCAPE '\\'
             ORDER BY visits DESC, last DESC LIMIT ?2",
            params![like(q), 8],
        ) {
            if out.len() >= 7 {
                break;
            }
            if !out.iter().any(|o| o.url == e.url) {
                out.push(e);
            }
        }
        out
    }

    fn query(&self, sql: &str, p: impl rusqlite::Params) -> Vec<Entry> {
        let Ok(mut stmt) = self.db.prepare_cached(sql) else {
            return Vec::new();
        };
        stmt.query_map(p, |r| {
            Ok(Entry {
                url: r.get(0)?,
                title: r.get(1)?,
                time: r.get(2)?,
            })
        })
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
    }

    // ---- mods ------------------------------------------------------------------

    pub fn mods_dir(&self) -> PathBuf {
        self.dir.join("mods")
    }

    /// All `mods/ui/*.css` files, concatenated, applied to the browser UI.
    pub fn ui_mods(&self) -> String {
        read_matching(&self.mods_dir().join("ui"), |name| name.ends_with(".css"))
    }

    /// CSS and JS mods for a website: `all.css`/`all.js` plus every file named
    /// after the host or one of its parent domains, e.g. `youtube.com.css`.
    pub fn site_mods(&self, host: &str) -> (String, String) {
        let dir = self.mods_dir().join("sites");
        let mut names = vec!["all".to_string()];
        let host = host.trim_start_matches("www.");
        let parts: Vec<&str> = host.split('.').collect();
        for i in (0..parts.len().saturating_sub(1)).rev() {
            names.push(parts[i..].join("."));
        }
        let mut css = String::new();
        let mut js = String::new();
        for n in names {
            if let Ok(s) = fs::read_to_string(dir.join(format!("{n}.css"))) {
                css.push_str(&s);
                css.push('\n');
            }
            if let Ok(s) = fs::read_to_string(dir.join(format!("{n}.js"))) {
                js.push_str(&format!("try{{\n{s}\n}}catch(e){{console.error('[lurt mod {n}]',e)}}\n"));
            }
        }
        (css, js)
    }
}

fn read_matching(dir: &Path, keep: impl Fn(&str) -> bool) -> String {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(&keep))
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n")
}

fn write_mods_readme(dir: &Path) {
    let path = dir.join("README.txt");
    if path.exists() {
        return;
    }
    let _ = fs::write(
        path,
        "Lurt mods\n\
         =========\n\n\
         ui\\*.css           Restyles the browser itself. Every .css file here is applied\n\
         \x20                  on top of the built-in theme (alphabetical order).\n\n\
         sites\\all.css       Applied to every website.\n\
         sites\\all.js        Runs on every website.\n\
         sites\\<domain>.css  Applied to that site and its subdomains,\n\
         \x20                  e.g. youtube.com.css or news.ycombinator.com.css\n\
         sites\\<domain>.js   Runs on that site after the page has loaded.\n\n\
         Site mods are re-read on every page load, so edits show up on refresh.\n\
         UI mods apply when you press \"Reload mods\" in Settings or restart Lurt.\n",
    );
}
