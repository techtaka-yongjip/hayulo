# Hayulo 2.0 Draft Specification

Hayulo is an experimental programming language for AI-assisted software creation.

This document describes the current `2.0.0a0` draft implemented in this repository. The draft is intentionally breaking from the historical 1.0 line: it prefers explicit bindings, explicit recoverable errors, structured API actions, and diagnostics that coding agents can use during repair loops.

The historical 1.0 contract remains in [docs/stable_contract_1_0.md](docs/stable_contract_1_0.md).

## Design Goals

Hayulo should be:

1. readable by humans
2. easy for LLMs to generate and edit
3. easy for compilers to check
4. predictable across projects
5. friendly to automated repair loops
6. suitable for real app-building as the ecosystem grows

## Source Files

Hayulo files use the `.hayulo` extension.

A source file may contain:

- a `module` declaration
- an `intent` metadata block
- function declarations
- test declarations
- API `app` declarations
- top-level API record declarations used by route bodies

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
exclude = []

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

## Modules

A module declaration names the file's logical module.

```hayulo
module app.main
```

Modules are parsed and reported, but the current prototype does not link multiple modules.

## Intent Blocks

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

Intent blocks are skipped during execution and exposed by `hayulo check --json` as metadata.

## Functions

Functions use braces and explicit parameter lists.

```hayulo
fn add(a: Int, b: Int) -> Int {
  return a + b
}
```

A file run with `hayulo run` must contain `fn main()`.

## Bindings and Reassignment

New bindings use `let`.

```hayulo
let name = "Ada"
let age = 36
```

Reassignment uses `set`.

```hayulo
let sum = 0
for score in scores {
  set sum = sum + score
}
```

Bare assignment is rejected:

```hayulo
name = "Ada"
```

Use `let name = ...` for a new binding or `set name = ...` for reassignment.

## Values

Supported values:

- `Int`
- `Float`
- `Text`
- `Bool`
- `List<T>`
- `Map`
- record values
- `Option<T>`
- `Result<T, E>`

Examples:

```hayulo
let x = 10
let y = 2.5
let name = "Ada"
let active = true
```

## Lists, Maps, and Indexing

List literals use square brackets.

```hayulo
let scores = [90, 95, 100]
let first = scores[0]
```

Map literals use braces and expression keys.

```hayulo
let labels = {"role": "admin", "team": "language"}
let team = labels["team"]
```

Lists are indexed with `Int` values. Maps are indexed with existing keys.

## Records

Script files support basic record values without static record declarations.

```hayulo
let user = User {
  name: "Ada",
  scores: [90, 95, 100]
}

print(user.name)
print(user.scores[0])
```

Record fields are accessed with `.`.

## Option and Result

Missing values use `Option<T>`.

```hayulo
fn find_user(id: Int) -> Option<User> {
  if id == 1 {
    return Some(User { name: "Ada" })
  }
  return None
}
```

Recoverable errors use `Result<T, E>`.

```hayulo
fn user_name(id: Int) -> Result<Text, Text> {
  let user = try find_user(id)
  return Ok(user.name)
}
```

`Some`, `None`, `Ok`, and `Err` use Pascal-case variants. `try expr` unwraps `Some(value)` or `Ok(value)`. It returns early with `None` or `Err(error)` when the value cannot be unwrapped.

Postfix `?` is rejected in the 2.0 draft. Use prefix `try expr`.

## Match

`match` is statement-only in the current draft.

```hayulo
match result {
  Ok(value) => {
    print(value)
  }
  Err(error) => {
    print(error)
  }
}
```

