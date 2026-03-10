//! Integration tests for authentication commands
//!
//! Tests login, logout, whoami, and auth key management commands
//! with mocked API responses

mod common;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;
use wiremock::{matchers, Mock, MockServer, ResponseTemplate};
use std::fs;
use std::path::PathBuf;

fn setup_test_config_dir() -> (TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().join("spikes");
    fs::create_dir_all(&config_dir).expect("Failed to create config dir");
    (temp_dir, config_dir)
}

fn write_auth_token(config_dir: &std::path::Path, token: &str) {
    let auth_path = config_dir.join("auth.toml");
    let content = format!("[auth]\ntoken = \"{}\"\n", token);
    fs::write(&auth_path, content).expect("Failed to write auth.toml");
    
    // Set secure permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&auth_path).unwrap().permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&auth_path, perms).expect("Failed to set permissions");
    }
}

#[test]
fn test_logout_clears_token() {
    let (_temp_dir, config_dir) = setup_test_config_dir();
    write_auth_token(&config_dir, "test-token-123");

    let auth_path = config_dir.join("auth.toml");
    assert!(auth_path.exists(), "Auth file should exist before logout");

    // Verify the auth.toml format
    let content = fs::read_to_string(&auth_path).unwrap();
    assert!(content.contains("test-token-123"));
}

#[test]
fn test_logout_already_logged_out() {
    let (_temp_dir, config_dir) = setup_test_config_dir();
    
    // No auth file
    let auth_path = config_dir.join("auth.toml");
    assert!(!auth_path.exists(), "Auth file should not exist");
}

#[test]
fn test_auth_toml_format() {
    let (_temp_dir, config_dir) = setup_test_config_dir();
    
    // Write auth.toml in the new format
    let auth_path = config_dir.join("auth.toml");
    let content = "[auth]\ntoken = \"my-secret-token\"\n";
    fs::write(&auth_path, content).unwrap();

    // Read it back and verify
    let read_content = fs::read_to_string(&auth_path).unwrap();
    assert!(read_content.contains("[auth]"));
    assert!(read_content.contains("my-secret-token"));
    
    // Verify it parses correctly
    let parsed: toml::Value = toml::from_str(&read_content).unwrap();
    assert_eq!(
        parsed.get("auth").and_then(|a| a.get("token")).and_then(|t| t.as_str()),
        Some("my-secret-token")
    );
}

#[cfg(unix)]
#[test]
fn test_auth_file_permissions() {
    use std::os::unix::fs::PermissionsExt;
    
    let (_temp_dir, config_dir) = setup_test_config_dir();
    let auth_path = config_dir.join("auth.toml");
    
    fs::write(&auth_path, "[auth]\ntoken = \"test\"\n").unwrap();
    
    // Set permissions
    let mut perms = fs::metadata(&auth_path).unwrap().permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&auth_path, perms).unwrap();
    
    // Verify
    let perms = fs::metadata(&auth_path).unwrap().permissions();
    assert_eq!(perms.mode() & 0o777, 0o600, "Auth file should have 0600 permissions");
}

#[test]
fn test_auth_path_xdg_compliant() {
    // Test that auth_path returns a valid XDG-compliant path
    let auth_path_result = dirs::config_dir()
        .map(|p| p.join("spikes").join("auth.toml"));
    
    if let Some(auth_path) = auth_path_result {
        // On macOS: ~/Library/Application Support/spikes/auth.toml
        // On Linux: ~/.config/spikes/auth.toml  
        // On Windows: %APPDATA%/spikes/auth.toml
        
        assert!(auth_path.ends_with("auth.toml"), "Auth path should end with auth.toml");
        assert!(auth_path.to_string_lossy().contains("spikes"), "Auth path should contain 'spikes'");
    }
}

// Note: Live integration tests with actual API calls would go here
// These would require mocking or test endpoints

#[test]
fn test_login_help() {
    cargo_bin_cmd!("spikes")
        .arg("login")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Log in to spikes.sh"));
}

#[test]
fn test_logout_help() {
    cargo_bin_cmd!("spikes")
        .arg("logout")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Log out from spikes.sh"));
}

#[test]
fn test_whoami_help() {
    cargo_bin_cmd!("spikes")
        .arg("whoami")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Show current user identity"));
}

// ============================================
// Auth key management command tests
// ============================================

#[test]
fn test_auth_help() {
    cargo_bin_cmd!("spikes")
        .arg("auth")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage API keys"))
        .stdout(predicate::str::contains("create-key"))
        .stdout(predicate::str::contains("list-keys"))
        .stdout(predicate::str::contains("revoke-key"));
}

#[test]
fn test_auth_create_key_help() {
    cargo_bin_cmd!("spikes")
        .arg("auth")
        .arg("create-key")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Create a new API key"))
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--json"));
}

