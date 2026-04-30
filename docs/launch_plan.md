# Launch Plan

Hayulo should launch as a serious but humble open-source project.

The goal of the first launch is not mass adoption. The goal is to attract thoughtful contributors, gather feedback, and show a coherent direction.

## Pre-launch checklist

Before public launch:

- README explains the mission clearly
- examples run successfully
- tests pass
- docs explain that Hayulo has a usable 2.0 draft but is not recommended for critical production systems
- roadmap is visible
- issues are organized
- license is included
- code of conduct is included
- security policy is included
- contribution guide is included
- name/domain/trademark clearance has been considered

## Launch message

The first launch should be transparent:

```text
Hayulo 2.0.0a0 is a small draft release for an AI-native programming language. It is still early, but the documented CLI, syntax subset, project format, diagnostics, formatter, and REST API generation workflow are ready for outside testing.
```

## Launch channels

Possible launch channels:

- GitHub repository
- project blog post
- Hacker News Show HN
- Reddit programming language communities
- X/LinkedIn thread
- Discord or community invite later
- AI coding researcher communities

Start with GitHub and a thoughtful post. Avoid over-marketing.

## First launch post structure

1. Problem: LLMs can write code, but generated projects are fragile.
2. Thesis: languages can be designed for generation and repair.
3. Introduction: Hayulo.
4. Demo: tiny program and `hayulo check --json`.
5. Roadmap: parser, formatter, static checker, repair protocol.
6. Invitation: feedback, contributors, examples.
7. Honesty: small stable core, not recommended for critical production systems.

## Demo script

```bash
git clone <repo>
cd hayulo-lang
python -m venv .venv
. .venv/bin/activate
pip install -e .
hayulo run examples/hello.hayulo
hayulo test examples/hello.hayulo
hayulo check examples/hello.hayulo --json
```

## First issues to create

Create clear issues before launch:

- Add `hayulo --version`
- Add parser source spans
- Add diagnostic snapshot tests
- Add list literals
- Add map literals
- Add record syntax design
- Draft formatter rules
- Write first tutorial
- Create first repair benchmark
- Add design record for braces

## First release

Suggested tag:

```text
v0.1.0-seed
```

Release notes should include:

- what works
- what does not work
- docs added
- examples included
- roadmap
- invitation for feedback

## Launch tone

Use:

- experimental
- seed prototype
- exploring
- designed for
- early feedback wanted

Avoid:

- production-ready
- replaces Python
- AI builds anything
- no-code for all software
- solved programming

## Success criteria

A good first launch means:

- people understand the idea
- a few people clone and run it
- useful issues are opened
- contributors ask design questions
- feedback improves the roadmap

It does not require viral popularity.
