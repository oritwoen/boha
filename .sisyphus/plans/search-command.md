# CLI: Add `search` command (Issue #89)

## Context

### Original Request
GitHub Issue #89: Add a `boha search <query>` command to search puzzles by various criteria.

### Interview Summary
**Key Discussions**:
- Search main identifier fields (id, address, pubkey, key, solver, transactions.txid, chain names) - NOT all fields like assets, profiles, dates
- Show which field matched the query
- Sort results by relevance (number of matching fields, position in string)
- Support `--exact`, `--case-sensitive`, `--limit`, `--collection` flags
- TDD approach with tests first

**Research Findings**:
- CLI uses clap derive macros in `src/cli.rs`
- `output_puzzles()` handles all output formats - needs extension for matched fields
- `cmd_list()` at line 771 shows filtering pattern
- Test infrastructure exists in `tests/cli.rs` with `assert_cmd` and `predicates`

### Metis Review
**Identified Gaps** (addressed):
- Explicit list of searchable fields: confirmed all fields including txids and chain names
- Display format for matched fields: table column + JSON/YAML property
- Edge cases: empty/whitespace query → error
- Added `--collection` flag for filtering

---

## Work Objectives

### Core Objective
Add `boha search <query>` command that searches all puzzle fields and displays results with matched field information, sorted by relevance.

### Concrete Deliverables
- New `Search` variant in `Commands` enum in `src/cli.rs`
- New `cmd_search()` function in `src/cli.rs`
- Extended output structs to include matched fields
- CLI integration tests in `tests/cli.rs`

### Definition of Done
- [x] `boha search 1BgGZ` returns puzzles matching that address prefix
- [x] `boha search sha256` returns `hash_collision/sha256` puzzle
- [x] `boha search --exact b1000/66` returns only exact ID match
- [x] `boha search --collection zden level` searches only zden collection
- [x] `boha search ""` returns error
- [x] All output formats (table/json/yaml/csv/jsonl) include matched field info
- [x] Results sorted by relevance
- [x] All tests pass: `cargo test --all-features`

### Must Have
- Substring search (case-insensitive by default)
- Search across main identifier fields only: id, address.value, address.hash160, address.witness_program, pubkey.value, key.hex, key.wif.encrypted, key.wif.decrypted, key.seed.phrase, key.mini, solver.name, solver.addresses[], transactions[].txid, chain name
- NOT searched: address.kind, redeem_script, tx.date, profiles, assets, entropy, shares (these are metadata, not identifiers)
- Flags: `--exact`, `--case-sensitive`, `--limit N`, `--collection <name>`
- Display matched fields in output
- Sort by relevance (descending)
- Support all output formats via global `-o` flag

### Must NOT Have (Guardrails)
- NO regex support - substring only
- NO fuzzy matching - exact substring only
- NO new crate dependencies
- NO field-specific search flags (e.g., `--field address`)
- NO over-engineered relevance scoring (simple: count of matched fields + position)

