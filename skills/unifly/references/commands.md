# unifly Command Reference

Complete reference for all unifly CLI commands, flags, and arguments.

## Devices

### `unifly devices list`

List all adopted devices.

```bash
unifly devices list [--limit N] [--offset N] [--all] [--filter EXPR] [-o FORMAT]
```

### `unifly devices get <id|mac>`

Get detailed information about a specific device.

```bash
unifly devices get "aa:bb:cc:dd:ee:ff" -o json
unifly devices get "device-uuid" -o json
```

### `unifly devices adopt <mac>`

Adopt a device pending adoption.

```bash
unifly devices adopt "aa:bb:cc:dd:ee:ff" [--ignore-limit]
```

Flags:

- `--ignore-limit` — Adopt even if device limit is reached

### `unifly devices remove <id|mac>`

Remove (unadopt) a device from the controller.

```bash
unifly devices remove "aa:bb:cc:dd:ee:ff"
```

### `unifly devices restart <id|mac>`

Reboot a device.

```bash
unifly devices restart "aa:bb:cc:dd:ee:ff"
```

### `unifly devices locate <mac>`

Toggle the locate LED on a device (blink to physically identify it).

```bash
unifly devices locate "aa:bb:cc:dd:ee:ff"
```

### `unifly devices port-cycle <id|mac> <port_idx>`

Power-cycle a PoE port on a switch. The port index is zero-based.

```bash
unifly devices port-cycle "aa:bb:cc:dd:ee:ff" 5
```

### `unifly devices stats <id|mac>`

Get real-time statistics for a device (CPU, memory, throughput, clients).

```bash
unifly devices stats "aa:bb:cc:dd:ee:ff" -o json
```

### `unifly devices pending`

List devices awaiting adoption.

```bash
unifly devices pending -o json
```

### `unifly devices upgrade <mac>`

Upgrade device firmware. Optionally specify a custom firmware URL.

```bash
unifly devices upgrade "aa:bb:cc:dd:ee:ff"
unifly devices upgrade "aa:bb:cc:dd:ee:ff" --url "https://fw.example.com/firmware.bin"
```

### `unifly devices provision <mac>`

Force re-provision of device configuration.

```bash
unifly devices provision "aa:bb:cc:dd:ee:ff"
```

### `unifly devices speedtest`

Run a WAN speed test on the gateway.

```bash
unifly devices speedtest -o json
```

### `unifly devices tags`

List device tags.

```bash
unifly devices tags -o json
```

---

## Clients

### `unifly clients list`

List all connected clients.

Aliases: `ls`

```bash
unifly clients list [--limit N] [--offset N] [--all] [--filter EXPR] [-o FORMAT]
```

### `unifly clients find <query>`

Find clients by IP, name, hostname, or MAC address. Case-insensitive
substring matching across all fields.

Aliases: `search`

```bash
# Find by partial name
unifly clients find "macbook"

# Find by IP subnet
unifly clients find "10.4.22"

# Find by MAC prefix
unifly clients find "aa:bb:cc"
```

### `unifly clients get <id|mac>`

Get detailed information about a specific client.

```bash
unifly clients get "aa:bb:cc:dd:ee:ff" -o json
```

### `unifly clients authorize <client_id>`

Grant guest network access to a client.

```bash
unifly clients authorize <client_id> \
  --minutes MINUTES \
  [--data-limit-mb N] \
  [--tx-limit-kbps N] \
  [--rx-limit-kbps N]
```

Flags:

- `--minutes` — Access duration in minutes (required)
- `--data-limit-mb` — Data transfer limit in MB
- `--tx-limit-kbps` — Upload bandwidth limit
- `--rx-limit-kbps` — Download bandwidth limit

### `unifly clients unauthorize <client_id>`

Revoke guest access for a client.

```bash
unifly clients unauthorize <client_id>
```

### `unifly clients block <mac>`

Block a client from the network (Legacy API).

```bash
unifly clients block "aa:bb:cc:dd:ee:ff"
```

### `unifly clients unblock <mac>`

