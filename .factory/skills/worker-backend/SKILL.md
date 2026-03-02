---
name: worker-backend
description: Implements Cloudflare Worker backend changes (security, validation, auth, Stripe, rate limiting)
---

# Backend Worker

NOTE: Startup and cleanup are handled by `worker-base`. This skill defines the WORK PROCEDURE.

## When to Use This Skill

Features that modify the Cloudflare Worker backend at `../spikes-hosted/worker/`. This includes:
- Security fixes (password hashing, path traversal, XSS)
- Request validation (Zod schemas)
- Database schema changes (migrations, foreign keys)
- Authentication endpoints (magic links, token rotation)
- Stripe webhook handling
- Rate limiting
- API endpoint changes
- Worker modularization

## Work Procedure

1. **Read the feature description thoroughly.** Understand preconditions and expected behavior.

2. **Understand current code.** Read relevant source files in `../spikes-hosted/worker/src/`. The worker may be a single `index.ts` (early milestones) or modularized (later milestones). Read `schema.sql` and any migrations to understand the database.

3. **Write tests FIRST (TDD).** Create test files in `../spikes-hosted/worker/test/` using Vitest + Miniflare patterns:
   - Import env from `cloudflare:test`
   - Use `SELF.fetch()` to test request/response cycles
   - Seed D1 data in `beforeEach`
   - Test both success and error paths
   - Run tests: `cd ../spikes-hosted/worker && npx vitest run` — they MUST fail (red phase)

4. **Write database migrations if needed.** Create numbered migration files in `../spikes-hosted/worker/migrations/`. Migration names should be descriptive (e.g., `002_add_rate_limits_table.sql`).

5. **Implement the feature.** Make the tests pass (green phase). Follow existing patterns in the codebase. If modularizing, create new files in `src/handlers/`, `src/middleware/`, `src/db/`, or `src/utils/`.

6. **Run all tests.** `cd ../spikes-hosted/worker && npx vitest run` — all tests must pass (not just new ones).

7. **Run typecheck.** `cd ../spikes-hosted/worker && npx tsc --noEmit` — must pass with zero errors.

8. **Manually verify via wrangler dev.** Start `cd ../spikes-hosted/worker && npx wrangler dev --port 8787` and test the endpoint with curl. Verify the happy path and at least one error path.

9. **Commit with descriptive message.** Commit changes in `../spikes-hosted/worker/` with a message describing what was done.

## Example Handoff

```json
{
  "salientSummary": "Implemented PBKDF2 password hashing with random salt. Added migration 002_add_password_salt.sql. Created crypto.ts utility module. 8 tests pass covering hash generation, verification, and migration from old SHA-256 hashes. Verified via wrangler dev: POST /shares with password=test123 stores salted hash, GET /s/slug?pw=test123 succeeds, GET /s/slug?pw=wrong fails with 401.",
  "whatWasImplemented": "PBKDF2-HMAC-SHA256 password hashing in src/utils/crypto.ts with 100k iterations and 16-byte random salt. Migration adds password_salt column to shares table. Updated handleCreateShare to use new hashing. Updated handleShareAccess to verify with new format and transparently migrate old SHA-256 hashes.",
  "whatWasLeftUndone": "",
  "verification": {
    "commandsRun": [
      { "command": "cd ../spikes-hosted/worker && npx vitest run", "exitCode": 0, "observation": "8 tests pass: hashPassword generates different hashes for same password with different salts, verifyPassword succeeds with correct password, verifyPassword fails with incorrect, migrateLegacyHash converts SHA-256 to PBKDF2" },
      { "command": "cd ../spikes-hosted/worker && npx tsc --noEmit", "exitCode": 0, "observation": "No type errors" },
      { "command": "curl -X POST http://localhost:8787/shares -H 'Authorization: Bearer test-token' -F 'metadata={\"password\":\"test123\"}' -F 'file=@test.html'", "exitCode": 0, "observation": "201 Created, share with password protection created" },
      { "command": "curl http://localhost:8787/s/test-slug/?pw=test123", "exitCode": 0, "observation": "200 OK, HTML content returned" },
      { "command": "curl http://localhost:8787/s/test-slug/?pw=wrong", "exitCode": 0, "observation": "401 Unauthorized, {\"error\": \"Invalid password\", \"code\": \"AUTH_FAILED\"}" }
    ],
    "interactiveChecks": []
  },
  "tests": {
    "added": [
      {
        "file": "../spikes-hosted/worker/test/crypto.test.ts",
        "cases": [
          { "name": "hashPassword generates unique salts", "verifies": "PBKDF2 salt randomness" },
          { "name": "verifyPassword succeeds with correct password", "verifies": "password verification" },
          { "name": "verifyPassword fails with incorrect password", "verifies": "rejection of wrong passwords" },
          { "name": "migrateLegacyHash converts SHA-256 to PBKDF2", "verifies": "backward-compatible migration" }
        ]
      }
    ]
  },
  "discoveredIssues": []
}
```

## When to Return to Orchestrator

- D1 migration would break existing data and needs a strategy decision
- Stripe webhook testing requires real credentials not available locally
- Feature depends on an auth endpoint that doesn't exist yet
- Worker modularization conflicts with another in-progress feature
- Wrangler dev fails to start (environment issue)
