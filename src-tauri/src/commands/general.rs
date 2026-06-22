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
    // Get the current mining status before update
    let old_mining = state.mining_enabled.load(Ordering::Relaxed);
    let new_mining = settings.mining_enabled;

    // Update reactive flags
    state.mining_enabled.store(new_mining, Ordering::Relaxed);

    // If mining status changed, update consensus AND broadcast to network
    if old_mining != new_mining {
        // Update local consensus
        {
            let mut c = state.consensus.lock().unwrap();
            if let Some(ref peer_id) = c.local_peer_id.clone() {
                c.set_peer_mining_status(&peer_id, new_mining);
            }
        }

        // Broadcast to network via P2P command channel
        if let Some(ref sender) = *state.p2p_cmd_sender.lock().unwrap() {
            let cmd = crate::network::P2PCommand::BroadcastMiningStatus {
                mining_active: new_mining,
            };
            let _ = sender.try_send(cmd);
            log::info!(
                "Settings: Mining status changed to {}, broadcast sent to network",
                new_mining
            );
        } else {
            log::warn!("Settings: Mining status changed but P2P not running");
        }
    }

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
