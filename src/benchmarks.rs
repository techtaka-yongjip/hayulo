use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use regex::Regex;
use serde_json::{Value, json};

use crate::diagnostics::{Diagnostic, HayuloError, HayuloResult};

const LLM_BENCHMARK_SCHEMA: &str = "hayulo.llm_benchmark@0.1";
const LLM_BENCHMARK_TASK_SCHEMA: &str = "hayulo.llm_benchmark.task@0.1";

#[derive(Clone, Debug)]
struct LlmBenchmarkTask {
    id: String,
    title: String,
    category: String,
    difficulty: String,
    prompt: String,
    comparison_targets: Vec<String>,
    success_metrics: Vec<String>,
    expected_outputs: Vec<String>,
    hayulo_baseline: PathBuf,
    manual_checks: Vec<String>,
    source_file: PathBuf,
}

impl LlmBenchmarkTask {
    fn to_json(&self, root: &Path) -> Value {
        json!({
            "id": self.id,
            "title": self.title,
            "category": self.category,
            "difficulty": self.difficulty,
            "prompt": self.prompt,
            "comparison_targets": self.comparison_targets,
            "success_metrics": self.success_metrics,
            "expected_outputs": self.expected_outputs,
            "hayulo_baseline": relative_path(&self.hayulo_baseline, root),
            "manual_checks": self.manual_checks,
            "source_file": relative_path(&self.source_file, root),
        })
    }
}

pub fn llm_benchmark_payload(root: &Path, suite: &str) -> HayuloResult<Value> {
    if suite != "llm" {
        return Err(HayuloError::new(
            Diagnostic::new(
                "benchmark.unknown_suite",
                format!("Unknown benchmark suite: {suite}."),
            )
            .detail("suite", json!(suite))
            .suggestion("Use: hayulo benchmark llm."),
        ));
    }
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let tasks = load_llm_benchmark_tasks(&root)?;
    let runs = load_recorded_runs(&root, &tasks)?;
    Ok(json!({
        "schema": LLM_BENCHMARK_SCHEMA,
        "status": "ok",
        "kind": "llm-benchmark-suite",
        "suite": "llm",
        "root": root,
        "tasks_dir": relative_path(&tasks_dir(&root), &root),
        "runs_dir": relative_path(&runs_dir(&root), &root),
        "summary": suite_summary(&tasks, &runs),
        "tasks": tasks.iter().map(|task| task.to_json(&root)).collect::<Vec<_>>(),
        "recorded_runs": runs,
    }))
}

fn load_llm_benchmark_tasks(root: &Path) -> HayuloResult<Vec<LlmBenchmarkTask>> {
    let directory = tasks_dir(root);
    if !directory.is_dir() {
        return Err(HayuloError::new(
            Diagnostic::new(
                "benchmark.missing_tasks_dir",
                format!(
                    "Missing LLM benchmark tasks directory: {}.",
                    directory.display()
                ),
            )
            .file(Some(&directory.to_string_lossy()))
            .suggestion("Create benchmarks/llm/tasks with task JSON files."),
        ));
    }
    let mut paths = std::fs::read_dir(&directory)
        .map_err(|error| file_error("benchmark.file_read_failed", &directory, error))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|v| v.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();
    let mut tasks = Vec::new();
    let mut seen = BTreeMap::new();
    for path in paths {
        let task = parse_task_file(&path, root)?;
        if let Some(first) = seen.insert(task.id.clone(), path.clone()) {
            return Err(HayuloError::new(
                Diagnostic::new(
                    "benchmark.duplicate_task",
                    format!("Duplicate benchmark task id: {}.", task.id),
                )
                .file(Some(&path.to_string_lossy()))
                .detail("first_file", json!(relative_path(&first, root)))
                .detail("duplicate_file", json!(relative_path(&path, root)))
                .suggestion("Give every benchmark task a unique id."),
            ));
        }
        tasks.push(task);
    }
    if tasks.is_empty() {
        return Err(HayuloError::new(
            Diagnostic::new(
                "benchmark.empty_suite",
                "The LLM benchmark suite has no task JSON files.",
            )
            .file(Some(&directory.to_string_lossy()))
            .suggestion("Add at least one task under benchmarks/llm/tasks."),
        ));
    }
    Ok(tasks)
}

