use sha2::{Digest, Sha256};
use std::fmt::Write;

pub struct AntigravityVDF {
    pub difficulty: u64,
}

impl AntigravityVDF {
    pub fn new(difficulty: u64) -> Self {
        AntigravityVDF { difficulty }
    }

    pub fn solve(&self, challenge: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(challenge);
        let mut result = hasher.finalize();

        for _ in 0..self.difficulty {
            let mut h = Sha256::new();
            h.update(result);
            result = h.finalize();
        }

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
        let vdf = AntigravityVDF::new(100);
        let challenge = b"test_challenge";
        let proof = vdf.solve(challenge);
        assert!(vdf.verify(challenge, &proof));
        assert!(!vdf.verify(b"wrong_challenge", &proof));
    }
}
