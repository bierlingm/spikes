---
name: worker-widget
description: Implements widget behavior changes (config, error handling, visual indicators) and syncs all widget copies
---

# Widget Worker

NOTE: Startup and cleanup are handled by `worker-base`. This skill defines the WORK PROCEDURE.

## When to Use This Skill

Features that modify widget behavior (`widget/spikes.js`), widget config parsing, POST/sync logic, visual indicators, or require syncing the 4 widget copies. Also handles widget-related documentation updates.

## Required Skills

- `agent-browser` — for manual verification of widget behavior in the browser. Invoke after implementation to verify interactive behaviors (button appearance, error indicators, tooltips, POST targets).

## Work Procedure

1. **Read the feature description thoroughly.** Understand the behavioral change and acceptance criteria.

2. **Read the architecture notes:**
   - `.factory/library/architecture.md` — especially "Widget Endpoint Resolution" and "Widget Error Visibility" sections
   - Current widget source: `widget/spikes.js` (source of truth)

3. **Write tests first (if applicable).**
   - This widget has no JS test harness. Instead, plan your manual verification steps BEFORE coding.
   - List the specific scenarios you will verify with agent-browser.

4. **Implement the change in `widget/spikes.js`.**
   - `widget/spikes.js` is the SOURCE OF TRUTH. Edit this file first.
   - Follow existing patterns: inline styles, no external dependencies, IIFE pattern.
   - For config changes: modify the `config` object initialization (~line 30-35).
   - For POST changes: modify `saveSpike()` (~line 260-295).
   - For button changes: modify `createButton()` and related functions.

5. **Sync all 4 widget copies.** After editing `widget/spikes.js`, copy it to:
   - `site/spikes.js`
   - `site/widget.js`
   - `cli/assets/spikes.js`
   Verify with: `diff widget/spikes.js site/spikes.js && diff widget/spikes.js site/widget.js && diff widget/spikes.js cli/assets/spikes.js`

6. **Run automated tests:**
   - `cd cli && cargo test --jobs 5` — all tests must pass
   - `gzip -c widget/spikes.js | wc -c` — must be ≤15,360 bytes (15KB)

7. **Verify with agent-browser.**
   - Create a test HTML page in `/tmp/spikes-widget-test/` that embeds the widget from a local path
   - Start a local HTTP server: `python3 -m http.server 8899 --directory /tmp/spikes-widget-test/`
   - Use agent-browser to load `http://localhost:8899/test.html`
   - Verify each behavioral change: click button, submit spike, check POST URL, check error indicators
   - Each verified scenario = one `interactiveChecks` entry with full sequence and outcome
   - STOP the HTTP server when done: `lsof -ti :8899 | xargs kill 2>/dev/null || true`

8. **For documentation updates:**
   - Update embed examples in `README.md`, `site/docs.html`, `site/index.html` as needed
   - Ensure examples are consistent with new behavior

9. **Commit changes.**

## Example Handoff

```json
{
  "salientSummary": "Implemented default endpoint logic: widget now POSTs to https://spikes.sh/spikes when data-project is explicitly set and data-endpoint is absent. Added red error dot indicator on button for POST failures (onerror, non-2xx, catch). Upgraded console.warn to console.error. Synced all 4 widget copies. Verified via agent-browser: default endpoint works on localhost:8899, error dot appears with bad endpoint, clears on success.",
  "whatWasImplemented": "In widget/spikes.js: (1) Config parsing now checks script.hasAttribute('data-project') to set endpoint default to https://spikes.sh/spikes. (2) All 3 POST failure paths upgraded to console.error with URL+status. (3) Red dot child div on button (12px, #dc2626, top-right) with tooltip. (4) Error state resets on 2xx response. Synced to site/spikes.js, site/widget.js, cli/assets/spikes.js.",
  "whatWasLeftUndone": "",
  "verification": {
    "commandsRun": [
      { "command": "cd cli && cargo test --jobs 5", "exitCode": 0, "observation": "160+ tests pass, 0 failures" },
      { "command": "gzip -c widget/spikes.js | wc -c", "exitCode": 0, "observation": "5912 bytes (under 15KB limit)" },
      { "command": "diff widget/spikes.js site/spikes.js && diff widget/spikes.js site/widget.js && diff widget/spikes.js cli/assets/spikes.js", "exitCode": 0, "observation": "All 4 copies identical" },
      { "command": "grep -n 'console.warn' widget/spikes.js | grep -i 'sync\\|endpoint\\|post'", "exitCode": 1, "observation": "No console.warn in POST section" }
    ],
    "interactiveChecks": [
      { "action": "Loaded http://localhost:8899/test.html with data-project='test-app' (no data-endpoint). Clicked widget button, selected page, rated 'like', commented 'test'. Clicked Save.", "observed": "Network tab shows POST to https://spikes.sh/spikes. Response 200. No red dot on button. Toast 'Spike saved!' appeared immediately." },
      { "action": "Loaded test page with data-endpoint='https://nonexistent.invalid/spikes'. Submitted spike.", "observed": "Red dot appeared on button top-right corner after network error. Hovering button shows tooltip 'Last feedback failed to sync — see console'." },
      { "action": "Ran window.Spikes.config.endpoint = 'https://spikes.sh/spikes' in console. Submitted another spike.", "observed": "POST succeeded (200). Red dot disappeared. Tooltip reverted to default." }
    ]
  },
  "tests": {
    "added": []
  },
  "discoveredIssues": []
}
```

## When to Return to Orchestrator

- Widget gzipped size exceeds 15KB limit
- `cargo test` fails after widget copy sync (indicates CLI bundling issue)
- Cannot start local HTTP server on port 8899 (port conflict)
- CORS issues prevent testing against spikes.sh endpoint
- The existing widget code has changed significantly from what's described in architecture.md
