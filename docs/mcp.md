# MCP Server — Agent Native Feedback

Feed your AI agent structured feedback directly. No copy-paste. No context switching.

---

## What It Is

The Spikes MCP (Model Context Protocol) server exposes your feedback as queryable tools. Agents like Claude and Cursor can ask questions like _"What's blocking?"_ or _"What did they say about the nav?"_ and get structured answers.

Runs on stdio. Zero network config. Zero fuss.

---

## Quick Start

```bash
spikes mcp serve
```

That's it. The server starts and listens on stdin.

---

## Configure Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "spikes": {
      "command": "spikes",
      "args": ["mcp", "serve"]
    }
  }
}
```

Location:
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%/Claude/claude_desktop_config.json`

---

## Configure Cursor

Add to `.cursor/mcp.json` (project-level) or Cursor settings:

```json
{
  "mcpServers": {
    "spikes": {
      "command": "spikes",
      "args": ["mcp", "serve"],
      "env": {}
    }
  }
}
```

Or via Cursor Settings → MCP → Add Server.

---

## Available Tools

### `get_spikes`

Dig into the feedback pile. Returns all spikes with optional filters.

| Parameter | Type | Description |
|-----------|------|-------------|
| `page` | `string?` | Filter by page (e.g., `"index.html"`) |
| `rating` | `string?` | Filter by rating: `love`, `like`, `meh`, `no` |
| `unresolved_only` | `boolean?` | Only return unresolved spikes |

**Example:**
```json
{
  "page": "about.html",
  "rating": "no",
  "unresolved_only": true
}
```

Returns formatted text with spike IDs, ratings, selectors, comments, reviewer names, and resolution status.

---

### `get_element_feedback`

Target lock on a specific CSS selector. Zoom in on what reviewers said about one element.

| Parameter | Type | Description |
|-----------|------|-------------|
| `selector` | `string` | **Required.** CSS selector (e.g., `".hero-title"`) |
| `page` | `string?` | Optional page filter |

**Example:**
```json
{
  "selector": ".nav-button",
  "page": "index.html"
}
```

Returns all feedback for that element across all reviewers.

---

### `get_hotspots`

Heat map mode. Find elements with the most feedback.

| Parameter | Type | Description |
|-----------|------|-------------|
| `limit` | `number?` | Max hotspots to return (default: 10) |

**Example:**
```json
{
  "limit": 5
}
```

Returns ranked list: selector + count.

---

## Example Session

**Agent:** _"Check my feedback hotspots."_  
→ Calls `get_hotspots` with `limit: 10`

```
Top 3 hotspot(s):

1. .hero-title (4 feedback items)
2. .cta-button (2 feedback items)
3. .mobile-nav (2 feedback items)
```

**Agent:** _"What did they say about the hero title?"_  
→ Calls `get_element_feedback` with `selector: ".hero-title"`

```
Found 4 feedback item(s) for '.hero-title':

[spike0a1b] element on index.html
  Rating: no
  Selector: .hero-title
  Element text: Welcome to Spikes
  Comments: Font too small on mobile
  Reviewer: Alice
  Timestamp: 2024-03-15T10:30:00Z
  Status: Unresolved

[spike2c4d] element on index.html
  Rating: meh
  Selector: .hero-title
  Element text: Welcome to Spikes
  Comments: Contrast could be better
  Reviewer: Bob
  Timestamp: 2024-03-15T11:15:00Z
  Status: Unresolved
...
```

**Agent:** _"Give me all unresolved 'no' ratings."_  
→ Calls `get_spikes` with `rating: "no"`, `unresolved_only: true`

```
Found 2 spike(s):

[spike0a1b] element on index.html
  Rating: no
  ...

[spike9f8e] element on about.html
  Rating: no
  ...
```

---

## Troubleshooting

### "No spikes found matching the criteria"

Working directory must contain `.spikes/` directory with `feedback.jsonl`. Run from project root or run `spikes init` first.

### "ERROR: Could not load spikes"

Check that `.spikes/feedback.jsonl` exists and is readable. The MCP server reads from local storage, same as the CLI.

### Logs go to stderr

All MCP logging goes to stderr. stdout is reserved for JSON-RPC.

```bash
spikes mcp serve 2> mcp.log
```

### Agent can't see tools

- Confirm `spikes` binary is in PATH
- Test manually: `spikes mcp serve` should start without error
- Check config file syntax (trailing commas, valid JSON)
- Restart the host application after config changes

### Empty results

- Verify you're in the right directory (must have `.spikes/`)
- Run `spikes list` to confirm feedback exists
- Check that spikes haven't been resolved (`get_spikes` with `unresolved_only: false`)

---

**Your agent now has eyes. Use them wisely.**
