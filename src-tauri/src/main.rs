#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod apps;
mod autostart;
mod catalog;
mod commands;
mod controller;
mod engine;
mod i18n;
mod probe;
mod settings;
mod system;
mod tray;
mod updater;

use std::sync::Arc;

use tauri::{Manager, RunEvent, WindowEvent};

use controller::Controller;

fn main() {
    let start_in_tray = std::env::args().any(|arg| arg == autostart::TRAY_ARG);

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| tray::show_window(app)))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(updater::Pending::default())
        .setup(move |app| {
            updater::watch(app.handle().clone());
            let controller = Arc::new(Controller::new(app.handle())?);
            tray::create(app.handle(), &controller.settings())?;
            app.manage(controller.clone());
            let watcher = controller.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(45)).await;
                    watcher.health_check().await;
                }
            });
            if start_in_tray {
                tauri::async_runtime::spawn(async move { controller.connect().await });
            } else {
                tray::show_window(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::init,
            commands::connect,
            commands::disconnect,
            commands::save_preferences,
            commands::set_language,
            commands::set_autostart,
            commands::quit,
            commands::install_update,
            commands::open_discord,
            commands::open_repository,
        ])
        .build(tauri::generate_context!())
        .expect("failed to start FreeDPI");

    app.run(|app, event| match event {
        RunEvent::ExitRequested { api, code, .. } if code.is_none() => api.prevent_exit(),
        RunEvent::Exit => {
            if let Some(controller) = app.try_state::<Arc<Controller>>() {
                tauri::async_runtime::block_on(controller.disconnect());
            }
        }
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => tray::show_window(app),
        _ => {}
    });
}
