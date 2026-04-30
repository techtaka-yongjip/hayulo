# Ambitions

Hayulo is starting as a small interpreter, but the ambition is larger: a practical, open-source language and ecosystem for AI-assisted software creation.

This document describes the project's ambitions without pretending they are already implemented.

## Short-term ambition: prove the loop

The first ambition is to prove the basic Hayulo loop:

```text
write code
run hayulo check --json
get structured diagnostics
repair code
run hayulo test
```

If Hayulo can make this loop better than the usual generated Python or JavaScript experience for small programs, the project has a strong foundation.

## Medium-term ambition: useful small software

Hayulo should become useful for:

- command-line tools
- file-processing scripts
- data cleanup tools
- JSON utilities
- small web APIs
- local dashboards
- simple workflow automations
- educational programming exercises
- AI-assisted document and data tools

The project should first become good at small useful programs before trying to become a universal platform.

## Long-term ambition: software from intent

The deeper ambition is to make software development more intent-driven.

A user should be able to state a goal and constraints:

```text
Build a tool for my tutoring business.
It should track students, lessons, invoices, and parent messages.
It should never send a message without approval.
```

Hayulo should help turn that into:

- source intent
- domain types
- generated functions
- tests
- permission declarations
- approval gates
- diagnostics
- reviewable changes

The final result should be code, not just a black box.

## Educational ambition

Hayulo could become a strong teaching language for the AI era.

Students could learn:

- functions
- types
- data modeling
- tests
- errors
- APIs
- compiler feedback
- safe generated code

An educational Hayulo environment could show:

- the human intent
- the generated code
- the compiler explanation
- the failing test
- the repair patch
- the final working result

This teaches programming as a conversation between human, model, and toolchain.

## Accessibility ambition

Hayulo could help more people create useful software:

- nontechnical creators
- teachers
- students
- researchers
- public servants
- artists
- community organizers
- small business owners
- professionals who know their domain but not a modern stack

The goal is not to pretend that anyone can safely build anything. The goal is to lower the barrier for many useful tools while keeping code inspectable.

## Professional developer ambition

Hayulo should also serve serious developers.

For professionals, the value is not "no code." The value is faster iteration with stronger structure:

- less boilerplate
- consistent app architecture
- typed APIs
- generated tests
- repairable diagnostics
- safer AI code review
- less dependency sprawl
- clear effects and permissions

A professional should eventually be able to use Hayulo for internal tools, prototypes, automations, and selected production services.

## Ecosystem ambition

A mature Hayulo ecosystem could include:

- official package registry
- package signing and integrity checks
- language server
- editor extensions
- formatter
- compiler daemon
- browser playground
- local app runtime
- cloud deployment adapters
- AI repair engine
- tutorial gallery
- public package templates
- benchmark suite for LLM coding reliability

The ecosystem should be designed around trust. Generated packages and AI-suggested dependencies should be inspectable, versioned, and permission-aware.

## Research ambition

Hayulo can become a research platform for questions like:

- Which language features make LLM code generation more reliable?
- Which diagnostic formats produce the best automated repairs?
- How should intent be represented in source code?
- How can compilers detect unsafe generated behavior?
- How can tests be generated from human constraints?
- How can models modify code without drifting from original purpose?

The project can contribute not only software but knowledge.

## Safety ambition

A language for AI-generated software must take safety seriously.

Long-term, Hayulo should make it possible to run generated programs under clear restrictions:

```text
This program can read local CSV files.
This program can write only to the output directory.
This program can call approved HTTP APIs.
This program cannot send email.
This program cannot spend money.
This program cannot delete user data.
```

This requires effects, permissions, sandboxing, audit logs, and review gates.

## Cultural ambition

Hayulo should become known for thoughtful building:

- build useful things
- preserve intent
- make tools understandable
- let AI help, but keep humans in control
- prefer boring reliability over clever magic
- welcome contributors who care about quality

## What Hayulo should not try to become immediately

Hayulo should not try to be everything.

It should not compete with C or Rust for operating system kernels.

It should not compete with mature numerical ecosystems immediately.

It should not become a giant framework with hidden behavior.

It should not promise that AI can safely build all software without review.

It should not add features just because other languages have them.

## North star

> Hayulo should be the most reliable path from human intent to useful, understandable, AI-assisted software.
