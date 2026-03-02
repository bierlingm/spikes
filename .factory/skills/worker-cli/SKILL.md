---
name: worker-cli
description: Implements Rust CLI changes (tests, commands, token storage, error handling)
---

# CLI Worker

NOTE: Startup and cleanup are handled by `worker-base`. This skill defines the WORK PROCEDURE.

## When to Use This Skill

Features that modify the Rust CLI at `./cli/`. This includes:
- Adding unit tests and integration tests
- New CLI commands (delete, resolve, login, logout, whoami, billing, usage, upgrade)
- Token storage changes
- Error message improvements
- Dependency changes (adding/removing crates)
- Pull/push sync improvements

## Work Procedure

1. **Read the feature description thoroughly.** Understand preconditions and expected behavior.

2. **Understand current code.** Read relevant source files in `./cli/src/`. Key files:
   - `main.rs` — Command routing (clap derive)
   - `spike.rs` — Data models
   - `storage.rs` — JSONL read/write
   - `config.rs` — TOML configuration
   - `commands/*.rs` — Individual command implementations
   - `Cargo.toml` — Dependencies

3. **Write tests FIRST (TDD).** 
   - **Unit tests:** Add `#[cfg(test)] mod tests { ... }` in the same file as the code being tested. Use `tempfile` for filesystem tests.
   - **Integration tests:** Add files in `./cli/tests/` using `assert_cmd` and `predicates`.
   - Run `cd cli && cargo test` — new tests MUST fail (red phase).

4. **Add dependencies if needed.** Add to `[dev-dependencies]` in `cli/Cargo.toml`:
   - `assert_cmd = "2"` for CLI integration tests
   - `tempfile = "3"` for temp directories
   - `wiremock = "0.6"` for HTTP mocking
   - `predicates = "3"` for test assertions
   - `reqwest = { version = "0.12", features = ["json"] }` for HTTP test client

5. **Implement the feature.** Make the tests pass (green phase). Follow existing clap derive patterns in `main.rs`. Match the coding style of existing commands.

6. **Run all tests.** `cd cli && cargo test` — all tests must pass.

7. **Build check.** `cd cli && cargo build` — must succeed with zero warnings (fix any warnings you introduce).

8. **Manually verify the command.** Run the command directly and verify output. Test with `--json` flag if applicable. Test error paths (missing args, bad input).

9. **Commit with descriptive message.**

## Example Handoff

```json
{
  "salientSummary": "Implemented `spikes delete <id>` command with prefix support and --force flag. Added 6 unit tests for storage deletion and 2 integration tests via assert_cmd. Verified: delete by full ID works, delete by 4-char prefix works, prefix collision prompts user, --force skips confirmation, --json outputs {\"deleted\": \"<id>\"}.",
  "whatWasImplemented": "New command `spikes delete <id>` in cli/src/commands/delete.rs. Supports ID prefix matching (minimum 4 chars), interactive confirmation (skippable with --force), and --json output. Updated main.rs with Delete variant. Added storage::remove_spike() function.",
  "whatWasLeftUndone": "",
  "verification": {
    "commandsRun": [
      { "command": "cd cli && cargo test", "exitCode": 0, "observation": "14 tests pass including 6 new delete tests" },
      { "command": "cd cli && cargo build", "exitCode": 0, "observation": "No warnings" },
      { "command": "cd cli && cargo run -- delete abc123 --force --json", "exitCode": 0, "observation": "{\"deleted\": \"abc123def456...\"}" },
      { "command": "cd cli && cargo run -- delete nonexistent --force", "exitCode": 1, "observation": "Error: No spike found with ID starting with 'nonexistent'" }
    ],
    "interactiveChecks": []
  },
  "tests": {
    "added": [
      {
        "file": "cli/src/commands/delete.rs",
        "cases": [
          { "name": "delete_spike_by_full_id", "verifies": "spike removed from JSONL" },
          { "name": "delete_spike_by_prefix", "verifies": "4-char prefix matching" },
          { "name": "delete_nonexistent_returns_error", "verifies": "error handling" }
        ]
      },
      {
        "file": "cli/tests/integration_delete.rs",
        "cases": [
          { "name": "delete_with_force_flag", "verifies": "CLI --force skips confirmation" },
          { "name": "delete_json_output", "verifies": "JSON output format" }
        ]
      }
    ]
  },
  "discoveredIssues": []
}
```

## When to Return to Orchestrator

- A new dependency is needed that isn't in the approved list
- The command needs to call a hosted API endpoint that doesn't exist yet
- Removing ratatui/crossterm causes unexpected compilation errors
- Feature requires changes to the data model (spike.rs Spike struct) that could break other commands
