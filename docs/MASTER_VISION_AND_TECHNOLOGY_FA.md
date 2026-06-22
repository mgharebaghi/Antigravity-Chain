# Centichain / Antigravity Chain
# سند جامع چشم‌انداز، معماری و فناوری‌ها

**نسخه:** 1.0  
**تاریخ:** ژوئن ۲۰۲۶  
**زبان:** فارسی  
**مخاطب:** تیم توسعه، مشارکت‌کنندگان، و تصمیم‌گیرندگان فنی  

**هدف رسمی پروژه:**

> ساخت **سریع‌ترین**، **امن‌ترین** و **غیرمتمرکزترین** شبکه بلاکچینی جهان برای **ذخیره ارزش**، با **ساده‌ترین پیاده‌سازی ممکن برای نود/کلاینت خانگی**.

---

## فهرست مطالب

1. [خلاصه اجرایی](#۱-خلاصه-اجرایی)
2. [فلسفه طراحی و پاسخ به تریلمای بلاکچین](#۲-فلسفه-طراحی-و-پاسخ-به-تریلمای-بلاکچین)
3. [معماری کلان سیستم](#۳-معماری-کلان-سیستم)
4. [فناوری‌های هسته — توضیح کامل + رفرنس](#۴-فناوری‌های-هسته--توضیح-کامل--رفرنس)
5. [اجماع AHSP و Proof of Patience (PoP)](#۵-اجماع-ahsp-و-proof-of-patience-pop)
6. [شاردینگ افقی CHSP](#۶-شاردینگ-افقی-chsp)
7. [اتمی بودن بین‌شاردی](#۷-اتمی-بودن-بین‌شاردی)
8. [شبکه P2P و سادگی اجرای نود](#۸-شبکه-p2p-و-سادگی-اجرای-نود)
9. [رمزنگاری و مدل امنیتی](#۹-رمزنگاری-و-مدل-امنیتی)
10. [ذخیره‌سازی، State و Pruning](#۱۰-ذخیره‌سازی-state-و-pruning)
11. [اجرای موازی تراکنش‌ها](#۱۱-اجرای-موازی-تراکنش‌ها)
12. [توکنومیکس AGT](#۱۲-توکنومیکس-agt)
13. [انواع نود و سادگی کلاینت](#۱۳-انواع-نود-و-سادگی-کلاینت)
14. [اهداف عملکردی و مدل TPS](#۱۴-اهداف-عملکردی-و-مدل-tps)
15. [نقشه راه پیاده‌سازی (فازبندی)](#۱۵-نقشه-راه-پیاده‌سازی-فازبندی)
16. [مقایسه با شبکه‌های موجود](#۱۶-مقایسه-با-شبکه‌های-موجود)
17. [نگاشت به کدبیس فعلی و شکاف‌ها](#۱۷-نگاشت-به-کدبیس-فعلی-و-شکاف‌ها)
18. [مراجع و منابع](#۱۸-مراجع-و-منابع)
19. [واژه‌نامه](#۱۹-واژه‌نامه)

---

## ۱. خلاصه اجرایی

**Centichain** (در برخی اسناد: **Antigravity Chain**) یک بلاکچین **لایه ۱ (L1)** است که با تمرکز بر سه محور طراحی شده:

| محور | راهبرد |
|------|--------|
| **سرعت** | مقیاس‌پذیری افقی: هر نود validator سهمی معقول (~۱٬۵۰۰ TPS per shard) دارد؛ با رشد جمعیت نودها، TPS کل شبکه خطی رشد می‌کند |
| **امنیت** | PoP (ورود سخت + زمان صبر) + انتخاب رهبر deterministic + اعتبارسنجی رمزنگاری کامل + slashing |
| **غیرمتمرکزسازی** | نود روی سخت‌افزار خانگی (۴ هسته CPU، ۸GB RAM) — بدون ASIC، بدون stake اجباری، بدون دیتاسنتر |
| **سادگی کلاینت** | ۹۹٪ کاربران فقط wallet/light client؛ validator اختیاری و ارزان (~۲–۵ دلار/ماه برق) |

**شعار طراحی:** *«Speed through Population, Not Power»* — سرعت از جمعیت نودها، نه از قدرت سخت‌افزار یک دیتاسنتر.

**پشته فنی:**

- **هسته:** Rust + Tokio (async)
- **اپ دسکتاپ:** Tauri v2 + React + TypeScript
- **شبکه:** libp2p (Gossipsub, Kademlia, Relay, DCUtR)
- **دیتابیس:** ReDB (embedded KV)
- **اجماع:** AHSP = Adaptive Hierarchical Sharded Proof-of-Patience

**اسناد مرتبط در همین مخزن:**

- [README.md](../README.md) — راه‌اندازی سریع و ساختار پروژه
- [CENTICHAIN_WHITEPAPER.md](../CENTICHAIN_WHITEPAPER.md) — وایت‌پیپر انگلیسی (چشم‌انداز و roadmap)

---

## ۲. فلسفه طراحی و پاسخ به تریلمای بلاکچین

### ۲.۱ تریلمای بلاکچین (Blockchain Trilemma)

مفهوم مطرح‌شده توسط **Vitalik Buterin**: یک سیستم توزیع‌شده نمی‌تواند همزمان سه ویژگی را به حداکثر برساند:

1. **Decentralization** — تعداد زیاد نود مستقل
2. **Security** — مقاومت در برابر حملات (۵۱٪، Sybil، double-spend)
3. **Scalability** — throughput بالا

**رفرنس:** [Ethereum.org — Blockchain Trilemma](https://ethereum.org/en/roadmap/vision/#the-scalability-trilemma)

### ۲.۲ راهبرد Centichain

به‌جای **Vertical Scaling** (یک سوپرکامپیوتر سریع — مدل Solana)، **Horizontal Scaling** انتخاب شده:

```
TPS کل شبکه ≈ تعداد_shard × 1,500 TPS
تعداد_shard = max(1, validators / 50)
```

**مثال:**

| Validators | Shards | TPS تئوری |
|-----------|--------|-----------|
| 50 | 1 | 1,500 |
| 500 | 10 | 15,000 |
| 5,000 | 100 | 150,000 |
| 100,000 | 2,000 | 3,000,000 |

### ۲.۳ اصل Hardware Neutrality (پروتکل عدالت)

- VDF **Memory-Hard** (وابسته به latency حافظه RAM، نه فقط CPU)
- ASIC نمی‌تواند به‌طور نامتناسب سریع‌تر شود
- هر لپ‌تاپ ۴ هسته‌ای با اینترنت خانگی باید بتواند validator باشد

**رفرنس الهام‌بخش:**

- **Argon2** — Password Hashing Competition winner, memory-hard  
  [RFC 9106 — Argon2](https://www.rfc-editor.org/rfc/rfc9106.html)
- **Ethash** (Ethereum legacy PoW) — memory-hard mining  
  [Yellow Paper — Ethereum](https://ethereum.github.io/yellowpaper/paper.pdf)

### ۲.۴ اصل Proof of Patience (اثبات صبر)

- **زمان** را نمی‌توان با پول زیاد «خرید» (برخلاف stake سنگین)
- نود جدید باید VDF حل کند + دوره quarantine بگذراند
- پس از فعال‌سازی، در چرخش رهبری شرکت می‌کند — **بدون رقابت مداوم PoW**

**رفرنس مفهومی:**

- **Verifiable Delay Functions (VDF)** — Boneh et al.  
  [VDF Survey (ePrint 2018/623)](https://eprint.iacr.org/2018/623)
- **Proof of Space/Time** (Chia) — ایده «زمان به‌عنوان منبع کمیاب»  
  [Chia Green Paper](https://www.chia.net/whitepaper/)

### ۲.۵ اصل سادگی برای کلاینت

کاربر نهایی **نباید** نود کامل اجرا کند. سلسله‌مراتب:

```
Wallet-only  →  Light Client  →  Pruned Validator  →  Full Validator
   (ساده‌ترین)                                              (کامل‌ترین)
```

---

## ۳. معماری کلان سیستم

### ۳.۱ نمای لایه‌ای

```
┌─────────────────────────────────────────────────────────────┐
│  لایه ۴: کاربر (Tauri App / Mobile Wallet / Web RPC)      │
├─────────────────────────────────────────────────────────────┤
│  لایه ۳: API (Tauri IPC / rpc_node HTTP+WebSocket)          │
├─────────────────────────────────────────────────────────────┤
│  لایه ۲: اجماع + اجرا (AHSP, Sharding, Mempool, VDF)       │
├─────────────────────────────────────────────────────────────┤
│  لایه ۱: شبکه (libp2p — Gossip, DHT, Relay, Sync)          │
├─────────────────────────────────────────────────────────────┤
│  لایه ۰: ذخیره‌سازی (ReDB — Blocks, State, Mempool, Keys)   │
└─────────────────────────────────────────────────────────────┘
```

### ۳.۲ اجزای منطقی

| جزء | مسئولیت | مسیر کد (فعلی) |
|-----|---------|----------------|
| **Mining Loop** | تولید بلاک، انتخاب تراکنش، VDF بلاک | `src-tauri/src/node/mining.rs` |
| **P2P Node** | gossip، sync، discovery | `src-tauri/src/network/p2p.rs` |
| **Consensus** | PoP، leadership، sharding | `src-tauri/src/consensus/` |
| **Chain** | Block, Tx, Receipt, Merkle | `src-tauri/src/chain/` |
| **Storage** | persistence | `src-tauri/src/storage/mod.rs` |
| **Wallet** | کلید Ed25519، امضا | `src-tauri/src/wallet/mod.rs` |
| **UI** | اکسپلورر، wallet، dashboard | `src/` |

### ۳.۳ State Machine نود

```
Stopped → Connecting (Relay) → Discovering (DHT/mDNS)
    → [بدون همسایه] Genesis Creation
    → [با همسایه] Synchronizing → Grace Period → Active
         → Patience Mode → Queue → Leader (Mining)
```

**رفرنس:** بخش ۳.۳ همین سند — State Machine

### ۳.۴ جریان داده تراکنش

```
کاربر UI → submit_transaction → Mempool (اعتبارسنجی محلی)
    → Gossipsub (shard_txs topic) → نودهای دیگر
    → Leader slot → Block → Gossipsub (shard_blocks)
    → Storage (state update) → UI event
```

---

## ۴. فناوری‌های هسته — توضیح کامل + رفرنس

### ۴.۱ Rust

**چیست:** زبان سیستمی با memory safety بدون garbage collector.

**چرا در Centichain:**

| مزیت | توضیح |
|------|--------|
| Memory Safety | جلوگیری از buffer overflow و use-after-free در کد consensus |
| Performance | سرعت نزدیک C/C++ — مناسب پردازش هزاران تراکنش |
| Concurrency | مدل ownership + Tokio برای هزاران اتصال P2P همزمان |
| Ecosystem | crateهای بالغ: libp2p، sha2، ed25519 |

**رفرنس:**

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust Async Book — Tokio](https://tokio.rs/tokio/tutorial)

**نسخه در پروژه:** Edition 2021 (`src-tauri/Cargo.toml`)

---

### ۴.۲ Tokio

**چیست:** runtime آسنکرون برای Rust (event loop، task scheduling، I/O non-blocking).

**کاربرد در Centichain:**

- حلقه اصلی P2P (`tokio::select!` برای gossip، sync، timer)
- Mining loop بدون block کردن شبکه
- کانال‌های mpsc برای block/tx/receipt broadcast

**رفرنس:** [Tokio Documentation](https://docs.rs/tokio/latest/tokio/)

**الگوی استفاده:**

```rust
// p2p.rs — چند منبع رویداد همزمان
tokio::select! {
    Some(cmd) = cmd_rx.recv() => { /* ... */ }
    event = swarm.select_next_some() => { /* ... */ }
    Some(block) = block_receiver.recv() => { /* broadcast */ }
}
```

---

### ۴.۳ Tauri v2

**چیست:** فریمورک اپ دسکتاپ — WebView برای UI + Rust برای backend native.

**چرا به‌جای Electron:**

| معیار | Electron | Tauri |
|-------|----------|-------|
| حافظه | ~۱۵۰MB+ | ~۳۰–۵۰MB |
| باینری | بزرگ (Chromium bundled) | کوچک‌تر |
| امنیت | Node در frontend | منطق حساس فقط در Rust |

**کاربرد:** اپ Centichain — wallet، explorer، network map، VDF visualizer.

**رفرنس:**

- [Tauri v2 Docs](https://v2.tauri.app/)
- [Tauri Security](https://v2.tauri.app/security/)

**ارتباط UI ↔ Rust:** Tauri Commands (`invoke`) — JSON serialization

```rust
// lib.rs — نمونه commandها
commands::chain::submit_transaction,
commands::node::start_node,
```

---

### ۴.۴ React + TypeScript + Vite + Tailwind

| فناوری | نقش |
|--------|-----|
| **React 19** | UI components، state، pages |
| **TypeScript** | type safety برای داده‌های دریافتی از Rust |
| **Vite** | dev server سریع، HMR |
| **TailwindCSS** | استایل utility-first |
| **Framer Motion** | انیمیشن (PageTransition، Welcome) |

**اصل امنیتی:** هیچ کلید خصوصی یا منطق consensus در frontend اجرا **نمی‌شود** — فقط نمایش و فراخوانی command.

**رفرنس:**

- [React Docs](https://react.dev/)
- [Vite Guide](https://vite.dev/guide/)

---

### ۴.۵ libp2p

**چیست:** stack شبکه P2P ماژولار — استاندارد de facto برای بلاکچین‌های جدید (IPFS، Filecoin، Polkadot ecosystem).

**رفرنس اصلی:** [libp2p Specification](https://github.com/libp2p/specs)

#### ۴.۵.۱ پروتکل‌های استفاده‌شده در Centichain

| پروتکل | کاربرد | توضیح |
|--------|--------|-------|
| **TCP + Noise** | Transport + Encryption | اتصال رمزنگاری‌شده بین peers |
| **Yamux** | Stream multiplexing | چند stream روی یک TCP |
| **Gossipsub** | Pub/Sub | پخش بلاک، تراکنش، receipt، topology |
| **Kademlia DHT** | Peer discovery | پیدا کردن نودها در شبکه بزرگ |
| **mDNS** | Local discovery | کشف نود در LAN (توسعه/خانگی) |
| **Relay v2** | NAT traversal | نود پشت روتر بدون IP عمومی |
| **DCUtR** | Direct connection upgrade | upgrade از relay به اتصال مستقیم |
| **Identify** | Protocol handshake | exchange listen addrs |
| **Ping** | Keepalive | سلامت اتصال |
| **Request-Response (CBOR)** | Sync | `GetHeight`, `GetBlocksRange`, `GetHeaders` |

**پیاده‌سازی:** `src-tauri/src/network/behaviour.rs` — struct `CentichainBehaviour`

**Gossip Topics (فعلی):**

```
centichain-shard-blocks
centichain-shard-txs
centichain-receipts
centichain-vdf-proofs
centichain-topology
centichain-status
```

**رفرنس‌های تفصیلی:**

- [Gossipsub v1.1](https://github.com/libp2p/specs/blob/master/pubsub/gossipsub/gossipsub-v1.1.md)
- [Kademlia Paper — Maymounkov & Mazières](https://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf)
- [Relay v2 Spec](https://github.com/libp2p/specs/blob/master/relay/README.md)
- [DCUtR Spec](https://github.com/libp2p/specs/blob/master/relay/DCUtR.md)

#### ۴.۵.۲ چرا Relay برای سادگی نود خانگی حیاتی است

بیشتر کاربران ISP خانگی **NAT/CGNAT** دارند — بدون IP عمومی نمی‌توانند peer شوند.

```
نود خانگی ←→ Relay (VPS عمومی) ←→ نودهای دیگر
         DCUtR → اتصال مستقیم (در صورت امکان)
```

**هزینه نود خانگی:** صفر برای relay — relay توسط داوطلبان/جامعه (~۵$/ماه VPS)  
**هزینه validator:** فقط برق لپ‌تاپ

**ثابت‌های فعلی:** `src-tauri/src/utils/constants.rs` — `RELAY_ADDRESSES` (فعلاً localhost برای dev)

---

### ۴.۶ ReDB

**چیست:** embedded key-value database برای Rust — ACID، بدون سرور جداگانه.

**چرا به‌جای PostgreSQL/RocksDB:**

- نصب صفر — فایل `.db` محلی
- latency بسیار پایین برای read/write بلاک
- مناسب اپ دسکتاپ تک‌کاربره

**جداول فعلی:**

| Table | Key | Value |
|-------|-----|-------|
| `blocks` | u64 (index) | JSON Block |
| `state` | address | balance (u64) |
| `mempool` | tx_id | JSON Transaction |
| `wallet` | "main_key" | keypair JSON |
| `settings` | key | JSON settings |

**رفرنس:** [redb crate](https://docs.rs/redb/latest/redb/)

**هدف آینده:** جداول per-shard state، tx index، consensus persistence

---

### ۴.۷ Serde + bincode + JSON

| Crate | کاربرد |
|-------|--------|
| **serde** | serialize/deserialize structها |
| **serde_json** | ذخیره بلاک در DB، API responses |
| **bincode** | (آماده) serialization فشرده برای شبکه |

**رفرنس:** [Serde Book](https://serde.rs/)

---

### ۴.۸ SHA-256

**چیست:** تابع hash رمزنگاری — خروجی ۲۵۶ بیت، مقاوم در برابر preimage و collision (با فرض امنیت فعلی).

**کاربرد در Centichain:**

- hash بلاک (`block.calculate_hash()`)
- Merkle tree
- shard assignment: `SHA256(peer_id || epoch)`
- leader election randomness: `SHA256(shard || epoch || slot)`

**رفرنس:** [FIPS 180-4 — SHA-256](https://csrc.nist.gov/publications/detail/fips/180/4/final)

**crate:** `sha2` — [docs.rs/sha2](https://docs.rs/sha2/latest/sha2/)

---

### ۴.۹ Ed25519

**چیست:** امضای دیجیتال روی منحنی Edwards25519 — سریع، کوتاه (۶۴ بایت signature)، امن.

**کاربرد هدف:**

- امضای تراکنش
- هویت peer (libp2p Keypair)
- امضای CrossLink (آینده)

**رفرنس:**

- [RFC 8032 — Edwards-Curve Digital Signature Algorithm](https://www.rfc-editor.org/rfc/rfc8032)
- [ed25519-dalek](https://docs.rs/ed25519-dalek/latest/ed25519_dalek/) (از طریق libp2p identity)

**وضعیت فعلی:** Keypair در wallet هست؛ امضای تراکنش هنوز placeholder (`"sig"`) — **باید در فاز ۱ تکمیل شود**.

---

### ۴.۱۰ Axum (rpc_node)

**چیست:** فریمورک HTTP async برای Rust.

**کاربرد:** باینری `rpc_node` — REST API + WebSocket برای صرافی‌ها و سرویس‌های خارجی.

**Endpoints:** `/api/v1/status`, `/balance/:address`, `/broadcast`, `/blocks`, `/ws`

**رفرنس:** [Axum Docs](https://docs.rs/axum/latest/axum/)

**سند اتصال:** پیوست ج — RPC API در همین سند

---

## ۵. اجماع AHSP و Proof of Patience (PoP)

### ۵.۱ AHSP چیست؟

**Adaptive Hierarchical Sharded Proof-of-Patience** — ترکیب:

1. **PoP** — ورود سخت به validator set
2. **Round-Robin / Weighted Leader Election** — تولید بلاک بدون رقابت PoW
3. **Dynamic Sharding** — رشد shard با رشد validators
4. **Trust Score + Slashing** — تنبیه رفتار بد

**مسیر کد:** `src-tauri/src/consensus/`

### ۵.۲ Proof of Patience — مکانیزم کامل

#### مرحله ۱: پیوستن به شبکه

```
نود جدید → register در consensus → solve VDF(challenge)
challenge = SHA256(peer_id || "Patience")
→ broadcast VdfProofMessage روی gossip
→ peers دیگر verify می‌کنند → is_verified = true
```

**کد:** `consensus/sharding.rs` — `get_vdf_challenge()`  
**کد:** `node/vdf.rs` — heartbeat loop

#### مرحله ۲: Quarantine (دوره صبر)

```
duration = f(validator_count)
  - solo (1 node): 300 ثانیه (۵ دقیقه)
  - network: min(300 + validators×3600, 72×3600)
```

**منطق:** هر چه شبکه بزرگ‌تر، ورود Sybil سخت‌تر.

**کد:** `consensus/mod.rs` — `get_quarantine_duration()`

#### مرحله ۳: Activation

پس از `uptime >= quarantine` و `is_verified`:

```
node.activate() → activated_at = now → is_active = true
```

**Grandfather Clause:** نود فعال‌شده حتی اگر quarantine بعداً طولانی‌تر شود، eligible می‌ماند.

**کد:** `consensus/node_state.rs` — `is_permanently_eligible()`

#### مرحله ۴: Leader Election

```
eligible = validators در shard با:
  - mining_active = true
  - activated_at.is_some() OR (verified + quarantine done)
  - trust_score >= 0.01

leader(slot) = eligible[ SHA256(shard, epoch, slot) % len(eligible) ]
```

**کد:** `consensus/leadership.rs` — `get_shard_leader()`

**Slot/Epoch:**

| پارامتر | مقدار | ثابت |
|---------|-------|------|
| Slot duration | ۲ ثانیه | `SLOT_DURATION` |
| Epoch duration | ۶۰۰ ثانیه (۱۰ دقیقه) | `EPOCH_DURATION` |

### ۵.۳ VDF — وضعیت فعلی vs هدف

#### وضعیت فعلی (MVP)

```rust
// consensus/vdf.rs
// Memory buffer 16MB + difficulty iterations
// verify() = solve() دوباره ← NOT a true VDF
```

**محدودیت:** verify باید **خیلی سریع‌تر** از solve باشد.

#### هدف Production

| گزینه | verify | solve | رفرنس |
|-------|--------|-------|-------|
| **Argon2id** | میلی‌ثانیه | دقیقه | [RFC 9106](https://www.rfc-editor.org/rfc/rfc9106.html) |
| **Class-group VDF** | میلی‌ثانیه (Wesolowski proof) | ثانیه–دقیقه | [Boneh et al. 2018](https://eprint.iacr.org/2018/623) |
| **Chia VDF** | سریع | کند | [Chia consensus](https://docs.chia.net/consensus-intro/) |

**پارامترهای هدف (از constants):**

```
VDF_BASE_DIFFICULTY = 3_000_000 iterations
VDF_DIFFICULTY_PER_VALIDATOR = 500_000
Buffer RAM = 64–128 MB (memory-hard)
```

### ۵.۴ Slashing

```
missed slot as leader → trust_score *= 0.5
trust_score < 0.01 → deactivate, activated_at = None
```

**کد:** `consensus/mod.rs` — `slash_node()`, `slash_missed_slots()`

**هدف آینده:** slashing on-chain با proof قابل verify

### ۵.۵ چرا PoP هزینه نود را کم می‌کند

| هزینه | Bitcoin PoW | Ethereum PoS | Centichain PoP |
|-------|-------------|--------------|----------------|
| سخت‌افزار | ASIC/GPU | سرور + ۳۲ ETH | لپ‌تاپ ۴ هسته |
| برق مداوم | بالا (رقابت) | متوسط | پایین (idle + بلاک هر ۲s اگر leader) |
| VDF/PoW | هر بلاک | — | **فقط یک‌بار ورود** |
| Stake | — | اجباری | **اختیاری (آینده)** |

**تخمین هزینه validator خانگی:** ۲–۵ USD/ماه (برق) + اینترنت موجود

---

## ۶. شاردینگ افقی CHSP

### ۶.۱ Centichain Horizontal Scaling Protocol

**ایده:** state space به N shard تقسیم می‌شود؛ هر shard زنجیره نیمه‌مستقل با leader rotation خودش.

### ۶.۲ فرمول shard

```
active_shards = max(1, floor(validators / 50))
```

**حداقل ۵۰ validator per shard** — برای حفظ امنیت BFT-like (فرض: ≤۱/۳ Byzantine)

**کد:** `consensus/sharding.rs` — `calculate_active_shards()`

### ۶.۳ تخصیص validator به shard

```
shard_id = SHA256(peer_id || epoch) mod active_shards
```

**خواص:**

- **Deterministic** — همه نودها نتیجه یکسان
- **Epoch-based reshuffle** — هر ۱۰ دقیقه تعادل بار
- **مقاوم در برابر grinding** — peer_id ثابت، epoch از زمان جهانی

### ۶.۴ مسیریابی تراکنش

```
sender_shard = get_assigned_shard(sender_address, epoch)
tx.shard_id = sender_shard
```

تراکنش‌هایی که `receiver` در shard دیگر است → **cross-shard** → Receipt

**کد:** `commands/chain.rs` — shard_id در submit  
**کد:** `node/helpers.rs` — `collect_shard_transactions()`

### ۶.۵ Beacon Layer (هدف)

وایت‌پیپر Beacon Chain را به‌عنوان coordinator معرفی می‌کند:

- ثبت global validator set
- جمع‌آوری CrossLink از shardها
- finality سراسری

**وضعیت فعلی:** struct `CrossLink` در `chain/receipt.rs` — **پیاده‌سازی نشده**

**رفرنس الهام:** Ethereum 2.0 Beacon Chain — [Ethereum Consensus Specs](https://github.com/ethereum/consensus-specs)

---

## ۷. اتمی بودن بین‌شاردی

### ۷.۱ مشکل

کاربر A در Shard ۱ به کاربر B در Shard ۲ پول بفرستد — بدون پروتکل اتمی، یا double-spend می‌شود یا پول گم می‌شود.

### ۷.۲ پروتکل ۳ فاز (هدف — از وایت‌پیپر)

```
فاز ۱ — Burn (Shard مبدأ):
  - موجودی A کسر شود
  - PendingReceipt ساخته و broadcast شود

فاز ۲ — Mint (Shard مقصد):
  - Receipt validate شود (merkle proof + block hash)
  - موفق → mint به B، status = Claimed
  - شکست/timeout → RevertReceipt

فاز ۳ — Rollback (Shard مبدأ):
  - در صورت Revert → بازگشت وجه به A
```

**Structها:** `chain/receipt.rs` — `Receipt`, `ReceiptStatus`

**وضعیت فعلی:**

- Receipt **ساخته** و **broadcast** می‌شود (`p2p.rs`)
- Handler برای claim/revert/timeout **وجود ندارد**
- `merkle_proof` خالی، `block_hash` = `"pending"`

### ۷.۳ رفرنس‌های الگوریتم‌های مشابه

| پروژه | مکانیزم | رفرنس |
|-------|---------|-------|
| Ethereum 2.0 | Cross-shard via beacon | [Sharding FAQ](https://ethereum.org/en/roadmap/danksharding/) |
| Polkadot | XCMP cross-chain | [Polkadot Wiki](https://wiki.polkadot.network/docs/learn-xcm) |
| Cosmos | IBC protocol | [IBC Spec](https://github.com/cosmos/ibc) |
| Zilliqa | Account-based sharding | [Zilliqa Whitepaper](https://docs.zilliqa.com/whitepaper.pdf) |

---

## ۸. شبکه P2P و سادگی اجرای نود

### ۸.۱ راه‌اندازی نود — جریان هدف (۱۰ دقیقه)

```
1. نصب اپ Tauri (یا cargo run)
2. ایجاد/وارد کردن wallet
3. Start Node
4. اتصال خودکار به Relay
5. Discovery همسایه (DHT + mDNS)
6. Sync زنجیره
7. [اختیاری] VDF + Patience → Validator
```

### ۸.۲ همگام‌سازی (Sync Protocol)

**Request types** (`chain/block.rs`):

```rust
enum SyncRequest {
    GetBlock(u64),
    GetBlocksRange(u64, u64),  // batch تا ۱۰۰ بلاک
    GetHeaders(u64, u64),      // headers-first (هدف)
    GetHeight,
    GetMempool,
}
```

**استراتژی:**

1. `GetHeight` — مقایسه ارتفاع
2. `GetBlocksRange` — دانلود batch
3. Grace period ۵ ثانیه پس از sync
4. `is_synced = true` → شروع mining

**ثابت‌ها:** `SYNC_GRACE_PERIOD_SECS = 5`, `MAX_SYNC_WAIT_SECS = 300`

**رفرنس الهام:** Bitcoin Headers-First Sync — [Bitcoin Developer Guide](https://developer.bitcoin.org/devguide/block_chain.html#headers-first)

### ۸.۳ Fork Choice (هدف — فاز ۱)

**قانون پیشنهادی:**

```
بهترین زنجیره = بلندترین زنجیره معتبر
اعتبار = cumulative slot work یا تعداد validators تأییدکننده
```

**وضعیت فعلی:** بلاک با index یکسان از منابع مختلف ممکن است overwrite شود — **نیاز به fork choice**

### ۸.۴ محدودیت اتصال

```
DEFAULT_MAX_PEERS = 50
DHT_PEER_THRESHOLD_FOR_RELAY_FREE = 3
```

---

## ۹. رمزنگاری و مدل امنیتی

### ۹.۱ لایه‌های امنیت

```
┌──────────────────────────────────────┐
│ L4: Economic (reward, slashing, fee) │
├──────────────────────────────────────┤
│ L3: Consensus (PoP, leader, shard)   │
├──────────────────────────────────────┤
│ L2: Block validation (hash, merkle)  │
├──────────────────────────────────────┤
│ L1: Transaction crypto (Ed25519)     │
├──────────────────────────────────────┤
│ L0: Transport (Noise, TLS-like)      │
└──────────────────────────────────────┘
```

### ۹.۲ ساختار بلاک

```rust
// chain/block.rs — فیلدهای امنیتی
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub hash: String,
    pub merkle_root: String,
    pub state_root: String,      // هدف: MPT root
    pub vdf_proof: String,
    pub vdf_difficulty: u64,
    pub shard_id: u32,
    pub author: String,          // PeerId
    pub signature: String,       // هدف: امضای author
    // ...
}
```

### ۹.۳ اعتبارسنجی بلاک (هدف — فاز ۱)

تابع `validate_block(block, chain_tip, consensus) -> Result<(), Error>`:

| # | چک | توضیح |
|---|-----|-------|
| 1 | `index == tip.index + 1` | ترتیب |
| 2 | `previous_hash == tip.hash` | پیوستگی |
| 3 | `merkle_root == calculate_merkle_root(txs)` | یکپارچگی تراکنش |
| 4 | `author == get_shard_leader(shard, slot)` | رهبر مجاز |
| 5 | `is_vdf_valid()` | اثبات تأخیر |
| 6 | هر tx: signature + balance | امنیت تراکنش |
| 7 | `coinbase <= calculate_mining_reward(index)` | ضد تورم |
| 8 | `len(txs) <= MAX_TXS_PER_BLOCK` | anti-spam |

### ۹.۴ ساختار تراکنش و امضا (هدف)

```
message = SHA256(sender || receiver || amount || nonce || timestamp || shard_id)
signature = Ed25519.sign(message, private_key)
```

**ضد replay:** `nonce` یا `timestamp` + window در mempool

### ۹.۵ مقاومت در برابر حملات

| حمله | دفاع |
|------|------|
| **Sybil** | PoP quarantine + VDF difficulty scaling |
| **51% / Byzantine** | shard min 50 validators؛ slashing (هدف: BFT finality) |
| **Double-spend** | account balance check + nonce |
| **Eclipse** | چند relay + DHT random walk |
| **Spam txs** | fee (0.01%) + block capacity |
| **Long-range** | checkpoint در beacon (هدف) |
| **Grinding leader** | hash randomness از epoch global |

### ۹.۶ Merkle Tree

**پیاده‌سازی:** `chain/merkle.rs` — SHA256 pairwise تا root

**هدف آینده:** Merkle proof برای SPV light client

**رفرنس:** [Merkle Trees — Bitcoin Whitepaper §7](https://bitcoin.org/bitcoin.pdf)

---

## ۱۰. ذخیره‌سازی، State و Pruning

### ۱۰.۱ مدل State فعلی

**Account-based ساده:**

```
state_table: address → balance (u64)
```

به‌روزرسانی در `storage.save_block()` — کسر از sender (+ fee)، اضافه به receiver.

### ۱۰.۲ مدل State هدف

**Merkle Patricia Trie (MPT)** — مانند Ethereum:

- `state_root` در هر بلاک
- proof برای light client
- مقاومت در برابر tampering

**رفرنس:** [Ethereum Yellow Paper — Patricia Tree](https://ethereum.github.io/yellowpaper/paper.pdf)

### ۱۰.۳ Pruning — Infinity Scaling

**ایده وایت‌پیپر:**

- نودهای pruned بدنه تراکنش‌های >۲۴h را حذف می‌کنند
- header chain حفظ می‌شود → SPV ممکن
- نیاز دیسک ثابت می‌ماند (~۵۰–۲۰۰ GB)

**پیاده‌سازی فعلی:**

```rust
// NodeType::Pruned → prune_history(2000) blocks
// PRUNED_HISTORY_BLOCKS = 2000
```

**کد:** `storage/mod.rs` — `prune_history()`

**رفرنس الهام:** Bitcoin pruning mode — [Bitcoin Core pruning](https://bitcoin.org/en/full-node#reduce-storage)

---

## ۱۱. اجرای موازی تراکنش‌ها

### ۱۱.۱ Block-STM (هدف)

**Paper:** *Block-STM: Scaling Blockchain Execution by Turning Ordering Curse to Performance Blessing* — Sarat Chandra et al., Aptos/Libra

**رفرنس:** [Block-STM Paper (arXiv:2203.06871)](https://arxiv.org/abs/2203.06871)

**ایده:**

1. تراکنش‌ها را speculatively اجرا کن
2. اگر conflict (همان account) → rollback و retry
3. تراکنش‌های `is_independent()` موازی اجرا می‌شوند

**کد آماده:** `transaction.rs` — `is_independent()`

```rust
pub fn is_independent(&self, other: &Self) -> bool {
    self.sender != other.sender
        && self.sender != other.receiver
        && self.receiver != other.sender
        && self.receiver != other.receiver
}
```

**وضعیت:** تابع وجود دارد؛ engine اجرای موازی **پیاده نشده**

### ۱۱.۲ ظرفیت بلاک

```
TARGET_BLOCK_TIME = 2 seconds
MAX_TXS_PER_BLOCK = 3,000
→ 1,500 TPS per shard (هدف)
MAX_BLOCK_SIZE = 1.5 MB
```

---

## ۱۲. توکنومیکس AGT

### ۱۲.۱ پارامترها

| پارامتر | مقدار | ثابت |
|---------|-------|------|
| نام توکن | AGT (Antigravity Token) | — |
| Decimals | 6 | `AGT_DECIMALS` |
| Max Supply | 21,000,000 AGT | `TOTAL_SUPPLY` |
| Genesis Allocation | 5,000,000 AGT | `GENESIS_SUPPLY` |
| Initial Reward | ~0.127 AGT/block | `INITIAL_REWARD` |
| Halving Interval | ~۴ سال (63,072,000 blocks @ 2s) | `HALVING_INTERVAL` |

### ۱۲.۲ فرمول پاداش

```rust
// chain/block.rs
fn calculate_mining_reward(index: u64) -> u64 {
    let halving_count = index / HALVING_INTERVAL;
    INITIAL_REWARD >> halving_count  // until 0
}
```

**الگو:** Bitcoin halving — [Bitcoin Whitepaper §6](https://bitcoin.org/bitcoin.pdf)

### ۱۲.۳ کارمزد

```
fee = max(0.001 AGT, ceil(amount × 0.0001))
```

**هدف:** anti-spam؛ نه منبع اصلی درآمد validator (پاداش بلاک مهم‌تر است)

### ۱۲.۴ توزیع Genesis

بلاک ۰: `SYSTEM → genesis_validator` با ۵M AGT

**هدف mainnet:** Genesis Ceremony چندامضایی — شفاف و عمومی

---

## ۱۳. انواع نود و سادگی کلاینت

### ۱۳.۱ سلسله‌مراتب نودها

```
┌─────────────────────────────────────────────────────────────┐
│ Tier 0: Wallet-Only                                         │
│   - فقط کلید + امضا                                         │
│   - اتصال به Public RPC                                     │
│   - RAM: ~256MB | دیسک: 0                                    │
├─────────────────────────────────────────────────────────────┤
│ Tier 1: Light Client (فاز ۴)                                │
│   - sync header chain                                       │
│   - SPV verify با merkle proof                              │
│   - RAM: ~512MB | دیسک: ~1GB                                 │
├─────────────────────────────────────────────────────────────┤
│ Tier 2: Pruned Validator                                    │
│   - PoP + leader rotation                                   │
│   - prune_history(2000)                                     │
│   - RAM: 4GB | دیسک: ~50GB                                   │
├─────────────────────────────────────────────────────────────┤
│ Tier 3: Full Validator                                      │
│   - تمام history + archive                                  │
│   - RAM: 8GB | دیسک: ~500GB+                                 │
├─────────────────────────────────────────────────────────────┤
│ Tier 4: RPC Node (rpc_node binary)                          │
│   - Full/Pruned + HTTP API + WebSocket                      │
│   - برای صرافی و سرویس‌ها                                     │
├─────────────────────────────────────────────────────────────┤
│ Tier 5: Relay Node (relay_server binary)                    │
│   - فقط NAT traversal — بدون consensus                       │
│   - VPS ارزان — داوطلبانه                                    │
└─────────────────────────────────────────────────────────────┘
```

### ۱۳.۲ حداقل سخت‌افزار Validator (هدف)

| منبع | حداقل | توصیه‌شده |
|------|-------|----------|
| CPU | ۴ هسته x64 | ۴+ هسته |
| RAM | ۴ GB | ۸ GB |
| دیسک | ۵۰ GB SSD (pruned) | ۱۰۰ GB SSD |
| شبکه | ۱۰ Mbps پایدار | ۵۰+ Mbps |
| OS | Windows 10+, Linux, macOS | — |

### ۱۳.۳ باینری‌های پروژه

| باینری | دستور | نقش |
|--------|-------|-----|
| `centichain` | `npm run tauri dev` | اپ دسکتاپ کامل |
| `rpc_node` | `cargo run --bin rpc_node` | API عمومی |
| `relay_server` | `cargo run --bin relay_server` | Relay (تنها باینری relay) |
| `bench_sharding` | `cargo run --bin bench_sharding` | بنچمارک shard assignment |

---

## ۱۴. اهداف عملکردی و مدل TPS

### ۱۴.۱ فرمول TPS

```
TPS_global = TPS_per_shard × active_shards
TPS_per_shard = MAX_TXS_PER_BLOCK / TARGET_BLOCK_TIME
              = 3000 / 2 = 1500
```

### ۱۴.۲ بنچمارک‌های هدف

| سناریو | Validators | Shards | TPS هدف |
|--------|-----------|--------|---------|
| Dev (solo) | 1 | 1 | ~۱۰ (فعلی) |
| Testnet Alpha | 50 | 1 | 1,500 |
| Testnet Beta | 500 | 10 | 15,000 |
| Mainnet Year 1 | 5,000 | 100 | 150,000 |

### ۱۴.۳ بنچمارک فعلی (`bench_sharding`)

فقط **سرعت تخصیص shard** را می‌سنجد — نه TPS واقعی:

```
100,000 validators → 2,000 shards → assignment ~402ms
```

**برای TPS واقعی نیاز:** integration test با تراکنش‌های synthetic + latency network

### ۱۴.۴ گلوگاه‌های احتمالی

| گلوگاه | راه‌حل |
|--------|--------|
| VDF solve در هر بلاک (فعلی) | VDF فقط در ورود؛ بلاک فقط hash سبک |
| JSON serialization در gossip | bincode/protobuf |
| Full chain scan در mempool | tx index در DB |
| Single-threaded execution | Block-STM |
| sync serial | parallel batch download |

---

## ۱۵. نقشه راه پیاده‌سازی (فازبندی)

### فاز ۰ — پایه (✅ انجام‌شده ~۴۰٪)

- [x] ساختار Rust modular
- [x] Tauri UI کامل
- [x] libp2p P2P + gossip + sync
- [x] ساختار AHSP + sharding logic
- [x] توکنومیکس + mempool
- [x] Pruned node option
- [x] rpc_node skeleton

### فاز ۱ — امنیت پایه (۳–۴ ماه) — **CRITICAL**

| # | کار | فایل‌های اصلی | معیار پذیرش |
|---|-----|---------------|-------------|
| 1.1 | Ed25519 tx signing | `wallet/`, `commands/chain.rs` | reject unsigned tx |
| 1.2 | `validate_block()` | `chain/block.rs` جدید | invalid block rejected |
| 1.3 | اعمال validation در P2P | `network/p2p.rs` | no blind save_block |
| 1.4 | Fork choice (longest valid) | `consensus/` یا `chain/` | ۳ نود consensus |
| 1.5 | Persist NodeState | `storage/mod.rs` | survive restart |
| 1.6 | حذف mark_peer_active ناامن | `leadership.rs` | VDF-only activation |
| 1.7 | tx index در storage | `storage/mod.rs` | O(1) tx lookup |
| 1.8 | Integration test ۳ نود | `tests/` | ۷ روز stable LAN |

### فاز ۲ — PoP واقعی + Testnet (۴–۶ ماه)

| # | کار | رفرنس فنی |
|---|-----|-----------|
| 2.1 | Argon2id VDF (verify سریع) | RFC 9106 |
| 2.2 | Quarantine سخت‌گیرانه | — |
| 2.3 | Relay عمومی (۳+ VPS) | libp2p relay v2 |
| 2.4 | Bootstrap peer list | — |
| 2.5 | Testnet Alpha + faucet | — |
| 2.6 | rpc_node validation کامل | پیوست ج |
| 2.7 | مستندات «نود در ۱۰ دقیقه» | — |

**معیار:** ۱۰۰+ نود testnet، VDF solve < ۱۰ min

### فاز ۳ — مقیاس (۶–۹ ماه)

| # | کار | رفرنس |
|---|-----|-------|
| 3.1 | State per-shard در DB | Ethereum sharding docs |
| 3.2 | Cross-shard receipt handler | وایت‌پیپر §3.4 |
| 3.3 | Beacon + CrossLink | Eth2 consensus specs |
| 3.4 | state_root با MPT | Yellow Paper |
| 3.5 | Block-STM execution | arXiv:2203.06871 |
| 3.6 | BLS aggregation (اختیاری) | [BLS12-381](https://github.com/zkcrypto/bls12_381) |

**معیار:** cross-shard بدون از دست رفتن funds؛ ≥۱۰۰۰ TPS (۱ shard)

### فاز ۴ — Mainnet + ذخیره ارزش (۹–۱۸ ماه)

| # | کار |
|---|-----|
| 4.1 | Genesis Ceremony |
| 4.2 | Light client (mobile) |
| 4.3 | Security audit (خارجی) |
| 4.4 | Bug bounty |
| 4.5 | ۶+ ماه battle-test |
| 4.6 | Slashing on-chain |
| 4.7 | Checkpoint / weak subjectivity |

**معیار mainnet:** ۱ سال testnet بدون exploit بحرانی + ۱۰۰۰+ validator

---

## ۱۶. مقایسه با شبکه‌های موجود

| ویژگی | Centichain (هدف) | Bitcoin | Ethereum 2.0 | Solana |
|-------|-----------------|---------|--------------|--------|
| **Consensus** | PoP + Round-Robin | PoW | PoS | PoH + PoS |
| **Scaling** | Horizontal sharding | L2 only | L2 + rollup | Vertical (HW) |
| **Validator HW** | لپ‌تاپ ۴ هسته | ASIC | ۳۲ ETH + سرور | ۱۲۸GB RAM |
| **Node cost/month** | ~$۲–۵ | ~$۵۰–۵۰۰+ | ~$۱۰۰+ (+ stake) | ~$۲۰۰+ |
| **TPS (theoretical)** | Unbounded (shards) | ~7 | ~۱۰۰K (w/ L2) | ~۶۵K |
| **Finality** | Slot-based (هدف: BFT) | ~۶۰ min | ~۱۵ min | ~۰.۴s |
| **ASIC resistance** | Memory-hard VDF | ❌ (ASIC wins) | N/A | N/A |
| **Client simplicity** | Wallet-only OK | Full node سنگین | Light via RPC | RPC-heavy |
| **Supply cap** | 21M AGT | 21M BTC | Unlimited ETH | Unlimited SOL |

---

## ۱۷. نگاشت به کدبیس فعلی و شکاف‌ها

### ۱۷.۱ جدول شکاف (Gap Analysis)

| قابلیت | سند/هدف | کد فعلی | اولویت |
|--------|---------|---------|--------|
| Tx Ed25519 signing | فاز ۱ | `signature: "sig"` | 🔴 P0 |
| Block validation | فاز ۱ | save بدون validate | 🔴 P0 |
| Fork choice | فاز ۱ | ندارد | 🔴 P0 |
| True VDF | فاز ۲ | verify=solve | 🟠 P1 |
| Quarantine enforce | فاز ۲ | mark_peer_active bypass | 🟠 P1 |
| Cross-shard claim | فاز ۳ | receipt broadcast only | 🟡 P2 |
| state_root MPT | فاز ۳ | always zero | 🟡 P2 |
| Block-STM | فاز ۳ | is_independent only | 🟡 P2 |
| Beacon/CrossLink | فاز ۳ | struct only | 🟡 P2 |
| Light client | فاز ۴ | ندارد | 🟢 P3 |
| Production relay | فاز ۲ | localhost | 🟠 P1 |

### ۱۷.۲ فایل‌های کلیدی برای شروع فاز ۱

```
src-tauri/src/
├── chain/
│   ├── block.rs          ← اضافه: validate_block()
│   └── transaction.rs    ← اضافه: sign(), verify()
├── commands/
│   └── chain.rs          ← fix: real signing
├── network/
│   └── p2p.rs            ← fix: validate before save
├── consensus/
│   └── leadership.rs     ← fix: remove unsafe activation
└── storage/
    └── mod.rs            ← add: consensus state, tx index
```

---

## ۱۸. مراجع و منابع

### ۱۸.۱ مقالات آکادمیک

| موضوع | مرجع |
|-------|------|
| Bitcoin | [Nakamoto — Bitcoin Whitepaper (2008)](https://bitcoin.org/bitcoin.pdf) |
| Ethereum | [Wood — Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf) |
| VDF | [Boneh et al. — Verifiable Delay Functions (2018)](https://eprint.iacr.org/2018/623) |
| Block-STM | [Gelashvili et al. — arXiv:2203.06871](https://arxiv.org/abs/2203.06871) |
| Gossip Protocols | [Demers et al. — Epidemic Algorithms (1987)](https://www.cs.cornell.edu/home/rvr/papers/iptps03.pdf) |
| Kademlia DHT | [Maymounkov & Mazières (2002)](https://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf) |
| BFT | [Castro & Liskov — PBFT (1999)](https://pmg.csail.mit.edu/papers/osdi99.pdf) |
| Memory-Hard | [Percival — scrypt (2009)](https://www.tarsnap.com/scrypt/scrypt.pdf) |
| Argon2 | [RFC 9106](https://www.rfc-editor.org/rfc/rfc9106.html) |

### ۱۸.۲ مشخصات و استانداردها

| موضوع | مرجع |
|-------|------|
| libp2p | [github.com/libp2p/specs](https://github.com/libp2p/specs) |
| Gossipsub 1.1 | [pubsub/gossipsub/gossipsub-v1.1.md](https://github.com/libp2p/specs/blob/master/pubsub/gossipsub/gossipsub-v1.1.md) |
| Ed25519 | [RFC 8032](https://www.rfc-editor.org/rfc/rfc8032) |
| SHA-256 | [FIPS 180-4](https://csrc.nist.gov/publications/detail/fips/180/4/final) |
| Ethereum Consensus | [ethereum/consensus-specs](https://github.com/ethereum/consensus-specs) |
| IBC (cross-chain) | [cosmos/ibc](https://github.com/cosmos/ibc) |

### ۱۸.۳ مستندات فناوری پروژه

| فناوری | URL |
|--------|-----|
| Rust | https://doc.rust-lang.org/book/ |
| Tokio | https://tokio.rs/ |
| Tauri v2 | https://v2.tauri.app/ |
| libp2p Rust | https://docs.rs/libp2p/latest/libp2p/ |
| ReDB | https://docs.rs/redb/latest/redb/ |
| React | https://react.dev/ |
| Axum | https://docs.rs/axum/latest/axum/ |

### ۱۸.۴ اسناد داخلی پروژه

| سند | مسیر |
|-----|------|
| README | [README.md](../README.md) |
| Whitepaper EN | [CENTICHAIN_WHITEPAPER.md](../CENTICHAIN_WHITEPAPER.md) |
| مرجع فنی FA | [MASTER_VISION_AND_TECHNOLOGY_FA.md](./MASTER_VISION_AND_TECHNOLOGY_FA.md) |

---

## ۱۹. واژه‌نامه

| اصطلاح | توضیح |
|--------|-------|
| **AHSP** | Adaptive Hierarchical Sharded Proof-of-Patience — نام اجماع Centichain |
| **AGT** | Antigravity Token — توکن بومی شبکه |
| **PoP** | Proof of Patience — اثبات صبر؛ ورود validator با VDF + زمان |
| **VDF** | Verifiable Delay Function — تابع با تأخیر قابل اثبات |
| **CHSP** | Centichain Horizontal Scaling Protocol — پروتکل شاردینگ افقی |
| **Slot** | پنجره ۲ ثانیه‌ای برای یک بلاک |
| **Epoch** | دوره ۱۰ دقیقه‌ای؛ reshuffle shard |
| **Quarantine** | دوره انتظار قبل از فعال‌سازی validator |
| **Shard** | پارتیشن افقی state/validators |
| **Receipt** | مدرک cross-shard برای انتقال اتمی |
| **CrossLink** | خلاصه header shard برای beacon |
| **Mempool** | استخر تراکنش‌های pending |
| **Pruning** | حذف history قدیمی برای صرفه‌جویی دیسک |
| **SPV** | Simplified Payment Verification — تأیید سبک بدون full node |
| **Sybil** | حمله با هویت‌های جعلی زیاد |
| **Slashing** | جریمه validator بدکار |
| **TPS** | Transactions Per Second |
| **BFT** | Byzantine Fault Tolerance — تحمل تا ۱/۳ نود مخرب |
| **NAT** | Network Address Translation — مانع اتصال مستقیم نود خانگی |
| **DCUtR** | Direct Connection Upgrade through Relay |
| **MPT** | Merkle Patricia Trie — ساختار state Ethereum |

---

## پیوست الف — نمودار اجماع PoP

```
                    ┌─────────────┐
                    │  نود جدید   │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ Register +  │
                    │ Solve VDF   │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │  Verified   │
                    └──────┬──────┘
                           │
              ┌────────────▼────────────┐
              │   Quarantine Wait       │
              │   (5min → 72h)          │
              └────────────┬────────────┘
                           │
                    ┌──────▼──────┐
                    │  Activated  │
                    │ (eligible)  │
                    └──────┬──────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
  ┌──────▼──────┐   ┌──────▼──────┐   ┌──────▼──────┐
  │   Queue     │   │   Leader    │   │  Slashed    │
  │ (waiting)   │   │ (producing) │   │ (demoted)   │
  └─────────────┘   └─────────────┘   └─────────────┘
```

---

## پیوست ب — نمودار شاردینگ

```
                    ┌──────────────────┐
                    │  Beacon (هدف)   │
                    │  Finality Layer  │
                    └────────┬─────────┘
                             │ CrossLinks
           ┌─────────────────┼─────────────────┐
           │                 │                 │
    ┌──────▼──────┐   ┌──────▼──────┐   ┌──────▼──────┐
    │  Shard 0    │   │  Shard 1    │   │  Shard N    │
    │ 1500 TPS    │   │ 1500 TPS    │   │ 1500 TPS    │
    │ Leaders RR  │   │ Leaders RR  │   │ Leaders RR  │
    └──────┬──────┘   └──────┬──────┘   └──────┬──────┘
           │                 │                 │
           └─────────────────┼─────────────────┘
                             │
                    ┌────────▼─────────┐
                    │  Receipts (P2P)  │
                    │  Cross-shard TX  │
                    └──────────────────┘
```

---

## پیوست ج — RPC Node API

برای اتصال صرافی، wallet وب، یا سرویس خارجی از باینری `rpc_node` استفاده کنید.

### اجرا

```bash
cd src-tauri
cargo run --release --bin rpc_node
```

| سرویس | آدرس |
|-------|------|
| REST API | `http://localhost:3000/api/v1` |
| WebSocket | `ws://localhost:3000/ws` |
| P2P | TCP `9091` |

### Endpoints

| Method | Path | توضیح |
|--------|------|-------|
| GET | `/status` | وضعیت نود (height, peers) |
| GET | `/network/stats` | توکنومیکس و difficulty |
| GET | `/balance/:address` | موجودی (base units) |
| GET | `/blocks?page=&limit=` | لیست بلاک |
| GET | `/transactions/:id` | جزئیات تراکنش |
| POST | `/broadcast` | ارسال تراکنش امضا‌شده |

### WebSocket events

- `NewBlock` — بلاک جدید
- `NewTransaction` — تراکنش جدید در mempool

### نکات امنیتی

- RPC کلید خصوصی تولید یا امضا **نمی‌کند**
- تراکنش‌ها باید در سمت کلاینت امضا شوند (cold-wallet pattern)
- اعتبارسنجی کامل signature در RPC — هدف فاز ۲

---

*این سند مرجع رسمی چشم‌انداز و فناوری Centichain است. با پیشرفت پیاده‌سازی، نسخه‌ها به‌روزرسانی می‌شوند.*

**نگهدارنده:** تیم Centichain / Antigravity Chain  
**بازبینی بعدی:** پس از تکمیل فاز ۱ امنیت
