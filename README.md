# Centichain

Layer-1 blockchain for **value storage** with horizontal scaling, **Proof of Patience (PoP)** consensus, and **home-grade validators** (~$2–5/month to run a node).

Built with **Rust + Tauri + React + libp2p**.

## Features (target design)

- **AHSP consensus** — PoP onboarding + round-robin leaders + dynamic sharding
- **Low node cost** — laptop validator, no ASIC, no mandatory stake
- **Simple clients** — most users only need a wallet; validators are optional
- **21M AGT cap** — Bitcoin-style halving
- **libp2p mesh** — gossip, DHT, relay for NAT traversal

## Quick start

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) stable
- Windows: [VS Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

### Desktop app (validator + wallet)

```bash
npm install
npm run tauri dev
```

### RPC node (HTTP API for integrations)

```bash
cd src-tauri
cargo run --release --bin rpc_node
# API: http://localhost:3000/api/v1
# WebSocket: ws://localhost:3000/ws
```

### Relay server (NAT traversal for home nodes)

```bash
cd src-tauri
cargo run --release --bin relay_server
```

### Sharding benchmark (dev)

```bash
cd src-tauri
cargo run --release --bin bench_sharding
```

## Project layout

```
src-tauri/src/
  consensus/   # PoP, leadership, sharding, mempool, VDF
  network/     # libp2p P2P layer
  node/        # mining loop, relay, network init
  chain/       # block, transaction, receipt, merkle
  storage/     # ReDB persistence
  wallet/      # Ed25519 keys
  commands/    # Tauri IPC commands
src/           # React UI
docs/          # Technical reference (Persian)
```

## Documentation

| Document | Description |
|----------|-------------|
| [CENTICHAIN_WHITEPAPER.md](CENTICHAIN_WHITEPAPER.md) | English whitepaper (vision, consensus, roadmap) |
| [docs/MASTER_VISION_AND_TECHNOLOGY_FA.md](docs/MASTER_VISION_AND_TECHNOLOGY_FA.md) | Full technical reference (Persian) — architecture, every technology, phases, gaps |

## Tokenomics (summary)

| | |
|---|---|
| Token | AGT (6 decimals) |
| Max supply | 21,000,000 |
| Genesis | 5,000,000 AGT |
| Block time | 2 s |
| Fee | ≥ 0.001 AGT or 0.01% |

## Implementation status

**Prototype** — P2P, UI, mining, and tokenomics work locally.  
**Next (Phase 1)** — transaction signing, block validation, fork choice.

See whitepaper §8 and the master doc §15 for the full roadmap.

## License

Private / training project — see repository owner for terms.
