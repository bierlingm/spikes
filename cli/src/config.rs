use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub project: ProjectConfig,
    #[serde(default)]
    pub widget: WidgetConfig,
    #[serde(default)]
    pub remote: RemoteConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    /// Project key for grouping spikes
    pub key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    /// Widget theme: "dark" or "light"
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Collect email from reviewers
    #[serde(default)]
    pub collect_email: bool,
    /// Button position: "bottom-right", "bottom-left", "top-right", "top-left"
    #[serde(default = "default_position")]
    pub position: String,
    /// Accent color (hex)
    #[serde(default = "default_color")]
    pub color: String,
}

impl Default for WidgetConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            collect_email: false,
            position: default_position(),
            color: default_color(),
        }
    }
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_position() -> String {
    "bottom-right".to_string()
}

fn default_color() -> String {
    "#e74c3c".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteConfig {
    /// Remote endpoint URL
    pub endpoint: Option<String>,
    /// Auth token for remote
    pub token: Option<String>,
    /// Use hosted spikes.sh backend
    #[serde(default)]
    pub hosted: bool,
}

impl Config {
    /// Load config from .spikes/config.toml, or return defaults
    pub fn load() -> Result<Self> {
        Self::load_from(Path::new(".spikes/config.toml"))
    }

    /// Load config from a specific path
    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content).map_err(|e| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid config.toml: {}", e),
            ))
        })?;

        Ok(config)
    }

    /// Save config to .spikes/config.toml
    pub fn save(&self) -> Result<()> {
        self.save_to(Path::new(".spikes/config.toml"))
    }

    /// Save config to a specific path
    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self).map_err(|e| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to serialize config: {}", e),
            ))
        })?;

        fs::write(path, content)?;
        Ok(())
    }

    /// Get effective endpoint (remote.endpoint or hosted fallback)
    ///
    /// Priority:
    /// 1. If remote.endpoint is explicitly set, use it (explicit wins over hosted)
    /// 2. If remote.hosted is true, use the canonical hosted URL https://spikes.sh
    /// 3. Otherwise, return None
    pub fn effective_endpoint(&self) -> Option<String> {
        // Explicit endpoint takes precedence over hosted flag
        if let Some(ref endpoint) = self.remote.endpoint {
            return Some(endpoint.clone());
        }

        if self.remote.hosted {
            Some("https://spikes.sh".to_string())
        } else {
            None
        }
    }

    /// Get effective project key (from config or current directory name)
    pub fn effective_project_key(&self) -> String {
        self.project.key.clone().unwrap_or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
                .unwrap_or_else(|| "default".to_string())
        })
    }

    /// Generate widget script tag attributes from config
    pub fn widget_attributes(&self) -> String {
        let mut attrs = vec![
            format!("data-project=\"{}\"", self.effective_project_key()),
            format!("data-theme=\"{}\"", self.widget.theme),
            format!("data-position=\"{}\"", self.widget.position),
            format!("data-color=\"{}\"", self.widget.color),
        ];

        if self.widget.collect_email {
            attrs.push("data-collect-email=\"true\"".to_string());
        }

        if let Some(endpoint) = self.effective_endpoint() {
            // Normalize endpoint: strip trailing "/spikes" if present to avoid double paths
            // This handles legacy configs where users may have manually edited endpoint to include /spikes
            let base_endpoint = endpoint
                .trim_end_matches('/')
                .trim_end_matches("/spikes");

            let full_endpoint = if let Some(token) = &self.remote.token {
                format!("{}/spikes?token={}", base_endpoint, token)
            } else {
                format!("{}/spikes", base_endpoint)
            };
            attrs.push(format!("data-endpoint=\"{}\"", full_endpoint));
        }

        attrs.join(" ")
    }
}

/// Ensure .spikes directory exists, creating with defaults if needed
pub fn ensure_initialized() -> Result<bool> {
    let spikes_dir = Path::new(".spikes");
    
    if spikes_dir.exists() {
        return Ok(false); // Already existed
    }

    fs::create_dir_all(spikes_dir)?;
    
    let config = Config::default();
    config.save()?;
    
    fs::write(spikes_dir.join("feedback.jsonl"), "")?;
    
    Ok(true) // Newly created
}

#[allow(dead_code)]
pub const DEFAULT_CONFIG_TEMPLATE: &str = "# Spikes configuration
# https://spikes.sh

[project]
# Project key for grouping spikes (default: directory name)
# key = \"my-project\"

[widget]
# Widget appearance
theme = \"dark\"           # \"dark\" or \"light\"
position = \"bottom-right\" # \"bottom-right\", \"bottom-left\", \"top-right\", \"top-left\"
color = \"#e74c3c\"        # Accent color (hex)
collect_email = false    # Ask reviewers for email (builds prospect list)

