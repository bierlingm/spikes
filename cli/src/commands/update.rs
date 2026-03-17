use crate::error::{Error, Result};
use std::process::Command;

pub fn run() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");

    // Fetch latest version from crates.io
    let latest_version = fetch_latest_version()?;

    if latest_version == current_version {
        println!("Already up to date (v{})", current_version);
        return Ok(());
    }

    println!("Updating spikes v{} → v{}...", current_version, latest_version);

    let status = Command::new("cargo")
        .args(["install", "spikes", "--force"])
        .status()
        .map_err(|e| Error::RequestFailed(format!("Failed to run cargo install: {}", e)))?;

    if !status.success() {
        return Err(Error::RequestFailed(format!(
            "cargo install failed with exit code {}",
            status.code().unwrap_or(-1)
        )));
    }

    println!("Updated to v{}", latest_version);
    Ok(())
}

fn fetch_latest_version() -> Result<String> {
    let response: serde_json::Value = ureq::get("https://crates.io/api/v1/crates/spikes")
        .set("User-Agent", &format!("spikes/{} (self-update)", env!("CARGO_PKG_VERSION")))
        .call()
        .map_err(|e| Error::RequestFailed(format!("Failed to check crates.io: {}", e)))?
        .into_json()
        .map_err(|e| Error::RequestFailed(format!("Failed to parse crates.io response: {}", e)))?;

    response["crate"]["max_version"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| Error::RequestFailed("Could not find version in crates.io response".to_string()))
}
