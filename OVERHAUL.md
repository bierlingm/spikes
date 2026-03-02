# Spikes Platform Overhaul

**Date:** 2026-03-02
**Status:** Strategic plan — no code changes in this document
**Scope:** Both repos (spikes + spikes-hosted), all 26 open issues

---

## Executive Summary

**Who is this for:** The solo builder at 2am who needs someone to look at this. The freelancer who needs to look professional before a client meeting. The small team losing feedback in Signal threads.

**The moat isn't technical — it's cultural.** The widget is trivially cloneable. The moat is workflow integration (MCP server, context exports, CLI ergonomics), personality (Spike the hedgehog, the zine energy, the community wall), and the refusal to be yet another SaaS tool. Brand/community is weak early but strong long-term — invest in it now.

**Revenue model: explore, don't lock in.** Six models are analyzed below — from punk zine (free forever, merch + patronage) to Radiohead (pay what you feel) to Sublime Text (one-time license) to 37signals ONCE (free self-host, pay for hosted). The recommended hybrid: pay-what-you-feel for hosted, one-time license for agencies, optional patronage for the community. Every payment should feel like solidarity, not a transaction.

**What's broken technically:** Auth system is fundamentally broken (bearer tokens *are* identity — no recovery, no rotation, no Stripe linkage). Passwords use unsalted SHA-256. No rate limiting. Pro tier enforcement doesn't work.

**The priorities:** Security first, then a minimal account system (magic links, not passwords), then make the monetization plumbing work, then the personality layer that makes this a *place* and not just a product. Don't build the TUI — remove it. Don't build collaborative features. Focus on agent integration (MCP server, context exports) for defensibility and the personality/community layer for love.

**Timeline:** 8 weeks across 6 phases for the technical overhaul. The personality and business model work is a parallel, ongoing track.

---

## Part 1: The Business Case

### A. Who Is This Actually For?

Not segments. People. Humans at specific moments of need.

**The Solo Builder at 2am.** They've been vibe-coding with Claude Code for six hours. The prototype works. They're proud of it. They need someone to look at it before they go to bed or they'll dream about CSS bugs. Their options are either too heavy (set up a staging site, invite people, manage permissions) or too broken (screenshots with arrows drawn in Preview). They want to say "look at this" and have the looking actually work. *They pay $0-29, depending on whether they remember to pay at all. They're evangelists, not revenue.*

**The Freelancer Before a Client Meeting.** It's Thursday. The client meeting is Friday. They've built three versions of the checkout flow and need the client to pick one without saying "I don't like it" with no specifics. They've tried Google Docs with screenshots pasted in (client scrolls past), Figma comments (client doesn't have Figma), and email threads with numbered lists ("Issue #7: the button"). They need Spikes because `clientname.freelancer.spikes.sh` lands differently than "open this HTML file on your computer." *They pay $19-29/month without thinking about it, because it's cheaper than one hour of rework.*

**The Small Team (2-5 People).** Two founders. One designer, one developer. They're in a Signal group sharing screenshots and losing track of what was already fixed. They need the minimal viable coordination tool — not a project management system, just a way to say "this thing, right here, change it" and have that statement survive longer than a chat message. *They pay $29/month for the team, or one person pays and shares the login (honor system — we don't care).*

**The Agency.** They've got Jira, Figma, real designers. But the feedback loop between design and dev is still broken. They need Spikes as the bridge — designers spike the built prototype, devs pull structured data. *They pay $149 once, honor system for seat count, because they'd rather pay once than get billed forever for a utility.*

### B. Revenue Models — A Creative Exploration

We're not picking one. We're exploring the possibility space. Each model is argued with conviction because conviction is what makes a model work.

#### Model 1: The Zine Model (The Punk Option)

**What it is:** Spikes is free. Forever. Full stop. Revenue comes from merchandise, sponsorships, and people who just want the thing to exist.

**What it feels like:** Like walking into a DIY show and grabbing a zine off the table. The zine is free. If you want to support the band, buy a shirt. Or don't. The band plays either way.

**How it makes money:** Stickers, shirts, pins with the mascot. Sponsors on spikes.sh (hosting companies, dev tools that want the audience). Patreon for "insiders" who want to see the roadmap and vote on features. Consulting/integration services for agencies who need custom setups.

**What it says about Spikes:** We don't need your money. We need your use. We're building infrastructure for a community, not extracting rent from customers.

**What's risky:** Requires building a real community first. No revenue until you have fans, not users.

**What's delightful:** Nobody feels ripped off. Nobody debates whether it's "worth it." The question becomes "do I want this to exist?" instead of "is the price fair?"

#### Model 2: The Radiohead Model (Pay What You Feel)

**What it is:** Hosted version exists. You choose what to pay, with a suggested minimum of $19. Some pay $9. Some pay $200. Both are valid.

**What it feels like:** Like buying music directly from the artist. There's a suggested price, but you're not locked out if you can't pay it. No guilt trip. The artist trusts you.

