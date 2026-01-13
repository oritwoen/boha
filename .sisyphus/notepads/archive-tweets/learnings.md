# Learnings

- `playwright` crate (octaltree/playwright-rust) is fully async; entrypoint is `Playwright::initialize().await`.
- Browser install is separate from init: call `playwright.install_chromium()?` (or `prepare()` / `install_*`).
- To reuse login without `user-data-dir`, serialize `BrowserContext::storage_state().await?` to JSON and later pass it via `browser.context_builder().storage_state(StorageState)`.
- Screenshot of a specific DOM node: `element_handle.screenshot_builder().await.screenshot().await?`.
- Page navigation is via builder: `page.goto_builder(url).wait_until(DocumentLoadState::DomContentLoaded).goto().await?`.
- Wayback Machine availability endpoint (`https://archive.org/wayback/available?url=...`) returns `closest.status` as a **string** (e.g. `"200"`), and can return `archived_snapshots: {}` for "no snapshot".
- `reqwest::Url::parse_with_params` is a clean way to avoid manual URL encoding for the Wayback `url=` query.
- Playwright screenshot builders differ:
  - `element_handle.screenshot_builder()` is async (needs `.await` before `.screenshot().await`).
  - `page.screenshot_builder()` is **not** async (no `.await` before `.screenshot().await`).

## Task 3: TOML Update Utilities (source_archives)

- Created `scripts/src/utils/source_archives.rs` with URL extraction and TOML update functions.
- `extract_twitter_urls()` checks three locations: `[metadata] source_url`, `[puzzles.assets] source_url`, and `[puzzles.key.seed.entropy.source] url`.
- Only extracts URLs containing `/status/` to filter out profile URLs (e.g., `https://x.com/bobbyclee` is ignored).
- `canonicalize_url()` normalizes `twitter.com` → `x.com` and strips query parameters.
- `update_source_archives()` uses `toml_edit::DocumentMut` to preserve TOML formatting while adding `source_archives` array entries.
- Uses `as_array_of_tables()` / `as_array_of_tables_mut()` to iterate over `[[puzzles]]` array while tracking array index.
- Module compiles cleanly with `cargo check --manifest-path scripts/Cargo.toml` (only unused-code warnings expected for utility functions).

## Task 4: archive-tweet binary

- When invoking a script with `cargo run --manifest-path scripts/Cargo.toml` from the repo root, relative paths like `../data` resolve against the **current working directory**, not the scripts crate dir; prefer deriving paths from `env!("CARGO_MANIFEST_DIR")`.
- Adding `#[allow(dead_code)]` on the per-bin `mod utils { include!(...) }` wrapper is an easy way to avoid noisy dead-code warnings without changing the shared utils file.
