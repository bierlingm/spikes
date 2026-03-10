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
- `STRIPE_PRO_PRICE_ID` — Stripe Price ID used by checkout/upgrade flows
- `STRIPE_AGENT_PRICE_ID` — Stripe metered Price ID for agent tier (NEW — must be created in Stripe Dashboard)
- `SENTRY_DSN` — Sentry error reporting

## Stripe Metered Billing (Agent Tier)

Stripe supports usage-based billing via Meters API:
1. Create a Meter (e.g., "spike_consumption") via Dashboard or API — defines aggregation (sum)
2. Create a metered Price linked to the meter — defines cost per unit
3. Report usage via POST /v1/billing/meter_events with event_name, customer, value
4. Stripe automatically bills at end of billing period

For Spikes: two meters needed (spike_consumption, share_consumption).
Worker reports events via ctx.waitUntil() — failures must not block main request.
See: https://docs.stripe.com/billing/subscriptions/usage-based/implementation-guide

## D1 Bindings

- Database name: `spikes-sh-db`
- Binding name: `DB`

## R2 Bindings

- Bucket name: `spikes-hosted-assets`
- Binding name: `SHARES_BUCKET`
