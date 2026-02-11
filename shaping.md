---
shaping: true
---

# Spikes — Shaping Document

**Status:** Shaping in progress  
**Domain:** spikes.sh  
**Tagline:** Drop-in feedback for static mockups

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

### Problem

- Designers share static HTML mockups with clients for review
- Feedback arrives scattered across WhatsApp, Slack, email — unstructured and disconnected from the mockups themselves
- No way to reference specific elements precisely ("the third button" is ambiguous)
- Feedback isn't programmatically accessible — can't pipe it to other tools or automate workflows
- Existing tools require accounts, backends, subscriptions — friction for simple mockup reviews
- Agents can't easily consume or act on the feedback

### Outcome

- Reviewers leave structured feedback directly on the mockup with one script tag
- Page-level AND element-level feedback with precise selector capture
- All feedback is locally stored (offline-first) with optional cloud sync
- CLI provides robot-friendly access to all feedback data (JSON output, query commands)
- Agents can read, filter, and act on feedback programmatically
- Zero accounts required for basic usage; one-time purchase for commercial/cloud features

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
| **A7** | **CLI: TUI Dashboard** | ⚠️ |
| A7.1 | `spikes dashboard` — FrankenTUI interactive view | ⚠️ |
| A7.2 | Table widget: sortable by page, rating, timestamp, reviewer | ⚠️ |
| A7.3 | Filter input: search across pages, comments, reviewers | ⚠️ |
| A7.4 | Detail pane: show full spike with element context | ⚠️ |
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
| R15 | TUI dashboard | Nice-to-have | ⚠️ |
| R16 | CLI works with pipes/agents | Must-have | ✅ |
| R17 | One-time purchase (not subscription) | Must-have | ✅ |
| R18 | Free tier = full functionality | Must-have | ✅ |
| R19 | Reviewer must identify themselves | Must-have | ✅ |
| R20 | One-command deploy to user's CF/Vercel | Must-have | ✅ |
| R21 | Works fully without any cloud setup | Must-have | ✅ |

**Notes:**
- R15 depends on A7 (FrankenTUI integration) which is flagged — needs spike to understand FrankenTUI API
- A9 (Vercel backend) is flagged — needs spike to understand Vercel KV API
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
| `data-endpoint` | `null` | Optional POST endpoint for cloud sync |
| `data-reviewer` | `null` | Optional reviewer name to attach to spikes |

---

## CLI Commands

```
spikes — Feedback collection for static mockups

USAGE:
    spikes <COMMAND>

COMMANDS:
    init        Initialize a .spikes/ directory
    list        List all spikes [--json] [--page X] [--rating X] [--reviewer X]
    show        Show a single spike by ID [--json]
    export      Export all spikes [--format json|csv|jsonl]
    hotspots    Show elements with most feedback
    reviewers   List all reviewers who left feedback
    inject      Add widget <script> to HTML files in a directory
    serve       Start local server (static files + POST collector)
    pull        Fetch spikes from configured endpoint
    push        Upload local spikes to configured endpoint
    deploy      Deploy backend to your infrastructure
      cloudflare  Deploy Cloudflare Worker + D1
      vercel      Deploy Vercel function + KV
      --self-hosted  Output standalone server code
    dashboard   Interactive TUI dashboard (FrankenTUI)
    config      Show/edit configuration
    version     Show version

OPTIONS:
    --json      Output as JSON (for piping to jq, agents, etc.)
    --help      Show help

EXAMPLES:
    # Basic local workflow
    spikes inject ./mockups/
    spikes serve
    # → share localhost URL with reviewer
    spikes list --json | jq '.[] | select(.rating == "no")'
    
    # Deploy to Cloudflare (one-time setup)
    spikes deploy cloudflare
    # → outputs endpoint URL
    # → add data-endpoint="..." to your widget script tags
    
    # Multi-reviewer workflow
    spikes pull
    spikes list --reviewer "Patricia"
    spikes hotspots
```

---

## License Model

