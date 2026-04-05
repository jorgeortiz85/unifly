# Installation

## Quick Install (Linux / macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/hyperb1iss/unifly/main/install.sh | sh
```

Detects your platform, downloads the latest release binary, and installs to `/usr/local/bin` (or `~/.local/bin`).

## Quick Install (Windows PowerShell)

```powershell
irm https://raw.githubusercontent.com/hyperb1iss/unifly/main/install.ps1 | iex
```

Installs `unifly.exe` into `%LOCALAPPDATA%\unifly\bin` and adds that directory to your user `PATH` if needed.

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

Requires Rust 1.94+ (edition 2024):

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

```powershell
# PowerShell
unifly completions powershell | Out-String | Invoke-Expression
```

## Verify Installation

```bash
unifly --version
```

## Next Steps

You're installed! Now configure your first controller profile:

- [Quick Start](/guide/quick-start): run `unifly config init` and explore your network
- [Authentication](/guide/authentication): choose the right auth mode
- [Troubleshooting](/troubleshooting): if something goes wrong
