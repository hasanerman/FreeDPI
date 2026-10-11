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
    pub urls: &'static [&'static str],
    pub domains: &'static [&'static str],
}

pub const TARGETS: &[Target] = &[
    Target {
        id: "discord",
        name: "Discord",
        urls: &["https://discord.com/api/v9/gateway", "https://gateway.discord.gg/", "https://cdn.discordapp.com/"],
        domains: &[
            "discord.com",
            "discord.gg",
            "discord.media",
            "discord.dev",
            "discord.new",
            "discord.co",
            "discordapp.com",
            "discordapp.net",
            "discordcdn.com",
            "discordstatus.com",
            "dis.gd",
        ],
    },
    Target {
        id: "youtube",
        name: "YouTube",
        urls: &["https://www.youtube.com/generate_204"],
        domains: &["youtube.com", "youtu.be", "googlevideo.com", "ytimg.com", "ggpht.com", "youtube-nocookie.com", "youtubei.googleapis.com"],
    },
    Target {
        id: "roblox",
        name: "Roblox",
        urls: &["https://www.roblox.com/"],
        domains: &["roblox.com", "rbxcdn.com", "rbx.com", "robloxlabs.com"],
    },
    Target {
        id: "wattpad",
        name: "Wattpad",
        urls: &["https://www.wattpad.com/"],
        domains: &["wattpad.com", "wattpad.net"],
    },
];

pub fn target_domains(ids: &[String], custom: &[String]) -> Vec<String> {
    let mut domains: Vec<String> = TARGETS
        .iter()
        .filter(|t| ids.iter().any(|id| id == t.id))
        .flat_map(|t| t.domains.iter().map(|d| d.to_string()))
        .collect();
    domains.extend(custom.iter().filter_map(|d| normalize_domain(d)));
    if domains.is_empty() {
        domains.extend(TARGETS[0].domains.iter().map(|d| d.to_string()));
    }
    domains
}

pub fn covers(domains: &[String], host: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    domains
        .iter()
        .any(|d| host == *d || host.strip_suffix(d.as_str()).is_some_and(|rest| rest.ends_with('.')))
}

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
        "split=1 disorder=1",
        "tlsrec=sni+1 split=sni+3 disorder=1",
        "split=sni+1 disorder=1",
        "tlsrec=sni+1 split=1",
        "tlsrec=sni+1 split=sni+3",
        "split=1",
        "split=sni+1",
        "tlsrec=sni+1",
        "split=2",
        "split=1 delay=50",
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
        .flat_map(|t| t.urls.iter().map(|u| u.to_string()))
        .collect();
    urls.extend(custom.iter().filter_map(|d| normalize_domain(d)).map(|d| format!("https://{d}/")));
    if urls.is_empty() {
        urls.extend(TARGETS[0].urls.iter().map(|u| u.to_string()));
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
    fn covers_domains_and_subdomains() {
        let domains = target_domains(&["discord".into()], &["example.com".into()]);
        assert!(covers(&domains, "discord.com"));
        assert!(covers(&domains, "gateway.discord.gg"));
        assert!(covers(&domains, "CDN.discordapp.com."));
        assert!(covers(&domains, "a.example.com"));
        assert!(!covers(&domains, "notdiscord.com"));
        assert!(!covers(&domains, "google.com"));
    }

    #[test]
    fn empty_selection_falls_back_to_discord() {
        let expected: Vec<String> = TARGETS[0].urls.iter().map(|u| u.to_string()).collect();
        assert_eq!(target_urls(&[], &[]), expected);
    }
}
