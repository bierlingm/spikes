# Feedback loop v2: closing all four arcs

Status: design draft, 2026-09-14. Written after the Prosser Home review round exposed that Spikes covers one arc of its own tagline. Companion to `docs/agent-readiness.md` (March 2026), which covers agent auth and pricing at the account level; this doc is about the loop itself.

## 1. What the Prosser round showed

The reviewer (a client) left six element comments over two days. Nothing told the builder. The agent that built the site could not read them: the repo's `.spikes/config.toml` held a revoked `sk_spikes_` key (401 `AUTH_FAILED`, no hint why), the local `feedback.jsonl` was five months stale with no "last pulled" stamp, no CLI was installed, and `spikes login` needs a browser. The comments were read and resolved with raw D1 queries. The reviewer still cannot see that anything happened. The "decisions" the reviewer was asked to make lived in a `mailto:` form and never arrived.

Every one of those failures is a missing arc, not a bug.

## 2. What exists today (verified against code, 2026-09-14)

Hosted worker (`spikes-hosted/worker`):

| Capability | State |
|---|---|
| `POST /spikes` | Public. Project key + server-enforced origin allowlist + per-IP and per-project rate limits (60/min each). Zod-validated. Ignores client `id`. |
| `GET /spikes`, `GET/PATCH/DELETE /spikes/:id` | Auth required (user token or `sk_spikes_` key). Owner-scoped. PATCH only toggles `resolved`. |
| `GET /me/projects/:key/spikes` | User bearer token only. API keys rejected with 403. Filters: page, rating, resolved. No URL-prefix filter. |
| `POST /projects` | User bearer token only. API keys rejected with 403. |
| API keys (`api_keys`) | Account-scoped (`user_id`), scopes `full/read/write`, optional expiry and monthly cap. No project scope. |
| Device-code login | Implemented (`device_codes`, `/auth/device*`) and wired as the default of `spikes login`. Still needs a browser once. |
| Webhooks | Per share only, Pro only, fire on spike create. Nothing per project. |
| Email | Magic links and recovery only. No notification on new spikes. |
| Replies, statuses beyond `resolved`, versions, questions | Do not exist in the schema. |

CLI (`cli/`): `pull` fetches `GET /spikes` with the repo's `[remote] token`, appends unseen ids to `feedback.jsonl`, no `--since`, no stamp. `resolve`, `delete`, `whoami`, `auth keys` exist. `spikes mcp` exposes nine tools (`get_spikes`, `get_element_feedback`, `get_hotspots`, `submit_spike`, `resolve_spike`, `delete_spike`, `create_share`, `list_shares`, `get_usage`) and authenticates with `SPIKES_TOKEN` only. `packages/spikes-mcp` is a thin npm wrapper around that binary.

Defects found while verifying:

- A revoked key returns `AUTH_FAILED` with no distinction from a typo, and the CLI keeps it in config forever.
- Any `GET` to `/spikes`, `/projects`, or `/me/projects` **with a query string** returns the marketing site's HTML with status 200; the same paths without a query reach the worker. Cause: those three are exact-match route patterns in `spikes-hosted/worker/wrangler.toml`, and the exact form does not match when a query string is present. Broken today: MCP `get_spikes` with any filter, and `spikes pull --from <share-url>`. Fix (uncommitted, not deployed): the three patterns are widened to `spikes.sh/spikes*`, `spikes.sh/projects*`, `spikes.sh/me/projects*`. Verify after deploy with `curl -H 'Authorization: Bearer x' 'https://spikes.sh/spikes?page=1'`, which should answer 401 JSON.
- The endpoint contract for embedders is documented in `docs/API.md` for shape, but not for CORS, origin allowlist semantics, or which fields the widget sends. The Prosser decision form was written by reading the worker source.
- The Prosser decision-form failure could not be reproduced: the exact browser payload, the CORS preflight, and the live `project.js` all check out from curl with `Origin: https://statecraft.systems`. Two test rows (comments containing "curl") now sit in project `yvs`; delete them from the dashboard. The deployed form now shows the HTTP status on failure, so the next real attempt is the diagnostic.

## 3. The four arcs

The tagline promises a loop. Today only arc 1 is a product.

1. **Capture** (exists). Element and page spikes with selector, box, URL, viewport, reviewer name. Missing: questions asked of the reviewer, version awareness, reviewer identity that survives across pages.
2. **Delivery to the builder** (missing). The agent that built the page should have new feedback in its working context within minutes, without a human relaying "my comments are on the link".
3. **Action** (manual). The agent replies per spike, marks it addressed in a version, or declines with a reason. Resolution should be a by-product of the work, not a database write.
4. **Closure for the reviewer** (missing). On the page where they left the comment, the reviewer sees status and reply, and gets one message when a new version is up listing what changed for them.

## 4. Roadmap: smallest shippable slice first

Each slice is independently useful and unblocks the next. Effort is a gut estimate in agent-days.

### Slice 1: headless agent access (fixes what broke this week)

Goal: a fresh agent in a repo with only `.spikes/config.toml` can list, resolve, and reply to feedback for one project, and can tell when its key is dead.

