---
name: boha
description: Use boha as a Rust library for crypto puzzle and bounty data. Trigger when code imports `boha`, references Bitcoin puzzle transactions, hash collision bounties, or needs programmatic access to crypto challenge collections. Do not use for CLI usage - see boha-cli skill instead.
metadata:
  author: oritwoen
  version: "0.16.0"
---

Rust library for crypto bounties, puzzles and challenges data. Eight collections across six blockchains (Bitcoin, Ethereum, Litecoin, Monero, Decred, Arweave). All data embedded at compile time as `&'static` references.

## Install

```toml
[dependencies]
boha = "0.16"

# With async balance fetching (requires tokio runtime)
boha = { version = "0.16", features = ["balance"] }
tokio = { version = "1", features = ["full"] }
```

## Puzzle ID Format

IDs follow `collection/identifier` pattern. Two exceptions have no slash.

| Collection | ID example | Notes |
|------------|-----------|-------|
| b1000 | `b1000/66` | Number 1-256, accepts u32/usize/&str |
| arweave | `arweave/weave1` | Name string |
| ballet | `ballet/AA007448` | Serial number |
| bitaps | `bitaps` | Single puzzle, no slash |
| bitimage | `bitimage/kitten` | Name string |
| gsmg | `gsmg` | Single puzzle, no slash |
| hash_collision | `hash_collision/sha256` | sha1, sha256, ripemd160, hash160, hash256, op_abs |
| zden | `zden/level_1` | snake_case level names |

## API

```rust
use boha::{b1000, hash_collision, zden, Status, Chain};

// Fetch by collection-specific getter
let p = b1000::get(66).unwrap();
println!("{} - {}", p.address.value, p.status);

// Fetch by universal ID
let p = boha::get("hash_collision/sha256").unwrap();

// Iterate and filter
let targets: Vec<_> = b1000::all()
    .filter(|p| p.status == Status::Unsolved)
    .filter(|p| p.pubkey.is_some())
    .collect();

// Key range (b1000 only) - puzzle N has key in [2^(N-1), 2^N-1]
let range = b1000::get(90).unwrap().key_range().unwrap();

// Big key range for bits > 128
let (lo, hi) = b1000::get(200).unwrap().key_range_big().unwrap();

// Stats
let s = boha::stats();
println!("Total: {}, Unsolved: {}", s.total, s.unsolved);

// Explorer URLs
let url = p.chain.tx_explorer_url("txid_here");
let addr_url = p.explorer_url();
```

### Balance fetching (feature: `balance`)

```rust
use boha::balance;

#[tokio::main]
async fn main() {
    let bal = balance::fetch("1A1zP1...", boha::Chain::Bitcoin).await.unwrap();
    println!("{} sats confirmed, {:.8} BTC total", bal.confirmed, bal.total_btc());
}
```

Supports Bitcoin (mempool.space), Litecoin (litecoinspace.org) and Ethereum (Etherscan).

## Key Data Types

See [types.md](references/types.md) for full struct definitions.

Core types: `Puzzle`, `Address`, `Key`, `Chain`, `Status` (Solved/Unsolved/Claimed/Swept), `Author`, `Solver`, `Transaction`.

A puzzle has: address, chain, status, optional prize, optional pubkey, optional private key (if solved), transactions history, solver info, and assets (images/hints for visual puzzles).

## Collections Overview

See [collections.md](references/collections.md) for detailed collection data.

## Limitations

- All data is `&'static` - no heap allocation, no runtime loading.
- `key_range()` works for bits <= 128. Use `key_range_big()` for larger.
- Balance fetching is async and requires the `balance` feature flag.
- Puzzle data lives in `data/*.jsonc` - don't hardcode in Rust source.
