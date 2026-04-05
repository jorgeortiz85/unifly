#![allow(clippy::unwrap_used)]
// Integration tests for `LegacyClient` using wiremock.

use serde_json::json;
use url::Url;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use unifly_api::{ControllerPlatform, Error, LegacyClient, TransportConfig};

// ── Helpers ─────────────────────────────────────────────────────────

async fn setup() -> (MockServer, LegacyClient) {
    let server = MockServer::start().await;
    let base_url = Url::parse(&server.uri()).unwrap();
    let client = LegacyClient::with_client(
        reqwest::Client::new(),
        base_url,
        "default".into(),
        ControllerPlatform::ClassicController,
    );
    (server, client)
}

/// Setup with a cookie jar — required for MFA tests (cookie injection).
async fn setup_with_jar() -> (MockServer, LegacyClient) {
    let server = MockServer::start().await;
    let base_url = Url::parse(&server.uri()).unwrap();
    let transport = TransportConfig::default().with_cookie_jar();
    let client = LegacyClient::new(
        base_url,
        "default".into(),
        ControllerPlatform::ClassicController,
        &transport,
    )
    .unwrap();
    (server, client)
}

fn site_path(suffix: &str) -> String {
    format!("/api/s/default/{suffix}")
}

// ── Authentication tests ────────────────────────────────────────────

#[tokio::test]
async fn test_login_success() {
    let (server, client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/api/login"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&server)
        .await;

    let secret: secrecy::SecretString = "test-password".to_string().into();
    client.login("admin", &secret, None).await.unwrap();
}

#[tokio::test]
async fn test_login_failure() {
    let (server, client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/api/login"))
        .respond_with(ResponseTemplate::new(403).set_body_string("Forbidden"))
        .mount(&server)
        .await;

    let secret: secrecy::SecretString = "wrong-password".to_string().into();
    let result = client.login("admin", &secret, None).await;

    assert!(
        matches!(result, Err(Error::Authentication { .. })),
        "expected Authentication error, got: {result:?}"
    );
}

// ── Device tests ────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_devices() {
    let (server, client) = setup().await;

    let envelope = json!({
        "meta": { "rc": "ok" },
        "data": [{
            "_id": "abc123",
            "mac": "aa:bb:cc:dd:ee:ff",
            "type": "usw",
            "name": "Switch-24",
            "model": "US24",
            "adopted": true,
            "state": 1
        }]
    });

    Mock::given(method("GET"))
        .and(path(site_path("stat/device")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&envelope))
        .mount(&server)
        .await;

    let devices = client.list_devices().await.unwrap();

    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].mac, "aa:bb:cc:dd:ee:ff");
    assert_eq!(devices[0].name.as_deref(), Some("Switch-24"));
    assert_eq!(devices[0].device_type, "usw");
    assert!(devices[0].adopted);
    assert_eq!(devices[0].state, 1);
}

// ── Event tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_events() {
    let (server, client) = setup().await;

    let envelope = json!({
        "meta": { "rc": "ok" },
        "data": [
            {
                "_id": "evt001",
                "key": "EVT_WU_Connected",
                "msg": "User connected",
                "datetime": "2024-06-15T10:30:00Z",
                "subsystem": "wlan"
            },
            {
                "_id": "evt002",
                "key": "EVT_LU_Disconnected",
                "msg": "User disconnected",
                "datetime": "2024-06-15T10:35:00Z",
                "subsystem": "lan"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path(site_path("stat/event")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&envelope))
        .mount(&server)
        .await;

    let events = client.list_events(None).await.unwrap();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].id, "evt001");
    assert_eq!(events[0].key.as_deref(), Some("EVT_WU_Connected"));
    assert_eq!(events[1].subsystem.as_deref(), Some("lan"));
}

#[tokio::test]
async fn test_list_events_with_limit() {
    let (server, client) = setup().await;

    let envelope = json!({
        "meta": { "rc": "ok" },
        "data": [{
            "_id": "evt001",
            "key": "EVT_WU_Connected"
        }]
    });

    Mock::given(method("GET"))
        .and(path(site_path("stat/event")))
        .and(query_param("_limit", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&envelope))
        .mount(&server)
        .await;

    let events = client.list_events(Some(5)).await.unwrap();

    assert_eq!(events.len(), 1);
}

