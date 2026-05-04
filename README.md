# DecoraChain 🛋️⛓️
> AI-powered room design marketplace — rewarded on Stellar.

---

## Problem
A home renter in Metro Manila wants to redesign their small apartment but can't afford an interior designer, has no trusted source for theme-based furniture recommendations, and gets no reward for sharing their room ideas with others online.

## Solution
DecoraChain lets users submit their room layout + chosen theme, receive AI-generated furniture and decor suggestions, share designs publicly, and earn **DCOR tokens** (issued on Stellar) when the community upvotes their work — with rewards claimable instantly via Stellar's low-cost, fast settlement.

---

## Stellar Features Used
| Feature | Usage |
|---|---|
| **Soroban Smart Contracts** | Design storage, upvote logic, reward distribution |
| **Custom Token (DCOR)** | Reward token issued on Stellar for community participation |
| **XLM / USDC** | Eventual conversion of DCOR via Stellar DEX |
| **Built-in DEX** | DCOR ↔ USDC swap for reward cash-out |
| **Trustlines** | Users accept DCOR token before receiving rewards |

---

## Target Users
- **Who**: Renters, first-time homeowners, interior design enthusiasts aged 18–35
- **Where**: Southeast Asia (Philippines, Indonesia, Vietnam)
- **Why**: No budget for designers, high mobile usage, love sharing aesthetic content

---

## MVP Core Feature (Demo Flow)
```
User submits room layout hash + theme (e.g. "boho")
  → submit_design() called on Soroban contract
  → Design stored on-chain with 0 upvotes

Community member upvotes the design
  → upvote_design() called
  → Designer's DCOR balance += 1 on-chain

Designer reaches 10 DCOR
  → claim_rewards() called
  → Balance resets, DCOR transferred via Stellar anchor
```
Demo time: ~90 seconds ✅

---

## Why This Wins
DecoraChain combines real creative utility (AI room design) with tangible on-chain rewards using Stellar's speed and low fees — making it compelling for SEA users who want both creative tools and financial inclusion. Judges see real users, real money movement, and a live Soroban contract demo.

---

## Optional Edge: AI Integration
The off-chain AI layer (Claude API) reads the submitted theme + room dimensions and returns a structured furniture/decor recommendation list, which is then pinned to IPFS and hashed on-chain via `layout_hash`.

---

## Vision & Purpose
DecoraChain turns interior design inspiration into a participatory economy. Every aesthetic choice becomes an on-chain contribution. Every upvote is a micro-reward. The goal is a decentralized design marketplace where talented home decorators — regardless of professional credentials — earn real value from their creativity.

---

## Prerequisites
- Rust `1.74+`
- Soroban CLI `v20.x` — install via:
  ```bash
  cargo install --locked soroban-cli
  ```
- Stellar Testnet account funded via [Friendbot](https://friendbot.stellar.org)

---

## Build
```bash
soroban contract build
# Output: target/wasm32-unknown-unknown/release/decora_chain.wasm
```

## Test
```bash
cargo test
# Runs all 5 tests in test.rs
```

## Deploy to Testnet
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/decora_chain.wasm \
  --source YOUR_SECRET_KEY \
  --network testnet
# Returns: CONTRACT_ID
```

---

## Sample CLI Invocations

### Submit a design
```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --source YOUR_SECRET_KEY \
  --network testnet \
  -- submit_design \
  --owner GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX \
  --layout_hash "QmXyZ123abc" \
  --theme "boho"
```

### Upvote a design
```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --source VOTER_SECRET_KEY \
  --network testnet \
  -- upvote_design \
  --voter GVOTERXXXXXXXXXXXXXXXXXXXXXXX \
  --designer GDESIGNERXXXXXXXXXXXXXXXXXX
```

### Claim rewards
```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --source YOUR_SECRET_KEY \
  --network testnet \
  -- claim_rewards \
  --owner GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

### Check reward balance
```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --network testnet \
  -- get_reward_balance \
  --owner GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

---

## Project Structure
```
decora_chain/
├── Cargo.toml
├── README.md
├── lib.rs        ← Soroban smart contract
└── test.rs       ← 5 contract tests
```

---

## License
MIT © 2025 DecoraChain

## CONTRACT
https://stellar.expert/explorer/testnet/tx/3fc8847ae60eb03cbebf3c3f5074e0592c88f533ccd40611d3776b1fd245aad1
https://lab.stellar.org/r/testnet/contract/CCEMANUWO27NK7G5MZ2GXUEAHL4ZOMX7IJ7CF3NLFBH7EQ2HUM6CPEZ6