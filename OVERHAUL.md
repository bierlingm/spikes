# Spikes Platform Overhaul

**Date:** 2026-03-02
**Status:** Strategic plan — no code changes in this document
**Scope:** Both repos (spikes + spikes-hosted), all 26 open issues

---

## Executive Summary

**Who pays:** Freelancers showing work to clients. Solo AI builders are evangelists, not payers.

**The moat:** Workflow integration depth—MCP server, context exports, and CLI/agent ergonomics. Not the widget (trivially cloneable), not the data (easily exportable).

**What's broken:** The auth system is fundamentally broken—bearer tokens *are* identity, so there's no recovery, no rotation, and no Stripe linkage. Passwords use unsalted SHA-256. No rate limiting. Pro tier enforcement doesn't actually work.

**The priorities:** Security first (unsalted passwords, path traversal, no rate limiting). Then a minimal account system (magic links, not passwords—solo developers hate password management). Then the monetization plumbing actually works. Everything else is polish.

**Timeline:** 8 weeks across 6 phases. Don't build the TUI dashboard—remove it. Don't build collaborative features—out of scope. Focus on agent integration (MCP server, context exports) for defensibility.

---

## Part 1: Business Case Analysis

### 1. Who Actually Pays for This?

| Segment | Size | Willingness to Pay | Verdict |
|---------|------|-------------------|---------|
| **Solo AI builders** (Claude Code, Cursor users) | Large | Low | Free tier evangelists |
| **Freelancers with client work** | Medium | **Medium** | **Primary conversion target** |
| **Small agencies (2-10 people)** | Small | Medium | Need Team tier |

**The honest assessment:** Solo builders reviewing their own work will pay $0-9 as appreciation, not $19/mo as subscription. The conversion target is freelancers who need to look professional sending `clientname.freelancer.spikes.sh` links instead of explaining "open this HTML file."

### 2. What's the Moat?

**The uncomfortable truth:** The widget is trivially cloneable. The CLI is nice but not defensible. Any competent developer could rebuild the core functionality in a weekend.

**Actual moats (ranked):**

| Moat | Strength | Time to Build |
|------|----------|---------------|
| **Workflow integration (CLI + agent context)** | Strong | 6-12 months |
| **MCP/AI coding tool integrations** | Strong | 6-12 months |
| **Hosted convenience + network effects** | Medium | 3-6 months |
| **Data accumulation over time** | Medium | 12+ months |
| **Brand/community** | Weak (early) | 24+ months |

**The lasting value** is in agent integration depth—MCP server (`spikes mcp serve`) and context export formats (`spikes export --format cursor-context`) create dependencies that are harder to replace than a simple data export. The hosted layer is monetizable but not defensible—it's a convenience tax, not a lock-in mechanism.

### 3. Recommended Pricing Model

| Tier | Price | Features |
|------|-------|----------|
| **Free** | $0 | 5 shares, 1000 spikes each, 50MB, random URL, badge required |
| **Pro** | $19/mo | Unlimited, custom subdomain, badge removable, password protection, webhooks, API access |
| **Team** | $29/mo | Everything in Pro + 3 seats, team dashboard, consolidated billing |
| **+Seats** | $8/mo each | Additional team members |

**The badge is the primary conversion lever.** For freelancers showing work to clients, having a third-party badge on their deliverables feels unprofessional. Lead with: "Remove the 'Powered by Spikes' badge and share with a custom subdomain for a fully branded client experience."

**Webhooks are the workflow upgrade trigger**—serious teams need CI integration.

### 4. What Features Drive Conversion?

**Must-have for Pro tier:**
1. Custom subdomain
2. Badge removal
3. Password protection

**Should-have:**
4. Webhooks (CI integration)
5. Unlimited shares

**Don't build:** Real-time collaboration, rich text comments, screenshot capture, analytics dashboards, mobile apps. These are scope creep for a tool serving solo developers and small teams.

### 5. Competitive Positioning

| Tool | Price | Spikes Differentiation |
|------|-------|------------------------|
| **Markup.io** | $25+/mo | Spikes is local-first, CLI-native, agent-readable |
| **BugHerd** | $41+/mo | Spikes is simpler, no ticketing overhead |
| **Userback** | $79+/mo | Spikes is for internal/team review, not public feedback |
| **Hotjar** | $39+/mo | Spikes is developer-focused, not analytics |

