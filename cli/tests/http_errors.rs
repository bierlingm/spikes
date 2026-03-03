//! Integration tests for HTTP error handling in CLI commands.
//!
//! Uses wiremock to simulate different HTTP error responses and verifies
//! that the CLI produces actionable error messages.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use wiremock::{matchers, Mock, MockServer, ResponseTemplate};
use std::fs;

fn setup_test_project() -> TempDir {
    let dir = TempDir::new().unwrap();
    let spikes_dir = dir.path().join(".spikes");
    fs::create_dir_all(&spikes_dir).unwrap();
    
    // Create config.toml with mock endpoint
    let config_content = r#"
[remote]
endpoint = "http://mock-endpoint.test"
token = "test-token"
"#;
    fs::write(spikes_dir.join("config.toml"), config_content).unwrap();
    
    dir
}

#[tokio::test]
async fn test_401_shows_auth_failed_message() {
    let mock_server = MockServer::start().await;
    let dir = setup_test_project();
    
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/spikes"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&mock_server)
        .await;
    
    let mut cmd = Command::cargo_bin("spikes").unwrap();
    cmd.current_dir(dir.path())
        .arg("pull")
        .arg("--endpoint")
        .arg(&format!("http://{}", mock_server.address()))
        .arg("--token")
        .arg("test-token");
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "Authentication failed. Run `spikes login` to refresh your token."
        ));
}

#[tokio::test]
async fn test_429_spike_limit_shows_upgrade_message() {
    let mock_server = MockServer::start().await;
    let dir = setup_test_project();
    
    // First, create a spike so push has something to push
    let spike_json = r#"{"id":"test-id","type":"page","projectKey":"test","page":"index","url":"http://test","reviewer":{"id":"r1","name":"Test"},"rating":"love","comments":"Test comment","timestamp":"2025-01-01T00:00:00Z","viewport":{"width":1920,"height":1080}}"#;
    fs::write(dir.path().join(".spikes/feedback.jsonl"), spike_json).unwrap();
    
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/spikes"))
        .respond_with(ResponseTemplate::new(200).set_body_json::<Vec<serde_json::Value>>(vec![]))
        .mount(&mock_server)
        .await;
    
    Mock::given(matchers::method("POST"))
        .and(matchers::path("/spikes"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(serde_json::json!({
                    "error": "Spike limit reached",
                    "code": "SPIKE_LIMIT"
                }))
        )
        .mount(&mock_server)
        .await;
    
    let mut cmd = Command::cargo_bin("spikes").unwrap();
    cmd.current_dir(dir.path())
        .arg("push")
        .arg("--endpoint")
        .arg(&format!("http://{}", mock_server.address()))
        .arg("--token")
        .arg("test-token");
    
    // Note: push returns success even when individual spikes fail (partial success design)
    // The error message appears in stderr
    cmd.assert()
        .stderr(predicate::str::contains(
            "Share has reached spike limit. Upgrade at https://spikes.sh/pro"
        ));
}

#[tokio::test]
async fn test_429_share_limit_shows_limit_message() {
    let mock_server = MockServer::start().await;
    let dir = setup_test_project();
    
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/spikes"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(serde_json::json!({
                    "error": "Share limit reached",
                    "code": "SHARE_LIMIT"
                }))
        )
        .mount(&mock_server)
        .await;
    
    let mut cmd = Command::cargo_bin("spikes").unwrap();
    cmd.current_dir(dir.path())
        .arg("pull")
        .arg("--endpoint")
        .arg(&format!("http://{}", mock_server.address()))
        .arg("--token")
        .arg("test-token");
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "You've reached the free tier limit (5 shares). Delete a share or upgrade"
        ));
}

#[tokio::test]
async fn test_413_shows_payload_too_large_message() {
    let mock_server = MockServer::start().await;
    let dir = setup_test_project();
    
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/spikes"))
        .respond_with(ResponseTemplate::new(413))
        .mount(&mock_server)
        .await;
    
    let mut cmd = Command::cargo_bin("spikes").unwrap();
    cmd.current_dir(dir.path())
        .arg("pull")
        .arg("--endpoint")
        .arg(&format!("http://{}", mock_server.address()))
        .arg("--token")
        .arg("test-token");
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "Files too large. Max size is 50MB. Consider removing large assets."
        ));
}

#[tokio::test]
async fn test_500_shows_server_error_message() {
    let mock_server = MockServer::start().await;
    let dir = setup_test_project();
    
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/spikes"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;
    
    let mut cmd = Command::cargo_bin("spikes").unwrap();
    cmd.current_dir(dir.path())
        .arg("pull")
        .arg("--endpoint")
        .arg(&format!("http://{}", mock_server.address()))
        .arg("--token")
        .arg("test-token");
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "Server error. Please try again in a moment or contact support if it persists."
        ));
}

#[tokio::test]
async fn test_502_shows_server_error_message() {
    let mock_server = MockServer::start().await;
    let dir = setup_test_project();
    
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/spikes"))
        .respond_with(ResponseTemplate::new(502))
        .mount(&mock_server)
        .await;
    
    let mut cmd = Command::cargo_bin("spikes").unwrap();
    cmd.current_dir(dir.path())
        .arg("pull")
        .arg("--endpoint")
        .arg(&format!("http://{}", mock_server.address()))
        .arg("--token")
        .arg("test-token");
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "Server error. Please try again in a moment or contact support"
        ));
}