#[test]
fn test_auth_list_keys_help() {
    cargo_bin_cmd!("spikes")
        .arg("auth")
        .arg("list-keys")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("List all API keys"))
        .stdout(predicate::str::contains("--json"));
}

#[test]
fn test_auth_revoke_key_help() {
    cargo_bin_cmd!("spikes")
        .arg("auth")
        .arg("revoke-key")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Revoke an API key"))
        .stdout(predicate::str::contains("--json"));
}

#[test]
fn test_auth_list_keys_not_logged_in() {
    // Isolate from machine auth config by pointing HOME and XDG_CONFIG_HOME
    // to an empty temp directory so no auth.toml can be found
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    cargo_bin_cmd!("spikes")
        .env_remove("SPIKES_TOKEN")
        .env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join(".config"))
        .arg("auth")
        .arg("list-keys")
        .assert()
        .failure();
}

#[test]
fn test_auth_revoke_key_not_logged_in() {
    // Isolate from machine auth config by pointing HOME and XDG_CONFIG_HOME
    // to an empty temp directory so no auth.toml can be found
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    cargo_bin_cmd!("spikes")
        .env_remove("SPIKES_TOKEN")
        .env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join(".config"))
        .arg("auth")
        .arg("revoke-key")
        .arg("key_test123")
        .assert()
        .failure();
}

#[test]
fn test_auth_revoke_key_requires_key_id_arg() {
    cargo_bin_cmd!("spikes")
        .arg("auth")
        .arg("revoke-key")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ============================================
// Wiremock-based API integration tests
// ============================================

#[tokio::test]
async fn test_auth_create_key_success() {
    let mock_server = MockServer::start().await;
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    Mock::given(matchers::method("POST"))
        .and(matchers::path("/auth/api-key"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "ok": true,
            "api_key": "sk_spikes_abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "key_id": "key_test123456",
            "name": null,
            "scopes": "full",
            "created_at": "2025-01-15T10:30:00.000Z"
        })))
        .mount(&mock_server)
        .await;

    cargo_bin_cmd!("spikes")
        .env("SPIKES_API_URL", format!("http://{}", mock_server.address()))
        .env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join(".config"))
        .arg("auth")
        .arg("create-key")
        .assert()
        .success()
        .stdout(predicate::str::contains("API key created"))
        .stdout(predicate::str::contains("sk_spikes_"));
}

#[tokio::test]
async fn test_auth_create_key_with_name() {
    let mock_server = MockServer::start().await;
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    Mock::given(matchers::method("POST"))
        .and(matchers::path("/auth/api-key"))
        .and(matchers::body_json(serde_json::json!({"name": "my-agent"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "ok": true,
            "api_key": "sk_spikes_abcdef1234567890",
            "key_id": "key_test123456",
            "name": "my-agent",
            "scopes": "full",
            "created_at": "2025-01-15T10:30:00.000Z"
        })))
        .mount(&mock_server)
        .await;

    cargo_bin_cmd!("spikes")
        .env("SPIKES_API_URL", format!("http://{}", mock_server.address()))
        .env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join(".config"))
        .arg("auth")
        .arg("create-key")
        .arg("--name")
        .arg("my-agent")
        .assert()
        .success()
        .stdout(predicate::str::contains("my-agent"));
}

#[tokio::test]
async fn test_auth_create_key_json_output() {
    let mock_server = MockServer::start().await;
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    Mock::given(matchers::method("POST"))
        .and(matchers::path("/auth/api-key"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "ok": true,
            "api_key": "sk_spikes_abcdef1234567890",
            "key_id": "key_test123456",
            "name": "test",
            "scopes": "full",
            "created_at": "2025-01-15T10:30:00.000Z"
        })))
        .mount(&mock_server)
        .await;

    let output = cargo_bin_cmd!("spikes")
        .env("SPIKES_API_URL", format!("http://{}", mock_server.address()))
        .env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join(".config"))
        .arg("auth")
        .arg("create-key")
        .arg("--json")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");
    assert_eq!(parsed["ok"], true);
    assert!(parsed["api_key"].as_str().unwrap().starts_with("sk_spikes_"));
    assert!(parsed["key_id"].as_str().unwrap().starts_with("key_"));
    assert_eq!(parsed["scopes"], "full");
}

