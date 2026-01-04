//! # Centichain Constants
//!
//! All global constants used throughout the blockchain.

// ============================================================================
// Network Configuration
// ============================================================================

/// Multiple relay node addresses for decentralization and failover
/// Each relay is geographically distributed to ensure network availability
pub const RELAY_ADDRESSES: &[&str] = &[
    "/ip4/127.0.0.1/tcp/9090", // Primary relay (local/development)
    "/ip4/127.0.0.1/tcp/9091", // Secondary relay (local/development)
    "/ip4/127.0.0.1/tcp/9092", // Tertiary relay (local/development)
                               // TODO: Replace with production relay addresses:
                               // "/dns4/relay1.centichain.io/tcp/4001/p2p/12D3KooW...",
                               // "/dns4/relay2.centichain.io/tcp/4001/p2p/12D3KooW...",
                               // "/dns4/relay3.centichain.io/tcp/4001/p2p/12D3KooW...",
];

/// Maximum relay connection attempts per address
pub const MAX_RELAY_ATTEMPTS_PER_ADDRESS: u32 = 10;

/// Delay between relay connection attempts (seconds)
pub const RELAY_RETRY_DELAY_SECS: u64 = 3;

/// Minimum DHT peers required to operate without relay
pub const DHT_PEER_THRESHOLD_FOR_RELAY_FREE: usize = 3;

/// Maximum number of peer connections
pub const DEFAULT_MAX_PEERS: u32 = 50;

// ============================================================================
// Consensus Configuration
// ============================================================================

/// Duration of each slot in seconds (matches block time)
pub const SLOT_DURATION: u64 = 2;

/// Duration of each epoch in seconds (10 minutes)
pub const EPOCH_DURATION: u64 = 600;

/// Base quarantine duration for solo nodes (5 minutes)
pub const BASE_QUARANTINE_SECS: u64 = 300;

/// Maximum quarantine duration (72 hours)
pub const MAX_QUARANTINE_SECS: u64 = 72 * 3600;

// ============================================================================
// Performance Parameters (1500 TPS per Shard)
// ============================================================================

/// Target block time in seconds
pub const TARGET_BLOCK_TIME: u64 = 2;

/// Maximum transactions per block (3000 tx / 2s = 1500 TPS)
pub const MAX_TXS_PER_BLOCK: u64 = 3_000;

/// Maximum block size in bytes (1.5 MB)
pub const MAX_BLOCK_SIZE: u64 = 1_500_000;

// ============================================================================
// Synchronization Parameters
// ============================================================================

/// Grace period after sync before mining (allows gossip blocks to arrive)
pub const SYNC_GRACE_PERIOD_SECS: u64 = 5;

/// How long to wait into a slot before producing a block
pub const SLOT_PRODUCTION_DELAY_SECS: u64 = 1;

/// Maximum time to wait for initial sync (5 minutes)
pub const MAX_SYNC_WAIT_SECS: u64 = 300;

/// How many seconds to confirm sync stability
pub const SYNC_CONFIRMATION_TICKS: u64 = 3;

// ============================================================================
// Tokenomics
// ============================================================================

/// AGT token decimals
pub const AGT_DECIMALS: u32 = 6;

/// One AGT in smallest units
pub const ONE_AGT: u64 = 1_000_000;

/// Total supply (21 million AGT)
pub const TOTAL_SUPPLY: u64 = 21_000_000 * ONE_AGT;

/// Genesis supply (5 million AGT)
pub const GENESIS_SUPPLY: u64 = 5_000_000 * ONE_AGT;

/// Initial block reward (~0.12 AGT)
pub const INITIAL_REWARD: u64 = 126_839;

/// Halving interval in blocks (4 years at 2s blocks)
pub const HALVING_INTERVAL: u64 = 63_072_000;

// ============================================================================
// VDF Configuration
// ============================================================================

/// Base VDF difficulty for solo nodes
pub const VDF_BASE_DIFFICULTY: u64 = 3_000_000;

/// Additional difficulty per validator
pub const VDF_DIFFICULTY_PER_VALIDATOR: u64 = 500_000;

// ============================================================================
// Storage
// ============================================================================

/// Number of blocks to keep for pruned nodes
pub const PRUNED_HISTORY_BLOCKS: u64 = 2000;

// ============================================================================
// Sharding
// ============================================================================

/// Validators per shard (auto-sharding threshold)
pub const VALIDATORS_PER_SHARD: usize = 50;
