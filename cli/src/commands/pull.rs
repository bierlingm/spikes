use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::auth::get_api_base;
use crate::error::{map_http_error, map_network_error, Error, Result};
use crate::spike::{PaginatedResponse, Spike};

pub struct PullOptions {
    pub endpoint: Option<String>,
    pub token: Option<String>,
    pub from: Option<String>,
    pub json: bool,
}

struct RemoteConfig {
    endpoint: String,
    token: String,
}

pub fn run(options: PullOptions) -> Result<()> {
    // If --from URL is provided, fetch from public share
    if let Some(ref url) = options.from {
        return run_from_share(url, options.json);
    }

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
        (Some(endpoint), Some(token)) => {
            if endpoint.ends_with("/spikes") {
                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Endpoint should be the base URL (e.g. https://spikes.sh), not the /spikes path",
                )));
            }
            Ok(RemoteConfig { endpoint, token })
        }
        (None, _) => Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No remote endpoint. Add [remote] endpoint = \"...\" to .spikes/config.toml",
        ))),
        (_, None) => Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No token. Add token = \"...\" under [remote] in .spikes/config.toml",
        ))),
    }
}

fn fetch_remote_spikes(config: &RemoteConfig) -> Result<Vec<Spike>> {
    let url = format!("{}/spikes", config.endpoint.trim_end_matches('/'));

    // Use ureq for synchronous HTTP (simpler than async for CLI)
    let response = match ureq::get(&url)
        .set("Authorization", &format!("Bearer {}", config.token))
        .call()
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(status, response)) => {
            // Non-2xx response - try to read body for error details
            let body = response.into_string().ok();
            return Err(map_http_error(status, body.as_deref()));
        }
        Err(e) => return Err(map_network_error(&e.to_string())),
    };

    let status = response.status();

    if status != 200 {
        let body = response.into_string().ok();
        return Err(map_http_error(status, body.as_deref()));
    }

    let body = response
        .into_string()
        .map_err(|e| Error::RequestFailed(format!("Failed to read response: {}", e)))?;

    if body.starts_with('<') {
        return Err(Error::RequestFailed(
            "Got HTML instead of JSON. Check that the endpoint URL is correct.".to_string()
        ));
    }

    // API returns paginated response: { data: [...spikes], next_cursor: string|null }
    let response: PaginatedResponse<Spike> = serde_json::from_str(&body)?;
    Ok(response.data)
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

fn run_from_share(url: &str, json_output: bool) -> Result<()> {
    // Parse share slug from URL (e.g., https://spikes.sh/s/governance-x7k2m)
    let share_id = parse_share_slug(url)?;

    // Fetch spikes from public endpoint (respects SPIKES_API_URL env var)
    let api_url = format!("{}/spikes?project={}", get_api_base(), share_id);

    let response = match ureq::get(&api_url).call() {
        Ok(resp) => resp,
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().ok();
            return Err(map_http_error(status, body.as_deref()));
        }
        Err(e) => return Err(map_network_error(&e.to_string())),
    };

    let status = response.status();

    if status != 200 {
        let body = response.into_string().ok();
        return Err(map_http_error(status, body.as_deref()));
    }

    let body = response
        .into_string()
        .map_err(|e| Error::RequestFailed(format!("Failed to read response: {}", e)))?;

    // API returns paginated response: { data: [...spikes], next_cursor: string|null }
    let response: PaginatedResponse<Spike> = serde_json::from_str(&body)?;
    let remote_spikes = response.data;

    // Load local spikes and merge
    let feedback_path = Path::new(".spikes/feedback.jsonl");
    let local_spikes = load_local_spikes(feedback_path)?;

    let existing_ids: HashSet<String> = local_spikes.iter().map(|s| s.id.clone()).collect();

    let new_spikes: Vec<&Spike> = remote_spikes
        .iter()
        .filter(|s| !existing_ids.contains(&s.id))
        .collect();

    let new_count = new_spikes.len();

    if !new_spikes.is_empty() {
        // Ensure .spikes directory exists
        if let Some(parent) = feedback_path.parent() {
            fs::create_dir_all(parent)?;
        }

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

    if json_output {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "source": url,
                "share_id": share_id,
                "fetched": remote_spikes.len(),
                "new": new_count,
                "existing": local_spikes.len(),
                "total": local_spikes.len() + new_count
            })
        );
    } else {
        println!();
        println!("  🗡️  Pulled from share");
        println!();
        println!("  Source:         {}", url);
        println!("  Remote spikes:  {}", remote_spikes.len());
        println!("  New spikes:     {}", new_count);
        println!("  Local total:    {}", local_spikes.len() + new_count);
        println!();
    }

    Ok(())
}

