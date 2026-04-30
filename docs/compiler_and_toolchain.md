# Compiler and Toolchain Plan

Hayulo's toolchain is as important as its syntax. The project should be judged by the full development experience: creating a project, writing code, checking it, testing it, repairing it, and eventually building or deploying it.

## Toolchain philosophy

Hayulo should have one official command-line tool:

```bash
hayulo
```

That tool should cover the common workflow:

```bash
hayulo new
hayulo run
hayulo test
hayulo check
hayulo format
hayulo build
hayulo repair
```

The more fragmented the toolchain becomes, the harder it is for humans and LLMs to use reliably.

## Command roadmap

Implemented in seed prototype:

```bash
hayulo --version
hayulo new my-app
hayulo new api todo-service
hayulo run examples/hello.hayulo
hayulo test examples/hello.hayulo
hayulo test --json
hayulo check examples/hello.hayulo --json
hayulo format <path>
hayulo format --check <path>
hayulo summarize --json
hayulo build examples/todo_api/main.hayulo
```

Medium-term commands:

```bash
hayulo init
hayulo add <package>
hayulo doc
hayulo watch
```

Long-term commands:

```bash
hayulo repair
hayulo deploy
hayulo audit
hayulo policy check
hayulo package publish
```

## Compiler stages

The compiler should eventually expose internal stages for debugging and research:

```bash
hayulo dump tokens file.hayulo
hayulo dump ast file.hayulo
hayulo dump hir file.hayulo
hayulo dump types file.hayulo
hayulo dump effects file.hayulo
```

## Diagnostic schema

Diagnostics should be stable and machine-readable.

Suggested shape:

```json
{
  "schema": "hayulo.diagnostics@0.1",
  "status": "failed",
  "diagnostics": [
    {
      "code": "type.mismatch",
      "severity": "error",
      "message": "Expected Int but found Text.",
      "explanation": "The function add expects both arguments to be Int values.",
      "location": {
        "file": "src/main.hayulo",
        "line": 12,
        "column": 18,
        "end_line": 12,
        "end_column": 23
      },
      "details": {
        "expected": "Int",
        "actual": "Text"
      },
      "suggestions": [
        {
          "kind": "convert_value",
          "message": "Convert the Text to Int with Int.parse(value)? if the input is numeric.",
          "safety": "requires_error_handling"
        }
      ],
      "next_checks": ["hayulo test"]
    }
  ]
}
```

## Diagnostic principles

- Codes must be stable.
- Messages should be short.
- Explanations can be longer.
- Suggestions should be safe, not merely possible.
- Locations should be precise.
- JSON mode should not require parsing prose.
- Human mode should be friendly.
- Diagnostics should include enough context for repair tools.

## Repair protocol

The repair protocol is the heart of the AI-native toolchain.

Future command:

```bash
hayulo check --json --repair-hints
```

Later:

```bash
hayulo repair --agent <adapter>
```

The first version of repair should not try to be magical. It can provide structured information to an external LLM.

A repair session should include:

- project metadata
- current command
- diagnostics
- relevant source snippets
- intent blocks
- failing tests
- allowed edit scope
- forbidden permission changes
- suggested next commands

A repair tool should not automatically:

- expand permissions
- delete tests
- remove intent constraints
- ignore failing tests
- edit unrelated files
- add dependencies without explanation
- hide generated-code changes

## Formatter

`hayulo format` is deterministic and intentionally conservative in the current prototype.

Goals:

- reduce diff noise
- normalize LLM-generated code
- avoid style arguments
- make examples consistent

Current rules are specified in `SPEC.md`: two-space indentation, block-aware indentation for braces and multi-line lists, no trailing whitespace, no trailing blank lines, and one final newline.

CI should run:

```bash
hayulo format --check .
```

## Test runner

Tests are central to the Hayulo workflow.

The test runner should support:

- single-file tests
- project-wide test discovery
- test filtering
- structured JSON output
- source spans for failures
- timing information
- snapshot tests eventually

Example output:

```json
{
  "schema": "hayulo.test@0.1",
  "status": "failed",
  "summary": {
    "passed": 12,
    "failed": 1,
    "skipped": 0
  },
  "failures": [
    {
      "test": "invoice total includes tax",
      "file": "tests/invoice_test.hayulo",
      "line": 18,
      "expected": "Money.usd(110)",
      "actual": "Money.usd(100)"
    }
  ]
}
```

## Language server

A Hayulo language server should eventually provide:

- syntax highlighting
- diagnostics
- go to definition
- hover documentation
- rename symbol
- format on save
- code actions
- test discovery
- intent block display
- effect/permission warnings

## Package manager

The package manager should be conservative at first.

Near-term:

- local dependencies
- version pinning
- lockfile
- official registry design

Long-term:

- package signing
- dependency permissions
- package audit
- generated-code metadata
- documentation hosting

## Playground

A web playground would be valuable for adoption.

Features:

- run Hayulo examples in browser
- show tokens/AST/diagnostics
- share snippets
- run tests
- demonstrate repair diagnostics

## CI integration

Hayulo should be easy to run in CI:

```yaml
- run: hayulo format --check
- run: hayulo check --json
- run: hayulo test --json
```

Structured output should integrate with GitHub annotations eventually.
