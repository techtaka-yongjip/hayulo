# Language Design Principles

This document describes principles that should guide Hayulo's syntax, semantics, type system, and standard patterns.

## Principle 1: Readability before cleverness

Hayulo code should be easy to read aloud, explain, and review.

A feature that saves a few characters but makes code harder to inspect should usually be rejected.

Good:

```hayulo
fn add(a: Int, b: Int) -> Int {
  return a + b
}
```

## Principle 2: One obvious way for common tasks

The language should minimize redundant styles.

There should be one normal way to:

- define a function
- define a record
- write a test
- import a module
- handle missing values
- handle recoverable errors
- declare a route
- declare effects

This helps humans learn and helps LLMs generate consistent code.

## Principle 3: Braces over significant whitespace

Hayulo should use braces for blocks:

```hayulo
if ready {
  start()
} else {
  wait()
}
```

Significant whitespace can be pleasant for humans, but braces are safer for machine editing, copying, patching, and refactoring.

The formatter should still enforce clean indentation.

## Principle 4: No semicolons by default

Hayulo should avoid semicolons unless a future grammar requirement makes them necessary.

This keeps code lighter while braces preserve structure.

## Principle 5: Static checking with useful inference

Hayulo should catch errors before runtime, but it should not require excessive annotation.

Public APIs should be explicit:

```hayulo
pub fn calculate_total(items: List<Item>) -> Money {
  ...
}
```

Local variables can be inferred:

```hayulo
total = Money.zero("USD")
```

This balances readability, safety, and convenience.

## Principle 6: No null by default

Missing values should use `Option<T>`.

```hayulo
fn find_user(id: UserId) -> Option<User> {
  ...
}
```

Using a possibly missing value should require explicit handling:

```hayulo
match find_user(id) {
  Some(user) => print(user.name)
  None => print("User not found")
}
```

## Principle 7: Recoverable errors use Result

Expected failures should use `Result<T, E>`.

```hayulo
fn read_config(path: Path) -> Result<Config, FileError> {
  text = files.read_text(path)?
  return Config.parse(text)
}
```

Unexpected defects can still be internal errors or panics, but normal failures should be part of the type signature.

## Principle 8: Records over classes early

Hayulo should begin with records and functions rather than complex class hierarchies.

```hayulo
type User = record {
  id: Id<User>
  email: Email
  name: Text
}
```

Behavior can be added later through functions, traits, or simple methods. Deep inheritance should not be part of the early design.

## Principle 9: Intent blocks are first-class metadata

Every module may contain an `intent` block:

```hayulo
intent {
  purpose: "Manage user accounts."
  constraints: [
    "Email must be unique.",
    "Deleted users should be soft-deleted first."
  ]
}
```

Intent should be parseable and available to tools.

It should eventually support:

- purpose
- constraints
- non-goals
- examples
- safety notes
- owner/contact metadata
- generated/handwritten status

## Principle 10: Effects should be visible

Functions that touch the outside world should declare or infer effects:

```hayulo
fn save_report(path: Path, report: Report) -> Result<(), FileError>
  effects [files.write]
{
  return files.write_text(path, report.to_text())
}
```

Effects are important for generated-code safety, project permissions, and review.

## Principle 11: Tests are language citizens

Tests should be built in:

```hayulo
test "total includes tax" {
  expect total(Money.usd(100), tax: 0.1) == Money.usd(110)
}
```

The test runner should understand source spans, diagnostics, and structured output.

## Principle 12: Diagnostics are API contracts

A diagnostic code is an API. Once tools depend on it, it should not change casually.

Examples:

```text
syntax.expected_token
name.unknown_symbol
type.mismatch
type.option_not_handled
effect.permission_missing
test.expectation_failed
```

Diagnostic stability matters for AI repair tools.

## Principle 13: The formatter defines style

Style debates should be settled by the official formatter.

Generated code should run through `hayulo format` before review.

The formatter should prefer:

- two-space indentation
- stable import ordering
- stable record field layout
- readable line wrapping
- minimal diff noise

## Principle 14: Interop is practical, but not at the cost of clarity

Hayulo should eventually interoperate with existing ecosystems. But interop should not leak confusing semantics into the core language.

For example, if compiling to JavaScript, Hayulo should not inherit JavaScript's implicit coercions or `undefined` behavior.

## Principle 15: Small features must compose

Each feature should work well with:

- formatter
- type checker
- diagnostics
- tests
- intent metadata
- package system
- AI repair protocol

A feature that cannot be checked, explained, or repaired may not belong in the language yet.

## Feature acceptance questions

Before accepting a feature, ask:

1. Does it make common programs clearer?
2. Can it be explained in the spec simply?
3. Can the compiler diagnose mistakes well?
4. Can an LLM generate it consistently?
5. Can the formatter handle it cleanly?
6. Does it interact safely with effects and permissions?
7. Does it help the roadmap now, or is it premature?
