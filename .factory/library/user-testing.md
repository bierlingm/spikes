# User Testing

Testing surface: tools, URLs, setup steps, isolation notes, known quirks.

**What belongs here:** How to manually test, what tools to use, test accounts, known issues with testing setup.

---

## Testing Surfaces

### Widget (browser)
- Start: `cd cli && cargo run -- serve --port 3847`
- URL: `http://localhost:3847/`
- Tool: agent-browser
- Notes: Widget auto-injects on served HTML files. Click the `/` button to enter spike mode.

### CLI (terminal)
- Commands: `spikes list`, `spikes show <id>`, `spikes delete <id>`, etc.
- JSON output: Add `--json` to any command
- Test data: `.spikes/feedback.jsonl` (JSONL format, one spike per line)

### Hosted Worker (API)
- Start: `cd ../spikes-hosted/worker && npx wrangler dev --port 8787`
- Base URL: `http://localhost:8787`
- Health check: `GET /health`
- Tool: curl
- Auth: Bearer token in Authorization header for protected endpoints

### Share Flow (full integration)
1. Start wrangler dev (port 8787)
2. `spikes share ./mockups/` → uploads to local worker
3. Open returned URL in browser → see widget on shared page
4. Leave feedback → POST /spikes
5. `spikes pull` → fetch feedback to local JSONL

## Known Quirks

- Widget requires `<body>` tag in HTML to inject properly
- `file://` URLs work for widget but not for share flow (needs HTTP)
- wrangler dev may take 5-10 seconds to start on first run
- D1 in wrangler dev uses local SQLite file (`.wrangler/state/`)
