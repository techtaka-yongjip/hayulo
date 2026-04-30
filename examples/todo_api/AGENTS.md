# Agent instructions for this Hayulo API example

Primary source is `main.hayulo`. Prefer editing Hayulo source instead of generated files.

Use these commands:

```bash
PYTHONPATH=../../src python -m hayulo check main.hayulo --json
PYTHONPATH=../../src python -m hayulo build main.hayulo
cd generated
npm test
```

When adding routes, also add or update record types and keep validation constraints in the Hayulo source.
