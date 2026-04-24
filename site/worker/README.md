# spikes.sh Worker (Source Pointer)

This directory does **not** contain the Worker source code.

## Where is the Worker source?

The canonical Worker source lives in the private `spikes-hosted` repository:

```
../spikes-hosted/worker/
```

## Why is the source elsewhere?

The Worker backend contains:
- Stripe billing integration (private keys)
- Authentication logic and D1 schema migrations
- Deployment configuration via separate wrangler environment

This code is kept private and deployed separately from the open-source CLI/widget.

## What's in this directory?

This `site/worker/` directory exists for deployment artifacts only. The compiled Worker is deployed to Cloudflare Workers from the `spikes-hosted` repo, not from here.

## Testing the Worker

```bash
cd ../spikes-hosted/worker
npm test          # Worker test suite (vitest)
npm run dev       # Local wrangler dev server
```
