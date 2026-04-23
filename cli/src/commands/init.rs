use std::fs;
use std::io::{self, BufRead, BufReader, IsTerminal, Write};
use std::path::Path;

use crate::error::Result;

/// Config template for hosted spikes.sh (the default)
const HOSTED_CONFIG: &str = r##"# Spikes configuration
# https://spikes.sh

[project]
# Project key for grouping spikes
# key = "my-project"

[widget]
# Widget appearance
theme = "dark"           # "dark" or "light"
position = "bottom-right" # "bottom-right", "bottom-left", "top-right", "top-left"
color = "#e74c3c"        # Accent color (hex)
collect_email = false    # Ask reviewers for email (builds prospect list)

[remote]
# Use spikes.sh hosted backend (default)
hosted = true
endpoint = "https://spikes.sh"
# token = "your-token-here"  # Optional: for password-protected shares
"##;

/// Config template for self-host path
const SELF_HOST_CONFIG: &str = r##"# Spikes configuration
# https://spikes.sh

[project]
# Project key for grouping spikes
# key = "my-project"

[widget]
# Widget appearance
theme = "dark"           # "dark" or "light"
position = "bottom-right" # "bottom-right", "bottom-left", "top-right", "top-left"
color = "#e74c3c"        # Accent color (hex)
collect_email = false    # Ask reviewers for email (builds prospect list)

[remote]
# Cloud sync configuration - uncomment and fill in after running `spikes deploy cloudflare`
# endpoint = "https://your-worker.workers.dev"
# token = "your-token-here"
# hosted = false  # Set to true to use spikes.sh instead of self-hosted
"##;

const SPIKES_GITIGNORE_ENTRY: &str = ".spikes/\n";

/// Prompt the user for hosted vs self-host choice in interactive mode
/// Returns true for hosted (default), false for self-host
fn prompt_hosted_interactive() -> io::Result<bool> {
    println!("Use hosted spikes.sh (default) or self-host? [H/s]");

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();

    // Empty (Enter), 'h', or 'H' = hosted (default)
    // 's' or 'S' = self-host
    match trimmed {
        "" | "h" | "H" => Ok(true),
        "s" | "S" => Ok(false),
        _ => {
            // Invalid input defaults to hosted
            println!("Invalid input, defaulting to hosted spikes.sh");
            Ok(true)
        }
    }
}

/// Check if stdin is interactive (TTY) or not
fn is_interactive() -> bool {
    io::stdin().is_terminal()
}

pub fn run(json: bool, self_host: bool) -> Result<()> {
    let spikes_dir = Path::new(".spikes");

    if spikes_dir.exists() {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "success": false,
                    "error": ".spikes directory already exists"
                })
            );
        } else {
            eprintln!(".spikes directory already exists");
        }
        return Ok(());
    }

    // Determine hosted vs self-host
    let use_hosted = if self_host {
        // Explicit --self-host flag overrides everything
        false
    } else if json || !is_interactive() {
        // Non-interactive (piped stdin or --json flag) defaults to hosted
        true
    } else {
        // Interactive mode - prompt the user
        prompt_hosted_interactive().unwrap_or(true) // Default to hosted on error
    };

    // Select the appropriate config template
    let config_content = if use_hosted {
        HOSTED_CONFIG
    } else {
        SELF_HOST_CONFIG
    };

    fs::create_dir_all(spikes_dir)?;
    fs::write(spikes_dir.join("config.toml"), config_content)?;
    fs::write(spikes_dir.join("feedback.jsonl"), "")?;

    // Update .gitignore
    let gitignore_path = Path::new(".gitignore");
    let gitignore_updated = update_gitignore(gitignore_path)?;

    if json {
        let mut created = vec![
            ".spikes/config.toml",
            ".spikes/feedback.jsonl",
        ];
        if gitignore_updated {
            created.push(".gitignore");
        }
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "created": created,
                "hosted": use_hosted,
                "remote": {
                    "hosted": use_hosted,
                    "endpoint": if use_hosted { Some("https://spikes.sh") } else { None::<&str> }
                }
            })
        );
    } else {
        if use_hosted {
            println!("Initialized .spikes/ directory (hosted spikes.sh)");
        } else {
            println!("Initialized .spikes/ directory (self-host mode)");
            println!("  Run `spikes deploy cloudflare` to scaffold your own backend");
        }
        println!("  Created: .spikes/config.toml");
        println!("  Created: .spikes/feedback.jsonl");
        if gitignore_updated {
            println!("  Updated: .gitignore");
        }
    }

    Ok(())
}

