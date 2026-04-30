# Hayulo 0.1 Seed Specification

Hayulo is an experimental programming language for AI-assisted software creation.

This document describes the seed version implemented in this repository. It is intentionally small.

## Design goals

Hayulo should be:

1. readable by humans
2. easy for LLMs to generate and edit
3. easy for compilers to check
4. predictable across projects
5. friendly to automated repair loops
6. suitable for real app-building as the ecosystem grows

## Source files

Hayulo files use the `.hayulo` extension.

A source file may contain:

- a `module` declaration
- an `intent` metadata block
- function declarations
- test declarations

Example:

```hayulo
module hello

intent {
  purpose: "Explain why this file exists."
}

fn main() {
  print("Hello")
}
```

## Projects

A Hayulo project has a `hayulo.toml` file at the project root.

```toml
[project]
name = "my-app"
version = "0.1.0"
src = "src"
tests = "tests"

[permissions]
allow = []
deny = []
```

The current project config supports:

- `name`: project name
- `version`: project version
- `src`: source directory path or list of paths
- `tests`: test directory path or list of paths
- `exclude`: optional list of files or directories to skip
- `[permissions].allow`: optional list of allowed generated effects
- `[permissions].deny`: optional list of denied generated effects

`hayulo new <project-dir>` creates:

```text
hayulo.toml
src/main.hayulo
tests/main_test.hayulo
```

`hayulo check` with no file checks the current project. `hayulo test` with no file runs project tests. Passing a `.hayulo` file keeps single-file behavior.

## Effects and permissions preview

Hayulo 0.8 introduces a project-level permission preview for generated API behavior. The current implementation checks REST API source files against `hayulo.toml`; future versions can extend the same model to function-level effect annotations.

Project permissions use lowercase dotted names:

```toml
[permissions]
allow = ["api.read", "api.write", "storage.local"]
deny = ["api.delete"]
```

The API generator currently requires:

- `api.read` for `GET` routes
- `api.write` for `POST`, `PUT`, and `PATCH` routes
- `api.delete` for `DELETE` routes
- `storage.local` because the generated MVP server uses a local JSON file store

If a required permission is missing from `allow`, `hayulo check` and `hayulo build` fail with `permission.missing`. If a required permission appears in `deny`, they fail with `permission.denied`.

Future language-level syntax may look like:

```hayulo
fn export_report(path: Path) -> Result<(), FileError>
  effects [files.write]
{
  return files.write_text(path, "report")
}
```

That syntax is design direction only in 0.8; the enforced surface today is project permissions for generated API actions.

## Comments

Line comments start with `//`.

```hayulo
// This is a comment.
```

## Modules

A module declaration names the file's logical module.

```hayulo
module app.main
```

In the current prototype, modules are parsed but not linked.

## Intent blocks

Intent blocks preserve human purpose and constraints near the implementation.

```hayulo
intent {
  purpose: "Manage todos for authenticated users."
  constraints: [
    "Users can only see their own todos.",
    "Todo titles cannot be empty."
  ]
}
```

In the current prototype, intent blocks are skipped during execution and exposed by `hayulo check --json` as metadata. Future compilers should also expose them to documentation, refactoring tools, and AI repair systems.

## Functions

Functions use braces and explicit parameter lists.

```hayulo
fn add(a: Int, b: Int) -> Int {
  return a + b
}
```

The current prototype accepts type annotations but does not enforce them yet.

A file run with `hayulo run` must contain `fn main()`.

## Variables

Variables are assigned with `=`.

```hayulo
name = "Ada"
age = 36
```

The current prototype uses function-local variables.

## Values

Supported values in the current prototype:

- `Int`
- `Float`
- `Text`
- `Bool`
- `List`
- `Map`
- record values

Examples:

```hayulo
x = 10
y = 2.5
name = "Ada"
active = true
```

## Lists, maps, and indexing

List literals use square brackets.

```hayulo
scores = [90, 95, 100]
first = scores[0]
```

Map literals use braces and expression keys.

```hayulo
labels = {"role": "admin", "team": "language"}
team = labels["team"]
```

Lists are indexed with `Int` values. Maps are indexed with existing keys.

## Records

The current prototype supports basic record values without static record declarations in script files.

```hayulo
user = User {
  name: "Ada",
  scores: [90, 95, 100]
}

print(user.name)
print(user.scores[0])
```

Record fields are accessed with `.`.

## Operators

Arithmetic:

```hayulo
+ - * / %
```

Comparison:

```hayulo
== != < <= > >=
```

Boolean:

```hayulo
and or not
```

String concatenation uses `+`.

```hayulo
message = "Hello, " + name
```

## Control flow

```hayulo
if score >= 90 {
  return "A"
} else {
  return "B"
}
```

For loops iterate over lists and map keys.

```hayulo
sum = 0
for score in scores {
  sum = sum + score
}

for key in labels {
  print(labels[key])
}
```

## Return

```hayulo
return value
```

A function without an explicit return returns `None` in the prototype runtime.

## Built-ins

The prototype includes:

```hayulo
print(value)
len(value)
```

## Static checking preview

`hayulo check` runs a static checking preview for script files.

The checker currently reports:

- unknown local names and functions
- wrong function call arity
- basic argument type mismatches from annotations
- local type inference for literals, lists, maps, records, indexing, and calls
- return values that do not match explicit return annotations
- invalid record field access when the record value is locally known
- invalid indexing and invalid `for` loop targets

