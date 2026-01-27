# boha

[![Crates.io](https://img.shields.io/crates/v/boha?style=flat&colorA=130f40&colorB=474787)](https://crates.io/crates/boha)
[![Downloads](https://img.shields.io/crates/d/boha?style=flat&colorA=130f40&colorB=474787)](https://crates.io/crates/boha)
[![License](https://img.shields.io/crates/l/boha?style=flat&colorA=130f40&colorB=474787)](LICENSE)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/oritwoen/boha)

Crypto bounties, puzzles and challenges data library.

## Community

- [Bitcointalk](https://bitcointalk.org/index.php?topic=5570614) - Discussion thread

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
boha = "0.15"
```

With balance fetching:

```toml
[dependencies]
boha = { version = "0.15", features = ["balance"] }
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
boha show b1000/90
boha show gsmg
boha show hash_collision/sha256

# Show puzzle and open asset in browser
boha show zden/Level\ 4 --open

# Get key range
boha range 90

# Check balance (requires --features balance)
boha balance b1000/71

# Verify private key derives correct address
boha verify b1000/66
boha verify --all
boha verify --all --quiet; echo $?

# Export full database (JSON/JSONL only)
boha export
boha export b1000 zden
boha export --unsolved
boha export -o jsonl | jq .
boha export --compact

# Output formats (default: table)
boha -o json stats
boha -o yaml show b1000/90
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

**Note:** `export` command supports JSON and JSONL only. Use `list` command for CSV/YAML output.

### Library

```rust
use boha::{b1000, bitaps, gsmg, hash_collision, zden, Status};

let p90 = b1000::get(90).unwrap();
println!("Address: {}", p90.address.value);
println!("HASH160: {}", p90.address.hash160.unwrap());
println!("Funded: {}", p90.start_date.unwrap_or("unknown"));

let range = p90.key_range().unwrap();
println!("Range: 0x{:x} - 0x{:x}", range.start(), range.end());

if let Some(txid) = p90.claim_txid() {
    println!("Claimed in: {}", txid);
    println!("Explorer: {}", p90.chain.tx_explorer_url(txid));
}

let unsolved: Vec<_> = b1000::all()
    .filter(|p| p.status == Status::Unsolved)
    .filter(|p| p.pubkey.is_some())
    .collect();

let gsmg_puzzle = gsmg::get();
let sha256 = hash_collision::get("sha256").unwrap();
let level1 = zden::get("Level 1").unwrap();

let puzzle = boha::get("b1000/90").unwrap();
let puzzle = boha::get("gsmg").unwrap();
let puzzle = boha::get("bitaps").unwrap();
let puzzle = boha::get("bitimage/kitten").unwrap();
let puzzle = boha::get("zden/Level 1").unwrap();

// Access puzzle assets (images, hints)
if let Some(path) = puzzle.asset_path() {
    println!("Local: {}", path);
}
if let Some(url) = puzzle.asset_url() {
    println!("Remote: {}", url);
}
```

### Balance fetching (async)

```rust
use boha::{b1000, balance};

#[tokio::main]
async fn main() {
    let puzzle = b1000::get(71).unwrap();
    let bal = balance::fetch(puzzle.address.value).await.unwrap();
    
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

### zden

[Zden's Visual Crypto Puzzles](https://crypto.haluska.sk/) - Artistic puzzles where private keys are encoded in images, animations, and visual patterns.

| Chain | Solved | Unsolved | Total |
|-------|--------|----------|-------|
| Bitcoin | 9 | 2 | 11 |
| Ethereum | 2 | 0 | 2 |
| Litecoin | 1 | 0 | 1 |
| Decred | 1 | 0 | 1 |

**Unsolved:** Level 5, Level HALV

### bitaps

[Bitaps Mnemonic Challenge](https://bitaps.com/mnemonic/challenge) - Shamir Secret Sharing Scheme (SSSS) puzzle where the original 12-word mnemonic was split into 5 shares using 3-of-5 threshold.

| Address | Status | Prize |
|---------|--------|-------|
| bc1qyjwa0tf0en4x09magpuwmt2smpsrlaxwn85lh6 | Unsolved | ~1.0 BTC |

Two of three required shares are published. Goal: break the SSSS scheme or find implementation bugs.

### bitimage

[Bitimage](https://github.com/coreyphillips/bitimage) puzzles - Bitcoin addresses derived from arbitrary files using SHA256(Base64(file)) as BIP39 entropy.

| Puzzle | Passphrase | Status | Prize |
|--------|------------|--------|-------|
| kitten | No | ✅ Solved (2019-07-09) | 0.00095 BTC |
| kitten_passphrase | Yes | ⏳ Unsolved | ~0.01 BTC |

Both puzzles use the same source file (Antonopoulos kitten tweet). The passphrase puzzle requires an unknown BIP39 passphrase.

## Data

All puzzle data is embedded at compile time from JSONC files in `data/`.

Each puzzle includes: address (with HASH160 and type), chain, status, prize, public key (if exposed), private key (if solved), key source, solve date (if solved), solve time, start date (when funded), transactions history, solver information, and assets (puzzle images, hints).

## Assets

Visual puzzle collections (zden, gsmg, bitimage) include embedded assets in `assets/` directory:

```
assets/
├── zden/           # 15 puzzle images
├── gsmg/           # puzzle.png, follow_the_white_rabbit.png
└── bitimage/       # kitten images
```

Access via library:

```rust
let puzzle = zden::get("Level 4").unwrap();
println!("{}", puzzle.asset_path().unwrap());  // assets/zden/level-4/puzzle.png
println!("{}", puzzle.asset_url().unwrap());   // https://raw.githubusercontent.com/...
```

## Related Tools

| Tool | Description |
|------|-------------|
| [vuke](https://github.com/oritwoen/vuke) | Research tool for studying vulnerable Bitcoin key generation practices. Analyze solved puzzles for weak patterns. |
| [vgen](https://github.com/oritwoen/vgen) | Bitcoin vanity address generator with regex pattern matching and GPU acceleration. |

## License

MIT
