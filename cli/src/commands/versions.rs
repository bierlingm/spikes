//! `spikes versions add|list|notes` — declare review versions for a project.

use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, ContentArrangement, Table};

use crate::api::{data_array, encode, require_project_key, ApiClient};
use crate::error::{Error, Result};
use crate::output::print_json;

pub fn add(label: &str, prefix: &str, notes: Option<String>, json: bool) -> Result<()> {
    if label.trim().is_empty() {
        return Err(Error::RequestFailed(
            "Version label cannot be empty".to_string(),
        ));
    }
    let project = require_project_key()?;
    let client = ApiClient::from_env()?;

    let mut body = serde_json::json!({ "label": label, "url_prefix": prefix });
    if let Some(ref n) = notes {
        body["notes"] = serde_json::Value::String(n.clone());
    }

    let response = client.post(
        &format!("/me/projects/{}/versions", encode(&project)),
        &body,
    )?;

    if json {
        print_json(&serde_json::json!({ "success": true, "version": response }));
    } else {
        println!();
        println!("  / Version added to '{}'", project);
        println!();
        println!("  Label:   {}", label);
        println!("  Prefix:  {}", prefix);
        if let Some(ref n) = notes {
            println!("  Notes:   {}", n);
        }
        println!();
    }
    Ok(())
}

pub fn list(json: bool) -> Result<()> {
    let project = require_project_key()?;
    let client = ApiClient::from_env()?;
    let response = client.get(&format!("/me/projects/{}/versions", encode(&project)))?;
    let versions = data_array(&response);

    if json {
        print_json(&versions);
        return Ok(());
    }

    if versions.is_empty() {
        println!();
        println!("  No versions for '{}'. Add one with: spikes versions add <label> --prefix <url-prefix>", project);
        println!();
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Label", "URL prefix", "Spikes", "Open", "Notes"]);

    for v in &versions {
        let get_str = |k: &str, alt: &str| {
            v.get(k)
                .or_else(|| v.get(alt))
                .and_then(|x| x.as_str())
                .unwrap_or("-")
                .to_string()
        };
        let get_num = |k: &str, alt: &str| {
            v.get(k)
                .or_else(|| v.get(alt))
                .map(|x| x.to_string())
                .unwrap_or_else(|| "-".to_string())
        };
        table.add_row(vec![
            Cell::new(get_str("label", "label")),
            Cell::new(get_str("urlPrefix", "url_prefix")),
            Cell::new(get_num("spikeCount", "spike_count")),
            Cell::new(get_num("openCount", "open_count")),
            Cell::new(get_str("notes", "notes")),
        ]);
    }

    println!();
    println!("{}", table);
    println!();
    Ok(())
}

pub fn notes(label: &str, text: &str, json: bool) -> Result<()> {
    let project = require_project_key()?;
    let client = ApiClient::from_env()?;
    let body = serde_json::json!({ "notes": text });
    let response = client.patch(
        &format!(
            "/me/projects/{}/versions/{}",
            encode(&project),
            encode(label)
        ),
        &body,
    )?;

    if json {
        print_json(&serde_json::json!({ "success": true, "version": response }));
    } else {
        println!();
        println!("  / Notes updated for version '{}'", label);
        println!();
    }
    Ok(())
}
