# spikes.sh Worker

Cloudflare Worker + D1 backend for collecting feedback on spikes.sh.

## Setup

1. Install dependencies:
```bash
npm install
```

2. Create the D1 database:
```bash
npm run db:create
```

3. Copy the `database_id` from the output and update `wrangler.toml`

4. Generate a secure token and update `SPIKES_TOKEN` in `wrangler.toml`:
```bash
openssl rand -hex 32
```

5. Run the migration:
```bash
npm run db:migrate
```

6. Deploy:
```bash
npm run deploy
```

## Endpoints

### POST /spikes (public)
Create a new spike. No auth required (for widget).

### GET /spikes?token=XXX
List all spikes. Requires token.

### GET /prospects?token=XXX
Export collected emails for prospect list. Requires token.

```bash
curl "https://spikes-sh-worker.YOUR_SUBDOMAIN.workers.dev/prospects?token=YOUR_TOKEN"
```

## Local Development

```bash
npm run dev
```
