fn main() {
    let level = match std::env::var("PROFILE").as_deref() {
        Ok("release") => "requireAdministrator",
        _ => "asInvoker",
    };
    let manifest = include_str!("windows/app.manifest").replace("{{level}}", level);
    let windows = tauri_build::WindowsAttributes::new().app_manifest(manifest);
    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
        .expect("failed to run tauri build script");
}
