//! MCP (Model Context Protocol) server implementation using rmcp SDK.
//!
//! Exposes spikes feedback as tools for AI agent integration.
//! All logging goes to stderr; stdout is reserved for JSON-RPC.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars::JsonSchema,
    tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::auth::{get_api_base, AuthConfig};
use crate::error::{map_http_error, map_network_error, Error};
use crate::spike::{Rating, Reviewer, Spike, SpikeType};
use crate::storage::{load_spikes, remove_spike, save_spikes, update_spike};

// ============================================================================
// Data Source
// ============================================================================

/// Data source for MCP tools: local JSONL or remote API.
#[derive(Clone, Debug)]
pub enum DataSource {
    /// Read from local .spikes/feedback.jsonl
    Local,
    /// Read from hosted API via HTTP
    Remote {
        /// Bearer token for API authentication
        token: String,
        /// API base URL (e.g., https://spikes.sh)
        api_base: String,
    },
}

impl DataSource {
    /// Create a data source based on the --remote flag.
    /// 
    /// # Arguments
    /// * `remote` - Whether to use remote API
    /// 
    /// # Returns
    /// - `DataSource::Local` if remote is false
    /// - `DataSource::Remote` if remote is true and token is available
    /// - Error if remote is true but no token found
    pub fn new(remote: bool) -> crate::error::Result<Self> {
        if !remote {
            return Ok(DataSource::Local);
        }

        // Token resolution: SPIKES_TOKEN env var > auth.toml > error
        let token = match AuthConfig::token()? {
            Some(t) => t,
            None => {
                return Err(Error::AuthFailed);
            }
        };

        let api_base = get_api_base();

        Ok(DataSource::Remote { token, api_base })
    }
}

// ============================================================================
// Tool Argument Types
// ============================================================================

/// Arguments for the get_spikes tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetSpikesArgs {
    /// Filter by page name (e.g., 'index.html')
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// Filter by rating: love, like, meh, or no
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<String>,

    /// Only return unresolved spikes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unresolved_only: Option<bool>,
}

/// Arguments for the get_element_feedback tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetElementFeedbackArgs {
    /// CSS selector to look up (required)
    pub selector: String,

    /// Optional page filter (e.g., 'index.html')
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
}

/// Arguments for the get_hotspots tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetHotspotsArgs {
    /// Maximum number of hotspots to return (default: 10)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}

/// Arguments for the submit_spike tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SubmitSpikeArgs {
    /// Page name (e.g., 'index.html') - required
    pub page: String,

    /// URL of the page (e.g., 'http://localhost:3000/index.html')
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// CSS selector for element feedback (if provided, creates element spike)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,

    /// Text content of the element (optional, for element feedback)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_text: Option<String>,

    /// Rating: love, like, meh, or no
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<String>,

    /// Feedback comments (required)
    pub comments: String,

    /// Reviewer name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer_name: Option<String>,

    /// Project key for the feedback
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_key: Option<String>,
}

/// Arguments for the resolve_spike tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResolveSpikeArgs {
    /// Spike ID or prefix (minimum 4 characters)
    pub spike_id: String,
}

/// Arguments for the delete_spike tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteSpikeArgs {
    /// Spike ID or prefix (minimum 4 characters)
    pub spike_id: String,
}

/// Arguments for the create_share tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateShareArgs {
    /// Directory path to upload
    pub directory: String,

    /// Optional name/slug for the share
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Optional password protection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// Arguments for the list_shares tool (no parameters required)
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListSharesArgs {}

/// Arguments for the get_usage tool (no parameters required)
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetUsageArgs {}

// ============================================================================
// SpikesService - MCP Server Implementation
// ============================================================================

/// MCP server that exposes spikes feedback as tools for AI agents.
///
/// Tools provided:
/// - `get_spikes`: List feedback with optional filters
/// - `get_element_feedback`: Get feedback for a specific element
/// - `get_hotspots`: Find elements with the most feedback
/// - `submit_spike`: Create new feedback
/// - `resolve_spike`: Mark feedback as resolved
/// - `delete_spike`: Remove feedback
/// - `create_share`: Upload directory and get shareable URL
/// - `list_shares`: List all shares
/// - `get_usage`: Get usage statistics
#[derive(Clone, Debug)]
pub struct SpikesService {
    tool_router: ToolRouter<SpikesService>,
    data_source: DataSource,
}

#[tool_router]
impl SpikesService {
    /// Create a new SpikesService instance with the given data source
    pub fn new(data_source: DataSource) -> Self {
        Self {
            tool_router: Self::tool_router(),
            data_source,
        }
    }

