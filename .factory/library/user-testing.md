# User Testing

Testing surface: tools, URLs, setup steps, isolation notes, known quirks.

**What belongs here:** How to manually test, what tools to use, test accounts, known issues with testing setup.

---

## Testing Surfaces

### CLI (terminal)
- Commands: `spikes list`, `spikes show <id>`, `spikes export --format <fmt>`, etc.
- JSON output: Add `--json` to any command
- Test data: `.spikes/feedback.jsonl` (JSONL format, one spike per line)
- Tool: direct CLI invocation, assert_cmd in tests

### MCP Server (stdio)
- Start: `cd cli && cargo run -- mcp serve`
- Pipe JSON-RPC messages to stdin, read responses from stdout
- Test: `echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | cargo run -- mcp serve 2>/dev/null`
- Multi-request: `printf '...line1...\n...line2...\n' | cargo run -- mcp serve 2>/dev/null`
- Tool: terminal piping

### Context Exports
- Run: `cd cli && cargo run -- export --format cursor-context` (or claude-context)
- Requires: `.spikes/feedback.jsonl` with test data
- Tool: terminal output capture

### GitHub Action (static files)
- Files: `action/action.yml`, `action/check.sh`, `action/README.md`
- Test gate logic: `bash action/check.sh` with environment variables and fixture data
- Tool: shell execution

## Known Quirks

- Widget requires `<body>` tag in HTML to inject properly
- `file://` URLs work for widget but not for share flow (needs HTTP)
- wrangler dev may take 5-10 seconds to start on first run
- D1 in wrangler dev uses local SQLite file (`.wrangler/state/`)
- If `wrangler d1 migrations apply --local` fails due legacy/local drift (e.g. `duplicate column` / `no such table`), reinitialize test schema with `cd ../spikes-hosted/worker && npx wrangler d1 execute spikes-sh-db --local --file=schema.sql` before user-testing runs.
- Subdomain wildcard behavior on localhost may differ from `*.spikes.sh`; for XSS checks, combine live route probes with direct escaping verification.
- `wrangler d1 execute` output may obscure long hash fields; use direct SQLite query in `.wrangler/state/v3/d1/` when validating password hash/salt values.
- CLI `whoami` currently calls `https://spikes.sh/me` directly. If production routing serves static HTML at that path, CLI identity assertions can be blocked even when local worker `GET /me` works at `http://localhost:8787/me`.
- `spikes pull --from <share-url>` respects `SPIKES_API_URL` and now parses paginated worker responses (`{ "data": [...], "next_cursor": ... }`) correctly (validated in monetization rerun round 3).
- Share HTML currently injects widget script with `data-endpoint="https://spikes.sh"`; local widget submissions may not hit `localhost:8787` unless endpoint behavior is overridden.
- Local Miniflare D1 state can contain multiple SQLite files; when validating API-key linkage, select the DB file that actually contains the `api_keys` table before concluding schema issues.
- In isolated CLI auth tests, revoking the stored `auth.api_key` can break subsequent key-management commands in the same HOME; pass `SPIKES_TOKEN=<bearer>` explicitly for follow-up commands.
- `/auth/api-key` enforces 10/hour/IP rate limiting in local worker runs; repeated setup attempts can hit 429 and constrain extra key-status fixture generation, so prefer reusing namespace-scoped keys where possible.
- If local billing checkout endpoints return `500 Billing not configured`, you can seed `users.tier='pro'` in local D1 for a namespace-scoped user to validate Pro unlimited-limit behavior.
- MCP (rmcp SDK) requires `notifications/initialized` after `initialize` before `tools/list` / `tools/call`; skipping it can yield protocol-state errors.
- When piping multiple JSON-RPC lines into `spikes mcp serve`, keep stdin open long enough for all responses (small inter-line delays help avoid premature pipe close).
- MCP `create_share` may normalize/truncate a requested share name and append a generated suffix; validate downstream list checks against the returned slug/URL rather than the raw requested name.

## Flow Validator Guidance: Hosted Worker API

