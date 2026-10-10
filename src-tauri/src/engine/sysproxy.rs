use std::process::Command;

use super::EngineError;

const NETWORKSETUP: &str = "/usr/sbin/networksetup";

const PROXY_VARS: &[&str] = &["HTTPS_PROXY", "HTTP_PROXY", "ALL_PROXY", "https_proxy", "http_proxy"];

fn set_session_proxy(value: Option<&str>) {
    for name in PROXY_VARS {
        let mut command = Command::new("/bin/launchctl");
        match value {
            Some(url) => command.args(["setenv", name, url]),
            None => command.args(["unsetenv", name]),
        };
        let _ = command.status();
    }
}

pub fn enable(port: u16) -> Result<(), EngineError> {
    set_session_proxy(Some(&format!("http://127.0.0.1:{port}")));
    let port = port.to_string();
    let commands: Vec<Vec<String>> = services()
        .into_iter()
        .flat_map(|service| {
            [
                vec!["-setwebproxy".into(), service.clone(), "127.0.0.1".into(), port.clone()],
                vec!["-setsecurewebproxy".into(), service, "127.0.0.1".into(), port.clone()],
            ]
        })
        .collect();
    apply(&commands)
}

pub fn disable() -> Result<(), EngineError> {
    set_session_proxy(None);
    let commands: Vec<Vec<String>> = services()
        .into_iter()
        .flat_map(|service| {
            [
                vec!["-setwebproxystate".into(), service.clone(), "off".into()],
                vec!["-setsecurewebproxystate".into(), service, "off".into()],
            ]
        })
        .collect();
    apply(&commands)
}

fn services() -> Vec<String> {
    let Ok(output) = Command::new(NETWORKSETUP).arg("-listallnetworkservices").output() else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .skip(1)
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('*'))
        .map(String::from)
        .collect()
}

fn apply(commands: &[Vec<String>]) -> Result<(), EngineError> {
    if commands.is_empty() {
        return Err(EngineError::SystemProxy);
    }
    let direct = commands.iter().all(|args| {
        Command::new(NETWORKSETUP).args(args).output().is_ok_and(|out| {
            let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
            out.status.success() && !text.contains("Error")
        })
    });
    if direct {
        return Ok(());
    }

    let script = commands
        .iter()
        .map(|args| {
            std::iter::once(NETWORKSETUP.to_string())
                .chain(args.iter().map(|a| shell_quote(a)))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("; ");
    let apple = format!(
        "do shell script \"{}\" with administrator privileges",
        script.replace('\\', "\\\\").replace('"', "\\\"")
    );
    match Command::new("/usr/bin/osascript").arg("-e").arg(apple).status() {
        Ok(status) if status.success() => Ok(()),
        _ => Err(EngineError::SystemProxy),
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
