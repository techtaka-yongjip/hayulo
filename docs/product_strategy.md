# Product Strategy

Hayulo is a programming language, but adoption will depend on the product experience around it. A beautiful syntax is not enough. The project needs a clear first audience, a clear first use case, and a clear reason to choose Hayulo over existing languages.

## Core positioning

Hayulo should be positioned as:

> An open-source programming language and toolchain for AI-assisted software creation.

The important words are:

- **open-source**: community-owned, inspectable, and extensible
- **programming language**: real code, not only prompts or no-code blocks
- **toolchain**: compiler, tests, diagnostics, formatter, packages, and repair loop
- **AI-assisted**: designed for humans and LLMs working together
- **software creation**: practical output, not only experiments

## The wedge

Hayulo should not initially compete with every language everywhere. The first adoption wedge should be:

> Small useful apps and tools generated or modified with LLM assistance.

Examples:

- todo apps
- internal admin tools
- CSV processors
- invoice extractors
- simple APIs
- local automation scripts
- file renamers
- lightweight dashboards
- classroom projects
- individual productivity tools

These are places where predictable syntax, strong diagnostics, and built-in tests can quickly show value.

## Primary users

### AI-assisted builders

People who use LLMs to build software but get stuck when generated code breaks.

They need:

- simple project setup
- fewer dependencies
- clear errors
- automatic tests
- repair guidance
- examples they can copy

### Professional developers

Developers who already code but want a better AI coding loop.

They need:

- strong tooling
- reliable checking
- predictable standard library
- editor support
- clear roadmap
- transparent design decisions

### Educators and learners

Teachers and students exploring programming in an AI-assisted world.

They need:

- readable syntax
- clear mental models
- immediate feedback
- visible tests
- explanations
- a safe environment

### Tool builders and researchers

People studying LLM coding, compiler feedback, program repair, and human-AI workflows.

They need:

- structured diagnostics
- stable test corpus
- benchmarkable tasks
- clear internal architecture
- language server and repair protocol

## First product experience

The first delightful Hayulo experience should be small:

```bash
hayulo new hello
cd hello
hayulo run
hayulo test
hayulo check --json
```

Then:

```text
Ask an LLM to add a feature.
Run hayulo check.
Copy the JSON diagnostic back to the LLM.
Watch it fix the code.
Run hayulo test.
```

The product should prove that Hayulo is easier to repair than a typical generated Python or JavaScript project.

## Differentiation

Hayulo should not say:

> We are better than Python at everything.

That is not credible.

Hayulo should say:

> Existing languages were not designed around LLM generation and repair. Hayulo is.

The differentiators are:

- intent blocks
- machine-readable diagnostics
- built-in tests
- repair protocol
- predictable project layout
- effects and permissions
- official app-building standard library
- one toolchain
- LLM-friendly syntax and semantics

## Minimum lovable product

The minimum lovable product is not a full production compiler. It is a small system that feels coherent.

Required:

- `hayulo new`
- `hayulo run`
- `hayulo test`
- `hayulo check --json`
- formatter
- helpful diagnostics
- examples
- tutorial
- simple standard library
- one small real app demo

A user should be able to install Hayulo, create a project, run tests, and understand why the language exists in under ten minutes.

## Public demos

### Demo 1: Repair loop

1. Show a broken Hayulo program.
2. Run `hayulo check --json`.
3. Show structured diagnostic.
4. Apply a small patch.
5. Run tests successfully.

### Demo 2: Intent-preserving feature addition

1. Start with a todo API.
2. Show `intent` constraints.
3. Ask an LLM to add due dates.
4. Verify that ownership and validation constraints remain intact.

### Demo 3: Effects and permissions concept

1. Show code that tries to send an email.
2. Show compiler/runtime requiring permission.
3. Show human approval concept.

### Demo 4: Beginner app

Create a small CLI tool from scratch.

## Adoption strategy

### Stage 1: Credibility

- Publish repo with working prototype.
- Provide clear docs and examples.
- Be honest that it is pre-alpha.
- Ask for design feedback.

### Stage 2: Useful toy apps

- Add enough features for CLI tools and data processing.
- Build 5 to 10 polished examples.
- Create tutorials and videos.

### Stage 3: AI repair benchmark

- Compare Hayulo repair loops against small Python/JS examples.
- Measure how often LLMs fix errors from structured diagnostics.
- Publish results transparently.

### Stage 4: Simple web services

- Add HTTP and SQLite.
- Build a todo app, notes app, and invoice tracker.
- Show generated tests and diagnostics.

### Stage 5: Community packages

- Create package format.
- Create registry prototype.
- Add package quality guidelines.

## Message discipline

Avoid overclaiming.

Use phrases like:

- experimental
- early prototype
- designed for
- aims to
- roadmap
- pre-alpha

Avoid phrases like:

- replaces all programming
- AI builds anything safely
- production-ready today
- no coding required for all apps

Trust is more valuable than hype.

## Product north star metric

A useful north star metric:

> Time from human intent to passing tests for a small useful program.

Secondary metrics:

- number of compiler errors repaired by LLM using JSON diagnostics
- percentage of examples generated correctly from prompts
- time to first successful app
- number of contributors
- number of packages
- number of projects using `intent` blocks meaningfully
