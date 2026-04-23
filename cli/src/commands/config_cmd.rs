use std::path::Path;

use crate::config::Config;
use crate::error::{Error, Result};

/// Show current configuration
pub fn run(json: bool) -> Result<()> {
    // Check if .spikes/ directory exists - parity with `spikes list`
    // Must be a directory, not just any file type (handles file/symlink edge case)
    if !Path::new(".spikes").is_dir() {
        return Err(Error::NoSpikesDir);
    }

    let config = Config::load()?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "project": {
                    "key": config.effective_project_key()
                },
                "widget": {
                    "theme": config.widget.theme,
                    "position": config.widget.position,
                    "color": config.widget.color,
                    "collect_email": config.widget.collect_email
                },
                "remote": {
                    "endpoint": config.effective_endpoint(),
                    "hosted": config.remote.hosted,
                    "has_token": config.remote.token.is_some()
                }
            })
        );
    } else {
        println!();
        println!("  / Spikes Configuration");
        println!();
        println!("  Project:  {}", config.effective_project_key());
        println!();
        println!("  Widget:");
        println!("    theme:         {}", config.widget.theme);
        println!("    position:      {}", config.widget.position);
        println!("    color:         {}", config.widget.color);
        println!("    collect_email: {}", config.widget.collect_email);
        println!();
        println!("  Remote:");
        if let Some(endpoint) = config.effective_endpoint() {
            println!("    endpoint: {}", endpoint);
            println!("    hosted:   {}", config.remote.hosted);
            println!("    token:    {}", if config.remote.token.is_some() { "(set)" } else { "(not set)" });
        } else {
            println!("    (not configured)");
        }
        println!();
        println!("  Widget tag attributes:");
        println!("    {}", config.widget_attributes());
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use std::sync::Mutex;

    // Use a mutex to prevent parallel test execution for tests that change current directory
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_config_requires_spikes_dir() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Without .spikes/ directory, config should fail with NoSpikesDir
        let result = run(false);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NoSpikesDir));

        std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn test_config_requires_spikes_dir_json() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Without .spikes/ directory, config --json should also fail with NoSpikesDir
        let result = run(true);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NoSpikesDir));

        std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn test_config_succeeds_with_spikes_dir() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let spikes_dir = temp_dir.path().join(".spikes");
        fs::create_dir_all(&spikes_dir).unwrap();

        // Create a minimal config.toml
        let config_content = concat!(
            "[project]\n",
            "key = \"test-project\"\n",
            "\n",
            "[widget]\n",
            "theme = \"dark\"\n",
            "position = \"bottom-right\"\n",
            "color = \"#e74c3c\"\n",
            "collect_email = false\n",
            "\n",
            "[remote]\n",
            "hosted = true\n",
            "endpoint = \"https://spikes.sh\"\n"
        );
        fs::write(spikes_dir.join("config.toml"), config_content).unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // With .spikes/ directory and config, config should succeed
        let result = run(false);
        assert!(result.is_ok(), "config should succeed when .spikes/ exists: {:?}", result);

        std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn test_config_json_succeeds_with_spikes_dir() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let spikes_dir = temp_dir.path().join(".spikes");
        fs::create_dir_all(&spikes_dir).unwrap();

        // Create a minimal config.toml
        let config_content = concat!(
            "[project]\n",
            "key = \"test-project\"\n",
            "\n",
            "[widget]\n",
            "theme = \"dark\"\n",
            "position = \"bottom-right\"\n",
            "color = \"#e74c3c\"\n",
            "collect_email = false\n",
            "\n",
            "[remote]\n",
            "hosted = true\n",
            "endpoint = \"https://spikes.sh\"\n"
        );
        fs::write(spikes_dir.join("config.toml"), config_content).unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // With .spikes/ directory and config, config --json should succeed
        let result = run(true);
        assert!(result.is_ok(), "config --json should succeed when .spikes/ exists: {:?}", result);

        std::env::set_current_dir(original_cwd).unwrap();
    }

    /// Regression test: .spikes as a file (not directory) should fail with NoSpikesDir
    /// This ensures parity with `spikes list` behavior per VAL-CROSS-007
    #[test]
    fn test_config_requires_spikes_dir_not_file() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();

        // Create .spikes as a FILE, not a directory
        let spikes_file = temp_dir.path().join(".spikes");
        fs::write(&spikes_file, "not a directory").unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // When .spikes is a file, config should fail with NoSpikesDir
        // (same as spikes list behavior - parity check)
        let result = run(false);
        assert!(result.is_err(), "config should fail when .spikes is a file");
        assert!(matches!(result.unwrap_err(), Error::NoSpikesDir),
            "config should return NoSpikesDir when .spikes is a file, not a directory");

        // Also verify spikes list fails the same way (parity)
        let list_result = crate::storage::load_spikes();
        assert!(list_result.is_err(), "list should fail when .spikes is a file");
        assert!(matches!(list_result.unwrap_err(), Error::NoSpikesDir),
            "list should return NoSpikesDir when .spikes is a file, not a directory");

        std::env::set_current_dir(original_cwd).unwrap();
    }

    /// Regression test: .spikes as a file with --json flag
    #[test]
    fn test_config_requires_spikes_dir_not_file_json() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();

        // Create .spikes as a FILE, not a directory
        let spikes_file = temp_dir.path().join(".spikes");
        fs::write(&spikes_file, "not a directory").unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // When .spikes is a file, config --json should also fail with NoSpikesDir
        let result = run(true);
        assert!(result.is_err(), "config --json should fail when .spikes is a file");
        assert!(matches!(result.unwrap_err(), Error::NoSpikesDir),
            "config --json should return NoSpikesDir when .spikes is a file");

        std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn test_config_shows_correct_hosted_endpoint() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let spikes_dir = temp_dir.path().join(".spikes");
        fs::create_dir_all(&spikes_dir).unwrap();

        // Create a config with hosted=true
        let config_content = concat!(
            "[remote]\n",
            "hosted = true\n",
            "endpoint = \"https://spikes.sh\"\n"
        );
        fs::write(spikes_dir.join("config.toml"), config_content).unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Load the config and verify it has the correct hosted settings
        let config = Config::load().unwrap();
        assert_eq!(config.effective_endpoint(), Some("https://spikes.sh".to_string()));
        assert!(config.remote.hosted);

        std::env::set_current_dir(original_cwd).unwrap();
    }
}
