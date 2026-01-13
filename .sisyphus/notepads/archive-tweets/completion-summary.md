## Archive Tweets Implementation - COMPLETE ✅

**Date**: 2026-01-09
**Issue**: #77
**Plan**: `.sisyphus/plans/archive-tweets.md`

---

## Summary

Successfully implemented a Rust script to archive X/Twitter source tweets as markdown + screenshots with automatic TOML updates and Git LFS storage.

---

## Deliverables

### Code
- ✅ `scripts/src/bin/archive_tweet.rs` - Main CLI script (369 lines)
- ✅ `scripts/src/utils/playwright.rs` - Browser automation (8.8KB)
- ✅ `scripts/src/utils/wayback.rs` - Wayback Machine fallback (12.4KB)
- ✅ `scripts/src/utils/source_archives.rs` - TOML manipulation (7.9KB)
- ✅ `.gitattributes` - Git LFS configuration for `assets/**/source_archive.png`

### Data
- ✅ `assets/ballet/AA007448/source_archive.{md,png}` - Bobby Lee tweet (49KB PNG)
- ✅ `assets/bitimage/kitten/source_archive.{md,png}` - aantonop tweet (43KB PNG)
- ✅ `data/ballet.toml` - Updated with `source_archives` for 3 puzzles
- ✅ `data/bitimage.toml` - Updated with `source_archives` for 2 puzzles

### Commits
1. `0755212` - "chore: configure Git LFS for source archive screenshots"
2. `5d3e973` - "feat(scripts): add archive-tweet script for preserving source tweets"
3. `4ef9af2` - "feat(data): archive source tweets for ballet and bitimage collections"

---

## Features Implemented

### Core Functionality
- ✅ Playwright-based X/Twitter screenshot capture
- ✅ Manual session login with persistent storage state
- ✅ Wayback Machine fallback for unavailable tweets
- ✅ Automatic TOML update with `source_archives` array
- ✅ Shared archive storage (first puzzle owns file, others reference)
- ✅ Git LFS integration for PNG files

### CLI Interface
```bash
archive-tweet <collection>              # Archive all tweets for collection
archive-tweet --url <url> --collection  # Archive single tweet
archive-tweet --dry-run <collection>    # Preview without changes
archive-tweet --force <collection>      # Re-archive existing
```

### Markdown Format
```markdown
---
url: https://x.com/...
author: "@handle"
date: "YYYY-MM-DD"
archived: "YYYY-MM-DD"
---

Tweet text content...

![screenshot](source_archive.png)
```

---

## Technical Highlights

### Playwright Integration
- Session persistence via `storage_state` JSON file
- Manual login flow (headed browser)
- Retry policy: 3 attempts with exponential backoff
- Fixed viewport: 1280px width
- Author extraction: DOM selector → URL fallback → "unknown"
- Date extraction from `<time datetime>` attribute

### Wayback Machine Fallback
- API: `https://archive.org/wayback/available?url=...`
- Rate limiting: 5s between requests (~12/min)
- Respects `Retry-After` header on 429 responses
- User-Agent: `boha-scripts/0.1 (+https://github.com/oritwoen/boha)`
- Handles empty `archived_snapshots: {}` gracefully

### TOML Manipulation
- Uses `toml_edit` to preserve formatting
- Extracts URLs from 3 locations:
  - `[metadata] source_url`
  - `[puzzles.assets] source_url`
  - `[puzzles.key.seed.entropy.source] url`
- URL canonicalization: `twitter.com` ↔ `x.com`
- Strips tracking params (`?s=20`, `?ref_src=...`)
- Ignores profile URLs (only status URLs with `/status/`)

### Git LFS
- Pattern: `assets/**/source_archive.png` (narrow, doesn't affect existing PNGs)
- Initialized with `git lfs install --local`
- PNGs stored as LFS pointers (not full binary in git)

---

## Verification Results

### Ballet Collection
- **URL**: https://x.com/bobbyclee/status/1289004702122643456
- **Author**: @bobbyclee
- **Date**: 2020-07-31
- **Archived**: 2026-01-09
- **Storage**: `assets/ballet/AA007448/` (first puzzle)
- **References**: 3 puzzles (AA007448, AA009926, AA012381)
- **PNG Size**: 49KB (LFS tracked)

### Bitimage Collection
- **URL**: https://x.com/aantonop/status/603701870482300928
- **Author**: @aantonop
- **Date**: 2015-05-27
- **Archived**: 2026-01-09
- **Storage**: `assets/bitimage/kitten/` (first puzzle)
- **References**: 2 puzzles (kitten, kitten_passphrase)
- **PNG Size**: 43KB (LFS tracked)

---

## Lessons Learned

### Git LFS Setup
- **Issue**: LFS wasn't initialized for repo, PNGs staged as full binary
- **Solution**: Run `git lfs install --local`, unstage and re-add PNGs
- **Verification**: `git lfs ls-files` shows LFS pointers, not binary data

### X/Twitter Login
- **Challenge**: X detects Playwright automation and blocks login
- **Solution**: Manual login in headed browser, session persisted to JSON file
- **Result**: Session reusable across runs, no repeated logins needed

### Shared Archives
- **Pattern**: Multiple puzzles can reference same tweet URL
- **Implementation**: First puzzle (lowest array_index) owns the file
- **TOML**: All puzzles get `source_archives = ["owner/source_archive.md"]`

---

## Future Enhancements (Not in Scope)

- Video/GIF support (currently PNG only)
- Thread/reply archiving (currently first tweet only)
- OCR of screenshots (currently raw PNG)
- Auto-detection of new URLs (currently manual trigger)
- Collection-specific templates (currently uniform format)

---

## All Tasks Complete ✅

- [x] Task 0: Configure Git LFS
- [x] Task 1: Create Playwright utilities
- [x] Task 2: Create Wayback Machine utilities
- [x] Task 3: Create TOML update utilities
- [x] Task 4: Create main archive-tweet script
- [x] Task 5: Archive existing source tweets

**Total Implementation Time**: ~2 hours (across 2 sessions)
**Lines of Code**: ~1,200 (implementation + utilities)
**Files Changed**: 10 (4 new utilities, 1 CLI, 1 config, 4 data files)
