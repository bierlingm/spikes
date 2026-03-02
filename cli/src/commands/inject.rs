use std::fs;
use std::path::Path;

use walkdir::WalkDir;

use crate::config::Config;
use crate::error::{Error, Result};

const SCRIPT_MARKER: &str = "spikes";

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
            let widget_url = opts.widget_url.as_deref().unwrap_or("https://spikes.sh/widget.js");
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
        if !injected.is_empty() {
            let widget_url = opts.widget_url.as_deref().unwrap_or("https://spikes.sh/widget.js");
            if opts.widget_url.is_none() {
                println!();
                println!("Tip: Using CDN (https://spikes.sh/widget.js). For local-only use: spikes serve");
            } else if widget_url.starts_with('/') {
                println!();
                println!("Note: Using relative path \"{}\". This requires \"spikes serve\" or your own server to host the widget file.", widget_url);
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_inject_script_tag_before_body() {
        let html = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body><h1>Hello</h1></body>
</html>"#;

        let script_tag = r#"<script src="https://spikes.sh/widget.js" data-project="test"></script>"#;
        let result = inject_script_tag(html, script_tag);

        assert!(result.contains("</script>\n</body>"));
        assert!(result.contains("spikes.sh/widget.js"));
    }

    #[test]
    fn test_inject_script_tag_before_html() {
        // HTML without </body> tag
        let html = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body><h1>Hello</h1>
</html>"#;

        let script_tag = r#"<script src="/spikes.js"></script>"#;
        let result = inject_script_tag(html, script_tag);

        assert!(result.contains("</script>\n</html>"));
    }

    #[test]
    fn test_inject_script_tag_no_closing_tags() {
        // Minimal HTML without proper closing tags
        let html = r#"<h1>Hello World</h1>"#;

        let script_tag = r#"<script src="/widget.js"></script>"#;
        let result = inject_script_tag(html, script_tag);

        // Should append at the end
        assert!(result.ends_with("</script>"));
    }

    #[test]
    fn test_remove_script_tag() {
        let html = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body>
<h1>Hello</h1>
<script src="https://spikes.sh/widget.js" data-project="test"></script>
</body>
</html>"#;

        let result = remove_script_tag(html);

        assert!(!result.contains("spikes.sh/widget.js"));
        assert!(result.contains("<h1>Hello</h1>"));
        assert!(result.contains("</body>"));
    }

    #[test]
    fn test_remove_script_tag_preserves_original_newlines() {
        let html = "<html><body>Content</body></html>\n"; // Has trailing newline

        let result = remove_script_tag(html);
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_inject_into_directory() {
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("test.html");

        std::fs::write(&html_path, r#"<!DOCTYPE html>
<html><body><h1>Test</h1></body></html>"#).unwrap();

        let opts = InjectOptions {
            directory: temp_dir.path().to_string_lossy().to_string(),
            remove: false,
            widget_url: Some("https://test.widget.js".to_string()),
            json: true,
        };

        run(opts).unwrap();

        let content = std::fs::read_to_string(&html_path).unwrap();
        assert!(content.contains("test.widget.js"));
    }

    #[test]
    fn test_inject_skips_files_with_widget() {
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("test.html");

        // File already has widget
        std::fs::write(&html_path, r#"<!DOCTYPE html>
<html><body>
<h1>Test</h1>
<script src="https://spikes.sh/widget.js"></script>
</body></html>"#).unwrap();

        let opts = InjectOptions {
            directory: temp_dir.path().to_string_lossy().to_string(),
            remove: false,
            widget_url: None,
            json: true,
        };

        run(opts).unwrap();

        // Content should be unchanged
        let content = std::fs::read_to_string(&html_path).unwrap();
        // Should still have exactly one script tag
        assert_eq!(content.matches("spikes").count(), 1);
    }

    #[test]
    fn test_remove_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("test.html");

        std::fs::write(&html_path, r#"<!DOCTYPE html>
<html><body>
<h1>Test</h1>
<script src="https://spikes.sh/widget.js" data-project="test"></script>
</body></html>"#).unwrap();

        let opts = InjectOptions {
            directory: temp_dir.path().to_string_lossy().to_string(),
            remove: true,
            widget_url: None,
            json: true,
        };

        run(opts).unwrap();

        let content = std::fs::read_to_string(&html_path).unwrap();
        assert!(!content.contains("spikes.sh/widget.js"));
        assert!(content.contains("<h1>Test</h1>"));
    }

    #[test]
    fn test_remove_skips_files_without_widget() {
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("test.html");

        std::fs::write(&html_path, r#"<!DOCTYPE html>
<html><body><h1>Test</h1></body></html>"#).unwrap();

        let opts = InjectOptions {
            directory: temp_dir.path().to_string_lossy().to_string(),
            remove: true,
            widget_url: None,
            json: true,
        };

        run(opts).unwrap();

        // Content should be unchanged
        let content = std::fs::read_to_string(&html_path).unwrap();
        assert!(content.contains("<h1>Test</h1>"));
    }

    #[test]
    fn test_inject_nonexistent_directory() {
        let opts = InjectOptions {
            directory: "/nonexistent/path".to_string(),
            remove: false,
            widget_url: None,
            json: true,
        };

        let result = run(opts);
        assert!(result.is_err());
    }

    #[test]
    fn test_inject_file_path() {
        // Should reject a file path (not a directory)
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.html");
        std::fs::write(&file_path, "<html></html>").unwrap();

        let opts = InjectOptions {
            directory: file_path.to_string_lossy().to_string(),
            remove: false,
            widget_url: None,
            json: true,
        };

        let result = run(opts);
        assert!(result.is_err());
    }

    #[test]
    fn test_inject_nested_files() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("subdir/nested.html");
        std::fs::create_dir_all(nested_path.parent().unwrap()).unwrap();
        std::fs::write(&nested_path, r#"<html><body>Nested</body></html>"#).unwrap();

        let opts = InjectOptions {
            directory: temp_dir.path().to_string_lossy().to_string(),
            remove: false,
            widget_url: None,
            json: true,
        };

        run(opts).unwrap();

        let content = std::fs::read_to_string(&nested_path).unwrap();
        assert!(content.contains("spikes.sh/widget.js"));
    }

    #[test]
    fn test_inject_skips_non_html_files() {
        let temp_dir = TempDir::new().unwrap();

        let html_path = temp_dir.path().join("test.html");
        let css_path = temp_dir.path().join("style.css");
        let js_path = temp_dir.path().join("script.js");

        std::fs::write(&html_path, "<html><body>HTML</body></html>").unwrap();
        std::fs::write(&css_path, "body { color: red; }").unwrap();
        std::fs::write(&js_path, "console.log('test');").unwrap();

        let opts = InjectOptions {
            directory: temp_dir.path().to_string_lossy().to_string(),
            remove: false,
            widget_url: None,
            json: true,
        };

        run(opts).unwrap();

        let css_content = std::fs::read_to_string(&css_path).unwrap();
        let js_content = std::fs::read_to_string(&js_path).unwrap();

        // CSS and JS should be unchanged
        assert!(!css_content.contains("spikes"));
        assert!(!js_content.contains("spikes"));

        // HTML should be modified
        let html_content = std::fs::read_to_string(&html_path).unwrap();
        assert!(html_content.contains("spikes.sh/widget.js"));
    }

    #[test]
    fn test_inject_htm_extension() {
        let temp_dir = TempDir::new().unwrap();
        let htm_path = temp_dir.path().join("test.htm");

        std::fs::write(&htm_path, r#"<html><body>HTM file</body></html>"#).unwrap();

        let opts = InjectOptions {
            directory: temp_dir.path().to_string_lossy().to_string(),
            remove: false,
            widget_url: None,
            json: true,
        };

        run(opts).unwrap();

        let content = std::fs::read_to_string(&htm_path).unwrap();
        assert!(content.contains("spikes.sh/widget.js"));
    }
}
