# Technical Plan

This document describes how Hayulo can grow from the current seed interpreter into a real programming language implementation.

The plan favors incremental progress. Each phase should produce a usable artifact.

## Current state

The seed prototype includes:

- lexer
- parser
- abstract syntax tree
- tree-walking interpreter
- command-line interface
- basic tests
- JSON diagnostics
- examples

The prototype intentionally supports a small subset of the intended language.

## Desired architecture

A mature Hayulo implementation should have these layers:

```text
Source files (.hayulo)
  -> Lexer
  -> Parser
  -> AST
  -> Name resolver
  -> Type checker
  -> Effect checker
  -> Lowering to Hayulo IR
  -> Optimizer / analyzer
  -> Interpreter or compiler backend
  -> Runtime / standard library
  -> Diagnostics and repair protocol
```

The exact implementation can change, but the separation of concerns matters.

## Phase 1: Stabilize the front end

Goals:

- make parsing predictable
- improve error messages
- define the grammar clearly
- add tests for valid and invalid syntax

Work items:

- add parser recovery after common syntax errors
- add source spans to AST nodes
- add diagnostic codes for parser errors
- add snapshot tests for diagnostics
- write grammar details in `SPEC.md`
- add more examples

Why this matters: LLMs need stable syntax and clear feedback. A weak parser makes every later feature harder.

## Phase 2: Useful data structures

Goals:

- list literals
- map literals
- indexing
- record declarations
- record construction
- field access
- simple functions over records

Example target:

```hayulo
type User = record {
  name: Text
  age: Int
}

fn main() {
  let users = [
    User { name: "Ada", age: 36 },
    User { name: "Grace", age: 85 }
  ]

  print(users[0].name)
}
```

## Phase 3: Control flow and errors

Goals:

- `for item in items { ... }`
- `match`
- `Option<T>`
- `Result<T, E>`
- prefix `try` for early error return
- standard error type conventions

Example target:

```hayulo
fn find_user(id: UserId) -> Option<User> {
  return users.find(id)
}

fn show_user(id: UserId) -> Result<Text, AppError> {
  let user = try find_user(id)
  return Ok(user.name)
}
```

## Phase 4: Static checker

Goals:

- name resolution
- type inference for local variables
- explicit types for public APIs
- function call arity checks
- field existence checks
- return type checks
- unreachable code warnings
- unused variable warnings
- `Option` exhaustiveness checks
- `Result` handling checks

Type checker philosophy: Hayulo should be statically checked but not verbose. Local inference should reduce boilerplate, while public APIs remain explicit.

## Phase 5: Hayulo IR

An intermediate representation can support:

- better analysis
- optimization
- multiple backends
- code generation
- precise diagnostics
- effect checking
- test instrumentation

The first IR can be simple:

- modules
- functions
- basic blocks
- expressions
- calls
- branches
- returns
- type annotations
- source spans

Do not over-engineer the IR early.

## Phase 6: Standard library foundation

Initial modules:

- `std.text`
- `std.list`
- `std.map`
- `std.json`
- `std.files`
- `std.cli`
- `std.test`
- `std.log`
- `std.time`

Every standard library function should have:

- documentation
- examples
- tests
- stable errors
- clear effects if applicable

## Phase 7: Project system

Goals:

- `hayulo.toml`
- module imports
- package metadata
- project-level permissions
- test discovery
- source folder conventions
- dependency declaration

Example:

```toml
name = "todo_app"
version = "0.1.0"
language = "hayulo@0.1"

[app]
entry = "src/main.hayulo"

[permissions]
allow = ["files.read", "network.read"]
deny = ["files.delete", "money.spend"]
```

## Phase 8: Compiler backend

Hayulo can start with an interpreter but should eventually compile to another target.

Candidate backends:

1. TypeScript: best early target for web and Node ecosystems.
2. Python: easy interoperability and fast prototyping.
3. WebAssembly: sandboxing and portability.
4. LLVM/native: long-term performance.

Recommended path: start with TypeScript or Python code generation after the interpreter is stable.

## Phase 9: Effects and permissions

Goals:

- effect annotations
- effect inference
- project permission policy
- runtime permission checks
- sandbox adapters
- approval gates

Example:

```hayulo
fn write_report(path: Path, content: Text) -> Result<(), FileError>
  effects [files.write]
{
  return files.write_text(path, content)
}
```

The checker should ensure the project allows `files.write`.

## Phase 10: AI repair protocol

Goals:

- `hayulo check --json`
- stable diagnostic schema
- patch suggestion schema
- codebase summary command
- test failure schema
- repair session log
- eventually `hayulo repair` adapter interface

Example command:

```bash
hayulo check --json --repair-hints
```

Example output:

```json
{
  "status": "failed",
  "diagnostics": [
    {
      "code": "unknown_field",
      "message": "Type User has no field 'emial'.",
      "location": {
        "file": "src/users.hayulo",
        "line": 18,
        "column": 12
      },
      "suggestions": [
        {
          "kind": "rename_field",
          "from": "emial",
          "to": "email",
          "confidence": 0.97
        }
      ]
    }
  ]
}
```

## Testing strategy

Hayulo should test itself at multiple levels:

- lexer unit tests
- parser unit tests
- interpreter tests
- type checker tests
- diagnostic snapshot tests
- example tests
- end-to-end CLI tests
- repair-loop tests

The examples should be part of the test suite.

## Implementation language

The current prototype is in Python, which is appropriate for speed of iteration.

Future options:

- keep Python for prototype and tooling
- rewrite compiler core in Rust for performance
- maintain Python reference implementation
- generate parts of compiler from grammar

A rewrite should not happen until the language design is more stable.

## Technical risks

### Too many features too early

Mitigation: maintain a strict roadmap and require examples/tests for new syntax.

### Weak diagnostics

Mitigation: treat diagnostics as product features, not internal errors.

### Poor LLM reliability

Mitigation: benchmark repair loops and adjust language design based on evidence.

### Ecosystem gap

Mitigation: start with batteries-included standard library and practical interop.