// ── Error tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_unauthorized_without_cookie_jar_reports_invalid_api_key() {
    let (server, client) = setup().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let result = client.list_devices().await;

    assert!(
        matches!(result, Err(Error::InvalidApiKey)),
        "expected InvalidApiKey, got: {result:?}"
    );
}

#[tokio::test]
async fn test_unauthorized_with_cookie_jar_reports_session_expired() {
    let (server, client) = setup_with_jar().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let result = client.list_devices().await;

    assert!(
        matches!(result, Err(Error::SessionExpired)),
        "expected SessionExpired, got: {result:?}"
    );
}

#[tokio::test]
async fn test_legacy_api_error() {
    let (server, client) = setup().await;

    let envelope = json!({
        "meta": { "rc": "error", "msg": "api.err.InvalidObject" },
        "data": []
    });

    Mock::given(method("GET"))
        .and(path(site_path("stat/device")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&envelope))
        .mount(&server)
        .await;

    let result = client.list_devices().await;

    match result {
        Err(Error::LegacyApi { ref message }) => {
            assert!(
                message.contains("InvalidObject"),
                "expected 'InvalidObject' in message, got: {message}"
            );
        }
        other => panic!("expected LegacyApi error, got: {other:?}"),
    }
}

// ── MFA/TOTP tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_mfa_challenge_without_totp_returns_two_factor_required() {
    let (server, client) = setup_with_jar().await;

    Mock::given(method("POST"))
        .and(path("/api/login"))
        .respond_with(ResponseTemplate::new(499).set_body_json(json!({
            "errors": ["ubic_2fa_token required"],
            "token": "UBIC_2FA=eyJhbGciOiJIUzI1NiJ9"
        })))
        .mount(&server)
        .await;

    let secret: secrecy::SecretString = "password".to_string().into();
    let result = client.login("admin", &secret, None).await;

    assert!(
        matches!(result, Err(Error::TwoFactorRequired)),
        "expected TwoFactorRequired, got: {result:?}"
    );
}

#[tokio::test]
async fn test_mfa_login_with_valid_totp_succeeds() {
    let (server, client) = setup_with_jar().await;

    // Use body matching to distinguish the two login attempts:
    // - First POST has no ubic_2fa_token → 499 challenge
    // - Second POST has ubic_2fa_token → 200 success
    //
    // wiremock matches most-recently-mounted first, so mount
    // the success (TOTP) mock first, then the challenge mock.
    Mock::given(method("POST"))
        .and(path("/api/login"))
        .and(wiremock::matchers::body_string_contains("ubic_2fa_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .named("mfa-complete")
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/login"))
        .respond_with(ResponseTemplate::new(499).set_body_json(json!({
            "errors": ["ubic_2fa_token required"],
            "token": "UBIC_2FA=eyJhbGciOiJIUzI1NiJ9"
        })))
        .named("mfa-challenge")
        .mount(&server)
        .await;

    let secret: secrecy::SecretString = "password".to_string().into();
    let totp: secrecy::SecretString = "123456".to_string().into();
    client.login("admin", &secret, Some(&totp)).await.unwrap();
}

#[tokio::test]
async fn test_mfa_rejects_invalid_totp_format() {
    let (server, client) = setup_with_jar().await;

    Mock::given(method("POST"))
        .and(path("/api/login"))
        .respond_with(ResponseTemplate::new(499).set_body_json(json!({
            "errors": ["ubic_2fa_token required"],
            "token": "UBIC_2FA=eyJhbGciOiJIUzI1NiJ9"
        })))
        .mount(&server)
        .await;

    let secret: secrecy::SecretString = "password".to_string().into();
    let bad_totp: secrecy::SecretString = "not6digits".to_string().into();
    let result = client.login("admin", &secret, Some(&bad_totp)).await;

    assert!(
        matches!(result, Err(Error::Authentication { .. })),
        "expected Authentication error for bad TOTP, got: {result:?}"
    );
}
