# Ecosystem

Hayulo is a programming language, but the ecosystem will determine whether it becomes useful.

The ecosystem should make the AI-assisted development loop practical:

```text
create project
  -> write or generate code
  -> check
  -> test
  -> repair
  -> format
  -> build
  -> share
  -> deploy
```

## Core ecosystem components

A mature Hayulo ecosystem may include:

- language specification
- reference implementation
- official CLI
- formatter
- test runner
- package manager
- package registry
- standard library
- language server
- editor plugins
- playground
- documentation site
- example gallery
- AI repair protocol
- benchmark suite
- security audit tools
- deployment adapters

## Official CLI

The `hayulo` command should be the front door:

```bash
hayulo new <app>
hayulo run
hayulo test
hayulo check
hayulo format
hayulo build
hayulo repair
```

One official toolchain reduces confusion for humans and LLMs.

## Standard library

Hayulo should include official libraries for common work:

- JSON
- text
- lists
- maps
- files
- paths
- CLI apps
- logging
- time
- configuration
- HTTP
- SQLite
- validation
- tests
- AI model calls eventually

The standard library should reduce early dependency chaos.

## Package registry

The registry should be designed around trust:

- lockfiles
- checksums
- package signing
- dependency metadata
- effect summaries
- permission requirements
- documentation generation
- security advisories

A package should not only say what it exports. It should say what it can do.

## Editor support

Developers live in editors. Hayulo should eventually provide:

- syntax highlighting
- diagnostics
- format on save
- go to definition
- hover docs
- rename symbol
- code actions
- test discovery
- intent block display
- effect warnings

## Playground

A web playground can make the language easy to try.

Good playground features:

- run examples
- show output
- show diagnostics
- show AST or tokens for learning
- run tests
- share snippets
- demonstrate repair hints

## Example gallery

Examples are critical because both humans and LLMs learn from them.

The gallery should include:

- hello world
- calculator
- word counter
- JSON formatter
- file organizer
- todo API
- notes app
- invoice tracker concept
- repair demo
- permission/effect demo

Each example should include:

- intent block
- source code
- tests
- expected output
- explanation

## Ecosystem naming

Possible names for future tools are placeholders:

```text
Hayulo        language and CLI
Beacon        language server
Hammer        repair protocol/tool
Anvil         package/build cache
Foundry       build/deployment service
Mold          project templates
```

These should be checked before public branding. The first priority is making the core language and toolchain useful.

## Ecosystem principle

The Hayulo ecosystem should prefer coherent defaults over endless configuration.

Advanced users can customize later, but the beginner and LLM path should be simple:

```bash
hayulo new app
hayulo test
hayulo check --json
hayulo format
```
