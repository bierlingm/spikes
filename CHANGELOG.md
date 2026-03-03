# Changelog

All notable changes to Spikes will be documented in this file.

## [0.3.0] - 2025-03-03

### Added

**MCP Server**
- `spikes mcp serve` - MCP server for agent integration (rmcp SDK, stdio transport)
- 3 tools: `list_spikes`, `get_spike`, `add_spike` for remote feedback management

**Context Exports**
- `--format cursor-context` - Export optimized for Cursor IDE context
- `--format claude-context` - Export optimized for Claude context window

**GitHub Action**
- `action/` directory with composite action for CI integration
- CI gating on feedback: block builds if unresolved spikes exist

**CI / Testing**
- Test workflow for PRs and main branch pushes
- Secret validation in Pages deploy workflow

**Documentation**
- MCP integration guide
- Complete CLI reference
- Self-hosting guide

### Technical

- MCP server uses rmcp SDK 0.17 with stdio transport
- Context exports structured for LLM consumption
- CI action provides `spikes-feedback` output for workflow conditionals

## [0.2.0] - 2024-12

### Added

**Authentication & Identity**
- `spikes login` - Magic link authentication via CLI
- `spikes logout` - Clear stored credentials
- `spikes whoami` - Show current authenticated user
- Secure token storage in OS keychain/keyring

**Sharing (Hosted Service)**
- `spikes share <dir>` - Upload project to spikes.sh
- `spikes shares` - List all hosted shares
- `spikes unshare <slug>` - Remove hosted share
- Free tier with rate limits, Pro tier with Stripe billing

**Spike Management**
- `spikes delete <id>` - Delete feedback by ID
- `spikes resolve <id>` - Mark spike as resolved
- `spikes list --unresolved` - Filter unresolved feedback

**Billing**
- `spikes billing` - Open Stripe Customer Portal
- `spikes upgrade` - Subscribe to Pro tier
- `spikes usage` - Check current usage limits

**Widget Improvements**
- Duplicate spike detection (prevent double-submits)
- Review button UX enhancements
- Toast notifications for user feedback
- `data-admin` attribute for review mode toggle

**Security**
- CORS hardening with configurable origin allowlist
- Path traversal protection in `spikes serve`
- Rate limiting on API endpoints

### Changed

**CLI**
- Removed TUI dashboard (`spikes dashboard` now shows help message)
- Actionable error messages for all HTTP errors
- `SPIKES_API_URL` env var for custom endpoints
- `.gitignore` handling in `spikes init`

**Worker**
- Modularized into handlers/middleware/utils structure
- Zod validation for all API inputs
- Pagination support on list endpoints

### Removed

- ratatui and crossterm dependencies (TUI code eliminated)
- Dashboard TUI—CLI is the primary interface

### Technical

- Comprehensive test infrastructure with serial_test for env var tests
- Validation framework with scrutiny and user-testing synthesis
- Foundation, reliability, identity-auth, monetization, and misc-2 validation passes

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
