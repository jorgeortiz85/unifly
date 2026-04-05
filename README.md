<h1 align="center">
  <br>
  🌐 unifly
  <br>
</h1>

<p align="center">
  <strong>Your UniFi Network, at Your Fingertips</strong><br>
  <sub>✦ CLI + TUI for UniFi Network Controllers ✦</sub>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.94+-e135ff?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/Edition-2024-80ffea?style=for-the-badge&logo=rust&logoColor=0a0a0f" alt="Edition 2024">
  <img src="https://img.shields.io/badge/ratatui-TUI-ff6ac1?style=for-the-badge&logo=gnometerminal&logoColor=white" alt="ratatui">
  <img src="https://img.shields.io/badge/opaline-Theme-e135ff?style=for-the-badge&logo=rust&logoColor=white" alt="opaline">
  <img src="https://img.shields.io/badge/tokio-Async-f1fa8c?style=for-the-badge&logo=rust&logoColor=0a0a0f" alt="tokio">
  <img src="https://img.shields.io/badge/License-Apache--2.0-50fa7b?style=for-the-badge&logo=apache&logoColor=0a0a0f" alt="License">
  <a href="https://github.com/sponsors/hyperb1iss">
    <img src="https://img.shields.io/badge/Sponsor-ff6ac1?style=for-the-badge&logo=githubsponsors&logoColor=white" alt="Sponsor">
  </a>
</p>

<p align="center">
  <a href="#-features">Features</a> •
  <a href="#-install">Install</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-cli">CLI</a> •
  <a href="#-tui">TUI</a> •
  <a href="#-architecture">Architecture</a> •
  <a href="#-library">Library</a> •
  <a href="#-ai-agent-skill">AI Agent Skill</a> •
  <a href="#-development">Development</a>
</p>

---

## 💜 What is unifly?

A complete command-line toolkit for managing Ubiquiti UniFi network controllers. One binary with 26 top-level commands for scripting and a built-in TUI dashboard for real-time monitoring, powered by a shared async engine that speaks every UniFi API dialect.

> _Manage devices, monitor clients, inspect VLANs, stream events, and watch bandwidth charts, all without leaving your terminal._

UniFi controllers expose multiple APIs with different capabilities. unifly unifies them all into a single, coherent interface so you never have to think about which endpoint to hit.

---

