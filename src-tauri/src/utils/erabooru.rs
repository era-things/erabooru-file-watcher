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
        .post(url)
        .json(&serde_json::json!({ "filename": filename }))
        .send()
        .map_err(|e| e.to_string())?;
    
    // Parse the JSON response to extract the actual upload URL
    let response_text = resp.text().map_err(|e| e.to_string())?;
    
    let upload_response: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse upload URL response: {}", e))?;
    
    let upload_url = upload_response["url"]
        .as_str()
        .ok_or("No 'url' field in upload response")?;

    let put_resp = client
        .put(upload_url)
        .header(reqwest::header::CONTENT_TYPE, content_type)
        .header(reqwest::header::IF_NONE_MATCH, "*")
        .body(data)
        .send()
        .map_err(|e| e.to_string())?;

    match put_resp.status().as_u16() {
        200 | 201 | 204 => Ok(UploadResult::Uploaded(filename)),
        412 => Ok(UploadResult::Duplicate(filename)),
        status => {
            Err(format!("Upload failed with status {}: {}", status, put_resp.status().canonical_reason().unwrap_or("Unknown error")))
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
    client
        .post(url)
        .json(&tags)
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    Ok(())
}
