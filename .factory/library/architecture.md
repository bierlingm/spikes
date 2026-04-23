# Architecture

Architectural decisions, patterns discovered, and design principles.

**What belongs here:** High-level architecture notes, resolved decisions, patterns that workers should follow.

---

## Resolved Decisions

| Decision | Resolution |
|----------|------------|
| Auth method | Magic links (not passwords) |
| Token storage | Per-user in `~/.config/spikes/auth.toml` + `SPIKES_TOKEN` env var |
| Local storage | Keep JSONL (not SQLite) |
| Widget communication | Both direct (hosted shares) and proxy (local dev) |
| Self-hosted template | Consolidate into spikes-hosted repo with shared core |
| Password hashing | PBKDF2-HMAC-SHA256 with 100k iterations |
| Rate limiting | D1-based sliding window |
| Validation | Zod schemas for all POST endpoints |
| Pagination | Cursor-based with `?cursor=` and `?limit=` |

## Hosted-First CLI (Mission 02 — v0.4.0)

The CLI treats `https://spikes.sh` as the default backend:

- Canonical hosted base URL: `https://spikes.sh` (NOT `api.spikes.sh` — that string is dead and has been purged from the repo).
- `Config::effective_endpoint()` returns `Some("https://spikes.sh")` when `remote.hosted == true`. Widget attributes then append `/spikes` so `data-endpoint` resolves to `https://spikes.sh/spikes`.
- `spikes init` defaults to hosted. The written `.spikes/config.toml` has `[remote] hosted = true` AND `endpoint = "https://spikes.sh"` (auditable decision). Self-host path uses `[remote]` without `hosted = true`.
- `.spikes/config.toml` uses the `[remote]` section (matches `Config` struct). The legacy `[sync]` section was replaced in v0.4.0 init output.
- `spikes inject` takes `--endpoint <url>`. Resolution order: `--endpoint` flag > `.spikes/config.toml` `[remote]` > nothing (widget runtime smart default applies). Path prefixes and query strings pass through verbatim.
- `spikes deploy cloudflare` prints a hosted-warning prompt when `[remote] hosted = true`, with `--force` to bypass. `--dir .` and `--dir <empty-path>` are now accepted. Help text contrasts `spikes share` (quick previews) vs `spikes deploy cloudflare` (data isolation / custom domain).
- `spikes config` requires `.spikes/` to exist (parity with `spikes list`). Error message: `No .spikes/ directory found. Run 'spikes init' first.`
- `spikes shares` exits 0 on empty success (previously exited 2).

## Widget Endpoint Resolution (Mission 01)

The widget resolves its POST endpoint in this priority order:
1. `data-endpoint` attribute (explicit) — always wins
2. `data-project` attribute set (explicit, non-empty) → `https://spikes.sh/spikes`
3. Neither set → `/spikes` on current origin (local dev fallback)

Key: `data-project` defaults to `location.hostname` when the attribute is absent — but this default does NOT trigger the spikes.sh endpoint. Only an explicitly-provided `data-project` attribute triggers it. The check must use `script.hasAttribute('data-project')` or equivalent to distinguish explicit from default.

## Widget Error Visibility (Mission 01)

POST failures are surfaced via:
- `console.error` (upgraded from `console.warn`) with endpoint URL + HTTP status
- Red dot indicator on widget button (child div, absolutely positioned top-right)
- Tooltip on button hover: "Last feedback failed to sync — see console"
- Error state resets on next successful POST

The "Saved!" toast fires synchronously before the XHR response — it is NOT gated on POST success.

## Two-Repo Structure

The CLI at `./cli/` talks to the Worker at `../spikes-hosted/worker/`. The widget at `./widget/` is served by both the CLI (locally) and the Worker (hosted). Changes to API contracts must be reflected in both repos.

## Worker Modularization Target

Split `index.ts` (~824 lines) into:
- `src/index.ts` — Router, < 200 lines
- `src/handlers/spike.ts` — Spike CRUD
- `src/handlers/share.ts` — Share management
- `src/handlers/auth.ts` — Authentication
- `src/handlers/stripe.ts` — Stripe webhooks
- `src/middleware/rate-limit.ts` — Rate limiting
- `src/middleware/validation.ts` — Zod schemas
- `src/db/queries.ts` — D1 query helpers
- `src/utils/crypto.ts` — Password hashing, HMAC

