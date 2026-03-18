use crate::error::{Error, Result};
use std::env::consts::{ARCH, OS};

const GITHUB_REPO: &str = "bierlingm/spikes";

pub fn run() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");

    let (latest_tag, download_url) = fetch_latest_release()?;
    let latest_version = latest_tag.trim_start_matches('v');

    if latest_version == current_version {
        println!("Already up to date (v{})", current_version);
        return Ok(());
    }

    println!("Updating spikes v{} → v{}...", current_version, latest_version);

    let target = detect_target()?;
    let asset_name = format!("spikes-{}.tar.gz", target);

    // Find matching asset URL
    let asset_url = download_url
        .ok_or_else(|| Error::RequestFailed(format!(
            "No binary asset '{}' found in release {}. \
             You can update manually: https://github.com/{}/releases/latest",
            asset_name, latest_tag, GITHUB_REPO
        )))?;

    // Download to temp dir
    let tmp_dir = std::env::temp_dir().join("spikes-update");
    std::fs::create_dir_all(&tmp_dir)
        .map_err(|e| Error::RequestFailed(format!("Failed to create temp dir: {}", e)))?;

    let tarball_path = tmp_dir.join(&asset_name);

    println!("  Downloading {}...", asset_name);
    download_file(&asset_url, &tarball_path)?;

    // Extract
    println!("  Extracting...");
    let status = std::process::Command::new("tar")
        .args(["-xzf", &tarball_path.to_string_lossy(), "-C", &tmp_dir.to_string_lossy()])
        .status()
        .map_err(|e| Error::RequestFailed(format!("Failed to extract: {}", e)))?;

    if !status.success() {
        return Err(Error::RequestFailed("Failed to extract tarball".to_string()));
    }

    // Replace current binary
    let new_binary = tmp_dir.join("spikes");
    let current_binary = std::env::current_exe()
        .map_err(|e| Error::RequestFailed(format!("Cannot determine current binary path: {}", e)))?;

    std::fs::copy(&new_binary, &current_binary)
        .map_err(|e| Error::RequestFailed(format!("Failed to replace binary: {}", e)))?;

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp_dir);

    println!("Updated to v{}", latest_version);
    Ok(())
}

fn detect_target() -> Result<String> {
    let arch = match ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => return Err(Error::RequestFailed(format!("Unsupported architecture: {}", ARCH))),
    };

    let os = match OS {
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-gnu",
        _ => return Err(Error::RequestFailed(format!("Unsupported OS: {}", OS))),
    };

    Ok(format!("{}-{}", arch, os))
}

fn fetch_latest_release() -> Result<(String, Option<String>)> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO);
    let target = detect_target()?;
    let asset_name = format!("spikes-{}.tar.gz", target);

    let response: serde_json::Value = ureq::get(&url)
        .set("User-Agent", &format!("spikes/{} (self-update)", env!("CARGO_PKG_VERSION")))
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| Error::RequestFailed(format!("Failed to check GitHub releases: {}", e)))?
        .into_json()
        .map_err(|e| Error::RequestFailed(format!("Failed to parse GitHub response: {}", e)))?;

    let tag = response["tag_name"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| Error::RequestFailed("Could not find tag_name in GitHub response".to_string()))?;

    let download_url = response["assets"]
        .as_array()
        .and_then(|assets| {
            assets.iter().find_map(|a| {
                let name = a["name"].as_str()?;
                if name == asset_name {
                    a["browser_download_url"].as_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
        });

    Ok((tag, download_url))
}

fn download_file(url: &str, path: &std::path::Path) -> Result<()> {
    let response = ureq::get(url)
        .set("User-Agent", &format!("spikes/{} (self-update)", env!("CARGO_PKG_VERSION")))
        .call()
        .map_err(|e| Error::RequestFailed(format!("Download failed: {}", e)))?;

    let mut reader = response.into_reader();
    let mut file = std::fs::File::create(path)
        .map_err(|e| Error::RequestFailed(format!("Failed to create file: {}", e)))?;

    std::io::copy(&mut reader, &mut file)
        .map_err(|e| Error::RequestFailed(format!("Failed to write download: {}", e)))?;

    Ok(())
}