**The wedge:** Spikes is built for the AI-assisted development workflow. Zero-dependency widget (~8KB), works on `file://` and localhost, CLI with JSON output, CSS selector capture, self-hostable, MIT licensed.

---

## Part 2: Architectural Assessment

### What's Working Well (Preserve These)

1. **Local-first JSONL storage** — Human-readable, append-only, git-friendly, zero dependencies
2. **Zero-dependency widget** — Single file, no build step, broad compatibility
3. **Agent-friendly JSON output** — `--json` flag on every command enables composability
4. **Clean data model** — Strongly typed Rust enums, consistent field naming
5. **Cloudflare-native backend** — D1 + R2 + Workers, low operational cost (~$5/month)

### What's Fundamentally Broken (Requires Redesign)

#### 2.1 The Bearer-Token-as-Identity System (Critical)

The current auth model uses raw bearer tokens (UUIDs) as both authentication *and* identity:

```typescript
// The token IS the owner identity
await env.DB.prepare(
  'INSERT INTO shares (id, slug, owner_token, ...) VALUES (?, ?, ?, ...)'
).bind(shareId, slug, ownerToken, ...);
```

**Consequences:**
- No account recovery—lose your token, lose all your shares permanently (spikes-hosted#12)
- No token rotation—can't invalidate a compromised token without losing access
- No multi-device—each "login" is a different random token with no linking
- No Stripe integration path—webhook updates `users.tier` by `stripe_customer_id`, but bearer tokens aren't linked to user records

**Resolution:** Move to minimal accounts with magic links (see Part 3, Phase 2).

#### 2.2 Data Integrity Gaps (High)

- No foreign keys: `spikes.share_id` has no FK constraint to `shares(id)` (spikes-hosted#5)
- Denormalized tier: `shares.tier` duplicates `users.tier`, creating two sources of truth (spikes-hosted#5)
- Drifting counters: `spike_count` is manually incremented; if the UPDATE fails, count becomes wrong (spikes-hosted#5)
- Orphaned data risk: R2 cleanup isn't atomic with D1 deletion (spikes-hosted#15)

#### 2.3 Security Issues (Critical)

| Issue | Location | GitHub Issue |
|-------|----------|--------------|
| Unsalted password hashing | `worker/src/index.ts` | hosted#1 |
| No rate limiting | Entire worker | hosted#2 |
| Path traversal in R2 keys | `worker/src/index.ts` | hosted#8 |
| Timing attack in password compare | `worker/src/index.ts` | hosted#1 |
| No request body validation | `worker/src/index.ts` | hosted#7 |
| XSS in subdomain listing | `worker/src/index.ts` | hosted#14 |
| Overly permissive CORS | `cli/src/commands/serve.rs` | spikes#2 |

**Password hashing (unsalted SHA-256):**
```typescript
// VULNERABLE TO RAINBOW TABLES
const hashBuffer = await crypto.subtle.digest('SHA-256', data);
```

#### 2.4 Pro Tier Enforcement Gap (High)

The code has Pro tier checks but they don't actually work—they always enforce free limits regardless of tier.

**GitHub issue:** spikes-hosted#9

#### 2.5 Self-Hosted vs. Hosted Divergence (Medium)

The self-hosted template has drifted from the hosted backend. Creates maintenance burden: fixes in one don't propagate to the other.

### Key Architectural Decisions (Resolved)

| Decision | Resolution |
|----------|------------|
| **Auth method** | **Magic links** — lower friction than passwords for solo developers, no password hashing complexity, recovery built-in |
| **Token storage** | Per-user in `~/.config/spikes/auth.toml`, with `SPIKES_TOKEN` env var override |
| **Local storage format** | Keep JSONL (zero dependencies, human-readable), add `--format sqlite` opt-in for high-volume scenarios |
| **Widget communication** | Both: direct to hosted for static shares, proxy through CLI server for local development |
| **Self-hosted template** | Consolidate into single repo with exports—hosted repo becomes source for both |

---

## Part 3: Prioritized Overhaul Plan

### Summary Table

| Phase | Goal | Complexity | Duration | Dependencies |
|-------|------|------------|----------|--------------|
| 0 | Test infrastructure + critical security | XL | 2 weeks | None |
| 1 | Data integrity, validation, pagination | L | 1.5 weeks | Phase 0 |
| 2 | Account model, auth, Stripe linkage | L | 1.5 weeks | Phase 1 |
| 3 | UX polish, documentation | M | 1 week | Phase 2 |
| 4 | Monetization completeness | M | 1 week | Phase 2 |
| 5 | Growth integrations (MCP, GitHub Action) | M | 1 week | Phase 3 |
| **Total** | | | **8 weeks** | |

---

### Phase 0: Foundation — Test Infrastructure & Critical Security

**Goal:** The system is testable, CI-enforced, and free of critical security vulnerabilities. Nothing else ships until this is solid.

**Scope:**
| Issue | Repo | Title | Severity |
|-------|------|-------|----------|
| #1 | spikes-hosted | Password hashing lacks salt | CRITICAL |
| #8 | spikes-hosted | R2 filename path traversal | CRITICAL |
| #10 | spikes-hosted | No test suite | HIGH |
| #3 | spikes | No automated tests | MEDIUM |
| #16 | spikes-hosted | No staging environment | MEDIUM |
| #14 | spikes-hosted | XSS in subdomain listing | HIGH |
| #2 | spikes | serve: path traversal + CORS | MEDIUM |

**Key Decisions:**
- Test framework: Vitest for Workers, standard Rust test + assert_cmd for CLI
- CI strategy: Staging Worker + D1 on every push; production deploy only on release tag
- Password hashing: PBKDF2-HMAC-SHA256 with 100k iterations
- Rate limiting: **D1-based sliding window** (consistent, testable; KV noted as future optimization)

**What to Build:**

1. **Security fixes:**
   - Replace SHA-256 with PBKDF2 using per-share random salts
   - Add constant-time comparison for password verification
   - Sanitize R2 filenames to prevent path traversal
   - HTML-escape all values in subdomain listing (XSS fix)
   - Fix CLI `serve` path traversal with `canonicalize()`, safe CORS defaults

2. **Test infrastructure:**
   - Vitest with Miniflare for local D1/R2 mocking
   - Coverage: spike submission, share creation, password hash/verify, bearer token auth, Stripe webhook verification
   - Rust unit tests for JSONL, config, export, inject
   - Integration tests for `init` → `inject` → `serve` → `list` workflow

3. **CI/CD:**
   - Staging Worker with isolated D1/R2 bindings
   - Deploy to staging on every push to main
   - Deploy to production only on release tags
   - Test gate must pass before any deploy
   - Document `wrangler rollback` procedure

---

### Phase 1: Core Reliability — Data Integrity, Validation & Pagination

**Goal:** The system handles errors gracefully, data stays consistent, and API consumers can't accidentally break things.

**Scope:**
| Issue | Repo | Title | Severity |
|-------|------|-------|----------|
| #5 | spikes-hosted | Data integrity: no FKs, denormalized tier | HIGH |
| #3 | spikes-hosted | No pagination on GET /spikes | HIGH |
| #7 | spikes-hosted | No request body validation | MEDIUM |
| #4 | spikes-hosted | Client-provided spike IDs | MEDIUM |
| #6 | spikes-hosted | Webhook delivery fire-and-forget | MEDIUM |
| #15 | spikes-hosted | R2 cleanup not atomic | MEDIUM |
| #2 | spikes-hosted | No rate limiting | HIGH |
| #13 | spikes-hosted | No cache headers on shares | MEDIUM |
| #8 | spikes | Pull/push error handling is poor | MEDIUM |

**What to Build:**

1. **Data model repair:**
   - Add foreign key constraints (D1 supports them)
   - Remove denormalized `shares.tier` column—always look up tier from owner via JOIN
   - Replace `spike_count` counter with `COUNT(*)` queries
   - Always generate spike IDs server-side with `crypto.randomUUID()`

2. **Request validation (Zod):**
   - Validate all POST bodies with clear error messages
   - Standardize error shape: `{ "error": "human message", "code": "MACHINE_CODE" }`

3. **Rate limiting (D1-based):**
   - POST /spikes: 60/min per IP
   - Password attempts: 5/min per slug per IP
   - POST /shares: 10/min per bearer token
   - Return 429 with `Retry-After` header

4. **Webhook reliability:**
   - Add HMAC-SHA256 signature headers
   - Validate webhook URLs (HTTPS only, no internal IPs)
   - One retry after 5s using `waitUntil()`
   - Log all delivery attempts

5. **Cleanup flow:**
   - Delete D1 records before R2 files
   - Periodic cleanup job for orphaned R2 files

6. **CLI improvements:**
   - Map HTTP errors to actionable messages ("401 → run `spikes login`")
   - Add `--verbose` flag

---

### Phase 2: Identity & Auth — Account Model, Token Lifecycle & Stripe Linkage

**Goal:** Users have recoverable accounts, tokens can be rotated, and Stripe subscriptions link to actual user identities.

**Scope:**
| Issue | Repo | Title | Severity |
|-------|------|-------|----------|
| #12 | spikes-hosted | Bearer token system: no recovery, no rotation | HIGH |
| #11 | spikes-hosted | Stripe webhook incomplete | HIGH |
| #9 | spikes-hosted | Pro tier limits not enforced correctly | HIGH |
| #1 | spikes | Token stored in plaintext | HIGH |

**What to Build:**

1. **Minimal user accounts (magic links):**
   - `POST /auth/register` — email verification
   - `POST /auth/login` — magic link sent to email
   - `POST /auth/rotate-token` — invalidate old, issue new
   - Link bearer tokens to user accounts in database

2. **Secure token storage:**
   - Tokens stored in `~/.config/spikes/auth.toml` (XDG/platform-native)
   - Support `SPIKES_TOKEN` environment variable as override
   - Remove token from `.spikes/config.toml`
   - `spikes init` adds `.spikes/` to `.gitignore`

3. **Stripe integration completion:**
   - Handle `customer.subscription.created/updated/deleted` with UPSERT
   - Handle `customer.deleted` to clean up
   - Handle `invoice.payment_failed` for grace period
   - Store processed Stripe event IDs to prevent replay

4. **Pro tier enforcement:**
   - Check user's tier from `users` table before enforcing limits
   - Bypass free limits for pro users
   - 5-share limit only for free tier

---

### Phase 3: UX & Polish — Widget Improvements, Error Handling & Documentation

**Goal:** The widget feels reliable, review mode is discoverable, and users can actually figure out how to use the system.

**Scope:**
| Issue | Repo | Title | Severity |
|-------|------|-------|----------|
| #4 | spikes | Widget UX gaps: save confirmation, quota, dedup | MEDIUM |
| #7 | spikes | Review mode undiscoverable | LOW |
| #9 | spikes | Incomplete docs | LOW |
| #10 | spikes | Widget z-index + drag | LOW |
| #6 | spikes | No edit/delete for spikes | LOW |

**What to Build:**

1. **Widget improvements:**
   - Toast notification on save ("Spike saved!")
   - localStorage quota handling with graceful degradation
   - Duplicate detection: skip if identical (selector + reviewer + comment) within 30 seconds
   - z-index 2147483647, `data-offset-x/y` attributes for positioning

2. **Review mode discoverability:**
   - Add "Review" button when `data-admin="true"` is set
   - Document review mode with screenshots

3. **CLI spike management:**
   - `spikes delete <id>` — remove spike by ID
   - `spikes resolve <id>` — mark as resolved
   - `spikes list --unresolved` filter

4. **Documentation:**
   - Complete widget `data-*` attribute reference
   - CLI command reference with all flags
   - Self-hosting setup guide

---

### Phase 4: Monetization Infrastructure — Pro Tier Enforcement & Billing Portal

**Goal:** The business model is technically complete—users can upgrade, limits are enforced, and billing is automated.

**Scope:** No new issues; completes infrastructure from Phase 2.

**What to Build:**

1. **Billing portal:**
   - `GET /billing/portal` endpoint for Stripe Customer Portal
   - `spikes billing` CLI command opens portal in browser

2. **Usage visibility:**
   - `GET /usage` endpoint returning spike count and limit
   - `spikes usage` CLI command
   - Usage indicator in `spikes shares` output

3. **Limit enforcement:**
   - Clear 429 errors with upgrade CTA when limits hit
   - `spikes upgrade` CLI command opens Stripe checkout

4. **Share provisioning:**
   - Set `max_spikes` based on owner's tier
   - Propagate tier changes to existing shares

---

### Phase 5: Growth Features — MCP Server, GitHub Actions & Context Export

**Goal:** Spikes becomes a first-class citizen in agent workflows and CI pipelines.

**What to Build:**

1. **MCP server:**
   - `spikes mcp serve` command starts Model Context Protocol server (stdio transport)
   - Expose tools: `get_spikes`, `get_element_feedback`, `get_hotspots`

2. **Context export formats:**
   - `spikes export --format cursor-context` for `.cursor/feedback.md`
   - `spikes export --format claude-context` for `CLAUDE.md`
   - Include blocking feedback, hotspots, element-specific issues

3. **GitHub Action:**
   - Publish `spikes-action` that fails builds when negative feedback exists
   - Configurable threshold, ignore paths, resolution requirements

4. **Remove TUI dashboard:**
   - **Recommendation:** Remove the `spikes dashboard` stub command and ratatui/crossterm dependencies
   - The HTML dashboard + CLI `--json` output covers the use cases
   - Target market (solo developers using agents) doesn't need a TUI

---

### Deferred / Not Worth Building

| Feature | Reason |
|---------|--------|
| **TUI dashboard** | Duplicates HTML dashboard; adds heavy ratatui dependency; target market prefers web or CLI --json output. **Action:** Remove the stub command. |
| **Widget edit/delete UI** | Adds complexity; CLI `delete`/`resolve` covers it |
| **Drag handle for widget** | Offset attributes solve positioning without drag complexity |
| **Real-time sync / WebSockets** | Polling or manual pull/push is sufficient |
| **Screenshot capture** | Out of scope per shaping document |
| **Email notifications** | Webhook integration lets users build their own |
| **Versioning / history** | Append-only with resolve flag covers the need |
| **Mobile-specific UI** | HTML prototypes are typically desktop-reviewed |
| **i18n** | English-only sufficient for initial market |
| **Collaborative features** | Spike comments/threads are scope creep for a small tool |

---

## Part 4: Cross-Repo Consistency Plan

### 4.1 API Contract Between CLI and Hosted Backend

**Current state:** API is implicitly defined by the worker implementation. No versioning. Inconsistent error shapes. CLI sends token in query params for push operations (logs tokens in access logs).

**Required contract:**

**Versioning:**
- Add `Accept: application/vnd.spikes.v1+json` header support
- Default to v1 if no header (current behavior)

**Authentication:**
- Bearer token in `Authorization: Bearer {token}` header for all CLI operations
- Query param `?token=` **deprecated**—remove from push.rs

**Response shape (standardized):**
```typescript
// Success
{ "ok": true, ...data }

// Error
{ "error": "human message", "code": "MACHINE_CODE" }
```

**Error codes:** `AUTH_FAILED`, `SPIKE_LIMIT`, `SHARE_LIMIT`, `SIZE_LIMIT`, `NOT_FOUND`, `VALIDATION_ERROR`, `INTERNAL_ERROR`

### 4.2 Widget Version Syncing

**Current state:** CI downloads widget from GitHub raw content during deploy. No version pinning, no checksum validation.

**Required changes:**
1. Pin to release tag: `https://raw.githubusercontent.com/bierlingm/spikes/v${VERSION}/widget/spikes.js`
2. Add SHA-256 checksum verification (store expected hash in `widget/.checksum`)
3. Bundle widget into worker as base64 fallback
4. Add `?v={hash}` query param for cache-busting

### 4.3 Self-Hosted Template vs. Hosted Divergence

**Decision:** Controlled divergence with shared core.

The self-hosted template should **NOT** mirror `spikes-hosted` completely (no Stripe, no user accounts). The **core API surface** must be identical for CLI compatibility.

**Shared core (must match exactly):**
- Spike schema and validation
- Share create/list/delete endpoints
- Response shapes
- CORS behavior
- Widget injection mechanism

**Hosted-only features (template excludes):**
- Users table and user management
- Stripe webhooks
- Pro tier enforcement
- Subdomain routing
- Password protection

**Action:** Create shared core module in `spikes-hosted/worker/src/core/` and copy/symlink to template at build time.

### 4.4 Shared Types/Schemas

**Current problems:**
- Field name mismatch: `project_key` (Rust) vs `project` (TS)
- Type mismatch: Rust sends objects, D1 stores JSON strings
- Widget sends camelCase, D1 stores snake_case

**Solution:** Create `spikes-hosted/worker/src/schema.ts` with Zod schemas as single source of truth. Maintain parallel Rust types with comments indicating source of truth.

---

## Part 5: Mission Specifications

Each mission is a self-contained implementation prompt. Missions proceed in phase order with dependencies noted.

---

### Phase 0: Foundation

**Objective:** Establish test infrastructure, secure CI/CD pipeline, and fix critical security vulnerabilities.

**Success Criteria:**
1. CI pipeline runs tests on every PR
2. Staging environment auto-deploys
3. Passwords use PBKDF2 with random salt, constant-time comparison
4. No path traversal vulnerabilities
5. Basic rate limiting on spike creation
6. XSS in subdomain listing eliminated

**Key Files:**
- `spikes-hosted/worker/src/index.ts` — password hashing, R2 upload
- `spikes-hosted/worker/schema.sql` — add salt column
- `spikes/cli/src/commands/serve.rs` — path traversal fix
- `spikes-hosted/.github/workflows/deploy.yml` — CI/CD overhaul

**What to Build:**
1. Test infrastructure (Vitest + Miniflare)
2. CI/CD with staging/production separation
3. PBKDF2 password hashing with migration
4. R2 filename sanitization
5. D1-based rate limiting
6. HTML-escaping for subdomain listing

**What NOT to Do:**
- No new features (webhooks, TUI, etc.)
- No API response shape changes
- No new dependencies without security review

---

### Phase 1: Core Reliability

**Objective:** Fix data integrity, add validation, implement pagination.

**Success Criteria:**
1. Foreign key constraints added, tier denormalization removed
2. GET /spikes paginated (default 100, max 1000)
3. All POST bodies validated with Zod
4. Spike IDs always server-generated
5. Rate limiting active on sensitive endpoints
6. Webhooks have HMAC signatures, URL validation, one retry
7. Cache headers on share content
8. CLI pull/push reports partial failures

**What to Build:**
1. Zod validation layer
2. Cursor pagination with `?cursor=` and `?limit=`
3. FK constraints and `COUNT(*)` for spike counts
4. Server-side UUID generation for spikes
5. Webhook HMAC signatures and retry logic
6. CLI error message mapping

---

### Phase 2: Identity & Auth

**Objective:** Implement minimal account system with magic links.

**Success Criteria:**
1. Users can register with email verification
2. Bearer tokens linked to user accounts
3. Token rotation works
4. Account recovery via email
5. Stripe customer ID linked to user
6. Pro tier enforcement works

**Key Decisions (Already Made):**
- Keep simple bearer tokens (no JWT, no OAuth)
- **Magic links** (not passwords)—lower friction for target market
- Token storage in `~/.config/spikes/auth.toml`

**What to Build:**
1. User-token linking with `user_tokens` table
2. Auth endpoints: `/auth/register`, `/auth/login`, `/auth/rotate-token`
3. Pro tier enforcement via user lookup
4. CLI token management: `login`, `logout`, `whoami`
5. Stripe webhook UPSERT handling

**What NOT to Do:**
- No OAuth (Google/GitHub login)
- No password-based login (use magic links)
- No session cookies (stateless bearer tokens only)

---

### Phase 3: UX & Polish

**Objective:** Improve widget UX, add CLI spike management, complete documentation.

**Success Criteria:**
1. Widget shows save confirmation toast
2. Widget handles localStorage quota gracefully
3. No duplicate spikes within 30-second window
4. `spikes delete <id>` and `spikes resolve <id>` work
5. Review mode accessible via widget UI
6. All widget attributes documented

**What to Build:**
1. Widget toast, quota handling, dedup logic
2. Review mode button (visible with `data-admin="true"`)
3. CLI `delete` and `resolve` commands
4. Widget z-index fix, offset attributes
5. Complete documentation (widget attributes, CLI reference, self-hosting guide)

---

### Phase 4: Monetization Infrastructure

**Objective:** Complete billing portal, usage tracking, limit enforcement.

**Success Criteria:**
1. Stripe Customer Portal accessible via API and CLI
2. Users can self-manage subscriptions
3. Usage visible via `spikes usage`
4. Limits enforced with clear upgrade prompts
5. Tier changes propagate to existing shares

**What to Build:**
1. `GET /billing/portal` endpoint and `spikes billing` CLI command
2. `GET /usage` endpoint and `spikes usage` CLI command
3. Limit enforcement with upgrade CTAs
4. `spikes upgrade` command
5. Pro feature enforcement (password protection, webhooks)

---

### Phase 5: Growth Features

**Objective:** Add MCP server, context exports, GitHub Action.

**Success Criteria:**
1. MCP server exposes `get_spikes`, `get_element_feedback`, `get_hotspots`
2. Agents can query Spikes via MCP
3. `spikes export --format cursor-context` produces valid markdown
4. `spikes export --format claude-context` produces valid markdown
5. `spikes-action` GitHub Action published
6. **TUI stub removed** (not built)

**What to Build:**
1. MCP server (`spikes mcp serve`, stdio transport)
2. Context export formats for Cursor and Claude
3. GitHub Action for CI gating
4. **Remove `spikes dashboard` stub command and ratatui/crossterm dependencies**

**What NOT to Do:**
- Do NOT build the TUI dashboard (remove instead)
- Do NOT add collaborative features (spike comments/threads)
- Do NOT add real-time collaboration (WebSockets)

---

### Issue Mapping

| Issue | Phase | Mission |
|-------|-------|---------|
| spikes#1 (token plaintext) | Phase 2 | Identity & Auth |
| spikes#2 (path traversal) | Phase 0 | Foundation |
| spikes#3 (no tests) | Phase 0 | Foundation |
| spikes#4 (widget UX gaps) | Phase 3 | UX & Polish |
| spikes#5 (TUI unimplemented) | Phase 5 | Growth Features — **Remove, don't build** |
| spikes#6 (spike management) | Phase 3 | UX & Polish |
| spikes#7 (review mode undiscoverable) | Phase 3 | UX & Polish |
| spikes#8 (pull/push errors) | Phase 1 | Core Reliability |
| spikes#9 (docs incomplete) | Phase 3 | UX & Polish |
| spikes#10 (widget z-index) | Phase 3 | UX & Polish |
| spikes-hosted#1 (unsalted passwords) | Phase 0 | Foundation |
| spikes-hosted#2 (rate limiting) | Phase 0 | Foundation |
| spikes-hosted#3 (pagination) | Phase 1 | Core Reliability |
| spikes-hosted#4 (client spike IDs) | Phase 1 | Core Reliability |
| spikes-hosted#5 (data integrity) | Phase 1 | Core Reliability |
| spikes-hosted#6 (webhook reliability) | Phase 4 | Monetization |
| spikes-hosted#7 (validation) | Phase 1 | Core Reliability |
| spikes-hosted#8 (R2 path traversal) | Phase 0 | Foundation |
| spikes-hosted#9 (Pro enforcement) | Phase 2, 4 | Identity + Monetization |
| spikes-hosted#10 (no tests) | Phase 0 | Foundation |
| spikes-hosted#11 (Stripe incomplete) | Phase 4 | Monetization |
| spikes-hosted#12 (bearer token system) | Phase 2 | Identity & Auth |
| spikes-hosted#13 (cache headers) | Phase 1 | Core Reliability |
| spikes-hosted#14 (XSS in subdomains) | Phase 0 | Foundation |
| spikes-hosted#15 (R2 cleanup atomicity) | Phase 1 | Core Reliability |
| spikes-hosted#16 (no staging/tests in CI) | Phase 0 | Foundation |

---

## Summary

**8 weeks. 6 phases. 26 issues resolved.**

The path forward is clear: fix the security holes first (unsalted passwords, path traversal, no rate limiting), then build the minimal account system with magic links (not passwords), then make the monetization plumbing actually work. Everything else is polish.

**Don't build the TUI.** Remove it instead. The HTML dashboard and CLI are sufficient for solo developers using AI tools.

**Don't build collaborative features.** Spike comments and real-time collaboration are scope creep. Stay focused on the core value: structured feedback that agents can act on.

**Invest in the MCP server and context exports.** That's where the defensibility lives—not in the widget, not in the hosted convenience layer, but in how deeply Spikes integrates into the AI-assisted development workflow.

The business case is viable: 5-8% free-to-Pro conversion, 50-100 subscribers at $19-29/mo, $1-2k MRR in 6 months. Not a rocket ship, but a sustainable indie SaaS serving a specific, growing niche.

Start with Phase 0. Nothing else ships until the foundation is solid.
