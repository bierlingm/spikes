# Spikes Self-Host Template

Run your own Spikes backend on Cloudflare Workers.

## Prerequisites

- [Cloudflare account](https://dash.cloudflare.com/sign-up)
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/install-and-update/) installed and authenticated

```bash
npm install -g wrangler
wrangler login
```

## Quick Start

Run the setup script:

```bash
./setup.sh
```

Or manually:

```bash
# 1. Create D1 database
wrangler d1 create my-spikes-db
# Copy the database_id from output

# 2. Create R2 bucket
wrangler r2 bucket create my-spikes-assets

# 3. Create wrangler.toml from template
cp wrangler.toml.example wrangler.toml
# Edit wrangler.toml with your database_id

# 4. Run schema
wrangler d1 execute my-spikes-db --file=schema.sql

# 5. Deploy
wrangler deploy
```

## Configure CLI

Point the Spikes CLI to your worker:

```bash
# In your project directory
echo "https://my-spikes-worker.<your-subdomain>.workers.dev" > .spikes/endpoint
```

Or set globally:

```bash
export SPIKES_ENDPOINT="https://my-spikes-worker.<your-subdomain>.workers.dev"
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/shares` | Create share (Bearer auth) |
| GET | `/shares` | List shares (Bearer auth) |
| DELETE | `/shares/:id` | Delete share (Bearer auth) |
| POST | `/spikes` | Create spike (public) |
| GET | `/s/:slug` | Serve shared project |

## Customization

Edit `src/index.ts` to:
- Add authentication
- Add rate limiting
- Customize widget injection
- Add webhooks
