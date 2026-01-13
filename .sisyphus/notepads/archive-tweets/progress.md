## Progress Report - Archive Tweets Implementation

### Completed Tasks

#### Task 0: Git LFS Configuration ✅
- Created `.gitattributes` with pattern `assets/**/source_archive.png`
- Verified LFS tracking
- Committed: `0755212` - "chore: configure Git LFS for source archive screenshots"

#### Task 1: Playwright Utilities ✅
- Implemented `scripts/src/utils/playwright.rs`
- `PlaywrightContext` struct with session persistence
- `TweetArchive` struct for return data
- Author extraction with 3-level fallback (DOM → URL → "unknown")
- Date extraction from `time[datetime]` attribute
- Retry policy: 3 attempts with exponential backoff
- Fixed viewport: 1280px width

#### Task 2: Wayback Machine Utilities ✅
- Implemented `scripts/src/utils/wayback.rs`
- `check_availability()` - checks Wayback API for archived snapshots
- `screenshot_wayback()` - renders archived pages with Playwright
- Rate limiting: 5s between requests (~12/min)
- Exponential backoff with jitter
- Respects `Retry-After` header on 429
- User-Agent: `boha-scripts/0.1 (+https://github.com/oritwoen/boha)`
- Verified: aantonop tweet IS archived (2022-05-01)

#### Task 3: TOML Utilities ✅
- Implemented `scripts/src/utils/source_archives.rs`
- `PuzzlePath` struct: `{ collection, puzzle_name, array_index }`
- `extract_twitter_urls()` - finds status URLs in 3 locations
- `canonicalize_url()` - normalizes twitter.com ↔ x.com, strips params
- `update_source_archives()` - adds references to TOML, preserves formatting
- Only extracts URLs with `/status/` (ignores profile URLs)

#### Task 4: Main Script ✅
- Implemented `scripts/src/bin/archive_tweet.rs`
- CLI with clap: `archive-tweet <collection>` or `--url <url> --collection <collection>`
- Flags: `--dry-run`, `--force`
- Workflow: extract URLs → dedupe → screenshot → save markdown+PNG → update TOML
- Shared archive logic: first puzzle stores file, others reference it
- Committed: `5d3e973` - "feat(scripts): add archive-tweet script for preserving source tweets"

### Remaining Task

#### Task 5: Run Script and Archive Tweets ⏸️ BLOCKED
**Status**: Requires manual X login - cannot be automated

**What needs to be done**:
1. Run `cargo run --manifest-path scripts/Cargo.toml --bin archive-tweet -- ballet`
2. Browser opens - user logs in to X manually
3. Press Enter in terminal to continue
4. Script archives Bobby Lee tweet to `assets/ballet/AA007448/source_archive.*`
5. Updates `data/ballet.toml` with `source_archives` references
6. Run `cargo run --manifest-path scripts/Cargo.toml --bin archive-tweet -- bitimage`
7. Script archives aantonop tweet (or uses Wayback fallback)
8. Verify files created
9. Commit results

**Verification commands**:
```bash
# Check files created
ls assets/ballet/AA007448/source_archive.*
ls assets/bitimage/kitten/source_archive.*

# Check LFS tracking
git lfs ls-files | grep source_archive

# Check TOML updates
grep -A2 "source_archives" data/ballet.toml
grep -A2 "source_archives" data/bitimage.toml
```

**Final commit**:
```bash
git add assets/ data/
git commit -m "feat(data): archive source tweets for ballet and bitimage collections"
```

### Technical Learnings

1. **Playwright Session Persistence**:
   - Use `storage_state` JSON file (not user-data-dir)
   - Location: `scripts/.playwright-state.json` (gitignored)
   - Reusable across script runs

2. **Wayback Machine**:
   - API returns `status` as STRING not integer
   - Empty snapshots: `{"archived_snapshots": {}}`
   - Rate limiting critical: 10-15 req/min max
   - aantonop tweet archived: http://web.archive.org/web/20220501070445/https://twitter.com/aantonop/status/603701870482300928

3. **TOML Editing**:
   - `toml_edit` preserves formatting
   - Nested paths require careful unwrapping
   - Relative paths: `"AA007448/source_archive.md"` (not full paths)

4. **Shared Archives**:
   - ballet: 3 puzzles share 1 tweet → store in AA007448 (first puzzle)
   - bitimage: 2 puzzles share 1 tweet → store in kitten (first puzzle)
   - Other puzzles get `source_archives` pointing to shared location

5. **Build System**:
   - Repo is NOT a Cargo workspace
   - Use `--manifest-path scripts/Cargo.toml` (NOT `-p boha-scripts`)
   - LSP can show stale errors - always verify with `cargo build`

### Files Created/Modified

**New files**:
- `.gitattributes` - Git LFS configuration
- `scripts/src/bin/archive_tweet.rs` - Main CLI script
- `scripts/src/utils/playwright.rs` - Browser automation
- `scripts/src/utils/wayback.rs` - Wayback Machine fallback
- `scripts/src/utils/source_archives.rs` - TOML manipulation

**Modified files**:
- `.gitignore` - Added `scripts/.playwright-state.json`
- `scripts/Cargo.toml` - Added playwright, clap dependencies + bin entry
- `scripts/src/utils/mod.rs` - Exported new modules
- `scripts/src/utils/dcrdata.rs` - Formatting changes (rustfmt)

### Commits

1. `0755212` - chore: configure Git LFS for source archive screenshots
2. `5d3e973` - feat(scripts): add archive-tweet script for preserving source tweets

**Pending commit** (after Task 5 manual execution):
3. feat(data): archive source tweets for ballet and bitimage collections

## [2026-01-09T22:10:00Z] Task 5: COMPLETED

**Status**: All tasks complete! ✅

**What was done**:
1. Ran `archive-tweet bitimage` - successfully archived aantonop tweet
   - User manually logged in to X when browser opened
   - Session saved to `scripts/.playwright-state.json`
   - Files created: `assets/bitimage/kitten/source_archive.{md,png}`
   - TOML updated: `data/bitimage.toml` with `source_archives` references

2. Ran `archive-tweet ballet` - successfully archived Bobby Lee tweet
   - Reused existing X session (no login needed)
   - Files created: `assets/ballet/AA007448/source_archive.{md,png}`
   - TOML updated: `data/ballet.toml` with `source_archives` references

3. Fixed Git LFS tracking:
   - Discovered LFS wasn't initialized for repo
   - Ran `git lfs install --local`
   - Re-added PNG files to convert to LFS pointers
   - Verified: `git lfs ls-files` shows both PNGs tracked

4. Committed changes:
   - Commit: `4ef9af2` - "feat(data): archive source tweets for ballet and bitimage collections"
   - Files: 6 changed (2 markdown, 2 PNG via LFS, 2 TOML updates)

**Final verification**:
- ✅ Ballet: 1 tweet archived, shared by 3 puzzles (AA007448, AA009926, AA012381)
- ✅ Bitimage: 1 tweet archived, shared by 2 puzzles (kitten, kitten_passphrase)
- ✅ PNGs tracked by Git LFS (49KB and 43KB)
- ✅ TOMLs updated with `source_archives` arrays
- ✅ All acceptance criteria met

**Total implementation time**: ~2 hours (across 2 sessions)