> ### 🤖 AI Agent? 👤 Human? Both Welcome.
>
> unifly speaks fluent silicon *and* carbon.
>
> **Coding agents** get a dedicated [skill bundle](skills/unifly/SKILL.md): full CLI reference, automation workflows, and a ready-made network manager agent that can provision VLANs, audit firewalls, and diagnose connectivity without asking permission for every command. One command to install:
>
> ```bash
> npx skills add hyperb1iss/unifly
> ```
>
> **Humans** get a gorgeous 10-screen TUI, shell completions, pipe-friendly output, and the quiet satisfaction of never opening the UniFi web UI again. Keep scrolling to [Install](#-install).

---

## ✦ Features

| Capability | What You Get |
| --- | --- |
| 🔮 **Dual API Engine** | Integration API (REST, API key) + Legacy API (session, cookie/CSRF) with automatic Hybrid negotiation |
| ⚡ **Real-Time TUI** | 10-screen dashboard with area-fill traffic charts, CPU/MEM gauges, live client counts, zoomable topology |
| 🦋 **26 Top-Level Commands** | Devices, clients, networks, WiFi, firewall policies, zones, ACLs, NAT, DNS, VPN, DPI, RADIUS, topology, raw API passthrough, `tui`... |
| 💎 **Flexible Output** | Table, JSON, compact JSON, YAML, and plain text. Pipe-friendly for scripting |
| 🔒 **Secure Credentials** | OS keyring storage for API keys and passwords, with plaintext config support when you choose it |
| 🌐 **Multi-Profile** | Named profiles for multiple controllers. Switch with a single flag |
| 🧠 **Smart Config** | Interactive wizard, environment variables, TOML config, CLI overrides |
| 📡 **WebSocket Events** | Live event streaming with 10K rolling buffer, severity filtering, pause/scroll-back |
| 📊 **Historical Stats** | WAN bandwidth area fills, client counts, DPI app/category breakdown (1h to 30d) |
| 🎨 **SilkCircuit Theme** | Neon-on-dark color palette powered by [opaline](https://crates.io/crates/opaline). Token-based theming across CLI and TUI with ANSI fallback |

---

## ⚡ Install

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/hyperb1iss/unifly/main/install.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/hyperb1iss/unifly/main/install.ps1 | iex
```

### Other Methods

| Method | Command |
| --- | --- |
| **Homebrew** | `brew install hyperb1iss/tap/unifly` |
| **AUR** | `yay -S unifly-bin` |
| **Cargo** | `cargo install --git https://github.com/hyperb1iss/unifly.git unifly` |
| **Binary** | Download from [GitHub Releases](https://github.com/hyperb1iss/unifly/releases/latest) |

---

## 🔮 Quick Start

Run the interactive setup wizard:

```bash
unifly config init
```

The wizard walks you through controller URL, authentication method, and site selection. Credentials can be stored in your OS keyring or saved in plaintext config, depending on the option you choose.

Once configured:

```bash
unifly devices list          # All adopted devices
unifly clients list          # Connected clients
unifly networks list         # VLANs and subnets
unifly events watch          # Live event feed
```

```
 ID                                   | Name            | Model           | Status
--------------------------------------+-----------------+-----------------+--------
 a1b2c3d4-e5f6-7890-abcd-ef1234567890 | Office Gateway  | UDM-Pro         | ONLINE
 b2c3d4e5-f6a7-8901-bcde-f12345678901 | Living Room AP  | U6-LR           | ONLINE
 c3d4e5f6-a7b8-9012-cdef-123456789012 | Garage Switch   | USW-Lite-8-PoE  | ONLINE
```

---

## 🔐 Authentication

### API Key (recommended)

Generate a key on your controller under **Settings > Integrations**. Full CRUD access via the Integration API.

```bash
unifly config init                     # Select "API Key" during setup
unifly --api-key <KEY> devices list    # Or pass directly
```

### Username / Password

Legacy session-based auth with cookie and CSRF token handling. Required for events, statistics, and device commands not yet in the Integration API.

```bash
unifly config init                     # Select "Username/Password" during setup
```

### Hybrid Mode

Best of both worlds: API key for Integration API CRUD, username/password for Legacy API features. The wizard offers this when both are available.

### Environment Variables

| Variable | Description |
| --- | --- |
| `UNIFI_API_KEY` | Integration API key |
| `UNIFI_URL` | Controller URL |
| `UNIFI_PROFILE` | Profile name |
| `UNIFI_SITE` | Site name or UUID |
| `UNIFI_OUTPUT` | Default output format |
| `UNIFI_INSECURE` | Accept self-signed TLS certs |
| `UNIFI_TIMEOUT` | Request timeout (seconds) |

---

## 💻 CLI

### Commands

| Command | Alias | Description |
| --- | --- | --- |
| `acl` | | Manage ACL rules |
| `admin` | | Administrator management |
| `alarms` | | Manage alarms |
| `clients` | `cl` | Manage clients and DHCP reservations |
| `completions` | | Generate shell completions |
| `config` | | Manage CLI configuration |
| `countries` | | List available country codes |
| `devices` | `dev`, `d` | Manage adopted and pending devices |
| `dns` | | Manage DNS policies (local records) |
| `dpi` | | DPI reference data |
| `events` | | View and stream events |
| `firewall` | `fw` | Manage firewall policies and zones |
| `nat` | | Manage NAT policies (masquerade, SNAT, DNAT) |
| `hotspot` | | Manage hotspot vouchers |
| `networks` | `net`, `n` | Manage networks and VLANs |
| `radius` | | View RADIUS profiles |
| `sites` | | Manage sites |
| `stats` | | Query statistics and reports |
| `system` | `sys` | System operations and info |
| `topology` | `topo` | Show network topology tree |
| `traffic-lists` | | Manage traffic matching lists |
| `vpn` | | View VPN servers and tunnels |
| `wans` | | View WAN interfaces |
| `wifi` | `w` | Manage WiFi broadcasts (SSIDs) |
| `api` | | Raw API passthrough (GET/POST to any endpoint) |
| `tui` | | Launch the real-time terminal dashboard |

Most resource groups support `list` and `get`; some also expose `create`, `update`, `delete`, `patch`, or specialized actions. Run `unifly <command> --help` for details.

### Global Flags

```
-p, --profile <NAME>     Controller profile to use
-c, --controller <URL>   Controller URL (overrides profile)
-s, --site <SITE>        Site name or UUID
-o, --output <FORMAT>    Output: table, json, json-compact, yaml, plain
-k, --insecure           Accept self-signed TLS certificates
-v, --verbose            Increase verbosity (-v, -vv, -vvv)
-q, --quiet              Suppress non-error output
-y, --yes                Skip confirmation prompts
    --timeout <SECS>     Request timeout (default: 30)
    --color <MODE>       Color: auto, always, never
```

### Shell Completions

```bash
# Bash
unifly completions bash > ~/.local/share/bash-completion/completions/unifly

# Zsh
unifly completions zsh > ~/.zfunc/_unifly

# Fish
unifly completions fish > ~/.config/fish/completions/unifly.fish

# PowerShell
unifly completions powershell | Out-String | Invoke-Expression
```

---

## 🖥️ TUI

The `unifly tui` subcommand launches a real-time terminal dashboard for monitoring and managing your UniFi network. Ten screens cover everything from live bandwidth charts to firewall policy management.

```bash
unifly tui                   # Launch with default profile
unifly tui -p office         # Use a specific profile
unifly tui -k                # Accept self-signed TLS certs
unifly tui -v                # Verbose logging
```

### Screens

Navigate with number keys `1`-`8`, `Tab`/`Shift+Tab`, or `,` for Settings:

| Key | Screen | Description |
| --- | --- | --- |
| `1` | **Dashboard** | btop-style overview: area-fill WAN traffic chart, gateway info, connectivity health, CPU/MEM bars, networks with IPv6, WiFi AP experience, top clients, recent events |
| `2` | **Devices** | Adopted devices with model, IP, CPU/MEM, TX/RX, uptime. 5-tab detail panel (Overview, Performance, Radios, Clients, Ports) |
| `3` | **Clients** | Connected clients: hostname, IP, MAC, VLAN, signal bars, traffic. Filterable by type (All/Wireless/Wired/VPN/Guest) |
| `4` | **Networks** | VLAN topology: subnets, DHCP, IPv6, gateway type. Inline edit overlay for live config changes |
| `5` | **Firewall** | Policies, zones, ACL rules, and NAT policies across four sub-tabs with visual rule reordering |
| `6` | **Topology** | Zoomable network topology tree: gateway to switches to APs, color-coded by type and state |
| `7` | **Events** | Live event stream with 10K rolling buffer. Pause, scroll back, severity color-coding |
| `8` | **Stats** | Historical charts: WAN bandwidth area fills, client counts, DPI app/category breakdown (1h/24h/7d/30d) |
| `,` | **Settings** | Controller profile, theme selector, display preferences |
| | **Onboarding** | First-run setup wizard for controller connection |

### Dashboard

The dashboard packs eight live panels into a dense, information-rich overview:

<p align="center">
  <img src="docs/images/dashboard.png" alt="unifly tui dashboard" width="900">
</p>

- **WAN Traffic** · Area-fill chart with Braille line overlay, live TX/RX rates, peak tracking
- **Gateway** · Model, firmware, WAN IP, IPv6, DNS, ISP, latency, uptime
- **Connectivity** · Subsystem status dots (WAN/WWW/WLAN/LAN/VPN), aggregate traffic bars
- **Capacity** · Color-coded CPU/MEM gauges, load averages, device/client fleet summary
- **Networks** · VLANs sorted by ID with IPv6 prefix delegation and SLAAC mode
- **WiFi / APs** · Client count, WiFi experience %, channel info per access point
- **Top Clients** · Proportional traffic bars with fractional block characters
- **Recent Events** · Two-column compact event display, color-coded by severity

### Devices & Clients

<p align="center">
  <img src="docs/images/devices.png" alt="unifly tui devices" width="900">
</p>

<p align="center">
  <img src="docs/images/clients.png" alt="unifly tui clients" width="900">
</p>

### Networks & Firewall

<p align="center">
  <img src="docs/images/networks.png" alt="unifly tui networks" width="900">
</p>

<p align="center">
  <img src="docs/images/firewall.png" alt="unifly tui firewall" width="900">
</p>

### Stats

Historical statistics with selectable time windows and dual-API data sourcing:

<p align="center">
  <img src="docs/images/stats.png" alt="unifly tui stats" width="900">
</p>

- **WAN Bandwidth** · TX/RX area fills with Braille line overlay, auto-scaling axes
- **Client Count** · Braille line chart tracking connected clients over time
- **Top Applications** · DPI application breakdown with proportional bars (Integration API names, Legacy fallback)
- **Traffic by Category** · Percentage bars for streaming, gaming, social, etc.

### Key Bindings

#### Global

| Key | Action |
| --- | --- |
| `1`-`8` | Jump to screen |
| `Tab` / `Shift+Tab` | Next / previous screen |
| `j` / `k` / `↑` / `↓` | Navigate up / down |
| `g` / `G` | Jump to top / bottom |
| `Ctrl+d` / `Ctrl+u` | Page down / up |
| `Enter` | Select / expand detail |
| `Esc` | Close detail / go back |
| `/` | Search |
| `?` | Help overlay |
| `,` | Settings |
| `q` | Quit |

#### Screen-Specific

| Screen | Key | Action |
| --- | --- | --- |
| **Devices** | `R` | Restart device |
| **Devices** | `L` | Locate (flash LED) |
| **Devices** (detail) | `h` / `l` | Previous / next detail tab |
| **Clients** | `Tab` | Cycle filter (All → Wireless → Wired → VPN → Guest) |
| **Clients** | `b` / `B` | Block / unblock client |
| **Clients** | `x` | Kick client |
| **Networks** | `e` | Edit selected network |
| **Firewall** | `h` / `l` | Cycle sub-tabs (Policies / Zones / ACL Rules / NAT) |
| **Firewall** | `K` / `J` | Reorder policy up / down |
| **Topology** | `←` `→` `↑` `↓` | Pan canvas |
| **Topology** | `+` / `-` | Zoom in / out |
| **Topology** | `f` | Fit to view |
| **Events** | `Space` | Pause / resume live stream |
| **Stats** | `h` `d` `w` `m` | Period: 1h / 24h / 7d / 30d |
| **Stats** | `r` | Refresh |

---

## 🏗️ Architecture

Two crates, clean dependency chain:

```
  unifly (CLI + TUI binaries)
       │
       ▼
  unifly-api (library)
  ├── HTTP/WS transport
  ├── Controller lifecycle
  ├── Reactive DataStore
  └── Domain models
```

| Crate | Purpose |
| --- | --- |
| **unifly-api** | Async HTTP/WebSocket client, Controller lifecycle, reactive DataStore (`DashMap` + `tokio::watch`), entity models. Published on [crates.io](https://crates.io/crates/unifly-api) |
| **unifly** | Single binary: CLI commands + `unifly tui` dashboard via feature flags, profile/keyring config, clap command routing, 10-screen ratatui dashboard with SilkCircuit theme |

### Data Flow

```
Controller URL ──▶ Integration API ──▶ REST (API key auth)
                   Legacy API ────────▶ REST (cookie + CSRF)
                   WebSocket ─────────▶ Push events

                         │
                         ▼
                    Controller
                   ┌──────────┐
                   │ DataStore │ ◀── DashMap + watch channels
                   │ Refresh   │ ◀── Background polling (30s)
                   │ Commands  │ ◀── Action channel (mpsc)
                   └──────────┘
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
           CLI out    TUI render  Event stream
```

The `Controller` (in `unifly-api`) wraps `Arc<ControllerInner>` for cheap cloning across async tasks. `EntityStream<T>` wraps `tokio::watch::Receiver` for reactive subscriptions so the TUI receives updates without polling.

---

## ⚙️ Configuration

Config lives in your platform-standard config directory:

| OS | Path |
| --- | --- |
| Linux | `~/.config/unifly/config.toml` |
| macOS | `~/Library/Application Support/unifly/config.toml` |
| Windows | `%APPDATA%\unifly\config.toml` |

```toml
default_profile = "home"

[defaults]
output = "table"
color = "auto"
insecure = false
timeout = 30

[profiles.home]
controller = "https://192.168.1.1"
site = "default"
auth_mode = "hybrid"
# API key + password stored in OS keyring

[profiles.office]
controller = "https://10.0.0.1"
site = "default"
auth_mode = "legacy"
username = "admin"
insecure = true
```

```bash
unifly config init             # Interactive setup
unifly config use office       # Switch active profile
unifly config profiles         # List profiles (* marks active)
unifly --profile home devices  # One-off override
```

---

## 📦 Library

The engine is published on [crates.io](https://crates.io). Use it to build your own UniFi tools, integrations, or automations in Rust.

[![unifly-api](https://img.shields.io/crates/v/unifly-api.svg)](https://crates.io/crates/unifly-api) · Async HTTP/WebSocket transport, high-level Controller, reactive DataStore, domain models

### Quick Start

**Low-level API access** · talk directly to the controller:

```rust
use unifly_api::{IntegrationClient, TransportConfig, TlsMode, ControllerPlatform};
use secrecy::SecretString;

let transport = TransportConfig::new(TlsMode::DangerAcceptInvalid);
let client = IntegrationClient::from_api_key(
    "https://192.168.1.1",
    &SecretString::from("your-api-key"),
    &transport,
    ControllerPlatform::UnifiOs,
)?;
let devices = client.list_devices("default").await?;
```

**High-level Controller** · reactive streams, automatic refresh, data merging:

```rust
use unifly_api::{Controller, ControllerConfig, AuthCredentials, TlsVerification};
use secrecy::SecretString;

let config = ControllerConfig {
    base_url: "https://192.168.1.1".parse()?,
    auth: AuthCredentials::ApiKey(SecretString::from("your-api-key")),
    tls: TlsVerification::DangerAcceptInvalid,
    ..Default::default()
};
let controller = Controller::new(config);
controller.connect().await?;

let devices = controller.devices().current();
println!("Found {} devices", devices.len());
```

Full API documentation on [docs.rs/unifly-api](https://docs.rs/unifly-api).

---

## 🤖 AI Agent Skill

### Install Options

```bash
npx skills add hyperb1iss/unifly                    # Claude Code, Cursor, Copilot, Codex, Gemini, ...
npx skills add hyperb1iss/unifly -a claude-code     # Target a specific agent
/plugin marketplace add hyperb1iss/unifly           # As a Claude Code plugin
```

### What's Included

| Component | Description |
| --- | --- |
| **unifly skill** | Complete CLI reference, command patterns, output formats, automation tips |
| **Network Manager agent** | Autonomous agent for provisioning, diagnostics, and security audits |
| **Reference docs** | Command reference, UniFi networking concepts, workflow patterns |

---

## 🦋 Development

### Prerequisites

- Rust 1.94+ (edition 2024)
- A UniFi Network controller (Cloud Key, Dream Machine, or self-hosted)

### Build

```bash
git clone https://github.com/hyperb1iss/unifly.git
cd unifly
cargo build --workspace
```

### Test & Lint

```bash
cargo test --workspace
cargo clippy --workspace --all-targets
```

### Run

```bash
cargo run -p unifly -- devices list
cargo run -p unifly -- tui
```

### Workspace Layout

```
crates/
  unifly-api/      # Library: HTTP/WS transport, Controller, DataStore, domain models
  unifly/          # Single binary: CLI commands + tui subcommand, config, profiles
```

### Lint Policy

Pedantic clippy with `unsafe_code = "forbid"`. See `Cargo.toml` workspace lints for the full configuration. It's opinionated and we like it that way.

---

## ⚖️ License

Apache-2.0. See [LICENSE](LICENSE)

---

<p align="center">
  <a href="https://github.com/sponsors/hyperb1iss">
    <img src="https://img.shields.io/badge/Sponsor-hyperb1iss-e135ff?style=for-the-badge&logo=githubsponsors&logoColor=white" alt="Sponsor on GitHub">
  </a>
  &nbsp;
  <a href="https://github.com/hyperb1iss/unifly">
    <img src="https://img.shields.io/github/stars/hyperb1iss/unifly?style=for-the-badge&logo=github&logoColor=white&color=80ffea" alt="Star on GitHub">
  </a>
</p>

<p align="center">
  <sub>
    If unifly keeps your network running smooth, <a href="https://github.com/sponsors/hyperb1iss"><strong>sponsor the project</strong></a> or give it a ⭐
    <br><br>
    ✦ Built with obsession by <a href="https://hyperbliss.tech"><strong>Hyperbliss Technologies</strong></a> ✦
  </sub>
</p>
