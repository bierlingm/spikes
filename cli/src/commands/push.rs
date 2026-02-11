use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::error::{Error, Result};
use crate::spike::Spike;

pub struct PushOptions {
    pub endpoint: Option<String>,
    pub token: Option<String>,
    pub json: bool,
}

struct RemoteConfig {
    endpoint: String,
    token: String,
}

pub fn run(options: PushOptions) -> Result<()> {
    let config = get_remote_config(options.endpoint, options.token)?;

    // Load local spikes
    let feedback_path = Path::new(".spikes/feedback.jsonl");
    let local_spikes = load_local_spikes(feedback_path)?;

    if local_spikes.is_empty() {
        if options.json {
            println!(
                "{}",
                serde_json::json!({
                    "success": true,
                    "pushed": 0,
                    "message": "No local spikes to push"
                })
            );
        } else {
            println!("No local spikes to push");
        }
        return Ok(());
    }

    // Fetch existing remote spike IDs
    let remote_ids = fetch_remote_spike_ids(&config)?;

    // Find spikes that don't exist remotely
    let new_spikes: Vec<&Spike> = local_spikes
        .iter()
        .filter(|s| !remote_ids.contains(&s.id))
        .collect();

    let new_count = new_spikes.len();

    // Push each new spike
    let mut success_count = 0;
    let mut error_count = 0;

    for spike in &new_spikes {
        match push_spike(&config, spike) {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                if !options.json {
                    eprintln!("  Failed to push spike {}: {}", spike.id, e);
                }
            }
        }
    }

    if options.json {
        println!(
            "{}",
            serde_json::json!({
                "success": error_count == 0,
                "local": local_spikes.len(),
                "new": new_count,
                "pushed": success_count,
                "errors": error_count
            })
        );
    } else {
        println!();
        println!("  🗡️  Pushed to remote");
        println!();
        println!("  Local spikes:   {}", local_spikes.len());
        println!("  New to push:    {}", new_count);
        println!("  Pushed:         {}", success_count);
        if error_count > 0 {
            println!("  Errors:         {}", error_count);
        }
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

fn fetch_remote_spike_ids(config: &RemoteConfig) -> Result<std::collections::HashSet<String>> {
    let url = format!(
        "{}/spikes?token={}",
        config.endpoint.trim_end_matches('/'),
        config.token
    );

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
    Ok(spikes.into_iter().map(|s| s.id).collect())
}

fn push_spike(config: &RemoteConfig, spike: &Spike) -> Result<()> {
    let url = format!(
        "{}/spikes?token={}",
        config.endpoint.trim_end_matches('/'),
        config.token
    );

    let body = serde_json::to_string(spike)?;

    let response = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string(&body)
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    if response.status() == 401 {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Authentication failed. Check your token.",
        )));
    }

    if response.status() != 201 && response.status() != 200 {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Remote returned status {}", response.status()),
        )));
    }

    Ok(())
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
