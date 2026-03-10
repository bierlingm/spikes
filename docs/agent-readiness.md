# Spikes Agent Readiness Plan

**Date:** 2026-03-10
**Status:** Strategic plan — no code changes in this document
**Context:** Audit against Aaron Levie's "Building for Trillions of Agents" thesis and @code_rams's distillation

---

## Source Material

**@code_rams (Ramya Chinnadurai)** distilled Levie's article into three imperatives:

1. **Add an API for every feature.** If it's not in the API, it doesn't exist for agents.
2. **Let agents sign up without human steps.** Account creation via API is table stakes.
3. **Move pricing to consumption-based.** Seat pricing breaks when agents do 10x the work.

**Aaron Levie's full thesis** ("Building for trillions of agents", March 2026):

- Agents are no longer chatbots — they have sandboxed compute, file systems, long-term memory, and can write/run code.
- Agents will become the primary user of all software.
- "Make something agents want" — the successor to "make something people want."
- Everything must become API-first. If you don't have an API for a feature, it might as well not exist. If it can't be exposed through a CLI or MCP server, you're at a disadvantage.
- "If an agent can't easily sign up for your service and start using it, you're basically dead to agents."
- Agents will need their own identities, budgets, wallets, and the ability to communicate with others.
- Security, compliance, and governance become major problems — agents accessing sensitive data need strict controls.
- Consumption or volume-based business models are required for survival in an agentic future.

---

## Part 1: Audit

### Criterion 1 — API for Every Feature

| Surface | Current State | Gap |
|---------|--------------|-----|
| **CLI** | `--json` on every command. Fully scriptable. | None — this is solid. |
| **MCP Server** | 3 read-only tools: `get_spikes`, `get_element_feedback`, `get_hotspots`. Stdio transport via rmcp SDK. | No write tools. Agent can't submit spikes, resolve feedback, create shares, or manage anything through MCP. |
| **REST API (Hosted)** | Documented endpoints for spikes, shares, auth, billing, usage. Paginated. Standardized error codes. | API exists but MCP doesn't surface it. No remote MCP mode. |
| **Site** | `llms.txt` and `llms-full.txt` for LLM context windows. JSON-LD schema markup. | No MCP tool descriptions or agent quickstart in `llms.txt`. |
| **Context Exports** | `spikes export --format cursor-context` and `--format claude-context`. | Read-only. No way for agent to push context back. |

**Verdict: Partial pass.** The read side is strong. The write side is incomplete — the MCP server is read-only, and agents can't create or manage feedback programmatically through MCP.

### Criterion 2 — Agent Self-Signup

| Surface | Current State | Gap |
|---------|--------------|-----|
| **Auth flow** | Magic link: `POST /auth/login` sends email → human clicks → `POST /auth/verify` returns bearer token. | Human required to click magic link. No programmatic account creation. |
| **CLI** | `spikes login` triggers magic link flow. | Same human bottleneck. |
| **API keys** | Don't exist. | No service account concept. No agent identity. No way to create credentials without email verification. |
| **Token management** | Bearer tokens in `~/.config/spikes/auth.toml` or `SPIKES_TOKEN` env var. | Tokens exist but can't be created without human auth. |

**Verdict: Fail.** An agent cannot sign up or authenticate without a human clicking a magic link. This is the biggest gap.

### Criterion 3 — Consumption-Based Pricing

| Surface | Current State | Gap |
|---------|--------------|-----|
| **Limits** | Free: 5 shares, 1000 spikes/share. Pro: unlimited. | Consumption-aware (limits exist) but not consumption-priced. |
| **Pricing model** | Pay-what-you-feel ($9–$29+). One-time agency license ($149). | Human-oriented. No metered billing, no per-spike pricing, no agent tier. |
| **Usage tracking** | `GET /usage` returns counts and limits. `spikes usage` CLI command. | Exists but no cost/spend visibility. No budget controls. |
| **Billing API** | `GET /billing/checkout`, `GET /billing/portal`. | Can create checkout sessions but no metered/usage-based plans. |

**Verdict: Partial pass.** The infrastructure for tracking consumption exists. The pricing model doesn't yet accommodate agent-scale volume.