The checker requires `Option` matches to cover `Some` and `None`, and `Result` matches to cover `Ok` and `Err`. Expression-form `match` is deferred.

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
let message = "Hello, " + name
```

## Control Flow

```hayulo
if score >= 90 {
  return "A"
} else {
  return "B"
}
```

For loops iterate over lists and map keys.

```hayulo
let sum = 0
for score in scores {
  set sum = sum + score
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

## Static Checking Preview

`hayulo check` runs a static checking preview for script files.

The checker currently reports:

- unknown local names and functions
- reassignment before binding
- duplicate local bindings
- wrong function call arity
- basic argument type mismatches from annotations
- local type inference for literals, lists, maps, records, indexing, calls, `Option`, `Result`, and `try`
- return values that do not match explicit return annotations
- invalid `try` targets
- non-exhaustive `match` for `Option` and `Result`
- invalid record field access when the record value is locally known
- invalid indexing and invalid `for` loop targets

This is not a complete type system. The checker is intentionally local and conservative.

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

Use `hayulo format --check <file-or-project>` in repair loops and CI.

## Diagnostics

Hayulo diagnostics are designed to be useful to both humans and LLMs.

Text output is human-readable. JSON output is machine-readable:

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

Failing `hayulo test --json` output uses `hayulo.test@0.1`.

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

Examples of stable diagnostic code namespaces include:

```text
syntax.binding_requires_let_or_set
syntax.postfix_try_removed
name.unknown_symbol
name.duplicate_definition
name.reassignment_before_binding
call.arity_mismatch
type.argument_mismatch
type.return_mismatch
type.invalid_try_target
type.try_return_mismatch
match.non_exhaustive
record.unknown_field
route.body_requires_action
api.inline_constraints_removed
permission.missing
permission.denied
```

---

# API Draft Addendum

Hayulo's first practical compiler target is REST API generation.

This API subset is intentionally narrow and exists to make AI coding tools better at producing useful backend software.

## API App Block

```hayulo
app TodoApi {
  database sqlite "todo.db"

  openapi {
    title: "Todo API"
    version: "0.1.0"
  }

  type Todo = record {
    id: Id<Todo>
    title: Text { min: 1, max: 200 }
    done: Bool = false
    created_at: Time = now()
  }

  route GET "/todos" -> List<Todo> {
    effect api.read
    effect storage.local
    action list Todo
  }
}
```

## Supported API Declarations

```text
app Name { ... }
database sqlite "file.db"
openapi { title: "..." version: "..." }
type Name = record { field: Type { constraints } }
route METHOD "/path" [body name: Type] -> Type { effect ... action ... }
```

## Field Constraints

Field constraints use structured attribute blocks.

```hayulo
type Todo = record {
  title: Text { min: 1, max: 200 }
  sku: Text { unique: true }
  internal_notes: Text { private: true } = ""
}
```

Inline constraints such as `Text min 1 max 200` are rejected.

## Route Effects and Actions

Every route body contains zero or more `effect` lines and exactly one `action`.

```hayulo
route POST "/todos" body input: CreateTodo -> Todo {
  effect api.write
  effect storage.local
  action create Todo from input
}
```

Supported CRUD actions:

```text
action list Record
action get Record by id
action create Record from input
action update Record by id from input
action update Record by id set { field: value }
action delete Record by id
```

Imperative route bodies such as `return db.Todo.insert(...)` are rejected.

## Effects and Permissions

Route effects use lowercase dotted names.

```hayulo
effect api.read
effect storage.local
```

Project permissions use the same names:

```toml
[permissions]
allow = ["api.read", "api.write", "api.delete", "storage.local"]
deny = []
```

`hayulo check` and `hayulo build` fail when a declared route effect is missing from `allow` or appears in `deny`.

## Supported API Types

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

## Build Output

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

## API Project Scaffold

`hayulo new api <project-dir>` creates a Hayulo project with `hayulo.toml` and `src/main.hayulo` containing a small todo API. The generated project is expected to work with:

```bash
hayulo check
hayulo build src/main.hayulo
cd src/generated
npm test
npm start
```

`hayulo serve` is not implemented. The supported serve path is the generated Node server. TypeScript generation is deferred until the API IR and OpenAPI output are stable enough to make generated types a public artifact.
