use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::spike::Spike;

pub struct UnshareOptions {
    pub slug: String,
    pub force: bool,
    pub json: bool,
}

#[derive(Debug, Deserialize)]
struct AuthConfig {
    token: String,
}

#[derive(Debug, Deserialize)]
struct ShareInfo {
    id: String,
    #[allow(dead_code)]
    slug: String,
    #[serde(default)]
    exported_spikes: Vec<Spike>,
}

#[derive(Debug, Serialize)]
struct UnshareResult {
    success: bool,
    slug: String,
    spikes_saved: usize,
    backup_path: Option<String>,
}

pub fn run(options: UnshareOptions) -> Result<()> {
    let token = load_auth_token()?;

    // Get share info first to retrieve the ID and spikes
    let share_info = fetch_share_info(&token, &options.slug)?;

    // Confirm unless --force
    if !options.force && !options.json {
        print!(
            "Delete share '{}'? This cannot be undone. [y/N] ",
            options.slug
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Delete the share
    delete_share(&token, &share_info.id)?;

    // Save exported spikes to .spikes/{slug}.jsonl
    let backup_path = save_exported_spikes(&options.slug, &share_info.exported_spikes)?;

    let result = UnshareResult {
        success: true,
        slug: options.slug.clone(),
        spikes_saved: share_info.exported_spikes.len(),
        backup_path: backup_path.clone(),
    };

    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&result).expect("Failed to serialize to JSON")
        );
    } else {
        println!();
        println!("  🗡️  Share deleted");
        println!();
        println!("  Slug:           {}", options.slug);
        println!("  Spikes saved:   {}", share_info.exported_spikes.len());
        if let Some(path) = backup_path {
            println!("  Backup:         {}", path);
        }
        println!();
    }

    Ok(())
}

fn load_auth_token() -> Result<String> {
    let home = std::env::var("HOME").map_err(|_| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "HOME environment variable not set",
        ))
    })?;

    let auth_path = Path::new(&home).join(".config/spikes/auth.json");

    if !auth_path.exists() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Not logged in. Run 'spikes login' first.\n\
             Expected auth file: ~/.config/spikes/auth.json",
        )));
    }

    let content = fs::read_to_string(&auth_path)?;
    let auth: AuthConfig = serde_json::from_str(&content).map_err(|e| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid auth.json: {}", e),
        ))
    })?;

    Ok(auth.token)
}

fn fetch_share_info(token: &str, slug: &str) -> Result<ShareInfo> {
    let url = format!("https://spikes.sh/shares/{}", slug);

    let response = ureq::get(&url)
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    if response.status() == 401 {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Authentication failed. Run 'spikes login' to re-authenticate.",
        )));
    }

    if response.status() == 404 {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Share '{}' not found", slug),
        )));
    }

    if response.status() != 200 {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Server returned status {}", response.status()),
        )));
    }

    let body = response
        .into_string()
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    let share_info: ShareInfo = serde_json::from_str(&body)?;
    Ok(share_info)
}

fn delete_share(token: &str, id: &str) -> Result<()> {
    let url = format!("https://spikes.sh/shares/{}", id);

    let response = ureq::delete(&url)
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    if response.status() == 401 {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Authentication failed. Run 'spikes login' to re-authenticate.",
        )));
    }

    if response.status() != 200 && response.status() != 204 {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to delete share: status {}", response.status()),
        )));
    }

    Ok(())
}

fn save_exported_spikes(slug: &str, spikes: &[Spike]) -> Result<Option<String>> {
    if spikes.is_empty() {
        return Ok(None);
    }

    let spikes_dir = Path::new(".spikes");
    if !spikes_dir.exists() {
        fs::create_dir_all(spikes_dir)?;
    }

    let backup_path = spikes_dir.join(format!("{}.jsonl", slug));

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&backup_path)?;

    for spike in spikes {
        let mut json = serde_json::to_string(spike)?;
        json.push('\n');
        file.write_all(json.as_bytes())?;
    }

    Ok(Some(backup_path.to_string_lossy().to_string()))
}
