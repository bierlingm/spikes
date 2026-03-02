<div align="center">

# <span>/</span> Spikes

**The feedback loop for AI-assisted building**

[![MIT License](https://img.shields.io/badge/license-MIT-e74c3c?style=flat-square)](LICENSE)
[![Widget Size](https://img.shields.io/badge/widget-8KB_gzipped-22c55e?style=flat-square)](#)
[![Free Forever](https://img.shields.io/badge/price-$0_forever-3b82f6?style=flat-square)](#pricing)

AI can build a prototype in an hour.<br>
Turning feedback into action is still the slow part.

[Get Started](#get-started) · [How It Works](#how-it-works) · [Docs](https://spikes.sh/docs.html) · [Website](https://spikes.sh)

</div>

---

## Get Started

Paste this into Claude Code, Cursor, or your agent:

```
Install spikes (curl -fsSL https://spikes.sh/install.sh | sh),
then run spikes inject on my project and spikes serve to preview it
```

<details>
<summary><b>Or install manually</b></summary>

```bash
# Install the CLI
curl -fsSL https://spikes.sh/install.sh | sh

# Or with Cargo
cargo install spikes
```

</details>

```bash
spikes inject ./mockups/        # Add widget to your HTML files
spikes serve                    # Start local server → http://localhost:3847

# Share the URL, collect feedback, then:
spikes list                     # See all feedback
spikes list --rating no         # Find problems
spikes hotspots                 # Most-spiked elements
spikes list --json              # Feed to agents
```

---

## The Problem

You build something with Claude Code, Cursor, or v0. Now you need feedback.

> *"Make that card bigger"* — which card? How much bigger?

> *"The button thing isn't right"* — sent via WhatsApp voice note

> *"See attached screenshot"* — with a red circle drawn in MS Paint

You become the translator between human feedback and code changes. Feedback is scattered across five apps, none of it structured, none of it actionable.

**Spikes closes that loop.**

---

## How It Works

<table>
<tr>
<td width="50%">

### What reviewers see

```
┌─────────────────────────┐
│  My Mockup              │
│                         │
│  ┌───────────────┐      │
│  │ Pricing Card  │ ← /  │
│  │ $19/mo        │      │
│  └───────────────┘      │
│                         │
│  ┌─ spike ────────────┐ │
│  │ Sarah (Product)    │ │
│  │ "Needs more        │ │
│  │  breathing room"   │ │
│  │ Rating: meh        │ │
│  └────────────────────┘ │
└─────────────────────────┘
```

</td>
<td width="50%">

### What your agent sees

```json
$ spikes list --json

[{
  "selector": ".pricing-card",
  "comments": "needs more breathing room",
  "reviewer": "Sarah (Product)",
  "rating": "meh",
  "boundingBox": { "x": 24, "y": 120,
                   "width": 340, "height": 200 },
  "page": "/pricing.html"
}]
```

*Exact selector. Exact position. Actionable.*

</td>
</tr>
</table>

**Three steps:**

**1 →** Add `<script src="https://spikes.sh/widget.js"></script>` to any HTML file

**2 →** Click any element. Rate it. Leave a comment. Spikes captures the CSS selector, bounding box, text, and viewport.

**3 →** `spikes list --json` — structured feedback, ready for your agent to act on.

---

## Two Ways to Use It

<table>
<tr>
<td width="50%" valign="top">

### /1 Review your agent's work

```
Agent builds → You spike it → Agent fixes
```

Click elements, leave comments. Your agent gets exact CSS selectors and context — no more describing visual issues in chat.

</td>
<td width="50%" valign="top">

### /2 Collect feedback from others

```
Share link → They spike it → Agent implements
```

One link. Non-technical reviewers click and comment. You get structured JSON. Feedback lives with the prototype.

</td>
</tr>
</table>

---

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

Works on `file://`, `localhost`, and any domain. No accounts, no build step.

---

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

---

<details>
<summary><b>Multi-Reviewer Sync</b></summary>

By default, feedback lives in each reviewer's browser (localStorage). For team reviews, deploy to your own Cloudflare account:

```bash
spikes deploy cloudflare
cd spikes-worker && npx wrangler deploy
```

Then point the widget at your endpoint:

```html
<script src="https://spikes.sh/widget.js" data-endpoint="https://...workers.dev/spikes"></script>
```

Sync local feedback with `spikes pull` and `spikes push`.

</details>

<details>
<summary><b>Data Format</b></summary>

Each spike captures everything an agent needs:

```typescript
interface Spike {
  id: string;                    // nanoid (21 chars)
  type: "page" | "element";
  projectKey: string;
  page: string;
  url: string;
  reviewer: { id: string; name: string };
  selector?: string;             // Element spikes only
  elementText?: string;
  boundingBox?: { x, y, width, height };
  rating: "love" | "like" | "meh" | "no" | null;
  comments: string;
  timestamp: string;             // ISO 8601
  viewport: { width, height };
}
```

</details>

---

## Why Spikes

| | |
|---|---|
| **Zero friction** | One script tag, no accounts, no build step |
| **Works anywhere** | `file://`, `localhost`, any domain |
| **Precise** | Element-level feedback with exact CSS selectors and bounding boxes |
| **Agent-friendly** | JSON everywhere, pipes, queryable CLI |
| **Your infrastructure** | Self-host on your own Cloudflare, or just use localStorage |
| **Tiny** | Widget is 8KB gzipped |
| **Private** | No tracking, no analytics, your data stays yours |

---

## Pricing

<div align="center">

### $0

**Everything. Forever. MIT licensed.**

Full widget. Full CLI. Unlimited mockups, unlimited reviewers.<br>No tracking, no accounts, no feature gates.

*No catch.*

---

**Spike us back** · $19+ pay what you feel · Supporter badge, priority issues

**Agency** · $149 once · Whole team, logo on spikes.sh, priority support

Payment is appreciation, not access. [Read the philosophy →](LICENSE-MODEL.md)

</div>

---

<div align="center">

**[Website](https://spikes.sh)** · **[Docs](https://spikes.sh/docs.html)** · **[GitHub](https://github.com/bierlingm/spikes)** · **[Issues](https://github.com/bierlingm/spikes/issues)**

MIT License · Built for builders who work with AI.

</div>
