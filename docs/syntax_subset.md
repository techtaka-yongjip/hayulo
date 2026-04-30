# Hayulo 2.0 Draft Syntax Subset

This is the active `2.0.0a0` draft syntax subset. It is intentionally breaking from the historical 1.0 stable core and is optimized for reliable LLM generation, compiler checking, and repair loops.

See [stable_contract_1_0.md](stable_contract_1_0.md) for the historical 1.0 contract.

## Source Structure

Supported:

```hayulo
module app.main

intent {
  purpose: "Explain why this file exists."
  constraints: [
    "Keep generated behavior explicit."
  ]
}
```

Current rules:

- `module` is parsed and reported but modules are not linked.
- top-level `intent` supports string values and lists of strings.
- comments use `//`.

## Script Subset

Supported declarations:

```hayulo
fn greet(name: Text) -> Text {
  return "Hello, " + name
}

test "greet works" {
  expect greet("Ada") == "Hello, Ada"
}
```

Supported statements:

- `let name = value` for new bindings
- `set name = value` for reassignment
- `return`
- expression statements
- `if` / `else`
- `for name in value`
- statement-form `match`
- `expect` inside tests

Supported values:

- `Int`
- `Float`
- `Text`
- `Bool`
- `List<T>`
- `Map`
- basic record values
- `Option<T>`
- `Result<T, E>`

Supported expressions:

- arithmetic: `+`, `-`, `*`, `/`, `%`
- comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
- boolean: `and`, `or`, `not`
- function calls
- list literals
- map literals
- indexing
- field access
- record literals
- `Some(value)`, `None`, `Ok(value)`, `Err(value)`
- prefix `try expr`

Example:

```hayulo
fn total(scores: List<Int>) -> Int {
  let sum = 0
  for score in scores {
    set sum = sum + score
  }
  return sum
}
```

## Option and Result

Supported:

```hayulo
fn find_user(id: Int) -> Option<User> {
  if id == 1 {
    return Some(User { name: "Ada" })
  }
  return None
}

fn user_name(id: Int) -> Result<Text, Text> {
  let user = try find_user(id)
  return Ok(user.name)
}

fn show(result: Result<Text, Text>) {
  match result {
    Ok(value) => {
      print(value)
    }
    Err(error) => {
      print(error)
    }
  }
}
```

Current limits:

- `match` is statement-only.
- `Option` matches must cover `Some` and `None`.
- `Result` matches must cover `Ok` and `Err`.
- postfix `?` is rejected; use `try expr`.

## API Subset

Supported API declarations:

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

  route POST "/todos" body input: CreateTodo -> Todo {
    effect api.write
    effect storage.local
    action create Todo from input
  }
}
```

Supported API pieces:

- `app Name { ... }`
- `database sqlite "file.db"`
- `openapi { title: "..." version: "..." }`
- `type Name = record { ... }`
- `route GET|POST|PUT|PATCH|DELETE "/path" [body name: Type] -> Type { ... }`
- structured field constraints: `Text { min: 1, max: 200 }`
- route effects: `effect api.read`, `effect api.write`, `effect api.delete`, `effect storage.local`
- route actions: `list`, `get`, `create`, `update`, `delete`
- types: `Text`, `Int`, `Float`, `Bool`, `Time`, `Email`, `Status`, `Id<T>`, `List<T>`, and record names

Supported route actions:

```text
action list Record
action get Record by id
action create Record from input
action update Record by id from input
action update Record by id set { field: value }
action delete Record by id
```

## Project Subset

Supported `hayulo.toml`:

```toml
[project]
name = "my-app"
version = "0.1.0"
src = "src"
tests = "tests"
exclude = []

[permissions]
allow = ["api.read", "api.write", "api.delete", "storage.local"]
deny = []
```

Supported project behavior:

- project root discovery from subdirectories
- `src` and `tests` as strings or arrays of strings
- `exclude` as an array of files or directories
- project-wide `hayulo check`
- project-wide `hayulo test`
- project-wide `hayulo format --check`
- API permission checks for declared route effects

## Stable Preview JSON Interfaces

Current stable preview envelopes:

- `hayulo.diagnostics@0.1`
- `hayulo.test@0.1`

The legacy compact `errors`, `passed`, and `failed` fields remain for compatibility.

## Rejected 1.x Syntax

The 2.0 draft rejects old syntax with targeted diagnostics:

- bare `name = value`: use `let name = value` or `set name = value`
- postfix `?`: use `try expr`
- inline constraints such as `Text min 1 max 200`: use `Text { min: 1, max: 200 }`
- imperative API route bodies such as `return db.Todo.insert(...)`: use `effect` plus `action`

## Out of Scope for 2.0 Draft

Not implemented in the active draft:

- imports and module linking
- package manager and dependency resolution
- complete type inference
- user-defined static record declarations for script files
- expression-form `match`
- custom API actions
- jobs and workflows
- function-level effects and full language-level effects enforcement outside generated API routes
- `hayulo serve`
- TypeScript generation
- real SQLite migrations
- auth enforcement in generated API servers
- language server protocol support