[remote]
# Cloud sync configuration
# endpoint = \"https://your-worker.workers.dev\"
# token = \"your-token-here\"
# hosted = false  # Use spikes.sh managed backend instead of self-hosted
";

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert!(config.project.key.is_none());
        assert_eq!(config.widget.theme, "dark");
        assert_eq!(config.widget.position, "bottom-right");
        assert_eq!(config.widget.color, "#e74c3c");
        assert!(!config.widget.collect_email);
        assert!(config.remote.endpoint.is_none());
        assert!(config.remote.token.is_none());
        assert!(!config.remote.hosted);
    }

    #[test]
    fn test_load_missing_config() {
        let temp_dir = TempDir::new().unwrap();
        let missing_path = temp_dir.path().join("nonexistent/config.toml");

        let config = Config::load_from(&missing_path).unwrap();

        // Should return defaults for missing file
        assert!(config.project.key.is_none());
    }

    #[test]
    fn test_load_valid_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let content = concat!(
            "[project]\n",
            "key = \"my-awesome-project\"\n",
            "\n",
            "[widget]\n",
            "theme = \"light\"\n",
            "position = \"top-left\"\n",
            "color = \"#3498db\"\n",
            "collect_email = true\n",
            "\n",
            "[remote]\n",
            "endpoint = \"https://api.example.com\"\n",
            "token = \"secret-token\"\n",
            "hosted = true\n",
        );
        std::fs::write(&config_path, content).unwrap();

        let config = Config::load_from(&config_path).unwrap();

        assert_eq!(config.project.key, Some("my-awesome-project".to_string()));
        assert_eq!(config.widget.theme, "light");
        assert_eq!(config.widget.position, "top-left");
        assert_eq!(config.widget.color, "#3498db");
        assert!(config.widget.collect_email);
        assert_eq!(config.remote.endpoint, Some("https://api.example.com".to_string()));
        assert_eq!(config.remote.token, Some("secret-token".to_string()));
        assert!(config.remote.hosted);
    }

    #[test]
    fn test_load_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        std::fs::write(&config_path, "this is not valid toml [[[[").unwrap();

        let result = Config::load_from(&config_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut config = Config::default();
        config.project.key = Some("saved-project".to_string());
        config.widget.theme = "light".to_string();

        config.save_to(&config_path).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("saved-project"));
        assert!(content.contains("light"));
    }

    #[test]
    fn test_save_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nested/dir/config.toml");

        let config = Config::default();
        config.save_to(&config_path).unwrap();

        assert!(config_path.exists());
    }

    #[test]
    fn test_effective_endpoint_hosted() {
        // DEPRECATED: This test was for the old api.spikes.sh endpoint
        // Replaced by test_effective_endpoint_hosted_returns_canonical_url
        // Keeping for backward compatibility check - now expects canonical endpoint
        let mut config = Config::default();
        config.remote.hosted = true;

        assert_eq!(config.effective_endpoint(), Some("https://spikes.sh".to_string()));
    }

    #[test]
    fn test_effective_endpoint_custom() {
        let mut config = Config::default();
        config.remote.endpoint = Some("https://custom.example.com".to_string());

        assert_eq!(config.effective_endpoint(), Some("https://custom.example.com".to_string()));
    }

    #[test]
    fn test_effective_endpoint_none() {
        let config = Config::default();
        assert!(config.remote.endpoint.is_none());
        assert!(!config.remote.hosted);
        // effective_endpoint returns None when neither hosted nor custom endpoint
        assert!(config.effective_endpoint().is_none());
    }

    #[test]
    fn test_effective_project_key_from_config() {
        let mut config = Config::default();
        config.project.key = Some("configured-key".to_string());

        assert_eq!(config.effective_project_key(), "configured-key");
    }

    #[test]
    fn test_widget_attributes_basic() {
        let config = Config::default();
        let attrs = config.widget_attributes();

        assert!(attrs.contains("data-project"));
        assert!(attrs.contains("data-theme=\"dark\""));
        assert!(attrs.contains("data-position=\"bottom-right\""));
        assert!(attrs.contains("data-color=\"#e74c3c\""));
    }

    #[test]
    fn test_widget_attributes_with_collect_email() {
        let mut config = Config::default();
        config.widget.collect_email = true;

        let attrs = config.widget_attributes();
        assert!(attrs.contains("data-collect-email=\"true\""));
    }

    #[test]
    fn test_widget_attributes_with_hosted() {
        let mut config = Config::default();
        config.remote.hosted = true;

        let attrs = config.widget_attributes();
        assert!(attrs.contains("data-endpoint"));
        assert!(attrs.contains("spikes.sh"));
    }

    #[test]
    fn test_widget_attributes_with_custom_endpoint() {
        let mut config = Config::default();
        config.remote.endpoint = Some("https://api.custom.com".to_string());

        let attrs = config.widget_attributes();
        assert!(attrs.contains("data-endpoint=\"https://api.custom.com/spikes\""));
    }

    #[test]
    fn test_widget_attributes_with_token() {
        let mut config = Config::default();
        config.remote.endpoint = Some("https://api.custom.com".to_string());
        config.remote.token = Some("my-token".to_string());

        let attrs = config.widget_attributes();
        assert!(attrs.contains("token=my-token"));
    }

    #[test]
    fn test_effective_endpoint_hosted_returns_canonical_url() {
        // VAL-CONFIG-001: When hosted=true, effective_endpoint() returns https://spikes.sh
        let mut config = Config::default();
        config.remote.hosted = true;

        assert_eq!(config.effective_endpoint(), Some("https://spikes.sh".to_string()));
    }

    #[test]
    fn test_widget_attributes_hosted_produces_correct_spikes_url() {
        // VAL-CONFIG-002: widget_attributes() includes data-endpoint="https://spikes.sh/spikes" when hosted=true
        let mut config = Config::default();
        config.remote.hosted = true;

        let attrs = config.widget_attributes();
        assert!(attrs.contains("data-endpoint=\"https://spikes.sh/spikes\""));
        // Ensure no double /spikes
        assert!(!attrs.contains("/spikes/spikes"));
    }

    #[test]
    fn test_explicit_endpoint_wins_over_hosted() {
        // VAL-CONFIG-003: Explicit endpoint wins when both hosted=true and endpoint are set
        let mut config = Config::default();
        config.remote.hosted = true;
        config.remote.endpoint = Some("https://my.worker.dev".to_string());

        // When both are set, the explicit endpoint should win
        assert_eq!(config.effective_endpoint(), Some("https://my.worker.dev".to_string()));
    }

    #[test]
    fn test_widget_attributes_explicit_endpoint_with_hosted() {
        // VAL-CROSS-005: Config with [remote] endpoint="https://my.worker.dev" and hosted=true
        // resolves to the explicit endpoint
        let mut config = Config::default();
        config.remote.hosted = true;
        config.remote.endpoint = Some("https://my.worker.dev".to_string());

        let attrs = config.widget_attributes();
        assert!(attrs.contains("data-endpoint=\"https://my.worker.dev/spikes\""));
        // Should NOT contain spikes.sh since explicit endpoint wins
        assert!(!attrs.contains("spikes.sh"));
    }

    #[test]
    fn test_legacy_config_with_trailing_spikes_not_doubled() {
        // VAL-CONFIG-004: Legacy config with endpoint ending in "/spikes" does NOT produce "/spikes/spikes"
        let mut config = Config::default();
        config.remote.endpoint = Some("https://spikes.sh/spikes".to_string());

        let attrs = config.widget_attributes();
        // Should strip trailing /spikes before appending
        assert!(attrs.contains("data-endpoint=\"https://spikes.sh/spikes\""));
        // Should NOT have double /spikes
        assert!(!attrs.contains("/spikes/spikes"));
    }

    #[test]
    fn test_legacy_config_with_path_prefix_preserved() {
        // Test that endpoints with path prefixes work correctly
        let mut config = Config::default();
        config.remote.endpoint = Some("https://my.worker.dev/api".to_string());

        let attrs = config.widget_attributes();
        assert!(attrs.contains("data-endpoint=\"https://my.worker.dev/api/spikes\""));
    }

    #[test]
    fn test_config_without_remote_section_loads_cleanly() {
        // VAL-CONFIG-024: Configs with no [remote] at all still load cleanly
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let content = r#"
[project]
key = "no-remote-project"

[widget]
theme = "light"
"#;
        std::fs::write(&config_path, content).unwrap();

        let config = Config::load_from(&config_path).unwrap();

        // Should use defaults for missing remote section
        assert!(!config.remote.hosted);
        assert!(config.remote.endpoint.is_none());
        assert!(config.remote.token.is_none());
    }

    #[test]
    fn test_partial_config() {
        // Test that partial configs use defaults for missing fields
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let content = r#"
[project]
key = "partial-project"
"#;
        std::fs::write(&config_path, content).unwrap();

        let config = Config::load_from(&config_path).unwrap();

        assert_eq!(config.project.key, Some("partial-project".to_string()));
        // Widget should use defaults
        assert_eq!(config.widget.theme, "dark");
        assert_eq!(config.widget.position, "bottom-right");
    }
}
