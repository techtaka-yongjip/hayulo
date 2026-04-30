# Contributing to Hayulo

Hayulo is an experimental open-source language for AI-assisted software creation.

Contributions are welcome, especially clear design critiques, small implementation improvements, examples, tests, and diagnostics.

## Development setup

```bash
python -m venv .venv
. .venv/bin/activate
pip install -e .
python -m unittest discover -s tests
```

Run examples:

```bash
hayulo run examples/hello.hayulo
hayulo test examples/hello.hayulo
```

## Contribution principles

- Keep the language small and predictable.
- Prefer one obvious way to do common things.
- Optimize for human readability and LLM repairability.
- Add tests for parser, interpreter, and diagnostics changes.
- Make errors more helpful, not just more precise.
- Avoid adding complex features before the core is stable.

## Good first issues

- Add parser tests for invalid syntax.
- Improve diagnostic suggestions.
- Add lists.
- Add `for` loops.
- Add `match`.
- Add a formatter.
- Add examples of CLI tools.

## Pull request checklist

- Tests pass.
- New behavior has tests.
- User-facing behavior is documented.
- Diagnostics include clear messages and suggestions.
