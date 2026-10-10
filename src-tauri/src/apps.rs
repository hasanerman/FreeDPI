#[cfg(target_os = "macos")]
pub fn relaunch_discord(port: u16) -> Result<(), String> {
    use std::process::Command;
    use std::time::Duration;

    let _ = Command::new("/usr/bin/osascript").args(["-e", "quit app \"Discord\""]).status();
    std::thread::sleep(Duration::from_secs(2));
    let status = Command::new("/usr/bin/open")
        .args(["-a", "Discord", "--args", &format!("--proxy-server=http://127.0.0.1:{port}")])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() { Ok(()) } else { Err("discord_missing".into()) }
}

#[cfg(not(target_os = "macos"))]
pub fn relaunch_discord(_port: u16) -> Result<(), String> {
    Err("unsupported".into())
}
