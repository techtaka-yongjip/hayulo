use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};
use serde_json::{Value, json};

use crate::VERSION;
use crate::api::{
    check_api_permissions, generate_api, looks_like_api_source, parse_api_source,
    required_api_permissions,
};
use crate::benchmarks::llm_benchmark_payload;
use crate::checker::check_program;
use crate::diagnostics::{
    Diagnostic, HayuloError, HayuloResult, TEST_SCHEMA, diagnostic_failure_payload,
};
use crate::formatter::check_format;
use crate::intent::parse_top_level_intent;
use crate::interpreter::Interpreter;
use crate::lexer::lex;
use crate::parser::parse;
use crate::project::{
    ProjectConfig, find_project_root, load_project, project_files, project_name_to_module,
    project_relative, read_project_config,
};

#[derive(Parser)]
#[command(name = "hayulo", about = "Hayulo prototype language toolchain", version = VERSION)]
struct HayuloCli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "parse and validate a Hayulo file or project")]
    Check(TargetJson),
    #[command(about = "run fn main() in a Hayulo script file")]
    Run(FileJson),
    #[command(about = "run tests in a Hayulo script file or project")]
    Test(TargetJson),
    #[command(about = "format a Hayulo file or project")]
    Format(FormatArgs),
    #[command(about = "summarize a Hayulo file or project for repair loops")]
    Summarize(TargetJson),
    #[command(about = "validate and summarize benchmark suites")]
    Benchmark(BenchmarkArgs),
    #[command(about = "create a Hayulo project")]
    New(NewArgs),
    #[command(about = "build a Hayulo API file into a runnable REST API")]
    Build(BuildArgs),
}

