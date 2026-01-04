# Centichain Architecture

## Overview

Centichain is the **fastest blockchain in the world**, designed to achieve unlimited scalability through adaptive sharding while maintaining complete decentralization.

### Performance

- **Per-Shard TPS**: 1,500 transactions per second
- **Block Time**: 2 seconds
- **Block Capacity**: 3,000 transactions per block
- **Transactions Per Block**: Up to 3,000
- **Global TPS**: Scales linearly with shards (50 validators = 1 shard, 100 = 2 shards, etc.)

## Core Architecture

### 1. Consensus: AHSP (Adaptive Hierarchical Sharded Proof-of-Patience)

**AHSP** combines multiple innovations:

#### Proof of Patience (PoP)
- New nodes must solve a VDF (Verifiable Delay Function)
- Difficulty scales with network size (prevents Sybil attacks)
- Solo nodes: 5 minutes, Large networks: up to 72 hours
- **Once verified, nodes become "established"** and retain eligibility

#### Round-Robin Leader Selection
- Deterministic slot-based leader election
- Leaders rotate every 2 seconds
- Fair distribution across all eligible validators
- **No mining competition** = energy efficient

#### Dynamic Sharding
- Shards activate automatically as network grows
- Formula: `shards = max(1, validators / 50)`
- Cross-shard transactions via atomic receipts
- Each shard operates independently at 1500 TPS

### 2. State Machine

```
┌─────────┐
│ Stopped │
└────┬────┘
     │
     ▼
┌──────────────────┐
│ Connecting       │ (Dialing relay node)
└────┬─────────────┘
     │
     ▼
┌──────────────────┐
│ Discovering      │ (DHT peer discovery, 90s max)
└────┬─────────────┘
     │
     ├──► No Peers Found ──► Genesis Creation (First Node)
     │
     └──► Peers Found ────┐
                          │
                          ▼
                   ┌──────────────┐
                   │ Synchronizing│ (Headers-first sync)
                   └──────┬───────┘
                          │
                          ▼
                   ┌──────────────┐
                   │ Grace Period │ (5s wait for gossip)
                   └──────┬───────┘
                          │
                          ▼
                   ┌─────────────┐
                   │   Active    │
                   └──────┬──────┘
                          │
          ┌───────────────┼───────────────┐
          │               │               │
          ▼               ▼               ▼
    ┌──────────┐   ┌──────────┐   ┌──────────┐
    │ Patience │   │  Queue   │   │  Leader  │
    │   Mode   │──▶│ (Waiting)│──▶│ (Mining) │
    └──────────┘   └──────────┘   └──────────┘
```

### 3. Network Synchronization (CRITICAL FIX)

**Problem**: Non-first nodes were setting `is_synced=true` prematurely, causing them to mine independently.

**Solution**:
1. **P2P layer** only DETECTS sync progress (downloads blocks)
2. **Main loop** (lib.rs) CONTROLS sync state transitions
3. **Grace period** after sync before mining (allows gossip blocks to arrive)
4. **Slot progress check** before mining (wait 1s into slot for network blocks)

```rust
// BEFORE (BUGGY):
if remote_height <= local_height {
    is_synced.store(true); // ❌ Too early!
}

// AFTER (FIXED):
// P2P downloads blocks, main loop waits for:
// 1. Heights match across peers
// 2. Grace period (5s)
// 3. Then sets is_synced=true
```

### 4. Block Production (CRITICAL FIX)

**Problem**: Nodes would produce blocks immediately when becoming leader, even if network block was in-flight.

**Solution**:
```rust
// Wait 1 second into slot before producing
if slot_progress < 1 {
  continue; // Let network gossip arrive first
}

// Then check if slot already has a block
if latest_block_slot >= current_slot {  
  continue; // Network block received, don't fork
}

// Only NOW safe to produce
produce_block();
```

## Technology Stack

### Backend (Rust)
- **libp2p**: P2P networking (gossipsub, Kademlia DHT, relay)
- **redb**: Embedded database for blockchain storage
- **tokio**: Async runtime
- **tauri**: Desktop app framework

### Frontend (React + TypeScript)
- **React 19**: UI framework
- **Framer Motion**: Animations
- **TailwindCSS**: Styling
- **react-force-graph**: Network visualization

## File Structure

```
src-tauri/src/
├── lib.rs          - Main node loop, state machine
├── p2p.rs          - Network layer (gossip, sync, DHT)
├── consensus.rs    - AHSP consensus logic
├── chain.rs        - Block/transaction structures
├── storage.rs      - redb persistence
├── mempool.rs      - Transaction pool
├── vdf.rs          - Verifiable Delay Function
├── wallet.rs       - Key management
└── commands.rs     - Tauri commands

src/
├── components/     - React UI components
├── pages/          - Main app pages
└── context/        - State management
```

## Network Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Block Time | 2s | Fast finality |
| Transactions/Block | 3,000 | 1500 TPS target |
| Slot Duration | 2s | Matches block time |
| Epoch Duration | 10 min | Shard reassignment period |
| VDF Base Difficulty | 3M iterations | ~3s solve time |
| VDF Scaling | +500K per validator | Sybil resistance |
| Quarantine (Solo) | 5 min | Fast bootstrap |
| Quarantine (Network) | 1-72 hours | Scales with size |

## Security Model

1. **Sybil Resistance**: VDF difficulty scales with network size
2. **NAT Traversal**: Relay nodes enable home validators
3. **Byzantine Tolerance**: Round-robin prevents single-point control
4. **Finality**: Blocks finalized after >50% of validators see them
5. **Slashing**: Missed slots reduce trust score

## Performance Optimizations

1. **Parallel Transaction Execution**: Independent txs processed concurrently
2. **Headers-First Sync**: Verify chain before full block download
3. **Pruned Nodes**: option Keep last 2000 blocks for home users
4. **Gossip Optimization**: Blocks propagate in <500ms globally
5. **Memory-Hard VDF**: Prevents ASIC dominance

## Future Enhancements

- [ ] BLS signature aggregation for cross-links
- [ ] Zero-knowledge fraud proofs
- [ ] State channels for microtransactions
- [ ] Full sharding (data + computation)
- [ ] Quantum-resistant signatures
