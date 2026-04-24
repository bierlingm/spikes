# Spikes — Claude Code Context

Read AGENTS.md for full project details, architecture, and patterns.

## First thing every session

```bash
werk tree          # See what's in play
werk survey        # What's urgent
```

All work tracking lives in werk. Update tensions as you go — `werk reality <id>`, `werk note add <id>`, `werk resolve <id>`.

## Quick reference

- **What:** Feedback tool for AI-assisted building. Click elements, rate them, get JSON with CSS selectors.
- **Stack:** Rust CLI + vanilla JS widget + Cloudflare Workers/D1/R2 + Stripe
- **Domain:** spikes.sh
- **CLI version:** 0.4.0
- **Root goal:** 10 paying users (werk tension #1)
- **Tests:** `cd cli && cargo test` / `cd site/worker && npm test`
- **Agent-CI:** `npx agent-ci run --workflow workflows-local/test.yml` (local only)
- **Deploy:** push to main (site) or tag v* (binaries)

Note: `workflows-local/test.yml` replaces deleted `.github/workflows/test.yml`. GitHub Actions still runs deploy.yml and release.yml.
