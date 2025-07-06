use serde::{Deserialize, Serialize};
use tauri_plugin_store::StoreExt;


#[derive(Default, Serialize, Deserialize, Clone)]
pub struct AutoTagRule {
    pub folder: String,
    pub tags: String,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub folder: String,
    pub server: String,
    #[serde(default)]
    pub auto_tags: Vec<AutoTagRule>,
    #[serde(default)]
    pub override_upload_date: bool,
}

pub fn get_settings(app: &tauri::AppHandle) -> Result<Settings, String> {
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
