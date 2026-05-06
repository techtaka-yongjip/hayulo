use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::diagnostics::{Diagnostic, HayuloError, HayuloResult};

pub const CONFIG_NAME: &str = "hayulo.toml";
const DEFAULT_SKIP_DIRS: &[&str] = &[
    "generated",
    "__pycache__",
    ".git",
    ".venv",
    "venv",
    "node_modules",
];

#[derive(Clone, Debug)]
pub struct ProjectConfig {
    pub root: PathBuf,
    pub name: String,
    pub version: String,
    pub source_dirs: Vec<PathBuf>,
    pub test_dirs: Vec<PathBuf>,
    pub excludes: BTreeSet<PathBuf>,
    pub permissions: ProjectPermissions,
}

impl ProjectConfig {
    pub fn config_path(&self) -> PathBuf {
        self.root.join(CONFIG_NAME)
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProjectPermissions {
    pub allow: BTreeSet<String>,
    pub deny: BTreeSet<String>,
}

#[derive(Clone, Debug)]
enum TomlValue {
    String(String),
    Strings(Vec<String>),
}

pub fn load_project(start: &Path) -> HayuloResult<ProjectConfig> {
    let root = find_project_root(start).ok_or_else(|| {
        HayuloError::new(
            Diagnostic::new(
                "project.missing_config",
                format!(
                    "No {CONFIG_NAME} found for project target: {}.",
                    start.display()
                ),
            )
            .file(Some(&start.to_string_lossy()))
            .suggestion(format!(
                "Run hayulo new <name> or create {CONFIG_NAME} in the project root."
            )),
        )
    })?;
    read_project_config(&root)
}

pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut path = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    if path.is_file() {
        path = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    }
    let mut current = Some(path.as_path());
    while let Some(candidate) = current {
        if candidate.join(CONFIG_NAME).is_file() {
            return Some(candidate.to_path_buf());
        }
        current = candidate.parent();
    }
    None
}

pub fn read_project_config(root: &Path) -> HayuloResult<ProjectConfig> {
    let path = root.join(CONFIG_NAME);
    let data = parse_hayulo_toml(&path)?;
    let project = data.get("project");
    let name = project
        .and_then(|section| section.get("name"))
        .map(|value| as_string(value, "name", &path))
        .transpose()?
        .unwrap_or_else(|| {
            root.file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("app")
                .to_string()
        });
    let version = project
        .and_then(|section| section.get("version"))
        .map(|value| as_string(value, "version", &path))
        .transpose()?
        .unwrap_or_else(|| "0.1.0".to_string());
    let source_dirs = paths_from_value(project.and_then(|s| s.get("src")), root, &path, "src")?;
    let test_dirs = paths_from_value(project.and_then(|s| s.get("tests")), root, &path, "tests")?;
    let excludes = paths_from_value(
        project.and_then(|s| s.get("exclude")),
        root,
        &path,
        "exclude",
    )?
    .into_iter()
    .collect();
    let permissions = permissions_from_section(data.get("permissions"), &path)?;
    Ok(ProjectConfig {
        root: root.to_path_buf(),
        name,
        version,
        source_dirs,
        test_dirs,
        excludes,
        permissions,
    })
}

fn parse_hayulo_toml(path: &Path) -> HayuloResult<BTreeMap<String, BTreeMap<String, TomlValue>>> {
    let text = std::fs::read_to_string(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            HayuloError::new(
                Diagnostic::new("project.missing_config", format!("Missing {CONFIG_NAME}."))
                    .file(Some(&path.to_string_lossy())),
            )
        } else {
            HayuloError::new(
                Diagnostic::new(
                    "project.config_read_failed",
                    format!("Could not read {CONFIG_NAME}: {error}"),
                )
                .file(Some(&path.to_string_lossy())),
            )
        }
    })?;
    let mut data: BTreeMap<String, BTreeMap<String, TomlValue>> = BTreeMap::new();
    let mut section: Option<String> = None;
    for (index, raw) in text.lines().enumerate() {
        let line_no = index + 1;
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            let name = line[1..line.len() - 1].trim();
            if name.is_empty() {
                config_error(
                    path,
                    line_no,
                    "project.invalid_config",
                    "Empty TOML section name.",
                )?;
            }
            section = Some(name.to_string());
            data.entry(name.to_string()).or_default();
            continue;
        }
        let section_name = section.clone().ok_or_else(|| {
            HayuloError::new(
                Diagnostic::new(
                    "project.invalid_config",
                    "Expected a TOML section before key/value entries.",
                )
                .file(Some(&path.to_string_lossy()))
                .line(Some(line_no))
                .suggestion("Check hayulo.toml syntax."),
            )
        })?;
        let (key, raw_value) = line.split_once('=').ok_or_else(|| {
            HayuloError::new(
                Diagnostic::new(
                    "project.invalid_config",
                    format!("Invalid TOML line: {line:?}."),
                )
                .file(Some(&path.to_string_lossy()))
                .line(Some(line_no))
                .suggestion("Check hayulo.toml syntax."),
            )
        })?;
        let key = key.trim();
        if !is_identifier(key) {
            config_error(
                path,
                line_no,
                "project.invalid_config",
                &format!("Invalid TOML key: {key:?}."),
            )?;
        }
        let value = parse_value(raw_value.trim(), path, line_no)?;
        data.entry(section_name)
            .or_default()
            .insert(key.to_string(), value);
    }
    Ok(data)
}

