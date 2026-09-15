//! `spikes status` — which credential is in use, whether it works, and how
//! fresh the local cache is. Exits 1 when the credential is unusable.

use serde::Serialize;

use crate::api::{resolve_credential, resolve_endpoint, ApiClient, CredentialSource};
use crate::error::{Error, Result};
use crate::state::{credential_prefix, format_age, SyncState};

#[derive(Debug, Serialize)]
pub struct CredentialReport {
    pub source: CredentialSource,
    pub source_description: &'static str,
    pub prefix: String,
    pub kind: &'static str,
}

#[derive(Debug, Serialize, Default)]
pub struct AuthReport {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// Raw `GET /me` response when the credential works
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct CacheReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_pulled_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<String>,
    pub stale: bool,
}

#[derive(Debug, Serialize)]
pub struct StatusReport {
    pub endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<CredentialReport>,
    pub auth: AuthReport,
    pub cache: CacheReport,
}

/// Build the report without printing (used by `run` and tests).
pub fn build_report() -> Result<StatusReport> {
    let endpoint = resolve_endpoint();
    let credential = resolve_credential()?;
    let state = SyncState::load();
    let cache = CacheReport {
        last_pulled_at: state.last_pulled_at.clone(),
        age: state.age().map(format_age),
        stale: state.is_stale(),
    };

    let (credential_report, auth) = match credential {
        None => (
            None,
            AuthReport {
                ok: false,
                code: Some("NO_CREDENTIAL".to_string()),
                message: Some(Error::NoCredential.to_string()),
                ..Default::default()
            },
        ),
        Some(cred) => {
            let report = CredentialReport {
                source: cred.source,
                source_description: cred.source.describe(),
                prefix: credential_prefix(&cred.token),
                kind: if cred.is_api_key() {
                    "api_key"
                } else {
                    "user_token"
                },
            };
            let client = ApiClient::new(&endpoint, &cred.token);
            let auth = match client.get("/me") {
                Ok(identity) => AuthReport {
                    ok: true,
                    identity: Some(identity),
                    ..Default::default()
                },
                Err(Error::AuthRevoked(at)) => AuthReport {
                    ok: false,
                    code: Some("TOKEN_REVOKED".to_string()),
                    message: Some(Error::AuthRevoked(at.clone()).to_string()),
                    revoked_at: at,
                    ..Default::default()
                },
                Err(Error::AuthExpired(at)) => AuthReport {
                    ok: false,
                    code: Some("TOKEN_EXPIRED".to_string()),
                    message: Some(Error::AuthExpired(at.clone()).to_string()),
                    expires_at: at,
                    ..Default::default()
                },
                Err(Error::AuthFailed) => AuthReport {
                    ok: false,
                    code: Some("AUTH_FAILED".to_string()),
                    message: Some(
                        "Credential not recognised by the server. Check the token or create a new key."
                            .to_string(),
                    ),
                    ..Default::default()
                },
                Err(e) => AuthReport {
                    ok: false,
                    code: Some("UNVERIFIED".to_string()),
                    message: Some(format!("Could not verify credential: {}", e)),
                    ..Default::default()
                },
            };
            (Some(report), auth)
        }
    };

    Ok(StatusReport {
        endpoint,
        credential: credential_report,
        auth,
        cache,
    })
}

pub fn run(json: bool) -> Result<()> {
    let report = build_report()?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("Failed to serialize to JSON")
        );
    } else {
        print_text(&report);
    }

    if !report.auth.ok {
        std::process::exit(1);
    }
    Ok(())
}

fn print_text(report: &StatusReport) {
    println!();
    println!("  Endpoint:    {}", report.endpoint);
    match &report.credential {
        Some(c) => {
            println!("  Credential:  {}… ({})", c.prefix, c.kind);
            println!("  Source:      {}", c.source_description);
        }
        None => println!("  Credential:  (none)"),
    }

    if report.auth.ok {
        let identity = report.auth.identity.as_ref();
        let email = identity
            .and_then(|i| {
                i.get("email")
                    .or_else(|| i.get("user").and_then(|u| u.get("email")))
            })
            .and_then(|v| v.as_str());
        let tier = identity
            .and_then(|i| {
                i.get("tier")
                    .or_else(|| i.get("user").and_then(|u| u.get("tier")))
            })
            .and_then(|v| v.as_str());
        let project = identity
            .and_then(|i| i.get("project_key"))
            .and_then(|v| v.as_str());
        let scopes = identity
            .and_then(|i| i.get("scopes"))
            .and_then(|v| v.as_str());
        let mut who = String::from("valid");
        if let Some(e) = email {
            who.push_str(&format!(", {}", e));
        }
        if let Some(t) = tier {
            who.push_str(&format!(" ({})", t));
        }
        println!("  Auth:        {}", who);
        if let Some(p) = project {
            println!("  Project key: scoped to '{}'", p);
        }
        if let Some(s) = scopes {
            println!("  Scopes:      {}", s);
        }
    } else {
        println!(
            "  Auth:        INVALID ({})",
            report.auth.code.as_deref().unwrap_or("UNKNOWN")
        );
        if let Some(ref at) = report.auth.revoked_at {
            println!("               revoked at {}", at);
        }
        if let Some(ref at) = report.auth.expires_at {
            println!("               expired at {}", at);
        }
        if let Some(ref m) = report.auth.message {
            println!("               {}", m);
        }
    }

    match (&report.cache.last_pulled_at, &report.cache.age) {
        (Some(ts), Some(age)) => println!(
            "  Cache:       pulled {} ago ({}){}",
            age,
            ts,
            if report.cache.stale { " — stale" } else { "" }
        ),
        _ => println!("  Cache:       never pulled"),
    }
    println!();
}
