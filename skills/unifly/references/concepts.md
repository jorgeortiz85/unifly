# UniFi Networking Concepts

Reference for UniFi networking concepts and unifly-specific operational
details. Consult this file when the user's task requires decisions about
authentication mode, troubleshooting auth errors, or understanding the
dual-API architecture.

## Architecture

### Controller

The UniFi controller (UniFi OS Console) is the central management platform
for UniFi network devices. It runs on dedicated hardware (UDM, UDM Pro, UCG,
UXG) or as a self-hosted application (UniFi Network application for Linux).
unifly communicates with the controller via its REST APIs and WebSocket
events channel.

### Sites

A controller can manage multiple **sites**, which are logical groupings of
devices and configuration. Each site has its own networks, firewall rules,
WiFi SSIDs, and client database. Most unifly commands operate within a
single site context (set via `--site`, `UNIFI_SITE`, or the config profile).
The default site is named `default`.

### Device Types

| Prefix | Type                   | Examples                       |
| ------ | ---------------------- | ------------------------------ |
| UDM    | Dream Machine          | UDM, UDM Pro, UDM SE           |
| UCG    | Cloud Gateway          | UCG Ultra, UCG Max             |
| UXG    | Cloud Gateway (newer)  | UXG Pro, UXG Lite              |
| USG    | Security Gateway       | USG, USG Pro (legacy)          |
| USW    | Switch                 | USW Lite 8, USW Pro 24 PoE     |
| UAP    | Access Point (legacy)  | UAP-AC-Lite, UAP-AC-HD         |
| U6     | WiFi 6 Access Point    | U6-Lite, U6-Pro, U6-Enterprise |
| U7     | WiFi 7 Access Point    | U7-Pro, U7-Pro-Max             |
| UNVR   | Network Video Recorder | UNVR, UNVR Pro                 |
| UXBG   | Building Bridge        | UXBG Pro                       |

### Device States

- **ONLINE**: Device is connected and operating normally
- **OFFLINE**: Device is unreachable
- **PENDING**: Device discovered but not yet adopted
- **ADOPTING**: Adoption in progress
- **UPGRADING**: Firmware upgrade in progress
- **PROVISIONING**: Configuration being applied

## Dual-API Architecture

unifly is unusual among UniFi tools because it speaks both the modern
Integration API **and** the older Legacy API, reconciling data between them
where necessary. Understanding which API handles which operation is the most
important operational knowledge for agents.

### Integration API

- **Auth:** API key in `X-API-KEY` header
- **Base path:** `/proxy/network/integration/v1/`
- **Format:** Modern JSON with UUIDs
- **Returns:** Clean, well-typed responses without envelope wrapping
- **Limitation:** Missing several device-level operations, historical stats,
  WebSocket events, and some client enrichment fields (bytes, hostname,
  uplink MAC, VLAN, wireless)
- **Best for:** Configuration CRUD (networks, WiFi, firewall, NAT, DNS, ACL,
  traffic lists, hotspot, WANs, RADIUS)

### Legacy API

- **Auth:** Session cookie plus CSRF token (username + password login)
- **Base path:** `/proxy/network/api/` and `/proxy/network/v2/api/`
- **Format:** Envelope-wrapped JSON (`{"meta": {...}, "data": [...]}`)
- **Returns:** Everything the controller web UI sees, including fields the
  Integration API omits
- **Best for:** Events (WebSocket), stats, device commands (restart, adopt,
  locate, speedtest, port-cycle, upgrade, provision), DPI lifecycle, admin
  management, backups, site management

### Hybrid Mode

Hybrid merges both clients at login time. On every `devices list` or
`clients list`, unifly fetches the Integration API first, then supplements
each record with Legacy fields: `tx_bytes`/`rx_bytes`, `hostname`,
`wireless`, `uplink_device_mac`, `vlan`, `client_count` (mapped from Legacy
`num_sta`). Without Hybrid, these fields are silently absent.

