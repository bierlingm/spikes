//! Tests for the Spikes GitHub Action gate logic
//!
//! These tests verify the check.sh script behavior:
//! - Threshold comparison
//! - ignore-paths filtering
//! - require-resolution mode
//! - Edge cases (missing data, empty files)

mod common;

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper to create a test spike JSON
fn make_spike(id: &str, page: &str, rating: Option<&str>, resolved: bool) -> String {
    let rating_json = match rating {
        Some(r) => format!("\"{}\"", r),
        None => "null".to_string(),
    };
    let resolved_json = if resolved {
        r#""resolved": true, "resolvedAt": "2024-01-02T00:00:00Z""#
    } else {
        r#""resolved": false"#
    };

    format!(
        r#"{{"id":"{}","type":"page","projectKey":"test","page":"{}","url":"http://test/{}","reviewer":{{"id":"r1","name":"Test"}},"rating":{},"comments":"Test comment","timestamp":"2024-01-01T00:00:00Z",{}}}"#,
        id, page, page, rating_json, resolved_json
    )
}

/// Helper to set up a test project with spikes
fn setup_test_project() -> TempDir {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create .spikes directory
    fs::create_dir_all(temp_dir.path().join(".spikes")).unwrap();

    // Create config
    fs::write(
        temp_dir.path().join(".spikes/config.toml"),
        "project_key = \"test\"\n",
    )
    .unwrap();

    // Create empty feedback file
    fs::write(temp_dir.path().join(".spikes/feedback.jsonl"), "").unwrap();

    temp_dir
}

/// Helper to write spikes to feedback file
fn write_spikes(dir: &TempDir, spikes: &[String]) {
    let content = spikes.join("\n");
    fs::write(dir.path().join(".spikes/feedback.jsonl"), content).unwrap();
}

/// Get path to the check.sh script
fn check_script_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    // Navigate from cli/ to action/
    path.pop(); // Remove 'cli'
    path.push("action");
    path.push("check.sh");
    path
}

/// Get path to the spikes binary
fn spikes_binary_path() -> PathBuf {
    // Use cargo_bin_cmd to get the binary path
    cargo_bin_cmd!("spikes")
        .arg("--help")
        .assert()
        .success();

    // Get the path to the binary
    let output = cargo_bin_cmd!("spikes")
        .arg("version")
        .assert()
        .get_output()
        .to_owned();

    // The binary should be at target/debug/spikes
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove 'deps'
    path.push("spikes");

    if !path.exists() {
        // Try release build
        path.pop();
        path.push("release");
        path.push("spikes");
    }

    path
}

// ============================================================================
// Test: Action passes when no negative feedback exists
// ============================================================================