fn parse_share_slug(url: &str) -> Result<String> {
    // Handle both full URLs and bare slugs
    // Full URL: https://spikes.sh/s/governance-x7k2m
    // Bare slug: governance-x7k2m

    if url.starts_with("http://") || url.starts_with("https://") {
        // Parse URL to extract slug after /s/
        if let Some(pos) = url.find("/s/") {
            let slug = &url[pos + 3..];
            // Remove any trailing slashes or query params
            let slug = slug.split(&['/', '?', '#'][..]).next().unwrap_or(slug);
            if slug.is_empty() {
                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid share URL: no slug found after /s/",
                )));
            }
            return Ok(slug.to_string());
        }
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid share URL: expected format https://spikes.sh/s/<slug>",
        )));
    }

    // Assume it's a bare slug
    if url.is_empty() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Share slug cannot be empty",
        )));
    }

    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    const SPIKES_API_URL_ENV: &str = "SPIKES_API_URL";

    #[test]
    fn test_parse_share_slug_from_url() {
        let slug = parse_share_slug("https://spikes.sh/s/governance-x7k2m").unwrap();
        assert_eq!(slug, "governance-x7k2m");
    }

    #[test]
    fn test_parse_share_slug_from_url_with_trailing_slash() {
        let slug = parse_share_slug("https://spikes.sh/s/governance-x7k2m/").unwrap();
        assert_eq!(slug, "governance-x7k2m");
    }

    #[test]
    fn test_parse_share_slug_from_url_with_query() {
        let slug = parse_share_slug("https://spikes.sh/s/governance-x7k2m?foo=bar").unwrap();
        assert_eq!(slug, "governance-x7k2m");
    }

    #[test]
    fn test_parse_share_slug_bare() {
        let slug = parse_share_slug("governance-x7k2m").unwrap();
        assert_eq!(slug, "governance-x7k2m");
    }

    #[test]
    fn test_parse_share_slug_invalid_url() {
        let result = parse_share_slug("https://spikes.sh/invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_share_slug_empty() {
        let result = parse_share_slug("");
        assert!(result.is_err());
    }

    #[test]
    #[serial(api_url)]
    fn test_pull_from_share_uses_api_base_default() {
        // Save current value
        let original = std::env::var(SPIKES_API_URL_ENV).ok();

        // Clear env var
        std::env::remove_var(SPIKES_API_URL_ENV);

        // Verify get_api_base returns default
        let base = get_api_base();
        assert_eq!(base, "https://spikes.sh");

        // Restore original value
        if let Some(val) = original {
            std::env::set_var(SPIKES_API_URL_ENV, val);
        }
    }

    #[test]
    #[serial(api_url)]
    fn test_pull_from_share_uses_api_base_env_override() {
        // Save current value
        let original = std::env::var(SPIKES_API_URL_ENV).ok();

        // Set custom API URL (e.g., for local dev with wrangler)
        std::env::set_var(SPIKES_API_URL_ENV, "http://localhost:8787");

        // Verify get_api_base returns env var value
        let base = get_api_base();
        assert_eq!(base, "http://localhost:8787");

        // Simulate URL construction for pull --from
        let share_id = "test-share-123";
        let api_url = format!("{}/spikes?project={}", get_api_base(), share_id);
        assert_eq!(api_url, "http://localhost:8787/spikes?project=test-share-123");

        // Restore original value
        if let Some(val) = original {
            std::env::set_var(SPIKES_API_URL_ENV, val);
        } else {
            std::env::remove_var(SPIKES_API_URL_ENV);
        }
    }

    #[test]
    #[serial(api_url)]
    fn test_pull_from_share_url_construction_with_custom_host() {
        // Save current value
        let original = std::env::var(SPIKES_API_URL_ENV).ok();

        // Set custom self-hosted API URL
        std::env::set_var(SPIKES_API_URL_ENV, "https://spikes.example.com");

        // Verify URL construction uses custom host
        let share_id = "my-project-abc";
        let api_url = format!("{}/spikes?project={}", get_api_base(), share_id);
        assert_eq!(api_url, "https://spikes.example.com/spikes?project=my-project-abc");

        // Restore original value
        if let Some(val) = original {
            std::env::set_var(SPIKES_API_URL_ENV, val);
        } else {
            std::env::remove_var(SPIKES_API_URL_ENV);
        }
    }
}
