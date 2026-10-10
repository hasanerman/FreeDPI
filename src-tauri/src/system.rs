use std::io;
use std::process::{Command, Output, Stdio};

pub fn run(program: &str, args: &[&str]) -> io::Result<Output> {
    let mut command = Command::new(program);
    command.args(args).stdin(Stdio::null());
    hide_window(&mut command);
    command.output()
}

#[cfg(windows)]
pub fn hide_window(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
pub fn hide_window(_command: &mut Command) {}

pub fn output_text(output: &Output) -> String {
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    text.trim().to_string()
}
