use walkdir::WalkDir;
use reqwest::blocking::Client;
use std::time::Duration;
use crate::utils;

#[tauri::command]
pub fn scan_folder(folder: String) -> Result<(u64, u64, u64), String> {
    let mut videos = 0u64;
    let mut images = 0u64;
    let mut size = 0u64;
    for entry in WalkDir::new(&folder).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if utils::files::is_media_file(path) {
            let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
            size += metadata.len();
            let guess = mime_guess::MimeGuess::from_path(path).first();
            if let Some(m) = guess {
                if m.type_() == mime_guess::mime::IMAGE {
                    images += 1;
                } else if m.type_() == mime_guess::mime::VIDEO {
                    videos += 1;
                }
            }
        }
    }
    Ok((videos, images, size))
}

#[tauri::command]
pub fn upload_folder(app: tauri::AppHandle, folder: String) -> Result<(), String> {
    let settings = utils::store::get_settings(&app)?;
    if settings.server.is_empty() {
        return Err("server not set".into());
    }
    let client = Client::new();
    for entry in WalkDir::new(&folder).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if utils::files::is_media_file(path) {
            let data = match utils::files::retry_read_file(path, 3, Duration::from_millis(1000)) {
                Ok(d) => d,
                Err(e) => {
                    println!("Failed to read file {}: {}", path.display(), e);
                    continue;
                }
            };
            let content_type = utils::files::get_file_mime_type(path)
                .unwrap_or_else(|| "application/octet-stream".into());
            match utils::erabooru::upload_media(&client, &settings.server, data, &content_type) {
                Ok(()) => println!("Uploaded {}", path.display()),
                Err(e) => println!("Failed to upload {}: {}", path.display(), e),
            }
        }
    }
    Ok(())
}
