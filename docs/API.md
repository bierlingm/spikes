# Spikes API Reference

API contract for self-hosted Spikes backends.

## Authentication

| Method | Use Case | Header/Param |
|--------|----------|--------------|
| User token | Everything the user owns (from `spikes login`) | `Authorization: Bearer <token>` |
| Account API key | Same reach as the user, limited by `scopes` | `Authorization: Bearer sk_spikes_…` |
| Project API key | One project only, limited by `scopes` (`POST /auth/api-key` with `project_key`) | `Authorization: Bearer sk_spikes_…` |
| Query param | Admin operations (self-hosted) | `?token=<SPIKES_TOKEN>` |

Scopes: `read` allows GET/HEAD only, `write` allows mutations only, `full` allows both.

`/me/projects/:key/*`, `/spikes*`, `/me/projects/:key/versions*` and `/me/projects/:key/questions*` accept all three bearer kinds. A project key used against a different project answers `404 PROJECT_NOT_FOUND`, never 403. `GET /spikes` with a project key returns only that project's spikes. `POST /projects`, `PATCH /me/projects/:key`, `/billing/*`, `/usage` and `/auth/*` reject API keys with `403 AUTH_FORBIDDEN`.

**Auth failures are specific.** When the bearer value is recognised but unusable:

| Situation | Status | `code` | Extra fields |
|-----------|--------|--------|--------------|
| Token or key was revoked | 401 | `TOKEN_REVOKED` | `revoked_at` |
| Token or key passed its `expires_at` | 401 | `TOKEN_EXPIRED` | `expires_at` |
| Not recognised at all | 401 | `AUTH_FAILED` | — |

`GET /me` with a project key returns `{ "type": "api_key", "scopes", "project_key", "key_prefix", "expires_at", "user": { "id", "email", "tier" } }`.

## Endpoints

### Shares

#### POST /shares

Create a new share.

- **Auth:** Bearer token
- **Content-Type:** `multipart/form-data`
- **Body:** `metadata` JSON field + file uploads

```bash
curl -X POST https://your-api.example/shares \
  -H "Authorization: Bearer $TOKEN" \
  -F 'metadata={"slug":"my-project","title":"My Project"}' \
  -F "file=@index.html"
```

**Response:**
```json
{
  "ok": true,
  "url": "https://your-api.example/s/my-project",
  "share_id": "abc123",
  "slug": "my-project"
}
```

---

#### GET /shares

List user's shares.

- **Auth:** Bearer token

```bash
curl https://your-api.example/shares \
  -H "Authorization: Bearer $TOKEN"
```

**Response:**
```json
[
  {
    "id": "abc123",
    "slug": "my-project",
    "url": "https://your-api.example/s/my-project",
    "spike_count": 5,
    "created_at": "2024-01-15T10:30:00Z"
  }
]
```

---

#### DELETE /shares/:id

Delete a share and export its spikes.

- **Auth:** Bearer token

```bash
curl -X DELETE https://your-api.example/shares/abc123 \
  -H "Authorization: Bearer $TOKEN"
```

**Response:**
```json
{
  "ok": true,
  "exported_spikes": [
    { "id": "spike1", "rating": "love", "comments": "Great!" }
  ]
}
```

---

### Auth

Magic link authentication endpoints.

#### POST /auth/login

Request a magic link login email.

- **Auth:** None
- **Content-Type:** `application/json`

```bash
curl -X POST https://your-api.example/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com"}'
```

**Response:**
```json
{
  "ok": true,
  "message": "Check your email"
}
```

---

#### POST /auth/verify

Verify a magic link token and receive a bearer token.

- **Auth:** None
- **Content-Type:** `application/json`

```bash
curl -X POST https://your-api.example/auth/verify \
  -H "Content-Type: application/json" \
  -d '{"login_token": "uuid-token-from-email"}'
```

**Response:**
```json
{
  "ok": true,
  "token": "bearer-token-uuid",
  "user": {
    "id": "user-uuid",
    "email": "user@example.com",
    "tier": "free"
  }
}
```

---

#### POST /auth/rotate-token

Rotate the current bearer token (invalidates old, returns new).

- **Auth:** Bearer token

