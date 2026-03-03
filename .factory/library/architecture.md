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

## Monetization Scrutiny Notes (2026-03-03)

- `GET /spikes` intentionally allows unauthenticated access only when a `project` query param is provided; otherwise auth is still required.
- Stripe webhook handlers should return `200 { received: true }` for events missing `customer` IDs (safe no-op behavior).
- Downgrade flows use a two-step pattern: update user tier, then clear Pro-only share fields (`password_hash`, `password_salt`, `webhook_url`, `webhook_secret`).
- Stripe idempotency uses `stripe_events`: check for existing event before processing and record the event after successful handling.
- Upgrade CTAs are currently standardized to `https://spikes.sh/pro` across Worker limit responses and CLI messaging.
