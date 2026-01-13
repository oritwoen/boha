# Decisions

- Implemented Playwright utilities as async APIs (`PlaywrightContext::new` / `screenshot_tweet`) because the `playwright` crate is async-first and wrapping it into sync APIs would require runtime juggling / blocking.
- Session persistence is implemented strictly via `storage_state` JSON (no user-data-dir), stored at `scripts/.playwright-state.json` and gitignored.
- Wayback rate limiting is implemented as a process-wide limiter (`OnceLock<Mutex<Instant>>`) with a conservative 5s minimum interval (~12 req/min) plus exponential backoff and small jitter on retry.
- Added a minimal `PlaywrightContext::new_page()` helper (pub(crate)) so other utils (Wayback) can open pages without exposing internal fields.
