# Spikes Widget Test Suite

Playwright-based regression suite for the Spikes widget. Runs locally via `agent-ci` (no GitHub Actions).

## Quick Start

```bash
cd tests/widget
npm install
npx playwright install chromium  # One-time (~120MB)
npm test
```

The suite completes in under 60 seconds and includes:
- 6 bytes-freshness marker assertions (Node-level, no browser)
- Playwright behavioral tests (chromium-only)

## Network Isolation

All tests use `page.route()` mocking. **No real network requests** to `https://spikes.sh` are made during test execution.

## Fixture Server

The Playwright `webServer` config starts a minimal static server on `http://localhost:4717`:
- Serves `fixtures/*.html`
- Resolves `/spikes.js` to the **canonical** `widget/spikes.js` at the repo root (not a copy)

## Marker Checks

The marker tests verify the canonical widget bytes contain expected code paths:
- `spikes-error-dot` >= 1 (visible error indicator element)
- `setErrorState` >= 4 (error-state function and call sites)
- `[Spikes] Sync failed` >= 1 (console diagnostic)
- > 100 lines (anti-minification guard)

These catch the M01-on-force-merge regression class where visible-error code could be accidentally stripped.

## CI Entry Point

From the repo root:

```bash
npx agent-ci run --workflow workflows-local/widget.yml
```