```bash
curl -X POST https://your-api.example/auth/rotate-token \
  -H "Authorization: Bearer $TOKEN"
```

**Response:**
```json
{
  "ok": true,
  "token": "new-bearer-token-uuid",
  "user": {
    "id": "user-uuid",
    "email": "user@example.com",
    "tier": "pro"
  }
}
```

---

#### POST /auth/recover

Request account recovery via magic link.

- **Auth:** None
- **Content-Type:** `application/json`

```bash
curl -X POST https://your-api.example/auth/recover \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com"}'
```

**Response:** (Same regardless of whether email exists)
```json
{
  "ok": true,
  "message": "Recovery email sent"
}
```

---

#### GET /me

Get current user information.

- **Auth:** Bearer token

```bash
curl https://your-api.example/me \
  -H "Authorization: Bearer $TOKEN"
```

**Response:**
```json
{
  "email": "user@example.com",
  "tier": "pro"
}
```

---

### Billing

Stripe subscription management endpoints.

#### GET /billing/portal

Get Stripe Customer Portal URL for managing subscription.

- **Auth:** Bearer token

```bash
curl https://your-api.example/billing/portal \
  -H "Authorization: Bearer $TOKEN"
```

**Response:**
```json
{
  "url": "https://billing.stripe.com/session/..."
}
```

---

#### GET /billing/checkout

Create a Stripe Checkout session for Pro subscription.

- **Auth:** Bearer token

```bash
curl https://your-api.example/billing/checkout \
  -H "Authorization: Bearer $TOKEN"
```

**Response:**
```json
{
  "url": "https://checkout.stripe.com/..."
}
```

Returns `null` url if user already has Pro subscription.

---

#### GET /usage

Get current usage statistics and limits.

- **Auth:** Bearer token

```bash
curl https://your-api.example/usage \
  -H "Authorization: Bearer $TOKEN"
```

**Response:**
```json
{
  "spikes": 45,
  "spike_limit": 100,
  "shares": 3,
  "share_limit": 5,
  "tier": "free",
  "reset_at": "2024-02-01T00:00:00Z"
}
```

---

### Webhooks

#### POST /webhooks/stripe

Stripe webhook receiver for subscription events. Signature verified.

- **Auth:** Stripe signature verification (via `Stripe-Signature` header)

---

### Spikes

#### POST /spikes

Create a spike. This is the public endpoint the widget (and any custom embedder) posts to.

- **Auth:** None. The request is authorised by `project` + the `Origin` header (see below).
- **Content-Type:** `application/json`
- **Rate limits:** 60/min per client IP and 60/min per project (`429 RATE_LIMIT` / `429 RATE_LIMITED` with `Retry-After`).

**Exactly what the widget sends** (`widget/spikes.js`, `createSpike`):

```json
{
  "id": "V1StGXR8_Z5jdHi6B-myT",
  "type": "element",
  "projectKey": "my-project",
  "page": "Home – My Site",
  "url": "https://example.com/pricing/#plans",
  "reviewer": { "id": "r_8bNb…", "name": "Jane", "email": "jane@example.com" },
  "rating": "love",
  "comments": "Perfect placement",
  "timestamp": "2026-09-14T10:30:00.000Z",
  "viewport": { "width": 1440, "height": 900 },
  "selector": "#plans > button.cta",
  "xpath": "/html/body/main/section[2]/button",
  "elementText": "Start free",
  "boundingBox": { "x": 412, "y": 830, "width": 160, "height": 44 }
}
```

`selector`, `xpath`, `elementText`, `boundingBox` are present only for `type: "element"`. `reviewer.email` is present only when `data-collect-email="true"`. `page` is `document.title` (falls back to the path). The widget's nanoid `id` is ignored and the server generates its own (see *Idempotent retries* below for client-chosen ids).

**Field rules** (`createSpikeSchema`):

