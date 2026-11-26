//! Import bookmarks and history from other browsers and from Google Takeout.
//!
//! Sources:
//! - Chromium browsers installed here (Chrome, Edge, Brave, Vivaldi, Opera): every profile's
//!   `Bookmarks` (JSON) and `History` (SQLite, copied first because the browser may hold it open).
//! - Files: Google Takeout `BrowserHistory.json`, and bookmark `.html` exports (Netscape
//!   format, which Takeout, Chrome, Edge and Firefox all produce).
//!
//! Passwords and cookies are deliberately not here: Chromium encrypts them so that only the
//! browser that saved them can read them.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use rusqlite::{Connection, OpenFlags, params};
use serde::Serialize;
use serde_json::Value;

/// One bookmark or history entry. Times are Unix seconds.
pub struct Item {
    pub url: String,
    pub title: String,
    pub time: i64,
    pub visits: i64,
    /// Bookmarks: the folders it sits in, outermost first. Empty = on the bar itself.
    pub folder: Vec<String>,
}

#[derive(Default)]
pub struct Batch {
    pub bookmarks: Vec<Item>,
    pub history: Vec<Item>,
}

#[derive(Serialize, Clone)]
pub struct Source {
    pub id: String,
    pub browser: String,
    pub profile: String,
}

/// Chromium "WebKit" timestamps: microseconds since 1601-01-01.
fn webkit_to_unix(us: i64) -> i64 {
    (us / 1_000_000 - 11_644_473_600).max(0)
}

fn keep(url: &str) -> bool {
    (url.starts_with("http://") || url.starts_with("https://")) && url.len() < 4096
}

fn browsers() -> Vec<(&'static str, PathBuf)> {
    let local = dirs::data_local_dir().unwrap_or_default();
    let roaming = dirs::data_dir().unwrap_or_default();
    vec![
        ("Chrome", local.join(r"Google\Chrome\User Data")),
        ("Edge", local.join(r"Microsoft\Edge\User Data")),
        ("Brave", local.join(r"BraveSoftware\Brave-Browser\User Data")),
        ("Vivaldi", local.join(r"Vivaldi\User Data")),
        ("Opera", roaming.join(r"Opera Software\Opera Stable")),
        ("Opera GX", roaming.join(r"Opera Software\Opera GX Stable")),
    ]
}

/// Every browser profile on this machine that has bookmarks or history.
pub fn detect() -> Vec<Source> {
    let mut out = Vec::new();
    for (browser, root) in browsers() {
        if !root.exists() {
            continue;
        }
        // Opera keeps a single profile in the root itself.
        if root.join("History").exists() || root.join("Bookmarks").exists() {
            out.push(Source { id: root.to_string_lossy().into(), browser: browser.into(), profile: String::new() });
            continue;
        }
        let names = profile_names(&root);
        let Ok(rd) = fs::read_dir(&root) else { continue };
        let mut dirs: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.join("History").exists() || p.join("Bookmarks").exists())
            .collect();
        dirs.sort();
        for dir in dirs {
            let key = dir.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            let profile = names.get(&key).cloned().unwrap_or(key);
            out.push(Source { id: dir.to_string_lossy().into(), browser: browser.into(), profile });
        }
    }
    out
}

/// Friendly profile names (usually the Google account name) from `Local State`.
fn profile_names(root: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Ok(text) = fs::read_to_string(root.join("Local State")) else { return map };
    let Ok(v) = serde_json::from_str::<Value>(&text) else { return map };
    if let Some(cache) = v["profile"]["info_cache"].as_object() {
        for (dir, info) in cache {
            let name = info["name"].as_str().unwrap_or("");
            let user = info["user_name"].as_str().unwrap_or("");
            let label = match (name.is_empty(), user.is_empty()) {
                (false, false) => format!("{name} ({user})"),
                (false, true) => name.to_string(),
                (true, false) => user.to_string(),
                _ => dir.clone(),
            };
            map.insert(dir.clone(), label);
        }
    }
    map
}

/// Read one detected profile. `id` must be a path previously returned by `detect`.
pub fn read_profile(id: &str, scratch: &Path) -> Result<Batch, String> {
    if !detect().iter().any(|s| s.id == id) {
        return Err("Unknown browser profile".into());
    }
    let dir = PathBuf::from(id);
    let mut batch = Batch::default();
    if let Ok(text) = fs::read_to_string(dir.join("Bookmarks")) {
        chromium_bookmarks(&text, &mut batch.bookmarks);
    }
    let hist = dir.join("History");
    if hist.exists() {
        // The browser may have the database open; read a copy.
        let copy = scratch.join("import-history.tmp");
        fs::copy(&hist, &copy).map_err(|_| {
            "Couldn't read its history. Close that browser and try again.".to_string()
        })?;
        let r = chromium_history(&copy, &mut batch.history);
        let _ = fs::remove_file(&copy);
        r?;
    }
    Ok(batch)
}