Hybrid is the recommended default for agent use unless the task is
strictly configuration CRUD and controller credentials are intentionally
unavailable.

## Command Authentication Gate Matrix

Each unifly command calls either `ensure_integration_access` or
`ensure_legacy_access` (or both) before running. Commands in the wrong mode
fail with `Unsupported { required: "..." }`. Use this matrix to pick the
right `auth_mode`.

### Integration API required (API key)

- `acl` (list/get/create/update/delete/reorder)
- `dns` (list/get/create/update/delete)
- `firewall policies` (all subcommands)
- `firewall zones` (all subcommands)
- `hotspot` (list/get/create/delete/purge)
- `nat policies` (list/get/create/delete)
- `networks` (list/get/create/update/delete/refs)
- `traffic-lists` (list/get/create/update/delete)
- `wans` (list)
- `wifi` (list/get/create/update/delete)
- `countries`
- `radius profiles`

### Legacy API required (username + password)

- `admin` (list/invite/revoke/update): `/rest/admin`
- `alarms` (list/archive/archive-all)
- `clients reservations`, `clients set-ip`, `clients remove-ip`: `/rest/user`
- `devices` adopt, remove, restart, locate, port-cycle, upgrade, provision,
  speedtest (all route through `cmd/devmgr` and `cmd/stamgr`)
- `clients` authorize, unauthorize, block, unblock, kick, forget (via
  `cmd/stamgr`)
- `dpi status | enable | disable`: `/set/setting/dpi`
- `events list`, `events watch`: `/stat/event` and WebSocket
- `sites create | delete`
- `stats site | device | client | gateway | dpi`: `/stat/report/*`
- `system health | sysinfo | backup | reboot | poweroff`

### Hybrid-enriched (works in any mode, but richer in Hybrid)

- `clients list`: Integration fetch, Legacy fields merged by IP match
- `clients find`: inherits the merged view
- `devices list`: Integration fetch, Legacy `num_sta` merged by MAC
- `topology`: depends on merged `uplink_device_mac` for tree construction

### Raw API escape hatch

- `api <path>`: Routes through the Legacy client (handles CSRF and session
  automatically). Can reach Legacy, v2 (`v2/api/site/...`), and Integration
  (`integration/v1/...`) endpoints regardless of auth mode.

## Auth Mode Decision Tree

1. **"I only have an API key"** → `auth_mode = "integration"`. Configuration
   CRUD works. Events, stats, device commands will not.
2. **"I have username + password only"** → `auth_mode = "legacy"`. Events,
   stats, device commands work. Modern entities (DNS policies, NAT policies,
   traffic lists, ACL) require Integration and will fail.
3. **"I have both"** → `auth_mode = "hybrid"`. **Recommended default.**
   Everything works, client and device records are enriched.
4. **"Agent will manage multiple sites/controllers"** → Use named profiles
   (`-p home`, `-p office`) with Hybrid on each. Credentials go in the OS
   keyring via `unifly config set-password --profile <name>`.

## Configuration

### Platform-Native Config Paths

| OS      | Config File                                        |
| ------- | -------------------------------------------------- |
| Linux   | `~/.config/unifly/config.toml`                     |
| macOS   | `~/Library/Application Support/unifly/config.toml` |
| Windows | `%APPDATA%\unifly\config.toml`                     |

The CLI uses `ProjectDirs` for per-OS resolution. Agents should not assume
Linux paths on macOS or Windows.

### Environment Variables

Agent operations should prefer the `UNIFI_` prefix environment variables
over CLI flags when running in automation contexts:

