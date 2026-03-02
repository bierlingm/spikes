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
