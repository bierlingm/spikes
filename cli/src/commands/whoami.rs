//! Whoami command - show current user identity

use serde::Deserialize;

use crate::auth::{get_api_base, AuthConfig};
use crate::error::{map_http_error, map_network_error, Error, Result};

#[derive(Debug, Deserialize)]
struct UserInfo {
    email: String,
    tier: String,
}

pub fn run(json: bool) -> Result<()> {
    // Check if user has a token
    let token = AuthConfig::token()?
        .ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Not logged in. Run 'spikes login' first.",
            ))
        })?;

    // Call /me endpoint to get user info
    let user_info = fetch_user_info(&token)?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "email": user_info.email,
                "tier": user_info.tier
            })
        );
    } else {
        println!();
        println!("  Email: {}", user_info.email);
        println!("  Tier:  {}", user_info.tier);
        println!();
    }

    Ok(())
}

fn fetch_user_info(token: &str) -> Result<UserInfo> {
    let api_base = get_api_base();
    let url = format!("{}/me", api_base.trim_end_matches('/'));

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

    if status != 200 {
        let body = response.into_string().ok();
        return Err(map_http_error(status, body.as_deref()));
    }

    let body = response
        .into_string()
        .map_err(|e| Error::RequestFailed(format!("Failed to read response: {}", e)))?;

    let user_info: UserInfo = serde_json::from_str(&body)?;

    Ok(user_info)
}
