# BOHA - Project Knowledge Base

**Generated:** 2025-12-31
**Commit:** 08a8987
**Branch:** main

## OVERVIEW

Rust library + CLI for crypto puzzle/bounty data. Build-time TOML→Rust codegen. Three collections: b1000 (256 puzzles), gsmg (1 puzzle), hash_collision (6 bounties).

## STRUCTURE

```
boha/
├── src/
│   ├── lib.rs              # Library entry: get(), all(), stats()
│   ├── cli.rs              # CLI binary (--features cli) - NOT main.rs
│   ├── puzzle.rs           # Puzzle, Status, Chain, KeySource, AddressType
│   ├── balance.rs          # Async balance fetch (--features balance)
│   └── collections/
│       ├── b1000.rs        # 256 puzzles, includes generated code
│       ├── gsmg.rs         # Single puzzle
│       └── hash_collision.rs # 6 bounties
├── data/
│   ├── b1000.toml          # Source of truth for b1000
│   ├── gsmg.toml           # Source of truth for gsmg
│   └── hash_collision.toml # Source of truth for hash_collision
├── scripts/                # Separate Cargo project for utilities
│   └── src/bin/            # generate_h160, generate_script_hash, generate_solve_time
├── build.rs                # TOML→Rust codegen at compile time
└── tests/validation.rs     # 49 data validation tests
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add puzzle collection | `data/*.toml` + `build.rs` + `src/collections/` | Follow b1000 pattern |
| Update puzzle data | `data/*.toml` | Rebuild auto-triggers |
| Add CLI command | `src/cli.rs` | clap derive macros |
| Modify Puzzle struct | `src/puzzle.rs` + `build.rs` | Must sync both |
| Add KeySource variant | `src/puzzle.rs` + `build.rs` | Enum in both |
| Generate h160/hashes | `scripts/src/bin/` | Run with `cargo run -p scripts --bin generate_*` |

## CODE MAP

| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `get(id)` | fn | lib.rs:27 | Universal puzzle lookup by ID |
| `all()` | fn | lib.rs:49 | Iterator over all puzzles |
| `stats()` | fn | lib.rs:65 | Aggregate statistics |
| `Puzzle` | struct | puzzle.rs:117 | Core data type (13 fields) |
| `Status` | enum | puzzle.rs:43 | Solved/Unsolved/Claimed/Swept |
| `KeySource` | enum | puzzle.rs:91 | Unknown/Direct/Derived/Script |
| `Chain` | enum | puzzle.rs:8 | Bitcoin/Ethereum/Litecoin/Monero/Decred |
| `b1000::get(n)` | fn | collections/b1000.rs | Get by puzzle number |
| `b1000::solved()` | fn | collections/b1000.rs | Iterator over solved |

## BUILD-TIME CODEGEN

**Non-standard pattern**: Puzzle data in `data/*.toml` → compiled to Rust via `build.rs`.

```
data/b1000.toml  ──build.rs──>  $OUT_DIR/b1000_data.rs  ──include!()──>  src/collections/b1000.rs
```

- `cargo:rerun-if-changed` triggers rebuild on TOML changes
- Generated: `static PUZZLES: &[Puzzle] = &[...]`
- build.rs validates private key bits match declared ranges

## FEATURES

| Feature | Adds | Key deps |
|---------|------|----------|
| `cli` | Binary at `src/cli.rs`, output formats | clap, tabled, owo-colors |
| `balance` | Async balance fetch | reqwest, tokio |

## CONVENTIONS

- **IDs**: `collection/identifier` (e.g., `b1000/66`); exception: `gsmg` (no slash)
- **Static data**: All `&'static` - no heap allocation
- **Optional fields**: `Option<T>` for missing data

## ANTI-PATTERNS

- **Don't hardcode puzzle data in Rust** → Put in `data/*.toml`
- **Don't add runtime config** → All data embedded at compile time
- **Don't use non-static strings** → Must be `&'static str`

## COMMANDS

```bash
just test          # cargo test --all-features
just build         # cargo build --release --features cli,balance
just clippy        # cargo clippy --all-features -- -D warnings
just release X.Y.Z # Full release workflow

# CLI
cargo run --features cli -- stats
cargo run --features cli -- list b1000 --unsolved
cargo run --features cli -- show b1000/66
```

## TESTING

Data-driven validation (49 tests in `tests/validation.rs`):
- Cryptographic: h160 matches address, script_hash matches redeem_script
- Range: private keys within declared bit ranges
- Format: dates, hex strings, URLs
- Invariants: solved have private_key, unsolved don't have solve_time

## NOTES

- b1000 puzzle #N: private key in `[2^(N-1), 2^N - 1]`
- `key_range()` for ≤128 bits, `key_range_big()` for any size
- hash_collision: Peter Todd's P2SH bounties
- Balances via mempool.space API
- b1000 puzzles 1 and 2 have `pre_genesis = true`: transactions predate puzzle creation (2015-01-15)
  - Puzzle 1: trivial key (1) was claimed in 2013 before the puzzle existed
  - Puzzle 2: author's test transaction from 2014
