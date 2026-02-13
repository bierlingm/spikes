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

List spikes (admin endpoint).

- **Auth:** Query token
- **Query params:** `project`, `page`, `reviewer`, `rating`

```bash
# All spikes
curl "https://your-api.example/spikes?token=$SPIKES_TOKEN"

# Filter by project and rating
curl "https://your-api.example/spikes?token=$SPIKES_TOKEN&project=my-project&rating=no"
```

**Response:**
```json
[
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
]
```

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

## Error Responses

All endpoints return errors in this format:

```json
{
  "ok": false,
  "error": "Description of what went wrong"
}
```

| Status | Meaning |
|--------|---------|
| 400 | Bad request (invalid body/params) |
| 401 | Unauthorized (missing/invalid token) |
| 404 | Resource not found |
| 500 | Server error |
