# Issues / Gotchas

- `scripts/src/utils/mod.rs` is included via `include!("../utils/mod.rs")` inside a module block in each bin; file-level inner attributes like `#![allow(dead_code)]` are **not permitted** there (they don't apply to the enclosing module and break compilation).
- `cargo check` emits many `dead_code` warnings because each bin includes a shared utils module but only uses a subset of it. (Not fatal; consider addressing globally later if desired.)
- Running binaries from the repo root (common with `--manifest-path scripts/Cargo.toml`) means `Path::new("../data")` points outside the repo; either run from `scripts/` or compute paths relative to `env!("CARGO_MANIFEST_DIR")`.
- In `playwright` v0.0.20, `Page::screenshot_builder()` is not async, but `ElementHandle::screenshot_builder()` is async. Mixing them up causes confusing compile errors (`ScreenshotBuilder is not a future`).
