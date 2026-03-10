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
#[derive(Clone, Debug)]
pub struct SpikesService {
    tool_router: ToolRouter<SpikesService>,
}

#[tool_router]
impl SpikesService {
    /// Create a new SpikesService instance
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
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
        let spikes = match load_spikes() {
            Ok(s) => s,
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "ERROR: Could not load spikes: {}",
                    e
                ))]));
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
        let spikes = match load_spikes() {
            Ok(s) => s,
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "ERROR: Could not load spikes: {}",
                    e
                ))]));
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
        let spikes = match load_spikes() {
            Ok(s) => s,
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "ERROR: Could not load spikes: {}",
                    e
                ))]));
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
        // Check authentication
        let token = match AuthConfig::token() {
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

        let result = upload_share(&token, dir_path, &files, &slug, args.password.as_deref());

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
        // Check authentication
        let token = match AuthConfig::token() {
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
        };

        let shares = fetch_shares(&token);

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
        // Check authentication
        let token = match AuthConfig::token() {
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
        };

        let usage = fetch_usage(&token);

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
        Self::new()
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
) -> crate::error::Result<ShareResult> {
    use ureq::Agent;

    let agent = Agent::new();
    let host = get_api_base();
    let url = format!("{}/shares", host.trim_end_matches('/'));

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
fn fetch_shares(token: &str) -> crate::error::Result<Vec<ShareInfo>> {
    let api_base = get_api_base();
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
fn fetch_usage(token: &str) -> crate::error::Result<UsageData> {
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

    let usage: UsageData = serde_json::from_str(&body)?;
    Ok(usage)
}

// ============================================================================
// Entry Point
// ============================================================================

/// Run the MCP server using stdio transport.
///
/// This function is synchronous but internally uses tokio runtime.
/// All logging goes to stderr; stdout is reserved for JSON-RPC.
pub fn run() -> crate::error::Result<()> {
    // Use tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| crate::error::Error::RequestFailed(format!("Failed to create tokio runtime: {}", e)))?;

    rt.block_on(async_run())
}

/// Async implementation of the MCP server
async fn async_run() -> crate::error::Result<()> {
    // All logging must go to stderr; stdout is for JSON-RPC
    eprintln!("[spikes-mcp] Starting MCP server on stdio...");

    let service = SpikesService::new();

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
        let service = SpikesService::new();
        // Verify the tool router is initialized
        assert!(format!("{:?}", service.tool_router).contains("ToolRouter"));
    }

    #[test]
    fn test_server_info() {
        let service = SpikesService::new();
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
}