| Field | Required | Rule |
|-------|----------|------|
| `project` or `projectKey` | one of them | non-empty string; `projectKey` wins when both are sent |
| `page` | yes | non-empty string |
| `type` | yes | `"page"` or `"element"` |
| `url` | no | must be a valid URL when present |
| `rating` | no | `"love"`, `"like"`, `"meh"`, `"no"`, or `null` |
| `comments` | no | string |
| `reviewer` | no | `{ id?, name?, email? }`; `email` must be a valid address when present |
| `timestamp` | no | ISO 8601 |
| `viewport` | no | `{ width, height }` positive integers |
| `selector`, `xpath`, `elementText` | no | strings |
| `boundingBox` | no | `{ x, y, width, height }` numbers |
| `share_id` | no | UUID of a hosted share |
| `id` | no | UUID: used as the spike id and idempotency key. Any other value is ignored |

Unknown fields are ignored.

**Idempotent retries.** To retry a POST safely after a timeout or a lost response, choose the spike id yourself: send a UUID as `id` in the body or as an `Idempotency-Key: <uuid>` header (case-insensitive; stored lower-case), and send the same value on every retry.

- First write: `201 { "ok": true, "id": "<your uuid>" }`.
- Replay of an id that already exists in the same project: `200 { "ok": true, "id": "…", "duplicate": true }`. Nothing else happens: the stored spike keeps its original content, and no webhook, email, meter event or share-limit count fires. The replay still counts against the rate limits.
- The id already exists in another project: `409 ID_CONFLICT`.
- `Idempotency-Key` that is not a UUID, or that differs from a UUID body `id`: `400 VALIDATION_ERROR`.
- Concurrent requests with the same id create one spike. The others answer like a replay.

```js
const id = crypto.randomUUID();           // once per spike, reused for retries
for (let attempt = 0; attempt < 4; attempt++) {
  try {
    const res = await fetch('https://spikes.sh/spikes', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Idempotency-Key': id },
      body: JSON.stringify({ projectKey: 'my-project', type: 'page', page: document.title, url: location.href, comments }),
    });
    if (res.status === 201 || res.status === 200) break;             // stored (200 = already stored)
    if (res.status !== 429 && res.status < 500) throw new Error(await res.text()); // do not retry 4xx
    await new Promise(r => setTimeout(r, (Number(res.headers.get('Retry-After')) || 2 ** attempt) * 1000));
  } catch (e) { if (attempt === 3) throw e; await new Promise(r => setTimeout(r, 2 ** attempt * 1000)); }
}
```

**Origin check.** The server compares the request `Origin` header with the project's `allowed_origins` list (default `["http://localhost:*", "null"]`, set at `POST /projects` or `PATCH /me/projects/:key`):

- Each entry is matched against the whole `Origin` string; `*` matches any run of characters except `/`. So `https://*.example.com` matches `https://app.example.com` but not `https://example.com`; `http://localhost:*` matches any port; `https://*` matches any HTTPS origin.
- Scheme is strict: `http://*` never matches an `https://` origin.
- The literal entry `null` allows a missing `Origin` header and the literal header value `Origin: null` (file URLs, sandboxed iframes, `curl` without `-H Origin`).
- Headers containing CR, LF, NUL, or a comma are rejected.
- Mismatch → `403 ORIGIN_NOT_ALLOWED`. Unknown project → `404 PROJECT_NOT_FOUND`.

**Preflight.** `OPTIONS /spikes` answers `204` with `Access-Control-Allow-Origin: *`, `Access-Control-Allow-Headers: Content-Type, Authorization, Idempotency-Key`, `Access-Control-Allow-Methods: GET, POST, DELETE, PATCH, OPTIONS`. Every response, including errors, carries `Access-Control-Allow-Origin: *`, so a browser can read the error body.

```bash
curl -X POST https://spikes.sh/spikes \
  -H "Content-Type: application/json" \
  -H "Origin: https://example.com" \
  -d '{ "type": "page", "projectKey": "my-project", "page": "Home", "url": "https://example.com/", "comments": "Looks good" }'
```

**Response** `201`:
```json
{ "ok": true, "id": "df5df3e3-9716-4121-8f60-d4089671bcd4" }
```

`200 { "ok": true, "id", "duplicate": true }` for a replayed client id (see above).

**Errors:**

