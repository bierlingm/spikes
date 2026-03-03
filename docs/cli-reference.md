# CLI Command Reference

Complete reference for all Spikes CLI commands. Install with:

```bash
cargo install spikes-cli
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

Mark a spike as resolved (or unresolved).

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
| `--unresolve` | Mark as unresolved instead |
| `--json` | Output as JSON |

**Description:** Adds `resolved: true` and `resolvedAt` timestamp to the spike. Resolved spikes are excluded from `spikes list --unresolved`.

**Examples:**
```bash
spikes resolve abc123
spikes resolve abc123 --unresolve
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
| `--json` | Output as JSON |

**Examples:**
```bash
spikes pull
spikes pull --from "https://spikes.sh/s/my-project"
spikes pull --endpoint "https://api.example.com/spikes" --token "secret"
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
spikes mcp serve
```

**Description:** Exposes three tools for agents: `get_spikes`, `get_element_feedback`, and `get_hotspots`. Uses stdio transport. All logging goes to stderr; stdout is reserved for JSON-RPC.

**Examples:**
```bash
spikes mcp serve
spikes mcp serve 2> mcp.log
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

**Description:** Fetches the latest release from GitHub and updates the binary.

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
| `SPIKES_TOKEN` | Override auth token (takes precedence over config file) |
| `SPIKES_API_URL` | Override API base URL (default: https://spikes.sh) |

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