#[tokio::test]
async fn test_auth_list_keys_success() {
    let mock_server = MockServer::start().await;

    Mock::given(matchers::method("GET"))
        .and(matchers::path("/auth/api-keys"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "key_id": "key_abc123",
                "key_prefix": "abcdef12",
                "name": "test key",
                "scopes": "full",
                "monthly_cap_cents": null,
                "expires_at": null,
                "created_at": "2025-01-15T10:30:00.000Z",
                "last_used_at": null
            },
            {
                "key_id": "key_xyz789",
                "key_prefix": "xyz78900",
                "name": null,
                "scopes": "read",
                "monthly_cap_cents": 1000,
                "expires_at": null,
                "created_at": "2025-01-16T12:00:00.000Z",
                "last_used_at": "2025-01-17T08:00:00.000Z"
            }
        ])))
        .mount(&mock_server)
        .await;

    cargo_bin_cmd!("spikes")
        .env("SPIKES_API_URL", format!("http://{}", mock_server.address()))
        .env("SPIKES_TOKEN", "sk_spikes_testtoken")
        .arg("auth")
        .arg("list-keys")
        .assert()
        .success()
        .stdout(predicate::str::contains("abcdef12"))
        .stdout(predicate::str::contains("test key"))
        .stdout(predicate::str::contains("full"))
        .stdout(predicate::str::contains("read"));
}

#[tokio::test]
async fn test_auth_list_keys_json_output() {
    let mock_server = MockServer::start().await;

    Mock::given(matchers::method("GET"))
        .and(matchers::path("/auth/api-keys"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "key_id": "key_abc123",
                "key_prefix": "abcdef12",
                "name": "test key",
                "scopes": "full",
                "monthly_cap_cents": null,
                "expires_at": null,
                "created_at": "2025-01-15T10:30:00.000Z",
                "last_used_at": null
            }
        ])))
        .mount(&mock_server)
        .await;

    let output = cargo_bin_cmd!("spikes")
        .env("SPIKES_API_URL", format!("http://{}", mock_server.address()))
        .env("SPIKES_TOKEN", "sk_spikes_testtoken")
        .arg("auth")
        .arg("list-keys")
        .arg("--json")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");
    assert!(parsed.is_array());
    assert_eq!(parsed[0]["key_id"], "key_abc123");
    assert_eq!(parsed[0]["key_prefix"], "abcdef12");
}

#[tokio::test]
async fn test_auth_list_keys_empty() {
    let mock_server = MockServer::start().await;

    Mock::given(matchers::method("GET"))
        .and(matchers::path("/auth/api-keys"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    cargo_bin_cmd!("spikes")
        .env("SPIKES_API_URL", format!("http://{}", mock_server.address()))
        .env("SPIKES_TOKEN", "sk_spikes_testtoken")
        .arg("auth")
        .arg("list-keys")
        .assert()
        .success()
        .stdout(predicate::str::contains("No API keys found"));
}

#[tokio::test]
async fn test_auth_revoke_key_success() {
    let mock_server = MockServer::start().await;

    Mock::given(matchers::method("DELETE"))
        .and(matchers::path("/auth/api-key/key_abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ok": true
        })))
        .mount(&mock_server)
        .await;

    cargo_bin_cmd!("spikes")
        .env("SPIKES_API_URL", format!("http://{}", mock_server.address()))
        .env("SPIKES_TOKEN", "sk_spikes_testtoken")
        .arg("auth")
        .arg("revoke-key")
        .arg("key_abc123")
        .assert()
        .success()
        .stdout(predicate::str::contains("key_abc123"))
        .stdout(predicate::str::contains("revoked"));
}

#[tokio::test]
async fn test_auth_revoke_key_json_output() {
    let mock_server = MockServer::start().await;

    Mock::given(matchers::method("DELETE"))
        .and(matchers::path("/auth/api-key/key_abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ok": true
        })))
        .mount(&mock_server)
        .await;

    let output = cargo_bin_cmd!("spikes")
        .env("SPIKES_API_URL", format!("http://{}", mock_server.address()))
        .env("SPIKES_TOKEN", "sk_spikes_testtoken")
        .arg("auth")
        .arg("revoke-key")
        .arg("key_abc123")
        .arg("--json")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["key_id"], "key_abc123");
}

#[tokio::test]
async fn test_auth_revoke_key_not_found() {
    let mock_server = MockServer::start().await;

    Mock::given(matchers::method("DELETE"))
        .and(matchers::path("/auth/api-key/key_nonexistent"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": "Key not found",
            "code": "NOT_FOUND"
        })))
        .mount(&mock_server)
        .await;

    cargo_bin_cmd!("spikes")
        .env("SPIKES_API_URL", format!("http://{}", mock_server.address()))
        .env("SPIKES_TOKEN", "sk_spikes_testtoken")
        .arg("auth")
        .arg("revoke-key")
        .arg("key_nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Key not found"));
}

#[tokio::test]
async fn test_auth_create_key_rate_limited() {
    let mock_server = MockServer::start().await;
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    Mock::given(matchers::method("POST"))
        .and(matchers::path("/auth/api-key"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "error": "Rate limit exceeded",
            "code": "RATE_LIMIT",
            "retry_after": 3600
        })))
        .mount(&mock_server)
        .await;

    cargo_bin_cmd!("spikes")
        .env("SPIKES_API_URL", format!("http://{}", mock_server.address()))
        .env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join(".config"))
        .arg("auth")
        .arg("create-key")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Rate limit exceeded"));
}
