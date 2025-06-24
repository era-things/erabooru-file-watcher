// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::mpsc::{Sender};
use std::sync::Mutex;
use std::thread::JoinHandle;

use once_cell::sync::Lazy;
mod commands;
mod utils;

use tauri::{
  menu::{Menu, MenuItem},
  tray::{TrayIconBuilder},
  Manager, WindowEvent
};

struct WatcherHandle {
    stop: Sender<()>,
    handle: JoinHandle<()>,
}

struct AppState {
    watcher: Mutex<Option<WatcherHandle>>,
}

static STATE: Lazy<AppState> = Lazy::new(|| AppState {
    watcher: Mutex::new(None),
});

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::settings::save_settings,
            commands::settings::load_settings,
            commands::watching::start_watching,
            commands::watching::stop_watching,
        ])
        .setup(|app| {
            /* -------- tray menu -------- */
            let show = MenuItem::with_id(app, "show", "Show window", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            /* -------- tray icon -------- */
            TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                "show" => {
                    if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                    }
                }
                "quit" => app.exit(0),
                _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}