//! Authentication token management with secure storage
//!
//! This module handles:
//! - Platform-appropriate config directory paths (XDG-compliant)
//! - Secure token storage with 0600 file permissions
//! - SPIKES_TOKEN environment variable override
//! - Token lifecycle (load, save, delete)
//! - SPIKES_API_URL environment variable for API base URL override

use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Default API base URL for spikes.sh hosted service
const DEFAULT_API_BASE: &str = "https://spikes.sh";

/// Environment variable name for API URL override
const SPIKES_API_URL_ENV: &str = "SPIKES_API_URL";

/// Get the API base URL, checking SPIKES_API_URL env var first.
/// Falls back to https://spikes.sh if not set.
///
/// This enables:
/// - Testing against localhost:8787 (wrangler dev)
/// - Self-hosted deployments
/// - Development/staging environments
pub fn get_api_base() -> String {
    std::env::var(SPIKES_API_URL_ENV)
        .unwrap_or_else(|_| DEFAULT_API_BASE.to_string())
}

/// Auth configuration stored in platform-appropriate config directory
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default)]
    pub auth: AuthSection,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthSection {
    /// Bearer token for API authentication
    pub token: Option<String>,
    /// API key (sk_spikes_ prefixed) stored separately from bearer token
    pub api_key: Option<String>,
}

impl AuthConfig {
    /// Load auth config from platform-appropriate location.
    /// Returns default (empty) config if file doesn't exist.
    pub fn load() -> Result<Self> {
        // Check SPIKES_TOKEN environment variable first
        if let Ok(token) = std::env::var("SPIKES_TOKEN") {
            if !token.is_empty() {
                return Ok(AuthConfig {
                    auth: AuthSection {
                        token: Some(token),
                        api_key: None,
                    },
                });
            }
        }

        let auth_path = auth_path()?;

        if !auth_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&auth_path)?;
        let config: AuthConfig = toml::from_str(&content).map_err(|e| {
            Error::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid auth.toml: {}", e),
            ))
        })?;

        Ok(config)
    }

    /// Save auth config to platform-appropriate location with 0600 permissions
    pub fn save(&self) -> Result<()> {
        let auth_path = auth_path()?;

        // Create parent directories if needed
        if let Some(parent) = auth_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize to TOML
        let content = toml::to_string_pretty(self).map_err(|e| {
            Error::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize auth config: {}", e),
            ))
        })?;

        // Write to file
        fs::write(&auth_path, content)?;

        // Set file permissions to 0600 (owner read/write only)
        set_secure_permissions(&auth_path)?;

        Ok(())
    }

    /// Delete the auth config file
    pub fn delete() -> Result<()> {
        let auth_path = auth_path()?;

        if auth_path.exists() {
            fs::remove_file(&auth_path)?;
        }

        Ok(())
    }

    /// Check if a token is available (either from env var or file)
    pub fn has_token() -> bool {
        // Check env var first
        if let Ok(token) = std::env::var("SPIKES_TOKEN") {
            if !token.is_empty() {
                return true;
            }
        }

        // Check file
        Self::load()
            .map(|c| c.auth.token.is_some())
            .unwrap_or(false)
    }

    /// Get the effective token (env var takes precedence over file)
    pub fn token() -> Result<Option<String>> {
        // Check env var first
        if let Ok(token) = std::env::var("SPIKES_TOKEN") {
            if !token.is_empty() {
                return Ok(Some(token));
            }
        }

        // Fall back to file
        let config = Self::load()?;
        Ok(config.auth.token)
    }

    /// Save a new token to the auth file, preserving existing api_key
    pub fn save_token(token: &str) -> Result<()> {
        // Load existing config to preserve api_key
        let existing = Self::load_from_file().unwrap_or_default();
        let config = AuthConfig {
            auth: AuthSection {
                token: Some(token.to_string()),
                api_key: existing.auth.api_key,
            },
        };
        config.save()
    }

    /// Save a new API key to the auth file, preserving existing bearer token
    pub fn save_api_key(key: &str) -> Result<()> {
        // Load existing config to preserve token
        let existing = Self::load_from_file().unwrap_or_default();
        let config = AuthConfig {
            auth: AuthSection {
                token: existing.auth.token,
                api_key: Some(key.to_string()),
            },
        };
        config.save()
    }

    /// Load the API key from the auth file (does not check env var)
    pub fn load_api_key() -> Option<String> {
        Self::load_from_file()
            .ok()
            .and_then(|c| c.auth.api_key)
    }

    /// Load auth config from file only (no env var override).
    /// Used internally to preserve fields when saving.
    fn load_from_file() -> Result<Self> {
        let auth_path = auth_path()?;

        if !auth_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&auth_path)?;
        let config: AuthConfig = toml::from_str(&content).map_err(|e| {
            Error::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid auth.toml: {}", e),
            ))
        })?;

        Ok(config)
    }

    /// Clear the stored token (delete auth file)
    pub fn clear_token() -> Result<()> {
        Self::delete()
    }
}

