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
| `url_prefix` | `string?` | Only spikes whose URL starts with this prefix (e.g. `"/versions/v0-5/"`) |
| `since` | `string?` | Only spikes created or updated after this ISO 8601 timestamp |

**Example:**
```json
{
  "page": "about.html",
  "rating": "no",
  "unresolved_only": true
}
```

Returns formatted text with spike IDs, ratings, selectors, comments, reviewer names, and resolution status. Hosted spikes also carry `Outcome` (open / addressed / wont_do), `Addressed in`, `Version`, `Replies`, and the last reply.

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

### `submit_spike`, `resolve_spike`, `delete_spike`, `create_share`, `list_shares`, `get_usage`

Write and account tools; see the [README](../README.md#mcp-server--16-tools) table for a one-line summary of each.

---

## Feedback-loop tools (hosted mode)

These tools need `spikes mcp serve --remote` (a `SPIKES_TOKEN`, a `spikes login`, or a project key in `.spikes/config.toml`). In local mode they return a clear "hosted only" error. Tools that act on the project read `[project].key` from `.spikes/config.toml`.

### `reply_to_spike`

Answer a reviewer; the reply appears on the page where the comment was left.

| Parameter | Type | Description |
|-----------|------|-------------|
| `spike_id` | `string` | **Required.** Full spike ID |
| `body` | `string` | **Required.** Reply text |
| `version_label` | `string?` | Version the reply refers to, e.g. `"v0.5"` |
| `status` | `string?` | Set the spike status in the same call: `open`, `addressed`, `wont_do` |
| `addressed_in` | `string?` | Version label recorded as `addressed_in` |

### `set_spike_status`

| Parameter | Type | Description |
|-----------|------|-------------|
| `spike_id` | `string` | **Required.** Full spike ID |
| `status` | `string` | **Required.** `open`, `addressed`, or `wont_do` |
| `addressed_in` | `string?` | Version label |

### `list_versions` / `add_version`

`list_versions` takes no parameters and returns each version's label, URL prefix, spike count, open count, and notes.

`add_version`:

| Parameter | Type | Description |
|-----------|------|-------------|
| `label` | `string` | **Required.** e.g. `"v0.5"` |
| `url_prefix` | `string` | **Required.** URL prefix of this version's pages, e.g. `"/versions/v0-5/"` |
| `notes` | `string?` | What changed, shown to reviewers |

### `list_questions` / `ask_question` / `get_question_answers`

| Tool | Parameters |
|------|------------|
| `list_questions` | `status?` (`open` default, or `closed`) |
| `ask_question` | `title` (required), `body?` |
| `get_question_answers` | `question_id` (required) |

---

## Turn-start pattern

Feedback only helps if the agent sees it before it starts working. At the start of every session:

1. Call `get_spikes` with `unresolved_only: true` and `since` set to the time of the last session (omit `since` on the first run).
2. Call `list_questions` to see whether the reviewer answered anything.
3. Do the work.
4. For every spike you acted on, call `reply_to_spike` with `status: "addressed"` and the version label, or `status: "wont_do"` with the reason. The reviewer sees the outcome on the page.

The server sends this guidance to MCP clients as its `instructions` string, so agents that read server instructions pick it up without extra prompting. For a push instead of a poll, run `spikes watch --exec 'herdr agent prompt <agent>'` next to the session.

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
