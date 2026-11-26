//! Small helpers: turning address-bar input into a URL, host extraction.

pub fn search_url(engine: &str, q: &str) -> String {
    let enc: String = url::form_urlencoded::byte_serialize(q.as_bytes()).collect();
    let template = match engine {
        "duckduckgo" => "https://duckduckgo.com/?q=%s",
        "bing" => "https://www.bing.com/search?q=%s",
        "brave" => "https://search.brave.com/search?q=%s",
        "startpage" => "https://www.startpage.com/do/search?q=%s",
        "ecosia" => "https://www.ecosia.org/search?q=%s",
        custom if custom.contains("%s") && custom.starts_with("http") => custom,
        _ => "https://www.google.com/search?q=%s",
    };
    template.replace("%s", &enc)
}

/// A Windows path to a file (`C:\dir\file.pdf`, `\\server\share\x.png`, optionally in
/// quotes as Explorer's "Copy as path" gives it) as a `file:///` URL.
pub fn local_file_url(input: &str) -> Option<String> {
    let s = input.trim().trim_matches('"').trim();
    let b = s.as_bytes();
    let drive = b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/');
    let unc = s.starts_with(r"\\");
    if !drive && !unc {
        return None;
    }
    url::Url::from_file_path(s.replace('/', r"\")).ok().map(|u| u.to_string())
}

/// Decide whether what the user typed is an address or a search.
pub fn resolve_input(input: &str, engine: &str) -> String {
    let s = input.trim();
    if s.is_empty() {
        return String::new();
    }
    // Local files: images, PDFs and HTML open right in the browser.
    if let Some(u) = local_file_url(s) {
        return u;
    }
    let lower = s.to_ascii_lowercase();
    for scheme in ["http://", "https://", "file://", "about:", "edge://", "view-source:", "data:"] {
        if lower.starts_with(scheme) {
            return s.to_string();
        }
    }
    if s.contains(char::is_whitespace) {
        return search_url(engine, s);
    }
    // localhost, IPs and host:port go over plain http.
    let host = lower.split(['/', '?', '#']).next().unwrap_or("");
    let bare = host.split(':').next().unwrap_or("");
    let is_ip = bare.split('.').count() == 4 && bare.split('.').all(|p| p.parse::<u8>().is_ok());
    if bare == "localhost" || is_ip {
        return format!("http://{s}");
    }
    // something.tld (tld = 2+ letters) is treated as a domain.
    if let Some((name, tld)) = bare.rsplit_once('.') {
        if !name.is_empty()
            && tld.len() >= 2
            && tld.chars().all(|c| c.is_ascii_alphabetic())
            && bare.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        {
            return format!("https://{s}");
        }
    }
    search_url(engine, s)
}

pub fn host_of(u: &str) -> String {
    url::Url::parse(u)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.trim_start_matches("www.").to_string()))
        .unwrap_or_default()
}

/// Pages that should not land in history.
pub fn is_internal(u: &str) -> bool {
    u.is_empty() || u.starts_with("about:") || u.starts_with("data:") || u.starts_with("edge:")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_resolution() {
        assert_eq!(resolve_input("github.com", "google"), "https://github.com");
        assert_eq!(resolve_input("localhost:3000", "google"), "http://localhost:3000");
        assert_eq!(resolve_input("192.168.1.1", "google"), "http://192.168.1.1");
        assert_eq!(
            resolve_input("rust lang", "google"),
            "https://www.google.com/search?q=rust+lang"
        );
        assert_eq!(resolve_input("hello", "duckduckgo"), "https://duckduckgo.com/?q=hello");
        assert_eq!(resolve_input("https://a.b/c d", "google"), "https://a.b/c d");
        assert_eq!(resolve_input("v1.2", "google"), "https://www.google.com/search?q=v1.2");
        assert_eq!(resolve_input(r"C:\Users\me\My Docs\a.pdf", "google"), "file:///C:/Users/me/My%20Docs/a.pdf");
        assert_eq!(resolve_input(r#""D:\pics\cat.png""#, "google"), "file:///D:/pics/cat.png");
        assert_eq!(resolve_input("C:/site/index.html", "google"), "file:///C:/site/index.html");
        assert_eq!(resolve_input(r"\\nas\share\x.jpg", "google"), "file://nas/share/x.jpg");
        assert_eq!(resolve_input("file:///C:/a.html", "google"), "file:///C:/a.html");
    }
}
