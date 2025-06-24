// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::thread::JoinHandle;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tauri_plugin_store::StoreExt;

use mime_guess::MimeGuess;
use notify::{recommended_watcher, EventKind, RecursiveMode, Watcher};
use reqwest::blocking::Client;
use xxhash_rust::xxh3::xxh3_128;
use tauri::AppHandle;

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
    watcher: Mutex<Option<WatcherHandle>>,
}

static STATE: Lazy<AppState> = Lazy::new(|| AppState {
    watcher: Mutex::new(None),
});

fn is_media_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let guess = MimeGuess::from_path(path).first();
    if let Some(m) = guess {
        m.type_() == mime_guess::mime::IMAGE || m.type_() == mime_guess::mime::VIDEO
    } else {
        false
    }
}

fn upload_file(client: &Client, server: &str, path: &Path) -> Result<(), String> {
    println!("Uploading file: {}", path.display());

    let data = retry_read_file(path, 3, std::time::Duration::from_millis(1000))?;
    let hash = xxh3_128(&data);

    println!("File hash: {:032x}", hash);

    let filename = format!("{:032x}", hash);
    let url = format!("{}/api/media/upload-url", server.trim_end_matches('/'));

    println!("Making request to: {}", url);
    let resp = client
        .post(url)
        .json(&serde_json::json!({ "filename": filename }))
        .send()
        .map_err(|e| e.to_string())?;

    println!("Response status: {}", resp.status());
    
    // Parse the JSON response to extract the actual upload URL
    let response_text = resp.text().map_err(|e| e.to_string())?;
    println!("Upload URL response: {}", response_text);
    
    let upload_response: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse upload URL response: {}", e))?;
    
    let upload_url = upload_response["url"]
        .as_str()
        .ok_or("No 'url' field in upload response")?;

    println!("Extracted upload URL: {}", upload_url);

    let content_type = MimeGuess::from_path(path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();

    println!("Content type: {}", content_type);
    let put_resp = client
        .put(upload_url)
        .header(reqwest::header::CONTENT_TYPE, content_type)
        .body(data)
        .send()
        .map_err(|e| e.to_string())?;

    println!("Upload response status: {}", put_resp.status());
    Ok(())
}

fn retry_read_file(
    path: &Path,
    max_retries: u32,
    delay: std::time::Duration,
) -> Result<Vec<u8>, String> {
    for attempt in 0..max_retries {
        match std::fs::read(path) {
            Ok(data) => return Ok(data),
            Err(e) if e.raw_os_error() == Some(32) => {
                // OS error 32 = file is locked
                println!(
                    "File locked, attempt {}/{}, waiting...",
                    attempt + 1,
                    max_retries
                );
                std::thread::sleep(delay);
                continue;
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    Err("File remained locked after all retry attempts".to_string())
}

fn get_settings_from_store(app: &tauri::AppHandle) -> Result<Settings, String> {
    let store = app.store("store.json").map_err(|e| e.to_string())?;

    let settings_value = store.get("settings");
    if let Some(settings_json) = settings_value {
        if let Ok(settings) = serde_json::from_value::<Settings>(settings_json) {
            return Ok(settings);
        }
    }

    // Return default settings if none found
    Ok(Settings::default())
}

#[tauri::command]
fn start_watching(app: AppHandle) -> Result<(), String> {
    let mut watcher_lock = STATE.watcher.lock().unwrap();
    if watcher_lock.is_some() {
        return Ok(());
    }
    
    // Get settings from persistent store instead of memory
    let settings = get_settings_from_store(&app)?;
    if settings.folder.is_empty() || settings.server.is_empty() {
        return Err("folder or server not set".into());
    }
    
    let (stop_tx, stop_rx) = channel::<()>();
    let folder = settings.folder.clone();
    let server = settings.server.clone();
    let app_handle = app.clone();

    println!("Starting watcher for folder: {}", folder);
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
            if stop_rx.try_recv().is_ok() {
                break;
            }
            if let Ok(Ok(event)) = rx.recv_timeout(std::time::Duration::from_millis(500)) {
                if matches!(event.kind, EventKind::Create(_)) {
                    for path in event.paths {
                        if is_media_file(&path) {
                            // Get fresh settings from store for each upload
                            let current_settings = match get_settings_from_store(&app_handle) {
                                Ok(s) => s,
                                Err(e) => {
                                    println!("Failed to get settings: {}", e);
                                    continue;
                                }
                            };
                            
                            match upload_file(&client, &current_settings.server, &path) {
                                Ok(()) => println!("Successfully uploaded: {}", path.display()),
                                Err(e) => println!("Failed to upload {}: {}", path.display(), e),
                            }
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
    println!("Stopping watcher");
    if let Some(w) = STATE.watcher.lock().unwrap().take() {
        let _ = w.stop.send(());
        let _ = w.handle.join();
    }
}

// Add save_settings and load_settings commands to work with the store
#[tauri::command]
fn save_settings(app: AppHandle, folder: String, server: String) -> Result<(), String> {
    println!("Saving settings: folder = {}, server = {}", folder, server);
    
    let store = app.store("store.json").map_err(|e| e.to_string())?;
    let settings = Settings { folder, server };
    
    store.set("settings", serde_json::to_value(&settings).map_err(|e| e.to_string())?);
    store.save().map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
fn load_settings(app: AppHandle) -> Result<Settings, String> {
    println!("Loading settings");
    get_settings_from_store(&app)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            save_settings,
            load_settings,
            start_watching,
            stop_watching
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}