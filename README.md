# boha

Crypto bounties, puzzles and challenges data library.

## Installation

```toml
[dependencies]
boha = { git = "https://github.com/oritwoen/boha" }
```

With balance fetching:

```toml
[dependencies]
boha = { git = "https://github.com/oritwoen/boha", features = ["balance"] }
```

With CLI:

```bash
cargo install --git https://github.com/oritwoen/boha --features cli,balance
```

## Usage

### Library

```rust
use boha::{b1000, hash_collision, Status};

let p66 = b1000::get(66).unwrap();
println!("Address: {}", p66.address);

let range = b1000::key_range(66).unwrap();
println!("Range: 0x{:x} - 0x{:x}", range.start(), range.end());

let unsolved: Vec<_> = b1000::all()
    .filter(|p| p.status == Status::Unsolved)
    .filter(|p| p.pubkey.is_some())
    .collect();

let sha256 = hash_collision::get("sha256").unwrap();

let puzzle = boha::get("b1000/66").unwrap();
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

### CLI

```bash
# Statistics
boha stats

# List puzzles
boha list b1000
boha list b1000 --unsolved
boha list b1000 --with-pubkey

# Show puzzle details
boha show b1000/66
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

## Features

| Feature | Description |
|---------|-------------|
| `cli` | Command-line interface |
| `balance` | Blockchain balance fetching via mempool.space API |

## Collections

| Collection | Count | Description |
|------------|-------|-------------|
| b1000 | 256 | Bitcoin Puzzle Transaction (~1000 BTC) |
| hash_collision | 6 | Peter Todd's hash collision bounties |

## Data

All puzzle data is embedded at compile time from TOML files in `data/`.

## License

MIT
