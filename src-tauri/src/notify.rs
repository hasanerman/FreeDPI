use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

pub fn show(app: &AppHandle, body: &str) {
    let _ = app.notification().builder().title("FreeDPI").body(body).show();
}