/// Update .gitignore to include .spikes/ entry.
/// Returns true if the file was created or modified.
fn update_gitignore(path: &Path) -> Result<bool> {
    if !path.exists() {
        // Create new .gitignore with .spikes/ entry
        let mut file = fs::File::create(path)?;
        file.write_all(SPIKES_GITIGNORE_ENTRY.as_bytes())?;
        return Ok(true);
    }

    // Check if .spikes/ already exists in .gitignore
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed == ".spikes" || trimmed == ".spikes/" {
            // Already present, no need to add
            return Ok(false);
        }
    }

    // Append .spikes/ entry
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(path)?;

    // Ensure there's a newline before our entry if the file doesn't end with one
    let metadata = file.metadata()?;
    if metadata.len() > 0 {
        let existing = fs::read_to_string(path)?;
        if !existing.ends_with('\n') {
            file.write_all(b"\n")?;
        }
    }

    file.write_all(SPIKES_GITIGNORE_ENTRY.as_bytes())?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_update_gitignore_creates_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore_path = temp_dir.path().join(".gitignore");

        let result = update_gitignore(&gitignore_path).unwrap();
        assert!(result, "Should return true when creating new file");
        assert!(gitignore_path.exists(), ".gitignore should be created");

        let content = fs::read_to_string(&gitignore_path).unwrap();
        assert!(content.contains(".spikes/"), "Should contain .spikes/");
    }

    #[test]
    fn test_update_gitignore_appends_to_existing() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore_path = temp_dir.path().join(".gitignore");

        fs::write(&gitignore_path, "target/\n*.log\n").unwrap();

        let result = update_gitignore(&gitignore_path).unwrap();
        assert!(result, "Should return true when appending");

        let content = fs::read_to_string(&gitignore_path).unwrap();
        assert!(content.contains("target/"), "Should preserve existing content");
        assert!(content.contains("*.log"), "Should preserve existing content");
        assert!(content.contains(".spikes/"), "Should append .spikes/");
    }

    #[test]
    fn test_update_gitignore_skips_if_already_present() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore_path = temp_dir.path().join(".gitignore");

        fs::write(&gitignore_path, "target/\n.spikes/\n*.log\n").unwrap();

        let result = update_gitignore(&gitignore_path).unwrap();
        assert!(!result, "Should return false when already present");

        let content = fs::read_to_string(&gitignore_path).unwrap();
        let count = content.matches(".spikes/").count();
        assert_eq!(count, 1, "Should only have one .spikes/ entry");
    }

    #[test]
    fn test_update_gitignore_recognizes_dotspikes_without_slash() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore_path = temp_dir.path().join(".gitignore");

        fs::write(&gitignore_path, "target/\n.spikes\n").unwrap();

        let result = update_gitignore(&gitignore_path).unwrap();
        assert!(!result, "Should return false when .spikes (without slash) is present");
    }

    #[test]
    fn test_update_gitignore_handles_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore_path = temp_dir.path().join(".gitignore");

        fs::write(&gitignore_path, "").unwrap();

        let result = update_gitignore(&gitignore_path).unwrap();
        assert!(result, "Should return true when appending to empty file");

        let content = fs::read_to_string(&gitignore_path).unwrap();
        assert!(content.contains(".spikes/"), "Should contain .spikes/");
    }

    #[test]
    fn test_update_gitignore_handles_no_trailing_newline() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore_path = temp_dir.path().join(".gitignore");

        // File without trailing newline
        fs::write(&gitignore_path, "target/").unwrap();

        let result = update_gitignore(&gitignore_path).unwrap();
        assert!(result, "Should return true when appending");

        let content = fs::read_to_string(&gitignore_path).unwrap();
        assert!(content.ends_with(".spikes/\n"), "Should properly add entry with newline");
    }
}
