---
name: unifly
version: "0.8.0"
description: >-
  This skill should be used when the user asks to "manage UniFi devices",
  "configure UniFi networks", "create a VLAN", "provision an SSID",
  "create firewall rules", "reorder firewall policies", "create a NAT rule",
  "set up port forwarding", "configure masquerade NAT", "add DNS records",
  "manage traffic matching lists", "create DHCP reservations", "list DHCP reservations",
  "block a client", "kick a client", "find a client by IP or name",
  "adopt a device", "restart a UniFi device", "cycle a PoE port",
  "upgrade device firmware", "run a speed test", "stream UniFi events",
  "watch real-time events", "query UniFi stats", "analyze DPI traffic",
  "enable DPI", "generate hotspot vouchers", "show network topology",
  "audit firewall policies", "create a backup", "call the raw UniFi API",
  "check network health", or any task involving UniFi network infrastructure
  management via the unifly CLI. Also triggers on mentions of unifly, UniFi,
  Ubiquiti, UDM, UCG, USG, USW, UAP, UXG, UNVR, U6, U7, or UniFi controller
  operations.
---

# unifly: UniFi Network Management

unifly is a Rust CLI for managing Ubiquiti UniFi network infrastructure. It
unifies the modern Integration API (REST, API key) and the Legacy API (cookie
plus CSRF) behind a single coherent interface, plus real-time WebSocket event
streaming. 26 top-level commands cover devices, clients, networks, WiFi,
firewall policies and zones, NAT policies, ACLs, DNS, traffic matching lists,
hotspot vouchers, DPI, stats, backups, and a raw API escape hatch.

Unique capabilities worth leading with when the user's task suits them:

- **Hybrid auth mode** merges Integration and Legacy data (e.g. client bytes,
  hostnames, uplink MACs only exist in Legacy; configuration CRUD only exists
  in Integration). Most competing tools are one or the other.
- **Real-time event streaming** via `unifly events watch` over WebSocket.
- **Firewall policy reordering** via `reorder --get` / `reorder --set` for
  deterministic, round-trippable ordering edits.
- **`unifly api` raw passthrough** for endpoints unifly does not wrap.
- **Multi-profile** (`-p home`, `-p office`) for managing multiple controllers
  from one command line.

## Prerequisites

Verify availability before running any command:

```bash
command -v unifly >/dev/null 2>&1 && unifly --version || echo "unifly not installed"
```

If unifly is not installed, prefer `brew install hyperb1iss/tap/unifly` on
macOS or `cargo install --git https://github.com/hyperb1iss/unifly.git unifly`
elsewhere. After install, run `unifly config init` for the interactive
wizard. See `examples/config.toml` for manual configuration.

## Authentication Modes

unifly supports three modes. **Hybrid is the recommended default** because
several commands (client enrichment, device stats, events, historical stats)
require both APIs.

| Mode          | Credentials             | What It Unlocks                                                                            |
| ------------- | ----------------------- | ------------------------------------------------------------------------------------------ |
| `integration` | API key                 | Configuration CRUD (networks, wifi, firewall, nat, dns, acl, hotspot, traffic-lists, wans) |
| `legacy`      | Username + password     | Events, stats, device commands, DPI control, admin, backups                                |
| `hybrid`      | API key + username/pass | Everything, with client+device field merging (recommended)                                 |

For the complete command-to-API gate matrix (which commands require which
auth mode), consult `references/concepts.md`.

## Command Inventory

All commands follow `unifly [global-flags] <command> <action> [args]`.