Unblock a previously blocked client (Legacy API).

```bash
unifly clients unblock "aa:bb:cc:dd:ee:ff"
```

### `unifly clients kick <mac>`

Disconnect a wireless client (Legacy API). The client may reconnect.

```bash
unifly clients kick "aa:bb:cc:dd:ee:ff"
```

### `unifly clients forget <mac>`

Remove a client from the controller's history entirely (Legacy API).

```bash
unifly clients forget "aa:bb:cc:dd:ee:ff"
```

### `unifly clients set-ip <mac> --ip <ipv4> [--network <name|id>]`

Set a DHCP reservation (fixed IP) for a client. Auto-detects the network
from the IP subnet, or specify explicitly with `--network`.

Aliases: `reserve`

```bash
# Auto-detect network from IP
unifly clients set-ip "00:11:22:33:44:55" --ip 10.4.22.11

# Explicit network
unifly clients set-ip "00:11:22:33:44:55" --ip 10.4.22.11 --network IoT
```

### `unifly clients remove-ip <mac>`

Remove a DHCP reservation from a client.

Aliases: `unreserve`

```bash
unifly clients remove-ip "00:11:22:33:44:55"
```

---

## Networks

### `unifly networks list`

List all configured networks.

```bash
unifly networks list [--limit N] [--all] [-o FORMAT]
```

### `unifly networks get <id>`

Get detailed network configuration including IPv4, DHCP, IPv6 settings.

```bash
unifly networks get "network-uuid" -o json
```

### `unifly networks create`

Create a new network (VLAN).

```bash
unifly networks create \
  --name "IoT" \
  --vlan 30 \
  --management gateway \
  --ipv4-host 10.0.30.1/24 \
  --dhcp --dhcp-start 10.0.30.100 --dhcp-stop 10.0.30.254 \
  [--zone <zone-id>] \
  [--isolated]
```

Flags:

- `--name` — Network name (required)
- `--vlan` — VLAN ID (1-4094)
- `--management` — `gateway`, `switch`, or `unmanaged`
- `--ipv4-host` — Gateway IP with CIDR prefix (e.g., `10.0.30.1/24`)
- `--dhcp` — Enable DHCP server
- `--dhcp-start` — DHCP range start IP
- `--dhcp-stop` — DHCP range end IP
- `--dhcp-lease` — Lease time in seconds
- `--zone` — Firewall zone to attach to
- `--isolated` — Enable client isolation
- `--internet` — Allow internet access (default: true)
- `-F` / `--from-file` — Create from JSON file

### `unifly networks update <id>`

Update an existing network. Supports same flags as create plus `--from-file`.

```bash
unifly networks update "network-uuid" \
  [--name "New Name"] \
  [--enabled true|false] \
  [--vlan N]
unifly networks update "network-uuid" -F network.json
```

### `unifly networks delete <id>`

Delete a network.

```bash
unifly networks delete "network-uuid"
```

### `unifly networks refs <id>`

Show cross-references — what entities reference this network (WiFi SSIDs,
firewall zones, port profiles, etc.).

```bash
unifly networks refs "network-uuid" -o json
```

---

## WiFi

### `unifly wifi list`

List all WiFi broadcasts (SSIDs).

```bash
unifly wifi list [-o FORMAT]
```

### `unifly wifi get <id>`

Get SSID configuration details.

```bash
unifly wifi get "wifi-uuid" -o json
```

### `unifly wifi create`

Create a new WiFi broadcast.

```bash
unifly wifi create \
  --name "Guest WiFi" \
  --broadcast-type standard \
  --security wpa2-personal \
  --passphrase "SecurePass123!" \
  --network "network-uuid" \
  [--frequencies 2.4,5] \
  [--band-steering] \
  [--fast-roaming]
```

Flags:

