# LLM Generation Benchmarks

Hayulo should improve by measurement, not preference. The LLM benchmark suite asks whether coding agents can generate, check, test, and repair Hayulo programs more reliably than comparable mainstream stacks.

The first suite is intentionally local and manual. It does not call hosted LLM APIs. It defines tasks, validates Hayulo baselines, and gives humans or agents a repeatable place to record results.

## Commands

Validate the benchmark task catalog:

```bash
hayulo benchmark llm --json
```

Validate the Hayulo baseline sources:

```bash
hayulo check benchmarks/llm/baselines --json
```

Run the local benchmark gate:

```bash
make benchmark
```

`make verify` includes `make benchmark`, so benchmark metadata and baselines are checked with the normal project gate.
The benchmark gate validates task JSON, checks every Hayulo baseline, confirms baseline formatting, builds every generated API baseline, and runs generated smoke tests when Node is available.

## What The Suite Measures

The initial suite compares:

- Hayulo
- Python with FastAPI
- TypeScript with Fastify
- Go with the standard library

The current metrics are:

- `first_pass_check`: generated source passes the stack's compiler/checker without repair
- `first_pass_tests`: generated tests pass without repair
- `repair_iterations`: number of agent repair turns needed before the gate passes
- `generated_api_smoke`: generated REST API smoke test passes
- task-specific quality metrics such as validation, private fields, safety constraints, and schema completeness

## Initial Tasks

| Task | Difficulty | Purpose |
| --- | --- | --- |
| `api.todo_crud` | baseline | Todo CRUD API with validation and smoke-testable routes. |
| `api.inventory` | baseline | Inventory API with numeric constraints and update behavior. |
| `api.reading_list` | baseline | Bookmark API with tags and read state. |
| `api.support_ticket` | intermediate | Ticket API with private fields and status updates. |
| `api.webhook_receiver` | intermediate | Webhook receiver that stores payloads without executing them. |

Each task lives under `benchmarks/llm/tasks/` and points to a Hayulo baseline under `benchmarks/llm/baselines/`.

## Task Format

Task JSON uses schema `hayulo.llm_benchmark.task@0.1`:

```json
{
  "schema": "hayulo.llm_benchmark.task@0.1",
  "id": "api.todo_crud",
  "title": "Todo CRUD REST API",
  "category": "rest-api-generation",
  "difficulty": "baseline",
  "prompt": "Generate a small Todo REST API...",
  "comparison_targets": ["hayulo", "python-fastapi", "typescript-fastify", "go-stdlib"],
  "success_metrics": ["first_pass_check", "first_pass_tests", "repair_iterations"],
  "expected_outputs": ["source file", "OpenAPI document", "smoke tests"],
  "hayulo_baseline": "benchmarks/llm/baselines/todo_crud_api/main.hayulo",
  "manual_checks": [
    "hayulo check benchmarks/llm/baselines/todo_crud_api/main.hayulo --json"
  ]
}
```

The suite summary emitted by `hayulo benchmark llm --json` uses schema `hayulo.llm_benchmark@0.1`.

## Manual Run Loop

For each model and target stack:

1. Give the model the task prompt only.
2. Ask it to produce the target stack implementation, tests, and run instructions.
3. Run the stack's check/test/smoke commands.
4. If it fails, give the model structured error output and count one repair iteration.
5. Stop when the gate passes or the run reaches a fixed repair limit.
6. Record the result under `benchmarks/llm/runs/`.

Example recorded run:

```json
{
  "task_id": "api.todo_crud",
  "target": "hayulo",
  "model": "codex",
  "status": "passed",
  "metrics": {
    "first_pass_check": true,
    "first_pass_tests": true,
    "repair_iterations": 0,
    "generated_api_smoke": true
  }
}
```

## How Results Should Influence Syntax

Syntax changes should be justified by benchmark outcomes. A proposal is stronger when it can show at least one of these:

- fewer repair iterations
- fewer parser/checker failures
- clearer diagnostics for generated mistakes
- less generated boilerplate for the same app behavior
- fewer unsafe or ambiguous generated behaviors

For example, future custom API actions should be measured against the current declarative CRUD action style before becoming part of the default syntax.

See [Syntax Philosophy and Rulebook](syntax_rulebook.md) for the full syntax-change acceptance process.
