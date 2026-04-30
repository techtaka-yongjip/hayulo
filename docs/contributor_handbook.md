# Contributor Handbook

Welcome to Hayulo. This project is building an experimental programming language and toolchain for AI-assisted software creation.

This handbook explains how to contribute in a way that helps the project stay coherent.

## Ways to contribute

You can help by:

- improving documentation
- adding examples
- writing tests
- improving diagnostics
- fixing parser/interpreter bugs
- proposing language design changes
- building standard library modules
- testing LLM repair workflows
- improving error messages
- creating tutorials
- reviewing issues and pull requests

## What matters most early

The project is young. The most valuable contributions are not huge features. They are small changes that improve reliability and clarity.

High-value early contributions:

- add failing tests before fixing parser bugs
- improve a diagnostic message
- add a simple example
- document a design decision
- simplify confusing code
- make CLI output clearer
- add a snapshot test for JSON diagnostics

## Development setup

```bash
python -m venv .venv
. .venv/bin/activate
pip install -e .
python -m unittest discover -s tests
```

Run examples:

```bash
hayulo run examples/hello.hayulo
hayulo test examples/hello.hayulo
hayulo check examples/hello.hayulo --json
```

Without installing:

```bash
PYTHONPATH=src python -m hayulo run examples/hello.hayulo
```

## Contribution principles

### Keep the language small

Do not add a feature only because it exists elsewhere. Explain why Hayulo needs it now.

### Make diagnostics better

Every error should help the user take the next step.

### Add tests

Compiler projects break easily. Tests protect the language.

### Update docs

If user-facing behavior changes, update documentation and examples.

### Think about LLM repair

Ask whether a model could understand and fix an error from the output.

## Pull request checklist

Before opening a pull request:

- tests pass
- examples still run
- new behavior has tests
- user-facing changes are documented
- diagnostics are clear
- formatting is consistent
- scope is focused

## Language feature proposal checklist

For a language feature, include:

- problem statement
- examples
- alternatives considered
- interaction with types
- interaction with effects/permissions
- diagnostic strategy
- formatter strategy
- tests required
- migration concern if any

## Diagnostic contribution guide

A good diagnostic has:

- stable code
- clear message
- precise location
- explanation
- safe suggestions
- JSON fields that tools can use

Bad diagnostic:

```text
Error: invalid expression
```

Better diagnostic:

```text
error syntax.expected_expression: Expected an expression after '='.
  --> src/main.hayulo:12:10
suggestion: Add a value, variable name, or function call after '='.
```

## Example contribution guide

A good example should include:

- intent block
- clear code
- tests
- expected output
- comments only where helpful
- no unsupported features unless clearly marked as future design

## Design discussions

Use issues for design discussion. For major decisions, propose a design record.

Do not rush syntax decisions. Syntax is hard to change once users copy examples.

## Review culture

Reviews should be kind and direct.

Good review comments:

- explain the reason
- suggest alternatives
- point to docs or roadmap
- separate individual preference from project direction

Avoid:

- dismissive comments
- vague rejection
- arguing without examples
- large unrelated requests in small PRs

## Good first issue ideas

- add `hayulo --version`
- improve missing brace diagnostic
- add more lexer tests
- add parser tests for invalid syntax
- add `docs/design_records/0001-braces.md`
- add examples for string operations
- add JSON diagnostic snapshots
- add a small tutorial