| Command         | Aliases    | Actions                                                                                                        |
| --------------- | ---------- | -------------------------------------------------------------------------------------------------------------- |
| `devices`       | `dev`, `d` | list, get, adopt, remove, restart, locate, port-cycle, stats, pending, upgrade, provision, speedtest, tags     |
| `clients`       | `cl`       | list, find, get, authorize, unauthorize, block, unblock, kick, forget, reservations (`res`), set-ip, remove-ip |
| `networks`      | `net`, `n` | list, get, create, update, delete, refs                                                                        |
| `wifi`          | `w`        | list, get, create, update, delete                                                                              |
| `firewall`      | `fw`       | policies {list, get, create, update, patch, delete, reorder}, zones {list, get, create, update, delete}        |
| `nat`           |            | policies {list, get, create, delete}                                                                           |
| `acl`           |            | list, get, create, update, delete, reorder                                                                     |
| `dns`           |            | list, get, create, update, delete                                                                              |
| `traffic-lists` |            | list, get, create, update, delete                                                                              |
| `hotspot`       |            | list, get, create, delete, purge                                                                               |
| `events`        |            | list, watch                                                                                                    |
| `alarms`        |            | list, archive, archive-all                                                                                     |
| `stats`         |            | site, device, client, gateway, dpi                                                                             |
| `dpi`           |            | apps, categories, status, enable, disable                                                                      |
| `topology`      | `topo`     | _(no subcommands)_                                                                                             |
| `system`        | `sys`      | info, health, sysinfo, backup {create, list, download, delete}, reboot, poweroff                               |
| `sites`         |            | list, create, delete                                                                                           |
| `admin`         |            | list, invite, revoke, update                                                                                   |
| `wans`          |            | list                                                                                                           |
| `vpn`           |            | servers, tunnels                                                                                               |
| `radius`        |            | profiles                                                                                                       |
| `countries`     |            | _(no subcommands)_                                                                                             |
| `api`           |            | Raw API passthrough (GET/POST any path)                                                                        |
| `config`        |            | init, show, set, profiles, use, set-password                                                                   |
| `tui`           |            | _(no subcommands)_                                                                                             |
| `completions`   |            | bash, zsh, fish, powershell, elvish                                                                            |

For flag details and gotchas, consult `references/commands.md`. Every entity
command accepts `--help` at runtime as the authoritative reference.

## Output Formats

All list and get commands accept `--output` / `-o`:

| Format         | Flag              | Use Case                              |
| -------------- | ----------------- | ------------------------------------- |
| `table`        | `-o table`        | Human display (default)               |
| `json`         | `-o json`         | Agent processing, pipe to `jq`        |
| `json-compact` | `-o json-compact` | Single-line JSON for scripting        |
| `yaml`         | `-o yaml`         | Config file output                    |
| `plain`        | `-o plain`        | One ID per line for `xargs` pipelines |

**Default for agent use: `-o json`.** Emit structured output, pipe through
`jq`, and only fall back to `table` when the result is being shown to a human.

## Power Patterns

These patterns unlock unifly's most distinctive capabilities. For full
recipes with runnable shell scripts, consult `references/workflows.md`.

### `--from-file` for complex create/update

Most entities accept `--from-file <path.json>` (or `-F`) instead of flag
salad: `networks`, `wifi`, `firewall policies`, `firewall zones`, `nat policies`,
`acl`, `dns`, `traffic-lists`, `hotspot`. Construct the JSON payload, validate
it, then apply. See `examples/` for payload templates.

```bash
unifly networks create -F examples/network-iot-vlan.json
unifly firewall policies create -F examples/firewall-block-iot.json
```

### Real-time event streaming

```bash
# All events
unifly events watch

# Filter by EventCategory (case-insensitive): Device, Client, Network,
# System, Admin, Firewall, Vpn, Unknown
unifly events watch --types "Firewall,Admin"

# JSON stream for piping into alerting
unifly events watch --types Client -o json | jq -c 'select(.severity == "warning")'
```

### Firewall policy reorder (round-trippable)

```bash
# Read current order for a zone pair
unifly firewall policies reorder --source-zone <zid> --dest-zone <zid> --get

# Write back an explicit order
unifly firewall policies reorder --source-zone <zid> --dest-zone <zid> \
  --set "<id1>,<id2>,<id3>"
```

