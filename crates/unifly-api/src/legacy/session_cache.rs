// Persistent session cache for legacy API auth.
//
// Stores session cookies and CSRF tokens on disk so subsequent CLI
// invocations can skip the login handshake (especially valuable when
// MFA/TOTP is enabled). Cache files live under `$XDG_CACHE_HOME/unifly/`
// and are keyed by `{profile_name}_{host_hash}.json`.
//
// Security: files are created with 0600 permissions on Unix.
// Expiry: parsed from the JWT `exp` claim with a 60-second safety margin.
// Validation: a lightweight probe to `/api/s/{site}/self` confirms the
// session is still alive before trusting the cache.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Cached session data persisted to disk.
#[derive(Debug, Serialize, Deserialize)]
struct CachedSession {
    /// Session cookie string (e.g. `TOKEN=abc...`).
    cookie: String,
    /// CSRF token for UniFi OS.
    csrf_token: Option<String>,
    /// Unix timestamp when the session expires.
    expires_at: u64,
}

/// Handle for reading/writing a session cache file.
pub struct SessionCache {
    path: PathBuf,
}

impl SessionCache {
    /// Create a new cache handle for the given profile and controller URL.
    ///
    /// Returns `None` if the cache directory can't be determined.
    pub fn new(profile_name: &str, controller_url: &str) -> Option<Self> {
        let cache_dir = cache_dir()?;
        // Hash the URL to avoid path-unsafe characters
        let url_hash = simple_hash(controller_url);
        let filename = format!("{profile_name}_{url_hash}.json");
        Some(Self {
            path: cache_dir.join(filename),
        })
    }

    /// Load a cached session if it exists and hasn't expired.
    ///
    /// Returns `(cookie_header, csrf_token)` on success.
    pub fn load(&self) -> Option<(String, Option<String>)> {
        let data = fs::read_to_string(&self.path).ok()?;
        let session: CachedSession = serde_json::from_str(&data).ok()?;

        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();

        if now >= session.expires_at {
            debug!("cached session expired, removing");
            self.clear();
            return None;
        }

        debug!(
            expires_in_secs = session.expires_at.saturating_sub(now),
            "loaded cached session"
        );
        Some((session.cookie, session.csrf_token))
    }

    /// Save a session to disk with atomic write.
    pub fn save(&self, cookie: &str, csrf_token: Option<&str>, expires_at: u64) {
        let session = CachedSession {
            cookie: cookie.to_owned(),
            csrf_token: csrf_token.map(str::to_owned),
            expires_at,
        };

        let Ok(json) = serde_json::to_string_pretty(&session) else {
            warn!("failed to serialize session cache");
            return;
        };

        if let Err(e) = atomic_write(&self.path, json.as_bytes()) {
            warn!(error = %e, "failed to write session cache");
        } else {
            debug!("session cached to {}", self.path.display());
        }
    }

    /// Remove the cache file.
    pub fn clear(&self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Extract the `exp` claim from a JWT token string.
///
/// Parses the payload (second segment) of the JWT to read the expiry.
/// Returns `None` if the token is malformed or missing `exp`.
pub fn jwt_expiry(token: &str) -> Option<u64> {
    // JWT cookies are "TOKEN=eyJ...", extract just the JWT value
    let jwt = token.split(';').next()?.split('=').nth(1)?;

    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let payload = URL_SAFE_NO_PAD.decode(parts[1]).ok()?;
    let claims: serde_json::Value = serde_json::from_slice(&payload).ok()?;
    claims["exp"].as_u64()
}

/// Default fallback expiry: 2 hours from now.
pub fn fallback_expiry() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() + 2 * 3600)
        .unwrap_or(0)
}

/// Safety margin subtracted from JWT expiry (60 seconds).
pub const EXPIRY_MARGIN_SECS: u64 = 60;

// ── Internals ──────────────────────────────────────────────────────

/// Resolve the cache directory: `$XDG_CACHE_HOME/unifly/` or `~/.cache/unifly/`.
fn cache_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "unifly").map(|dirs| dirs.cache_dir().to_owned())
}

/// Simple non-cryptographic hash for URL → filename mapping.
fn simple_hash(s: &str) -> String {
    let mut hash: u64 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(u64::from(byte));
    }
    format!("{hash:016x}")
}

/// Atomic write: write to a temp file in the same directory, then rename.
fn atomic_write(path: &Path, data: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp_path = path.with_extension("tmp");
    let mut file = fs::File::create(&tmp_path)?;
    file.write_all(data)?;
    file.flush()?;

    // Restrict to owner-only access
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
    }

    drop(file);

    // On Windows, rename fails if the target exists — remove it first.
    #[cfg(windows)]
    let _ = fs::remove_file(path);

    fs::rename(&tmp_path, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jwt_expiry_parses_valid_token() {
        // Build a minimal JWT: header.payload.signature
        let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"HS256"}"#);
        let payload = URL_SAFE_NO_PAD.encode(r#"{"exp":1700000000}"#);
        let token = format!("TOKEN={header}.{payload}.sig");
        assert_eq!(jwt_expiry(&token), Some(1_700_000_000));
    }

    #[test]
    fn jwt_expiry_returns_none_for_garbage() {
        assert_eq!(jwt_expiry("not-a-jwt"), None);
        assert_eq!(jwt_expiry("TOKEN=a.b"), None);
    }

    #[test]
    fn simple_hash_is_deterministic() {
        let a = simple_hash("https://192.168.1.1");
        let b = simple_hash("https://192.168.1.1");
        assert_eq!(a, b);
        assert_ne!(a, simple_hash("https://10.0.0.1"));
    }

    #[test]
    fn session_cache_round_trips() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let cache = SessionCache {
            path: dir.path().join("test.json"),
        };

        cache.save("TOKEN=abc", Some("csrf123"), fallback_expiry());
        let loaded = cache.load().expect("cache should load");
        assert_eq!(loaded.0, "TOKEN=abc");
        assert_eq!(loaded.1.as_deref(), Some("csrf123"));
    }

    #[test]
    fn expired_session_returns_none() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let cache = SessionCache {
            path: dir.path().join("expired.json"),
        };

        cache.save("TOKEN=old", None, 0); // expired at epoch
        assert!(cache.load().is_none());
    }
}
