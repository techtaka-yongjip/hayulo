# Release Checklist

Use this checklist for Hayulo 1.0 and later stable-core releases.

## Local Requirements

- Python 3.11 or newer
- Node.js 20 or newer for generated API smoke tests
- GitHub CLI when closing queue issues

## Commands

```bash
make queue-active
make release-check
git status --short
```

`make release-check` runs:

- full unit tests
- project and example checks
- formatter checks
- script examples
- API build
- generated API smoke test
- `hayulo --version`
- `git diff --check`

## Manual Review

Before tagging:

- README status and install instructions match the package version
- [SPEC.md](../SPEC.md) lists the stable contract
- [stable_contract_1_0.md](stable_contract_1_0.md) matches implemented CLI commands
- [compatibility.md](compatibility.md) and [migration_policy.md](migration_policy.md) are current
- [standard_library_core.md](standard_library_core.md) matches implemented built-ins
- active queue issue is closed or intentionally left open
- generated example output is not committed accidentally
- no private or local-only content is staged

## Tagging

```bash
git tag v1.0.0
git push origin v1.0.0
```

Only tag after the queue issue for the release is closed and the worktree contains no unintended changes.
