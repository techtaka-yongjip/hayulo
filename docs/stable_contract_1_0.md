# Hayulo 1.0 Stable Core Contract

Hayulo 1.0 stabilizes a small, explicit contract for AI-assisted software creation. The stable core is intentionally narrow: scripts, tests, project discovery, formatting, diagnostics, REST API generation, generated smoke tests, and project permissions.

The 1.0 contract means these interfaces should not break within the 1.x line without a documented migration path.

## Stable CLI

Stable commands:

```bash
hayulo --version
hayulo new <project-dir>
hayulo new api <project-dir>
hayulo check [file-or-project] [--json]
hayulo run <file.hayulo> [--json]
hayulo test [file-or-project] [--json]
hayulo format [file-or-project] [--check] [--json]
hayulo summarize [file-or-project] --json
hayulo build <api-file.hayulo> [--out <dir>] [--no-clean] [--json]
```

Stable quality gates:

```bash
make test
make check
make verify
make release-check
```

## Stable Language Subset

Stable script syntax:

- `module`
- top-level `intent`
- `fn`
- `test`
- `expect`
- assignment
- `return`
- `if` / `else`
- `for name in value`
- literals: integer, float, text, bool, list, map, record
- function calls
- indexing
- field access
- arithmetic, comparison, and boolean operators

Stable API syntax:

- `app Name { ... }`
- `database sqlite "file.db"`
- `openapi { title: "..." version: "..." }`
- `type Name = record { ... }`
- `route METHOD "/path" [body name: Type] -> Type { ... }`
- methods: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`
- field constraints: `min`, `max`, `unique`, `private`
- API types: `Text`, `Int`, `Float`, `Bool`, `Time`, `Email`, `Status`, `Id<T>`, `List<T>`, and record names

## Stable Project Format

Stable `hayulo.toml` fields:

```toml
[project]
name = "my-app"
version = "1.0.0"
src = "src"
tests = "tests"
exclude = []

[permissions]
allow = ["api.read", "api.write", "api.delete", "storage.local"]
deny = []
```

Stable behavior:

- project discovery walks up from the current directory or file path
- `src`, `tests`, and `exclude` accept strings or arrays of strings as documented
- generated directories and hidden dependency directories are skipped
- single-file commands continue to work outside a project

## Stable Formatter

Stable formatter behavior:

- two-space indentation
- indentation follows `{ ... }` and multi-line `[ ... ]`
- trailing whitespace is removed
- repeated trailing blank lines are removed
- formatted files end with exactly one newline
- `hayulo format --check` never rewrites files

## Stable JSON Schemas

Stable envelopes:

- `hayulo.diagnostics@0.1`
- `hayulo.test@0.1`

The compatibility aliases `errors`, `passed`, and `failed` remain available through 1.x.

## Stable Generated API Contract

Stable generated files:

```text
hayulo.ir.json
openapi.json
package.json
server.mjs
smoke_test.mjs
README.md
```

Stable generated behavior:

- dependency-free Node.js server
- local JSON file store
- `/health`
- `/openapi.json`
- generated OpenAPI 3.1 document
- generated smoke test executable through `npm test`
- project permission checks before build

## Not Stabilized in 1.0

These remain future work:

- module imports and linking
- packages and dependency resolution
- language-level effects enforcement
- full static type system
- `Option`, `Result`, and `match`
- language server protocol
- TypeScript generation
- real SQLite migrations
- generated auth enforcement
- deployment targets
