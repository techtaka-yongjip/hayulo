# Migration Policy

Hayulo should make breaking changes deliberately and visibly. This policy applies after the 1.0 stable core.

For the active 2.0 draft migration, see [migration_2_0.md](migration_2_0.md).

## Migration Requirements

Every breaking change must include:

- the old behavior
- the new behavior
- why the change is needed
- affected files or commands
- a migration example
- diagnostics or warnings where practical
- tests for old failure and new success paths

## Deprecation Flow

Preferred flow:

1. Add the new behavior behind a backward-compatible path.
2. Document the old behavior as deprecated.
3. Emit a diagnostic or warning where practical.
4. Keep the old behavior for at least one minor release.
5. Remove the old behavior only in a major release.

## Migration Notes Format

Use this shape in release notes:

````markdown
### Breaking: route permission rename

Old:
`api.write`

New:
`api.mutate`

Why:
The new name distinguishes storage mutation from write-only transport effects.

Migration:
Update `hayulo.toml`:

```toml
[permissions]
allow = ["api.read", "api.mutate", "api.delete", "storage.local"]
```

Validation:
Run `hayulo check --json` and `make test`.
````

## Automated Help

When possible, Hayulo should provide:

- stable diagnostics that point to the obsolete construct
- clear suggestions for replacement syntax
- `hayulo summarize --json` context to help coding agents repair projects
- fixture tests for common migrations

## Emergency Exceptions

Security issues may require faster breaking changes. In that case, the release note must explain the risk, the mitigation, and the fastest safe migration path.
