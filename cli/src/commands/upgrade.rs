//! Upgrade command - open Stripe Checkout for Pro subscription
//!
//! VAL-MON-006: spikes upgrade CLI Command
//! Calls /billing/checkout endpoint and opens the returned URL in the default browser.
//! After successful payment processing via webhook, user's tier updates to pro.

use serde::Deserialize;

use crate::auth::{get_api_base, AuthConfig};
use crate::error::{map_http_error, map_network_error, Error, Result};

#[derive(Debug, Deserialize)]
struct CheckoutResponse {
    url: Option<String>,
    message: Option<String>,
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

    // Call /billing/checkout endpoint
    let result = fetch_checkout(&token)?;

    if let Some(message) = &result.message {
        // User is already Pro
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "message": message
                })
            );
        } else {
            println!("{}", message);
        }
        return Ok(());
    }

    let checkout_url = result.url.ok_or_else(|| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No checkout URL returned from server",
        ))
    })?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "url": checkout_url
            })
        );
    } else {
        println!("Opening checkout...");
        println!("After completing payment, your account will be upgraded to Pro.");
        
        // Open URL in default browser
        // webbrowser crate handles cross-platform browser opening
        if let Err(e) = webbrowser::open(&checkout_url) {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to open browser: {}", e),
            )));
        }
    }

    Ok(())
}

fn fetch_checkout(token: &str) -> Result<CheckoutResponse> {
    let api_base = get_api_base();
    let url = format!("{}/billing/checkout", api_base.trim_end_matches('/'));

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

    let checkout: CheckoutResponse = serde_json::from_str(&body)?;

    Ok(checkout)
}
