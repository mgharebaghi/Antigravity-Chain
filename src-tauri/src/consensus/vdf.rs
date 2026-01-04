use sha2::{Digest, Sha256};
use std::fmt::Write;

pub struct CentichainVDF {
    pub difficulty: u64,
}

impl CentichainVDF {
    pub fn new(difficulty: u64) -> Self {
        CentichainVDF { difficulty }
    }

    pub fn solve(&self, challenge: &[u8]) -> String {
        // Memory-Hard VDF (Simplified Scrypt-like approach)
        // 1. Initialize a large buffer (e.g., 16MB for demo, 64-128MB for prod)
        // We use a smaller buffer for dev efficiency but structure allows scaling.
        const BUFFER_SIZE: usize = 16 * 1024 * 1024; // 16 MB
        let mut buffer = vec![0u8; BUFFER_SIZE];

        // 2. Fill buffer deterministically derived from challenge
        let mut hasher = Sha256::new();
        hasher.update(challenge);
        let seed = hasher.finalize();

        let mut fill_rng = Sha256::new();
        fill_rng.update(seed);
        let mut fill_stream = fill_rng.finalize(); // Initial 32 bytes

        // Fill buffer in 32-byte chunks (pseudo-randomly)
        for i in (0..BUFFER_SIZE).step_by(32) {
            fill_rng = Sha256::new();
            fill_rng.update(fill_stream);
            fill_stream = fill_rng.finalize();
            let chunk_size = std::cmp::min(32, BUFFER_SIZE - i);
            buffer[i..i + chunk_size].copy_from_slice(&fill_stream[0..chunk_size]);
        }

        // 3. Perform Random Memory Accesses (The "Work")
        // Number of iterations = difficulty.
        // Each iteration reads from a random location and writes to another.
        // This forces CPU to wait for RAM (latency bound).

        let mut pointer: usize = 0;
        // Use a u32 from seed to pick indices
        let mut index_gen = u32::from_le_bytes(seed[0..4].try_into().unwrap());

        for _ in 0..self.difficulty {
            // Pick a random reading location dependent on current state
            // LCG for speed: x = (a * x + c) % m
            index_gen = index_gen.wrapping_mul(1664525).wrapping_add(1013904223);
            let read_index = (index_gen as usize) % BUFFER_SIZE;

            // Read value
            let val = buffer[read_index];

            // Modify current pointer location
            buffer[pointer] = buffer[pointer].wrapping_add(val).wrapping_mul(3);

            // Move pointer
            pointer = (pointer + 1) % BUFFER_SIZE;

            // Periodically re-hash a chunk to chain dependencies
            // (Skipping for pure speed in this demo, relying on dependency chain of 'buffer' values)
        }

        // 4. Final Hash of the entire buffer (or a sample)
        let mut final_hasher = Sha256::new();
        final_hasher.update(&buffer);
        let result = final_hasher.finalize();

        // Convert to hex string
        let mut s = String::with_capacity(2 * result.len());
        for byte in result {
            write!(&mut s, "{:02x}", byte).unwrap();
        }
        s
    }

    pub fn verify(&self, challenge: &[u8], proof: &str) -> bool {
        let calculated = self.solve(challenge);
        calculated == proof
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vdf_solve_verify() {
        let vdf = CentichainVDF::new(100);
        let challenge = b"test_challenge";
        let proof = vdf.solve(challenge);
        assert!(vdf.verify(challenge, &proof));
        assert!(!vdf.verify(b"wrong_challenge", &proof));
    }
}
