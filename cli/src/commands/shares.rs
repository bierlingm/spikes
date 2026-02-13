use std::fs;
use std::path::Path;

use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, ContentArrangement, Table};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

pub struct SharesOptions {
    pub json: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Share {
    pub id: String,
    pub slug: String,
    pub url: String,
    pub spike_count: usize,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
struct AuthConfig {
    token: String,
}

pub fn run(options: SharesOptions) -> Result<()> {
    let token = load_auth_token()?;
    let shares = fetch_shares(&token)?;

    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&shares).expect("Failed to serialize to JSON")
        );
    } else {
        print_shares_table(&shares);
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

fn fetch_shares(token: &str) -> Result<Vec<Share>> {
    let response = ureq::get("https://spikes.sh/shares")
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    if response.status() == 401 {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Authentication failed. Run 'spikes login' to re-authenticate.",
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

    let shares: Vec<Share> = serde_json::from_str(&body)?;
    Ok(shares)
}

fn print_shares_table(shares: &[Share]) {
    if shares.is_empty() {
        println!("No shares found.");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Slug", "URL", "Spikes", "Created"]);

    for share in shares {
        table.add_row(vec![
            Cell::new(&share.slug),
            Cell::new(&share.url),
            Cell::new(share.spike_count),
            Cell::new(&share.created_at),
        ]);
    }

    println!("{table}");
}
