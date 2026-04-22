# Agent-CI Configuration for Spikes

This directory contains local agent-CI configuration.

## Local Workflows

Moved to `workflows-local/` and runnable via agent-ci:

```bash
# Run tests locally (was .github/workflows/test.yml)
npx agent-ci run --workflow workflows-local/test.yml

# Or run cargo directly
cd cli && cargo test --jobs 2
cd cli && cargo clippy --all-targets -- -D warnings
```

## GitHub Actions Remaining

The following workflows remain in `.github/workflows/`:
- `deploy.yml` - Cloudflare Pages deploy on push to main
- `release.yml` - Cross-platform binary builds on tags (v*)

## Deleted Workflows

This was removed from `.github/workflows/` and is now local-only:
- `test.yml` - moved to `workflows-local/test.yml`
