<div align="center">

# <span>/</span> Spikes

**The feedback loop for AI-assisted building**

[![MIT License](https://img.shields.io/badge/license-MIT-e74c3c?style=flat-square)](LICENSE)
[![Widget Size](https://img.shields.io/badge/widget-8KB_gzipped-22c55e?style=flat-square)](#)
[![Free + Pro](https://img.shields.io/badge/free_+_pro-3b82f6?style=flat-square)](#pricing)

AI can build a prototype in an hour.<br>
Turning feedback into action is still the slow part.

[Quick Start](#quick-start) · [CLI Reference](#cli-commands) · [Widget Docs](docs/widget-attributes.md) · [Self-Hosting](docs/self-hosting.md)

</div>

---

## What Is This?

Spikes is a feedback tool for AI-assisted development. It lets reviewers leave targeted feedback directly on web pages — no screenshots, no "that button over there", no lost context.

Click any element. Rate it. Comment. Spikes captures the exact CSS selector, bounding box, and page context. Your AI agent gets structured JSON it can act on immediately.

**No accounts required. No build step. Works on `file://`, localhost, anywhere.**

---

## Quick Start

### 1. Install the CLI

```bash
curl -fsSL https://spikes.sh/install.sh | sh
# Or: cargo install spikes-cli
```

### 2. Add the widget to your HTML

```bash
spikes inject ./mockups/        # Injects widget script tag
spikes serve                    # http://localhost:3847
```

### 3. Collect and use feedback

```bash
spikes list                     # See all feedback
spikes list --json              # Feed to your agent
spikes list --rating no         # Find problems
spikes hotspots                 # Elements with most feedback
spikes resolve <id>             # Mark items done
```

---

## CLI Commands

| Command | Description |
|---------|-------------|
| `spikes init` | Create `.spikes/` directory with config |
| `spikes list` | List feedback (`--json`, `--page`, `--reviewer`, `--rating`, `--unresolved`) |
| `spikes show <id>` | Show single spike details |
| `spikes export` | Export to JSON/CSV/JSONL |
| `spikes hotspots` | Elements with most feedback |
| `spikes reviewers` | List all reviewers |
| `spikes inject <dir>` | Add/remove widget from HTML files |
| `spikes serve` | Local dev server (`--port`, `--marked`, `--cors-allow-origin`) |
| `spikes pull/push/sync` | Sync with remote endpoint |
| `spikes share <dir>` | Upload to spikes.sh for instant sharing |
| `spikes login/logout/whoami` | Authentication management |
| `spikes upgrade/billing` | Pro tier subscription via Stripe |
| `spikes deploy cloudflare` | Scaffold self-hosted Worker + D1 |

All commands support `--json` for scripting. See [full CLI reference](docs/cli-reference.md).

---

## Widget Attributes

```html
<script src="https://spikes.sh/spikes.js"
  data-project="my-app"
  data-position="bottom-right"
  data-color="#e74c3c"
  data-theme="dark"
  data-reviewer="Pat"
  data-endpoint="https://api.example.com/spikes"
  data-collect-email="true"
  data-admin="true">
</script>
```

| Attribute | Description | Default |
|-----------|-------------|---------|
| `data-project` | Group feedback by project key | `location.hostname` |
| `data-position` | Button corner: `bottom-right`, `bottom-left`, `top-right`, `top-left` | `bottom-right` |
| `data-color` | Accent color (any CSS color) | `#e74c3c` |
| `data-theme` | Modal theme: `dark` or `light` | `dark` |
| `data-reviewer` | Pre-set reviewer name | (prompts user) |
| `data-endpoint` | Backend URL for multi-reviewer sync | (local only) |
| `data-collect-email` | Show email field in prompt | `false` |
| `data-admin` | Enable review mode features | `false` |
| `data-offset-x/y` | Button offset from edge | — |

See [Widget Attributes Reference](docs/widget-attributes.md) for complete documentation.

---

## Architecture

Spikes has three components that work together or standalone:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   CLI       │────▶│   Widget    │◄────│   Worker    │
│  (Rust)     │     │  (Vanilla   │     │ (Cloudflare │
│             │     │    JS)      │     │  + D1)      │
└─────────────┘     └─────────────┘     └─────────────┘
     │                                            │
     │          spikes.sh (hosted)                │
     └────────────────────────────────────────────┘
```

**CLI** — Rust binary for local development, spike management, and deployment. Stores spikes in `~/.local/share/spikes/`.

**Widget** — 8KB gzipped vanilla JS. Captures element selectors, bounding boxes, ratings, and comments. Works offline via localStorage.

**Worker** — Optional Cloudflare Worker + D1 backend for multi-reviewer sync, sharing, and hosted deployments. Lives in `spikes-hosted/`.

---

## Development

### CLI

```bash
cd cli
cargo build --release
cargo test              # 160+ tests
cargo run -- --help
```

### Widget

```bash
cd widget
# Edit src/spikes.js
# Test by running: spikes serve from the project root
```

### Worker

```bash
cd ../spikes-hosted/worker
npm install
npx vitest run          # 284+ tests
npx wrangler dev
```

---

## Self-Hosting

Want your own backend? One command:

```bash
spikes deploy cloudflare    # Creates spikes-worker/ directory
cd spikes-worker && npx wrangler deploy
```

See [Self-Hosting Guide](docs/self-hosting.md) for full setup with D1 database, authentication, and Stripe billing integration.

---

## What's Changed (Recent Overhaul)

- **Security**: PBKDF2 password hashing, path traversal fixes, XSS protection
- **Auth**: Magic link authentication (no passwords to forget)
- **Billing**: Stripe integration with Pro tier support
- **Testing**: 160 CLI tests + 284 worker tests
- **Architecture**: Modular worker with clean separation of concerns
- **CI/CD**: Automated testing and deployment pipelines

---

## Detailed Documentation

- [CLI Reference](docs/cli-reference.md) — Complete command documentation
- [Widget Attributes](docs/widget-attributes.md) — All configuration options
- [Self-Hosting Guide](docs/self-hosting.md) — Deploy your own backend
- [API Reference](docs/API.md) — REST API documentation
- [Rollback Guide](docs/rollback.md) — Emergency procedures

---

## Why Spikes

| | |
|---|---|
| **Zero friction** | One script tag, no signup required, no build step |
| **Works anywhere** | `file://`, `localhost`, any domain |
| **Precise** | Element-level feedback with exact CSS selectors |
| **Agent-native** | JSON everywhere, pipes, queryable CLI |
| **Your infrastructure** | Self-host or use hosted — your choice |
| **Tiny** | Widget is 8KB gzipped |
| **Private** | No tracking, your data stays yours |

---

## Pricing

<div align="center">

### Free forever. Pro if you want more.

No accounts required to start. Login when you need Pro features.

</div>

| | **Free** | **Pro** |
|---|---|---|
| **Price** | $0 forever | Pay what you can |
| **Shares** | 5 | Unlimited |
| **Spikes per share** | 1,000 | Unlimited |
| **Widget + CLI** | Full | Full |
| **Self-hosting** | Yes | Yes |
| **Password protection** | — | Yes |
| **Webhooks** | — | Yes |
| **Badge removal** | — | Yes |

<div align="center">

MIT licensed. Payment is appreciation, not access.

`spikes upgrade` when you're ready. No pressure.

</div>

---

<div align="center">

**[Website](https://spikes.sh)** · **[Docs](https://spikes.sh/docs.html)** · **[GitHub](https://github.com/bierlingm/spikes)** · **[Issues](https://github.com/bierlingm/spikes/issues)**

MIT License · Built for builders who work with AI.

</div>
