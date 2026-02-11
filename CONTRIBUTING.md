# Contributing to Spikes

## Quality Bar

We welcome issues and PRs from anyone, including AI-assisted ones. Use whatever tools you like.

However, our time is limited. We reserve the right to close low-quality issues and PRs without explanation. Before submitting, please read what you're having your agent write and make sure it passes basic sanity checks:

- Does the issue describe a real problem with reproduction steps?
- Does the PR actually solve what it claims to solve?
- Does the code run?

This isn't about policing AI usage — it's about respecting everyone's time.

## Reporting Issues

[GitHub Issues](https://github.com/bierlingm/spikes/issues) is the place for:

- Bug reports (with reproduction steps)
- Feature requests
- Questions about behavior

Before opening an issue, search existing ones to avoid duplicates.

## Pull Requests

We welcome PRs for:

- Bug fixes
- Documentation improvements
- Performance improvements
- New features that align with the project philosophy

### Before You Start

For non-trivial changes, **open an issue first** to discuss the approach. This saves everyone time if the change doesn't fit the project direction.

### PR Guidelines

1. **Keep it focused** — One concern per PR
2. **Match the style** — Follow existing patterns in the codebase
3. **Test your changes** — CLI: `cargo test`, Widget: manual verification
4. **Update docs if needed** — But don't over-document

### Commit Style

Clear, imperative messages:

```
Fix element selector escaping for IDs with colons
Add --since flag to list command
```

No need for conventional commits or elaborate formatting.

## Development Setup

```bash
# CLI
cd cli && cargo build

# Widget (no build step)
# Edit widget/spikes.js directly
# Test by opening widget/dashboard.html or any HTML with the widget
```

## Project Philosophy

Spikes values:

- **Simplicity** — Solve the problem, nothing more
- **Zero dependencies** — Widget stays vanilla JS
- **Agent-friendly** — JSON output, scriptable, composable
- **User infrastructure** — No vendor lock-in

PRs that add complexity without clear value will likely be declined.

## Using Spikes on spikes.sh

You might notice the Spikes widget on our own site. That's for quick impressions and dogfooding. For substantive feedback or feature requests, please use GitHub Issues instead — it's where we track and prioritize work.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
