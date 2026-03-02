# Environment

Environment variables, external dependencies, and setup notes.

**What belongs here:** Required env vars, external API keys/services, dependency quirks, platform-specific notes.
**What does NOT belong here:** Service ports/commands (use `.factory/services.yaml`).

---

## Repos

- **spikes** (working directory): `/Users/moritzbierling/werk/repos/spikes` — Rust CLI + widget + site
- **spikes-hosted**: `/Users/moritzbierling/werk/repos/spikes-hosted` — CF Worker + D1 + R2

## Toolchain

- Rust 1.95.0-nightly, Cargo 1.95.0-nightly
- Node v25.6.1, npm 11.9.0
- Wrangler 4.61.1

## Worker Secrets (production only, not needed for dev/test)

- `SPIKES_TOKEN` — Admin API token
- `STRIPE_WEBHOOK_SECRET` — Stripe webhook verification
- `STRIPE_SECRET_KEY` — Stripe API key
- `SENTRY_DSN` — Sentry error reporting

## D1 Bindings

- Database name: `spikes-sh-db`
- Binding name: `DB`

## R2 Bindings

- Bucket name: `spikes-hosted-assets`
- Binding name: `SHARES_BUCKET`
