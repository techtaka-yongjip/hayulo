# Hayulo REST API MVP

Hayulo's first practical target is REST API generation for AI-assisted development.

The goal is not to compete with Python, Java, or TypeScript as a general-purpose language on day one. The goal is narrower and more useful:

> Make coding agents excellent at producing working REST APIs from human intent.

Tools like Codex, Claude Code, and other coding agents already know how to edit files and run commands. Hayulo gives them a smaller and more predictable surface area:

```text
records
routes
validation
storage
OpenAPI
tests
structured diagnostics
```

## Current MVP

The current `hayulo build` command accepts an API-style `.hayulo` file and generates:

```text
generated/
  hayulo.ir.json
  openapi.json
  package.json
  server.mjs
  smoke_test.mjs
  README.md
```

The generated server uses Node.js built-ins and a local JSON file store. This keeps the MVP dependency-free while proving the language workflow.

## Example command flow

```bash
PYTHONPATH=src python -m hayulo check examples/todo_api/main.hayulo --json
PYTHONPATH=src python -m hayulo build examples/todo_api/main.hayulo
cd examples/todo_api/generated
npm test
npm start
```

## Why REST APIs first?

REST APIs are a strong first target because they have a predictable shape:

- records model data
- fields define validation constraints
- routes define behavior
- request and response types define contracts
- OpenAPI documents the interface
- tests can exercise the generated server

This structure is exactly what AI coding tools need. A coding agent should not need to guess which router, validator, ORM, test runner, and OpenAPI generator to wire together for every small app.

## MVP syntax

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

## Current limitations

The MVP intentionally supports a narrow subset:

- It parses API structure, not arbitrary route logic.
- It infers common route actions such as list, create, mark done, and delete.
- It uses a JSON file store instead of real SQLite migrations.
- It generates JavaScript (`server.mjs`) rather than TypeScript for the first runnable prototype.
- Auth syntax is parsed but not enforced by the generated server yet.

These limits are acceptable for the first proof. They keep the system small enough to test and improve.

## Next improvements

1. Generate TypeScript types and runtime validation.
2. Generate real SQLite schema and migrations.
3. Add auth primitives.
4. Add `hayulo new api` project scaffolding.
5. Add route action diagnostics when Hayulo cannot infer behavior.
6. Add AI-focused suggested fixes in diagnostics.
7. Add formatter support for API files.
8. Add more examples: notes, inventory, appointments, invoices.

## Definition of success

Hayulo's REST API MVP is successful when a human can say:

> Build a todo API with validation and tests.

And a coding agent can create or modify a Hayulo source file, run:

```bash
hayulo check --json
hayulo build
npm test
```

Then produce a working API with fewer framework mistakes than hand-generating a normal backend stack.
