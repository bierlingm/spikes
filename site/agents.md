# Spikes — Agent Integration Guide

> Spikes is the feedback loop tool for AI-assisted building. Collect structured feedback with exact CSS selectors, ratings, and comments — then feed it to your agent as JSON via MCP.

## What Spikes Does

Spikes turns visual feedback into structured, machine-readable data:

- **Element-level feedback** with exact CSS selectors your agent can act on
- **Page-level feedback** with ratings (love / like / meh / no) and comments
- **JSON output** everywhere — CLI, MCP, API all speak structured data
- **MCP integration** — 9 tools for reading, writing, and managing feedback
- **Two workflows**: (1) You reviewing agent work, (2) Collecting feedback from others via shareable links

## Authentication

Spikes uses API keys with the `sk_spikes_` prefix for agent authentication.

### Create an API Key

```bash
spikes auth create-key --name "my-agent"
```

Or via the REST API (no auth required):

```bash
curl -X POST https://spikes.sh/auth/api-key \
  -H "Content-Type: application/json" \
  -d '{"name": "my-agent"}'
```

The raw key is returned **once** at creation. Store it securely.

### Use Your API Key

Pass the key as a Bearer token:

```bash
curl https://spikes.sh/me \
  -H "Authorization: Bearer sk_spikes_your_key_here"
```

### Key Management

```bash
spikes auth list-keys       # List all keys with status
spikes auth revoke-key KEY  # Revoke a key
```

### Scopes

API keys support scoped access:

| Scope  | Allows                          |
|--------|---------------------------------|
| `full` | All read and write operations   |
| `read` | GET endpoints only              |
| `write`| POST, DELETE, PATCH only        |

## MCP Tools Reference

Spikes exposes **9 MCP tools** via `spikes mcp serve`. These tools let AI agents read, write, and manage feedback programmatically through the Model Context Protocol.

### Read Tools

#### get_spikes
Get all feedback spikes with optional filters.

| Parameter       | Type    | Required | Description                              |
|-----------------|---------|----------|------------------------------------------|
| page            | String  | No       | Filter by page name (e.g., 'index.html') |
| rating          | String  | No       | Filter by rating: love, like, meh, or no |
| unresolved_only | Boolean | No       | Only return unresolved spikes             |

**Returns:** Formatted text with spike details (ID, type, page, rating, comments, reviewer, status).

#### get_element_feedback
Get feedback for a specific CSS selector.

| Parameter | Type   | Required | Description                  |
|-----------|--------|----------|------------------------------|
| selector  | String | Yes      | CSS selector to look up      |
| page      | String | No       | Filter by page name          |

**Returns:** Matching element feedback with selector, rating, and comments.

#### get_hotspots
Find elements with the most feedback.

| Parameter | Type    | Required | Description                                 |
|-----------|---------|----------|---------------------------------------------|
| limit     | Integer | No       | Maximum number of hotspots to return (default: 10) |

**Returns:** Ranked list of CSS selectors by feedback count.

### Write Tools

#### submit_spike
Create new feedback programmatically.

| Parameter     | Type   | Required | Description                            |
|---------------|--------|----------|----------------------------------------|
| page          | String | Yes      | Page name (e.g., 'index.html')         |
| comments      | String | Yes      | Feedback comments                      |
| url           | String | No       | URL of the page                        |
| selector      | String | No       | CSS selector (makes it element-level)  |
| element_text  | String | No       | Text content of the targeted element   |
| rating        | String | No       | Rating: love, like, meh, or no         |
| reviewer_name | String | No       | Reviewer name (default: "MCP Agent")   |
| project_key   | String | No       | Project key (default: "default")       |

**Returns:** Confirmation with spike ID.

#### resolve_spike
Mark a spike as resolved.

| Parameter | Type   | Required | Description                              |
|-----------|--------|----------|------------------------------------------|
| spike_id  | String | Yes      | Spike ID or prefix (minimum 4 characters)|

**Returns:** Confirmation with resolved timestamp.

