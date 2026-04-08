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

#[test]
fn session_profile_smoke_test_reads_sysinfo() {
    let ctx = E2eContext::session();

    let output = ctx.run(&["system", "sysinfo", "-o", "json"]);

    assert!(
        output.status.success(),
        "expected system sysinfo to succeed, stderr:\n{}",
        stderr_text(&output)
    );

    let payload = stdout_json(&output);
    assert!(payload.is_object(), "expected JSON object, got {payload}");
}

#[test]
fn session_profile_lists_simulated_devices() {
    let ctx = E2eContext::session();
    let output = ctx.run(&["devices", "list", "--all", "-o", "json"]);

    assert!(
        output.status.success(),
        "expected devices list to succeed, stderr:\n{}",
        stderr_text(&output)
    );

    let payload = stdout_json(&output);
    let devices = payload
        .as_array()
        .expect("devices list should render a JSON array");

    assert!(
        !devices.is_empty(),
        "expected simulation mode to expose demo devices"
    );
}

#[test]
fn session_profile_lists_default_site() {
    let ctx = E2eContext::session();
    let output = ctx.run(&["sites", "list", "--all", "-o", "json"]);

    assert!(
        output.status.success(),
        "expected sites list to succeed, stderr:\n{}",
        stderr_text(&output)
    );

    let payload = stdout_json(&output);
    let sites = payload
        .as_array()
        .expect("sites list should render a JSON array");

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

    assert!(
        output.status.success(),
        "expected raw api sysinfo to succeed, stderr:\n{}",
        stderr_text(&output)
    );

    let payload = stdout_json(&output);
    assert!(payload.is_object(), "expected JSON object, got {payload}");
}

#[test]
fn session_profile_output_formats_render_cleanly() {
    let ctx = E2eContext::session();

    let table = ctx.run(&["sites", "list", "--all"]);
    assert!(
        table.status.success(),
        "table output failed:\n{}",
        stderr_text(&table)
    );
    assert!(stdout_text(&table).contains("default") || stdout_text(&table).contains("Default"));

    let yaml = ctx.run(&["sites", "list", "--all", "-o", "yaml"]);
    assert!(
        yaml.status.success(),
        "yaml output failed:\n{}",
        stderr_text(&yaml)
    );
    assert!(stdout_text(&yaml).contains("name:"));

    let plain = ctx.run(&["sites", "list", "--all", "-o", "plain"]);
    assert!(
        plain.status.success(),
        "plain output failed:\n{}",
        stderr_text(&plain)
    );
    assert!(!stdout_text(&plain).trim().is_empty());
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
