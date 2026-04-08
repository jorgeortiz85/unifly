#![cfg(feature = "e2e")]
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;
use std::process::Output;

use assert_cmd::cargo::cargo_bin_cmd;
use tempfile::TempDir;

struct E2eConfig {
    controller_url: String,
    username: String,
    password: String,
    site: String,
}

impl E2eConfig {
    fn from_env() -> Self {
        Self {
            controller_url: std::env::var("UNIFLY_E2E_URL")
                .unwrap_or_else(|_| "https://localhost:8443".into()),
            username: std::env::var("UNIFLY_E2E_USERNAME").unwrap_or_else(|_| "admin".into()),
            password: std::env::var("UNIFLY_E2E_PASSWORD").unwrap_or_else(|_| "admin".into()),
            site: std::env::var("UNIFLY_E2E_SITE").unwrap_or_else(|_| "default".into()),
        }
    }

    fn with_password(&self, password: &str) -> Self {
        Self {
            controller_url: self.controller_url.clone(),
            username: self.username.clone(),
            password: password.into(),
            site: self.site.clone(),
        }
    }
}

struct E2eContext {
    config: E2eConfig,
    tempdir: TempDir,
}

impl E2eContext {
    fn session() -> Self {
        Self::with_config(E2eConfig::from_env(), "session")
    }

    fn session_with_password(password: &str) -> Self {
        let config = E2eConfig::from_env().with_password(password);
        Self::with_config(config, "session")
    }

    fn with_config(config: E2eConfig, auth_mode: &str) -> Self {
        let tempdir = tempfile::tempdir().unwrap();
        write_profile(tempdir.path(), &config, auth_mode);
        Self { config, tempdir }
    }

    fn cmd(&self) -> assert_cmd::Command {
        let mut cmd = cargo_bin_cmd!("unifly");
        cmd.env("HOME", self.tempdir.path())
            .env("XDG_CONFIG_HOME", self.tempdir.path())
            .env("APPDATA", self.tempdir.path())
            .env("LOCALAPPDATA", self.tempdir.path())
            .env("USERPROFILE", self.tempdir.path())
            .env("UNIFI_USERNAME", &self.config.username)
            .env("UNIFI_PASSWORD", &self.config.password)
            .env("UNIFI_TIMEOUT", "60")
            .env_remove("UNIFI_PROFILE")
            .env_remove("UNIFI_API_KEY");
        cmd
    }

    fn run(&self, args: &[&str]) -> Output {
        self.cmd().args(args).output().unwrap()
    }
}

fn write_profile(root: &Path, config: &E2eConfig, auth_mode: &str) {
    let config_dir = root.join("unifly");
    fs::create_dir_all(&config_dir).unwrap();

    let body = format!(
        r#"
default_profile = "default"

[profiles.default]
controller = "{controller}"
site = "{site}"
auth_mode = "{auth_mode}"
insecure = true
"#,
        controller = config.controller_url,
        site = config.site,
        auth_mode = auth_mode,
    );

    fs::write(config_dir.join("config.toml"), body.trim_start()).unwrap();
}

fn stdout_json(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} failed, stderr:\n{}",
        stderr_text(output)
    );
}

fn json_array<'a>(payload: &'a serde_json::Value, context: &str) -> &'a [serde_json::Value] {
    payload
        .as_array()
        .unwrap_or_else(|| panic!("expected {context} to render a JSON array, got {payload}"))
}

fn json_bool(payload: &serde_json::Value, key: &str) -> bool {
    payload
        .get(key)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or_else(|| panic!("expected boolean field '{key}' in {payload}"))
}

fn bool_arg(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

struct RestoreCommand<'a> {
    ctx: &'a E2eContext,
    args: Vec<String>,
}

impl<'a> RestoreCommand<'a> {
    fn new(ctx: &'a E2eContext, args: &[&str]) -> Self {
        Self {
            ctx,
            args: args.iter().map(ToString::to_string).collect(),
        }
    }
}

impl Drop for RestoreCommand<'_> {
    fn drop(&mut self) {
        let args = self.args.iter().map(String::as_str).collect::<Vec<_>>();
        let _ = self.ctx.run(&args);
    }
}

