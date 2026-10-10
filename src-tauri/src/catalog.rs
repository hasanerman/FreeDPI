use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Region {
    #[default]
    Auto,
    Turkey,
    Russia,
    Other,
}

pub struct Target {
    pub id: &'static str,
    pub name: &'static str,
    pub url: &'static str,
}

pub const TARGETS: &[Target] = &[
    Target { id: "discord", name: "Discord", url: "https://discord.com/api/v9/gateway" },
    Target { id: "youtube", name: "YouTube", url: "https://www.youtube.com/generate_204" },
    Target { id: "roblox", name: "Roblox", url: "https://www.roblox.com/" },
    Target { id: "wattpad", name: "Wattpad", url: "https://www.wattpad.com/" },
];

#[cfg_attr(not(windows), allow(dead_code))]
pub fn goodbyedpi_profiles(region: Region) -> Vec<&'static str> {
    const TURKEY: &[&str] = &["-5 --wrong-seq --reverse-frag", "-5 --set-ttl 5", "-5", "--set-ttl 3", "-9"];
    const RUSSIA: &[&str] = &["-9", "-5 --wrong-seq --reverse-frag", "-8", "-7"];
    const COMMON: &[&str] = &["-9", "-6", "-7", "-8", "-5", "-4", "-3", "-2", "-1"];
    let lists: &[&[&str]] = match region {
        Region::Turkey => &[TURKEY, COMMON],
        Region::Russia => &[RUSSIA, COMMON],
        Region::Other => &[COMMON],
        Region::Auto => &[TURKEY, RUSSIA, COMMON],
    };
    let mut out: Vec<&'static str> = Vec::new();
    for profile in lists.iter().flat_map(|list| list.iter()) {
        if !out.contains(profile) {
            out.push(profile);
        }
    }
    out
}

#[cfg_attr(windows, allow(dead_code))]
pub fn proxy_profiles(_region: Region) -> Vec<&'static str> {
    vec![
        "split=1",
        "tlsrec=sni+1 split=1",
        "split=2",
        "split=1 delay=50",
        "split=sni+1",
        "tlsrec=sni+1",
        "tlsrec=sni+1 split=sni+3",
    ]
}

pub fn ordered(mut profiles: Vec<&'static str>, preferred: Option<&str>) -> Vec<String> {
    if let Some(index) = preferred.and_then(|p| profiles.iter().position(|x| *x == p)) {
        let item = profiles.remove(index);
        profiles.insert(0, item);
    }
    profiles.into_iter().map(String::from).collect()
}

pub fn target_urls(ids: &[String], custom: &[String]) -> Vec<String> {
    let mut urls: Vec<String> = TARGETS
        .iter()
        .filter(|t| ids.iter().any(|id| id == t.id))
        .map(|t| t.url.to_string())
        .collect();
    urls.extend(custom.iter().filter_map(|d| normalize_domain(d)).map(|d| format!("https://{d}/")));
    if urls.is_empty() {
        urls.push(TARGETS[0].url.to_string());
    }
    urls
}

pub fn normalize_domain(input: &str) -> Option<String> {
    let lower = input.trim().to_ascii_lowercase();
    let rest = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
        .unwrap_or(&lower);
    let host = rest.split(['/', '?', '#', ':']).next()?.trim_end_matches('.');
    let valid = !host.is_empty()
        && host.len() <= 253
        && host.contains('.')
        && host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        });
    valid.then(|| host.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domains_are_normalized() {
        assert_eq!(normalize_domain(" https://Discord.com/app "), Some("discord.com".into()));
        assert_eq!(normalize_domain("x.com:443"), Some("x.com".into()));
        assert_eq!(normalize_domain("localhost"), None);
        assert_eq!(normalize_domain("bad_domain.com"), None);
        assert_eq!(normalize_domain("-a.com"), None);
    }

    #[test]
    fn preferred_profile_goes_first() {
        assert_eq!(ordered(vec!["a", "b", "c"], Some("c")), vec!["c", "a", "b"]);
        assert_eq!(ordered(vec!["a", "b"], Some("z")), vec!["a", "b"]);
    }

    #[test]
    fn auto_region_has_no_duplicates() {
        let profiles = goodbyedpi_profiles(Region::Auto);
        let mut unique = profiles.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(profiles.len(), unique.len());
        assert_eq!(profiles[0], "-5 --wrong-seq --reverse-frag");
    }

    #[test]
    fn empty_selection_falls_back_to_discord() {
        assert_eq!(target_urls(&[], &[]), vec![TARGETS[0].url.to_string()]);
    }
}
