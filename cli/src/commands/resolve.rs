//! `spikes resolve` — mark a spike addressed, won't-do, or open again.
//!
//! Updates the local cache when the spike is there, and sends the change to
//! the hosted API when a remote is configured (`[remote]` in
//! `.spikes/config.toml`). Without flags the request body stays
//! `{ "resolved": true }`; `--in`, `--wont-do`, and `--undo` send the v2
//! `status` field.

use crate::api::{encode, resolve_credential, resolve_endpoint, ApiClient};
use crate::config::Config;
use crate::error::{Error, Result};
use crate::output::print_json;
use crate::spike::Spike;
use crate::storage::{find_spike_by_id, load_spikes, save_spikes};

pub struct ResolveOptions {
    pub id: String,
    /// Mark as unresolved (legacy alias of `--undo`)
    pub unresolve: bool,
    /// Reopen the spike (status: open)
    pub undo: bool,
    /// Version label the spike was addressed in
    pub addressed_in: Option<String>,
    /// Mark as won't do
    pub wont_do: bool,
    pub json: bool,
}

impl ResolveOptions {
    /// Target status implied by the flags.
    pub fn target_status(&self) -> &'static str {
        if self.undo || self.unresolve {
            "open"
        } else if self.wont_do {
            "wont_do"
        } else {
            "addressed"
        }
    }

    /// True when a v2 flag was used, so the remote body carries `status`.
    pub fn uses_status_flags(&self) -> bool {
        self.undo || self.wont_do || self.addressed_in.is_some()
    }

    /// JSON body for `PATCH /spikes/:id`.
    pub fn remote_body(&self) -> serde_json::Value {
        if self.uses_status_flags() {
            let mut body = serde_json::json!({ "status": self.target_status() });
            if let Some(ref v) = self.addressed_in {
                body["addressed_in"] = serde_json::Value::String(v.clone());
            }
            body
        } else {
            serde_json::json!({ "resolved": !self.unresolve })
        }
    }
}

/// Get current time as ISO 8601 timestamp
fn current_timestamp() -> String {
    chrono::Local::now().to_rfc3339()
}

fn apply_local(spike: &mut Spike, options: &ResolveOptions) {
    let status = options.target_status();
    if status == "open" {
        spike.resolved = None;
        spike.resolved_at = None;
        spike.status = if spike.status.is_some() {
            Some("open".to_string())
        } else {
            None
        };
    } else {
        spike.resolved = Some(true);
        spike.resolved_at = Some(current_timestamp());
        if options.uses_status_flags() || spike.status.is_some() {
            spike.status = Some(status.to_string());
        }
    }
    if options.addressed_in.is_some() {
        spike.addressed_in = options.addressed_in.clone();
    }
}

pub fn run(options: ResolveOptions) -> Result<()> {
    if options.wont_do && options.addressed_in.is_some() {
        return Err(Error::RequestFailed(
            "--wont-do and --in cannot be combined".to_string(),
        ));
    }
    if (options.undo || options.unresolve) && (options.wont_do || options.addressed_in.is_some()) {
        return Err(Error::RequestFailed(
            "--undo cannot be combined with --in or --wont-do".to_string(),
        ));
    }

    // Local lookup (optional when a remote is configured)
    let local = load_spikes().ok();
    let local_match = local
        .as_ref()
        .and_then(|spikes| find_spike_by_id(spikes, &options.id).ok());
    let full_id = local_match
        .as_ref()
        .map(|s| s.id.clone())
        .unwrap_or_else(|| options.id.clone());

    // Remote first, so a failed PATCH does not leave the cache ahead of the server
    let remote_configured = Config::load()
        .map(|c| c.effective_endpoint().is_some())
        .unwrap_or(false);
    let mut remote_response: Option<serde_json::Value> = None;
    if remote_configured {
        let cred = resolve_credential()?.ok_or(Error::NoCredential)?;
        let client = ApiClient::new(&resolve_endpoint(), &cred.token);
        let response = client.patch(
            &format!("/spikes/{}", encode(&full_id)),
            &options.remote_body(),
        )?;
        remote_response = Some(response);
    } else if local_match.is_none() {
        return Err(Error::SpikeNotFound(options.id.clone()));
    }

    // Local update
    let mut updated_spike: Option<Spike> = None;
    if let (Some(mut spikes), Some(matched)) = (local, local_match) {
        for s in &mut spikes {
            if s.id == matched.id {
                apply_local(s, &options);
                updated_spike = Some(s.clone());
                break;
            }
        }
        save_spikes(&spikes)?;
    }

    let status = options.target_status();
    if options.json {
        match (updated_spike, remote_response) {
            (Some(spike), remote) => print_json(&serde_json::json!({
                "success": true,
                "spike": spike,
                "status": status,
                "remote": remote,
            })),
            (None, remote) => print_json(&serde_json::json!({
                "success": true,
                "id": full_id,
                "status": status,
                "remote": remote,
            })),
        }
    } else {
        match status {
            "open" if options.unresolve && !options.undo => {
                println!("Unresolved spike {}.", full_id)
            }
            "open" => println!("Reopened spike {}.", full_id),
            "wont_do" => println!("Marked spike {} as won't do.", full_id),
            _ => match options.addressed_in {
                Some(ref v) => println!("Resolved spike {} (addressed in {}).", full_id, v),
                None => println!("Resolved spike {}.", full_id),
            },
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> ResolveOptions {
        ResolveOptions {
            id: "abcd".to_string(),
            unresolve: false,
            undo: false,
            addressed_in: None,
            wont_do: false,
            json: false,
        }
    }

    #[test]
    fn test_plain_resolve_sends_resolved_true() {
        assert_eq!(
            opts().remote_body(),
            serde_json::json!({ "resolved": true })
        );
    }

    #[test]
    fn test_unresolve_sends_resolved_false() {
        let o = ResolveOptions {
            unresolve: true,
            ..opts()
        };
        assert_eq!(o.remote_body(), serde_json::json!({ "resolved": false }));
        assert_eq!(o.target_status(), "open");
    }

    #[test]
    fn test_in_sends_status_and_label() {
        let o = ResolveOptions {
            addressed_in: Some("v0.5".to_string()),
            ..opts()
        };
        assert_eq!(
            o.remote_body(),
            serde_json::json!({ "status": "addressed", "addressed_in": "v0.5" })
        );
    }

    #[test]
    fn test_wont_do_and_undo() {
        let o = ResolveOptions {
            wont_do: true,
            ..opts()
        };
        assert_eq!(o.remote_body(), serde_json::json!({ "status": "wont_do" }));
        let u = ResolveOptions {
            undo: true,
            ..opts()
        };
        assert_eq!(u.remote_body(), serde_json::json!({ "status": "open" }));
    }

    #[test]
    fn test_apply_local_sets_fields() {
        let mut s = Spike::default();
        apply_local(
            &mut s,
            &ResolveOptions {
                addressed_in: Some("v1".to_string()),
                ..opts()
            },
        );
        assert_eq!(s.resolved, Some(true));
        assert_eq!(s.status.as_deref(), Some("addressed"));
        assert_eq!(s.addressed_in.as_deref(), Some("v1"));
        apply_local(
            &mut s,
            &ResolveOptions {
                undo: true,
                ..opts()
            },
        );
        assert_eq!(s.resolved, None);
        assert_eq!(s.status.as_deref(), Some("open"));
    }

    #[test]
    fn test_apply_local_plain_keeps_legacy_shape() {
        let mut s = Spike::default();
        apply_local(&mut s, &opts());
        assert_eq!(s.resolved, Some(true));
        assert!(s.status.is_none());
    }
}
