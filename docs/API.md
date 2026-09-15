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

`selector`, `xpath`, `elementText`, `boundingBox` are present only for `type: "element"`. `reviewer.email` is present only when `data-collect-email="true"`. `page` is `document.title` (falls back to the path). `id` is ignored; the server generates its own.

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

Unknown fields are ignored.

**Origin check.** The server compares the request `Origin` header with the project's `allowed_origins` list (default `["http://localhost:*", "null"]`, set at `POST /projects` or `PATCH /me/projects/:key`):

- Each entry is matched against the whole `Origin` string; `*` matches any run of characters except `/`. So `https://*.example.com` matches `https://app.example.com` but not `https://example.com`; `http://localhost:*` matches any port; `https://*` matches any HTTPS origin.
- Scheme is strict: `http://*` never matches an `https://` origin.
- The literal entry `null` allows a missing `Origin` header and the literal header value `Origin: null` (file URLs, sandboxed iframes, `curl` without `-H Origin`).
- Headers containing CR, LF, NUL, or a comma are rejected.
- Mismatch → `403 ORIGIN_NOT_ALLOWED`. Unknown project → `404 PROJECT_NOT_FOUND`.

**Preflight.** `OPTIONS /spikes` answers `204` with `Access-Control-Allow-Origin: *`, `Access-Control-Allow-Headers: Content-Type, Authorization`, `Access-Control-Allow-Methods: GET, POST, DELETE, PATCH, OPTIONS`. Every response, including errors, carries `Access-Control-Allow-Origin: *`, so a browser can read the error body.

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

**Errors:**

| Status | `code` | When |
|--------|--------|------|
| 400 | `INVALID_JSON` | body is not JSON |
| 400 | `VALIDATION_ERROR` | field rules above (`details[]` names the field) |
| 403 | `ORIGIN_NOT_ALLOWED` | `Origin` not in the project's allowlist |
| 404 | `PROJECT_NOT_FOUND` | unknown `project`/`projectKey` |
| 404 | `NOT_FOUND` | `share_id` given but unknown |
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
  - `resolved` — `true` / `false`
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

`resolved: true` is `status: "addressed"`; `resolved: false` is `status: "open"`. `addressed_in` may accompany any status and may be `null`. Returns the updated spike.

---

#### DELETE /spikes/:id

Deletes the spike and its replies. `204`.

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

No bearer token. Authorised exactly like `POST /spikes`: the `Origin` header must match the project's `allowed_origins` (`403 ORIGIN_NOT_ALLOWED`), unknown project → `404 PROJECT_NOT_FOUND`, rate limit 120/min per IP. Responses never include reviewer emails, user agents, or bounding boxes.

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

`201 { "id": "a1", "questionId": "q1", "createdAt": "…" }`. `400 VALIDATION_ERROR` for an empty body, `404 NOT_FOUND` for an unknown or closed question.

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

Configured per project with `PATCH /me/projects/:key`. Delivery uses the same signing and retry logic as share webhooks (HMAC-SHA256 of the raw body with the project's `webhook_secret` in the signature header).

```json
{
  "event": "spike.created",
  "project": "my-project",
  "timestamp": "2026-09-15T08:00:00.000Z",
  "spike": { "…SpikeResponse" },
  "reply": null,
  "answer": null
}
```

`event` is one of `spike.created`, `spike.replied` (`reply` set), `spike.status_changed`, `question.answered` (`answer`: `{ questionId, questionTitle, reviewer, body, createdAt }`, `spike` null).

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

Endpoints are rate limited with sliding window counters. When rate limited, responses include a `Retry-After` header with seconds until retry.

| Endpoint | Limit | Window | Identifier |
|----------|-------|--------|------------|
| `POST /spikes` | 60 | 1 minute | Client IP |
| `POST /spikes` | 60 | 1 minute | Project key |
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
| 429 | Rate limit or usage limit exceeded |
| 500 | Server error |

---

## Pro Feature Gating

The following features require a Pro subscription:

- **Password-protected shares** — Returns `403 UPGRADE_REQUIRED` if free user attempts to set a password
- **Webhook URLs** — Returns `403 UPGRADE_REQUIRED` if free user attempts to configure a webhook URL
- **Project webhooks and email notifications** (`PATCH /me/projects/:key`) — `402 UPGRADE_REQUIRED` on the free tier
