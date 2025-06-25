use xxhash_rust::xxh3::xxh3_128;
use reqwest::blocking::Client;

pub fn upload_media(client: &Client, server: &str, data: Vec<u8>, content_type: &str) -> Result<(), String> {
    println!("Uploading file");

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
