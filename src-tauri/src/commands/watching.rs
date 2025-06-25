use std::path::Path;
use std::sync::mpsc::channel;
use notify::{recommended_watcher, EventKind, RecursiveMode, Watcher};
use reqwest::blocking::Client;

use crate::utils::erabooru::UploadResult;
// Import from your other modules
use crate::{STATE, WatcherHandle};
use crate::utils;

#[tauri::command]
pub fn start_watching(app: tauri::AppHandle) -> Result<(), String> {
    let mut watcher_lock = STATE.watcher.lock().unwrap();
    if watcher_lock.is_some() {
        return Ok(());
    }
    
    // Get settings from persistent store instead of memory
    let settings = utils::store::get_settings(&app)?;
    if settings.folder.is_empty() || settings.server.is_empty() {
        return Err("folder or server not set".into());
    }
    
    let (stop_tx, stop_rx) = channel::<()>();
    let folder = settings.folder.clone();
    let server = settings.server.clone();

    //todo: lock settings changes while watching
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
                            let data = match utils::files::retry_read_file(&path, 3, std::time::Duration::from_millis(1000)) {
                                Ok(d) => d,
                                Err(e) => {
                                    println!("Failed to read file {}: {}", path.display(), e);
                                    continue;
                                }
                            };

                            let content_type = utils::files::get_file_mime_type(&path)
                                .unwrap_or_else(|| "application/octet-stream".into());

                            match utils::erabooru::upload_media(&client, &settings.server, data, &content_type) {
                                Ok(UploadResult::Uploaded) => println!("✓ Uploaded: {}", path.display()),
                                Ok(UploadResult::Duplicate) => println!("⚠ Skipped (duplicate): {}", path.display()),
                                Err(e) => println!("✗ Failed to upload {}: {}", path.display(), e),
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

