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
