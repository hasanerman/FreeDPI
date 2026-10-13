use std::process::{Command, Stdio};

use super::EngineError;

const HOST: &str = "127.0.0.1";

fn run(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn gsettings(args: &[&str]) {
    run("gsettings", args);
}

fn kde(group_key: &[(&str, &str)]) {
    for tool in ["kwriteconfig6", "kwriteconfig5"] {
        let mut ok = true;
        for (key, value) in group_key {
            ok &= run(tool, &["--file", "kioslaverc", "--group", "Proxy Settings", "--key", key, value]);
        }
        if ok {
            return;
        }
    }
}

pub fn enable(port: u16) -> Result<(), EngineError> {
    let port = port.to_string();
    for scheme in ["http", "https"] {
        let schema = format!("org.gnome.system.proxy.{scheme}");
        gsettings(&["set", &schema, "host", HOST]);
        gsettings(&["set", &schema, "port", &port]);
    }
    gsettings(&["set", "org.gnome.system.proxy", "mode", "manual"]);
    let url = format!("http://{HOST} {port}");
    kde(&[("ProxyType", "1"), ("httpProxy", &url), ("httpsProxy", &url)]);
    Ok(())
}

pub fn disable() -> Result<(), EngineError> {
    gsettings(&["set", "org.gnome.system.proxy", "mode", "none"]);
    kde(&[("ProxyType", "0")]);
    Ok(())
}
