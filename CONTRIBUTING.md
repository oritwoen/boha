# Contributing

Contributions welcome! This includes code, bug fixes, new puzzle collections, and data updates.

## Getting Started

1. Fork the repository
2. Clone your fork
3. Create a feature branch (`git checkout -b feature/my-change`)

## Development

```bash
cargo build
cargo test
cargo fmt
cargo clippy
```

## Pull Requests

1. Push to your fork
2. Open a PR against `main`
3. CI will run tests, formatting, and linting checks

## Adding Puzzle Data

Puzzle data lives in `data/*.toml` files. The build script generates Rust code from these at compile time.

When adding or updating puzzles:
- Follow the existing TOML structure
- Verify addresses are valid
- Include source references where possible

## Questions?

Open an issue if something is unclear.
