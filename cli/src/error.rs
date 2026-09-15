use std::io;

use thiserror::Error;

/// Error codes returned by the API for rate limiting, budget, and scope enforcement
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiErrorCode {
    SpikeLimit,
    ShareLimit,
    BudgetExceeded,
    RateLimited,
    ScopeDenied,
    TokenRevoked,
    TokenExpired,
    ProjectNotFound,
    VersionExists,
    UpgradeRequired,
    Unknown(String),
}

impl std::fmt::Display for ApiErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiErrorCode::SpikeLimit => write!(f, "SPIKE_LIMIT"),
            ApiErrorCode::ShareLimit => write!(f, "SHARE_LIMIT"),
            ApiErrorCode::BudgetExceeded => write!(f, "BUDGET_EXCEEDED"),
            ApiErrorCode::RateLimited => write!(f, "RATE_LIMITED"),
            ApiErrorCode::ScopeDenied => write!(f, "SCOPE_DENIED"),
            ApiErrorCode::TokenRevoked => write!(f, "TOKEN_REVOKED"),
            ApiErrorCode::TokenExpired => write!(f, "TOKEN_EXPIRED"),
            ApiErrorCode::ProjectNotFound => write!(f, "PROJECT_NOT_FOUND"),
            ApiErrorCode::VersionExists => write!(f, "VERSION_EXISTS"),
            ApiErrorCode::UpgradeRequired => write!(f, "UPGRADE_REQUIRED"),
            ApiErrorCode::Unknown(code) => write!(f, "{}", code),
        }
    }
}

impl From<&str> for ApiErrorCode {
    fn from(s: &str) -> Self {
        match s {
            "SPIKE_LIMIT" => ApiErrorCode::SpikeLimit,
            "SHARE_LIMIT" => ApiErrorCode::ShareLimit,
            "BUDGET_EXCEEDED" => ApiErrorCode::BudgetExceeded,
            "RATE_LIMITED" => ApiErrorCode::RateLimited,
            "SCOPE_DENIED" => ApiErrorCode::ScopeDenied,
            "TOKEN_REVOKED" => ApiErrorCode::TokenRevoked,
            "TOKEN_EXPIRED" => ApiErrorCode::TokenExpired,
            "PROJECT_NOT_FOUND" => ApiErrorCode::ProjectNotFound,
            "VERSION_EXISTS" => ApiErrorCode::VersionExists,
            "UPGRADE_REQUIRED" => ApiErrorCode::UpgradeRequired,
            other => ApiErrorCode::Unknown(other.to_string()),
        }
    }
}

/// Parsed API error response
#[derive(Debug, Clone)]
pub struct ApiError {
    pub error: String,
    pub code: Option<ApiErrorCode>,
    /// `revoked_at` from a TOKEN_REVOKED response
    pub revoked_at: Option<String>,
    /// `expires_at` from a TOKEN_EXPIRED response
    pub expires_at: Option<String>,
    /// `upgrade_url` from an UPGRADE_REQUIRED response
    pub upgrade_url: Option<String>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref code) = self.code {
            write!(f, "{} (code: {})", self.error, code)
        } else {
            write!(f, "{}", self.error)
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("No .spikes/ directory found. Run 'spikes init' first.")]
    NoSpikesDir,

    #[error("Spike not found: {0}")]
    SpikeNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    // HTTP/API errors with actionable messages
    #[error("Authentication failed. Run `spikes login` to refresh your token.")]
    AuthFailed,

    #[error("Credential revoked{}. Create a new key with `spikes auth create-key` or run `spikes login`, then update .spikes/config.toml.", .0.as_deref().map(|t| format!(" at {}", t)).unwrap_or_default())]
    AuthRevoked(Option<String>),

    #[error("Credential expired{}. Create a new key with `spikes auth create-key` or run `spikes login`.", .0.as_deref().map(|t| format!(" at {}", t)).unwrap_or_default())]
    AuthExpired(Option<String>),

    #[error("No credential found. Run `spikes login`, set SPIKES_TOKEN, or add token = \"...\" under [remote] in .spikes/config.toml.")]
    NoCredential,

    #[error("No project key configured. Add key = \"<project>\" under [project] in .spikes/config.toml (or run `spikes projects create <key>`).")]
    ProjectKeyMissing,

    #[error("This feature requires a Pro or Agent plan. Upgrade at {0}")]
    UpgradeRequired(String),

    #[error("Share has reached spike limit. Upgrade at https://spikes.sh/pro")]
    SpikeLimitReached,

    #[error("You've reached the free tier limit (5 shares). Delete a share or upgrade at https://spikes.sh/pro")]
    ShareLimitReached,

    #[error("Budget exceeded: monthly spending cap reached. Raise your cap or wait for the next billing period.")]
    BudgetExceeded,

    #[error("Permission denied: your API key scope does not allow this operation.")]
    ScopeDenied,

    #[error("Files too large. Max size is 50MB. Consider removing large assets.")]
    PayloadTooLarge,

    #[error("Server error. Please try again in a moment or contact support if it persists.")]
    ServerFailure,

    #[error("Connection failed. Check your internet connection.")]
    ConnectionFailed,

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("API error: {0}")]
    ApiError(ApiError),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Map an HTTP status code and optional response body to an actionable Error.