### Deeper Audit Against Levie's Full Article

| Levie Point | Current State | Gap |
|---|---|---|
| "Everything must become API-first" | CLI is API-first. Hosted API is documented. | MCP is read-only. No write tools. |
| "Expose through CLI or MCP server" | Both exist. | MCP only reads local JSONL. Can't talk to hosted backend. No remote MCP mode. |
| "Confusing APIs and conflicting paths" | Some inconsistencies: `project_key` (Rust) vs `project` (TS), camelCase/snake_case mix. Query param auth still exists alongside Bearer. | Needs schema unification. |
| "Agents need identities" | Bearer tokens exist but tied to human email accounts. | No service account / agent identity concept. |
| "Agents need to manage budgets" | Stripe integration for human billing. | No programmatic billing. Agent can't check spend or upgrade via API autonomously. |
| "Security, compliance, governance" | Critical security issues documented (unsalted passwords, path traversal, XSS). Phase 0 planned but not yet executed. | Must complete security foundation before agents can safely use the platform. |
| "Agents will need infrastructure" | Cloudflare Workers + D1 + R2. Self-host option. | Good fit. Cloudflare is explicitly named by Levie as agent infra. |
| "Agents need to communicate" | Webhook support (Pro feature). | Webhooks are fire-and-forget. No agent-to-agent messaging. |

---

## Part 2: The Plan

### Execution Order

```
Phase D (Security Foundation)   ████████████████████  2 weeks  ← MUST DO FIRST
Phase A (Agent Write Path)      ████████████████████  1 week
Phase B (Agent Identity & Auth) ████████████████████  1 week
Phase C (Agent-Native Pricing)  ████████████         3 days
Phase E (Agent Discovery)       ████████████         3 days
                                ──────────────────────────────
                                Total: ~5 weeks
```

---

### Phase D: Security Foundation (Prerequisites)

**Goal:** Nothing ships on an insecure foundation.

This is the existing OVERHAUL.md Phase 0. It must complete before any agent-readiness work.

| Task | Detail |
|------|--------|
| Password hashing | Replace SHA-256 with PBKDF2-HMAC-SHA256, 100k iterations, per-share random salt |
| Path traversal | Sanitize R2 filenames. Fix CLI `serve` with `canonicalize()` |
| Rate limiting | D1-based sliding window: 60/min POST /spikes, 5/min password attempts, 10/min POST /shares |
| XSS | HTML-escape all values in subdomain listing |
| CORS | Safe defaults in CLI serve |
| Test infrastructure | Vitest + Miniflare for Worker. Rust unit + integration tests for CLI |
| CI/CD | Staging on push, production on release tag. Test gate required |

**Success criteria:**
- All critical security vulnerabilities resolved
- CI pipeline runs tests on every PR
- Staging environment auto-deploys

**Key files:**
- `spikes-hosted/worker/src/index.ts` — password hashing, R2 upload
- `spikes-hosted/worker/schema.sql` — add salt column
- `spikes/cli/src/commands/serve.rs` — path traversal fix
- `spikes-hosted/.github/workflows/deploy.yml` — CI/CD

---

### Phase A: Agent Write Path (MCP + API)

**Goal:** Agents can do everything through MCP, not just read.

#### A1. Expand MCP Server with Write Tools

Add to `cli/src/commands/mcp.rs`:

| New Tool | Purpose | Parameters |
|----------|---------|------------|
| `submit_spike` | Create feedback programmatically | `page`, `selector?`, `rating`, `comments`, `reviewer_name` |
| `resolve_spike` | Mark feedback as addressed | `spike_id` |
| `delete_spike` | Remove feedback | `spike_id` |
| `create_share` | Upload files, get shareable URL | `directory`, `slug?`, `title?` |
| `list_shares` | See active shares | (none) |
| `get_usage` | Check current limits and consumption | (none) |

An agent reviewing its own work can now: build a page → submit self-review spikes → iterate → resolve them. An agent orchestrating human review can: create a share → wait for spikes → query feedback → act on it → resolve.

#### A2. Remote MCP Mode

```bash
spikes mcp serve --remote    # Uses hosted backend instead of local JSONL
```

