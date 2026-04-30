from __future__ import annotations

import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .diagnostics import Diagnostic, HayuloError


LLM_BENCHMARK_SCHEMA = "hayulo.llm_benchmark@0.1"
LLM_BENCHMARK_TASK_SCHEMA = "hayulo.llm_benchmark.task@0.1"


@dataclass(frozen=True)
class LlmBenchmarkTask:
    id: str
    title: str
    category: str
    difficulty: str
    prompt: str
    comparison_targets: list[str]
    success_metrics: list[str]
    expected_outputs: list[str]
    hayulo_baseline: Path
    manual_checks: list[str]
    source_file: Path

    def to_dict(self, root: Path) -> dict[str, Any]:
        return {
            "id": self.id,
            "title": self.title,
            "category": self.category,
            "difficulty": self.difficulty,
            "prompt": self.prompt,
            "comparison_targets": self.comparison_targets,
            "success_metrics": self.success_metrics,
            "expected_outputs": self.expected_outputs,
            "hayulo_baseline": relative_path(self.hayulo_baseline, root),
            "manual_checks": self.manual_checks,
            "source_file": relative_path(self.source_file, root),
        }


def llm_benchmark_payload(root: Path, suite: str = "llm") -> dict[str, Any]:
    if suite != "llm":
        raise HayuloError(
            Diagnostic(
                code="benchmark.unknown_suite",
                message=f"Unknown benchmark suite: {suite}.",
                details={"suite": suite},
                suggestions=["Use: hayulo benchmark llm."],
            )
        )

    root = root.resolve()
    tasks = load_llm_benchmark_tasks(root)
    runs = load_recorded_runs(root, tasks)
    return {
        "schema": LLM_BENCHMARK_SCHEMA,
        "status": "ok",
        "kind": "llm-benchmark-suite",
        "suite": "llm",
        "root": str(root),
        "tasks_dir": relative_path(tasks_dir(root), root),
        "runs_dir": relative_path(runs_dir(root), root),
        "summary": suite_summary(tasks, runs),
        "tasks": [task.to_dict(root) for task in tasks],
        "recorded_runs": runs,
    }


def load_llm_benchmark_tasks(root: Path) -> list[LlmBenchmarkTask]:
    directory = tasks_dir(root)
    if not directory.is_dir():
        raise HayuloError(
            Diagnostic(
                code="benchmark.missing_tasks_dir",
                message=f"Missing LLM benchmark tasks directory: {directory}.",
                file=str(directory),
                suggestions=["Create benchmarks/llm/tasks with task JSON files."],
            )
        )

    tasks: list[LlmBenchmarkTask] = []
    seen: dict[str, Path] = {}
    for path in sorted(directory.glob("*.json")):
        task = parse_task_file(path, root)
        if task.id in seen:
            raise HayuloError(
                Diagnostic(
                    code="benchmark.duplicate_task",
                    message=f"Duplicate benchmark task id: {task.id}.",
                    file=str(path),
                    details={"first_file": relative_path(seen[task.id], root), "duplicate_file": relative_path(path, root)},
                    suggestions=["Give every benchmark task a unique id."],
                )
            )
        seen[task.id] = path
        tasks.append(task)

    if not tasks:
        raise HayuloError(
            Diagnostic(
                code="benchmark.empty_suite",
                message="The LLM benchmark suite has no task JSON files.",
                file=str(directory),
                suggestions=["Add at least one task under benchmarks/llm/tasks."],
            )
        )
    return tasks


def parse_task_file(path: Path, root: Path) -> LlmBenchmarkTask:
    data = read_json_object(path, "benchmark.invalid_task_json")
    schema = require_string(data, "schema", path)
    if schema != LLM_BENCHMARK_TASK_SCHEMA:
        invalid_task(path, f"Task schema must be {LLM_BENCHMARK_TASK_SCHEMA!r}.")

    task_id = require_string(data, "id", path)
    if not re.fullmatch(r"[a-z0-9][a-z0-9_.-]*", task_id):
        invalid_task(path, "Task id must use lowercase letters, digits, dots, underscores, or hyphens.")

    baseline_text = require_string(data, "hayulo_baseline", path)
    baseline = (root / baseline_text).resolve()
    if not baseline.is_file():
        raise HayuloError(
            Diagnostic(
                code="benchmark.missing_baseline",
                message=f"Task {task_id} references a missing Hayulo baseline: {baseline_text}.",
                file=str(path),
                details={"task_id": task_id, "hayulo_baseline": baseline_text},
                suggestions=["Add the baseline file or fix hayulo_baseline in the task JSON."],
            )
        )

    return LlmBenchmarkTask(
        id=task_id,
        title=require_string(data, "title", path),
        category=require_string(data, "category", path),
        difficulty=require_string(data, "difficulty", path),
        prompt=require_string(data, "prompt", path),
        comparison_targets=require_string_list(data, "comparison_targets", path),
        success_metrics=require_string_list(data, "success_metrics", path),
        expected_outputs=require_string_list(data, "expected_outputs", path),
        hayulo_baseline=baseline,
        manual_checks=require_string_list(data, "manual_checks", path),
        source_file=path.resolve(),
    )