/// Chrome's `Bookmarks` file. The bookmarks bar lands on Lurt's bar as-is; "Other
/// bookmarks" and "Mobile bookmarks" become folders on the bar.
fn chromium_bookmarks(text: &str, out: &mut Vec<Item>) {
    fn walk(node: &Value, path: &mut Vec<String>, out: &mut Vec<Item>) {
        match node["type"].as_str() {
            Some("url") => {
                let url = node["url"].as_str().unwrap_or("");
                if keep(url) {
                    let added = node["date_added"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0);
                    out.push(Item {
                        url: url.into(),
                        title: node["name"].as_str().unwrap_or("").into(),
                        time: webkit_to_unix(added),
                        visits: 0,
                        folder: path.clone(),
                    });
                }
            }
            _ => {
                if let Some(children) = node["children"].as_array() {
                    for c in children {
                        let sub = c["type"].as_str() == Some("folder");
                        if sub {
                            path.push(c["name"].as_str().unwrap_or("Folder").to_string());
                        }
                        walk(c, path, out);
                        if sub {
                            path.pop();
                        }
                    }
                }
            }
        }
    }
    if let Ok(v) = serde_json::from_str::<Value>(text) {
        if let Some(roots) = v["roots"].as_object() {
            for (key, root) in roots {
                let mut path = match key.as_str() {
                    "bookmark_bar" => Vec::new(),
                    "other" => vec!["Other bookmarks".to_string()],
                    "synced" => vec!["Mobile bookmarks".to_string()],
                    _ => continue,
                };
                walk(root, &mut path, out);
            }
        }
    }
}

fn chromium_history(db: &Path, out: &mut Vec<Item>) -> Result<(), String> {
    let conn = Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("Couldn't open history: {e}"))?;
    let mut stmt = conn
        .prepare("SELECT url, title, visit_count, last_visit_time FROM urls WHERE hidden = 0")
        .map_err(|e| format!("Unexpected history format: {e}"))?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Item {
                url: r.get(0)?,
                title: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                visits: r.get::<_, Option<i64>>(2)?.unwrap_or(1).max(1),
                time: webkit_to_unix(r.get::<_, Option<i64>>(3)?.unwrap_or(0)),
                folder: Vec::new(),
            })
        })
        .map_err(|e| e.to_string())?;
    out.extend(rows.flatten().filter(|i| keep(&i.url)));
    Ok(())
}

/// Parse an uploaded file: Takeout `BrowserHistory.json`, a Chrome `Bookmarks` JSON,
/// or a bookmarks `.html` export.
pub fn read_file(name: &str, text: &str) -> Result<Batch, String> {
    let mut batch = Batch::default();
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".html") || lower.ends_with(".htm") || text.contains("NETSCAPE-Bookmark-file") {
        netscape_bookmarks(text, &mut batch.bookmarks);
    } else {
        let v: Value = serde_json::from_str(text).map_err(|_| "That file isn't JSON or HTML.".to_string())?;
        if let Some(list) = v["Browser History"].as_array() {
            takeout_history(list, &mut batch.history);
        } else if v["roots"].is_object() {
            chromium_bookmarks(text, &mut batch.bookmarks);
        } else {
            return Err("Unrecognized file. Use BrowserHistory.json or a bookmarks .html export.".into());
        }
    }
    if batch.bookmarks.is_empty() && batch.history.is_empty() {
        return Err("Nothing to import in that file.".into());
    }
    Ok(batch)
}

fn takeout_history(list: &[Value], out: &mut Vec<Item>) {
    // Takeout lists every visit; fold them into one entry per URL.
    let mut by_url: HashMap<String, Item> = HashMap::new();
    for v in list {
        let url = v["url"].as_str().unwrap_or("");
        if !keep(url) {
            continue;
        }
        let time = v["time_usec"].as_i64().unwrap_or(0) / 1_000_000;
        let title = v["title"].as_str().unwrap_or("");
        let e = by_url.entry(url.to_string()).or_insert_with(|| Item {
            url: url.into(),
            title: String::new(),
            time: 0,
            visits: 0,
            folder: Vec::new(),
        });
        e.visits += 1;
        if time >= e.time {
            e.time = time;
            if !title.is_empty() {
                e.title = title.into();
            }
        }
    }
    out.extend(by_url.into_values());
}

