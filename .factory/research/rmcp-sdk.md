# rmcp SDK Research (v0.17)

Official Rust MCP SDK: https://github.com/modelcontextprotocol/rust-sdk
Crates.io: https://crates.io/crates/rmcp
Docs: https://docs.rs/rmcp

## Dependencies

```toml
rmcp = { version = "0.17", features = ["server", "transport-io", "macros"] }
schemars = "1.0"
```

## Pattern: stdio MCP Server

```rust
use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::{ErrorData as McpError, *},
    schemars, tool, tool_handler, tool_router, ServerHandler, ServiceExt,
    transport::stdio,
};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct SpikesService {
    tool_router: ToolRouter<SpikesService>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetSpikesRequest {
    #[schemars(description = "Filter by page name")]
    pub page: Option<String>,
}

#[tool_router]
impl SpikesService {
    pub fn new() -> Self {
        Self { tool_router: Self::tool_router() }
    }

    #[tool(description = "Get all feedback spikes")]
    async fn get_spikes(
        &self,
        Parameters(request): Parameters<GetSpikesRequest>,
    ) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text("result")]))
    }
}

#[tool_handler]
impl ServerHandler for SpikesService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Spikes MCP server".to_string()),
        }
    }
}

// Entry point
async fn serve() -> anyhow::Result<()> {
    let service = SpikesService::new();
    let server = service.serve(stdio()).await?;
    server.waiting().await?;
    Ok(())
}
```

## Key Notes

- `#[tool_router]` generates routing table automatically
- `#[tool(description = "...")]` registers function as MCP tool
- `#[tool_handler]` implements ServerHandler for tool dispatch
- Parameters use `schemars::JsonSchema` derive for schema generation
- `schemars` v1.0 required (not 0.8)
- stdio transport: `rmcp::transport::stdio` (feature: "transport-io")
- Logging MUST go to stderr (stdout is JSON-RPC): use `tracing_subscriber::fmt().with_writer(std::io::stderr)`
- The SDK is async — needs tokio runtime
- `CallToolResult::success(vec![Content::text(s)])` for success responses
- `McpError` (aliased from `ErrorData`) for error responses