///
/// The API returns errors in the format: `{"error": "...", "code": "SOME_CODE"}`
/// where `code` distinguishes between different types of rate limits.
///
/// # Arguments
/// * `status` - HTTP status code
/// * `body` - Optional response body (JSON)
///
/// # Returns
/// An `Error` with actionable remediation guidance.
pub fn map_http_error(status: u16, body: Option<&str>) -> Error {
    // Try to parse the body for more specific error info
    let api_error = body.and_then(parse_api_error);

    match status {
        401 => match api_error.as_ref() {
            Some(err) if err.code == Some(ApiErrorCode::TokenRevoked) => {
                Error::AuthRevoked(err.revoked_at.clone())
            }
            Some(err) if err.code == Some(ApiErrorCode::TokenExpired) => {
                Error::AuthExpired(err.expires_at.clone())
            }
            _ => Error::AuthFailed,
        },
        402 => Error::UpgradeRequired(
            api_error
                .as_ref()
                .and_then(|e| e.upgrade_url.clone())
                .unwrap_or_else(|| "https://spikes.sh/pro".to_string()),
        ),
        403 => {
            // 403 could be auth-related, feature-gated, or scope-denied
            match api_error.as_ref().and_then(|e| e.code.as_ref()) {
                Some(ApiErrorCode::ScopeDenied) => Error::ScopeDenied,
                _ => {
                    if let Some(ref err) = api_error {
                        Error::ApiError(err.clone())
                    } else {
                        Error::AuthFailed
                    }
                }
            }
        }
        404 => {
            if let Some(ref err) = api_error {
                Error::ApiError(err.clone())
            } else {
                Error::RequestFailed("Resource not found".to_string())
            }
        }
        429 => {
            // Rate limit or budget exceeded - check the code field to distinguish
            match api_error.as_ref().and_then(|e| e.code.as_ref()) {
                Some(ApiErrorCode::SpikeLimit) => Error::SpikeLimitReached,
                Some(ApiErrorCode::ShareLimit) => Error::ShareLimitReached,
                Some(ApiErrorCode::BudgetExceeded) => Error::BudgetExceeded,
                _ => Error::RequestFailed(
                    "Rate limit exceeded. Please wait a moment and try again.".to_string(),
                ),
            }
        }
        413 => Error::PayloadTooLarge,
        500..=599 => Error::ServerFailure,
        _ => {
            if let Some(err) = api_error {
                Error::ApiError(err)
            } else {
                Error::RequestFailed(format!("Request failed with status {}", status))
            }
        }
    }
}

/// Map a network/transport error to an actionable Error.
///
/// This handles ureq errors which include HTTP status codes in their messages
/// for non-2xx responses (format: "Network error: http://url: status code NNN").
pub fn map_network_error(err: &str) -> Error {
    let err_lower = err.to_lowercase();

    // First check if this is an HTTP error embedded in a ureq error message
    // Format: "Network error: http://url: status code NNN"
    if let Some(status) = extract_status_code(err) {
        // This is an HTTP error, not a true network error
        return map_http_error(status, None);
    }

    // Check for common network error patterns
    if err_lower.contains("connection refused")
        || err_lower.contains("connection reset")
        || err_lower.contains("network is unreachable")
        || err_lower.contains("no route to host")
        || err_lower.contains("dns")
        || err_lower.contains("name resolution")
        || err_lower.contains("timeout")
        || err_lower.contains("timed out")
    {
        Error::ConnectionFailed
    } else {
        Error::RequestFailed(format!("Network error: {}", err))
    }
}

/// Extract HTTP status code from ureq error message.
/// ureq embeds status codes like "status code 404" in error messages for non-2xx responses.
fn extract_status_code(err: &str) -> Option<u16> {
    // Look for "status code NNN" pattern
    if let Some(pos) = err.find("status code ") {
        let rest = &err[pos + 12..]; // "status code " is 12 chars
                                     // Parse the next 3 digits
        if let Ok(num_str) = rest.chars().take(3).collect::<String>().parse::<u16>() {
            return Some(num_str);
        }
    }
    None
}

