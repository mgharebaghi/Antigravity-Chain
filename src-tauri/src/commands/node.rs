use crate::state::AppState;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn start_node(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    crate::node::manager::start_node_service(app_handle, state).await
}

#[tauri::command]
pub fn stop_node(state: State<'_, AppState>) -> Result<String, String> {
    state.is_running.store(false, Ordering::Relaxed);
    // Note: We don't necessarily need to increment run_id here since is_running=false is checked.
    // But incrementing ensures double safety.
    state.run_id.fetch_add(1, Ordering::Relaxed);
    Ok("Node stopped".to_string())
}
