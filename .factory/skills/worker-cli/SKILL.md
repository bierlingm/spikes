---
name: worker-cli
description: Implements Rust CLI changes and GitHub Action artifacts (tests, commands, export formats, action files)
---

# CLI Worker

NOTE: Startup and cleanup are handled by `worker-base`. This skill defines the WORK PROCEDURE.

## When to Use This Skill

Features that modify the Rust CLI at `./cli/` or create GitHub Action files at `./action/`. This includes:
- MCP server implementation (rmcp SDK)
- Export format additions
- GitHub Action (action.yml, check.sh, README)
- Integration tests and unit tests
- Dependency changes in Cargo.toml

## Work Procedure

1. **Read the feature description thoroughly.** Understand preconditions, expected behavior, and verification steps.

2. **Understand current code.** Read relevant source files in `./cli/src/`. Key files:
   - `main.rs` — Command routing (clap derive). MCP already wired.
   - `commands/mcp.rs` — MCP server (REPLACE entirely, has syntax error)
   - `commands/export.rs` — Export formats (EXTEND with new variants)
   - `spike.rs` — Data models (Spike, Rating, SpikeType, is_resolved())
   - `storage.rs` — load_spikes() reads .spikes/feedback.jsonl
   - `error.rs` — Error types
   - `Cargo.toml` — Dependencies

3. **Write tests FIRST (TDD).**
   - **Unit tests:** Add `#[cfg(test)] mod tests { ... }` in the same file as the code being tested.
   - **Integration tests:** Add/extend files in `./cli/tests/` using `assert_cmd` and `predicates`.
   - For MCP tests: test tool logic (filtering, matching) as unit tests. Test JSON-RPC protocol flow by piping to binary in integration tests.
   - For export tests: create fixture spike data, test output contains expected sections.
   - Run `cd cli && cargo test` — new tests MUST fail (red phase).

4. **Add dependencies if needed.** For MCP feature:
   - Add `rmcp = { version = "0.17", features = ["server", "transport-io", "macros"] }` to [dependencies]
   - Add `schemars = "1.0"` to [dependencies]
   - tokio is already a dependency.

5. **Implement the feature.** Make the tests pass (green phase).
   - For MCP: Use rmcp macros (#[tool_router], #[tool], #[tool_handler]). Implement SpikesService struct.
   - For exports: Add enum variants, implement markdown generation functions.
   - For action: Create action/ directory with action.yml, check.sh (chmod +x), README.md.
   - Follow existing patterns. No unwrap() in production. Punk/zine energy in user-facing strings.

6. **Run all tests.** `cd cli && cargo test` — ALL tests must pass (existing + new).

7. **Build check.** `cd cli && cargo build` — must succeed with zero errors.

8. **Manually verify.** Run the command directly:
   - MCP: `echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | cargo run -- mcp serve`
   - Export: Create test .spikes/ dir, run `cargo run -- export --format cursor-context`
   - Action: Run `bash action/check.sh` with fixture data

9. **Commit with descriptive message.**

## Example Handoff

```json
{
  "salientSummary": "Rewrote MCP server using rmcp 0.17 SDK with #[tool_router] macro. Exposes get_spikes (page/rating/unresolved filters), get_element_feedback (selector), get_hotspots (limit). Added 10 unit tests for tool logic and 2 integration tests piping JSON-RPC to binary. All 172 tests pass.",
  "whatWasImplemented": "Complete MCP server rewrite in cli/src/commands/mcp.rs using rmcp SDK. SpikesService struct with 3 tools via #[tool_router]. Async stdio transport. Added rmcp 0.17 and schemars 1.0 to Cargo.toml. Replaced hand-rolled JSON-RPC with SDK macros. Server info: spikes-mcp, protocol 2024-11-05.",
  "whatWasLeftUndone": "",
  "verification": {
    "commandsRun": [
      { "command": "cd cli && cargo test", "exitCode": 0, "observation": "172 tests pass including 12 new MCP tests" },
      { "command": "cd cli && cargo build", "exitCode": 0, "observation": "No errors" },
      { "command": "echo '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}' | cargo run -- mcp serve 2>/dev/null", "exitCode": 0, "observation": "Response contains protocolVersion, capabilities.tools, serverInfo.name=spikes-mcp" },
      { "command": "printf '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\\n{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\\n' | cargo run -- mcp serve 2>/dev/null", "exitCode": 0, "observation": "Two valid JSON responses, tools/list shows 3 tools" }
    ],
    "interactiveChecks": []
  },
  "tests": {
    "added": [
      {
        "file": "cli/src/commands/mcp.rs",
        "cases": [
          { "name": "test_get_spikes_no_filter", "verifies": "returns all spikes" },
          { "name": "test_get_spikes_filter_page", "verifies": "page filter works" },
          { "name": "test_get_spikes_filter_rating", "verifies": "rating filter works" },
          { "name": "test_get_spikes_unresolved_only", "verifies": "unresolved filter works" },
          { "name": "test_get_element_feedback_by_selector", "verifies": "selector matching" },
          { "name": "test_get_element_feedback_missing_selector", "verifies": "error on missing required param" },
          { "name": "test_get_hotspots_default_limit", "verifies": "returns top 10 sorted desc" },
          { "name": "test_get_hotspots_custom_limit", "verifies": "limit parameter respected" },
          { "name": "test_get_hotspots_empty", "verifies": "no element feedback message" },
          { "name": "test_tools_list", "verifies": "3 tools returned with schemas" }
        ]
      },
      {
        "file": "cli/tests/mcp_integration.rs",
        "cases": [
          { "name": "test_mcp_initialize", "verifies": "JSON-RPC initialize handshake" },
          { "name": "test_mcp_sequential_requests", "verifies": "multiple requests on same connection" }
        ]
      }
    ]
  },
  "discoveredIssues": []
}
```

## When to Return to Orchestrator

- rmcp crate fails to compile on this platform/toolchain
- The rmcp API has changed from what's documented (breaking changes in 0.17)
- A new dependency is needed beyond rmcp and schemars
- Feature requires changes to spike.rs Spike struct that could break other commands
- cargo test reveals failures in existing (non-MCP) tests that aren't caused by your changes
