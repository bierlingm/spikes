use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::error::{Error, Result};
use crate::spike::Spike;

pub struct PullOptions {
    pub endpoint: Option<String>,
    pub token: Option<String>,
    pub json: bool,
}

struct RemoteConfig {
    endpoint: String,
    token: String,
}

pub fn run(options: PullOptions) -> Result<()> {
    let config = get_remote_config(options.endpoint, options.token)?;

    // Fetch remote spikes
    let remote_spikes = fetch_remote_spikes(&config)?;

    // Load local spikes
    let feedback_path = Path::new(".spikes/feedback.jsonl");
    let local_spikes = load_local_spikes(feedback_path)?;

    // Build set of existing IDs
    let existing_ids: HashSet<String> = local_spikes.iter().map(|s| s.id.clone()).collect();

    // Find new spikes
    let new_spikes: Vec<&Spike> = remote_spikes
        .iter()
        .filter(|s| !existing_ids.contains(&s.id))
        .collect();

    let new_count = new_spikes.len();

    // Append new spikes to local file
    if !new_spikes.is_empty() {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(feedback_path)?;

        for spike in &new_spikes {
            let mut json = serde_json::to_string(spike)?;
            json.push('\n');
            file.write_all(json.as_bytes())?;
        }
    }

    if options.json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "fetched": remote_spikes.len(),
                "new": new_count,
                "existing": local_spikes.len(),
                "total": local_spikes.len() + new_count
            })
        );
    } else {
        println!();
        println!("  🗡️  Pulled from remote");
        println!();
        println!("  Remote spikes:  {}", remote_spikes.len());
        println!("  New spikes:     {}", new_count);
        println!("  Local total:    {}", local_spikes.len() + new_count);
        println!();
    }

    Ok(())
}

fn get_remote_config(
    endpoint_arg: Option<String>,
    token_arg: Option<String>,
) -> Result<RemoteConfig> {
    // Try command-line args first
    if let (Some(endpoint), Some(token)) = (endpoint_arg.clone(), token_arg.clone()) {
        return Ok(RemoteConfig { endpoint, token });
    }

    // Fall back to config file
    let config_path = Path::new(".spikes/config.toml");
    if !config_path.exists() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No endpoint specified and .spikes/config.toml not found.\n\
             Use --endpoint and --token, or configure in .spikes/config.toml:\n\n\
             [remote]\n\
             endpoint = \"https://your-worker.workers.dev\"\n\
             token = \"your-token\"",
        )));
    }

    let content = fs::read_to_string(config_path)?;
    let config: toml::Value = content.parse().map_err(|e: toml::de::Error| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid config.toml: {}", e),
        ))
    })?;

    let endpoint = endpoint_arg.or_else(|| {
        config
            .get("remote")
            .and_then(|r| r.get("endpoint"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    });

    let token = token_arg.or_else(|| {
        config
            .get("remote")
            .and_then(|r| r.get("token"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    });

    match (endpoint, token) {
        (Some(endpoint), Some(token)) => Ok(RemoteConfig { endpoint, token }),
        (None, _) => Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No endpoint configured. Use --endpoint or set [remote].endpoint in .spikes/config.toml",
        ))),
        (_, None) => Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No token configured. Use --token or set [remote].token in .spikes/config.toml",
        ))),
    }
}

fn fetch_remote_spikes(config: &RemoteConfig) -> Result<Vec<Spike>> {
    let url = format!("{}/spikes?token={}", config.endpoint.trim_end_matches('/'), config.token);

    // Use ureq for synchronous HTTP (simpler than async for CLI)
    let response = ureq::get(&url)
        .call()
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    if response.status() == 401 {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Authentication failed. Check your token.",
        )));
    }

    if response.status() != 200 {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Remote returned status {}", response.status()),
        )));
    }

    let body = response
        .into_string()
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    let spikes: Vec<Spike> = serde_json::from_str(&body)?;
    Ok(spikes)
}

fn load_local_spikes(path: &Path) -> Result<Vec<Spike>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut spikes = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(spike) = serde_json::from_str(&line) {
            spikes.push(spike);
        }
    }

    Ok(spikes)
}