fn parse_value(value: &str, path: &Path, line: usize) -> HayuloResult<TomlValue> {
    if value.starts_with('"') && value.ends_with('"') {
        return Ok(TomlValue::String(unquote(value, path, line)?));
    }
    if value.starts_with('[') && value.ends_with(']') {
        let inner = &value[1..value.len() - 1];
        let mut values = Vec::new();
        for raw in inner.split(',') {
            let item = raw.trim();
            if item.is_empty() {
                continue;
            }
            if !item.starts_with('"') || !item.ends_with('"') {
                config_error(
                    path,
                    line,
                    "project.invalid_config",
                    "Only arrays of strings are supported.",
                )?;
            }
            values.push(unquote(item, path, line)?);
        }
        return Ok(TomlValue::Strings(values));
    }
    config_error(
        path,
        line,
        "project.invalid_config",
        "Only quoted strings and string arrays are supported.",
    )?;
    unreachable!()
}

fn unquote(value: &str, path: &Path, line: usize) -> HayuloResult<String> {
    let body = &value[1..value.len() - 1];
    let mut result = String::new();
    let mut escaped = false;
    for ch in body.chars() {
        if escaped {
            result.push(match ch {
                'n' => '\n',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                other => other,
            });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else {
            result.push(ch);
        }
    }
    if escaped {
        config_error(
            path,
            line,
            "project.invalid_config",
            "Invalid string value.",
        )?;
    }
    Ok(result)
}

fn as_string(value: &TomlValue, key: &str, path: &Path) -> HayuloResult<String> {
    match value {
        TomlValue::String(value) => Ok(value.clone()),
        _ => Err(HayuloError::new(
            Diagnostic::new(
                "project.invalid_config",
                format!("Project field {key:?} must be a string."),
            )
            .file(Some(&path.to_string_lossy())),
        )),
    }
}

fn paths_from_value(
    value: Option<&TomlValue>,
    root: &Path,
    path: &Path,
    key: &str,
) -> HayuloResult<Vec<PathBuf>> {
    let values = match value {
        Some(TomlValue::String(value)) => vec![value.clone()],
        Some(TomlValue::Strings(values)) => values.clone(),
        None if key == "src" => vec!["src".to_string()],
        None if key == "tests" => vec!["tests".to_string()],
        None => Vec::new(),
    };
    if value.is_some() && values.is_empty() && key != "exclude" {
        return Err(HayuloError::new(
            Diagnostic::new(
                "project.invalid_config",
                format!("Project field {key:?} must be a string or array of strings."),
            )
            .file(Some(&path.to_string_lossy())),
        ));
    }
    Ok(values
        .into_iter()
        .map(|item| {
            let path = root.join(&item);
            path.canonicalize().unwrap_or(path)
        })
        .collect())
}

fn permissions_from_section(
    section: Option<&BTreeMap<String, TomlValue>>,
    path: &Path,
) -> HayuloResult<ProjectPermissions> {
    let mut permissions = ProjectPermissions::default();
    if let Some(section) = section {
        for key in section.keys() {
            if key != "allow" && key != "deny" {
                return Err(HayuloError::new(
                    Diagnostic::new(
                        "project.invalid_config",
                        format!("Unknown permissions field {key:?}."),
                    )
                    .file(Some(&path.to_string_lossy()))
                    .suggestion("Use [permissions] allow = [...] and deny = [...]."),
                ));
            }
        }
        permissions.allow = permission_set_from_value(section.get("allow"), path, "allow")?;
        permissions.deny = permission_set_from_value(section.get("deny"), path, "deny")?;
    }
    Ok(permissions)
}

fn permission_set_from_value(
    value: Option<&TomlValue>,
    path: &Path,
    key: &str,
) -> HayuloResult<BTreeSet<String>> {
    let values = match value {
        Some(TomlValue::String(value)) => vec![value.clone()],
        Some(TomlValue::Strings(values)) => values.clone(),
        None => Vec::new(),
    };
    let mut result = BTreeSet::new();
    for item in values {
        if !is_permission_name(&item) {
            return Err(HayuloError::new(
                Diagnostic::new(
                    "project.invalid_permission",
                    format!("Invalid permission name: {item:?}."),
                )
                .file(Some(&path.to_string_lossy()))
                .detail("permission", serde_json::json!(item))
                .suggestion("Use lowercase dotted names such as api.read or storage.local."),
            ));
        }
        result.insert(item);
    }
    if value.is_some() && result.is_empty() && key != "allow" && key != "deny" {
        return Err(HayuloError::new(
            Diagnostic::new(
                "project.invalid_config",
                format!("Permissions field {key:?} must be a string or array of strings."),
            )
            .file(Some(&path.to_string_lossy())),
        ));
    }
    Ok(result)
}

pub fn project_files(config: &ProjectConfig, include_tests: bool) -> Vec<PathBuf> {
    let mut roots = config.source_dirs.clone();
    if include_tests {
        roots.extend(config.test_dirs.clone());
    }
    let mut files = BTreeSet::new();
    for root in roots {
        if !root.exists() {
            continue;
        }
        if root.is_file()
            && root.extension().and_then(|v| v.to_str()) == Some("hayulo")
            && !is_excluded(&root, config)
        {
            files.insert(root.canonicalize().unwrap_or(root));
            continue;
        }
        if !root.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if path.is_file()
                && path.extension().and_then(|v| v.to_str()) == Some("hayulo")
                && !is_excluded(path, config)
            {
                files.insert(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
            }
        }
    }
    files.into_iter().collect()
}

pub fn is_excluded(path: &Path, config: &ProjectConfig) -> bool {
    if let Ok(relative) = path.strip_prefix(&config.root) {
        if relative
            .components()
            .filter_map(|component| component.as_os_str().to_str())
            .any(|part| DEFAULT_SKIP_DIRS.contains(&part) || part.starts_with('.'))
        {
            return true;
        }
    }
    config
        .excludes
        .iter()
        .any(|excluded| path == excluded || path.starts_with(excluded))
}

pub fn project_name_to_module(name: &str) -> String {
    let value = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if value.is_empty() {
        "app".to_string()
    } else {
        value
    }
}

pub fn project_relative(config: &ProjectConfig, path: &Path) -> String {
    path.strip_prefix(&config.root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

fn config_error(path: &Path, line: usize, code: &str, message: &str) -> HayuloResult<()> {
    Err(HayuloError::new(
        Diagnostic::new(code, message)
            .file(Some(&path.to_string_lossy()))
            .line(Some(line))
            .suggestion("Check hayulo.toml syntax."),
    ))
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn is_permission_name(value: &str) -> bool {
    let mut parts = value.split('.');
    let Some(first) = parts.next() else {
        return false;
    };
    !first.is_empty()
        && std::iter::once(first).chain(parts).all(|part| {
            let mut chars = part.chars();
            matches!(chars.next(), Some(ch) if ch.is_ascii_lowercase())
                && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
        })
}
