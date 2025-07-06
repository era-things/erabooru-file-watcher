use xxhash_rust::xxh3::xxh3_128;
use reqwest::blocking::Client;
use std::{path::Path, thread, time::Duration};

use crate::utils::{files, tagging, store::AutoTagRule};

#[derive(Debug)]
pub enum UploadResult {
    Uploaded(String),
    Duplicate(String),
}

pub fn upload_media(
    client: &Client,
    server: &str,
    data: Vec<u8>,
    content_type: &str,
) -> Result<UploadResult, String> {
    let hash = xxh3_128(&data);
    let filename = format!("{:032x}", hash);
    let url = format!("{}/api/media/upload-url", server.trim_end_matches('/'));

    let resp = client
        .post(&url)
        .json(&serde_json::json!({ "filename": filename }))
        .send()
        .map_err(|e| format!("Failed to get upload URL: {}", e))?;
    
    if !resp.status().is_success() {
        return Err(format!("Upload URL request failed with status: {}", resp.status()));
    }
    
    let response_text = resp.text().map_err(|e| format!("Failed to read response: {}", e))?;
    
    let upload_response: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse upload URL response: {}", e))?;
    
    let upload_url_path = upload_response["url"]
        .as_str()
        .ok_or("No 'url' field in upload response")?;

    // Construct the full URL by combining server base URL with the relative path
    let full_upload_url = if upload_url_path.starts_with("http") {
        // Already a full URL
        upload_url_path.to_string()
    } else {
        // Relative URL, combine with server base
        format!("{}{}", server.trim_end_matches('/'), upload_url_path)
    };

    let put_resp = client
        .put(&full_upload_url)  // Use the full URL here
        .header(reqwest::header::CONTENT_TYPE, content_type)
        .header(reqwest::header::IF_NONE_MATCH, "*")
        .body(data)
        .send()
        .map_err(|e| format!("Failed to upload file: {}", e))?;

    match put_resp.status().as_u16() {
        200 | 201 | 204 => Ok(UploadResult::Uploaded(filename)),
        412 => Ok(UploadResult::Duplicate(filename)),
        status => {
            let error_text = put_resp.text().unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Upload failed with status {}: {}", status, error_text))
        }
    }
}

pub fn add_tags(
    client: &Client,
    server: &str,
    id: &str,
    tags: &[&str],
) -> Result<(), String> {
    let url = format!("{}/api/media/{}/tags", server.trim_end_matches('/'), id);
    
    println!("Adding tags to {}: {:?}", id, tags);
    
    let resp = client
        .post(&url)
        .json(&serde_json::json!({ "tags": tags }))  
        .send()
        .map_err(|e| format!("Failed to add tags: {}", e))?;
    
    let status = resp.status();
    if status.is_success() {
        println!("Tags added successfully");
        Ok(())
    } else {
        let error_text = resp.text().unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Failed to add tags, status {}: {}", status, error_text))
    }
}

pub fn add_date(
    client: &Client,
    server: &str,
    id: &str,
    name: &str,
    value: &str,
) -> Result<(), String> {
    let url = format!("{}/api/media/{}/dates", server.trim_end_matches('/'), id);

    let resp = client
        .post(&url)
        .json(&serde_json::json!({ "dates": [{ "name": name, "value": value }] }))
        .send()
        .map_err(|e| format!("Failed to add date: {}", e))?;

    let status = resp.status();
    if status.is_success() {
        Ok(())
    } else {
        let error_text = resp.text().unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Failed to add date, status {}: {}", status, error_text))
    }
}

pub fn apply_tags_and_date(
    client: &Client,
    server: &str,
    path: &Path,
    id: &str,
    auto_tags: &[AutoTagRule],
    override_upload_date: bool,
) {
    // Wait for the media to be indexed before applying tags and dates
    if let Err(e) = wait_for_media_indexing(
        client, 
        server, 
        id, 
        10, // max attempts
        Duration::from_millis(500) // delay between attempts
    ) {
        println!("Failed to wait for media indexing for {}: {}", path.display(), e);
        return;
    }
    
    let tags = tagging::tags_for_path(path, auto_tags);
    if !tags.is_empty() {
        let tag_refs: Vec<&str> = tags.iter().map(|t| t.as_str()).collect();
        if let Err(e) = add_tags(client, server, id, &tag_refs) {
            println!("Failed to tag {}: {}", path.display(), e);
        }
    }

    if override_upload_date {
        if let Ok(date) = files::file_modified_utc(path) {
            if let Err(e) = add_date(client, server, id, "upload", &date) {
                println!("Failed to set date for {}: {}", path.display(), e);
            }
        }
    }
}

pub fn check_media_exists(
    client: &Client,
    server: &str,
    id: &str,
) -> Result<bool, String> {
    let url = format!("{}/api/media/{}", server.trim_end_matches('/'), id);
    
    let resp = client
        .get(&url)
        .send()
        .map_err(|e| format!("Failed to check media existence: {}", e))?;
    
    match resp.status().as_u16() {
        200 => Ok(true),
        404 => Ok(false),
        status => Err(format!("Unexpected status when checking media: {}", status)),
    }
}

pub fn wait_for_media_indexing(
    client: &Client,
    server: &str,
    id: &str,
    max_attempts: u32,
    delay: Duration,
) -> Result<(), String> {
    println!("Waiting for media {} to be indexed...", id);
    
    for attempt in 1..=max_attempts {
        match check_media_exists(client, server, id) {
            Ok(true) => {
                println!("Media {} is now available (attempt {})", id, attempt);
                return Ok(());
            }
            Ok(false) => {
                if attempt < max_attempts {
                    println!("Media {} not yet available, waiting... (attempt {}/{})", id, attempt, max_attempts);
                    thread::sleep(delay);
                } else {
                    return Err(format!("Media {} was not indexed after {} attempts", id, max_attempts));
                }
            }
            Err(e) => {
                println!("Error checking media existence: {}", e);
                if attempt < max_attempts {
                    thread::sleep(delay);
                } else {
                    return Err(format!("Failed to verify media indexing: {}", e));
                }
            }
        }
    }
    
    Ok(())
}
