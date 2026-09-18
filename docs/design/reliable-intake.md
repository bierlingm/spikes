# Reliable intake: nothing a reviewer sends gets lost

Status: implemented 2026-09-18 on branch `reliable-intake` (bierlingm/spikes-hosted and bierlingm/spikes). Not deployed to production. Written after four intake failures were reproduced against production on 18 Sep 2026. Companion to `feedback-loop-v2.md`; everything in `feedback-loop-v2-api.md` still holds except where this doc changes it.

## 1. What broke

1. **Concurrent first requests fail with 500.** `checkAndRecordRateLimit` did a SELECT, then `recordRateLimit` did another SELECT and an INSERT when no row existed. `rate_limits` has `PRIMARY KEY (endpoint, identifier)`, so when several `POST /spikes` from an IP with no row arrive together, every INSERT but one hits the constraint and the request answers `500 INTERNAL_ERROR`. Reproduced on production: after deleting our IP's `POST /spikes` row, 8 parallel POSTs gave 6 × 500 and 2 × 201. That is every reviewer's first burst. The same helper guards `/public/*`, `POST /shares`, `POST /projects`, `POST /auth/api-key` and share password attempts.
2. **Retries create duplicates.** `POST /spikes` ignores the client `id` and always generates one. A client whose response was lost cannot retry safely.
3. **No atomic multi-answer intake.** A decision form with five text answers has to fire five `POST /public/questions/:id/answers`. Some can land and some not, and nothing makes a retry safe.
4. **Webhooks are best-effort.** One retry after 5 s inside `ctx.waitUntil`, nothing persisted. A receiver that is down for a minute loses the event.

## 2. Contract

### 2.1 Rate limiting is one statement

Every limit is checked and counted by one `INSERT … ON CONFLICT(endpoint, identifier) DO UPDATE … RETURNING request_count, window_start`. The CASE expressions roll the window when `window_start` is older than the window. The count is capped at `limit + 1`, so rejected requests do not grow it. Allow or deny is decided from the returned row, with no check-then-write. Limits, windows, `429` bodies (`code`, `retry_after`) and `Retry-After` headers are unchanged. `checkRateLimit` and `recordRateLimit` are removed.

### 2.2 Idempotent `POST /spikes`

- Optional body field `id`. When it is a UUID it is the spike's id and the idempotency key. A value that is not a UUID is ignored and the server generates an id, as before. This is deliberate: the widget sends 21-character nanoids today, and rejecting them would break every installed widget.
- Optional header `Idempotency-Key: <uuid>` does the same. A key that is not a UUID answers `400 VALIDATION_ERROR`. When both are sent and they differ: `400`.
- First write: `201 { ok: true, id }`, as today.
- Replay (same id, same project): `200 { ok: true, id, duplicate: true }`. No webhook, no email, no meter event, no share-limit count. Rate limits count the replay like any request.
- Same id under another project: `409 ID_CONFLICT`.
- The insert is `INSERT … ON CONFLICT(id) DO NOTHING`, so two concurrent requests with one id produce one row. Whichever request loses answers like a replay.

### 2.3 Batch answers: `POST /public/submissions`

Public, like `POST /spikes`: project must exist, same origin allowlist (a missing or `null` Origin passes only if the project lists `"null"`), per-IP limit (60/min, bucket `POST /public/submissions`) and per-project limit (60/min, bucket `project:<key>`), CORS open.

```json
{
  "project": "yvs",
  "submission_id": "5b0e8c1e-2a0c-4f7e-9d8a-3f1c2b4a5d6e",
  "reviewer": { "id": "r_abc", "name": "Tyler" },
  "page": "/decisions",
  "url": "https://statecraft.systems/decisions",
  "answers": [
    { "question_id": "q-uuid", "body": "Option B" },
    { "key": "budget", "title": "Budget ceiling", "body": "40k" }
  ]
}
```

- `submission_id` is required, a UUID minted by the client once per submit and reused for retries. Mint a new one when the answers change.
- 1 to 50 answers, `body` 1 to 4000 characters after trimming. `title` up to 200 characters, `key` up to 100 characters.
- With `question_id`: it must be an open question of that project, else `400 VALIDATION_ERROR` with `details[].field = "answers.<i>.question_id"`. `title` defaults to the question's title.
- Without `question_id`: `title` is required and `key` is optional. These are free-form answers for embedders with their own forms.
- `201 { ok: true, submissionId, answerCount }`.
- Replay with the same content: `200 { ok: true, submissionId, answerCount, duplicate: true }`, nothing fires. Same `submission_id` with different answers: `409 IDEMPOTENCY_CONFLICT`. Same `submission_id` under another project: `409 ID_CONFLICT`.

Storage (migration 021): `submissions (id = submission_id, project_key, reviewer_id, reviewer_name, page, url, answer_count, content_hash, created_at)` and `submission_answers (id, submission_id, position, question_id, key, title, body)`. Every answer of a submission is a `submission_answers` row. Answers tied to a question are also written to `answers` with the same id, so question listings, answer counts, the widget panel and the email digest need no change. Everything is written in one D1 batch, which is one transaction. Answer ids are deterministic (`<submission_id>:<position>`) and each answer insert only runs if the `submissions` row carries this request's content hash. Concurrent replays therefore cannot duplicate rows, and a conflicting replay cannot add any.