#### delete_spike
Permanently remove a spike.

| Parameter | Type   | Required | Description                              |
|-----------|--------|----------|------------------------------------------|
| spike_id  | String | Yes      | Spike ID or prefix (minimum 4 characters)|

**Returns:** Confirmation of deletion.

### Share & Usage Tools

#### create_share
Upload a directory and get a shareable URL.

| Parameter | Type   | Required | Description              |
|-----------|--------|----------|--------------------------|
| directory | String | Yes      | Directory path to upload  |
| name      | String | No       | Name/slug for the share   |
| password  | String | No       | Password protection       |

**Returns:** Share URL, slug, and file count.

#### list_shares
List all your shares.

No parameters required.

**Returns:** Share metadata (slug, URL, spike count, creation date).

#### get_usage
View usage statistics.

No parameters required.

**Returns:** Spike/share counts, limits, tier, and cost info (agent tier).

## Rate Limits

| Endpoint              | Limit                  |
|-----------------------|------------------------|
| POST /auth/api-key    | 10 per hour per IP     |
| POST /spikes          | Per-share spike limits  |
| POST /shares          | Per-tier share limits   |
| All authenticated     | Fair use               |

Budget controls are available for agent-tier keys via `monthly_cap_cents`. When the budget is exceeded, the API returns `429 BUDGET_EXCEEDED`.

## Pricing Tiers

| Tier    | Price                      | Spikes        | Shares    |
|---------|----------------------------|---------------|-----------|
| Free    | $0                         | 1,000/month   | 5         |
| Agent   | $0.001/spike, $0.01/share  | Metered       | Metered   |
| Pro     | $9–$29+/month              | Unlimited     | Unlimited |

### Agent Tier

The agent tier is consumption-based — pay only for what you use:

- **$0.001 per spike** submitted
- **$0.01 per share** created
- Set a **budget cap** (`monthly_cap_cents`) on your API key to control spend
- No cap = unlimited usage
- Budget enforcement: `429 BUDGET_EXCEEDED` when cap is hit

## Quickstart

### 1. Install the CLI

```bash
curl -fsSL https://spikes.sh/install.sh | sh
```

### 2. Create an API Key

```bash
spikes auth create-key --name "my-agent"
```

### 3. Configure MCP

**Claude Desktop** (`~/Library/Application Support/Claude/claude_desktop_config.json`):

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

**Cursor** (`.cursor/mcp.json` in your project root):

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

Or auto-detect your client:

```bash
spikes mcp install
```

### 4. First Query

Ask your agent:

> "Use the get_spikes tool to show me all feedback"

### 5. Submit Feedback Programmatically

```bash
# Via MCP tool
submit_spike(page: "index.html", comments: "Button too small", selector: ".cta-btn", rating: "no")

# Via REST API
curl -X POST https://spikes.sh/spikes \
  -H "Authorization: Bearer sk_spikes_your_key" \
  -H "Content-Type: application/json" \
  -d '{"page": "index.html", "comments": "Button too small", "selector": ".cta-btn", "rating": "no", "projectKey": "default"}'
```

## MCP Transport Modes

| Mode   | Command                                            | Use Case                     |
|--------|----------------------------------------------------|------------------------------|
| stdio  | `spikes mcp serve`                                 | Claude Desktop, Cursor       |
| HTTP   | `spikes mcp serve --transport http --port 3848`    | Sandboxed agents (Devin, Codex) |
| Remote | `spikes mcp serve --remote`                        | Read/write against hosted API|

Modes compose: `spikes mcp serve --transport http --remote` serves MCP over HTTP using the hosted API.

## Links

- **Website:** [https://spikes.sh](https://spikes.sh)
- **GitHub:** [https://github.com/moritzbierling/spikes](https://github.com/moritzbierling/spikes)
- **npm:** `npx spikes-mcp` (no install needed)
- **Smithery:** [spikes on Smithery](https://smithery.ai/server/spikes)
- **llms.txt:** [https://spikes.sh/llms.txt](https://spikes.sh/llms.txt)
