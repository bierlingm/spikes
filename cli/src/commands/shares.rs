use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, ContentArrangement, Table};
use serde::{Deserialize, Serialize};

use crate::auth::{get_api_base, AuthConfig};
use crate::error::{map_http_error, map_network_error, Error, Result};

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
struct UsageResponse {
    spikes: u64,
    spike_limit: Option<u64>,
    shares: u64,
    share_limit: Option<u64>,
    tier: String,
}

pub fn run(options: SharesOptions) -> Result<()> {
    let token = AuthConfig::token()?
        .ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Not logged in. Run 'spikes login' first.",
            ))
        })?;
    
    // Fetch both shares and usage
    let shares = fetch_shares(&token)?;
    let usage = fetch_usage(&token)?;

    if options.json {
        let output = serde_json::json!({
            "shares": shares,
            "usage": {
                "spikes": usage.spikes,
                "spike_limit": usage.spike_limit,
                "shares_count": usage.shares,
                "share_limit": usage.share_limit,
                "tier": usage.tier,
            }
        });
        println!("{}", serde_json::to_string_pretty(&output).expect("Failed to serialize to JSON"));
    } else {
        print_shares_table(&shares);
        print_usage_summary(&usage);
    }

    Ok(())
}

fn fetch_shares(token: &str) -> Result<Vec<Share>> {
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

    if status != 200 {
        let body = response.into_string().ok();
        return Err(map_http_error(status, body.as_deref()));
    }

    let body = response
        .into_string()
        .map_err(|e| Error::RequestFailed(format!("Failed to read response: {}", e)))?;

    let shares: Vec<Share> = serde_json::from_str(&body)?;
    Ok(shares)
}

fn fetch_usage(token: &str) -> Result<UsageResponse> {
    let api_base = get_api_base();
    let url = format!("{}/usage", api_base.trim_end_matches('/'));

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

    let usage: UsageResponse = serde_json::from_str(&body)?;
    Ok(usage)
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

fn print_usage_summary(usage: &UsageResponse) {
    println!();
    println!("Usage Summary ({} tier):", usage.tier.to_uppercase());

    match (usage.share_limit, usage.spike_limit) {
        (Some(share_limit), Some(spike_limit)) => {
            // Free tier with limits
            let share_pct = if share_limit > 0 {
                (usage.shares as f64 / share_limit as f64 * 100.0) as u64
            } else {
                0
            };
            let spike_pct = if spike_limit > 0 {
                (usage.spikes as f64 / spike_limit as f64 * 100.0) as u64
            } else {
                0
            };

            println!(
                "  Shares: {}/{} ({}%)  Spikes: {}/{} ({}%)",
                usage.shares, share_limit, share_pct,
                usage.spikes, spike_limit, spike_pct
            );

            // Show upgrade message if near limits
            if share_pct >= 80 || spike_pct >= 80 {
                println!();
                println!("⚠️  Approaching limits. Upgrade: https://spikes.sh/pro");
            }
        }
        _ => {
            // Pro tier - unlimited
            println!(
                "  Shares: {}  Spikes: {}  (Unlimited)",
                usage.shares, usage.spikes
            );
        }
    }
}
