//! Integration tests for spikes CLI workflows
//!
//! Tests the full command workflows:
//! - init -> inject -> serve
//! - spike storage and retrieval

mod common;

use assert_cmd::cargo::cargo_bin_cmd;
use common::{TestProject, minimal_html, html_with_widget, sample_spike_json};
use predicates::prelude::*;

#[test]
fn test_init_creates_directory() {
    let temp_dir = tempfile::tempdir().unwrap();

    cargo_bin_cmd!("spikes")
        .current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized .spikes/ directory"));

    // Verify files were created
    assert!(temp_dir.path().join(".spikes/config.toml").exists());
    assert!(temp_dir.path().join(".spikes/feedback.jsonl").exists());
}

#[test]
fn test_init_json_output() {
    let temp_dir = tempfile::tempdir().unwrap();

    cargo_bin_cmd!("spikes")
        .current_dir(temp_dir.path())
        .arg("init")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"success\":true"))
        .stdout(predicate::str::contains(".spikes/config.toml"));
}

#[test]
fn test_init_existing_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(temp_dir.path().join(".spikes")).unwrap();

    cargo_bin_cmd!("spikes")
        .current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_init_creates_gitignore_if_missing() {
    let temp_dir = tempfile::tempdir().unwrap();

    cargo_bin_cmd!("spikes")
        .current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success();

    // Verify .gitignore was created with .spikes/ entry
    let gitignore_path = temp_dir.path().join(".gitignore");
    assert!(gitignore_path.exists(), ".gitignore should be created");

    let content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(content.contains(".spikes/"), ".gitignore should contain .spikes/");
}

#[test]
fn test_init_appends_to_existing_gitignore() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create existing .gitignore with content
    let gitignore_path = temp_dir.path().join(".gitignore");
    std::fs::write(&gitignore_path, "target/\n*.log\n").unwrap();

    cargo_bin_cmd!("spikes")
        .current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success();

    // Verify existing content preserved and .spikes/ appended
    let content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(content.contains("target/"), "existing content should be preserved");
    assert!(content.contains("*.log"), "existing content should be preserved");
    assert!(content.contains(".spikes/"), ".spikes/ should be appended");
}

#[test]
fn test_init_does_not_duplicate_spikes_in_gitignore() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create .gitignore that already contains .spikes/
    let gitignore_path = temp_dir.path().join(".gitignore");
    std::fs::write(&gitignore_path, "target/\n.spikes/\n*.log\n").unwrap();

    cargo_bin_cmd!("spikes")
        .current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success();

    // Verify .spikes/ only appears once
    let content = std::fs::read_to_string(&gitignore_path).unwrap();
    let count = content.matches(".spikes/").count();
    assert_eq!(count, 1, ".spikes/ should only appear once in .gitignore");
}

#[test]
fn test_init_json_output_includes_gitignore() {
    let temp_dir = tempfile::tempdir().unwrap();

    cargo_bin_cmd!("spikes")
        .current_dir(temp_dir.path())
        .arg("init")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains(".gitignore"));
}

#[test]
fn test_inject_basic() {
    let temp_dir = tempfile::tempdir().unwrap();
    let html_path = temp_dir.path().join("index.html");
    std::fs::write(&html_path, minimal_html()).unwrap();

    cargo_bin_cmd!("spikes")
        .current_dir(temp_dir.path())
        .arg("inject")
        .arg(".")
        .assert()
        .success()
        .stdout(predicate::str::contains("Injected widget"));

    // Verify widget was injected
    let content = std::fs::read_to_string(&html_path).unwrap();
    assert!(content.contains("spikes.sh/widget.js"));
}

#[test]
fn test_inject_json_output() {
    let temp_dir = tempfile::tempdir().unwrap();
    let html_path = temp_dir.path().join("index.html");
    std::fs::write(&html_path, minimal_html()).unwrap();

    cargo_bin_cmd!("spikes")
        .current_dir(temp_dir.path())
        .arg("inject")
        .arg(".")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"action\": \"inject\""))
        .stdout(predicate::str::contains("\"injected\":"));
}

