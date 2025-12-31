# boha

[![CI](https://github.com/oritwoen/boha/actions/workflows/ci.yml/badge.svg)](https://github.com/oritwoen/boha/actions/workflows/ci.yml)
[![autofix.ci](https://img.shields.io/badge/autofix.ci-yes-success?logo=data:image/svg+xml;base64,PHN2ZyBmaWxsPSIjZmZmIiB2aWV3Qm94PSIwIDAgMTI4IDEyOCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cGF0aCB0cmFuc2Zvcm09InNjYWxlKDAuMDYxLC0wLjA2MSkgdHJhbnNsYXRlKC0yNTAsLTE3NTApIiBkPSJNMTc4NiAxMzA3cS0xMDAgMCAtMTc4IC0xN3EtNzkgLTE4IC0xNTQgLTYycS01MCAtMjkgLTEwNCAtNzhxLTU0IC00OCAtMTAyIC0xMTBxLTQ4IC02MiAtOTAgLTEzOHEtNDEgLTc2IC03MyAtMTY3cS02NSAtMTgwIC02NSAtMzk5cTAgLTk0IDI0IC0xNjYuNXE4IC0yMyAxOS41IC01MHQyOSAtNTguNXQzOC41IC02NS41cTQ0IC03MSA5OC41IC0xMjcuNXQ4OSAtODEuNXE1OCAtNDIgMTM0LjUgLTcwLjV0MTQwLjUgLTM2LjVxNjMgLTkgMTIzIC02cTE2IDAgMzguNSA2dDQzLjUgMTUuNXQ0MCAyMS41cTQ1IDMxIDYzIDUzcS0xNiAtMSAtNDEgLTEwcS0yNSAtOSAtMzggLTEycS00NSAtOSAtOTIgLTlxLTc1IDAgLTE2NyAyOS41dC0xNzUgNjcuNXEtODMgMzkgLTE0NSAxMTAuNXQtODYgMTY2LjVxLTI2IDk4IC0yNSAxNzRxMCA3MiAxMyAxNDVxMTMgNzIgNDcuNSAxNjAuNXQ1OC41IDEzMi41cTQ2IDg3IDE0MSAxNjkuNXQyMDkgMTI0LjVxNTUgMjEgMTE4LjUgMjV0MTI3LjUgLTEycTY1IC0xNyAxMjQgLTU2cTc0IC00OCAxMjQgLTEyN3E0NyAtNzYgNjEgLTE3NnE2IC00NyA2IC05OHEwIC0xMTQgLTMwIC0yMTUuNXQtNzUgLTE2Ny41cS00NSAtNjYgLTExMyAtMTI5LjV0LTEyOCAtMTAyLjVxLTYwIC0zOSAtMTQ4IC03OXEtODcgLTQwIC0xMjggLTUxcS00IC0xNSAtOCAtMjguNXQtOCAtMjUuNWwtMTAgLTM1bC0xMyAtNTdxMTggMCAzNS41IC0xdDM1LjUgLTVxNDAgLTggODMgLjV0ODIgMTguNXE0MiAxMiA2My41IDI0dDQyLjUgMjhxMjIgMTcgNTAgNDl0NTkgNzJxMjUgMzMgNTQuNSA4NC41dDUwLjUgMTAzLjVxNDIgMTAzIDQyIDI1NHEwIDE3NCAtNjggMzA1cS02OCAxMzIgLTE3NCAyMDhxLTEwNiA3NiAtMjMyIDk3cS01MSA4IC05OSA4eiIvPjwvc3ZnPg==)](https://autofix.ci)

Crypto bounties, puzzles and challenges data library.

## Installation

### CLI

Arch Linux (AUR):

```bash
paru -S boha
```

From crates.io:

```bash
cargo install boha --features cli,balance
```

### Library

```toml
[dependencies]
boha = "0.1"
```

With balance fetching:

```toml
[dependencies]
boha = { version = "0.1", features = ["balance"] }
```

## Usage

### CLI

```bash
# Statistics
boha stats

# List puzzles
boha list
boha list b1000
boha list b1000 --unsolved
boha list b1000 --with-pubkey

# Show puzzle details
boha show b1000/66
boha show gsmg
boha show hash_collision/sha256

# Get key range
boha range 66

# Check balance (requires --features balance)
boha balance b1000/71

# Output formats (default: table)
boha -o json stats
boha -o yaml show b1000/66
boha -o csv list b1000 > puzzles.csv
boha -o jsonl list b1000 --unsolved | jq .
```

#### Output formats

| Format | Flag | Description |
|--------|------|-------------|
| `table` | `-o table` | TUI table with colors (default) |
| `json` | `-o json` | Pretty-printed JSON |
| `jsonl` | `-o jsonl` | JSON Lines (one object per line) |
| `yaml` | `-o yaml` | YAML |
| `csv` | `-o csv` | CSV with header |

### Library

```rust
use boha::{b1000, gsmg, hash_collision, Status};

let p66 = b1000::get(66).unwrap();
println!("Address: {}", p66.address);
println!("H160: {}", p66.h160.unwrap());
println!("Funded: {}", p66.start_date.unwrap_or("unknown"));

let range = p66.key_range().unwrap();
println!("Range: 0x{:x} - 0x{:x}", range.start(), range.end());

let unsolved: Vec<_> = b1000::all()
    .filter(|p| p.status == Status::Unsolved)
    .filter(|p| p.pubkey.is_some())
    .collect();

let gsmg_puzzle = gsmg::get();
let sha256 = hash_collision::get("sha256").unwrap();

let puzzle = boha::get("b1000/66").unwrap();
let puzzle = boha::get("gsmg").unwrap();
```

### Balance fetching (async)

```rust
use boha::{b1000, balance};

#[tokio::main]
async fn main() {
    let puzzle = b1000::get(71).unwrap();
    let bal = balance::fetch(puzzle.address).await.unwrap();
    
    println!("Confirmed: {} sats", bal.confirmed);
    println!("Total: {:.8} BTC", bal.total_btc());
}
```

## Features

| Feature | Description |
|---------|-------------|
| `cli` | Command-line interface |
| `balance` | Blockchain balance fetching via mempool.space API |

## Collections

### b1000

[Bitcoin Puzzle Transaction](https://privatekeys.pw/puzzles/bitcoin-puzzle-tx) - 256 puzzles where each puzzle N has a private key in range `[2^(N-1), 2^N - 1]`.

**Solved (82):** 1-70, 75, 80, 85, 90, 95, 100, 105, 110, 115, 120, 125, 130

**Unsolved with public key (6):** 135, 140, 145, 150, 155, 160

**Unsolved (72):** 71-74, 76-79, 81-84, 86-89, 91-94, 96-99, 101-104, 106-109, 111-114, 116-119, 121-124, 126-129, 131-134, 136-139, 141-144, 146-149, 151-154, 156-159

**Empty - no funds (96):** 161-256

### gsmg

[GSMG.IO 5 BTC Puzzle](https://gsmg.io/puzzle) - Multi-phase cryptographic challenge with a single Bitcoin address.

| Address | Status | Prize |
|---------|--------|-------|
| 1GSMG1JC9wtdSwfwApgj2xcmJPAwx7prBe | Unsolved | ~1.25 BTC |

Originally 5 BTC, prize halves with each Bitcoin halving.

### hash_collision

[Peter Todd's hash collision bounties](https://bitcointalk.org/index.php?topic=293382.0) - P2SH addresses that can be claimed by finding hash collisions.

| Puzzle | Hash | Status | Prize |
|--------|------|--------|-------|
| sha1 | SHA-1 | ✅ Claimed (2017-02-23) | 2.48 BTC |
| sha256 | SHA-256 | ⏳ Unsolved | 0.277 BTC |
| ripemd160 | RIPEMD-160 | ⏳ Unsolved | 0.116 BTC |
| hash160 | HASH160 | ⏳ Unsolved | 0.100 BTC |
| hash256 | HASH256 | ⏳ Unsolved | 0.100 BTC |
| op_abs | OP_ABS | ✅ Claimed (2013-09-13) | - |

## Data

All puzzle data is embedded at compile time from TOML files in `data/`.

Each puzzle includes: address, h160 (Hash160 for P2PKH addresses), status, BTC amount, public key (if exposed), solve date (if solved), and start date (when funded).

## License

MIT