| Variable         | Purpose                                              |
| ---------------- | ---------------------------------------------------- |
| `UNIFI_URL`      | Controller URL (overrides profile)                   |
| `UNIFI_API_KEY`  | Integration API key                                  |
| `UNIFI_USERNAME` | Legacy API username                                  |
| `UNIFI_PASSWORD` | Legacy API password (prefer keyring in interactive)  |
| `UNIFI_SITE`     | Target site name or UUID                             |
| `UNIFI_PROFILE`  | Active profile                                       |
| `UNIFI_OUTPUT`   | Default output format                                |
| `UNIFI_INSECURE` | `1` to accept self-signed TLS certs                  |
| `UNIFI_TIMEOUT`  | Request timeout in seconds                           |
| `UNIFI_TOTP`     | One-time password for MFA-protected accounts         |
| `NO_COLOR`       | Standard no-color flag (respected by output painter) |
| `UNIFLY_THEME`   | TUI theme name (TUI only, not CLI)                   |

Resolution priority (highest wins): CLI flags > environment variables >
config file > built-in defaults.

### MFA / TOTP

If the controller requires TOTP two-factor auth, set a totp source:

```toml
[profiles.home]
controller = "https://192.168.1.1"
auth_mode = "hybrid"
totp_env = "UNIFI_TOTP"
```

Supply the current code via the env var at runtime:

```bash
UNIFI_TOTP=$(op read "op://Personal/UniFi/one-time password") \
  unifly devices list
```

The `--totp` CLI flag also exists but is hidden from `--help`. Use it for
one-shot operations.

### Session Cache

unifly caches the Legacy session cookie across commands for speed. To force
a fresh login (e.g. after password rotation):

```bash
unifly --no-cache devices list
```

## Networking Primitives

### VLANs

Virtual LANs segment the network at Layer 2. UniFi networks accept a VLAN
ID in the range `1-4009` (unifly enforces this). The default network
typically uses VLAN 1 or is untagged.

Common VLAN design:

| VLAN | Name    | Subnet         | Purpose         |
| ---- | ------- | -------------- | --------------- |
| 1    | Default | 192.168.1.0/24 | Management      |
| 10   | Trusted | 10.0.10.0/24   | Trusted devices |
| 20   | Guest   | 10.0.20.0/24   | Guest isolation |
| 30   | IoT     | 10.0.30.0/24   | IoT devices     |
| 40   | Cameras | 10.0.40.0/24   | Surveillance    |

### Network Management Types

- **Gateway**: Routed network with DHCP, NAT, firewall (most common)
- **Switch**: Layer 2 only, no routing
- **VLAN-only**: Tag without a subnet, used in trunk scenarios

### DHCP

UniFi supports three DHCP modes:

- **Server**: Controller or gateway runs DHCP (most common)
- **Relay**: Forward DHCP to an upstream server
- **None**: No DHCP, static IPs only

### IPv6

Dual-stack networking supports:

- **SLAAC**: Stateless Address Autoconfiguration
- **DHCPv6**: Stateful IPv6 address assignment
- **Prefix delegation**: Automatic prefix from upstream (PD)

## Security Model

### Firewall Zones and Policies

UniFi uses a **zone-based firewall**. Zones group networks, and policies
control traffic between zone pairs.

Built-in zones:

- **Internal**: LAN networks
- **External**:WAN/Internet traffic
- **DMZ**: Public-facing services
- **VPN**: VPN-originated traffic
- **Hotspot**: Guest/hotspot networks

Policies define rules between source and destination zones:

- **Action**:`ALLOW`, `BLOCK`, `REJECT`
- **Direction**: Implied by zone pair
- **Logging**: Optional rule-level logging
- **Order**: First match wins; ordering matters

### NAT Policies

NAT policies (`unifly nat policies`) support three kinds:

- **Masquerade**: Source NAT using the outgoing interface address (most
  common for Internet-bound traffic)
- **Source NAT**: Explicit source address rewrite
- **Destination NAT**: Port forwarding and DNAT

### ACLs

ACLs (`unifly acl`) provide device-level access control independent of
firewall zones, filtering by IP, MAC, port, or protocol.

### Traffic Matching Lists

