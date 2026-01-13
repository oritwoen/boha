# Archive Source Tweets (Issue #77)

## Context

### Original Request
Archive key tweets as markdown with YAML frontmatter + screenshots to preserve ephemeral social media sources. X/Twitter blocks automated fetching and accounts can be deleted/banned.

### Interview Summary
**Key Discussions**:
- Directory structure: `assets/{collection}/{puzzle_name}/source_archive.md` (alongside existing puzzle assets)
- Git LFS: User enabled LFS, needs `.gitattributes` configuration
- Naming: snake_case convention
- Playwright automation with manual X session login
- Full tweet content + screenshot in markdown
- `source_archives` as array field in TOML
- Script in `scripts/src/bin/archive_tweet.rs`
- Automatic TOML update when archiving
- Wayback Machine as fallback when tweet unavailable
- Shared archive for puzzles referencing same URL

**Research Findings**:
- ballet.toml: 1 unique tweet (Bobby Lee) - all 3 puzzles share same `source_url`
- bitimage.toml: 1 unique tweet (aantonop from 2015) - high risk of deletion
- URL locations: `[metadata] source_url`, `[puzzles.assets] source_url`, `[puzzles.key.seed.entropy.source] url`
- Existing scripts use `toml_edit` to preserve formatting
- Scripts pattern: fetch → cache → process → update TOML

### Metis Review
**Identified Gaps** (addressed):
- Git LFS not configured: Added `.gitattributes` setup task
- Tweet availability risk: Added Wayback Machine fallback
- Path structure inconsistency: Confirmed `assets/{collection}/{puzzle_name}/` pattern
- Session persistence: Will use `scripts/.playwright-state.json` (gitignored)
- TOML field type: Confirmed array `source_archives = [...]`

