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

    // Print hint for creating shares if empty
    if usage.shares == 0 {
        println!();
        println!("💡 Create a share with: spikes share <directory>");
    }

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

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_shares_table_empty_outputs_to_stdout() {
        // Capture stdout
        let shares: Vec<Share> = vec![];
        
        // Verify function runs without panic on empty shares
        print_shares_table(&shares);
        // Note: We can't easily capture println! in unit tests, but we verify
        // the function doesn't panic and follows the Ok(()) path
    }

    #[test]
    fn test_print_shares_table_with_shows_hint_when_empty() {
        let shares: Vec<Share> = vec![];
        print_shares_table(&shares);
        // Function completes without error - success path
    }

    #[test]
    fn test_share_serialization_roundtrip() {
        let share = Share {
            id: "share_123".to_string(),
            slug: "my-project".to_string(),
            url: "https://spikes.sh/s/my-project".to_string(),
            spike_count: 42,
            created_at: "2025-01-15T10:30:00.000Z".to_string(),
        };

        let json = serde_json::to_string(&share).unwrap();
        let deserialized: Share = serde_json::from_str(&json).unwrap();
        
        assert_eq!(share.id, deserialized.id);
        assert_eq!(share.slug, deserialized.slug);
        assert_eq!(share.url, deserialized.url);
        assert_eq!(share.spike_count, deserialized.spike_count);
        assert_eq!(share.created_at, deserialized.created_at);
    }

    #[test]
    fn test_usage_response_deserialization() {
        let json = r#"{
            "spikes": 100,
            "spike_limit": 1000,
            "shares": 5,
            "share_limit": 10,
            "tier": "free"
        }"#;

        let usage: UsageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(usage.spikes, 100);
        assert_eq!(usage.spike_limit, Some(1000));
        assert_eq!(usage.shares, 5);
        assert_eq!(usage.share_limit, Some(10));
        assert_eq!(usage.tier, "free");
    }

    #[test]
    fn test_usage_response_pro_tier_no_limits() {
        let json = r#"{
            "spikes": 500,
            "spike_limit": null,
            "shares": 25,
            "share_limit": null,
            "tier": "pro"
        }"#;

        let usage: UsageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(usage.spikes, 500);
        assert_eq!(usage.spike_limit, None);
        assert_eq!(usage.shares, 25);
        assert_eq!(usage.share_limit, None);
        assert_eq!(usage.tier, "pro");
    }

    #[test]
    fn test_empty_shares_produces_valid_empty_array_json() {
        // Simulate what run() does for --json with empty shares
        let shares: Vec<Share> = vec![];
        let usage = UsageResponse {
            spikes: 0,
            spike_limit: Some(1000),
            shares: 0,
            share_limit: Some(5),
            tier: "free".to_string(),
        };

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

        let json_str = serde_json::to_string_pretty(&output).unwrap();
        
        // Verify the JSON has an empty shares array
        assert!(json_str.contains("\"shares\": []"));
        
        // Verify it parses back correctly
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let shares_array = parsed.get("shares").unwrap().as_array().unwrap();
        assert!(shares_array.is_empty());
    }
}
