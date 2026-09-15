//! Shared client for the hosted API (spikes.sh or a self-hosted worker).
//!
//! Credential precedence (contract §9):
//! 1. `[remote] token` in `.spikes/config.toml`
//! 2. `SPIKES_TOKEN` environment variable
//! 3. the global auth file (`spikes login` token, then a stored API key)
//!
//! Endpoint: `[remote] endpoint` / `hosted = true` in `.spikes/config.toml`,
//! otherwise `SPIKES_API_URL`, otherwise `https://spikes.sh`.

use serde::Serialize;

use crate::auth::{get_api_base, AuthConfig};
use crate::config::Config;
use crate::error::{map_http_error, map_network_error, Error, Result};

/// Where a credential was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialSource {
    /// `[remote] token` in `.spikes/config.toml`
    RepoConfig,
    /// `SPIKES_TOKEN` environment variable
    EnvVar,
    /// Bearer token from the global auth file (`spikes login`)
    AuthFile,
    /// API key from the global auth file (`spikes auth create-key`)
    AuthFileApiKey,
}

impl CredentialSource {
    pub fn describe(&self) -> &'static str {
        match self {
            CredentialSource::RepoConfig => ".spikes/config.toml [remote].token",
            CredentialSource::EnvVar => "SPIKES_TOKEN environment variable",
            CredentialSource::AuthFile => "global auth file (spikes login)",
            CredentialSource::AuthFileApiKey => "global auth file (API key)",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Credential {
    pub token: String,
    pub source: CredentialSource,
}

impl Credential {
    /// True for `sk_spikes_` API keys (account- or project-scoped).
    pub fn is_api_key(&self) -> bool {
        self.token.starts_with("sk_spikes_")
    }
}

/// Resolve the credential to use, following the contract precedence.
pub fn resolve_credential() -> Result<Option<Credential>> {
    let config = Config::load()?;
    if let Some(token) = config.remote.token.filter(|t| !t.is_empty()) {
        return Ok(Some(Credential {
            token,
            source: CredentialSource::RepoConfig,
        }));
    }
    if let Ok(token) = std::env::var("SPIKES_TOKEN") {
        if !token.is_empty() {
            return Ok(Some(Credential {
                token,
                source: CredentialSource::EnvVar,
            }));
        }
    }
    let auth = AuthConfig::load()?;
    if let Some(token) = auth.auth.token.filter(|t| !t.is_empty()) {
        return Ok(Some(Credential {
            token,
            source: CredentialSource::AuthFile,
        }));
    }
    if let Some(key) = AuthConfig::load_api_key().filter(|k| !k.is_empty()) {
        return Ok(Some(Credential {
            token: key,
            source: CredentialSource::AuthFileApiKey,
        }));
    }
    Ok(None)
}

/// Resolve the API base URL: repo config first, then env/default.
pub fn resolve_endpoint() -> String {
    let from_config = Config::load().ok().and_then(|c| c.effective_endpoint());
    let base = from_config.unwrap_or_else(get_api_base);
    base.trim_end_matches('/')
        .trim_end_matches("/spikes")
        .to_string()
}

/// The project key from `[project].key`, or a clear error.
pub fn require_project_key() -> Result<String> {
    let config = Config::load()?;
    config
        .project
        .key
        .filter(|k| !k.is_empty())
        .ok_or(Error::ProjectKeyMissing)
}

/// Minimal JSON client for the hosted API.
#[derive(Debug, Clone)]
pub struct ApiClient {
    pub base: String,
    pub token: String,
}

impl ApiClient {
    pub fn new(base: &str, token: &str) -> Self {
        Self {
            base: base.trim_end_matches('/').to_string(),
            token: token.to_string(),
        }
    }

    /// Build a client from the resolved credential and endpoint.
    pub fn from_env() -> Result<Self> {
        let cred = resolve_credential()?.ok_or(Error::NoCredential)?;
        Ok(Self::new(&resolve_endpoint(), &cred.token))
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}/{}", self.base, path.trim_start_matches('/'))
    }

    pub fn get(&self, path: &str) -> Result<serde_json::Value> {
        self.request("GET", path, None)
    }

    pub fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        self.request("POST", path, Some(body))
    }

