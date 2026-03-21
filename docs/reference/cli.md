# CLI Reference

## Commands

| Command | Alias | Description |
|---|---|---|
| `devices` | `d` | Manage adopted and pending devices |
| `clients` | `cl` | Manage connected clients |
| `networks` | `n` | Manage networks and VLANs |
| `wifi` | `w` | Manage WiFi broadcasts (SSIDs) |
| `firewall` | `fw` | Manage firewall policies and zones |
| `acl` | | Manage ACL rules |
| `dns` | | Manage DNS policies (local records) |
| `traffic-lists` | | Manage traffic matching lists |
| `hotspot` | | Manage hotspot vouchers |
| `vpn` | | View VPN servers and tunnels |
| `sites` | | Manage sites |
| `events` | | View and stream events |
| `alarms` | | Manage alarms |
| `stats` | | Query statistics and reports |
| `system` | `sys` | System operations and info |
| `admin` | | Administrator management |
| `dpi` | | DPI reference data |
| `radius` | | View RADIUS profiles |
| `wans` | | View WAN interfaces |
| `countries` | | List available country codes |
| `config` | | Manage CLI configuration |
| `completions` | | Generate shell completions |

Most resource groups support `list` and `get`; some also expose `create`, `update`, `delete`, `patch`, or specialized actions. Run `unifly <command> --help` for details.

## Devices

```bash
unifly devices list                   # All adopted devices
unifly devices list --filter online   # Filter by status
unifly devices get <ID>               # Device details
unifly devices restart <ID>           # Restart a device
unifly devices upgrade <ID>           # Trigger firmware upgrade
unifly devices adopt <MAC>            # Adopt a pending device
```

## Clients

```bash
unifly clients list                   # Connected clients
unifly clients get <MAC>              # Client details
unifly clients block <MAC>            # Block a client
unifly clients unblock <MAC>          # Unblock a client
unifly clients kick <MAC>             # Force reconnection
```

## Networks

```bash
unifly networks list                  # All networks/VLANs
unifly networks get <ID>              # Network details
unifly networks create --name "IoT" --management gateway --vlan 20 --ipv4-host "10.0.20.1/24"
unifly networks update <ID> --enabled false
unifly networks delete <ID>
```

## WiFi

```bash
unifly wifi list                      # All SSIDs
unifly wifi get <ID>                  # SSID details
unifly wifi create --name "Guest" --network native --security wpa2-personal --passphrase "..."
unifly wifi update <ID> --enabled false
unifly wifi delete <ID>
```

## Firewall

```bash
unifly firewall policies list         # List firewall policies
unifly firewall zones list            # List firewall zones
unifly firewall policies get <ID>     # Policy details
```

## Events

```bash
unifly events list                    # Recent events
unifly events watch                   # Live event feed
unifly events watch --types EVT_SW_*  # Filter by event type
```

## Statistics

```bash
unifly stats gateway                  # Gateway bandwidth stats
unifly stats client                   # Client statistics
unifly stats site                     # Site-level statistics
```

Intervals: `5m`, `hourly`, `daily`, `monthly`

```bash
unifly stats gateway --interval hourly
```

## Configuration

```bash
unifly config init                    # Interactive setup
unifly config profiles                # List profiles
unifly config use <PROFILE>           # Switch active profile
unifly config show                    # Show current config
```

## Global Flags

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
    --api-key <KEY>      Integration API key
```
