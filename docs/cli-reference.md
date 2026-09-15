# CLI Command Reference

Complete reference for all Spikes CLI commands. Install with:

```bash
cargo install spikes
```

## Global Options

| Flag | Description | Default |
|------|-------------|---------|
| `-p, --port <PORT>` | Port for dev server (magic mode) | 3847 |
| `-h, --help` | Print help | |
| `-V, --version` | Print version | |

---

## Spike Management

### spikes init

Initialize a `.spikes/` directory in the current project.

```bash
spikes init [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

**Description:** Creates `.spikes/` directory with `config.toml` and adds `.spikes/` to `.gitignore` if it exists.

**Examples:**
```bash
spikes init
spikes init --json
```

---

### spikes list

List all spikes with optional filters.

```bash
spikes list [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |
| `--page <PAGE>` | Filter by page name |
| `--reviewer <REVIEWER>` | Filter by reviewer name |
| `--rating <RATING>` | Filter by rating (love, like, meh, no) |
| `--unresolved` | Show only unresolved spikes |

**Examples:**
```bash
spikes list
spikes list --rating no
spikes list --reviewer "Pat" --unresolved
spikes list --json
```

---

### spikes show

Show a single spike by ID.

```bash
spikes show <ID> [OPTIONS]
```

**Arguments:**
| Argument | Description |
|----------|-------------|
| `<ID>` | Spike ID or prefix (minimum 4 characters) |

**Options:**
| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

**Examples:**
```bash
spikes show abc123def456
spikes show abc1 --json
```

---

### spikes export

Export all spikes to a file.

```bash
spikes export [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `-f, --format <FORMAT>` | Output format: json, csv, jsonl, cursor-context, claude-context | json |

**Examples:**
```bash
spikes export
spikes export --format csv > feedback.csv
spikes export --format jsonl > feedback.jsonl
spikes export --format cursor-context > cursor-feedback.md
spikes export --format claude-context > claude-feedback.md
```

---

### spikes hotspots

Show elements with the most feedback.

```bash
spikes hotspots [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

**Description:** Aggregates spikes by CSS selector to identify elements receiving the most feedback.

**Examples:**
```bash
spikes hotspots
spikes hotspots --json
```

---

### spikes reviewers

List all reviewers who left feedback.

```bash
spikes reviewers [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

**Examples:**
```bash
spikes reviewers
spikes reviewers --json
```

---

### spikes delete

Delete a spike from local storage.

```bash
spikes delete <ID> [OPTIONS]
```

**Arguments:**
| Argument | Description |
|----------|-------------|
| `<ID>` | Spike ID or prefix (minimum 4 characters) |

**Options:**
| Flag | Description |
|------|-------------|
| `-f, --force` | Skip confirmation prompt |
| `--json` | Output as JSON |

**Examples:**
```bash
spikes delete abc123
spikes delete abc1 --force
```

---

### spikes resolve

Mark a spike as addressed, won't do, or open again.

```bash
spikes resolve <ID> [OPTIONS]
```

**Arguments:**
| Argument | Description |
|----------|-------------|
| `<ID>` | Spike ID or prefix (minimum 4 characters) |

**Options:**
| Flag | Description |
|------|-------------|
| `--in <LABEL>` | Mark as addressed in a version (sets `status: addressed`, `addressed_in`) |
| `--wont-do` | Mark as won't do (`status: wont_do`) |
| `--undo` | Reopen the spike (`status: open`) |
| `--unresolve` | Alias of `--undo` |
| `--json` | Output as JSON |

**Description:** Updates the local cache when the spike is there. When a remote is configured (`[remote]` in `.spikes/config.toml`), the change is also sent as `PATCH /spikes/:id`. Without flags the request body stays `{ "resolved": true }`; `--in`, `--wont-do`, and `--undo` send the `status` field. Resolved spikes are excluded from `spikes list --unresolved`.

**Examples:**
```bash
spikes resolve abc123
spikes resolve abc123 --in v0.5
spikes resolve abc123 --wont-do
spikes resolve --undo abc123
```

---

### spikes reply

Answer a reviewer's spike. The reply shows up on the page where the comment was left (hosted projects).

```bash
spikes reply <ID> <TEXT> [OPTIONS]
```

**Arguments:**
| Argument | Description |
|----------|-------------|
| `<ID>` | Spike ID or prefix (a prefix is expanded from the local cache) |
| `<TEXT>` | Reply text |

**Options:**
| Flag | Description |
|------|-------------|
| `--version <LABEL>` | Version this reply refers to; implies `--status addressed` and sets `addressed_in` |
| `--status <STATUS>` | `addressed`, `wont_do`, or `open` |
| `--name <AUTHOR>` | Author name shown to the reviewer (default: Builder) |
| `--json` | Output as JSON |

**Description:** Sends `POST /spikes/:id/replies`. The local cache entry (if any) is updated with the new status, `addressedIn`, `replyCount`, and `lastReply`.

**Examples:**
```bash
spikes reply abc123 "Moved the hero up, see v0.5" --version v0.5
spikes reply abc123 "Out of scope for this round" --status wont-do
spikes reply abc123 "Did you mean the mobile menu?" --name Moritz
```

---

### spikes status

Show which credential is in use, whether the server accepts it, and how fresh the local cache is.

```bash
spikes status [--json]
```

**Description:** Resolves the credential in this order: `[remote] token` in `.spikes/config.toml`, then `SPIKES_TOKEN`, then the global auth file. Calls `GET /me` and reports the identity, or the specific failure: `TOKEN_REVOKED` (with `revoked_at`), `TOKEN_EXPIRED` (with `expires_at`), or `AUTH_FAILED`. Reports the age of `.spikes/state.json`. **Exits 1 when the credential is unusable.**

**Example output:**
```
  Endpoint:    https://spikes.sh
  Credential:  sk_spikes_ab… (api_key)
  Source:      .spikes/config.toml [remote].token
  Auth:        INVALID (TOKEN_REVOKED)
               revoked at 2026-09-01T00:00:00Z
  Cache:       pulled 5 days ago (2026-09-10T11:38:00Z) — stale
```

---

### spikes watch

Stream new and updated feedback as JSON lines, one object per event.

```bash
spikes watch [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--since <ISO\|last>` | Start point: an ISO 8601 timestamp, or `last` for the previous pull (default; falls back to now) |
| `--interval <SECONDS>` | Poll interval (default: 30) |
| `--url-prefix <PREFIX>` | Only spikes whose URL starts with this prefix |
| `--exec <COMMAND>` | Run a shell command per event with the JSON on stdin |
| `--once` | Poll once and exit |

**Events:** `spike.created`, `spike.updated` (each with a `spike` object), and, when `[project].key` is configured, `question.answered` (with `question` and `answer`). The stamp in `.spikes/state.json` is updated after every poll, so the next `spikes pull --since last` continues from here. A non-zero exit of the `--exec` command is logged to stderr and polling continues. Revoked or expired credentials stop the watch with exit 1.

**Examples:**
```bash
spikes watch
spikes watch --once --since last | jq .
spikes watch --url-prefix https://statecraft.systems/prosser/versions/v0-5/

# Herdr bridge: prompt a named agent with every new comment
spikes watch --exec 'herdr agent prompt builder'
```

---

## Local Development

### spikes inject

Add or remove the Spikes widget script tag in HTML files.

```bash
spikes inject <DIRECTORY> [OPTIONS]
```

**Arguments:**
| Argument | Description |
|----------|-------------|
| `<DIRECTORY>` | Directory containing HTML files |

**Options:**
| Flag | Description |
|------|-------------|
| `--remove` | Remove widget script tags instead of adding |
| `--widget-url <URL>` | URL for widget script (default: `/spikes.js` for local serve) |
| `--json` | Output as JSON |

**Description:** Recursively finds all `.html` files and injects `<script src="spikes.js"></script>` before `</body>`. Use `--remove` to clean up.

**Examples:**
```bash
spikes inject ./mockups
spikes inject ./mockups --widget-url "https://spikes.sh/spikes.js"
spikes inject ./mockups --remove
```

---

### spikes serve

Start a local development server.

```bash
spikes serve [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `-p, --port <PORT>` | Port to listen on | 3847 |
| `-d, --dir <DIR>` | Directory to serve | . |
| `-m, --marked` | Enable review mode with spike markers on pages | |
| `--cors-allow-origin <ORIGIN>` | Allowed CORS origin | (same-origin only) |

**Description:** Serves static files and provides `/spikes` API for the widget. Without `--cors-allow-origin`, CORS is disabled (same-origin only).

**Examples:**
```bash
spikes serve
spikes serve --port 3000 --dir ./public
spikes serve --marked
spikes serve --cors-allow-origin "https://spikes.sh"
```

---

## Remote Sync

### spikes pull

Fetch spikes from remote and merge with local.

```bash
spikes pull [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--endpoint <URL>` | Remote endpoint URL (or from config) |
| `--token <TOKEN>` | Auth token (or from config) |
| `--from <URL>` | Pull from a public share URL |
| `--since <ISO\|last>` | Only spikes updated after a timestamp, or since the previous pull (`last`) |
| `--url-prefix <PREFIX>` | Only spikes whose URL starts with this prefix |
| `--json` | Output as JSON |

**Description:** New spikes are appended to `.spikes/feedback.jsonl`; spikes that already exist locally are replaced in place when the remote copy changed (status, replies). Every successful pull writes `.spikes/state.json` (`last_pulled_at`, `endpoint`, `credential_prefix`). `list`, `show`, `export`, and `hotspots` warn on stderr when a remote is configured and this stamp is missing or older than 24 hours.

**Examples:**
```bash
spikes pull
spikes pull --since last
spikes pull --since 2026-09-14T00:00:00Z --url-prefix https://example.com/v0-5/
spikes pull --from "https://spikes.sh/s/my-project"
spikes pull --endpoint "https://api.example.com" --token "secret"
```

---

### spikes push

Upload local spikes to remote.

```bash
spikes push [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--endpoint <URL>` | Remote endpoint URL (or from config) |
| `--token <TOKEN>` | Auth token (or from config) |
| `--json` | Output as JSON |

**Examples:**
```bash
spikes push
spikes push --endpoint "https://api.example.com/spikes" --token "secret"
```

---

### spikes sync

Sync with remote (pull then push).

```bash
spikes sync [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

**Examples:**
```bash
spikes sync
```

---

### spikes remote

Manage remote endpoint configuration.

```bash
spikes remote <COMMAND>
```

**Subcommands:**
| Command | Description |
|---------|-------------|
| `add` | Add or update remote endpoint |
| `remove` | Remove remote configuration |
| `show` | Show current remote configuration |

#### spikes remote add

```bash
spikes remote add <ENDPOINT> [OPTIONS]
```

**Arguments:**
| Argument | Description |
|----------|-------------|
| `<ENDPOINT>` | Endpoint URL |

**Options:**
| Flag | Description |
|------|-------------|
| `--token <TOKEN>` | Auth token |
| `--hosted` | Use spikes.sh hosted backend |

**Examples:**
```bash
spikes remote add https://api.example.com/spikes --token secret
spikes remote add https://spikes.sh/api --hosted
```

#### spikes remote remove

```bash
spikes remote remove
```

#### spikes remote show

```bash
spikes remote show [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

---

### spikes projects

Manage hosted projects.

```bash
spikes projects create <KEY> [--origin <ORIGIN>]... [--json]
spikes projects list [--json]
```

**Description:** `create` calls `POST /projects` (user token or account key) and, when a `.spikes/` directory exists without a `[project].key`, writes the key into `.spikes/config.toml`. `list` calls `GET /me/projects`.

**Examples:**
```bash
spikes projects create prosser --origin https://statecraft.systems
spikes projects list
```

---

### spikes versions

Declare review versions for the configured project (`[project].key`).

```bash
spikes versions add <LABEL> --prefix <URL_PREFIX> [--notes <TEXT>] [--json]
spikes versions list [--json]
spikes versions notes <LABEL> <TEXT> [--json]
```

**Description:** A spike belongs to the version with the longest `url_prefix` matching its URL. Notes are shown to reviewers as "what changed".

**Examples:**
```bash
spikes versions add v0.5 --prefix /prosser/versions/v0-5/ --notes "Addresses comments 3, 5 and 6"
spikes versions list
spikes versions notes v0.5 "Also fixed the mobile menu"
```

---

### spikes questions

Ask reviewers questions and read their answers (configured project).

```bash
spikes questions add <TITLE> [--body <TEXT>] [--json]
spikes questions list [--closed] [--json]
spikes questions answers <ID> [--json]
spikes questions close <ID> [--json]
```

**Examples:**
```bash
spikes questions add "Pool hours?" --body "Should the calendar show night bookings?"
spikes questions list
spikes questions answers q_abc123
spikes questions close q_abc123
```

---

## Authentication

### spikes login

Log in to spikes.sh hosted service.

```bash
spikes login [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--token <TOKEN>` | Auth token (or enter interactively) |
| `--json` | Output as JSON |

**Description:** Opens browser for magic link authentication. Token stored in `~/.config/spikes/auth.toml` with 0600 permissions.

**Examples:**
```bash
spikes login
spikes login --token "abc123"
```

---

### spikes logout

Log out from spikes.sh.

```bash
spikes logout [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

**Examples:**
```bash
spikes logout
```

---

### spikes whoami

Show current user identity.

```bash
spikes whoami [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

**Examples:**
```bash
spikes whoami
spikes whoami --json
```

---

### spikes auth create-key

Create an API key for agent authentication.

```bash
spikes auth create-key [--name <NAME>] [--project <KEY>] [--save] [--json]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--name <NAME>` | Label for the key |
| `--project <KEY>` | Scope the key to one project (requires `spikes login`); it can only read and write that project |
| `--save` | Write the key into `.spikes/config.toml` under `[remote]` (and set `[project].key` if unset) |
| `--json` | Output as JSON |

**Description:** Account keys are stored in the global auth file. Project keys are not; put them in the repo's `.spikes/config.toml` (or pass `--save`). A project key never expires unless revoked, and leaking it exposes one project, not the account.

**Examples:**
```bash
spikes auth create-key --name "my-agent"
spikes auth create-key --project prosser --save
spikes auth list-keys
spikes auth revoke-key key_abc123
```

---

## Sharing

### spikes share

Upload a directory to spikes.sh for instant sharing.

```bash
spikes share <DIRECTORY> [OPTIONS]
```

**Arguments:**
| Argument | Description |
|----------|-------------|
| `<DIRECTORY>` | Directory to upload |

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--name <NAME>` | Custom name for the share URL | (auto-generated) |
| `--password <PASSWORD>` | Password-protect the share (Pro only) | |
| `--host <HOST>` | Host URL for the API | https://spikes.sh |
| `--json` | Output as JSON | |

**Examples:**
```bash
spikes share ./mockups
spikes share ./mockups --name "design-review-v2"
spikes share ./mockups --password "secret123"
```

---

### spikes shares

List your shared projects on spikes.sh.

```bash
spikes shares [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

**Examples:**
```bash
spikes shares
spikes shares --json
```

---

### spikes unshare

Delete a shared project from spikes.sh.

```bash
spikes unshare <SLUG> [OPTIONS]
```

**Arguments:**
| Argument | Description |
|----------|-------------|
| `<SLUG>` | Share slug to delete |

**Options:**
| Flag | Description |
|------|-------------|
| `-f, --force` | Skip confirmation prompt |
| `--json` | Output as JSON |

**Examples:**
```bash
spikes unshare my-project
spikes unshare my-project --force
```

---

## MCP

### spikes mcp serve

Start the MCP (Model Context Protocol) server for AI agent integration.

```bash
spikes mcp serve [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--remote` | Use hosted API instead of local JSONL | |
| `--transport <MODE>` | Transport mode: stdio or http | stdio |
| `--port <PORT>` | Port for HTTP transport | 3848 |
| `--bind <ADDR>` | Bind address for HTTP transport | 127.0.0.1 |

**Description:** Exposes 9 MCP tools for agents: `get_spikes`, `get_element_feedback`, `get_hotspots`, `submit_spike`, `resolve_spike`, `delete_spike`, `create_share`, `list_shares`, and `get_usage`. Supports stdio (default) and HTTP transport. All logging goes to stderr; stdout is reserved for JSON-RPC.

**Examples:**
```bash
spikes mcp serve                              # Local mode, stdio
spikes mcp serve --remote                     # Remote mode, stdio
spikes mcp serve --transport http --port 3848 # Local mode, HTTP
```

See [MCP Server Guide](./mcp.md) for configuration details.

---

## Billing

### spikes billing

Open Stripe Customer Portal to manage subscription.

```bash
spikes billing [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

**Examples:**
```bash
spikes billing
```

---

### spikes upgrade

Upgrade to Pro subscription via Stripe Checkout.

```bash
spikes upgrade [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

**Examples:**
```bash
spikes upgrade
```

---

### spikes usage

Display current usage statistics.

```bash
spikes usage [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

**Examples:**
```bash
spikes usage
spikes usage --json
```

---

## Utility

### spikes config

Show current configuration.

```bash
spikes config [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--json` | Output as JSON |

**Examples:**
```bash
spikes config
spikes config --json
```

---

### spikes update

Update Spikes CLI and widget to the latest version.

```bash
spikes update
```

**Description:** Checks crates.io for the latest version and runs `cargo install spikes --force` to update.

**Examples:**
```bash
spikes update
```

---

### spikes version

Show version.

```bash
spikes version
```

**Examples:**
```bash
spikes version
```

---

## Deployment

### spikes deploy

Deploy backend to Cloudflare.

```bash
spikes deploy <BACKEND>
```

**Subcommands:**
| Command | Description |
|---------|-------------|
| `cloudflare` | Scaffold Cloudflare Worker + D1 for multi-reviewer sync |

#### spikes deploy cloudflare

```bash
spikes deploy cloudflare [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--dir <DIR>` | Output directory | ./spikes-worker |
| `--json` | Output as JSON | |

**Description:** Generates a Cloudflare Worker with D1 database bindings for hosting your own Spikes backend.

**Examples:**
```bash
spikes deploy cloudflare
spikes deploy cloudflare --dir ./my-spikes-worker
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SPIKES_TOKEN` | Auth token. Precedence for hosted commands: `[remote] token` in `.spikes/config.toml`, then `SPIKES_TOKEN`, then the global auth file |
| `SPIKES_API_URL` | Override API base URL (default: https://spikes.sh); `[remote] endpoint` in `.spikes/config.toml` wins when set |

**Examples:**
```bash
SPIKES_TOKEN=abc123 spikes whoami
SPIKES_API_URL=http://localhost:8787 spikes shares
```

---

## Magic Mode

Running `spikes` without a subcommand starts a development server on port 3847:

```bash
spikes          # Equivalent to: spikes serve --port 3847
spikes --port 3000  # Serve on port 3000
```

This provides a quick way to serve mockups with the Spikes widget active.
