//! Sync state stamp: `.spikes/state.json`
//!
//! Records when the local cache (`.spikes/feedback.jsonl`) was last pulled
//! from a remote, which endpoint it came from, and a fingerprint of the
//! credential used. Read commands warn when a remote is configured and the
//! cache is missing a stamp or older than 24 hours, so a stale cache never
//! silently passes for current state.

use std::fs;
use std::path::Path;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::Result;

pub const STATE_FILE: &str = ".spikes/state.json";

/// Age after which the cache is considered stale.
pub const STALE_AFTER_HOURS: i64 = 24;

/// Length of the credential fingerprint stored in the stamp.
pub const CREDENTIAL_PREFIX_LEN: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SyncState {
    /// ISO 8601 timestamp of the last successful pull/sync/watch poll
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_pulled_at: Option<String>,
    /// Endpoint the cache was pulled from
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    /// First 12 characters of the credential used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_prefix: Option<String>,
}

impl SyncState {
    /// Load the stamp from `.spikes/state.json`, or defaults when missing/invalid.
    pub fn load() -> Self {
        Self::load_from(Path::new(STATE_FILE))
    }

    pub fn load_from(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save the stamp to `.spikes/state.json`.
    pub fn save(&self) -> Result<()> {
        self.save_to(Path::new(STATE_FILE))
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Age of the stamp, or `None` when there is no valid stamp.
    pub fn age(&self) -> Option<Duration> {
        self.age_at(Utc::now())
    }

    pub fn age_at(&self, now: DateTime<Utc>) -> Option<Duration> {
        let ts = self.last_pulled_at.as_deref()?;
        let pulled = DateTime::parse_from_rfc3339(ts).ok()?.with_timezone(&Utc);
        Some(now - pulled)
    }

    /// True when there is no stamp or it is older than [`STALE_AFTER_HOURS`].
    pub fn is_stale(&self) -> bool {
        self.is_stale_at(Utc::now())
    }

    pub fn is_stale_at(&self, now: DateTime<Utc>) -> bool {
        match self.age_at(now) {
            Some(age) => age > Duration::hours(STALE_AFTER_HOURS),
            None => true,
        }
    }
}

/// First [`CREDENTIAL_PREFIX_LEN`] characters of a token, for stamping.
pub fn credential_prefix(token: &str) -> String {
    token.chars().take(CREDENTIAL_PREFIX_LEN).collect()
}

/// Record a successful pull from `endpoint` with `token` at the current time.
pub fn stamp(endpoint: &str, token: &str) -> Result<()> {
    let state = SyncState {
        last_pulled_at: Some(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
        endpoint: Some(endpoint.to_string()),
        credential_prefix: Some(credential_prefix(token)),
    };
    state.save()
}

/// Human-readable age, e.g. "3 hours", "2 days", "just now".
pub fn format_age(age: Duration) -> String {
    let secs = age.num_seconds();
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        let m = secs / 60;
        format!("{} minute{}", m, if m == 1 { "" } else { "s" })
    } else if secs < 86_400 {
        let h = secs / 3600;
        format!("{} hour{}", h, if h == 1 { "" } else { "s" })
    } else {
        let d = secs / 86_400;
        format!("{} day{}", d, if d == 1 { "" } else { "s" })
    }
}

/// Print a one-line stderr warning when a remote is configured and the local
/// cache is unstamped or stale. Silent otherwise, and never touches stdout.
pub fn warn_if_stale() {
    let remote_configured = Config::load()
        .map(|c| c.effective_endpoint().is_some())
        .unwrap_or(false);
    if !remote_configured {
        return;
    }
    let state = SyncState::load();
    if !state.is_stale() {
        return;
    }
    match state.age() {
        Some(age) => eprintln!(
            "  ! Local cache last pulled {} ago. Run `spikes pull` to refresh.",
            format_age(age)
        ),
        None => eprintln!(
            "  ! Local cache has never been pulled from the remote. Run `spikes pull` to refresh."
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_is_stale() {
        let s = SyncState::default();
        assert!(s.is_stale());
        assert!(s.age().is_none());
    }

    #[test]
    fn test_fresh_stamp_not_stale() {
        let now = Utc::now();
        let s = SyncState {
            last_pulled_at: Some((now - Duration::hours(1)).to_rfc3339()),
            endpoint: None,
            credential_prefix: None,
        };
        assert!(!s.is_stale_at(now));
    }

    #[test]
    fn test_old_stamp_is_stale() {
        let now = Utc::now();
        let s = SyncState {
            last_pulled_at: Some((now - Duration::hours(25)).to_rfc3339()),
            endpoint: None,
            credential_prefix: None,
        };
        assert!(s.is_stale_at(now));
    }

    #[test]
    fn test_invalid_timestamp_is_stale() {
        let s = SyncState {
            last_pulled_at: Some("not a date".to_string()),
            endpoint: None,
            credential_prefix: None,
        };
        assert!(s.is_stale());
    }

    #[test]
    fn test_credential_prefix_truncates() {
        assert_eq!(
            credential_prefix("sk_spikes_abcdefghijklmnop"),
            "sk_spikes_ab"
        );
        assert_eq!(credential_prefix("short"), "short");
    }

    #[test]
    fn test_format_age() {
        assert_eq!(format_age(Duration::seconds(5)), "just now");
        assert_eq!(format_age(Duration::minutes(1)), "1 minute");
        assert_eq!(format_age(Duration::minutes(30)), "30 minutes");
        assert_eq!(format_age(Duration::hours(1)), "1 hour");
        assert_eq!(format_age(Duration::hours(5)), "5 hours");
        assert_eq!(format_age(Duration::days(3)), "3 days");
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".spikes/state.json");
        let s = SyncState {
            last_pulled_at: Some("2026-09-14T12:00:00.000Z".to_string()),
            endpoint: Some("https://spikes.sh".to_string()),
            credential_prefix: Some("sk_spikes_ab".to_string()),
        };
        s.save_to(&path).unwrap();
        let loaded = SyncState::load_from(&path);
        assert_eq!(loaded, s);
    }

    #[test]
    fn test_load_missing_file_is_default() {
        let dir = tempfile::tempdir().unwrap();
        let loaded = SyncState::load_from(&dir.path().join("nope.json"));
        assert_eq!(loaded, SyncState::default());
    }
}
