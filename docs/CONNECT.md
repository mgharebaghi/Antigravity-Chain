# Connecting to Antigravity Network (RPC/REST)

This guide explains how exchanges, wallets, and external services can connect to the **Antigravity Network** using the `rpc_node` binary.

## Overview

The Antigravity Network is a P2P blockchain. To interact with it from external systems (like a web server or exchange backend), you must run an **RPC Node**. This node acts as a bridge: it participates in the P2P network (gossipsub, DHT) and exposes a standard HTTP JSON-REST API for your application.

## 1. Running the RPC Node

The RPC node is a headless binary included in the source code.

### Prerequisites
- Windows/Linux/macOS
- Rust Toolchain installed (for building)
- Network access (P2P ports)

### Building and Running
Navigate to the `src-tauri` directory and run:

```bash
cd src-tauri
cargo run --release --bin rpc_node
```

The node will start and listen on:
- **API Port**: `3000` (HTTP & WebSocket)
- **P2P Port**: `9091` (TCP)

> **Note**: Ensure port `9091` is open if you want to peer with external nodes.

## 2. API Reference

All endpoints return JSON responses.

### Base URL: `http://localhost:3000/api/v1`

### Get Node Status
**GET** `/status`

Check if the node is synced and healthy.

**Response:**
```json
{
  "node_type": "RPC",
  "chain_height": 1205,
  "peer_count": 8,
  "network": "Antigravity Mainnet"
}
```

### Get Network Stats
**GET** `/network/stats`

Get detailed network statistics (Supply, Halving, Difficulty).

**Response:**
```json
{
  "supply": 5000000000000,
  "max_supply": 21000000000000,
  "circulating": 5000000000000,
  "halving_block": 100000,
  "current_reward": 40000000,
  "mining_difficulty": 200000
}
```

### Get Account Balance
**GET** `/balance/:address`

Get the *available* balance of a wallet address (Balance - Pending Spends).

**Example:**
`GET /balance/12D3KooWD...`

**Response:**
```json
{
  "address": "12D3KooWD...",
  "balance": 5000000000,
  "currency": "AGT"
}
```
*(Balance is in base units. 1 AGT = 1,000,000 units)*

### Get Blocks (Paginated)
**GET** `/blocks?page=0&limit=20`

Get a list of blocks.

**Response:**
```json
[
  { "index": 105, "hash": "...", "transactions": [...] },
  { "index": 104, "hash": "...", "transactions": [...] }
]
```

### Get Transaction
**GET** `/transactions/:id`

Get details of a transaction by its ID (UUID).

**Response:**
```json
{
  "transaction": {
    "id": "550e8400-e29b-...",
    "sender": "12D3...",
    "receiver": "12D3...",
    "amount": 1000000,
    "timestamp": 1678888888,
    "signature": "..."
  },
  "status": "confirmed", // or "pending"
  "block_index": 105,
  "block_hash": "a1b2..."
}
```

### Submit Transaction
**POST** `/broadcast`

Submit a signed transaction to the network.

**Payload:**
```json
{
  "transaction": {
    "id": "generated-uuid",
    "sender": "your-public-key-id",
    "receiver": "dest-public-key-id",
    "amount": 1000000,
    "timestamp": 1234567890,
    "signature": "hex-encoded-signature"
  }
}
```

**Response:**
```json
{
  "status": "accepted",
  "tx_id": "generated-uuid"
}
```

## 3. Real-time Updates (WebSocket)

Connect to the WebSocket endpoint to receive real-time events.

**URL**: `ws://localhost:3000/ws`

### Events

#### New Block Mined
Sent when a new valid block is added to the chain.
```json
{
  "type": "NewBlock",
  "data": {
    "index": 106,
    "hash": "...",
    "transactions": [...]
  }
}
```

#### New Transaction
Sent when a new valid transaction enters the mempool.
```json
{
  "type": "NewTransaction",
  "data": {
    "id": "...",
    "sender": "...",
    "amount": 1000000
  }
}
```

## 4. Integration Best Practices

- **Syncing**: When you first start the `rpc_node`, allow it a few minutes to discover peers and sync the blockchain before relying on balance data.
- **Failover**: You can run multiple instances of `rpc_node` behind a Load Balancer (Nginx) for high availability.
- **Security**: The RPC API does **not** generate keys or sign transactions. You must manage private keys securely in your own application ("Cold Wallet" approach) and only send signed transaction objects to the node.

---
*Antigravity Development Team*
