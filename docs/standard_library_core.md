# Standard Library Core

Hayulo 1.0 has a deliberately small standard library surface.

## Script Built-ins

Stable built-ins:

```hayulo
print(value)
len(value)
```

### `print(value)`

Writes a value to the program output.

Current behavior:

- values are converted to text by the interpreter
- multiple arguments are joined with spaces
- `hayulo run --json` returns captured output in `output`
- script tests capture output but do not require assertions on output yet

### `len(value)`

Returns the length of a supported value.

Supported targets:

- `Text`
- `List`
- `Map`

Stable diagnostics:

- `arity_mismatch` when called with the wrong number of arguments
- `invalid_len_target` when called with an unsupported value

## Stable Core Types

Stable type names accepted by annotations and API generation:

- `Text`
- `Int`
- `Float`
- `Bool`
- `Time`
- `Email`
- `Status`
- `Any`
- `List<T>`
- `Id<T>`

Script annotations are checked by the static checker where the local preview can infer enough information. API types are checked before generation.

## API Runtime Helpers

Generated API servers expose stable behavior rather than a public Hayulo standard library module:

- `GET /health`
- `GET /openapi.json`
- CRUD-like behavior inferred from routes
- generated validation for request bodies
- generated local JSON store
- generated smoke test

These generated helpers are part of the 1.0 app-building contract, but the generated JavaScript internals are not a stable API for hand editing.

## Not in the 1.0 Core

Not included:

- file system standard library
- network standard library
- database standard library
- time library beyond generated `now()` defaults in API records
- package manager
- effects-aware standard library APIs
- AI model APIs

These should be added only after the effects and permissions model can describe their behavior clearly.
