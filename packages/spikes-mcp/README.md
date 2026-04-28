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

This wrapper downloads the appropriate platform binary and starts the Spikes MCP server on stdio. The server exposes 9 tools for AI agents to read, write, and manage structured feedback:

- `get_spikes` — List feedback with filters
- `get_element_feedback` — Get feedback for specific elements
- `get_hotspots` — Find elements with most feedback
- `submit_spike` — Create feedback programmatically
- `resolve_spike` — Mark feedback as addressed
- `delete_spike` — Remove a spike
- `create_share` — Upload files for sharing
- `list_shares` — See active shares
- `get_usage` — Check usage stats and limits

## Environment Variables

- `SPIKES_TOKEN` — Bearer token for hosted API (optional, only needed for remote mode)
- `SPIKES_API_URL` — Override the API base URL (defaults to https://spikes.sh/api)

## More Information

- Website: https://spikes.sh
- Full project docs: See the [root README](../../README.md)
- MCP docs: https://modelcontextprotocol.io

## License

MIT — see the root repository for full license text.
