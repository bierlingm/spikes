//! Integration tests for the feedback-loop v2 CLI surface:
//! status, pull --since/--url-prefix, reply, resolve status flags,
//! projects, versions, questions, watch, auth create-key --project.
//!
//! Every hosted call is served by wiremock; the CLI reads the endpoint and
//! token from `.spikes/config.toml` in a temp project.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;
use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

const TOKEN: &str = "sk_spikes_testkey0000000000";

fn project_with_remote(endpoint: &str, project_key: Option<&str>) -> TempDir {
    let dir = TempDir::new().unwrap();
    let spikes_dir = dir.path().join(".spikes");
    fs::create_dir_all(&spikes_dir).unwrap();
    let mut config = String::new();
    if let Some(key) = project_key {
        config.push_str(&format!("[project]\nkey = \"{}\"\n\n", key));
    }
    config.push_str(&format!(
        "[remote]\nendpoint = \"{}\"\ntoken = \"{}\"\n",
        endpoint, TOKEN
    ));
    fs::write(spikes_dir.join("config.toml"), config).unwrap();
    fs::write(spikes_dir.join("feedback.jsonl"), "").unwrap();
    dir
}

fn spike_json(id: &str, url: &str, updated: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "type": "element",
        "projectKey": "yvs",
        "page": "index.html",
        "url": url,
        "reviewer": {"id": "r1", "name": "Tyler"},
        "selector": ".hero",
        "rating": "meh",
        "comments": "Too big",
        "timestamp": "2026-09-12T10:00:00.000Z",
        "viewport": {"width": 1200, "height": 800},
        "resolved": false,
        "status": "open",
        "addressedIn": null,
        "createdAt": "2026-09-12T10:00:00.000Z",
        "updatedAt": updated,
        "replyCount": 0,
        "lastReply": null,
        "version": "v0-3"
    })
}

fn spikes_cmd(dir: &TempDir) -> assert_cmd::Command {
    let mut cmd = cargo_bin_cmd!("spikes");
    cmd.current_dir(dir.path())
        // Isolate the global auth file and env token
        .env("XDG_CONFIG_HOME", dir.path().join("xdg"))
        .env_remove("SPIKES_TOKEN")
        .env_remove("SPIKES_API_URL");
    cmd
}

// ---------------------------------------------------------------------------
// spikes status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn status_reports_valid_credential_and_source() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/me"))
        .and(matchers::header(
            "Authorization",
            format!("Bearer {}", TOKEN).as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "type": "api_key", "scopes": "full", "project_key": "yvs",
            "user": {"email": "m@example.com", "tier": "pro"}
        })))
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), Some("yvs"));

    spikes_cmd(&dir)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("valid, m@example.com (pro)"))
        .stdout(predicate::str::contains("scoped to 'yvs'"))
        .stdout(predicate::str::contains(
            ".spikes/config.toml [remote].token",
        ))
        .stdout(predicate::str::contains("never pulled"));
}

#[tokio::test]
async fn status_reports_revoked_token_and_exits_1() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/me"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": "Revoked", "code": "TOKEN_REVOKED", "revoked_at": "2026-09-01T00:00:00Z"
        })))
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);

    spikes_cmd(&dir)
        .arg("status")
        .arg("--json")
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("\"code\": \"TOKEN_REVOKED\""))
        .stdout(predicate::str::contains(
            "\"revoked_at\": \"2026-09-01T00:00:00Z\"",
        ))
        .stdout(predicate::str::contains("\"source\": \"repo_config\""));
}

#[tokio::test]
async fn status_reports_expired_token() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/me"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": "Expired", "code": "TOKEN_EXPIRED", "expires_at": "2026-08-01T00:00:00Z"
        })))
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);

    spikes_cmd(&dir)
        .arg("status")
        .assert()
        .failure()
        .stdout(predicate::str::contains("INVALID (TOKEN_EXPIRED)"))
        .stdout(predicate::str::contains("expired at 2026-08-01T00:00:00Z"));
}