## Reliability Scrutiny Notes (2026-03-03)

- D1 foreign key violation behavior can vary by local runtime details; tests should assert constraint failure using message/code pattern matching rather than one exact error string.
- For Worker IP-based rate limiting, prioritize `CF-Connecting-IP`, then `X-Forwarded-For`, then local fallback for development.
- In the CLI (`ureq`), HTTP status often appears inside transport errors as `status code NNN`; parsing this pattern enables actionable error messaging even when no structured body is available.
- For Worker background tasks (e.g., webhooks), pass `ExecutionContext` through handlers and use `ctx.waitUntil(...)` for delivery/retry work that must survive response completion.

## Agent Readiness: Key Patterns

| Decision | Resolution |
|----------|------------|
| MCP data source | DataSource enum (Local/Remote) — Local reads JSONL, Remote uses ureq HTTP |
| MCP transport | stdio (default) or HTTP (--transport http, via axum) |
| API key format | sk_spikes_ prefix, PBKDF2 hash stored, raw key returned once |
| API key auth | Auth middleware checks sk_spikes_ prefix, hashes and looks up in api_keys table |
| Agent billing | Stripe Meters API — meter events for spike/share consumption, fire-and-forget via ctx.waitUntil() |
| Budget enforcement | Check cost vs monthly_cap_cents BEFORE persisting, 429 BUDGET_EXCEEDED if over |
| Agent tier detection | Webhook inspects price ID (STRIPE_AGENT_PRICE_ID vs STRIPE_PRO_PRICE_ID) |

## Phase 5: Growth Integrations (Mission 2)

| Decision | Resolution |
|----------|------------|
| MCP SDK | rmcp v0.17 (official Rust SDK) — replaces hand-rolled JSON-RPC |
| MCP transport | stdio (standard for CLI-based MCP servers) |
| Context export | Extend existing export command with cursor-context and claude-context formats |
| GitHub Action | Composite action in action/ dir, downloads pre-built binary from releases |
| Action distribution | Download from GitHub releases (not Docker, not npm) |

## Monetization Scrutiny Notes (2026-03-03)

- `GET /spikes` intentionally allows unauthenticated access only when a `project` query param is provided; otherwise auth is still required.
- Stripe webhook handlers should return `200 { received: true }` for events missing `customer` IDs (safe no-op behavior).
- Downgrade flows use a two-step pattern: update user tier, then clear Pro-only share fields (`password_hash`, `password_salt`, `webhook_url`, `webhook_secret`).
- Stripe idempotency uses `stripe_events`: check for existing event before processing and record the event after successful handling.
- Upgrade CTAs are currently standardized to `https://spikes.sh/pro` across Worker limit responses and CLI messaging.

## MCP Completeness Scrutiny Notes (2026-03-10)

- Hosted Worker `GET /spikes` returns paginated payload shape `{ data: Spike[], next_cursor?: string | null }` (not a raw array and not `{ spikes: [...] }`).
- Worker router currently exposes `GET /spikes/:id` but does **not** expose `PATCH /spikes/:id` or `DELETE /spikes/:id`; remote MCP mutation tools must align to available API routes.
- Worker spike schema requires at least one of `project` or `projectKey` for spike creation; remote MCP submit logic must always provide one.
- rmcp Streamable HTTP testing is sensitive to protocol details (e.g., proper `Accept` header and `notifications/initialized` flow). Preserve these in integration test fixtures to avoid false negatives.
- For streamable HTTP integration tests, propagate `Mcp-Session-Id` from the `initialize` response onto subsequent `notifications/initialized` and tool calls; otherwise rmcp treats later requests as uninitialized sessions.
- In this test setup, prefer reading HTTP MCP responses via `reqwest` `bytes()` (then UTF-8 decode) instead of `text()` to avoid SSE/body-read edge behavior during sequential protocol requests.
