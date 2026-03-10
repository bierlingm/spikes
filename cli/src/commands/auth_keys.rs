//! Auth key management commands — create, list, and revoke API keys
//!
//! VAL-ID-014: spikes auth create-key creates API key and stores in auth.toml
//! VAL-ID-015: spikes auth list-keys shows table of keys
//! VAL-ID-016: spikes auth revoke-key <key_id> revokes and confirms

use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, ContentArrangement, Table};
use serde::{Deserialize, Serialize};

use crate::auth::{get_api_base, AuthConfig};
use crate::error::{map_http_error, map_network_error, Error, Result};

// ============================================
// Shared types
// ============================================

/// Response from POST /auth/api-key
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateKeyResponse {
    pub ok: bool,
    pub api_key: String,
    pub key_id: String,
    pub name: Option<String>,
    pub scopes: String,
    pub created_at: String,
}

/// Single key entry from GET /auth/api-keys
#[derive(Debug, Deserialize, Serialize)]
pub struct ApiKeyEntry {
    pub key_id: String,
    pub key_prefix: String,
    pub name: Option<String>,
    pub scopes: String,
    pub monthly_cap_cents: Option<i64>,
    pub revoked_at: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

impl ApiKeyEntry {
    /// Derive key status from revoked_at and expires_at fields.
    /// - If revoked_at is set → "revoked"
    /// - If expires_at is set and in the past → "expired"
    /// - Otherwise → "active"
    pub fn status(&self) -> &'static str {
        if self.revoked_at.is_some() {
            return "revoked";
        }
        if let Some(ref expires) = self.expires_at {
            // Parse ISO 8601 and compare to now
            if is_past(expires) {
                return "expired";
            }
        }
        "active"
    }
}

/// Check if an ISO 8601 timestamp is in the past.
fn is_past(iso: &str) -> bool {
    use chrono::Utc;
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso) {
        dt < Utc::now()
    } else {
        // Try common ISO variants (e.g. "2025-01-15T10:30:00.000Z")
        chrono::NaiveDateTime::parse_from_str(iso, "%Y-%m-%dT%H:%M:%S%.fZ")
            .map(|ndt| ndt.and_utc() < Utc::now())
            .unwrap_or(false)
    }
}

/// Response from DELETE /auth/api-key/:key_id
#[derive(Debug, Deserialize)]
pub struct RevokeKeyResponse {
    #[allow(dead_code)]
    pub ok: bool,
}

// ============================================
// create-key
// ============================================

pub fn create_key(name: Option<String>, json: bool) -> Result<()> {
    let api_base = get_api_base();
    let url = format!("{}/auth/api-key", api_base.trim_end_matches('/'));

    // Build request body
    let mut body = serde_json::Map::new();
    if let Some(ref n) = name {
        body.insert("name".to_string(), serde_json::Value::String(n.clone()));
    }

    let response = match ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_json(serde_json::Value::Object(body))
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().ok();
            return Err(map_http_error(status, body.as_deref()));
        }
        Err(e) => return Err(map_network_error(&e.to_string())),
    };

    let status = response.status();
    if status != 201 && status != 200 {
        let body = response.into_string().ok();
        return Err(map_http_error(status, body.as_deref()));
    }

    let body = response
        .into_string()
        .map_err(|e| Error::RequestFailed(format!("Failed to read response: {}", e)))?;

    let key_response: CreateKeyResponse = serde_json::from_str(&body)?;

    // Store the API key separately in auth.toml (does NOT overwrite bearer token)
    AuthConfig::save_api_key(&key_response.api_key)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&key_response)
                .expect("Failed to serialize to JSON")
        );
    } else {
        println!();
        println!("  ┌────────────────────────────────────────────┐");
        println!("  │  🔑 API key created                        │");
        println!("  │                                            │");
        println!("  │  Key:    {}  │", pad_right(&key_response.api_key, 30));
        println!("  │  ID:     {}  │", pad_right(&key_response.key_id, 30));
        if let Some(ref n) = key_response.name {
            println!("  │  Name:   {}  │", pad_right(n, 30));
        }
        println!("  │  Scopes: {}  │", pad_right(&key_response.scopes, 30));
        println!("  │                                            │");
        println!("  │  ⚠️  Save this key — it won't be shown again │");
        println!("  │  Stored in auth.toml for CLI use.          │");
        println!("  └────────────────────────────────────────────┘");
        println!();
    }

    Ok(())
}