**How it makes money:** Hosted convenience (`spikes share` uploads to our infrastructure, gives you a clean URL). Suggested tiers: $9 (it's fine), $19 (like it), $29+ (love it). The tiers don't gate features — they just exist as social signals.

**What it says about Spikes:** We trust humans to be fair.

**What's risky:** Revenue is unpredictable. Month-to-month variance could be extreme.

**What's delightful:** The checkout page is a celebration. "Pay what feels right" instead of "SELECT YOUR PLAN." The confirmation email isn't a receipt — it's a thank you note.

**The twist:** If someone pays more than $50, send them something physical. A handwritten note, a sticker, something real in the mail.

#### Model 3: The Sublime Text Model (Perpetual License)

**What it is:** One-time purchase. $49 for individuals, $149 for agencies. Use it forever. No subscription fatigue.

**What it feels like:** Buying a tool at the hardware store. You own it. It doesn't phone home to check if you're still paying. It doesn't get worse if you stop paying.

**What it says about Spikes:** Software can be a product, not a service.

**What's risky:** No recurring revenue means no predictable cash flow.

**What's delightful:** People *love* software they own. Sublime Text has fans, not just users. The lack of subscription is a feature that gets talked about.

**The variant:** A "support renewal" option after a year. "Want to support continued development?" No pressure, no features locked. Just a human asking other humans for help.

#### Model 4: The 37signals ONCE Model (Free + Open, Pay for Convenience)

**What it is:** Everything is MIT licensed and self-hostable. Free. The hosted version at spikes.sh is a convenience layer you pay for, not a feature gate.

**What it feels like:** Like Campfire or Writebook. The code is yours. We're just running it for you if you don't want to.

**What it says about Spikes:** We're not afraid of our users having power. We're confident enough in our execution that we'll survive even if everyone self-hosts.

**What's risky:** Hosted revenue might be low if self-hosting is too easy.

**What's delightful:** The hosted version can be *better* because we control the infrastructure. Faster, more reliable, zero setup. People pay for that improvement, not for access.

#### Model 5: The Patron Model (Community-Sustained)

**What it is:** Spikes is developed in public, funded by patrons who want it to exist. Like NPR, but for a feedback widget.

**What it feels like:** Being part of something. The tool exists because a community decided it should. Your payment isn't a transaction — it's a vote.

**How it makes money:** Monthly patrons ($5, $10, $25 tiers). No features gated. Annual "fundraiser" campaigns for specific features. Corporate sponsors.

**What's risky:** Requires building a real community with shared identity. High engagement burden.

**What's delightful:** The patron wall becomes a social space. People are proud to be on it. Monthly "state of the spikes" updates — what got built, what didn't, what's next.

#### Model 6: The Generous Free Model (Simple Paid Convenience)

**What it is:** Free tier is genuinely generous (1000 spikes, 5 shares, no time limit). Paid tier removes constraints and adds convenience.

**What it feels like:** Like Buttondown. The free tier isn't a trial — it's a real product. You pay when you grow into it, not because we crippled the free version.

**What's delightful:** Nobody feels bait-and-switched. The upgrade is obvious when you hit limits — no dark patterns needed.

#### Recommended Combination: The Hybrid Punk Model

**Core principle:** Free and open source forever. Revenue from three sources that feel different:

1. **Pay what you feel (hosted)** — Default to $19, minimum $9, no maximum. For individuals who want zero setup.
2. **One-time license (agencies)** — $149 once, whole team, forever. For businesses who want to budget once and forget.
3. **Patronage (community)** — Optional $5-25/month for people who want to support the project regardless of usage.

**Why this combination:** It covers different psychologies without forcing anyone into a subscription they don't want. It keeps the open source promise intact. Every payment is an act of solidarity, not a market transaction. The checkout flows should feel like that.

### C. The Personality Layer

This is not marketing. This is product surface. The character of Spikes is something you interact with, not something you read about.

#### Spike the Hedgehog (Mascot Concept)

**Backstory:** Spike was a QA tester at a big enterprise feedback company. He got fired for being "too honest" in his bug reports. Now he runs an underground feedback collective. He believes in telling the truth, even when it's uncomfortable. He lives in a converted shipping container behind a coffee shop. He drinks too much espresso. He cares deeply about whether buttons are actually clickable.

**Visual:** A slightly grumpy but well-meaning hedgehog. Not cute-cartoon. More like a character from an indie comic. Think: Tank Girl meets Animal Crossing.

**Voice:** Dry, slightly sarcastic, ultimately helpful. The kind of tool that says "Saved" instead of "Your feedback has been successfully submitted to our cloud infrastructure!"

**Where he appears:**
- Loading states: "Spike is foraging for your data..."
- Error pages: "Spike knocked over the server. He's not sorry. We're fixing it."
- Empty states: "Nothing here yet. Go break something so Spike has something to do."
- CLI messages: `Spike says: you might want to run 'spikes list' first.`

#### The Sharing Experience

When someone clicks a spikes.sh link, it should feel like entering a space, not opening a tool.

**Sane default (zero config):** Page loads with a brief, handmade-feeling reveal. Dismissible widget intro: "Click the / to leave feedback. Click anything on the page to be specific. Spike will handle the rest." Toasts feel human: "Spike caught that." Errors feel human: "Spike tripped. Try again?"

**Optional configuration:** Sender can set a custom greeting ("Patricia wants your honest thoughts. Don't hold back." or "This is a rough draft. Be gentle."). Theme/personality customization wizard. Choice of how much personality the recipient sees.

**Full experience (opt-in):** Brief animation on first visit. Character introduction. "You've been invited to spike this." The works — for people who want to make the sharing experience itself memorable.

#### The Vanity Wall (spikes.sh/community)

This is the community surface. Not a marketing page — a place.

- **The Contributors Wall** — Everyone who opened a PR, filed a thoughtful issue, or wrote about Spikes. Not just code — any contribution counts.
- **The Supporters Gallery** — People who paid. Optional public names or anonymous hashes. No dollar amounts.
- **Most Hilarious Feedback** — Curated (with permission) funny spikes people have left. "The button looks like it regrets its life choices." Content marketing that doesn't feel like marketing.
- **Spike's Log** — Development updates in character. "This week I learned that rate limiting is important after someone tried to store their entire MP3 collection as spikes."
- **The Guestbook** — Anyone can sign. Old-web energy. "Was here. Love the tool. —Sarah from Portland"

#### Small Delights Throughout

**CLI messages:**
```
$ spikes share ./mockups/
Bundling... done
Uploading... done

Your mockups are live at:
https://spikes.sh/s/governance-x7k9m

Spike says: share this link wisely. Or recklessly. Your call.
```

**Error pages:** 404: "Spike searched everywhere. He even checked under the couch. Nothing." 500: "Spike broke something. He's pretending it wasn't him." Rate limited: "Spike needs a coffee break."

**Empty states:** No spikes yet: "Quiet in here. Too quiet." No shares yet: "Your shares would go here, if you had any. No pressure."

### D. What Features Drive Love (Not Just Conversion)

We're not optimizing for conversion. We're optimizing for "holy shit, you gotta try this."

**The "It Just Works" Factor.** `spikes share ./mockups/` works on first try. Because everything else requires setup. Figma requires accounts. Netlify requires config. This just works. Like finding a shortcut you didn't know existed.

**The "They Actually Listened" Factor.** The CLI outputs JSON. Every command. No exceptions. Most tools treat CLI users as second-class. We treat them as primary.

**The "Wait, It Does What?" Factor.** Element selection captures CSS selectors automatically. Turns "that button on the left" into `.pricing-card .btn-primary` without anyone opening DevTools.

**The "They Didn't Have To" Factor.** The widget works on `file://` URLs. Nobody supports `file://` anymore. We do. Because we care about your weird workflow.

**The "This Is Actually Fun" Factor.** Spike the mascot, the voice, the personality layer. Most dev tools are joyless. Spikes feels like it was built by someone who cares about more than just functionality.

**The "I Want To Tell People" Test.** If someone uses Spikes and doesn't tell at least one person about it, we've failed. What creates evangelists: surprising them, delighting them, saving them time, making them look good, respecting their values.

### E. Competitive Positioning

| Tool | What They Do | Why Spikes Instead |
|------|--------------|-------------------|
| **Markup.io** | Visual feedback on live sites | Local-first, works on `file://`, CLI-native, agent-readable |
| **BugHerd** | Bug tracking with screenshots | Simpler, no ticketing overhead, JSON export |
| **Userback** | User feedback widgets | For internal review, not public feedback, developer-focused |
| **Hotjar** | Analytics + heatmaps | Gives you the *why*, not just the *where* |
| **Figma comments** | Design tool feedback | For the built prototype, not the design file |
| **GitHub Issues** | Structured bug tracking | For the messy pre-bug phase, visual and fast |

**The real competition is doing nothing.** Most people don't use a feedback tool. They text screenshots. They send emails with descriptions. They try to remember what the client said. Spikes wins because it's faster than doing nothing (one script tag, no setup), better than using the wrong tool (designed for this specific problem), and doesn't require changing workflows.

**Why someone would choose Spikes over just texting a screenshot:** You take a screenshot. Open Preview. Draw an arrow. Export. Paste into a message. Type "the thing I circled." They reply "which thing?" You explain. By then you've forgotten what you meant. **With Spikes:** Click the /. Click the thing. Type "this needs more padding." Done. The selector is captured automatically. The context is preserved. The feedback is structured. That's why.

**The positioning (punk version):** "Spikes is how you get feedback without losing your mind. It's a widget, a CLI, and an attitude. It works on `file://` because we know you don't always have a server running. It outputs JSON because we know you're probably piping it somewhere. It's open source because we don't believe in holding your work hostage. And yes, there's a hedgehog."

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
