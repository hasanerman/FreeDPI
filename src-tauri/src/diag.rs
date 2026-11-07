use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use serde::Serialize;
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::engine::proxy::Resolver;
use crate::network;

const STEP_TIMEOUT: Duration = Duration::from_secs(6);
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);
const BLOCK_PAGES: &[Ipv4Addr] = &[Ipv4Addr::new(195, 175, 254, 2)];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Ok,
    Info,
    Warn,
    Fail,
}

#[derive(Clone, Debug, Serialize)]
pub struct Item {
    level: Level,
    text: String,
    hint: Option<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Section {
    id: &'static str,
    items: Vec<Item>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    sections: Vec<Section>,
    text: String,
}

pub struct Context {
    pub urls: Vec<String>,
    pub proxy_port: Option<u16>,
    pub connected: bool,
    pub summary: Vec<String>,
    pub history: Vec<String>,
}

fn item(level: Level, text: impl Into<String>, hint: Option<&'static str>) -> Item {
    Item { level, text: text.into(), hint }
}

pub fn mask(ip: IpAddr) -> String {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            format!("{}.x.x.{}", o[0], o[3])
        }
        IpAddr::V6(v6) => {
            let s = v6.segments();
            format!("{:x}:…:{:x}", s[0], s[7])
        }
    }
}

fn ms(start: Instant) -> u128 {
    start.elapsed().as_millis()
}

#[allow(dead_code)]
fn command(program: &str, args: &[&str]) -> Option<String> {
    crate::system::run(program, args)
        .ok()
        .map(|out| crate::system::output_text(&out))
}

pub fn classify(error: &reqwest::Error) -> (&'static str, String) {
    let mut chain = error.to_string();
    let mut source = std::error::Error::source(error);
    while let Some(inner) = source {
        chain.push_str(": ");
        chain.push_str(&inner.to_string());
        source = inner.source();
    }
    let lower = chain.to_ascii_lowercase();
    let code = if error.is_timeout() || lower.contains("timed out") {
        "site_timeout"
    } else if lower.contains("certificate") || lower.contains("unknownissuer") || lower.contains("unknown issuer") {
        "site_cert"
    } else if lower.contains("reset") || lower.contains("10054") || lower.contains("forcibly closed") {
        "site_reset"
    } else if lower.contains("refused") || lower.contains("10061") {
        "site_refused"
    } else if lower.contains("dns") || lower.contains("resolve") || lower.contains("no such host") {
        "site_dns"
    } else if lower.contains("eof") || lower.contains("handshake") || lower.contains("closed") {
        "site_closed"
    } else {
        "site_other"
    };
    (code, chain)
}

fn bogon(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_unspecified()
                || v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || BLOCK_PAGES.contains(v4)
        }
        IpAddr::V6(v6) => v6.is_unspecified() || v6.is_loopback(),
    }
}

fn hosts(urls: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for url in urls {
        if let Some(host) = reqwest::Url::parse(url).ok().and_then(|u| u.host_str().map(String::from)) {
            if !out.contains(&host) {
                out.push(host);
            }
        }
    }
    out
}

#[cfg(windows)]
fn vpn_hint() -> Option<String> {
    let text = command("ipconfig", &[])?.to_ascii_lowercase();
    const NAMES: &[&str] = &["wireguard", "openvpn", "tap-windows", "wintun", "nordlynx", "protonvpn", "vpn"];
    NAMES.iter().find(|n| text.contains(*n)).map(|n| n.to_string())
}

#[cfg(target_os = "macos")]
fn vpn_hint() -> Option<String> {
    let text = command("/usr/sbin/scutil", &["--nc", "list"])?;
    text.lines()
        .find(|l| l.contains("(Connected)"))
        .map(|l| l.trim().chars().take(60).collect())
}

