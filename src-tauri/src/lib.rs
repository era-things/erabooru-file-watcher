// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::thread::JoinHandle;

use once_cell::sync::Lazy;
mod commands;
mod utils;

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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}