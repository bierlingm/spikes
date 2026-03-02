# Spikes Documentation

Feedback your AI agent can act on. Workflows, CLI commands, widget configuration, and data formats.

## What is Spikes

Spikes is a feedback tool built for the AI-assisted building workflow. It solves a specific problem: **AI can build a prototype in an hour, but turning feedback into actionable code changes is still slow and manual.**

You add one script tag to any HTML file. A floating button appears. Click any element to "spike" it — rate it, leave a comment. Spikes captures the exact CSS selector, bounding box, text content, and viewport dimensions. Not a screenshot. Not a vague description. Structured data your agent can act on.

**How it works:** Reviewer clicks an element → Spikes records `.pricing-card`, bounding box `{x: 24, y: 120, width: 340, height: 200}`, and their comment → You run `spikes list --json` → Your agent knows exactly what to change and where.

There are two main use cases:

- **Reviewing your own agent's work** — You build with Claude Code or Cursor, then spike elements that need changes instead of describing them in chat
- **Collecting feedback from others** — Share a link with your team, client, or friends. They click and comment. You get structured JSON instead of scattered WhatsApp messages

The widget is 8KB gzipped, has zero dependencies, works on `file://`, `localhost`, and any domain. No accounts needed. Feedback is stored in localStorage by default, or synced to your own backend for multi-reviewer workflows.

## Installation

### Widget

Add a single script tag to your HTML file:

```html
<script src="https://spikes.sh/widget.js"></script>
```

### CLI — Install Script

```bash
curl -fsSL https://spikes.sh/install.sh | sh
```

### CLI — Cargo

```bash
cargo install spikes
```

## Quick Start

```bash
# Inject widget into all HTML files in a directory
spikes inject ./mockups/

# Start local server
spikes serve
# → http://localhost:3847

# Share URL with reviewer, collect feedback, then:
spikes list                    # See all feedback
spikes list --rating no        # Find problems
spikes hotspots                # Most-spiked elements
spikes list --json | jq '...'  # Feed to agents
```

**Tip:** First-time reviewers are prompted for their name. All spikes are tagged with reviewer identity.

## Workflow: Review Your Agent's Work

You've asked Claude Code to build a pricing page. It looks mostly right, but a few things are off. Instead of describing the problems in chat, spike them:

```bash
# 1. Add the widget to your mockup
spikes inject ./mockups/

# 2. Open it in your browser
spikes serve
# → http://localhost:3847
```

Now open the page. Click the floating `/` button to enter spike mode. Your cursor becomes a crosshair. Click any element that needs work — the pricing card that's too cramped, the button with the wrong color, the heading that needs rewording. Rate each one and leave a comment.

When you're done reviewing, pull the feedback into your agent:

```bash
# 3. See what needs fixing
spikes list --rating no
spikes list --rating meh

# 4. Feed structured data to your agent
spikes list --json
```

Paste the JSON output into Claude Code or Cursor. The agent gets exact CSS selectors, bounding boxes, and your comments — it knows precisely what to change without guessing.

**Tip:** Use `spikes hotspots` to find which elements got the most feedback. These are your highest-priority fixes.

## Workflow: Collect Feedback from Others

You've built a prototype for a client, or you want friends to review your side project. You need their feedback to be structured, not scattered across five messaging apps.

### Option A: Local server (same network)

```bash
# Inject widget and serve locally
spikes inject ./mockups/
spikes serve
# Share http://your-ip:3847 with people on your network
```

### Option B: Deploy your own backend (anyone, anywhere)

```bash
# Scaffold and deploy a Cloudflare Worker
spikes deploy cloudflare
cd spikes-worker && npx wrangler deploy

# Add the endpoint to your widget
spikes inject ./mockups/ --endpoint https://your-worker.workers.dev/spikes
```

Share the URL. Reviewers see your mockup with a floating `/` button. First-time reviewers are prompted for their name. They click elements, leave ratings and comments. Everything syncs to your backend.

```bash
# Pull feedback from all reviewers
spikes pull

# See who reviewed and what they said
spikes reviewers
spikes list --reviewer "Sarah"

# Export everything for your agent
spikes list --json
```

**Tip:** Non-technical reviewers don't need to install anything. They just open a link and click. Their feedback comes out as structured JSON with exact selectors — technical output from non-technical people.

## Workflow: Agent Integration

Every Spikes command supports `--json` output, making it straightforward to pipe feedback into any AI agent or automation tool.

### Feed feedback directly to Claude Code

```bash
# Paste this prompt into your agent:
Here is the feedback on my mockup. Fix each issue:
$(spikes list --json)
```

Your agent receives structured data with selectors like `.pricing-card`, ratings, comments, and exact bounding boxes. It can map each spike to a file and make precise changes.

### Filter before feeding

```bash
# Only fix negative feedback
spikes list --rating no --json

# Only fix feedback on a specific page
spikes list --page pricing.html --json

# Find the most-spiked elements first
spikes hotspots --json

# Export everything as CSV for a spreadsheet
spikes export --format csv --output feedback.csv
```

