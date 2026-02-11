use std::fs;
use std::path::Path;

use crate::error::Result;

const DEFAULT_CONFIG: &str = r#"# Spikes configuration
# https://spikes.sh

[project]
# Project key for grouping spikes
# key = "my-project"

[sync]
# Optional endpoint for cloud sync
# endpoint = "https://my-worker.workers.dev/spikes"
# token = "your-token-here"
"#;

pub fn run(json: bool) -> Result<()> {
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

    fs::create_dir_all(spikes_dir)?;
    fs::write(spikes_dir.join("config.toml"), DEFAULT_CONFIG)?;
    fs::write(spikes_dir.join("feedback.jsonl"), "")?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "created": [
                    ".spikes/config.toml",
                    ".spikes/feedback.jsonl"
                ]
            })
        );
    } else {
        println!("Initialized .spikes/ directory");
        println!("  Created: .spikes/config.toml");
        println!("  Created: .spikes/feedback.jsonl");
    }

    Ok(())
}