    pub fn patch(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        self.request("PATCH", path, Some(body))
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let url = self.url(path);
        let req = ureq::request(method, &url)
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("Accept", "application/json");

        let result = match body {
            Some(b) => req.set("Content-Type", "application/json").send_json(b),
            None => req.call(),
        };

        let response = match result {
            Ok(resp) => resp,
            Err(ureq::Error::Status(status, response)) => {
                let text = response.into_string().ok();
                return Err(map_http_error(status, text.as_deref()));
            }
            Err(e) => return Err(map_network_error(&e.to_string())),
        };

        let status = response.status();
        let text = response
            .into_string()
            .map_err(|e| Error::RequestFailed(format!("Failed to read response: {}", e)))?;

        if !(200..300).contains(&status) {
            return Err(map_http_error(status, Some(&text)));
        }

        if text.trim().is_empty() {
            return Ok(serde_json::Value::Null);
        }
        if text.trim_start().starts_with('<') {
            return Err(Error::RequestFailed(
                "Got HTML instead of JSON. Check that the endpoint URL is correct.".to_string(),
            ));
        }
        Ok(serde_json::from_str(&text)?)
    }
}

/// Append query parameters to a path, percent-encoding values.
pub fn with_query(path: &str, params: &[(&str, Option<&str>)]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for (k, v) in params {
        if let Some(v) = v {
            parts.push(format!("{}={}", k, encode(v)));
        }
    }
    if parts.is_empty() {
        path.to_string()
    } else if path.contains('?') {
        format!("{}&{}", path, parts.join("&"))
    } else {
        format!("{}?{}", path, parts.join("&"))
    }
}

/// Percent-encode a query value (RFC 3986 unreserved characters pass through).
pub fn encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// Extract `data` array from `{ data: [...] }`, or a bare array.
pub fn data_array(value: &serde_json::Value) -> Vec<serde_json::Value> {
    if let Some(arr) = value.as_array() {
        return arr.clone();
    }
    value
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_passes_unreserved() {
        assert_eq!(encode("abc-_.~09"), "abc-_.~09");
    }

    #[test]
    fn test_encode_escapes_reserved() {
        assert_eq!(encode("2026-09-14T12:00:00Z"), "2026-09-14T12%3A00%3A00Z");
        assert_eq!(encode("https://x/y?z"), "https%3A%2F%2Fx%2Fy%3Fz");
        assert_eq!(encode("ü"), "%C3%BC");
    }

    #[test]
    fn test_with_query_skips_none() {
        assert_eq!(
            with_query("/spikes", &[("since", None), ("x", None)]),
            "/spikes"
        );
        assert_eq!(
            with_query("/spikes", &[("since", Some("a b")), ("url_prefix", None)]),
            "/spikes?since=a%20b"
        );
        assert_eq!(
            with_query("/spikes?page=1", &[("since", Some("t"))]),
            "/spikes?page=1&since=t"
        );
    }

    #[test]
    fn test_client_url_joins() {
        let c = ApiClient::new("https://spikes.sh/", "tok");
        assert_eq!(c.url("/me"), "https://spikes.sh/me");
        assert_eq!(c.url("spikes"), "https://spikes.sh/spikes");
    }

    #[test]
    fn test_data_array_handles_both_shapes() {
        let wrapped = serde_json::json!({ "data": [1, 2] });
        assert_eq!(data_array(&wrapped).len(), 2);
        let bare = serde_json::json!([1]);
        assert_eq!(data_array(&bare).len(), 1);
        let none = serde_json::json!({ "ok": true });
        assert!(data_array(&none).is_empty());
    }

    #[test]
    fn test_credential_is_api_key() {
        let c = Credential {
            token: "sk_spikes_abc".to_string(),
            source: CredentialSource::EnvVar,
        };
        assert!(c.is_api_key());
        let u = Credential {
            token: "usertoken".to_string(),
            source: CredentialSource::AuthFile,
        };
        assert!(!u.is_api_key());
    }
}