#[test]
fn session_profile_smoke_test_reads_sysinfo() {
    let ctx = E2eContext::session();

    let output = ctx.run(&["system", "sysinfo", "-o", "json"]);

    assert_success(&output, "system sysinfo");

    let payload = stdout_json(&output);
    assert!(payload.is_object(), "expected JSON object, got {payload}");
}

#[test]
fn session_profile_lists_simulated_devices() {
    let ctx = E2eContext::session();
    let output = ctx.run(&["devices", "list", "--all", "-o", "json"]);

    assert_success(&output, "devices list");

    let payload = stdout_json(&output);
    let devices = json_array(&payload, "devices list");

    assert!(
        !devices.is_empty(),
        "expected simulation mode to expose demo devices"
    );
}

#[test]
fn session_profile_lists_default_site() {
    let ctx = E2eContext::session();
    let output = ctx.run(&["sites", "list", "--all", "-o", "json"]);

    assert_success(&output, "sites list");

    let payload = stdout_json(&output);
    let sites = json_array(&payload, "sites list");

    assert!(
        sites.iter().any(|site| {
            site.get("name")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|name| name == "Default" || name == "default")
        }),
        "expected the default site to be present, got {}",
        stdout_text(&output)
    );
}

#[test]
fn session_profile_raw_api_sysinfo_returns_json() {
    let ctx = E2eContext::session();
    let path = format!("api/s/{}/stat/sysinfo", ctx.config.site);
    let output = ctx
        .cmd()
        .args(["api", &path, "-o", "json"])
        .output()
        .unwrap();

    assert_success(&output, "raw api sysinfo");

    let payload = stdout_json(&output);
    assert!(payload.is_object(), "expected JSON object, got {payload}");
}

#[test]
fn session_profile_output_formats_render_cleanly() {
    let ctx = E2eContext::session();

    let table = ctx.run(&["sites", "list", "--all"]);
    assert_success(&table, "sites list table output");
    assert!(stdout_text(&table).contains("default") || stdout_text(&table).contains("Default"));

    let yaml = ctx.run(&["sites", "list", "--all", "-o", "yaml"]);
    assert_success(&yaml, "sites list yaml output");
    assert!(stdout_text(&yaml).contains("name:"));

    let plain = ctx.run(&["sites", "list", "--all", "-o", "plain"]);
    assert_success(&plain, "sites list plain output");
    assert!(!stdout_text(&plain).trim().is_empty());
}

#[test]
fn session_profile_lists_session_settings_and_export() {
    let ctx = E2eContext::session();

    let settings_output = ctx.run(&["settings", "list", "-o", "json"]);
    assert_success(&settings_output, "settings list");
    let settings_payload = stdout_json(&settings_output);
    let settings = json_array(&settings_payload, "settings list");

    for key in ["dpi", "teleport", "magic_site_to_site_vpn"] {
        assert!(
            settings.iter().any(|setting| {
                setting
                    .get("key")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|value| value == key)
            }),
            "expected settings list to include '{key}', got {}",
            stdout_text(&settings_output)
        );
    }

    let export = ctx.run(&["settings", "export", "-o", "json"]);
    assert_success(&export, "settings export");
    let export_payload = stdout_json(&export);
    let exported = json_array(&export_payload, "settings export");
    assert!(
        exported.iter().any(|setting| {
            setting
                .get("key")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value == "dpi")
        }),
        "expected settings export to include dpi, got {}",
        stdout_text(&export)
    );
}

#[test]
fn session_profile_reads_dpi_setting() {
    let ctx = E2eContext::session();

    let output = ctx.run(&["settings", "get", "dpi", "-o", "json"]);
    assert_success(&output, "settings get dpi");

    let payload = stdout_json(&output);
    assert_eq!(
        payload
            .get("key")
            .and_then(serde_json::Value::as_str)
            .expect("dpi setting should include a key"),
        "dpi"
    );
    assert!(json_bool(&payload, "enabled"));
    assert!(json_bool(&payload, "fingerprintingEnabled"));
}

