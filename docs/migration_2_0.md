# Hayulo 2.0 Draft Migration Guide

Hayulo `2.0.0a0` is intentionally breaking. The goal is to make generated programs easier for LLMs to write, check, repair, and review. This guide covers the 1.x-style syntax most likely to appear in older examples or agent output.

Run this after each migration step:

```bash
hayulo format --check .
hayulo check --json
hayulo test
hayulo benchmark llm --json
```

For API files, also run:

```bash
hayulo build path/to/main.hayulo --json
cd path/to/generated
npm test
```

## Bindings

Old:

```hayulo
total = 0
total = total + score
```

New:

```hayulo
let total = 0
set total = total + score
```

Use `let` exactly once per binding in a scope. Use `set` only after a name has already been bound.

Diagnostic:

```text
syntax.binding_requires_let_or_set
```

## Missing Values and Errors

Old:

```hayulo
user = find_user(id)?
return user.name
```

New:

```hayulo
let user = try find_user(id)
return user.name
```

Use `Option<T>` for missing values and `Result<T, E>` for recoverable errors.

```hayulo
fn find_user(id: Int) -> Option<User> {
  if id == 1 {
    return Some(User { name: "Ada" })
  }
  return None
}

fn user_name(id: Int) -> Result<Text, Text> {
  let user = try find_user(id)
  return Ok(user.name)
}
```

Use statement-form `match` when both branches should be handled locally:

```hayulo
match result {
  Ok(value) => {
    print(value)
  }
  Err(error) => {
    print(error)
  }
}
```

Diagnostics:

```text
syntax.postfix_try_removed
type.invalid_try_target
type.try_return_mismatch
match.non_exhaustive
```

## API Field Constraints

Old:

```hayulo
title: Text min 1 max 200
internal_notes: Text private
```

New:

```hayulo
title: Text { min: 1, max: 200 }
internal_notes: Text { private: true }
```

Constraint blocks use `key: value` entries. Supported keys are `min`, `max`, `unique`, and `private`.

Diagnostic:

```text
api.inline_constraints_removed
```

## API Route Bodies

Old:

```hayulo
route POST "/todos" body input: CreateTodo -> Todo {
  return db.Todo.insert(Todo { title: input.title })
}
```

New:

```hayulo
route POST "/todos" body input: CreateTodo -> Todo {
  effect api.write
  effect storage.local
  action create Todo from input
}
```

Routes now declare behavior instead of embedding database calls. Every route body contains zero or more `effect` lines and exactly one `action`.

Supported actions:

```text
action list Record
action get Record by id
action create Record from input
action update Record by id from input
action update Record by id set { field: value }
action delete Record by id
```

Diagnostics:

```text
route.body_requires_action
route.missing_action
route.missing_effect
route.invalid_action
```

## Permission Policy

Route effects must be allowed by `hayulo.toml`.

```toml
[permissions]
allow = ["api.read", "api.write", "api.delete", "storage.local"]
deny = []
```

Use the smallest allow-list that matches the generated API behavior. `deny` wins over `allow`.

## Recommended Repair Prompt

When asking an LLM to migrate source, provide this instruction:

```text
Migrate this Hayulo file to 2.0 draft syntax. Use let for new bindings, set for reassignment, try instead of postfix ?, structured field constraints, and route effect/action bodies. Preserve behavior and tests. Run hayulo check --json after editing and repair only the reported diagnostics.
```

## Verification Checklist

- `hayulo --version` reports `hayulo 2.0.0a0`
- no bare `name = value` remains in script code
- no postfix `?` remains
- API fields use structured constraint blocks
- API routes use `effect` and `action`
- `make benchmark` builds every Hayulo benchmark baseline
- `make verify` passes before committing
