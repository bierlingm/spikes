# Mission Plan: Agent Readiness

**Date:** 2026-03-10
**Source:** `docs/agent-readiness.md`
**Status:** Ready for review
**Timeline:** ~5 weeks (some missions can parallelize)

---

## Current State (Verified Against Codebase + Git History)

The OVERHAUL.md plan is fully implemented across both repos. All work was committed directly to `main` (no feature branches, no PRs). Current CLI version is **v0.3.1**.

**Housekeeping debt:** All 10 `spikes` issues and all 16 `spikes-hosted` issues remain OPEN on GitHub despite being resolved in code. These should be closed before starting agent-readiness work.

### Git Activity Summary

**spikes repo** (30 commits): test infra -> context exports -> MCP rewrite (rmcp 0.17) -> GitHub Action -> CI workflow -> docs -> clippy fixes -> v0.3.1

**spikes-hosted repo** (30 commits): initial setup -> Sentry -> modularize index.ts -> Vitest+Miniflare -> PBKDF2+XSS+R2 security -> CI/CD staging -> data integrity (FKs, server-side IDs) -> Zod validation -> pagination -> rate limiting -> webhooks (HMAC+retry) -> caching+R2 cleanup -> magic link auth -> Stripe webhooks -> Pro enforcement -> billing portal -> usage endpoint -> cross-area integration tests

**Both repos:** single `main` branch, no PRs, no feature branches.

### What's Implemented

**Hosted Worker (spikes-hosted/worker):**
- Modularized: `handlers/auth.ts`, `handlers/spikes.ts`, `handlers/shares.ts`, `handlers/billing.ts`, `handlers/webhooks.ts`, `handlers/usage.ts`
- Zod validation (`src/schema.ts`)
- D1 schema with FK constraints, rate limiting table, user tokens table, stripe events idempotency
- Magic link auth: `/auth/login`, `/auth/verify`, `/auth/rotate-token`, `/auth/recover`
- `/me` endpoint, `/usage` endpoint, `/billing/portal`, `/billing/checkout`
- Stripe webhook handling with idempotency
- PBKDF2 password hashing with salts
- Sentry integration

**CLI (spikes/cli):**
- All commands: init, list, show, export, hotspots, reviewers, inject, serve, pull, push, sync, delete, resolve, share, shares, unshare, login, logout, whoami, billing, upgrade, usage, mcp serve, deploy, config, remote, update, magic, version
- Export formats: json, csv, jsonl, cursor-context, claude-context
- `serve` uses `canonicalize()` (path traversal fixed), has `--cors-allow-origin` flag
- Magic link login flow with polling
- Storage: `load_spikes()`, `save_spikes()`, `find_spike_by_id()`, `remove_spike()`, `update_spike()`
- MCP server: 3 read-only tools (`get_spikes`, `get_element_feedback`, `get_hotspots`) via rmcp SDK, stdio transport only
- GitHub Action: `action/action.yml` (feedback gate, downloads binary, composite action)

**Issues resolved in code but still open on GitHub (close these first):**
- spikes: #1 (token plaintext — fixed, now in auth.toml), #2 (path traversal — fixed, canonicalize + CORS flag), #3 (no tests — fixed, CI added), #6 (no edit/delete — fixed, delete + resolve commands), #8 (pull/push errors — fixed, paginated + error mapping)
- spikes: #5 (TUI unimplemented — decision: remove, don't build), #7 (review mode — partially addressed with `--marked` flag), #9 (docs incomplete — docs added), #10 (widget z-index — offset attrs exist), #4 (widget UX — partial)
- spikes-hosted: #1-#16 — all resolved in code (PBKDF2, rate limiting, pagination, FKs, validation, XSS, path traversal, cache headers, R2 cleanup, bearer token system, Stripe webhooks, Pro enforcement, tests, CI/CD)

**What does NOT exist yet (the agent-readiness gaps):**
- No MCP write tools (submit, resolve, delete spikes; create share; list shares; get usage)
- No remote MCP mode (all MCP tools read local JSONL only)
- No HTTP/Streamable HTTP MCP transport (stdio only)
- No API key system (no `api_keys` table, no `POST /auth/api-key`, no `spikes auth create-key`)
- No agent/metered pricing tier (free + pro only, no consumption billing)
- No agent discovery (llms.txt has no MCP info, no npm wrapper, no registry listings)

---

## Mission 0: Housekeeping — Close Resolved Issues

