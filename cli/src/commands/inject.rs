use std::fs;
use std::path::Path;

use walkdir::WalkDir;

use crate::config::Config;
use crate::error::{Error, Result};

const SCRIPT_MARKER: &str = "spikes.js";

pub struct InjectOptions {
    pub directory: String,
    pub remove: bool,
    pub widget_url: Option<String>,
    pub json: bool,
}

pub fn run(opts: InjectOptions) -> Result<()> {
    let dir = Path::new(&opts.directory);

    if !dir.exists() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Directory not found: {}", opts.directory),
        )));
    }

    if !dir.is_dir() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Not a directory: {}", opts.directory),
        )));
    }

    let mut injected = Vec::new();
    let mut removed = Vec::new();
    let mut skipped = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "html" && ext != "htm" {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let has_script = content.contains(SCRIPT_MARKER);

        if opts.remove {
            if has_script {
                let new_content = remove_script_tag(&content);
                fs::write(path, new_content)?;
                removed.push(path.display().to_string());
            } else {
                skipped.push(path.display().to_string());
            }
        } else if has_script {
            skipped.push(path.display().to_string());
        } else {
            // Load config to get widget attributes
            let config = Config::load().unwrap_or_default();
            let widget_url = opts.widget_url.as_deref().unwrap_or("/spikes.js");
            let attrs = config.widget_attributes();
            let script_tag = format!(r#"<script src="{}" {}></script>"#, widget_url, attrs);
            let new_content = inject_script_tag(&content, &script_tag);
            fs::write(path, new_content)?;
            injected.push(path.display().to_string());
        }
    }

    if opts.json {
        let result = serde_json::json!({
            "action": if opts.remove { "remove" } else { "inject" },
            "injected": injected,
            "removed": removed,
            "skipped": skipped
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if opts.remove {
        if removed.is_empty() {
            println!("No script tags to remove.");
        } else {
            println!("Removed widget from {} files:", removed.len());
            for f in &removed {
                println!("  ✓ {}", f);
            }
        }
        if !skipped.is_empty() {
            println!("Skipped {} files (no widget found).", skipped.len());
        }
    } else {
        if injected.is_empty() {
            println!("No files to inject.");
        } else {
            println!("Injected widget into {} files:", injected.len());
            for f in &injected {
                println!("  ✓ {}", f);
            }
        }
        if !skipped.is_empty() {
            println!("Skipped {} files (already has widget).", skipped.len());
        }
    }

    Ok(())
}

fn inject_script_tag(content: &str, script_tag: &str) -> String {
    let content_lower = content.to_lowercase();

    if let Some(pos) = content_lower.rfind("</body>") {
        let (before, after) = content.split_at(pos);
        format!("{}\n{}\n{}", before.trim_end(), script_tag, after)
    } else if let Some(pos) = content_lower.rfind("</html>") {
        let (before, after) = content.split_at(pos);
        format!("{}\n{}\n{}", before.trim_end(), script_tag, after)
    } else {
        format!("{}\n{}", content, script_tag)
    }
}

fn remove_script_tag(content: &str) -> String {
    let mut result = String::new();
    let mut removed_blank_line = false;

    for line in content.lines() {
        if line.contains(SCRIPT_MARKER) {
            removed_blank_line = true;
            continue;
        }
        if removed_blank_line && line.trim().is_empty() {
            removed_blank_line = false;
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }

    if result.ends_with('\n') && !content.ends_with('\n') {
        result.pop();
    }

    result
}
