# Spikes Feedback Gate — GitHub Action

> **Gate CI on feedback quality.** Fail builds when unresolved negative feedback exceeds your threshold.

Your prototype looks great. Your agent built it fast. But someone left feedback — and it's not pretty. **Spikes Feedback Gate** makes sure you don't ship broken experiences.

## Quick Start

Add to your workflow (e.g., `.github/workflows/ci.yml`):

```yaml
name: CI

on: [push, pull_request]

jobs:
  feedback-gate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Check feedback gate
        uses: moritzbierling/spikes/action@v0.3.1
        with:
          threshold: 0           # Fail if any blocking spikes
          ignore-paths: ""       # Optional: pages to ignore
          require-resolution: false
```

That's it. Your CI will now fail if there's unresolved negative feedback.

## How It Works

1. **Downloads** the pre-built spikes binary for your platform (Linux/macOS, x64/ARM64)
2. **Loads** feedback from `.spikes/feedback.jsonl` in your repo
3. **Counts** blocking spikes:
   - Default: unresolved spikes with negative ratings (`meh` or `no`)
   - With `require-resolution: true`: ALL unresolved spikes
4. **Fails** if blocking count exceeds threshold

## Inputs

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `threshold` | No | `0` | Maximum allowed blocking spikes before failure |
| `ignore-paths` | No | `""` | Newline-separated glob patterns for pages to ignore |
| `require-resolution` | No | `false` | When `true`, any unresolved spike is blocking |

### `threshold`

The number of blocking spikes you're willing to tolerate. Default is `0` — zero tolerance.

```yaml
with:
  threshold: 2  # Allow up to 2 blocking spikes
```

### `ignore-paths`

Glob patterns for pages that should be excluded from the check. Useful for ignoring legacy docs or WIP pages.

```yaml
with:
  ignore-paths: |
    /docs/legacy/**
    /internal/**
    **/*.draft.html
```

Patterns match against the `page` field of each spike. Uses shell-style glob matching.

### `require-resolution`

When set to `true`, the action treats **any** unresolved spike as blocking, regardless of rating. This enforces a policy that all feedback must be addressed.

```yaml
with:
  require-resolution: true  # Any unresolved spike fails the build
```

## Outputs

| Output | Description |
|--------|-------------|
| `blocking_count` | Number of blocking spikes found |
| `status` | Result status: `passed` or `failed` |

Use outputs in subsequent steps:

```yaml
- uses: moritzbierling/spikes/action@v0.3.1
  id: spikes

- name: Report results
  if: always()
  run: |
    echo "Status: ${{ steps.spikes.outputs.status }}"
    echo "Blocking count: ${{ steps.spikes.outputs.blocking_count }}"
```

## Configuration Example

A complete workflow with sensible defaults:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Your test steps here
      - run: npm test

  feedback-gate:
    runs-on: ubuntu-latest
    needs: test  # Run after tests pass
    steps:
      - uses: actions/checkout@v4

      - name: Check feedback gate
        uses: moritzbierling/spikes/action@v0.3.1
        with:
          threshold: 0
          ignore-paths: |
            /docs/api/**
            /experiments/**
          require-resolution: false
```

## Blocking Criteria

A spike is considered **blocking** when:

- **Default mode**: Unresolved AND has rating `meh` or `no`
- **With `require-resolution: true`**: Unresolved (any rating)

Positive ratings (`love`, `like`) are never blocking. Resolved spikes are never blocking.

## Edge Cases

| Situation | Behavior |
|-----------|----------|
| No `.spikes/` directory | **Pass** with warning (clean slate) |
| Empty `feedback.jsonl` | **Pass** with warning (no feedback) |
| All spikes on ignored pages | **Pass** (count = 0) |
| Unsupported platform | **Fail** with clear error message |

## Supported Platforms

| OS | Architecture | Release Asset |
|----|--------------|---------------|
| Linux | x64 | `spikes-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | ARM64 | `spikes-aarch64-unknown-linux-gnu.tar.gz` |
| macOS | x64 | `spikes-x86_64-apple-darwin.tar.gz` |
| macOS | ARM64 (M1/M2) | `spikes-aarch64-apple-darwin.tar.gz` |

Windows is not currently supported. Use WSL or open an issue if you need native support.

## The Philosophy

> "Feedback is a gift. But sometimes gifts are cursed."

The Spikes Feedback Gate enforces a simple contract: **don't ship known problems**. It's not about perfection — it's about awareness. Set your threshold, ignore what you can't fix, and ship with confidence.

## License

MIT — Use it, fork it, make it yours.

---

Part of [Spikes](https://spikes.sh) — feedback that talks back.
