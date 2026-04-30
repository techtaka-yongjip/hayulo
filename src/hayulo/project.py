from __future__ import annotations

import ast
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .diagnostics import Diagnostic, HayuloError


CONFIG_NAME = "hayulo.toml"
DEFAULT_SKIP_DIRS = {"generated", "__pycache__", ".git", ".venv", "venv", "node_modules"}


@dataclass
class ProjectConfig:
    root: Path
    name: str
    version: str
    source_dirs: list[Path]
    test_dirs: list[Path]
    excludes: set[Path]

    @property
    def config_path(self) -> Path:
        return self.root / CONFIG_NAME


def load_project(start: Path) -> ProjectConfig:
    root = find_project_root(start)
    if root is None:
        raise HayuloError(
            Diagnostic(
                code="project.missing_config",
                message=f"No {CONFIG_NAME} found for project target: {start}.",
                file=str(start),
                suggestions=[f"Run hayulo new <name> or create {CONFIG_NAME} in the project root."],
            )
        )
    return read_project_config(root)


def find_project_root(start: Path) -> Path | None:
    path = start.resolve()
    if path.is_file():
        path = path.parent
    for candidate in [path, *path.parents]:
        if (candidate / CONFIG_NAME).is_file():
            return candidate
    return None


def read_project_config(root: Path) -> ProjectConfig:
    path = root / CONFIG_NAME
    data = parse_hayulo_toml(path)
    project = data.get("project", {})
    name = as_string(project.get("name"), "name", path) or root.name
    version = as_string(project.get("version"), "version", path) or "0.1.0"
    source_dirs = paths_from_value(project.get("src", "src"), root, path, "src")
    test_dirs = paths_from_value(project.get("tests", "tests"), root, path, "tests")
    excludes = set(paths_from_value(project.get("exclude", []), root, path, "exclude"))
    return ProjectConfig(root=root, name=name, version=version, source_dirs=source_dirs, test_dirs=test_dirs, excludes=excludes)


def parse_hayulo_toml(path: Path) -> dict[str, dict[str, Any]]:
    try:
        text = path.read_text(encoding="utf-8")
    except FileNotFoundError:
        raise HayuloError(Diagnostic(code="project.missing_config", message=f"Missing {CONFIG_NAME}.", file=str(path))) from None
    except OSError as exc:
        raise HayuloError(Diagnostic(code="project.config_read_failed", message=f"Could not read {CONFIG_NAME}: {exc}", file=str(path))) from exc

    data: dict[str, dict[str, Any]] = {}
    section: str | None = None
    for line_no, raw in enumerate(text.splitlines(), start=1):
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1].strip()
            if not section:
                config_error(path, line_no, "project.invalid_config", "Empty TOML section name.")
            data.setdefault(section, {})
            continue
        if section is None:
            config_error(path, line_no, "project.invalid_config", "Expected a TOML section before key/value entries.")
        if "=" not in line:
            config_error(path, line_no, "project.invalid_config", f"Invalid TOML line: {line!r}.")
        key, raw_value = [part.strip() for part in line.split("=", 1)]
        if not re.fullmatch(r"[A-Za-z_]\w*", key):
            config_error(path, line_no, "project.invalid_config", f"Invalid TOML key: {key!r}.")
        data[section][key] = parse_value(raw_value, path, line_no)
    return data


def parse_value(value: str, path: Path, line: int) -> Any:
    if value.startswith('"') and value.endswith('"'):
        try:
            return ast.literal_eval(value)
        except (SyntaxError, ValueError):
            config_error(path, line, "project.invalid_config", "Invalid string value.")
    if value.startswith("[") and value.endswith("]"):
        try:
            parsed = ast.literal_eval(value)
        except (SyntaxError, ValueError):
            config_error(path, line, "project.invalid_config", "Invalid array value.")
        if not isinstance(parsed, list) or not all(isinstance(item, str) for item in parsed):
            config_error(path, line, "project.invalid_config", "Only arrays of strings are supported.")
        return parsed
    config_error(path, line, "project.invalid_config", "Only quoted strings and string arrays are supported.")


def as_string(value: Any, key: str, path: Path) -> str | None:
    if value is None:
        return None
    if isinstance(value, str):
        return value
    raise HayuloError(Diagnostic(code="project.invalid_config", message=f"Project field {key!r} must be a string.", file=str(path)))


def paths_from_value(value: Any, root: Path, path: Path, key: str) -> list[Path]:
    if isinstance(value, str):
        values = [value]
    elif isinstance(value, list) and all(isinstance(item, str) for item in value):
        values = value
    else:
        raise HayuloError(Diagnostic(code="project.invalid_config", message=f"Project field {key!r} must be a string or array of strings.", file=str(path)))
    return [(root / item).resolve() for item in values]


def project_files(config: ProjectConfig, *, include_tests: bool = True) -> list[Path]:
    roots = list(config.source_dirs)
    if include_tests:
        roots.extend(config.test_dirs)

    files: dict[Path, None] = {}
    for root in roots:
        if not root.exists():
            continue
        if root.is_file() and root.suffix == ".hayulo" and not is_excluded(root.resolve(), config):
            files[root.resolve()] = None
            continue
        if not root.is_dir():
            continue
        for path in root.rglob("*.hayulo"):
            resolved = path.resolve()
            if not is_excluded(resolved, config):
                files[resolved] = None
    return sorted(files)


def is_excluded(path: Path, config: ProjectConfig) -> bool:
    if path.is_relative_to(config.root):
        if any(part in DEFAULT_SKIP_DIRS or part.startswith(".") for part in path.relative_to(config.root).parts):
            return True
    for excluded in config.excludes:
        if path == excluded or excluded in path.parents:
            return True
    return False


def config_error(path: Path, line: int, code: str, message: str) -> None:
    raise HayuloError(Diagnostic(code=code, message=message, file=str(path), line=line, suggestions=["Check hayulo.toml syntax."]))


def project_name_to_module(name: str) -> str:
    value = re.sub(r"[^A-Za-z0-9_]+", "_", name.strip()).strip("_").lower()
    return value or "app"
