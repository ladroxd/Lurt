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
    /// How big websites are shown, in percent (25..=500). Ctrl + scroll changes it.
    pub page_zoom: u32,
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
    /// Filter list ids (see `blocker::LISTS`).
    pub filter_lists: Vec<String>,
    /// Send Global Privacy Control and Do Not Track to sites.
    pub gpc: bool,
    /// Remove tracking parameters (utm_, fbclid...) from links you open.
    pub strip_tracking: bool,
    /// Block hyperlink auditing pings.
    pub block_pings: bool,
    /// Load sites over HTTPS when they support it.
    pub https_upgrade: bool,
    /// Hide hardware details and add noise to canvas and audio reads.
    pub fingerprint: bool,
    /// WebRTC uses only the public network interface (no local IPs; respects a VPN). Restart.
    pub webrtc_protect: bool,
    /// Secure DNS provider: "off" | "cloudflare" | "quad9" | "mullvad" | "google". Restart.
    pub doh: String,
    /// Site permissions: "ask" | "block".
    pub perm_location: String,
    pub perm_camera: String,
    pub perm_mic: String,
    pub perm_notifications: String,
    /// Every new tab is private.
    pub always_private: bool,
    /// Where downloads go; empty means the Windows Downloads folder.
    pub download_dir: String,
    /// Ask where to save every download.
    pub download_ask: bool,
    /// Per file type: what to do with downloads of that type.
    pub download_rules: Vec<DownloadRule>,
    /// Delete on quit.
    pub exit_clear_history: bool,
    pub exit_clear_cookies: bool,
    /// Unload background tabs after this many minutes. 0 = never.
    pub sleep_minutes: u32,
    /// Let cross-site iframes (ads, payment and video embeds) share their page's process
    /// instead of one process each. About half the memory; applies after a restart.
    pub share_processes: bool,
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
    /// Google accounts Lurt already offered to import browser data for.
    pub import_offered: Vec<String>,
    /// Bookmarks bar under the toolbar: "always" | "start" (start page only) | "never".
    pub bookmarks_bar: String,
    /// Delete history older than this many days, automatically. 0 = keep forever.
    pub history_days: u32,
}

/// What to do with downloads of one file type.
#[derive(Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct DownloadRule {
    /// Lowercase extension without the dot: "exe".
    pub ext: String,
    /// "allow" | "ask" (where to save) | "open" (when finished) | "block"
    pub action: String,
}