// ============================================
// list-keys
// ============================================

pub fn list_keys(json: bool) -> Result<()> {
    // Prefer api_key for key management operations, fall back to bearer token
    let token = AuthConfig::load_api_key()
        .or_else(|| AuthConfig::token().ok().flatten())
        .ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Not logged in. Run 'spikes login' or 'spikes auth create-key' first.",
            ))
        })?;

    let keys = fetch_keys(&token)?;

    if json {
        // Add derived 'status' field to each key object
        let keys_with_status: Vec<serde_json::Value> = keys
            .iter()
            .map(|key| {
                let mut obj = serde_json::to_value(key).expect("Failed to serialize key");
                if let Some(map) = obj.as_object_mut() {
                    map.insert("status".to_string(), serde_json::Value::String(key.status().to_string()));
                }
                obj
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&keys_with_status)
                .expect("Failed to serialize to JSON")
        );
    } else {
        print_keys_table(&keys);
    }

    Ok(())
}

fn fetch_keys(token: &str) -> Result<Vec<ApiKeyEntry>> {
    let api_base = get_api_base();
    let url = format!("{}/auth/api-keys", api_base.trim_end_matches('/'));

    let response = match ureq::get(&url)
        .set("Authorization", &format!("Bearer {}", token))
        .call()
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().ok();
            return Err(map_http_error(status, body.as_deref()));
        }
        Err(e) => return Err(map_network_error(&e.to_string())),
    };

    let status = response.status();
    if status != 200 {
        let body = response.into_string().ok();
        return Err(map_http_error(status, body.as_deref()));
    }

    let body = response
        .into_string()
        .map_err(|e| Error::RequestFailed(format!("Failed to read response: {}", e)))?;

    let keys: Vec<ApiKeyEntry> = serde_json::from_str(&body)?;
    Ok(keys)
}

fn print_keys_table(keys: &[ApiKeyEntry]) {
    if keys.is_empty() {
        println!();
        println!("  No API keys found. Create one with 'spikes auth create-key'.");
        println!();
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Key Prefix", "Name", "Scopes", "Status", "Created"]);

    for key in keys {
        let name_display = key.name.as_deref().unwrap_or("—");
        let created_display = format_date(&key.created_at);
        let status = key.status();

        table.add_row(vec![
            Cell::new(format!("sk_spikes_{}…", key.key_prefix)),
            Cell::new(name_display),
            Cell::new(&key.scopes),
            Cell::new(status),
            Cell::new(&created_display),
        ]);
    }

    println!();
    println!("{table}");
    println!();
}

// ============================================
// revoke-key
// ============================================

pub fn revoke_key(key_id: &str, json: bool) -> Result<()> {
    // Prefer api_key for key management operations, fall back to bearer token
    let token = AuthConfig::load_api_key()
        .or_else(|| AuthConfig::token().ok().flatten())
        .ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Not logged in. Run 'spikes login' or 'spikes auth create-key' first.",
            ))
        })?;

    let api_base = get_api_base();
    let url = format!(
        "{}/auth/api-key/{}",
        api_base.trim_end_matches('/'),
        key_id
    );

    let response = match ureq::request("DELETE", &url)
        .set("Authorization", &format!("Bearer {}", token))
        .call()
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().ok();
            return Err(map_http_error(status, body.as_deref()));
        }
        Err(e) => return Err(map_network_error(&e.to_string())),
    };

    let status = response.status();
    if status != 200 {
        let body = response.into_string().ok();
        return Err(map_http_error(status, body.as_deref()));
    }

    // Read and verify response
    let body = response
        .into_string()
        .map_err(|e| Error::RequestFailed(format!("Failed to read response: {}", e)))?;

    let _revoke_response: RevokeKeyResponse = serde_json::from_str(&body)?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "key_id": key_id,
                "message": "API key revoked"
            })
        );
    } else {
        println!();
        println!("  🗡️  API key {} revoked. It can no longer be used.", key_id);
        println!();
    }

    Ok(())
}

// ============================================
// Helpers
// ============================================

fn pad_right(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s.len()))
    }
}