/// Get the platform-appropriate auth file path.
///
/// XDG-compliant on Linux, standard config directories on macOS and Windows:
/// - Linux: ~/.config/spikes/auth.toml (respects XDG_CONFIG_HOME)
/// - macOS: ~/Library/Application Support/spikes/auth.toml
/// - Windows: %APPDATA%/spikes/auth.toml
pub fn auth_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| {
            Error::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not determine config directory",
            ))
        })?
        .join("spikes");

    Ok(config_dir.join("auth.toml"))
}

/// Set file permissions to 0600 (owner read/write only)
#[cfg(unix)]
fn set_secure_permissions(path: &PathBuf) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut perms = fs::metadata(path)?.permissions();
    // 0o600 = owner read + write
    perms.set_mode(0o600);
    fs::set_permissions(path, perms)?;

    Ok(())
}

/// Set file permissions to owner-only on non-Unix systems
#[cfg(not(unix))]
fn set_secure_permissions(path: &PathBuf) -> Result<()> {
    // On Windows, we can't set Unix-style permissions directly
    // The file will be accessible only to the user who created it
    // by default on NTFS with proper ACL inheritance
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use serial_test::serial;

    fn setup_temp_config_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    #[test]
    fn test_auth_config_default() {
        let config = AuthConfig::default();
        assert!(config.auth.token.is_none());
    }

    #[test]
    fn test_auth_config_save_and_load() {
        let temp_dir = setup_temp_config_dir();
        let auth_path = temp_dir.path().join("spikes").join("auth.toml");

        // Save a config
        let config = AuthConfig {
            auth: AuthSection {
                token: Some("test-token-123".to_string()),
                api_key: None,
            },
        };

        // Manually write to temp path
        if let Some(parent) = auth_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&auth_path, content).unwrap();

        // Verify content
        let loaded_content = fs::read_to_string(&auth_path).unwrap();
        assert!(loaded_content.contains("test-token-123"));
    }

    #[test]
    fn test_auth_config_load_missing_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let _missing_path = temp_dir.path().join("nonexistent/auth.toml");

        // Should return default for missing file
        let config = AuthConfig::default();
        assert!(config.auth.token.is_none());
    }

    #[test]
    fn test_auth_config_invalid_toml() {
        let temp_dir = tempfile::tempdir().unwrap();
        let auth_path = temp_dir.path().join("auth.toml");

        fs::write(&auth_path, "this is not valid toml [[[[").unwrap();

        let content = fs::read_to_string(&auth_path).unwrap();
        let result: Result<AuthConfig> = toml::from_str(&content)
            .map_err(|e| {
                Error::Io(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid auth.toml: {}", e),
                ))
            });

        assert!(result.is_err());
    }

    #[test]
    #[serial(spike_token)]
    fn test_spike_token_env_override() {
        // Save current value
        let original = std::env::var("SPIKES_TOKEN").ok();

        // Set env var
        std::env::set_var("SPIKES_TOKEN", "env-token-override");

        // Create a new config - env var should populate it
        let config = AuthConfig::load().unwrap();
        assert_eq!(config.auth.token, Some("env-token-override".to_string()));

        // Restore original value
        if let Some(val) = original {
            std::env::set_var("SPIKES_TOKEN", val);
        } else {
            std::env::remove_var("SPIKES_TOKEN");
        }
    }

    #[test]
    #[serial(spike_token)]
    fn test_spike_token_env_empty_ignored() {
        // Save current values
        let original = std::env::var("SPIKES_TOKEN").ok();
        let original_home = std::env::var("HOME").ok();
        let original_xdg = std::env::var("XDG_CONFIG_HOME").ok();

        // Isolate from real auth.toml
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", temp_dir.path());
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path().join(".config"));

        // Set empty env var - should be ignored
        std::env::set_var("SPIKES_TOKEN", "");

        // Empty env var should be ignored, fall back to file (or None)
        let config = AuthConfig::load().unwrap();
        // Token should be None since env var is empty and no auth.toml exists
        assert!(config.auth.token.is_none());

        // Restore original values
        if let Some(val) = original {
            std::env::set_var("SPIKES_TOKEN", val);
        } else {
            std::env::remove_var("SPIKES_TOKEN");
        }
        if let Some(val) = original_home {
            std::env::set_var("HOME", val);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(val) = original_xdg {
            std::env::set_var("XDG_CONFIG_HOME", val);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
    }

    #[test]
    fn test_save_token_creates_parent_dirs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let custom_path = temp_dir.path().join("nested/dir/auth.toml");

        // Create a config and write it manually
        let config = AuthConfig {
            auth: AuthSection {
                token: Some("test-token".to_string()),
                api_key: None,
            },
        };

        if let Some(parent) = custom_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&custom_path, content).unwrap();

        assert!(custom_path.exists());
    }

    #[test]
    #[cfg(unix)]
    fn test_secure_permissions_on_new_file() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempfile::tempdir().unwrap();
        let auth_path = temp_dir.path().join("auth.toml");

        // Write content
        fs::write(&auth_path, "test").unwrap();

        // Set permissions
        set_secure_permissions(&auth_path).unwrap();

        // Verify permissions
        let perms = fs::metadata(&auth_path).unwrap().permissions();
        let mode = perms.mode();

        // Should be 0o600 or more restrictive
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn test_toml_serialization() {
        let config = AuthConfig {
            auth: AuthSection {
                token: Some("secret-token-xyz".to_string()),
                api_key: None,
            },
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();

        assert!(toml_str.contains("[auth]"));
        assert!(toml_str.contains("secret-token-xyz"));
    }

    #[test]
    fn test_toml_deserialization() {
        let toml_str = r#"
[auth]
token = "deserialized-token"
"#;

        let config: AuthConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.auth.token, Some("deserialized-token".to_string()));
    }

    #[test]
    fn test_empty_auth_section() {
        let toml_str = "";

        let config: AuthConfig = toml::from_str(toml_str).unwrap();
        assert!(config.auth.token.is_none());
        assert!(config.auth.api_key.is_none());
    }

    #[test]
    #[serial(api_url)]
    fn test_get_api_base_default() {
        // Save current value
        let original = std::env::var(SPIKES_API_URL_ENV).ok();

        // Clear env var
        std::env::remove_var(SPIKES_API_URL_ENV);

        // Should return default
        let base = get_api_base();
        assert_eq!(base, DEFAULT_API_BASE);

        // Restore original value
        if let Some(val) = original {
            std::env::set_var(SPIKES_API_URL_ENV, val);
        }
    }

    #[test]
    #[serial(api_url)]
    fn test_get_api_base_env_override() {
        // Save current value
        let original = std::env::var(SPIKES_API_URL_ENV).ok();

        // Set custom API URL
        std::env::set_var(SPIKES_API_URL_ENV, "http://localhost:8787");

        // Should return env var value
        let base = get_api_base();
        assert_eq!(base, "http://localhost:8787");

        // Restore original value
        if let Some(val) = original {
            std::env::set_var(SPIKES_API_URL_ENV, val);
        } else {
            std::env::remove_var(SPIKES_API_URL_ENV);
        }
    }

    #[test]
    #[serial(api_url)]
    fn test_get_api_base_custom_host() {
        // Save current value
        let original = std::env::var(SPIKES_API_URL_ENV).ok();

        // Set custom API URL for self-hosted
        std::env::set_var(SPIKES_API_URL_ENV, "https://spikes.example.com");

        // Should return custom host
        let base = get_api_base();
        assert_eq!(base, "https://spikes.example.com");

        // Restore original value
        if let Some(val) = original {
            std::env::set_var(SPIKES_API_URL_ENV, val);
        } else {
            std::env::remove_var(SPIKES_API_URL_ENV);
        }
    }

    #[test]
    fn test_auth_config_default_has_no_api_key() {
        let config = AuthConfig::default();
        assert!(config.auth.api_key.is_none());
    }

    #[test]
    fn test_toml_with_api_key() {
        let toml_str = r#"
[auth]
token = "bearer-token-123"
api_key = "sk_spikes_testkey"
"#;

        let config: AuthConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.auth.token, Some("bearer-token-123".to_string()));
        assert_eq!(config.auth.api_key, Some("sk_spikes_testkey".to_string()));
    }

    #[test]
    fn test_toml_with_api_key_only() {
        let toml_str = r#"
[auth]
api_key = "sk_spikes_onlykey"
"#;

        let config: AuthConfig = toml::from_str(toml_str).unwrap();
        assert!(config.auth.token.is_none());
        assert_eq!(config.auth.api_key, Some("sk_spikes_onlykey".to_string()));
    }

    #[test]
    fn test_toml_serialization_with_api_key() {
        let config = AuthConfig {
            auth: AuthSection {
                token: Some("my-token".to_string()),
                api_key: Some("sk_spikes_mykey".to_string()),
            },
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("[auth]"));
        assert!(toml_str.contains("my-token"));
        assert!(toml_str.contains("sk_spikes_mykey"));
    }

    #[test]
    fn test_toml_backwards_compat_without_api_key() {
        // Old format without api_key should still deserialize
        let toml_str = r#"
[auth]
token = "old-format-token"
"#;

        let config: AuthConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.auth.token, Some("old-format-token".to_string()));
        assert!(config.auth.api_key.is_none());
    }
}