#[derive(Args)]
struct TargetJson {
    target: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct FileJson {
    file: PathBuf,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct FormatArgs {
    target: Option<PathBuf>,
    #[arg(long)]
    check: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct BenchmarkArgs {
    #[arg(default_value = "llm")]
    suite: String,
    #[arg(long, default_value = ".")]
    root: PathBuf,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct NewArgs {
    kind_or_path: String,
    path: Option<PathBuf>,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct BuildArgs {
    file: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    no_clean: bool,
    #[arg(long)]
    json: bool,
}

pub fn main(args: Vec<String>) -> i32 {
    let cli = HayuloCli::parse_from(args);
    match cli.command {
        Command::Check(args) => cmd_check(args),
        Command::Run(args) => cmd_run(args),
        Command::Test(args) => cmd_test(args),
        Command::Format(args) => cmd_format(args),
        Command::Summarize(args) => cmd_summarize(args),
        Command::Benchmark(args) => cmd_benchmark(args),
        Command::New(args) => cmd_new(args),
        Command::Build(args) => cmd_build(args),
    }
}

fn read_source(path: &Path) -> HayuloResult<String> {
    std::fs::read_to_string(path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => HayuloError::new(
            Diagnostic::new(
                "file_not_found",
                format!("Hayulo source file not found: {}.", path.display()),
            )
            .file(Some(&path.to_string_lossy()))
            .suggestion("Check the path and try again."),
        ),
        std::io::ErrorKind::InvalidData => HayuloError::new(
            Diagnostic::new(
                "invalid_source_encoding",
                "Hayulo source files must be valid UTF-8.",
            )
            .file(Some(&path.to_string_lossy()))
            .detail("encoding", json!("utf-8"))
            .suggestion("Save the file as UTF-8 and try again."),
        ),
        _ => HayuloError::new(
            Diagnostic::new(
                "file_read_failed",
                format!("Could not read Hayulo source file: {error}."),
            )
            .file(Some(&path.to_string_lossy()))
            .suggestion("Check file permissions and try again."),
        ),
    })
}

fn load_program(
    path: &Path,
    source: Option<&str>,
    filename: Option<&str>,
) -> HayuloResult<crate::ast::Program> {
    let owned;
    let source = match source {
        Some(source) => source,
        None => {
            owned = read_source(path)?;
            &owned
        }
    };
    let filename = filename.unwrap_or_else(|| path.to_str().unwrap_or("<source>"));
    parse(lex(source, Some(filename))?, Some(filename))
}

fn emit_json(payload: &Value) {
    println!("{}", serde_json::to_string_pretty(payload).unwrap());
}

fn handle_error(error: HayuloError, json_mode: bool) -> i32 {
    handle_errors(vec![error], json_mode)
}

fn handle_errors(errors: Vec<HayuloError>, json_mode: bool) -> i32 {
    if json_mode {
        emit_json(&diagnostic_failure_payload(&errors));
    } else {
        for error in errors {
            let d = error.diagnostic;
            let mut location = String::new();
            if let Some(file) = d.file {
                location.push_str(&file);
            }
            if let Some(line) = d.line {
                location.push_str(&format!(":{line}"));
            }
            if let Some(column) = d.column {
                location.push_str(&format!(":{column}"));
            }
            let prefix = if location.is_empty() {
                String::new()
            } else {
                format!("{location}: ")
            };
            eprintln!("{prefix}{}: {}", d.code, d.message);
            for suggestion in d.suggestions {
                eprintln!("  hint: {suggestion}");
            }
        }
    }
    1
}

fn cmd_check(args: TargetJson) -> i32 {
    let path = args.target.unwrap_or_else(|| PathBuf::from("."));
    if path.is_dir() || !path.exists() && path.extension().is_none() {
        return cmd_check_project(&path, args.json);
    }
    match check_file_payload(&path, None, None) {
        Ok(payload) => {
            if args.json {
                emit_json(&payload);
            } else {
                println!("ok: {}", path.display());
                if payload["kind"] == "api" {
                    println!("app: {}", payload["app"].as_str().unwrap_or(""));
                    let records = payload["records"]
                        .as_array()
                        .map(|v| {
                            v.iter()
                                .filter_map(Value::as_str)
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();
                    println!(
                        "records: {}",
                        if records.is_empty() {
                            "(none)"
                        } else {
                            &records
                        }
                    );
                    println!(
                        "routes: {}",
                        payload["routes"].as_array().map(Vec::len).unwrap_or(0)
                    );
                } else {
                    let functions = payload["functions"]
                        .as_array()
                        .map(|v| {
                            v.iter()
                                .filter_map(Value::as_str)
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();
                    println!(
                        "functions: {}",
                        if functions.is_empty() {
                            "(none)"
                        } else {
                            &functions
                        }
                    );
                    println!(
                        "tests: {}",
                        payload["tests"].as_array().map(Vec::len).unwrap_or(0)
                    );
                }
            }
            0
        }
        Err(error) => handle_error(error, args.json),
    }
}

fn check_file_payload(
    path: &Path,
    filename: Option<&str>,
    config: Option<&ProjectConfig>,
) -> HayuloResult<Value> {
    let filename_text = filename
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.to_string_lossy().to_string());
    let source = read_source(path)?;
    let intent = parse_top_level_intent(&source, Some(&filename_text))?;
    if looks_like_api_source(&source) {
        let spec = parse_api_source(&source, Some(&filename_text))?;
        let owned_config;
        let config = if let Some(config) = config {
            Some(config)
        } else {
            owned_config = project_config_for_path(path)?;
            owned_config.as_ref()
        };
        if let Some(config) = config {
            check_api_permissions(&spec, &config.permissions, Some(&filename_text))?;
        }
        let mut records = spec.records.keys().cloned().collect::<Vec<_>>();
        records.sort();
        return Ok(json!({
            "status": "ok",
            "kind": "api",
            "file": filename_text,
            "module": spec.module,
            "intent": intent,
            "permissions": {"required": required_api_permissions(&spec)},
            "app": spec.app_name,
            "database": spec.database,
            "records": records,
            "routes": spec.routes.iter().map(|route| route.to_public_json()).collect::<Vec<_>>(),
        }));
    }

    let program = load_program(path, Some(&source), Some(&filename_text))?;
    check_program(&program, Some(&filename_text))?;
    let mut functions = program.functions.keys().cloned().collect::<Vec<_>>();
    functions.sort();
    Ok(json!({
        "status": "ok",
        "kind": "script",
        "file": filename_text,
        "module": program.module,
        "intent": intent,
        "functions": functions,
        "tests": program.tests.iter().map(|test| test.name.clone()).collect::<Vec<_>>(),
    }))
}

fn cmd_check_project(target: &Path, json_mode: bool) -> i32 {
    let result = (|| -> HayuloResult<Value> {
        let config = load_project(target)?;
        let mut checked = Vec::new();
        let mut errors = Vec::new();
        for path in project_files(&config, true) {
            let filename = project_relative(&config, &path);
            match check_file_payload(&path, Some(&filename), Some(&config)) {
                Ok(payload) => checked.push(payload),
                Err(error) => errors.push(error),
            }
        }
        if !errors.is_empty() {
            return Err(errors.remove(0));
        }
        Ok(json!({
            "status": "ok",
            "kind": "project",
            "root": config.root,
            "config": config.config_path(),
            "project": {"name": config.name, "version": config.version},
            "checked": checked.len(),
            "files": checked,
        }))
    })();
    match result {
        Ok(payload) => {
            if json_mode {
                emit_json(&payload);
            } else {
                println!(
                    "ok: {} ({} files)",
                    payload["project"]["name"].as_str().unwrap_or("project"),
                    payload["checked"]
                );
                for item in payload["files"].as_array().unwrap_or(&Vec::new()) {
                    println!(
                        "  {}: {}",
                        item["file"].as_str().unwrap_or(""),
                        item["kind"].as_str().unwrap_or("")
                    );
                }
            }
            0
        }
        Err(error) => handle_error(error, json_mode),
    }
}

fn cmd_run(args: FileJson) -> i32 {
    match (|| -> HayuloResult<(Value, Vec<String>)> {
        let program = load_program(&args.file, None, None)?;
        let mut interpreter = Interpreter::new(&program, Some(&args.file.to_string_lossy()));
        let result = interpreter.run_main()?;
        let output = interpreter.output.clone();
        Ok((
            json!({"status": "ok", "file": args.file, "output": output, "result": interpreter.stringify(&result)}),
            interpreter.output,
        ))
    })() {
        Ok((payload, output)) => {
            if args.json {
                emit_json(&payload);
            } else {
                for line in output {
                    println!("{line}");
                }
            }
            0
        }
        Err(error) => handle_error(error, args.json),
    }
}

fn cmd_test(args: TargetJson) -> i32 {
    let path = args.target.unwrap_or_else(|| PathBuf::from("."));
    if path.is_dir() || !path.exists() && path.extension().is_none() {
        return cmd_test_project(&path, args.json);
    }
    match test_file_payload(&path, None) {
        Ok(payload) => {
            let failed = payload["failed"].as_u64().unwrap_or(0);
            if args.json {
                emit_json(&payload);
            } else {
                for result in payload["tests"].as_array().unwrap_or(&Vec::new()) {
                    let marker = if result["passed"].as_bool().unwrap_or(false) {
                        "PASS"
                    } else {
                        "FAIL"
                    };
                    println!("{marker} {}", result["name"].as_str().unwrap_or(""));
                    if let Some(error) = result["error"].as_str() {
                        println!("  {error}");
                    }
                }
                println!("{} passed, {} failed", payload["passed"], payload["failed"]);
            }
            if failed == 0 { 0 } else { 1 }
        }
        Err(error) => handle_error(error, args.json),
    }
}

fn test_json_payload(
    status: &str,
    file: Option<&str>,
    passed: usize,
    failed: usize,
    tests: Vec<Value>,
    output: Vec<String>,
    extra: Option<Value>,
) -> Value {
    let failures = tests
        .iter()
        .filter(|result| !result["passed"].as_bool().unwrap_or(false))
        .map(|result| {
            json!({
                "test": result["name"],
                "file": file,
                "line": result.get("line").cloned().unwrap_or(Value::Null),
                "message": result.get("error").and_then(Value::as_str).unwrap_or("Test failed."),
            })
        })
        .collect::<Vec<_>>();
    let mut payload = json!({
        "schema": TEST_SCHEMA,
        "status": status,
        "summary": {"passed": passed, "failed": failed},
        "failures": failures,
        "passed": passed,
        "failed": failed,
        "tests": tests,
        "output": output,
    });
    if let Some(file) = file {
        payload["file"] = json!(file);
    }
    if let Some(Value::Object(extra)) = extra {
        for (key, value) in extra {
            payload[key] = value;
        }
    }
    payload
}

fn test_file_payload(path: &Path, filename: Option<&str>) -> HayuloResult<Value> {
    let filename_text = filename
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.to_string_lossy().to_string());
    let program = load_program(path, None, Some(&filename_text))?;
    check_program(&program, Some(&filename_text))?;
    let mut interpreter = Interpreter::new(&program, Some(&filename_text));
    let results = interpreter.run_tests();
    let passed = results.iter().filter(|result| result.passed).count();
    let failed = results.len() - passed;
    Ok(test_json_payload(
        if failed == 0 { "ok" } else { "failed" },
        Some(&filename_text),
        passed,
        failed,
        results
            .into_iter()
            .map(|result| serde_json::to_value(result).unwrap())
            .collect(),
        interpreter.output,
        None,
    ))
}

fn cmd_test_project(target: &Path, json_mode: bool) -> i32 {
    let result = (|| -> HayuloResult<Value> {
        let config = load_project(target)?;
        let mut files = Vec::new();
        let mut errors = Vec::new();
        for path in project_files(&config, true) {
            let source = read_source(&path)?;
            if looks_like_api_source(&source) {
                continue;
            }
            let filename = project_relative(&config, &path);
            match test_file_payload(&path, Some(&filename)) {
                Ok(payload) => files.push(payload),
                Err(error) => errors.push(error),
            }
        }
        if !errors.is_empty() {
            return Err(errors.remove(0));
        }
        let passed: usize = files
            .iter()
            .map(|file| file["passed"].as_u64().unwrap_or(0) as usize)
            .sum();
        let failed: usize = files
            .iter()
            .map(|file| file["failed"].as_u64().unwrap_or(0) as usize)
            .sum();
        let failures = files
            .iter()
            .flat_map(|file| file["failures"].as_array().cloned().unwrap_or_default())
            .collect::<Vec<_>>();
        Ok(json!({
            "schema": TEST_SCHEMA,
            "status": if failed == 0 { "ok" } else { "failed" },
            "kind": "project-test",
            "root": config.root,
            "config": config.config_path(),
            "project": {"name": config.name, "version": config.version},
            "summary": {"passed": passed, "failed": failed},
            "failures": failures,
            "passed": passed,
            "failed": failed,
            "files": files,
        }))
    })();
    match result {
        Ok(payload) => {
            let failed = payload["failed"].as_u64().unwrap_or(0);
            if json_mode {
                emit_json(&payload);
            } else {
                for file in payload["files"].as_array().unwrap_or(&Vec::new()) {
                    println!(
                        "{}: {} passed, {} failed",
                        file["file"].as_str().unwrap_or(""),
                        file["passed"],
                        file["failed"]
                    );
                    for result in file["tests"].as_array().unwrap_or(&Vec::new()) {
                        let marker = if result["passed"].as_bool().unwrap_or(false) {
                            "PASS"
                        } else {
                            "FAIL"
                        };
                        println!("  {marker} {}", result["name"].as_str().unwrap_or(""));
                    }
                }
                println!("{} passed, {} failed", payload["passed"], payload["failed"]);
            }
            if failed == 0 { 0 } else { 1 }
        }
        Err(error) => handle_error(error, json_mode),
    }
}

fn cmd_build(args: BuildArgs) -> i32 {
    match (|| -> HayuloResult<Value> {
        let out_dir = args.out.clone().unwrap_or_else(|| {
            args.file
                .parent()
                .unwrap_or(Path::new("."))
                .join("generated")
        });
        let source = read_source(&args.file)?;
        let spec = parse_api_source(&source, Some(&args.file.to_string_lossy()))?;
        if let Some(config) = project_config_for_path(&args.file)? {
            check_api_permissions(
                &spec,
                &config.permissions,
                Some(&args.file.to_string_lossy()),
            )?;
        }
        let files = generate_api(&spec, &out_dir, !args.no_clean)?;
        Ok(json!({
            "status": "ok",
            "kind": "api-build",
            "file": args.file,
            "app": spec.app_name,
            "permissions": {"required": required_api_permissions(&spec)},
            "output_dir": out_dir,
            "generated": files,
            "next_commands": [format!("cd {}", out_dir.display()), "npm test".to_string(), "npm start".to_string()],
        }))
    })() {
        Ok(payload) => {
            if args.json {
                emit_json(&payload);
            } else {
                println!(
                    "built {} -> {}",
                    payload["app"].as_str().unwrap_or("app"),
                    payload["output_dir"].as_str().unwrap_or("")
                );
                for file in payload["generated"].as_array().unwrap_or(&Vec::new()) {
                    let path = file["path"].as_str().unwrap_or("");
                    let name = Path::new(path)
                        .file_name()
                        .and_then(|value| value.to_str())
                        .unwrap_or(path);
                    println!("  {}: {}", name, file["description"].as_str().unwrap_or(""));
                }
                println!("next:");
                for command in payload["next_commands"].as_array().unwrap_or(&Vec::new()) {
                    println!("  {}", command.as_str().unwrap_or(""));
                }
            }
            0
        }
        Err(error) => handle_error(error, args.json),
    }
}

fn cmd_format(args: FormatArgs) -> i32 {
    match (|| -> HayuloResult<Value> {
        let target = args.target.unwrap_or_else(|| PathBuf::from("."));
        let (config, files) = format_file_targets(&target)?;
        let mut results = Vec::new();
        let mut changed = Vec::new();
        for path in files {
            let source = read_source(&path)?;
            let result = check_format(&source);
            let label = config
                .as_ref()
                .map(|config| project_relative(config, &path))
                .unwrap_or_else(|| path.to_string_lossy().to_string());
            if result.changed {
                changed.push(label.clone());
                if !args.check {
                    write_source(&path, &result.source)?;
                }
            }
            results.push(json!({"file": label, "changed": result.changed}));
        }
        if args.check && !changed.is_empty() {
            return Err(HayuloError::new(
                Diagnostic::new("format.required", "Hayulo source is not formatted.")
                    .file(Some(&changed[0]))
                    .detail("files", json!(changed))
                    .suggestion("Run hayulo format on the target."),
            ));
        }
        Ok(json!({
            "status": "ok",
            "kind": "format",
            "mode": if args.check { "check" } else { "write" },
            "checked": results.len(),
            "changed": results.iter().filter(|item| item["changed"].as_bool().unwrap_or(false)).count(),
            "files": results,
        }))
    })() {
        Ok(payload) => {
            if args.json {
                emit_json(&payload);
            } else if args.check {
                println!("format ok: {} files", payload["checked"]);
            } else {
                println!(
                    "formatted {} of {} files",
                    payload["changed"], payload["checked"]
                );
            }
            0
        }
        Err(error) => handle_error(error, args.json),
    }
}

fn format_file_targets(target: &Path) -> HayuloResult<(Option<ProjectConfig>, Vec<PathBuf>)> {
    if target.is_dir() {
        let config = load_project(target)?;
        let files = project_files(&config, true);
        return Ok((Some(config), files));
    }
    if target.exists() && target.extension().and_then(|value| value.to_str()) != Some("hayulo") {
        return Err(HayuloError::new(
            Diagnostic::new(
                "format.unsupported_target",
                format!(
                    "Hayulo format only supports .hayulo files or project directories: {}.",
                    target.display()
                ),
            )
            .file(Some(&target.to_string_lossy()))
            .suggestion("Pass a .hayulo source file or a directory containing hayulo.toml."),
        ));
    }
    Ok((None, vec![target.to_path_buf()]))
}

fn write_source(path: &Path, source: &str) -> HayuloResult<()> {
    std::fs::write(path, source).map_err(|error| {
        HayuloError::new(
            Diagnostic::new(
                "file_write_failed",
                format!("Could not write Hayulo source file: {error}."),
            )
            .file(Some(&path.to_string_lossy()))
            .suggestion("Check file permissions and try again."),
        )
    })
}

fn cmd_summarize(args: TargetJson) -> i32 {
    let target = args.target.unwrap_or_else(|| PathBuf::from("."));
    match summarize_target(&target) {
        Ok(payload) => {
            if args.json {
                emit_json(&payload);
            } else if payload["kind"] == "project-summary" {
                println!(
                    "{}: {} files",
                    payload["project"]["name"].as_str().unwrap_or("project"),
                    payload["totals"]["files"]
                );
                println!(
                    "functions: {}, tests: {}, routes: {}",
                    payload["totals"]["functions"],
                    payload["totals"]["tests"],
                    payload["totals"]["routes"]
                );
            } else {
                println!(
                    "{}: {}",
                    payload["file"].as_str().unwrap_or(""),
                    payload["kind"].as_str().unwrap_or("")
                );
            }
            0
        }
        Err(error) => handle_error(error, args.json),
    }
}

fn summarize_target(target: &Path) -> HayuloResult<Value> {
    if target.is_dir() {
        let config = load_project(target)?;
        let files = project_files(&config, true)
            .into_iter()
            .map(|path| summarize_file(&path, Some(&project_relative(&config, &path))))
            .collect::<HayuloResult<Vec<_>>>()?;
        return Ok(json!({
            "status": "ok",
            "kind": "project-summary",
            "root": config.root,
            "config": config.config_path(),
            "project": {"name": config.name, "version": config.version},
            "totals": summarize_totals(&files),
            "files": files,
        }));
    }
    summarize_file(target, None)
}

fn summarize_file(path: &Path, filename: Option<&str>) -> HayuloResult<Value> {
    let filename_text = filename
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.to_string_lossy().to_string());
    let source = read_source(path)?;
    let intent = parse_top_level_intent(&source, Some(&filename_text))?;
    if looks_like_api_source(&source) {
        let spec = parse_api_source(&source, Some(&filename_text))?;
        return Ok(json!({
            "status": "ok",
            "kind": "api-summary",
            "file": filename_text,
            "module": spec.module,
            "intent": intent,
            "app": spec.app_name,
            "records": spec.records.values().map(|record| json!({"name": record.name, "fields": record.fields.iter().map(|field| field.name.clone()).collect::<Vec<_>>(), "line": record.line})).collect::<Vec<_>>(),
            "routes": spec.routes.iter().map(|route| json!({"method": route.method, "path": route.path, "response_type": route.response_type, "line": route.line})).collect::<Vec<_>>(),
        }));
    }
    let program = load_program(path, Some(&source), Some(&filename_text))?;
    check_program(&program, Some(&filename_text))?;
    Ok(json!({
        "status": "ok",
        "kind": "script-summary",
        "file": filename_text,
        "module": program.module,
        "intent": intent,
        "functions": program.functions.values().map(|function| json!({"name": function.name, "params": function.params.iter().map(|param| param.name.clone()).collect::<Vec<_>>(), "return_type": function.return_type, "line": function.line})).collect::<Vec<_>>(),
        "tests": program.tests.iter().map(|test| json!({"name": test.name, "line": test.line})).collect::<Vec<_>>(),
    }))
}

fn summarize_totals(files: &[Value]) -> Value {
    json!({
        "files": files.len(),
        "scripts": files.iter().filter(|file| file["kind"] == "script-summary").count(),
        "apis": files.iter().filter(|file| file["kind"] == "api-summary").count(),
        "functions": files.iter().map(|file| file["functions"].as_array().map(Vec::len).unwrap_or(0)).sum::<usize>(),
        "tests": files.iter().map(|file| file["tests"].as_array().map(Vec::len).unwrap_or(0)).sum::<usize>(),
        "records": files.iter().map(|file| file["records"].as_array().map(Vec::len).unwrap_or(0)).sum::<usize>(),
        "routes": files.iter().map(|file| file["routes"].as_array().map(Vec::len).unwrap_or(0)).sum::<usize>(),
    })
}

fn cmd_benchmark(args: BenchmarkArgs) -> i32 {
    match llm_benchmark_payload(&args.root, &args.suite) {
        Ok(payload) => {
            if args.json {
                emit_json(&payload);
            } else {
                println!(
                    "{} benchmark: {} tasks, {} recorded runs",
                    payload["suite"].as_str().unwrap_or("llm"),
                    payload["summary"]["tasks"],
                    payload["summary"]["recorded_runs"]
                );
                for task in payload["tasks"].as_array().unwrap_or(&Vec::new()) {
                    let targets = task["comparison_targets"]
                        .as_array()
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(Value::as_str)
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();
                    println!(
                        "  {}: {} [{}]",
                        task["id"].as_str().unwrap_or(""),
                        task["title"].as_str().unwrap_or(""),
                        targets
                    );
                }
            }
            0
        }
        Err(error) => handle_error(error, args.json),
    }
}

fn cmd_new(args: NewArgs) -> i32 {
    let result = (|| -> HayuloResult<Value> {
        let (template, root) = if args.kind_or_path == "api" {
            let Some(path) = args.path else {
                return Err(HayuloError::new(
                    Diagnostic::new(
                        "project.missing_path",
                        "hayulo new api requires a project directory.",
                    )
                    .suggestion("Use: hayulo new api my-api"),
                ));
            };
            ("api", path)
        } else if args.path.is_some() {
            return Err(HayuloError::new(
                Diagnostic::new(
                    "project.invalid_template",
                    format!("Unknown project template {:?}.", args.kind_or_path),
                )
                .suggestion("Use hayulo new <project-dir> or hayulo new api <project-dir>."),
            ));
        } else {
            ("script", PathBuf::from(&args.kind_or_path))
        };
        let name = args.name.unwrap_or_else(|| {
            root.file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("app")
                .to_string()
        });
        let module = project_name_to_module(&name);
        let files = if template == "api" {
            create_api_project(&root, &name, &module, &project_name_to_app(&name))?
        } else {
            create_project(&root, &name, &module)?
        };
        let next_commands = if template == "api" {
            vec![
                format!("cd {}", root.display()),
                "hayulo check".to_string(),
                "hayulo build src/main.hayulo".to_string(),
                "cd src/generated".to_string(),
                "npm test".to_string(),
                "npm start".to_string(),
            ]
        } else {
            vec![
                format!("cd {}", root.display()),
                "hayulo check".to_string(),
                "hayulo test".to_string(),
                "hayulo run src/main.hayulo".to_string(),
            ]
        };
        Ok(json!({
            "status": "ok",
            "kind": if template == "api" { "project-new-api" } else { "project-new" },
            "root": root,
            "template": template,
            "project": {"name": name, "version": "0.1.0"},
            "files": files,
            "next_commands": next_commands,
        }))
    })();
    match result {
        Ok(payload) => {
            if args.json {
                emit_json(&payload);
            } else {
                println!(
                    "created Hayulo {} project: {}",
                    payload["template"].as_str().unwrap_or("script"),
                    payload["root"].as_str().unwrap_or("")
                );
                for path in payload["files"].as_array().unwrap_or(&Vec::new()) {
                    println!("  {}", path.as_str().unwrap_or(""));
                }
                println!("next:");
                for command in payload["next_commands"].as_array().unwrap_or(&Vec::new()) {
                    println!("  {}", command.as_str().unwrap_or(""));
                }
            }
            0
        }
        Err(error) => handle_error(error, args.json),
    }
}

fn create_project(root: &Path, name: &str, module: &str) -> HayuloResult<Vec<PathBuf>> {
    ensure_new_project_root(root)?;
    std::fs::create_dir_all(root.join("src")).map_err(|error| create_error(root, error))?;
    std::fs::create_dir_all(root.join("tests")).map_err(|error| create_error(root, error))?;
    let files = vec![
        root.join("hayulo.toml"),
        root.join("src/main.hayulo"),
        root.join("tests/main_test.hayulo"),
    ];
    write_source(&files[0], &project_config_text(name))?;
    write_source(&files[1], &project_main_text(module))?;
    write_source(&files[2], &project_test_text(module))?;
    Ok(files)
}

fn create_api_project(
    root: &Path,
    name: &str,
    module: &str,
    app_name: &str,
) -> HayuloResult<Vec<PathBuf>> {
    ensure_new_project_root(root)?;
    std::fs::create_dir_all(root.join("src")).map_err(|error| create_error(root, error))?;
    let files = vec![root.join("hayulo.toml"), root.join("src/main.hayulo")];
    write_source(&files[0], &api_project_config_text(name))?;
    write_source(&files[1], &project_api_main_text(module, name, app_name))?;
    Ok(files)
}

fn ensure_new_project_root(root: &Path) -> HayuloResult<()> {
    if root.exists() && root.is_file() {
        return Err(HayuloError::new(
            Diagnostic::new(
                "project.exists",
                format!("Project path is a file: {}.", root.display()),
            )
            .file(Some(&root.to_string_lossy()))
            .suggestion("Choose a directory path."),
        ));
    }
    if root.exists()
        && root
            .read_dir()
            .map(|mut iter| iter.next().is_some())
            .unwrap_or(false)
    {
        return Err(HayuloError::new(
            Diagnostic::new(
                "project.exists",
                format!("Project directory is not empty: {}.", root.display()),
            )
            .file(Some(&root.to_string_lossy()))
            .suggestion("Choose an empty directory or a new project path."),
        ));
    }
    Ok(())
}

fn create_error(path: &Path, error: std::io::Error) -> HayuloError {
    HayuloError::new(
        Diagnostic::new(
            "project.create_failed",
            format!("Could not create project path {}: {error}.", path.display()),
        )
        .file(Some(&path.to_string_lossy())),
    )
}

fn project_name_to_app(name: &str) -> String {
    let parts = name
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let app = parts
        .iter()
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!(
                    "{}{}",
                    first.to_ascii_uppercase(),
                    chars.collect::<String>()
                ),
                None => String::new(),
            }
        })
        .collect::<String>();
    if app.is_empty() {
        "App".to_string()
    } else {
        app
    }
}

fn project_config_text(name: &str) -> String {
    format!(
        r#"[project]
name = {name:?}
version = "0.1.0"
src = "src"
tests = "tests"

[permissions]
allow = []
deny = []
"#
    )
}

fn api_project_config_text(name: &str) -> String {
    format!(
        r#"[project]
name = {name:?}
version = "0.1.0"
src = "src"
tests = "tests"

[permissions]
allow = ["api.read", "api.write", "api.delete", "storage.local"]
deny = []
"#
    )
}

fn project_main_text(module: &str) -> String {
    format!(
        r#"module {module}.main

intent {{
  purpose: "A small Hayulo project."
}}

fn greet(name: Text) -> Text {{
  return "Hello, " + name
}}

fn main() {{
  print(greet("Hayulo"))
}}
"#
    )
}

fn project_test_text(module: &str) -> String {
    format!(
        r#"module {module}.main_test

test "project test runs" {{
  expect 1 + 1 == 2
}}
"#
    )
}

fn project_api_main_text(module: &str, name: &str, app_name: &str) -> String {
    format!(
        r#"module {module}.api

intent {{
  purpose: "Define a small todo REST API."
  constraints: [
    "Keep the records and routes explicit for generated OpenAPI.",
    "Keep behavior simple enough for generated smoke tests."
  ]
}}

app {app_name} {{
  database sqlite "todo.db"

  openapi {{
    title: "{name} API"
    version: "0.1.0"
  }}

  type Todo = record {{
    id: Id<Todo>
    title: Text {{ min: 1, max: 200 }}
    done: Bool = false
    created_at: Time = now()
  }}

  route GET "/todos" -> List<Todo> {{
    effect api.read
    effect storage.local
    action list Todo
  }}

  route GET "/todos/{{id}}" -> Todo {{
    effect api.read
    effect storage.local
    action get Todo by id
  }}

  route POST "/todos" body input: CreateTodo -> Todo {{
    effect api.write
    effect storage.local
    action create Todo from input
  }}

  route PATCH "/todos/{{id}}/done" -> Todo {{
    effect api.write
    effect storage.local
    action update Todo by id set {{ done: true }}
  }}

  route DELETE "/todos/{{id}}" -> Status {{
    effect api.delete
    effect storage.local
    action delete Todo by id
  }}
}}

type CreateTodo = record {{
  title: Text {{ min: 1, max: 200 }}
}}
"#
    )
}

fn project_config_for_path(path: &Path) -> HayuloResult<Option<ProjectConfig>> {
    Ok(find_project_root(path)
        .map(|root| read_project_config(&root))
        .transpose()?)
}
