//! `spikes projects create|list` — manage hosted projects.

use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, ContentArrangement, Table};

use crate::api::{data_array, ApiClient};
use crate::config::Config;
use crate::error::Result;
use crate::output::print_json;

/// Create a project. Writes `[project].key` into `.spikes/config.toml` when
/// a `.spikes/` directory exists and no key is configured yet.
pub fn create(key: &str, origins: Vec<String>, json: bool) -> Result<()> {
    let client = ApiClient::from_env()?;

    let mut body = serde_json::json!({ "key": key });
    if !origins.is_empty() {
        body["allowed_origins"] = serde_json::Value::Array(
            origins
                .iter()
                .map(|o| serde_json::Value::String(o.clone()))
                .collect(),
        );
    }

    let response = client.post("/projects", &body)?;

    let mut saved_to_config = false;
    if std::path::Path::new(".spikes").exists() {
        let mut config = Config::load()?;
        if config.project.key.as_deref().unwrap_or("").is_empty() {
            config.project.key = Some(key.to_string());
            config.save()?;
            saved_to_config = true;
        }
    }

    if json {
        print_json(&serde_json::json!({
            "success": true,
            "project": response,
            "saved_to_config": saved_to_config,
        }));
    } else {
        let origins_display = response
            .get("allowed_origins")
            .or_else(|| {
                response
                    .get("project")
                    .and_then(|p| p.get("allowed_origins"))
            })
            .map(|v| v.to_string())
            .unwrap_or_else(|| {
                if origins.is_empty() {
                    "(default)".to_string()
                } else {
                    format!("{:?}", origins)
                }
            });
        println!();
        println!("  / Project created");
        println!();
        println!("  Key:      {}", key);
        println!("  Origins:  {}", origins_display);
        if saved_to_config {
            println!("  Config:   [project].key set in .spikes/config.toml");
        }
        println!();
        println!("  Next: spikes auth create-key --project {} --save", key);
        println!();
    }

    Ok(())
}

/// List the caller's projects.
pub fn list(json: bool) -> Result<()> {
    let client = ApiClient::from_env()?;
    let response = client.get("/me/projects")?;
    let projects = data_array(&response);

    if json {
        print_json(&projects);
        return Ok(());
    }

    if projects.is_empty() {
        println!();
        println!("  No projects yet. Create one with: spikes projects create <key>");
        println!();
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Key", "Spikes", "Last activity", "Origins"]);

    for p in &projects {
        let key = p.get("key").and_then(|v| v.as_str()).unwrap_or("-");
        let spikes = p
            .get("spike_count")
            .or_else(|| p.get("spikeCount"))
            .map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string());
        let last = p
            .get("last_activity")
            .or_else(|| p.get("lastActivity"))
            .and_then(|v| v.as_str())
            .map(|s| s.chars().take(10).collect::<String>())
            .unwrap_or_else(|| "-".to_string());
        let origins = p
            .get("allowed_origins")
            .or_else(|| p.get("allowedOrigins"))
            .map(|v| match v {
                serde_json::Value::Array(a) => a
                    .iter()
                    .filter_map(|o| o.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .unwrap_or_else(|| "-".to_string());
        table.add_row(vec![
            Cell::new(key),
            Cell::new(spikes),
            Cell::new(last),
            Cell::new(origins),
        ]);
    }

    println!();
    println!("{}", table);
    println!();
    Ok(())
}
