# Hayulo Roadmap

This roadmap is a planning document, not a guarantee. Hayulo is pre-alpha and will change as the project learns.

## 0.1 Seed Prototype

Status: implemented.

Goals:

- tiny runnable language
- lexer, parser, AST, interpreter
- CLI with `run`, `test`, and `check`
- JSON diagnostics
- examples and tests
- founding documentation

## 0.2 Useful Scripting Core

Goals:

- list literals
- map literals
- indexing
- comments and doc comments
- `for` loops
- `match`
- better strings
- basic records
- `hayulo --version`
- stronger parser recovery

Example target:

```hayulo
fn main() {
  names = ["Ada", "Grace", "Linus"]

  for name in names {
    print(name)
  }
}
```

## 0.3 Static Checking Preview

Goals:

- name resolution
- local type inference
- function call arity checks
- return type checks
- record field checks
- `Option<T>` basics
- `Result<T, E>` basics
- `?` operator
- stable diagnostic code namespace

## 0.4 Project System

Goals:

- `hayulo.toml`
- `hayulo new`
- multi-file modules
- `src/` and `tests/` conventions
- project-wide `hayulo check`
- project-wide `hayulo test`

## 0.5 Formatter and Repair Protocol

Goals:

- `hayulo format`
- JSON diagnostics v0.1
- repair hints
- failing test JSON schema
- codebase summary command
- first AI repair protocol draft

## 0.6 App-Building Preview

Goals:

- files library
- JSON library
- CLI library
- HTTP preview
- SQLite preview
- validation preview
- one complete small app example

## 0.7 Effects and Permissions Preview

Goals:

- effect annotations
- permission declarations in `hayulo.toml`
- missing permission diagnostics
- deny list enforcement
- approval gate design draft

## 0.8 Package System Preview

Goals:

- local packages
- dependency metadata
- lockfile design
- package docs
- package effect summaries

## 0.9 Public Alpha

Goals:

- stable enough syntax subset
- documentation site
- editor syntax support
- playground or hosted demo
- CI workflow examples
- repair benchmark results

## 1.0 Stable Core

Goals:

- stable core grammar
- stable project format
- stable formatter
- stable diagnostic schema
- stable test runner
- documented standard library core
- backwards compatibility policy
- migration story for breaking changes

## Roadmap principle

Every release should make Hayulo better at one or more of these:

- build useful programs
- detect errors earlier
- explain errors better
- help LLMs repair code
- preserve human intent
- make generated code safer

## Immediate pivot: REST API MVP

Hayulo's next milestone is a coding-agent-friendly REST API language. The project should prioritize:

1. Stronger API parser diagnostics.
2. `hayulo build` for API files.
3. OpenAPI generation.
4. Generated smoke tests.
5. TypeScript generation.
6. SQLite migrations.
7. Auth primitives.
8. `hayulo new api` scaffolding.

The first success metric is simple: one `.hayulo` source file should become a working REST API that can pass generated tests.