#[test]
fn test_gate_passes_with_positive_only() {
    let temp_dir = setup_test_project();

    // Add only positive spikes
    write_spikes(
        &temp_dir,
        &[
            make_spike("s1", "/index.html", Some("love"), false),
            make_spike("s2", "/about.html", Some("like"), false),
        ],
    );

    // Run check.sh
    let spikes_bin = spikes_binary_path();
    let output = Command::new(check_script_path())
        .args(["0", "", "false"])
        .env("SPIKES_BIN", &spikes_bin)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run check.sh");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should pass (exit 0)
    assert!(
        output.status.success(),
        "Should pass with positive only. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(stdout.contains("CLEAN SLATE") || stdout.contains("No blocking"));
}

// ============================================================================
// Test: Action fails when negative feedback exceeds threshold
// ============================================================================

#[test]
fn test_gate_fails_with_negative_spikes() {
    let temp_dir = setup_test_project();

    // Add negative spikes
    write_spikes(
        &temp_dir,
        &[
            make_spike("s1", "/index.html", Some("no"), false),
            make_spike("s2", "/about.html", Some("meh"), false),
        ],
    );

    // Run check.sh with threshold 0
    let spikes_bin = spikes_binary_path();
    let output = Command::new(check_script_path())
        .args(["0", "", "false"])
        .env("SPIKES_BIN", &spikes_bin)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run check.sh");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should fail (exit 1)
    assert!(
        !output.status.success(),
        "Should fail with negative spikes. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(stdout.contains("BLOCKING FEEDBACK") || stderr.contains("VIBES ARE OFF"));
}

// ============================================================================
// Test: Action passes when count is within threshold
// ============================================================================

#[test]
fn test_gate_passes_within_threshold() {
    let temp_dir = setup_test_project();

    // Add 2 negative spikes
    write_spikes(
        &temp_dir,
        &[
            make_spike("s1", "/index.html", Some("no"), false),
            make_spike("s2", "/about.html", Some("meh"), false),
        ],
    );

    // Run check.sh with threshold 2 (allow 2 blocking)
    let spikes_bin = spikes_binary_path();
    let output = Command::new(check_script_path())
        .args(["2", "", "false"])
        .env("SPIKES_BIN", &spikes_bin)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run check.sh");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should pass (exit 0) - 2 blocking <= threshold of 2
    assert!(
        output.status.success(),
        "Should pass within threshold. stdout: {}",
        stdout
    );
    assert!(stdout.contains("PASSED") || stdout.contains("acceptable"));
}

// ============================================================================
// Test: ignore-paths filters out matching pages
// ============================================================================

#[test]
fn test_gate_ignores_matching_paths() {
    let temp_dir = setup_test_project();

    // Add negative spikes on different pages
    write_spikes(
        &temp_dir,
        &[
            make_spike("s1", "/docs/index.html", Some("no"), false),
            make_spike("s2", "/about.html", Some("no"), false),
        ],
    );

    // Run check.sh with ignore-paths for /docs/**
    let spikes_bin = spikes_binary_path();
    let ignore_paths = "/docs/**";
    let output = Command::new(check_script_path())
        .args(["0", ignore_paths, "false"])
        .env("SPIKES_BIN", &spikes_bin)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run check.sh");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should fail - /about.html spike is not ignored
    assert!(
        !output.status.success(),
        "Should fail - /about.html not ignored. stdout: {}",
        stdout
    );
}

#[test]
fn test_gate_ignores_all_negative_with_wildcard() {
    let temp_dir = setup_test_project();

    // Add negative spikes on various pages
    write_spikes(
        &temp_dir,
        &[
            make_spike("s1", "/docs/index.html", Some("no"), false),
            make_spike("s2", "/docs/api.html", Some("meh"), false),
        ],
    );

    // Run check.sh with ignore-paths for /docs/**
    let spikes_bin = spikes_binary_path();
    let ignore_paths = "/docs/**";
    let output = Command::new(check_script_path())
        .args(["0", ignore_paths, "false"])
        .env("SPIKES_BIN", &spikes_bin)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run check.sh");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should pass - all spikes ignored
    assert!(
        output.status.success(),
        "Should pass - all spikes ignored. stdout: {}",
        stdout
    );
}

// ============================================================================
// Test: require-resolution mode fails on unresolved positive spikes
// ============================================================================

#[test]
fn test_require_resolution_fails_on_unresolved_positive() {
    let temp_dir = setup_test_project();

    // Add unresolved positive spikes (love, like)
    write_spikes(
        &temp_dir,
        &[
            make_spike("s1", "/index.html", Some("love"), false),
            make_spike("s2", "/about.html", Some("like"), false),
        ],
    );

    // Run check.sh with require-resolution=true
    let spikes_bin = spikes_binary_path();
    let output = Command::new(check_script_path())
        .args(["0", "", "true"])
        .env("SPIKES_BIN", &spikes_bin)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run check.sh");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should fail - unresolved spikes are blocking in require-resolution mode
    assert!(
        !output.status.success(),
        "Should fail with unresolved positive spikes. stdout: {}",
        stdout
    );
}

#[test]
fn test_require_resolution_passes_with_resolved() {
    let temp_dir = setup_test_project();

    // Add resolved spikes
    write_spikes(
        &temp_dir,
        &[
            make_spike("s1", "/index.html", Some("no"), true),
            make_spike("s2", "/about.html", Some("meh"), true),
        ],
    );

    // Run check.sh with require-resolution=true
    let spikes_bin = spikes_binary_path();
    let output = Command::new(check_script_path())
        .args(["0", "", "true"])
        .env("SPIKES_BIN", &spikes_bin)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run check.sh");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should pass - all spikes are resolved
    assert!(
        output.status.success(),
        "Should pass with all resolved. stdout: {}",
        stdout
    );
}

// ============================================================================
// Test: Missing .spikes/ directory passes with warning
// ============================================================================

#[test]
fn test_gate_passes_without_spikes_dir() {
    let temp_dir = tempfile::tempdir().unwrap();

    // No .spikes/ directory created

    // Run check.sh
    let spikes_bin = spikes_binary_path();
    let output = Command::new(check_script_path())
        .args(["0", "", "false"])
        .env("SPIKES_BIN", &spikes_bin)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run check.sh");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should pass (exit 0) with warning
    assert!(
        output.status.success(),
        "Should pass without .spikes/ dir. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("Clean slate") || stderr.contains("Clean slate") || stdout.contains("warning"),
        "Should show warning"
    );
}

// ============================================================================
// Test: Empty feedback file passes
// ============================================================================

#[test]
fn test_gate_passes_with_empty_feedback() {
    let temp_dir = setup_test_project();

    // Keep feedback.jsonl empty

    // Run check.sh
    let spikes_bin = spikes_binary_path();
    let output = Command::new(check_script_path())
        .args(["0", "", "false"])
        .env("SPIKES_BIN", &spikes_bin)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run check.sh");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should pass (exit 0) with warning
    assert!(
        output.status.success(),
        "Should pass with empty feedback. stdout: {}",
        stdout
    );
}

// ============================================================================
// Test: Resolved negative spikes are not blocking
// ============================================================================

#[test]
fn test_resolved_negative_not_blocking() {
    let temp_dir = setup_test_project();

    // Add resolved negative spikes
    write_spikes(
        &temp_dir,
        &[
            make_spike("s1", "/index.html", Some("no"), true),
            make_spike("s2", "/about.html", Some("meh"), true),
        ],
    );

    // Run check.sh with threshold 0
    let spikes_bin = spikes_binary_path();
    let output = Command::new(check_script_path())
        .args(["0", "", "false"])
        .env("SPIKES_BIN", &spikes_bin)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run check.sh");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should pass - resolved spikes are not blocking
    assert!(
        output.status.success(),
        "Should pass with resolved negative. stdout: {}",
        stdout
    );
}

// ============================================================================
// Test: Mixed resolved/unresolved only counts unresolved
// ============================================================================

#[test]
fn test_mixed_resolved_unresolved() {
    let temp_dir = setup_test_project();

    // Add mix of resolved and unresolved negative
    write_spikes(
        &temp_dir,
        &[
            make_spike("s1", "/index.html", Some("no"), false), // blocking
            make_spike("s2", "/about.html", Some("meh"), true), // resolved
            make_spike("s3", "/contact.html", Some("no"), false), // blocking
        ],
    );

    // Run check.sh with threshold 1
    let spikes_bin = spikes_binary_path();
    let output = Command::new(check_script_path())
        .args(["1", "", "false"])
        .env("SPIKES_BIN", &spikes_bin)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run check.sh");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should fail - 2 unresolved > threshold 1
    assert!(
        !output.status.success(),
        "Should fail with 2 unresolved. stdout: {}",
        stdout
    );
}

// ============================================================================
// Test: No rating is not blocking
// ============================================================================

#[test]
fn test_no_rating_not_blocking() {
    let temp_dir = setup_test_project();

    // Add spike with no rating
    write_spikes(&temp_dir, &[make_spike("s1", "/index.html", None, false)]);

    // Run check.sh with threshold 0
    let spikes_bin = spikes_binary_path();
    let output = Command::new(check_script_path())
        .args(["0", "", "false"])
        .env("SPIKES_BIN", &spikes_bin)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run check.sh");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should pass - no rating is not blocking
    assert!(
        output.status.success(),
        "Should pass with no rating. stdout: {}",
        stdout
    );
}