**Objective:** Close all GitHub issues that are already resolved in code. Clean baseline before new work.

**Duration:** 1 hour

### What to Do

**spikes repo** — close with commit references:
- #1: Token plaintext — resolved by auth.toml migration (commit `e78a07d` era)
- #2: Path traversal + CORS — resolved by `canonicalize()` + `--cors-allow-origin` in serve.rs
- #3: No tests — resolved by CI workflow (`c8e2571`)
- #5: TUI unimplemented — close as won't-fix per OVERHAUL decision (remove, don't build)
- #6: No edit/delete — resolved by `delete` and `resolve` commands
- #8: Pull/push errors — resolved by paginated API handling (`7094323`)
- #9: Docs incomplete — resolved by docs commits (`fd79901`)
- #4, #7, #10: Close or leave open with notes on remaining work

**spikes-hosted repo** — close all 16:
- #1 (PBKDF2), #2 (rate limiting), #3 (pagination), #4 (server-side IDs), #5 (FKs), #6 (webhooks), #7 (validation), #8 (R2 sanitization), #9 (Pro enforcement), #10 (tests), #11 (Stripe), #12 (bearer tokens), #13 (cache headers), #14 (XSS), #15 (R2 cleanup), #16 (staging/CI)

### Success Criteria

- [ ] All resolved issues closed with brief commit references
- [ ] Any genuinely remaining work noted in issue comments before closing, or left open

---

## Mission 1: MCP Write Tools (Phase A1)

**Objective:** Expand MCP server from read-only to full CRUD. An agent can submit, resolve, and delete spikes entirely through MCP.

**Duration:** 3 days

### What to Build

Add 6 new tools to `cli/src/commands/mcp.rs`:

| Tool | Purpose | Parameters |
|---|---|---|
| `submit_spike` | Create feedback programmatically | `page`, `selector?`, `rating`, `comments`, `reviewer_name` |
| `resolve_spike` | Mark feedback as addressed | `spike_id` |
| `delete_spike` | Remove feedback | `spike_id` |
| `create_share` | Upload files, get shareable URL | `directory`, `slug?`, `title?` |
| `list_shares` | See active shares | (none) |
| `get_usage` | Check limits and consumption | (none) |

### Implementation Notes

- `submit_spike`: construct a `Spike`, append to `.spikes/feedback.jsonl` via `load_spikes()` + push + `save_spikes()` (or add a new `append_spike()` helper)
- `resolve_spike`: use `storage::update_spike()` (same logic as `commands/resolve.rs`)
- `delete_spike`: use `storage::remove_spike()` (same logic as `commands/delete.rs`)
- `create_share`: reuse upload logic from `commands/share.rs` (requires bearer token from `auth::AuthConfig`)
- `list_shares` / `get_usage`: HTTP calls to hosted API using `ureq` + bearer token (same pattern as `commands/shares.rs`, `commands/usage.rs`)
- New arg structs: `SubmitSpikeArgs`, `ResolveSpikeArgs`, `DeleteSpikeArgs`, `CreateShareArgs`, `ListSharesArgs`, `GetUsageArgs`
- Generate spike IDs with timestamp + random suffix (match widget pattern)

### Success Criteria

- [ ] Agent can submit a spike via MCP and it appears in `spikes list`
- [ ] Agent can resolve/unresolve a spike via MCP
- [ ] Agent can delete a spike via MCP
- [ ] `create_share` uploads and returns a URL (requires auth)
- [ ] `list_shares` returns share metadata (requires auth)
- [ ] `get_usage` returns spike/share counts and limits (requires auth)
- [ ] All 3 existing read tools still work
- [ ] Unit tests for each new tool's mutation logic
- [ ] `cargo test` passes, `cargo clippy` clean

### Key Files to Modify

- `cli/src/commands/mcp.rs` — add tools + arg structs to `SpikesService`
- `cli/src/storage.rs` — may need new helpers or reuse existing ones

### Key Files to Reference

- `cli/src/commands/resolve.rs` — resolve logic pattern
- `cli/src/commands/delete.rs` — delete logic pattern
- `cli/src/commands/share.rs` — share upload logic
- `cli/src/commands/shares.rs` — list shares API call pattern
- `cli/src/commands/usage.rs` — usage API call pattern
- `cli/src/auth.rs` — token loading

---

## Mission 2: Remote MCP Mode (Phase A2)

**Objective:** MCP server can talk to the hosted backend instead of only reading local JSONL.