Reusable lists of ports, IPv4 addresses, or IPv6 addresses that firewall
policies, NAT policies, and ACLs reference. Reduces duplication across
rules.

## Events and Monitoring

### Event Categories

Events flowing through `unifly events watch` are tagged with one of these
categories (used by `--types` filter, case-insensitive):

| Category   | Examples                                             |
| ---------- | ---------------------------------------------------- |
| `Device`   | Adoption, restart, firmware update, port flap        |
| `Client`   | Connect, disconnect, roam, block, unblock, authorize |
| `Network`  | Interface state, VLAN changes, DHCP exhaustion       |
| `System`   | Controller restart, configuration push, backup       |
| `Admin`    | Login, logout, configuration change                  |
| `Firewall` | Policy hit (when logging enabled), IDS/IPS alert     |
| `Vpn`      | Tunnel up/down, client connect                       |
| `Unknown`  | Fallback for uncategorized events                    |

### Alarms

Alarms are persistent alerts that require acknowledgment. Distinct from
transient events. List with `unifly alarms list`, archive individually with
`unifly alarms archive <id>`, or clear everything with `unifly alarms
archive-all`.

### Historical Stats

`unifly stats` pulls from Legacy report endpoints. Supported intervals:

- `5minute`: High resolution, short retention window
- `hourly`: Medium resolution
- `daily`: Long-term trends
- `monthly`: Capacity planning

Subcommands: `site`, `device`, `client`, `gateway`, `dpi`. The `dpi`
subcommand supports `--group-by by-app` or `--group-by by-cat`.

### DPI (Deep Packet Inspection)

- `unifly dpi apps`: List known applications (Integration API)
- `unifly dpi categories`: List known categories (Integration API)
- `unifly dpi status`: Current DPI enable state (Legacy API)
- `unifly dpi enable`: Turn DPI on (Legacy API)
- `unifly dpi disable`: Turn DPI off (Legacy API)
- `unifly stats dpi`: Query DPI traffic breakdown

## Error Taxonomy

Common failures and how to diagnose them:

| Error                                         | Root Cause                                               | Fix                                         |
| --------------------------------------------- | -------------------------------------------------------- | ------------------------------------------- |
| `Unsupported { required: "Integration API" }` | Command needs API key, running in `legacy` mode          | Switch to `hybrid` or `integration` mode    |
| `Unsupported { required: "Legacy API" }`      | Command needs credentials, running in `integration` mode | Switch to `hybrid` or `legacy` mode         |
| 403 on POST/PUT/DELETE via `/proxy/network/`  | Missing CSRF token                                       | Re-login (cache invalidated); `--no-cache`  |
| `tls error: self-signed certificate`          | Controller uses self-signed TLS                          | Use `-k`/`--insecure` or `UNIFI_INSECURE=1` |
| `profile 'foo' not found`                     | No matching profile in config                            | Run `unifly config profiles` to list        |
| `keyring error`                               | Keyring daemon not running (Linux)                       | Unlock keyring or use plaintext config      |
| `Integration filter parse error`              | Bad filter DSL syntax                                    | Check `.eq('x')`, `.contains('y')` form     |
| Empty `clients list` wireless/bytes fields    | Integration-only mode                                    | Switch to Hybrid for enriched fields        |
| Silent result truncation (25 rows)            | Default list limit                                       | Pass `--all` or `--limit 200`               |

## Limits and Known Gaps

- **Local controllers only.** The Site Manager cloud API (`api.ui.com/v1/`)
  is not yet implemented. Do not attempt to use unifly against cloud-only
  controllers.
- **VPN mutations are not yet supported.** `unifly vpn servers` and
  `unifly vpn tunnels` are read-only.
- **Port forwarding** lives under `nat policies` with destination NAT, not
  a dedicated command.
- **No `nat policies update`.** Delete and recreate to modify a NAT policy.
- **DeviceFilter lacks a `BySite` variant.** Filter client-side after
  fetching if cross-site device filtering is required.
