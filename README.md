<div align="center">

# <span>/</span> Spikes

**The feedback loop for AI-assisted building**

[![MIT License](https://img.shields.io/badge/license-MIT-e74c3c?style=flat-square)](LICENSE)
[![Widget Size](https://img.shields.io/badge/widget-14KB_gzipped-22c55e?style=flat-square)](#)
[![Free + Pro](https://img.shields.io/badge/free_+_pro-3b82f6?style=flat-square)](#pricing)

AI can build a prototype in an hour.<br>
Turning feedback into action is still the slow part.

[Quick Start](#quick-start) · [CLI Reference](#cli-commands) · [Widget Docs](docs/widget-attributes.md) · [Self-Hosting](docs/self-hosting.md)

</div>

---

## What Is This?

Spikes is a feedback tool for AI-assisted development. It lets reviewers leave targeted feedback directly on web pages — no screenshots, no "that button over there", no lost context.

Click any element. Rate it. Comment. Spikes captures the exact CSS selector, bounding box, and page context. Your AI agent gets structured JSON it can act on immediately.

**No accounts required. No build step. Works on `file://`, localhost, anywhere.**

---

## Quick Start

### 1. Install the CLI

```bash
curl -fsSL https://spikes.sh/install.sh | sh
# Or: cargo install spikes
```

### 2. Initialize your project

```bash
spikes init                     # Defaults to hosted spikes.sh
# spikes init --self-host       # Opt out — scaffold a self-hosted config instead
```

`spikes init` creates `.spikes/config.toml` with a `[remote]` section pointing at `https://spikes.sh` by default. Pass `--self-host` (or answer `s` at the prompt) to skip the hosted defaults.

### 3. Add the widget to your HTML

```bash
spikes inject ./mockups/        # Injects widget script tag (uses configured endpoint)
spikes serve                    # http://localhost:3847
```

### 4. Collect and use feedback

```bash
spikes list                     # See all feedback
spikes list --json              # Feed to your agent
spikes list --rating no         # Find problems
spikes hotspots                 # Elements with most feedback
spikes resolve <id>             # Mark items done
```

---

## CLI Commands

| Command | Description |
|---------|-------------|
| `spikes init` | Create `.spikes/` directory with config (hosted by default; `--self-host` to opt out) |
| `spikes list` | List feedback (`--json`, `--page`, `--reviewer`, `--rating`, `--unresolved`) |
| `spikes show <id>` | Show single spike details |
| `spikes export` | Export to JSON/CSV/JSONL/Cursor/Claude context |
| `spikes hotspots` | Elements with most feedback |
| `spikes reviewers` | List all reviewers |
| `spikes inject <dir>` | Add/remove widget from HTML files (`--endpoint <url>` overrides the configured endpoint) |
| `spikes serve` | Local dev server (`--port`, `--marked`, `--cors-allow-origin`) |
| `spikes mcp serve` | Start MCP server for AI agent integration |
| `spikes pull/push/sync` | Sync with remote endpoint |
| `spikes share <dir>` | Upload to spikes.sh for instant sharing |
| `spikes login/logout/whoami` | Authentication management |
| `spikes upgrade/billing` | Pro tier subscription via Stripe |
| `spikes deploy cloudflare` | Scaffold self-hosted Worker + D1 |

All commands support `--json` for scripting. See [full CLI reference](docs/cli-reference.md).

**Viewing spikes for a hosted project:**

The hosted dashboard at **<https://spikes.sh/dashboard>** lists every project you own and lets you drill into individual spikes (filter by page, rating, resolved). Sign in with the same token you use for the CLI.

You can also hit the JSON API directly with your bearer token (`$SPIKES_TOKEN` from `spikes login`):

```bash
# List all projects you own (with spike_count + last_activity)
curl -H "Authorization: Bearer $SPIKES_TOKEN" \
  https://spikes.sh/me/projects

# List spikes for one of your projects (paginated, filterable)
curl -H "Authorization: Bearer $SPIKES_TOKEN" \
  "https://spikes.sh/me/projects/my-project/spikes?page=1&per_page=50"
```

Both endpoints are user-scoped: you only ever see projects and spikes you own.

---

## AI Agent Integration

Spikes speaks agent natively. Your AI coding assistant can read, write, and manage feedback without ever leaving its workflow.

```bash
npx spikes-mcp            # Zero-install MCP server — just works
# or: spikes mcp serve    # If you have the CLI installed
```

### MCP Server — 9 Tools

`spikes mcp serve` starts a [Model Context Protocol](https://modelcontextprotocol.io/) server that exposes 9 tools:

| Tool | Purpose |
|------|---------|
| `get_spikes` | List feedback with filters (page, rating, unresolved) |
| `get_element_feedback` | Get feedback for a specific CSS selector |
| `get_hotspots` | Find elements with the most feedback |
| `submit_spike` | Create feedback programmatically |
| `resolve_spike` | Mark feedback as addressed |
| `delete_spike` | Remove a spike |
| `create_share` | Upload files, get a shareable URL |
| `list_shares` | See your active shares |
| `get_usage` | Check usage stats, limits, and spend |

Supports **stdio** and **HTTP** transports, **local** and **remote** data modes:

```bash
spikes mcp serve                              # stdio, local JSONL
spikes mcp serve --remote                     # stdio, hosted API
spikes mcp serve --transport http --port 3848 # HTTP for sandboxed agents
```

### API Keys

Agents get their own identity. No email, no magic link, no human step:

```bash
spikes auth create-key --name "my-agent"   # → sk_spikes_...
spikes auth list-keys                       # See all keys
spikes auth revoke-key <key_id>            # Revoke
```

Keys support read/write/full scopes and optional budget caps.

### Agent-Tier Billing

Consumption-based pricing for agent-scale usage. Pay per spike, not per seat:

```bash
spikes usage    # See current spend and limits
```

Budget enforcement returns `429 BUDGET_EXCEEDED` when caps are hit — agents can check before they burn.

### Context Exports

Export structured markdown optimized for agent consumption:

```bash
spikes export --format cursor-context > cursor-feedback.md
spikes export --format claude-context > claude-feedback.md
```

### Discovery

- **[llms.txt](https://spikes.sh/llms.txt)** — All 9 MCP tools, parameters, agent quickstart
- **[agents.md](https://spikes.sh/agents.md)** — Machine-readable landing page for agents
- **[Smithery](https://smithery.ai)** — Listed in the MCP server registry
- `spikes mcp install` — Generates config for Claude Desktop / Cursor

Full details: [llms.txt](https://spikes.sh/llms.txt) · [agents.md](https://spikes.sh/agents.md)

---

## GitHub Action

Gate CI on feedback quality. The `spikes-action` fails builds when unresolved negative feedback exceeds your threshold.

```yaml
name: CI
on: [push, pull_request]

jobs:
  feedback-gate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: bierlingm/spikes/action@v0.3.1
        with:
          threshold: 0           # Fail if any blocking spikes
          ignore-paths: ""     # Optional: pages to ignore
          require-resolution: false
```

See [action/README.md](action/README.md) for full documentation.

---

## Widget Attributes

**Simple setup** — just add your project key and you're ready to collect feedback:

```html
<script src="https://spikes.sh/spikes.js" data-project="my-app"></script>
```

When you set `data-project`, feedback syncs automatically to `https://spikes.sh/spikes`. No need to configure an endpoint unless you're self-hosting.

**Full configuration** — all available attributes:

```html
<script src="https://spikes.sh/spikes.js"
  data-project="my-app"
  data-position="bottom-right"
  data-color="#e74c3c"
  data-theme="dark"
  data-reviewer="Pat"
  data-collect-email="true"
  data-admin="true">
</script>
```

**Self-hosting** — point to your own backend:

```html
<script src="/spikes.js"
  data-project="my-app"
  data-endpoint="https://my-worker.workers.dev/spikes">
</script>
```

| Attribute | Description | Default |
|-----------|-------------|---------|
| `data-project` | Group feedback by project key | `location.hostname` |
| `data-position` | Button corner: `bottom-right`, `bottom-left`, `top-right`, `top-left` | `bottom-right` |
| `data-color` | Accent color (any CSS color) | `#e74c3c` |
| `data-theme` | Modal theme: `dark` or `light` | `dark` |
| `data-reviewer` | Pre-set reviewer name | (prompts user) |
| `data-endpoint` | Backend URL for multi-reviewer sync. Optional — defaults to `https://spikes.sh/spikes` when `data-project` is set. | `https://spikes.sh/spikes` (with `data-project`), `/spikes` (local dev) |
| `data-collect-email` | Show email field in prompt | `false` |
| `data-admin` | Enable review mode features | `false` |
| `data-offset-x/y` | Button offset from edge | — |

See [Widget Attributes Reference](docs/widget-attributes.md) for complete documentation.

---

## Architecture

Spikes has three components that work together or standalone:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   CLI       │────▶│   Widget    │◄────│   Worker    │
│  (Rust)     │     │  (Vanilla   │     │ (Cloudflare │
│             │     │    JS)      │     │  + D1)      │
└─────────────┘     └─────────────┘     └─────────────┘
     │                                            │
     │          spikes.sh (hosted)                │
     └────────────────────────────────────────────┘
```

**CLI** — Rust binary for local development, spike management, and deployment. Stores spikes in `~/.local/share/spikes/`.

**Widget** — 14KB gzipped vanilla JS. Captures element selectors, bounding boxes, ratings, and comments. Works offline via localStorage.

**Worker** — Optional Cloudflare Worker + D1 backend for multi-reviewer sync, sharing, and hosted deployments. Lives in `spikes-hosted/`.

---

## Development

### CLI

```bash
cd cli
cargo build --release
cargo test              # 160+ tests
cargo run -- --help
```

### Widget

```bash
cd widget
# Edit src/spikes.js
# Test by running: spikes serve from the project root
```

### Worker

The Worker backend lives in the private `spikes-hosted` repo. To test it:

```bash
cd ../spikes-hosted/worker
npm test              # Worker test suite (vitest)
npx wrangler dev
```

---

## Self-Hosting

Want your own backend? One command:

```bash
spikes deploy cloudflare    # Creates spikes-worker/ directory
cd spikes-worker && npx wrangler deploy
```

See [Self-Hosting Guide](docs/self-hosting.md) for full setup with D1 database, authentication, and Stripe billing integration.

### Share vs. Deploy Cloudflare: Which to Choose?

Use **`spikes share`** when you want instant sharing without any setup — files are uploaded to spikes.sh and served from our infrastructure. Great for quick reviews, client feedback, or when you don't want to manage a backend.

Use **`spikes deploy cloudflare`** when you need **data isolation** (feedback stays in your Cloudflare account), a **custom domain**, or want full control over the infrastructure. Self-hosting requires a Cloudflare account and a one-time deployment setup.

---

## What's Changed (Recent Overhaul)

- **Security**: PBKDF2 password hashing, path traversal fixes, XSS protection
- **Auth**: Magic link authentication (no passwords to forget)
- **Billing**: Stripe integration with Pro tier support
- **Testing**: CLI test suite (Rust) + Worker test suite (in ../spikes-hosted/worker)
- **Architecture**: Modular worker with clean separation of concerns
- **CI/CD**: Automated testing and deployment pipelines

---

## Detailed Documentation

- [CLI Reference](docs/cli-reference.md) — Complete command documentation
- [Widget Attributes](docs/widget-attributes.md) — All configuration options
- [Self-Hosting Guide](docs/self-hosting.md) — Deploy your own backend
- [API Reference](docs/API.md) — REST API documentation
- [Rollback Guide](docs/rollback.md) — Emergency procedures

---

## Why Spikes

| | |
|---|---|
| **Zero friction** | One script tag, no signup required, no build step |
| **Works anywhere** | `file://`, `localhost`, any domain |
| **Precise** | Element-level feedback with exact CSS selectors |
| **Agent-native** | JSON everywhere, pipes, queryable CLI |
| **Your infrastructure** | Self-host or use hosted — your choice |
| **Tiny** | Widget is 14KB gzipped |
| **Private** | No tracking, your data stays yours |

---

## Pricing

<div align="center">

### Free forever. Pro if you want more.

No accounts required to start. Login when you need Pro features.

</div>

| | **Free** | **Pro** |
|---|---|---|
| **Price** | $0 forever | Pay what you can |
| **Shares** | 5 | Unlimited |
| **Spikes per share** | 1,000 | Unlimited |
| **Widget + CLI** | Full | Full |
| **Self-hosting** | Yes | Yes |
| **Password protection** | — | Yes |
| **Webhooks** | — | Yes |
| **Badge removal** | — | Yes |

<div align="center">

MIT licensed. Payment is appreciation, not access.

`spikes upgrade` when you're ready. No pressure.

</div>

---

<div align="center">

**[Website](https://spikes.sh)** · **[Docs](https://spikes.sh/docs.html)** · **[GitHub](https://github.com/bierlingm/spikes)** · **[Issues](https://github.com/bierlingm/spikes/issues)**

MIT License · Built for builders who work with AI.

</div>
