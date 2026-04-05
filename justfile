# ── unifly justfile ─────────────────────────────────────────────
# https://github.com/hyperb1iss/unifly

set dotenv-load

# List available recipes
default:
    @just --list --unsorted

# ── Build ───────────────────────────────────────────────────────

# Build all crates (debug)
build:
    cargo build --workspace

# Build all crates (release)
build-release:
    cargo build --workspace --release

# ── Install ─────────────────────────────────────────────────────

# Install unifly (CLI + TUI)
install:
    cargo install --path crates/unifly

# Install CLI only (no TUI dependencies)
install-cli:
    cargo install --path crates/unifly --no-default-features --features cli

# ── Quality ─────────────────────────────────────────────────────

# Run all checks (lint + test + clippy)
check: lint clippy test

# Run clippy with workspace lints
clippy:
    cargo clippy --workspace --all-targets

# Auto-fix clippy + formatting
fix:
    cargo clippy --workspace --all-targets --fix --allow-dirty
    cargo fmt --all

# Format all code
fmt:
    cargo fmt --all

# Check formatting without modifying
fmt-check:
    cargo fmt --all -- --check

# Lint = format check + clippy
lint: fmt-check clippy

# ── Test ────────────────────────────────────────────────────────

# Run all tests
test:
    cargo test --workspace

# Run tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Run tests for a specific crate
test-crate crate:
    cargo test -p {{crate}}

# Update insta snapshots
snap-review:
    cargo insta review

# ── Run ─────────────────────────────────────────────────────────

# Run the CLI with args
cli *args:
    cargo run -p unifly -- {{args}}

# Run the TUI dashboard
tui *args:
    cargo run -p unifly -- tui {{args}}

# ── Docs ────────────────────────────────────────────────────────

# Generate rustdoc for the workspace
doc:
    cargo doc --workspace --no-deps --open

# Format markdown and JSON with prettier
prettier:
    cd docs && npx prettier --write "**/*.md" "**/*.json"

# Check markdown and JSON formatting
prettier-check:
    cd docs && npx prettier --check "**/*.md" "**/*.json"

# ── Clean ───────────────────────────────────────────────────────

# Remove build artifacts
clean:
    cargo clean

# ── Release ─────────────────────────────────────────────────────

# Dry-run cargo-dist
dist-plan:
    cargo dist plan

# Build distributable artifacts
dist-build:
    cargo dist build

# Update AUR package for a new release
aur-update version:
    cd aur && ./update-aur.sh {{version}}
