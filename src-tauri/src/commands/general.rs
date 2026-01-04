use crate::state::{AppSettings, AppState};
use std::sync::atomic::Ordering;
use tauri::State;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn get_app_settings(state: State<'_, AppState>) -> AppSettings {
    match state.storage.get_setting("app_settings") {
        Ok(Some(json)) => serde_json::from_str(&json).unwrap_or_default(),
        _ => AppSettings::default(),
    }
}

#[tauri::command]
pub fn save_app_settings(state: State<'_, AppState>, settings: AppSettings) -> Result<(), String> {
    // Update reactive flags
    state
        .mining_enabled
        .store(settings.mining_enabled, Ordering::Relaxed);

    {
        let mut nt = state.node_type.lock().unwrap();
        *nt = settings.node_type.clone();
    }

    let json = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
    state
        .storage
        .save_setting("app_settings", &json)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn exit_app() {
    std::process::exit(0);
}
