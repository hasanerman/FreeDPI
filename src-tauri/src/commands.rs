use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, State};

use crate::catalog;
use crate::controller::{Controller, Status};
use crate::settings::{Preferences, Settings};

type Ctl<'a> = State<'a, Arc<Controller>>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetInfo {
    id: &'static str,
    name: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    status: Status,
    settings: Settings,
    targets: Vec<TargetInfo>,
    platform: &'static str,
    version: &'static str,
}

#[tauri::command]
pub fn init(language: String, controller: Ctl<'_>) -> Snapshot {
    controller.detect_language(&language);
    Snapshot {
        status: controller.status(),
        settings: controller.settings(),
        targets: catalog::TARGETS.iter().map(|t| TargetInfo { id: t.id, name: t.name }).collect(),
        platform: std::env::consts::OS,
        version: env!("CARGO_PKG_VERSION"),
    }
}

#[tauri::command]
pub fn connect(controller: Ctl<'_>) {
    let controller = controller.inner().clone();
    tauri::async_runtime::spawn(async move { controller.connect().await });
}

#[tauri::command]
pub fn disconnect(controller: Ctl<'_>) {
    let controller = controller.inner().clone();
    tauri::async_runtime::spawn(async move { controller.disconnect().await });
}

#[tauri::command]
pub fn save_preferences(preferences: Preferences, controller: Ctl<'_>) -> Settings {
    controller.save_preferences(preferences)
}

#[tauri::command]
pub fn set_language(language: String, controller: Ctl<'_>) -> Settings {
    controller.set_language(&language)
}

#[tauri::command]
pub async fn set_autostart(enabled: bool, controller: Ctl<'_>) -> Result<Settings, String> {
    controller.set_autostart(enabled).await
}

#[tauri::command]
pub async fn quit(app: AppHandle, controller: Ctl<'_>) -> Result<(), String> {
    controller.disconnect().await;
    app.exit(0);
    Ok(())
}
