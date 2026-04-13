//! Login command - authenticate with spikes.sh via browser device flow or direct token

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::auth::{get_api_base, AuthConfig};
use crate::error::{map_http_error, map_network_error, Error, Result};

pub struct LoginOptions {
    pub token: Option<String>,
    pub email: bool,
    pub json: bool,
}

#[derive(Debug, Serialize)]
struct LoginRequest {
    email: String,
}

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_url: String,
    #[serde(default = "default_expires_in")]
    expires_in: u64,
    #[serde(default = "default_interval")]
    interval: u64,
}

fn default_expires_in() -> u64 {
    600
}
fn default_interval() -> u64 {
    5
}

#[derive(Debug, Deserialize)]
struct DevicePollResponse {
    #[serde(default)]
    status: String,
    #[serde(default)]
    token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PollResponse {
    #[serde(default)]
    verified: bool,
    #[serde(default)]
    token: Option<String>,
}

pub fn run(options: LoginOptions) -> Result<()> {
    // Direct token: verify and save
    if let Some(token) = options.token {
        verify_and_save_token(&token, options.json)?;
        return Ok(());
    }

    // Email magic link flow (legacy / fallback)
    if options.email {
        return run_email_flow(options.json);
    }

    // Default: browser device code flow
    run_device_flow(options.json)
}

// --- Device code flow (default) ---

fn run_device_flow(json: bool) -> Result<()> {
    let device = request_device_code()?;

    let url_with_code = if device.verification_url.contains('?') {
        format!("{}&code={}", device.verification_url, device.user_code)
    } else {
        format!("{}?code={}", device.verification_url, device.user_code)
    };

    if !json {
        println!();
        println!("  \x1b[1m\x1b[31m/\x1b[0m \x1b[1mspikes.sh\x1b[0m");
        println!();
        print!("  Opening browser");
        io::stdout().flush().ok();
    }

    let browser_ok = webbrowser::open(&url_with_code).is_ok();

    if !json {
        if browser_ok {
            println!(" \x1b[2m— confirm in your browser to continue\x1b[0m");
        } else {
            println!();
            println!("  \x1b[2mOpen this URL to confirm:\x1b[0m");
            println!("  {}", url_with_code);
        }
        println!();
        print!("  \x1b[2mWaiting\x1b[0m");
        io::stdout().flush().ok();
    }

    let token = poll_device_code(&device.device_code, device.expires_in, device.interval, json)?;

    AuthConfig::save_token(&token)?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "message": "Logged in successfully"
            })
        );
    } else {
        // Clear the "Waiting..." line
        print!("\r\x1b[K");
        println!("  \x1b[32m✓\x1b[0m Logged in");
        println!();
    }

    Ok(())
}

fn request_device_code() -> Result<DeviceCodeResponse> {
    let api_base = get_api_base();
    let url = format!("{}/auth/device", api_base.trim_end_matches('/'));

    let response = match ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string("{}")
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().ok();
            return Err(map_http_error(status, body.as_deref()));
        }
        Err(e) => return Err(map_network_error(&e.to_string())),
    };

    let body = response
        .into_string()
        .map_err(|e| Error::RequestFailed(format!("Failed to read response: {}", e)))?;

    serde_json::from_str(&body).map_err(|e| {
        Error::RequestFailed(format!("Invalid device code response: {}", e))
    })
}

fn poll_device_code(device_code: &str, expires_in: u64, interval: u64, json: bool) -> Result<String> {
    let api_base = get_api_base();
    let url = format!(
        "{}/auth/device/poll?device_code={}",
        api_base.trim_end_matches('/'),
        device_code
    );

    let max_attempts = expires_in / interval;
    let poll_interval = Duration::from_secs(interval);
    let spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let mut tick: usize = 0;

    for _attempt in 0..max_attempts {
        thread::sleep(poll_interval);

        if !json {
            print!("\r  \x1b[2m{} Waiting\x1b[0m", spinner[tick % spinner.len()]);
            io::stdout().flush().ok();
            tick += 1;
        }

        let response = match ureq::get(&url).call() {
            Ok(resp) => resp,
            Err(ureq::Error::Status(status, _response)) => {
                if status == 428 {
                    continue;
                }
                let body = _response.into_string().ok();
                return Err(map_http_error(status, body.as_deref()));
            }
            Err(_) => {
                continue;
            }
        };

        let status = response.status();

        if status == 202 || status == 204 {
            continue;
        }

        if status != 200 {
            let body = response.into_string().ok();
            return Err(map_http_error(status, body.as_deref()));
        }

        let body = response
            .into_string()
            .map_err(|e| Error::RequestFailed(format!("Failed to read response: {}", e)))?;

        let poll: DevicePollResponse = serde_json::from_str(&body)?;

        if poll.status == "complete" || poll.status == "verified" {
            if let Some(token) = poll.token {
                return Ok(token);
            }
        }

        if !json {
            print!(".");
            io::stdout().flush().ok();
        }
    }

    Err(Error::Io(io::Error::new(
        io::ErrorKind::TimedOut,
        "Login timed out. Please try again.",
    )))
}

// --- Email magic link flow (fallback) ---

fn run_email_flow(json: bool) -> Result<()> {
    let email = prompt_for_email()?;

    request_magic_link(&email)?;

    if json {
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

    let token = poll_for_email_token(&email, json)?;
    AuthConfig::save_token(&token)?;

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
        println!("  /  Logged in successfully!");
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

    if !email.contains('@') || !email.contains('.') {
        return Err(Error::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid email address",
        )));
    }

    Ok(email)
}

fn request_magic_link(email: &str) -> Result<()> {
    let api_base = get_api_base();
    let url = format!("{}/auth/login", api_base.trim_end_matches('/'));

    let response = match ureq::post(&url)
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

    let status = response.status();
    if status != 200 && status != 202 {
        let body = response.into_string().ok();
        return Err(map_http_error(status, body.as_deref()));
    }

    Ok(())
}

fn poll_for_email_token(email: &str, json: bool) -> Result<String> {
    let max_attempts = 60;
    let poll_interval = Duration::from_secs(5);

    for _attempt in 0..max_attempts {
        thread::sleep(poll_interval);

        match poll_email_verification(email) {
            Ok(Some(token)) => return Ok(token),
            Ok(None) => {
                if !json {
                    print!(".");
                    io::stdout().flush().ok();
                }
                continue;
            }
            Err(e) => {
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

fn poll_email_verification(email: &str) -> Result<Option<String>> {
    let api_base = get_api_base();

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

    let url = format!(
        "{}/auth/poll?email={}",
        api_base.trim_end_matches('/'),
        encoded_email
    );

    let response = match ureq::get(&url).call() {
        Ok(resp) => resp,
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().ok();
            return Err(map_http_error(status, body.as_deref()));
        }
        Err(e) => return Err(map_network_error(&e.to_string())),
    };

    let status = response.status();

    if status == 204 || status == 202 {
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
    verify_token(token)?;
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
        println!("  /  Logged in successfully");
        println!();
    }

    Ok(())
}

fn verify_token(token: &str) -> Result<()> {
    let api_base = get_api_base();
    let url = format!("{}/shares", api_base.trim_end_matches('/'));

    let response = match ureq::get(&url)
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
