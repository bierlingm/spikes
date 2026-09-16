# Spikes — Claude Code Context

Read AGENTS.md for full project details, architecture, and patterns.

## First thing every session

```bash
werk tree          # See what's in play
werk list          # Active tensions (there is no `werk survey` subcommand)
```

All work tracking lives in werk. Update tensions as you go — `werk reality <id>`, `werk note add <id>`, `werk resolve <id>`. If `werk tree` reports no tensions (true as of 2026-09-16: no `.werk/` workspace exists in this repo), track work as GitHub issues in bierlingm/spikes and bierlingm/spikes-hosted instead.

## Quick reference

- **What:** Feedback tool for AI-assisted building. Click elements, rate them, get JSON with CSS selectors.
- **Stack:** Rust CLI + vanilla JS widget + Cloudflare Workers/D1/R2 + Stripe
- **Domain:** spikes.sh
- **CLI version:** 0.5.0
- **Root goal:** 10 paying users
- **Tests:** `cd cli && cargo test` / `cd ../spikes-hosted/worker && npm test`
- **Widget Tests:** `cd tests/widget && npm test` / `npx agent-ci run --workflow workflows-local/widget.yml`
- **Agent-CI:** `npx agent-ci run --workflow workflows-local/test.yml` (local only)
- **Deploy:** push to main (site) or tag v* (binaries)

Note: GitHub Actions runs test.yml (CLI tests), deploy.yml (site) and release.yml (binaries on v* tags); `workflows-local/test.yml` is the local agent-ci equivalent.
