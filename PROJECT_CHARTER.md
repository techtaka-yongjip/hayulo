# Hayulo Project Charter

## Mission

Hayulo exists to make AI-assisted software creation more reliable, understandable, and safe.

The project explores a simple thesis:

> A programming language for the AI era should preserve human intent, help LLMs generate and repair code, provide structured diagnostics, make testing natural, and expose risky effects before software acts on the world.

Hayulo is a real programming language project, not a prompt collection and not a no-code wrapper. It should produce readable source code that humans can inspect, review, test, and maintain.

## North star

Hayulo should become the most reliable path from human intent to useful, understandable, AI-assisted software.

That means the project should optimize for the whole loop:

```text
intent -> code -> checks -> tests -> diagnostics -> repair -> review -> useful result
```

## What Hayulo values

### Human intent

Software starts with human goals. Hayulo should make those goals visible in the source through `intent` blocks, tests, constraints, and documentation.

### Reliability

Generated code should not merely look plausible. It should parse, check, run, and pass tests.

### Understandability

The code should be readable by ordinary programmers. Cleverness that makes code harder to review is usually a bad trade.

### Repairability

Errors should be designed as useful feedback. Diagnostics should help both humans and LLMs understand what went wrong and how to fix it safely.

### Safety

Generated code should not silently gain dangerous powers. Effects, permissions, sandboxing, and review gates are core to the long-term language vision.

### Openness

The core language, prototype, docs, and design process should be open-source and inspectable.

### Humility

Hayulo is experimental. It should make ambitious claims about what it is trying to learn, not false claims about being production-ready before it is.

## Scope

Hayulo is intended to support:

- scripts
- command-line tools
- small web APIs
- internal tools
- local-first applications
- data processing tools
- educational projects
- AI-assisted app generation
- research into LLM coding reliability

Hayulo is not initially intended for:

- operating system kernels
- real-time embedded systems
- high-frequency trading systems
- safety-critical medical or aviation software
- replacing mature ecosystems in all domains

## Initial users

The first users are likely to be:

- people building with LLMs who want less fragile generated code
- developers interested in AI-native tooling
- educators teaching programming in an AI-assisted world
- researchers studying code generation and repair
- open-source contributors interested in language design

## Project promises

Hayulo should promise:

- clear documentation
- honest status labels
- tested examples
- readable design decisions
- structured diagnostics
- small, incremental releases
- respectful community standards

Hayulo should not promise:

- production readiness today
- that AI can safely build all software without review
- that it replaces Python, JavaScript, Java, Rust, or Go
- that syntax alone solves AI coding reliability

## Decision rule

When deciding whether to add a feature, ask:

1. Does this help humans express intent?
2. Does this help LLMs generate correct code?
3. Can the compiler diagnose mistakes clearly?
4. Can the formatter make it consistent?
5. Can tests verify the behavior?
6. Does it make risky actions visible?
7. Is it needed for the next milestone?

If the answer is no, the feature may be premature.

## Cultural standard

Hayulo should be built with a culture of thoughtful building:

- build useful things
- preserve intent
- prefer boring reliability over clever magic
- make tools understandable
- let AI help, but keep humans in control
- welcome contributors who care about quality

## Long-term aspiration

Hayulo can become more than a language. It can become a humane software creation environment where people can describe what they need, see the code that implements it, understand the risks, test the behavior, and improve the result with AI assistance.
