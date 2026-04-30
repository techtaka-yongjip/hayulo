# Standard Library Plan

Hayulo needs a strong standard library because the language is designed for LLM-assisted building. Every missing basic library increases dependency chaos and makes generated projects less reliable.

The standard library should not be huge at first, but it should provide official paths for common tasks.

## Standard library principles

### Boring and predictable

The standard library should prefer obvious APIs over clever abstractions.

### Typed errors

Functions that can fail should return `Result` with documented error types.

### Explicit effects

APIs that touch files, network, databases, environment variables, clocks, or external systems should declare effects.

### Examples for every module

Every standard module should have examples that LLMs can copy safely.

### Stable names

Renaming standard functions breaks generated examples and repair tools. Names should be chosen carefully.

## Core modules

### `std.text`

Text manipulation:

- trim
- split
- replace
- contains
- starts_with
- ends_with
- lines
- words
- parse numbers
- formatting helpers

### `std.list`

List operations:

- map
- filter
- reduce/fold
- find
- sort
- group_by
- length
- is_empty

Keep functional helpers readable and avoid overly clever chaining early.

### `std.map`

Map/dictionary operations:

- get
- set
- contains
- keys
- values
- merge

### `std.result`

Helpers for `Result<T, E>`:

- Ok
- Err
- map
- map_err
- unwrap_or
- ok_or

### `std.option`

Helpers for `Option<T>`:

- Some
- None
- map
- unwrap_or
- ok_or

### `std.json`

JSON support:

- parse
- stringify
- typed decode
- typed encode
- schema validation later

Example:

```hayulo
type User = record {
  name: Text
  age: Int
}

let user = try json.decode<User>(text)
```

### `std.files`

File operations:

- read_text
- write_text
- read_bytes
- write_bytes
- exists
- list_dir
- create_dir
- remove only with explicit effect

Effects:

- `files.read`
- `files.write`
- `files.delete`

### `std.path`

Path utilities:

- join
- extension
- file_name
- parent
- normalize
- relative_to

### `std.cli`

CLI app support:

- arguments
- flags
- commands
- help output
- exit codes
- structured errors

Example future design:

```hayulo
command wordcount {
  arg path: Path

  run {
    let text = try files.read_text(path)
    print(text.split_whitespace().length)
  }
}
```

### `std.time`

Time and date:

- current time
- date parsing
- formatting
- durations
- timezone-aware types eventually

Effect: `clock.read`.

### `std.log`

Logging:

- debug
- info
- warn
- error
- structured fields

### `std.env`

Environment variables and configuration:

- read env var
- parse config
- secrets handling guidelines

Effect: `env.read`.

## App-building modules

### `std.http`

HTTP client and server basics:

- request
- response
- routes
- middleware later
- typed JSON bodies
- typed path params

Effects:

- `network.read`
- `network.write`

### `std.web`

Higher-level web API support:

```hayulo
route GET "/todos" -> List<Todo> {
  return todos.all()
}
```

This may be part of the language or a library. The design should remain explicit.

### `std.sqlite`

SQLite support is important because it enables useful local apps without infrastructure.

Features:

- open database
- migrations
- typed queries eventually
- transactions
- simple table mapping

Effects:

- `db.read`
- `db.write`

### `std.validation`

Input validation:

- required fields
- min/max length
- ranges
- email validation
- URL validation
- custom validators

### `std.auth`

Authentication is hard and should not be rushed.

Initial goal:

- simple user identity model
- password hashing wrapper
- session helpers
- clear security documentation

Avoid building insecure toy auth that people copy into production.

## AI modules

Hayulo's standard library can include AI support, but it must be carefully designed.

### `std.ai`

Potential features:

- text generation
- structured extraction
- classification
- embeddings
- tool calling wrappers
- confidence/evidence containers

Example:

```hayulo
type Sentiment = enum { positive, neutral, negative }

fn classify(text: Text) -> Result<Belief<Sentiment>, AiError> {
  return ai.classify<Sentiment> {
    instruction: "Classify sentiment."
    input: text
  }
}
```

Important design rule: model output should not pretend to be verified truth.

## Testing modules

### `std.test`

Testing should include:

- expect
- expect_err
- expect_some
- fixtures
- temp directories
- test data
- property tests later

Example:

```hayulo
test "parse valid config" {
  let config = try Config.parse("name = app")
  expect config.name == "app"
}
```

## Security-related modules

### `std.crypto`

Crypto APIs should be high-level and safe.

Avoid exposing low-level primitives casually.

Initial APIs:

- secure random IDs
- hashing for non-password use
- password hashing wrapper
- HMAC wrapper

### `std.secrets`

Secret management:

- read secret
- avoid accidental logging
- redact output

## Documentation requirements

Every standard library module should include:

- overview
- examples
- type signatures
- error behavior
- effects
- permissions
- security notes
- tests