### Momus Review (Round 1)
**Critical corrections applied**:
- Git LFS pattern narrowed: `assets/**/source_archive.png` (not all PNGs - avoids affecting existing zden PNGs)
- Playwright crate: Use `playwright = "0.0.20"` from crates.io
- Gitignore location: Add to root `.gitignore` (not `scripts/.gitignore` which doesn't exist)
- URL canonicalization: Normalize `twitter.com` ↔ `x.com`, strip tracking params
- Wayback API: Handle `archived_snapshots: {}` (empty = no snapshot), `status` is string not int
- Profile URLs: Must ignore author profile URLs like `https://x.com/bobbyclee` (only archive status URLs)

### Momus Review (Round 2)
**Critical corrections applied**:
- Package name: Use `-p boha-scripts` (not `-p scripts`) in all cargo commands
- Playwright dependency: Exact Cargo.toml stanza provided
- Wayback screenshot strategy: Use Playwright to render archived page, same as live tweets
- Shared archive rule: Store under FIRST puzzle (in TOML file order) that references the URL
- Fixed line reference: `data/zden.toml:8` (not line 4)
- Added `scripts/src/utils/mod.rs` export requirement
- Clarified `--dry-run` semantics: skips both file creation AND TOML updates
- Specified date format extraction from `<time>` element

### Momus Review (Round 3)
**Critical corrections applied**:
- Cargo commands: Use `--manifest-path scripts/Cargo.toml` (NOT `-p boha-scripts` - repo is not a workspace)
- Playwright session: Use `storage_state` JSON file only; defined `PlaywrightContext` shared struct
- Author extraction: Specified DOM selector `[data-testid="User-Name"] a[role="link"]` + URL fallback + "unknown" fallback
- `--url` mode: Requires `--collection` flag; does not search all TOMLs; error if URL not in TOML
- `PuzzlePath` definition: `{ collection, puzzle_name, array_index }` struct
- TOML path format: Relative to `assets/{collection}/` matching existing convention
- Wayback selectors: Specified fallback chain for archived Twitter DOM

---

## Work Objectives

### Core Objective
Create a Rust script that archives X/Twitter source tweets as markdown + screenshots, with automatic TOML updates.

### Concrete Deliverables
- `scripts/src/bin/archive_tweet.rs` - main archive script
- `.gitattributes` - Git LFS configuration for source_archive.png files only
- `.gitignore` update (root) - exclude Playwright state
- Updated `data/ballet.toml` and `data/bitimage.toml` with `source_archives` field
- Archived tweets in `assets/{collection}/{puzzle_name}/source_archive.md` and `.png`

### Definition of Done
- [x] `cargo run --manifest-path scripts/Cargo.toml --bin archive-tweet -- ballet` archives Bobby Lee tweet
- [x] `cargo run --manifest-path scripts/Cargo.toml --bin archive-tweet -- bitimage` archives aantonop tweet (or gracefully skips if unavailable)
- [x] TOML files updated with `source_archives` references
- [x] Screenshots stored via Git LFS
- [x] Script handles unavailable tweets gracefully (Wayback fallback or skip with warning)

**Note on Cargo commands**: This repo is NOT a Cargo workspace. Scripts are a separate package. Always use `--manifest-path scripts/Cargo.toml` from repo root, or `cd scripts && cargo run --bin ...`

### Must Have
- YAML frontmatter: url, author, date, archived
- Full tweet text content
- Screenshot of tweet
- Automatic TOML update with `source_archives` array
- Wayback Machine fallback
- Retry policy (3 attempts)
- Graceful handling of unavailable tweets

### Must NOT Have (Guardrails)
- NO archiving non-X/Twitter URLs (Medium, GitHub, etc.)
- NO video/GIF support - static PNG only
- NO thread/reply archiving - first tweet only
- NO OCR of screenshots
- NO auto-detection of new URLs - manual trigger only
- NO collection-specific templates - same format for all
- NO API keys in repository - session state gitignored

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: NO (scripts project has no test framework)
- **User wants tests**: Manual verification
- **QA approach**: Manual verification with Playwright browser automation

### Manual QA Procedures

Each TODO includes verification via:
- Playwright browser for screenshot verification
- Shell commands for file/TOML checks
- Git commands for LFS verification

---

## Task Flow

```
Task 0 (Git LFS config)
    ↓
Task 1 (Playwright utils) → Task 2 (Wayback utils) → Task 3 (TOML utils)
                         ↘          ↓           ↙
                           Task 4 (Main script)
                                   ↓
                           Task 5 (Archive tweets)
```

## Parallelization

| Group | Tasks | Reason |
|-------|-------|--------|
| A | 1, 2, 3 | Independent utility modules |

| Task | Depends On | Reason |
|------|------------|--------|
| 4 | 1, 2, 3 | Main script uses all utilities |
| 5 | 0, 4 | Requires LFS config and working script |

---

## TODOs

- [x] 0. Configure Git LFS for source screenshots

  **What to do**:
  - Create `.gitattributes` with LFS tracking for `assets/**/source_archive.png` ONLY
  - Verify LFS is tracking the specific pattern correctly
  - NOTE: Narrow pattern to avoid affecting existing PNGs in `assets/zden/` etc.

  **Must NOT do**:
  - Don't track ALL PNGs (`assets/**/*.png`) - this would affect existing non-LFS images
  - Don't modify existing tracked files

  **Parallelizable**: NO (must be first - other tasks depend on this)

  **References**:
  
  **Pattern References**:
  - `assets/zden/` - existing PNGs that must NOT be moved to LFS
  
  **Documentation References**:
  - Git LFS docs: https://git-lfs.com/

  **Acceptance Criteria**:
  
  **Manual Execution Verification:**
  - [ ] File created: `.gitattributes`
  - [ ] Content includes: `assets/**/source_archive.png filter=lfs diff=lfs merge=lfs -text`
  - [ ] Command: `git lfs track` → shows `assets/**/source_archive.png` (NOT `assets/**/*.png`)
  - [ ] Command: `git add .gitattributes && git status` → `.gitattributes` staged

  **Commit**: YES
  - Message: `chore: configure Git LFS for source archive screenshots`
  - Files: `.gitattributes`
  - Pre-commit: `git lfs track`

---

- [x] 1. Create Playwright utilities for X/Twitter screenshots

  **What to do**:
  - Create `scripts/src/utils/playwright.rs` module
  - **Session persistence strategy**: Use `storage_state` JSON file ONLY (single file, easy to gitignore)
    - Location: `scripts/.playwright-state.json`
    - Add to ROOT `.gitignore` (not scripts/.gitignore which doesn't exist)
  - **Shared context model**: Create `PlaywrightContext` struct that:
    - Holds browser instance and authenticated context
    - Is passed to both `screenshot_tweet()` and `screenshot_wayback()` functions
    - Is initialized once at script start, reused for all URLs
  - Implement `PlaywrightContext::new(state_path) -> Result<Self>` - load existing session or prompt for manual login
  - Implement `PlaywrightContext::screenshot_tweet(url) -> Result<TweetArchive>` where `TweetArchive { text: String, author: String, date: String, png: Vec<u8> }`
  - **Author extraction**: Extract from `[data-testid="User-Name"] a[role="link"]` → text contains `@handle`
    - Fallback: parse from URL path segment (e.g., `/bobbyclee/status/...` → `@bobbyclee`)
    - If both fail: set `author: "unknown"` and log warning (don't fail)
  - **Date extraction**: Extract from `time[datetime]` attribute, format as `YYYY-MM-DD`
  - Fixed viewport: 1280px width
  - Retry policy: 3 attempts with exponential backoff

  **Must NOT do**:
  - Don't attempt to automate X login
  - Don't capture threads/replies - only the main tweet
  - Don't store session state in git

  **Parallelizable**: YES (with 2, 3)

  **References**:
  
  **Pattern References**:
  - `scripts/src/utils/mempool.rs` - HTTP client pattern, retry logic, error handling
  - `scripts/src/bin/generate_transactions.rs:14-39` - async fetch with caching pattern
  - `.gitignore` (root) - existing ignore patterns for scripts
  
  **External References**:
  - Playwright Rust: Use `playwright` crate from crates.io (v0.0.20+)
    - Cargo.toml: `playwright = "0.0.20"`
    - Alternative if crates.io version insufficient: `playwright = { git = "https://github.com/pdonorio/playwright-rs" }`
    - Bootstrap: Run `npx playwright install chromium` once to install browser
  - X tweet DOM structure: tweet text in `[data-testid="tweetText"]`, main tweet container `article[data-testid="tweet"]` for screenshot

  **Acceptance Criteria**:
  
  **Manual Execution Verification:**
  - [ ] File created: `scripts/src/utils/playwright.rs`
  - [ ] Root `.gitignore` updated with `scripts/.playwright-state.json`
  - [ ] Using Playwright browser automation:
    - Navigate to: https://x.com/bobbyclee/status/1289004702122643456
    - Verify: Tweet is visible and text is extractable
    - Screenshot: Save to test output, verify it captures full tweet
  - [ ] Verify module compiles: `cargo check --manifest-path scripts/Cargo.toml`

  **Commit**: NO (groups with 4)

---

- [x] 2. Create Wayback Machine fallback utilities

  **What to do**:
  - Create `scripts/src/utils/wayback.rs` module
  - Implement `check_availability(url) -> Result<Option<String>>` - returns archived Wayback URL if exists
  - Implement `screenshot_wayback(ctx: &PlaywrightContext, wayback_url: &str) -> Result<TweetArchive>`:
    - Reuse the `PlaywrightContext` from Task 1 (same browser/session)
    - Render the Wayback-archived page
    - **Selectors for Wayback** (Twitter DOM may be wrapped/modified):
      - Try live X selectors first: `[data-testid="tweetText"]`, `article[data-testid="tweet"]`
      - Fallback: `.tweet-text`, `.TweetTextSize`, or any `<p>` inside tweet container
      - Ultimate fallback: full page screenshot + extract any visible text
    - **Author extraction on Wayback**: Same strategy as live (DOM selector → URL fallback → "unknown")
    - **Date extraction on Wayback**: Same `time[datetime]` strategy
    - Gracefully skip if archived page is too broken to parse (return error, don't panic)
  - Use Wayback Machine API: `https://archive.org/wayback/available?url={url}`
  - Handle API response quirks:
    - `archived_snapshots: {}` (empty object) = no snapshot available
    - Root-level `"url"` may be missing - handle gracefully
    - `closest.status` is STRING (e.g. `"200"`), not integer - parse accordingly
    - Filter: require `available == true` AND HTTP 2xx status
  - Rate limiting (CRITICAL - archive.org bans aggressive scrapers):
    - Max 10-15 requests per minute
    - Exponential backoff with jitter on errors
    - Respect `Retry-After` header on 429 responses
    - Set descriptive User-Agent: `boha-scripts/0.1 (+https://github.com/oritwoen/boha)`
    - Do NOT spoof browser User-Agent

  **Must NOT do**:
  - Don't call Wayback Save API (`/save/`) - read-only endpoints only
  - Don't submit new archives to Wayback Machine
  - Don't parse complex/broken archived pages - gracefully skip
  - Don't assume `status` is integer - it's a string
  - Don't "power through" 429 errors - respect rate limits

  **Parallelizable**: YES (with 1, 3)

  **References**:
  
  **Pattern References**:
  - `scripts/src/utils/mempool.rs` - HTTP client pattern with reqwest
  - `scripts/src/bin/generate_transactions.rs:29-37` - async API fetch with error handling
  
  **External References**:
  - Wayback Machine API: https://archive.org/help/wayback_api.php
  - API response example: `{"archived_snapshots":{"closest":{"available":true,"url":"...","timestamp":"...","status":"200"}}}`

  **Acceptance Criteria**:
  
  **Manual Execution Verification:**
  - [ ] File created: `scripts/src/utils/wayback.rs`
  - [ ] API test: `curl "https://archive.org/wayback/available?url=https://twitter.com/aantonop/status/603701870482300928"` → check if snapshot exists
  - [ ] Handles empty `archived_snapshots: {}` gracefully (returns None, not error)
  - [ ] Verify module compiles: `cargo check --manifest-path scripts/Cargo.toml`

  **Commit**: NO (groups with 4)

---

- [x] 3. Create TOML update utilities for source_archives

  **What to do**:
  - Create `scripts/src/utils/source_archives.rs` module
  - **`PuzzlePath` definition**: A struct holding `{ collection: String, puzzle_name: String, array_index: usize }`
    - `collection`: e.g., "ballet", "bitimage"
    - `puzzle_name`: from `puzzles[i].name` field (e.g., "AA007448", "kitten")
    - `array_index`: position in `[[puzzles]]` array (0-indexed)
  - Implement `extract_twitter_urls(doc, collection) -> Vec<(PuzzlePath, String)>` - find all X/Twitter URLs in TOML
    - Check: `[metadata] source_url`
    - Check: `[puzzles.assets] source_url`
    - Check: `[puzzles.key.seed.entropy.source] url`
    - ONLY extract status URLs (contain `/status/`) - ignore profile URLs
    - Iterate over `doc["puzzles"]` array using `toml_edit::ArrayOfTables`, track index
  - Implement URL canonicalization:
    - Normalize `twitter.com` ↔ `x.com` (treat as same)
    - Strip tracking params (`?s=20`, `?ref_src=...`, etc.)
  - Implement `update_source_archives(doc, puzzle_path, archive_path)` - add to `source_archives` array
    - **Path format**: Relative to `assets/{collection}/`, matching existing `puzzles.assets.puzzle` convention
    - Example: `source_archives = ["AA007448/source_archive.md"]` (NOT full path)
  - Handle shared archives: multiple puzzles can reference same archive file (dedupe by canonical URL)
  - **Shared archive storage rule**: Store archive under the FIRST puzzle (lowest `array_index`) that references the URL
    - Determined during iteration: first puzzle encountered while iterating `[[puzzles]]` array
    - Example: ballet `[[puzzles]]` order is AA007448, AA009926, AA012381 → store in AA007448
    - All other puzzles referencing same URL get `source_archives` pointing to the shared location
  - Use `toml_edit` to preserve formatting (like existing scripts)

  **Must NOT do**:
  - Don't extract non-Twitter URLs (Medium, GitHub, etc.)
  - Don't extract author profile URLs like `https://x.com/bobbyclee` (no `/status/`)
  - Don't corrupt TOML formatting
  - Don't overwrite existing `source_archives` entries

  **Parallelizable**: YES (with 1, 2)

  **References**:
  
  **Pattern References**:
  - `scripts/src/bin/generate_transactions.rs:69-94` - TOML table manipulation with `toml_edit`
  - `scripts/src/bin/generate_transactions.rs:362-425` - iterating puzzles array, extracting fields
  - `data/ballet.toml:8` - author profile URL to IGNORE: `profiles = [{ name = "twitter", url = "https://x.com/bobbyclee" }]`
  - `data/ballet.toml:33-36` - `[puzzles.assets]` structure with `source_url` (status URL to EXTRACT)
  - `data/bitimage.toml:26-32` - `[puzzles.key.seed.entropy.source]` structure
  - `data/zden.toml:8` - author profile URL to IGNORE: `profiles = [{ name = "twitter", url = "https://twitter.com/zd3n" }]`
  
  **API/Type References**:
  - `toml_edit::DocumentMut` - preserves formatting
  - `toml_edit::Item`, `toml_edit::Value`, `toml_edit::Array`

  **Acceptance Criteria**:
  
  **Manual Execution Verification:**
  - [ ] File created: `scripts/src/utils/source_archives.rs`
  - [ ] Verify URL extraction logic finds STATUS URLs only:
    - ballet.toml: `https://x.com/bobbyclee/status/1289004702122643456` (4 locations)
    - bitimage.toml: `https://twitter.com/aantonop/status/603701870482300928` (4 locations)
  - [ ] Verify URL extraction IGNORES profile URLs:
    - ballet.toml: `https://x.com/bobbyclee` (profile) - NOT extracted
    - zden.toml: `https://twitter.com/zd3n` (profile) - NOT extracted
  - [ ] Verify URL canonicalization: `twitter.com` and `x.com` treated as equivalent
  - [ ] Verify module compiles: `cargo check --manifest-path scripts/Cargo.toml`

  **Commit**: NO (groups with 4)

---

- [x] 4. Create main archive-tweet script

  **What to do**:
  - Create `scripts/src/bin/archive_tweet.rs`
  - Add `[[bin]]` entry in `scripts/Cargo.toml`
  - Add dependencies to `scripts/Cargo.toml`:
    ```toml
    playwright = "0.0.20"
    ```
    - If crates.io version insufficient, use git: `playwright = { git = "https://github.com/pdonorio/playwright-rs" }`
  - Add `scripts/src/utils/mod.rs` export for new modules: `pub mod playwright; pub mod wayback; pub mod source_archives;`
  - CLI interface (from repo root):
    - `cargo run --manifest-path scripts/Cargo.toml --bin archive-tweet -- <collection>` - archive all tweets for collection
    - `cargo run --manifest-path scripts/Cargo.toml --bin archive-tweet -- --url <url> --collection <collection>` - archive single tweet
    - `--dry-run` flag - preview without writing (skips screenshot creation AND TOML updates, only prints what would be done)
    - `--force` flag - re-archive even if already exists
  - **`--url` mode behavior**:
    - REQUIRES `--collection` flag (e.g., `--url https://x.com/... --collection ballet`)
    - Does NOT search across all TOMLs - user must specify target collection
    - If URL not found in specified collection's TOML: warn and skip (no auto-detection per guardrails)
    - Output path: determined by finding puzzles in that collection that reference the URL (same "first puzzle" rule)
    - If URL exists in TOML but wasn't linked to any puzzle: create archive in collection root with URL-derived filename
  - Workflow:
    1. Parse args, load TOML
    2. Extract Twitter URLs from TOML
    3. For each unique URL:
       a. Check if archive already exists → skip (unless --force)
       b. Try Playwright screenshot
       c. If fails → try Wayback Machine
       d. If both fail → log warning, continue
    4. Save markdown + PNG to `assets/{collection}/{puzzle_name}/source_archive.md/.png`
    5. Update TOML with `source_archives` reference
  - Markdown format:
    ```markdown
    ---
    url: https://x.com/...
    author: "@handle"
    date: "2020-07-31"
    archived: "2026-01-09"
    ---

    Tweet text here...

    ![screenshot](source_archive.png)
    ```
  - Date extraction: Extract from `<time datetime="...">` element in tweet DOM, format as `YYYY-MM-DD` (UTC date only, no time)

  **Must NOT do**:
  - Don't archive non-Twitter URLs
  - Don't capture video/GIF - PNG only
  - Don't fail entire run if one tweet unavailable

  **Parallelizable**: NO (depends on 1, 2, 3)

  **References**:
  
  **Pattern References**:
  - `scripts/src/bin/generate_transactions.rs:433-553` - main function structure, arg parsing, collection iteration
  - `scripts/src/bin/generate_transactions.rs:427-431` - Mode enum pattern for --fetch/--process flags
  - `scripts/src/bin/generate_transactions.rs:486-495` - path handling for data directory
  - `scripts/Cargo.toml:22-53` - `[[bin]]` entry format
  
  **Documentation References**:
  - Issue #77: https://github.com/oritwoen/boha/issues/77

  **Acceptance Criteria**:
  
  **Manual Execution Verification:**
  - [ ] File created: `scripts/src/bin/archive_tweet.rs`
  - [ ] File updated: `scripts/src/utils/mod.rs` exports new modules
  - [ ] `scripts/Cargo.toml` updated with `[[bin]]` entry and `playwright` dependency
  - [ ] Command: `cargo build --manifest-path scripts/Cargo.toml --bin archive-tweet` → compiles successfully
  - [ ] Command: `cargo run --manifest-path scripts/Cargo.toml --bin archive-tweet -- --help` → shows usage
  - [ ] Command: `cargo run --manifest-path scripts/Cargo.toml --bin archive-tweet -- --dry-run ballet` → prints preview without creating files

  **Commit**: YES
  - Message: `feat(scripts): add archive-tweet script for preserving source tweets`
  - Files: `scripts/src/bin/archive_tweet.rs`, `scripts/src/utils/playwright.rs`, `scripts/src/utils/wayback.rs`, `scripts/src/utils/source_archives.rs`, `scripts/src/utils/mod.rs`, `scripts/Cargo.toml`, `.gitignore` (root - add playwright state)
  - Pre-commit: `cargo check --manifest-path scripts/Cargo.toml`

---

- [x] 5. Archive existing source tweets and update TOMLs

  **What to do**:
  - Manually log in to X in Playwright browser (one-time setup)
  - Run archive script for ballet collection
  - Run archive script for bitimage collection
  - Verify archives created correctly
  - Verify TOML files updated with `source_archives`

  **Must NOT do**:
  - Don't commit Playwright session state
  - Don't force archive if tweet unavailable - use Wayback or skip

  **Parallelizable**: NO (depends on 0, 4)

  **References**:
  
  **Pattern References**:
  - `data/ballet.toml` - target TOML for update
  - `data/bitimage.toml` - target TOML for update
  - `assets/ballet/` - target directory for archives
  - `assets/bitimage/` - target directory for archives (may need creation)

  **Acceptance Criteria**:
  
  **Manual Execution Verification:**
  
  **For ballet:**
  - [ ] Command: `cargo run --manifest-path scripts/Cargo.toml --bin archive-tweet -- ballet`
  - [ ] File created: `assets/ballet/AA007448/source_archive.md` (shared by all 3 puzzles)
  - [ ] File created: `assets/ballet/AA007448/source_archive.png`
  - [ ] Markdown contains:
    - YAML frontmatter with url, author (@bobbyclee), date, archived
    - Tweet text content
    - `![screenshot](source_archive.png)` reference
  - [ ] `data/ballet.toml` updated:
    - Each puzzle's `[puzzles.assets]` has `source_archives = ["AA007448/source_archive.md"]`
  - [ ] Command: `git lfs ls-files` → shows `assets/ballet/AA007448/source_archive.png`
  
  **For bitimage:**
  - [ ] Command: `cargo run --manifest-path scripts/Cargo.toml --bin archive-tweet -- bitimage`
  - [ ] If aantonop tweet available:
    - File created: `assets/bitimage/kitten/source_archive.md`
    - File created: `assets/bitimage/kitten/source_archive.png`
    - Both puzzles reference same archive
  - [ ] If unavailable:
    - Warning logged
    - No archive created (graceful skip)
    - OR Wayback Machine archive used (if available)
  - [ ] `data/bitimage.toml` updated with `source_archives` (if archived)

  **Commit**: YES
  - Message: `feat(data): archive source tweets for ballet and bitimage collections`
  - Files: `assets/ballet/*/source_archive.*`, `assets/bitimage/*/source_archive.*`, `data/ballet.toml`, `data/bitimage.toml`
  - Pre-commit: `git lfs ls-files | grep source_archive`

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 0 | `chore: configure Git LFS for source archive screenshots` | `.gitattributes` | `git lfs track` |
| 4 | `feat(scripts): add archive-tweet script for preserving source tweets` | `scripts/src/bin/archive_tweet.rs`, utils, `Cargo.toml` | `cargo check --manifest-path scripts/Cargo.toml` |
| 5 | `feat(data): archive source tweets for ballet and bitimage collections` | `assets/*/source_archive.*`, `data/*.toml` | `git lfs ls-files` |

---

## Success Criteria

### Verification Commands
```bash
# LFS configured
git lfs track                    # Expected: assets/**/source_archive.png

# Script works (from repo root)
cargo run --manifest-path scripts/Cargo.toml --bin archive-tweet -- --help  # Expected: usage info
cargo run --manifest-path scripts/Cargo.toml --bin archive-tweet -- --dry-run ballet  # Expected: preview (no files created)

# Archives created
ls assets/ballet/AA007448/source_archive.*  # Expected: .md and .png files
cat assets/ballet/AA007448/source_archive.md  # Expected: YAML frontmatter + content

# TOML updated
grep -A2 "source_archives" data/ballet.toml  # Expected: array with path

# LFS tracking
git lfs ls-files | grep source_archive  # Expected: PNG files listed
```

### Final Checklist
- [x] All "Must Have" present:
  - [x] YAML frontmatter with url, author, date, archived
  - [x] Full tweet text content
  - [x] Screenshot of tweet
  - [x] Automatic TOML update
  - [x] Wayback Machine fallback
  - [x] Retry policy
  - [x] Graceful unavailable tweet handling
- [x] All "Must NOT Have" absent:
  - [x] No non-Twitter URL archiving
  - [x] No video/GIF support
  - [x] No thread/reply archiving
  - [x] No OCR
  - [x] No auto-detection
  - [x] No collection-specific templates
  - [x] No API keys in repo
- [x] All screenshots tracked by Git LFS
- [x] All TOMLs preserve formatting
