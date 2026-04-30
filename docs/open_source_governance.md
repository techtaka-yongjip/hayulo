# Open Source Governance

Hayulo should be developed as a serious open-source project. Good governance matters because programming languages require long-term trust.

Contributors need to know how decisions are made, how disagreements are handled, and how the project avoids drifting away from its mission.

## Governance goals

Hayulo governance should be:

- transparent
- lightweight early
- documented
- welcoming to contributors
- protective of project quality
- resistant to hype-driven design
- respectful of users and maintainers

## Project roles

### Users

People who try Hayulo, report issues, ask questions, and build examples.

### Contributors

People who submit code, docs, tests, examples, design critiques, or tooling.

### Maintainers

People trusted to review and merge changes in one or more areas.

### Core maintainers

People responsible for final decisions on language design, releases, and project direction.

### Steering group later

If the project grows, a small steering group can handle roadmap and governance decisions.

## Early governance model

At the beginning, Hayulo should use a maintainer-led model with documented decision making.

This is appropriate because the project is young and needs coherence.

However, design decisions should be written down so the project can evolve toward broader governance later.

## Decision records

Important decisions should be captured as Hayulo Design Records, or HDRs.

Directory:

```text
docs/design_records/
```

Format:

```md
# HDR 0001: Use braces for blocks

Status: Accepted
Date: YYYY-MM-DD

## Context

## Decision

## Consequences

## Alternatives considered
```

Design records should be used for:

- syntax choices
- type system decisions
- package manager decisions
- standard library policy
- governance changes
- major toolchain architecture

## Proposal process

For small changes:

1. Open issue.
2. Discuss briefly.
3. Submit PR.
4. Maintainer reviews.
5. Merge if aligned.

For major changes:

1. Open design discussion.
2. Write an HDR draft.
3. Gather feedback.
4. Prototype if possible.
5. Decide explicitly.
6. Update docs and tests.

## Contribution standards

Contributions should generally include:

- tests
- documentation updates
- diagnostic updates if behavior changes
- examples for new user-facing features
- clear explanation of why the change fits Hayulo

## Language feature bar

A proposed language feature should satisfy:

- clear use case
- simple explanation
- useful diagnostics
- formatter support
- test coverage
- LLM generation/repair impact considered
- interaction with types/effects considered

Features should not be added only because another language has them.

## Maintainer responsibilities

Maintainers should:

- be respectful and constructive
- review contributions in good faith
- explain rejections clearly
- protect project scope
- avoid merging underdocumented features
- keep examples working
- prioritize user trust

## Community standards

Hayulo should be welcoming to:

- professional developers
- students
- language designers
- AI researchers
- educators
- beginners
- people from different countries and backgrounds

Disagreement is expected. Disrespect is not.

## Release process

Early releases can be lightweight:

```bash
git tag v0.1.0-seed
git push origin v0.1.0-seed
```

Each release should include:

- changelog
- compatibility notes
- known limitations
- examples tested
- roadmap update

## Versioning

Before 1.0, breaking changes are expected. Still, they should be documented.

Suggested scheme:

```text
0.1 seed prototype
0.2 useful scripting
0.3 static checking
0.4 project system
0.5 app-building preview
1.0 stable core
```

## Trademark and naming note

Hayulo's name should be treated as a project identity. Before broad launch or foundation formation, the project should conduct proper name, domain, and trademark clearance.

Open-source licensing does not automatically solve branding questions.

## Sustainability

If Hayulo grows, sustainability may require:

- donations
- sponsorship
- grants
- hosted services
- foundation structure
- commercial support around open core tools

The language core should remain open-source.
