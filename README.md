<p align="center">
  <strong>🗡️ Spikes</strong><br>
  <em>Drop-in feedback for static mockups</em>
</p>

<p align="center">
  <a href="https://spikes.sh">spikes.sh</a> •
  <a href="#install">Install</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#cli-reference">CLI Reference</a>
</p>

---

Add one script tag to your HTML mockups. Collect structured feedback from reviewers. Query it with a CLI that agents love.

## Install

**One-liner:**
```bash
curl -fsSL https://spikes.sh/install.sh | sh
```

**Or with Cargo:**
```bash
cargo install spikes
```

**Or download** from [Releases](https://github.com/moritzbierling/spikes/releases).

## Quick Start

### 1. Add the widget

```html
<script src="https://spikes.sh/widget.js"></script>
```

Or for local development:
```bash
spikes inject ./mockups/
spikes serve
# → http://localhost:3847
```

### 2. Collect feedback

Reviewers see a floating button. They can:
- **Rate the page** — click button, pick love/like/meh/no, add comments
- **Spike an element** — click button to enter spike mode, click any element to capture it with a precise CSS selector

First-time reviewers are prompted for their name.

### 3. Query feedback

```bash
# List all feedback
spikes list

# Filter by rating
spikes list --rating no

# Find problem areas
spikes hotspots

# Export for agents
spikes list --json | jq '.[] | select(.rating == "no")'

# Interactive TUI
spikes dashboard
```

## Widget Configuration

```html
<script 
  src="https://spikes.sh/widget.js"
  data-project="my-project"
  data-position="bottom-left"
  data-color="#3498db"
  data-endpoint="https://my-worker.workers.dev/spikes"
></script>
```

| Attribute | Default | Description |
|-----------|---------|-------------|
| `data-project` | hostname | Groups feedback across pages |
| `data-position` | `bottom-right` | Button position |
| `data-color` | `#e74c3c` | Button color |
| `data-endpoint` | — | POST endpoint for multi-reviewer sync |
| `data-reviewer` | — | Pre-set reviewer name (skips prompt) |

## Multi-Reviewer Sync

By default, feedback is stored in each reviewer's browser (localStorage). For team reviews, deploy a backend to your own Cloudflare account:

```bash
spikes deploy cloudflare
cd spikes-worker && npx wrangler deploy
```

Then add the endpoint URL to your widget:
```html
<script src="https://spikes.sh/widget.js" data-endpoint="https://...workers.dev/spikes"></script>
```

Sync local feedback:
```bash
spikes pull   # Fetch from remote
spikes push   # Upload local spikes
```

## CLI Reference

```
spikes init                    Initialize .spikes/ directory
spikes list [OPTIONS]          List feedback (--json, --page, --reviewer, --rating)
spikes show <ID>               Show single spike detail
spikes export [--format X]     Export as json, csv, or jsonl
spikes hotspots                Elements with most feedback
spikes reviewers               List all reviewers
spikes inject <DIR>            Add widget to HTML files
spikes inject --remove <DIR>   Remove widget from HTML files
spikes serve [--port N]        Local dev server (default: 3847)
spikes deploy cloudflare       Scaffold Cloudflare Worker + D1
spikes pull                    Fetch from remote endpoint
spikes push                    Upload to remote endpoint
spikes dashboard               Interactive TUI
```

All commands support `--json` for scripting and agent consumption.

## Data Format

```typescript
interface Spike {
  id: string;                    // Unique ID
  type: "page" | "element";
  projectKey: string;
  page: string;                  // Page title
  url: string;
  reviewer: { id: string; name: string };
  selector?: string;             // CSS selector (element spikes)
  elementText?: string;          // Captured text (truncated)
  boundingBox?: { x, y, width, height };
  rating: "love" | "like" | "meh" | "no" | null;
  comments: string;
  timestamp: string;             // ISO 8601
  viewport: { width, height };
}
```

## Why Spikes?

- **Zero friction** — One script tag, no accounts, no build step
- **Works anywhere** — file://, localhost, any domain
- **Precise feedback** — Element-level spikes with CSS selectors
- **Agent-friendly** — JSON output, pipes, queryable CLI
- **Your infrastructure** — Deploy to your own Cloudflare (free tier)
- **Tiny** — Widget is 8KB gzipped

## License

MIT — Free for everything. See [LICENSE](LICENSE).

---

<p align="center">
  <em>Built for designers who ship HTML mockups and the agents who help them.</em>
</p>
