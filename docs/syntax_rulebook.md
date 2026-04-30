# Hayulo Syntax Philosophy and Rulebook

This is the canonical rulebook for evolving Hayulo syntax after the 2.0 draft cleanup.

Hayulo's syntax should optimize for one loop:

```text
human intent -> LLM generation -> compiler check -> structured diagnosis -> repair -> test -> review
```

The language should be pleasant for humans, but it exists to make useful programs easier to generate, verify, repair, and maintain with coding agents.

## Core Philosophy

Hayulo syntax should be boring in the best way:

- obvious over clever
- explicit over magical
- structured over flexible
- checkable over expressive
- repairable over terse
- one standard path over many equivalent styles

The goal is not to invent impressive syntax. The goal is to make correct software easier to produce repeatedly.

## Priority Order

When tradeoffs conflict, use this order:

1. Correctness
2. Human readability
3. LLM generation reliability
4. Static checkability
5. Diagnostic quality
6. Formatter stability
7. Brevity
8. Familiarity with other languages

Brevity is useful only after the higher priorities are satisfied.

## Non-Negotiable Rules

### 1. One Normal Way

Every common task should have one official shape.

Hayulo should not accept multiple equivalent spellings for the same concept unless compatibility requires it.

Reject:

- multiple module syntaxes
- multiple function syntaxes
- competing route declaration styles
- optional punctuation debates
- aliases that only save characters

### 2. No Hidden Control Flow

Control flow must be visible in source.

Reject syntax that hides:

- early returns
- async boundaries
- retries
- transactions
- permission checks
- error swallowing
- background execution

If an operation can fail, block, retry, mutate state, call the network, or touch storage, that fact must become checkable.

### 3. No Implicit Dangerous Values

Hayulo should not have `null`, `undefined`, implicit truthiness, or implicit numeric/string coercions.

Use explicit constructs:

```hayulo
Option<User>
Result<User, UserError>
```

Missing values and recoverable errors must be visible in types and diagnostics.

### 4. Names Should Explain Themselves

Keywords should be short, but not cryptic.

Prefer:

```hayulo
fn
type
record
route
intent
effect
test
expect
```

Reject new abbreviations unless they become overwhelmingly clear in examples and benchmarks.

### 5. Blocks Use Braces

Hayulo uses braces for blocks and the formatter owns indentation.

```hayulo
if ready {
  start()
} else {
  wait()
}
```

Significant whitespace is not part of the syntax philosophy because machine edits, patch application, and generated snippets need visible block boundaries.

### 6. Declarations Beat Convention

Important program facts should appear in declarations rather than hidden naming conventions.

Prefer:

```hayulo
route POST "/todos" body input: CreateTodo -> Todo {
  effect api.write
  effect storage.local
  action create Todo from input
}
```

Reject designs where behavior depends on file names, magic function names, implicit global variables, or framework discovery rules unless the project system exposes them clearly.

### 7. App-Building Layers Should Be Declarative First

For APIs, jobs, workflows, data schemas, and permissions, Hayulo should prefer declarative structures that the compiler can inspect.

Route actions use declarative shapes:

```hayulo
route POST "/todos" body input: CreateTodo -> Todo {
  effect api.write
  effect storage.local
  action create Todo from input
}
```

The app-building layer should reduce boilerplate and framework decisions before exposing arbitrary imperative code.

### 8. Effects Must Be Reviewable

Effects should be declared or inferred in a way tools can show in review.

Examples:

```hayulo
effect api.write
effect storage.local
```

Any syntax that introduces side effects must define:

- how effects are discovered
- how permissions are checked
- how diagnostics explain missing or denied effects
- how generated code exposes the behavior

### 9. Diagnostics Are Part Of The Syntax

A syntax feature is incomplete until its mistakes have stable diagnostics.

Every feature proposal must include likely user and LLM mistakes with diagnostic codes.

Example:

```text
type.option_not_handled
effect.permission_missing
route.body_type_unknown
syntax.expected_record_field
```

If the compiler cannot explain a feature well, the feature is not ready.

### 10. The Formatter Decides Style

Style choices should not be social debates.

Every accepted syntax feature must define formatter behavior:

- indentation
- wrapping
- field ordering when applicable
- blank lines
- trailing commas or no trailing commas
- stable output for already formatted code

If the formatter cannot make it canonical, the syntax is too loose.

### 11. Benchmark Before Preference

Syntax changes should be evaluated against LLM benchmark tasks when possible.

Useful evidence:

- fewer repair iterations
- fewer syntax or type errors
- clearer generated tests
- less boilerplate for equal behavior
- fewer unsafe generated behaviors
- better diagnostic repair success

Taste is not enough to change syntax.

## Default Syntax Shape

Use these defaults unless there is a stronger documented reason.

| Concept | Default |
| --- | --- |
| Blocks | `{ ... }` |
| Indentation | two spaces, formatter-owned |
| Function keyword | `fn` |
| New binding | `let name = value` |
| Reassignment | `set name = value` |
| Type declaration | `type Name = record { ... }` |
| Metadata | `intent { ... }` |
| Tests | `test "name" { expect ... }` |
| Missing values | `Option<T>` with `Some` / `None` |
| Recoverable errors | `Result<T, E>` with `Ok` / `Err` |
| Unwrapping | prefix `try expr` |
| Branching on variants | statement-form `match` |
| Effects | explicit or compiler-inferred dotted names |
| API routes | `route METHOD "/path" ... -> Type { ... }` |
| Comments | `// line comment` |

## Feature Proposal Checklist

No syntax feature should be implemented until the proposal answers these questions:

1. What problem does this solve for real Hayulo programs?
2. What is the smallest syntax that solves it?
3. What existing syntax would it replace or simplify?
4. What is the one official way to write it?
5. How does the parser reject malformed versions?
6. What diagnostics are required?
7. What formatter output is canonical?
8. What tests prove the feature works?
9. What examples show expected use?
10. What LLM benchmark task should improve?
11. What migration or compatibility risk exists?
12. What should be explicitly rejected?

## Rejection Checklist

Reject or defer a feature if any answer is yes:

- Does it add another way to write something Hayulo already does?
- Does it make source shorter but less inspectable?
- Does it depend on hidden runtime behavior?
- Does it require a large spec explanation for a small gain?
- Does it make diagnostics vague?
- Does it make formatting subjective?
- Does it allow generated code to hide risky effects?
- Does it encourage untyped maps where records should exist?
- Does it introduce null-like behavior?
- Does it optimize for expert cleverness instead of repeated reliable generation?

## Change Process

Syntax changes should follow this path:

1. Create a queue issue.
2. Add or update an example showing the desired source.
3. Add parser/checker diagnostics for bad source.
4. Add formatter behavior.
5. Add tests.
6. Run `make benchmark` and relevant manual LLM tasks when possible.
7. Update `SPEC.md` and syntax docs.
8. Document migration impact.
9. Close the issue only after implementation, tests, docs, and verification are complete.

## Design Biases For The Near Future

Hayulo should lean toward:

- declarative API actions
- explicit `Option` and `Result`
- first-class effects and permissions
- records before classes
- modules before packages
- simple imports before dependency ecosystems
- stable diagnostics before new syntax volume
- benchmark tasks before syntax expansion

Hayulo should avoid for now:

- inheritance
- macros
- operator overloading
- implicit conversions
- runtime monkey patching
- reflection-heavy frameworks
- multiple package managers
- syntax aliases
- advanced type-level programming

## The Final Test

Before adding syntax, ask:

> Will this make a coding agent more likely to produce a correct, reviewable, repairable program on the first or second try?

If the answer is unclear, measure first or defer.
