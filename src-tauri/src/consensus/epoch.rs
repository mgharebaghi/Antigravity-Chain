//! # Epoch and Slot Module
//!
//! Handles time-based consensus mechanics including epochs and slots.
//! An epoch is a period of time during which shard assignments remain stable.
//! A slot is a window for a single block to be produced.

use super::Consensus;
use std::time::{SystemTime, UNIX_EPOCH};

// =============================================================================
// Epoch and Slot Constants and Methods
// =============================================================================

impl Consensus {
    /// Duration of an epoch in seconds (10 minutes for testing)
    pub const EPOCH_DURATION: u64 = 600;

    /// Duration of a slot in seconds (2 seconds)
    pub const SLOT_DURATION: u64 = 2;

    /// Returns the current epoch number
    pub fn current_epoch(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now / Self::EPOCH_DURATION
    }

    /// Returns the current slot number
    pub fn current_slot(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now / Self::SLOT_DURATION
    }

    /// Returns the start time of a given slot
    pub fn slot_start_time(slot: u64) -> u64 {
        slot * Self::SLOT_DURATION
    }

    /// Returns the end time of a given slot
    pub fn slot_end_time(slot: u64) -> u64 {
        (slot + 1) * Self::SLOT_DURATION
    }

    /// Returns the number of slots in an epoch
    pub fn slots_per_epoch() -> u64 {
        Self::EPOCH_DURATION / Self::SLOT_DURATION
    }

    /// Returns the epoch for a given slot
    pub fn epoch_for_slot(slot: u64) -> u64 {
        slot / Self::slots_per_epoch()
    }
}