fn parse_task_file(path: &Path, root: &Path) -> HayuloResult<LlmBenchmarkTask> {
    let data = read_json_object(path, "benchmark.invalid_task_json")?;
    let schema = require_string(&data, "schema", path)?;
    if schema != LLM_BENCHMARK_TASK_SCHEMA {
        invalid_task(
            path,
            &format!("Task schema must be {LLM_BENCHMARK_TASK_SCHEMA:?}."),
        )?;
    }
    let task_id = require_string(&data, "id", path)?;
    if !Regex::new(r"^[a-z0-9][a-z0-9_.-]*$")
        .unwrap()
        .is_match(&task_id)
    {
        invalid_task(
            path,
            "Task id must use lowercase letters, digits, dots, underscores, or hyphens.",
        )?;
    }
    let baseline_text = require_string(&data, "hayulo_baseline", path)?;
    let baseline = root
        .join(&baseline_text)
        .canonicalize()
        .unwrap_or_else(|_| root.join(&baseline_text));
    if !baseline.is_file() {
        return Err(HayuloError::new(
            Diagnostic::new(
                "benchmark.missing_baseline",
                format!("Task {task_id} references a missing Hayulo baseline: {baseline_text}."),
            )
            .file(Some(&path.to_string_lossy()))
            .detail("task_id", json!(task_id))
            .detail("hayulo_baseline", json!(baseline_text))
            .suggestion("Add the baseline file or fix hayulo_baseline in the task JSON."),
        ));
    }
    Ok(LlmBenchmarkTask {
        id: task_id,
        title: require_string(&data, "title", path)?,
        category: require_string(&data, "category", path)?,
        difficulty: require_string(&data, "difficulty", path)?,
        prompt: require_string(&data, "prompt", path)?,
        comparison_targets: require_string_list(&data, "comparison_targets", path)?,
        success_metrics: require_string_list(&data, "success_metrics", path)?,
        expected_outputs: require_string_list(&data, "expected_outputs", path)?,
        hayulo_baseline: baseline,
        manual_checks: require_string_list(&data, "manual_checks", path)?,
        source_file: path.canonicalize().unwrap_or_else(|_| path.to_path_buf()),
    })
}

fn load_recorded_runs(root: &Path, tasks: &[LlmBenchmarkTask]) -> HayuloResult<Vec<Value>> {
    let directory = runs_dir(root);
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let known = tasks
        .iter()
        .map(|task| task.id.clone())
        .collect::<BTreeSet<_>>();
    let mut paths = std::fs::read_dir(&directory)
        .map_err(|error| file_error("benchmark.file_read_failed", &directory, error))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|v| v.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();
    let mut runs = Vec::new();
    for path in paths {
        let data = read_json_value(&path, "benchmark.invalid_run_json")?;
        let records = if let Some(items) = data.as_array() {
            items.clone()
        } else {
            vec![data]
        };
        for record in records {
            let Some(object) = record.as_object() else {
                return Err(HayuloError::new(
                    Diagnostic::new(
                        "benchmark.invalid_run",
                        "Recorded benchmark runs must be JSON objects.",
                    )
                    .file(Some(&path.to_string_lossy())),
                ));
            };
            let task_id = require_string_value(object, "task_id", &path)?;
            if !known.contains(&task_id) {
                return Err(HayuloError::new(
                    Diagnostic::new(
                        "benchmark.unknown_run_task",
                        format!("Recorded run references unknown task id: {task_id}."),
                    )
                    .file(Some(&path.to_string_lossy()))
                    .detail("task_id", json!(task_id))
                    .suggestion("Use a task id from benchmarks/llm/tasks."),
                ));
            }
            runs.push(json!({
                "task_id": task_id,
                "target": require_string_value(object, "target", &path)?,
                "model": require_string_value(object, "model", &path)?,
                "status": require_string_value(object, "status", &path)?,
                "source_file": relative_path(&path, root),
                "metrics": object.get("metrics").cloned().unwrap_or_else(|| json!({})),
            }));
        }
    }
    Ok(runs)
}