/// Minimal parser for bookmark exports: `<H3>Folder</H3><DL>…</DL>` folders holding
/// `<A HREF="..." ADD_DATE="...">Title</A>` entries. The toolbar folder (the one marked
/// `PERSONAL_TOOLBAR_FOLDER`) maps to Lurt's bar itself.
fn netscape_bookmarks(html: &str, out: &mut Vec<Item>) {
    let lower = html.to_ascii_lowercase();
    // One entry per open <DL>: the folder name it adds to the path, if any.
    let mut stack: Vec<Option<String>> = Vec::new();
    let mut pending: Option<Option<String>> = None;
    let mut pos = 0;
    loop {
        let next = ["<a ", "<h3", "<dl", "</dl"]
            .iter()
            .filter_map(|t| lower[pos..].find(t).map(|i| (pos + i, *t)))
            .min_by_key(|(i, _)| *i);
        let Some((start, kind)) = next else { break };
        let Some(tag_end) = lower[start..].find('>').map(|j| start + j) else { break };
        match kind {
            "<dl" => {
                stack.push(pending.take().flatten());
                pos = tag_end + 1;
            }
            "</dl" => {
                stack.pop();
                pos = tag_end + 1;
            }
            "<h3" => {
                let close = lower[tag_end..].find("</h3>").map_or(html.len(), |j| tag_end + j);
                let tag = &lower[start..tag_end];
                let name = decode_entities(html[tag_end + 1..close].trim());
                pending = Some(if tag.contains("personal_toolbar_folder") { None } else { Some(name) });
                pos = close.min(html.len());
            }
            _ => {
                let tag = &html[start..tag_end];
                let close = lower[tag_end..].find("</a>").map_or(html.len(), |j| tag_end + j);
                let title = decode_entities(html[tag_end + 1..close].trim());
                let url = attr(tag, "href").map(|u| decode_entities(&u)).unwrap_or_default();
                if keep(&url) {
                    let time = attr(tag, "add_date").and_then(|d| d.parse().ok()).unwrap_or(0);
                    let folder = stack.iter().flatten().cloned().collect();
                    out.push(Item { url, title, time, visits: 0, folder });
                }
                pos = close.min(html.len());
            }
        }
        if pos >= html.len() {
            break;
        }
    }
}

