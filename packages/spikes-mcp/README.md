# spikes-mcp

MCP server wrapper for Spikes — zero-install Model Context Protocol server for structured UI feedback.

## Usage

Zero-install with npx:

```bash
npx spikes-mcp
```

Or install globally:

```bash
npm install -g spikes-mcp
spikes-mcp
```

## What It Does

This wrapper downloads the appropriate platform binary and starts the Spikes MCP server on stdio. The server exposes 16 tools for AI agents to read, write, and manage structured feedback:

- `get_spikes` — List feedback with filters
- `get_element_feedback` — Get feedback for specific elements
- `get_hotspots` — Find elements with most feedback
- `submit_spike` — Create feedback programmatically
- `resolve_spike` — Mark feedback as addressed
- `delete_spike` — Remove a spike
- `create_share` — Upload files for sharing
- `list_shares` — See active shares
- `get_usage` — Check usage stats and limits
- `reply_to_spike` — Answer a reviewer on the page, with optional status and version (hosted)
- `set_spike_status` — Mark a spike open, addressed, or won't do (hosted)
- `list_versions`, `add_version` — Review versions per project (hosted)
- `list_questions`, `ask_question`, `get_question_answers` — Questions for reviewers (hosted)

## Environment Variables

- `SPIKES_TOKEN` — Bearer token or `sk_spikes_` key for the hosted API (remote mode). A project-scoped key limits the agent to one project.
- `SPIKES_API_URL` — Override the API base URL (defaults to https://spikes.sh/api)

## More Information

- Website: https://spikes.sh
- Full project docs: See the [root README](../../README.md)
- MCP docs: https://modelcontextprotocol.io

## License

MIT — see the root repository for full license text.
