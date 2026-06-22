//! # Relay Connection Module
//!
//! Handles Phase 1 of the mining loop: connecting to the relay server.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Time to wait for relay connection (seconds)
pub const RELAY_CONNECTION_TIMEOUT: u64 = 10;

/// Waits for relay connection or timeout
///
/// Returns `true` if relay connected, `false` if timed out or stopped
pub async fn wait_for_relay(
    is_running: &Arc<AtomicBool>,
    run_id: &Arc<AtomicU64>,
    my_run_id: u64,
    relay_connected: &Arc<AtomicBool>,
    timeout_secs: u64,
) -> bool {
    for i in 0..timeout_secs {
        if !is_running.load(Ordering::Relaxed) || run_id.load(Ordering::Relaxed) != my_run_id {
            return false;
        }

        if relay_connected.load(Ordering::Relaxed) {
            log::info!("Mining Loop: Relay connected after {}s", i);
            return true;
        }

        log::debug!("Mining Loop: Waiting for relay... ({}s)", i);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    false
}

/// Emits relay error and waits for node stop
///
/// This function runs until the node is stopped, continuously
/// emitting error status to the frontend.
pub async fn emit_relay_error(
    app_handle: &AppHandle,
    is_running: &Arc<AtomicBool>,
    run_id: &Arc<AtomicU64>,
    my_run_id: u64,
) {
    let _ = app_handle.emit("node-status", "Error: Relay Unreachable");

    while is_running.load(Ordering::Relaxed) && run_id.load(Ordering::Relaxed) == my_run_id {
        let _ = app_handle.emit(
            "node-status",
            "Error: Relay Unreachable. Please check config/network.",
        );
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
