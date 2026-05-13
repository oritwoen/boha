# BOHA - Project Knowledge Base

**Updated:** 2026-03-30
**Branch:** main

## OVERVIEW

Rust library + CLI for crypto puzzle/bounty data. Build-time JSONC→Rust codegen with JSON Schema validation. Ten collections: arweave (11 bounties), b1000 (256 puzzles), ballet (3 puzzles), bitaps (1 SSSS puzzle), bitimage (2 puzzles), gsmg (1 puzzle), hash_collision (6 bounties), rushwallet (30 brainwallet contest), warp (6 WarpWallet challenges), zden (15 visual puzzles).

## STRUCTURE

```
boha/
├── src/
│   ├── lib.rs              # Library entry: get(), all(), stats()
│   ├── cli.rs              # CLI binary (--features cli) - NOT main.rs
│   ├── puzzle.rs           # Puzzle, Address, Key, Status, Chain, Profile structs
│   ├── balance.rs          # Multi-chain async balance fetch (BTC/LTC/ETH)
│   ├── verify.rs           # Cryptographic key→address verification (--features cli)
│   └── collections/        # Ten collection modules with generated data
├── data/
│   ├── *.jsonc             # Source of truth (arweave, b1000, ballet, bitaps, bitimage, gsmg, hash_collision, rushwallet, warp, zden)
│   ├── solvers.jsonc       # Solver definitions (referenced by ID in puzzle files)
│   ├── schemas/            # JSON Schema files for validation
│   └── cache/              # API response cache for scripts
├── scripts/                # Separate Cargo project - see scripts/AGENTS.md
├── build.rs                # JSONC→Rust codegen
└── tests/
    ├── validation.rs       # Data validation tests
    ├── cli.rs              # CLI integration tests
    └── author_lineage.rs   # Funding source and author metadata tests
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add puzzle collection | `data/*.jsonc` + `build.rs` + `src/collections/` | Follow b1000 pattern |
| Update puzzle data | `data/*.jsonc` | Rebuild auto-triggers |
| Add CLI command | `src/cli.rs` | clap derive macros |
| Modify Puzzle struct | `src/puzzle.rs` + `build.rs` | Must sync both |
| Add address type | `src/puzzle.rs` (kind field) | P2PKH/P2SH/P2WPKH/P2WSH/P2TR |
| Add chain support | `src/puzzle.rs` + `src/balance.rs` | Chain enum + API integration |
| Fetch/update data | `scripts/src/bin/` | `cargo run -p scripts --bin <name>` |

## CODE MAP

| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `get(id)` | fn | lib.rs:36 | Universal puzzle lookup by ID |
| `all()` | fn | lib.rs:65 | Iterator over all puzzles |
| `stats()` | fn | lib.rs:88 | Aggregate statistics |
| `Puzzle` | struct | puzzle.rs | Core data type (16 fields) |
| `Address` | struct | puzzle.rs | value, chain, kind, hash160, witness_program |
| `Key` | struct | puzzle.rs | hex, wif, seed, bits, shares |
| `Status` | enum | puzzle.rs | Solved/Unsolved/Claimed/Swept/Expired |
| `Chain` | enum | puzzle.rs | Bitcoin/Ethereum/Litecoin/Monero/Decred/Arweave |
| `Seed` | struct | puzzle.rs | BIP39: phrase, path, xpub, entropy |
| `Shares` | struct | puzzle.rs | SSSS: threshold, total, shares[] |
| `Profile` | struct | puzzle.rs | Social/web profile: name, url |
| `Author` | struct | puzzle.rs | name, addresses[], profiles[] |
| `Solver` | struct | puzzle.rs | name, addresses[], profiles[] |

## BUILD-TIME CODEGEN

**Non-standard pattern**: Puzzle data in `data/*.jsonc` → compiled to Rust via `build.rs`.

```
data/*.jsonc        ──build.rs──>  $OUT_DIR/*_data.rs  ──include!()──>  src/collections/*.rs
data/solvers.jsonc  ──build.rs──>  (solver references resolved during codegen)
data/schemas/       ──editor──>    (JSON Schema validation & autocomplete)
```

- `cargo:rerun-if-changed` triggers rebuild on JSONC changes
- Generated: `static PUZZLES: &[Puzzle] = &[...]`
- build.rs validates: key bits match hex, WIF↔hex consistency
- Solvers: defined once in `solvers.jsonc`, referenced by ID in puzzle files
- JSON Schema provides editor validation and autocomplete

## FEATURES

| Feature | Adds | Key deps |
|---------|------|----------|
| `cli` | Binary at `src/cli.rs`, output formats | clap, tabled, owo-colors, human-panic |
| `balance` | Multi-chain async fetch (BTC/LTC/ETH) | reqwest, tokio |

## CONVENTIONS

- **IDs**: `collection/identifier` (e.g., `b1000/66`, `bitimage/kitten`); exceptions: `gsmg`, `bitaps` (no slash)
- **Static data**: All `&'static` - no heap allocation
- **Address types**: P2PKH (legacy), P2SH (script), P2WPKH/P2WSH (SegWit), P2TR (Taproot)
- **Optional fields**: `Option<T>` for missing data
- **Solver vs Claimer**: Solver is who revealed/found the key (the "solution"). Claimer is who swept the funds. These may be different people - both are worth tracking.

## ANTI-PATTERNS

- **Don't hardcode puzzle data in Rust** → Put in `data/*.jsonc`
- **Don't add runtime config** → All data embedded at compile time
- **Don't use non-static strings** → Must be `&'static str`

## COMMANDS

```bash
cargo test --all-features                    # all tests (254 passed, 15 ignored)
cargo test --all-features -- test_name       # single test
cargo build --release --features cli,balance # release build
cargo clippy --all-features -- -D warnings   # lint
cargo fmt --check                            # format check

# CLI dev
cargo run --features cli -- stats
cargo run --features cli -- list b1000 --unsolved
cargo run --features cli -- show b1000/90
cargo run --features cli,balance -- balance b1000/71
cargo run --features cli -- verify --all --quiet
```

## TESTING

Data-driven validation (254 tests, 3 test files):
- **validation.rs**: Cryptographic checks (h160, script_hash), range validation, format checks
- **cli.rs**: Integration tests via assert_cmd
- **author_lineage.rs**: Funding source tracking and author metadata

## NOTES

- b1000 puzzle #N: private key in `[2^(N-1), 2^N - 1]`
- b1000 puzzles 1-2: `pre_genesis = true` (claimed before puzzle creation 2015-01-15)
- ballet: Physical crypto wallet cards with BIP38 encrypted keys (Bobby Lee's challenge)
- bitaps: Shamir Secret Sharing - 2 of 3 shares published, third unknown
- bitimage: Keys derived from files using SHA256(Base64(file)) as BIP39 entropy
- hash_collision: Peter Todd's P2SH bounties for finding hash collisions
- rushwallet: Dmitri Kryptokov / Kryptokit 2014 brainwallet contest, 30 targets, derivation `sha256(passphrase)` → uncompressed P2PKH; 28 passphrases recovered locally, #26 claimed on-chain with passphrase still unknown, #30 unclaimed/unsolved; all 30 UTXOs funded by `1GShq18eb4V6uBtqgwxkmuPTUHCtyBcNYA`
- warp: Keybase WarpWallet challenges - deterministic brainwallet (scrypt+pbkdf2) security tests
- zden: Visual puzzles - keys encoded in images/animations
- arweave: Tiamat's bounties on Arweave blockchain (chronobot.io)
- Balances: mempool.space (BTC/LTC), Etherscan (ETH)
