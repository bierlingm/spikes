//! `spikes reply <id> <text>` — answer a reviewer's spike, optionally changing
//! its status in the same request.

use crate::api::ApiClient;
use crate::error::{Error, Result};
use crate::output::print_json;
use crate::spike::Reply;
use crate::storage::{find_spike_by_id, load_spikes, update_spike};

pub struct ReplyOptions {
    pub id: String,
    pub text: String,
    pub version: Option<String>,
    pub status: Option<String>,
    pub name: Option<String>,
    pub json: bool,
}

pub const VALID_STATUSES: &[&str] = &["open", "addressed", "wont_do"];

/// Validate a status flag value.
pub fn validate_status(status: &str) -> Result<String> {
    let normalized = status.trim().to_lowercase().replace('-', "_");
    if VALID_STATUSES.contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        Err(Error::RequestFailed(format!(
            "Invalid status '{}'. Use one of: {}",
            status,
            VALID_STATUSES.join(", ")
        )))
    }
}

/// Expand an ID prefix to the full ID when the spike is in the local cache.
pub fn resolve_full_id(id_or_prefix: &str) -> String {
    match load_spikes() {
        Ok(spikes) => find_spike_by_id(&spikes, id_or_prefix)
            .map(|s| s.id)
            .unwrap_or_else(|_| id_or_prefix.to_string()),
        Err(_) => id_or_prefix.to_string(),
    }
}

pub fn run(options: ReplyOptions) -> Result<()> {
    if options.text.trim().is_empty() {
        return Err(Error::RequestFailed(
            "Reply text cannot be empty".to_string(),
        ));
    }
    let status = options.status.as_deref().map(validate_status).transpose()?;

    let client = ApiClient::from_env()?;
    let id = resolve_full_id(&options.id);

    let mut body = serde_json::json!({ "body": options.text });
    if let Some(ref name) = options.name {
        body["author_name"] = serde_json::Value::String(name.clone());
    }
    if let Some(ref v) = options.version {
        body["version_label"] = serde_json::Value::String(v.clone());
        // A version label on a reply implies the spike was addressed there,
        // unless the caller chose a different status explicitly.
        if status.is_none() {
            body["status"] = serde_json::Value::String("addressed".to_string());
        }
        body["addressed_in"] = serde_json::Value::String(v.clone());
    }
    if let Some(ref s) = status {
        body["status"] = serde_json::Value::String(s.clone());
    }

    let response = client.post(
        &format!("/spikes/{}/replies", crate::api::encode(&id)),
        &body,
    )?;
    let reply: Option<Reply> = serde_json::from_value(response.clone()).ok();

    // Best effort: keep the local cache in step.
    let effective_status = body
        .get("status")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let addressed_in = options.version.clone();
    let reply_for_cache = reply.clone();
    let _ = update_spike(&id, |spike| {
        if let Some(ref s) = effective_status {
            spike.status = Some(s.clone());
            spike.resolved = Some(s != "open");
            if s != "open" && spike.resolved_at.is_none() {
                spike.resolved_at = Some(chrono::Utc::now().to_rfc3339());
            }
            if s == "open" {
                spike.resolved_at = None;
            }
        }
        if addressed_in.is_some() {
            spike.addressed_in = addressed_in.clone();
        }
        spike.reply_count = Some(spike.reply_count.unwrap_or(0) + 1);
        if let Some(ref r) = reply_for_cache {
            spike.last_reply = Some(r.clone());
        }
    });

    if options.json {
        print_json(&serde_json::json!({
            "success": true,
            "spike_id": id,
            "reply": response,
        }));
    } else {
        println!();
        println!("  / Replied to spike {}", id);
        if let Some(ref r) = reply {
            println!("  Reply ID: {}", r.id);
        }
        if let Some(s) = body.get("status").and_then(|v| v.as_str()) {
            println!("  Status:   {}", s);
        }
        if let Some(ref v) = options.version {
            println!("  Version:  {}", v);
        }
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_status_accepts_known() {
        assert_eq!(validate_status("addressed").unwrap(), "addressed");
        assert_eq!(validate_status("WONT-DO").unwrap(), "wont_do");
        assert_eq!(validate_status(" open ").unwrap(), "open");
    }

    #[test]
    fn test_validate_status_rejects_unknown() {
        assert!(validate_status("done").is_err());
    }
}
