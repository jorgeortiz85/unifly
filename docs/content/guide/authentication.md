+++
title = "Authentication"
description = "API key, credentials, hybrid, and cloud auth modes explained"
weight = 4
+++

Unifly supports three authentication modes. The right choice depends on whether you need live WebSocket streams.

## Which Mode Do I Need?

{% mermaid() %}
flowchart TD
START["What do you need?"] --> Q1{"Live event streaming<br/>(events watch or TUI)?"}
Q1 -->|"No"| APIKEY["API Key Mode"]
Q1 -->|"Yes"| HYBRID["Hybrid Mode"]
Q1 -->|"No API key<br/>available"| SESSION["Username/Password Mode"]

    style APIKEY fill:#50fa7b,color:#0a0a0f
    style HYBRID fill:#80ffea,color:#0a0a0f
    style SESSION fill:#f1fa8c,color:#0a0a0f

{% end %}

{% tip(title="Recommended") %}
**API Key mode** is the right default on UniFi OS controllers. A single Integration API key reaches both the Integration API and the Session API HTTP endpoints, covering almost every CLI command without a username or password. Only switch to Hybrid if you need live WebSocket features (`events watch`, `tui`).
{% end %}

## API Key

Generate a key on your controller under **Settings > Integrations**. On UniFi OS, the same key also authenticates the Session API HTTP endpoints (`/proxy/network/api/*` and `/proxy/network/v2/api/*`), so API key mode covers CRUD, device commands, stats, DHCP reservations, admin operations, and `events list`.

```bash
unifly config init                     # Select "API Key" during setup
unifly --api-key <KEY> devices list    # Or pass directly
```

| Pros                                     | Limitations                                     |
| ---------------------------------------- | ----------------------------------------------- |
| Simplest setup — no password juggling    | No live WebSocket events (requires cookie)      |
| Authenticates both Integration + Session | `events watch` and TUI live refresh unavailable |
| No CSRF bookkeeping                      | Classic standalone controllers may differ       |
| Stable, no token expiry                  |                                                 |

Best for: CI/CD pipelines, scripted provisioning, daily CLI automation, most everyday workflows.

## Username / Password

Session-based auth with cookie and CSRF token handling. Use this when you lack an Integration API key, when you need live WebSocket events, or when your controller is a classic standalone that does not accept API keys on the Session HTTP surface.

```bash
unifly config init                     # Select "Username/Password" during setup
```

| Pros                             | Limitations                               |
| -------------------------------- | ----------------------------------------- |
| Live WebSocket events + TUI live | Sessions expire periodically              |
| Full Session API access          | No access to modern Integration endpoints |
| Admin management                 | DNS, ACL, traffic lists unavailable       |

Best for: Monitoring-focused setups where you primarily care about live event streaming.

## Hybrid Mode

API key for HTTP (both Integration and Session) plus username/password for the live WebSocket cookie session. The setup wizard offers this when you provide both.

```bash
unifly config init                     # Select "Hybrid" during setup
```

| Capability                                        | How it's reached                               |
| ------------------------------------------------- | ---------------------------------------------- |
| Integration CRUD (networks, WiFi, firewall, etc.) | Integration API via `X-API-KEY`                |
| Session HTTP (stats, device commands, admin)      | Session API via `X-API-KEY`                    |
| Live event streaming (`events watch`, TUI)        | Session WebSocket via cookie session           |
| Client/device field enrichment                    | Session HTTP (merged into Integration records) |

How it works: unifly uses the API key for every HTTP request and the cookie session only to establish the live WebSocket. Everything else an API key can reach is reached with the API key.

To verify Hybrid is working, run `unifly events watch` — if events stream, the WebSocket cookie session is active.

## Credential Storage

All credentials are stored in your OS keyring:

| OS      | Backend                                 |
| ------- | --------------------------------------- |
| macOS   | Keychain                                |
| Linux   | Secret Service (GNOME Keyring, KWallet) |
| Windows | Windows Credential Manager              |

The `config.toml` file stores non-sensitive settings like controller URLs and site names. The setup wizard offers keyring storage by default, but also provides a plaintext config fallback for environments where the keyring isn't available (headless servers, WSL, CI).

To update a stored password:

```bash
unifly config set-password              # Updates the active profile
unifly config set-password --profile office  # Updates a specific profile
```

## Environment Variables

For CI/CD and scripting, pass credentials via environment:

```bash
export UNIFI_API_KEY="your-api-key-here"
export UNIFI_URL="https://192.168.1.1"
unifly devices list
```

{% mermaid() %}
graph LR
A["CLI Flags"] -->|highest| RESULT["Final Value"]
B["Environment Variables"] -->|medium| RESULT
C["Config File"] -->|lowest| RESULT

    style A fill:#ff6ac1,color:#0a0a0f
    style B fill:#f1fa8c,color:#0a0a0f
    style C fill:#80ffea,color:#0a0a0f

{% end %}

CLI flags override environment variables, which override config file values.

## MFA / TOTP

If your controller requires two-factor authentication:

```bash
# One-shot with 1Password CLI
UNIFI_TOTP=$(op read "op://Personal/UniFi/one-time password") \
  unifly devices list

# Or set totp_env in your config.toml profile:
# [profiles.home]
# totp_env = "UNIFI_TOTP"
```

{% tip() %}
The `totp_env` setting must be edited directly in `config.toml`. It is not yet supported by `unifly config set`.
{% end %}

## Next Steps

- [Configuration](/guide/configuration): full profile reference, environment variables, and precedence rules
- [CLI Commands](/reference/cli): what you can do with each auth mode
- [Troubleshooting](/troubleshooting): common auth errors and fixes
