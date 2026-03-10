//! Usage command - display current usage statistics
//!
//! VAL-MON-004: spikes usage CLI Command
//! VAL-PRICE-030: spikes usage for agent-tier shows cost and budget cap
//! Calls /usage endpoint and displays formatted usage information with progress bars
//! or percentages toward limits. Shows upgrade message if near limits.
//! For agent-tier users, displays cost, budget cap, and billing period.

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
    /// Cost this billing period in cents (agent tier only)
    #[serde(skip_serializing_if = "Option::is_none")]
    cost_this_period_cents: Option<u64>,
    /// Monthly budget cap in cents (agent tier only, None = no cap)
    #[serde(skip_serializing_if = "Option::is_none")]
    monthly_cap_cents: Option<u64>,
    /// Billing period end date (agent tier only)
    #[serde(skip_serializing_if = "Option::is_none")]
    period_ends: Option<String>,
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

    if options.json {
        // In JSON mode, pass through the raw API response to preserve all fields
        let raw = fetch_usage_raw(&token)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&raw).expect("Failed to serialize to JSON")
        );
    } else {
        let usage = fetch_usage(&token)?;
        print_usage_table(&usage);
    }

    Ok(())
}

/// Fetch raw JSON from /usage for --json passthrough (preserves all API fields)
fn fetch_usage_raw(token: &str) -> Result<serde_json::Value> {
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

    let value: serde_json::Value = serde_json::from_str(&body)?;
    Ok(value)
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
        Cell::new(usage.share_limit.map(|l| l.to_string()).unwrap_or_else(|| "∞".to_string())),
        Cell::new(&share_usage),
    ]);

    table.add_row(vec![
        Cell::new("Spikes"),
        Cell::new(&spike_display),
        Cell::new(usage.spike_limit.map(|l| l.to_string()).unwrap_or_else(|| "∞".to_string())),
        Cell::new(&spike_usage),
    ]);

    println!("{table}");

    // Agent tier: show cost, budget cap, and billing period
    if usage.tier == "agent" {
        println!();
        if let Some(cost_cents) = usage.cost_this_period_cents {
            println!("  Cost this period: {}", format_cost(cost_cents));
        }
        match usage.monthly_cap_cents {
            Some(cap) => println!("  Budget cap: {}", format_cost(cap)),
            None => println!("  Budget cap: None"),
        }
        if let Some(ref period_ends) = usage.period_ends {
            println!("  Period ends: {}", period_ends);
        }
    }

    // Show upgrade message if near limits (>= 80% usage) — free tier only
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