fn attr(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let at = lower.find(&format!("{name}="))? + name.len() + 1;
    let rest = &tag[at..];
    let (q, rest) = match rest.chars().next()? {
        c @ ('"' | '\'') => (c, &rest[1..]),
        _ => (' ', rest),
    };
    Some(rest.split(q).next().unwrap_or("").to_string())
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

/// Merge into Lurt's database. Returns (bookmarks added, history entries merged).
pub fn save(db_path: &Path, batch: &Batch) -> Result<(usize, usize), String> {
    let mut conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    let _ = conn.busy_timeout(std::time::Duration::from_secs(5));
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut added = 0;
    crate::store::migrate_bookmarks(&tx);
    {
        // Folders are matched by name, so importing twice doesn't duplicate them.
        let mut folders: std::collections::HashMap<Vec<String>, i64> = Default::default();
        for b in &batch.bookmarks {
            let mut parent = 0;
            for depth in 1..=b.folder.len() {
                let key = b.folder[..depth].to_vec();
                parent = *folders
                    .entry(key)
                    .or_insert_with(|| crate::store::folder_in(&tx, parent, &b.folder[depth - 1]));
            }
            let exists = tx
                .query_row("SELECT parent FROM bookmarks WHERE url = ?1", params![b.url], |r| r.get::<_, i64>(0))
                .ok();
            match exists {
                // Already saved: only file it into its folder if it's still loose on the bar.
                Some(0) if parent != 0 => {
                    let pos = crate::store::next_pos(&tx, parent);
                    let _ = tx.execute(
                        "UPDATE bookmarks SET parent = ?2, pos = ?3 WHERE url = ?1",
                        params![b.url, parent, pos],
                    );
                }
                Some(_) => {}
                None => {
                    let pos = crate::store::next_pos(&tx, parent);
                    added += tx
                        .execute(
                            "INSERT OR IGNORE INTO bookmarks(url, title, created, parent, pos) VALUES(?1, ?2, ?3, ?4, ?5)",
                            params![b.url, b.title, b.time, parent, pos],
                        )
                        .unwrap_or(0);
                }
            }
        }
        let mut stmt = tx
            .prepare(
                "INSERT INTO history(url, title, visits, last) VALUES(?1, ?2, ?3, ?4)
                 ON CONFLICT(url) DO UPDATE SET
                   visits = MAX(visits, excluded.visits),
                   last = MAX(last, excluded.last),
                   title = CASE WHEN title = '' THEN excluded.title ELSE title END",
            )
            .map_err(|e| e.to_string())?;
        for h in &batch.history {
            let _ = stmt.execute(params![h.url, h.title, h.visits.max(1), h.time]);
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok((added, batch.history.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_netscape_html() {
        let html = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
            <DL><p><DT><H3>Bar</H3><DL><p>
            <DT><A HREF="https://rust-lang.org/" ADD_DATE="1700000000">Rust &amp; Co</A>
            <DT><A HREF="javascript:void(0)">skip</A>
            <DT><a href='https://example.com/?a=1&amp;b=2'>Ex</a>
            </DL></DL>"#;
        let b = read_file("bookmarks.html", html).unwrap();
        assert_eq!(b.bookmarks.len(), 2);
        assert_eq!(b.bookmarks[0].title, "Rust & Co");
        assert_eq!(b.bookmarks[0].time, 1_700_000_000);
        assert_eq!(b.bookmarks[1].url, "https://example.com/?a=1&b=2");
    }

    #[test]
    fn folds_takeout_history() {
        let json = r#"{"Browser History":[
            {"title":"A","url":"https://a.com/","time_usec":1700000000000000},
            {"title":"A2","url":"https://a.com/","time_usec":1700000100000000},
            {"title":"x","url":"chrome://settings","time_usec":1}]}"#;
        let b = read_file("BrowserHistory.json", json).unwrap();
        assert_eq!(b.history.len(), 1);
        assert_eq!(b.history[0].visits, 2);
        assert_eq!(b.history[0].title, "A2");
        assert_eq!(b.history[0].time, 1_700_000_100);
    }

    #[test]
    fn walks_chromium_bookmarks() {
        let json = r#"{"roots":{"bookmark_bar":{"type":"folder","children":[
            {"type":"url","name":"G","url":"https://g.com/","date_added":"13340000000000000"},
            {"type":"folder","children":[{"type":"url","name":"H","url":"https://h.com/"}]}]},
            "other":{"type":"folder","children":[]}}}"#;
        let b = read_file("Bookmarks", json).unwrap();
        assert_eq!(b.bookmarks.len(), 2);
        assert!(b.bookmarks[0].time > 1_600_000_000);
        assert!(b.bookmarks[0].folder.is_empty());
    }

    #[test]
    fn keeps_folders() {
        let json = r#"{"roots":{"bookmark_bar":{"type":"folder","children":[
            {"type":"folder","name":"Dev","children":[{"type":"folder","name":"Rust","children":[
              {"type":"url","name":"R","url":"https://rust-lang.org/"}]}]}]},
            "other":{"type":"folder","children":[{"type":"url","name":"O","url":"https://o.com/"}]}}}"#;
        let b = read_file("Bookmarks", json).unwrap();
        let f: Vec<_> = b.bookmarks.iter().map(|i| i.folder.join("/")).collect();
        assert!(f.contains(&"Dev/Rust".to_string()) && f.contains(&"Other bookmarks".to_string()));

        let html = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1><DL><p>
          <DT><H3 PERSONAL_TOOLBAR_FOLDER="true">Bookmarks bar</H3><DL><p>
            <DT><A HREF="https://a.com/">A</A>
            <DT><H3>News</H3><DL><p><DT><A HREF="https://n.com/">N</A></DL><p>
          </DL><p>
          <DT><A HREF="https://loose.com/">L</A>
        </DL>"#;
        let b = read_file("bookmarks.html", html).unwrap();
        let f: Vec<_> = b.bookmarks.iter().map(|i| (i.url.as_str(), i.folder.join("/"))).collect();
        assert_eq!(f, [("https://a.com/", String::new()), ("https://n.com/", "News".into()), ("https://loose.com/", String::new())]);
    }
}

#[cfg(test)]
mod live {
    /// Imports every local browser profile into a throwaway profile at $LURT_DATA.
    #[test]
    #[ignore]
    fn import_local_browsers() {
        let store = crate::store::Store::open();
        for s in super::detect() {
            let r = super::read_profile(&s.id, &store.dir).and_then(|b| {
                let (nb, nh) = (b.bookmarks.len(), b.history.len());
                super::save(&store.db_path(), &b).map(|r| (nb, nh, r))
            });
            match r {
                Ok((nb, nh, (added, merged))) => {
                    println!("{} [{}]: read {nb} bookmarks, {nh} history -> added {added}, merged {merged}", s.browser, s.profile.len())
                }
                Err(e) => println!("{} : ERROR {e}", s.browser),
            }
        }
        println!("top sites now: {}", store.top_sites(8).len());
    }
}