#[test]
fn test_inject_custom_widget_url() {
    let temp_dir = tempfile::tempdir().unwrap();
    let html_path = temp_dir.path().join("index.html");
    std::fs::write(&html_path, minimal_html()).unwrap();

    cargo_bin_cmd!("spikes")
        .current_dir(temp_dir.path())
        .arg("inject")
        .arg(".")
        .arg("--widget-url")
        .arg("/custom/widget.js")
        .assert()
        .success();

    let content = std::fs::read_to_string(&html_path).unwrap();
    assert!(content.contains("/custom/widget.js"));
}

#[test]
fn test_inject_skips_existing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let html_path = temp_dir.path().join("index.html");
    std::fs::write(&html_path, html_with_widget()).unwrap();

    cargo_bin_cmd!("spikes")
        .current_dir(temp_dir.path())
        .arg("inject")
        .arg(".")
        .assert()
        .success()
        .stdout(predicate::str::contains("Skipped"));

    // Should still only have one widget reference
    let content = std::fs::read_to_string(&html_path).unwrap();
    assert_eq!(content.matches("spikes").count(), 1);
}

#[test]
fn test_inject_remove() {
    let temp_dir = tempfile::tempdir().unwrap();
    let html_path = temp_dir.path().join("index.html");
    std::fs::write(&html_path, html_with_widget()).unwrap();

    cargo_bin_cmd!("spikes")
        .current_dir(temp_dir.path())
        .arg("inject")
        .arg(".")
        .arg("--remove")
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed widget"));

    let content = std::fs::read_to_string(&html_path).unwrap();
    assert!(!content.contains("spikes.sh/widget.js"));
}

#[test]
fn test_inject_nonexistent_directory() {
    cargo_bin_cmd!("spikes")
        .arg("inject")
        .arg("/nonexistent/path")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Directory not found"));
}

#[test]
fn test_list_empty() {
    let project = TestProject::new();

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No spikes found"));
}

#[test]
fn test_list_with_spikes() {
    let project = TestProject::new();
    project.add_spike(sample_spike_json());

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("abc123"));
}

#[test]
fn test_list_json_output() {
    let project = TestProject::new();
    project.add_spike(sample_spike_json());

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\": \"abc123\""));
}

#[test]
fn test_list_filter_by_rating() {
    let project = TestProject::new();
    project.add_spike(sample_spike_json());
    project.add_spike("{\"id\":\"def456\",\"type\":\"page\",\"projectKey\":\"test\",\"page\":\"index.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r1\",\"name\":\"Test\"},\"rating\":\"no\",\"comments\":\"Bad\",\"timestamp\":\"2024-01-01T00:00:00Z\"}");

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("list")
        .arg("--rating")
        .arg("like")
        .assert()
        .success()
        .stdout(predicate::str::contains("abc123"))
        .stdout(predicate::str::contains("def456").not());
}

#[test]
fn test_show_spike() {
    let project = TestProject::new();
    project.add_spike(sample_spike_json());

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("show")
        .arg("abc123")
        .assert()
        .success()
        .stdout(predicate::str::contains("abc123"));
}

#[test]
fn test_show_spike_json() {
    let project = TestProject::new();
    project.add_spike(sample_spike_json());

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("show")
        .arg("abc123")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\": \"abc123\""));
}

#[test]
fn test_show_nonexistent_spike() {
    let project = TestProject::new();

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("show")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_export_default_json() {
    let project = TestProject::new();
    project.add_spike(sample_spike_json());

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\": \"abc123\""));
}

#[test]
fn test_export_jsonl_format() {
    let project = TestProject::new();
    project.add_spike(sample_spike_json());

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("export")
        .arg("--format")
        .arg("jsonl")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\":\"abc123\""));
}

#[test]
fn test_hotspots_empty() {
    let project = TestProject::new();

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("hotspots")
        .assert()
        .success()
        .stdout(predicate::str::contains("No element spikes found"));
}

