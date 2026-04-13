# MCP Directory Submissions

Steps to get `spikes-mcp` listed in the canonical MCP registry and the major IDE directories. All prep files are in the repo; the actual submissions need your GitHub/account auth so they're one-command-at-a-time for you.

## 1. MCP Registry (canonical, Anthropic-blessed)

The [MCP Registry](https://registry.modelcontextprotocol.io/) is the official index consumed by Claude Code, Cursor marketplace, and most other MCP clients. Listing here propagates everywhere downstream.

**Prep already done:**
- `packages/spikes-mcp/package.json` now has `"mcpName": "io.github.bierlingm/spikes"`
- `packages/spikes-mcp/server.json` created with full manifest (transport, env vars, repo)

**Steps for you:**

1. Republish the NPM package so it carries the new `mcpName` field:
   ```bash
   cd packages/spikes-mcp
   npm version patch   # bumps 0.3.1 → 0.3.2 (or skip if you want to stay at 0.3.1 and --force)
   # update server.json "version" fields to match
   npm publish --access public
   ```

2. Install the publisher CLI (macOS):
   ```bash
   brew install mcp-publisher
   ```
   (or curl one-liner from https://github.com/modelcontextprotocol/registry/releases/latest)

3. Authenticate with GitHub:
   ```bash
   cd packages/spikes-mcp
   mcp-publisher login github
   # visit https://github.com/login/device, paste the code it prints
   ```

4. Publish:
   ```bash
   mcp-publisher publish
   ```

5. Verify:
   ```bash
   curl "https://registry.modelcontextprotocol.io/v0.1/servers?search=io.github.bierlingm/spikes"
   ```

Total time: about 5 minutes, blocked only on the GitHub device-code paste.

## 2. cursor.directory (community directory behind cursor.com)

[cursor.directory](https://cursor.directory) is maintained in the [pontusab/cursor.directory](https://github.com/pontusab/cursor.directory) repo. MCP entries live under `apps/` or `packages/` (check the repo layout at submission time — it's been restructuring through 2026).

**Steps:**

1. Fork `pontusab/cursor.directory`.
2. Find the MCP servers data file (currently under `apps/mcp/` or `packages/data/mcp-servers/` — open the repo and check the latest layout).
3. Add an entry like this (format may have drifted — match whatever's in the file):
   ```json
   {
     "name": "Spikes",
     "slug": "spikes",
     "description": "Structured UI feedback for AI-assisted building. Reviewers click elements and rate them; agents read, triage, and resolve feedback.",
     "url": "https://spikes.sh",
     "github": "https://github.com/bierlingm/spikes",
     "install": "claude mcp add spikes \"npx -y spikes-mcp\"",
     "category": "developer-tools",
     "tags": ["feedback", "ui", "agents", "claude-code", "cursor"]
   }
   ```
4. Open a PR titled `feat(mcp): add Spikes` with a 2-sentence description.

PRs get merged within a few days. After merge, the site auto-deploys.

## 3. Cursor official marketplace (cursor.com/marketplace)

The Cursor marketplace has a formal publish flow at https://cursor.com/marketplace/publish. It requires a Cursor plugin manifest, not just an MCP listing.

**Steps:**

1. Install Cursor locally if you haven't.
2. Create a plugin scaffold:
   ```
   mkdir -p ~/.cursor/plugins/local/spikes/.cursor-plugin
   ```
3. Add `~/.cursor/plugins/local/spikes/.cursor-plugin/plugin.json`:
   ```json
   {
     "name": "spikes",
     "description": "Structured UI feedback for AI-assisted building",
     "version": "0.3.1",
     "author": { "name": "Moritz Bierling" },
     "mcp_servers": [
       {
         "name": "spikes",
         "command": "npx",
         "args": ["-y", "spikes-mcp"]
       }
     ]
   }
   ```
4. Push it to its own public GitHub repo (e.g., `bierlingm/spikes-cursor-plugin`) — all Cursor plugins must be open source.
5. Go to https://cursor.com/marketplace/publish, point it at the repo, submit.
6. Manual security review takes a few days.

Note: this is *Cursor-specific glue*. You can skip it if the MCP Registry listing is enough, since Cursor also reads the registry. The marketplace listing gets you the "Add to Cursor" one-click button.

## 4. Windsurf (Codeium)

Windsurf's MCP Marketplace is curated by the Codeium team and has no public submission form as of this writing.

**Steps:**

1. Email [hi@codeium.com](mailto:hi@codeium.com) with the draft below.
2. Also post in https://discord.gg/codeium (#windsurf channel) — the team is active there and faster than email.

**Draft email/message:**

> Subject: MCP Marketplace submission — Spikes (structured UI feedback for agents)
>
> Hi Windsurf team,
>
> I run [Spikes](https://spikes.sh), an MCP server for capturing structured UI feedback that AI coding agents can act on directly. Reviewers click elements on a page, rate them, and optionally comment; the agent reads JSON (selector + bounding box + viewport + rating + comment) and writes diffs.
>
> It's already live on NPM as `spikes-mcp` and will be on the official MCP Registry as `io.github.bierlingm/spikes`. Install for Cascade is a one-liner:
>
> ```bash
> npx -y spikes-mcp
> ```
>
> Would love to get it into the Windsurf MCP Marketplace. GitHub: https://github.com/bierlingm/spikes. Happy to send a demo gif or jump on a quick call.
>
> Thanks,
> Moritz

## 5. Suggested order and follow-up

1. **Start with the MCP Registry** — highest leverage, listed once → picked up by many clients. This is the one to do first.
2. **cursor.directory PR** — quick, community-reviewed, good SEO.
3. **Windsurf outreach** — low effort, async.
4. **Cursor official marketplace** — highest effort (needs a plugin repo + security review). Only do this if you see Cursor users struggling to find Spikes via the registry alone.

**Post-submission metric to watch:** daily NPM install count on `spikes-mcp`. Pre-submission baseline, then compare weekly.
