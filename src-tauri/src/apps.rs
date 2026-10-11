#[cfg(target_os = "macos")]
pub fn relaunch_discord(port: u16) -> Result<(), String> {
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use std::time::Duration;

    let proxy = format!("http://127.0.0.1:{port}");
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let exe = [
        PathBuf::from("/Applications/Discord.app/Contents/MacOS/Discord"),
        home.join("Applications/Discord.app/Contents/MacOS/Discord"),
    ]
    .into_iter()
    .find(|path| path.is_file())
    .ok_or("discord_missing")?;

    let _ = Command::new("/usr/bin/osascript").args(["-e", "quit app \"Discord\""]).status();
    std::thread::sleep(Duration::from_secs(2));

    Command::new(exe)
        .arg(format!("--proxy-server={proxy}"))
        .env("HTTPS_PROXY", &proxy)
        .env("HTTP_PROXY", &proxy)
        .env("ALL_PROXY", &proxy)
        .env("https_proxy", &proxy)
        .env("http_proxy", &proxy)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

pub const REPOSITORY: &str = "https://github.com/hasanerman/FreeDPI";

pub fn open_repository() -> Result<(), String> {
    open_url(REPOSITORY)
}

pub fn open_url(url: &str) -> Result<(), String> {
    #[cfg(windows)]
    let program = "explorer.exe";
    #[cfg(target_os = "macos")]
    let program = "/usr/bin/open";
    #[cfg(not(any(windows, target_os = "macos")))]
    let program = "xdg-open";
    std::process::Command::new(program)
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn relaunch_discord(_port: u16) -> Result<(), String> {
    Err("unsupported".into())
}
