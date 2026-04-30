# Hayulo Roadmap

This roadmap is a planning document, not a guarantee. Hayulo is pre-alpha and will change as the project learns.

## 1.0 Direction

Hayulo 1.0 targets a small but stable general-purpose language and toolchain for coding-agent-assisted software creation.

REST API generation remains the flagship integration example, but 1.0 is not only an API generator. The 1.0 target is:

- stable core grammar
- stable project format
- stable formatter
- stable diagnostic schema
- stable test runner
- documented standard library core
- package foundation
- backwards compatibility and migration policy

## Execution Model

GitHub Issues are the execution queue.

- There is one linear queue, ordered by `priority/N` labels.
- Milestones group work, but priority labels decide execution order.
- Only one issue should have the `active` label.
- No issue closes without relevant tests and docs updates.
- Every merged issue must pass `make test`, `make check`, and any relevant API smoke test.

See [Issue Queue](docs/issue_queue.md) for labels, issue shape, and operating rules.

## 0.1 Seed Prototype

Status: implemented.

Completed:

- tiny runnable script language
- lexer, parser, AST, interpreter
- CLI with `run`, `test`, and `check`
- JSON diagnostics
- examples and tests
- REST API MVP path with `hayulo build`
- generated OpenAPI JSON and smoke tests
- project documentation

## 0.2 Stabilization

Goal: make the current prototype reliable enough for regular agent iteration.

- Add `hayulo --version`.
- Remove uncaught Python tracebacks for normal CLI errors.
- Improve JSON diagnostics consistency.
- Add diagnostic snapshot tests.
- Expose `intent` metadata in `hayulo check --json`.

## 0.3 Core Language

Goal: grow the script language beyond toy examples.

- Add list literals, map literals, and indexing.
- Add `for` loops.
- Add basic records in the core language.
- Add focused examples and tests for every supported language feature.

## 0.4 Static Checking

Goal: catch common generated-code mistakes before runtime.

- Add name resolution.
- Add function arity checks before runtime.
- Add basic local type inference.
- Add return type checks.
- Add record field checks.
- Introduce a stable diagnostic code namespace.

## 0.5 Project System

Goal: support real projects while preserving single-file mode.

- Add `hayulo.toml`.
- Add `hayulo new`.
- Add `src/` and `tests/` conventions.
- Add project-wide `hayulo check`.
- Add project-wide `hayulo test`.

## 0.6 Formatter and Repair Protocol

Goal: make generated code and repair loops repeatable.

- Add `hayulo format`.
- Add `hayulo format --check`.
- Define stable JSON diagnostic schema v0.1.
- Add failing-test JSON schema.
- Add `hayulo summarize --json`.
- Create repair benchmark fixtures.

## 0.7 App-Building Preview

Goal: prove Hayulo can build useful small software, with REST APIs as the flagship example.

- Continue the REST API MVP as the main app-building proof.
- Add `hayulo new api`.
- Add `hayulo serve`, or document the generated-server workflow if `serve` is deferred.
- Improve generated OpenAPI and smoke tests.
- Add TypeScript generation, or explicitly defer it with a documented reason.

## 0.8 Effects and Permissions Preview

Goal: make risky generated behavior visible.

- Add syntax and design notes for effects.
- Add `hayulo.toml` permission declarations.
- Add missing-permission diagnostics.
- Add deny-list enforcement for generated API actions.

## 0.9 Public Alpha

Goal: make the project usable by outside testers.

- Add documentation site or publishable docs structure.
- Add editor syntax support plan or minimal grammar file.
- Add CI examples.
- Publish repair benchmark results.
- Freeze candidate syntax subset.

## 1.0 Stable Core

Goal: stabilize the language and toolchain contract.

- Freeze core grammar.
- Freeze project format.
- Freeze formatter behavior.
- Freeze diagnostic schema.
- Document standard library core.
- Add backwards compatibility policy.
- Add migration policy for breaking changes.

## Roadmap Principle

Every release should make Hayulo better at one or more of these:

- build useful programs
- detect errors earlier
- explain errors better
- help coding agents repair code
- preserve human intent
- make generated code safer
