//! `spikes watch` — poll the hosted API for new or updated spikes (and new
//! answers to questions) and print one JSON object per line per event.
//!
//! With `--exec <command>` each event is piped, as JSON, to the command's
//! stdin (`sh -c <command>`), which makes the Herdr bridge a one-liner:
//!
//! ```text
//! spikes watch --exec 'herdr agent prompt builder'
//! ```

use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::api::{data_array, encode, resolve_credential, resolve_endpoint, with_query, ApiClient};
use crate::config::Config;
use crate::error::{Error, Result};
use crate::spike::Spike;
use crate::state::{self, SyncState};

pub struct WatchOptions {
    /// ISO 8601 timestamp or "last" (default: "last", falling back to now)
    pub since: Option<String>,
    /// Poll interval in seconds
    pub interval: u64,
    pub url_prefix: Option<String>,
    /// Command to run per event with the JSON on stdin
    pub exec: Option<String>,
    /// Poll once and exit
    pub once: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum WatchEvent {
    #[serde(rename = "spike.created")]
    SpikeCreated { spike: Spike },
    #[serde(rename = "spike.updated")]
    SpikeUpdated { spike: Spike },
    #[serde(rename = "question.answered")]
    QuestionAnswered {
        question: serde_json::Value,
        answer: serde_json::Value,
    },
}

/// Resolve the starting timestamp: explicit ISO, or the stamp for "last",
/// or now when there is no stamp.
pub fn resolve_since(arg: Option<&str>, state: &SyncState, now: DateTime<Utc>) -> Result<String> {
    match arg {
        None | Some("last") => Ok(state
            .last_pulled_at
            .clone()
            .unwrap_or_else(|| now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))),
        Some(ts) => {
            DateTime::parse_from_rfc3339(ts).map_err(|e| {
                Error::RequestFailed(format!(
                    "Invalid --since value '{}': {} (use ISO 8601 or 'last')",
                    ts, e
                ))
            })?;
            Ok(ts.to_string())
        }
    }
}

/// Timestamp used to decide whether a spike is new or changed.
fn spike_marker(spike: &Spike) -> &str {
    spike
        .updated_at
        .as_deref()
        .or(spike.created_at.as_deref())
        .unwrap_or(&spike.timestamp)
}

fn is_after(ts: &str, since: &str) -> bool {
    match (
        DateTime::parse_from_rfc3339(ts),
        DateTime::parse_from_rfc3339(since),
    ) {
        (Ok(a), Ok(b)) => a > b,
        _ => ts > since,
    }
}

/// Classify spikes fetched with `?since=` into events, skipping ones already
/// seen with the same marker. Returns the events and the newest marker seen.
pub fn classify_spikes(
    spikes: Vec<Spike>,
    since: &str,
    seen: &mut HashMap<String, String>,
) -> (Vec<WatchEvent>, Option<String>) {
    let mut events = Vec::new();
    let mut newest: Option<String> = None;

    for spike in spikes {
        let marker = spike_marker(&spike).to_string();
        if seen.get(&spike.id) == Some(&marker) {
            continue;
        }
        if newest
            .as_deref()
            .map(|n| is_after(&marker, n))
            .unwrap_or(true)
        {
            newest = Some(marker.clone());
        }
        seen.insert(spike.id.clone(), marker.clone());

        let created = match (&spike.created_at, &spike.updated_at) {
            (Some(c), Some(u)) => c == u || is_after(c, since),
            (Some(c), None) => is_after(c, since),
            _ => true,
        };
        if created {
            events.push(WatchEvent::SpikeCreated { spike });
        } else {
            events.push(WatchEvent::SpikeUpdated { spike });
        }
    }

    (events, newest)
}

/// Next `since` cursor after a poll: the newest marker seen when it is later
/// than the current cursor, else the poll start time. `None` when the poll
/// failed, so the caller keeps the cursor and does not stamp the state file.
fn next_cursor(
    poll_ok: bool,
    newest_marker: Option<String>,
    since: &str,
    poll_started: &str,
) -> Option<String> {
    if !poll_ok {
        return None;
    }
    Some(match newest_marker {
        Some(m) if is_after(&m, since) => m,
        _ => poll_started.to_string(),
    })
}