def load_recorded_runs(root: Path, tasks: list[LlmBenchmarkTask]) -> list[dict[str, Any]]:
    directory = runs_dir(root)
    if not directory.is_dir():
        return []

    known_ids = {task.id for task in tasks}
    runs: list[dict[str, Any]] = []
    for path in sorted(directory.glob("*.json")):
        data = read_json_value(path, "benchmark.invalid_run_json")
        records = data if isinstance(data, list) else [data]
        for record in records:
            if not isinstance(record, dict):
                raise HayuloError(Diagnostic(code="benchmark.invalid_run", message="Recorded benchmark runs must be JSON objects.", file=str(path)))
            task_id = require_string(record, "task_id", path)
            if task_id not in known_ids:
                raise HayuloError(
                    Diagnostic(
                        code="benchmark.unknown_run_task",
                        message=f"Recorded run references unknown task id: {task_id}.",
                        file=str(path),
                        details={"task_id": task_id},
                        suggestions=["Use a task id from benchmarks/llm/tasks."],
                    )
                )
            runs.append(
                {
                    "task_id": task_id,
                    "target": require_string(record, "target", path),
                    "model": require_string(record, "model", path),
                    "status": require_string(record, "status", path),
                    "source_file": relative_path(path.resolve(), root),
                    "metrics": record.get("metrics", {}),
                }
            )
    return runs


def suite_summary(tasks: list[LlmBenchmarkTask], runs: list[dict[str, Any]]) -> dict[str, Any]:
    return {
        "tasks": len(tasks),
        "categories": count_values(task.category for task in tasks),
        "difficulties": count_values(task.difficulty for task in tasks),
        "comparison_targets": count_values(target for task in tasks for target in task.comparison_targets),
        "recorded_runs": len(runs),
        "run_statuses": count_values(str(run["status"]) for run in runs),
    }


def tasks_dir(root: Path) -> Path:
    return root / "benchmarks" / "llm" / "tasks"


def runs_dir(root: Path) -> Path:
    return root / "benchmarks" / "llm" / "runs"


def read_json_object(path: Path, code: str) -> dict[str, Any]:
    data = read_json_value(path, code)
    if not isinstance(data, dict):
        raise HayuloError(Diagnostic(code=code, message="Expected a JSON object.", file=str(path)))
    return data


def read_json_value(path: Path, code: str) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise HayuloError(
            Diagnostic(
                code=code,
                message=f"Invalid JSON: {exc.msg}.",
                file=str(path),
                line=exc.lineno,
                column=exc.colno,
                suggestions=["Fix the JSON syntax and try again."],
            )
        ) from exc
    except OSError as exc:
        raise HayuloError(Diagnostic(code="benchmark.file_read_failed", message=f"Could not read benchmark file: {exc}.", file=str(path))) from exc


def require_string(data: dict[str, Any], key: str, path: Path) -> str:
    value = data.get(key)
    if isinstance(value, str) and value:
        return value
    invalid_task(path, f"Field {key!r} must be a non-empty string.")


def require_string_list(data: dict[str, Any], key: str, path: Path) -> list[str]:
    value = data.get(key)
    if isinstance(value, list) and value and all(isinstance(item, str) and item for item in value):
        return value
    invalid_task(path, f"Field {key!r} must be a non-empty list of strings.")


def invalid_task(path: Path, message: str) -> None:
    raise HayuloError(
        Diagnostic(
            code="benchmark.invalid_task",
            message=message,
            file=str(path),
            suggestions=["Check the task format in docs/llm_benchmarks.md."],
        )
    )


def count_values(values: Any) -> dict[str, int]:
    counts: dict[str, int] = {}
    for value in values:
        counts[str(value)] = counts.get(str(value), 0) + 1
    return dict(sorted(counts.items()))


def relative_path(path: Path, root: Path) -> str:
    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return str(path)
