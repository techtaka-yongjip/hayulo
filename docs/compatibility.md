# Compatibility Policy

Hayulo 1.0 stabilizes the documented core contract for the 1.x line.

## Versioning

Hayulo uses semantic versioning for public contracts:

- patch releases fix bugs without changing documented behavior
- minor releases add backward-compatible syntax, CLI flags, diagnostics, or generated output
- major releases may remove or change documented behavior after a migration plan

## Stable in 1.x

The following are stable within 1.x:

- CLI commands listed in [stable_contract_1_0.md](stable_contract_1_0.md)
- documented script and API syntax subset
- `hayulo.toml` project fields
- formatter behavior
- JSON diagnostic envelope `hayulo.diagnostics@0.1`
- JSON test envelope `hayulo.test@0.1`
- generated REST API file names
- generated OpenAPI and smoke-test workflow
- project permission names used by generated API checks

## Allowed Patch Changes

Patch releases may:

- improve diagnostic messages without changing stable codes
- fix incorrect line or column reporting
- fix runtime bugs
- improve generated code while preserving generated file names and command workflow
- add tests and docs

## Allowed Minor Changes

Minor releases may:

- add new CLI flags if existing command behavior remains valid
- add new diagnostic codes
- add optional JSON fields
- add syntax that does not change existing parsing
- add generated files if existing generated files remain
- add project config fields with defaults

## Breaking Changes

Breaking changes require a major version or an explicitly documented migration exception. Examples:

- removing or renaming stable CLI commands
- changing formatter output for already supported syntax
- renaming stable diagnostic codes
- removing compatibility JSON fields
- changing `hayulo.toml` field meaning
- changing generated file names used by the documented API workflow
- changing permission names required by existing generated API behavior

## Experimental Areas

The following are not covered by 1.x compatibility until explicitly stabilized:

- future language-level effects syntax
- package manager design
- language server protocol
- TypeScript generation
- deployment targets
- auth enforcement in generated servers
- undocumented internal Python APIs
