# Hayulo 1.0 Syntax Subset

This is the Hayulo 1.0 stable syntax subset. Testers can rely on this document for the supported 1.x core. Future releases may add syntax, but existing documented syntax should remain compatible according to [compatibility.md](compatibility.md).

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

- assignment with `=`
- `return`
- expression statements
- `if` / `else`
- `for name in value`
- `expect` inside tests

Supported values:

- `Int`
- `Float`
- `Text`
- `Bool`
- `List`
- `Map`
- basic record values

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
    title: Text min 1 max 200
    done: Bool = false
    created_at: Time = now()
  }

  route GET "/todos" -> List<Todo> {
    return db.Todo.all(order: created_at desc)
  }

  route POST "/todos" body input: CreateTodo -> Todo {
    return db.Todo.insert(Todo { title: input.title })
  }
}
```

Supported API pieces:

- `app Name { ... }`
- `database sqlite "file.db"`
- `openapi { title: "..." version: "..." }`
- `type Name = record { ... }`
- `route GET|POST|PUT|PATCH|DELETE "/path" [body name: Type] -> Type { ... }`
- field constraints: `min`, `max`, `unique`, `private`
- types: `Text`, `Int`, `Float`, `Bool`, `Time`, `Email`, `Status`, `Id<T>`, `List<T>`, and record names

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
- API permission checks for generated behavior

## Stable Preview JSON Interfaces

Current stable preview envelopes:

- `hayulo.diagnostics@0.1`
- `hayulo.test@0.1`

The legacy compact `errors`, `passed`, and `failed` fields remain for compatibility.

## Out of Scope for 1.0

Not implemented in the public-alpha subset:

- imports and module linking
- package manager and dependency resolution
- full type inference
- user-defined static record declarations for script files
- `match`
- `Option` and `Result`
- language-level effects enforcement
- `hayulo serve`
- TypeScript generation
- real SQLite migrations
- auth enforcement in generated API servers
- language server protocol support