#[tokio::test]
async fn status_reports_generic_auth_failed() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/me"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": "Invalid or revoked bearer token", "code": "AUTH_FAILED"
        })))
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);

    spikes_cmd(&dir)
        .arg("status")
        .assert()
        .failure()
        .stdout(predicate::str::contains("INVALID (AUTH_FAILED)"));
}

#[test]
fn status_without_credential_exits_1() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join(".spikes")).unwrap();
    spikes_cmd(&dir)
        .arg("status")
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("Credential:  (none)"))
        .stdout(predicate::str::contains("NO_CREDENTIAL"));
}

// ---------------------------------------------------------------------------
// pull --since / --url-prefix and the state stamp
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pull_writes_stamp_and_since_last_uses_it() {
    let server = MockServer::start().await;
    // First pull: no since
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/spikes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [spike_json("aaaa1111", "https://x/v0-3/", "2026-09-12T10:00:00.000Z")],
            "next_cursor": null
        })))
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);

    spikes_cmd(&dir)
        .arg("pull")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"new\":1"));

    let state: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.path().join(".spikes/state.json")).unwrap())
            .unwrap();
    assert_eq!(state["endpoint"], server.uri());
    assert_eq!(state["credential_prefix"], "sk_spikes_te");
    let stamp = state["last_pulled_at"].as_str().unwrap().to_string();

    // Second pull with --since last must send since=<stamp> and url_prefix
    server.reset().await;
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/spikes"))
        .and(matchers::query_param("since", stamp.as_str()))
        .and(matchers::query_param("url_prefix", "https://x/v0-3/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [spike_json("aaaa1111", "https://x/v0-3/", "2026-09-14T10:00:00.000Z")],
            "next_cursor": null
        })))
        .expect(1)
        .mount(&server)
        .await;

    spikes_cmd(&dir)
        .args([
            "pull",
            "--since",
            "last",
            "--url-prefix",
            "https://x/v0-3/",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"updated\":1"))
        .stdout(predicate::str::contains("\"new\":0"));

    // The local copy was replaced in place, not duplicated
    let lines: Vec<String> = fs::read_to_string(dir.path().join(".spikes/feedback.jsonl"))
        .unwrap()
        .lines()
        .map(|l| l.to_string())
        .collect();
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("2026-09-14T10:00:00.000Z"));
}

#[test]
fn pull_rejects_invalid_since() {
    let dir = project_with_remote("http://127.0.0.1:9", None);
    spikes_cmd(&dir)
        .args(["pull", "--since", "yesterday"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid --since value"));
}

#[test]
fn list_warns_when_remote_configured_and_never_pulled() {
    let dir = project_with_remote("http://127.0.0.1:9", None);
    spikes_cmd(&dir)
        .arg("list")
        .assert()
        .success()
        .stderr(predicate::str::contains("never been pulled"));
}

#[test]
fn list_does_not_warn_without_remote() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join(".spikes")).unwrap();
    fs::write(dir.path().join(".spikes/feedback.jsonl"), "").unwrap();
    spikes_cmd(&dir)
        .arg("list")
        .assert()
        .success()
        .stderr(predicate::str::contains("pulled").not());
}

#[test]
fn list_warns_when_stamp_is_old() {
    let dir = project_with_remote("http://127.0.0.1:9", None);
    fs::write(
        dir.path().join(".spikes/state.json"),
        r#"{"last_pulled_at":"2026-01-01T00:00:00Z","endpoint":"x","credential_prefix":"y"}"#,
    )
    .unwrap();
    spikes_cmd(&dir)
        .arg("list")
        .assert()
        .success()
        .stderr(predicate::str::contains("Local cache last pulled"))
        .stderr(predicate::str::contains("days ago"));
}

// ---------------------------------------------------------------------------
// reply
// ---------------------------------------------------------------------------

