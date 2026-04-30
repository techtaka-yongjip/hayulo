# CI Examples

Hayulo public alpha uses a small CI gate:

```bash
make test
make check
make verify
```

`make verify` also runs formatter checks, examples, API build, and the generated API smoke test. Use it for local release checks when Node.js is available.

## GitHub Actions

The repository includes `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: "3.11"
      - uses: actions/setup-node@v4
        with:
          node-version: "20"
      - run: python -m pip install -e .
      - run: make test
      - run: make check
      - run: make verify
```

## Minimal CI Without Node

If a CI environment cannot run Node.js, use:

```bash
make test
make check
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=src python3 -m hayulo format --check .
```

This skips generated REST API smoke tests but still validates the compiler, diagnostics, examples, project checks, and public-alpha docs tests.

## Release Checklist

Before tagging a public alpha build:

```bash
make queue-active
make verify
git status --short
```

The active GitHub issue should match the queue item being released, and the worktree should contain no unintended source changes.