| Status | `code` | When |
|--------|--------|------|
| 400 | `INVALID_JSON` | body is not JSON |
| 400 | `VALIDATION_ERROR` | field rules above (`details[]` names the field), or a bad `Idempotency-Key` |
| 403 | `ORIGIN_NOT_ALLOWED` | `Origin` not in the project's allowlist |
| 404 | `PROJECT_NOT_FOUND` | unknown `project`/`projectKey` |
| 404 | `NOT_FOUND` | `share_id` given but unknown |
| 409 | `ID_CONFLICT` | client `id` already used by another project |
| 429 | `RATE_LIMIT` | per-IP limit, `retry_after` seconds |
| 429 | `RATE_LIMITED` | per-project limit, `retry_after` seconds |
| 429 | `SPIKE_LIMIT` | free-tier share limit, `upgrade_url` |
| 429 | `BUDGET_EXCEEDED` | agent-tier monthly cap |
| 500 | `INTERNAL_ERROR` | server error |

---

#### GET /spikes

List the caller's spikes (owner-scoped) with cursor-based pagination.

- **Auth:** Bearer (user token, account key, or project key — a project key sees only its project). Admin `?token=` on self-hosted.
- **Query params:**
  - `page` — Filter by page path
  - `reviewer` — Filter by reviewer ID
  - `rating` — Filter by rating value
  - `since` — ISO 8601; only spikes whose `updatedAt` is later
  - `url_prefix` — only spikes whose `url` starts with this literal string
  - `cursor` — Pagination cursor (optional)
  - `limit` — Items per page (default 100, max 1000)

```bash
curl "https://spikes.sh/spikes?since=2026-09-14T00:00:00Z&url_prefix=https://example.com/v0-5/" \
  -H "Authorization: Bearer $SPIKES_TOKEN"
```

**Response:**
```json
{
  "data": [
    {
      "id": "df5df3e3-…",
      "type": "element",
      "projectKey": "my-project",
      "page": "Home",
      "url": "https://example.com/v0-5/",
      "selector": "button.submit",
      "xpath": null,
      "elementText": "Submit",
      "boundingBox": { "x": 10, "y": 20, "width": 100, "height": 40 },
      "rating": "love",
      "comments": "Perfect placement",
      "reviewer": { "id": "r123", "name": "Jane", "email": null },
      "viewport": { "width": 1920, "height": 1080 },
      "userAgent": "Mozilla/5.0 …",
      "timestamp": "2026-09-14T10:30:00.000Z",
      "createdAt": "2026-09-14T10:30:01.000Z",
      "updatedAt": "2026-09-15T08:00:00.000Z",
      "status": "addressed",
      "addressedIn": "v0.5",
      "resolved": true,
      "resolvedAt": "2026-09-15T08:00:00.000Z",
      "replyCount": 1,
      "lastReply": { "id": "rp1", "authorType": "agent", "authorName": "Builder", "body": "Done in v0.5", "versionLabel": "v0.5", "createdAt": "2026-09-15T08:00:00.000Z" },
      "version": "v0.5"
    }
  ],
  "next_cursor": null
}
```

`status` is `open`, `addressed`, or `wont_do`; `resolved` is `status != "open"`. `version` is the label of the project version with the longest `urlPrefix` matching the spike's `url`, or `null`.

---

#### GET /spikes/:id

One spike, same shape as a list item. `404 SPIKE_NOT_FOUND` for unknown or not-owned ids.

---

#### PATCH /spikes/:id

Change a spike's status. Also available as `PATCH /me/projects/:key/spikes/:id`.

- **Auth:** Bearer (write or full scope)
- **Body:** at least one of `resolved` or `status`

```json
{ "resolved": true }
{ "status": "addressed", "addressed_in": "v0.5" }
{ "status": "wont_do" }
{ "status": "open" }
```

`resolved: true` is `status: "addressed"`; `resolved: false` is `status: "open"`. `addressed_in` may accompany any status and may be `null`. `PATCH /spikes/:id` returns `{ ok, id, resolved, status, addressed_in, updated_at }` (legacy shape); `PATCH /me/projects/:key/spikes/:id` returns the full spike object.

---

#### DELETE /spikes/:id

Deletes the spike and its replies. `200 { ok: true, id }`.

---

#### POST /spikes/:id/replies

Reply to a spike as the builder or an agent (user token → `authorType: "owner"`, API key → `"agent"`).

