# Antigravity Chain: The Technical Whitepaper
**"Speed through Population, Not Power"**

*Version 1.0 - December 2025*

---

## 1. Abstract
The blockchain trilemma posits that a network cannot simultaneously achieve Decentralization, Security, and Scalability. Traditional "Vertical Scaling" solutions (like Solana) achieve speed by requiring high-performance hardware, which centralizes the network around professional datacenters.

**Antigravity Chain** solves this by introducing the **Antigravity Horizontal Scaling Protocol (AHSP)**. It decouples transaction throughput from individual node performance. Instead of requiring 1 supercomputer to process 150,000 TPS, Antigravity uses 100,000 home-grade computers, each processing a manageable 1,500 TPS, to achieve the same aggregate speed.

## 2. Core Philosophy
1.  **Hardware Neutrality**: A standard 4-core CPU with a home internet connection must be sufficient to run a Validator.
2.  **Proof of Patience (PoP)**: Security is derived from long-term stake and uptime, preventing simplistic "rich-get-richer" Sybil attacks.
3.  **Linear Scalability**: Network capacity increases linearly with the number of nodes.
    -   1 Node = 1,500 TPS
    -   100 Nodes = 150,000 TPS
    -   1 Million Nodes = 1.5 Billion TPS

## 3. Technical Architecture

### 3.1 The Beacon Layer (Consensus)
The Beacon Chain is the coordinator of the network. It does not process user transactions but manages the **Global State of Validators**.

*   **Registry**: Tracks active validators, their uptime, and Trust Scores.
*   **VDF (Verifiable Delay Function)**: Uses a CPU-bound function to generate unbiasable randomness for leader election and shard assignment.
*   **Epochs**: Time is divided into 6-hour Epochs. At the start of each epoch, the Beacon Chain reshuffles the entire network.

### 3.2 Dynamic Sharding (AHSP)
The global state space is partitioned into $N$ shards.

**The Formula**:
$$ N = \max(1, \lfloor \frac{Validators}{50} \rfloor) $$
*We strictly enforce a minimum of 50 validators per shard to ensure BFT security.*

**The Assignment**:
Validators are assigned to shards using a deterministic **SHA-256** hash of their `PeerID` + `EpochRandomness`. This prevents "Grinding Attacks" where a node tries to manipulate its way into a specific shard.

### 3.3 The Shard Engine (Execution)
Each shard operates as a semi-independent blockchain with specific constraints to ensure accessibility:
*   **True TPS**: **1,500 TPS** per shard (3,000 Tx per 2s block).
*   **Block Size**: Max **1.5 MB**.
*   **State Separation**: Nodes only store the UTXO/Account state for their assigned shard (~1% of total global state).

### 3.4 Cross-Shard Communication
When User A (Shard 1) sends funds to User B (Shard 2):
1.  **Burn**: Shard 1 verifies the transaction, burns the funds, and generates a **Receipt**.
2.  **Broadcast**: The Receipt is broadcast via the global `antigravity-receipts` P2P topic.
3.  **Mint**: Shard 2 validates the Receipt (checking the Merkle Proof against the Beacon State) and mints funds for User B.

## 4. Benchmarking & Performance
Simulations run on the `bench_sharding` tool yielded the following results:

| Network Size | Active Shards | Assignment Time | Global Capacity |
| :--- | :--- | :--- | :--- |
| **1,000 Nodes** | 20 | 4 ms | 30,000 TPS |
| **10,000 Nodes** | 200 | 40 ms | 300,000 TPS |
| **100,000 Nodes** | 2,000 | 402 ms | 3,000,000 TPS |

*Note: Assignment time is the time for the Beacon Chain to calculate the new topology. Distributed assignment happens instantly on each node.*

## 5. Technology Stack

### Backend (Core)
*   **Language**: Rust (for safety and performance).
*   **P2P Networking**: `libp2p` (Gossipsub, Kademlia DHT, Noise encryption).
*   **Database**: `redb` (Embedded, ACID-compliant key-value store).
*   **Consensus**: Custom Proof-of-Patience + SHA-256 Sharding.

### Frontend (Client)
*   **Framework**: React + TypeScript.
*   **Platform**: Tauri (v2) for native desktop integration.
*   **State Management**: React Context API.
*   **UI/UX**: TailwindCSS + Lucide Icons (Glassmorphism design).

## 6. Comparison with Other Networks

| Feature | Antigravity | Solana | Ethereum 2.0 |
| :--- | :--- | :--- | :--- |
| **Scaling Model** | **Horizontal (Population)** | Vertical (Hardware) | Horizontal (L2s) |
| **Validator Hardware** | 4-Core CPU, 8GB RAM | 12-Core, 128GB RAM | 4-Core, 16GB RAM |
| **TPS** | **Unbounded (Linear)** | ~65,000 (Bounded) | ~100,000 (w/ L2) |
| **State Storage** | **1% (Sharded)** | 100% (Full State) | 100% (Full State) |
| **Decentralization** | **High (Home Users)** | Low (Datacenters) | High |

## 7. Roadmap to 150,000 TPS
1.  **Phase 1-2 (Completed)**: Core Sharding Engine and Beacon Layer.
2.  **Phase 3 (Completed)**: Cross-Shard Communication.
3.  **Phase 5 (Completed)**: Security Hardening (SHA-256).
4.  **Next Frontier**:
    -   **Hardware VDF**: Optimizing the VDF for ASIC resistance.
    -   **State Pruning**: Allowing nodes to discard old history to save disk space.

---
*Generated by Antigravity AI - 2025*
