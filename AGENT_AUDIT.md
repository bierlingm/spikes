# Agent-friendliness audit — spikes

Framework: @poteto's inverse pyramid (design > static analysis > minimal prose > human review).
Already strong: Rust CLI with 523 tests + clippy `-D warnings` in CI, Playwright widget suite, AGENTS.md.

## Evidence (221 commits, PRs #11–#27)

1. **Docs restate facts that drift from code — the most common rework.** `468a661` removed a stale
   "284 tests" literal; `287a4d4` re-aligned spikes-mcp to 0.4.1 "with README and server manifest";
   `b837d41` shortened `server.json` for the registry limit; `f64a9c0`, `cb9a337`, `eb8b01c`, `3765da2`
   are README catch-ups. Still drifting today: AGENTS.md said CLI v0.4.0 while `cli/Cargo.toml` is
   0.5.0 (`669eb76`), README says "All 9 MCP tools" in the llms.txt bullet while the code has 16, and
   README's Action example pins `bierlingm/spikes/action@v0.3.1`. README (23 commits) and `site/index.html`
   (22) are among the most-churned files.
2. **Review rounds catch what tests don't.** `9613f51` and `278e64e` ("review findings on
   feedback-loop-v2"), `445cbfd`/`67a5304` (scrutiny rounds 1–2 tightening the same marker test),
   `fe03025` (structured JSON error in an uninitialized dir), `e0c5dd1` (inject loaded config from the wrong dir).
   `cli/src/main.rs` is the top churn file (24 commits) — 36 subcommands dispatched from one file.
3. **Widget tests never run on GitHub.** `workflows-local/widget.yml` is local-only ("cost-driven
   decision"); CI (`test.yml`) runs only the Rust CLI. The widget is the product's surface.
   Also CI uses `rust-toolchain@nightly` while users build with stable — and deps now need > rustc 1.85.

## Plan (by pyramid)

**1. Codebase design**
- Generate doc facts instead of writing them: the MCP tool list/count, subcommand list and version
  rendered into README/llms.txt/AGENTS.md from `spikes --help` / the tool router (a `cargo xtask docs`).
- Split `main.rs` dispatch so each subcommand owns its args + handler + JSON error shape; one shared
  `CommandError -> JSON` path would have made `fe03025` impossible to get wrong per command.
- A single `ProjectContext::load(dir)` used by every command (fixes the `e0c5dd1` class).

**2. Static analysis**
- (done) `scripts/check-doc-facts.sh`: checks version and MCP tool count in AGENTS/CLAUDE/README/llms.txt
  against code. It caught the live 0.4.0 drift, fixed here.
- (staged) `docs/agent-audit/ci-additions.yml`: add it to `test.yml`; run widget tests on widget-touching PRs.
- Next: pin CI to stable + an MSRV job, so CI matches what `install.sh` users compile.

**3. Minimal prose** — (done) verify skill + Verify pointer in AGENTS.md. Consider trimming AGENTS.md
counts once generated.

**4. Human review** — product/landing copy.

## Open questions resolved by prototyping
- *Do the counts in docs match code?* Counted `#[tool(` = 16 and `spikes --help` = 36 subcommands —
  match; the version did not.
- *Is CI green locally?* With stable rustc 1.99: build ✔, 523 tests ✔, clippy `-D warnings` ✔.
  With the box's rustc 1.85 the build fails (dependency MSRV) — worth an MSRV note.

## What this PR changes
`scripts/check-doc-facts.sh`, AGENTS.md version fix + Verify section, verify skill, staged CI additions.
No CLI/widget code, site content, deploy or release config changed. README drift (9 tools, action@v0.3.1) reported, not edited.
