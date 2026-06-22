# Centichain Technical Whitepaper

**"Speed through Population, Not Power"**

*Version 3.0 — June 2026*

---

## 1. Abstract

The blockchain trilemma states that decentralization, security, and scalability are hard to maximize together. Vertical scaling (high-end datacenter hardware) improves throughput but centralizes validation.

**Centichain** targets horizontal scaling: many home-grade validators, each handling a bounded shard workload (~1,500 TPS per shard). Aggregate throughput grows linearly with validator count.

**Design goals:**

- Fastest practical L1 throughput via sharding
- Strong value-storage security via cryptographic validation and Proof of Patience (PoP)
- Maximum decentralization via low-cost home validators (~$2–5/month electricity)
- Simplest client experience: wallet-only users do not run full nodes

> **Implementation status (June 2026):** Core prototype is functional (P2P, mining loop, UI, tokenomics). Security hardening, true VDF, cross-shard finality, and mainnet readiness are **in progress**. See [Roadmap](#8-roadmap) and [docs/MASTER_VISION_AND_TECHNOLOGY_FA.md](docs/MASTER_VISION_AND_TECHNOLOGY_FA.md).

---

## 2. Core Philosophy

### 2.1 Hardware Neutrality

A 4-core CPU, 8 GB RAM, and home internet must be enough to run a **pruned validator**. Entry uses a **memory-hard** delay function (RAM-latency bound) to limit ASIC advantage.

### 2.2 Proof of Patience (PoP)

New validators prove commitment with:

1. One-time VDF solve (challenge = `SHA256(peer_id || "Patience")`)
2. Quarantine wait (5 minutes solo → up to 72 hours on large networks)
3. Deterministic leader rotation after activation — **no continuous PoW race**

Time cannot be bought with capital alone (unlike large PoS stake requirements).

### 2.3 Linear Scalability (CHSP)

```
active_shards = max(1, validators / 50)
global_TPS    ≈ active_shards × 1,500
```

| Validators | Shards | Theoretical TPS |
|-----------|--------|-----------------|
| 50 | 1 | 1,500 |
| 500 | 10 | 15,000 |
| 5,000 | 100 | 150,000 |
| 100,000 | 2,000 | 3,000,000 |

---

## 3. Consensus: AHSP

**Adaptive Hierarchical Sharded Proof-of-Patience**

| Component | Description |
|-----------|-------------|
| **PoP** | Sybil-resistant validator onboarding |
| **Leader election** | Deterministic per slot: `SHA256(shard, epoch, slot) % eligible` |
| **Sharding** | `shard = SHA256(peer_id, epoch) % active_shards` |
| **Trust & slashing** | Missed slots reduce trust; deactivation below threshold |
| **Slot / Epoch** | 2 s slots, 600 s (10 min) epochs |

### 3.1 Validator lifecycle

```
Join → Solve VDF → Quarantine → Activate → Queue / Leader
```

Once activated, a validator keeps eligibility (grandfather clause) unless slashed.

### 3.2 Block production rules

- Block time target: **2 seconds**
- Max transactions per block: **3,000** (1,500 TPS per shard)
- Leader waits ~1 s into slot for gossip before producing
- Block reward + fees paid via SYSTEM coinbase transaction

---

## 4. Architecture

### 4.1 Layers

```
UI (Tauri + React) → Rust Core (consensus, chain, wallet)
                  → libp2p (gossip, DHT, relay, sync)
                  → ReDB (blocks, state, mempool)
```

### 4.2 Network (libp2p)

- **Gossipsub:** blocks, transactions, receipts, VDF proofs
- **Kademlia + mDNS:** peer discovery
- **Relay v2 + DCUtR:** NAT traversal for home nodes
- **Request-response:** chain sync (`GetHeight`, `GetBlocksRange`)

### 4.3 Node tiers

| Tier | Role | Typical hardware |
|------|------|------------------|
| Wallet-only | Sign & broadcast via RPC | Phone / browser |
| Light client *(planned)* | Header + SPV verify | Mobile |
| Pruned validator | PoP + block production | Laptop, 4 GB RAM |
| Full validator | Full history | 8 GB RAM, larger SSD |
| RPC node | P2P + HTTP API | Server or desktop |
| Relay node | NAT helper only | Low-cost VPS |

### 4.4 Beacon layer *(planned)*

Coordinator for global validator registry, cross-shard **CrossLink** aggregation, and finality. Structs exist in code; full protocol not yet implemented.

### 4.5 Cross-shard atomicity *(planned)*

Three-phase receipt protocol:

1. **Burn** on source shard → `PendingReceipt`
2. **Mint** or **Revert** on target shard
3. **Rollback** on source if reverted

Receipt gossip exists; claim/revert handlers are roadmap items.

### 4.6 State & pruning

- Account balances in embedded ReDB
- Pruned validators keep recent blocks (`PRUNED_HISTORY_BLOCKS = 2000`)
- Target: stable disk usage for long-running home nodes
- Target: Merkle Patricia Trie `state_root` per block

---

## 5. Cryptography

| Use | Algorithm |
|-----|-----------|
| Transaction & peer identity | Ed25519 |
| Hashing | SHA-256 |
| Block / Merkle root | SHA-256 pairwise tree |
| Transport | Noise (libp2p) |
| PoP entry | Memory-hard VDF *(Argon2id / class-group VDF planned)* |

**Current gap:** Transaction signatures and full block validation must be enforced before mainnet (Phase 1).

---

## 6. Tokenomics (AGT)

| Parameter | Value |
|-----------|-------|
| Max supply | 21,000,000 AGT |
| Decimals | 6 (1 AGT = 1,000,000 units) |
| Genesis allocation | 5,000,000 AGT (block 0) |
| Initial block reward | ~0.127 AGT (`INITIAL_REWARD` base units) |
| Halving interval | ~4 years (63,072,000 blocks @ 2 s) |
| Transaction fee | max(0.001 AGT, 0.01% of amount) |

Halving follows a Bitcoin-style schedule: `reward = INITIAL_REWARD >> halving_count`.

---

## 7. Performance model

```
TPS_per_shard = MAX_TXS_PER_BLOCK / TARGET_BLOCK_TIME = 3000 / 2 = 1500
```

`bench_sharding` measures shard **assignment** speed only, not live transaction throughput. Real TPS benchmarks are a Phase 3 deliverable.

---

## 8. Roadmap

| Phase | Focus | Status |
|-------|-------|--------|
| **0 — Foundation** | Tauri app, libp2p, mining loop, mempool, UI | ✅ Prototype |
| **1 — Security** | Ed25519 tx signing, `validate_block`, fork choice, persist consensus | 🔲 Next |
| **2 — PoP + Testnet** | Real VDF (fast verify), quarantine enforcement, public relays | 🔲 Planned |
| **3 — Scale** | Per-shard state, cross-shard receipts, MPT, Block-STM | 🔲 Planned |
| **4 — Mainnet** | Genesis ceremony, light client, audit, battle-testing | 🔲 Planned |

Detailed tasks, technology references, and gap analysis:  
**[docs/MASTER_VISION_AND_TECHNOLOGY_FA.md](docs/MASTER_VISION_AND_TECHNOLOGY_FA.md)**

---

## 9. Comparison

| Feature | Centichain (target) | Bitcoin | Ethereum 2.0 | Solana |
|---------|---------------------|---------|--------------|--------|
| Scaling | Horizontal shards | L2 | L2 + rollup | Vertical HW |
| Validator cost | Low (home PC) | High (ASIC) | Stake + server | High-end server |
| Client simplicity | Wallet-only OK | Heavy full node | Light via RPC | RPC-heavy |
| Supply cap | 21M AGT | 21M BTC | Uncapped ETH | Uncapped SOL |
| ASIC resistance | Memory-hard PoP | No (ASIC wins) | N/A | N/A |

---

## 10. References

- Nakamoto — [Bitcoin Whitepaper](https://bitcoin.org/bitcoin.pdf)
- Boneh et al. — [Verifiable Delay Functions](https://eprint.iacr.org/2018/623)
- Gelashvili et al. — [Block-STM (arXiv:2203.06871)](https://arxiv.org/abs/2203.06871)
- libp2p — [github.com/libp2p/specs](https://github.com/libp2p/specs)
- RFC 8032 (Ed25519), RFC 9106 (Argon2)

---

*Centichain — built for decentralized value storage at home-validator scale.*