- **Auth:** Bearer (write or full scope)
- **Body:**

```json
{
  "body": "Done in v0.5, the button is now brand blue.",
  "author_name": "Builder",
  "version_label": "v0.5",
  "status": "addressed",
  "addressed_in": "v0.5"
}
```

`body` is required (1–4000 chars). `status` / `addressed_in` are optional and applied to the spike in the same request. Bumps the spike's `updatedAt`.

**Response** `201`:
```json
{ "id": "rp1", "spikeId": "df5df3e3-…", "authorType": "agent", "authorName": "Builder", "body": "Done in v0.5…", "versionLabel": "v0.5", "createdAt": "2026-09-15T08:00:00.000Z" }
```

---

#### GET /spikes/:id/replies

`{ "data": [reply, …] }`, oldest first.

---

### Public read-back (`/public/*`)

No bearer token. The `Origin` header must be present and match the project's `allowed_origins` (`403 ORIGIN_NOT_ALLOWED`); unlike `POST /spikes`, a missing `Origin` or the literal `null` is always refused, even when the allowlist contains `"null"`, unknown project → `404 PROJECT_NOT_FOUND`, rate limit 120/min per IP. Responses never include reviewer emails, user agents, or bounding boxes.

#### GET /public/spikes?project=&url=

Spikes for one page. `url` is compared without its fragment. Max 200, newest first.

```json
{
  "data": [
    {
      "id": "df5df3e3-…", "type": "element", "selector": "button.submit", "xpath": null, "elementText": "Submit",
      "rating": "no", "comments": "Colour clashes", "status": "addressed", "addressedIn": "v0.5",
      "reviewer": { "id": "r123", "name": "Jane" }, "timestamp": "2026-09-14T10:30:00.000Z",
      "lastReply": { "id": "rp1", "authorType": "agent", "authorName": "Builder", "body": "Changed to brand blue", "versionLabel": "v0.5", "createdAt": "2026-09-15T08:00:00.000Z" },
      "version": "v0.5"
    }
  ]
}
```

#### GET /public/versions?project=

```json
{ "data": [ { "label": "v0.5", "urlPrefix": "https://example.com/v0-5/", "notes": "Addresses comments 1, 2, 3", "createdAt": "…" } ] }
```

#### GET /public/questions?project=

Open questions only.

```json
{ "data": [ { "id": "q1", "title": "Which hero photo?", "body": "A: pool, B: kitchen", "createdAt": "…" } ] }
```

#### POST /public/questions/:id/answers

```json
{ "body": "B, the kitchen", "reviewer": { "id": "r123", "name": "Jane" } }
```

`201 { "id": "a1", "questionId": "q1", "createdAt": "…" }`. `400 VALIDATION_ERROR` for an empty body, `404 NOT_FOUND` for an unknown or closed question. This endpoint is not idempotent. For a form with several answers, use `POST /public/submissions`.

#### POST /public/submissions

Submit a set of text answers at once (a decision form, a survey step), atomically and idempotently. Access rules are those of `POST /spikes`, not the read-back endpoints: the project must exist, and the `Origin` must match its allowlist. A missing or `null` Origin passes only if the list contains `"null"`. Limits are 60/min per IP (`429 RATE_LIMIT`) and 60/min per project (`429 RATE_LIMITED`), both with `Retry-After`. CORS is open.

```json
{
  "project": "my-project",
  "submission_id": "5b0e8c1e-2a0c-4f7e-9d8a-3f1c2b4a5d6e",
  "reviewer": { "id": "r123", "name": "Jane" },
  "page": "/decisions",
  "url": "https://example.com/decisions",
  "answers": [
    { "question_id": "q1", "body": "B, the kitchen" },
    { "key": "budget", "title": "Budget ceiling", "body": "40k" }
  ]
}
```

| Field | Required | Rule |
|-------|----------|------|
| `project` | yes | project key |
| `submission_id` | yes | UUID minted by the client once per submit and reused for every retry of it. Mint a new one when the answers change |
| `reviewer` | no | `{ id?, name? }`, default `anon` / `Anonymous` |
| `page`, `url` | no | strings; `url` must be a valid URL |
| `answers` | yes | 1 to 50 items |
| `answers[].body` | yes | 1 to 4000 characters after trimming |
| `answers[].question_id` | no | an **open** question of this project (`GET /public/questions`); else `400 VALIDATION_ERROR` with `details[].field = "answers.<i>.question_id"` |
| `answers[].title` | when no `question_id` | up to 200 characters; defaults to the question's title when `question_id` is set |
| `answers[].key` | no | your own field name, up to 100 characters |