#[tokio::test]
async fn reply_posts_body_status_and_version() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("POST"))
        .and(matchers::path("/spikes/aaaa1111/replies"))
        .and(matchers::header(
            "Authorization",
            format!("Bearer {}", TOKEN).as_str(),
        ))
        .and(matchers::body_json(serde_json::json!({
            "body": "Done, moved the hero up",
            "version_label": "v0.5",
            "status": "addressed",
            "addressed_in": "v0.5"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": "rep1", "spikeId": "aaaa1111", "authorType": "agent",
            "authorName": "Builder", "body": "Done, moved the hero up",
            "versionLabel": "v0.5", "createdAt": "2026-09-14T12:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);
    // local cache holds the spike, so a prefix resolves to the full id
    fs::write(
        dir.path().join(".spikes/feedback.jsonl"),
        format!(
            "{}\n",
            spike_json("aaaa1111", "https://x/", "2026-09-12T10:00:00.000Z")
        ),
    )
    .unwrap();

    spikes_cmd(&dir)
        .args([
            "reply",
            "aaaa",
            "Done, moved the hero up",
            "--version",
            "v0.5",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Replied to spike aaaa1111"))
        .stdout(predicate::str::contains("Reply ID: rep1"))
        .stdout(predicate::str::contains("Status:   addressed"));

    // local cache updated
    let cache = fs::read_to_string(dir.path().join(".spikes/feedback.jsonl")).unwrap();
    assert!(cache.contains("\"status\":\"addressed\""));
    assert!(cache.contains("\"addressedIn\":\"v0.5\""));
    assert!(cache.contains("\"replyCount\":1"));
}

#[tokio::test]
async fn reply_with_wont_do_status_and_name() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("POST"))
        .and(matchers::path("/spikes/full-id-1/replies"))
        .and(matchers::body_json(serde_json::json!({
            "body": "Out of scope for this round",
            "author_name": "Moritz",
            "status": "wont_do"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": "rep2", "body": "Out of scope for this round"
        })))
        .expect(1)
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);

    spikes_cmd(&dir)
        .args([
            "reply",
            "full-id-1",
            "Out of scope for this round",
            "--status",
            "wont-do",
            "--name",
            "Moritz",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"spike_id\": \"full-id-1\""));
}

#[test]
fn reply_rejects_bad_status() {
    let dir = project_with_remote("http://127.0.0.1:9", None);
    spikes_cmd(&dir)
        .args(["reply", "abcd", "text", "--status", "done"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid status 'done'"));
}

// ---------------------------------------------------------------------------
// resolve with status flags
// ---------------------------------------------------------------------------

#[tokio::test]
async fn resolve_in_sends_status_addressed_with_label() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("PATCH"))
        .and(matchers::path("/spikes/aaaa1111"))
        .and(matchers::body_json(serde_json::json!({
            "status": "addressed", "addressed_in": "v0.5"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(spike_json(
            "aaaa1111",
            "https://x/",
            "2026-09-14T10:00:00.000Z",
        )))
        .expect(1)
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);
    fs::write(
        dir.path().join(".spikes/feedback.jsonl"),
        format!(
            "{}\n",
            spike_json("aaaa1111", "https://x/", "2026-09-12T10:00:00.000Z")
        ),
    )
    .unwrap();

    spikes_cmd(&dir)
        .args(["resolve", "aaaa", "--in", "v0.5"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Resolved spike aaaa1111 (addressed in v0.5)",
        ));

    let cache = fs::read_to_string(dir.path().join(".spikes/feedback.jsonl")).unwrap();
    assert!(cache.contains("\"status\":\"addressed\""));
    assert!(cache.contains("\"addressedIn\":\"v0.5\""));
    assert!(cache.contains("\"resolved\":true"));
}

#[tokio::test]
async fn resolve_plain_still_sends_resolved_true() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("PATCH"))
        .and(matchers::path("/spikes/aaaa1111"))
        .and(matchers::body_json(serde_json::json!({ "resolved": true })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
        .expect(1)
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);

    spikes_cmd(&dir)
        .args(["resolve", "aaaa1111"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved spike aaaa1111."));
}

#[tokio::test]
async fn resolve_wont_do_and_undo() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("PATCH"))
        .and(matchers::path("/spikes/s1s1s1s1"))
        .and(matchers::body_json(
            serde_json::json!({ "status": "wont_do" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(matchers::method("PATCH"))
        .and(matchers::path("/spikes/s2s2s2s2"))
        .and(matchers::body_json(serde_json::json!({ "status": "open" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
        .expect(1)
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);

    spikes_cmd(&dir)
        .args(["resolve", "s1s1s1s1", "--wont-do"])
        .assert()
        .success()
        .stdout(predicate::str::contains("won't do"));
    spikes_cmd(&dir)
        .args(["resolve", "--undo", "s2s2s2s2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Reopened spike s2s2s2s2"));
}

#[test]
fn resolve_local_only_without_remote_keeps_working() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join(".spikes")).unwrap();
    fs::write(
        dir.path().join(".spikes/feedback.jsonl"),
        format!(
            "{}\n",
            spike_json("aaaa1111", "https://x/", "2026-09-12T10:00:00.000Z")
        ),
    )
    .unwrap();
    spikes_cmd(&dir)
        .args(["resolve", "aaaa", "--wont-do", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"wont_do\""));
}

#[tokio::test]
async fn resolve_falls_back_to_local_when_remote_returns_404() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("PATCH"))
        .and(matchers::path("/spikes/aaaa1111"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": "Spike not found", "code": "SPIKE_NOT_FOUND"
        })))
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);
    fs::write(
        dir.path().join(".spikes/feedback.jsonl"),
        format!(
            "{}\n",
            spike_json("aaaa1111", "https://x/", "2026-09-12T10:00:00.000Z")
        ),
    )
    .unwrap();
    let path = dir.path().join(".spikes/feedback.jsonl");
    tokio::task::spawn_blocking(move || {
        spikes_cmd(&dir)
            .args(["resolve", "aaaa", "--wont-do", "--json"])
            .assert()
            .success()
            .stderr(predicate::str::contains("resolved locally only"))
            .stdout(predicate::str::contains("\"status\": \"wont_do\""));
        let saved = fs::read_to_string(&path).unwrap();
        assert!(
            saved.contains("\"status\":\"wont_do\"") || saved.contains("\"status\": \"wont_do\"")
        );
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn resolve_propagates_remote_404_without_local_match() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("PATCH"))
        .and(matchers::path("/spikes/zzzz9999"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": "Spike not found", "code": "SPIKE_NOT_FOUND"
        })))
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);
    tokio::task::spawn_blocking(move || {
        spikes_cmd(&dir)
            .args(["resolve", "zzzz9999"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Spike not found"));
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn resolve_does_not_fall_back_on_revoked_credential() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("PATCH"))
        .and(matchers::path("/spikes/aaaa1111"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": "revoked", "code": "TOKEN_REVOKED", "revoked_at": "2026-09-01T00:00:00Z"
        })))
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);
    fs::write(
        dir.path().join(".spikes/feedback.jsonl"),
        format!(
            "{}\n",
            spike_json("aaaa1111", "https://x/", "2026-09-12T10:00:00.000Z")
        ),
    )
    .unwrap();
    tokio::task::spawn_blocking(move || {
        spikes_cmd(&dir)
            .args(["resolve", "aaaa"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("revoked"));
    })
    .await
    .unwrap();
}

#[test]
fn resolve_rejects_conflicting_flags() {
    let dir = project_with_remote("http://127.0.0.1:9", None);
    spikes_cmd(&dir)
        .args(["resolve", "abcd", "--in", "v1", "--wont-do"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be combined"));
}

// ---------------------------------------------------------------------------
// projects
// ---------------------------------------------------------------------------

#[tokio::test]
async fn projects_create_posts_and_saves_key_to_config() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("POST"))
        .and(matchers::path("/projects"))
        .and(matchers::body_json(serde_json::json!({
            "key": "prosser", "allowed_origins": ["https://statecraft.systems"]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": "p1", "key": "prosser", "allowed_origins": ["https://statecraft.systems"]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);

    spikes_cmd(&dir)
        .args([
            "projects",
            "create",
            "prosser",
            "--origin",
            "https://statecraft.systems",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Project created"))
        .stdout(predicate::str::contains("[project].key set"));

    let config = fs::read_to_string(dir.path().join(".spikes/config.toml")).unwrap();
    assert!(config.contains("key = \"prosser\""));
    // the remote token survived the config rewrite
    assert!(config.contains(TOKEN));
}

#[tokio::test]
async fn projects_list_renders_table() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/me/projects"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"key": "yvs", "allowed_origins": ["https://*"], "spike_count": 6, "last_activity": "2026-09-13T08:00:00Z"}],
            "pagination": {"page": 1}
        })))
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);

    spikes_cmd(&dir)
        .args(["projects", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("yvs"))
        .stdout(predicate::str::contains("https://*"))
        .stdout(predicate::str::contains("2026-09-13"));
}

// ---------------------------------------------------------------------------
// versions
// ---------------------------------------------------------------------------

#[tokio::test]
async fn versions_add_list_notes() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("POST"))
        .and(matchers::path("/me/projects/yvs/versions"))
        .and(matchers::body_json(serde_json::json!({
            "label": "v0.5", "url_prefix": "/prosser/versions/v0-5/", "notes": "addresses 3, 5, 6"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": "v1", "label": "v0.5", "urlPrefix": "/prosser/versions/v0-5/", "notes": "addresses 3, 5, 6", "spikeCount": 0
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/me/projects/yvs/versions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"label": "v0.5", "urlPrefix": "/prosser/versions/v0-5/", "notes": "addresses 3, 5, 6", "spikeCount": 4, "openCount": 1}]
        })))
        .mount(&server)
        .await;
    Mock::given(matchers::method("PATCH"))
        .and(matchers::path("/me/projects/yvs/versions/v0.5"))
        .and(matchers::body_json(serde_json::json!({ "notes": "final" })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "label": "v0.5", "notes": "final" })),
        )
        .expect(1)
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), Some("yvs"));

    spikes_cmd(&dir)
        .args([
            "versions",
            "add",
            "v0.5",
            "--prefix",
            "/prosser/versions/v0-5/",
            "--notes",
            "addresses 3, 5, 6",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Version added to 'yvs'"));
    spikes_cmd(&dir)
        .args(["versions", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("v0.5"))
        .stdout(predicate::str::contains("addresses 3, 5, 6"));
    spikes_cmd(&dir)
        .args(["versions", "notes", "v0.5", "final"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Notes updated"));
}

#[test]
fn versions_requires_project_key() {
    let dir = project_with_remote("http://127.0.0.1:9", None);
    spikes_cmd(&dir)
        .args(["versions", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No project key configured"));
}

// ---------------------------------------------------------------------------
// questions
// ---------------------------------------------------------------------------

#[tokio::test]
async fn questions_add_list_answers_close() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("POST"))
        .and(matchers::path("/me/projects/yvs/questions"))
        .and(matchers::body_json(serde_json::json!({ "title": "Pool hours?", "body": "Open at night?" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": "q1", "title": "Pool hours?", "body": "Open at night?", "status": "open", "answerCount": 0
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/me/projects/yvs/questions"))
        .and(matchers::query_param("status", "open"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "q1", "title": "Pool hours?", "status": "open", "answerCount": 1,
                      "lastAnswer": {"reviewerName": "Tyler", "body": "Until 10pm", "createdAt": "2026-09-14T09:00:00Z"}}]
        })))
        .mount(&server)
        .await;
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/me/projects/yvs/questions/q1/answers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "a1", "reviewer": {"id": "r1", "name": "Tyler"}, "body": "Until 10pm", "createdAt": "2026-09-14T09:00:00Z"}]
        })))
        .mount(&server)
        .await;
    Mock::given(matchers::method("PATCH"))
        .and(matchers::path("/me/projects/yvs/questions/q1"))
        .and(matchers::body_json(
            serde_json::json!({ "status": "closed" }),
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "id": "q1", "status": "closed" })),
        )
        .expect(1)
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), Some("yvs"));

    spikes_cmd(&dir)
        .args([
            "questions",
            "add",
            "Pool hours?",
            "--body",
            "Open at night?",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("ID:     q1"));
    spikes_cmd(&dir)
        .args(["questions", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pool hours?"))
        .stdout(predicate::str::contains("Tyler: Until 10pm"));
    spikes_cmd(&dir)
        .args(["questions", "answers", "q1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Tyler (2026-09-14T09:00:00Z)"))
        .stdout(predicate::str::contains("Until 10pm"));
    spikes_cmd(&dir)
        .args(["questions", "close", "q1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Question q1 closed"));
}

// ---------------------------------------------------------------------------
// watch
// ---------------------------------------------------------------------------

#[tokio::test]
async fn watch_once_prints_events_and_stamps() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/spikes"))
        .and(matchers::query_param("since", "2026-09-14T00:00:00Z"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                spike_json("new00001", "https://x/v0-5/", "2026-09-14T11:00:00.000Z"),
                {
                    "id": "old00001", "type": "page", "projectKey": "yvs", "page": "index.html",
                    "url": "https://x/v0-3/", "reviewer": {"id": "r1", "name": "Tyler"},
                    "rating": null, "comments": "Old one", "timestamp": "2026-09-01T00:00:00Z",
                    "viewport": null, "status": "addressed", "createdAt": "2026-09-01T00:00:00Z",
                    "updatedAt": "2026-09-14T12:00:00.000Z", "replyCount": 1,
                    "lastReply": {"id": "r", "authorType": "agent", "authorName": "Builder", "body": "Done", "createdAt": "2026-09-14T12:00:00.000Z"}
                }
            ],
            "next_cursor": null
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/me/projects/yvs/questions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "q1", "title": "Pool hours?", "answerCount": 1,
                      "lastAnswer": {"reviewerName": "Tyler", "body": "Until 10pm", "createdAt": "2026-09-14T09:00:00Z"}}]
        })))
        .mount(&server)
        .await;
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/me/projects/yvs/questions/q1/answers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "a1", "reviewer": {"id": "r1", "name": "Tyler"}, "body": "Until 10pm", "createdAt": "2026-09-14T09:00:00Z"}]
        })))
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), Some("yvs"));

    // new00001 has createdAt before since but createdAt != updatedAt -> updated;
    // to get a "created" event, createdAt must be after since: adjust via a
    // spike whose createdAt is 2026-09-14T11:00 (spike_json fixes createdAt to
    // 2026-09-12), so both are "updated" here. The question answer is emitted.
    let output = spikes_cmd(&dir)
        .args(["watch", "--once", "--since", "2026-09-14T00:00:00Z"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).unwrap();
    let lines: Vec<serde_json::Value> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("each line is JSON"))
        .collect();
    assert_eq!(lines.len(), 3, "got: {}", text);
    assert_eq!(lines[0]["event"], "spike.updated");
    assert_eq!(lines[0]["spike"]["id"], "new00001");
    assert_eq!(lines[1]["event"], "spike.updated");
    assert_eq!(lines[1]["spike"]["lastReply"]["body"], "Done");
    assert_eq!(lines[2]["event"], "question.answered");
    assert_eq!(lines[2]["question"]["id"], "q1");
    assert_eq!(lines[2]["answer"]["body"], "Until 10pm");

    let state: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.path().join(".spikes/state.json")).unwrap())
            .unwrap();
    assert!(state["last_pulled_at"].is_string());
}