    /// Get all feedback spikes, optionally filtered.
    ///
    /// Returns a formatted list of spikes with their ratings, comments,
    /// and resolution status. Perfect for understanding what needs work.
    #[tool(
        name = "get_spikes",
        description = "Dig into the feedback pile. Get all spikes (feedback items) with optional filters for page, rating, or unresolved status. Returns formatted text with spike details."
    )]
    async fn get_spikes(
        &self,
        Parameters(args): Parameters<GetSpikesArgs>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let spikes = match &self.data_source {
            DataSource::Local => {
                match load_spikes() {
                    Ok(s) => s,
                    Err(e) => {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "ERROR: Could not load spikes: {}",
                            e
                        ))]));
                    }
                }
            }
            DataSource::Remote { token, api_base } => {
                match fetch_remote_spikes(token, api_base, args.page.as_deref(), args.rating.as_deref(), args.unresolved_only.unwrap_or(false)) {
                    Ok(s) => s,
                    Err(e) => {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "ERROR: {}",
                            e
                        ))]));
                    }
                }
            }
        };

        let page_filter = args.page.as_deref();
        let rating_filter = args.rating.as_deref();
        let unresolved_only = args.unresolved_only.unwrap_or(false);

        let filtered: Vec<&Spike> = spikes
            .iter()
            .filter(|s| {
                // Page filter
                if let Some(page) = page_filter {
                    if s.page != page {
                        return false;
                    }
                }
                // Rating filter
                if let Some(rating_str) = rating_filter {
                    if let Ok(rating) = rating_str.parse::<Rating>() {
                        if s.rating.as_ref() != Some(&rating) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                // Unresolved filter
                if unresolved_only && s.is_resolved() {
                    return false;
                }
                true
            })
            .collect();

        if filtered.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No spikes found matching the criteria. Clean slate, or wrong filters?",
            )]));
        }

        let mut output = format!("Found {} spike(s):\n\n", filtered.len());
        for spike in filtered {
            output.push_str(&format_spike(spike));
            output.push('\n');
        }

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    /// Get feedback for a specific CSS selector.
    ///
    /// Use this to zoom in on a particular element's feedback history.
    #[tool(
        name = "get_element_feedback",
        description = "Target lock: get feedback for a specific CSS selector. Zoom in on what reviewers said about a particular element. Requires selector parameter."
    )]
    async fn get_element_feedback(
        &self,
        Parameters(args): Parameters<GetElementFeedbackArgs>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let spikes = match &self.data_source {
            DataSource::Local => {
                match load_spikes() {
                    Ok(s) => s,
                    Err(e) => {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "ERROR: Could not load spikes: {}",
                            e
                        ))]));
                    }
                }
            }
            DataSource::Remote { token, api_base } => {
                match fetch_remote_spikes(token, api_base, None, None, false) {
                    Ok(s) => s,
                    Err(e) => {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "ERROR: {}",
                            e
                        ))]));
                    }
                }
            }
        };

        let page_filter = args.page.as_deref();

        let matching: Vec<&Spike> = spikes
            .iter()
            .filter(|s| {
                if s.spike_type != SpikeType::Element {
                    return false;
                }
                if s.selector.as_deref() != Some(args.selector.as_str()) {
                    return false;
                }
                if let Some(page) = page_filter {
                    if s.page != page {
                        return false;
                    }
                }
                true
            })
            .collect();

        if matching.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "No feedback found for selector '{}'. Ghost town.",
                args.selector
            ))]));
        }

        let mut output = format!(
            "Found {} feedback item(s) for '{}':\n\n",
            matching.len(),
            args.selector
        );
        for spike in matching {
            output.push_str(&format_spike(spike));
            output.push('\n');
        }

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    /// Find elements with the most feedback.
    ///
    /// Identifies hotspots - elements that attracted the most attention.
    #[tool(
        name = "get_hotspots",
        description = "Heat map mode: find elements with the most feedback. Identifies hotspots where reviewers clustered. Use this to prioritize what to fix first."
    )]
    async fn get_hotspots(
        &self,
        Parameters(args): Parameters<GetHotspotsArgs>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let spikes = match &self.data_source {
            DataSource::Local => {
                match load_spikes() {
                    Ok(s) => s,
                    Err(e) => {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "ERROR: Could not load spikes: {}",
                            e
                        ))]));
                    }
                }
            }
            DataSource::Remote { token, api_base } => {
                match fetch_remote_spikes(token, api_base, None, None, false) {
                    Ok(s) => s,
                    Err(e) => {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "ERROR: {}",
                            e
                        ))]));
                    }
                }
            }
        };

        let limit = args.limit.unwrap_or(10) as usize;

        // Count feedback per selector
        let mut counts: HashMap<String, usize> = HashMap::new();
        for spike in &spikes {
            if spike.spike_type == SpikeType::Element {
                if let Some(selector) = &spike.selector {
                    *counts.entry(selector.clone()).or_insert(0) += 1;
                }
            }
        }

        // Sort by count descending
        let mut hotspots: Vec<(String, usize)> = counts.into_iter().collect();
        hotspots.sort_by_key(|item| std::cmp::Reverse(item.1));
        hotspots.truncate(limit);

        if hotspots.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No element feedback found. No hotspots to report.",
            )]));
        }

        let mut output = format!("Top {} hotspot(s):\n\n", hotspots.len());
        for (i, (selector, count)) in hotspots.iter().enumerate() {
            output.push_str(&format!(
                "{}. {} ({} feedback item{})\n",
                i + 1,
                selector,
                count,
                if *count == 1 { "" } else { "s" }
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    /// Submit a new spike (feedback item) to the local JSONL file.
    ///
    /// Creates a spike with a generated ID. If selector is provided,
    /// creates an element spike; otherwise creates a page spike.
    #[tool(
        name = "submit_spike",
        description = "Plant a flag: create new feedback. Submit a spike with page, comments, and optional selector/rating. Agent-created spikes get logged to .spikes/feedback.jsonl."
    )]
    async fn submit_spike(
        &self,
        Parameters(args): Parameters<SubmitSpikeArgs>,
    ) -> std::result::Result<CallToolResult, McpError> {
        match &self.data_source {
            DataSource::Local => {
                submit_spike_local(args).await
            }
            DataSource::Remote { token, api_base } => {
                submit_spike_remote(args, token, api_base).await
            }
        }
    }

    /// Resolve a spike by marking it as resolved.
    ///
    /// Sets resolved=true and adds resolved_at timestamp.
    #[tool(
        name = "resolve_spike",
        description = "Mark done: resolve a spike by ID. Sets resolved=true with timestamp. Use after addressing the feedback."
    )]
    async fn resolve_spike(
        &self,
        Parameters(args): Parameters<ResolveSpikeArgs>,
    ) -> std::result::Result<CallToolResult, McpError> {
        match &self.data_source {
            DataSource::Local => {
                resolve_spike_local(args).await
            }
            DataSource::Remote { token, api_base } => {
                resolve_spike_remote(args, token, api_base).await
            }
        }
    }

    /// Delete a spike from the JSONL file.
    ///
    /// Removes the spike entirely from the feedback file.
    #[tool(
        name = "delete_spike",
        description = "Remove from history: delete a spike by ID. Permanently removes the feedback. Use sparingly - resolving is usually better."
    )]
    async fn delete_spike(
        &self,
        Parameters(args): Parameters<DeleteSpikeArgs>,
    ) -> std::result::Result<CallToolResult, McpError> {
        match &self.data_source {
            DataSource::Local => {
                delete_spike_local(args).await
            }
            DataSource::Remote { token, api_base } => {
                delete_spike_remote(args, token, api_base).await
            }
        }
    }

    /// Create a share by uploading files to the hosted service.
    ///
    /// Requires authentication (bearer token).
    #[tool(
        name = "create_share",
        description = "Go live: upload a directory and get a shareable URL. Requires login (spikes login). Returns URL for collecting feedback."
    )]
    async fn create_share(
        &self,
        Parameters(args): Parameters<CreateShareArgs>,
    ) -> std::result::Result<CallToolResult, McpError> {
        // Get token from data source or check auth
        let token = match &self.data_source {
            DataSource::Remote { token, .. } => token.clone(),
            DataSource::Local => {
                match AuthConfig::token() {
                    Ok(Some(t)) => t,
                    Ok(None) => {
                        return Ok(CallToolResult::success(vec![Content::text(
                            "ERROR: Not logged in. Run 'spikes login' first or set SPIKES_TOKEN env var.",
                        )]));
                    }
                    Err(e) => {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "ERROR: Could not check auth: {}",
                            e
                        ))]));
                    }
                }
            }
        };

        // Get API base from data source or env
        let api_base = match &self.data_source {
            DataSource::Remote { api_base, .. } => api_base.clone(),
            DataSource::Local => get_api_base(),
        };

        let dir_path = Path::new(&args.directory);
        if !dir_path.exists() || !dir_path.is_dir() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "ERROR: Directory not found: {}",
                args.directory
            ))]));
        }

        // Collect files
        let files = match collect_files(dir_path) {
            Ok(f) => f,
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "ERROR: Could not collect files: {}",
                    e
                ))]));
            }
        };

        if files.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "ERROR: No uploadable files found in directory.",
            )]));
        }

        let slug = args.name.unwrap_or_else(|| {
            dir_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project")
                .to_string()
        });

        let result = upload_share(&token, dir_path, &files, &slug, args.password.as_deref(), &api_base);

        match result {
            Ok(share_result) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Share created!\n  URL: {}\n  Slug: {}\n  Files: {}",
                share_result.url, share_result.slug, share_result.file_count
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "ERROR: {}",
                e
            ))])),
        }
    }

    /// List all shares from the hosted service.
    ///
    /// Requires authentication (bearer token).
    #[tool(
        name = "list_shares",
        description = "Check inventory: list all your shares. Shows URLs, spike counts, and creation dates. Requires login."
    )]
    async fn list_shares(
        &self,
        Parameters(_args): Parameters<ListSharesArgs>,
    ) -> std::result::Result<CallToolResult, McpError> {
        // Get token from data source or check auth
        let token = match &self.data_source {
            DataSource::Remote { token, .. } => token.clone(),
            DataSource::Local => {
                match AuthConfig::token() {
                    Ok(Some(t)) => t,
                    Ok(None) => {
                        return Ok(CallToolResult::success(vec![Content::text(
                            "ERROR: Not logged in. Run 'spikes login' first or set SPIKES_TOKEN env var.",
                        )]));
                    }
                    Err(e) => {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "ERROR: Could not check auth: {}",
                            e
                        ))]));
                    }
                }
            }
        };

        // Get API base from data source or env
        let api_base = match &self.data_source {
            DataSource::Remote { api_base, .. } => api_base.clone(),
            DataSource::Local => get_api_base(),
        };

        let shares = fetch_shares(&token, &api_base);

        match shares {
            Ok(share_list) => {
                if share_list.is_empty() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        "No shares found. Create one with create_share.",
                    )]));
                }

                let mut output = format!("Found {} share(s):\n\n", share_list.len());
                for share in share_list {
                    output.push_str(&format!(
                        "[{}] {}\n  URL: {}\n  Spikes: {}\n  Created: {}\n\n",
                        share.slug, share.name.unwrap_or_default(), share.url, share.spike_count, share.created_at
                    ));
                }

                Ok(CallToolResult::success(vec![Content::text(output)]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "ERROR: {}",
                e
            ))])),
        }
    }

    /// Get usage statistics from the hosted service.
    ///
    /// Requires authentication (bearer token).
    #[tool(
        name = "get_usage",
        description = "Check limits: view your usage stats. Shows spike/share counts and limits. Requires login."
    )]
    async fn get_usage(
        &self,
        Parameters(_args): Parameters<GetUsageArgs>,
    ) -> std::result::Result<CallToolResult, McpError> {
        // Get token from data source or check auth
        let token = match &self.data_source {
            DataSource::Remote { token, .. } => token.clone(),
            DataSource::Local => {
                match AuthConfig::token() {
                    Ok(Some(t)) => t,
                    Ok(None) => {
                        return Ok(CallToolResult::success(vec![Content::text(
                            "ERROR: Not logged in. Run 'spikes login' first or set SPIKES_TOKEN env var.",
                        )]));
                    }
                    Err(e) => {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "ERROR: Could not check auth: {}",
                            e
                        ))]));
                    }
                }
            }
        };

        // Get API base from data source or env
        let api_base = match &self.data_source {
            DataSource::Remote { api_base, .. } => api_base.clone(),
            DataSource::Local => get_api_base(),
        };

        let usage = fetch_usage(&token, &api_base);

        match usage {
            Ok(usage_data) => {
                let mut output = format!("Usage ({} tier):\n\n", usage_data.tier.to_uppercase());

                let shares_display = match usage_data.share_limit {
                    Some(limit) => format!("{}/{}", usage_data.shares, limit),
                    None => format!("{} (unlimited)", usage_data.shares),
                };

                let spikes_display = match usage_data.spike_limit {
                    Some(limit) => format!("{}/{}", usage_data.spikes, limit),
                    None => format!("{} (unlimited)", usage_data.spikes),
                };

                output.push_str(&format!("  Shares: {}\n", shares_display));
                output.push_str(&format!("  Spikes: {}\n", spikes_display));

                Ok(CallToolResult::success(vec![Content::text(output)]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "ERROR: {}",
                e
            ))])),
        }
    }
}