#[test]
fn session_profile_can_toggle_dpi_setting() {
    let ctx = E2eContext::session();

    let initial = ctx.run(&["settings", "get", "dpi", "-o", "json"]);
    assert_success(&initial, "settings get dpi before toggle");
    let initial_payload = stdout_json(&initial);
    let original_enabled = json_bool(&initial_payload, "enabled");
    let toggled_enabled = !original_enabled;
    let restore = RestoreCommand::new(
        &ctx,
        &[
            "settings",
            "set",
            "dpi",
            "enabled",
            bool_arg(original_enabled),
        ],
    );

    let update = ctx.run(&[
        "settings",
        "set",
        "dpi",
        "enabled",
        bool_arg(toggled_enabled),
    ]);
    assert_success(&update, "settings set dpi enabled");

    let after = ctx.run(&["settings", "get", "dpi", "-o", "json"]);
    assert_success(&after, "settings get dpi after toggle");
    let after_payload = stdout_json(&after);
    assert_eq!(json_bool(&after_payload, "enabled"), toggled_enabled);

    drop(restore);
}

#[test]
fn session_profile_lists_session_vpn_settings() {
    let ctx = E2eContext::session();

    let output = ctx.run(&["vpn", "settings", "list", "--all", "-o", "json"]);
    assert_success(&output, "vpn settings list");

    let payload = stdout_json(&output);
    let settings = json_array(&payload, "vpn settings list");

    for key in ["magic_site_to_site_vpn", "peer_to_peer", "teleport"] {
        assert!(
            settings.iter().any(|setting| {
                setting
                    .get("key")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|value| value == key)
            }),
            "expected vpn settings list to include '{key}', got {}",
            stdout_text(&output)
        );
    }
}

#[test]
fn session_profile_can_toggle_teleport_vpn_setting() {
    let ctx = E2eContext::session();

    let initial = ctx.run(&["vpn", "settings", "get", "teleport", "-o", "json"]);
    assert_success(&initial, "vpn settings get teleport before toggle");
    let initial_payload = stdout_json(&initial);
    let original_enabled = json_bool(&initial_payload, "enabled");
    let toggled_enabled = !original_enabled;
    let restore = RestoreCommand::new(
        &ctx,
        &[
            "vpn",
            "settings",
            "set",
            "teleport",
            "--enabled",
            bool_arg(original_enabled),
        ],
    );

    let update = ctx.run(&[
        "vpn",
        "settings",
        "set",
        "teleport",
        "--enabled",
        bool_arg(toggled_enabled),
    ]);
    assert_success(&update, "vpn settings set teleport");

    let after = ctx.run(&["vpn", "settings", "get", "teleport", "-o", "json"]);
    assert_success(&after, "vpn settings get teleport after toggle");
    let after_payload = stdout_json(&after);
    assert_eq!(json_bool(&after_payload, "enabled"), toggled_enabled);

    drop(restore);
}

#[test]
fn session_profile_lists_empty_nat_and_vpn_collections() {
    let ctx = E2eContext::session();

    for args in [
        ["nat", "policies", "list", "--all", "-o", "json"],
        ["vpn", "clients", "list", "--all", "-o", "json"],
        ["vpn", "connections", "list", "--all", "-o", "json"],
        ["vpn", "remote-access", "list", "--all", "-o", "json"],
        ["vpn", "site-to-site", "list", "--all", "-o", "json"],
    ] {
        let output = ctx.run(&args);
        let context = args.join(" ");
        assert_success(&output, &context);

        let payload = stdout_json(&output);
        let values = json_array(&payload, &context);
        assert!(
            values.is_empty(),
            "expected '{context}' to be empty in simulation mode, got {}",
            stdout_text(&output)
        );
    }
}

#[test]
fn session_profile_wrong_credentials_fail_with_auth_exit_code() {
    let ctx = E2eContext::session_with_password("not-the-right-password");
    let output = ctx.run(&["system", "sysinfo", "-o", "json"]);

    assert_eq!(
        output.status.code(),
        Some(3),
        "stderr:\n{}",
        stderr_text(&output)
    );
    assert!(
        stderr_text(&output).contains("Authentication failed"),
        "expected auth failure, got:\n{}",
        stderr_text(&output)
    );
}

#[test]
fn session_profile_integration_commands_surface_unsupported() {
    let ctx = E2eContext::session();
    let output = ctx.run(&["networks", "list", "--all", "-o", "json"]);

    assert_eq!(
        output.status.code(),
        Some(5),
        "stderr:\n{}",
        stderr_text(&output)
    );
    assert!(
        stderr_text(&output).contains("Integration API"),
        "expected integration guidance, got:\n{}",
        stderr_text(&output)
    );
}
