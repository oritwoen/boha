## [0.14.0] - 2026-01-13

### Features

- *(zden)* Add private key for Level 1 puzzle (#81)
- *(build)* WIF validation at build time (#93)

### Other

- *(zden)* Add private key for Level 2 puzzle (#85)
- *(zden)* Add Level 3 private key (#87)

### Refactor

- *(pubkey)* Rename key to value and use inline TOML tables (#83)

### Documentation

- Update AGENTS.md metadata to current commit
## [0.13.0] - 2026-01-05

### Features

- *(collection)* Add Ballet challenge (#76)
- *(data)* Add WIF format to solved puzzles (#79)
- *(release)* Update README.md version during release

### Documentation

- Sync AGENTS.md with Ballet collection addition

### Miscellaneous Tasks

- *(release)* V0.13.0
## [0.12.1] - 2026-01-05

### Bug Fixes

- *(package)* Exclude assets from crates.io package

### Miscellaneous Tasks

- *(release)* V0.12.1
## [0.12.0] - 2026-01-05

### Features

- *(solvers)* Add `retired_coder` (#71)
- *(assets)* Add puzzle assets support (#74)

### Documentation

- Add Related Tools section with vuke and vgen

### Miscellaneous Tasks

- *(release)* V0.12.0
## [0.11.0] - 2026-01-04

### Features

- *(address)* Add Address struct (#60)
- *(puzzle)* Add Key struct (#61)
- *(cli)* Add human-panic for friendly crash reports (#62)
- *(balance)* Add Ethereum API support (#63)
- *(scripts)* Add Decred API support (#64)
- *(address)* Add SegWit support (#67)
- *(address)* Add Taproot (P2TR) address type support (#68)
- *(bitaps)* Add mnemonic challenge (#69)
- *(collections)* Add bitimage puzzle collection (#70)

### Refactor

- *(solver)* Extract solvers to dedicated TOML file (#72) (#73)

### Documentation

- Sync AGENTS.md with zden collection
- Update outdated code references
- Sync AGENTS.md with recent changes

### Miscellaneous Tasks

- Add `context7.json`
- *(release)* V0.11.0
## [0.10.0] - 2026-01-02

### Features

- *(zden)* Add visual crypto puzzles (#57)
- *(data)* Add timestamps to dates (#59)

### Miscellaneous Tasks

- *(release)* V0.10.0
## [0.9.0] - 2026-01-02

### Features

- *(puzzle)* Add claim_txid accessors and explorer URLs (#43)
- *(puzzle)* Add solver information for solved puzzles (#51)

### Testing

- Add pubkey to h160 validation (#41)
- *(balance)* Add coverage for balance feature (#42)
- Add private key to address derivation verification (#45)

### Miscellaneous Tasks

- *(release)* V0.9.0
## [0.8.0] - 2026-01-02

### Features

- *(puzzle)* Add transaction history (#34)

### Miscellaneous Tasks

- *(release)* V0.8.0
## [0.7.0] - 2026-01-02

### Features

- *(puzzle)* Add KeySource enum for key derivation semantics (#33)
- *(author)* Add Author struct for collections (#36)

### Documentation

- Update AGENTS.md with code map and testing info

### Miscellaneous Tasks

- Add `deepwiki` badge
- Ad `deepwiki` badge
- *(release)* V0.7.0
## [0.6.0] - 2025-12-31

### Features

- *(puzzle)* Generalize key_range (#30)
- *(puzzle)* Add solve_time field (#32)

### Miscellaneous Tasks

- *(release)* V0.6.0
## [0.5.0] - 2025-12-31

### Features

- *(puzzle)* Add h160 field for P2PKH addresses (#17)
- *(puzzle)* Add script_hash for P2SH (#22)
- *(ci)* Integrate autofix.ci for automatic formatting (#23)

### Refactor

- *(data)* Rename btc field to prize in TOML files (#25)

### Documentation

- Add gsmg collection to AGENTS.md

### Miscellaneous Tasks

- *(release)* V0.5.0
## [0.4.0] - 2025-12-31

### Features

- *(puzzle)* Add Chain enum (#15)
- *(puzzle)* Add pubkey_format field (#16)

### Miscellaneous Tasks

- *(release)* V0.4.0
## [0.3.0] - 2025-12-31

### Features

- *(collections)* Add GSMG puzzle collection (#10)

### Miscellaneous Tasks

- *(release)* V0.3.0
## [0.2.0] - 2025-12-30

### Features

- *(puzzle)* Add start_date field (#7)
- *(puzzle)* Add source_url field (#8)

### Miscellaneous Tasks

- *(release)* V0.2.0
## [0.1.0] - 2025-12-30

### Bug Fixes

- Limit version sed to package section only

### Other

- Crypto bounties, puzzles and challenges data library
- Update balances from mempool.space API

### Refactor

- Move hash_collision data to TOML source of truth

### Documentation

- Reorder README sections - CLI before Library
- Add boha list without args to README
- Add detailed puzzle list to Collections section
- Fix b1000 puzzle status breakdown
- Add CONTRIBUTING.md
- Add AGENTS.md project knowledge base

### Styling

- Fix formatting in cli.rs

### Miscellaneous Tasks

- Track Cargo.lock for reproducible builds
- *(release)* V0.1.0