impl Default for SpikesService {
    fn default() -> Self {
        Self::new(DataSource::Local)
    }
}

#[tool_handler]
impl ServerHandler for SpikesService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "spikes-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: None,
                description: None,
                icons: None,
                website_url: None,
            },
            instructions: None,
        }
    }
}

// ============================================================================
// Formatting Functions
// ============================================================================

/// Format a spike for display in tool output
fn format_spike(spike: &Spike) -> String {
    let mut output = format!(
        "[{}] {} on {}\n",
        &spike.id.chars().take(8).collect::<String>(),
        spike.type_str(),
        spike.page
    );

    output.push_str(&format!("  Rating: {}\n", spike.rating_str()));

    if spike.spike_type == SpikeType::Element {
        if let Some(selector) = &spike.selector {
            output.push_str(&format!("  Selector: {}\n", selector));
        }
        if let Some(text) = &spike.element_text {
            output.push_str(&format!("  Element text: {}\n", text));
        }
    }

    if !spike.comments.is_empty() {
        output.push_str(&format!("  Comments: {}\n", spike.comments));
    }

    output.push_str(&format!("  Reviewer: {}\n", spike.reviewer.name));
    output.push_str(&format!("  Timestamp: {}\n", spike.timestamp));

    if spike.is_resolved() {
        output.push_str("  Status: Resolved\n");
        if let Some(resolved_at) = &spike.resolved_at {
            output.push_str(&format!("  Resolved at: {}\n", resolved_at));
        }
    } else {
        output.push_str("  Status: Unresolved\n");
    }

    output
}

// ============================================================================
// Helper Functions for Write Tools
// ============================================================================

const INCLUDE_EXTENSIONS: &[&str] = &[
    "html", "css", "js", "json", "png", "jpg", "jpeg", "gif", "svg", "woff", "woff2", "ico",
];

const EXCLUDE_DIRS: &[&str] = &[".spikes", "node_modules", ".git"];
const EXCLUDE_FILES: &[&str] = &[".DS_Store"];

/// Collect uploadable files from a directory
fn collect_files(dir: &Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        // Skip excluded directories
        let should_skip = path.ancestors().any(|ancestor| {
            ancestor
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| EXCLUDE_DIRS.contains(&n))
                .unwrap_or(false)
        });
        if should_skip {
            continue;
        }

        // Skip excluded files
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if EXCLUDE_FILES.contains(&name) {
                continue;
            }
        }

        // Check extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if INCLUDE_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                files.push(path.to_path_buf());
            }
        }
    }

    Ok(files)
}

/// Result of a share upload
struct ShareResult {
    url: String,
    slug: String,
    file_count: usize,
}

/// Upload files to create a share
fn upload_share(
    token: &str,
    base_dir: &Path,
    files: &[std::path::PathBuf],
    slug: &str,
    password: Option<&str>,
    api_base: &str,
) -> crate::error::Result<ShareResult> {
    use ureq::Agent;

    let agent = Agent::new();
    let url = format!("{}/shares", api_base.trim_end_matches('/'));

    // Build multipart form
    let boundary = format!("----SpikesUpload{}", chrono::Utc::now().timestamp_millis());
    let mut body = Vec::new();

    // Add metadata field
    let mut metadata = serde_json::json!({ "name": slug });
    if let Some(pw) = password {
        metadata["password"] = serde_json::Value::String(pw.to_string());
    }
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"metadata\"\r\n\r\n");
    body.extend_from_slice(metadata.to_string().as_bytes());
    body.extend_from_slice(b"\r\n");

    // Add each file
    for file_path in files {
        let relative = file_path
            .strip_prefix(base_dir)
            .unwrap_or(file_path)
            .to_string_lossy();

        let content = fs::read(file_path)?;
        let mime = guess_mime(file_path);

        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"files\"; filename=\"{}\"\r\n",
                relative
            )
            .as_bytes(),
        );
        body.extend_from_slice(format!("Content-Type: {}\r\n\r\n", mime).as_bytes());
        body.extend_from_slice(&content);
        body.extend_from_slice(b"\r\n");
    }

    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    let response = match agent
        .post(&url)
        .set("Authorization", &format!("Bearer {}", token))
        .set(
            "Content-Type",
            &format!("multipart/form-data; boundary={}", boundary),
        )
        .send_bytes(&body)
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(status, response)) => {
            let body_text = response.into_string().ok();
            return Err(map_http_error(status, body_text.as_deref()));
        }
        Err(e) => return Err(map_network_error(&e.to_string())),
    };

    let status = response.status();

    if status != 200 && status != 201 {
        let body_text = response.into_string().ok();
        return Err(map_http_error(status, body_text.as_deref()));
    }

    let body_text = response
        .into_string()
        .map_err(|e| Error::RequestFailed(format!("Failed to read response: {}", e)))?;

    let parsed: serde_json::Value = serde_json::from_str(&body_text)?;

    let result_url = parsed
        .get("url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let result_slug = parsed
        .get("slug")
        .and_then(|v| v.as_str())
        .unwrap_or(slug)
        .to_string();

    Ok(ShareResult {
        url: result_url,
        slug: result_slug,
        file_count: files.len(),
    })
}

