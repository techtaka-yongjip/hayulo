# Public Pitch

This document collects short messaging for README, website copy, launch posts, and conversations.

## One-sentence pitch

Hayulo is an open-source programming language and toolchain for AI-assisted software creation.

## Slightly longer pitch

Hayulo helps humans and coding agents build software together through intent blocks, predictable syntax, built-in tests, machine-readable diagnostics, and a repair-friendly compiler workflow.

## Short pitch

People can now describe software more easily than they can build it. LLMs can generate code, but the result is often fragile. Hayulo is a programming language and toolchain designed for the missing loop: human intent, AI-generated code, compiler checks, tests, structured diagnostics, repair, and human review.

## README tagline options

- Hayulo: where human ideas become working code.
- Hayulo: build software from intent.
- Hayulo: an AI-native language for useful software.
- Hayulo: human intent, reliable code.
- Hayulo: a repairable programming language for AI-assisted development.

Recommended tagline:

> Hayulo: where human ideas become working code.

## Problem statement

AI can write code, but today's languages and toolchains were not designed around AI repair loops. Generated projects often fail because of missing dependencies, unclear errors, inconsistent structure, weak tests, and hidden framework conventions.

Hayulo is designed around the loop from the start.

## Solution statement

Hayulo combines:

- readable syntax
- intent blocks
- built-in tests
- structured compiler diagnostics
- stable project layout
- explicit `Option` and `Result`
- declared route effects and project permissions
- an AI repair protocol
- one official toolchain

## Short launch post

Today I am publishing Hayulo 2.0.0a0, a breaking draft of an experimental open-source programming language for AI-assisted software creation.

The idea is simple: humans should be able to express intent, LLMs should be able to generate and repair code, and the compiler should provide structured diagnostics and tests that make the loop reliable.

The core is small, but real: it has a lexer, parser, interpreter, CLI, formatter, examples, tests, JSON diagnostics, project permissions, explicit `Option`/`Result` handling, and declarative REST API generation.

The larger ambition is to build a language where intent, code, tests, permissions, and repair are part of one coherent workflow.

## GitHub repository description

An experimental programming language and toolchain for AI-assisted software creation.

## Website hero copy

```text
Hayulo
Where human ideas become working code.

Hayulo is an open-source programming language and toolchain designed for AI-assisted software creation: intent blocks, predictable syntax, built-in tests, structured diagnostics, and repair loops from day one.
```

## What makes Hayulo different?

Hayulo is not trying to be another general-purpose syntax experiment. It is designed around a new development loop:

```text
human intent -> LLM code -> compiler diagnostics -> tests -> repair -> review
```

The language and toolchain are built together for that loop.

## Honest limitation statement

Hayulo 2.0.0a0 is a draft, not a production-ready release. The current version is intended for outside testing, repair-loop research, REST API generation experiments, and contributor feedback.

## Comparison language

Use this carefully:

> Hayulo aims to combine Python-like approachability, TypeScript-like structure, Go-like practicality, Rust-inspired error handling, and AI-native compiler diagnostics.

Avoid claiming superiority over mature ecosystems.

## FAQ snippets

### Is Hayulo a no-code tool?

No. Hayulo is a real programming language. It is designed to work well with AI assistance, but humans can read and write it directly.

### Is Hayulo production-ready?

No. The current prototype is experimental.

### Why create a new language?

Because LLM-assisted development creates new requirements: structured diagnostics, repairability, intent preservation, safe effects, and predictable project layout.

### Why not just use Python or TypeScript?

Those languages are powerful and mature. Hayulo's goal is different: become the best language and toolchain for human-AI software creation workflows.