- `--name` — SSID name (required)
- `--broadcast-type` — `standard` or `iot-optimized` (default: standard)
- `--security` — `open`, `wpa2-personal`, `wpa3-personal`, `wpa2-wpa3-personal`, `wpa2-enterprise`, `wpa3-enterprise`
- `--passphrase` — WiFi password (required for personal security)
- `--network` — Associated network UUID or name
- `--frequencies` — Comma-separated radio bands (e.g., `2.4,5`)
- `--hidden` — Hide SSID from broadcast
- `--band-steering` — Enable band steering (boolean flag)
- `--fast-roaming` — Enable 802.11r fast roaming (boolean flag)
- `-F` / `--from-file` — Create from JSON file

### `unifly wifi update <id>`

Update an existing SSID.

```bash
unifly wifi update "wifi-uuid" \
  [--name "New SSID"] \
  [--passphrase "NewPass456!"] \
  [--enabled true|false]
unifly wifi update "wifi-uuid" -F wifi.json
```

### `unifly wifi delete <id>`

Delete a WiFi broadcast.

```bash
unifly wifi delete "wifi-uuid"
```

---

## Firewall

### Policies

#### `unifly firewall policies list`

List all firewall policies.

```bash
unifly firewall policies list [-o FORMAT]
```

#### `unifly firewall policies get <id>`

Get firewall policy details.

```bash
unifly firewall policies get "policy-uuid" -o json
```

#### `unifly firewall policies create`

Create a new firewall policy with optional traffic filters.

```bash
unifly firewall policies create \
  --name "Block IoT to LAN" \
  --action allow|block|reject \
  --source-zone <zone-uuid> \
  --dest-zone <zone-uuid> \
  [--src-network <network-id,...>] \
  [--src-ip <ip,cidr,range,...>] \
  [--src-port <port,range,...>] \
  [--dst-network <network-id,...>] \
  [--dst-ip <ip,cidr,range,...>] \
  [--dst-port <port,range,...>] \
  [--states NEW,ESTABLISHED,RELATED,INVALID] \
  [--ip-version IPV4_ONLY|IPV6_ONLY|IPV4_AND_IPV6] \
  [--logging] \
  [--from-file policy.json]
```

**Traffic filter flags:**
- `--src-network` / `--dst-network` — Filter by network IDs (comma-separated UUIDs)
- `--src-ip` / `--dst-ip` — Filter by IP addresses, CIDRs, or ranges (e.g. `10.0.0.1,10.0.0.0/24,10.0.0.1-10.0.0.100`)
- `--src-port` / `--dst-port` — Filter by ports or port ranges (e.g. `80,443,8000-9000`)

Priority: if multiple filter types specified, network > ip > port (first wins).

#### `unifly firewall policies update <id>`

Update a firewall policy. Supports the same traffic filter flags as create.
Preserves existing fields not specified in the update.

```bash
# Update destination to specific IPs
unifly firewall policies update "policy-uuid" \
  --dst-ip 10.4.20.21,10.4.20.20

# Update via JSON file
unifly firewall policies update "policy-uuid" -F policy.json
```

#### `unifly firewall policies patch <id>`

Quick toggle for logging and enabled state.

```bash
unifly firewall policies patch "policy-uuid" --logging false
unifly firewall policies patch "policy-uuid" --enabled false
```

#### `unifly firewall policies delete <id>`

Delete a firewall policy.

```bash
unifly firewall policies delete "policy-uuid"
```

#### `unifly firewall policies reorder`

Get or set the evaluation order of firewall policies between zone pairs.

```bash
# Get current ordering
unifly firewall policies reorder \
  --source-zone "zone-uuid" --dest-zone "zone-uuid" --get

# Set new ordering (first match wins)
unifly firewall policies reorder \
  --source-zone "zone-uuid" --dest-zone "zone-uuid" \
  --set "id1,id2,id3"
```

Flags:

- `--source-zone` — Source zone UUID (required)
- `--dest-zone` — Destination zone UUID (required)
- `--get` — Print current policy order (conflicts with `--set`)
- `--set` — Comma-separated policy IDs in desired order (conflicts with `--get`)

### Zones

#### `unifly firewall zones list`

List all firewall zones.

```bash
unifly firewall zones list [-o FORMAT]
```

#### `unifly firewall zones get <id>`