#[test]
fn test_hotspots_with_element_spikes() {
    let project = TestProject::new();
    project.add_spike("{\"id\":\"elem1\",\"type\":\"element\",\"projectKey\":\"test\",\"page\":\"index.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r1\",\"name\":\"Test\"},\"selector\":\".hero\",\"rating\":\"love\",\"comments\":\"Great\",\"timestamp\":\"2024-01-01T00:00:00Z\"}");
    project.add_spike("{\"id\":\"elem2\",\"type\":\"element\",\"projectKey\":\"test\",\"page\":\"index.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r2\",\"name\":\"Test2\"},\"selector\":\".hero\",\"rating\":\"like\",\"comments\":\"Nice\",\"timestamp\":\"2024-01-01T00:01:00Z\"}");
    project.add_spike("{\"id\":\"elem3\",\"type\":\"element\",\"projectKey\":\"test\",\"page\":\"index.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r3\",\"name\":\"Test3\"},\"selector\":\".footer\",\"rating\":\"meh\",\"comments\":\"OK\",\"timestamp\":\"2024-01-01T00:02:00Z\"}");

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("hotspots")
        .assert()
        .success()
        .stdout(predicate::str::contains(".hero"));
}

#[test]
fn test_reviewers() {
    let project = TestProject::new();
    project.add_spike("{\"id\":\"s1\",\"type\":\"page\",\"projectKey\":\"test\",\"page\":\"index.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r1\",\"name\":\"Alice\"},\"rating\":\"like\",\"comments\":\"Good\",\"timestamp\":\"2024-01-01T00:00:00Z\"}");
    project.add_spike("{\"id\":\"s2\",\"type\":\"page\",\"projectKey\":\"test\",\"page\":\"index.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r2\",\"name\":\"Bob\"},\"rating\":\"love\",\"comments\":\"Great\",\"timestamp\":\"2024-01-01T00:01:00Z\"}");

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("reviewers")
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob"));
}

#[test]
fn test_config_show() {
    let project = TestProject::with_config();

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("config")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-project"));
}

#[test]
fn test_version() {
    cargo_bin_cmd!("spikes")
        .arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("spikes"));
}

// ============================================================================
// Delete command tests
// ============================================================================

#[test]
fn test_delete_spike_with_force() {
    let project = TestProject::new();
    project.add_spike("{\"id\":\"delete-test-123\",\"type\":\"page\",\"projectKey\":\"test\",\"page\":\"index.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r1\",\"name\":\"Test\"},\"rating\":\"like\",\"comments\":\"Test\",\"timestamp\":\"2024-01-01T00:00:00Z\"}");

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("delete")
        .arg("delete-test-123")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted spike"));

    // Verify spike was removed
    let spikes = project.read_spikes();
    assert!(spikes.is_empty(), "Spike should be deleted");
}

#[test]
fn test_delete_spike_json_output() {
    let project = TestProject::new();
    project.add_spike("{\"id\":\"delete-json-test\",\"type\":\"page\",\"projectKey\":\"test\",\"page\":\"index.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r1\",\"name\":\"Test\"},\"rating\":\"like\",\"comments\":\"Test\",\"timestamp\":\"2024-01-01T00:00:00Z\"}");

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("delete")
        .arg("delete-json-test")
        .arg("--force")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"deleted\": true"))
        .stdout(predicate::str::contains("\"id\": \"delete-json-test\""));
}

#[test]
fn test_delete_nonexistent_spike() {
    let project = TestProject::new();

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("delete")
        .arg("nonexistent-id-xyz")
        .arg("--force")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Spike not found"));
}

#[test]
fn test_delete_prefix_too_short() {
    let project = TestProject::new();
    project.add_spike("{\"id\":\"prefix-test-123\",\"type\":\"page\",\"projectKey\":\"test\",\"page\":\"index.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r1\",\"name\":\"Test\"},\"rating\":\"like\",\"comments\":\"Test\",\"timestamp\":\"2024-01-01T00:00:00Z\"}");

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("delete")
        .arg("abc")  // Only 3 characters
        .arg("--force")
        .assert()
        .failure()
        .stderr(predicate::str::contains("at least 4 characters"));
}

