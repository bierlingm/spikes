# Spikes

Sharp feedback for static mockups.

One script tag. Pinpoint any element. Query with a CLI that agents love.

## What It Does

Add `<script src="https://spikes.sh/widget.js"></script>` to any HTML file. Reviewers click the floating button to rate the page or spike specific elements with precise CSS selectors. All feedback is structured, queryable, and ready for automation.

## Install

```bash
# CLI (recommended)
curl -fsSL https://spikes.sh/install.sh | sh

# Or with Cargo
cargo install spikes

# Widget only (add to HTML)
<script src="https://spikes.sh/widget.js"></script>
```

## Quick Start

```bash
# Inject widget into all mockups
spikes inject ./mockups/

# Start local server
spikes serve
# → http://localhost:3847

# Share URL with reviewer, collect feedback, then:
spikes list                           # See all feedback
spikes list --rating no               # Find problems
spikes hotspots                       # Most-spiked elements
spikes list --json | jq '...'         # Feed to agents
```

## How It Works

1. Reviewer clicks floating button → enters spike mode
2. Click any element → captures CSS selector, bounding box, text
3. Rate (love/like/meh/no) + add comments
4. Feedback stored locally or synced to your backend

First-time reviewers are prompted for their name. All spikes tagged with reviewer identity.

## CLI Reference

```
spikes init                    Create .spikes/ directory
spikes list [OPTIONS]          List feedback (--json, --page, --reviewer, --rating)
spikes show <ID>               Show single spike
spikes export [--format X]     Export as json/csv/jsonl
spikes hotspots                Elements with most feedback
spikes reviewers               List all reviewers
spikes inject <DIR>            Add widget to HTML files
spikes serve [--port N]        Local dev server
spikes deploy cloudflare       Scaffold Cloudflare Worker + D1
spikes pull / push             Sync with remote endpoint
spikes dashboard               Interactive TUI
```

All commands support `--json` for scripting and agents.

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
| `data-reviewer` | — | Pre-set reviewer name |

## Multi-Reviewer Sync

By default, feedback lives in each reviewer's browser (localStorage). For team reviews:

```bash
# Deploy to your Cloudflare account
spikes deploy cloudflare
cd spikes-worker && npx wrangler deploy

# Add endpoint to widget
<script src="https://spikes.sh/widget.js" data-endpoint="https://...workers.dev/spikes"></script>

# Sync
spikes pull   # Fetch remote
spikes push   # Upload local
```

## Data Format

```typescript
interface Spike {
  id: string;
  type: "page" | "element";
  projectKey: string;
  page: string;
  url: string;
  reviewer: { id: string; name: string };
  selector?: string;           // Element spikes only
  elementText?: string;
  boundingBox?: { x, y, width, height };
  rating: "love" | "like" | "meh" | "no" | null;
  comments: string;
  timestamp: string;           // ISO 8601
  viewport: { width, height };
}
```

## Pricing

**Free** — Everything. Forever. MIT licensed.

**Spike Us Back** — $19+ (pay what you feel, min $9)  
You've collected spikes. Now spike us. Badge, priority issues, supporters page.

**Agency** — $149 once  
Whole team covered. Logo on spikes.sh. Priority support.

No feature gating. Payment is appreciation, not access.

See [LICENSE-MODEL.md](LICENSE-MODEL.md) for details.

## Why Spikes?

- **Zero friction** — One script tag, no accounts, no build step
- **Works anywhere** — file://, localhost, any domain
- **Precise** — Element-level feedback with CSS selectors
- **Agent-friendly** — JSON everywhere, pipes, queryable
- **Your infrastructure** — Deploy to your own Cloudflare
- **Tiny** — Widget is 8KB gzipped
- **Private** — No tracking, no analytics, your data

## Links

- **Website:** [spikes.sh](https://spikes.sh)
- **GitHub:** [github.com/bierlingm/spikes](https://github.com/bierlingm/spikes)
- **Issues:** [Report a bug](https://github.com/bierlingm/spikes/issues)

## License

MIT — Free for everything.

---

Built for designers who ship HTML mockups and the agents who help them.