Get zone details including attached networks.

```bash
unifly firewall zones get "zone-uuid" -o json
```

#### `unifly firewall zones create`

Create a custom firewall zone.

```bash
unifly firewall zones create \
  --name "IoT Zone" \
  --networks "net-uuid-1,net-uuid-2"
```

#### `unifly firewall zones update <id>`

Update a zone.

```bash
unifly firewall zones update "zone-uuid" \
  [--name "Renamed Zone"] \
  [--networks "net-uuid-1,net-uuid-2"]
```

#### `unifly firewall zones delete <id>`

Delete a zone.

```bash
unifly firewall zones delete "zone-uuid"
```

---

## ACL (Access Control Lists)

### `unifly acl list`

List ACL rules.

```bash
unifly acl list [-o FORMAT]
```

### `unifly acl get <id>`

Get ACL rule details.

```bash
unifly acl get "acl-uuid" -o json
```

### `unifly acl create`

Create an ACL rule.

```bash
unifly acl create \
  --rule-type ipv4|mac \
  --action allow|block \
  [additional flags per type]
```

### `unifly acl update <id>`

Update an ACL rule.

```bash
unifly acl update "acl-uuid" [flags]
```

### `unifly acl delete <id>`

Delete an ACL rule.

```bash
unifly acl delete "acl-uuid"
```

### `unifly acl reorder`

Get or set ACL rule evaluation order.

```bash
# Get current ordering
unifly acl reorder --get

# Set new ordering
unifly acl reorder --set "id1,id2,id3"
```

---

## DNS

### `unifly dns list`

List local DNS policies/records.

```bash
unifly dns list [-o FORMAT]
```

### `unifly dns get <id>`

Get DNS record details.

```bash
unifly dns get "dns-uuid" -o json
```

### `unifly dns create`

Create a DNS record.

```bash
unifly dns create \
  --record-type A|AAAA|CNAME|MX|TXT|SRV|Forward \
  --domain "app.local" \
  --value "10.0.1.50" \
  [--ttl 3600] \
  [--priority 10]
```

Supported record types:

| Type    | Description    | Value Format                  |
| ------- | -------------- | ----------------------------- |
| A       | IPv4 address   | `10.0.1.50`                   |
| AAAA    | IPv6 address   | `fd00::1`                     |
| CNAME   | Canonical name | `other.local`                 |
| MX      | Mail exchange  | `mail.example.com`            |
| TXT     | Text record    | `"v=spf1 ..."`                |
| SRV     | Service record | `target:port:weight:priority` |
| Forward | DNS forwarding | `8.8.8.8`                     |

### `unifly dns update <id>`

Update a DNS record via JSON file.

```bash
unifly dns update "dns-uuid" -F dns-record.json
```

### `unifly dns delete <id>`

Delete a DNS record.

```bash
unifly dns delete "dns-uuid"
```

---

## Traffic Lists

### `unifly traffic-lists list`

List traffic matching lists.

```bash
unifly traffic-lists list [-o FORMAT]
```

### `unifly traffic-lists create`

Create a traffic matching list with port, IPv4, or IPv6 items.

```bash
unifly traffic-lists create \
  --name "Blocked Ports" \
  --list-type ports|ipv4|ipv6 \
  --items "80,443,8080"
```

### `unifly traffic-lists update <id>`

Update a traffic list.

```bash
unifly traffic-lists update "list-uuid" [--name "..."] [--items "..."]
```

### `unifly traffic-lists delete <id>`

Delete a traffic list.

```bash
unifly traffic-lists delete "list-uuid"
```

---

## Hotspot (Vouchers)

### `unifly hotspot list`

List guest vouchers.

```bash
unifly hotspot list [-o FORMAT]
```

### `unifly hotspot get <id>`

Get details for a specific voucher.

```bash
unifly hotspot get "voucher-uuid" -o json
```

### `unifly hotspot create`

Generate guest vouchers.

