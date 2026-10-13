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

#[cfg(target_os = "linux")]
pub fn relaunch_discord(port: u16) -> Result<(), String> {
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use std::time::Duration;

    let proxy = format!("http://127.0.0.1:{port}");
    let flag = format!("--proxy-server={proxy}");
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let native = [
        PathBuf::from("/usr/bin/discord"),
        PathBuf::from("/usr/share/discord/Discord"),
        PathBuf::from("/opt/discord/Discord"),
        PathBuf::from("/opt/Discord/Discord"),
        home.join(".local/share/discord/Discord"),
    ]
    .into_iter()
    .find(|path| path.is_file());

    let mut command = match native {
        Some(exe) => {
            let mut command = Command::new(exe);
            command.arg(&flag);
            command
        }
        None => {
            let installed = Command::new("flatpak")
                .args(["info", "com.discordapp.Discord"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|s| s.success());
            if !installed {
                return Err("discord_missing".into());
            }
            let mut command = Command::new("flatpak");
            command.args(["run", "com.discordapp.Discord", &flag]);
            command
        }
    };

    let _ = Command::new("pkill").args(["-x", "Discord"]).status();
    std::thread::sleep(Duration::from_secs(2));

    command
        .env("HTTPS_PROXY", &proxy)
        .env("HTTP_PROXY", &proxy)
        .env("https_proxy", &proxy)
        .env("http_proxy", &proxy)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn relaunch_discord(_port: u16) -> Result<(), String> {
    Err("unsupported".into())
}
