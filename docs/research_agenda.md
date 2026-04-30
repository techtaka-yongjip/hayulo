# Research Agenda

Hayulo can be both a practical language project and a research platform for AI-assisted programming.

## Core research question

> What should a programming language look like when LLMs are common code authors and repair agents?

This question touches syntax, diagnostics, type systems, testing, project layout, safety, and human-computer interaction.

## 1. LLM-friendly syntax

Questions:

- Which syntax patterns do LLMs generate most reliably?
- Are braces better than indentation for automated patching?
- How many language constructs are too many?
- Do explicit return types improve generated code correctness?
- How should examples be structured for maximum model reliability?

Possible experiments:

- Ask models to generate equivalent programs in different syntactic variants.
- Measure parse success, test success, and repair success.
- Compare concise syntax against explicit syntax.

## 2. Structured diagnostics

Questions:

- What diagnostic fields help LLMs repair code most effectively?
- Do patch hints improve success or cause unsafe overfitting?
- How stable should diagnostic codes be?
- How much source context should be included?
- Should diagnostics include examples of correct code?

Possible experiments:

- Create a benchmark of broken programs.
- Compare natural language errors, JSON diagnostics, and JSON diagnostics with patch hints.
- Measure repair rate and number of iterations.

## 3. Intent blocks

Questions:

- Do intent blocks reduce incorrect feature modifications?
- What fields are most useful: purpose, constraints, examples, non-goals, safety notes?
- How should intent blocks be validated?
- Can tests be generated from intent constraints?
- Can tools detect when code drifts from stated intent?

Possible experiments:

- Give models feature-addition tasks with and without intent blocks.
- Measure whether constraints are preserved.

## 4. Type systems for generated code

Questions:

- What type features catch the most LLM-generated bugs?
- Are `Option` and `Result` enough early on?
- How much type inference is ideal?
- Should public APIs require explicit types?
- How can type errors be explained in repairable ways?

Possible experiments:

- Compare dynamic, gradual, and static variants on generated app tasks.
- Measure runtime failure reduction and repair complexity.

## 5. Effects and permissions

Questions:

- Can effect systems make AI-generated code safer without becoming annoying?
- Which effects matter most in practical apps?
- Should effects be explicit, inferred, or both?
- Can LLMs correctly respond to missing permission diagnostics?
- How should permission expansion be reviewed?

Possible experiments:

- Generate programs with file/network/database tasks.
- Test whether models add risky behavior when not constrained.
- Measure how permission diagnostics influence patches.

## 6. Test generation and repair

Questions:

- Should Hayulo encourage tests next to code or separate test files?
- What test syntax produces best LLM-generated tests?
- Can compiler diagnostics suggest tests?
- Can intent constraints become property tests?
- How can the toolchain detect shallow or meaningless tests?

Possible experiments:

- Ask models to generate code with tests under different documentation styles.
- Measure mutation score or bug-catching ability.

## 7. Codebase summaries

Questions:

- What is the best compact representation of a Hayulo project for LLM context?
- Should summaries include intent, types, exports, effects, tests, or call graphs?
- How often should summaries be regenerated?
- Can summaries reduce context-window usage without losing correctness?

Possible experiments:

- Compare full-code prompts to summary-plus-relevant-files prompts.
- Measure edit success and hallucinated API use.

## 8. Repair safety

Questions:

- How can the toolchain prevent unsafe repairs?
- Should patches be constrained by allowed edit scope?
- How should tests and permissions be protected from deletion?
- Can the compiler detect suspicious repair patterns?

Possible experiments:

- Give models failing code where the easiest fix is unsafe.
- Measure whether policy constraints reduce unsafe patches.

## 9. Human review experience

Questions:

- How should AI-generated Hayulo patches be presented to humans?
- What explanations help reviewers trust or reject changes?
- Should diffs show intent, effects, and tests side by side?
- How much provenance metadata is useful?

Possible experiments:

- User studies comparing ordinary diffs to Hayulo-aware diffs.

## 10. Education

Questions:

- Can Hayulo help students understand generated code?
- Do intent blocks improve learning?
- How should explanations be integrated into tutorials?
- Can students learn debugging through structured diagnostics?

Possible experiments:

- Classroom exercises using broken Hayulo programs.
- Compare learning outcomes with Python/JavaScript AI-assisted tasks.

## Research artifacts

The project should eventually maintain:

- benchmark suite
- broken program corpus
- repair attempt logs
- diagnostic evaluation harness
- model comparison reports
- design notes
- papers or blog posts

## Ethical considerations

Research should avoid overstating results.

If Hayulo improves LLM repair rates on small tasks, that does not prove autonomous software creation is safe for all domains.

Reports should clearly state limitations.
