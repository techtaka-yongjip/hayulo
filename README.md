# Hayulo

Hayulo is an experimental open-source programming language and toolchain for AI-assisted software creation.

The core idea is simple:

> Humans express intent. Coding agents generate and repair code. Hayulo checks, builds, tests, and helps turn the result into useful software.

Hayulo is pre-alpha. It is not production-ready. The current implementation is a Python-based prototype with two tracks:

1. A tiny script interpreter for early language experiments.
2. A new REST API MVP path that checks a `.hayulo` API source file and generates a runnable Node.js REST server.

The REST API path is now the main product direction: make Hayulo the easiest language for tools like Codex, Claude Code, and other coding agents to use when producing useful backend software.

## Status

Hayulo currently supports:

- `module` declarations
- `intent` blocks as source metadata
- script functions, variables, conditionals, `return`, `test`, and `expect`
- JSON diagnostics from the CLI
- API `app` blocks
- API `type ... = record` declarations
- API `database sqlite "..."` declarations
- API `openapi` metadata
- REST `route` declarations for `GET`, `POST`, `PATCH`, and `DELETE`
- field constraints such as `min`, `max`, `unique`, and `private`
- `hayulo build` generation of a runnable REST API server
- generated OpenAPI JSON
- generated smoke tests

The generated REST API uses Node.js built-ins and a local JSON file store for the MVP, so it can run without external runtime dependencies. Future versions can target TypeScript, Hono/Fastify, real SQLite migrations, auth adapters, and deployment targets.

## Script example

```hayulo
module hello

intent {
  purpose: "Show the smallest useful Hayulo program."
}

fn greet(name: Text) -> Text {
  return "Hello, " + name
}

fn main() {
  print(greet("human"))
}

test "greet returns a friendly message" {
  expect greet("Mina") == "Hello, Mina"
}
```

## REST API example

```hayulo
module todo_api

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

  route PATCH "/todos/{id}/done" -> Todo {
    todo = db.Todo.get(id)?
    return db.Todo.update(todo with { done: true })
  }

  route DELETE "/todos/{id}" -> Status {
    db.Todo.delete(id)?
    return no_content
  }
}

type CreateTodo = record {
  title: Text min 1 max 200
}
```

Build it:

```bash
PYTHONPATH=src python -m hayulo check examples/todo_api/main.hayulo --json
PYTHONPATH=src python -m hayulo build examples/todo_api/main.hayulo
cd examples/todo_api/generated
npm test
npm start
```

Then use:

```bash
curl http://localhost:3000/health
curl http://localhost:3000/openapi.json
curl http://localhost:3000/todos
```

## Quick start

```bash
python -m venv .venv
. .venv/bin/activate
pip install -e .
hayulo --version
hayulo run examples/hello.hayulo
hayulo test examples/hello.hayulo
hayulo check examples/hello.hayulo --json
hayulo check examples/todo_api/main.hayulo --json
hayulo build examples/todo_api/main.hayulo
```

Without installation:

```bash
PYTHONPATH=src python -m hayulo run examples/hello.hayulo
PYTHONPATH=src python -m hayulo test examples/hello.hayulo --json
PYTHONPATH=src python -m hayulo build examples/todo_api/main.hayulo
```

## Commands

```bash
hayulo --version
hayulo run <file.hayulo>
hayulo test <file.hayulo>
hayulo check <file.hayulo>
hayulo check <file.hayulo> --json
hayulo build <api-file.hayulo>
hayulo build <api-file.hayulo> --out generated --json
```

## Why Hayulo?

Most programming languages were designed primarily for humans writing code by hand. Hayulo is designed around a newer loop:

```text
human intent
  -> coding agent writes Hayulo
  -> Hayulo returns structured diagnostics
  -> agent repairs source
  -> Hayulo generates app code
  -> tests run
  -> human reviews and ships
```

The first wedge is REST APIs because they have clear structure: records, routes, validation, database storage, tests, and OpenAPI documentation. Hayulo should make those concepts explicit so an AI coding tool has fewer hidden framework decisions to make.

## Repository layout

```text
hayulo-lang/
  src/hayulo/                prototype interpreter, API builder, and CLI
  examples/hello.hayulo      script example
  examples/todo_api/         REST API example
  tests/                     Python tests for the prototype
  docs/                      design and planning documents
  SPEC.md                    seed language specification
  ROADMAP.md                 high-level roadmap
  PROJECT_CHARTER.md         mission, values, and scope
```

## Project documents

Start here:

- [Project Charter](PROJECT_CHARTER.md)
- [Vision](docs/vision.md)
- [Philosophy](docs/philosophy.md)
- [Product Strategy](docs/product_strategy.md)
- [Technical Plan](docs/technical_plan.md)
- [REST API MVP](docs/rest_api_mvp.md)
- [AI-Native Design](docs/ai_native_design.md)
- [Safety and Trust](docs/safety_and_trust.md)
- [Detailed Roadmap](docs/detailed_roadmap.md)
- [Whitepaper](docs/whitepaper.md)
- [Docs Index](docs/INDEX.md)

## Good first contributions

Early useful contributions include:

- improve API parser diagnostics
- add source spans to API records and route clauses
- generate TypeScript alongside `server.mjs`
- generate real SQLite migrations
- add auth primitives
- add `hayulo new api`
- add formatter support for API files
- expand JSON diagnostic snapshots
- write tutorials for Codex, Claude Code, and other coding agents
- test LLM repair loops using `hayulo check --json`

## License

MIT. See [LICENSE](LICENSE).
