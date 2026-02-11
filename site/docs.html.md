# Spikes Documentation

Feedback your AI agent can act on. CLI commands, widget configuration, and data formats.

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
| `data-project` | hostname | Project key for grouping feedback |
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
