# Roadmap

What's planned, what's known to be broken, and what's on the wish list.

## Known Gaps

Issues that exist in the current release and are documented so nobody wastes time rediscovering them.

- **Controller reconnect is broken.** The internal `CancellationToken` becomes permanent after the first disconnect. Reconnect does not work correctly yet.
- **Device `radios` field is always empty.** Parsing radio data from the `interfaces` JSON is not implemented.
- **TUI has no test coverage.** The 10 screens and widgets are untested. Adding tests here is welcomed.
- **CLI test coverage is thin.** `crates/unifly/tests/cli_test.rs` covers happy paths but many commands lack end-to-end tests.
- **NAT policies have no `update` subcommand.** Workaround: delete and recreate.
- **`nat` and `events` commands skip the Integration access gate.** They should call `ensure_integration_access` for clean error messages.
- **Plugin manifest version sync is manual.** `.claude-plugin/` and `.cursor-plugin/` manifests must be bumped by hand during releases.
- **ClawHub skill publish is manual.** Not yet wired into the release workflow.
- **AUR package update is manual.** Requires `just aur-update <version>` after each release.

## Next Up

Near-term priorities for the next 1-2 releases.

- **Cloud/Site Manager API support.** The `AuthCredentials::Cloud` variant exists but the transport is not yet implemented. This would enable managing controllers through Ubiquiti's cloud dashboard without direct network access.
- **`networks refs` for all entities.** Currently only networks have a "what depends on this?" command. Extend to WiFi, firewall zones, and other entities to make safe deletion easier.
- **Automate plugin manifest sync.** Add a CI step to the shared release workflow that patches version fields in plugin manifests before tagging.
- **Automate ClawHub publish.** Wire `npx clawhub publish ./skills/unifly` into the release workflow.
- **Fix controller reconnect lifecycle.** Replace the one-shot `CancellationToken` with proper reconnect state management.

## Wish List

Longer-term ideas, not yet committed.

- **Backup management.** Create, download, and restore controller backups via the Legacy API.
- **Firmware management.** Check for updates, schedule upgrades, track firmware versions across the fleet.
- **Multi-site dashboard.** Aggregate view across multiple sites in the TUI.
- **Device radio parsing.** Fill in the empty `radios` field from `interfaces` JSON so the TUI Devices detail panel shows radio info.
- **TUI test harness.** Headless rendering tests for screen components using `ratatui`'s test backend.
- **Config file migration.** Auto-upgrade config files when the schema changes between versions.
- **Batch operations.** Apply changes across multiple devices/networks in a single command (restart all APs, update all SSIDs, etc.).
