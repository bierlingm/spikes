use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::error::{Error, Result};

pub struct LoginOptions {
    pub token: Option<String>,
    pub json: bool,
}

pub fn run(options: LoginOptions) -> Result<()> {
    let token = match options.token {
        Some(t) => t,
        None => prompt_for_token()?,
    };

    // Verify token with API
    verify_token(&token)?;

    // Save token
    save_token(&token)?;

    if options.json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "message": "Logged in successfully"
            })
        );
    } else {
        println!();
        println!("  🗡️  Logged in successfully");
        println!();
    }

    Ok(())
}

fn prompt_for_token() -> Result<String> {
    println!();
    println!("  Visit https://spikes.sh/account to get your token");
    println!();
    print!("  Token: ");
    io::stdout().flush()?;

    let mut token = String::new();
    io::stdin().read_line(&mut token)?;
    let token = token.trim().to_string();

    if token.is_empty() {
        return Err(Error::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Token cannot be empty",
        )));
    }

    Ok(token)
}

fn verify_token(token: &str) -> Result<()> {
    let response = ureq::get("https://spikes.sh/shares")
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .map_err(|e| {
            Error::Io(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to verify token: {}", e),
            ))
        })?;

    if response.status() == 401 {
        return Err(Error::Io(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Invalid token",
        )));
    }

    if response.status() != 200 {
        return Err(Error::Io(io::Error::new(
            io::ErrorKind::Other,
            format!("Server returned status {}", response.status()),
        )));
    }

    Ok(())
}

fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| {
            Error::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not determine config directory",
            ))
        })?
        .join("spikes");

    Ok(config_dir)
}

fn save_token(token: &str) -> Result<()> {
    let config_dir = get_config_dir()?;
    fs::create_dir_all(&config_dir)?;

    let auth_path = config_dir.join("auth.json");
    let auth_data = serde_json::json!({ "token": token });
    fs::write(&auth_path, serde_json::to_string_pretty(&auth_data)?)?;

    Ok(())
}

pub fn get_saved_token() -> Result<Option<String>> {
    let config_dir = get_config_dir()?;
    let auth_path = config_dir.join("auth.json");

    if !auth_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&auth_path)?;
    let auth: serde_json::Value = serde_json::from_str(&content)?;

    Ok(auth.get("token").and_then(|v| v.as_str()).map(String::from))
}