/// Format cents as a dollar amount (e.g., 1234 -> "$12.34")
fn format_cost(cents: u64) -> String {
    let dollars = cents / 100;
    let remainder = cents % 100;
    format!("${}.{:02}", dollars, remainder)
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

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // UsageResponse deserialization tests
    // ========================================================================

    #[test]
    fn test_usage_response_free_tier_no_agent_fields() {
        let json = r#"{
            "spikes": 50,
            "spike_limit": 1000,
            "shares": 3,
            "share_limit": 5,
            "tier": "free",
            "reset_at": null
        }"#;

        let usage: UsageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(usage.spikes, 50);
        assert_eq!(usage.spike_limit, Some(1000));
        assert_eq!(usage.shares, 3);
        assert_eq!(usage.share_limit, Some(5));
        assert_eq!(usage.tier, "free");
        assert!(usage.cost_this_period_cents.is_none());
        assert!(usage.monthly_cap_cents.is_none());
        assert!(usage.period_ends.is_none());
    }

    #[test]
    fn test_usage_response_pro_tier_no_agent_fields() {
        let json = r#"{
            "spikes": 500,
            "spike_limit": null,
            "shares": 10,
            "share_limit": null,
            "tier": "pro",
            "reset_at": null
        }"#;

        let usage: UsageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(usage.spike_limit, None);
        assert_eq!(usage.share_limit, None);
        assert_eq!(usage.tier, "pro");
        assert!(usage.cost_this_period_cents.is_none());
        assert!(usage.monthly_cap_cents.is_none());
        assert!(usage.period_ends.is_none());
    }

    #[test]
    fn test_usage_response_agent_tier_with_cost_fields() {
        let json = r#"{
            "spikes": 250,
            "spike_limit": null,
            "shares": 8,
            "share_limit": null,
            "tier": "agent",
            "reset_at": null,
            "cost_this_period_cents": 1234,
            "monthly_cap_cents": 5000,
            "period_ends": "2026-04-01T00:00:00Z"
        }"#;

        let usage: UsageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(usage.tier, "agent");
        assert_eq!(usage.cost_this_period_cents, Some(1234));
        assert_eq!(usage.monthly_cap_cents, Some(5000));
        assert_eq!(usage.period_ends, Some("2026-04-01T00:00:00Z".to_string()));
    }

    #[test]
    fn test_usage_response_agent_tier_no_cap() {
        let json = r#"{
            "spikes": 100,
            "spike_limit": null,
            "shares": 2,
            "share_limit": null,
            "tier": "agent",
            "reset_at": null,
            "cost_this_period_cents": 500,
            "monthly_cap_cents": null,
            "period_ends": "2026-04-01T00:00:00Z"
        }"#;

        let usage: UsageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(usage.tier, "agent");
        assert_eq!(usage.cost_this_period_cents, Some(500));
        assert!(usage.monthly_cap_cents.is_none());
    }

    // ========================================================================
    // JSON serialization tests (--json mode passthrough)
    // ========================================================================

    #[test]
    fn test_usage_response_json_includes_agent_fields() {
        let usage = UsageResponse {
            spikes: 250,
            spike_limit: None,
            shares: 8,
            share_limit: None,
            tier: "agent".to_string(),
            reset_at: None,
            cost_this_period_cents: Some(1234),
            monthly_cap_cents: Some(5000),
            period_ends: Some("2026-04-01T00:00:00Z".to_string()),
        };

        let json = serde_json::to_string_pretty(&usage).unwrap();
        assert!(json.contains("\"cost_this_period_cents\": 1234"));
        assert!(json.contains("\"monthly_cap_cents\": 5000"));
        assert!(json.contains("\"period_ends\": \"2026-04-01T00:00:00Z\""));
    }

    #[test]
    fn test_usage_response_json_omits_absent_agent_fields() {
        let usage = UsageResponse {
            spikes: 50,
            spike_limit: Some(1000),
            shares: 3,
            share_limit: Some(5),
            tier: "free".to_string(),
            reset_at: None,
            cost_this_period_cents: None,
            monthly_cap_cents: None,
            period_ends: None,
        };

        let json = serde_json::to_string_pretty(&usage).unwrap();
        assert!(!json.contains("cost_this_period_cents"));
        assert!(!json.contains("monthly_cap_cents"));
        assert!(!json.contains("period_ends"));
    }

    // ========================================================================
    // format_cost tests
    // ========================================================================

    #[test]
    fn test_format_cost_zero() {
        assert_eq!(format_cost(0), "$0.00");
    }

    #[test]
    fn test_format_cost_cents_only() {
        assert_eq!(format_cost(5), "$0.05");
    }

    #[test]
    fn test_format_cost_dollars_and_cents() {
        assert_eq!(format_cost(1234), "$12.34");
    }

    #[test]
    fn test_format_cost_even_dollars() {
        assert_eq!(format_cost(5000), "$50.00");
    }

    #[test]
    fn test_format_cost_one_cent() {
        assert_eq!(format_cost(1), "$0.01");
    }

    // ========================================================================
    // print_usage_table output tests (capture stdout)
    // ========================================================================

    #[test]
    fn test_json_passthrough_preserves_unknown_fields() {
        // When the API returns extra fields, raw Value passthrough preserves them
        let raw_json = r#"{
            "spikes": 250,
            "spike_limit": null,
            "shares": 8,
            "share_limit": null,
            "tier": "agent",
            "reset_at": null,
            "cost_this_period_cents": 1234,
            "monthly_cap_cents": 5000,
            "period_ends": "2026-04-01T00:00:00Z",
            "some_future_field": "preserved"
        }"#;

        let value: serde_json::Value = serde_json::from_str(raw_json).unwrap();
        let output = serde_json::to_string_pretty(&value).unwrap();
        // All fields preserved including unknown ones
        assert!(output.contains("cost_this_period_cents"));
        assert!(output.contains("monthly_cap_cents"));
        assert!(output.contains("period_ends"));
        assert!(output.contains("some_future_field"));
        assert!(output.contains("preserved"));
    }

    #[test]
    fn test_print_usage_table_free_tier_no_panic() {
        let usage = UsageResponse {
            spikes: 50,
            spike_limit: Some(1000),
            shares: 3,
            share_limit: Some(5),
            tier: "free".to_string(),
            reset_at: None,
            cost_this_period_cents: None,
            monthly_cap_cents: None,
            period_ends: None,
        };
        // Should not panic
        print_usage_table(&usage);
    }

    #[test]
    fn test_print_usage_table_pro_tier_no_panic() {
        let usage = UsageResponse {
            spikes: 500,
            spike_limit: None,
            shares: 10,
            share_limit: None,
            tier: "pro".to_string(),
            reset_at: None,
            cost_this_period_cents: None,
            monthly_cap_cents: None,
            period_ends: None,
        };
        // Should not panic
        print_usage_table(&usage);
    }

    #[test]
    fn test_print_usage_table_agent_tier_no_panic() {
        let usage = UsageResponse {
            spikes: 250,
            spike_limit: None,
            shares: 8,
            share_limit: None,
            tier: "agent".to_string(),
            reset_at: None,
            cost_this_period_cents: Some(1234),
            monthly_cap_cents: Some(5000),
            period_ends: Some("2026-04-01T00:00:00Z".to_string()),
        };
        // Should not panic
        print_usage_table(&usage);
    }

    #[test]
    fn test_print_usage_table_agent_tier_no_cap_no_panic() {
        let usage = UsageResponse {
            spikes: 100,
            spike_limit: None,
            shares: 2,
            share_limit: None,
            tier: "agent".to_string(),
            reset_at: None,
            cost_this_period_cents: Some(500),
            monthly_cap_cents: None,
            period_ends: Some("2026-04-01T00:00:00Z".to_string()),
        };
        // Should not panic — budget cap should show "None"
        print_usage_table(&usage);
    }

    // ========================================================================
    // progress_bar tests
    // ========================================================================

    #[test]
    fn test_progress_bar_empty() {
        let bar = progress_bar(0, 100);
        assert_eq!(bar, "░░░░░░░░░░");
    }

    #[test]
    fn test_progress_bar_full() {
        let bar = progress_bar(100, 100);
        assert!(bar.contains('█')); // High usage char at 100%
    }

    #[test]
    fn test_progress_bar_zero_limit() {
        let bar = progress_bar(0, 0);
        assert_eq!(bar, "░░░░░░░░░░");
    }
}
