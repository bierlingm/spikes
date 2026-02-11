# Changelog

All notable changes to Spikes will be documented in this file.

## [0.1.0] - 2024-02-11

### Added

**Widget (8KB gzipped)**
- Drop-in `<script>` tag for any HTML page
- Page-level feedback with ratings (love/like/meh/no) and comments
- Element spike mode: click to capture specific elements with CSS selectors
- Reviewer identity: prompts for name on first spike, persists across sessions
- localStorage persistence (works offline, on file://, localhost, https)
- Configurable via data attributes: `data-project`, `data-position`, `data-color`, `data-endpoint`

**CLI**
- `spikes init` - Initialize .spikes/ directory
- `spikes list` - List all feedback with filtering (--page, --reviewer, --rating, --json)
- `spikes show <id>` - Show single spike detail
- `spikes export` - Export as JSON, CSV, or JSONL
- `spikes hotspots` - Show elements with most feedback
- `spikes reviewers` - List all reviewers
- `spikes inject <dir>` - Add widget to HTML files
- `spikes serve` - Local dev server with POST endpoint
- `spikes deploy cloudflare` - Scaffold Cloudflare Worker + D1 backend
- `spikes pull` / `spikes push` - Sync with remote endpoint
- `spikes dashboard` - Interactive TUI with filtering and detail view

**Dashboard**
- Static HTML dashboard reads from localStorage
- Filter by page, reviewer, rating
- Export as JSON or CSV
- Served at /dashboard when using `spikes serve`

### Technical

- Widget: Vanilla JS, IIFE pattern, zero dependencies
- CLI: Rust with clap, serde, axum, ratatui
- All commands support `--json` for agent/script consumption
