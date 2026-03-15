---
name: boha-cli
description: Use the boha CLI to browse, search, verify and export crypto puzzle data. Trigger when running boha commands, scripting puzzle lookups, piping puzzle data to other tools, or checking balances from the terminal. Do not use for Rust library integration - see boha skill instead.
metadata:
  author: oritwoen
  version: "0.16.0"
---

CLI for browsing crypto bounties, puzzles and challenges. Eight collections across six blockchains. Install with `cargo install boha --features cli,balance` or `paru -S boha` on Arch.

## Puzzle ID Format

IDs follow `collection/identifier` pattern. Two exceptions (`gsmg`, `bitaps`) have no slash.

| Collection | ID example | Notes |
|------------|-----------|-------|
| b1000 | `b1000/66` | Number 1-256 |
| arweave | `arweave/weave1` | Name string |
| ballet | `ballet/AA007448` | Serial number |
| bitaps | `bitaps` | Single puzzle, no slash |
| bitimage | `bitimage/kitten` | Name string |
| gsmg | `gsmg` | Single puzzle, no slash |
| hash_collision | `hash_collision/sha256` | sha1, sha256, ripemd160, hash160, hash256, op_abs |
| zden | `zden/level_1` | snake_case level names |

## Commands

### List puzzles

```bash
boha list                              # all puzzles
boha list b1000                        # single collection
boha list b1000 --unsolved             # filter by status
boha list b1000 --with-pubkey          # only with known pubkey
boha list --chain bitcoin              # filter by chain
boha list b1000 --with-transactions    # only with tx history
```

### Show puzzle details

```bash
boha show b1000/90
boha show b1000/90 --transactions      # include tx history
boha show zden/level_4 --open          # open asset in browser
```

### Statistics

```bash
boha stats
```

### Key range (b1000 only)

```bash
boha range 90                          # hex range for puzzle #90
```

### Search

```bash
boha search "sha256"
boha search "kitten" --collection bitimage
boha search "1A1zP1" --exact
boha search "puzzle" --limit 5
```

### Balance (requires `balance` feature)

```bash
boha balance b1000/71
```

### Verify private key

```bash
boha verify b1000/66                   # single puzzle
boha verify --all                      # all solved puzzles
boha verify --all --quiet; echo $?     # exit code only
```

### Collection author

```bash
boha author b1000
boha author zden
```

### Export (JSON/JSONL only)

```bash
boha export                            # full database
boha export b1000 zden                 # specific collections
boha export --unsolved                 # filter
boha export -o jsonl | jq .            # pipe to jq
boha export --compact                  # minimal output
boha export --no-authors --no-stats    # skip metadata
```

## Output Formats

Global flag `-o` works with all commands except `export` (JSON/JSONL only).

```bash
boha -o json show b1000/90
boha -o yaml stats
boha -o csv list b1000 > puzzles.csv
boha -o jsonl list b1000 --unsolved | jq .
```

| Format | Flag | Notes |
|--------|------|-------|
| table | `-o table` | Default. Colored TUI table |
| json | `-o json` | Pretty-printed |
| jsonl | `-o jsonl` | One object per line, good for piping |
| yaml | `-o yaml` | |
| csv | `-o csv` | With header row |

## Collections Overview

See [collections.md](references/collections.md) for detailed collection data.

## Limitations

- `export` only supports JSON and JSONL. Use `list` for CSV/YAML.
- `balance` requires the `balance` feature at install time.
- `range` only works for b1000 puzzles.
