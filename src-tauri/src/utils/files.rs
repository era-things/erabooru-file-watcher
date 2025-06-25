use std::path::Path;
use mime_guess::MimeGuess;

pub fn is_media_file(path: &Path) -> bool {
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

pub fn get_file_mime_type(path: &Path) -> Option<String> {
    if !path.is_file() {
        return None;
    }
    let guess = MimeGuess::from_path(path).first();
    guess.map(|m| m.essence_str().to_string())
}

pub fn retry_read_file(
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