Answers with `question_id` also show up as answers to that question (`GET /me/projects/:key/questions/:id/answers`, the widget panel, the email digest). Answers without one are free-form and are kept only with the submission.

| Status | Body | When |
|--------|------|------|
| 201 | `{ "ok": true, "submissionId", "answerCount" }` | stored; all answers or none |
| 200 | `{ "ok": true, "submissionId", "answerCount", "duplicate": true }` | same `submission_id`, same answers: nothing is written, nothing fires |
| 409 | `IDEMPOTENCY_CONFLICT` | same `submission_id` with different answers; nothing is written |
| 409 | `ID_CONFLICT` | `submission_id` belongs to another project |
| 400 | `VALIDATION_ERROR` / `INVALID_JSON` | see the rules above |
| 403 / 404 / 429 | `ORIGIN_NOT_ALLOWED` / `PROJECT_NOT_FOUND` / `RATE_LIMIT`, `RATE_LIMITED` | as for `POST /spikes` |

A stored submission fires one `submission.received` project webhook with the whole submission, then one `question.answered` per answer that has a `question_id` (see *Project webhooks*).

```js
const submission_id = crypto.randomUUID(); // keep it until the server has answered 200 or 201
async function send() {
  const res = await fetch('https://spikes.sh/public/submissions', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ project: 'my-project', submission_id, reviewer, page: location.pathname, url: location.href, answers }),
  });
  if (res.status === 200 || res.status === 201) return res.json();
  if (res.status === 429 || res.status >= 500) throw new Error('retry'); // retry later with the same submission_id
  throw new Error(`rejected: ${res.status} ${await res.text()}`);       // fix the request, do not retry as is
}
```

