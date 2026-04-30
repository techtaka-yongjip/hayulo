# Public Alpha Guide

Hayulo public alpha is for outside testers who want to try the current AI-friendly language and REST API generator, understand the limits, and report useful issues.

Hayulo is still pre-1.0. The alpha goal is not production use. The goal is to make the loop repeatable:

```text
write or generate Hayulo
run hayulo format --check
run hayulo check --json
run hayulo test
run hayulo build for API sources
run generated smoke tests
repair the source from structured diagnostics
```

## Install

From a checkout:

```bash
python -m venv .venv
. .venv/bin/activate
pip install -e .
hayulo --version
```

Without installation:

```bash
PYTHONPATH=src python -m hayulo --version
```

## Try the Script Path

```bash
hayulo run examples/hello.hayulo
hayulo test examples/hello.hayulo
hayulo run examples/data_core.hayulo
hayulo test examples/data_core.hayulo
hayulo check examples/hello.hayulo --json
```

## Try the REST API Path

```bash
hayulo check examples/todo_api/main.hayulo --json
hayulo build examples/todo_api/main.hayulo
cd examples/todo_api/generated
npm test
npm start
```

In another terminal:

```bash
curl http://localhost:3000/health
curl http://localhost:3000/openapi.json
curl http://localhost:3000/todos
```

## Create a New API Project

```bash
hayulo new api todo-service
cd todo-service
hayulo check
hayulo build src/main.hayulo
cd src/generated
npm test
npm start
```

## What Is Stable Enough to Test

- CLI commands documented in [README.md](../README.md)
- candidate language subset in [syntax_subset.md](syntax_subset.md)
- JSON diagnostic envelope `hayulo.diagnostics@0.1`
- JSON test envelope `hayulo.test@0.1`
- `hayulo.toml` project discovery
- REST API generation to dependency-free Node.js
- generated OpenAPI and smoke tests
- project permissions for generated API behavior

## Known Limits

- Hayulo is not production-ready.
- Module imports and packages are not implemented.
- Static checking is intentionally local and conservative.
- REST API route bodies are parsed as structure; arbitrary route logic is not compiled.
- The generated API uses a local JSON file store, not real SQLite migrations.
- `hayulo serve` and TypeScript generation are deferred.
- Editor support is a minimal grammar preview, not a language server.
- Language-level effects syntax is documented as direction, but only project API permissions are enforced today.

## Useful Feedback

Good alpha feedback includes:

- a `.hayulo` source file
- the exact command that failed
- JSON output from `hayulo check --json` or `hayulo test --json`
- whether the behavior was from the script path or API path
- whether a coding agent was able to repair the problem

## Public Alpha Docs Map

- [Syntax subset](syntax_subset.md)
- [Repair benchmark results](repair_benchmarks.md)
- [CI examples](ci.md)
- [Editor support](editor_support.md)
- [REST API MVP](rest_api_mvp.md)
- [Diagnostics](diagnostics.md)
- [Safety and Trust](safety_and_trust.md)