```bash
unifly hotspot create \
  --name "Conference" \
  --count 10 \
  --minutes 1440 \
  [--guest-limit 1] \
  [--data-limit-mb 500] \
  [--tx-limit-kbps 5000] \
  [--rx-limit-kbps 10000]
```

Flags:

- `--name` — Voucher batch name (required)
- `--count` — Number of vouchers to generate (default: 1)
- `--minutes` — Duration in minutes (required, 1440 = 24 hours)
- `--guest-limit` — Max concurrent guests per voucher
- `--data-limit-mb` — Data cap in MB
- `--tx-limit-kbps` — Upload bandwidth cap
- `--rx-limit-kbps` — Download bandwidth cap

### `unifly hotspot delete <id>`

Delete a single voucher.

```bash
unifly hotspot delete "voucher-uuid"
```

### `unifly hotspot purge`

Bulk delete vouchers matching a filter.

```bash
unifly hotspot purge --filter "status.eq('UNUSED')"
```

---

## VPN

### `unifly vpn servers`

List VPN server configurations.

```bash
unifly vpn servers [-o FORMAT]
```

### `unifly vpn tunnels`

List site-to-site VPN tunnels.

```bash
unifly vpn tunnels [-o FORMAT]
```

---

## Sites

### `unifly sites list`

List sites on the controller.

```bash
unifly sites list [-o FORMAT]
```

### `unifly sites create`

Create a new site (Legacy API).

```bash
unifly sites create --name "Branch Office"
```

### `unifly sites delete`

Delete a site (Legacy API).

```bash
unifly sites delete --name "Branch Office"
```

---

## Events

### `unifly events list`

List recent events.

```bash
unifly events list [--within 24] [-o FORMAT]
```

- `--within` — Lookback period in hours (default: 24)

### `unifly events watch`

Stream real-time events via WebSocket.

```bash
unifly events watch [--types "EVT_SW_*"]
```

- `--types` — Filter by event type pattern (comma-separated glob matching)

---

## Alarms

### `unifly alarms list`

List alarms.

```bash
unifly alarms list [--unarchived] [-o FORMAT]
```

- `--unarchived` — Show only active (unarchived) alarms

### `unifly alarms archive <id>`

Archive a single alarm.

```bash
unifly alarms archive "alarm-id"
```

### `unifly alarms archive-all`

Archive all alarms.

```bash
unifly alarms archive-all
```

---

## Statistics

### `unifly stats site`

Site-level statistics.

```bash
unifly stats site \
  [--interval 5m|hourly|daily|monthly] \
  [--start "2024-01-01T00:00:00Z"] \
  [--end "2024-01-31T23:59:59Z"] \
  [--attrs "bytes,num_sta"] \
  [-o FORMAT]
```

### `unifly stats device`

Per-device statistics.

```bash
unifly stats device \
  [--macs "aa:bb:cc:dd:ee:ff"] \
  [--interval hourly] \
  [--start "..."] [--end "..."] \
  [-o FORMAT]
```

### `unifly stats client`

Per-client statistics.

```bash
unifly stats client \
  [--macs "aa:bb:cc:dd:ee:ff"] \
  [--interval hourly] \
  [-o FORMAT]
```

### `unifly stats gateway`

Gateway statistics.

```bash
unifly stats gateway [--interval hourly] [-o FORMAT]
```

### `unifly stats dpi`

Deep packet inspection traffic analysis.

```bash
unifly stats dpi \
  [--group-by by-app|by-cat] \
  [--macs "aa:bb:cc:dd:ee:ff"] \
  [-o FORMAT]
```

Flags common to all stats commands:

- `--interval` — Aggregation interval: `5m`, `hourly`, `daily`, `monthly`
- `--start` — Start of time range (ISO 8601)
- `--end` — End of time range (ISO 8601)
- `--attrs` — Comma-separated attribute names to include
- `--macs` — Filter by specific device/client MAC addresses

---

## System

### `unifly system info`

Show application version information.

```bash
unifly system info [-o FORMAT]
```

### `unifly system health`

Show site health summary.

```bash
unifly system health [-o FORMAT]
```

### `unifly system sysinfo`