- Use your assigned data namespace for all generated resources (e.g. slug prefix, filenames, reviewer IDs).
- Use your assigned bearer token only; do not read/modify rows created by other validators.
- If database setup is needed, prefix seeded user IDs/subdomains/slugs with the namespace.
- Record exact `curl` commands and key response fields in the flow report evidence.

## Flow Validator Guidance: Identity Auth API

- Use only your assigned namespace email (`<namespace>@example.com`) and slug prefixes (`<namespace>-*`).
- Do not inspect or mutate other namespaces' rows when reading local D1 state.
- Keep all auth tokens scoped to your namespace and never reuse another validator's bearer token.
- Include explicit request/response evidence for each assertion (status code, body excerpt, and DB query output when required).

## Flow Validator Guidance: Browser Subdomain Surface

- If browser checks are required, use only your assigned namespace subdomain/slug values.
- Never use shared/non-namespaced URLs.
- Include screenshot evidence references and rendered/page-source snippets in the flow report.

## Flow Validator Guidance: CLI Serve

- Use an isolated temp directory per validator namespace (e.g. `/tmp/spikes-utv-<namespace>`).
- Start `spikes serve` on a unique port for your validator to avoid cross-test interference.
- Run traversal and CORS probes only against your own started server instance.
- Include exact request/response header evidence in the flow report.

## Flow Validator Guidance: CLI Pull/Push Surface

- Use only the assigned namespace tokens/slugs so your rate-limit and error conditions are isolated.
- Prefer targeting local worker `http://localhost:8787` and avoid changing global CLI config in the repo.
- For auth tests, set token via command flags or isolated env vars per command (do not overwrite shared auth files).
- Include exact CLI command output snippets showing the actionable message text for each mapped error case.

## Flow Validator Guidance: CLI Auth Commands

- Use an isolated HOME per validator namespace (e.g. `HOME=/tmp/spikes-utv-<namespace>-home`) so auth file paths and permissions checks are not shared.
- Do not run interactive browser login; use deterministic local setup (`spikes login --token <token>`) when validating storage/logout/whoami behavior.
- When validating `SPIKES_TOKEN` precedence, set env vars per command invocation and avoid exporting globally.
- Capture evidence with file path checks (`auth.toml` location + mode), command output (`login/logout/whoami`), and token-source behavior.

## Flow Validator Guidance: Widget UX Surface

- Use only namespace-scoped fixture pages under `/tmp/spikes-utv-<namespace>/` and avoid shared files.
- Start widget serving from `cli` on port `3847`; do not start additional web servers.
- For duplicate checks, submit identical selector/reviewer/comment payloads within 30 seconds in the same namespace only.
- For localStorage quota handling, trigger failure via in-page monkey patch in the active session; restore behavior before ending.
- Capture screenshot evidence for toast text/timing, review button visibility, z-index layering, and offset positioning.

## Flow Validator Guidance: CLI Spike Management

- Use a unique temp workspace per namespace (e.g. `/tmp/spikes-utv-<namespace>-cli`) with an isolated `.spikes/feedback.jsonl`.
- Use ID prefixes unique to namespace data when testing `delete`/`resolve` prefix behavior.
- Validate confirmation prompts in non-`--force` mode and explicit bypass with `--force` in separate commands.
- Verify unresolved filtering using a mixed resolved/unresolved dataset created only in the assigned namespace.

## Flow Validator Guidance: Docs & Dependency Verification

- Use read-only checks for documentation assertions (`docs/widget-attributes.md`, `docs/cli-reference.md`, `docs/self-hosting.md`).
- Validate dependency-removal assertions via Cargo manifest/tree and filesystem module checks without modifying source.
- Record exact command outputs for `cargo tree`, `rg`, and path existence checks to support VAL-UX-013/014/015 evidence.

## Flow Validator Guidance: Monetization API & CLI

- Use one namespace-scoped email/token pair per validator (e.g. `utv-mon-<ns>@example.com`, `utv-mon-<ns>-token`) and do not reuse credentials across groups.
- Keep slugs, share IDs, Stripe customer IDs, and event IDs namespace-prefixed to avoid collisions in local D1 state.
- For `spikes billing`, `spikes usage`, and `spikes upgrade`, run commands with `SPIKES_API_URL=http://localhost:8787` and isolated `HOME` directories.
- If browser-open behavior cannot be observed directly, capture deterministic command output and endpoint responses as evidence and mark only truly untestable external Stripe dependencies as blocked.

