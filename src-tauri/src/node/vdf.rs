use crate::chain::VdfProofMessage;
use crate::consensus::vdf::CentichainVDF;
use crate::consensus::Consensus;
use crate::state::VdfStatus;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub fn spawn_vdf_heartbeat(
    app_handle: AppHandle,
    is_running: Arc<AtomicBool>,
    run_id: Arc<AtomicU64>,
    vdf_ips: Arc<AtomicU64>,
    my_run_id: u64,
) {
    tauri::async_runtime::spawn(async move {
        println!("VDF Heartbeat: Thread started for run_id: {}", my_run_id);
        let mut last_emit = std::time::Instant::now();
        let difficulty = 200_000; // Standard difficulty for display

        loop {
            if !is_running.load(Ordering::Relaxed) || run_id.load(Ordering::Relaxed) != my_run_id {
                break;
            }

            // Benchmark VDF performance
            let start = std::time::Instant::now();
            let vdf = CentichainVDF::new(50_000); // Small batch for responsiveness
            vdf.solve(b"heartbeat_challenge");
            let elapsed = start.elapsed();

            let ips = (50_000.0 / elapsed.as_secs_f64()) as u64;
            vdf_ips.store(ips, Ordering::Relaxed);

            if last_emit.elapsed() >= Duration::from_secs(1) {
                let _ = app_handle.emit(
                    "vdf-status",
                    VdfStatus {
                        iterations_per_second: ips,
                        difficulty,
                        is_active: true,
                    },
                );
                last_emit = std::time::Instant::now();
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        println!("VDF Heartbeat: Terminating run_id {}", my_run_id);
    });
}

pub fn spawn_vdf_solver(
    app_handle: AppHandle,
    is_running: Arc<AtomicBool>,
    is_synced: Arc<AtomicBool>,
    consensus: Arc<Mutex<Consensus>>,
    vdf_sender: tokio::sync::mpsc::Sender<VdfProofMessage>,
) {
    tauri::async_runtime::spawn(async move {
        loop {
            if !is_running.load(Ordering::Relaxed) {
                break;
            }
            tokio::time::sleep(Duration::from_secs(5)).await;

            // Strict Sync Check: Do not attempt PoP until we are synced with the chain tip
            if !is_synced.load(Ordering::Relaxed) {
                continue;
            }

            let (my_peer_id, needs_proof, validator_count) = {
                let c = consensus.lock().unwrap();
                if let Some(pid) = &c.local_peer_id {
                    if let Some(node) = c.nodes.get(pid) {
                        (Some(pid.clone()), !node.is_verified, c.nodes.len())
                    } else {
                        (None, false, 0)
                    }
                } else {
                    (None, false, 0)
                }
            };

            if let Some(pid) = my_peer_id {
                if needs_proof {
                    // We are UNVERIFIED. Solve VDF.
                    println!("VDF Solver: Starting VDF computation for {}...", pid);
                    let _ = app_handle.emit("node-status", "Solving Proof of Patience...");

                    let challenge = {
                        let c = consensus.lock().unwrap();
                        c.get_vdf_challenge(&pid)
                    };

                    // Adaptive Difficulty (Memory-Hard):
                    // Memory access is slower than SHA256.
                    // Base: 3,000,000 iterations (~3s on DDR4)
                    // Step: 500,000 per peer
                    let difficulty: u64 = 3_000_000 + (validator_count as u64 * 500_000);
                    println!(
                        "VDF Solver: Calculated Difficulty: {} (approx {}s)",
                        difficulty,
                        difficulty / 1_000_000
                    );

                    let _ = app_handle.emit(
                        "vdf-status",
                        VdfStatus {
                            iterations_per_second: 1_000_000, // Rough estimate for UI
                            difficulty: difficulty,
                            is_active: true,
                        },
                    );

                    let vdf = CentichainVDF::new(difficulty);
                    let proof = vdf.solve(challenge.as_bytes());

                    println!("VDF Solver: Solved! Proof: {}", proof);

                    // 1. Verify Self
                    {
                        let mut c = consensus.lock().unwrap();
                        c.verify_peer(pid.clone(), proof.clone());
                    }

                    // 2. Broadcast Proof to Network
                    let msg = VdfProofMessage {
                        peer_id: pid.clone(),
                        proof: proof.clone(),
                        challenge: challenge.clone(),
                    };

                    println!("VDF Solver: Broadcasting proof for {}", pid);
                    if let Err(e) = vdf_sender.send(msg).await {
                        eprintln!("Failed to send VDF proof to P2P: {}", e);
                    }

                    let _ = app_handle.emit("node-status", "Active (Verified)");
                    // Force verify UI
                    let _ = app_handle.emit("vdf-solved", true);
                }
            }
        }
    });
}
