# Feedback loop v2: API and CLI contract

Companion to `feedback-loop-v2.md`. This is the contract that the hosted worker (`spikes-hosted`), the CLI, the MCP server, and the widget implement together. Anything not listed here keeps its current behaviour. Field names in HTTP responses are camelCase, as in the existing `SpikeResponse`.

## 1. Credentials and auth errors

Three credential kinds, all sent as `Authorization: Bearer <value>`:

| Kind | Source | Scope |
|---|---|---|
| User token | `spikes login` (device code or magic link) | Everything the user owns |
| Account API key | `POST /auth/api-key` without `project_key` | Everything the user owns, limited by `scopes` |
| Project API key | `POST /auth/api-key` with `project_key` | One project only, limited by `scopes` |

`api_keys` gains `project_key TEXT NULL REFERENCES projects(key)`. `POST /auth/api-key` accepts `project_key` (must be a project the caller owns, else `404 PROJECT_NOT_FOUND`) and returns it. `GET /auth/api-keys` lists it. Project keys never expire unless `expires_at` is set explicitly or the key is revoked.

Endpoints under `/me/projects/:key/*` and `/spikes*` accept all three kinds. A project key whose `project_key` differs from `:key` (or from the spike's project) gets `404 PROJECT_NOT_FOUND`, never 403. `GET /spikes` with a project key returns only that project's spikes. `POST /projects`, `PATCH /me/projects/:key` (settings), `/billing/*`, `/usage`, and `/auth/*` still reject project keys with `403 AUTH_FORBIDDEN`. Scopes apply as today: `read` allows GET/HEAD only, `write` allows mutations only, `full` allows both.

Auth failures become specific. When a bearer value is recognised but unusable:

| Situation | Status | `code` | Extra fields |
|---|---|---|---|
| Token or key has `revoked_at` | 401 | `TOKEN_REVOKED` | `revoked_at` |
| Token or key has passed `expires_at` | 401 | `TOKEN_EXPIRED` | `expires_at` |
| Not recognised at all | 401 | `AUTH_FAILED` | none |

Revoked and expired lookups happen by hash, so no timing or enumeration change. `GET /me` with a project key returns `{ type: "api_key", scopes, project_key, key_prefix, expires_at, user: { id, email, tier } }`; with a user token it returns what it returns today.

## 2. Spikes: new columns, filters, status

`spikes` gains:

- `created_at TEXT NOT NULL` (server clock at insert; backfilled from `timestamp`)
- `updated_at TEXT NOT NULL` (bumped on any status change or reply; backfilled from `timestamp`)
- `status TEXT NOT NULL DEFAULT 'open'` with values `open`, `addressed`, `wont_do` (backfilled: `resolved = 1` becomes `addressed`)
- `addressed_in TEXT NULL` (free-text version label)

`resolved` and `resolved_at` stay and are kept consistent: `resolved = (status != 'open')`, `resolved_at` set when leaving `open`, cleared when returning to it.

New query parameters on `GET /spikes` and `GET /me/projects/:key/spikes`:

- `since=<ISO 8601>`: only spikes with `updated_at > since`
- `url_prefix=<string>`: only spikes whose `url` starts with the string (literal match; `%` and `_` escaped)

`PATCH /spikes/:id` and `PATCH /me/projects/:key/spikes/:id` accept:

```json
{ "resolved": true }
{ "status": "addressed", "addressed_in": "v0.5" }
{ "status": "wont_do" }
{ "status": "open" }
```

At least one of `resolved` or `status` is required. `resolved: true` means `status: addressed`; `resolved: false` means `status: open`. `addressed_in` may be set with any status and may be `null`.

Every spike in a response gains `status`, `addressedIn`, `createdAt`, `updatedAt`, `replyCount` (integer), `lastReply` (`{ id, authorType, authorName, body, versionLabel, createdAt }` or `null`), and `version` (label of the matching version, see section 5, or `null`).

## 3. Replies

Table `spike_replies`:

```
id TEXT PRIMARY KEY
spike_id TEXT NOT NULL REFERENCES spikes(id) ON DELETE CASCADE
author_type TEXT NOT NULL      -- 'owner' | 'agent'
author_name TEXT NOT NULL      -- free text, default 'Builder'
body TEXT NOT NULL             -- 1..4000 chars
version_label TEXT NULL
created_at TEXT NOT NULL
```

`author_type` is `owner` for user tokens and `agent` for API keys.

- `POST /spikes/:id/replies` body `{ body, author_name?, version_label?, status?, addressed_in? }`. Creates the reply and, when `status` or `addressed_in` is present, applies them to the spike in the same request with section 2 semantics. Bumps `updated_at`. Returns 201 with the reply object `{ id, spikeId, authorType, authorName, body, versionLabel, createdAt }`.
- `GET /spikes/:id/replies` returns `{ data: [reply, ...] }` oldest first.
- Both require write or read scope respectively and the same ownership rules as `GET /spikes/:id`.

## 4. Public read-back for the widget

All `/public/*` endpoints: no bearer token, origin allowlist enforced like `POST /spikes` (`403 ORIGIN_NOT_ALLOWED`) with one difference: a missing `Origin` header or the literal `null` is always refused, even when the project's allowlist contains `"null"` (that entry exists for file:// widget posts, not for reads), per-IP rate limit 120/min, responses never include reviewer email, user agent, or bounding boxes of other reviewers' spikes.

- `GET /public/spikes?project=<key>&url=<url>` returns `{ data: [ { id, type, selector, xpath, elementText, rating, comments, status, addressedIn, reviewer: { id, name }, timestamp, lastReply, version } ] }` for spikes whose `url` equals the given URL with any fragment removed. Max 200, newest first.
- `GET /public/versions?project=<key>` returns `{ data: [ { label, urlPrefix, notes, createdAt } ] }`.
- `GET /public/questions?project=<key>` returns open questions `{ data: [ { id, title, body, createdAt } ] }`.
- `POST /public/questions/:id/answers` body `{ body, reviewer: { id, name } }`, returns 201 `{ id, questionId, createdAt }`. `400 VALIDATION_ERROR` for empty body, `404 NOT_FOUND` for unknown or closed question.

Worker route: `spikes.sh/public*`. Marketing site `_routes.json` excludes `/public*`.

## 5. Versions

Table `versions`:

```
id TEXT PRIMARY KEY
project_key TEXT NOT NULL REFERENCES projects(key) ON DELETE CASCADE
label TEXT NOT NULL
url_prefix TEXT NOT NULL
notes TEXT NULL
created_at TEXT NOT NULL
UNIQUE (project_key, label)
```

A spike's `version` is the label of the version with the longest `url_prefix` that is a prefix of the spike's `url`, or `null`.

- `POST /me/projects/:key/versions` body `{ label, url_prefix, notes? }` → 201 version object `{ id, label, urlPrefix, notes, createdAt, spikeCount }`. `409 VERSION_EXISTS` on duplicate label.
- `GET /me/projects/:key/versions` → `{ data: [...] }` newest first, each with `spikeCount` and `openCount`.
- `PATCH /me/projects/:key/versions/:label` body `{ url_prefix?, notes? }`.
- `DELETE /me/projects/:key/versions/:label` → 204.

Auth: user token, account key, or project key for `:key`.

## 6. Questions and answers

Tables:

```
questions
  id TEXT PRIMARY KEY
  project_key TEXT NOT NULL REFERENCES projects(key) ON DELETE CASCADE
  title TEXT NOT NULL            -- 1..200
  body TEXT NULL                 -- up to 4000
  status TEXT NOT NULL DEFAULT 'open'   -- 'open' | 'closed'
  created_at TEXT NOT NULL

answers
  id TEXT PRIMARY KEY
  question_id TEXT NOT NULL REFERENCES questions(id) ON DELETE CASCADE
  reviewer_id TEXT NOT NULL
  reviewer_name TEXT NOT NULL
  body TEXT NOT NULL             -- 1..4000
  created_at TEXT NOT NULL
```

- `POST /me/projects/:key/questions` body `{ title, body? }` → 201 `{ id, title, body, status, createdAt, answerCount }`.
- `GET /me/projects/:key/questions[?status=open|closed]` → `{ data: [...] }` with `answerCount` and `lastAnswer` (`{ reviewerName, body, createdAt }` or `null`).
- `PATCH /me/projects/:key/questions/:id` body `{ title?, body?, status? }`.
- `GET /me/projects/:key/questions/:id/answers` → `{ data: [ { id, reviewer: { id, name }, body, createdAt } ] }` oldest first.
- Public endpoints in section 4.

Auth as for versions.

## 7. Project settings, webhooks, email

`projects` gains `webhook_url TEXT NULL`, `webhook_secret TEXT NULL`, `notify_email INTEGER NOT NULL DEFAULT 0`, `last_notified_at TEXT NULL`.

- `PATCH /me/projects/:key` body `{ allowed_origins?, webhook_url?: string|null, notify_email?: boolean }`. User token or account key only. Setting `webhook_url` or `notify_email: true` requires tier `pro` or `agent`, else `402 UPGRADE_REQUIRED` with `upgrade_url`. `webhook_url` is validated with the existing share webhook validator; a fresh `webhook_secret` is generated whenever the URL changes and is returned once in that response as `webhookSecret`.
- `GET /me/projects` and `GET /me/projects/:key` items gain `webhookUrl`, `notifyEmail`.

Webhook delivery reuses the share webhook delivery code (HMAC header, retries, `ctx.waitUntil`). Payload:

```json
{ "event": "spike.created" | "spike.replied" | "spike.status_changed" | "question.answered",
  "project": "<key>", "timestamp": "<ISO>",
  "spike": { ...SpikeResponse } | null,
  "reply": { ...reply } | null,
  "answer": { "questionId", "questionTitle", "reviewer", "body", "createdAt" } | null }
```

Email: on `spike.created` and `question.answered`, when `notify_email` is on, the owner is pro or agent, and `last_notified_at` is null or older than one hour, send one email via Resend to the owner: subject `New feedback on <key>`, body lists the count of spikes and answers since `last_notified_at`, the pages involved, and a link to `https://spikes.sh/dashboard/p/<key>`. Then set `last_notified_at`. Never more than one email per project per hour.

## 8. Dashboard review page

`https://spikes.sh/dashboard/p/<key>` (same SPA, new route) shows: versions with notes and counts; spikes grouped by URL with status, rating, comment, replies, and a reply box that posts to `/spikes/:id/replies` with an optional status change; questions with answers and a close button; project settings (origins, webhook, email). Uses the user token from the dashboard login as today.

## 9. CLI

New or changed commands, all with `--json`:

- `spikes status`: which credential is in use (`SPIKES_TOKEN`, `.spikes/config.toml [remote].token`, or the global auth file), the endpoint, the result of `GET /me` (identity or the specific 401 code with `revoked_at` / `expires_at`), and the cache age from `.spikes/state.json`. Exit 1 when the credential is unusable.
- `.spikes/state.json`: `{ "last_pulled_at": ISO, "endpoint": string, "credential_prefix": first 12 chars }`, written by `pull`, `sync`, and `watch`. `list`, `show`, `export`, and `hotspots` print a one-line stderr warning when a remote is configured and the stamp is missing or older than 24 hours.
- `spikes pull --since <ISO|last> [--url-prefix <p>]`. `last` reads the stamp. Existing behaviour without flags is unchanged apart from writing the stamp.
- `spikes projects create <key> [--origin <o>]...` and `spikes projects list` (user token or account key).
- `spikes auth create-key --project <key>` mints a project key (existing `create-key` gains the flag).
- `spikes reply <id> <text> [--version <label>] [--status addressed|wont_do|open] [--name <author>]`. `--version <label>` also sets the spike to `addressed` with `addressed_in = <label>` unless `--status` is given explicitly.
- `spikes resolve <id> [--in <label>] [--wont-do]` sends `status` per section 2; `spikes resolve --undo <id>` sends `status: open`. Existing `resolve` without flags keeps sending `resolved: true`.
- `spikes versions add <label> --prefix <url-prefix> [--notes <text>]`, `spikes versions list`, `spikes versions notes <label> <text>`.
- `spikes questions add <title> [--body <text>]`, `spikes questions list [--closed]`, `spikes questions answers <id>`, `spikes questions close <id>`.
- `spikes watch [--since <ISO|last>] [--interval <seconds, default 30>] [--url-prefix <p>] [--exec <command>]`: polls `GET /spikes?since=` (and `/me/projects/:key/questions` for new answers when a project key is configured), prints one JSON object per line per new or updated spike or answer, updates the stamp after each poll. With `--exec`, runs the command with the JSON on stdin per event; a non-zero exit is logged and polling continues. `--once` polls one time and exits.

Remote mode for all of the above uses the `[remote]` section of `.spikes/config.toml`, then `SPIKES_TOKEN`, then the global auth file, in that order of precedence, and the project key from `[project].key` when an endpoint needs one.

## 10. MCP tools

`get_spikes` gains `url_prefix` and `since`. New tools (remote mode; local mode returns a clear "hosted only" error):

- `reply_to_spike { spike_id, body, version_label?, status?, addressed_in? }`
- `set_spike_status { spike_id, status, addressed_in? }`
- `list_versions {}`, `add_version { label, url_prefix, notes? }`
- `list_questions { status? }`, `ask_question { title, body? }`, `get_question_answers { question_id }`

Every spike returned by MCP tools includes `status`, `addressedIn`, `version`, `replyCount`, `lastReply`. `docs/mcp.md` documents the turn-start pattern: call `get_spikes` with `unresolved_only: true` and `since` set to the last session's time before doing anything else.

## 11. Widget

When `data-project` is set and the endpoint is hosted, on load the widget calls `GET /public/spikes`, `GET /public/versions`, and `GET /public/questions` for the current page (three requests, failures are silent). It then:

- renders existing spikes as pins at their selectors with a colour per status (`open`: the configured colour, `addressed`: green, `wont_do`: grey); clicking a pin shows the comment, status, `addressedIn`, and the last reply; spikes whose `reviewer.id` matches `spikes:reviewer` in localStorage are labelled "yours"
- shows a "Questions for you (n)" entry in the panel that lists open questions with a textarea each; sending posts to `/public/questions/:id/answers` with the stored reviewer identity and marks the question answered in localStorage
- shows the matching version label and notes in the panel header when the versions list is non-empty

New attributes: `data-readback="off"` disables the existing-spike pins, `data-questions="off"` disables the questions entry. Both default to on. The version banner is fetched while either is on; with both off the widget makes no `/public/*` requests. `site/spikes.js` is the deployed copy of `widget/spikes.js` and must stay identical.

## 12. Embedder contract (documented in `docs/API.md`)

`POST /spikes` is public. The document must state: the exact JSON the widget sends; which fields are required (`page`, `type`, and one of `project`/`projectKey`); that `Origin` is checked server-side against the project's `allowed_origins` (patterns, `null` for file URLs, wildcard rules); that a preflight is answered with `204` and `Access-Control-Allow-Origin: *`; the rate limits (60/min per IP, 60/min per project); every error code with status; and that the response is `201 { ok: true, id }`. The same section lists the `/public/*` endpoints.