#[test]
fn test_delete_by_prefix() {
    let project = TestProject::new();
    project.add_spike("{\"id\":\"unique-prefix-test\",\"type\":\"page\",\"projectKey\":\"test\",\"page\":\"index.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r1\",\"name\":\"Test\"},\"rating\":\"like\",\"comments\":\"Test\",\"timestamp\":\"2024-01-01T00:00:00Z\"}");

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("delete")
        .arg("uniq")  // 5-char unique prefix
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted spike"));

    let spikes = project.read_spikes();
    assert!(spikes.is_empty(), "Spike should be deleted");
}

// ============================================================================
// Resolve command tests
// ============================================================================

#[test]
fn test_resolve_spike() {
    let project = TestProject::new();
    project.add_spike("{\"id\":\"resolve-test-123\",\"type\":\"page\",\"projectKey\":\"test\",\"page\":\"index.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r1\",\"name\":\"Test\"},\"rating\":\"like\",\"comments\":\"Test\",\"timestamp\":\"2024-01-01T00:00:00Z\"}");

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("resolve")
        .arg("resolve-test-123")
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved spike"));

    // Verify spike has resolved field
    let spikes = project.read_spikes();
    assert!(spikes[0].contains("\"resolved\":true"), "Spike should be resolved");
    assert!(spikes[0].contains("\"resolvedAt\""), "Spike should have resolvedAt timestamp");
}

#[test]
fn test_resolve_spike_json_output() {
    let project = TestProject::new();
    project.add_spike("{\"id\":\"resolve-json-test\",\"type\":\"page\",\"projectKey\":\"test\",\"page\":\"index.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r1\",\"name\":\"Test\"},\"rating\":\"like\",\"comments\":\"Test\",\"timestamp\":\"2024-01-01T00:00:00Z\"}");

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("resolve")
        .arg("resolve-json-test")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"resolved\": true"))
        .stdout(predicate::str::contains("\"resolvedAt\""));
}

#[test]
fn test_unresolve_spike() {
    let project = TestProject::new();
    project.add_spike("{\"id\":\"unresolve-test\",\"type\":\"page\",\"projectKey\":\"test\",\"page\":\"index.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r1\",\"name\":\"Test\"},\"rating\":\"like\",\"comments\":\"Test\",\"timestamp\":\"2024-01-01T00:00:00Z\",\"resolved\":true,\"resolvedAt\":\"2024-01-02T00:00:00Z\"}");

    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("resolve")
        .arg("unresolve-test")
        .arg("--unresolve")
        .assert()
        .success()
        .stdout(predicate::str::contains("Unresolved spike"));

    // Verify spike no longer has resolved field
    let spikes = project.read_spikes();
    assert!(!spikes[0].contains("\"resolved\":true"), "Spike should not be resolved");
}

#[test]
fn test_list_unresolved_filter() {
    let project = TestProject::new();
    project.add_spike("{\"id\":\"resolved-spike\",\"type\":\"page\",\"projectKey\":\"test\",\"page\":\"index.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r1\",\"name\":\"Test\"},\"rating\":\"like\",\"comments\":\"Resolved\",\"timestamp\":\"2024-01-01T00:00:00Z\",\"resolved\":true,\"resolvedAt\":\"2024-01-02T00:00:00Z\"}");
    project.add_spike("{\"id\":\"unresolved-spike\",\"type\":\"page\",\"projectKey\":\"test\",\"page\":\"about.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r2\",\"name\":\"Test2\"},\"rating\":\"love\",\"comments\":\"Unresolved\",\"timestamp\":\"2024-01-01T00:01:00Z\"}");

    // Should only show unresolved spike
    let output = cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("list")
        .arg("--unresolved")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .to_owned();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Note: "unresolved-spike" contains "resolved-spike" as a substring, so we need to check for the full ID with quotes
    assert!(stdout.contains("\"id\": \"unresolved-spike\""), "Should contain unresolved-spike ID");
    assert!(!stdout.contains("\"id\": \"resolved-spike\""), "Should not contain resolved-spike ID");
}

