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
12 |   let total = add(price, "5")
   |                  ^^^ expected Int
help: Convert the text with Int.parse(value) and handle the Result with try.
```

### JSON mode

The current prototype emits a stable v0.1 diagnostic envelope:

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

The `errors` field is a compact compatibility alias for older scripts. Repair tools should use `schema` and `diagnostics`.

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

Hayulo 2.0.0a0 uses namespaced static checker diagnostics as the stable preview namespace for script checking:

```text
name.unknown_symbol
name.duplicate_definition
name.reassignment_before_binding
call.arity_mismatch
type.argument_mismatch
type.return_mismatch
type.operator_mismatch
type.invalid_try_target
type.try_return_mismatch
type.invalid_index
type.invalid_index_target
type.not_iterable
match.non_exhaustive
record.unknown_field
record.invalid_field_target
permission.missing
permission.denied
project.invalid_permission
syntax.binding_requires_let_or_set
syntax.postfix_try_removed
route.body_requires_action
api.inline_constraints_removed
```

Existing parser, runtime, and API diagnostics still include earlier prototype codes. Those should be migrated into namespaces before the diagnostic schema is frozen.

Future namespaces should follow the same pattern:

```text
syntax.expected_token
type.unknown_type
effect.permission_missing
test.expectation_failed
project.missing_config
project.invalid_config
project.exists
package.version_conflict
```

Diagnostic codes should be treated as public API once documented. Once external tools depend on a code, it should not be renamed casually.

## Permission diagnostics

Hayulo 2.0.0a0 includes stable preview diagnostics for generated API permission checks:

- `permission.missing`: an API source requires behavior that is not present in `[permissions].allow`
- `permission.denied`: an API source requires behavior explicitly listed in `[permissions].deny`
- `project.invalid_permission`: `hayulo.toml` contains a permission name that is not lowercase dotted syntax

Permission diagnostics include `details.permission` and the relevant allow/deny/required lists so repair tools can decide whether to remove behavior or ask the human to update policy.

## Suggestions must be safe

A suggestion should not merely be possible. It should be safe enough to recommend.

Bad suggestion:

```text
Change the parameter type to Any.
```

Better suggestion:

```text
Convert the value to Int with Int.parse(value) and handle the Result with try.
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
  "schema": "hayulo.test@0.1",
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

The current prototype has the first stable JSON envelopes:

- `hayulo.diagnostics@0.1` for normal command failures
- `hayulo.test@0.1` for test results and failing-test summaries
- legacy `errors`, `passed`, and `failed` fields retained for compatibility

The roadmap is to keep adding spans, related locations, safer suggestions, and repair hints without breaking the v0.1 fields.