## Flow Validator Guidance: Cross-Area Monetization Flows

- Use a dedicated namespace workspace under `/tmp/spikes-utv-<namespace>/` for each flow validator.
- Do not share `.spikes/` state, auth files, or uploaded share slugs between validators.
- Keep all end-to-end flows scoped to local worker (`http://localhost:8787`) and local serve surface (`http://localhost:3847`) only.
- For widget/browser checks, use namespaced reviewer labels and comments so pulled feedback can be attributed unambiguously.

## Flow Validator Guidance: MCP Server (CLI stdio)

- Use an isolated temp workspace per namespace (e.g. `/tmp/spikes-utv-<namespace>-mcp`) with a namespace-scoped `.spikes/feedback.jsonl` fixture file.
- Run `cargo run -- mcp serve` from `cli/` and keep stdout clean for JSON-RPC; redirect stderr separately for evidence when validating stdout purity assertions.
- For multi-request checks, use a single stdin stream with multiple JSON-RPC lines in your namespace workspace; do not reuse another validator's fixture data.
- Capture exact request and response JSON for initialize, tools/list, and tools/call, including error cases.

## Flow Validator Guidance: Context Exports (CLI)

- Use a separate isolated temp workspace per namespace (e.g. `/tmp/spikes-utv-<namespace>-export`) and create all fixture spikes locally in that workspace.
- Validate both `cursor-context` and `claude-context` from the same fixture dataset before cross-checking with MCP-derived blocking output.
- Keep assertions deterministic by using explicit IDs/selectors/comments in fixture data and recording exact markdown excerpts as evidence.
- Do not modify repo source files for export validation; only run CLI commands against fixture data.

## Flow Validator Guidance: GitHub Action CI Surface

- Use an isolated temp workspace per namespace (e.g. `/tmp/spikes-utv-<namespace>-action`) and create fixture `.spikes/feedback.jsonl` data only inside that workspace.
- Execute gate checks via `bash action/check.sh` with per-command environment variables (`INPUT_THRESHOLD`, `INPUT_IGNORE_PATHS`, `INPUT_REQUIRE_RESOLUTION`, `GITHUB_OUTPUT`) to avoid shared state.
- When validating install mapping logic, treat `action/action.yml` and `action/check.sh` as read-only artifacts; do not edit source files during validation.
- Capture evidence for exit codes and output file values (`blocking_count`, `status`) for each assertion scenario.

## Flow Validator Guidance: CLI Build Verification

- Run `cargo build` and `cargo test` from `cli/` in an isolated namespace workspace or direct repo root without mutating tracked files.
- Record concrete evidence: command, exit code, and key output snippets (compiled target summary and total tests passed).
- If build/test fails, capture first failing test or compiler error line verbatim and mark assertion as failed or blocked with root cause.

## Flow Validator Guidance: MCP HTTP + Remote

- Use one namespace-scoped temp workspace per validator (for example `/tmp/spikes-utv-<namespace>-mcp-http`) and keep `.spikes/feedback.jsonl` fixtures local to that workspace.
- Use a namespace-scoped auth identity: request login token via local worker `/auth/login`, exchange via `/auth/verify`, and pass the resulting bearer token through command-scoped `SPIKES_TOKEN` only.
- For HTTP transport checks, run only one MCP HTTP server instance per validator and use unique ports if multiple background servers are needed.
- For remote-mode checks, set `SPIKES_API_URL=http://localhost:8787` explicitly per command so no validator depends on global CLI auth config.
- Capture evidence for initialize, notifications/initialized, tools/list, tools/call, and network-failure cases (command, status/JSON-RPC response, and relevant stderr lines).
- **Critical for VAL-MCP-021 (remote read tools):** GET /spikes requires authentication (unlike POST /spikes which is public). Ensure the bearer token is properly stored in the local D1 `user_tokens` table BEFORE testing. Use the exact token returned by `/auth/verify`, not the login token. Verify token works via `curl -H "Authorization: Bearer $TOKEN" http://localhost:8787/spikes` before MCP testing.
