<coding_guidelines>
# Spikes — Agent Instructions

## Project Overview

Spikes is the feedback loop tool for AI-assisted building. Building prototypes is easy now (Claude Code, Cursor, v0). The bottleneck is turning feedback into action. Spikes makes feedback structured, precise, and agent-ready.

**Two core use cases:**
1. **You reviewing agent work** — Click elements, leave comments, agent gets exact CSS selectors
2. **Collecting feedback from others** — Share a link, they spike it, you get JSON for your agent

**Components:**
- **Widget:** Vanilla JS (`widget/spikes.js`, ~14KB gzipped, zero dependencies)
- **CLI:** Rust binary (`cli/`, v0.4.0, 30 subcommands)
- **MCP Server:** 9 tools via `spikes mcp serve` (stdio or HTTP transport)
- **Site:** Landing page and docs (`site/`, deployed to Cloudflare Pages)
- **Worker:** Cloudflare Workers backend (D1 + R2)
- **NPM:** `spikes-mcp` package for zero-install MCP

## Work Tracking with werk

**This project uses `werk` for all task/tension tracking.** Always check werk before starting work.

```bash
werk tree                    # See full tension tree
werk survey                  # Field survey — tensions by urgency
werk show <id>               # Tension details + history
werk list                    # List all active tensions
werk reality <id> "new..."   # Update current reality
werk note add <id> "text"    # Add observational note
werk resolve <id>            # Mark tension resolved
werk add "desired" "actual" --parent <id>  # Create child tension
```

The root tension is #1: "Have 10 paying users on spikes.sh". All work should relate to this goal or its children.

## Architecture

```
widget/
  spikes.js           # Drop-in widget (14KB gzipped, IIFE pattern)

cli/
  Cargo.toml
  src/
    main.rs           # 30 subcommands via clap
    commands/         # list, show, export, inject, serve, mcp, login, share, billing, etc.
    spike.rs          # Spike data structure
    storage.rs        # JSONL persistence (.spikes/feedback.jsonl)
    auth.rs           # Token/API key management
    config.rs         # TOML config (.spikes/config.toml)
    output.rs         # JSON/table formatting
    error.rs          # Error types
  templates/
    cloudflare/       # Self-host scaffold (Worker + D1 + R2)

site/
  index.html          # Landing page
  docs.html           # Documentation
  agents.md           # Machine-readable agent discovery
  llms.txt            # LLM context index
  spikes.js           # Widget served from spikes.sh

packages/
  spikes-mcp/         # NPM package for zero-install MCP (npx spikes-mcp)

action/               # GitHub Action for CI gating on feedback
```

## Key Patterns

### Widget (JavaScript)
- IIFE pattern, no global pollution except `window.Spikes` for config
- All styles inline (no external CSS)
- localStorage for persistence: `spikes:{project}` for data, `spikes:reviewer` for identity
- Configurable via data attributes: `data-project`, `data-position`, `data-color`, `data-endpoint`

### CLI (Rust)
- Use `clap` for argument parsing
- All commands support `--json` flag for machine-readable output
- Data file: `.spikes/feedback.jsonl` (one spike per line)
- Auth token stored in `~/.local/share/spikes/auth.toml` (0600 permissions)
- `SPIKES_TOKEN` env var overrides stored token
- `SPIKES_API_URL` env var overrides API base (default: https://spikes.sh/api)

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

## MCP Server

`spikes mcp serve` exposes 9 tools (feedback CRUD, shares, usage).
Transports: `stdio` (default), `http` (`--transport http --port 3848`).
Modes: `local` (default, reads JSONL), `remote` (`--remote`, uses API).

## Auth Flow

Current: email magic link (`spikes login` → email → click → CLI polls → token saved).
Target: browser-based device code flow (CLI opens browser → confirm → CLI auto-detects). See werk tension #13.

## Business Model

**Free (MIT licensed):**
- Full widget + CLI + MCP server
- Local workflow (inject + serve + collect + export)
- 5 shares, 1,000 spikes/share
- Self-hosting (one-command Cloudflare scaffold)

**Pro (Lifetime purchase via Stripe):**
- Unlimited shares & spikes
- Password-protected shares
- Badge removal
- Agent-tier consumption pricing

## Deployment

**Site:** Push to `main` → GitHub Actions → Cloudflare Pages (automatic)
**CLI:** Tag `v*` → GitHub Actions → cross-platform binaries (macOS Intel/ARM, Linux x64/ARM64)
**crates.io:** `cargo publish` from `cli/`
**npm:** Publish `packages/spikes-mcp/`

## Testing

```bash
cd cli && cargo test                    # CLI test suite (Rust)
cd ../spikes-hosted/worker && npm test  # Worker test suite (vitest)
```

**Note:** The Worker source code lives in the private `spikes-hosted` repo at `../spikes-hosted/worker/`. The `site/worker/` directory in this repo contains only this README pointer for deployment purposes.

## References

- Domain: spikes.sh
- Repo: github.com/bierlingm/spikes
- MCP registry: Smithery
</coding_guidelines>
