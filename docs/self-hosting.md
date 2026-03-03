# Self-Hosting Guide

This guide explains how to deploy your own Spikes backend using Cloudflare Workers, D1 database, and R2 storage. Self-hosting gives you full control over your feedback data without relying on spikes.sh.

## Prerequisites

Before you begin, ensure you have:

| Requirement | Version | Check |
|-------------|---------|-------|
| Node.js | 18+ | `node --version` |
| Wrangler CLI | 4.0+ | `wrangler --version` |
| Cloudflare account | - | Login at dash.cloudflare.com |

Install Wrangler if needed:

```bash
npm install -g wrangler
```

Authenticate with Cloudflare:

```bash
wrangler login
```

---

## Overview

Self-hosting is for advanced users who want full control over their feedback data. The hosted backend at spikes.sh uses this same architecture:

- **Cloudflare Worker** — API endpoints and request handling
- **D1 Database** — Structured feedback storage
- **R2 Storage** — Static file serving for shares

The reference implementation lives in [`spikes-hosted`](https://github.com/bierlingm/spikes-hosted) — a separate repo you can fork or reference.

---

## Quick Start

### Step 1: Scaffold the Worker

One command generates a complete Worker project:

```bash
spikes deploy cloudflare --dir ./my-spikes-worker
cd my-spikes-worker
```

This creates:
```
my-spikes-worker/
├── wrangler.toml      # Worker configuration
├── src/
│   └── index.ts      # Worker code
├── schema.sql        # D1 database schema
└── package.json      # Dependencies
```

### Step 2: Create D1 Database

Create a D1 database:

```bash
wrangler d1 create spikes-db
```

Note the `database_id` from the output. Update `wrangler.toml`:

```toml
[[d1_databases]]
binding = "DB"
database_name = "spikes-db"
database_id = "YOUR_DATABASE_ID_HERE"  # <-- Replace this
```

Apply the schema:

```bash
wrangler d1 execute spikes-db --file=schema.sql
```

### Step 3: Create R2 Bucket (Optional)

For serving static files via shares, create an R2 bucket:

```bash
wrangler r2 bucket create spikes-shares
```

Add to `wrangler.toml`:

```toml
[[r2_buckets]]
binding = "R2"
bucket_name = "spikes-shares"
```

### Step 4: Generate Auth Token

Generate a secure token for API authentication:

```bash
openssl rand -hex 32
```

Set it as an environment variable in `wrangler.toml`:

```toml
[vars]
SPIKES_TOKEN = "YOUR_GENERATED_TOKEN_HERE"
```

For production, use a secret instead:

```bash
wrangler secret put SPIKES_TOKEN
```

### Step 5: Install Dependencies

```bash
npm install
```

### Step 6: Deploy

```bash
wrangler deploy
```

Your Worker is now live at `https://spikes-worker.YOUR_SUBDOMAIN.workers.dev`.

---

## Configuration

### wrangler.toml Reference

```toml
name = "spikes-worker"
main = "src/index.ts"
compatibility_date = "2024-01-01"

[vars]
SPIKES_TOKEN = "your-secure-token"

[[d1_databases]]
binding = "DB"
database_name = "spikes-db"
database_id = "your-database-id"

# Optional: for static file serving
[[r2_buckets]]
binding = "R2"
bucket_name = "spikes-shares"
```

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `SPIKES_TOKEN` | Auth token for admin operations | Yes |

Set secrets for production:

```bash
wrangler secret put SPIKES_TOKEN
```

---

## Using Your Self-Hosted Backend

### Configure the Widget

Set `data-endpoint` to your Worker URL with the token:

```html
<script src="https://spikes.sh/spikes.js"
  data-endpoint="https://spikes-worker.YOUR_SUBDOMAIN.workers.dev/spikes?token=YOUR_TOKEN">
</script>
```

### Configure the CLI

Add your remote endpoint with token:

```bash
spikes remote add https://spikes-worker.YOUR_SUBDOMAIN.workers.dev --token <YOUR_TOKEN>
```

Then sync spikes:

```bash
spikes pull   # Fetch from your Worker
spikes push   # Upload to your Worker
```

**Managing remote configuration:**

```bash
spikes remote show              # View current endpoint
spikes remote remove            # Remove configuration
```

The CLI stores endpoint and token in `.spikes/config.toml`. You can also use environment variables to override:

```bash
SPIKES_API_URL=https://your-worker.workers.dev SPIKES_TOKEN=your-token spikes pull
```

### Use Environment Variable Override

```bash
SPIKES_API_URL=https://spikes-worker.YOUR_SUBDOMAIN.workers.dev spikes shares
SPIKES_TOKEN=your-token spikes pull
```

---

## API Endpoints

Your self-hosted Worker provides these endpoints:

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| `GET` | `/health` | Health check | None |
| `POST` | `/spikes` | Create spike | Query token |
| `GET` | `/spikes` | List spikes | Query token |
| `GET` | `/spikes/:id` | Get spike by ID | Query token |

Authentication via query parameter:

```bash
curl "https://your-worker.workers.dev/spikes?token=YOUR_TOKEN"
```

---

## Database Schema

The D1 schema includes:

```sql
CREATE TABLE spikes (
    id TEXT PRIMARY KEY,
    project TEXT NOT NULL,
    page TEXT NOT NULL,
    url TEXT,
    type TEXT NOT NULL,
    selector TEXT,
    xpath TEXT,
    element_text TEXT,
    bounding_box TEXT,
    rating TEXT,
    comments TEXT,
    reviewer_id TEXT NOT NULL,
    reviewer_name TEXT NOT NULL,
    reviewer_email TEXT,
    timestamp TEXT NOT NULL,
    viewport TEXT,
    user_agent TEXT
);

CREATE INDEX idx_spikes_project ON spikes(project);
CREATE INDEX idx_spikes_page ON spikes(page);
CREATE INDEX idx_spikes_reviewer ON spikes(reviewer_id);
CREATE INDEX idx_spikes_timestamp ON spikes(timestamp);
```

---

## Advanced Configuration

### Custom Domain

Add a custom domain to your Worker:

```bash
wrangler domains add your-domain.com
```

Or via the Cloudflare dashboard: Workers > your-worker > Settings > Triggers > Custom Domains.

### CORS Configuration

For cross-origin widget requests, the Worker includes default CORS headers. Modify `corsHeaders` in `src/index.ts` if needed:

```typescript
const corsHeaders = {
  'Access-Control-Allow-Origin': 'https://your-frontend.com',
  'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type',
};
```

### Rate Limiting

The basic template doesn't include rate limiting. For production use, implement rate limiting using D1:

```sql
CREATE TABLE rate_limits (
    identifier TEXT PRIMARY KEY,
    count INTEGER,
    window_start TEXT
);
```

### Multi-User Authentication

For multi-user scenarios, extend the schema:

```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE,
    tier TEXT DEFAULT 'free',
    stripe_customer_id TEXT
);

CREATE TABLE user_tokens (
    token TEXT PRIMARY KEY,
    user_id TEXT,
    created_at TEXT,
    revoked_at TEXT
);
```

---

## Monitoring

### View Logs

```bash
wrangler tail
```

### Metrics

View Worker metrics in the Cloudflare dashboard: Workers > your-worker > Metrics.

### Health Check

```bash
curl https://your-worker.workers.dev/health
# {"status":"ok","service":"spikes-worker"}
```

---

## Troubleshooting

### Database Connection Errors

Ensure your database ID matches:

```bash
wrangler d1 list
# Compare database_id with wrangler.toml
```

### Token Authentication Fails

Verify the token:

1. Check `wrangler.toml` for `[vars].SPIKES_TOKEN`
2. Or check secrets: `wrangler secret list`
3. Test: `curl "https://your-worker.workers.dev/spikes?token=wrong" # Should return 401`

### CORS Errors

Ensure the Worker returns correct CORS headers for your origin. Check `src/index.ts` `corsHeaders` object.

### Deployment Fails

Run with verbose output:

```bash
wrangler deploy --verbose
```

---

## Cost Estimates

Cloudflare's free tier includes:

| Service | Free Tier |
|---------|-----------|
| Workers | 100,000 requests/day |
| D1 | 5 GB storage, 5 million rows read/day |
| R2 | 10 GB storage, 1 million Class A ops/month |

For typical Spikes usage, the free tier is sufficient. See [Cloudflare Pricing](https://www.cloudflare.com/pricing/) for paid plans.

---

## Security Considerations

1. **Token Security**: Never commit `SPIKES_TOKEN` to source control. Use `wrangler secret` for production.

2. **HTTPS Only**: Workers are served over HTTPS by default. Do not disable.

3. **CORS**: Restrict `Access-Control-Allow-Origin` to your actual frontend domains.

4. **Input Validation**: The Worker validates and sanitizes input, but review any custom modifications.

5. **Rate Limiting**: Implement rate limiting for production use to prevent abuse.

---

## Next Steps

- Review the [CLI Reference](./cli-reference.md) for available commands
- Check the [Widget Attributes](./widget-attributes.md) for widget configuration
- See [API.md](./API.md) for the full API contract
