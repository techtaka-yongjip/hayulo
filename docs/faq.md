# Hayulo FAQ

## What is Hayulo?

Hayulo is an experimental open-source programming language and toolchain for AI-assisted software creation.

## Is Hayulo a no-code tool?

No. Hayulo is a real programming language. It is designed to work well with LLM assistance, but the output is source code that humans can read, review, test, and maintain.

## Is Hayulo production-ready?

No. Hayulo is pre-alpha. The current prototype is a seed interpreter meant to explore the design and invite contributors.

## Why create a new language?

Because LLM-assisted development creates new requirements:

- preserving human intent
- structured diagnostics for repair
- built-in tests
- predictable project layout
- safe effects and permissions
- codebase summaries for model context
- one toolchain for generation, checking, repair, and review

Existing languages can add some of this through tooling. Hayulo explores what happens when these needs are designed into the language and toolchain from the beginning.

## Why not just use Python?

Python is excellent and mature. Hayulo is not trying to beat Python everywhere.

Hayulo's goal is narrower: provide a language and toolchain optimized for AI-assisted software creation, especially for small apps and tools where repairability, diagnostics, and intent matter.

## Why not just use TypeScript?

TypeScript is powerful and widely used. Hayulo can learn from it.

The difference is that Hayulo can avoid inheriting JavaScript's historical complexity and can design diagnostics, project layout, effects, permissions, and repair protocols as core features.

## What does AI-native mean?

AI-native means the language and toolchain assume that LLMs may generate, edit, and repair code.

It does not mean humans are removed. It means humans and models work together through a structured loop.

## What is an intent block?

An `intent` block stores purpose and constraints in source code.

Example:

```hayulo
intent {
  purpose: "Manage user accounts."
  constraints: [
    "Email must be unique.",
    "Users should be soft-deleted before permanent deletion."
  ]
}
```

Future tools can use intent blocks to guide LLM edits and detect drift.

## What is the current implementation written in?

The seed prototype is written in Python for fast iteration.

A future compiler core may be rewritten in Rust or another systems language after the language design is more stable.

## What should contributors work on first?

Good early work:

- parser diagnostics
- source spans
- examples
- tests
- `hayulo --version`
- docs
- list/map literals
- formatter design
- diagnostic snapshots

## What is the long-term goal?

The long-term goal is to make Hayulo the most reliable path from human intent to useful, understandable, AI-assisted software.
