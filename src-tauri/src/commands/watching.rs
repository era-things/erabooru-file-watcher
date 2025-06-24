use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;
use notify::{recommended_watcher, EventKind, RecursiveMode, Watcher};
use reqwest::blocking::Client;

// Import from your other modules
use crate::{STATE, WatcherHandle};

use mime_guess::MimeGuess;
use xxhash_rust::xxh3::xxh3_128;
use crate::utils;

#[tauri::command]
pub fn start_watching(app: tauri::AppHandle) -> Result<(), String> {
    let mut watcher_lock = STATE.watcher.lock().unwrap();
    if watcher_lock.is_some() {
        return Ok(());
    }
    
    // Get settings from persistent store instead of memory
    let settings = utils::store::get_settings_from_store(&app)?;
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
                        if utils::files::is_media_file(&path) {
                            // Get fresh settings from store for each upload
                            let current_settings = match utils::store::get_settings_from_store(&app_handle) {
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
pub fn stop_watching() {
    println!("Stopping watcher");
    if let Some(w) = STATE.watcher.lock().unwrap().take() {
        let _ = w.stop.send(());
        let _ = w.handle.join();
    }
}

fn upload_file(client: &Client, server: &str, path: &Path) -> Result<(), String> {
    println!("Uploading file: {}", path.display());

    let data = utils::files::retry_read_file(path, 3, std::time::Duration::from_millis(1000))?;
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

