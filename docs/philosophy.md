# Philosophy

Hayulo is guided by a simple belief: the future of programming is not just humans writing code or AI writing code. It is humans and AI working through a disciplined loop of intent, implementation, checking, testing, repair, and review.

The language should be designed for that loop.

## Human intent comes first

Most source code explains what the machine should do. It rarely explains why the software exists.

Hayulo treats intent as part of the program:

```hayulo
intent {
  purpose: "Manage invoices for freelancers."
  constraints: [
    "Never delete invoices permanently.",
    "All money values must include currency."
  ]
}
```

This matters because LLMs modify code better when they know the goal and constraints. Humans also review code better when the purpose is visible.

## Code should be readable after generation

AI-generated code is only useful if humans can inspect it.

Hayulo should avoid language features that produce clever but opaque code. It should favor boring clarity:

- explicit module structure
- readable function declarations
- records for data
- tests close to behavior
- consistent formatting
- few redundant patterns

## Repairability is a language feature

A language designed for AI-assisted coding must care deeply about errors.

A weak error says:

```text
Invalid expression
```

A repairable diagnostic says:

```json
{
  "code": "syntax.expected_expression",
  "message": "Expected an expression after '='.",
  "location": { "file": "src/main.hayulo", "line": 12, "column": 10 },
  "suggestions": [
    { "kind": "insert_expression", "message": "Add a value, variable name, or function call." }
  ]
}
```

The diagnostic is part of the programming experience. It is also part of the AI repair protocol.

## Safety should be visible

Generated code can act quickly. It can read files, call APIs, write databases, send messages, delete records, or expose private data.

Hayulo should make those actions visible through effects and permissions:

```hayulo
fn export_report(path: Path, report: Report) -> Result<(), FileError>
  effects [files.write]
{
  return files.write_text(path, report.to_text())
}
```

The user should be able to see what a program can do before it does it.

## Tests are not optional decoration

A generation loop without tests is fragile. Hayulo should make tests normal:

```hayulo
test "total includes tax" {
  expect total(Money.usd(100), Tax.percent(10)) == Money.usd(110)
}
```

Tests let humans state expected behavior. They give LLMs a target. They make repair measurable.

## One toolchain is better than many mysteries

Modern software stacks often require a language, package manager, build tool, formatter, linter, framework, test runner, bundler, runtime, and deployment system. That can be powerful, but it creates confusion for new users and LLMs.

Hayulo should have one official toolchain:

```bash
hayulo new
hayulo run
hayulo test
hayulo check
hayulo format
hayulo build
hayulo repair
```

The goal is not to hide complexity forever. The goal is to provide a reliable default path.

## Avoid magic where possible

Framework magic is hard for humans to debug and hard for LLMs to repair. Hayulo should prefer explicit structure over hidden behavior.

If the language adds a convenient feature, it should still be explainable by the compiler and visible to tools.

## AI-native does not mean AI-only

Hayulo must remain usable by humans without a model. The language should be teachable, readable, and manually debuggable.

AI-native means the language assumes LLMs are common collaborators. It does not mean humans stop understanding the code.

## Humility is part of the philosophy

Hayulo should not claim that AI can safely build all software. It should not claim that a new syntax replaces mature ecosystems.

Its honest claim is narrower and stronger:

> Existing languages were not designed around LLM generation and repair. Hayulo is exploring what happens when that workflow becomes a first-class design constraint.
