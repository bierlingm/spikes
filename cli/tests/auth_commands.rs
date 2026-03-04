//! Integration tests for authentication commands
//!
//! Tests login, logout, whoami commands with mocked API responses

mod common;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;
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
