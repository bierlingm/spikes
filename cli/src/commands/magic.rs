use crate::config::{self, Config};
use crate::error::Result;

use super::inject::InjectOptions;
use super::serve::ServeOptions;

/// Magic mode: auto-init, inject, and serve current directory
pub fn run(port: u16) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let dir_name = cwd
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "spikes".to_string());

    // Check if there are any HTML files to serve
    let has_html = std::fs::read_dir(&cwd)?
        .filter_map(|e| e.ok())
        .any(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "html" || ext == "htm")
                .unwrap_or(false)
        });

    if !has_html {
        eprintln!();
        eprintln!("  / No HTML files found in current directory");
        eprintln!();
        eprintln!("  Try one of:");
        eprintln!("    spikes serve --dir ./mockups");
        eprintln!("    spikes inject ./mockups && spikes serve");
        eprintln!();
        return Ok(());
    }

    // Auto-init if needed
    let initialized = config::ensure_initialized()?;
    if initialized {
        eprintln!("  / Initialized .spikes/");
    }

    // Load config
    let config = Config::load()?;
    
    println!();
    println!("  / Spikes — {}", dir_name);
    println!();

    // Check if widget is already injected
    let index_path = cwd.join("index.html");
    let needs_inject = if index_path.exists() {
        let content = std::fs::read_to_string(&index_path)?;
        !content.contains("spikes.js")
    } else {
        true
    };

    if needs_inject {
        // Use inject command internally
        let inject_result = super::inject::run(InjectOptions {
            directory: ".".to_string(),
            remove: false,
            widget_url: None,
            json: false,
        });

        if let Err(e) = inject_result {
            eprintln!("  Warning: Could not inject widget: {}", e);
        }
    }

    // Show config summary
    if let Some(endpoint) = config.effective_endpoint() {
        println!("  Remote:  {}", endpoint);
    } else {
        println!("  Remote:  local only (configure with: spikes remote add <url>)");
    }
    println!("  Project: {}", config.effective_project_key());
    println!();

    // Start server
    super::serve::run(ServeOptions {
        port,
        directory: ".".to_string(),
        marked: false,
    })
}
