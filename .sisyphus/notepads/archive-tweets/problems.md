# Unresolved / Follow-ups

- If we want to eliminate the pervasive `dead_code` warnings in `scripts` binaries, we likely need to change each bin's `mod utils { ... }` wrapper to include `#[allow(dead_code)]` (done in `archive-tweet`), or split utils into a proper library crate/module graph instead of `include!`.
