# Contributing to unifly

Thanks for your interest in contributing! Whether it's a bug report, feature idea, or code contribution — it all helps make unifly better.

## Reporting Bugs

Open a [GitHub Issue](https://github.com/hyperb1iss/unifly/issues) with:

- unifly version (`unifly --version`)
- Controller model and firmware version
- Steps to reproduce
- Expected vs. actual behavior
- Relevant error output (use `-vvv` for full debug logging)

## Requesting Features

Open a [GitHub Issue](https://github.com/hyperb1iss/unifly/issues) describing:

- The use case — what problem does this solve?
- How you'd expect it to work (CLI syntax, TUI behavior, etc.)
- Whether it relates to the Integration API, Legacy API, or both

Check the [ROADMAP.md](ROADMAP.md) first — it might already be planned.

## Development Setup

### Prerequisites

- **Rust 1.94+** (edition 2024) — install via [rustup](https://rustup.rs/)
- **Nightly rustfmt** — `rustup component add rustfmt --toolchain nightly`
- A UniFi Network controller for integration testing (Cloud Key, Dream Machine, or self-hosted)

### Build & Test

```bash
git clone https://github.com/hyperb1iss/unifly.git
cd unifly
cargo build --workspace
cargo test --workspace
```

### Run from Source

```bash
cargo run -p unifly -- devices list
cargo run -p unifly -- tui
```

### Workspace Structure

```
crates/
  unifly-api/      # Library — HTTP/WS transport, Controller, DataStore, domain models
  unifly/          # Single binary: CLI commands + tui subcommand, config, profiles
```

Dependency chain: `unifly` depends on `unifly-api`.

See the [README](README.md#-architecture) for the full architecture overview.

## Code Style

### Formatting

```bash
cargo +nightly fmt --all
```

The project uses nightly rustfmt with a custom `rustfmt.toml` (100-char max width, field init shorthand, try shorthand).

### Linting

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

The workspace has opinionated clippy configuration — pedantic lints enabled. Key rules:

- `unsafe_code = "forbid"` — no unsafe code, period
- `unwrap_used = "deny"` — use `?`, `.ok()`, `.unwrap_or()`, or proper error handling
- `pedantic = "deny"` — clippy pedantic lints are enforced

### Conventions

- Error types use `thiserror` with `miette` for rich diagnostics
- Async runtime is `tokio`; all public async APIs are `Send + Sync`
- Entity IDs use the `EntityId` enum (`Uuid` | `Legacy`) for dual-API compatibility
- Secrets are wrapped in `secrecy::SecretString` — never log or display credentials

## Pull Request Workflow

1. **Fork** the repository
2. **Branch** from `main` (`feature/my-feature` or `fix/my-fix`)
3. **Implement** your changes with tests where applicable
4. **Ensure CI passes** locally:
   ```bash
   cargo +nightly fmt --all --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```
5. **Open a PR** targeting `main`
6. Describe what changed and why — link the relevant issue if one exists

PRs are reviewed for correctness, style consistency, and architectural fit. Don't worry about perfection on the first pass — feedback is collaborative, not adversarial.

## License

By contributing, you agree that your contributions will be licensed under the [Apache License 2.0](LICENSE), the same license covering the project.