- **Project-scoped keys.** Add `project_key` (nullable) to `api_keys`. `spikes auth keys create --project <key> --scopes read,write` mints a key that can only touch that project. `GET/PATCH /me/projects/:key/spikes` and `/spikes/:id` accept such keys. Store it in the repo config; a leak exposes one project, not the account. Never expires unless revoked.
- **`spikes status`.** Calls `GET /me` with whatever credential is configured and prints: which credential (env, repo config, global auth), whether it works, and, for a 401, whether the key is revoked, expired, or unknown. Requires the worker to distinguish `TOKEN_REVOKED` / `TOKEN_EXPIRED` from `AUTH_FAILED` (the columns already exist).
- **`spikes pull --since <ts|last>` and a stamp.** Worker: `?since=` on the list endpoints. CLI: write `.spikes/state.json` with `last_pulled_at` and the credential fingerprint; `spikes list` warns when the cache is older than a day.
- **URL-prefix filter** on `GET /me/projects/:key/spikes` (`?url_prefix=`), so an agent can ask for "everything on `/prosser/versions/v0-3/`".
- **Replies.** New table `spike_replies (id, spike_id, author, body, version_label, created_at)`. `POST /spikes/:id/replies`. `spikes reply <id> "done in v0.5"`. MCP tool `reply_to_spike`. This is the first half of arc 3 and the data arc 4 needs.
- **Project creation by API key** stays user-only. Instead, `spikes projects create` under a logged-in user does it once; the key it mints is what goes in the repo.

Effort: 2 to 3 days. Depends on nothing.

### Slice 2: delivery

Goal: the builder's agent learns about new feedback without being told.

- **Per-project webhook** (`projects.webhook_url`, `webhook_secret`), same delivery code as share webhooks, fired on spike create and on reply. Pro and agent tiers.
- **Owner email** on first new spike per project per hour (digest, not one mail per click). Reuses the auth mailer.
- **`spikes watch`.** Long-polls `?since=` and prints each new spike as one JSON line. Composable: `spikes watch --project yvs | while read s; do herdr agent prompt builder "$s"; done`. Herdr on this machine already has `herdr agent prompt <name>`, so the bridge is a shell pipe, not a feature.
- **MCP at turn start.** Document the pattern "call `get_spikes` with `unresolved_only` at the start of every session" in `docs/mcp.md`. No code.

Effort: 2 days. Depends on slice 1 for `since`.

### Slice 3: closure

Goal: the reviewer sees what happened to their comment where they left it.

- **Status on spikes**: replace the boolean `resolved` with `status in (open, addressed, wont_do)` plus `addressed_in` (free-text version label). Keep `resolved` as a computed compatibility field in responses.
- **Widget reads back**: `GET /widget/spikes?project=&url=` returns status and latest reply per spike for that URL, public, origin-checked like POST. The widget shows a pin per existing spike with status colour and reply text. Reviewer identity: the widget already generates a reviewer id in localStorage; keep it and let the reviewer filter "mine".
- **Per-project review page** at `spikes.sh/p/<key>`: list of spikes grouped by URL, status, replies. Replaces the hand-built project room's comment section. Owner-only until a share token is added.

Effort: 3 to 4 days. Depends on slice 1.

### Slice 4: versions and questions

- **Versions**: `versions (project_key, label, url_prefix, created_at)`. A spike's version is derived by longest matching `url_prefix`; no widget change. `spikes versions add v0-5 --prefix /prosser/versions/v0-5/`. A version can carry `notes` ("addresses 3, 5, 6"), which is the reviewer-facing changelog.
- **Questions**: `questions (id, project_key, title, body, status)` and `answers (question_id, reviewer_id, body)`. Rendered by the widget on any page of the project as a small "3 questions for you" panel, and on the review page. This retires the mailto form.
- **Hosted snapshots** (upload a static build as a version, widget injected automatically) are the natural end state and what `export_review.py` prototypes. Not in this slice; it is a deploy product, and `spikes deploy` already exists for shares, so it may be a small step later.

Effort: 4 to 5 days. Depends on slices 1 and 3.

## 5. Design questions, with a recommendation each

**Spike or review round as the unit of work?** Clients and builders talk in rounds ("your v0.3 comments are in v0.5"). Model it as data, not as an object: a round is a version plus the spikes whose URL falls under it plus their replies. Slice 4's `versions` table gives that without a new entity. Introduce a first-class `rounds` object only if the review page needs state the version does not have (sign-off, deadline).

**Should Spikes host the versions?** Yes, eventually. It collapses widget injection, version list, and "what changed" into one upload. It is also the point where Spikes stops being a widget and becomes the review surface, which is the 100x claim. Sequence it after slice 3 so there is something worth hosting.

**Agent-native auth.** Project-scoped keys that never expire unless revoked, mintable by a logged-in owner from the CLI, safe to commit to a repo config. The account-level key stays for the owner's own tooling. Device-code login remains for humans. This is slice 1 and is the highest-leverage change in the doc.

**Pricing.** Project keys, webhooks, `watch`, and the review page belong in Pro and agent tiers. Capture stays free so reviewers never hit a wall. Questions and versions gate on Pro. Nothing here needs a new Stripe price.

## 6. Decision needed

Slice 1 is the recommendation: it is the only slice that would have turned this week's D1 queries into three CLI commands, and every later slice builds on its `since`, replies, and keys. The alternative worth arguing for is slice 3 first, because the reviewer-facing gap is what a paying client notices. Pick one; the other follows.