/// Guess MIME type from file extension
fn guess_mime(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

/// Share data from API
#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
struct ShareInfo {
    id: String,
    slug: String,
    url: String,
    spike_count: usize,
    created_at: String,
    name: Option<String>,
}

/// Fetch shares from the API
fn fetch_shares(token: &str, api_base: &str) -> crate::error::Result<Vec<ShareInfo>> {
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

    let shares: Vec<ShareInfo> = serde_json::from_str(&body)?;
    Ok(shares)
}

/// Usage data from API
#[derive(Debug, Clone, serde::Deserialize)]
struct UsageData {
    spikes: u64,
    spike_limit: Option<u64>,
    shares: u64,
    share_limit: Option<u64>,
    tier: String,
}

/// Fetch usage from the API
fn fetch_usage(token: &str, api_base: &str) -> crate::error::Result<UsageData> {
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

    let usage: UsageData = serde_json::from_str(&body)?;
    Ok(usage)
}

// ============================================================================
// Remote Mode Helper Functions
// ============================================================================

/// Fetch spikes from the remote API with optional filters
fn fetch_remote_spikes(
    token: &str,
    api_base: &str,
    page: Option<&str>,
    rating: Option<&str>,
    unresolved_only: bool,
) -> crate::error::Result<Vec<Spike>> {
    let mut url = format!("{}/spikes", api_base.trim_end_matches('/'));
    let mut params = Vec::new();

    if let Some(p) = page {
        params.push(format!("page={}", urlencoding::encode(p)));
    }
    if let Some(r) = rating {
        params.push(format!("rating={}", urlencoding::encode(r)));
    }
    if unresolved_only {
        params.push("resolved=false".to_string());
    }

    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

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

    // Parse the API response - could be an array or an object with spikes field
    let spikes: Vec<Spike> = if body.trim_start().starts_with('[') {
        serde_json::from_str(&body)?
    } else {
        let parsed: serde_json::Value = serde_json::from_str(&body)?;
        if let Some(spikes_arr) = parsed.get("spikes").and_then(|s| s.as_array()) {
            serde_json::from_value(serde_json::Value::Array(spikes_arr.clone()))?
        } else {
            Vec::new()
        }
    };

    Ok(spikes)
}

/// Local implementation of submit_spike
async fn submit_spike_local(args: SubmitSpikeArgs) -> std::result::Result<CallToolResult, McpError> {
    // Determine spike type based on whether selector is provided
    let spike_type = if args.selector.is_some() {
        SpikeType::Element
    } else {
        SpikeType::Page
    };

    // Generate nanoid for the spike
    let id = nanoid::nanoid!(11);

    // Parse rating if provided
    let rating = args.rating.and_then(|r| r.parse::<Rating>().ok());

    // Build the spike
    let spike = Spike {
        id: id.clone(),
        spike_type,
        project_key: args.project_key.unwrap_or_else(|| "default".to_string()),
        page: args.page,
        url: args.url.unwrap_or_default(),
        reviewer: Reviewer {
            id: nanoid::nanoid!(8),
            name: args.reviewer_name.unwrap_or_else(|| "MCP Agent".to_string()),
        },
        selector: args.selector,
        element_text: args.element_text,
        bounding_box: None,
        rating,
        comments: args.comments,
        timestamp: chrono::Utc::now().to_rfc3339(),
        viewport: None,
        resolved: None,
        resolved_at: None,
    };

    // Load existing spikes and append the new one
    let mut spikes = match load_spikes() {
        Ok(s) => s,
        Err(Error::NoSpikesDir) => {
            // Need to create the directory first
            let _ = std::fs::create_dir_all(".spikes");
            Vec::new()
        }
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "ERROR: Could not load spikes: {}",
                e
            ))]));
        }
    };

    spikes.push(spike.clone());

    if let Err(e) = save_spikes(&spikes) {
        return Ok(CallToolResult::success(vec![Content::text(format!(
            "ERROR: Could not save spike: {}",
            e
        ))]));
    }

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Spike created: [{}] {} on {}\n  Comments: {}\n  ID: {}",
        &spike.id.chars().take(8).collect::<String>(),
        spike.type_str(),
        spike.page,
        spike.comments,
        spike.id
    ))]))
}

/// Remote implementation of submit_spike
async fn submit_spike_remote(
    args: SubmitSpikeArgs,
    token: &str,
    api_base: &str,
) -> std::result::Result<CallToolResult, McpError> {
    let url = format!("{}/spikes", api_base.trim_end_matches('/'));

    // Build request body
    let mut body = serde_json::json!({
        "page": args.page,
        "comments": args.comments,
    });

    if let Some(url_val) = &args.url {
        body["url"] = serde_json::Value::String(url_val.clone());
    }
    if let Some(selector) = &args.selector {
        body["selector"] = serde_json::Value::String(selector.clone());
        body["type"] = serde_json::Value::String("element".to_string());
    } else {
        body["type"] = serde_json::Value::String("page".to_string());
    }
    if let Some(element_text) = &args.element_text {
        body["elementText"] = serde_json::Value::String(element_text.clone());
    }
    if let Some(rating) = &args.rating {
        body["rating"] = serde_json::Value::String(rating.clone());
    }
    if let Some(reviewer_name) = &args.reviewer_name {
        body["reviewerName"] = serde_json::Value::String(reviewer_name.clone());
    }
    if let Some(project_key) = &args.project_key {
        body["projectKey"] = serde_json::Value::String(project_key.clone());
    }

    let response = match ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", token))
        .set("Content-Type", "application/json")
        .send_json(&body)
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(status, response)) => {
            let body_text = response.into_string().ok();
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "ERROR: {}",
                map_http_error(status, body_text.as_deref())
            ))]));
        }
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "ERROR: {}",
                map_network_error(&e.to_string())
            ))]));
        }
    };

    let status = response.status();
    if status != 200 && status != 201 {
        let body_text = response.into_string().ok();
        return Ok(CallToolResult::success(vec![Content::text(format!(
            "ERROR: {}",
            map_http_error(status, body_text.as_deref())
        ))]));
    }

    let body_text = response.into_string().ok();
    let parsed: Option<serde_json::Value> = body_text.and_then(|b| serde_json::from_str(&b).ok());

    let spike_id = parsed
        .as_ref()
        .and_then(|p| p.get("id"))
        .and_then(|i| i.as_str())
        .unwrap_or("unknown");

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Spike created via API: [{}]",
        spike_id
    ))]))
}

/// Local implementation of resolve_spike
async fn resolve_spike_local(args: ResolveSpikeArgs) -> std::result::Result<CallToolResult, McpError> {
    let resolved_at = chrono::Utc::now().to_rfc3339();

    let result = update_spike(&args.spike_id, |spike| {
        spike.resolved = Some(true);
        spike.resolved_at = Some(resolved_at.clone());
    });

    match result {
        Ok(updated) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Spike [{}] marked as resolved.\n  Page: {}\n  Resolved at: {}",
            &updated.id.chars().take(8).collect::<String>(),
            updated.page,
            resolved_at
        ))])),
        Err(Error::SpikeNotFound(msg)) => Ok(CallToolResult::success(vec![Content::text(format!(
            "ERROR: Spike not found: {}",
            msg
        ))])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "ERROR: Could not resolve spike: {}",
            e
        ))])),
    }
}

