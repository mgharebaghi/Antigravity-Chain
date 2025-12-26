use std::time::Instant;
use tauri_appantigravity_chain_lib::consensus::Consensus;

fn main() {
    println!("=== Antigravity Sharding Benchmark ===");

    let mut consensus = Consensus::new();
    let epoch_seed = 123456789;

    let scenarios = vec![1_000, 10_000, 50_000, 100_000];

    for &node_count in &scenarios {
        println!("\nTesting with {} validators:", node_count);

        // 1. Population
        let start_pop = Instant::now();
        for i in 0..node_count {
            consensus.register_node(format!("node_{}", i));
        }
        let pop_time = start_pop.elapsed();
        println!("  Population Time: {:.2}ms", pop_time.as_millis());

        // 2. Shard Calculation
        let start_calc = Instant::now();
        let total_shards = consensus.calculate_active_shards();
        let calc_time = start_calc.elapsed();
        println!(
            "  Shard Calculation Time: {:.4}ms",
            calc_time.as_micros() as f64 / 1000.0
        );
        println!("  Active Shards: {}", total_shards);

        // 3. Assignment Speed (Simulate network-wide shuffle)
        let start_assign = Instant::now();
        let mut distribution = vec![0; total_shards as usize];

        for i in 0..node_count {
            let peer_id = format!("node_{}", i);
            let shard_id = consensus.get_assigned_shard(&peer_id, epoch_seed);
            distribution[shard_id as usize] += 1;
        }
        let assign_time = start_assign.elapsed();

        println!(
            "  Assignment Time (All Nodes): {:.2}ms",
            assign_time.as_millis()
        );
        println!(
            "  Avg Time per Node: {:.4}us",
            assign_time.as_micros() as f64 / node_count as f64
        );

        // Check uniformity (first 5 shards)
        println!(
            "  Sample Distribution (First 5 shards): {:?}",
            &distribution[0..5.min(distribution.len())]
        );

        // Reset for next run
        consensus.nodes.clear();
    }
}
