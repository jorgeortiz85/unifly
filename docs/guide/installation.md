# Installation

## Quick Install (Linux / macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/hyperb1iss/unifly/main/install.sh | sh
```

Detects your platform, downloads the latest release binary, and installs to `/usr/local/bin` (or `~/.local/bin`).

## Homebrew (macOS / Linux)

```bash
brew install hyperb1iss/tap/unifly
```

## AUR (Arch Linux)

```bash
yay -S unifly-bin
```

## GitHub Releases

Download the latest binary for your platform from [GitHub Releases](https://github.com/hyperb1iss/unifly/releases/latest).

## Cargo (from source)

Requires Rust 1.86+ (edition 2024):

```bash
cargo install --git https://github.com/hyperb1iss/unifly.git unifly
```

Or from crates.io:

```bash
cargo install unifly
```

## Build from Source

```bash
git clone https://github.com/hyperb1iss/unifly.git
cd unifly
cargo build --workspace --release
```

The binary is placed in `target/release/unifly`.

## Shell Completions

Generate completions for your shell after installation:

```bash
# Bash
unifly completions bash > ~/.local/share/bash-completion/completions/unifly

# Zsh
unifly completions zsh > ~/.zfunc/_unifly

# Fish
unifly completions fish > ~/.config/fish/completions/unifly.fish
```

## Verify Installation

```bash
unifly --version
```
