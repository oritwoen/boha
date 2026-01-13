# Draft: Archive Source Tweets (Issue #77)

## Requirements (confirmed)
- Archive key tweets as markdown with YAML frontmatter + screenshots
- Directory structure: per collection (`assets/sources/ballet/...`, `assets/sources/bitimage/...`)
- Git LFS enabled for images
- Snake_case naming convention for files
- Full tweet content in markdown (not just metadata)
- `source_archives` field as array in TOML (multiple sources possible)
- Automatic TOML update when archiving

## Technical Decisions
- Playwright with manual session login for X authentication
- Script location: `scripts/src/bin/archive_tweet.rs`
- URL sources: `[metadata] source_url`, `[puzzles.assets] source_url`, `[puzzles.key.seed.entropy.source] url`

## Research Findings
- ballet: 1 unique tweet (Bobby Lee) - `https://x.com/bobbyclee/status/1289004702122643456`
- bitimage: 1 unique tweet (aantonop) - `https://twitter.com/aantonop/status/603701870482300928`
- zden: Author profile only, no source tweets
- Total: 2 unique tweets to archive initially

## Scope Boundaries
- INCLUDE: Tweet announcements, key reveals, hints from authors
- EXCLUDE: All mentions/comments, non-X URLs (Medium articles etc.)

## Open Questions
- None remaining
