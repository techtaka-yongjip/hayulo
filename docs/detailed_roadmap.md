# Detailed Roadmap

This document expands `ROADMAP.md`. GitHub Issues are the source of truth for execution order; this document explains the milestone intent and acceptance boundaries.

## Version 0.1: Seed Prototype

Status: implemented.

Completed:

- lexer, parser, AST, and interpreter for the script prototype
- CLI commands: `run`, `test`, `check`, and `build`
- JSON diagnostics for script and API checks
- REST API MVP parser and generator
- generated OpenAPI JSON, Node server, and smoke test
- examples and CI coverage

Success criteria:

- `hayulo run examples/hello.hayulo` works
- `hayulo test examples/hello.hayulo` works
- `hayulo check examples/hello.hayulo --json` works
- `hayulo check examples/todo_api/main.hayulo --json` works
- `hayulo build examples/todo_api/main.hayulo --out /tmp/hayulo-generated --json` works

## Version 0.2: Stabilization

Goal: make the current prototype reliable enough for regular coding-agent iteration.

Work:

- add `hayulo --version`
- return structured diagnostics instead of uncaught Python tracebacks for normal file, parse, runtime, and API build errors
- make JSON diagnostics consistent enough for snapshot tests
- add diagnostic snapshot fixtures
- expose parsed `intent` metadata in `hayulo check --json`

Success criteria:

- common invalid inputs produce stable JSON diagnostics
- supported examples still pass
- contributors can use diagnostics without parsing terminal prose

## Version 0.3: Core Language

Goal: grow the general language beyond toy script examples.

Work:

- add list literals, map literals, and indexing
- add `for` loops
- add basic records in the core language
- add focused examples and tests for every supported feature
- update `SPEC.md` with each public syntax addition

Success criteria:

- a user can write small data-processing scripts without API-specific syntax
- unsupported syntax is rejected with useful diagnostics
- all examples are covered by tests

## Version 0.4: Static Checking

Goal: catch common generated-code mistakes before runtime.

Work:

- add name resolution
- add function arity checks before runtime
- add basic local type inference
- add return type checks
- add record field checks
- introduce stable diagnostic code namespace

Success criteria:

- common agent-generated mistakes produce useful errors
- type and name errors can be repaired from JSON diagnostics
- diagnostic codes are documented before becoming public contract

## Version 0.5: Project System

Goal: support real projects while keeping single-file mode working.

Work:

- add `hayulo.toml`
- add `hayulo new`
- add `src/` and `tests/` conventions
- add project-wide `hayulo check`
- add project-wide `hayulo test`
- keep single-file `run`, `test`, `check`, and `build` behavior compatible

Success criteria:

- a user can create a project and run all tests
- examples can be organized as real projects
- generated output conventions are documented

## Version 0.6: Formatter and Repair Protocol

Goal: make generated code and repair loops repeatable.

Work:

- add `hayulo format`
- add `hayulo format --check`
- define stable JSON diagnostic schema v0.1
- add failing-test JSON schema
- add `hayulo summarize --json`
- create repair benchmark fixtures

Success criteria:

- generated patches produce stable formatted code
- repair agents can consume diagnostics and failing-test output without scraping prose
- benchmark fixtures protect diagnostic stability

## Version 0.7: App-Building Preview

Goal: prove Hayulo can build useful small software, with REST APIs as the flagship integration.

Work:

- continue REST API MVP as the main app-building proof
- add `hayulo new api`
- add `hayulo serve`, or document the generated-server workflow if `serve` is deferred
- improve generated OpenAPI and smoke tests
- add TypeScript generation, or explicitly defer it with a documented reason

Success criteria:

- one complete API works end to end
- tests cover generated API behavior
- coding agents can modify the API source, run checks, and repair failures

## Version 0.8: Effects and Permissions Preview

Goal: make risky generated behavior visible.

Work:

- add syntax and design notes for effects
- add `hayulo.toml` permission declarations
- add missing-permission diagnostics
- add deny-list enforcement for generated API actions

Success criteria:

- generated code cannot silently introduce denied effects
- diagnostics explain why permissions are needed
- permission changes are visible in review

## Version 0.9: Public Alpha

Goal: make the project usable by outside testers.

Work:

- add documentation site or publishable docs structure
- add editor syntax support plan or minimal grammar file
- add CI examples
- publish repair benchmark results
- freeze candidate syntax subset

Success criteria:

- outside users can try Hayulo without maintainer help
- feedback can be gathered from real use
- experimental limits are clear

## Version 1.0: Stable Core

Goal: stabilize the language and toolchain contract.

Work:

- freeze core grammar
- freeze project format
- freeze formatter behavior
- freeze diagnostic schema
- document standard library core
- add backwards compatibility policy
- add migration policy for breaking changes

Success criteria:

- 1.0 users can depend on stable core behavior
- breaking changes have a documented migration path
- the standard library, diagnostics, and formatter are documented as public contracts

## Priority Rules

When choosing work, prioritize:

1. queue order in GitHub Issues
2. features needed by examples
3. diagnostics that improve repair loops
4. tests that prevent regressions
5. documentation that helps contributors
6. architecture that enables the next milestone

Avoid:

- adding advanced features without examples
- rewriting the compiler too early
- creating packages before the core is stable
- hiding limitations
