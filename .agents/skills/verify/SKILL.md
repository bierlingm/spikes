---
name: verify
description: How to check a Spikes change (CLI, widget, docs) before calling it done.
---
# Verify a change

```sh
cd cli && cargo build && cargo test && cargo clippy --all-targets -- -D warnings && cd ..
./scripts/check-doc-facts.sh                # AGENTS/CLAUDE/README/llms.txt facts match the code
cd tests/widget && npm ci && npm test       # if widget/ changed (Playwright)
```
Needs a recent stable Rust (deps need > 1.85) plus `pkg-config libssl-dev`.

Preview: `./cli/target/debug/spikes serve` in a folder with an HTML file, open it, leave a spike,
then `spikes list --json`. For site changes open `site/index.html` locally.

If you add an MCP tool or bump the version, update the counts/versions the doc-facts check lists.
Never push to main (it deploys the site) or push `v*` tags (it releases binaries).
