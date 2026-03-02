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
    pub fn effective_endpoint(&self) -> Option<String> {
        if self.remote.hosted {
            Some("https://api.spikes.sh".to_string())
        } else {
            self.remote.endpoint.clone()
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
            let full_endpoint = if let Some(token) = &self.remote.token {
                format!("{}/spikes?token={}", endpoint.trim_end_matches('/'), token)
            } else {
                format!("{}/spikes", endpoint.trim_end_matches('/'))
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
    use std::io::Write;

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
        let mut config = Config::default();
        config.remote.hosted = true;

        assert_eq!(config.effective_endpoint(), Some("https://api.spikes.sh".to_string()));
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
