# Security Policy

## Supported Versions

Security fixes are applied to the latest release on the `main` branch. Older versions do not
receive backports.

| Version | Supported |
| ------- | --------- |
| Latest  | Yes       |
| Older   | No        |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly.

**Email:** stef@hyperbliss.tech

**What to include:**

- Description of the vulnerability
- Steps to reproduce
- Impact assessment (what can an attacker do?)
- Suggested fix, if you have one

**Response timeline:**

- Acknowledgment within 48 hours
- Initial assessment within 7 days
- Fix or mitigation plan within 30 days for confirmed vulnerabilities

Please do not open public issues for security vulnerabilities. We'll coordinate disclosure
with you once a fix is available.

## Scope

unifly is a CLI and TUI that communicates with UniFi Network controllers over HTTPS and manages
network infrastructure. The primary security considerations include:

- **Credential handling** — API keys, session cookies, and login credentials flow through the
  tool and are stored via OS keyring or config files
- **TLS communication** — connections to controllers support custom CA certificates and an
  explicit "accept invalid" mode for self-signed certs
- **Network configuration** — the tool can modify firewall rules, NAT policies, DNS settings,
  ACLs, and other security-sensitive infrastructure
- **Cloud API access** — Site Manager fleet operations authenticate against `api.ui.com`

We take all reports seriously. Vulnerabilities that could lead to credential exposure or
unauthorized network configuration changes are treated as critical.
