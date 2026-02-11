<coding_guidelines>
# Spikes — Agent Instructions

## Project Overview

Spikes is the feedback loop tool for AI-assisted building. Building prototypes is easy now (Claude Code, Cursor, v0). The bottleneck is turning feedback into action. Spikes makes feedback structured, precise, and agent-ready.

**Two core use cases:**
1. **You reviewing agent work** — Click elements, leave comments, agent gets exact CSS selectors
2. **Collecting feedback from others** — Share a link, they spike it, you get JSON for your agent

**Components:**
- **Widget:** Vanilla JS (`widget/spikes.js`) that injects a floating button
- **CLI:** Rust binary (`cli/`) with robot-friendly JSON output
- **Dashboard:** Static HTML (`widget/dashboard.html`) for viewing feedback
- **Site:** Landing page and docs (`site/`)

## Task Tracking

**Use beads (br) for all task tracking:**
```bash
br ready --json              # See current work
br create "Task description" # Create new task
br close <id>                # Complete task
br show <id>                 # View task details
```

All work should flow through beads. Check `br ready` before starting. Create beads for discoveries during implementation.

## Architecture

```
widget/
  spikes.js       # Drop-in widget (<10KB gzipped)
  dashboard.html  # Static feedback viewer

cli/
  Cargo.toml
  src/
    main.rs
    commands/     # list, show, export, inject, serve, deploy, etc.
    tui/          # FrankenTUI dashboard (V8)
  templates/
    cloudflare/   # Worker + D1 scaffolding
```

## Key Patterns

### Widget (JavaScript)
- IIFE pattern, no global pollution except `window.Spikes` for config
- All styles inline (no external CSS)
- localStorage for persistence: `spikes:{project}` for data, `spikes:reviewer` for identity
- Target: <10KB gzipped, zero dependencies

### CLI (Rust)
- Use `clap` for argument parsing
- All commands support `--json` flag for machine-readable output
- Follow `beads_rust` patterns for output formatting
- Data file: `.spikes/feedback.jsonl` (one spike per line)

### Data Format

```typescript
interface Spike {
  id: string;                    // nanoid
  type: "page" | "element";
  projectKey: string;
  page: string;
  url: string;
  reviewer: { id: string; name: string };
  selector?: string;             // element only
  elementText?: string;          // element only
  boundingBox?: { x, y, width, height };
  rating: "love" | "like" | "meh" | "no" | null;
  comments: string;
  timestamp: string;             // ISO 8601
  viewport: { width, height };
}
```

## Slice Dependencies

Two parallel tracks:

**Widget Track:** V1 → V2 → V3 → V4
**CLI Track:** V5 → V6 → V7

V8 (TUI) depends on V5 but can be built in parallel with V6/V7.

## Testing

- Widget: Manual testing on file://, localhost, https://
- CLI: `cargo test` + manual verification
- Integration: `spikes inject` + `spikes serve` + browser + `spikes list`

## Build Commands

```bash
# CLI
cd cli && cargo build --release

# Widget (no build step, but check size)
gzip -c widget/spikes.js | wc -c  # should be <10KB
```

## Deployment

**Site deploys via GitHub → Cloudflare Pages.** Push to `main` triggers automatic deployment:

```bash
git push origin main  # This IS the deploy command
```

No wrangler, no manual deploy scripts. Just push.

## Business Model

**Free forever (MIT):**
- Full widget + CLI
- Local workflow (`inject` + `serve`)
- BYO backend (Cloudflare deploy)
- Unlimited local usage

**Paid hosting (planned):**
- Instant shareable links: `yourname.spikes.sh/project`
- No wrangler setup, no CF account needed
- Multi-reviewer persistence
- API access, webhooks
- Time-limited/password-protected links

**Agent integrations (planned):**
- MCP server for agent harnesses
- Cursor/Claude Code context file export
- GitHub Actions integration (fail deploy on negative feedback)
- Webhook triggers for autonomous agent workflows

## References

- Shaping doc: `shaping.md`
- Original prototype: `/Users/moritzbierling/werk/gate/patricia-arribalzaga/mockups/`
- FrankenTUI: https://github.com/Dicklesworthstone/frankentui
</coding_guidelines>
