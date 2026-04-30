# Repair Benchmark Results

Hayulo repair benchmarks are small broken files with expected stable diagnostics. They are designed for humans and coding agents to run in a loop:

```bash
hayulo check <fixture> --json
```

The current public-alpha benchmark is intentionally small. It checks whether diagnostics are stable enough for an LLM repair loop to understand the problem without parsing terminal prose.

## Baseline

Baseline captured: 2026-04-30

Command:

```bash
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=src python3 -m unittest tests.test_formatter_and_repair tests.test_permissions
```

Result:

```text
OK
```

## Repair Fixtures

| Fixture | Expected code | Purpose |
| --- | --- | --- |
| `tests/repair_fixtures/unknown_name.hayulo` | `name.unknown_symbol` | Detect missing local names or functions before runtime. |
| `tests/repair_fixtures/bad_arity.hayulo` | `call.arity_mismatch` | Detect wrong function call arity before runtime. |
| `tests/repair_fixtures/bad_record_field.hayulo` | `record.unknown_field` | Detect field access on locally known records. |

## Snapshot Fixtures

| Fixture | Expected code | Purpose |
| --- | --- | --- |
| `tests/fixtures/syntax_error.hayulo` | `syntax_error` | Parser diagnostic stability. |
| `tests/fixtures/missing_file.hayulo` | `file_not_found` | Controlled file failure stability. |
| `tests/fixtures/runtime_error.hayulo` | `unknown_function` | Controlled runtime diagnostic stability. |
| `tests/fixtures/api_error.hayulo` | `api_without_routes` | API parser/build diagnostic stability. |

## Permission Fixtures

The permission tests generate temporary projects and verify:

| Scenario | Expected code |
| --- | --- |
| Invalid permission name in `hayulo.toml` | `project.invalid_permission` |
| API route requires a permission missing from `[permissions].allow` | `permission.missing` |
| API route requires a permission listed in `[permissions].deny` | `permission.denied` |

## How to Extend

When adding a repair fixture:

1. Put the broken source under `tests/repair_fixtures/` or `tests/fixtures/`.
2. Add a test that asserts the exact diagnostic code.
3. Add or update a JSON snapshot when the output envelope matters.
4. Document the fixture in this page.
5. Run `make test` and `make check`.

The benchmark should prefer small files with one clear failure. A coding agent should be able to repair each fixture with one focused edit.
