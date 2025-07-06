use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use crate::utils;

#[tauri::command]
pub fn load_settings(app: AppHandle) -> Result<utils::store::Settings, String> {
    println!("Loading settings");
    utils::store::get_settings(&app)
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    folder: String,
    server: String,
    auto_tags: Vec<utils::store::AutoTagRule>,
    override_upload_date: bool,
) -> Result<(), String> {
    println!("Saving settings: folder = {}, server = {}", folder, server);

    let store = app.store("store.json").map_err(|e| e.to_string())?;
    let settings = utils::store::Settings {
        folder,
        server,
        auto_tags,
        override_upload_date,
    };

    store.set(
        "settings",
        serde_json::to_value(&settings).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;
    
    Ok(())
}
