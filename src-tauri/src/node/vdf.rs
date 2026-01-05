//! # VDF (Verifiable Delay Function) Module
//!
//! This module handles:
//! - VDF heartbeat for performance monitoring
//! - VDF solver for Proof of Patience verification
//!
//! IMPORTANT: VDF computation is CPU-intensive and runs in a dedicated blocking
//! thread pool to avoid blocking the async runtime.

use crate::chain::VdfProofMessage;
use crate::consensus::vdf::CentichainVDF;
use crate::consensus::Consensus;
use crate::state::VdfStatus;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

// =============================================================================
// Constants
// =============================================================================

/// Base VDF difficulty for Proof of Patience
const VDF_BASE_DIFFICULTY: u64 = 3_000_000;

/// Additional difficulty per network validator (Sybil resistance)
const VDF_DIFFICULTY_PER_VALIDATOR: u64 = 500_000;

/// Small difficulty for heartbeat benchmarking (keeps UI responsive)
const VDF_HEARTBEAT_DIFFICULTY: u64 = 50_000;

// =============================================================================
// VDF Heartbeat - Performance Monitoring
// =============================================================================

/// Spawns a background task that periodically benchmarks VDF performance.
///
/// This provides real-time "iterations per second" metrics for the UI
/// and ensures the VDF subsystem is responsive.
pub fn spawn_vdf_heartbeat(
    app_handle: AppHandle,
    is_running: Arc<AtomicBool>,
    run_id: Arc<AtomicU64>,
    vdf_ips: Arc<AtomicU64>,
    my_run_id: u64,
) {
    tauri::async_runtime::spawn(async move {
        log::info!("VDF Heartbeat: Started for run_id: {}", my_run_id);

        let mut last_emit = std::time::Instant::now();
        let display_difficulty = 200_000;

        loop {
            // Check if we should stop
            if !is_running.load(Ordering::Relaxed) || run_id.load(Ordering::Relaxed) != my_run_id {
                break;
            }

            // Run VDF benchmark in blocking thread pool (non-blocking to async runtime)
            let benchmark_result = tokio::task::spawn_blocking(|| {
                let start = std::time::Instant::now();
                let vdf = CentichainVDF::new(VDF_HEARTBEAT_DIFFICULTY);
                vdf.solve(b"heartbeat_challenge");
                start.elapsed()
            })
            .await;

            if let Ok(elapsed) = benchmark_result {
                let ips = (VDF_HEARTBEAT_DIFFICULTY as f64 / elapsed.as_secs_f64()) as u64;
                vdf_ips.store(ips, Ordering::Relaxed);

                // Emit status update every second
                if last_emit.elapsed() >= Duration::from_secs(1) {
                    let _ = app_handle.emit(
                        "vdf-status",
                        VdfStatus {
                            iterations_per_second: ips,
                            difficulty: display_difficulty,
                            is_active: true,
                        },
                    );
                    last_emit = std::time::Instant::now();
                }
            }

            // Brief pause between benchmarks
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        log::info!("VDF Heartbeat: Terminated for run_id: {}", my_run_id);
    });
}

// =============================================================================
// VDF Solver - Proof of Patience
// =============================================================================

/// Spawns the VDF solver task for Proof of Patience verification.
///
/// This task:
/// 1. Waits until the node is synced with the network
/// 2. Checks if the local node needs VDF verification
/// 3. Solves the VDF challenge (in a blocking thread to not block async runtime)
/// 4. Broadcasts the proof to the network
///
/// # Key Design Decision
/// The VDF computation runs in `spawn_blocking` so it doesn't block the
/// async runtime. This allows mining, P2P, and other operations to continue
/// while VDF solving happens in the background.
pub fn spawn_vdf_solver(
    app_handle: AppHandle,
    is_running: Arc<AtomicBool>,
    is_synced: Arc<AtomicBool>,
    consensus: Arc<Mutex<Consensus>>,
    vdf_sender: tokio::sync::mpsc::Sender<VdfProofMessage>,
) {
    tauri::async_runtime::spawn(async move {
        log::info!("VDF Solver: Started");

        loop {
            // Check if we should stop
            if !is_running.load(Ordering::Relaxed) {
                break;
            }

            // Wait between checks
            tokio::time::sleep(Duration::from_secs(5)).await;

            // Requirement: Must be synced before attempting Proof of Patience
            if !is_synced.load(Ordering::Relaxed) {
                continue;
            }

            // Get node state
            let (my_peer_id, needs_proof, validator_count) = {
                let c = consensus.lock().unwrap();
                match (
                    &c.local_peer_id,
                    c.local_peer_id.as_ref().and_then(|pid| c.nodes.get(pid)),
                ) {
                    (Some(pid), Some(node)) => {
                        (Some(pid.clone()), !node.is_verified, c.nodes.len())
                    }
                    _ => (None, false, 0),
                }
            };

            // Skip if already verified or no peer ID
            let Some(pid) = my_peer_id else { continue };
            if !needs_proof {
                continue;
            }

            // === VDF Proof Required ===
            log::info!("VDF Solver: Starting Proof of Patience for {}", pid);
            let _ = app_handle.emit("node-status", "Solving Proof of Patience...");

            // Calculate challenge
            let challenge = {
                let c = consensus.lock().unwrap();
                c.get_vdf_challenge(&pid)
            };

            // Calculate adaptive difficulty (Sybil resistance)
            let difficulty =
                VDF_BASE_DIFFICULTY + (validator_count as u64 * VDF_DIFFICULTY_PER_VALIDATOR);
            log::info!(
                "VDF Solver: Difficulty = {} (~{}s expected)",
                difficulty,
                difficulty / 1_000_000
            );

            // Update UI with solving status
            let _ = app_handle.emit(
                "vdf-status",
                VdfStatus {
                    iterations_per_second: 1_000_000, // Rough estimate
                    difficulty,
                    is_active: true,
                },
            );

            // === CRITICAL: Run VDF in blocking thread pool ===
            // This prevents blocking the async runtime, allowing mining and P2P to continue
            let challenge_bytes = challenge.clone().into_bytes();
            let solve_result = tokio::task::spawn_blocking(move || {
                let vdf = CentichainVDF::new(difficulty);
                vdf.solve(&challenge_bytes)
            })
            .await;

            let proof = match solve_result {
                Ok(p) => p,
                Err(e) => {
                    log::error!("VDF Solver: spawn_blocking failed: {}", e);
                    continue;
                }
            };

            log::info!(
                "VDF Solver: Solved! Proof: {}",
                &proof[..16.min(proof.len())]
            );

            // Verify self
            {
                let mut c = consensus.lock().unwrap();
                if c.verify_peer(pid.clone(), proof.clone()) {
                    log::info!("VDF Solver: Self-verification successful");
                } else {
                    log::warn!("VDF Solver: Self-verification failed!");
                }
            }

            // Broadcast proof to network
            let msg = VdfProofMessage {
                peer_id: pid.clone(),
                proof,
                challenge,
            };

            if let Err(e) = vdf_sender.send(msg).await {
                log::error!("VDF Solver: Failed to broadcast proof: {}", e);
            } else {
                log::info!("VDF Solver: Proof broadcast to network for {}", pid);
            }

            // Update UI
            let _ = app_handle.emit("node-status", "Active (Verified)");
            let _ = app_handle.emit("vdf-solved", true);
        }

        log::info!("VDF Solver: Terminated");
    });
}
