//! Usage command - display current usage statistics
//!
//! VAL-MON-004: spikes usage CLI Command
//! Calls /usage endpoint and displays formatted usage information with progress bars
//! or percentages toward limits. Shows upgrade message if near limits.

use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, ContentArrangement, Table};

use crate::auth::{get_api_base, AuthConfig};
use crate::error::{map_http_error, map_network_error, Error, Result};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct UsageResponse {
    spikes: u64,
    spike_limit: Option<u64>,
    shares: u64,
    share_limit: Option<u64>,
    tier: String,
    #[allow(dead_code)]
    reset_at: Option<String>,
}

pub struct UsageOptions {
    pub json: bool,
}

pub fn run(options: UsageOptions) -> Result<()> {
    // Check if user has a token
    let token = AuthConfig::token()?
        .ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Not logged in. Run 'spikes login' first.",
            ))
        })?;

    // Call /usage endpoint
    let usage = fetch_usage(&token)?;

    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&usage).expect("Failed to serialize to JSON")
        );
    } else {
        print_usage_table(&usage);
    }

    Ok(())
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

fn print_usage_table(usage: &UsageResponse) {
    println!("Account: {} tier", usage.tier.to_uppercase());
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Resource", "Used", "Limit", "Usage"]);

    // Shares row
    let (share_display, share_usage) = match usage.share_limit {
        Some(limit) => {
            let percentage = if limit > 0 {
                (usage.shares as f64 / limit as f64 * 100.0) as u64
            } else {
                0
            };
            let usage_bar = progress_bar(usage.shares, limit);
            let display = format!("{} / {}", usage.shares, limit);
            (display, format!("{} ({}%)", usage_bar, percentage))
        }
        None => {
            // Pro tier - unlimited
            (usage.shares.to_string(), "Unlimited".to_string())
        }
    };

    // Spikes row
    let (spike_display, spike_usage) = match usage.spike_limit {
        Some(limit) => {
            let percentage = if limit > 0 {
                (usage.spikes as f64 / limit as f64 * 100.0) as u64
            } else {
                0
            };
            let usage_bar = progress_bar(usage.spikes, limit);
            let display = format!("{} / {}", usage.spikes, limit);
            (display, format!("{} ({}%)", usage_bar, percentage))
        }
        None => {
            // Pro tier - unlimited
            (usage.spikes.to_string(), "Unlimited".to_string())
        }
    };

    table.add_row(vec![
        Cell::new("Shares"),
        Cell::new(&share_display),
        Cell::new(&usage.share_limit.map(|l| l.to_string()).unwrap_or_else(|| "∞".to_string())),
        Cell::new(&share_usage),
    ]);

    table.add_row(vec![
        Cell::new("Spikes"),
        Cell::new(&spike_display),
        Cell::new(&usage.spike_limit.map(|l| l.to_string()).unwrap_or_else(|| "∞".to_string())),
        Cell::new(&spike_usage),
    ]);

    println!("{table}");

    // Show upgrade message if near limits (>= 80% usage)
    if let (Some(share_limit), Some(spike_limit)) = (usage.share_limit, usage.spike_limit) {
        let share_pct = usage.shares as f64 / share_limit as f64;
        let spike_pct = usage.spikes as f64 / spike_limit as f64;

        if share_pct >= 0.8 || spike_pct >= 0.8 {
            println!();
            println!("⚠️  You're approaching your free tier limits!");
            println!("   Upgrade to Pro for unlimited shares and spikes:");
            println!("   https://spikes.sh/pro");
        }
    }
}

/// Generate a simple ASCII progress bar
fn progress_bar(current: u64, limit: u64) -> String {
    if limit == 0 {
        return "░░░░░░░░░░".to_string();
    }

    let ratio = (current as f64 / limit as f64).min(1.0);
    let filled = (ratio * 10.0).round() as usize;
    let empty = 10 - filled;

    let filled_char = if ratio >= 0.9 { '█' } else if ratio >= 0.7 { '▓' } else { '░' };

    format!("{}{}", filled_char.to_string().repeat(filled), "░".repeat(empty))
}
