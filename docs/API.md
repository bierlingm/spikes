# Spikes API Reference

API contract for self-hosted Spikes backends.

## Authentication

Two authentication methods:

| Method | Use Case | Header/Param |
|--------|----------|--------------|
| Bearer token | Share management (CLI) | `Authorization: Bearer <token>` |
| Query param | Admin operations | `?token=<SPIKES_TOKEN>` |

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

Create a spike (called by widget).

- **Auth:** None (public endpoint)
- **Content-Type:** `application/json`

```bash
curl -X POST https://your-api.example/spikes \
  -H "Content-Type: application/json" \
  -d '{
    "type": "element",
    "projectKey": "my-project",
    "page": "/",
    "url": "https://example.com/",
    "selector": "button.submit",
    "rating": "love",
    "comments": "Perfect placement",
    "reviewer": { "id": "r123", "name": "Jane" },
    "viewport": { "width": 1920, "height": 1080 },
    "timestamp": "2024-01-15T10:30:00Z"
  }'
```

**Response:**
```json
{
  "ok": true,
  "id": "spike_xyz789"
}
```

---

#### GET /spikes

List spikes with cursor-based pagination.

- **Auth:** None when filtering by `project`; Query token otherwise
- **Query params:**
  - `project` — Filter by project/share_id (public when provided)
  - `page` — Filter by page path
  - `reviewer` — Filter by reviewer ID
  - `rating` — Filter by rating value
  - `cursor` — Pagination cursor (optional)
  - `limit` — Items per page (default 100, max 1000)

```bash
# Public access when filtering by project
curl "https://your-api.example/spikes?project=my-project"

# Admin access with token
curl "https://your-api.example/spikes?token=$SPIKES_TOKEN"

# With pagination
curl "https://your-api.example/spikes?project=my-project&limit=50&cursor=abc123"
```

**Response:**
```json
{
  "data": [
    {
      "id": "spike_xyz789",
      "type": "element",
      "projectKey": "my-project",
      "page": "/",
      "url": "https://example.com/",
      "selector": "button.submit",
      "rating": "love",
      "comments": "Perfect placement",
      "reviewer": { "id": "r123", "name": "Jane" },
      "viewport": { "width": 1920, "height": 1080 },
      "timestamp": "2024-01-15T10:30:00Z"
    }
  ],
  "next_cursor": "abc123"
}
```

`next_cursor` is `null` when there are no more pages.

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
| `AUTH_FAILED` | Invalid or expired credentials |
| `AUTH_REQUIRED` | Authentication required |
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
| 401 | Unauthorized (missing/invalid token) |
| 403 | Forbidden (upgrade required or insufficient permissions) |
| 404 | Resource not found |
| 429 | Rate limit or usage limit exceeded |
| 500 | Server error |

---

## Pro Feature Gating

The following features require a Pro subscription:

- **Password-protected shares** — Returns `403 UPGRADE_REQUIRED` if free user attempts to set a password
- **Webhook URLs** — Returns `403 UPGRADE_REQUIRED` if free user attempts to configure a webhook URL
