use std::path::Path;

use crate::config::Config;
use crate::error::{Error, Result};

/// Internal run function that returns a Result without side-effects.
/// Separates logic from exit handling for testability.
fn run_internal(json: bool) -> Result<Option<serde_json::Value>> {
    // Check if .spikes/ directory exists - parity with `spikes list`
    // Must be a directory, not just any file type (handles file/symlink edge case)
    if !Path::new(".spikes").is_dir() {
        if json {
            // JSON mode: return structured error as JSON value
            let error_json = serde_json::json!({
                "error": "No .spikes/ directory found. Run 'spikes init' first."
            });
            return Ok(Some(error_json));
        }
        return Err(Error::NoSpikesDir);
    }

    let config = Config::load()?;

    if json {
        let output = serde_json::json!({
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
        });
        return Ok(Some(output));
    }

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

    Ok(None)
}

/// Show current configuration
pub fn run(json: bool) -> Result<()> {
    match run_internal(json) {
        Ok(Some(json_value)) => {
            // Check if this is an error response
            if json_value.get("error").is_some() {
                // JSON error mode: print to stdout and exit non-zero
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json_value)
                        .expect("Failed to serialize JSON")
                );
                std::process::exit(1);
            }
            // Normal JSON success output
            println!(
                "{}",
                serde_json::to_string_pretty(&json_value)
                    .expect("Failed to serialize JSON")
            );
            Ok(())
        }
        Ok(None) => Ok(()),
        Err(e) => Err(e),
    }
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
        let result = run_internal(false);
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

        // Without .spikes/ directory, config --json should return a JSON error value
        // (via run_internal, which doesn't call exit())
        let result = run_internal(true);
        assert!(result.is_ok(), "run_internal should succeed and return JSON error");
        
        let json_value = result.unwrap();
        assert!(json_value.is_some(), "should return JSON value");
        
        let json = json_value.unwrap();
        assert!(json.get("error").is_some(), "JSON should contain error field");
        
        let error_msg = json.get("error").unwrap().as_str().unwrap();
        assert!(error_msg.contains(".spikes/"), "Error should mention .spikes/");
        assert!(error_msg.contains("spikes init"), "Error should reference 'spikes init'");

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
        // Use run_internal which returns None for success (no JSON output)
        let result = run_internal(false);
        assert!(result.is_ok(), "config should succeed when .spikes/ exists: {:?}", result);
        assert!(result.unwrap().is_none(), "Success in non-JSON mode should return None");

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
        // Use run_internal which returns Some(JSON) for success in JSON mode
        let result = run_internal(true);
        assert!(result.is_ok(), "config --json should succeed when .spikes/ exists: {:?}", result);
        
        let json_value = result.unwrap();
        assert!(json_value.is_some(), "JSON mode should return Some(JSON value)");
        
        let json = json_value.unwrap();
        assert!(json.get("project").is_some(), "JSON should contain project");
        assert!(json.get("widget").is_some(), "JSON should contain widget");
        assert!(json.get("remote").is_some(), "JSON should contain remote");
        
        // Verify hosted flag is present
        let remote = json.get("remote").unwrap();
        assert_eq!(remote.get("hosted").unwrap().as_bool(), Some(true));

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
        let result = run_internal(false);
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

        // When .spikes is a file, config --json should return a JSON error value
        // (via run_internal, which doesn't call exit())
        let result = run_internal(true);
        assert!(result.is_ok(), "config --json (via run_internal) should return JSON error");
        
        let json_value = result.unwrap();
        assert!(json_value.is_some(), "should return JSON value");
        
        let json = json_value.unwrap();
        assert!(json.get("error").is_some(), "JSON should contain error field");
        
        let error_msg = json.get("error").unwrap().as_str().unwrap();
        assert!(error_msg.contains(".spikes/"), "Error should mention .spikes/");
        assert!(error_msg.contains("spikes init"), "Error should reference 'spikes init'");

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

    /// Regression test for VAL-CONFIG-016: `spikes config --json` in uninitialized
    /// directory should emit structured JSON error with `error` field, not plain text.
    /// 
    /// This test verifies both paths:
    /// 1. json=false: returns Err(Error::NoSpikesDir) with plain text message
    /// 2. json=true: returns Ok(Some(json)) with {"error": "..."} structure
    #[test]
    fn test_config_json_error_contains_init_guidance() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Test non-JSON path: should return Err with NoSpikesDir
        let result = run_internal(false);
        assert!(result.is_err());
        
        let err = result.unwrap_err();
        assert!(matches!(err, Error::NoSpikesDir), "Should return NoSpikesDir error");
        
        // Verify error message mentions both .spikes/ and spikes init
        let err_string = err.to_string();
        assert!(err_string.contains(".spikes/"), "Error should mention .spikes/ directory: {}", err_string);
        assert!(err_string.contains("spikes init"), "Error should reference 'spikes init' command: {}", err_string);

        // Test JSON path: should return Ok with JSON error
        let json_result = run_internal(true);
        assert!(json_result.is_ok(), "JSON mode should return Ok with JSON error");
        
        let json_value = json_result.unwrap();
        assert!(json_value.is_some(), "JSON mode should return Some(JSON)");
        
        let json = json_value.unwrap();
        assert!(json.get("error").is_some(), "JSON should contain error field");
        
        let error_msg = json.get("error").unwrap().as_str().unwrap();
        assert!(error_msg.contains(".spikes/"), "JSON error should mention .spikes/");
        assert!(error_msg.contains("spikes init"), "JSON error should reference 'spikes init'");

        std::env::set_current_dir(original_cwd).unwrap();
    }
}
