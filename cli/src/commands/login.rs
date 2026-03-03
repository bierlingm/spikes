//! Login command - authenticate with spikes.sh via magic link or direct token

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::auth::AuthConfig;
use crate::error::{map_http_error, map_network_error, Error, Result};

pub struct LoginOptions {
    pub token: Option<String>,
    pub json: bool,
}

#[derive(Debug, Serialize)]
struct LoginRequest {
    email: String,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    #[serde(default)]
    ok: bool,
    #[serde(default)]
    message: String,
}

#[derive(Debug, Deserialize)]
struct PollResponse {
    #[serde(default)]
    verified: bool,
    #[serde(default)]
    token: Option<String>,
}

pub fn run(options: LoginOptions) -> Result<()> {
    // If token is provided directly, use it
    if let Some(token) = options.token {
        verify_and_save_token(&token, options.json)?;
        return Ok(());
    }

    // Otherwise, initiate magic link flow
    let email = prompt_for_email()?;

    // Request magic link
    request_magic_link(&email)?;

    if options.json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "message": "Check your email for a magic link",
                "email": email
            })
        );
    } else {
        println!();
        println!("  ┌────────────────────────────────────────────┐");
        println!("  │  /  Check your email                       │");
        println!("  │                                            │");
        println!("  │  We sent a magic link to:                  │");
        println!("  │  {}                            │", pad_right(&email, 36));
        println!("  │                                            │");
        println!("  │  Waiting for verification...              │");
        println!("  │  (Press Ctrl+C to cancel)                  │");
        println!("  └────────────────────────────────────────────┘");
    }

    // Poll for verification
    let token = poll_for_token(&email, options.json)?;

    // Save the token
    AuthConfig::save_token(&token)?;

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
        println!("  🗡️  Logged in successfully!");
        println!();
    }

    Ok(())
}

fn pad_right(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s.len()))
    }
}

fn prompt_for_email() -> Result<String> {
    print!("  Email: ");
    io::stdout().flush()?;

    let mut email = String::new();
    io::stdin().read_line(&mut email)?;
    let email = email.trim().to_string();

    if email.is_empty() {
        return Err(Error::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Email cannot be empty",
        )));
    }

    // Basic email validation
    if !email.contains('@') || !email.contains('.') {
        return Err(Error::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid email address",
        )));
    }

    Ok(email)
}

fn request_magic_link(email: &str) -> Result<()> {
    let response = match ureq::post("https://spikes.sh/auth/login")
        .set("Content-Type", "application/json")
        .send_json(&LoginRequest {
            email: email.to_string(),
        }) {
        Ok(resp) => resp,
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().ok();
            return Err(map_http_error(status, body.as_deref()));
        }
        Err(e) => return Err(map_network_error(&e.to_string())),
    };

    // Accept 202 (Accepted) or 200 (OK) as success
    let status = response.status();
    if status != 200 && status != 202 {
        let body = response.into_string().ok();
        return Err(map_http_error(status, body.as_deref()));
    }

    Ok(())
}

fn poll_for_token(email: &str, json: bool) -> Result<String> {
    // Poll for up to 5 minutes
    let max_attempts = 60; // 60 attempts * 5 seconds = 5 minutes
    let poll_interval = Duration::from_secs(5);

    for _attempt in 0..max_attempts {
        thread::sleep(poll_interval);

        // Try to poll the verification endpoint
        match poll_verification(email) {
            Ok(Some(token)) => return Ok(token),
            Ok(None) => {
                // Not verified yet, continue polling
                if !json {
                    print!(".");
                    io::stdout().flush().ok();
                }
                continue;
            }
            Err(e) => {
                // Log error but continue polling
                if !json {
                    eprintln!("\n  Polling error: {}", e);
                }
                continue;
            }
        }
    }

    Err(Error::Io(io::Error::new(
        io::ErrorKind::TimedOut,
        "Login timed out. Please try again.",
    )))
}

fn poll_verification(email: &str) -> Result<Option<String>> {
    // URL encode the email manually
    let encoded_email: String = email
        .chars()
        .map(|c| match c {
            '@' => "%40".to_string(),
            '.' => "%2E".to_string(),
            '+' => "%2B".to_string(),
            '-' => "%2D".to_string(),
            '_' => "%5F".to_string(),
            c if c.is_alphanumeric() => c.to_string(),
            c => format!("%{:02X}", c as u8),
        })
        .collect();

    let response = match ureq::get(&format!(
        "https://spikes.sh/auth/poll?email={}",
        encoded_email
    ))
    .call()
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().ok();
            return Err(map_http_error(status, body.as_deref()));
        }
        Err(e) => return Err(map_network_error(&e.to_string())),
    };

    let status = response.status();

    if status == 204 || status == 202 {
        // Still pending
        return Ok(None);
    }

    if status != 200 {
        let body = response.into_string().ok();
        return Err(map_http_error(status, body.as_deref()));
    }

    let body = response
        .into_string()
        .map_err(|e| Error::RequestFailed(format!("Failed to read response: {}", e)))?;

    let poll_response: PollResponse = serde_json::from_str(&body)?;

    if poll_response.verified {
        Ok(poll_response.token)
    } else {
        Ok(None)
    }
}

fn verify_and_save_token(token: &str, json: bool) -> Result<()> {
    // Verify token with API
    verify_token(token)?;

    // Save token using auth module
    AuthConfig::save_token(token)?;

    if json {
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

fn verify_token(token: &str) -> Result<()> {
    let response = match ureq::get("https://spikes.sh/shares")
        .set("Authorization", &format!("Bearer {}", token))
        .call()
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().ok();
            return Err(map_http_error(status, body.as_deref()));
        }
        Err(e) => return Err(map_network_error(&e.to_string())),
    };

    let status = response.status();

    if status == 401 {
        return Err(Error::Io(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Invalid token",
        )));
    }

    if status != 200 {
        let body = response.into_string().ok();
        return Err(map_http_error(status, body.as_deref()));
    }

    Ok(())
}

/// Get the stored token, checking SPIKES_TOKEN env var first, then auth file.
/// Returns None if not logged in.
pub fn get_stored_token() -> Result<Option<String>> {
    AuthConfig::token()
}