This is not a complete type system. The checker is intentionally local and conservative. Unknown or unannotated values are allowed so early Hayulo programs remain easy to write.

## Tests

Tests are part of the language.

```hayulo
test "add works" {
  expect add(2, 3) == 5
}
```

Run them:

```bash
hayulo test examples/hello.hayulo
hayulo test examples/hello.hayulo --json
```

## Formatting

`hayulo format` provides deterministic formatting for supported Hayulo source files.

Current formatting rules:

- two-space indentation
- indentation follows `{ ... }` and multi-line `[ ... ]` blocks
- trailing whitespace is removed
- repeated trailing blank lines are removed
- formatted files end with exactly one newline

Use `hayulo format --check <file-or-project>` in repair loops and CI to fail when a source file is not formatted without rewriting it.

## Diagnostics

Hayulo diagnostics are designed to be useful to both humans and LLMs.

Text output is human-readable.

JSON output is machine-readable:

```json
{
  "schema": "hayulo.diagnostics@0.1",
  "status": "failed",
  "diagnostics": [
    {
      "code": "syntax_error",
      "severity": "error",
      "message": "Expected ')' after arguments.",
      "location": {
        "file": "examples/broken.hayulo",
        "line": 4,
        "column": 15
      },
      "details": {},
      "suggestions": [
        {
          "message": "Check punctuation near this location."
        }
      ]
    }
  ],
  "errors": [
    {
      "code": "syntax_error",
      "message": "Expected ')' after arguments.",
      "file": "examples/broken.hayulo",
      "line": 4,
      "column": 15,
      "suggestions": ["Check punctuation near this location."]
    }
  ]
}
```

The `errors` field is retained as a compact compatibility alias during the prototype. New repair tools should prefer `schema` and `diagnostics`.

Failing `hayulo test --json` output uses `hayulo.test@0.1`:

```json
{
  "schema": "hayulo.test@0.1",
  "status": "failed",
  "summary": {
    "passed": 0,
    "failed": 1
  },
  "failures": [
    {
      "test": "add works",
      "file": "tests/main_test.hayulo",
      "line": 3,
      "message": "Expectation failed."
    }
  ]
}
```

Successful `hayulo check --json` output includes top-level intent metadata when present:

```json
{
  "status": "ok",
  "kind": "script",
  "file": "examples/hello.hayulo",
  "module": "hello",
  "intent": {
    "purpose": "Show the smallest useful Hayulo program.",
    "constraints": [
      "The greeting should be friendly."
    ]
  },
  "functions": ["greet", "main"],
  "tests": ["greet returns a friendly message"]
}
```

The static checker uses namespaced diagnostic codes for new stable checks. Examples include:

```text
name.unknown_symbol
name.duplicate_definition
call.arity_mismatch
type.argument_mismatch
type.return_mismatch
type.invalid_index
type.invalid_index_target
type.not_iterable
record.unknown_field
record.invalid_field_target
```

## Planned but not yet implemented

Hayulo 0.1 seed intentionally leaves these for future work:

- static type checker
- `Option` and `Result`
- `match`
- module imports
- package manager
- full language-level effects
- app framework libraries
- TypeScript or WebAssembly backend

---

# API MVP Addendum

Hayulo's first practical compiler target is REST API generation.

This API subset is intentionally narrow and exists to make AI coding tools better at producing useful backend software.

## API app block

```hayulo
app TodoApi {
  database sqlite "todo.db"

  openapi {
    title: "Todo API"
    version: "0.1.0"
  }

  type Todo = record {
    id: Id<Todo>
    title: Text min 1 max 200
    done: Bool = false
    created_at: Time = now()
  }

  route GET "/todos" -> List<Todo> {
    return db.Todo.all(order: created_at desc)
  }
}
```

## Supported API declarations

```text
app Name { ... }
database sqlite "file.db"
openapi { title: "..." version: "..." }
type Name = record { field: Type constraints }
route METHOD "/path" [body name: Type] -> Type { ... }
```

## Supported field constraints

```text
min <number>
max <number>
unique
private
```

## Supported MVP types

```text
Text
Int
Float
Bool
Time
Email
Status
Id<T>
List<T>
record types
```

## Build output

`hayulo build <api-file.hayulo>` generates:

```text
hayulo.ir.json
openapi.json
package.json
server.mjs
smoke_test.mjs
README.md
```

The MVP generator uses Node.js built-ins and a local JSON file store. This is a proof of the workflow, not the final backend architecture.

## API project scaffold

`hayulo new api <project-dir>` creates a Hayulo project with `hayulo.toml` and `src/main.hayulo` containing a small todo API. The generated project is expected to work with:

```bash
hayulo check
hayulo build src/main.hayulo
cd src/generated
npm test
npm start
```

`hayulo serve` is not part of the 0.7 API preview. The supported serve path is the generated Node server. TypeScript generation is also deferred until the API IR and OpenAPI output are stable enough to make generated types a public artifact.

API projects generated by `hayulo new api` include:

```toml
[permissions]
allow = ["api.read", "api.write", "api.delete", "storage.local"]
deny = []
```

Remove permissions from `allow` or add them to `deny` to make risky generated behavior fail during `hayulo check` and `hayulo build`.
