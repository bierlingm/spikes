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
- Subdomain wildcard behavior on localhost may differ from `*.spikes.sh`; for XSS checks, combine live route probes with direct escaping verification.
- `wrangler d1 execute` output may obscure long hash fields; use direct SQLite query in `.wrangler/state/v3/d1/` when validating password hash/salt values.

## Flow Validator Guidance: Hosted Worker API

- Use your assigned data namespace for all generated resources (e.g. slug prefix, filenames, reviewer IDs).
- Use your assigned bearer token only; do not read/modify rows created by other validators.
- If database setup is needed, prefix seeded user IDs/subdomains/slugs with the namespace.
- Record exact `curl` commands and key response fields in the flow report evidence.

## Flow Validator Guidance: Browser Subdomain Surface

- If browser checks are required, use only your assigned namespace subdomain/slug values.
- Never use shared/non-namespaced URLs.
- Include screenshot evidence references and rendered/page-source snippets in the flow report.

## Flow Validator Guidance: CLI Serve

- Use an isolated temp directory per validator namespace (e.g. `/tmp/spikes-utv-<namespace>`).
- Start `spikes serve` on a unique port for your validator to avoid cross-test interference.
- Run traversal and CORS probes only against your own started server instance.
- Include exact request/response header evidence in the flow report.