Owner side (user token, account key, or the project's key):

| Method | Path | Result |
|--------|------|--------|
| `GET` | `/me/projects/:key/submissions[?since=<ISO>&limit=1..200]` | `{ "data": [Submission] }`. Without `since`, newest first. With `since`, only submissions created after it, oldest first, so a poller can pass the last `createdAt` as the next `since`. Default `limit` 50 |
| `GET` | `/me/projects/:key/submissions/:id` | `Submission`, or `404 NOT_FOUND` |

`Submission` = `{ id, projectKey, reviewer: { id, name }, page, url, answerCount, createdAt, answers: [ { id, position, questionId, key, title, body } ] }`.

---

### Projects

#### POST /projects

Create a project. User token only.

```json
{ "key": "my-project", "allowed_origins": ["https://example.com", "http://localhost:*"] }
```

#### GET /me/projects

`{ "data": [ { "id", "key", "allowed_origins", "created_at", "spike_count", "last_activity", "webhookUrl", "notifyEmail" } ], "pagination" }`.

#### PATCH /me/projects/:key

Project settings. User token or account key (project keys are rejected with `403 AUTH_FORBIDDEN`).

```json
{ "allowed_origins": ["https://example.com"], "webhook_url": "https://hooks.example.com/spikes", "notify_email": true }
```

`webhook_url` (HTTPS, no private addresses) and `notify_email: true` need the `pro` or `agent` tier, else `402 UPGRADE_REQUIRED` with `upgrade_url`. When the webhook URL changes a new secret is generated and returned once as `webhookSecret`. Set `webhook_url` to `null` to remove it.

#### GET /me/projects/:key/spikes

Paginated list for one project: `?page`, `?per_page` (max 200), `?filter_page`, `?filter_rating`, `?filter_resolved`, `?since`, `?url_prefix`. Items have the `GET /spikes` shape.

---

### Versions

A version is a label plus a URL prefix. Spikes whose `url` starts with the prefix belong to it (longest prefix wins).

| Method | Path | Body / Result |
|--------|------|---------------|
| `POST` | `/me/projects/:key/versions` | `{ "label": "v0.5", "url_prefix": "https://example.com/v0-5/", "notes": "…" }` → `201 { id, label, urlPrefix, notes, createdAt, spikeCount }`; `409 VERSION_EXISTS` on duplicate label |
| `GET` | `/me/projects/:key/versions` | `{ "data": [ …, each with spikeCount, openCount ] }`, newest first |
| `PATCH` | `/me/projects/:key/versions/:label` | `{ "url_prefix"?, "notes"? }` |
| `DELETE` | `/me/projects/:key/versions/:label` | `204` |

Auth: user token, account key, or the project's key.

---

### Questions

| Method | Path | Body / Result |
|--------|------|---------------|
| `POST` | `/me/projects/:key/questions` | `{ "title": "Which hero photo?", "body": "A or B" }` → `201 { id, title, body, status, createdAt, answerCount }` |
| `GET` | `/me/projects/:key/questions[?status=open\|closed]` | `{ "data": [ …, each with answerCount, lastAnswer: { reviewerName, body, createdAt } \| null ] }` |
| `PATCH` | `/me/projects/:key/questions/:id` | `{ "title"?, "body"?, "status"?: "open" \| "closed" }` |
| `GET` | `/me/projects/:key/questions/:id/answers` | `{ "data": [ { id, reviewer: { id, name }, body, createdAt } ] }`, oldest first |

Auth as for versions. Reviewers answer through `POST /public/questions/:id/answers`.

---

### Project webhooks

Configured per project with `PATCH /me/projects/:key`. Every event is a `POST` of JSON to the project's `webhook_url`:

```json
{
  "event": "spike.created",
  "project": "my-project",
  "timestamp": "2026-09-15T08:00:00.000Z",
  "spike": { "…SpikeResponse" },
  "reply": null,
  "answer": null,
  "submission": null,
  "deliveryId": "0f6c5a52-1f0e-4d53-a8c4-6a1b0b8f2e11"
}
```

`event` is one of:

- `spike.created`
- `spike.replied` (`reply` set)
- `spike.status_changed`
- `question.answered` (`answer`: `{ questionId, questionTitle, reviewer, body, createdAt }`, `spike` null)
- `submission.received` (`submission`: the `Submission` object of `POST /public/submissions`)

Share webhooks (`POST /shares` with `webhook_url`) send `{ "event": "spike.created", "spike": { id, type, page, rating, comments, selector, reviewer, timestamp }, "deliveryId" }` and follow the same delivery rules.

**Headers.**

| Header | Value |
|--------|-------|
| `X-Spikes-Signature` | `t=<unix seconds>,v1=<hex HMAC-SHA256 of "<t>.<raw body>" keyed with the webhook secret>` |
| `X-Spikes-Delivery-Id` | UUID of this delivery. The same on every retry, and equal to `deliveryId` in the body (which the signature covers) |
| `X-Spikes-Event` | the `event` value |
| `X-Spikes-Delivery-Attempt` | `1` for the first attempt, then `2`, `3`, … |

**Delivery semantics: at least once.** Each event is written to a durable outbox before the triggering request returns, then sent immediately.

- A `2xx` answer (within 10 s) completes the delivery.
- Timeouts, network errors, `408`, `429` and `5xx` are retried after 1 min, 5 min, 30 min, 2 h, 6 h and 16 h, so there are 7 attempts over about 24.5 h before the delivery is marked failed.
- Any other `4xx` stops the delivery at once.
- The body is byte-identical on every attempt. The signature timestamp is fresh each time.
- Retries go to the project's current `webhook_url`, signed with its current secret. If the webhook is removed or the owner leaves the pro/agent tier, pending retries stop.

A receiver can get the same delivery twice, for example when it processed the request but its response was lost. Dedupe on `X-Spikes-Delivery-Id` and answer `2xx` quickly (queue the work). Events can arrive out of order. Use `timestamp` and the ids in the payload, not arrival order.

**Email digest.** With `notify_email` on (pro/agent), the owner receives at most one email per project per hour summarising new spikes and answers since the previous email, with a link to `https://spikes.sh/dashboard/p/<key>`.

---

### Share Serving

#### GET /s/:slug

Serve shared HTML with widget injected.

- **Auth:** None (public)

```bash
curl https://your-api.example/s/my-project
```

**Response:** HTML content with Spikes widget script injected.

---

## Rate Limiting

Endpoints are rate limited with fixed one-minute windows, counted atomically (concurrent requests never fail because of the counter itself). When rate limited, responses include a `Retry-After` header with seconds until retry.

| Endpoint | Limit | Window | Identifier |
|----------|-------|--------|------------|
| `POST /spikes` | 60 | 1 minute | Client IP |
| `POST /spikes` | 60 | 1 minute | Project key |
| `POST /public/submissions` | 60 | 1 minute | Client IP |
| `POST /public/submissions` | 60 | 1 minute | Project key |
| `GET /public/*`, `POST /public/questions/:id/answers` | 120 | 1 minute | Client IP |
| `POST /shares` | 10 | 1 minute | Bearer token |
| Password attempts per share | 5 | 1 minute | Slug + IP |

**Rate Limit Response (429):**
```json
{
  "error": "Rate limit exceeded",
  "code": "RATE_LIMIT",
  "retry_after": 45
}
```

---

## Error Responses

All endpoints return errors in a standardized format:

```json
{
  "error": "Human readable message",
  "code": "MACHINE_CODE"
}
```

Validation errors include field-level details:

```json
{
  "error": "Validation failed",
  "code": "VALIDATION_ERROR",
  "details": [
    {
      "field": "email",
      "message": "Invalid email format",
      "code": "INVALID_FIELD"
    }
  ]
}
```

### Error Codes

| Code | Meaning |
|------|---------|
| `AUTH_FAILED` | Credentials not recognised |
| `TOKEN_REVOKED` | Token or API key was revoked (`revoked_at` included) |
| `TOKEN_EXPIRED` | Token or API key passed its expiry (`expires_at` included) |
| `AUTH_REQUIRED` | Authentication required |
| `AUTH_FORBIDDEN` | Credential kind not allowed on this endpoint |
| `SCOPE_DENIED` | API key scope does not allow this method |
| `ORIGIN_NOT_ALLOWED` | `Origin` header not in the project's allowlist |
| `PROJECT_NOT_FOUND` | Unknown project, or a project the credential cannot see |
| `SPIKE_NOT_FOUND` | Unknown spike, or a spike the credential cannot see |
| `VERSION_EXISTS` | A version with that label already exists |
| `ID_CONFLICT` | A client-chosen spike id or `submission_id` already belongs to another project |
| `IDEMPOTENCY_CONFLICT` | A `submission_id` was reused with different answers |
| `INVALID_JSON` | Body is not valid JSON |
| `RATE_LIMITED` | Per-project rate limit exceeded |
| `BUDGET_EXCEEDED` | Agent-tier monthly budget cap reached |
| `VALIDATION_ERROR` | Request validation failed |
| `NOT_FOUND` | Resource not found |
| `RATE_LIMIT` | Rate limit exceeded |
| `SPIKE_LIMIT` | Per-share spike limit reached (free tier) |
| `SHARE_LIMIT` | Share limit reached (free tier) |
| `UPGRADE_REQUIRED` | Pro feature requires subscription |

### HTTP Status Codes

| Status | Meaning |
|--------|---------|
| 400 | Bad request (invalid body/params) |
| 401 | Unauthorized (missing, revoked, expired, or unknown token) |
| 402 | Upgrade required (project webhooks, email notifications) |
| 403 | Forbidden (origin not allowed, wrong credential kind, insufficient scope) |
| 404 | Resource not found |
| 409 | Idempotency conflict (`ID_CONFLICT`, `IDEMPOTENCY_CONFLICT`) |
| 429 | Rate limit or usage limit exceeded |
| 500 | Server error |

---

## Pro Feature Gating

The following features require a Pro subscription:

- **Password-protected shares** — Returns `403 UPGRADE_REQUIRED` if free user attempts to set a password
- **Webhook URLs** — Returns `403 UPGRADE_REQUIRED` if free user attempts to configure a webhook URL
- **Project webhooks and email notifications** (`PATCH /me/projects/:key`) — `402 UPGRADE_REQUIRED` on the free tier