Widget: `Spikes.submit(answers, opts)` mints the `submission_id`, sends the stored reviewer identity, and retries 429, 5xx and network errors with the same id (`docs/widget-attributes.md`). `widget/spikes.js` and `site/spikes.js` stay identical.

Events: one `submission.received` with the full submission, then one `question.answered` per linked answer. The second keeps existing webhook consumers working. Both go through the webhook outbox (2.4). The email digest counts free-form answers as answers.

Owner side (user token, account key, or that project's key; same rules as `/me/projects/:key/questions`):

- `GET /me/projects/:key/submissions[?since=<ISO>&limit=<1..200>]` returns `{ data: [Submission] }`, default limit 50. Without `since` the list is newest first. With `since` it holds only rows with `created_at > since`, oldest first, so a poller can pass the last `createdAt` as the next `since`.
- `GET /me/projects/:key/submissions/:id`: one `Submission`, or `404 NOT_FOUND`.

`Submission` = `{ id, projectKey, reviewer: { id, name }, page, url, answerCount, createdAt, answers: [{ id, position, questionId, key, title, body }] }`.

### 2.4 Durable webhook delivery (outbox)

Table `webhook_deliveries (id, source 'project'|'share', source_id, event, payload, url, status pending|delivered|failed, attempts, next_attempt_at, last_status, last_error, created_at, delivered_at)` (migration 022).

- `notifyProjectEvent`, and the share webhook in `POST /spikes`, first insert the outbox row inside the request, before the response. Then they try delivery once under `ctx.waitUntil`. If the insert fails (for example, the migration is not applied yet), they fall back to the old direct delivery. The event is never dropped because of the outbox.
- A 2xx response marks the row `delivered`. 408, 429, 5xx, network errors and timeouts (10 s) are retried after 1 m, 5 m, 30 m, 2 h, 6 h and 16 h, so there are 7 attempts over about 24.5 h. After that the row is `failed`. Any other 4xx fails at once, as before.
- A `scheduled` handler (cron `* * * * *`, staging and production) claims up to 25 due rows in one `UPDATE … RETURNING` that pushes their `next_attempt_at` out by a 2-minute lease, then delivers them in parallel. A worker that dies mid-attempt leaves the row due again once the lease expires. Once an hour the handler deletes delivered rows older than 7 days and failed rows older than 30 days.
- Retries re-read the source. The current URL and secret are used, and the tier gate (pro/agent for projects, pro for shares) is re-checked. A removed webhook or a downgraded owner marks the row `failed` (`last_error = "webhook_removed"`).
- Every request carries `X-Spikes-Delivery-Id` (stable across retries), `X-Spikes-Event`, `X-Spikes-Delivery-Attempt` (1-based) and the existing `X-Spikes-Signature: t=<unix>,v1=<hmac>` over `<t>.<body>`. The body is stored once and sent byte-for-byte on every attempt. It also carries `deliveryId`, which the signature covers.
- **Semantics: at least once.** A receiver can see the same delivery more than once, for example when it processed a request but the response was lost. Receivers dedupe on `X-Spikes-Delivery-Id`. Ordering is not guaranteed across deliveries.

## 3. Migration

- `021_submissions.sql`: `submissions`, `submission_answers`, indexes. Rollback: `rollbacks/021_submissions.rollback.sql` (destructive: drops submissions and free-form answers; linked answers stay in `answers`).
- `022_webhook_deliveries.sql`: `webhook_deliveries` plus the due index. Rollback: `rollbacks/022_webhook_deliveries.rollback.sql` (drops the outbox, losing pending retries).
- No change to existing tables. `rate_limits` keeps its shape. `schema.sql` is updated.
- `wrangler.toml` gains `[triggers] crons = ["* * * * *"]` for the default, `staging` and `production` envs.

## 4. Rollout

1. Merge the spikes-hosted PR. CI deploys staging. Apply migrations to staging: `npx wrangler d1 migrations apply spikes-staging-db --remote --env staging`.
2. Smoke on staging. Parallel `POST /spikes` from a fresh IP give no 500. Replay a spike id and get 200 `duplicate`. `POST /public/submissions`, then replay it. A webhook pointed at a failing receiver shows up as `pending` with `attempts` rising, then `delivered` once the receiver is back.
3. Production, in this order: apply the migrations (`npx wrangler d1 migrations apply spikes-sh-db --remote --env production`), then deploy (tag `v*`, or `workflow_dispatch` with `production`). The code tolerates a missing outbox table (it falls back to direct delivery), but `/public/submissions` needs migration 021.
4. Confirm the cron is registered (`wrangler deployments`/dashboard → Triggers) and watch `webhook_deliveries` for `failed` rows in the first day.
5. Roll back by redeploying the previous worker. The new tables can stay; they are inert without the code.

## 5. Follow-ups (not in this change)

- `spikes submissions list|show` in the CLI and a `get_submissions` MCP tool, reading the owner endpoints above.
- The widget could send UUID spike ids (`crypto.randomUUID()`) so its own POSTs become retry-safe.
- An owner-facing view of failed webhook deliveries (`GET /me/projects/:key/webhook-deliveries`) with a manual redeliver.
- Transactional outbox. The outbox row is written after the data write rather than in the same batch. A worker killed between the two statements can still lose the event, but that window is milliseconds instead of the receiver's downtime.