fn suite_summary(tasks: &[LlmBenchmarkTask], runs: &[Value]) -> Value {
    json!({
        "tasks": tasks.len(),
        "categories": count_values(tasks.iter().map(|task| task.category.as_str())),
        "difficulties": count_values(tasks.iter().map(|task| task.difficulty.as_str())),
        "comparison_targets": count_values(tasks.iter().flat_map(|task| task.comparison_targets.iter().map(String::as_str))),
        "recorded_runs": runs.len(),
        "run_statuses": count_values(runs.iter().filter_map(|run| run["status"].as_str())),
    })
}

fn tasks_dir(root: &Path) -> PathBuf {
    root.join("benchmarks").join("llm").join("tasks")
}

fn runs_dir(root: &Path) -> PathBuf {
    root.join("benchmarks").join("llm").join("runs")
}

fn read_json_object(path: &Path, code: &str) -> HayuloResult<serde_json::Map<String, Value>> {
    let value = read_json_value(path, code)?;
    value.as_object().cloned().ok_or_else(|| {
        HayuloError::new(
            Diagnostic::new(code, "Expected a JSON object.").file(Some(&path.to_string_lossy())),
        )
    })
}

fn read_json_value(path: &Path, code: &str) -> HayuloResult<Value> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| file_error("benchmark.file_read_failed", path, error))?;
    serde_json::from_str(&text).map_err(|error| {
        HayuloError::new(
            Diagnostic::new(code, format!("Invalid JSON: {}.", error))
                .file(Some(&path.to_string_lossy()))
                .line(Some(error.line()))
                .column(Some(error.column()))
                .suggestion("Fix the JSON syntax and try again."),
        )
    })
}

fn require_string(
    data: &serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> HayuloResult<String> {
    require_string_value(data, key, path)
}

fn require_string_value(
    data: &serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> HayuloResult<String> {
    match data.get(key).and_then(Value::as_str) {
        Some(value) if !value.is_empty() => Ok(value.to_string()),
        _ => invalid_task(path, &format!("Field {key:?} must be a non-empty string.")),
    }
}

fn require_string_list(
    data: &serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> HayuloResult<Vec<String>> {
    match data.get(key).and_then(Value::as_array) {
        Some(values)
            if !values.is_empty()
                && values
                    .iter()
                    .all(|value| value.as_str().is_some_and(|text| !text.is_empty())) =>
        {
            Ok(values
                .iter()
                .map(|value| value.as_str().unwrap().to_string())
                .collect())
        }
        _ => invalid_task(
            path,
            &format!("Field {key:?} must be a non-empty list of strings."),
        ),
    }
}

fn invalid_task<T>(path: &Path, message: &str) -> HayuloResult<T> {
    Err(HayuloError::new(
        Diagnostic::new("benchmark.invalid_task", message)
            .file(Some(&path.to_string_lossy()))
            .suggestion("Check the task format in docs/llm_benchmarks.md."),
    ))
}

fn count_values<'a>(values: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value.to_string()).or_insert(0) += 1;
    }
    counts
}

fn relative_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

fn file_error(code: &str, path: &Path, error: std::io::Error) -> HayuloError {
    HayuloError::new(
        Diagnostic::new(code, format!("Could not read benchmark file: {error}."))
            .file(Some(&path.to_string_lossy())),
    )
}
