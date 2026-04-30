# Vision

Hayulo is a programming language for the next software creation workflow.

For decades, programming languages have been designed mainly for humans typing instructions into editors. That will remain important, but it is no longer the whole story. Increasingly, people describe what they want to build, LLMs generate code, tools check it, tests catch mistakes, and models repair the result.

Hayulo is designed around that loop from the beginning.

## The vision in one sentence

Hayulo should make it easier for humans and LLMs to turn clear intent into useful, understandable, tested software.

## The workflow Hayulo is built for

```text
Human describes the goal
  -> LLM generates Hayulo code
  -> Hayulo compiler checks it
  -> Hayulo tests verify it
  -> structured diagnostics explain failures
  -> human or LLM repairs it
  -> human reviews and uses it
```

The vision is not autonomous software magic. The vision is a better partnership between human judgment and machine assistance.

## Why a new language?

Python, JavaScript, TypeScript, Java, Go, and Rust are excellent languages. Hayulo does not need to replace them to matter.

Hayulo exists because AI-assisted development creates new language needs:

- source code should preserve human intent
- errors should be structured for repair tools
- tests should be built into the normal workflow
- generated code should have predictable structure
- risky effects should be visible
- project layouts should be standardized
- common app libraries should be official and boring
- compilers should guide repair, not merely reject code

Existing languages can add some of these through tooling. Hayulo can design them as core assumptions.

## What success looks like

In the short term, success means:

- a newcomer can run a Hayulo program
- a contributor can understand the compiler prototype
- examples are tested
- diagnostics are clear
- the project has a coherent roadmap

In the medium term, success means:

- Hayulo can build useful scripts and small tools
- LLMs can repair common errors from JSON diagnostics
- projects have stable structure
- tests are natural to write
- the formatter makes generated code consistent

In the long term, success means:

- Hayulo can build real small apps and internal tools
- effects and permissions make generated code safer
- package metadata makes dependencies more inspectable
- users can move from idea to working prototype faster
- researchers can study AI coding reliability using Hayulo

## Human-centered software creation

Hayulo should help more people create software without hiding what the software does.

That means Hayulo should be accessible to:

- professional developers
- students
- domain experts
- teachers
- small business owners
- researchers
- community organizers
- people who understand a problem but not a modern software stack

But accessibility should not mean pretending that all software is safe to generate without review. Hayulo should lower barriers while making risks visible.

## The long-term dream

A person should be able to say:

> Build a tool for my tutoring business. It should track students, lessons, invoices, and parent messages. It should never send a message without approval.

Hayulo should help turn that into:

- intent blocks
- typed domain models
- routes or UI components
- tests
- permission declarations
- review gates
- clear diagnostics
- deployment instructions

And when the user later asks for a change, the original intent should still be visible.

## North star

> Hayulo should be the most reliable path from human intent to useful, understandable, AI-assisted software.

Every major decision should be judged against that sentence.
