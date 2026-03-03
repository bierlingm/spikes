//! MCP (Model Context Protocol) server implementation using rmcp SDK.
//!
//! Exposes spikes feedback as tools for AI agent integration.
//! All logging goes to stderr; stdout is reserved for JSON-RPC.

use std::collections::HashMap;

use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars::JsonSchema,
    tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};

use crate::spike::{Rating, Spike, SpikeType};
use crate::storage::load_spikes;

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
        hotspots.sort_by(|a, b| b.1.cmp(&a.1));
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
        hotspots.sort_by(|a, b| b.1.cmp(&a.1));

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
        hotspots.sort_by(|a, b| b.1.cmp(&a.1));
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
}