| Tier | Price | What You Get |
|------|-------|--------------|
| **Free** | $0 | Everything: Widget + CLI + local storage + HTML dashboard + BYO backend deploy + TUI — unlimited, forever |
| **Supporter** | $19+ (pay what you want, min $9) | Same features + license badge for your site + priority GitHub issues + our gratitude |
| **Team** | $49 once | Team license (5 seats) + priority support + logo on spikes.sh |

**Philosophy:** 
- No feature gating. Free tier is 100% functional.
- You deploy to YOUR infrastructure (Cloudflare, Vercel) — we don't pay for your hosting, you don't pay us for features.
- Payment is appreciation, not access. Like Sublime Text or WinRAR.
- If Spikes saves you time and makes your workflow better, throw us some money. If not, use it free forever.

**Why this works:**
- Cloudflare D1/Workers free tier is generous — most users will never pay Cloudflare anything
- Vercel has similar free tier
- We have near-zero infrastructure costs (static site + GitHub)
- Revenue comes from people who *want* to pay, not people forced to

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
| Q1 | FrankenTUI API — what crates to use, how to structure the dashboard? | Needs spike |
| Q2 | Selector algorithm — use existing lib (e.g., `finder`, `optimal-select`) or roll our own? | Needs spike |
| Q3 | Vercel KV API — how does it compare to D1, what's the schema? | Needs spike |
| Q4 | Widget bundling — esbuild? rollup? hand-rolled IIFE? | Decide |
| Q5 | How to handle localStorage quota limits on very large feedback sets? | Consider |
| Q6 | Should `spikes deploy` shell out to wrangler/vercel CLI, or use their APIs directly? | Decide |
| Q7 | Nanoid vs UUID for IDs — nanoid is smaller, but is it universal enough? | Decide |

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

| # | Slice | Parts | Demo |
|---|-------|-------|------|
| V1 | Widget: Page Feedback | A1.1, A1.2, A3.1-A3.3 | "Add script, click button, rate page, see in localStorage" |
| V2 | Widget: Element Spike Mode | A2.1-A2.7, A1.3, A1.4 | "Enter spike mode, click element, selector captured" |
| V3 | Widget: Reviewer Identity | A4.1-A4.5 | "First spike prompts name, all spikes tagged" |
| V4 | HTML Dashboard | A11.1-A11.5 | "Open dashboard.html, filter by page/reviewer/rating" |
| V5 | CLI: Core Commands | A5.1-A5.5, A6.1 | "`spikes list --json`, `spikes hotspots` work" |
| V6 | CLI: Inject + Serve | A5.6-A5.7 | "`spikes inject ./mockups/`, `spikes serve` full local flow" |
| V7 | BYO Backend: Cloudflare | A8.1-A8.7, A3.4, A6.2-A6.4 | "`spikes deploy cloudflare`, multi-reviewer sync" |
| V8 | CLI: TUI Dashboard | A7.1-A7.4 | "`spikes dashboard` interactive FrankenTUI" |

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

### V8: CLI — TUI Dashboard

**Goal:** Interactive terminal dashboard using FrankenTUI.

**Affordances:**
- U19: Table view (sortable columns: page, rating, reviewer, time)
- U20: Filter input (search across all fields)
- U21: Detail pane (full spike info when row selected)
- U22: Keyboard navigation (j/k, enter, q)
- U23: Rating filter buttons
- N41: FrankenTUI app setup (Elm architecture)
- N42: Load spikes from file
- N43: Table widget with selection state
- N44: Filter logic (reactive)
- N45: Detail view renderer

**Acceptance:**
- [ ] `spikes dashboard` opens TUI
- [ ] Table shows all spikes with columns
- [ ] Arrow keys / j/k navigate rows
- [ ] Enter on row → detail pane shows full spike
- [ ] Typing in filter → table filters live
- [ ] Rating buttons filter by rating
- [ ] `q` quits
- [ ] Handles 1000+ spikes without lag

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

## References

- Original implementation: `/Users/moritzbierling/werk/gate/patricia-arribalzaga/mockups/`
- FrankenTUI: https://github.com/Dicklesworthstone/frankentui, https://frankentui.com
- Ryan Singer's shaping skills: https://github.com/rjs/shaping-skills
- beads_rust CLI patterns: `br --help`
- Berkeley Mono license model: https://usgraphics.com/catalog