**Duration:** 2 days

### What to Build

```bash
spikes mcp serve --remote    # Uses hosted API instead of local JSONL
```

- Add `--remote` flag to `McpAction::Serve` in `main.rs`
- When `--remote`, tools make HTTP calls to the hosted API instead of reading `.spikes/feedback.jsonl`
- Reads bearer token from `~/.config/spikes/auth.toml` (via `auth::AuthConfig`) or `SPIKES_TOKEN` env var
- Same tool interface, different data source

### Implementation Notes

- Introduce a `DataSource` enum (`Local` / `Remote { token, base_url }`) passed to `SpikesService::new()`
- `Local` uses `storage::load_spikes()` (current behavior)
- `Remote` uses `ureq` HTTP calls to `GET /spikes`, `POST /spikes`, `DELETE /shares/:id`, etc.
- Base URL defaults to `https://spikes.sh` (same as `auth::get_api_base()`)
- All 9 tools (3 read + 6 write from Mission 1) work in both modes

### Success Criteria

- [ ] `spikes mcp serve --remote` starts and serves all tools over stdio
- [ ] Read tools return data from hosted API
- [ ] Write tools (from Mission 1) operate against hosted API
- [ ] Token resolution: `SPIKES_TOKEN` env var > `auth.toml` > clear error message
- [ ] Graceful error when API is unreachable or token missing
- [ ] Default (no `--remote`) still reads local JSONL
- [ ] `cargo test` passes

### Key Files

- `cli/src/commands/mcp.rs` — `DataSource` abstraction, conditional tool logic
- `cli/src/main.rs` — `--remote` flag on `McpAction::Serve`
- `cli/src/auth.rs` — token loading (already exists)

---

## Mission 3: Streamable HTTP Transport (Phase A3)

**Objective:** MCP server can serve over HTTP, enabling sandboxed agents (Devin, Codex, OpenClaw) to connect without stdio.

**Duration:** 2 days

### What to Build

```bash
spikes mcp serve --transport http --port 3848
spikes mcp serve --transport http --port 3848 --remote   # HTTP + remote backend
```

- Add `--transport` flag: `stdio` (default) or `http`
- Add `--port` flag (default 3848, only used with HTTP transport)
- HTTP transport implements MCP Streamable HTTP spec
- Bind to `127.0.0.1` by default

### Implementation Notes

- Check `rmcp` crate features: may support `transport-sse-server` or HTTP server natively
- If rmcp doesn't have built-in HTTP transport, wrap with axum (already a dependency) to bridge JSON-RPC over HTTP
- Composes with `--remote`: transport and data source are orthogonal
- Add `--bind` flag for explicit bind address (default `127.0.0.1`)

### Success Criteria

- [ ] `spikes mcp serve --transport http` starts an HTTP server on port 3848
- [ ] MCP client can connect over HTTP and call all tools
- [ ] Stdio remains the default transport
- [ ] `--remote` composes with `--transport http`
- [ ] Binds to localhost by default (security)
- [ ] `cargo test` passes

### Key Files

- `cli/src/commands/mcp.rs` — transport selection, HTTP server setup
- `cli/src/main.rs` — `--transport` and `--port` flags
- `cli/Cargo.toml` — may need additional `rmcp` features

---

## Mission 4: API Key System (Phase B)

**Objective:** Agents can create credentials and start using Spikes without any human intervention.

**Duration:** 3 days

### What to Build

#### Backend (spikes-hosted/worker)

New D1 table:
```sql
CREATE TABLE IF NOT EXISTS api_keys (
  id TEXT PRIMARY KEY,              -- "key_..."
  key_hash TEXT NOT NULL,           -- PBKDF2 hash of "sk_spikes_..."
  key_prefix TEXT NOT NULL,         -- First 8 chars for display
  name TEXT,
  user_id TEXT,                     -- NULL if standalone, FK to users if linked
  scopes TEXT DEFAULT 'full',       -- 'read', 'write', 'full'
  monthly_cap_cents INTEGER,        -- NULL = no cap
  expires_at TEXT,                  -- NULL = no expiry
  created_at TEXT NOT NULL,
  last_used_at TEXT,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);
```

New endpoints:
```
POST   /auth/api-key          → Create API key (UNAUTHENTICATED — agent self-signup)
GET    /auth/api-keys          → List keys (requires bearer token or API key)
DELETE /auth/api-key/:key_id   → Revoke key (requires bearer token or API key)
POST   /auth/link-key          → Link API key to user account (requires bearer token)
```

