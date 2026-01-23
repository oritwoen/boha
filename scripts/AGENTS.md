# Scripts - Data Maintenance Utilities

**Parent:** [../AGENTS.md](../AGENTS.md)

## OVERVIEW

Separate Cargo project with 8 binaries that fetch/compute data for `../data/*.jsonc` files.

## STRUCTURE

```
scripts/
├── src/
│   ├── main.rs             # fetch-start-dates
│   ├── bin/
│   │   ├── generate_h160.rs
│   │   ├── generate_script_hash.rs
│   │   ├── generate_transactions.rs
│   │   ├── add_timestamps.rs
│   │   ├── derive_pubkey_from_xpub.rs
│   │   ├── generate_wif.rs
│   │   └── extract_pubkey.rs
│   └── utils/
│       ├── mempool.rs      # Bitcoin API (mempool.space)
│       ├── etherscan.rs    # Ethereum API
│       └── dcrdata.rs      # Decred API
└── Cargo.toml
```

## UTILITIES

| Binary | Purpose | Updates |
|--------|---------|---------|
| `fetch-start-dates` | First funding date from mempool.space | `start_date` |
| `generate-h160` | HASH160 from P2PKH addresses | `address.hash160` |
| `generate-script-hash` | Script hash from redeem scripts | `address.redeem_script.hash` |
| `generate-transactions` | Full tx history from chain APIs | `transactions[]` |
| `add-timestamps` | Date → datetime conversion, solve time calculation (use `--recalculate` to force recalculation from cache) | `*.date` fields, `solve_time` |
| `derive-pubkey-from-xpub` | BIP32 pubkey derivation | `pubkey` |
| `generate-wif` | WIF format from hex private keys | `key.wif.decrypted` |
| `extract-pubkey` | Extract public keys from transactions | `pubkey` |

## COMMANDS

```bash
cargo run -p scripts --bin generate-transactions
cargo run -p scripts --bin generate-h160
cargo run -p scripts --bin add-timestamps
cargo run -p scripts --bin add-timestamps -- --recalculate  # Force recalculation from cache
```

## CONVENTIONS

- **Caching**: JSON responses in `../data/cache/` to avoid repeated API calls
- **Rate limiting**: 500ms-3s delays between requests
- **Error handling**: Skip failures, continue processing
- **JSONC editing**: Uses `serde_json` for JSON manipulation
- **Progress output**: Console logs per-puzzle status

## ANTI-PATTERNS

- **Don't skip cache check** → API rate limits will block you
- **Don't hardcode API keys** → Use `.env` file (see `.env.example`)

## NOTES

- Requires `ETHERSCAN_API_KEY` env var for Ethereum transactions
- Cache files are gitignored except for b1000/zden reference data
- Run after adding new puzzles to populate computed fields
