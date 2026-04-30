from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

from . import __version__
from .api import generate_api, looks_like_api_source, parse_api_source
from .checker import check_program
from .diagnostics import Diagnostic, HayuloError
from .intent import parse_top_level_intent
from .interpreter import Interpreter
from .lexer import Lexer
from .parser import Parser


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


def load_program(path: Path, source: str | None = None):
    if source is None:
        source = read_source(path)
    tokens = Lexer(source, filename=str(path)).lex()
    return Parser(tokens, filename=str(path)).parse()


def emit_json(payload: dict[str, Any]) -> None:
    print(json.dumps(payload, indent=2, sort_keys=True))


def handle_error(error: HayuloError, json_mode: bool) -> int:
    if json_mode:
        emit_json({"status": "failed", "errors": [error.diagnostic.to_dict()]})
    else:
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


def cmd_check(args: argparse.Namespace) -> int:
    path = Path(args.file)
    try:
        source = read_source(path)
        intent = parse_top_level_intent(source, filename=str(path))
        if looks_like_api_source(source):
            spec = parse_api_source(source, filename=str(path))
            payload = {
                "status": "ok",
                "kind": "api",
                "file": str(path),
                "module": spec.module,
                "intent": intent,
                "app": spec.app_name,
                "database": spec.database.to_dict() if spec.database else None,
                "records": sorted(spec.records.keys()),
                "routes": [route.to_dict() for route in spec.routes],
            }
        else:
            program = load_program(path, source)
            check_program(program, filename=str(path))
            payload = {
                "status": "ok",
                "kind": "script",
                "file": str(path),
                "module": program.module,
                "intent": intent,
                "functions": sorted(program.functions.keys()),
                "tests": [test.name for test in program.tests],
            }
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
    path = Path(args.file)
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
        emit_json(
            {
                "status": status,
                "file": str(path),
                "passed": passed,
                "failed": failed,
                "tests": [result.to_dict() for result in results],
                "output": interpreter.output,
            }
        )
    else:
        for result in results:
            marker = "PASS" if result.passed else "FAIL"
            print(f"{marker} {result.name}")
            if result.error:
                print(f"  {result.error}")
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


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="hayulo", description="Hayulo prototype language toolchain")
    parser.add_argument("--version", action="version", version=f"hayulo {__version__}")
    sub = parser.add_subparsers(dest="command", required=True)

    check = sub.add_parser("check", help="parse and validate a Hayulo file")
    check.add_argument("file")
    check.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    check.set_defaults(func=cmd_check)

    run = sub.add_parser("run", help="run fn main() in a Hayulo script file")
    run.add_argument("file")
    run.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    run.set_defaults(func=cmd_run)

    test = sub.add_parser("test", help="run tests in a Hayulo script file")
    test.add_argument("file")
    test.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    test.set_defaults(func=cmd_test)

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