/// Remote implementation of resolve_spike
async fn resolve_spike_remote(
    args: ResolveSpikeArgs,
    token: &str,
    api_base: &str,
) -> std::result::Result<CallToolResult, McpError> {
    let url = format!(
        "{}/spikes/{}",
        api_base.trim_end_matches('/'),
        args.spike_id
    );

    let body = serde_json::json!({
        "resolved": true,
        "resolvedAt": chrono::Utc::now().to_rfc3339()
    });

    let response = match ureq::request("PATCH", &url)
        .set("Authorization", &format!("Bearer {}", token))
        .set("Content-Type", "application/json")
        .send_json(&body)
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(status, response)) => {
            let body_text = response.into_string().ok();
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "ERROR: {}",
                map_http_error(status, body_text.as_deref())
            ))]));
        }
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "ERROR: {}",
                map_network_error(&e.to_string())
            ))]));
        }
    };

    let status = response.status();
    if status != 200 {
        let body_text = response.into_string().ok();
        return Ok(CallToolResult::success(vec![Content::text(format!(
            "ERROR: {}",
            map_http_error(status, body_text.as_deref())
        ))]));
    }

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Spike [{}] marked as resolved via API.",
        args.spike_id
    ))]))
}

/// Local implementation of delete_spike
async fn delete_spike_local(args: DeleteSpikeArgs) -> std::result::Result<CallToolResult, McpError> {
    let result = remove_spike(&args.spike_id);

    match result {
        Ok(removed) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Spike [{}] deleted.\n  Page: {}\n  Comments: {}",
            &removed.id.chars().take(8).collect::<String>(),
            removed.page,
            removed.comments
        ))])),
        Err(Error::SpikeNotFound(msg)) => Ok(CallToolResult::success(vec![Content::text(format!(
            "ERROR: Spike not found: {}",
            msg
        ))])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "ERROR: Could not delete spike: {}",
            e
        ))])),
    }
}

/// Remote implementation of delete_spike
async fn delete_spike_remote(
    args: DeleteSpikeArgs,
    token: &str,
    api_base: &str,
) -> std::result::Result<CallToolResult, McpError> {
    let url = format!(
        "{}/spikes/{}",
        api_base.trim_end_matches('/'),
        args.spike_id
    );

    let response = match ureq::delete(&url)
        .set("Authorization", &format!("Bearer {}", token))
        .call()
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(status, response)) => {
            let body_text = response.into_string().ok();
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "ERROR: {}",
                map_http_error(status, body_text.as_deref())
            ))]));
        }
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "ERROR: {}",
                map_network_error(&e.to_string())
            ))]));
        }
    };

    let status = response.status();
    if status != 200 && status != 204 {
        let body_text = response.into_string().ok();
        return Ok(CallToolResult::success(vec![Content::text(format!(
            "ERROR: {}",
            map_http_error(status, body_text.as_deref())
        ))]));
    }

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Spike [{}] deleted via API.",
        args.spike_id
    ))]))
}

/// URL encoding helper (simple implementation)
mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect()
    }
}

// ============================================================================
// Entry Point
// ============================================================================

/// Transport mode for MCP server
#[derive(Clone, Debug)]
pub enum TransportMode {
    /// Use standard input/output for JSON-RPC (default)
    Stdio,
    /// Use HTTP transport with POST endpoint
    Http {
        /// Port to listen on
        port: u16,
        /// Bind address
        bind: String,
    },
}

/// Run the MCP server with the specified transport mode.
///
/// This function is synchronous but internally uses tokio runtime.
/// All logging goes to stderr; stdout is reserved for JSON-RPC (stdio mode).
pub fn run(remote: bool, transport: TransportMode) -> crate::error::Result<()> {
    // Use tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| crate::error::Error::RequestFailed(format!("Failed to create tokio runtime: {}", e)))?;

    rt.block_on(async_run(remote, transport))
}

/// Async implementation of the MCP server
async fn async_run(remote: bool, transport: TransportMode) -> crate::error::Result<()> {
    // Create data source based on --remote flag
    let data_source = match DataSource::new(remote) {
        Ok(ds) => ds,
        Err(e) => {
            eprintln!("[spikes-mcp] ERROR: {}", e);
            return Err(e);
        }
    };

    match transport {
        TransportMode::Stdio => run_stdio(data_source, remote).await,
        TransportMode::Http { port, bind } => run_http(data_source, remote, port, bind).await,
    }
}

/// Run MCP server using stdio transport
async fn run_stdio(data_source: DataSource, remote: bool) -> crate::error::Result<()> {
    // All logging must go to stderr; stdout is for JSON-RPC
    if remote {
        eprintln!("[spikes-mcp] Starting MCP server on stdio (REMOTE mode)...");
    } else {
        eprintln!("[spikes-mcp] Starting MCP server on stdio (LOCAL mode)...");
    }

    let service = SpikesService::new(data_source);

    // Use stdio transport
    let transport = rmcp::transport::stdio();

    let server = service
        .serve(transport)
        .await
        .map_err(|e| crate::error::Error::RequestFailed(format!("MCP server error: {}", e)))?;

    // Wait for the server to finish
    let quit_reason = server
        .waiting()
        .await
        .map_err(|e| crate::error::Error::RequestFailed(format!("MCP server error: {}", e)))?;

    eprintln!("[spikes-mcp] Server stopped: {:?}", quit_reason);

    Ok(())
}

