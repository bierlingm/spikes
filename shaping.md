---
shaping: true
---

# Spikes — Shaping Document

**Status:** V1-V8 shipped, V9-V10 in shaping  
**Domain:** spikes.sh  
**Tagline:** Feedback your AI agent can act on

---

## Source

> I noticed that this functionality of having a way to give feedback directly on the thing shown itself rather than in whatsapp or Slack messages was really useful, because it also means you have it all programmatically accessible and can even hook it up to other systems.

> Maybe it even has different modes, like "Shift + click" anywhere gives feedback on that specific element.

> I'm thinking this could work very well as a dev tool type thing with a ".sh" domain and a one time license or pay what you want or maybe a $20 price tag.

> It should have a CLI and robot friendly commands like beads_rust, and an insanely smooth and minimal user experience using FrankenTUI.

> The pitch should be something like "a lightweight drop-in feedback gathering tool". Maybe we should be opinionated about working really nicely with Cloudflare.

> The game here is to make it super nicely usable with agents.

---

## Frame

### The Shift

Building prototypes is easy now. Claude Code, Cursor, v0 — an agent can build a working UI in an hour. **The bottleneck has moved to the feedback loop.** Turning "I don't like that card thing" into code changes is still manual, slow, and lossy.

### Problem

- **For solo builders reviewing agent work:** Describing visual issues in chat is imprecise. "Make that card bigger" — which card? How much bigger? The agent needs selectors, not vibes.
- **For collecting feedback from others:** Feedback arrives scattered across WhatsApp, Slack, email — all screenshots and vague descriptions you have to translate into code.
- **For agent workflows:** Feedback isn't programmatically accessible. You become the translator between humans and agents.
- Existing tools require accounts, backends, subscriptions — friction for simple mockup reviews.

### Outcome

- **Use case 1 (self-review):** Click the element. Leave a comment. Your agent gets the exact CSS selector, bounding box, and context to act on it.
- **Use case 2 (collect from others):** Share one link. Reviewers click elements and comment. You get structured JSON to paste straight into your agent.
- All feedback is locally stored (offline-first) with optional hosted sync
- CLI provides robot-friendly access to all feedback data (JSON output, query commands)
- Agents can read, filter, and act on feedback programmatically
- Zero accounts required for local usage; paid hosting for instant shareable links

---

## Requirements (R)

| ID | Requirement | Status |
|----|-------------|--------|
| **R0** | **Reviewers can leave structured feedback directly on HTML mockups** | Core goal |
| R1 | Widget works with a single `<script>` tag — no build step, no dependencies | Must-have |
| R2 | Widget works on `file://`, `localhost`, and any hosted domain | Must-have |
| R3 | Page-level feedback: rate and comment on the whole page | Must-have |
| R4 | Element-level feedback: spike mode captures specific element with selector | Must-have |
| R5 | Feedback includes rating (love/like/meh/no) + free-text comments | Must-have |
| R6 | Feedback stored locally in browser (localStorage) by default | Must-have |
| R7 | All feedback exportable as JSON/CSV | Must-have |
| R8 | CLI can list, filter, show, and export feedback with `--json` flag | Must-have |
| R9 | Widget size < 10KB gzipped | Must-have |
| R10 | Element selector captured is minimal and unique (not brittle xpath) | Must-have |
| R11 | Visual feedback when element is captured (highlight/pulse) | Must-have |
| R12 | Dashboard shows all feedback across all pages with filtering | Must-have |
| R13 | Widget is configurable (position, color, project key) | Nice-to-have |
| R14 | Optional: POST feedback to an endpoint (Cloudflare Worker) for multi-reviewer sync | Nice-to-have |
| R15 | Optional: TUI dashboard via FrankenTUI for terminal-native review | Nice-to-have |
| R16 | CLI commands work well with pipes and agent workflows | Must-have |
| R17 | One-time purchase model (not subscription) | Must-have |
| R18 | Free tier has full functionality; paid tier adds convenience, not power | Must-have |
| R19 | Reviewer must identify themselves before saving first spike | Must-have |
| R20 | One-command deploy to user's own Cloudflare/Vercel account | Must-have |
| R21 | Works without any cloud setup (local-only is fully functional) | Must-have |

---

## Shape A: Local-First with BYO Backend

### Summary

A vanilla JS widget (`spikes.js`) that injects a floating button onto any HTML page. Click the button to enter "spike mode" — then click any element to spike it, or click the button again for page-level feedback. First-time users prompted for their name (stored locally). All data stored in localStorage under a project key. A Rust CLI (`spikes`) reads the same data format and provides robot-friendly access. One-command deploy to user's own Cloudflare or Vercel account for multi-reviewer sync.

### Parts

