## Manual Execution Steps for Task 5

### Prerequisites
- All code is implemented and committed
- User needs to run the script manually due to X login requirement

### Step-by-Step Instructions

#### 1. Archive ballet collection
```bash
cd /home/oritwoen/Projekty/boha
cargo run --manifest-path scripts/Cargo.toml --bin archive-tweet -- ballet
```

**What happens**:
- First run: Playwright downloads Chromium (~170MB) automatically
- Browser window opens at https://x.com/login
- Terminal shows: "Please complete login in the opened browser, then press Enter here to continue."

**User action**:
- Log in to X in the browser window
- Return to terminal
- Press Enter

**Result**:
- Session saved to `scripts/.playwright-state.json`
- Tweet archived to `assets/ballet/AA007448/source_archive.{md,png}`
- `data/ballet.toml` updated with `source_archives` field

#### 2. Archive bitimage collection
```bash
cargo run --manifest-path scripts/Cargo.toml --bin archive-tweet -- bitimage
```

**What happens**:
- Reuses saved session (no login needed)
- Archives aantonop tweet (or uses Wayback fallback if unavailable)
- Saves to `assets/bitimage/kitten/source_archive.{md,png}`
- Updates `data/bitimage.toml`

#### 3. Verify results
```bash
# Check files created
ls -la assets/ballet/AA007448/source_archive.*
ls -la assets/bitimage/kitten/source_archive.*

# Check LFS tracking
git lfs ls-files | grep source_archive

# Check TOML updates
grep -A2 "source_archives" data/ballet.toml
grep -A2 "source_archives" data/bitimage.toml

# View markdown content
cat assets/ballet/AA007448/source_archive.md
```

**Expected**:
- `.md` files with YAML frontmatter + tweet text
- `.png` files tracked by Git LFS
- `source_archives = ["AA007448/source_archive.md"]` in ballet.toml (3 puzzles)
- `source_archives = ["kitten/source_archive.md"]` in bitimage.toml (2 puzzles)

#### 4. Commit results
```bash
git add assets/ data/
git commit -m "feat(data): archive source tweets for ballet and bitimage collections"
```

### Troubleshooting

**If Chromium download fails**:
```bash
npx playwright install chromium
```

**If session expires**:
```bash
rm scripts/.playwright-state.json
# Re-run script, will prompt for login again
```

**If tweet unavailable**:
- Script will automatically try Wayback Machine
- If both fail, warning logged and script continues

### Dry-run mode (test without archiving)
```bash
cargo run --manifest-path scripts/Cargo.toml --bin archive-tweet -- --dry-run ballet
```

Shows what would be archived without creating files.