#[test]
fn test_list_unresolved_composes_with_other_filters() {
    let project = TestProject::new();
    project.add_spike("{\"id\":\"spike-page-a\",\"type\":\"page\",\"projectKey\":\"test\",\"page\":\"page-a.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r1\",\"name\":\"Alice\"},\"rating\":\"like\",\"comments\":\"Test\",\"timestamp\":\"2024-01-01T00:00:00Z\"}");
    project.add_spike("{\"id\":\"spike-page-b\",\"type\":\"page\",\"projectKey\":\"test\",\"page\":\"page-b.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r2\",\"name\":\"Bob\"},\"rating\":\"love\",\"comments\":\"Test\",\"timestamp\":\"2024-01-01T00:01:00Z\",\"resolved\":true,\"resolvedAt\":\"2024-01-02T00:00:00Z\"}");
    project.add_spike("{\"id\":\"spike-page-c\",\"type\":\"page\",\"projectKey\":\"test\",\"page\":\"page-a.html\",\"url\":\"http://localhost\",\"reviewer\":{\"id\":\"r3\",\"name\":\"Charlie\"},\"rating\":\"meh\",\"comments\":\"Test\",\"timestamp\":\"2024-01-01T00:02:00Z\",\"resolved\":true,\"resolvedAt\":\"2024-01-02T00:00:00Z\"}");

    // --page filter + --unresolved should only show unresolved spikes from page-a
    cargo_bin_cmd!("spikes")
        .current_dir(project.path())
        .arg("list")
        .arg("--page")
        .arg("page-a")
        .arg("--unresolved")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("spike-page-a"))
        .stdout(predicate::str::contains("spike-page-c").not());  // Resolved, should be filtered out
}

// ============================================================================
// Init hosted-by-default tests
// ============================================================================

/// Helper to run spikes binary in a temp directory with null stdin (non-interactive mode)
fn run_spikes_init(temp_dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    let binary = cargo_bin_cmd!("spikes").get_program().to_owned();
    let mut cmd = std::process::Command::new(binary);
    cmd.current_dir(temp_dir)
        .arg("init")
        .stdin(std::process::Stdio::null());  // Non-interactive
    for arg in args {
        cmd.arg(arg);
    }
    cmd.output().expect("Failed to run spikes init")
}

#[test]
fn test_init_non_interactive_defaults_to_hosted() {
    // Non-interactive (stdin closed) defaults to hosted
    let temp_dir = tempfile::tempdir().unwrap();

    let output = run_spikes_init(temp_dir.path(), &[]);
    assert!(output.status.success(), "init should succeed");

    // Verify config contains [remote] section with hosted=true and endpoint
    let config_path = temp_dir.path().join(".spikes/config.toml");
    let content = std::fs::read_to_string(&config_path).unwrap();
    
    // VAL-CONFIG-005: Must contain [remote], hosted=true, endpoint="https://spikes.sh"
    assert!(content.contains("[remote]"), "Config must contain [remote] section");
    assert!(content.contains("hosted = true"), "Config must have hosted = true");
    assert!(content.contains("endpoint = \"https://spikes.sh\""), "Config must have endpoint = \"https://spikes.sh\"");
}

