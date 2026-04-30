# GitHub Issue Queue

GitHub Issues are the source of truth for Hayulo execution.

## Queue Rules

- The queue is linear and ordered by `priority/N` labels.
- Milestones group work, but they do not decide execution order.
- Only one open issue should have the `active` label.
- Use `blocked` only when an issue cannot proceed without another issue or external decision.
- Do not close an issue unless implementation, tests, and required docs updates are complete.
- Every merged issue must pass `make test`, `make check`, and any relevant generated API smoke test.

See [Development Loop](development_loop.md) for the repeatable build, test, issue, and repair workflow.

## Labels

Queue state:

- `queue`
- `active`
- `blocked`

Milestones:

- `milestone-0.2`
- `milestone-0.3`
- `milestone-0.4`
- `milestone-0.5`
- `milestone-0.6`
- `milestone-0.7`
- `milestone-0.8`
- `milestone-0.9`
- `milestone-1.0`
- `milestone-1.1`

Areas:

- `area-parser`
- `area-runtime`
- `area-api`
- `area-diagnostics`
- `area-docs`
- `area-tests`
- `area-tooling`

Priority labels:

- `priority/001`
- `priority/002`
- continue in queue order
- `priority/011` starts the post-1.0 improvement queue

## Issue Shape

Every queue issue should include:

- Goal
- Scope
- Acceptance criteria
- Required tests
- Required docs update
- Commands to run

Use the `Hayulo queue item` issue template for all queue work.

## Post-1.0 Queue

The post-1.0 queue starts with measurement:

- Queue 011: add the LLM generation benchmark harness.
- Later syntax, type-system, effect, and API-action changes should cite benchmark evidence when possible.
- New benchmark-related work should use `area-tooling`, `area-tests`, and `area-docs` unless a more specific parser/runtime/API area owns the change.