/// A download in the downloads panel.
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Download {
    pub id: u32,
    pub name: String,
    pub path: String,
    pub url: String,
    /// Bytes; -1 or 0 when the size is unknown.
    pub total: i64,
    pub recv: i64,
    /// "progress" | "paused" | "done" | "failed" | "canceled"
    pub state: String,
    /// Why it failed, in words.
    pub reason: String,
    /// Unix seconds when it started.
    pub at: i64,
    /// From a private tab: shown but not saved.
    #[serde(skip)]
    pub private: bool,
    /// Open the file when it finishes (a file type rule).
    #[serde(skip)]
    pub open: bool,
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
            page_zoom: 100,
            search: "google".into(),
            homepage: String::new(),
            restore_session: true,
            adblock: true,
            adblock_allow: Vec::new(),
            tracking: "balanced".into(),
            filter_lists: ["easylist", "easyprivacy", "peterlowe", "ublock", "badware"].map(String::from).to_vec(),
            gpc: true,
            strip_tracking: true,
            block_pings: true,
            https_upgrade: true,
            fingerprint: false,
            webrtc_protect: false,
            doh: "off".into(),
            perm_location: "ask".into(),
            perm_camera: "ask".into(),
            perm_mic: "ask".into(),
            perm_notifications: "ask".into(),
            always_private: false,
            download_dir: String::new(),
            download_ask: false,
            download_rules: Vec::new(),
            exit_clear_history: false,
            exit_clear_cookies: false,
            sleep_minutes: 30,
            share_processes: true,
            ui_css: String::new(),
            save_passwords: true,
            autofill: true,
            shortcuts: Vec::new(),
            hidden_tiles: Vec::new(),
            import_offered: Vec::new(),
            bookmarks_bar: "always".into(),
            history_days: 0,
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

/// The bookmarks bar as flat lists; `parent` 0 is the bar itself.
#[derive(Serialize)]
pub struct BarFolder {
    pub id: i64,
    pub title: String,
    pub parent: i64,
    pub pos: f64,
}

#[derive(Serialize)]
pub struct BarItem {
    pub url: String,
    pub title: String,
    pub parent: i64,
    pub pos: f64,
}

/// One entry in a reorder request: a folder (by id) or a bookmark (by url).
pub enum Slot {
    Folder(i64),
    Mark(String),
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

/// Bookmarks gained a place on the bar (folder + order). Older profiles get the
/// columns, keeping their old newest-first order.
pub fn migrate_bookmarks(db: &Connection) {
    if db.prepare("SELECT parent FROM bookmarks LIMIT 0").is_err() {
        let _ = db.execute_batch(
            "ALTER TABLE bookmarks ADD COLUMN parent INTEGER NOT NULL DEFAULT 0;
             ALTER TABLE bookmarks ADD COLUMN pos REAL NOT NULL DEFAULT 0;
             UPDATE bookmarks SET pos = -created;",
        );
    }
}

/// Find a folder by name inside `parent`, creating it at the end if it's missing.
pub fn folder_in(db: &Connection, parent: i64, title: &str) -> i64 {
    if let Ok(id) = db.query_row(
        "SELECT id FROM bm_folders WHERE parent = ?1 AND title = ?2",
        params![parent, title],
        |r| r.get(0),
    ) {
        return id;
    }
    let pos = next_pos(db, parent);
    let _ = db.execute(
        "INSERT INTO bm_folders(title, parent, pos) VALUES(?1, ?2, ?3)",
        params![title, parent, pos],
    );
    db.last_insert_rowid()
}

/// The position after the last folder or bookmark in `parent`.
pub fn next_pos(db: &Connection, parent: i64) -> f64 {
    db.query_row(
        "SELECT MAX(p) FROM (SELECT MAX(pos) p FROM bookmarks WHERE parent = ?1
                             UNION ALL SELECT MAX(pos) p FROM bm_folders WHERE parent = ?1)",
        params![parent],
        |r| r.get::<_, Option<f64>>(0),
    )
    .ok()
    .flatten()
    .map_or(0.0, |p| p + 1.0)
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
                url TEXT PRIMARY KEY, title TEXT NOT NULL DEFAULT '', created INTEGER NOT NULL);
             CREATE TABLE IF NOT EXISTS bm_folders(
                id INTEGER PRIMARY KEY, title TEXT NOT NULL DEFAULT '',
                parent INTEGER NOT NULL DEFAULT 0, pos REAL NOT NULL DEFAULT 0);
             CREATE TABLE IF NOT EXISTS icons(host TEXT PRIMARY KEY, url TEXT NOT NULL DEFAULT '');",
        );
        migrate_bookmarks(&db);
        Self { db, dir }
    }

    pub fn db_path(&self) -> PathBuf {
        self.dir.join("lurt.db")
    }

    pub fn webview_dir() -> PathBuf {
        // A custom profile (LURT_DATA) keeps its own cookies and cache too.
        if let Some(d) = std::env::var_os("LURT_DATA") {
            return PathBuf::from(d).join("WebView");
        }
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

    /// Sizes the startup skeleton copies (see `splash::Shape`).
    pub fn load_shape(&self) -> crate::splash::Shape {
        fs::read_to_string(self.dir.join("skeleton.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save_shape(&self, s: &crate::splash::Shape) {
        if let Ok(json) = serde_json::to_string(s) {
            let _ = fs::write(self.dir.join("skeleton.json"), json);
        }
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

    /// Delete everything Lurt keeps in its own files: history, bookmarks and folders,
    /// the saved session and the skeleton sizes. (Settings are reset by the caller.)
    pub fn wipe(&self) {
        let _ = self.db.execute_batch("DELETE FROM history; DELETE FROM bookmarks; DELETE FROM bm_folders; DELETE FROM icons; VACUUM;");
        let _ = fs::remove_file(self.dir.join("session.json"));
        let _ = fs::remove_file(self.dir.join("skeleton.json"));
        let _ = fs::remove_file(self.dir.join("downloads.json"));
    }

    /// The downloads panel's list, newest first. Downloads cut off by closing Lurt show as failed.
    pub fn load_downloads(&self) -> Vec<Download> {
        let mut list: Vec<Download> = fs::read_to_string(self.dir.join("downloads.json"))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        for d in &mut list {
            if d.state == "progress" || d.state == "paused" {
                d.state = "failed".into();
                d.reason = "Lurt was closed".into();
            }
        }
        list
    }
    pub fn save_downloads(&self, list: &[Download]) {
        let keep: Vec<&Download> = list.iter().filter(|d| !d.private).take(100).collect();
        if let Ok(t) = serde_json::to_string(&keep) {
            let _ = fs::write(self.dir.join("downloads.json"), t);
        }
    }

    /// Icons that sites declare in their pages (for sites without /favicon.ico):
    /// host -> icon URL, or "" when the site has none.
    pub fn icons(&self) -> Vec<(String, String)> {
        let Ok(mut q) = self.db.prepare("SELECT host, url FROM icons") else { return Vec::new() };
        q.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).map(|it| it.flatten().collect()).unwrap_or_default()
    }

    /// Remember a site's icon; true if it changed.
    pub fn set_icon(&self, host: &str, url: &str) -> bool {
        let old: Option<String> = self.db.query_row("SELECT url FROM icons WHERE host=?1", [host], |r| r.get(0)).ok();
        if old.as_deref() == Some(url) {
            return false;
        }
        let _ = self.db.execute("INSERT OR REPLACE INTO icons(host, url) VALUES(?1, ?2)", [host, url]);
        true
    }

    /// How much is saved: (bookmarks, folders, history entries).
    pub fn counts(&self) -> (i64, i64, i64) {
        let n = |sql: &str| self.db.query_row(sql, [], |r| r.get::<_, i64>(0)).unwrap_or(0);
        (n("SELECT COUNT(*) FROM bookmarks"), n("SELECT COUNT(*) FROM bm_folders"), n("SELECT COUNT(*) FROM history"))
    }

    /// Delete these pages from history. Returns how many went.
    pub fn delete_history_many(&self, urls: &[String]) -> usize {
        let Ok(tx) = self.db.unchecked_transaction() else { return 0 };
        let mut n = 0;
        for u in urls {
            n += tx.execute("DELETE FROM history WHERE url = ?1", params![u]).unwrap_or(0);
        }
        let _ = tx.commit();
        n
    }

    /// Delete what was visited in the last `secs` seconds (0 = everything).
    pub fn clear_history_since(&self, secs: i64) -> usize {
        if secs <= 0 {
            let n = self.db.query_row("SELECT COUNT(*) FROM history", [], |r| r.get::<_, i64>(0)).unwrap_or(0);
            self.clear_history();
            return n as usize;
        }
        self.db
            .execute("DELETE FROM history WHERE last >= ?1", params![now() - secs])
            .unwrap_or(0)
    }

    /// Auto-delete: drop history older than `days` days.
    pub fn prune_history(&self, days: u32) -> usize {
        if days == 0 {
            return 0;
        }
        self.db
            .execute("DELETE FROM history WHERE last < ?1", params![now() - i64::from(days) * 86_400])
            .unwrap_or(0)
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
            self.add_bookmark(url, title, 0);
            true
        }
    }

    /// Add (or move) a bookmark to the end of a folder on the bar.
    pub fn add_bookmark(&self, url: &str, title: &str, parent: i64) {
        let pos = next_pos(&self.db, parent);
        let _ = self.db.execute(
            "INSERT INTO bookmarks(url, title, created, parent, pos) VALUES(?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(url) DO UPDATE SET parent = ?4, pos = ?5,
               title = CASE WHEN ?2 = '' THEN title ELSE ?2 END",
            params![url, title, now(), parent, pos],
        );
    }

    /// Everything on the bar, for the UI to build its tree.
    pub fn bar(&self) -> (Vec<BarFolder>, Vec<BarItem>) {
        let folders = self
            .db
            .prepare("SELECT id, title, parent, pos FROM bm_folders")
            .and_then(|mut s| {
                s.query_map([], |r| {
                    Ok(BarFolder { id: r.get(0)?, title: r.get(1)?, parent: r.get(2)?, pos: r.get(3)? })
                })
                .map(|rows| rows.flatten().collect())
            })
            .unwrap_or_default();
        let items = self
            .db
            .prepare("SELECT url, title, parent, pos FROM bookmarks")
            .and_then(|mut s| {
                s.query_map([], |r| {
                    Ok(BarItem { url: r.get(0)?, title: r.get(1)?, parent: r.get(2)?, pos: r.get(3)? })
                })
                .map(|rows| rows.flatten().collect())
            })
            .unwrap_or_default();
        (folders, items)
    }

    pub fn add_folder(&self, title: &str, parent: i64) -> i64 {
        let pos = next_pos(&self.db, parent);
        let _ = self.db.execute(
            "INSERT INTO bm_folders(title, parent, pos) VALUES(?1, ?2, ?3)",
            params![title, parent, pos],
        );
        self.db.last_insert_rowid()
    }

    pub fn rename_folder(&self, id: i64, title: &str) {
        let _ = self.db.execute("UPDATE bm_folders SET title = ?2 WHERE id = ?1", params![id, title]);
    }

    /// Delete a folder with everything inside it.
    pub fn delete_folder(&self, id: i64) {
        let _ = self.db.execute_batch(&format!(
            "WITH RECURSIVE sub(id) AS (SELECT {id} UNION SELECT f.id FROM bm_folders f JOIN sub ON f.parent = sub.id)
             DELETE FROM bookmarks WHERE parent IN sub;
             WITH RECURSIVE sub(id) AS (SELECT {id} UNION SELECT f.id FROM bm_folders f JOIN sub ON f.parent = sub.id)
             DELETE FROM bm_folders WHERE id IN sub;"
        ));
    }

    /// Change a bookmark's address and/or name.
    pub fn edit_bookmark(&self, url: &str, new_url: &str, title: &str) {
        if url != new_url && self.is_bookmarked(new_url) {
            self.remove_bookmark(new_url);
        }
        let _ = self.db.execute(
            "UPDATE bookmarks SET url = ?2, title = ?3 WHERE url = ?1",
            params![url, new_url, title],
        );
    }

    /// Put these folders and bookmarks into `parent`, in this order.
    pub fn reorder(&self, parent: i64, slots: &[Slot]) {
        // A folder can't go inside itself or one of its own subfolders.
        let mut p = parent;
        for _ in 0..64 {
            if p == 0 {
                break;
            }
            if slots.iter().any(|s| matches!(s, Slot::Folder(id) if *id == p)) {
                return;
            }
            p = self
                .db
                .query_row("SELECT parent FROM bm_folders WHERE id = ?1", params![p], |r| r.get(0))
                .unwrap_or(0);
        }
        for (i, s) in slots.iter().enumerate() {
            let _ = match s {
                Slot::Folder(id) => self.db.execute(
                    "UPDATE bm_folders SET parent = ?2, pos = ?3 WHERE id = ?1",
                    params![id, parent, i as f64],
                ),
                Slot::Mark(url) => self.db.execute(
                    "UPDATE bookmarks SET parent = ?2, pos = ?3 WHERE url = ?1",
                    params![url, parent, i as f64],
                ),
            };
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
