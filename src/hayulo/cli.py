from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

from . import __version__
from .api import generate_api, looks_like_api_source, parse_api_source
from .checker import check_program
from .diagnostics import TEST_SCHEMA, Diagnostic, HayuloError, diagnostic_failure_payload
from .formatter import check_format
from .intent import parse_top_level_intent
from .interpreter import Interpreter
from .lexer import Lexer
from .parser import Parser
from .project import ProjectConfig, load_project, project_files, project_name_to_module


def read_source(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        raise HayuloError(
            Diagnostic(
                code="file_not_found",
                message=f"Hayulo source file not found: {path}.",
                file=str(path),
                suggestions=["Check the path and try again."],
            )
        ) from None
    except UnicodeDecodeError as exc:
        raise HayuloError(
            Diagnostic(
                code="invalid_source_encoding",
                message="Hayulo source files must be valid UTF-8.",
                file=str(path),
                details={"encoding": "utf-8"},
                suggestions=["Save the file as UTF-8 and try again."],
            )
        ) from exc
    except OSError as exc:
        details: dict[str, Any] = {}
        if exc.errno is not None:
            details["errno"] = exc.errno
        raise HayuloError(
            Diagnostic(
                code="file_read_failed",
                message=f"Could not read Hayulo source file: {exc.strerror or exc}.",
                file=str(path),
                details=details,
                suggestions=["Check file permissions and try again."],
            )
        ) from exc


def load_program(path: Path, source: str | None = None, filename: str | None = None):
    if source is None:
        source = read_source(path)
    filename = filename or str(path)
    tokens = Lexer(source, filename=filename).lex()
    return Parser(tokens, filename=filename).parse()


def emit_json(payload: dict[str, Any]) -> None:
    print(json.dumps(payload, indent=2, sort_keys=True))


def handle_error(error: HayuloError, json_mode: bool) -> int:
    return handle_errors([error], json_mode)


def handle_errors(errors: list[HayuloError], json_mode: bool) -> int:
    if json_mode:
        emit_json(diagnostic_failure_payload(errors))
    else:
        for error in errors:
            d = error.diagnostic
            location = ""
            if d.file:
                location += d.file
            if d.line is not None:
                location += f":{d.line}"
            if d.column is not None:
                location += f":{d.column}"
            prefix = f"{location}: " if location else ""
            print(f"{prefix}{d.code}: {d.message}", file=sys.stderr)
            for suggestion in d.suggestions:
                print(f"  hint: {suggestion}", file=sys.stderr)
    return 1


def test_json_payload(
    *,
    status: str,
    file: str | None,
    passed: int,
    failed: int,
    tests: list[dict[str, Any]],
    output: list[str],
    extra: dict[str, Any] | None = None,
) -> dict[str, Any]:
    failures = [
        {
            "test": result["name"],
            "file": file,
            "line": result.get("line"),
            "message": result.get("error", "Test failed."),
        }
        for result in tests
        if not result.get("passed")
    ]
    payload: dict[str, Any] = {
        "schema": TEST_SCHEMA,
        "status": status,
        "summary": {"passed": passed, "failed": failed},
        "failures": failures,
        "passed": passed,
        "failed": failed,
        "tests": tests,
        "output": output,
    }
    if file is not None:
        payload["file"] = file
    if extra:
        payload.update(extra)
    return payload


def check_file_payload(path: Path, *, filename: str | None = None) -> dict[str, Any]:
    filename = filename or str(path)
    source = read_source(path)
    intent = parse_top_level_intent(source, filename=filename)
    if looks_like_api_source(source):
        spec = parse_api_source(source, filename=filename)
        return {
            "status": "ok",
            "kind": "api",
            "file": filename,
            "module": spec.module,
            "intent": intent,
            "app": spec.app_name,
            "database": spec.database.to_dict() if spec.database else None,
            "records": sorted(spec.records.keys()),
            "routes": [route.to_dict() for route in spec.routes],
        }

    program = load_program(path, source, filename=filename)
    check_program(program, filename=filename)
    return {
        "status": "ok",
        "kind": "script",
        "file": filename,
        "module": program.module,
        "intent": intent,
        "functions": sorted(program.functions.keys()),
        "tests": [test.name for test in program.tests],
    }


def cmd_check(args: argparse.Namespace) -> int:
    path = Path(args.target) if args.target else Path(".")
    if args.target is None or path.is_dir():
        return cmd_check_project(path, args.json)

    try:
        payload = check_file_payload(path)
    except HayuloError as error:
        return handle_error(error, args.json)

    if args.json:
        emit_json(payload)
    else:
        print(f"ok: {path}")
        if payload["kind"] == "api":
            print(f"app: {payload['app']}")
            print(f"records: {', '.join(payload['records']) or '(none)'}")
            print(f"routes: {len(payload['routes'])}")
        else:
            print(f"functions: {', '.join(payload['functions']) or '(none)'}")
            print(f"tests: {len(payload['tests'])}")
    return 0


def cmd_check_project(target: Path, json_mode: bool) -> int:
    try:
        config = load_project(target)
        checked: list[dict[str, Any]] = []
        errors: list[HayuloError] = []
        for path in project_files(config):
            filename = project_relative(config, path)
            try:
                checked.append(check_file_payload(path, filename=filename))
            except HayuloError as error:
                errors.append(error)
        if errors:
            return handle_errors(errors, json_mode)
    except HayuloError as error:
        return handle_error(error, json_mode)

    payload = {
        "status": "ok",
        "kind": "project",
        "root": str(config.root),
        "config": str(config.config_path),
        "project": {"name": config.name, "version": config.version},
        "checked": len(checked),
        "files": checked,
    }
    if json_mode:
        emit_json(payload)
    else:
        print(f"ok: {config.name} ({len(checked)} files)")
        for item in checked:
            print(f"  {item['file']}: {item['kind']}")
    return 0


def cmd_run(args: argparse.Namespace) -> int:
    path = Path(args.file)
    try:
        program = load_program(path)
        interpreter = Interpreter(program, filename=str(path))
        result = interpreter.run_main()
    except HayuloError as error:
        return handle_error(error, args.json)

    if args.json:
        emit_json({"status": "ok", "file": str(path), "output": interpreter.output, "result": result})
    else:
        for line in interpreter.output:
            print(line)
    return 0


def cmd_test(args: argparse.Namespace) -> int:
    path = Path(args.target) if args.target else Path(".")
    if args.target is None or path.is_dir():
        return cmd_test_project(path, args.json)

    try:
        program = load_program(path)
        interpreter = Interpreter(program, filename=str(path))
        results = interpreter.run_tests()
    except HayuloError as error:
        return handle_error(error, args.json)

    passed = sum(1 for result in results if result.passed)
    failed = len(results) - passed
    status = "ok" if failed == 0 else "failed"

    if args.json:
        emit_json(test_json_payload(status=status, file=str(path), passed=passed, failed=failed, tests=[result.to_dict() for result in results], output=interpreter.output))
    else:
        for result in results:
            marker = "PASS" if result.passed else "FAIL"
            print(f"{marker} {result.name}")
            if result.error:
                print(f"  {result.error}")
        print(f"{passed} passed, {failed} failed")
    return 0 if failed == 0 else 1


def test_file_payload(path: Path, *, filename: str | None = None) -> dict[str, Any]:
    filename = filename or str(path)
    program = load_program(path, filename=filename)
    check_program(program, filename=filename)
    interpreter = Interpreter(program, filename=filename)
    results = interpreter.run_tests()
    passed = sum(1 for result in results if result.passed)
    failed = len(results) - passed
    return test_json_payload(status="ok" if failed == 0 else "failed", file=filename, passed=passed, failed=failed, tests=[result.to_dict() for result in results], output=interpreter.output)


def cmd_test_project(target: Path, json_mode: bool) -> int:
    try:
        config = load_project(target)
        files: list[dict[str, Any]] = []
        errors: list[HayuloError] = []
        for path in project_files(config):
            source = read_source(path)
            if looks_like_api_source(source):
                continue
            filename = project_relative(config, path)
            try:
                files.append(test_file_payload(path, filename=filename))
            except HayuloError as error:
                errors.append(error)
        if errors:
            return handle_errors(errors, json_mode)
    except HayuloError as error:
        return handle_error(error, json_mode)

    passed = sum(file["passed"] for file in files)
    failed = sum(file["failed"] for file in files)
    payload = {
        "schema": TEST_SCHEMA,
        "status": "ok" if failed == 0 else "failed",
        "kind": "project-test",
        "root": str(config.root),
        "config": str(config.config_path),
        "project": {"name": config.name, "version": config.version},
        "summary": {"passed": passed, "failed": failed},
        "failures": [failure for file in files for failure in file["failures"]],
        "passed": passed,
        "failed": failed,
        "files": files,
    }
    if json_mode:
        emit_json(payload)
    else:
        for file in files:
            print(f"{file['file']}: {file['passed']} passed, {file['failed']} failed")
            for result in file["tests"]:
                marker = "PASS" if result["passed"] else "FAIL"
                print(f"  {marker} {result['name']}")
                if result.get("error"):
                    print(f"    {result['error']}")
        print(f"{passed} passed, {failed} failed")
    return 0 if failed == 0 else 1


def cmd_build(args: argparse.Namespace) -> int:
    path = Path(args.file)
    out_dir = Path(args.out) if args.out else path.parent / "generated"
    try:
        source = read_source(path)
        spec = parse_api_source(source, filename=str(path))
        files = generate_api(spec, out_dir, clean=not args.no_clean)
    except HayuloError as error:
        return handle_error(error, args.json)

    payload = {
        "status": "ok",
        "kind": "api-build",
        "file": str(path),
        "app": spec.app_name,
        "output_dir": str(out_dir),
        "generated": [file.to_dict() for file in files],
        "next_commands": [f"cd {out_dir}", "npm test", "npm start"],
    }
    if args.json:
        emit_json(payload)
    else:
        print(f"built {spec.app_name} -> {out_dir}")
        for file in files:
            print(f"  {file.path.name}: {file.description}")
        print("next:")
        print(f"  cd {out_dir}")
        print("  npm test")
        print("  npm start")
    return 0


def cmd_format(args: argparse.Namespace) -> int:
    try:
        config, files = format_file_targets(Path(args.target) if args.target else Path("."))
        results: list[dict[str, Any]] = []
        changed: list[Path] = []
        for path in files:
            source = read_source(path)
            result = check_format(source)
            label = project_relative(config, path) if config else str(path)
            if result.changed:
                changed.append(path)
                if not args.check:
                    write_source(path, result.source)
            results.append({"file": label, "changed": result.changed})

        if args.check and changed:
            labels = [project_relative(config, path) if config else str(path) for path in changed]
            raise HayuloError(
                Diagnostic(
                    code="format.required",
                    message="Hayulo source is not formatted.",
                    file=labels[0],
                    details={"files": labels},
                    suggestions=["Run hayulo format on the target."],
                )
            )
    except HayuloError as error:
        return handle_error(error, args.json)

    payload = {
        "status": "ok",
        "kind": "format",
        "mode": "check" if args.check else "write",
        "checked": len(results),
        "changed": sum(1 for result in results if result["changed"]),
        "files": results,
    }
    if args.json:
        emit_json(payload)
    elif args.check:
        print(f"format ok: {len(results)} files")
    else:
        print(f"formatted {payload['changed']} of {len(results)} files")
    return 0


def format_file_targets(target: Path) -> tuple[ProjectConfig | None, list[Path]]:
    if target.is_dir():
        config = load_project(target)
        return config, project_files(config)
    if target.exists() and target.suffix != ".hayulo":
        raise HayuloError(
            Diagnostic(
                code="format.unsupported_target",
                message=f"Hayulo format only supports .hayulo files or project directories: {target}.",
                file=str(target),
                suggestions=["Pass a .hayulo source file or a directory containing hayulo.toml."],
            )
        )
    return None, [target]


def write_source(path: Path, source: str) -> None:
    try:
        path.write_text(source, encoding="utf-8")
    except OSError as exc:
        details: dict[str, Any] = {}
        if exc.errno is not None:
            details["errno"] = exc.errno
        raise HayuloError(
            Diagnostic(
                code="file_write_failed",
                message=f"Could not write Hayulo source file: {exc.strerror or exc}.",
                file=str(path),
                details=details,
                suggestions=["Check file permissions and try again."],
            )
        ) from exc


def cmd_summarize(args: argparse.Namespace) -> int:
    target = Path(args.target) if args.target else Path(".")
    try:
        payload = summarize_target(target)
    except HayuloError as error:
        return handle_error(error, args.json)

    if args.json:
        emit_json(payload)
    else:
        if payload["kind"] == "project-summary":
            print(f"{payload['project']['name']}: {payload['totals']['files']} files")
            print(f"functions: {payload['totals']['functions']}, tests: {payload['totals']['tests']}, routes: {payload['totals']['routes']}")
        else:
            print(f"{payload['file']}: {payload['kind']}")
    return 0


def summarize_target(target: Path) -> dict[str, Any]:
    if target.is_dir():
        config = load_project(target)
        files = [summarize_file(path, filename=project_relative(config, path)) for path in project_files(config)]
        return {
            "status": "ok",
            "kind": "project-summary",
            "root": str(config.root),
            "config": str(config.config_path),
            "project": {"name": config.name, "version": config.version},
            "totals": summarize_totals(files),
            "files": files,
        }
    return summarize_file(target)


def summarize_file(path: Path, *, filename: str | None = None) -> dict[str, Any]:
    filename = filename or str(path)
    source = read_source(path)
    intent = parse_top_level_intent(source, filename=filename)
    if looks_like_api_source(source):
        spec = parse_api_source(source, filename=filename)
        return {
            "status": "ok",
            "kind": "api-summary",
            "file": filename,
            "module": spec.module,
            "intent": intent,
            "app": spec.app_name,
            "records": [
                {"name": record.name, "fields": [field.name for field in record.fields], "line": record.line}
                for record in spec.records.values()
            ],
            "routes": [
                {"method": route.method, "path": route.path, "response_type": route.response_type, "line": route.line}
                for route in spec.routes
            ],
        }

    program = load_program(path, source, filename=filename)
    check_program(program, filename=filename)
    return {
        "status": "ok",
        "kind": "script-summary",
        "file": filename,
        "module": program.module,
        "intent": intent,
        "functions": [
            {
                "name": function.name,
                "params": [param.name for param in function.params],
                "return_type": function.return_type,
                "line": function.line,
            }
            for function in program.functions.values()
        ],
        "tests": [{"name": test.name, "line": test.line} for test in program.tests],
    }


def summarize_totals(files: list[dict[str, Any]]) -> dict[str, int]:
    return {
        "files": len(files),
        "scripts": sum(1 for file in files if file["kind"] == "script-summary"),
        "apis": sum(1 for file in files if file["kind"] == "api-summary"),
        "functions": sum(len(file.get("functions", [])) for file in files),
        "tests": sum(len(file.get("tests", [])) for file in files),
        "records": sum(len(file.get("records", [])) for file in files),
        "routes": sum(len(file.get("routes", [])) for file in files),
    }


def cmd_new(args: argparse.Namespace) -> int:
    root = Path(args.path)
    name = args.name or root.name
    module = project_name_to_module(name)

    try:
        files = create_project(root, name=name, module=module)
    except HayuloError as error:
        return handle_error(error, args.json)

    payload = {
        "status": "ok",
        "kind": "project-new",
        "root": str(root),
        "project": {"name": name, "version": "0.1.0"},
        "files": [str(path) for path in files],
        "next_commands": [f"cd {root}", "hayulo check", "hayulo test", "hayulo run src/main.hayulo"],
    }
    if args.json:
        emit_json(payload)
    else:
        print(f"created Hayulo project: {root}")
        for path in files:
            print(f"  {path}")
        print("next:")
        print(f"  cd {root}")
        print("  hayulo check")
        print("  hayulo test")
        print("  hayulo run src/main.hayulo")
    return 0


def create_project(root: Path, *, name: str, module: str) -> list[Path]:
    if root.exists() and root.is_file():
        raise HayuloError(Diagnostic(code="project.exists", message=f"Project path is a file: {root}.", file=str(root), suggestions=["Choose a directory path."]))
    if root.exists() and any(root.iterdir()):
        raise HayuloError(Diagnostic(code="project.exists", message=f"Project directory is not empty: {root}.", file=str(root), suggestions=["Choose an empty directory or a new project path."]))

    root.mkdir(parents=True, exist_ok=True)
    src = root / "src"
    tests = root / "tests"
    src.mkdir(exist_ok=True)
    tests.mkdir(exist_ok=True)

    files = [
        root / "hayulo.toml",
        src / "main.hayulo",
        tests / "main_test.hayulo",
    ]
    files[0].write_text(project_config_text(name), encoding="utf-8")
    files[1].write_text(project_main_text(module), encoding="utf-8")
    files[2].write_text(project_test_text(module), encoding="utf-8")
    return files


def project_config_text(name: str) -> str:
    return f"""[project]
name = {json.dumps(name)}
version = "0.1.0"
src = "src"
tests = "tests"
"""


def project_main_text(module: str) -> str:
    return f"""module {module}.main

intent {{
  purpose: "A small Hayulo project."
}}

fn greet(name: Text) -> Text {{
  return "Hello, " + name
}}

fn main() {{
  print(greet("Hayulo"))
}}
"""


def project_test_text(module: str) -> str:
    return f"""module {module}.main_test

test "project test runs" {{
  expect 1 + 1 == 2
}}
"""


def project_relative(config: ProjectConfig, path: Path) -> str:
    try:
        return path.relative_to(config.root).as_posix()
    except ValueError:
        return str(path)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="hayulo", description="Hayulo prototype language toolchain")
    parser.add_argument("--version", action="version", version=f"hayulo {__version__}")
    sub = parser.add_subparsers(dest="command", required=True)

    check = sub.add_parser("check", help="parse and validate a Hayulo file or project")
    check.add_argument("target", nargs="?", help="file or project directory; defaults to current project")
    check.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    check.set_defaults(func=cmd_check)

    run = sub.add_parser("run", help="run fn main() in a Hayulo script file")
    run.add_argument("file")
    run.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    run.set_defaults(func=cmd_run)

    test = sub.add_parser("test", help="run tests in a Hayulo script file or project")
    test.add_argument("target", nargs="?", help="file or project directory; defaults to current project")
    test.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    test.set_defaults(func=cmd_test)

    fmt = sub.add_parser("format", help="format a Hayulo file or project")
    fmt.add_argument("target", nargs="?", help="file or project directory; defaults to current project")
    fmt.add_argument("--check", action="store_true", help="check formatting without writing files")
    fmt.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    fmt.set_defaults(func=cmd_format)

    summarize = sub.add_parser("summarize", help="summarize a Hayulo file or project for repair loops")
    summarize.add_argument("target", nargs="?", help="file or project directory; defaults to current project")
    summarize.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    summarize.set_defaults(func=cmd_summarize)

    new = sub.add_parser("new", help="create a Hayulo project")
    new.add_argument("path")
    new.add_argument("--name", help="project name; defaults to the directory name")
    new.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    new.set_defaults(func=cmd_new)

    build = sub.add_parser("build", help="build a Hayulo API file into a runnable REST API")
    build.add_argument("file")
    build.add_argument("--out", help="output directory; defaults to <source-dir>/generated")
    build.add_argument("--no-clean", action="store_true", help="do not delete the output directory before generating")
    build.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    build.set_defaults(func=cmd_build)

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