### Automate with scripts

```bash
#!/bin/bash
# Pull latest feedback and process with jq
spikes pull
spikes list --json | jq '.[] | select(.rating == "no")' | \
  jq -s '{issues: length, selectors: [.[].selector]}'
```

**What your agent gets per spike:** CSS selector (`.pricing-card`), bounding box (`{x, y, width, height}`), element text, rating, reviewer comment, page URL, viewport dimensions, and timestamp. Everything needed to locate and fix the issue without asking follow-up questions.

## CLI Commands

All commands support `--json` for machine-readable output.

### init

Create a `.spikes/` directory in the current project.

```bash
spikes init
```

### list

List all feedback with optional filtering.

```bash
spikes list [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON array |
| `--page <PAGE>` | Filter by page URL or filename |
| `--reviewer <NAME>` | Filter by reviewer name |
| `--rating <RATING>` | Filter by rating (love, like, meh, no) |
| `--type <TYPE>` | Filter by type (page, element) |

```bash
# List all negative feedback
spikes list --rating no

# Get JSON for a specific page
spikes list --page index.html --json
```

### show

Display details for a single spike.

```bash
spikes show <ID> [--json]
```

### export

Export all feedback in various formats.

```bash
spikes export [--format <FORMAT>] [--output <FILE>]
```

Formats: `json`, `csv`, `jsonl` (default: json)

```bash
# Export as CSV for spreadsheets
spikes export --format csv --output feedback.csv

# Export as JSONL for streaming processing
spikes export --format jsonl
```

### hotspots

Find elements with the most feedback.

```bash
spikes hotspots [--json] [--limit <N>]
```

### reviewers

List all reviewers who have submitted feedback.

```bash
spikes reviewers [--json]
```

### inject

Add the Spikes widget to HTML files.

```bash
spikes inject <DIR> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--project <NAME>` | Set project key for grouping feedback |
| `--endpoint <URL>` | Set sync endpoint for multi-reviewer mode |
| `--recursive` | Process subdirectories |
| `--dry-run` | Show what would be changed |

```bash
# Add widget to all HTML files
spikes inject ./mockups/ --recursive

# Preview changes first
spikes inject ./mockups/ --dry-run
```

### serve

Start a local development server.

```bash
spikes serve [--port <PORT>] [--dir <DIR>]
```

Default port: 3847

### deploy

Scaffold deployment configuration.

```bash
spikes deploy cloudflare
```

Generates Cloudflare Worker + D1 scaffolding.

### pull / push

Sync feedback with a remote endpoint.

```bash
spikes pull [--endpoint <URL>]
spikes push [--endpoint <URL>]
```

### dashboard

Launch interactive TUI dashboard.

```bash
spikes dashboard
```

## Widget Configuration

Configure with data attributes:

```html
<script
  src="https://spikes.sh/widget.js"
  data-project="my-project"
  data-position="bottom-left"
  data-color="#3498db"
  data-endpoint="https://my-worker.workers.dev/spikes"
  data-reviewer="alice"
></script>
```

| Attribute | Default | Description |
|-----------|---------|-------------|
| `data-project` | hostname | Project key for grouping feedback across pages |
| `data-position` | `bottom-right` | Button position: bottom-right, bottom-left, top-right, top-left |
| `data-color` | `#e74c3c` | Button background color (any CSS color) |
| `data-endpoint` | — | POST endpoint for multi-reviewer sync |
| `data-reviewer` | — | Pre-set reviewer name (skip the name prompt) |

## Data Format

Every spike follows this structure:

```typescript
interface Spike {
  id: string;                    // nanoid
  type: "page" | "element";
  projectKey: string;
  page: string;
  url: string;
  reviewer: { id: string; name: string };
  selector?: string;             // Element spikes only
  elementText?: string;          // Element spikes only
  boundingBox?: { x: number; y: number; width: number; height: number };
  rating: "love" | "like" | "meh" | "no" | null;
  comments: string;
  timestamp: string;             // ISO 8601
  viewport: { width: number; height: number };
}
```

**Storage:** Feedback is stored in `.spikes/feedback.jsonl` — one spike per line in JSON Lines format.

## Multi-Reviewer Sync

By default, feedback lives in each reviewer's browser (localStorage). For team reviews, deploy your own backend:

### Deploy to Cloudflare

```bash
# Generate Cloudflare Worker scaffolding
spikes deploy cloudflare

# Deploy to your Cloudflare account
cd spikes-worker
npx wrangler deploy
```

### Configure Widget

```html
<script
  src="https://spikes.sh/widget.js"
  data-endpoint="https://your-worker.workers.dev/spikes"
></script>
```

### Sync from CLI

```bash
# Fetch all remote feedback
spikes pull

# Upload local feedback
spikes push
```

**Tip:** Set the `SPIKES_ENDPOINT` environment variable to avoid passing `--endpoint` every time.
