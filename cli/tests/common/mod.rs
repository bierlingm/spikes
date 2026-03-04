//! Common test utilities for spikes CLI tests
//!
//! This module provides helpers for:
//! - Creating temporary directories with test files
//! - Creating test spike data
//! - Mocking HTTP responses

#![allow(dead_code)]

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a temp directory with a .spikes/ structure
pub struct TestProject {
    pub dir: TempDir,
    pub spikes_dir: PathBuf,
    pub config_path: PathBuf,
    pub feedback_path: PathBuf,
}

impl TestProject {
    /// Create a new test project with initialized .spikes/ directory
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let spikes_dir = dir.path().join(".spikes");
        fs::create_dir_all(&spikes_dir).expect("Failed to create .spikes dir");

        let config_path = spikes_dir.join("config.toml");
        let feedback_path = spikes_dir.join("feedback.jsonl");

        // Create empty feedback file
        fs::File::create(&feedback_path).expect("Failed to create feedback.jsonl");

        Self {
            dir,
            spikes_dir,
            config_path,
            feedback_path,
        }
    }

    /// Create a test project with a default config
    pub fn with_config() -> Self {
        let project = Self::new();
        let config_content = concat!(
            "# Spikes configuration\n",
            "[project]\n",
            "key = \"test-project\"\n",
            "\n",
            "[widget]\n",
            "theme = \"dark\"\n",
            "position = \"bottom-right\"\n",
            "color = \"#e74c3c\"\n",
        );
        fs::write(&project.config_path, config_content).expect("Failed to write config");
        project
    }

    /// Add an HTML file to the project
    pub fn add_html_file(&self, name: &str, content: &str) -> PathBuf {
        let path = self.dir.path().join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&path, content).expect("Failed to write HTML file");
        path
    }

    /// Add a spike to the feedback file
    pub fn add_spike(&self, spike_json: &str) {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&self.feedback_path)
            .expect("Failed to open feedback file");
        writeln!(file, "{}", spike_json).expect("Failed to write spike");
    }

    /// Read all spikes from the feedback file
    pub fn read_spikes(&self) -> Vec<String> {
        let content = fs::read_to_string(&self.feedback_path).expect("Failed to read feedback");
        content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    /// Get the path to the project root
    pub fn path(&self) -> &std::path::Path {
        self.dir.path()
    }
}

impl Default for TestProject {
    fn default() -> Self {
        Self::new()
    }
}

/// Sample spike JSON for testing
pub fn sample_spike_json() -> &'static str {
    "{\"id\":\"abc123\",\"type\":\"page\",\"projectKey\":\"test\",\"page\":\"index.html\",\"url\":\"http://localhost:3847/index.html\",\"reviewer\":{\"id\":\"r1\",\"name\":\"Test Reviewer\"},\"rating\":\"like\",\"comments\":\"Looks good!\",\"timestamp\":\"2024-01-15T10:30:00Z\",\"viewport\":{\"width\":1920,\"height\":1080}}"
}

/// Sample element spike JSON for testing
pub fn sample_element_spike_json() -> &'static str {
    "{\"id\":\"def456\",\"type\":\"element\",\"projectKey\":\"test\",\"page\":\"index.html\",\"url\":\"http://localhost:3847/index.html\",\"reviewer\":{\"id\":\"r1\",\"name\":\"Test Reviewer\"},\"selector\":\".hero-title\",\"elementText\":\"Welcome\",\"boundingBox\":{\"x\":100,\"y\":50,\"width\":200,\"height\":40},\"rating\":\"love\",\"comments\":\"Love this!\",\"timestamp\":\"2024-01-15T10:31:00Z\",\"viewport\":{\"width\":1920,\"height\":1080}}"
}

/// Create a minimal valid HTML file
pub fn minimal_html() -> &'static str {
    "<!DOCTYPE html>\n<html>\n<head><title>Test</title></head>\n<body><h1>Hello World</h1></body>\n</html>"
}

/// Create HTML with an existing spikes script tag
pub fn html_with_widget() -> &'static str {
    "<!DOCTYPE html>\n<html>\n<head><title>Test</title></head>\n<body>\n<h1>Hello World</h1>\n<script src=\"https://spikes.sh/widget.js\" data-project=\"test\"></script>\n</body>\n</html>"
}
