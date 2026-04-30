# Development Loop

Hayulo should mature through a repeated loop:

1. inspect the queue
2. work exactly one active issue
3. build and test the project
4. turn newly found problems into queue issues
5. resolve the active issue with implementation, tests, and docs
6. close it and promote the next issue

The loop is intentionally small so a human or coding agent can run it many times without inventing process each time.

## Local Commands

Use one command as the local quality gate:

```sh
make verify
```

`make verify` runs:

- `make test`
- `make check`
- `make examples`
- `make api-smoke`

Use these helpers to inspect GitHub Issues from the repo:

```sh
make queue-active
make queue-status
```

`make queue-active` should show exactly one open issue.

## One-Issue Work Cycle

Start:

```sh
git pull --ff-only
make queue-active
make verify
```

Implement:

- read the active issue
- change only what the issue requires
- add or update tests
- update relevant docs
- keep unrelated cleanup out of the patch

Verify:

```sh
make verify
git status --short
git diff --stat
```

Finish:

- commit the implementation
- push to `main` or open a pull request
- comment on the issue with the commit and verification commands
- close the issue only after the required tests and docs are done
- remove `active` from the closed issue
- add `active` to the next open issue by priority

## Creating Issues From Problems

If verification, examples, or manual testing expose a new problem, create a queue issue instead of burying it in notes.

Each issue must include:

- Goal
- Scope
- Acceptance criteria
- Required tests
- Required docs update
- Commands to run

Use labels consistently:

- always add `queue`
- add one `priority/N` label
- add one milestone label
- add relevant `area-*` labels
- add `blocked` only when the issue cannot proceed

Do not add `active` unless the issue is the single current work item.

## Maturity Signals

The language is becoming more mature when:

- fewer failures are found by manual testing
- diagnostics become stable enough for snapshots
- every syntax feature has examples and tests
- public CLI behavior is documented
- generated API smoke tests catch regressions
- issues close with implementation, tests, and docs together

This loop is the operating system for the 1.0 roadmap. The roadmap says what to build; the queue and `make verify` decide whether each step is actually done.