#[test]
fn test_init_json_non_interactive_defaults_to_hosted() {
    // VAL-CONFIG-006a: --json with closed stdin defaults to hosted
    let temp_dir = tempfile::tempdir().unwrap();

    let output = run_spikes_init(temp_dir.path(), &["--json"]);
    assert!(output.status.success(), "init --json should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // JSON output should indicate success
    assert!(stdout.contains("\"success\":true"), "JSON output should indicate success");

    // Verify config contains hosted=true and endpoint
    let config_path = temp_dir.path().join(".spikes/config.toml");
    let content = std::fs::read_to_string(&config_path).unwrap();
    
    assert!(content.contains("[remote]"), "Config must contain [remote] section");
    assert!(content.contains("hosted = true"), "Config must have hosted = true");
    assert!(content.contains("endpoint = \"https://spikes.sh\""), "Config must have endpoint");
}

#[test]
fn test_init_creates_empty_feedback_jsonl() {
    // VAL-CONFIG-008: feedback.jsonl should be empty (0 bytes)
    let temp_dir = tempfile::tempdir().unwrap();

    let output = run_spikes_init(temp_dir.path(), &[]);
    assert!(output.status.success(), "init should succeed");

    let feedback_path = temp_dir.path().join(".spikes/feedback.jsonl");
    assert!(feedback_path.exists(), "feedback.jsonl should exist");
    
    let metadata = std::fs::metadata(&feedback_path).unwrap();
    assert_eq!(metadata.len(), 0, "feedback.jsonl should be exactly 0 bytes");
}

#[test]
fn test_init_config_uses_remote_not_sync() {
    // VAL-CONFIG-007: Config should use [remote], not [sync]
    let temp_dir = tempfile::tempdir().unwrap();

    let output = run_spikes_init(temp_dir.path(), &[]);
    assert!(output.status.success(), "init should succeed");

    let config_path = temp_dir.path().join(".spikes/config.toml");
    let content = std::fs::read_to_string(&config_path).unwrap();
    
    assert!(content.contains("[remote]"), "Config must contain [remote] section");
    assert!(!content.contains("[sync]"), "Config must NOT contain [sync] section");
}

#[test]
fn test_init_is_idempotent() {
    // VAL-CONFIG-013: Running init twice should not overwrite existing .spikes/
    let temp_dir = tempfile::tempdir().unwrap();

    // First init
    let output1 = run_spikes_init(temp_dir.path(), &[]);
    assert!(output1.status.success(), "First init should succeed");

    // Get the config content after first init
    let config_path = temp_dir.path().join(".spikes/config.toml");
    let first_content = std::fs::read_to_string(&config_path).unwrap();

    // Second init should fail gracefully
    let output2 = run_spikes_init(temp_dir.path(), &[]);
    // Note: status should still be success (exit 0), but with error message to stderr
    let stderr = String::from_utf8_lossy(&output2.stderr);
    assert!(stderr.contains("already exists"), "Should print 'already exists' message");

    // Verify config was not overwritten
    let second_content = std::fs::read_to_string(&config_path).unwrap();
    assert_eq!(first_content, second_content, "Config should not be overwritten");
}

#[test]
fn test_init_config_roundtrip() {
    // VAL-CONFIG-014: Config should survive serde roundtrip
    let temp_dir = tempfile::tempdir().unwrap();

    // Init creates config
    let output = run_spikes_init(temp_dir.path(), &[]);
    assert!(output.status.success(), "init should succeed");

    // Now use spikes config to read it back
    let binary = cargo_bin_cmd!("spikes").get_program().to_owned();
    let config_output = std::process::Command::new(binary)
        .current_dir(temp_dir.path())
        .arg("config")
        .arg("--json")
        .output()
        .expect("Failed to run spikes config");
    
    assert!(config_output.status.success(), "config command should succeed");

    let stdout = String::from_utf8_lossy(&config_output.stdout);
    
    // Verify JSON output has expected fields
    assert!(stdout.contains("\"hosted\":true"), "JSON should show hosted=true");
    assert!(stdout.contains("\"endpoint\":\"https://spikes.sh\""), "JSON should show the endpoint");
}

#[test]
fn test_init_non_interactive_self_host_flag() {
    // Non-interactive with --self-host flag should create self-host config
    let temp_dir = tempfile::tempdir().unwrap();

    let output = run_spikes_init(temp_dir.path(), &["--self-host"]);
    assert!(output.status.success(), "init --self-host should succeed");

    // Verify config has [remote] but NOT hosted=true
    let config_path = temp_dir.path().join(".spikes/config.toml");
    let content = std::fs::read_to_string(&config_path).unwrap();
    
    assert!(content.contains("[remote]"), "Config must contain [remote] section");
    // For self-host, hosted should not be true (either false, absent, or commented)
    assert!(!content.contains("hosted = true"), "Self-host config should NOT have hosted = true");
}
