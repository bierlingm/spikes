//! Billing command - open Stripe Customer Portal in browser
//!
//! VAL-MON-002: spikes billing CLI Command
//! Calls /billing/portal endpoint and opens the returned URL in the default browser.
//! Works on macOS, Linux, and Windows.

use serde::Deserialize;

use crate::auth::{get_api_base, AuthConfig};
use crate::error::{map_http_error, map_network_error, Error, Result};

#[derive(Debug, Deserialize)]
struct BillingPortalResponse {
    url: String,
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

    // Call /billing/portal endpoint
    let portal_url = fetch_billing_portal(&token)?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "url": portal_url
            })
        );
    } else {
        println!("Opening billing portal...");
        
        // Open URL in default browser
        // webbrowser crate handles cross-platform browser opening
        if let Err(e) = webbrowser::open(&portal_url) {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to open browser: {}", e),
            )));
        }
    }

    Ok(())
}

fn fetch_billing_portal(token: &str) -> Result<String> {
    let api_base = get_api_base();
    let url = format!("{}/billing/portal", api_base.trim_end_matches('/'));

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

    let portal: BillingPortalResponse = serde_json::from_str(&body)?;

    Ok(portal.url)
}
