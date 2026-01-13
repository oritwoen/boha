# search-command learnings

## Existing `tests/cli.rs` patterns
- Single helper `fn boha() -> assert_cmd::Command` sets `NO_COLOR=1` for stable output.
- Tests are grouped into `mod <command>` modules; each test uses `boha().args([...]).assert()...`.
- Assertions are written with `predicates` via `predicate::str::contains(...)` (and occasionally `predicate::str::is_match(...)`).

## Predicate usage examples
- Success + stdout contains:
  - `boha().arg("stats").assert().success().stdout(predicate::str::contains("Total puzzles"));`
- Failure + stderr contains:
  - `boha().args(["show", "b1000/999"]).assert().failure().stderr(predicate::str::contains("Error:"));`
- Negative assertions:
  - `...stdout(predicate::str::contains("b1000/1").not());` (via `predicates::prelude::*`)
- Exact stdout match:
  - `...stdout(predicate::str::diff("[]"));`

## Gotchas
- `assert_cmd::Command::cargo_bin` is deprecated; prefer `Command::new(env!("CARGO_BIN_EXE_boha"))` in integration tests.
- When a subcommand is missing, clap exits with code `2` and prints `error: unrecognized subcommand '...'` to stderr; this is useful to verify TDD RED-state for new commands.

## Task 2: Adding `Commands::Search` variant

### Clap derive patterns observed
- Each `Commands` enum variant uses `/// Doc comment` for help text (clap reads these for `--help` output).
- Field-level doc comments become argument descriptions: `/// Search query (required)` → shows in `boha search --help`.
- `#[arg(long)]` creates `--flag-name` from field name (e.g., `exact: bool` → `--exact`).
- `#[arg(long, name = "kebab-case")]` overrides flag name (not needed for `case_sensitive` - clap auto-converts to `--case-sensitive`).
- Optional fields use `Option<T>` (e.g., `limit: Option<usize>`, `collection: Option<String>`).

### Compilation requirement
- Adding a new `Commands` variant makes existing `match cli.command { ... }` non-exhaustive.
- To satisfy "build must compile" acceptance criteria, added placeholder `Commands::Search { .. } => todo!("...")` match arms in both:
  - `run_sync()` (line 1026) - for `#[cfg(feature = "balance")]`
  - `run()` (line 1057) - for `#[cfg(not(feature = "balance"))]`
- This allows `cargo build --features cli` to succeed and `boha search test` to parse args (then panic with "not yet implemented").
- Task 7 will replace these placeholders with actual `cmd_search()` calls.

### Verification results
- ✅ `cargo build --features cli` compiles (1 dead_code warning for `BalanceOutput` - expected without balance feature).
- ✅ `cargo run --features cli -- search --help` shows all 4 flags (`--exact`, `--case-sensitive`, `--limit`, `--collection`).
- ✅ `cargo run --features cli -- search test` parses args, then panics with `todo!("search command not implemented yet")`.
- ✅ `cargo clippy --all-features -- -D warnings` passes with no warnings.

## Task 3: Search output structs
- Added `SearchResult`, `SearchTableRow`, `SearchCsvRow` near other output structs in `src/cli.rs`.
- `SearchResult` uses `#[serde(flatten)]` for JSON/YAML/JSONL output; CSV needs a dedicated row type because the `csv` crate doesn’t support `#[serde(flatten)]`.
- `rust-analyzer` diagnostics can go stale when files are edited outside LSP; killing `rust-analyzer` (`pkill -f rust-analyzer`) forced it to reload and cleared a phantom non-exhaustive-match error.
- New output-only structs are dead-code until `search` is implemented; temporarily used `#[allow(dead_code)]` on the new structs to keep `cargo clippy --all-features -- -D warnings` green.

## Task 4: `puzzle_matches()` helper
- Implemented ordered scan across the 14 required fields, including optional subfields and deduping list fields (`solver.addresses`, `transactions.txid`).
- Matching logic: `==` for `--exact`, `contains` otherwise; case-insensitive compares `to_lowercase()` versions of both sides.
- Relevance score uses the byte index of the first match in the first matched field and `100usize.saturating_sub(position)` to avoid underflow.
- Kept the implementation comment-free to satisfy the repo’s comment/docstring hook (self-documenting structure + stable label strings).

## Task 5: `output_search_results()` helper
- Table output follows `output_puzzles()` patterns: `tabled` + `Style::rounded()`, then a `Total: N results` line.
- Empty result behavior is format-specific: table prints `No puzzles found matching '<query>'` to stderr; json/yaml print `[]` (no extra newline); jsonl prints nothing.
- CSV empty output requires manually writing headers (serde-based `csv::Writer::serialize` only emits headers when at least one row is serialized).
- `SearchResult` was made non-generic (`puzzle: &'static Puzzle`) so the required signature `fn output_search_results(results: &[SearchResult], ...)` compiles.

## Task 6: `cmd_search()` handler
- `SearchResult` requires `puzzle: &'static Puzzle`, so `cmd_search()` must collect as `Vec<&'static Puzzle>` (not `Vec<&Puzzle>`) to avoid losing the `'static` lifetime.
- Added `#[allow(dead_code)]` on `cmd_search()` to keep `cargo build --features cli` clean until Task 7 wires the command into `run()`/`run_sync()`.
- Until Task 7 replaces the `_ => todo!("search command not implemented yet")` match arm, `boha search ...` will still panic before `cmd_search()` runs.

