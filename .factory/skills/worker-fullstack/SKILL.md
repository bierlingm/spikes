---
name: worker-fullstack
description: Implements changes spanning both repos (widget, CI/CD, documentation, dependency cleanup)
---

# Fullstack Worker

NOTE: Startup and cleanup are handled by `worker-base`. This skill defines the WORK PROCEDURE.

## When to Use This Skill

Features that touch documentation, site content, npm packages, or span both repos. This includes:
- Site content (llms.txt, agents.md, llms-full.txt)
- npm package creation (spikes-mcp wrapper)
- Registry manifests (smithery.yaml)
- Documentation updates
- CI/CD workflow configuration

## Work Procedure

1. **Read the feature description thoroughly.** Understand what's being changed and in which repo(s).

2. **For widget changes:**
   - Read `./widget/spikes.js` (the source of truth)
   - Make changes to `widget/spikes.js`
   - Copy to `site/spikes.js` (CDN copy) if it exists
   - Verify widget size: `gzip -c widget/spikes.js | wc -c` — must stay under 15KB
   - Test manually: `cd cli && cargo run -- serve --port 3847`, open browser, verify behavior
   - Use agent-browser to verify interactive behaviors (toasts, dedup, z-index)

3. **For CI/CD changes:**
   - Read existing workflows in `.github/workflows/` (both repos)
   - Make changes to workflow YAML files
   - Validate YAML syntax
   - Test locally if possible (e.g., run the test commands that CI would run)

4. **For documentation:**
   - Create or update markdown files in `./docs/`
   - Ensure all CLI commands are accurate by running `spikes <cmd> --help`
   - Ensure all widget attributes are accurate by reading `widget/spikes.js`
   - Cross-reference with actual behavior

5. **For dependency cleanup:**
   - Edit `cli/Cargo.toml` to remove crates
   - Remove all source files that import the removed crates
   - Remove module declarations in `mod.rs` and `main.rs`
   - Run `cd cli && cargo build` — must succeed with no errors or warnings
   - Run `cd cli && cargo test` — all existing tests must still pass

6. **For test infrastructure:**
   - Create `vitest.config.ts` in `../spikes-hosted/worker/`
   - Create `tsconfig.json` in `../spikes-hosted/worker/`
   - Create test helper files in `../spikes-hosted/worker/test/`
   - Add dev dependencies to `package.json`
   - Verify: `cd ../spikes-hosted/worker && npx vitest run` passes

7. **Commit changes in appropriate repo(s).**

## Example Handoff

```json
{
  "salientSummary": "Set up Vitest + Miniflare test infrastructure for spikes-hosted worker. Created vitest.config.ts, tsconfig.json, test helpers for D1 migration and R2 seeding. Added 3 smoke tests (health endpoint, spike creation, share creation) that pass. Installed vitest ~3.2.0, @cloudflare/vitest-pool-workers, typescript, and wrangler as dev dependencies.",
  "whatWasImplemented": "Test infrastructure in ../spikes-hosted/worker/: vitest.config.ts with defineWorkersConfig, tsconfig.json with strict mode and workers types, test/setup.ts for D1 migrations, test/helpers.ts for common test utilities. Three smoke tests demonstrating the pattern. package.json updated with test script and dev dependencies.",
  "whatWasLeftUndone": "",
  "verification": {
    "commandsRun": [
      { "command": "cd ../spikes-hosted/worker && npx vitest run", "exitCode": 0, "observation": "3 tests pass: GET /health returns 200, POST /spikes creates spike, POST /shares creates share" },
      { "command": "cd ../spikes-hosted/worker && npx tsc --noEmit", "exitCode": 0, "observation": "No type errors" },
      { "command": "gzip -c widget/spikes.js | wc -c", "exitCode": 0, "observation": "5832 bytes (under 15KB limit)" }
    ],
    "interactiveChecks": [
      { "action": "Opened http://localhost:3847/ in agent-browser, clicked widget button", "observed": "Widget opens spike mode, crosshair cursor appears, elements highlight on hover" }
    ]
  },
  "tests": {
    "added": [
      {
        "file": "../spikes-hosted/worker/test/smoke.test.ts",
        "cases": [
          { "name": "GET /health returns 200", "verifies": "basic worker functionality" },
          { "name": "POST /spikes creates spike", "verifies": "spike creation with D1" },
          { "name": "POST /shares creates share", "verifies": "share creation with R2" }
        ]
      }
    ]
  },
  "discoveredIssues": []
}
```

## When to Return to Orchestrator

- Widget change exceeds 15KB gzipped limit
- CI/CD change requires GitHub secrets that aren't configured
- Documentation reveals undocumented behavior that needs a design decision
- Removing a dependency causes cascading compilation failures