#[tokio::test]
async fn watch_exec_pipes_json_to_command() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/spikes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [spike_json("exec0001", "https://x/", "2026-09-14T11:00:00.000Z")]
        })))
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);
    let sink = dir.path().join("events.log");

    spikes_cmd(&dir)
        .args([
            "watch",
            "--once",
            "--since",
            "2026-09-14T00:00:00Z",
            "--exec",
            &format!("cat >> {}", sink.display()),
        ])
        .assert()
        .success();

    let logged = fs::read_to_string(&sink).unwrap();
    assert!(logged.contains("\"event\":\"spike.updated\""));
    assert!(logged.contains("exec0001"));
}

#[tokio::test]
async fn watch_exec_failure_is_logged_not_fatal() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/spikes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [spike_json("fail0001", "https://x/", "2026-09-14T11:00:00.000Z")]
        })))
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);

    spikes_cmd(&dir)
        .args([
            "watch",
            "--once",
            "--since",
            "2026-09-14T00:00:00Z",
            "--exec",
            "exit 3",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("--exec exited with"));
}

#[tokio::test]
async fn watch_revoked_credential_is_fatal() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/spikes"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": "Revoked", "code": "TOKEN_REVOKED", "revoked_at": "2026-09-01T00:00:00Z"
        })))
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);

    spikes_cmd(&dir)
        .args(["watch", "--once"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Credential revoked at 2026-09-01T00:00:00Z",
        ));
}