fn deliver(event: &WatchEvent, exec: Option<&str>) -> Result<()> {
    let line = serde_json::to_string(event)?;
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    writeln!(handle, "{}", line)?;
    handle.flush()?;

    if let Some(cmd) = exec {
        match Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdin(Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(line.as_bytes());
                    let _ = stdin.write_all(b"\n");
                }
                match child.wait() {
                    Ok(status) if !status.success() => {
                        eprintln!("[spikes watch] --exec exited with {}", status);
                    }
                    Err(e) => eprintln!("[spikes watch] --exec failed: {}", e),
                    _ => {}
                }
            }
            Err(e) => eprintln!("[spikes watch] could not start --exec: {}", e),
        }
    }
    Ok(())
}

pub fn run(options: WatchOptions) -> Result<()> {
    let endpoint = resolve_endpoint();
    let cred = resolve_credential()?.ok_or(Error::NoCredential)?;
    let client = ApiClient::new(&endpoint, &cred.token);
    let project_key = Config::load()
        .ok()
        .and_then(|c| c.project.key)
        .filter(|k| !k.is_empty());

    let state = SyncState::load();
    let mut since = resolve_since(options.since.as_deref(), &state, Utc::now())?;
    let interval = Duration::from_secs(options.interval.max(1));

    eprintln!(
        "[spikes watch] polling {} every {}s since {}{}",
        endpoint,
        interval.as_secs(),
        since,
        project_key
            .as_deref()
            .map(|k| format!(" (project '{}')", k))
            .unwrap_or_default()
    );

    let mut seen_spikes: HashMap<String, String> = HashMap::new();
    let mut seen_answers: HashMap<String, String> = HashMap::new();
    let mut questions_warned = false;

    loop {
        let poll_started = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

        // Spikes
        let path = with_query(
            "/spikes",
            &[
                ("since", Some(since.as_str())),
                ("url_prefix", options.url_prefix.as_deref()),
            ],
        );
        let mut newest_marker: Option<String> = None;
        let mut poll_ok = false;
        match client.get(&path) {
            Ok(value) => {
                let spikes: Vec<Spike> = data_array(&value)
                    .into_iter()
                    .filter_map(|v| serde_json::from_value(v).ok())
                    .collect();
                let (events, newest) = classify_spikes(spikes, &since, &mut seen_spikes);
                newest_marker = newest;
                poll_ok = true;
                for event in &events {
                    deliver(event, options.exec.as_deref())?;
                }
            }
            Err(e @ Error::AuthFailed)
            | Err(e @ Error::AuthRevoked(_))
            | Err(e @ Error::AuthExpired(_)) => return Err(e),
            Err(e) => eprintln!("[spikes watch] poll failed: {}", e),
        }

        // Question answers (project-scoped)
        if let Some(ref key) = project_key {
            let qpath = format!("/me/projects/{}/questions", encode(key));
            match client.get(&qpath) {
                Ok(value) => {
                    for q in data_array(&value) {
                        let qid = match q.get("id").and_then(|v| v.as_str()) {
                            Some(id) => id.to_string(),
                            None => continue,
                        };
                        let last = q
                            .get("lastAnswer")
                            .or_else(|| q.get("last_answer"))
                            .and_then(|a| a.get("createdAt").or_else(|| a.get("created_at")))
                            .and_then(|v| v.as_str());
                        let Some(last) = last else { continue };
                        if !is_after(last, &since) {
                            continue;
                        }
                        let answers =
                            match client.get(&format!("{}/{}/answers", qpath, encode(&qid))) {
                                Ok(v) => data_array(&v),
                                Err(e) => {
                                    eprintln!(
                                        "[spikes watch] could not fetch answers for {}: {}",
                                        qid, e
                                    );
                                    continue;
                                }
                            };
                        let question = serde_json::json!({
                            "id": qid,
                            "title": q.get("title").cloned().unwrap_or(serde_json::Value::Null),
                        });
                        for a in answers {
                            let aid = a
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let created = a
                                .get("createdAt")
                                .or_else(|| a.get("created_at"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            if !is_after(&created, &since)
                                || seen_answers.get(&aid) == Some(&created)
                            {
                                continue;
                            }
                            seen_answers.insert(aid, created);
                            deliver(
                                &WatchEvent::QuestionAnswered {
                                    question: question.clone(),
                                    answer: a,
                                },
                                options.exec.as_deref(),
                            )?;
                        }
                    }
                }
                Err(e) => {
                    if !questions_warned {
                        eprintln!("[spikes watch] questions unavailable: {}", e);
                        questions_warned = true;
                    }
                }
            }
        }

        // Advance the cursor and the stamp only after a successful spikes poll;
        // a failed poll keeps `since` so nothing updated meanwhile is skipped.
        if let Some(next) = next_cursor(poll_ok, newest_marker, &since, &poll_started) {
            since = next;
            let _ = state::stamp(&endpoint, &cred.token);
        }

        if options.once {
            break;
        }
        std::thread::sleep(interval);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_poll_keeps_cursor() {
        assert_eq!(
            next_cursor(
                false,
                None,
                "2026-09-15T12:00:00.000Z",
                "2026-09-15T12:00:30.000Z"
            ),
            None
        );
        assert_eq!(
            next_cursor(
                false,
                Some("2026-09-15T12:00:10.000Z".into()),
                "2026-09-15T12:00:00.000Z",
                "2026-09-15T12:00:30.000Z"
            ),
            None
        );
    }

    #[test]
    fn successful_poll_advances_to_newest_marker_or_poll_start() {
        assert_eq!(
            next_cursor(
                true,
                Some("2026-09-15T12:00:10.000Z".into()),
                "2026-09-15T12:00:00.000Z",
                "2026-09-15T12:00:30.000Z"
            )
            .as_deref(),
            Some("2026-09-15T12:00:10.000Z")
        );
        assert_eq!(
            next_cursor(
                true,
                None,
                "2026-09-15T12:00:00.000Z",
                "2026-09-15T12:00:30.000Z"
            )
            .as_deref(),
            Some("2026-09-15T12:00:30.000Z")
        );
        // A marker older than the cursor never moves it backwards.
        assert_eq!(
            next_cursor(
                true,
                Some("2026-09-15T11:00:00.000Z".into()),
                "2026-09-15T12:00:00.000Z",
                "2026-09-15T12:00:30.000Z"
            )
            .as_deref(),
            Some("2026-09-15T12:00:30.000Z")
        );
    }

    fn spike(id: &str, created: &str, updated: &str) -> Spike {
        Spike {
            id: id.to_string(),
            created_at: Some(created.to_string()),
            updated_at: Some(updated.to_string()),
            timestamp: created.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_resolve_since_last_uses_stamp() {
        let state = SyncState {
            last_pulled_at: Some("2026-09-14T10:00:00Z".to_string()),
            ..Default::default()
        };
        let now = Utc::now();
        assert_eq!(
            resolve_since(Some("last"), &state, now).unwrap(),
            "2026-09-14T10:00:00Z"
        );
        assert_eq!(
            resolve_since(None, &state, now).unwrap(),
            "2026-09-14T10:00:00Z"
        );
    }

    #[test]
    fn test_resolve_since_without_stamp_is_now() {
        let now = Utc::now();
        let got = resolve_since(None, &SyncState::default(), now).unwrap();
        assert!(got.starts_with(&now.format("%Y-%m-%dT%H:%M").to_string()));
    }

    #[test]
    fn test_resolve_since_explicit_and_invalid() {
        let now = Utc::now();
        assert_eq!(
            resolve_since(Some("2026-01-01T00:00:00Z"), &SyncState::default(), now).unwrap(),
            "2026-01-01T00:00:00Z"
        );
        assert!(resolve_since(Some("yesterday"), &SyncState::default(), now).is_err());
    }

    #[test]
    fn test_classify_created_vs_updated_and_dedup() {
        let since = "2026-09-14T10:00:00Z";
        let mut seen = HashMap::new();
        let spikes = vec![
            spike("a", "2026-09-14T11:00:00Z", "2026-09-14T11:00:00Z"),
            spike("b", "2026-09-01T00:00:00Z", "2026-09-14T12:00:00Z"),
        ];
        let (events, newest) = classify_spikes(spikes.clone(), since, &mut seen);
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], WatchEvent::SpikeCreated { .. }));
        assert!(matches!(events[1], WatchEvent::SpikeUpdated { .. }));
        assert_eq!(newest.as_deref(), Some("2026-09-14T12:00:00Z"));

        // Same markers again: nothing new
        let (again, _) = classify_spikes(spikes, since, &mut seen);
        assert!(again.is_empty());

        // Changed marker: reported as updated
        let (changed, _) = classify_spikes(
            vec![spike("a", "2026-09-14T11:00:00Z", "2026-09-14T13:00:00Z")],
            "2026-09-14T12:00:00Z",
            &mut seen,
        );
        assert_eq!(changed.len(), 1);
        assert!(matches!(changed[0], WatchEvent::SpikeUpdated { .. }));
    }

    #[test]
    fn test_event_serialization_shape() {
        let e = WatchEvent::SpikeCreated {
            spike: spike("x", "t", "t"),
        };
        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&e).unwrap()).unwrap();
        assert_eq!(v["event"], "spike.created");
        assert_eq!(v["spike"]["id"], "x");
    }
}