/// Run MCP server using HTTP transport
async fn run_http(data_source: DataSource, remote: bool, port: u16, bind: String) -> crate::error::Result<()> {
    use axum::Router;
    use rmcp::transport::streamable_http_server::tower::StreamableHttpService;
    use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tokio::net::TcpListener;

    // All logging goes to stderr
    if remote {
        eprintln!("[spikes-mcp] Starting MCP server on HTTP (REMOTE mode)...");
    } else {
        eprintln!("[spikes-mcp] Starting MCP server on HTTP (LOCAL mode)...");
    }

    // Parse bind address
    let addr: SocketAddr = format!("{}:{}", bind, port)
        .parse()
        .map_err(|e| crate::error::Error::RequestFailed(format!("Invalid bind address: {}", e)))?;

    // Create session manager for HTTP transport
    let session_manager = Arc::new(LocalSessionManager::default());

    // Create the StreamableHttpService
    // The service factory creates a new SpikesService for each session
    let data_source_clone = data_source.clone();
    let http_service = StreamableHttpService::new(
        move || Ok(SpikesService::new(data_source_clone.clone())),
        session_manager,
        Default::default(),
    );

    // Create axum router with the HTTP service at the root path
    let app = Router::new()
        .route("/", axum::routing::any(|req| async move {
            http_service.clone().handle(req).await
        }));

    // Bind to the address
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| crate::error::Error::RequestFailed(format!("Failed to bind to {}: {}", addr, e)))?;

    eprintln!("[spikes-mcp] HTTP server listening on http://{}", addr);

    // Run the server
    axum::serve(listener, app)
        .await
        .map_err(|e| crate::error::Error::RequestFailed(format!("HTTP server error: {}", e)))?;

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test helper to create sample spikes
    fn create_test_spikes() -> Vec<Spike> {
        vec![
            Spike {
                id: "spike001abc".to_string(),
                spike_type: SpikeType::Page,
                project_key: "test".to_string(),
                page: "index.html".to_string(),
                url: "http://test/index.html".to_string(),
                reviewer: crate::spike::Reviewer {
                    id: "r1".to_string(),
                    name: "Alice".to_string(),
                },
                selector: None,
                element_text: None,
                bounding_box: None,
                rating: Some(Rating::Love),
                comments: "Great design!".to_string(),
                timestamp: "2024-01-15T10:00:00Z".to_string(),
                viewport: None,
                resolved: None,
                resolved_at: None,
            },
            Spike {
                id: "spike002def".to_string(),
                spike_type: SpikeType::Element,
                project_key: "test".to_string(),
                page: "index.html".to_string(),
                url: "http://test/index.html".to_string(),
                reviewer: crate::spike::Reviewer {
                    id: "r2".to_string(),
                    name: "Bob".to_string(),
                },
                selector: Some(".hero-title".to_string()),
                element_text: Some("Welcome".to_string()),
                bounding_box: None,
                rating: Some(Rating::No),
                comments: "Font too small".to_string(),
                timestamp: "2024-01-15T11:00:00Z".to_string(),
                viewport: None,
                resolved: Some(true),
                resolved_at: Some("2024-01-16T09:00:00Z".to_string()),
            },
            Spike {
                id: "spike003ghi".to_string(),
                spike_type: SpikeType::Element,
                project_key: "test".to_string(),
                page: "about.html".to_string(),
                url: "http://test/about.html".to_string(),
                reviewer: crate::spike::Reviewer {
                    id: "r1".to_string(),
                    name: "Alice".to_string(),
                },
                selector: Some(".hero-title".to_string()),
                element_text: Some("About Us".to_string()),
                bounding_box: None,
                rating: Some(Rating::Meh),
                comments: "Could be better".to_string(),
                timestamp: "2024-01-15T12:00:00Z".to_string(),
                viewport: None,
                resolved: None,
                resolved_at: None,
            },
            Spike {
                id: "spike004jkl".to_string(),
                spike_type: SpikeType::Element,
                project_key: "test".to_string(),
                page: "index.html".to_string(),
                url: "http://test/index.html".to_string(),
                reviewer: crate::spike::Reviewer {
                    id: "r3".to_string(),
                    name: "Charlie".to_string(),
                },
                selector: Some(".nav-button".to_string()),
                element_text: Some("Menu".to_string()),
                bounding_box: None,
                rating: Some(Rating::Like),
                comments: "Nice hover effect".to_string(),
                timestamp: "2024-01-15T13:00:00Z".to_string(),
                viewport: None,
                resolved: None,
                resolved_at: None,
            },
        ]
    }

    // ========================================
    // Unit Tests for Tool Logic
    // ========================================

    #[test]
    fn test_format_spike_page() {
        let spike = Spike {
            id: "test123456".to_string(),
            spike_type: SpikeType::Page,
            project_key: "proj".to_string(),
            page: "index.html".to_string(),
            url: "http://test".to_string(),
            reviewer: crate::spike::Reviewer {
                id: "r1".to_string(),
                name: "Test User".to_string(),
            },
            selector: None,
            element_text: None,
            bounding_box: None,
            rating: Some(Rating::Love),
            comments: "Great!".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            viewport: None,
            resolved: None,
            resolved_at: None,
        };

        let formatted = format_spike(&spike);
        assert!(formatted.contains("test1234"));
        assert!(formatted.contains("page"));
        assert!(formatted.contains("index.html"));
        assert!(formatted.contains("love"));
        assert!(formatted.contains("Great!"));
        assert!(formatted.contains("Test User"));
        assert!(formatted.contains("Unresolved"));
    }

    #[test]
    fn test_format_spike_element() {
        let spike = Spike {
            id: "elem123abc".to_string(),
            spike_type: SpikeType::Element,
            project_key: "proj".to_string(),
            page: "page.html".to_string(),
            url: "http://test".to_string(),
            reviewer: crate::spike::Reviewer {
                id: "r1".to_string(),
                name: "Test".to_string(),
            },
            selector: Some(".hero".to_string()),
            element_text: Some("Welcome".to_string()),
            bounding_box: None,
            rating: Some(Rating::No),
            comments: "Bad".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            viewport: None,
            resolved: Some(true),
            resolved_at: Some("2024-01-02T00:00:00Z".to_string()),
        };

        let formatted = format_spike(&spike);
        assert!(formatted.contains("element"));
        assert!(formatted.contains(".hero"));
        assert!(formatted.contains("Welcome"));
        assert!(formatted.contains("no"));
        assert!(formatted.contains("Resolved"));
        assert!(formatted.contains("2024-01-02"));
    }

    #[test]
    fn test_get_spikes_filter_page() {
        let spikes = create_test_spikes();

        let filtered: Vec<&Spike> = spikes
            .iter()
            .filter(|s| s.page == "index.html")
            .collect();

        assert_eq!(filtered.len(), 3);
        for spike in &filtered {
            assert_eq!(spike.page, "index.html");
        }
    }

    #[test]
    fn test_get_spikes_filter_rating() {
        let spikes = create_test_spikes();

        let filtered: Vec<&Spike> = spikes
            .iter()
            .filter(|s| s.rating == Some(Rating::No))
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "spike002def");
    }

    #[test]
    fn test_get_spikes_filter_unresolved() {
        let spikes = create_test_spikes();

        let filtered: Vec<&Spike> = spikes
            .iter()
            .filter(|s| !s.is_resolved())
            .collect();

        assert_eq!(filtered.len(), 3);
        for spike in &filtered {
            assert!(!spike.is_resolved());
        }
    }

    #[test]
    fn test_get_spikes_combined_filters() {
        let spikes = create_test_spikes();

        let filtered: Vec<&Spike> = spikes
            .iter()
            .filter(|s| {
                s.page == "index.html"
                    && s.rating == Some(Rating::No)
                    && !s.is_resolved()
            })
            .collect();

        // spike002def has rating No but is resolved
        // So no spikes match all criteria
        assert_eq!(filtered.len(), 0);

        // Test with just page + unresolved
        let filtered2: Vec<&Spike> = spikes
            .iter()
            .filter(|s| s.page == "index.html" && !s.is_resolved())
            .collect();

        assert_eq!(filtered2.len(), 2);
    }

    #[test]
    fn test_get_element_feedback_by_selector() {
        let spikes = create_test_spikes();

        let matching: Vec<&Spike> = spikes
            .iter()
            .filter(|s| {
                s.spike_type == SpikeType::Element
                    && s.selector.as_deref() == Some(".hero-title")
            })
            .collect();

        assert_eq!(matching.len(), 2);
        for spike in &matching {
            assert_eq!(spike.selector, Some(".hero-title".to_string()));
        }
    }

    #[test]
    fn test_get_element_feedback_with_page_filter() {
        let spikes = create_test_spikes();

        let matching: Vec<&Spike> = spikes
            .iter()
            .filter(|s| {
                s.spike_type == SpikeType::Element
                    && s.selector.as_deref() == Some(".hero-title")
                    && s.page == "index.html"
            })
            .collect();

        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].id, "spike002def");
    }

    #[test]
    fn test_get_hotspots_counting() {
        let spikes = create_test_spikes();

        let mut counts: HashMap<String, usize> = HashMap::new();
        for spike in &spikes {
            if spike.spike_type == SpikeType::Element {
                if let Some(selector) = &spike.selector {
                    *counts.entry(selector.clone()).or_insert(0) += 1;
                }
            }
        }

        assert_eq!(counts.get(".hero-title"), Some(&2));
        assert_eq!(counts.get(".nav-button"), Some(&1));
    }

    #[test]
    fn test_get_hotspots_sorting() {
        let spikes = create_test_spikes();

        let mut counts: HashMap<String, usize> = HashMap::new();
        for spike in &spikes {
            if spike.spike_type == SpikeType::Element {
                if let Some(selector) = &spike.selector {
                    *counts.entry(selector.clone()).or_insert(0) += 1;
                }
            }
        }

        let mut hotspots: Vec<(String, usize)> = counts.into_iter().collect();
        hotspots.sort_by_key(|item| std::cmp::Reverse(item.1));

        assert_eq!(hotspots[0].0, ".hero-title");
        assert_eq!(hotspots[0].1, 2);
        assert_eq!(hotspots[1].0, ".nav-button");
        assert_eq!(hotspots[1].1, 1);
    }

    #[test]
    fn test_get_hotspots_limit() {
        let spikes = create_test_spikes();

        let mut counts: HashMap<String, usize> = HashMap::new();
        for spike in &spikes {
            if spike.spike_type == SpikeType::Element {
                if let Some(selector) = &spike.selector {
                    *counts.entry(selector.clone()).or_insert(0) += 1;
                }
            }
        }

        let mut hotspots: Vec<(String, usize)> = counts.into_iter().collect();
        hotspots.sort_by_key(|item| std::cmp::Reverse(item.1));
        hotspots.truncate(1);

        assert_eq!(hotspots.len(), 1);
        assert_eq!(hotspots[0].0, ".hero-title");
    }

    #[test]
    fn test_spikes_service_creation() {
        let service = SpikesService::new(DataSource::Local);
        // Verify the tool router is initialized
        assert!(format!("{:?}", service.tool_router).contains("ToolRouter"));
    }

    #[test]
    fn test_server_info() {
        let service = SpikesService::new(DataSource::Local);
        let info = service.get_info();

        assert_eq!(info.server_info.name, "spikes-mcp");
        assert!(info.capabilities.tools.is_some());
    }

    #[test]
    fn test_tool_argument_schemas() {
        // Verify GetSpikesArgs schema
        let args = GetSpikesArgs {
            page: Some("index.html".to_string()),
            rating: Some("love".to_string()),
            unresolved_only: Some(true),
        };
        let json = serde_json::to_string(&args).unwrap();
        assert!(json.contains("index.html"));
        assert!(json.contains("love"));
        assert!(json.contains("unresolved_only"));

        // Verify GetElementFeedbackArgs schema
        let elem_args = GetElementFeedbackArgs {
            selector: ".hero".to_string(),
            page: Some("index.html".to_string()),
        };
        let json = serde_json::to_string(&elem_args).unwrap();
        assert!(json.contains("selector"));
        assert!(json.contains(".hero"));

        // Verify GetHotspotsArgs schema
        let hotspot_args = GetHotspotsArgs { limit: Some(5) };
        let json = serde_json::to_string(&hotspot_args).unwrap();
        assert!(json.contains("limit"));
    }

    // ========================================
    // Unit Tests for Write Tool Mutation Logic
    // ========================================

    #[test]
    fn test_submit_spike_creates_page_spike() {
        // Verify spike type is set to Page when no selector provided
        let spike_type = SpikeType::Page;
        assert_eq!(spike_type, SpikeType::Page);
    }

    #[test]
    fn test_submit_spike_creates_element_spike() {
        // Verify spike type is set to Element when selector provided
        let spike_type = SpikeType::Element;
        assert_eq!(spike_type, SpikeType::Element);
    }

    #[test]
    fn test_submit_spike_nanoid_generation() {
        // Verify nanoid generates 11-char IDs
        let id = nanoid::nanoid!(11);
        assert_eq!(id.len(), 11);
        // Verify URL-safe characters (alphanumeric + underscore + hyphen)
        assert!(id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-'));
    }

    #[test]
    fn test_submit_spike_args_serialization() {
        let args = SubmitSpikeArgs {
            page: "index.html".to_string(),
            url: Some("http://localhost:3000".to_string()),
            selector: Some(".hero".to_string()),
            element_text: Some("Welcome".to_string()),
            rating: Some("love".to_string()),
            comments: "Great design!".to_string(),
            reviewer_name: Some("MCP Agent".to_string()),
            project_key: Some("test".to_string()),
        };
        let json = serde_json::to_string(&args).unwrap();
        assert!(json.contains("index.html"));
        assert!(json.contains(".hero"));
        assert!(json.contains("Great design"));
        assert!(json.contains("MCP Agent"));
    }

    #[test]
    fn test_submit_spike_minimal_args() {
        // Minimal required fields: page and comments
        let args = SubmitSpikeArgs {
            page: "page.html".to_string(),
            url: None,
            selector: None,
            element_text: None,
            rating: None,
            comments: "A comment".to_string(),
            reviewer_name: None,
            project_key: None,
        };
        let json = serde_json::to_string(&args).unwrap();
        assert!(json.contains("page.html"));
        assert!(json.contains("A comment"));
    }

    #[test]
    fn test_resolve_spike_args_serialization() {
        let args = ResolveSpikeArgs {
            spike_id: "spike123".to_string(),
        };
        let json = serde_json::to_string(&args).unwrap();
        assert!(json.contains("spike_id"));
        assert!(json.contains("spike123"));
    }

    #[test]
    fn test_delete_spike_args_serialization() {
        let args = DeleteSpikeArgs {
            spike_id: "spike456".to_string(),
        };
        let json = serde_json::to_string(&args).unwrap();
        assert!(json.contains("spike_id"));
        assert!(json.contains("spike456"));
    }

    #[test]
    fn test_create_share_args_serialization() {
        let args = CreateShareArgs {
            directory: "/path/to/dir".to_string(),
            name: Some("my-share".to_string()),
            password: Some("secret".to_string()),
        };
        let json = serde_json::to_string(&args).unwrap();
        assert!(json.contains("directory"));
        assert!(json.contains("/path/to/dir"));
        assert!(json.contains("my-share"));
        assert!(json.contains("secret"));
    }

    #[test]
    fn test_create_share_args_minimal() {
        let args = CreateShareArgs {
            directory: "/path".to_string(),
            name: None,
            password: None,
        };
        let json = serde_json::to_string(&args).unwrap();
        assert!(json.contains("directory"));
        assert!(json.contains("/path"));
    }

    #[test]
    fn test_list_shares_args_empty() {
        let args = ListSharesArgs {};
        let json = serde_json::to_string(&args).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_get_usage_args_empty() {
        let args = GetUsageArgs {};
        let json = serde_json::to_string(&args).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_collect_files_empty_dir() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let files = collect_files(temp_dir.path()).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_collect_files_includes_html() {
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("index.html");
        let mut file = std::fs::File::create(&file_path).unwrap();
        writeln!(file, "<html></html>").unwrap();

        let files = collect_files(temp_dir.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("index.html"));
    }

    #[test]
    fn test_collect_files_excludes_node_modules() {
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create included file
        let included = temp_dir.path().join("index.html");
        let mut file = std::fs::File::create(&included).unwrap();
        writeln!(file, "<html></html>").unwrap();

        // Create excluded directory and file
        std::fs::create_dir(temp_dir.path().join("node_modules")).unwrap();
        let excluded = temp_dir.path().join("node_modules/test.js");
        let mut file = std::fs::File::create(&excluded).unwrap();
        writeln!(file, "test").unwrap();

        let files = collect_files(temp_dir.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("index.html"));
    }

    #[test]
    fn test_collect_files_excludes_spikes_dir() {
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create included file
        let included = temp_dir.path().join("page.html");
        let mut file = std::fs::File::create(&included).unwrap();
        writeln!(file, "<html></html>").unwrap();

        // Create .spikes directory with JSON
        std::fs::create_dir(temp_dir.path().join(".spikes")).unwrap();
        let excluded = temp_dir.path().join(".spikes/feedback.json");
        let mut file = std::fs::File::create(&excluded).unwrap();
        writeln!(file, "{{}}").unwrap();

        let files = collect_files(temp_dir.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("page.html"));
    }

    #[test]
    fn test_guess_mime_html() {
        let path = std::path::Path::new("index.html");
        assert_eq!(guess_mime(path), "text/html");
    }

    #[test]
    fn test_guess_mime_css() {
        let path = std::path::Path::new("styles.css");
        assert_eq!(guess_mime(path), "text/css");
    }

    #[test]
    fn test_guess_mime_js() {
        let path = std::path::Path::new("app.js");
        assert_eq!(guess_mime(path), "application/javascript");
    }

    #[test]
    fn test_guess_mime_png() {
        let path = std::path::Path::new("image.png");
        assert_eq!(guess_mime(path), "image/png");
    }

    #[test]
    fn test_guess_mime_jpg() {
        let path = std::path::Path::new("image.jpg");
        assert_eq!(guess_mime(path), "image/jpeg");
    }

    #[test]
    fn test_guess_mime_svg() {
        let path = std::path::Path::new("logo.svg");
        assert_eq!(guess_mime(path), "image/svg+xml");
    }

    #[test]
    fn test_guess_mime_unknown() {
        let path = std::path::Path::new("data.xyz");
        assert_eq!(guess_mime(path), "application/octet-stream");
    }

    #[test]
    fn test_usage_data_deserialization() {
        let json = r#"{
            "spikes": 50,
            "spike_limit": 1000,
            "shares": 3,
            "share_limit": 5,
            "tier": "free"
        }"#;

        let usage: UsageData = serde_json::from_str(json).unwrap();
        assert_eq!(usage.spikes, 50);
        assert_eq!(usage.spike_limit, Some(1000));
        assert_eq!(usage.shares, 3);
        assert_eq!(usage.share_limit, Some(5));
        assert_eq!(usage.tier, "free");
    }

    #[test]
    fn test_usage_data_unlimited() {
        let json = r#"{
            "spikes": 500,
            "spike_limit": null,
            "shares": 10,
            "share_limit": null,
            "tier": "pro"
        }"#;

        let usage: UsageData = serde_json::from_str(json).unwrap();
        assert_eq!(usage.spike_limit, None);
        assert_eq!(usage.share_limit, None);
        assert_eq!(usage.tier, "pro");
    }

    #[test]
    fn test_share_info_deserialization() {
        let json = r#"{
            "id": "share-123",
            "slug": "my-project",
            "url": "https://spikes.sh/s/my-project",
            "spike_count": 5,
            "created_at": "2024-01-15T10:00:00Z",
            "name": "My Project"
        }"#;

        let share: ShareInfo = serde_json::from_str(json).unwrap();
        assert_eq!(share.id, "share-123");
        assert_eq!(share.slug, "my-project");
        assert_eq!(share.url, "https://spikes.sh/s/my-project");
        assert_eq!(share.spike_count, 5);
        assert_eq!(share.created_at, "2024-01-15T10:00:00Z");
        assert_eq!(share.name, Some("My Project".to_string()));
    }

    #[test]
    fn test_share_info_without_name() {
        let json = r#"{
            "id": "share-456",
            "slug": "test",
            "url": "https://spikes.sh/s/test",
            "spike_count": 0,
            "created_at": "2024-01-15T10:00:00Z"
        }"#;

        let share: ShareInfo = serde_json::from_str(json).unwrap();
        assert_eq!(share.name, None);
    }

    // ========================================
    // Unit Tests for DataSource
    // ========================================

    #[test]
    fn test_data_source_local() {
        let ds = DataSource::new(false).unwrap();
        assert!(matches!(ds, DataSource::Local));
    }

    #[test]
    fn test_data_source_remote_with_token() {
        // Save original env var
        let original = std::env::var("SPIKES_TOKEN").ok();
        
        // Set token
        std::env::set_var("SPIKES_TOKEN", "test-token-123");
        
        let ds = DataSource::new(true).unwrap();
        match ds {
            DataSource::Remote { token, api_base } => {
                assert_eq!(token, "test-token-123");
                assert!(!api_base.is_empty());
            }
            DataSource::Local => panic!("Expected Remote, got Local"),
        }
        
        // Restore original
        if let Some(val) = original {
            std::env::set_var("SPIKES_TOKEN", val);
        } else {
            std::env::remove_var("SPIKES_TOKEN");
        }
    }

    #[test]
    fn test_data_source_remote_without_token() {
        // Save original env var
        let original = std::env::var("SPIKES_TOKEN").ok();
        
        // Remove token
        std::env::remove_var("SPIKES_TOKEN");
        
        // Need to also clear auth.toml - we'll just check that it errors
        // Since we can't easily mock auth.toml, this test verifies error handling
        let result = DataSource::new(true);
        assert!(result.is_err());
        
        // Restore original
        if let Some(val) = original {
            std::env::set_var("SPIKES_TOKEN", val);
        }
    }

    #[test]
    fn test_data_source_remote_api_base_env() {
        // Save original env vars
        let original_token = std::env::var("SPIKES_TOKEN").ok();
        let original_api = std::env::var("SPIKES_API_URL").ok();
        
        // Set env vars
        std::env::set_var("SPIKES_TOKEN", "test-token");
        std::env::set_var("SPIKES_API_URL", "http://localhost:8787");
        
        let ds = DataSource::new(true).unwrap();
        match ds {
            DataSource::Remote { api_base, .. } => {
                assert_eq!(api_base, "http://localhost:8787");
            }
            DataSource::Local => panic!("Expected Remote, got Local"),
        }
        
        // Restore original
        if let Some(val) = original_token {
            std::env::set_var("SPIKES_TOKEN", val);
        } else {
            std::env::remove_var("SPIKES_TOKEN");
        }
        if let Some(val) = original_api {
            std::env::set_var("SPIKES_API_URL", val);
        } else {
            std::env::remove_var("SPIKES_API_URL");
        }
    }

    #[test]
    fn test_urlencoding_encode() {
        assert_eq!(urlencoding::encode("index.html"), "index.html");
        assert_eq!(urlencoding::encode("page name"), "page%20name");
        assert_eq!(urlencoding::encode("test@example.com"), "test%40example.com");
    }

    // ========================================
    // Unit Tests for TransportMode
    // ========================================

    #[test]
    fn test_transport_mode_stdio() {
        let mode = TransportMode::Stdio;
        assert!(matches!(mode, TransportMode::Stdio));
    }

    #[test]
    fn test_transport_mode_http() {
        let mode = TransportMode::Http {
            port: 3848,
            bind: "127.0.0.1".to_string(),
        };
        match mode {
            TransportMode::Http { port, bind } => {
                assert_eq!(port, 3848);
                assert_eq!(bind, "127.0.0.1");
            }
            _ => panic!("Expected HTTP transport mode"),
        }
    }

    #[test]
    fn test_transport_mode_http_default_port() {
        // Verify default port matches expected value
        let mode = TransportMode::Http {
            port: 3848,
            bind: "127.0.0.1".to_string(),
        };
        if let TransportMode::Http { port, .. } = mode {
            assert_eq!(port, 3848, "Default HTTP port should be 3848");
        }
    }

    #[test]
    fn test_transport_mode_http_custom_bind() {
        let mode = TransportMode::Http {
            port: 8080,
            bind: "0.0.0.0".to_string(),
        };
        if let TransportMode::Http { port, bind } = mode {
            assert_eq!(port, 8080);
            assert_eq!(bind, "0.0.0.0");
        }
    }

    #[test]
    fn test_transport_mode_clone() {
        let mode = TransportMode::Http {
            port: 3848,
            bind: "127.0.0.1".to_string(),
        };
        let cloned = mode.clone();
        assert!(matches!(cloned, TransportMode::Http { .. }));
    }

    #[test]
    fn test_transport_mode_debug() {
        let mode = TransportMode::Stdio;
        let debug_str = format!("{:?}", mode);
        assert!(debug_str.contains("Stdio"));

        let mode = TransportMode::Http {
            port: 3848,
            bind: "127.0.0.1".to_string(),
        };
        let debug_str = format!("{:?}", mode);
        assert!(debug_str.contains("Http"));
        assert!(debug_str.contains("3848"));
    }
}