- Reads bearer token from `~/.config/spikes/auth.toml` or `SPIKES_TOKEN` env var
- Same tool interface, different data source (REST API calls instead of local file reads)
- Enables cloud-based agent workflows where JSONL isn't available

#### A3. Streamable HTTP Transport

```bash
spikes mcp serve --transport http --port 3848
```

- In addition to existing stdio transport
- Enables agents in sandboxed environments (Devin, Codex, OpenClaw) to reach Spikes over HTTP
- No stdio piping required — just an HTTP endpoint
- Supports the MCP Streamable HTTP transport spec

**Success criteria:**
- Agent can submit, query, resolve, and delete spikes entirely through MCP
- Remote mode works against hosted backend
- HTTP transport works for sandboxed agents

---

### Phase B: Agent Identity & Programmatic Auth

**Goal:** An agent can sign up and start using Spikes without any human intervention.

#### B1. API Key Creation Endpoint

```
POST /auth/api-key
Content-Type: application/json

{
  "name": "my-coding-agent"     // optional label
}

→ 201 Created
{
  "ok": true,
  "api_key": "sk_spikes_...",   // long-lived, store securely
  "key_id": "key_...",          // for management/revocation
  "name": "my-coding-agent"
}
```

- No email required. No magic link. No human step.
- API keys are first-class identities, not tied to user accounts.
- Optional: link to a user account later via `POST /auth/link-key` for billing consolidation.

#### B2. Agent Self-Signup via CLI

```bash
spikes auth create-key                          # Generate API key, store locally
spikes auth create-key --name "my-coding-agent" # Labeled key for tracking
spikes auth list-keys                           # See all keys
spikes auth revoke-key key_...                  # Revoke a key
```

No email. No magic link. No human step. An agent running `spikes auth create-key` gets working credentials in one call.

#### B3. Agent Identity in the System

- API keys get their own usage tracking (separate from user accounts)
- `GET /me` works with API keys: returns key metadata, usage stats, tier
- Keys appear in `spikes shares` output so humans can see what agents created
- Keys can be scoped: read-only, write-only, or full access
- Keys can have expiry dates for security

**Success criteria:**
- Agent can create credentials and start using Spikes in a single API call
- No human intervention required at any point
- Keys are trackable and revocable

---

### Phase C: Agent-Native Pricing

**Goal:** Pricing that works when an agent does 100x the volume of a human.

#### C1. Agent Tier

| Tier | Spikes | Shares | Price |
|------|--------|--------|-------|
| Free | 1,000/month | 5 | $0 |
| Agent | Metered | Metered | $0.001/spike, $0.01/share |
| Pro | Unlimited | Unlimited | Pay-what-you-feel ($9–$29+/mo) |

The agent tier is purely consumption-based. No seat concept. An API key on the agent tier pays for exactly what it uses.

#### C2. Programmatic Billing

```
GET /billing/checkout?tier=agent
→ { "url": "https://checkout.stripe.com/..." }

GET /usage
→ {
    "spikes_this_period": 4521,
    "shares_this_period": 12,
    "cost_this_period_cents": 572,
    "monthly_cap_cents": 10000,
    "tier": "agent",
    "period_ends": "2026-04-01T00:00:00Z"
  }
```

Agent can check its own spend before deciding to continue work.

#### C3. Budget Controls

API keys can have spending caps:

```
POST /auth/api-key
{
  "name": "budget-agent",
  "monthly_cap_cents": 1000    // $10/month cap
}
```

- Returns `429 BUDGET_EXCEEDED` when cap hit
- Agent or orchestrator can raise cap via `PATCH /auth/api-key/:key_id`
- Prevents runaway agent spend

**Success criteria:**
- Agent tier available with metered billing
- Usage and cost visible via API
- Budget controls prevent runaway spend

---

### Phase E: Agent Discovery & Onboarding

**Goal:** An agent encountering Spikes for the first time can figure out what it does and start using it.

#### E1. Enhanced llms.txt

Add to `site/llms.txt`:
- MCP tool descriptions (all tools, parameters, return types)
- API key creation instructions
- Agent quickstart: install → create key → submit first spike → query feedback
- Example MCP config blocks for Claude Code and Cursor