#[cfg(target_os = "linux")]
fn vpn_hint() -> Option<String> {
    let text = command("ip", &["-o", "link", "show", "up"])?;
    text.lines()
        .filter_map(|l| l.split(':').nth(1).map(|n| n.trim().to_string()))
        .find(|n| ["tun", "wg", "tap", "ppp", "nordlynx", "proton"].iter().any(|p| n.starts_with(p)))
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
fn vpn_hint() -> Option<String> {
    None
}

#[cfg(windows)]
fn other_proxy(_own_port: Option<u16>) -> Option<String> {
    let key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings";
    let enabled = command("reg", &["query", key, "/v", "ProxyEnable"])?;
    if !enabled.contains("0x1") {
        return None;
    }
    let server = command("reg", &["query", key, "/v", "ProxyServer"]).unwrap_or_default();
    Some(server.split_whitespace().last().unwrap_or("?").to_string())
}

#[cfg(target_os = "macos")]
fn other_proxy(own_port: Option<u16>) -> Option<String> {
    let text = command("/usr/sbin/scutil", &["--proxy"])?;
    if !text.contains("HTTPSEnable : 1") {
        return None;
    }
    let port = text
        .lines()
        .find_map(|l| l.trim().strip_prefix("HTTPSPort : "))
        .and_then(|p| p.trim().parse::<u16>().ok());
    let host = text
        .lines()
        .find_map(|l| l.trim().strip_prefix("HTTPSProxy : "))
        .unwrap_or("?")
        .trim()
        .to_string();
    let ours = host == "127.0.0.1" && port.is_some() && port == own_port;
    (!ours).then(|| format!("{host}:{}", port.map_or("?".into(), |p| p.to_string())))
}

#[cfg(target_os = "linux")]
fn other_proxy(own_port: Option<u16>) -> Option<String> {
    let mode = command("gsettings", &["get", "org.gnome.system.proxy", "mode"])?;
    if !mode.contains("manual") {
        return None;
    }
    let host = command("gsettings", &["get", "org.gnome.system.proxy.https", "host"]).unwrap_or_default();
    let port = command("gsettings", &["get", "org.gnome.system.proxy.https", "port"]).unwrap_or_default();
    let ours = host.contains("127.0.0.1") && own_port.is_some_and(|p| port.trim() == p.to_string());
    (!ours).then(|| format!("{}:{}", host.trim_matches('\''), port.trim()))
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
fn other_proxy(_own_port: Option<u16>) -> Option<String> {
    None
}

fn ipv6_address() -> Option<IpAddr> {
    let socket = UdpSocket::bind("[::]:0").ok()?;
    socket.connect("[2606:4700:4700::1111]:53").ok()?;
    let ip = socket.local_addr().ok()?.ip();
    (!ip.is_unspecified()).then_some(ip)
}

fn network_section(ctx: &Context) -> Section {
    let mut items = Vec::new();
    match network::local_address() {
        Some(ip) => items.push(item(Level::Ok, format!("Online, local address {}", mask(ip)), None)),
        None => items.push(item(Level::Fail, "No internet connection", Some("no_internet"))),
    }
    match ipv6_address() {
        Some(ip) => items.push(item(Level::Info, format!("IPv6 is available ({})", mask(ip)), Some("ipv6_on"))),
        None => items.push(item(Level::Info, "IPv6 is not available", None)),
    }
    match vpn_hint() {
        Some(name) => items.push(item(Level::Warn, format!("A VPN looks active ({name})"), Some("vpn"))),
        None => items.push(item(Level::Ok, "No VPN detected", None)),
    }
    match other_proxy(ctx.proxy_port) {
        Some(proxy) => items.push(item(Level::Warn, format!("Another system proxy is set: {proxy}"), Some("proxy_other"))),
        None => items.push(item(Level::Ok, "No other system proxy", None)),
    }
    if !ctx.connected {
        items.push(item(Level::Info, "FreeDPI is not connected; site checks run directly", Some("not_connected")));
    }
    Section { id: "network", items }
}

#[cfg_attr(not(windows), allow(dead_code))]
fn dns_query(name: &str) -> Vec<u8> {
    let mut packet = vec![0x46, 0x44, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    for label in name.split('.') {
        packet.push(label.len() as u8);
        packet.extend_from_slice(label.as_bytes());
    }
    packet.extend_from_slice(&[0x00, 0x00, 0x01, 0x00, 0x01]);
    packet
}

#[cfg_attr(not(windows), allow(dead_code))]
async fn redirect_resolver_ok() -> Option<u128> {
    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await.ok()?;
    socket.connect("77.88.8.8:1253").await.ok()?;
    let start = Instant::now();
    socket.send(&dns_query("discord.com")).await.ok()?;
    let mut buffer = [0u8; 512];
    let len = timeout(Duration::from_secs(3), socket.recv(&mut buffer)).await.ok()?.ok()?;
    (len > 12 && buffer[0] == 0x46 && buffer[1] == 0x44).then(|| ms(start))
}

async fn system_lookup(host: &str) -> Result<(Vec<IpAddr>, u128), String> {
    let start = Instant::now();
    match timeout(STEP_TIMEOUT, tokio::net::lookup_host((host, 443))).await {
        Ok(Ok(addrs)) => {
            let ips: Vec<IpAddr> = addrs.map(|a| a.ip()).collect();
            if ips.is_empty() {
                Err("no address".into())
            } else {
                Ok((ips, ms(start)))
            }
        }
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err("timed out".into()),
    }
}

async fn dns_section(hosts: &[String]) -> Section {
    let mut items = Vec::new();
    let resolver = Resolver::new();
    for host in hosts {
        let system = system_lookup(host).await;
        let secure = timeout(STEP_TIMEOUT, resolver.lookup(host)).await.ok().and_then(Result::ok);
        match system {
            Err(e) => items.push(item(Level::Fail, format!("{host}: system DNS failed ({e})"), Some("dns_failed"))),
            Ok((ips, time)) => {
                let first = ips[0];
                if ips.iter().any(bogon) {
                    items.push(item(
                        Level::Fail,
                        format!("{host}: system DNS returned {first}, a blocked or private address"),
                        Some("dns_bogon"),
                    ));
                    continue;
                }
                let system_set: HashSet<IpAddr> = ips.iter().copied().collect();
                match secure {
                    Some(secure) if !secure.is_empty() && secure.iter().all(|ip| !system_set.contains(ip)) => {
                        items.push(item(
                            Level::Warn,
                            format!("{host}: system DNS {first} ({time} ms), secure DNS {}", secure[0]),
                            Some("dns_mismatch"),
                        ));
                    }
                    Some(_) => items.push(item(
                        Level::Ok,
                        format!("{host}: {first} ({time} ms), matches secure DNS"),
                        None,
                    )),
                    None => items.push(item(
                        Level::Info,
                        format!("{host}: {first} ({time} ms), secure DNS unreachable"),
                        None,
                    )),
                }
            }
        }
    }
    #[cfg(windows)]
    match redirect_resolver_ok().await {
        Some(time) => items.push(item(Level::Ok, format!("FreeDPI DNS server 77.88.8.8:1253 answers ({time} ms)"), None)),
        None => items.push(item(Level::Warn, "FreeDPI DNS server 77.88.8.8:1253 does not answer", Some("dns_redirect_fail"))),
    }
    Section { id: "dns", items }
}

struct Attempt {
    ok: bool,
    text: String,
    hint: Option<&'static str>,
}

async fn https(url: &str, proxy: Option<u16>) -> Attempt {
    let builder = reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .connect_timeout(STEP_TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .pool_max_idle_per_host(0)
        .user_agent(concat!("FreeDPI/", env!("CARGO_PKG_VERSION")));
    let builder = match proxy.and_then(|port| reqwest::Proxy::all(format!("http://127.0.0.1:{port}")).ok()) {
        Some(p) => builder.proxy(p),
        None => builder.no_proxy(),
    };
    let Ok(client) = builder.build() else {
        return Attempt { ok: false, text: "client error".into(), hint: Some("site_other") };
    };
    let start = Instant::now();
    match client.get(url).send().await {
        Ok(response) => {
            let status = response.status();
            let time = ms(start);
            if status.is_server_error() {
                Attempt { ok: false, text: format!("HTTPS {status} ({time} ms)"), hint: Some("site_error") }
            } else {
                Attempt { ok: true, text: format!("HTTPS {} ({time} ms)", status.as_u16()), hint: None }
            }
        }
        Err(error) => {
            let (code, detail) = classify(&error);
            let short: String = detail.chars().take(140).collect();
            Attempt { ok: false, text: format!("HTTPS failed after {} ms: {short}", ms(start)), hint: Some(code) }
        }
    }
}

async fn site(url: String, proxy: Option<u16>, connected: bool) -> Vec<Item> {
    let mut items = Vec::new();
    let Some(host) = reqwest::Url::parse(&url).ok().and_then(|u| u.host_str().map(String::from)) else {
        return items;
    };
    let mut steps = Vec::new();
    let mut first_failure = None;
    let address = match system_lookup(&host).await {
        Ok((ips, time)) => {
            steps.push(format!("DNS {time} ms"));
            Some(ips[0])
        }
        Err(e) => {
            steps.push(format!("DNS failed ({e})"));
            first_failure = Some("site_dns");
            None
        }
    };
    if let Some(ip) = address {
        let start = Instant::now();
        match timeout(STEP_TIMEOUT, TcpStream::connect(SocketAddr::new(ip, 443))).await {
            Ok(Ok(_)) => steps.push(format!("TCP {} ms", ms(start))),
            Ok(Err(e)) => {
                let lower = e.to_string().to_ascii_lowercase();
                first_failure = Some(if lower.contains("refused") { "site_refused" } else { "site_reset" });
                steps.push(format!("TCP failed ({e})"));
            }
            Err(_) => {
                first_failure = Some("site_timeout");
                steps.push("TCP timed out".into());
            }
        }
    }
    let direct = https(&url, None).await;
    steps.push(direct.text.clone());
    let through_driver = cfg!(windows) && connected;
    let path = if through_driver { "through FreeDPI" } else { "direct" };
    let hint = if direct.ok { None } else { direct.hint.or(first_failure) };
    items.push(item(
        if direct.ok { Level::Ok } else { Level::Fail },
        format!("{host} ({path}): {}", steps.join(" · ")),
        hint,
    ));
    if let Some(port) = proxy {
        let via = https(&url, Some(port)).await;
        items.push(item(
            if via.ok { Level::Ok } else { Level::Fail },
            format!("{host} (through FreeDPI): {}", via.text),
            if via.ok { None } else { via.hint },
        ));
        match (direct.ok, via.ok) {
            (false, true) => items.push(item(Level::Info, format!("{host}: blocked directly, opens through FreeDPI"), Some("dpi_confirmed"))),
            (false, false) => items.push(item(Level::Warn, format!("{host}: does not open through FreeDPI either"), Some("both_fail"))),
            _ => {}
        }
    } else if through_driver && !direct.ok {
        items.push(item(Level::Warn, format!("{host}: does not open through FreeDPI"), Some("both_fail")));
    }
    items
}

async fn sites_section(ctx: &Context) -> Section {
    let handles: Vec<_> = ctx
        .urls
        .iter()
        .cloned()
        .map(|url| tokio::spawn(site(url, ctx.proxy_port, ctx.connected)))
        .collect();
    let mut items = Vec::new();
    for handle in handles {
        if let Ok(mut result) = handle.await {
            items.append(&mut result);
        }
    }
    Section { id: "sites", items }
}

#[cfg(windows)]
async fn engine_section(ctx: &Context) -> Section {
    let mut items = Vec::new();
    let tasks = command("tasklist", &["/NH", "/FO", "CSV"]).unwrap_or_default().to_ascii_lowercase();
    let running = tasks.contains("\"goodbyedpi.exe\"");
    match (ctx.connected, running) {
        (true, true) => items.push(item(Level::Ok, "GoodbyeDPI engine is running", None)),
        (true, false) => items.push(item(Level::Fail, "GoodbyeDPI engine is not running", Some("engine_down"))),
        (false, true) => items.push(item(Level::Warn, "A GoodbyeDPI process runs while FreeDPI is off", Some("other_dpi"))),
        (false, false) => items.push(item(Level::Info, "Engine is stopped", None)),
    }
    let driver = command("sc", &["query", "WinDivert"]).unwrap_or_default();
    if driver.contains("RUNNING") {
        items.push(item(Level::Ok, "WinDivert driver is loaded", None));
    } else if ctx.connected {
        items.push(item(Level::Fail, "WinDivert driver is not loaded", Some("windivert_missing")));
    } else {
        items.push(item(Level::Info, "WinDivert driver loads when you connect", None));
    }
    if ["\"avp.exe\"", "\"avpui.exe\"", "\"ksde.exe\""].iter().any(|p| tasks.contains(p)) {
        items.push(item(Level::Fail, "Kaspersky is running", Some("kaspersky")));
    }
    if tasks.contains("\"winws.exe\"") {
        items.push(item(Level::Warn, "zapret (winws.exe) is running", Some("other_dpi")));
    }
    Section { id: "engine", items }
}

#[cfg(not(windows))]
async fn engine_section(ctx: &Context) -> Section {
    let mut items = Vec::new();
    match ctx.proxy_port {
        Some(port) => {
            let up = timeout(Duration::from_secs(2), TcpStream::connect(("127.0.0.1", port)))
                .await
                .is_ok_and(|r| r.is_ok());
            if up {
                items.push(item(Level::Ok, format!("Local proxy listens on 127.0.0.1:{port}"), None));
            } else {
                items.push(item(Level::Fail, format!("Local proxy on 127.0.0.1:{port} does not answer"), Some("engine_down")));
            }
            #[cfg(target_os = "macos")]
            {
                let text = command("/usr/sbin/scutil", &["--proxy"]).unwrap_or_default();
                let set = text.contains("HTTPSEnable : 1") && text.contains(&format!("HTTPSPort : {port}"));
                if set {
                    items.push(item(Level::Ok, "System proxy points to FreeDPI", None));
                } else {
                    items.push(item(Level::Fail, "System proxy does not point to FreeDPI", Some("sysproxy_missing")));
                }
            }
            #[cfg(target_os = "linux")]
            {
                let mode = command("gsettings", &["get", "org.gnome.system.proxy", "mode"]).unwrap_or_default();
                if mode.contains("manual") {
                    items.push(item(Level::Ok, "GNOME system proxy is set", None));
                } else {
                    items.push(item(
                        Level::Info,
                        format!("System proxy is not set on this desktop; use 127.0.0.1:{port} in apps"),
                        Some("sysproxy_manual"),
                    ));
                }
            }
        }
        None => items.push(item(Level::Info, "Engine is stopped", None)),
    }
    Section { id: "engine", items }
}

fn history_section(ctx: &Context) -> Section {
    let mut items: Vec<Item> = ctx.summary.iter().map(|line| item(Level::Info, line.clone(), None)).collect();
    for line in &ctx.history {
        let level = if line.contains(" [E] ") {
            Level::Fail
        } else if line.contains(" [W] ") {
            Level::Warn
        } else {
            Level::Info
        };
        items.push(item(level, line.clone(), None));
    }
    Section { id: "history", items }
}

pub fn hint_text(code: &str) -> &'static str {
    match code {
        "no_internet" => "There is no internet connection.",
        "ipv6_on" => "IPv6 is on. Some filters treat IPv6 differently.",
        "vpn" => "A VPN is active. It can conflict with FreeDPI.",
        "proxy_other" => "Another proxy is set in the system. It can conflict with FreeDPI.",
        "not_connected" => "FreeDPI is off, so site checks show your plain connection.",
        "dns_failed" => "DNS could not find this address.",
        "dns_bogon" => "Your provider's DNS returned a fake or empty address: DNS blocking.",
        "dns_mismatch" => "Your provider's DNS and secure DNS disagree. Normal for some CDNs, but it can also be DNS blocking.",
        "dns_redirect_fail" => "The DNS server FreeDPI uses on Windows cannot be reached.",
        "site_reset" => "The connection was reset: most likely DPI blocking.",
        "site_timeout" => "Timed out: packets are dropped or the site is slow.",
        "site_cert" => "Certificate error: something sits in between (antivirus or provider).",
        "site_refused" => "The server refused the connection.",
        "site_closed" => "The secure connection was cut off: possibly DPI blocking.",
        "site_dns" => "The address could not be found.",
        "site_error" => "The site returned a server error; the problem may be on the site.",
        "site_other" => "The site could not be opened.",
        "dpi_confirmed" => "Blocked directly but opens through FreeDPI: the block is DPI and FreeDPI works.",
        "both_fail" => "Does not open through FreeDPI either. Try another region or reconnect.",
        "engine_down" => "The engine is not running. Disconnect and connect again.",
        "windivert_missing" => "The WinDivert driver did not load. An antivirus may block it.",
        "kaspersky" => "Kaspersky blocks the WinDivert driver. Remove it or add FreeDPI to its exclusions.",
        "other_dpi" => "Another DPI tool is running. Close it, it conflicts with FreeDPI.",
        "sysproxy_missing" => "The system proxy is not set to FreeDPI. Disconnect and connect again.",
        "sysproxy_manual" => "This desktop has no system proxy. Set the proxy address in your apps.",
        _ => "",
    }
}

fn render(sections: &[Section]) -> String {
    let mut out = vec![
        format!("FreeDPI {} diagnostics", env!("CARGO_PKG_VERSION")),
        format!("{} {} · {}", std::env::consts::OS, std::env::consts::ARCH, crate::log::timestamp()),
    ];
    for section in sections {
        out.push(String::new());
        out.push(format!("## {}", section.id));
        for entry in &section.items {
            let tag = match entry.level {
                Level::Ok => "OK  ",
                Level::Info => "INFO",
                Level::Warn => "WARN",
                Level::Fail => "FAIL",
            };
            out.push(format!("[{tag}] {}", entry.text));
            if let Some(code) = entry.hint {
                out.push(format!("       {}", hint_text(code)));
            }
        }
    }
    out.join("\n")
}

pub async fn run(ctx: Context) -> Report {
    let names = hosts(&ctx.urls);
    let network = network_section(&ctx);
    let (dns, sites, engine) = tokio::join!(dns_section(&names), sites_section(&ctx), engine_section(&ctx));
    let sections = vec![network, dns, sites, engine, history_section(&ctx)];
    let text = render(&sections);
    Report { sections, text }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_addresses() {
        assert_eq!(mask("192.168.1.42".parse().unwrap()), "192.x.x.42");
    }

    #[test]
    fn detects_bogon_answers() {
        assert!(bogon(&"0.0.0.0".parse().unwrap()));
        assert!(bogon(&"195.175.254.2".parse().unwrap()));
        assert!(!bogon(&"162.159.135.232".parse().unwrap()));
    }

    #[test]
    fn builds_dns_query() {
        let packet = dns_query("discord.com");
        assert_eq!(&packet[12..20], b"\x07discord");
        assert_eq!(packet.len(), 12 + 13 + 4);
    }

    #[test]
    fn every_hint_has_text() {
        for code in [
            "no_internet", "ipv6_on", "vpn", "proxy_other", "not_connected", "dns_failed", "dns_bogon",
            "dns_mismatch", "dns_redirect_fail", "site_reset", "site_timeout", "site_cert", "site_refused",
            "site_closed", "site_dns", "site_error", "site_other", "dpi_confirmed", "both_fail", "engine_down",
            "windivert_missing", "kaspersky", "other_dpi", "sysproxy_missing", "sysproxy_manual",
        ] {
            assert!(!hint_text(code).is_empty(), "{code}");
        }
    }
}