Auth middleware change: `getBearerToken()` must also accept `sk_spikes_` prefixed tokens, hash and look up in `api_keys` table.

Rate limit key creation: 10/hour per IP (reuse existing rate limiting infrastructure).

#### CLI

```bash
spikes auth create-key                          # Generate API key, store locally
spikes auth create-key --name "my-coding-agent" # Labeled
spikes auth list-keys                           # List all keys
spikes auth revoke-key key_...                  # Revoke
```

### Implementation Notes

- `sk_spikes_` prefix for identification (like Stripe's `sk_` convention)
- Store PBKDF2 hash only — raw key shown once at creation, never retrievable
- `POST /auth/api-key` is intentionally unauthenticated (the whole point is agent self-signup)
- Keys can optionally link to user accounts later for billing consolidation
- Update `GET /me` to return key metadata when called with an API key
- CLI stores API key in `~/.config/spikes/auth.toml` alongside bearer token

### Success Criteria

- [ ] `POST /auth/api-key` creates a key with no auth required
- [ ] API key works as bearer token for all existing API endpoints
- [ ] `GET /me` returns key metadata when called with API key
- [ ] `spikes auth create-key` works and stores key locally
- [ ] Keys can be listed, revoked via API and CLI
- [ ] Key creation rate-limited (10/hour per IP)
- [ ] Scopes enforced (read/write/full)
- [ ] Worker tests pass, CLI tests pass

### Key Files

- `spikes-hosted/worker/schema.sql` — `api_keys` table
- `spikes-hosted/worker/src/handlers/auth.ts` — new endpoints
- `spikes-hosted/worker/src/middleware/auth.ts` — support API key auth
- `spikes-hosted/worker/src/index.ts` — register new routes
- `cli/src/main.rs` — new `Auth` subcommand
- New `cli/src/commands/auth_keys.rs` — key management CLI

---

## Mission 5: Agent-Native Pricing (Phase C)

**Objective:** Consumption-based pricing that works when an agent does 100x human volume.

**Duration:** 3 days
**Depends on:** Mission 4 (needs `api_keys` table and budget controls)

### What to Build

#### Agent Tier

| Tier | Spikes | Shares | Price |
|---|---|---|---|
| Free | 1,000/month | 5 | $0 |
| Agent | Metered | Metered | $0.001/spike, $0.01/share |
| Pro | Unlimited | Unlimited | Pay-what-you-feel ($9-$29+/mo) |

#### Stripe Metered Billing

- Create metered Stripe Price for agent tier
- Extend `GET /billing/checkout` to accept `?tier=agent`
- Report usage to Stripe via `invoice.created` webhook
- Track per-key usage in D1 (new `api_key_usage` table or extend existing)

#### Cost Visibility

Extend `GET /usage` response for agent tier:
```json
{
  "spikes_this_period": 4521,
  "shares_this_period": 12,
  "cost_this_period_cents": 572,
  "monthly_cap_cents": 10000,
  "tier": "agent",
  "period_ends": "2026-04-01T00:00:00Z"
}
```

#### Budget Controls

- `monthly_cap_cents` field on `api_keys` table (from Mission 4)
- Return `429 BUDGET_EXCEEDED` when cap hit
- `PATCH /auth/api-key/:key_id` to raise cap

### Success Criteria

- [ ] Agent tier available via `GET /billing/checkout?tier=agent`
- [ ] Metered usage reported to Stripe
- [ ] `GET /usage` includes cost info for agent tier
- [ ] Budget cap enforcement returns 429 when exceeded
- [ ] Budget cap adjustable via API
- [ ] Existing free/pro tiers unaffected
- [ ] Worker tests pass

### Key Files

- `spikes-hosted/worker/src/handlers/billing.ts` — metered checkout
- `spikes-hosted/worker/src/handlers/webhooks.ts` — metered usage reporting
- `spikes-hosted/worker/src/handlers/usage.ts` — cost visibility
- `spikes-hosted/worker/src/handlers/auth.ts` — budget controls
- `cli/src/commands/usage.rs` — display cost info

---

## Mission 6: Agent Discovery (Phase E)

**Objective:** An agent encountering Spikes for the first time can discover, install, authenticate, and use it without human guidance.

**Duration:** 3 days
**Can start after:** Mission 1 (llms.txt needs tool descriptions)

### What to Build

#### E1. Enhanced llms.txt

Add to `site/llms.txt`:
- All MCP tool descriptions (9 tools with parameters and return types)
- API key creation instructions (one-liner: `POST /auth/api-key`)
- Agent quickstart: install -> create key -> submit first spike -> query
- MCP config blocks for Claude Code and Cursor (already in `docs/mcp.md`, add to llms.txt)

#### E2. NPM Wrapper

Create `packages/spikes-mcp/` npm package:
```bash
npx spikes-mcp    # Downloads platform binary, starts MCP server
```
- Detect OS/arch, download binary from GitHub releases (same URLs as `action/action.yml`)
- Start `spikes mcp serve` with stdio
- `package.json` with `bin` entry pointing to wrapper script

#### E3. MCP Registry Listings

- `smithery.yaml` manifest in repo root
- `spikes mcp install` subcommand generates config block for Claude/Cursor
- Include tool descriptions, auth requirements, example usage in metadata

#### E4. Agent Landing Page

Machine-readable content at `site/agents.md` (or added to `llms.txt`) answering:
- What does this tool do?
- How do I authenticate? (API key: `POST /auth/api-key`, no email)
- What MCP tools are available? (full list)
- What are the rate limits?
- How much does it cost? (agent tier)

### Success Criteria

- [ ] `llms.txt` includes MCP tool descriptions and agent quickstart
- [ ] `npx spikes-mcp` downloads binary and starts MCP server
- [ ] Listed in at least one MCP registry (Smithery)
- [ ] `spikes mcp install` outputs config block for Claude/Cursor
- [ ] Agent-oriented content at stable URL

### Key Files

- `site/llms.txt` — enhanced content
- New `packages/spikes-mcp/` — npm wrapper package
- `site/agents.md` or `site/agents.html` — agent landing page
- `cli/src/main.rs` — `McpAction::Install` subcommand
- `smithery.yaml` — registry manifest

---

## Execution Order & Dependencies

```
Mission 0 (Housekeeping)           ██           1 hour  ← CLEAN SLATE
Mission 1 (MCP Write Tools)        ███████████  3 days  ← START HERE
Mission 2 (Remote MCP Mode)        ████████     2 days  ← depends on M1
Mission 3 (HTTP Transport)         ████████     2 days  ← parallel with M2
Mission 4 (API Key System)         ███████████  3 days  ← parallel with M2/M3
Mission 5 (Agent-Native Pricing)   ███████████  3 days  ← depends on M4
Mission 6 (Agent Discovery)        ███████████  3 days  ← depends on M1, full after M4
                                   ─────────────────────
                                   Total: ~5 weeks sequential, ~3.5 weeks with parallelism
```

### Critical Path

```
M1 → M2 ──→ (complete)
  ↘ M3 ──→ (complete)
M4 → M5 ──→ (complete)
M1 → M6 ──→ (complete, full content after M4)
```

### Parallelization Opportunities

| Parallel Track A (CLI/MCP) | Parallel Track B (Backend) |
|---|---|
| Mission 1: MCP Write Tools | — |
| Mission 2: Remote MCP Mode | Mission 4: API Key System |
| Mission 3: HTTP Transport | Mission 4: API Key System (cont.) |
| Mission 6: Discovery (partial) | Mission 5: Agent Pricing |
| Mission 6: Discovery (complete) | — |

---

## Repos Involved

| Mission | spikes (CLI) | spikes-hosted (Worker) |
|---|---|---|
| M1: MCP Write Tools | Primary | — |
| M2: Remote MCP Mode | Primary | — (consumes existing API) |
| M3: HTTP Transport | Primary | — |
| M4: API Key System | Secondary (CLI commands) | Primary (new endpoints + table) |
| M5: Agent Pricing | Secondary (usage display) | Primary (Stripe metered billing) |
| M6: Agent Discovery | Primary (mcp install, npm) | — |

---

## Defensive Moat After Completion

1. **Only feedback tool with full-CRUD MCP server** — read + write, not just read
2. **CSS selectors are machine-actionable** — `button.submit` instead of "that button over there"
3. **Zero-friction agent signup** — API key in one call, no email, no human step
4. **Works in any sandbox** — local JSONL, stdio MCP, HTTP MCP, REST API
5. **Consumption-native pricing** — 10,000 reviews = $10, no sales call

The positioning shift: "feedback tool for developers" -> "feedback infrastructure for agents."