#### E2. NPM/pip Wrapper for MCP

```bash
npx spikes-mcp          # Downloads binary, starts MCP server
pip install spikes-mcp   # Same for Python agents
```

Zero-config for agents that can run npm or pip. Removes the "install Rust binary" barrier.

#### E3. MCP Server Registry Listing

- Publish to Smithery, Glama, and other MCP server registries
- `spikes mcp install` generates the config block for Claude Code / Cursor / other MCP clients
- Include in package metadata: tool descriptions, auth requirements, example usage

#### E4. Agent-Oriented Landing Page

A machine-readable page at `spikes.sh/agents` (or in llms.txt) that answers the questions an agent would ask:
- What does this tool do?
- How do I authenticate?
- What MCP tools are available?
- What are the rate limits?
- How much does it cost?

**Success criteria:**
- An agent can discover, install, authenticate, and use Spikes without human guidance
- Listed in major MCP registries
- `npx spikes-mcp` works

---

## Part 3: What Makes This Defensible

Levie's key insight: *"Agents won't be going to your webinar or seeing your ad; they're just going to use the best tool for the job."*

Spikes' agent moat is not the widget (trivially cloneable) or the CLI (replaceable). It's:

1. **The only feedback tool with an MCP server.** Agents can query and act on feedback natively — no screen scraping, no API wrapping, no adapter layer.

2. **Structured selectors.** No other tool gives agents `button.submit` instead of "that button over there." CSS selectors are machine-actionable. Screenshots with arrows are not.

3. **Zero-friction programmatic access.** API key in one call, no email, no human step. An agent can go from "I need a feedback tool" to "I'm using one" in seconds.

4. **Works in any agent sandbox.** Local JSONL for simple agents. Stdio MCP for Claude Code/Cursor. HTTP MCP for sandboxed environments (Devin, Codex, OpenClaw). Hosted REST API for everything else. The agent picks its transport.

5. **Consumption-native pricing.** An agent doing 10,000 reviews pays $10. No seat negotiation, no enterprise sales call, no "contact us for pricing."

The competitive positioning shifts from "feedback tool for developers" to "feedback infrastructure for agents." Every other tool in the space (Markup.io, BugHerd, Userback, Hotjar) is built for humans clicking through browsers. Spikes is built for agents operating through APIs.

---

## Part 4: Relationship to OVERHAUL.md

This plan does **not** replace the existing overhaul plan. It extends it with agent-specific phases that slot in after the foundation work:

| OVERHAUL.md Phase | Agent Readiness Phase | Relationship |
|---|---|---|
| Phase 0: Security Foundation | **Phase D** | Same work. Must complete first. |
| Phase 1: Core Reliability | — | Proceed as planned. Good for agents too (pagination, validation, error codes). |
| Phase 2: Identity & Auth | **Phase B** | Extends Phase 2 with API key concept alongside magic links. |
| Phase 3: UX & Polish | — | Human-oriented. Proceed as planned. |
| Phase 4: Monetization | **Phase C** | Extends Phase 4 with agent tier and metered billing. |
| Phase 5: Growth Features | **Phase A, E** | MCP write tools and discovery are agent-specific growth features. |

The combined timeline is approximately 10 weeks: 8 weeks from the original overhaul + 2 weeks of net-new agent work (the rest overlaps).

---

## Summary

| Gap | Fix | Phase | Effort |
|-----|-----|-------|--------|
| MCP is read-only | Add 6 write tools, remote mode, HTTP transport | A | 1 week |
| Agent can't sign up | API key endpoint, no email required | B | 1 week |
| Pricing is human-oriented | Agent tier with metered billing, budget controls | C | 3 days |
| Security foundation incomplete | Complete OVERHAUL.md Phase 0 | D | 2 weeks |
| Agents can't discover Spikes | Enhanced llms.txt, npm wrapper, registry listings | E | 3 days |

**Start with Phase D. Nothing else ships until security is solid. Then A → B → C → E.**

The goal: an agent encountering Spikes for the first time can discover it, sign up, authenticate, submit feedback, query results, and pay for usage — all without a human touching anything.