// ---------------------------------------------------------------------------
// auth create-key --project
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_key_with_project_sends_project_key_and_bearer() {
    let server = MockServer::start().await;
    Mock::given(matchers::method("POST"))
        .and(matchers::path("/auth/api-key"))
        .and(matchers::header("Authorization", "Bearer user-login-token"))
        .and(matchers::body_json(serde_json::json!({ "name": "agent", "project_key": "yvs" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "ok": true, "api_key": "sk_spikes_projkey000000000", "key_id": "key_1",
            "name": "agent", "scopes": "full", "created_at": "2026-09-14T12:00:00Z", "project_key": "yvs"
        })))
        .expect(1)
        .mount(&server)
        .await;
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join(".spikes")).unwrap();

    spikes_cmd(&dir)
        .env("SPIKES_TOKEN", "user-login-token")
        .env("SPIKES_API_URL", server.uri())
        .args([
            "auth",
            "create-key",
            "--name",
            "agent",
            "--project",
            "yvs",
            "--save",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Project:yvs"))
        .stdout(predicate::str::contains("Saved to .spikes/config.toml"));

    let config = fs::read_to_string(dir.path().join(".spikes/config.toml")).unwrap();
    assert!(config.contains("token = \"sk_spikes_projkey000000000\""));
    assert!(config.contains("hosted = true"));
    assert!(config.contains("key = \"yvs\""));
}

#[test]
fn create_key_with_project_requires_a_credential() {
    let dir = TempDir::new().unwrap();
    spikes_cmd(&dir)
        .env("SPIKES_API_URL", "http://127.0.0.1:9")
        .args(["auth", "create-key", "--project", "yvs"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "needs a user token or an account key",
        ));
}

#[tokio::test]
async fn create_key_with_project_refuses_project_scoped_credential() {
    let server = MockServer::start().await;
    // The configured [remote].token is itself a project key: /me says so.
    Mock::given(matchers::method("GET"))
        .and(matchers::path("/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "type": "api_key", "scopes": "full", "project_key": "other",
            "user": { "id": "u1", "email": "o@x.y", "tier": "pro" }
        })))
        .mount(&server)
        .await;
    Mock::given(matchers::method("POST"))
        .and(matchers::path("/auth/api-key"))
        .respond_with(ResponseTemplate::new(201))
        .expect(0)
        .mount(&server)
        .await;
    let dir = project_with_remote(&server.uri(), None);
    tokio::task::spawn_blocking(move || {
        spikes_cmd(&dir)
            .args(["auth", "create-key", "--project", "yvs"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("project-scoped key"));
    })
    .await
    .unwrap();
}

// ---------------------------------------------------------------------------
// MCP: server instructions carry the turn-start pattern
// (tool listing over HTTP is covered in tests/mcp_integration.rs; the stdio
// server stops at EOF before answering a second request, and the v2 tool
// methods have unit tests in src/commands/mcp.rs)
// ---------------------------------------------------------------------------

#[test]
fn mcp_initialize_returns_turn_start_instructions() {
    let input = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;
    let dir = TempDir::new().unwrap();
    spikes_cmd(&dir)
        .args(["mcp", "serve"])
        .write_stdin(input)
        .assert()
        .stdout(predicate::str::contains("\"instructions\""))
        .stdout(predicate::str::contains("At the start of every session"))
        .stdout(predicate::str::contains("reply_to_spike"));
}
