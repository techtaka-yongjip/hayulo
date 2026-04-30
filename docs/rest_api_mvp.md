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

## New API project flow

`hayulo new api <project-dir>` creates a small todo API project:

```text
todo-service/
  hayulo.toml
  src/main.hayulo
```

Run the generated project:

```bash
hayulo new api todo-service
cd todo-service
hayulo check
hayulo build src/main.hayulo
cd src/generated
npm test
npm start
```

This is the current app-building workflow. `hayulo serve` is deferred; the supported serve path is the generated Node server through `npm start` inside the generated directory. This keeps the Hayulo CLI focused on checking and generation while the runtime target is still moving.

API projects include a permission policy:

```toml
[permissions]
allow = ["api.read", "api.write", "api.delete", "storage.local"]
deny = []
```

`hayulo check` and `hayulo build` fail before generation when a route requires a permission that is missing or denied.

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

## Current limitations

The MVP intentionally supports a narrow subset:

- It parses API structure, not arbitrary route logic.
- It supports declared CRUD route actions: list, get, create, update, and delete.
- It uses a JSON file store instead of real SQLite migrations.
- It generates JavaScript (`server.mjs`) rather than TypeScript for the first runnable prototype.
- Auth syntax is not implemented in the generated server yet.

These limits are acceptable for the first proof. They keep the system small enough to test and improve.

## OpenAPI and smoke tests

Generated OpenAPI now includes:

- `/health` and `/openapi.json`
- success status codes that match runtime behavior, including `201` for create and `204` for delete
- path parameters for routes such as `/todos/{id}`
- request bodies for body-bearing routes
- shared `ErrorResponse` schema for validation and not-found responses

Generated smoke tests start the server on an ephemeral local port and check health, OpenAPI serving, list, create, validation failure, optional fetch-by-id, mark done, delete, and not-found behavior when the matching routes exist.

## TypeScript decision

TypeScript generation is explicitly deferred. The current generator keeps `server.mjs` dependency-free so the API proof can run with only Node.js built-ins. TypeScript should be added after the API IR and OpenAPI output settle enough that generated types can be treated as a stable public artifact.

## Next improvements

1. Generate TypeScript types after the API IR and OpenAPI shape are more stable.
2. Generate real SQLite schema and migrations.
3. Add auth primitives.
4. Add `hayulo serve` once generated-server lifecycle and watch behavior are defined.
5. Add route action diagnostics for custom actions after CRUD stabilizes.
6. Add AI-focused suggested fixes in diagnostics.
7. Add more examples: notes, inventory, appointments, invoices.

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