fn format_date(iso: &str) -> String {
    // Try to parse and format nicely, fall back to raw string
    if let Some(date_part) = iso.split('T').next() {
        date_part.to_string()
    } else {
        iso.to_string()
    }
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_key_response_deserialization() {
        let json = r#"{
            "ok": true,
            "api_key": "sk_spikes_abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "key_id": "key_abc123",
            "name": "test key",
            "scopes": "full",
            "created_at": "2025-01-15T10:30:00.000Z"
        }"#;

        let resp: CreateKeyResponse = serde_json::from_str(json).unwrap();
        assert!(resp.ok);
        assert!(resp.api_key.starts_with("sk_spikes_"));
        assert_eq!(resp.key_id, "key_abc123");
        assert_eq!(resp.name, Some("test key".to_string()));
        assert_eq!(resp.scopes, "full");
    }

    #[test]
    fn test_create_key_response_without_name() {
        let json = r#"{
            "ok": true,
            "api_key": "sk_spikes_abcdef1234567890",
            "key_id": "key_abc123",
            "name": null,
            "scopes": "full",
            "created_at": "2025-01-15T10:30:00.000Z"
        }"#;

        let resp: CreateKeyResponse = serde_json::from_str(json).unwrap();
        assert!(resp.ok);
        assert!(resp.name.is_none());
    }

    #[test]
    fn test_api_key_entry_deserialization() {
        let json = r#"{
            "key_id": "key_abc123",
            "key_prefix": "abcdef12",
            "name": "my agent",
            "scopes": "full",
            "monthly_cap_cents": 1000,
            "revoked_at": null,
            "expires_at": null,
            "created_at": "2025-01-15T10:30:00.000Z",
            "last_used_at": "2025-01-16T12:00:00.000Z"
        }"#;

        let entry: ApiKeyEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.key_id, "key_abc123");
        assert_eq!(entry.key_prefix, "abcdef12");
        assert_eq!(entry.name, Some("my agent".to_string()));
        assert_eq!(entry.scopes, "full");
        assert_eq!(entry.monthly_cap_cents, Some(1000));
        assert!(entry.revoked_at.is_none());
        assert!(entry.expires_at.is_none());
        assert!(entry.last_used_at.is_some());
    }

    #[test]
    fn test_api_key_entry_minimal() {
        let json = r#"{
            "key_id": "key_xyz789",
            "key_prefix": "xyz78900",
            "name": null,
            "scopes": "read",
            "monthly_cap_cents": null,
            "revoked_at": null,
            "expires_at": null,
            "created_at": "2025-01-15T10:30:00.000Z",
            "last_used_at": null
        }"#;

        let entry: ApiKeyEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.key_id, "key_xyz789");
        assert!(entry.name.is_none());
        assert!(entry.monthly_cap_cents.is_none());
        assert!(entry.revoked_at.is_none());
        assert!(entry.last_used_at.is_none());
    }

    #[test]
    fn test_api_key_entry_with_revoked_at() {
        let json = r#"{
            "key_id": "key_abc123",
            "key_prefix": "abcdef12",
            "name": "revoked key",
            "scopes": "full",
            "monthly_cap_cents": null,
            "revoked_at": "2025-02-01T00:00:00.000Z",
            "expires_at": null,
            "created_at": "2025-01-15T10:30:00.000Z",
            "last_used_at": null
        }"#;

        let entry: ApiKeyEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.revoked_at, Some("2025-02-01T00:00:00.000Z".to_string()));
    }

    #[test]
    fn test_revoke_key_response_deserialization() {
        let json = r#"{"ok": true}"#;
        let resp: RevokeKeyResponse = serde_json::from_str(json).unwrap();
        assert!(resp.ok);
    }

    #[test]
    fn test_format_date_iso() {
        assert_eq!(format_date("2025-01-15T10:30:00.000Z"), "2025-01-15");
    }

    #[test]
    fn test_format_date_no_time() {
        assert_eq!(format_date("2025-01-15"), "2025-01-15");
    }

    #[test]
    fn test_pad_right_shorter() {
        assert_eq!(pad_right("abc", 6), "abc   ");
    }

    #[test]
    fn test_pad_right_exact() {
        assert_eq!(pad_right("abc", 3), "abc");
    }

    #[test]
    fn test_pad_right_longer() {
        assert_eq!(pad_right("abcdef", 3), "abcdef");
    }

    #[test]
    fn test_print_keys_table_empty() {
        // Just ensure it doesn't panic
        print_keys_table(&[]);
    }

    #[test]
    fn test_print_keys_table_with_entries() {
        let keys = vec![
            ApiKeyEntry {
                key_id: "key_abc123".to_string(),
                key_prefix: "abcdef12".to_string(),
                name: Some("test key".to_string()),
                scopes: "full".to_string(),
                monthly_cap_cents: None,
                revoked_at: None,
                expires_at: None,
                created_at: "2025-01-15T10:30:00.000Z".to_string(),
                last_used_at: None,
            },
            ApiKeyEntry {
                key_id: "key_xyz789".to_string(),
                key_prefix: "xyz78900".to_string(),
                name: None,
                scopes: "read".to_string(),
                monthly_cap_cents: Some(500),
                revoked_at: None,
                expires_at: None,
                created_at: "2025-01-16T12:00:00.000Z".to_string(),
                last_used_at: Some("2025-01-17T08:00:00.000Z".to_string()),
            },
        ];
        // Just ensure it doesn't panic
        print_keys_table(&keys);
    }

    // ============================================
    // Status derivation tests
    // ============================================

    #[test]
    fn test_status_active() {
        let entry = ApiKeyEntry {
            key_id: "key_1".to_string(),
            key_prefix: "abcd1234".to_string(),
            name: None,
            scopes: "full".to_string(),
            monthly_cap_cents: None,
            revoked_at: None,
            expires_at: None,
            created_at: "2025-01-15T10:30:00.000Z".to_string(),
            last_used_at: None,
        };
        assert_eq!(entry.status(), "active");
    }

    #[test]
    fn test_status_revoked() {
        let entry = ApiKeyEntry {
            key_id: "key_2".to_string(),
            key_prefix: "abcd1234".to_string(),
            name: None,
            scopes: "full".to_string(),
            monthly_cap_cents: None,
            revoked_at: Some("2025-02-01T00:00:00.000Z".to_string()),
            expires_at: None,
            created_at: "2025-01-15T10:30:00.000Z".to_string(),
            last_used_at: None,
        };
        assert_eq!(entry.status(), "revoked");
    }

    #[test]
    fn test_status_expired() {
        let entry = ApiKeyEntry {
            key_id: "key_3".to_string(),
            key_prefix: "abcd1234".to_string(),
            name: None,
            scopes: "full".to_string(),
            monthly_cap_cents: None,
            revoked_at: None,
            expires_at: Some("2020-01-01T00:00:00.000Z".to_string()),
            created_at: "2019-01-15T10:30:00.000Z".to_string(),
            last_used_at: None,
        };
        assert_eq!(entry.status(), "expired");
    }

    #[test]
    fn test_status_future_expiry_is_active() {
        let entry = ApiKeyEntry {
            key_id: "key_4".to_string(),
            key_prefix: "abcd1234".to_string(),
            name: None,
            scopes: "full".to_string(),
            monthly_cap_cents: None,
            revoked_at: None,
            expires_at: Some("2099-12-31T23:59:59.000Z".to_string()),
            created_at: "2025-01-15T10:30:00.000Z".to_string(),
            last_used_at: None,
        };
        assert_eq!(entry.status(), "active");
    }

    #[test]
    fn test_status_revoked_takes_precedence_over_expired() {
        let entry = ApiKeyEntry {
            key_id: "key_5".to_string(),
            key_prefix: "abcd1234".to_string(),
            name: None,
            scopes: "full".to_string(),
            monthly_cap_cents: None,
            revoked_at: Some("2025-02-01T00:00:00.000Z".to_string()),
            expires_at: Some("2020-01-01T00:00:00.000Z".to_string()),
            created_at: "2019-01-15T10:30:00.000Z".to_string(),
            last_used_at: None,
        };
        assert_eq!(entry.status(), "revoked");
    }

    #[test]
    fn test_is_past_with_past_date() {
        assert!(is_past("2020-01-01T00:00:00.000Z"));
    }

    #[test]
    fn test_is_past_with_future_date() {
        assert!(!is_past("2099-12-31T23:59:59.000Z"));
    }

    #[test]
    fn test_is_past_with_invalid_date() {
        assert!(!is_past("not-a-date"));
    }

    #[test]
    fn test_create_key_response_serialization_roundtrip() {
        let resp = CreateKeyResponse {
            ok: true,
            api_key: "sk_spikes_test123".to_string(),
            key_id: "key_test".to_string(),
            name: Some("test".to_string()),
            scopes: "full".to_string(),
            created_at: "2025-01-15T10:30:00.000Z".to_string(),
        };

        let json_str = serde_json::to_string(&resp).unwrap();
        let deserialized: CreateKeyResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.api_key, resp.api_key);
        assert_eq!(deserialized.key_id, resp.key_id);
    }
}
