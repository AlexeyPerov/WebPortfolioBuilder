use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

const SETTINGS_FILE: &str = "studio-settings.json";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WorkspaceLayout {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sidebar_px: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_px: Option<u32>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StudioSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_project_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_layout: Option<WorkspaceLayout>,
}

pub fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(SETTINGS_FILE))
}

pub fn load_settings(app: &tauri::AppHandle) -> Result<StudioSettings, String> {
    let path = settings_path(app)?;
    if !path.is_file() {
        return Ok(StudioSettings::default());
    }
    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

pub fn save_settings(app: &tauri::AppHandle, settings: &StudioSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    let data = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())
}