Show controller system information.

```bash
unifly system sysinfo [-o FORMAT]
```

### `unifly system backup create`

Create a controller backup.

```bash
unifly system backup create
```

### `unifly system backup list`

List available backups.

```bash
unifly system backup list [-o FORMAT]
```

### `unifly system backup download <filename>`

Download a backup file.

```bash
unifly system backup download "autobackup_2024-01-15.unf"
```

### `unifly system backup delete <filename>`

Delete a backup.

```bash
unifly system backup delete "autobackup_2024-01-15.unf"
```

### `unifly system reboot`

Reboot the controller (UDM only).

```bash
unifly system reboot
```

### `unifly system poweroff`

Power off the controller (UDM only).

```bash
unifly system poweroff
```

---

## Admin

### `unifly admin list`

List site administrators.

```bash
unifly admin list [-o FORMAT]
```

### `unifly admin invite`

Invite a new administrator.

```bash
unifly admin invite \
  --name "Jane Admin" \
  --email "jane@example.com" \
  --role admin|readonly|viewer
```

### `unifly admin revoke`

Remove administrator access.

```bash
unifly admin revoke --email "jane@example.com"
```

### `unifly admin update`

Change an administrator's role.

```bash
unifly admin update --email "jane@example.com" --role readonly
```

---

## DPI

### `unifly dpi apps`

List DPI applications.

```bash
unifly dpi apps [-o FORMAT]
```

### `unifly dpi categories`

List DPI categories.

```bash
unifly dpi categories [-o FORMAT]
```

---

## RADIUS

### `unifly radius profiles`

List RADIUS profiles.

```bash
unifly radius profiles [-o FORMAT]
```

---

## WANs

### `unifly wans list`

List WAN interfaces.

```bash
unifly wans list [-o FORMAT]
```

---

## Topology

### `unifly topology`

Display a network topology tree showing the device hierarchy and connected
clients. The tree starts at the gateway and branches through switches and
access points, with clients grouped under their uplink device.

Aliases: `topo`

```bash
unifly topology
```

Output includes:
- Gateway at root with model and IP
- Infrastructure devices (switches, APs) grouped by type
- Clients listed under their uplink device
- VLAN/network labels per client
- Signal strength for wireless clients
- Color-coded by device type and connection state

No subcommands or flags beyond the standard global flags.

---

## TUI

### `unifly tui`

Launch the real-time terminal dashboard.

```bash
unifly tui
unifly tui -p office
```

The same global flags apply, including `--insecure`, `--timeout`, `--output`,
and `--profile`.

---

## Countries

### `unifly countries`

List available country/region codes. Useful when configuring WiFi radio
settings that require a regulatory country code.

```bash
unifly countries
unifly countries -o json
```

Output: two-column table of Code and Name.

---

## Config

### `unifly config init`

Interactive setup wizard for first-time configuration.

```bash
unifly config init
```

### `unifly config show`

Display the resolved configuration.

```bash
unifly config show
```

### `unifly config set <key> <value>`

Set a configuration value on the active profile. You can also target a named
profile explicitly with `profiles.<name>.<key>`.

```bash
unifly -p home config set controller "https://192.168.1.1"
unifly -p home config set auth_mode "hybrid"
unifly -p home config set api_key "your-api-key"
unifly config set profiles.home.controller "https://192.168.1.1"
```

Valid keys: `controller`, `site`, `auth_mode`, `api_key`, `api_key_env`,
`username`, `insecure`, `timeout`, `ca_cert`.

### `unifly config profiles`

List configured profiles.

```bash
unifly config profiles
```

### `unifly config use <name>`

Set the default profile.

```bash
unifly config use home
```

### `unifly config set-password <profile>`

Store a password in the OS keyring.

```bash
unifly config set-password --profile home
```

---

## Completions

Generate shell completions.

```bash
unifly completions bash > ~/.bash_completion.d/unifly
unifly completions zsh > ~/.zfunc/_unifly
unifly completions fish > ~/.config/fish/completions/unifly.fish
```

Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`.
