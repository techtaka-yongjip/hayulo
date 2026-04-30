# Project System

Hayulo supports a small project layout while keeping single-file commands working.

## Layout

```text
my-app/
  hayulo.toml
  src/
    main.hayulo
  tests/
    main_test.hayulo
```

`hayulo.toml` defines the project:

```toml
[project]
name = "my-app"
version = "0.1.0"
src = "src"
tests = "tests"
```

The supported project fields are:

- `name`: project name
- `version`: project version
- `src`: source directory path or array of paths
- `tests`: test directory path or array of paths
- `exclude`: optional array of files or directories to skip

## Creating a Project

```bash
hayulo new my-app
cd my-app
hayulo check
hayulo test
hayulo run src/main.hayulo
```

`hayulo new` refuses to overwrite a non-empty directory.

## Project-Wide Commands

Inside a project:

```bash
hayulo check
hayulo test
```

Or pass a project directory:

```bash
hayulo check path/to/project
hayulo test path/to/project
```

`hayulo check` parses and validates all discovered `.hayulo` files under `src` and `tests`. API files are checked as API sources. Script files also run the static checker.

`hayulo test` runs script tests from discovered script files. API files are skipped by the project test runner because API smoke tests are generated and run through `hayulo build` output.

## Single-File Compatibility

Single-file mode remains supported:

```bash
hayulo check examples/hello.hayulo
hayulo test examples/hello.hayulo
hayulo run examples/hello.hayulo
hayulo build examples/todo_api/main.hayulo
```