| Part | Mechanism | Flag |
|------|-----------|:----:|
| **A1** | **Widget: Core UI** | |
| A1.1 | IIFE that injects floating button (position configurable via data attributes) | |
| A1.2 | Page-mode modal: rating buttons + textarea + save/cancel | |
| A1.3 | Element-mode popover: appears near clicked element, same rating/comment UI | |
| A1.4 | Visual feedback: element highlight on hover (in spike mode), pulse on capture | |
| **A2** | **Widget: Spike Mode (Toggle Interaction)** | |
| A2.1 | Click floating button → enters "spike mode" | |
| A2.2 | Spike mode: cursor becomes crosshair, hovering elements get highlight outline | |
| A2.3 | Spike mode: click any element → element-mode popover anchored to that element | |
| A2.4 | Spike mode: click the button again (or press Escape) → page-mode modal | |
| A2.5 | Spike mode indicator: button pulses/changes color, small tooltip "Click element or button" | |
| A2.6 | Exit spike mode: completing feedback, pressing Escape, or clicking outside | |
| **A2.7** | **Element Capture** | |
| A2.7.1 | Compute minimal unique CSS selector (prefer ID → unique class → nth-child path) | |
| A2.7.2 | Capture includes: selector, xpath (fallback), textContent (truncated 100 chars), boundingBox | |
| A2.7.3 | Visual feedback: element pulses briefly when captured | |
| **A3** | **Widget: Storage** | |
| A3.1 | localStorage key: `spikes:{projectKey}` (projectKey from config or page hostname) | |
| A3.2 | Data format: JSON array of Spike objects | |
| A3.3 | Each Spike: `{id, type, page, selector?, elementText?, rating, comments, timestamp, viewport}` | |
| A3.4 | Optional: POST to configured endpoint on save (fire-and-forget, localStorage is source of truth) | |
| **A4** | **Widget: Reviewer Identity** | |
| A4.1 | First spike attempt → prompt "What should we call you?" before save | |
| A4.2 | Reviewer name + generated ID stored in localStorage, persists across sessions | |
| A4.3 | Small indicator "Reviewing as: [name]" in widget UI (click to change) | |
| A4.4 | Optional: pre-set via `data-reviewer` attribute (skips prompt) | |
| A4.5 | Reviewer object attached to every spike: `{name, id}` | |
| **A5** | **CLI: Core Commands** | |
| A5.1 | `spikes init` — create `.spikes/` config directory | |
| A5.2 | `spikes list [--json] [--page X] [--element Y] [--rating Z] [--reviewer R]` — list feedback | |
| A5.3 | `spikes show <id> [--json]` — show single spike detail | |
| A5.4 | `spikes export [--format json|csv|jsonl]` — dump all feedback | |
| A5.5 | `spikes hotspots` — show elements with most feedback | |
| A5.6 | `spikes inject <dir>` — add widget script tag to all HTML files | |
| A5.7 | `spikes serve [--port]` — static file server + POST collector | |
| **A6** | **CLI: Data Access & Sync** | |
| A6.1 | Read from `.spikes/feedback.jsonl` (local file, same format as localStorage export) | |
| A6.2 | `spikes pull` — fetch from configured endpoint | |
| A6.3 | `spikes push` — upload local feedback to endpoint | |
| A6.4 | Merge strategy: append-only with dedup by ID | |
| **A7** | **CLI: TUI Dashboard** | |
| A7.1 | `spikes dashboard` — ratatui interactive view | |
| A7.2 | Table widget: sortable by page, rating, timestamp, reviewer | |
| A7.3 | Filter input: search across pages, comments, reviewers | |
| A7.4 | Detail pane: show full spike with element context | |
| **A12** | **CLI: Magic Mode + Config** | |
| A12.1 | `spikes` (no args) = auto-init + inject + serve | |
| A12.2 | `.spikes/config.toml` as unified config source | |
| A12.3 | `spikes sync` = pull + push | |
| A12.4 | `spikes remote add/remove/show` for endpoint management | |
| A12.5 | Widget attributes generated from config | |
| **A13** | **Brand Alignment** | |
| A13.1 | Dark theme default across widget, dashboard, TUI | |
| A13.2 | `/` logo mark (not sword emoji) | |
| A13.3 | Rating symbols: `+ / ~ -` | |
| A13.4 | Berkeley Mono / system mono fonts | |
| A13.5 | Brand color palette: red #e74c3c, green #22c55e, blue #3b82f6, yellow #eab308 | |
| **A14** | **Email Collection (Prospect List)** | |
| A14.1 | `data-collect-email="true"` widget attribute | |
| A14.2 | Optional email field in reviewer prompt | |
| A14.3 | `reviewer_email` column in D1 schema | |
| A14.4 | `/prospects` API endpoint for email export | |
| **A8** | **BYO Backend: Cloudflare** | |
| A8.1 | `spikes deploy cloudflare` — scaffolds Worker + D1 schema | |
| A8.2 | Detects existing wrangler.toml or creates new | |
| A8.3 | Worker: POST /spikes (create), GET /spikes (list), GET /spikes/:id (show) | |
| A8.4 | D1 table: spikes (id, project, page, type, selector, rating, comments, reviewer_name, reviewer_id, timestamp, metadata JSON) | |
| A8.5 | Auth: project-specific URL token (e.g., `/spikes?token=abc123`) | |
| A8.6 | CORS: allow any origin (widget needs to POST from file://, localhost, anywhere) | |
| A8.7 | Outputs endpoint URL to configure in widget | |
| **A9** | **BYO Backend: Vercel** | ⚠️ |
| A9.1 | `spikes deploy vercel` — scaffolds serverless function + Vercel KV | ⚠️ |
| A9.2 | Same API shape as Cloudflare (POST/GET /api/spikes) | ⚠️ |
| A9.3 | Detects existing vercel.json or creates new | ⚠️ |
| **A10** | **BYO Backend: Self-Hosted** | |
| A10.1 | `spikes deploy --self-hosted` — outputs standalone Node/Deno/Bun server code | |
| A10.2 | SQLite storage (portable, no external DB needed) | |
| A10.3 | Single file, runs anywhere | |
| **A11** | **HTML Dashboard** | |
| A11.1 | Static `dashboard.html` that reads from localStorage | |
| A11.2 | Lists all spikes with color-coded ratings, grouped by reviewer | |
| A11.3 | Filter by page, rating, reviewer | |
| A11.4 | Export buttons (JSON, CSV) | |
| A11.5 | Clear all button with confirmation | |

---

## Fit Check (R × A)

| Req | Requirement | Status | A |
|-----|-------------|--------|---|
| R0 | Reviewers can leave structured feedback directly on HTML mockups | Core goal | ✅ |
| R1 | Widget works with a single `<script>` tag | Must-have | ✅ |
| R2 | Widget works on file://, localhost, and hosted domains | Must-have | ✅ |
| R3 | Page-level feedback | Must-have | ✅ |
| R4 | Element-level feedback via spike mode | Must-have | ✅ |
| R5 | Rating + comments | Must-have | ✅ |
| R6 | localStorage by default | Must-have | ✅ |
| R7 | Export as JSON/CSV | Must-have | ✅ |
| R8 | CLI with --json flag | Must-have | ✅ |
| R9 | Widget < 10KB gzipped | Must-have | ✅ |
| R10 | Minimal unique selector | Must-have | ✅ |
| R11 | Visual feedback on capture | Must-have | ✅ |
| R12 | Dashboard with filtering | Must-have | ✅ |
| R13 | Configurable widget | Nice-to-have | ✅ |
| R14 | Optional cloud sync | Nice-to-have | ✅ |
| R15 | TUI dashboard | Nice-to-have | ✅ |
| R16 | CLI works with pipes/agents | Must-have | ✅ |
| R17 | One-time purchase (not subscription) | Must-have | ✅ |
| R18 | Free tier = full functionality | Must-have | ✅ |
| R19 | Reviewer must identify themselves | Must-have | ✅ |
| R20 | One-command deploy to user's CF/Vercel | Must-have | ✅ |
| R21 | Works fully without any cloud setup | Must-have | ✅ |

**Notes:**
- A9 (Vercel backend) deprioritized — Cloudflare covers the use case
- All must-haves pass

---

## Data Model

### Spike Object

```typescript
interface Spike {
  id: string;              // nanoid (21 chars, URL-safe)
  type: "page" | "element";
  projectKey: string;      // groups spikes across pages
  page: string;            // document.title or pathname
  url: string;             // full URL for reference
  
  // Reviewer (required)
  reviewer: {
    id: string;            // generated once per browser, persists
    name: string;          // user-provided display name
  };
  
  // Element-specific (only if type === "element")
  selector?: string;       // minimal unique CSS selector
  xpath?: string;          // fallback xpath
  elementText?: string;    // truncated textContent (max 100 chars)
  boundingBox?: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
  
  // Feedback
  rating: "love" | "like" | "meh" | "no" | null;
  comments: string;
  
  // Metadata
  timestamp: string;       // ISO 8601
  viewport: {
    width: number;
    height: number;
  };
  userAgent?: string;
}
```

### Reviewer Object (stored separately in localStorage)

```typescript
interface StoredReviewer {
  id: string;              // nanoid, generated on first spike
  name: string;            // user-provided
  createdAt: string;       // ISO 8601
}
// Stored at: localStorage['spikes:reviewer']
```

### Storage Locations

| Context | Location | Format |
|---------|----------|--------|
| Browser | `localStorage['spikes:{projectKey}']` | JSON array |
| CLI local | `.spikes/feedback.jsonl` | JSONL (one spike per line) |
| Cloud | D1 table or KV namespace | JSON |

---

## Widget Configuration

```html
<!-- Minimal -->
<script src="https://spikes.sh/widget.js"></script>

<!-- Configured -->
<script 
  src="https://spikes.sh/widget.js"
  data-project="my-project"
  data-position="bottom-right"
  data-color="#e74c3c"
  data-endpoint="https://my-worker.workers.dev/spikes"
></script>
```

| Attribute | Default | Description |
|-----------|---------|-------------|
| `data-project` | `location.hostname` | Project key for grouping |
| `data-position` | `bottom-right` | Button position: `bottom-right`, `bottom-left`, `top-right`, `top-left` |
| `data-color` | `#e74c3c` | Button background color |
| `data-theme` | `dark` | Widget theme: `dark` or `light` |
| `data-endpoint` | `null` | Optional POST endpoint for cloud sync |
| `data-collect-email` | `false` | Ask reviewers for email (prospect list) |
| `data-reviewer` | `null` | Optional reviewer name to attach to spikes |

---

## CLI Commands

```
spikes — Feedback collection for static mockups

USAGE:
    spikes [COMMAND]
    
    (no command = magic mode: auto-init, inject widget, serve current dir)

COMMANDS:
    init        Initialize a .spikes/ directory
    list        List all spikes [--json] [--page X] [--rating X] [--reviewer X]
    show        Show a single spike by ID [--json]
    export      Export all spikes [--format json|csv|jsonl]
    hotspots    Show elements with most feedback
    reviewers   List all reviewers who left feedback
    inject      Add widget <script> to HTML files in a directory
    serve       Start local server (static files + POST collector)
    sync        Pull then push to remote (one command)
    pull        Fetch spikes from configured endpoint
    push        Upload local spikes to configured endpoint
    remote      Manage remote endpoint configuration
      add       Add or update remote endpoint
      remove    Remove remote configuration  
      show      Show current remote configuration
    deploy      Deploy backend to your infrastructure
      cloudflare  Deploy Cloudflare Worker + D1
    config      Show current configuration
    dashboard   Interactive TUI dashboard
    version     Show version

OPTIONS:
    -p, --port  Port for dev server (default: 3847)
    --json      Output as JSON (for piping to jq, agents, etc.)
    --help      Show help

EXAMPLES:
    # Zero-config local workflow (magic mode)
    cd ./mockups/
    spikes
    # → auto-init, inject widget, serve on :3847
    
    # Configure remote sync
    spikes remote add https://my-worker.workers.dev --token abc123
    spikes sync
    
    # Query feedback
    spikes list --json | jq '.[] | select(.rating == "no")'
    spikes hotspots
    
    # Deploy your own backend
    spikes deploy cloudflare
```

---

## License & Business Model

### What's Free Forever (MIT Licensed)

| Feature | Notes |
|---------|-------|
| Full widget | All functionality, no limits |
| Full CLI | All commands, --json output |
| Local workflow | `inject` + `serve` + localStorage |
| BYO backend | Deploy to your own Cloudflare/Vercel |
| HTML dashboard | Filter, export, manage locally |
| TUI dashboard | FrankenTUI interactive view |

**Philosophy:** The local workflow is the hook. Self-hostable, no infrastructure cost to us, should stay free.

### The Obvious Paid Value: Hosted Links

The instant-shareable link (`spikes.sh/yourproject` or `yourname.spikes.sh/site`) is the primary commercial opportunity because:
- Zero config — no wrangler setup, no CF account needed
- Professional URLs — `acme.spikes.sh/pricing-v2` vs localhost
- Multi-reviewer persistence — everyone's feedback synced
- Agent access via API — `curl https://acme.spikes.sh/project/spikes.json`
- Password protection — simple auth for client work
- Time-limited links — "Review this in 7 days" creates urgency

### Pricing Model Options (Research)

#### Option A: Appreciation Model (Current)

The "Spike Us Back" / Sublime Text approach. Everything free, payment is gratitude.

| Tier | Price | What You Get |
|------|-------|--------------|
| **Free** | $0 | Everything, forever |
| **Spike Us Back** | $9-$29 (pay what you feel) | Badge + supporters page |
| **Sponsor** | $149 once | Logo on spikes.sh + influence roadmap |

**Pros:** No friction, goodwill, word of mouth, no support burden
**Cons:** Revenue depends on generosity, hard to predict/scale

#### Option B: Fizzy Model (37signals, Dec 2025)

Generous free tier with usage-based constraint. Open source, can self-host.

| Tier | Price | Features |
|------|-------|----------|
| **Free Hosted** | $0 | 1000 spikes, no time limit, no user limit, random URL |
| **Pro** | $20/mo | Unlimited spikes, custom subdomain, API, webhooks |

**Key insight:** Constraint is usage depth (1000 spikes), not artificial limits (time, users). Deleted items still count. "If you'd prefer not to pay us, run it yourself."

**Reference:** fizzy.do — kanban, 1000 cards free, $20/mo unlimited

**Pros:** Clear upgrade path, generous free tier builds goodwill, predictable revenue
**Cons:** Need to build/run hosted infrastructure

#### Option C: ONCE Model (37signals, 2024)

Completely free + open source. Revenue from brand/ecosystem, not the product.

| Tier | Price | What You Get |
|------|-------|--------------|
| **Free** | $0 | Everything, self-hosted, MIT licensed |

**Reference:** once.com/campfire — group chat, 100% free, you get the code

**Pros:** Zero infrastructure cost, maximum goodwill, developer love
**Cons:** No direct revenue from product

#### Option D: Hosted SaaS (Classic)

Standard tiered pricing for hosted service.

| Tier | Price | Features |
|------|-------|----------|
| **Free Hosted** | $0 | 3 projects, 7-day expiry, random URL |
| **Pro** | $9/mo | Unlimited projects, custom subdomain, no expiry |
| **Team** | $29/mo | Multiple seats, aggregated dashboard, SSO |

**Pros:** Predictable revenue, clear value exchange
**Cons:** Feels extractive, competes with self-host option

#### Option E: Tool Suite Bundle (Every.to Model)

Multiple tools under one subscription.

```
spikes.sh  — Feedback loop for prototypes
marks.sh   — [future] Annotation for live sites  
drafts.sh  — [future] Version management for static sites
```

**Bundle: $19/mo for access to all tools**

**Pros:** Multiple acquisition channels, higher LTV, brand identity
**Cons:** Requires building multiple tools, complex to manage

### Current Recommendation

**Start with Option A (Appreciation) + build toward Option B (Fizzy) for hosted.**

Rationale:
- Appreciation model is already live and working
- Hosted links are the obvious paid value, but need to be built (V9)
- Fizzy model is the most aligned: generous free, simple paid, open source escape hatch
- Can layer on Option E later as more tools emerge

See `LICENSE-MODEL.md` for implementation details.

---

## Agent Harness Integrations

The deep value proposition: Spikes isn't just a feedback tool, it's a **bridge between human review and agent action**.

### Current Flow
```
Agent builds → Human reviews (Spikes) → Human pastes JSON → Agent fixes
```

### Target Flow (V10+)
```
Agent builds → Human reviews (Spikes) → Webhook → Agent context → Agent auto-implements
```

### Planned Integrations

| Integration | Description | Priority |
|-------------|-------------|----------|
| **Cursor context export** | `spikes export --format cursor-context > .cursor/spikes.md` | High |
| **Claude Code context** | Same format, auto-picked up by CLAUDE.md | High |
| **MCP server** | Spikes as tool agents can query: `get_spikes`, `get_feedback_for_selector` | High |
| **GitHub Actions** | Fail deploy if negative feedback exists | Medium |
| **Webhook on spike** | POST to URL when feedback added → trigger CI, notify Slack, update Linear | Medium |
| **Factory harness** | Native integration with Factory agent workflows | Medium |

### MCP Server Spec (Draft)

```typescript
// Tools exposed via MCP
tools: {
  get_spikes: {
    description: "Get all feedback spikes for the project",
    parameters: { rating?: string, page?: string, reviewer?: string },
    returns: Spike[]
  },
  get_element_feedback: {
    description: "Get feedback for a specific CSS selector",
    parameters: { selector: string },
    returns: Spike[]
  },
  get_hotspots: {
    description: "Get elements with most feedback",
    parameters: { limit?: number },
    returns: { selector: string, count: number, ratings: object }[]
  }
}
```

### Agent Workflow Examples

**1. Review Gate in CI:**
```yaml
# .github/workflows/deploy.yml
- name: Check for blocking feedback
  run: |
    spikes pull
    BLOCKERS=$(spikes list --rating no --json | jq 'length')
    if [ "$BLOCKERS" -gt 0 ]; then
      echo "::error::$BLOCKERS blocking feedback items found"
      exit 1
    fi
```

**2. Autonomous Implementation:**
```bash
# Agent prompt
"Read spikes.json. For each spike with rating 'no' or 'meh', 
implement the requested change. The selector field tells you 
exactly which element to modify."
```

**3. Context File Generation:**
```bash
spikes export --format cursor-context > .cursor/feedback.md
# Now Cursor sees all feedback in context automatically
```

---

## Technical Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  spikes.js (widget) — <10KB gzipped                             │
│  ├── Floating button → spike mode toggle                        │
│  ├── Spike mode: crosshair cursor, element highlights           │
│  ├── Click element → element-mode popover                       │
│  ├── Click button again → page-mode modal                       │
│  ├── First-time reviewer prompt ("What's your name?")           │
│  ├── Selector computation (ID → class → nth-child)              │
│  ├── localStorage read/write (spikes:{project}, spikes:reviewer)│
│  └── Optional POST to endpoint (fire-and-forget)                │
└──────────────────────────┬──────────────────────────────────────┘
                           │
         ┌─────────────────┴─────────────────┐
         │                                   │
         ▼                                   ▼
┌─────────────────────┐           ┌─────────────────────────────┐
│  localStorage       │           │  BYO Backend (user deploys) │
│  ├── spikes:{proj}  │           │  ┌───────────────────────┐  │
│  └── spikes:reviewer│           │  │ Cloudflare Worker+D1  │  │
└─────────┬───────────┘           │  └───────────────────────┘  │
          │                       │  ┌───────────────────────┐  │
          │ export                │  │ Vercel Function+KV    │  │
          │                       │  └───────────────────────┘  │
          │                       │  ┌───────────────────────┐  │
          │                       │  │ Self-hosted (SQLite)  │  │
          │                       │  └───────────────────────┘  │
          │                       └──────────────┬──────────────┘
          │                                      │
          ▼                                      │
┌─────────────────────────────────────────────────────────────────┐
│  CLI (spikes) — Rust binary                                     │
│  ├── Reads .spikes/feedback.jsonl                               │
│  ├── list, show, export, hotspots, reviewers, inject, serve     │
│  ├── pull/push for remote sync                                  │
│  ├── deploy cloudflare|vercel|--self-hosted                     │
│  ├── All commands support --json for agent consumption          │
│  └── dashboard: FrankenTUI interactive view                     │
└─────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│  dashboard.html — Static HTML                                   │
│  ├── Reads localStorage directly                                │
│  ├── Filter/sort by page, rating, reviewer, timestamp           │
│  └── Export buttons (JSON, CSV)                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
Reviewer loads mockup.html (with widget)
    │
    ▼
Widget loads, checks localStorage for reviewer identity
    │
    ├── No identity? → Prompt on first spike attempt
    │
    ▼
Reviewer clicks Spikes button → enters spike mode
    │
    ├── Click element → element spike (with selector)
    └── Click button again → page spike
    │
    ▼
Widget saves to localStorage (always)
    │
    ├── If endpoint configured → also POST to backend (fire-and-forget)
    │
    ▼
Designer runs CLI
    │
    ├── spikes list (reads .spikes/feedback.jsonl)
    ├── spikes pull (fetches from backend, merges)
    ├── spikes export --format jsonl > feedback.jsonl (for agents)
    └── spikes dashboard (interactive TUI)
```

---

## Open Questions / Spikes Needed

| # | Question | Status |
|---|----------|--------|
| Q1 | FrankenTUI API — what crates to use, how to structure the dashboard? | RESOLVED: using ratatui |
| Q2 | Selector algorithm — use existing lib (e.g., `finder`, `optimal-select`) or roll our own? | RESOLVED: rolled our own |
| Q3 | Vercel KV API — how does it compare to D1, what's the schema? | Deprioritized |
| Q4 | Widget bundling — esbuild? rollup? hand-rolled IIFE? | RESOLVED: hand-rolled IIFE |
| Q5 | How to handle localStorage quota limits on very large feedback sets? | Consider for V9 |
| Q6 | Should `spikes deploy` shell out to wrangler/vercel CLI, or use their APIs directly? | RESOLVED: shell out |
| Q7 | Nanoid vs UUID for IDs — nanoid is smaller, but is it universal enough? | RESOLVED: nanoid |
| **Q8** | **Hosted infrastructure — Workers + D1 vs Pages + KV?** | **Needs spike for V9** |
| **Q9** | **Payment integration — Stripe direct or Lemon Squeezy?** | **Decide for V9** |
| **Q10** | **MCP SDK — Rust or TypeScript implementation?** | **Needs spike for V10** |
| **Q11** | **Subdomain routing — Cloudflare Workers with custom domains?** | **Needs spike for V9** |

---

## Rabbit Holes (Avoid)

- **Screenshot/annotation** — Way out of scope. Selector capture is enough.
- **User accounts** — URL tokens only. No auth system. Reviewer identity is local, not authenticated.
- **Real-time sync** — Polling or manual pull/push is fine. No WebSockets.
- **Rich text comments** — Plain text only.
- **Versioning/history** — Append-only, no edit/delete in v1.
- **Browser extension** — Widget-only. No extension.
- **Our own hosted backend** — BYO only. We don't want to run infrastructure.
- **Payment processing complexity** — Gumroad or Lemon Squeezy, not Stripe integration.
- **Analytics/tracking** — None. Privacy-first.

---

## Out of Scope (v1)

- Mobile-specific UI
- Offline-first sync (just use localStorage + manual export)
- Comparison mode (side-by-side mockups)
- Email notifications
- Integrations (Slack, Linear, etc.) — users can build with CLI + JSON
- i18n (English only for v1)

---

## Success Criteria

1. Designer can add `<script src="spikes.sh/widget.js">` to 10 mockup HTML files and collect feedback from a client in < 5 minutes
2. Reviewer is prompted for their name on first feedback; all subsequent spikes tagged with their identity
3. Reviewer can spike whole pages AND specific elements via toggle mode (no modifier keys)
4. Designer can run `spikes list --json | jq '.[] | select(.rating == "no")'` to find all negative feedback
5. Designer can run `spikes hotspots` to see which elements got the most comments
6. Designer can run `spikes reviewers` to see who left feedback
7. Agent can consume feedback via `spikes export --format jsonl` and generate a task list
8. Designer can run `spikes deploy cloudflare` and have a working multi-reviewer backend in < 10 minutes
9. Multiple reviewers can leave feedback on the same mockups without stepping on each other
10. Designer can filter feedback by reviewer: `spikes list --reviewer "Patricia"`

---

## Slices

### Summary

| # | Slice | Status | Demo |
|---|-------|--------|------|
| V1 | Widget: Page Feedback | SHIPPED | "Add script, click button, rate page, see in localStorage" |
| V2 | Widget: Element Spike Mode | SHIPPED | "Enter spike mode, click element, selector captured" |
| V3 | Widget: Reviewer Identity | SHIPPED | "First spike prompts name, all spikes tagged" |
| V4 | HTML Dashboard | SHIPPED | "Open dashboard.html, filter by page/reviewer/rating" |
| V5 | CLI: Core Commands | SHIPPED | "`spikes list --json`, `spikes hotspots` work" |
| V6 | CLI: Inject + Serve | SHIPPED | "`spikes inject ./mockups/`, `spikes serve` full local flow" |
| V7 | BYO Backend: Cloudflare | SHIPPED | "`spikes deploy cloudflare`, multi-reviewer sync" |
| V8 | CLI: Magic Mode + Brand | SHIPPED | "`spikes` with no args, dark theme, unified config" |
| **V9** | **Hosted Links** | **Shaping** | **`spikes share` → instant spikes.sh/yourproject URL** |
| **V10** | **Agent Integrations** | **Shaping** | **MCP server, context export, webhooks** |

---

### V1: Widget — Page Feedback

**Goal:** Minimal working widget that collects page-level feedback.

**Affordances:**
- U1: Floating button (bottom-right, configurable)
- U2: Page-mode modal (rating buttons + textarea)
- U3: Save button
- U4: Cancel button
- N1: localStorage read/write (`spikes:{project}`)
- N2: Generate spike ID (nanoid)
- N3: Capture page metadata (title, url, viewport)

**Acceptance:**
- [ ] `<script src="spikes.js">` on any HTML page shows floating button
- [ ] Click button → modal appears with rating options + comment field
- [ ] Save → spike stored in localStorage as JSON array
- [ ] Refresh page → can add more spikes, array grows
- [ ] Works on `file://`, `localhost`, `https://`

---

### V2: Widget — Element Spike Mode

**Goal:** Toggle-based element selection with selector capture.

**Affordances:**
- U5: Spike mode indicator (button state change, tooltip)
- U6: Element highlight on hover (in spike mode)
- U7: Element-mode popover (anchored to element)
- U8: Captured element preview in popover
- N4: Spike mode state machine (idle → armed → capturing)
- N5: Element hover detection + highlight injection
- N6: Selector computation (ID → class → nth-child)
- N7: Capture boundingBox, textContent, xpath

**Acceptance:**
- [ ] Click button → enters spike mode (cursor changes, button pulses)
- [ ] Hover elements → highlight outline appears
- [ ] Click element → popover appears anchored to element
- [ ] Popover shows element preview (selector, text snippet)
- [ ] Save → spike includes selector, elementText, boundingBox
- [ ] Press Escape or click button → page-mode modal instead
- [ ] Click outside → exits spike mode

---

### V3: Widget — Reviewer Identity

**Goal:** Capture reviewer name on first spike, persist and tag all spikes.

**Affordances:**
- U9: "What should we call you?" prompt (inline in modal/popover)
- U10: "Reviewing as: [name]" indicator
- U11: Change name button
- N8: Check localStorage for existing reviewer
- N9: Generate reviewer ID (nanoid, once)
- N10: Store reviewer in `spikes:reviewer`
- N11: Attach reviewer to every spike

**Acceptance:**
- [ ] First spike attempt → name prompt appears before save
- [ ] Enter name → saved to localStorage, spike includes reviewer
- [ ] Subsequent spikes → no prompt, reviewer auto-attached
- [ ] "Reviewing as: X" shown in widget UI
- [ ] Click name → can change it
- [ ] `data-reviewer="Patricia"` attribute pre-sets name, skips prompt

---

### V4: HTML Dashboard

**Goal:** Static HTML page that displays all feedback from localStorage.

**Affordances:**
- U12: Spike list (cards with rating badges)
- U13: Filter by page dropdown
- U14: Filter by reviewer dropdown
- U15: Filter by rating buttons
- U16: Export JSON button
- U17: Export CSV button
- U18: Clear all button (with confirmation)
- N12: Read from localStorage
- N13: Parse and render spikes
- N14: Filter logic
- N15: Export formatters

**Acceptance:**
- [ ] Open `dashboard.html` → shows all spikes from localStorage
- [ ] Color-coded rating badges (green/blue/orange/red)
- [ ] Filter by page → list updates
- [ ] Filter by reviewer → list updates
- [ ] Filter by rating → list updates
- [ ] Export JSON → downloads file
- [ ] Export CSV → downloads file
- [ ] Clear all → confirmation → localStorage cleared

---

### V5: CLI — Core Commands

**Goal:** Rust CLI that reads feedback and outputs JSON for agents.

**Affordances:**
- N16: `spikes init` → create `.spikes/` directory
- N17: `spikes list` → read `.spikes/feedback.jsonl`, print table
- N18: `spikes list --json` → output JSON array
- N19: `spikes list --page X --reviewer Y --rating Z` → filtered
- N20: `spikes show <id>` → print single spike detail
- N21: `spikes export --format json|csv|jsonl` → dump all
- N22: `spikes hotspots` → aggregate by selector, sort by count
- N23: `spikes reviewers` → list unique reviewers

**Acceptance:**
- [ ] `spikes init` creates `.spikes/config.toml` and empty `feedback.jsonl`
- [ ] `spikes list` with sample data shows formatted table
- [ ] `spikes list --json` outputs valid JSON array
- [ ] `spikes list --reviewer "Patricia"` filters correctly
- [ ] `spikes show abc123` shows full spike detail
- [ ] `spikes export --format csv` produces valid CSV
- [ ] `spikes hotspots` shows "nav > a.pricing: 5 spikes"
- [ ] `spikes reviewers` shows "Patricia (12 spikes), John (3 spikes)"

---

### V6: CLI — Inject + Serve

**Goal:** Full local workflow with one command.

**Affordances:**
- N24: `spikes inject <dir>` → find HTML files, add script tag
- N25: `spikes inject --remove <dir>` → remove script tags
- N26: `spikes serve` → static file server on localhost
- N27: `spikes serve` → POST /spikes endpoint that appends to feedback.jsonl
- N28: Serve dashboard.html at /dashboard

**Acceptance:**
- [ ] `spikes inject ./mockups/` adds `<script>` to all `.html` files
- [ ] Injection is idempotent (doesn't double-add)
- [ ] `spikes inject --remove` cleanly removes script tags
- [ ] `spikes serve` starts server, prints URL
- [ ] Opening mockup in browser → widget works
- [ ] Saving spike → POST to server → appended to feedback.jsonl
- [ ] `/dashboard` serves the HTML dashboard
- [ ] Full loop: inject → serve → review → list

---

### V7: BYO Backend — Cloudflare

**Goal:** One-command deploy to user's Cloudflare account.

**Affordances:**
- N29: `spikes deploy cloudflare` → scaffold Worker + D1
- N30: Detect existing wrangler.toml or create new
- N31: Generate project token for auth
- N32: Worker: POST /spikes (create spike)
- N33: Worker: GET /spikes (list with filters)
- N34: Worker: GET /spikes/:id (show one)
- N35: D1 schema migration
- N36: CORS headers for any origin
- N37: Output endpoint URL + instructions
- N38: `spikes pull` → fetch from endpoint, merge to local
- N39: `spikes push` → upload local to endpoint
- N40: Widget: POST to endpoint if `data-endpoint` configured

**Acceptance:**
- [ ] `spikes deploy cloudflare` scaffolds `worker/` directory
- [ ] Running `wrangler deploy` succeeds
- [ ] Worker responds to POST /spikes with valid spike
- [ ] Worker responds to GET /spikes with list
- [ ] Token auth works (rejects requests without valid token)
- [ ] Widget with `data-endpoint` POSTs successfully
- [ ] `spikes pull` fetches remote spikes, merges with local
- [ ] `spikes push` uploads local spikes to remote
- [ ] Two reviewers on different browsers → both spikes appear

---

### V8: CLI — Magic Mode + Brand Alignment

**Goal:** Zero-config workflow, unified config system, brand-aligned UI across all surfaces.

**Affordances:**
- N41: `spikes` (no args) = magic mode: auto-init, inject, serve
- N42: `.spikes/config.toml` as single source of truth
- N43: `spikes sync` = pull + push in one command
- N44: `spikes remote add/remove/show` for endpoint management
- N45: `spikes config` shows effective configuration
- N46: Widget attributes auto-generated from config
- N47: Dark theme as default (matching spikes.sh brand)
- N48: `/` logo mark replaces sword emoji throughout
- N49: Rating symbols: `+ / ~ -` instead of emoji
- N50: TUI dashboard with brand colors (ratatui)
- N51: Email collection option (`data-collect-email="true"`)
- N52: `/prospects` endpoint for prospect list export

**Config File (`.spikes/config.toml`):**
```toml
[project]
key = "my-project"

[widget]
theme = "dark"
position = "bottom-right"
color = "#e74c3c"
collect_email = false

[remote]
endpoint = "https://my-worker.workers.dev"
token = "abc123"
# hosted = true  # for future spikes.sh managed backend
```

**Acceptance:**
- [x] `spikes` with no args serves current directory with widget
- [x] Auto-init creates `.spikes/` if missing
- [x] `spikes sync` pulls then pushes
- [x] `spikes remote add <url> --token <t>` configures endpoint
- [x] `spikes config --json` shows all settings
- [x] Widget injection uses config for all attributes
- [x] Dashboard dark theme with `/` branding
- [x] Widget dark theme with `/` button
- [x] TUI uses brand colors (red, green, blue, yellow)
- [x] Email collection works with D1 backend
- [x] spikes.sh site has widget integrated for dogfooding

---

### V9: Hosted Links — spikes.sh/yourproject

**Goal:** Instant shareable preview links without any infrastructure setup.

**Why this is the monetization core:**
- Local workflow is valuable but not monetizable (self-hostable)
- Hosted links require infrastructure we run → obvious value exchange
- Zero-config sharing is what people will pay for

**Affordances:**
- N46: `spikes share ./mockups/` → uploads files + widget to hosted endpoint
- N47: Returns URL: `spikes.sh/abc123` (free) or `yourname.spikes.sh/project` (pro)
- N48: Hosted endpoint stores spikes in D1/KV (our infrastructure)
- N49: API endpoint: `GET /project/spikes.json` returns all feedback
- N50: Optional expiry (7 days free, configurable for pro)
- N51: Optional password protection (pro)
- N52: `spikes pull --from https://spikes.sh/abc123` fetches remote to local
- U24: Share confirmation with URL and QR code

**Tiers:**
| Tier | URL Format | Expiry | Features |
|------|------------|--------|----------|
| Free | `spikes.sh/random-slug` | 7 days | 1 active project |
| Pro | `yourname.spikes.sh/*` | None | Unlimited, API, webhooks |

**Acceptance:**
- [ ] `spikes share ./mockups/` uploads and returns URL
- [ ] URL is accessible immediately, shows mockups with widget
- [ ] Feedback from multiple reviewers persists
- [ ] `spikes pull --from URL` downloads feedback locally
- [ ] Free tier expires after 7 days
- [ ] Pro tier requires auth token (Stripe integration)

---

### V10: Agent Integrations — MCP + Context Export

**Goal:** Make Spikes a first-class citizen in agent workflows.

**Affordances:**
- N53: `spikes export --format cursor-context` → markdown for .cursor/
- N54: `spikes export --format claude-context` → markdown for CLAUDE.md
- N55: MCP server binary: `spikes mcp serve`
- N56: MCP tools: `get_spikes`, `get_element_feedback`, `get_hotspots`
- N57: Webhook config: `spikes config set webhook https://...`
- N58: Webhook fires on new spike (POST with spike JSON)
- N59: GitHub Action: `spikes-action` checks for blocking feedback

**Context Export Format:**
```markdown
# Feedback Spikes

## Blocking (rating: no)

### `.pricing-card` on /pricing.html
> "This card needs more breathing room" — Sarah (Product)

Selector: `.pricing-card`
Bounding box: {x: 100, y: 200, width: 300, height: 150}

## Needs Attention (rating: meh)
...
```

**Acceptance:**
- [ ] `spikes export --format cursor-context` produces valid markdown
- [ ] MCP server starts and responds to tool calls
- [ ] Webhook fires within 1s of new spike
- [ ] GitHub Action fails build when `--rating no` spikes exist

---

## Future Vision (v2+)

The v1 scope solves feedback collection. But there's a larger workflow: **presenting mockups for review**. Future versions could own more of this:

### Mockup Gallery Generator
- `spikes gallery ./mockups/` → generates an `index.html` with cards for each mockup
- Auto-injects widget into all HTML files
- Configurable: titles, descriptions, "recommended" badges, ordering
- Like the Patricia gallery page, but generated from a simple config:

```yaml
# spikes.yaml
title: "Website Concepts for Patricia"
mockups:
  - file: 01-atelier-luxe.html
    title: "Atelier Luxe"
    description: "High-end minimalist approach"
    tags: [recommended, premium]
  - file: 02-school-of-sugar.html
    title: "School of Sugar"
    description: "Educational focus"
```

### Comparison Mode
- Side-by-side view of two mockups
- Synced scrolling
- "Which do you prefer?" UI with structured choice capture

### Review Sessions
- `spikes session create "Patricia Review Round 2"` 
- Time-boxed review with all feedback grouped
- Session summary: "Patricia reviewed 8 pages, left 23 spikes, 4 negative"

### Hosted Preview Links
- `spikes share ./mockups/` → uploads to temporary hosting (Cloudflare Pages?)
- Returns a shareable URL that expires in 7 days
- No client setup needed — just send the link

### Feedback-to-Tasks Pipeline
- `spikes tasks` → generates task list from negative/meh feedback
- Export to Linear, GitHub Issues, or plain markdown
- Agent-friendly: `spikes export --format tasks`

### Rich Element Context
- Capture computed styles of spiked element
- Screenshot of element (via html2canvas, opt-in)
- Before/after comparison when element changes

### Multi-Project Dashboard
- Single view across all projects
- "What needs my attention?" across clients

---

**For v1:** Stay focused on the core loop (widget → feedback → CLI → dashboard). These expansions come later, informed by real usage.

---

## Deployed Infrastructure

### spikes.sh Website
- **Hosting:** Cloudflare Pages (auto-deploy from `main` branch)
- **Domain:** spikes.sh
- **Widget:** Integrated on all pages with email collection

### spikes.sh Feedback Backend
- **Worker:** `spikes-sh-worker.moritzbierling.workers.dev`
- **Database:** D1 `spikes-sh-db` (WEUR region)
- **Endpoints:**
  - `POST /spikes` — public (widget writes)
  - `GET /spikes?token=X` — authenticated (CLI reads)
  - `GET /prospects?token=X` — authenticated (email export)
- **Token:** stored in wrangler.toml (not committed)

### Local Development
```bash
cd site/worker
npm run dev          # Local worker + D1
npm run db:migrate   # Apply schema changes
npm run deploy       # Push to production
```

---

## References

- Original implementation: `/Users/moritzbierling/werk/gate/patricia-arribalzaga/mockups/`
- FrankenTUI: https://github.com/Dicklesworthstone/frankentui, https://frankentui.com
- Ryan Singer's shaping skills: https://github.com/rjs/shaping-skills
- beads_rust CLI patterns: `br --help`
- Berkeley Mono license model: https://usgraphics.com/catalog
