# BOHA - Project Knowledge Base

**Generated:** 2025-12-31
**Commit:** 8a772aa
**Branch:** main

## OVERVIEW

Rust library + CLI for crypto puzzle/bounty data. Build-time TOML→Rust codegen. Three collections: b1000 (256 puzzles), gsmg (1 puzzle), hash_collision (6 bounties).

## STRUCTURE

```
boha/
├── src/
│   ├── lib.rs              # Library entry, universal get()
│   ├── cli.rs              # CLI binary (--features cli)
│   ├── puzzle.rs           # Puzzle struct, Status, AddressType, Chain
│   ├── balance.rs          # Blockchain balance fetch (--features balance)
│   └── collections/
│       ├── b1000.rs        # Bitcoin 1000 puzzle (includes generated code)
│       ├── gsmg.rs         # GSMG.IO 5 BTC puzzle (single puzzle)
│       └── hash_collision.rs
├── data/
│   ├── b1000.toml          # Source of truth for b1000
│   ├── gsmg.toml           # Source of truth for gsmg
│   └── hash_collision.toml # Source of truth for hash_collision
├── build.rs                # TOML→Rust codegen at compile time
└── tests/validation.rs
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add new puzzle collection | `data/*.toml` + `build.rs` + `src/collections/` | Follow b1000 pattern |
| Update puzzle data | `data/*.toml` | Rebuild triggers automatically |
| Add CLI command | `src/cli.rs` | Uses clap derive |
| Modify Puzzle struct | `src/puzzle.rs` + `build.rs` | Must sync both |

## BUILD-TIME CODEGEN

**Non-standard pattern**: Puzzle data lives in `data/*.toml`, compiled into Rust at build time via `build.rs`.

```
data/b1000.toml  ──build.rs──>  $OUT_DIR/b1000_data.rs  ──include!()──>  src/collections/b1000.rs
```

- Change TOML → rebuild auto-triggers (`cargo:rerun-if-changed`)
- Generated code: `static PUZZLES: &[Puzzle] = &[...]`
- Each collection module uses `include!(concat!(env!("OUT_DIR"), "/xxx_data.rs"))`

## FEATURES

| Feature | Adds | Dependencies |
|---------|------|--------------|
| `cli` | Binary, output formats | clap, tabled, serde_json, serde_yaml, csv |
| `balance` | Async balance fetch | reqwest, tokio, futures |

```bash
# Library only
cargo build

# With CLI
cargo build --features cli

# Full
cargo build --features cli,balance
```

## CONVENTIONS

- **IDs**: `collection/identifier` format (e.g., `b1000/66`, `hash_collision/sha256`); exception: `gsmg` (single puzzle, no slash)
- **Status enum**: `Solved`, `Unsolved`, `Claimed`, `Swept`
- **Static data**: All puzzle data is `&'static` - no heap allocation
- **Optional fields**: Use `Option<T>` for missing data (btc, pubkey, solve_date)

## ANTI-PATTERNS

- **Don't hardcode puzzle data in Rust** → Put in `data/*.toml`
- **Don't add runtime config files** → All data embedded at compile time
- **Don't use non-static strings in Puzzle** → Must be `&'static str`

## COMMANDS

```bash
cargo build
cargo test
cargo clippy
cargo fmt

# Run CLI
cargo run --features cli -- stats
cargo run --features cli -- list b1000 --unsolved
cargo run --features cli -- show b1000/66
cargo run --features cli -- show gsmg
```

## NOTES

- b1000 puzzle #N has private key in range `[2^(N-1), 2^N - 1]`
- `Puzzle::key_range()` returns the valid key range for puzzles with `bits` field
- hash_collision puzzles are Peter Todd's hash collision bounties (P2SH)
- Balances fetched from mempool.space API (with `balance` feature)
