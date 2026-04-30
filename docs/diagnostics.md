# Diagnostics

Diagnostics are one of Hayulo's most important product features.

A normal compiler error tells a human what went wrong. A Hayulo diagnostic should tell both a human and a repair tool what went wrong, where it happened, why it matters, what safe fixes are possible, and what should be checked next.

## Why diagnostics matter

Hayulo is designed for a loop:

```text
LLM writes code
  -> Hayulo checks it
  -> Hayulo returns structured diagnostics
  -> LLM or human repairs it
  -> tests run again
```

If diagnostics are vague, the loop fails. If diagnostics are precise and stable, they become an API for repair.

## Human and machine audiences

Hayulo should support two diagnostic modes:

### Human mode

Readable terminal output:

```text
error type.mismatch: Expected Int but found Text.
  --> src/main.hayulo:12:18
   |
12 |   total = add(price, "5")
   |                  ^^^ expected Int
help: Convert the text with Int.parse(value)? if it is numeric.
```

### JSON mode

The current 0.2 prototype emits a compact failure shape:

```json
{
  "status": "failed",
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

The stable diagnostic schema is planned separately. A future structured output should look closer to:

```json
{
  "version": "hayulo.diagnostics@0.1",
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
        "end_column": 21
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

## Diagnostic fields

A mature diagnostic should include:

- `code`: stable machine-readable identifier
- `severity`: error, warning, note, info
- `message`: short human-readable summary
- `explanation`: longer explanation
- `location`: file, line, column, and span
- `related`: related symbols or source locations
- `details`: structured values such as expected and actual types
- `suggestions`: safe possible fixes
- `patch_hints`: optional candidate edit information
- `next_checks`: commands to run after repair

## Diagnostic code namespace

Suggested namespaces:

```text
syntax.expected_token
syntax.expected_expression
syntax.unclosed_block
name.unknown_symbol
name.duplicate_definition
type.mismatch
type.unknown_type
type.option_not_handled
type.result_not_handled
effect.permission_missing
effect.denied
test.expectation_failed
project.missing_config
package.version_conflict
```

Diagnostic codes should be treated as public API. Once external tools depend on a code, it should not be renamed casually.

## Suggestions must be safe

A suggestion should not merely be possible. It should be safe enough to recommend.

Bad suggestion:

```text
Change the parameter type to Any.
```

Better suggestion:

```text
Convert the value to Int with Int.parse(value)? and handle the parse error.
```

The compiler should avoid suggestions that hide the problem, weaken the type system, remove tests, or expand permissions without explanation.

## Repair-oriented diagnostics

Future diagnostics can include repair hints:

```json
{
  "kind": "rename_field",
  "from": "emial",
  "to": "email",
  "confidence": 0.97,
  "patch_hint": {
    "file": "src/users.hayulo",
    "span": { "line": 18, "column": 12, "end_column": 17 },
    "replacement": "email"
  }
}
```

Patch hints should be optional. The core diagnostic should still be useful without them.

## Test diagnostics

Test failures should also be structured:

```json
{
  "version": "hayulo.test@0.1",
  "status": "failed",
  "summary": {
    "passed": 5,
    "failed": 1
  },
  "failures": [
    {
      "test": "invoice total includes tax",
      "file": "tests/invoice_test.hayulo",
      "line": 14,
      "expected": "Money.usd(110)",
      "actual": "Money.usd(100)",
      "message": "Expected invoice total to include tax."
    }
  ]
}
```

A repair agent should not need to parse terminal prose to understand failing tests.

## Diagnostic quality checklist

Before adding or changing a diagnostic, ask:

1. Does it point to the smallest useful source span?
2. Is the code stable and specific?
3. Can a beginner understand the message?
4. Can a repair tool use the JSON fields without parsing prose?
5. Are suggestions safe?
6. Does it mention the next useful command?
7. Is there a test for this diagnostic?

## Current prototype

The current seed prototype has basic JSON output. It does not yet have the full diagnostic schema described here.

The roadmap is to evolve current diagnostics into a stable repair protocol over time.
