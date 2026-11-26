//! Privacy protections that don't come from filter lists: clean links, HTTPS upgrades,
//! and the page script that hides fingerprinting details.

use std::collections::HashSet;

/// Query parameters that only exist to follow you from site to site.
const TRACKING_PARAMS: &[&str] = &[
    "fbclid", "gclid", "gclsrc", "dclid", "gbraid", "wbraid", "msclkid", "yclid", "twclid", "ttclid",
    "li_fat_id", "igshid", "igsh", "mc_cid", "mc_eid", "_hsenc", "_hsmi", "__hssc", "__hstc", "__hsfp",
    "hsctatracking", "mkt_tok", "oly_anon_id", "oly_enc_id", "rb_clickid", "s_cid", "vero_conv",
    "vero_id", "wickedid", "epik", "_openstat", "srsltid", "_ga", "_gl", "ga_source", "ga_medium",
    "ga_campaign", "ga_content", "ga_term", "ncid", "ocid", "sr_share", "spm", "scm",
];
/// Share-link trackers that only mean that on these sites.
const SITE_PARAMS: &[(&str, &[&str])] = &[
    ("youtube.com", &["si", "pp", "feature"]),
    ("youtu.be", &["si", "feature"]),
    ("spotify.com", &["si", "context"]),
    ("instagram.com", &["igsh", "img_index"]),
    ("twitter.com", &["s", "t", "ref_src", "ref_url"]),
    ("x.com", &["s", "t", "ref_src", "ref_url"]),
    ("amazon.", &["ref", "ref_", "pf_rd_r", "pf_rd_p", "pd_rd_r", "pd_rd_w", "pd_rd_wg", "linkcode", "tag"]),
    ("tiktok.com", &["is_from_webapp", "sender_device", "_r", "_t"]),
    ("reddit.com", &["share_id", "utm_name", "rdt"]),
];

fn tracking_param(host: &str, key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    if k.starts_with("utm_") || TRACKING_PARAMS.contains(&k.as_str()) {
        return true;
    }
    SITE_PARAMS.iter().any(|(site, keys)| {
        let on = if site.ends_with('.') {
            host.contains(&format!(".{site}")) || host.starts_with(site)
        } else {
            host == *site || host.ends_with(&format!(".{site}"))
        };
        on && keys.contains(&k.as_str())
    })
}

/// Hosts that can't have a public certificate, so they're never upgraded to HTTPS.
fn local_host(host: &str) -> bool {
    host.is_empty()
        || !host.contains('.')
        || host.starts_with('[')
        || host.parse::<std::net::Ipv4Addr>().is_ok()
        || [".local", ".lan", ".home", ".internal", ".localhost", ".test", ".arpa", ".corp"]
            .iter()
            .any(|s| host.ends_with(s))
}

/// The URL with tracking parameters removed and/or upgraded to HTTPS, or None if unchanged.
pub fn clean_url(url: &str, strip: bool, upgrade: bool, skip: &HashSet<String>) -> Option<String> {
    let mut u = url::Url::parse(url).ok()?;
    if !matches!(u.scheme(), "http" | "https") {
        return None;
    }
    let host = u.host_str().unwrap_or("").to_ascii_lowercase();
    let mut changed = false;
    if strip && u.query().is_some() {
        let pairs: Vec<(String, String)> = u.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
        let kept: Vec<&(String, String)> = pairs.iter().filter(|(k, _)| !tracking_param(&host, k)).collect();
        if kept.len() != pairs.len() {
            changed = true;
            if kept.is_empty() {
                u.set_query(None);
            } else {
                u.query_pairs_mut().clear().extend_pairs(kept.iter().map(|(k, v)| (k.as_str(), v.as_str())));
            }
        }
    }
    if upgrade && u.scheme() == "http" && u.port().is_none() && !local_host(&host) && !skip.contains(&host) {
        changed = u.set_scheme("https").is_ok() || changed;
    }
    changed.then(|| u.to_string())
}

/// Runs before each page's own scripts, in every frame. `cfg` is
/// `{"gpc":bool,"fp":bool,"seed":u32}`.
pub fn page_script(gpc: bool, fingerprint: bool, seed: u32) -> Option<String> {
    if !gpc && !fingerprint {
        return None;
    }
    let cfg = format!(r#"{{"gpc":{gpc},"fp":{fingerprint},"seed":{seed}}}"#);
    Some(include_str!("privacy.js").replace("__CFG__", &cfg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_links() {
        let none = HashSet::new();
        assert_eq!(
            clean_url("https://a.com/p?utm_source=x&id=3&fbclid=y", true, false, &none).as_deref(),
            Some("https://a.com/p?id=3")
        );
        assert_eq!(clean_url("https://a.com/p?utm_source=x", true, false, &none).as_deref(), Some("https://a.com/p"));
        assert_eq!(
            clean_url("https://www.youtube.com/watch?v=abc&si=zz", true, false, &none).as_deref(),
            Some("https://www.youtube.com/watch?v=abc")
        );
        // "si" means something else elsewhere.
        assert_eq!(clean_url("https://a.com/?si=1", true, false, &none), None);
        assert_eq!(clean_url("http://example.com/x", false, true, &none).as_deref(), Some("https://example.com/x"));
        assert_eq!(clean_url("http://localhost:3000/", false, true, &none), None);
        assert_eq!(clean_url("http://192.168.1.1/", false, true, &none), None);
        assert_eq!(clean_url("http://router.lan/", false, true, &none), None);
        let skip: HashSet<String> = ["old.com".to_string()].into();
        assert_eq!(clean_url("http://old.com/", false, true, &skip), None);
    }
}
