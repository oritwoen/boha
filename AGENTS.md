# BOHA - Project Knowledge Base

**Generated:** 2026-01-04
**Commit:** 01e6826
**Branch:** main

## OVERVIEW

Rust library + CLI for crypto puzzle/bounty data. Build-time TOML→Rust codegen. Six collections: b1000 (256 puzzles), bitaps (1 SSSS puzzle), bitimage (2 puzzles), gsmg (1 puzzle), hash_collision (6 bounties), zden (15 visual puzzles).

## STRUCTURE

```
boha/
├── src/
│   ├── lib.rs              # Library entry: get(), all(), stats()
│   ├── cli.rs              # CLI binary (--features cli) - NOT main.rs
│   ├── puzzle.rs           # Puzzle, Address, Key, Status, Chain, Profile structs
│   ├── balance.rs          # Multi-chain async balance fetch
│   └── collections/        # Six collection modules with generated data
├── data/
│   ├── *.toml              # Source of truth (b1000, bitaps, bitimage, gsmg, hash_collision, zden)
│   ├── solvers.toml        # Solver definitions (referenced by ID in puzzle files)
│   └── cache/              # API response cache for scripts
├── scripts/                # Separate Cargo project - see scripts/AGENTS.md
├── build.rs                # TOML→Rust codegen
└── tests/
    ├── validation.rs       # Data validation tests
    └── cli.rs              # CLI integration tests
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add puzzle collection | `data/*.toml` + `build.rs` + `src/collections/` | Follow b1000 pattern |
| Update puzzle data | `data/*.toml` | Rebuild auto-triggers |
| Add CLI command | `src/cli.rs` | clap derive macros |
| Modify Puzzle struct | `src/puzzle.rs` + `build.rs` | Must sync both |
| Add address type | `src/puzzle.rs` (kind field) | P2PKH/P2SH/P2WPKH/P2WSH/P2TR |
| Add chain support | `src/puzzle.rs` + `src/balance.rs` | Chain enum + API integration |
| Fetch/update data | `scripts/src/bin/` | `cargo run -p scripts --bin <name>` |

## CODE MAP

| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `get(id)` | fn | lib.rs:29 | Universal puzzle lookup by ID |
| `all()` | fn | lib.rs:56 | Iterator over all puzzles |
| `stats()` | fn | lib.rs:77 | Aggregate statistics |
| `Puzzle` | struct | puzzle.rs | Core data type (16 fields) |
| `Address` | struct | puzzle.rs | value, chain, kind, hash160, witness_program |
| `Key` | struct | puzzle.rs | hex, wif, seed, bits, shares |
| `Status` | enum | puzzle.rs | Solved/Unsolved/Claimed/Swept |
| `Chain` | enum | puzzle.rs | Bitcoin/Ethereum/Litecoin/Monero/Decred |
| `Seed` | struct | puzzle.rs | BIP39: phrase, path, xpub, entropy |
| `Shares` | struct | puzzle.rs | SSSS: threshold, total, shares[] |
| `Profile` | struct | puzzle.rs | Social/web profile: name, url |
| `Author` | struct | puzzle.rs | name, addresses[], profiles[] |
| `Solver` | struct | puzzle.rs | name, addresses[], profiles[] |

## BUILD-TIME CODEGEN

**Non-standard pattern**: Puzzle data in `data/*.toml` → compiled to Rust via `build.rs`.

```
data/*.toml        ──build.rs──>  $OUT_DIR/*_data.rs  ──include!()──>  src/collections/*.rs
data/solvers.toml  ──build.rs──>  (solver references resolved during codegen)
```

- `cargo:rerun-if-changed` triggers rebuild on TOML changes
- Generated: `static PUZZLES: &[Puzzle] = &[...]`
- build.rs validates: key bits match hex, WIF↔hex consistency
- Solvers: defined once in `solvers.toml`, referenced by ID in puzzle files

## FEATURES

| Feature | Adds | Key deps |
|---------|------|----------|
| `cli` | Binary at `src/cli.rs`, output formats | clap, tabled, owo-colors, human-panic |
| `balance` | Multi-chain async fetch (BTC/ETH/DCR) | reqwest, tokio |

## CONVENTIONS

- **IDs**: `collection/identifier` (e.g., `b1000/66`, `bitimage/kitten`); exceptions: `gsmg`, `bitaps` (no slash)
- **Static data**: All `&'static` - no heap allocation
- **Address types**: P2PKH (legacy), P2SH (script), P2WPKH/P2WSH (SegWit), P2TR (Taproot)
- **Optional fields**: `Option<T>` for missing data
- **Solver vs Claimer**: Solver is who revealed/found the key (the "solution"). Claimer is who swept the funds. These may be different people - both are worth tracking.

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
cargo run --features cli -- show b1000/90
cargo run --features cli -- balance b1000/71
```

## TESTING

Data-driven validation (121 tests total):
- **validation.rs** (77): Cryptographic checks (h160, script_hash), range validation, format checks
- **cli.rs** (44): Integration tests via assert_cmd

## NOTES

- b1000 puzzle #N: private key in `[2^(N-1), 2^N - 1]`
- b1000 puzzles 1-2: `pre_genesis = true` (claimed before puzzle creation 2015-01-15)
- bitaps: Shamir Secret Sharing - 2 of 3 shares published, third unknown
- bitimage: Keys derived from files using SHA256(Base64(file)) as BIP39 entropy
- hash_collision: Peter Todd's P2SH bounties for finding hash collisions
- zden: Visual puzzles - keys encoded in images/animations
- Balances: mempool.space (BTC), Etherscan (ETH), dcrdata (DCR)
