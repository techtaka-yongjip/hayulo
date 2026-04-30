# Editor Support

Public alpha editor support is intentionally minimal.

The repository includes a TextMate grammar preview at:

```text
editors/hayulo.tmLanguage.json
```

This grammar is suitable for early syntax highlighting experiments in editors that support TextMate grammars. It is not a language server.

## Current Highlighting Scope

The grammar highlights:

- comments
- strings
- numbers
- keywords such as `module`, `intent`, `fn`, `test`, `expect`, `app`, `route`, and `record`
- built-in types such as `Text`, `Int`, `Bool`, `List`, and `Id`
- route methods such as `GET`, `POST`, `PATCH`, and `DELETE`
- function names after `fn`
- type names after `type`

## Not Yet Implemented

- language server protocol
- go to definition
- hover docs
- diagnostics integration
- format on save
- code actions
- semantic highlighting

## VS Code Experiment

For a local experiment, create a small VS Code extension that contributes `editors/hayulo.tmLanguage.json` for `*.hayulo` files. This repo does not yet publish an editor package.

Future work should add a proper extension package once the syntax subset is closer to 1.0.
