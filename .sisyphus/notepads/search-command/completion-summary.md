# Search Command - Completion Summary

**Date**: 2026-01-13
**Session**: ses_448e4c3eeffexxOcBmWXbEbGvD
**Plan**: search-command.md

## Status: ✅ COMPLETE

All 7 tasks completed successfully following TDD workflow (RED-GREEN-REFACTOR).

## Commits

1. `f7bb859` - test(cli): add failing tests for search command
2. `252caa3` - feat(cli): add Search command struct
3. `41f8086` - feat(cli): add SearchResult and SearchTableRow structs
4. `327e814` - feat(cli): implement puzzle search matching logic
5. `858afa1` - feat(cli): implement search results output
6. `e9c08ae` - feat(cli): implement cmd_search function
7. `3512f62` - feat(cli): wire up search command

## Test Results

- **Total tests**: 173 passed, 0 failed
- **Search tests**: 14/14 passed
- **Clippy**: Clean (no warnings with -D warnings)
- **Build**: Success (both cli and cli,balance features)

## Features Implemented

### Core Functionality
- Substring search (case-insensitive by default)
- Search across 14 fields: id, address.value, address.hash160, address.witness_program, pubkey.value, key.hex, key.wif.encrypted, key.wif.decrypted, key.seed.phrase, key.mini, solver.name, solver.addresses[], transactions[].txid, chain name

### Flags
- `--exact`: Exact match instead of substring
- `--case-sensitive`: Case-sensitive search
- `--limit N`: Limit number of results
- `--collection <name>`: Filter by collection (b1000, ballet, bitaps, bitimage, gsmg, hash_collision, zden, all)

### Output Formats
- Table: Rounded table with "Matched" column showing comma-separated matched fields
- JSON: Array with `matched_fields` property
- JSONL: One result per line with `matched_fields`
- YAML: Array with `matched_fields`
- CSV: Flat format with semicolon-separated matched_fields column

### Edge Cases Handled
- Empty query → Error message, exit 1
- Whitespace-only query → Error message, exit 1
- Unknown collection → Error message, exit 1
- No results (table) → stderr message "No puzzles found matching '<query>'"
- No results (json/yaml/csv/jsonl) → Empty output (no corruption)

## Relevance Scoring

Results sorted by relevance score:
- Score = (number of matched fields × 100) + (100 - position of first match)
- Uses `saturating_sub` to avoid underflow
- Descending order (highest relevance first)

## Manual QA Verified

✅ Basic search: `boha search 1BgGZ` → Returns b1000/1 with "Matched: address.value"
✅ Exact match: `boha search --exact b1000/66` → Returns only b1000/66
✅ Case-sensitive: `boha search --case-sensitive GSMG` → No results (correct)
✅ Collection filter: `boha search --collection zden level` → 8 zden results
✅ Limit: `boha search --limit 2 1` → Max 2 results
✅ Unknown collection: `boha search --collection nonexistent test` → Error
✅ Empty query: `boha search ""` → Error
✅ JSON output: `boha -o json search sha256` → Has matched_fields array
✅ CSV output: `boha -o csv search bitcoin` → Correct format with semicolon-separated fields

## Technical Notes

### CSV Limitation
The `csv` crate does NOT support `#[serde(flatten)]` with nested structs. Solution: Created separate flat `SearchCsvRow` struct that manually extracts fields from `Puzzle`. This follows the existing `StatsCsvRow` pattern.

### Relevance Score Visibility
`relevance_score` is internal only - marked with `#[serde(skip)]` to prevent serialization in JSON/YAML/JSONL output. Used only for sorting.

### Collection Validation
Unknown collection names return error (exit 1) instead of falling back to "all". This differs from `cmd_list` behavior but provides clearer UX.

### No-Results Behavior
Search-specific UX (differs from `list`):
- Table format: prints message to stderr (exit 0)
- Machine formats: output empty representation (no messages)

## Definition of Done - All Met

- [x] `boha search 1BgGZ` returns puzzles matching that address prefix
- [x] `boha search sha256` returns `hash_collision/sha256` puzzle
- [x] `boha search --exact b1000/66` returns only exact ID match
- [x] `boha search --collection zden level` searches only zden collection
- [x] `boha search ""` returns error
- [x] All output formats include matched field info
- [x] Results sorted by relevance
- [x] All tests pass

## Files Modified

- `tests/cli.rs`: +145 lines (14 new tests)
- `src/cli.rs`: +448 lines (structs, functions, match arms)
- `.sisyphus/notepads/search-command/learnings.md`: Documentation

## Issue Resolution

Closes #89 - CLI: add `search` command for finding puzzles
