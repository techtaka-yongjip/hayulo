# Detailed Roadmap

This roadmap expands the shorter `ROADMAP.md` file. It is a planning document, not a guarantee.

## Version 0.1: Seed prototype

Status: implemented in the initial repository.

Goals:

- lexer
- parser
- AST
- interpreter
- CLI
- single-file run command
- single-file test command
- basic JSON diagnostics
- examples
- repository docs

Success criteria:

- `hayulo run examples/hello.hayulo` works
- `hayulo test examples/hello.hayulo` works
- `hayulo check examples/hello.hayulo --json` works
- contributors can understand the codebase

## Version 0.2: Useful scripting core

Goals:

- list literals
- map literals
- indexing
- `for` loops
- `match`
- comments and doc comments
- better string escapes
- basic records

Tooling:

- `hayulo --version`
- improved parser diagnostics
- diagnostic snapshot tests
- more examples

Standard library:

- text helpers
- list helpers
- file read/write
- CLI args
- JSON parse/stringify

Example apps:

- word counter
- JSON formatter
- file organizer

## Version 0.3: Types and diagnostics

Goals:

- name resolution
- type inference for locals
- explicit public function types
- function call checks
- return type checks
- field checks for records
- Option and Result basics
- `?` operator

Diagnostics:

- stable diagnostic code namespace
- JSON diagnostic schema v0.1
- safe suggestions
- source spans
- related symbols

Success criteria:

- common LLM-generated mistakes produce useful errors
- type errors can be repaired from JSON diagnostics

## Version 0.4: Projects

Goals:

- `hayulo.toml`
- `src/` and `tests/` conventions
- imports
- modules across files
- project-wide test discovery
- project-wide check command
- lockfile design draft

Commands:

```bash
hayulo new <name>
hayulo init
hayulo test
hayulo check
```

Success criteria:

- a user can create a project and run all tests
- examples can be organized as real projects

## Version 0.5: Formatter and repair protocol

Goals:

- `hayulo format`
- `hayulo format --check`
- repair-hint diagnostics
- codebase summary JSON
- failing test JSON schema
- first repair protocol draft

Commands:

```bash
hayulo summarize --json
hayulo check --json --repair-hints
```

Success criteria:

- LLMs can repair a benchmark set using structured diagnostics
- generated patches produce stable formatted code

## Version 0.6: App-building preview

Goals:

- HTTP server preview
- routing preview
- SQLite preview
- validation
- configuration
- logging

Examples:

- todo API
- notes app
- small dashboard concept
- local invoice tracker concept

Success criteria:

- one complete small app works end to end
- tests cover the app
- app can be generated from a prompt with LLM help

## Version 0.7: Effects and permissions preview

Goals:

- effect annotations
- project permission declarations
- missing permission diagnostics
- deny list enforcement
- approval gate design draft

Example:

```hayulo
fn send_email(...) -> Result<SendReceipt, EmailError>
  effects [email.send, network.write]
{
  ...
}
```

Success criteria:

- compiler can explain why a permission is needed
- generated code cannot silently introduce denied effects

## Version 0.8: Package system preview

Goals:

- local packages
- dependency declaration
- lockfile
- package metadata
- package docs
- package effect summaries

Commands:

```bash
hayulo add <package>
hayulo package check
```

Success criteria:

- examples can share reusable packages
- dependency effects are visible

## Version 0.9: Public alpha

Requirements:

- stable syntax subset
- clear migration notes
- language server preview
- playground or hosted demo
- documentation site
- CI integration
- benchmark results
- security warning docs

Success criteria:

- outside users can try Hayulo without maintainer help
- feedback can be gathered from real use

## Version 1.0: Stable core

Requirements:

- stable grammar for core language
- stable diagnostic schema
- stable project format
- stable formatter
- stable test runner
- static checker for core types
- documented standard library core
- backwards compatibility policy
- migration tooling for breaking changes

1.0 does not need every future feature. It does need a trustworthy core.

## Priority rules

When choosing work, prioritize:

1. features needed by examples
2. diagnostics that improve repair loops
3. tests that prevent regressions
4. documentation that helps contributors
5. architecture that enables the next milestone

Avoid:

- adding advanced features without examples
- rewriting the compiler too early
- creating packages before the core is stable
- hiding limitations
