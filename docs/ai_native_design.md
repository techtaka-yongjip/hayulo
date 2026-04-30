# AI-Native Design

Hayulo is designed around a specific claim:

> A language for AI-assisted software creation should be optimized for generation, checking, repair, and explanation, not only for manual typing.

This document explains what AI-native means for Hayulo.

## AI-native does not mean AI-only

Hayulo must remain a real programming language for humans. Humans should be able to read, write, review, and debug Hayulo code without relying on a model.

AI-native means the language and toolchain account for LLMs as common participants in the development loop.

## The Hayulo development loop

```text
1. Human describes intent.
2. LLM writes or edits Hayulo code.
3. Hayulo compiler checks syntax, types, effects, and tests.
4. Compiler returns structured diagnostics.
5. LLM repairs the code using diagnostics.
6. Tests run again.
7. Human reviews the result.
```

The language should reduce friction at every step.

## Preserve intent

LLMs often make bad edits because they lack context. Hayulo's `intent` blocks give code a stable place to store purpose and constraints.

```hayulo
intent {
  purpose: "Track invoices for freelancers."
  constraints: [
    "Do not delete invoices permanently.",
    "All amounts must include currency.",
    "Exports must work in spreadsheet software."
  ]
}
```

Future tools should extract intent blocks and provide them in model context automatically.

## Predictable structure

LLMs perform better when codebases have consistent structure.

Hayulo projects should look predictable:

```text
my_app/
  hayulo.toml
  src/
    main.hayulo
    models.hayulo
    api.hayulo
  tests/
    api_test.hayulo
  assets/
  migrations/
```

There should be one official formatter and one official project format.

## Fewer equivalent choices

Languages with many equivalent patterns create uncertainty for models.

Hayulo should avoid unnecessary alternatives:

- no multiple module systems
- no competing package managers
- no hidden import styles
- no optional semicolon debate
- no multiple inheritance
- no runtime monkey patching
- no common values that can be both `null` and `undefined`

This makes generated code more consistent.

## Structured diagnostics

Human-readable errors are not enough. Hayulo diagnostics should be designed for programs to consume.

A diagnostic should include:

- stable code
- severity
- file, line, column, and span
- short message
- explanation
- related symbols
- safe suggestions
- optional patch hints
- tests to run next

Example:

```json
{
  "code": "missing_result_handling",
  "severity": "error",
  "message": "This function returns Result<User, DbError>, but the error case is not handled.",
  "location": {
    "file": "src/users.hayulo",
    "line": 42,
    "column": 10
  },
  "suggestions": [
    {
      "kind": "use_try",
      "text": "Use try to return the error from the current function.",
      "patch_hint": "try db.users.find(id)"
    },
    {
      "kind": "match_result",
      "text": "Use match to handle Ok and Err explicitly."
    }
  ]
}
```

## Tests close to behavior

Hayulo should make tests easy to write and discover.

```hayulo
test "invoice total includes all line items" {
  let invoice = sample_invoice()
  expect invoice.total() == Money.usd(120)
}
```

LLMs are more reliable when they can generate tests alongside code.

## Stable formatting

Formatting should be automatic and canonical.

Generated code should be normalized before review. This reduces noisy diffs and makes it easier to compare AI-generated patches.

Command:

```bash
hayulo format
```

## Codebase summaries

LLM coding agents often need context, but dumping an entire repository into a prompt is inefficient.

Hayulo should eventually provide:

```bash
hayulo summarize --json
```

Possible output:

```json
{
  "modules": [
    {
      "name": "app.users",
      "intent": "Manage user accounts.",
      "exports": ["User", "create_user", "find_user"],
      "effects": ["db.read", "db.write"],
      "tests": 8
    }
  ]
}
```

## Safe patching

Hayulo should make it easier to apply patches safely.

A future patch should be rejected if it:

- changes files outside allowed scope
- expands permissions unexpectedly
- removes tests without explanation
- violates formatting
- introduces high-risk effects
- fails checks

## Explicit uncertainty around model calls

If Hayulo includes AI model calls, their outputs should be treated carefully.

Example future design:

```hayulo
type InvoiceData = record {
  vendor: Text
  total: Money
  due_date: Date
}

fn extract_invoice(doc: Document) -> Result<Belief<InvoiceData>, AiError> {
  return ai.extract<InvoiceData> {
    instruction: "Extract invoice data."
    input: doc
  }
}
```

A `Belief<T>` would carry confidence and evidence. This prevents model output from pretending to be ordinary verified data.

## Repair benchmarks

Hayulo should measure whether its AI-native design actually helps.

Possible benchmark:

1. Create small broken programs.
2. Run `hayulo check --json`.
3. Give diagnostics to an LLM.
4. Measure repair success rate.
5. Compare against equivalent Python/TypeScript tasks.

This would make the project evidence-driven.

## AI-native feature checklist

A proposed feature should answer:

- Can humans read it easily?
- Can LLMs generate it reliably?
- Can the compiler diagnose mistakes clearly?
- Can a repair tool suggest safe fixes?
- Does it preserve or use intent?
- Does it interact clearly with tests?
- Does it make risky effects visible?