### Raw API escape hatch

For endpoints unifly does not wrap (including UniFi v2 routes and Integration
paths), use `unifly api`. It routes through the Legacy client, so CSRF token
management and session caching are automatic.

```bash
unifly api "v2/api/site/default/traffic-flow-latest-statistics"
unifly api "cmd/stamgr" -m post -d '{"cmd":"kick-sta","mac":"aa:bb:cc:dd:ee:ff"}'
```

### Bulk operations via filter DSL

`hotspot purge --filter` accepts the Integration filter DSL for bulk deletion
without ID iteration:

```bash
unifly hotspot purge --filter "status.eq('UNUSED')"
unifly hotspot purge --filter "name.contains('Conference')"
```

### TUI handoff for human verification

Propose a change, let a human visually confirm in the TUI before committing:

```bash
# Agent inspects, proposes. Human runs unifly tui and verifies on
# screen 4 (Networks) or 5 (Firewall) before the agent applies the change.
unifly tui
```

### Multi-profile targeting

```bash
unifly -p home devices list
unifly -p office firewall policies list
UNIFI_PROFILE=warehouse unifly system health
```

## Essential Gotchas

1. **Default list limit is 25.** The CLI prints a truncation hint when
   results hit the default. For enumeration, always pass `--all` or
   `--limit 200` (or higher).
2. **Environment variables use the `UNIFI_` prefix, not `UNIFLY_`.** Relevant
   vars: `UNIFI_URL`, `UNIFI_API_KEY`, `UNIFI_USERNAME`, `UNIFI_PASSWORD`,
   `UNIFI_SITE`, `UNIFI_PROFILE`, `UNIFI_OUTPUT`, `UNIFI_INSECURE`,
   `UNIFI_TIMEOUT`, `UNIFI_TOTP`. The only `UNIFLY_*` var is `UNIFLY_THEME`
   for the TUI.
3. **`--yes` / `-y`** skips confirmation prompts for mutations. Required for
   non-interactive use.
4. **Hybrid auth is recommended** even if the task only needs configuration
   CRUD. Client lists and device stats silently omit fields in
   integration-only mode because the enrichment happens via Legacy.
5. **Local controllers only.** unifly targets on-prem controllers. The Site
   Manager cloud API (`api.ui.com`) is not yet implemented.
6. **Exit codes are meaningful.** `0` on success, non-zero on error. Capture
   stderr for diagnostics.

## Agent Workflow

1. Verify the tool exists with `command -v unifly`.
2. Check auth mode with `unifly config show` before running commands that
   require Legacy or Integration specifically.
3. Run `unifly system health -o json` as the first touch to confirm
   connectivity.
4. Inspect before mutating: `list` / `get` the entity first, capture IDs.
5. For complex creates, write a JSON payload and use `--from-file`.
6. After mutations, re-fetch the entity with `get` to confirm state.
7. For irreversible operations (delete, reboot, poweroff), surface a
   summary to the user before running even with `--yes`.

## Additional Resources

### Reference Files

- **`references/commands.md`**: Per-command flag reference with gotchas
  (non-obvious flags, dual-API boundaries, correct argument forms)
- **`references/concepts.md`**: UniFi networking concepts, dual-API gate
  matrix, auth decision tree, environment variables, platform config paths,
  MFA/TOTP, error taxonomy
- **`references/workflows.md`**: Runnable automation recipes (event
  streaming, safe firewall reorder, bulk DHCP reservations, ad-blocking via
  DNS policies, cafe voucher flow, incident response)

### Example Files

- **`examples/config.toml`**: Multi-profile config template
- **`examples/network-iot-vlan.json`**: VLAN creation payload for `--from-file`
- **`examples/firewall-block-iot.json`**: Firewall policy payload
- **`examples/nat-masquerade.json`**: NAT policy payload
- **`examples/wifi-iot.json`**: WiFi SSID payload