/// Parse an API error response body into an ApiError struct.
fn parse_api_error(body: &str) -> Option<ApiError> {
    // Try to parse as JSON
    let parsed: serde_json::Value = serde_json::from_str(body).ok()?;

    let error_msg = parsed.get("error")?.as_str()?.to_string();
    let code = parsed
        .get("code")
        .and_then(|c| c.as_str())
        .map(ApiErrorCode::from);

    let field = |name: &str| {
        parsed
            .get(name)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    };

    Some(ApiError {
        error: error_msg,
        code,
        revoked_at: field("revoked_at"),
        expires_at: field("expires_at"),
        upgrade_url: field("upgrade_url"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_401_to_auth_failed() {
        let error = map_http_error(401, None);
        assert_eq!(
            error.to_string(),
            "Authentication failed. Run `spikes login` to refresh your token."
        );
    }

    #[test]
    fn test_map_401_token_revoked() {
        let body =
            r#"{"error":"Revoked","code":"TOKEN_REVOKED","revoked_at":"2026-09-01T00:00:00Z"}"#;
        let error = map_http_error(401, Some(body));
        assert!(matches!(error, Error::AuthRevoked(Some(ref t)) if t == "2026-09-01T00:00:00Z"));
        assert!(error
            .to_string()
            .contains("revoked at 2026-09-01T00:00:00Z"));
    }

    #[test]
    fn test_map_401_token_expired() {
        let body =
            r#"{"error":"Expired","code":"TOKEN_EXPIRED","expires_at":"2026-08-01T00:00:00Z"}"#;
        let error = map_http_error(401, Some(body));
        assert!(matches!(error, Error::AuthExpired(Some(ref t)) if t == "2026-08-01T00:00:00Z"));
    }

    #[test]
    fn test_map_401_unknown_code_is_auth_failed() {
        let body = r#"{"error":"Invalid or revoked bearer token","code":"AUTH_FAILED"}"#;
        let error = map_http_error(401, Some(body));
        assert!(matches!(error, Error::AuthFailed));
    }

    #[test]
    fn test_map_402_upgrade_required() {
        let body = r#"{"error":"Upgrade","code":"UPGRADE_REQUIRED","upgrade_url":"https://spikes.sh/pro"}"#;
        let error = map_http_error(402, Some(body));
        assert_eq!(
            error.to_string(),
            "This feature requires a Pro or Agent plan. Upgrade at https://spikes.sh/pro"
        );
    }

    #[test]
    fn test_map_429_spike_limit() {
        let body = r#"{"error":"Spike limit reached","code":"SPIKE_LIMIT"}"#;
        let error = map_http_error(429, Some(body));
        assert_eq!(
            error.to_string(),
            "Share has reached spike limit. Upgrade at https://spikes.sh/pro"
        );
    }

    #[test]
    fn test_map_429_share_limit() {
        let body = r#"{"error":"Share limit reached","code":"SHARE_LIMIT"}"#;
        let error = map_http_error(429, Some(body));
        assert_eq!(
            error.to_string(),
            "You've reached the free tier limit (5 shares). Delete a share or upgrade at https://spikes.sh/pro"
        );
    }

    #[test]
    fn test_map_429_generic() {
        let error = map_http_error(429, None);
        assert!(error.to_string().contains("Rate limit exceeded"));
    }

    #[test]
    fn test_map_413_payload_too_large() {
        let error = map_http_error(413, None);
        assert_eq!(
            error.to_string(),
            "Files too large. Max size is 50MB. Consider removing large assets."
        );
    }

    #[test]
    fn test_map_500_server_error() {
        let error = map_http_error(500, None);
        assert_eq!(
            error.to_string(),
            "Server error. Please try again in a moment or contact support if it persists."
        );
    }

    #[test]
    fn test_map_502_bad_gateway() {
        let error = map_http_error(502, None);
        assert_eq!(
            error.to_string(),
            "Server error. Please try again in a moment or contact support if it persists."
        );
    }

    #[test]
    fn test_map_network_error_connection_refused() {
        let error = map_network_error("connection refused");
        assert_eq!(
            error.to_string(),
            "Connection failed. Check your internet connection."
        );
    }

    #[test]
    fn test_map_network_error_timeout() {
        let error = map_network_error("request timed out");
        assert_eq!(
            error.to_string(),
            "Connection failed. Check your internet connection."
        );
    }

    #[test]
    fn test_map_network_error_dns() {
        let error = map_network_error("dns resolution failed");
        assert_eq!(
            error.to_string(),
            "Connection failed. Check your internet connection."
        );
    }

    #[test]
    fn test_parse_api_error() {
        let body = r#"{"error":"Test error message","code":"TEST_CODE"}"#;
        let parsed = parse_api_error(body).unwrap();
        assert_eq!(parsed.error, "Test error message");
        assert_eq!(
            parsed.code,
            Some(ApiErrorCode::Unknown("TEST_CODE".to_string()))
        );
    }

    #[test]
    fn test_parse_api_error_without_code() {
        let body = r#"{"error":"Test error message"}"#;
        let parsed = parse_api_error(body).unwrap();
        assert_eq!(parsed.error, "Test error message");
        assert_eq!(parsed.code, None);
    }

    #[test]
    fn test_api_error_code_from_str() {
        assert_eq!(ApiErrorCode::from("SPIKE_LIMIT"), ApiErrorCode::SpikeLimit);
        assert_eq!(ApiErrorCode::from("SHARE_LIMIT"), ApiErrorCode::ShareLimit);
        assert_eq!(
            ApiErrorCode::from("BUDGET_EXCEEDED"),
            ApiErrorCode::BudgetExceeded
        );
        assert_eq!(
            ApiErrorCode::from("RATE_LIMITED"),
            ApiErrorCode::RateLimited
        );
        assert_eq!(
            ApiErrorCode::from("SCOPE_DENIED"),
            ApiErrorCode::ScopeDenied
        );
        assert_eq!(
            ApiErrorCode::from("OTHER"),
            ApiErrorCode::Unknown("OTHER".to_string())
        );
    }

    #[test]
    fn test_map_429_budget_exceeded() {
        let body = r#"{"error":"Monthly budget cap reached","code":"BUDGET_EXCEEDED"}"#;
        let error = map_http_error(429, Some(body));
        assert!(matches!(error, Error::BudgetExceeded));
        let err_lower = error.to_string().to_lowercase();
        assert!(
            err_lower.contains("budget"),
            "Error message should contain 'budget', got: {}",
            error
        );
        assert!(err_lower.contains("budget exceeded"));
    }

    #[test]
    fn test_map_429_rate_limited_generic() {
        // A 429 without BUDGET_EXCEEDED should still map to rate limit
        let body = r#"{"error":"Too many requests","code":"RATE_LIMITED"}"#;
        let error = map_http_error(429, Some(body));
        assert!(error.to_string().contains("Rate limit exceeded"));
    }

    #[test]
    fn test_map_403_scope_denied() {
        let body = r#"{"error":"Insufficient scope","code":"SCOPE_DENIED"}"#;
        let error = map_http_error(403, Some(body));
        assert!(matches!(error, Error::ScopeDenied));
        assert!(error
            .to_string()
            .to_lowercase()
            .contains("permission denied"));
    }

    #[test]
    fn test_map_403_generic_is_auth_failed() {
        // A 403 without SCOPE_DENIED code should still map to AuthFailed
        let error = map_http_error(403, None);
        assert!(matches!(error, Error::AuthFailed));
    }

    #[test]
    fn test_extract_status_code() {
        assert_eq!(
            extract_status_code("Network error: http://test: status code 401"),
            Some(401)
        );
        assert_eq!(
            extract_status_code("Network error: http://test: status code 429"),
            Some(429)
        );
        assert_eq!(
            extract_status_code("Network error: http://test: status code 500"),
            Some(500)
        );
        assert_eq!(extract_status_code("connection refused"), None);
        assert_eq!(extract_status_code("some other error"), None);
    }

    #[test]
    fn test_map_network_error_with_status_code_401() {
        // ureq-style error with 401 status
        let error = map_network_error("Network error: http://example.com/spikes: status code 401");
        assert_eq!(
            error.to_string(),
            "Authentication failed. Run `spikes login` to refresh your token."
        );
    }

    #[test]
    fn test_map_network_error_with_status_code_500() {
        // ureq-style error with 500 status
        let error = map_network_error("Network error: http://example.com/spikes: status code 500");
        assert_eq!(
            error.to_string(),
            "Server error. Please try again in a moment or contact support if it persists."
        );
    }

    #[test]
    fn test_map_network_error_with_status_code_413() {
        // ureq-style error with 413 status
        let error = map_network_error("Network error: http://example.com/spikes: status code 413");
        assert_eq!(
            error.to_string(),
            "Files too large. Max size is 50MB. Consider removing large assets."
        );
    }
}
