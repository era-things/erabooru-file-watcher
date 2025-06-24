// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::path::Path;
use std::sync::{Mutex};
use std::thread::JoinHandle;
use std::sync::mpsc::{channel, Sender};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use notify::{recommended_watcher, RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use mime_guess::MimeGuess;
use reqwest::blocking::Client;
use xxhash_rust::xxh3::xxh3_128;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Default, Serialize, Deserialize, Clone)]
struct Settings {
    folder: String,
    server: String,
}

struct WatcherHandle {
    stop: Sender<()>,
    handle: JoinHandle<()>,
}

struct AppState {
    settings: Mutex<Settings>,
    watcher: Mutex<Option<WatcherHandle>>,
}

static STATE: Lazy<AppState> = Lazy::new(|| AppState {
    settings: Mutex::new(Settings::default()),
    watcher: Mutex::new(None),
});

fn is_media_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let guess = MimeGuess::from_path(path).first();
    if let Some(m) = guess {
        m.type_() == mime::IMAGE || m.type_() == mime::VIDEO
    } else {
        false
    }
}

fn upload_file(client: &Client, server: &str, path: &Path) -> Result<(), String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let hash = xxh3_128(&data);
    let filename = format!("{:032x}", hash);
    let url = format!("{}/api/media/upload-url", server.trim_end_matches('/'));
    let resp = client
        .post(url)
        .json(&serde_json::json!({ "filename": filename }))
        .send()
        .map_err(|e| e.to_string())?;
    let upload_url = resp
        .text()
        .map_err(|e| e.to_string())?;
    let content_type = MimeGuess::from_path(path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();
    client
        .put(upload_url)
        .header(reqwest::header::CONTENT_TYPE, content_type)
        .body(data)
        .send()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn save_settings(folder: String, server: String) {
    let mut settings = STATE.settings.lock().unwrap();
    settings.folder = folder;
    settings.server = server;
}

#[tauri::command]
fn load_settings() -> Settings {
    STATE.settings.lock().unwrap().clone()
}

#[tauri::command]
fn start_watching() -> Result<(), String> {
    let mut watcher_lock = STATE.watcher.lock().unwrap();
    if watcher_lock.is_some() {
        return Ok(());
    }
    let settings = STATE.settings.lock().unwrap().clone();
    if settings.folder.is_empty() || settings.server.is_empty() {
        return Err("folder or server not set".into());
    }
    let (stop_tx, stop_rx) = channel::<()>();
    let folder = settings.folder.clone();
    let server = settings.server.clone();
    let handle = std::thread::spawn(move || {
        let client = Client::new();
        let (tx, rx) = channel();
        let mut watcher = recommended_watcher(move |res| {
            tx.send(res).ok();
        }).expect("watcher");
        watcher
            .watch(Path::new(&folder), RecursiveMode::NonRecursive)
            .expect("watch");
        loop {
            if let Ok(_) = stop_rx.try_recv() {
                break;
            }
            if let Ok(Ok(event)) = rx.recv_timeout(std::time::Duration::from_millis(500)) {
                if matches!(event.kind, EventKind::Create(_)) {
                    for path in event.paths {
                        if is_media_file(&path) {
                            let _ = upload_file(&client, &server, &path);
                        }
                    }
                }
            }
        }
    });
    *watcher_lock = Some(WatcherHandle { stop: stop_tx, handle });
    Ok(())
}

#[tauri::command]
fn stop_watching() {
    if let Some(w) = STATE.watcher.lock().unwrap().take() {
        let _ = w.stop.send(());
        let _ = w.handle.join();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .system_tray(tauri::SystemTray::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            save_settings,
            load_settings,
            start_watching,
            stop_watching
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
