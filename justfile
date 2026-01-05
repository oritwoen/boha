default:
    @just --list

test:
    cargo test --all-features

build:
    cargo build --release --features cli,balance

clippy:
    cargo clippy --all-features -- -D warnings

check:
    cargo check --all-features

changelog:
    git cliff -o CHANGELOG.md

release version:
    #!/usr/bin/env bash
    set -euo pipefail
    
    VERSION="{{version}}"
    
    if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "Error: Version must be in format X.Y.Z (e.g., 0.1.0)"
        exit 1
    fi
    
    echo "Releasing v$VERSION..."
    
    sed -i '0,/^version = /s/^version = \".*\"/version = \"'"$VERSION"'\"/' Cargo.toml
    sed -i "s/^pkgver=.*/pkgver=$VERSION/" PKGBUILD
    
    MAJOR_MINOR=$(echo "$VERSION" | sed 's/\.[0-9]*$//')
    sed -i 's/boha = "'"[0-9]*\.[0-9]*"'"/boha = "'"$MAJOR_MINOR"'"/' README.md
    sed -i 's/boha = { version = "'"[0-9]*\.[0-9]*"'"/boha = { version = "'"$MAJOR_MINOR"'"/' README.md
    
    cargo update -p boha
    
    git cliff --tag "v$VERSION" -o CHANGELOG.md
    
    git add Cargo.toml Cargo.lock PKGBUILD CHANGELOG.md README.md
    git commit -m "chore(release): v$VERSION"
    
    git tag -a "v$VERSION" -m "Release v$VERSION"
    
    echo ""
    echo "Release v$VERSION ready!"
    echo "Run 'git push && git push --tags' to publish"
