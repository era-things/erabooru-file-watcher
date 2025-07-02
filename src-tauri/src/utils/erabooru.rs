use xxhash_rust::xxh3::xxh3_128;
use reqwest::blocking::Client;

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