### Design Decisions
- **SearchResult wrapper struct**: YES - minimal wrapper to hold puzzle + matched_fields (required for output)
- **relevance_score visibility**: INTERNAL ONLY - used for sorting, NOT serialized to output (use `#[serde(skip)]`)
- **--collection unknown value**: ERROR - print error and exit(1), do NOT fall back to all
- **--collection accepted values**: `b1000`, `ballet`, `bitaps`, `bitimage`, `gsmg`, `hash_collision` (alias `peter_todd`), `zden`, `all`
- **Matched field naming**: Use dot notation, deduplicate, sort alphabetically in output
- **No-results output**: Search-specific UX (differs from `list`): For `table` format print "No puzzles found matching '<query>'" to stderr (exit 0); for machine-readable formats (`json`, `yaml`, `csv`, `jsonl`) output empty representation (don't corrupt output with messages)
- **Relevance score calculation**: Use `100usize.saturating_sub(position)` to avoid underflow for long strings

### Matched Field Labels (canonical list)
| Field | Label |
|-------|-------|
| `puzzle.id` | `id` |
| `puzzle.address.value` | `address.value` |
| `puzzle.address.hash160` | `address.hash160` |
| `puzzle.address.witness_program` | `address.witness_program` |
| `puzzle.pubkey.value` | `pubkey.value` |
| `puzzle.key.hex` | `key.hex` |
| `puzzle.key.wif.encrypted` | `key.wif.encrypted` |
| `puzzle.key.wif.decrypted` | `key.wif.decrypted` |
| `puzzle.key.seed.phrase` | `key.seed.phrase` |
| `puzzle.key.mini` | `key.mini` |
| `puzzle.solver.name` | `solver.name` |
| `puzzle.solver.addresses[*]` | `solver.addresses` (single label even if multiple match) |
| `puzzle.transactions[*].txid` | `transactions.txid` (single label even if multiple match) |
| `puzzle.chain.name()` | `chain` |

**Rules**:
- Multiple matches in array fields (solver.addresses, transactions) → single label, not duplicated
- Output sorted alphabetically: `["address.value", "chain", "id"]`

### CSV Limitation
The `csv` crate does NOT support `#[serde(flatten)]` with nested structs. For CSV output, use a separate flat `SearchCsvRow` struct that manually extracts fields from `Puzzle`. This is consistent with existing `StatsCsvRow` pattern in `src/cli.rs:170-213`.

---

## Verification Strategy (MANDATORY)

### Test Decision
- **Infrastructure exists**: YES (`tests/cli.rs` with `assert_cmd`)
- **User wants tests**: YES (TDD)
- **Framework**: `assert_cmd` + `predicates` (existing)

### TDD Workflow
Each TODO follows RED-GREEN-REFACTOR:
1. **RED**: Write failing test first
2. **GREEN**: Implement minimum code to pass
3. **REFACTOR**: Clean up while keeping green

---

## Task Flow

```
Task 1 (tests) → Task 2 (Search struct) → Task 3 (SearchResult struct) → Task 4 (search logic) → Task 5 (output) → Task 6 (cmd_search) → Task 7 (wire up)
```

## Parallelization

| Task | Depends On | Reason |
|------|------------|--------|
| 1 | - | Tests first (TDD) |
| 2 | 1 | Need tests to verify |
| 3 | 1 | Need tests to verify |
| 4 | 2, 3 | Uses Search struct and SearchResult |
| 5 | 3 | Uses SearchResult |
| 6 | 4, 5 | Uses search logic and output |
| 7 | 6 | Wires up cmd_search |

---

## TODOs

- [x] 1. Write failing CLI tests for search command

  **What to do**:
  - Add `mod search` in `tests/cli.rs`
  - Write tests for:
    - Basic substring search: `boha search 1BgGZ` → matches b1000/1 (address starts with 1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH)
    - Exact match: `boha search --exact b1000/66` → only exact ID match
    - Case insensitive (default): `boha search gsmg` → matches gsmg puzzle
    - Case insensitive uppercase: `boha search GSMG` → also matches gsmg puzzle  
    - Case sensitive no match: `boha search --case-sensitive GSMG` → no match (id is lowercase "gsmg")
    - Case sensitive match: `boha search --case-sensitive gsmg` → matches gsmg
    - Collection filter: `boha search --collection zden level` → only zden results (zden has "Level X" IDs)
    - Collection unknown error: `boha search --collection nonexistent test` → exit code 1
    - Limit: `boha search --limit 3 1` → max 3 results
    - Empty query error: `boha search ""` → exit code 1
    - Whitespace query error: `boha search "  "` → exit code 1
    - JSON output includes matched_fields: `boha -o json search sha256` → has "matched_fields" array
    - No results (table): `boha search xyznonexistent123456` → stderr contains "No puzzles found"
    - No results (json): `boha -o json search xyznonexistent123456` → outputs `[]`

  **Must NOT do**:
  - Don't test implementation details, only CLI behavior
  - Don't mock anything - use real CLI

  **Parallelizable**: NO (first task)

  **References**:
  - `tests/cli.rs:1-50` - Test setup pattern with `boha()` helper and `NO_COLOR` env
  - `tests/cli.rs:52-90` - Example test module structure (`mod stats`)
  - `tests/cli.rs` - Predicates usage: `predicate::str::contains()`, `predicate::str::is_empty().not()`

  **Acceptance Criteria**:
  - [ ] Test file compiles: `cargo test --all-features --no-run`
  - [ ] Tests fail with "unrecognized subcommand" or similar (command doesn't exist yet)
  - [ ] At least 10 test cases covering all flags and edge cases

  **Commit**: YES
  - Message: `test(cli): add failing tests for search command`
  - Files: `tests/cli.rs`
  - Pre-commit: `cargo test --all-features --no-run`

---

- [x] 2. Add `Search` command struct to CLI

  **What to do**:
  - Add `Search` variant to `Commands` enum with fields:
    ```rust
    Search {
        /// Search query (required)
        query: String,
        
        /// Require exact match
        #[arg(long)]
        exact: bool,
        
        /// Case-sensitive search
        #[arg(long, name = "case-sensitive")]
        case_sensitive: bool,
        
        /// Limit number of results
        #[arg(long)]
        limit: Option<usize>,
        
        /// Filter by collection
        #[arg(long)]
        collection: Option<String>,
    }
    ```
  - Add doc comment: `/// Search puzzles by query`

  **Must NOT do**:
  - Don't implement `cmd_search()` yet - just the struct
  - Don't add any new dependencies

  **Parallelizable**: NO (depends on 1)

  **References**:
  - `src/cli.rs:48-94` - `Commands` enum with existing variants
  - `src/cli.rs:51-69` - `List` variant structure with `#[arg()]` attributes
  - `src/cli.rs:61-62` - Example of `name` attribute for kebab-case flags

  **Acceptance Criteria**:
  - [ ] `cargo build --features cli` compiles
  - [ ] `cargo run --features cli -- search --help` shows help with all flags
  - [ ] `cargo run --features cli -- search test` runs (may panic, but parses args)

  **Commit**: YES
  - Message: `feat(cli): add Search command struct`
  - Files: `src/cli.rs`
  - Pre-commit: `cargo build --features cli`

---

- [x] 3. Create `SearchResult` wrapper struct for output

  **What to do**:
  - Create struct to wrap puzzle with matched fields info:
    ```rust
    #[derive(Serialize)]
    struct SearchResult<'a> {
        #[serde(flatten)]
        puzzle: &'a Puzzle,
        matched_fields: Vec<&'static str>,
        #[serde(skip)]  // Internal only - used for sorting, not exposed in output
        relevance_score: usize,
    }
    ```
  - Create `SearchTableRow` struct for table output:
    ```rust
    #[derive(Tabled)]
    struct SearchTableRow {
        #[tabled(rename = "ID")]
        id: String,
        #[tabled(rename = "Chain")]
        chain: String,
        #[tabled(rename = "Address")]
        address: String,
        #[tabled(rename = "Status")]
        status: String,
        #[tabled(rename = "Matched")]
        matched: String,
    }
    ```
  - Create `SearchCsvRow` struct for CSV output (flat, no serde flatten - csv crate limitation):
    ```rust
    #[derive(Serialize)]
    struct SearchCsvRow {
        id: String,
        chain: String,
        address: String,
        status: String,
        matched_fields: String,  // semicolon-separated
    }
    ```

  **Must NOT do**:
  - Don't modify existing `PuzzleTableRow` - create new struct
  - Don't implement conversion logic yet

  **Parallelizable**: NO (depends on 1)

  **References**:
  - `src/cli.rs:96-110` - `PuzzleTableRow` struct with `#[derive(Tabled)]`
  - `src/cli.rs:149-156` - `RangeOutput` struct with `#[derive(Serialize)]`
  - `src/puzzle.rs:260-277` - `Puzzle` struct fields

  **Acceptance Criteria**:
  - [ ] `cargo build --features cli` compiles
  - [ ] Structs have correct derives (Serialize, Tabled)

  **Commit**: YES
  - Message: `feat(cli): add SearchResult and SearchTableRow structs`
  - Files: `src/cli.rs`
  - Pre-commit: `cargo build --features cli`

---

- [x] 4. Implement search matching logic

  **What to do**:
  - Create function to check if puzzle matches query:
    ```rust
    fn puzzle_matches(
        puzzle: &Puzzle,
        query: &str,
        exact: bool,
        case_sensitive: bool,
    ) -> Option<(Vec<&'static str>, usize)>
    ```
  - Returns `Some((matched_fields, relevance_score))` if matches, `None` otherwise
  - Search these fields (in order - affects relevance):
    1. `id`
    2. `address.value`
    3. `address.hash160`
    4. `address.witness_program`
    5. `pubkey.value` (if Some)
    6. `key.hex` (if Some)
    7. `key.wif.encrypted` (if Some)
    8. `key.wif.decrypted` (if Some)
    9. `key.seed.phrase` (if Some)
    10. `key.mini` (if Some)
    11. `solver.name` (if Some)
    12. `solver.addresses[]` (if Some)
    13. `transactions[].txid` (each)
    14. `chain.name()` (e.g., "Bitcoin", "Ethereum")
  - Relevance score = number of matched fields × 100 + 100usize.saturating_sub(position of first match in first matched field)
  - For `--exact`: use `==` instead of `contains()`
  - For case-insensitive: compare `to_lowercase()` versions

  **Must NOT do**:
  - No regex
  - No fuzzy matching
  - No external dependencies

  **Parallelizable**: NO (depends on 2, 3)

  **References**:
  - `src/cli.rs:791-798` - Filter chain pattern in `cmd_list()`
  - `src/puzzle.rs:260-277` - Puzzle struct with all fields
  - `src/puzzle.rs:186-199` - Key struct with optional fields
  - `src/puzzle.rs:244-252` - Solver struct

  **Acceptance Criteria**:
  - [ ] `cargo build --features cli` compiles
  - [ ] Unit test: `puzzle_matches(b1000_1, "1BgGZ", false, false)` returns Some with "address.value"
  - [ ] Unit test: `puzzle_matches(b1000_1, "NONEXISTENT", false, false)` returns None

  **Commit**: YES
  - Message: `feat(cli): implement puzzle search matching logic`
  - Files: `src/cli.rs`
  - Pre-commit: `cargo test --all-features`

---

- [x] 5. Implement search results output functions

  **What to do**:
  - Create `output_search_results()` function:
    ```rust
    fn output_search_results(results: &[SearchResult], format: OutputFormat)
    ```
  - For `Table`: use `SearchTableRow`, show "Matched" column with comma-separated field names
  - For `Json`: serialize `SearchResult` with `matched_fields` array
  - For `Jsonl`: one `SearchResult` per line
  - For `Yaml`: serialize `SearchResult` array
  - For `Csv`: use separate flat `SearchCsvRow` struct (NOT `SearchResult` with flatten - csv crate doesn't support `serde(flatten)`), include `matched_fields` as semicolon-separated string
  - For empty results:
    - `Table`: print "No puzzles found matching '<query>'" to stderr
    - `Json`/`Yaml`: output empty array `[]`
    - `Jsonl`: output nothing (no lines)
    - `Csv`: output header only (no data rows)

  **Must NOT do**:
  - Don't modify existing `output_puzzles()` function
  - Don't create redundant code - extract helpers if needed

  **Parallelizable**: NO (depends on 3)

  **References**:
  - `src/cli.rs:216-256` - `output_puzzles()` function pattern
  - `src/cli.rs:218-236` - Table output with `tabled` crate
  - `src/cli.rs:237-254` - JSON/YAML/CSV output patterns

  **Acceptance Criteria**:
  - [ ] `cargo build --features cli` compiles
  - [ ] Table output shows "Matched" column
  - [ ] JSON output includes `matched_fields` array in each result

  **Commit**: YES
  - Message: `feat(cli): implement search results output`
  - Files: `src/cli.rs`
  - Pre-commit: `cargo build --features cli`

---

- [x] 6. Implement `cmd_search()` function

  **What to do**:
  - Create main search command handler:
    ```rust
    fn cmd_search(
        query: &str,
        exact: bool,
        case_sensitive: bool,
        limit: Option<usize>,
        collection: Option<&str>,
        format: OutputFormat,
    )
    ```
  - Validate query: if empty or whitespace only, print error and exit(1)
  - Validate collection: if specified and unknown, print error and exit(1) (do NOT fall back to all)
  - Get puzzles iterator based on collection filter (like `cmd_list`)
  - For each puzzle, call `puzzle_matches()`
  - Collect matches into `Vec<SearchResult>`
  - Sort by `relevance_score` descending
  - Apply limit if specified
  - Call `output_search_results()`

  **Must NOT do**:
  - Don't add new collection handling - reuse pattern from `cmd_list`
  - Don't over-optimize - linear scan is fine for 284 puzzles

  **Parallelizable**: NO (depends on 4, 5)

  **References**:
  - `src/cli.rs:771-801` - `cmd_list()` function structure
  - `src/cli.rs:780-789` - Collection matching pattern
  - `src/cli.rs:820-825` - Error handling pattern with colored output

  **Acceptance Criteria**:
  - [ ] `cargo build --features cli` compiles
  - [ ] `cargo run --features cli -- search ""` prints error, exits 1
  - [ ] `cargo run --features cli -- search 1BgGZ` returns results

  **Commit**: YES
  - Message: `feat(cli): implement cmd_search function`
  - Files: `src/cli.rs`
  - Pre-commit: `cargo test --all-features`

---

- [x] 7. Wire up search command in main/run functions

  **What to do**:
  - Add `Commands::Search` match arm in `run_sync()` (line 977-1005)
  - Add `Commands::Search` match arm in `run()` for non-balance feature (line 1008-1035)
  - Call `cmd_search()` with all parameters

  **Must NOT do**:
  - Don't add async handling - search is sync operation
  - Don't modify balance feature code paths

  **Parallelizable**: NO (depends on 6)

  **References**:
  - `src/cli.rs:976-1005` - `run_sync()` with match arms
  - `src/cli.rs:1007-1035` - `run()` for non-balance feature
  - `src/cli.rs:978-994` - Example match arm for `Commands::List`

  **Acceptance Criteria**:
  - [ ] `cargo build --features cli` compiles
  - [ ] `cargo build --features cli,balance` compiles
  - [ ] All search tests pass: `cargo test --all-features`
  - [ ] Manual test: `cargo run --features cli -- search 1BgGZ` shows results with "Matched" column

  **Commit**: YES
  - Message: `feat(cli): wire up search command`
  - Files: `src/cli.rs`
  - Pre-commit: `cargo test --all-features`

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 1 | `test(cli): add failing tests for search command` | tests/cli.rs | `cargo test --all-features --no-run` |
| 2 | `feat(cli): add Search command struct` | src/cli.rs | `cargo build --features cli` |
| 3 | `feat(cli): add SearchResult and SearchTableRow structs` | src/cli.rs | `cargo build --features cli` |
| 4 | `feat(cli): implement puzzle search matching logic` | src/cli.rs | `cargo test --all-features` |
| 5 | `feat(cli): implement search results output` | src/cli.rs | `cargo build --features cli` |
| 6 | `feat(cli): implement cmd_search function` | src/cli.rs | `cargo test --all-features` |
| 7 | `feat(cli): wire up search command` | src/cli.rs | `cargo test --all-features` |

---

## Success Criteria

### Verification Commands
```bash
# All tests pass
cargo test --all-features

# Build succeeds
cargo build --release --features cli,balance

# Clippy clean
cargo clippy --all-features -- -D warnings

# Manual verification
cargo run --features cli -- search 1BgGZ
cargo run --features cli -- search --exact b1000/66
cargo run --features cli -- search --collection zden level
cargo run --features cli -- -o json search sha256
```

### Final Checklist
- [x] All 7 tasks completed
- [x] All tests pass
- [x] No clippy warnings
- [x] Search works with all output formats
- [x] Matched fields displayed correctly
- [x] Results sorted by relevance
- [x] Empty/whitespace query returns error
- [x] `--exact`, `--case-sensitive`, `--limit`, `--collection` flags work
