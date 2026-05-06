use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::Serialize;
use serde_json::{Value, json};

use crate::diagnostics::{Diagnostic, HayuloError, HayuloResult};
use crate::formatter::strip_comment;
use crate::project::ProjectPermissions;

const BUILTIN_TYPES: &[&str] = &[
    "Text", "Int", "Float", "Bool", "Time", "Email", "Status", "Any",
];

#[derive(Clone, Debug, Serialize)]
pub struct ApiField {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub constraints: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ApiRecord {
    pub name: String,
    pub fields: Vec<ApiField>,
    pub line: usize,
}

impl ApiRecord {
    fn field_names(&self) -> BTreeSet<String> {
        self.fields.iter().map(|field| field.name.clone()).collect()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ApiRoute {
    pub method: String,
    pub path: String,
    pub response_type: String,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_type: Option<String>,
    pub effects: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<ApiAction>,
}

impl ApiRoute {
    pub fn to_public_json(&self) -> Value {
        let mut data = serde_json::Map::new();
        data.insert("method".to_string(), json!(self.method));
        data.insert("path".to_string(), json!(self.path));
        data.insert("response_type".to_string(), json!(self.response_type));
        data.insert("line".to_string(), json!(self.line));
        if let Some(body_type) = &self.body_type {
            data.insert(
                "body".to_string(),
                json!({"name": self.body_name, "type": body_type}),
            );
        }
        if let Some(auth_type) = &self.auth_type {
            data.insert(
                "auth".to_string(),
                json!({"name": self.auth_name, "type": auth_type}),
            );
        }
        data.insert("effects".to_string(), json!(self.effects));
        if let Some(action) = &self.action {
            data.insert("action".to_string(), action.to_public_json());
        }
        Value::Object(data)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ApiAction {
    pub kind: String,
    pub record: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_name: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub updates: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

impl ApiAction {
    fn to_public_json(&self) -> Value {
        let mut data = serde_json::Map::new();
        data.insert("kind".to_string(), json!(self.kind));
        data.insert("record".to_string(), json!(self.record));
        if let Some(source) = &self.source {
            data.insert("source".to_string(), json!(source));
        }
        if let Some(id) = &self.id_name {
            data.insert("id".to_string(), json!(id));
        }
        if !self.updates.is_empty() {
            data.insert("updates".to_string(), json!(self.updates));
        }
        if let Some(line) = self.line {
            data.insert("line".to_string(), json!(line));
        }
        Value::Object(data)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ApiDatabase {
    pub kind: String,
    pub value: String,
    pub line: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct ApiOpenApi {
    pub title: String,
    pub version: String,
}

impl Default for ApiOpenApi {
    fn default() -> Self {
        Self {
            title: "Hayulo API".to_string(),
            version: "0.1.0".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ApiSpec {
    pub module: Option<String>,
    pub app_name: String,
    pub database: Option<ApiDatabase>,
    pub openapi: ApiOpenApi,
    pub records: BTreeMap<String, ApiRecord>,
    pub routes: Vec<ApiRoute>,
}

impl ApiSpec {
    pub fn to_json(&self) -> Value {
        json!({
            "module": self.module,
            "app": self.app_name,
            "database": self.database,
            "openapi": self.openapi,
            "records": self.records.values().collect::<Vec<_>>(),
            "routes": self.routes.iter().map(ApiRoute::to_public_json).collect::<Vec<_>>(),
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct GeneratedFile {
    pub path: PathBuf,
    pub description: String,
}

impl GeneratedFile {
    fn new(path: PathBuf, description: &str) -> Self {
        Self {
            path,
            description: description.to_string(),
        }
    }
}

pub fn looks_like_api_source(source: &str) -> bool {
    Regex::new(r"(?m)(^|\n)\s*app\s+[A-Za-z_]\w*\s*\{")
        .unwrap()
        .is_match(source)
}

pub fn parse_api_source(source: &str, filename: Option<&str>) -> HayuloResult<ApiSpec> {
    let spec = ApiSourceParser::new(source, filename).parse()?;
    check_api_spec(&spec, filename)?;
    Ok(spec)
}

struct ApiSourceParser<'a> {
    lines: Vec<&'a str>,
    filename: Option<&'a str>,
    module: Option<String>,
    app_name: Option<String>,
    database: Option<ApiDatabase>,
    openapi: ApiOpenApi,
    records: BTreeMap<String, ApiRecord>,
    routes: Vec<ApiRoute>,
}

impl<'a> ApiSourceParser<'a> {
    fn new(source: &'a str, filename: Option<&'a str>) -> Self {
        Self {
            lines: source.lines().collect(),
            filename,
            module: None,
            app_name: None,
            database: None,
            openapi: ApiOpenApi::default(),
            records: BTreeMap::new(),
            routes: Vec::new(),
        }
    }

    fn parse(mut self) -> HayuloResult<ApiSpec> {
        let lines = self.lines.clone();
        self.parse_lines(&lines, 1, false)?;
        let app_name = self.app_name.clone().ok_or_else(|| {
            HayuloError::new(
                Diagnostic::new(
                    "missing_app",
                    "Hayulo API source must declare an app block.",
                )
                .file(self.filename)
                .line(Some(1))
                .suggestion("Add: app TodoApi { ... }"),
            )
        })?;
        Ok(ApiSpec {
            module: self.module,
            app_name,
            database: self.database,
            openapi: self.openapi,
            records: self.records,
            routes: self.routes,
        })
    }

    fn parse_lines(&mut self, lines: &[&str], base_line: usize, in_app: bool) -> HayuloResult<()> {
        let mut i = 0usize;
        while i < lines.len() {
            let raw = lines[i];
            let line_no = base_line + i;
            let line = strip_comment(raw).trim().to_string();
            if line.is_empty() {
                i += 1;
                continue;
            }
            if !in_app {
                if let Some(captures) =
                    Regex::new(r"^module\s+([A-Za-z_]\w*(?:\.[A-Za-z_]\w*)*)\s*$")
                        .unwrap()
                        .captures(&line)
                {
                    self.module = Some(captures[1].to_string());
                    i += 1;
                    continue;
                }
            }
            if Regex::new(r"^intent\s*\{").unwrap().is_match(&line) {
                let (_, next) = collect_block(lines, i, self.filename, "intent")?;
                i = next;
                continue;
            }
            if !in_app {
                if let Some(captures) = Regex::new(r"^app\s+([A-Za-z_]\w*)\s*\{")
                    .unwrap()
                    .captures(&line)
                {
                    self.app_name = Some(captures[1].to_string());
                    let (block, next) = collect_block(lines, i, self.filename, "app")?;
                    let refs = block.iter().map(|line| line.as_str()).collect::<Vec<_>>();
                    self.parse_lines(&refs, line_no + 1, true)?;
                    i = next;
                    continue;
                }
            }
            if in_app && line.starts_with("database ") {
                self.database = Some(self.database(&line, line_no)?);
                i += 1;
                continue;
            }
            if in_app && Regex::new(r"^openapi\s*\{").unwrap().is_match(&line) {
                let (block, next) = collect_block(lines, i, self.filename, "openapi")?;
                self.openapi = self.openapi(&block, line_no + 1)?;
                i = next;
                continue;
            }
            if line.starts_with("type ") {
                let (block, next) = collect_block(lines, i, self.filename, "record")?;
                let record = self.record(&line, &block, line_no)?;
                if self.records.contains_key(&record.name) {
                    return self.err(
                        "duplicate_record",
                        format!("Record {:?} is already defined.", record.name),
                        line_no,
                        vec!["Rename or remove the duplicate record."],
                    );
                }
                self.records.insert(record.name.clone(), record);
                i = next;
                continue;
            }
            if in_app && line.starts_with("route ") {
                let (block, next) = collect_block(lines, i, self.filename, "route")?;
                self.routes.push(self.route(&line, &block, line_no)?);
                i = next;
                continue;
            }
            if line.starts_with("test ") {
                let (_, next) = collect_block(lines, i, self.filename, "test")?;
                i = next;
                continue;
            }
            let allowed = if in_app {
                "database, openapi, type, route, or intent"
            } else {
                "module, intent, app, or type"
            };
            return self.err(
                "unexpected_api_line",
                format!("Unexpected Hayulo API line: {line:?}."),
                line_no,
                vec![format!("Expected {allowed}.")],
            );
        }
        Ok(())
    }

    fn database(&self, line: &str, line_no: usize) -> HayuloResult<ApiDatabase> {
        let re = Regex::new(r#"^database\s+([A-Za-z_]\w*)\s+(?:"([^"]+)"|([^\s]+))\s*$"#).unwrap();
        let captures = re.captures(line).ok_or_else(|| {
            HayuloError::new(
                Diagnostic::new("invalid_database", "Invalid database declaration.")
                    .file(self.filename)
                    .line(Some(line_no))
                    .suggestion("Use: database sqlite \"todo.db\"."),
            )
        })?;
        Ok(ApiDatabase {
            kind: captures[1].to_string(),
            value: captures
                .get(2)
                .or_else(|| captures.get(3))
                .unwrap()
                .as_str()
                .to_string(),
            line: line_no,
        })
    }

    fn openapi(&self, block: &[String], base_line: usize) -> HayuloResult<ApiOpenApi> {
        let mut result = ApiOpenApi::default();
        let re = Regex::new(r#"^([A-Za-z_]\w*)\s*:\s*"([^"]*)"\s*$"#).unwrap();
        for (offset, raw) in block.iter().enumerate() {
            let line = strip_comment(raw).trim().trim_end_matches(',').to_string();
            if line.is_empty() {
                continue;
            }
            let captures = re.captures(&line).ok_or_else(|| {
                HayuloError::new(
                    Diagnostic::new(
                        "invalid_openapi_field",
                        format!("Invalid openapi line: {line:?}."),
                    )
                    .file(self.filename)
                    .line(Some(base_line + offset))
                    .suggestion("Use title: \"...\" or version: \"...\"."),
                )
            })?;
            match &captures[1] {
                "title" => result.title = captures[2].to_string(),
                "version" => result.version = captures[2].to_string(),
                key => {
                    return self.err(
                        "unknown_openapi_field",
                        format!("Unknown openapi field {key:?}."),
                        base_line + offset,
                        vec!["Supported fields are title and version."],
                    );
                }
            }
        }
        Ok(result)
    }

    fn record(&self, header: &str, block: &[String], line_no: usize) -> HayuloResult<ApiRecord> {
        let captures = Regex::new(r"^type\s+([A-Za-z_]\w*)\s*=\s*record\s*\{")
            .unwrap()
            .captures(header)
            .ok_or_else(|| {
                HayuloError::new(
                    Diagnostic::new("invalid_record_header", "Invalid record declaration.")
                        .file(self.filename)
                        .line(Some(line_no))
                        .suggestion("Use: type Todo = record { ... }."),
                )
            })?;
        let mut fields = Vec::new();
        for (offset, raw) in block.iter().enumerate() {
            let line = strip_comment(raw).trim().trim_end_matches(',').to_string();
            if line.is_empty() {
                continue;
            }
            let captures = Regex::new(r"^([A-Za-z_]\w*)\s*:\s*(.+)$")
                .unwrap()
                .captures(&line)
                .ok_or_else(|| {
                    HayuloError::new(
                        Diagnostic::new("invalid_field", format!("Invalid field line: {line:?}."))
                            .file(self.filename)
                            .line(Some(line_no + 1 + offset))
                            .suggestion("Use: name: Text { min: 1, max: 200 }."),
                    )
                })?;
            fields.push(parse_field(
                &captures[1],
                &captures[2],
                line_no + 1 + offset,
                self.filename,
            )?);
        }
        if fields.is_empty() {
            return self.err(
                "empty_record",
                format!("Record {} has no fields.", &captures[1]),
                line_no,
                vec!["Add at least one field."],
            );
        }
        Ok(ApiRecord {
            name: captures[1].to_string(),
            fields,
            line: line_no,
        })
    }

    fn route(&self, header: &str, block: &[String], line_no: usize) -> HayuloResult<ApiRoute> {
        let head = header
            .rsplit_once('{')
            .map(|(left, _)| left)
            .unwrap_or(header)
            .trim();
        let re = Regex::new(r#"^route\s+(GET|POST|PUT|PATCH|DELETE)\s+"([^"]+)"\s*(.*?)\s*->\s*([A-Za-z_]\w*(?:<[^>]+>)?)\s*$"#).unwrap();
        let captures = re.captures(head).ok_or_else(|| {
            HayuloError::new(
                Diagnostic::new("invalid_route", "Invalid route declaration.")
                    .file(self.filename)
                    .line(Some(line_no))
                    .suggestion("Use: route GET \"/todos\" -> List<Todo> { ... }."),
            )
        })?;
        let clauses = captures.get(3).map(|m| m.as_str()).unwrap_or("");
        let body_re =
            Regex::new(r"\bbody\s+([A-Za-z_]\w*)\s*:\s*([A-Za-z_]\w*(?:<[^>]+>)?)").unwrap();
        let auth_re =
            Regex::new(r"\bauth\s+([A-Za-z_]\w*)\s*:\s*([A-Za-z_]\w*(?:<[^>]+>)?)").unwrap();
        let (body_name, body_type) = body_re
            .captures(clauses)
            .map(|c| (Some(c[1].to_string()), Some(compact_type(&c[2]))))
            .unwrap_or((None, None));
        let (auth_name, auth_type) = auth_re
            .captures(clauses)
            .map(|c| (Some(c[1].to_string()), Some(compact_type(&c[2]))))
            .unwrap_or((None, None));
        let (effects, action) = self.route_body(block, line_no + 1)?;
        Ok(ApiRoute {
            method: captures[1].to_string(),
            path: captures[2].to_string(),
            response_type: compact_type(&captures[4]),
            line: line_no,
            body_name,
            body_type,
            auth_name,
            auth_type,
            effects,
            action: Some(action),
        })
    }

    fn route_body(
        &self,
        block: &[String],
        base_line: usize,
    ) -> HayuloResult<(Vec<String>, ApiAction)> {
        let mut effects = Vec::new();
        let mut action = None;
        let effect_re =
            Regex::new(r"^effect\s+([a-z][a-z0-9_]*(?:\.[a-z][a-z0-9_]*)*)\s*$").unwrap();
        for (offset, raw) in block.iter().enumerate() {
            let line_no = base_line + offset;
            let line = strip_comment(raw).trim().trim_end_matches(',').to_string();
            if line.is_empty() {
                continue;
            }
            if let Some(captures) = effect_re.captures(&line) {
                effects.push(captures[1].to_string());
                continue;
            }
            if line.starts_with("return ") || line.starts_with("db.") || line.contains("db.") {
                return self.err(
                    "route.body_requires_action",
                    "Hayulo 2.0 API routes use declarative actions instead of db calls.",
                    line_no,
                    vec!["Use effect declarations and an action such as: action create Todo from input."],
                );
            }
            if line.starts_with("action ") {
                if action.is_some() {
                    return self.err(
                        "route.multiple_actions",
                        "Route body can contain exactly one action.",
                        line_no,
                        vec!["Remove the extra action or split the route."],
                    );
                }
                action = Some(self.action(&line, line_no)?);
                continue;
            }
            return self.err(
                "route.invalid_body_declaration",
                format!("Invalid route body declaration: {line:?}."),
                line_no,
                vec!["Use effect <name> or action <kind> ..."],
            );
        }
        let Some(action) = action else {
            return self.err(
                "route.missing_action",
                "Route body must declare exactly one action.",
                base_line,
                vec!["Add an action declaration such as: action list Todo."],
            );
        };
        Ok((effects, action))
    }

    fn action(&self, line: &str, line_no: usize) -> HayuloResult<ApiAction> {
        for (re, kind) in [
            (r"^action\s+list\s+([A-Za-z_]\w*)\s*$", "list"),
            (
                r"^action\s+get\s+([A-Za-z_]\w*)\s+by\s+([A-Za-z_]\w*)\s*$",
                "get",
            ),
            (
                r"^action\s+create\s+([A-Za-z_]\w*)\s+from\s+([A-Za-z_]\w*)\s*$",
                "create",
            ),
            (
                r"^action\s+update\s+([A-Za-z_]\w*)\s+by\s+([A-Za-z_]\w*)\s+from\s+([A-Za-z_]\w*)\s*$",
                "update_from",
            ),
            (
                r"^action\s+update\s+([A-Za-z_]\w*)\s+by\s+([A-Za-z_]\w*)\s+set\s*\{(.+)\}\s*$",
                "update_set",
            ),
            (
                r"^action\s+delete\s+([A-Za-z_]\w*)\s+by\s+([A-Za-z_]\w*)\s*$",
                "delete",
            ),
        ] {
            let regex = Regex::new(re).unwrap();
            if let Some(c) = regex.captures(line) {
                return Ok(match kind {
                    "list" => ApiAction {
                        kind: "list".to_string(),
                        record: c[1].to_string(),
                        source: None,
                        id_name: None,
                        updates: BTreeMap::new(),
                        line: Some(line_no),
                    },
                    "get" => ApiAction {
                        kind: "get".to_string(),
                        record: c[1].to_string(),
                        source: None,
                        id_name: Some(c[2].to_string()),
                        updates: BTreeMap::new(),
                        line: Some(line_no),
                    },
                    "create" => ApiAction {
                        kind: "create".to_string(),
                        record: c[1].to_string(),
                        source: Some(c[2].to_string()),
                        id_name: None,
                        updates: BTreeMap::new(),
                        line: Some(line_no),
                    },
                    "update_from" => ApiAction {
                        kind: "update".to_string(),
                        record: c[1].to_string(),
                        source: Some(c[3].to_string()),
                        id_name: Some(c[2].to_string()),
                        updates: BTreeMap::new(),
                        line: Some(line_no),
                    },
                    "update_set" => ApiAction {
                        kind: "update".to_string(),
                        record: c[1].to_string(),
                        source: None,
                        id_name: Some(c[2].to_string()),
                        updates: parse_action_updates(&c[3], self.filename, line_no)?,
                        line: Some(line_no),
                    },
                    "delete" => ApiAction {
                        kind: "delete".to_string(),
                        record: c[1].to_string(),
                        source: None,
                        id_name: Some(c[2].to_string()),
                        updates: BTreeMap::new(),
                        line: Some(line_no),
                    },
                    _ => unreachable!(),
                });
            }
        }
        self.err(
            "route.invalid_action",
            format!("Invalid route action: {line:?}."),
            line_no,
            vec!["Supported actions are list, get, create, update, and delete."],
        )
    }

    fn err<T>(
        &self,
        code: &str,
        message: impl Into<String>,
        line: usize,
        suggestions: Vec<impl Into<String>>,
    ) -> HayuloResult<T> {
        Err(HayuloError::new(
            Diagnostic::new(code, message.into())
                .file(self.filename)
                .line(Some(line))
                .suggestions(suggestions),
        ))
    }
}

fn collect_block(
    lines: &[&str],
    start: usize,
    filename: Option<&str>,
    label: &str,
) -> HayuloResult<(Vec<String>, usize)> {
    if !strip_comment(lines[start]).contains('{') {
        return Err(HayuloError::new(
            Diagnostic::new("missing_block", format!("Expected '{{' for {label} block."))
                .file(filename)
                .line(Some(start + 1)),
        ));
    }
    let mut depth = count_braces(lines[start]);
    let mut block = Vec::new();
    let mut i = start + 1;
    while i < lines.len() {
        let delta = count_braces(lines[i]);
        if depth + delta <= 0 {
            let before = lines[i]
                .rsplit_once('}')
                .map(|(left, _)| left)
                .unwrap_or(lines[i])
                .trim();
            if !before.is_empty() {
                block.push(before.to_string());
            }
            return Ok((block, i + 1));
        }
        block.push(lines[i].to_string());
        depth += delta;
        i += 1;
    }
    Err(HayuloError::new(
        Diagnostic::new("unterminated_block", format!("Unterminated {label} block."))
            .file(filename)
            .line(Some(start + 1))
            .suggestion("Add a closing '}'."),
    ))
}

fn count_braces(line: &str) -> isize {
    let text = strip_comment(line);
    let mut in_string = false;
    let mut escaped = false;
    let mut count = 0isize;
    for ch in text.chars() {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            in_string = !in_string;
        } else if !in_string && ch == '{' {
            count += 1;
        } else if !in_string && ch == '}' {
            count -= 1;
        }
    }
    count
}

fn parse_field(
    name: &str,
    rhs: &str,
    line: usize,
    filename: Option<&str>,
) -> HayuloResult<ApiField> {
    let mut default = None;
    let mut constraints = BTreeMap::new();
    let rhs = rhs.trim();
    let type_name;
    if rhs.contains('{') {
        let (before, after) = rhs.split_once('{').unwrap();
        let (constraints_text, rest) = after.split_once('}').unwrap_or(("", after));
        type_name = compact_type(before);
        constraints = parse_constraint_block(constraints_text, filename, line)?;
        let rest = rest.trim();
        if !rest.is_empty() {
            if !rest.starts_with('=') {
                return Err(HayuloError::new(
                    Diagnostic::new("invalid_field", format!("Invalid field suffix: {rest:?}."))
                        .file(filename)
                        .line(Some(line))
                        .suggestion("Use: name: Text { min: 1, max: 200 } = default."),
                ));
            }
            default = Some(rest[1..].trim().to_string());
        }
    } else {
        let mut rhs_text = rhs.to_string();
        if let Some((left, right)) = rhs.split_once('=') {
            rhs_text = left.to_string();
            default = Some(right.trim().to_string());
        }
        let parts = rhs_text.split_whitespace().collect::<Vec<_>>();
        if parts.len() > 1 {
            return Err(HayuloError::new(
                Diagnostic::new(
                    "api.inline_constraints_removed",
                    "Hayulo 2.0 uses structured field constraint blocks.",
                )
                .file(filename)
                .line(Some(line))
                .suggestion("Use: title: Text { min: 1, max: 200 }."),
            ));
        }
        type_name = parts
            .first()
            .map(|part| compact_type(part))
            .unwrap_or_default();
    }
    if type_name.is_empty() {
        return Err(HayuloError::new(
            Diagnostic::new(
                "missing_field_type",
                format!("Field {name:?} is missing a type."),
            )
            .file(filename)
            .line(Some(line)),
        ));
    }
    Ok(ApiField {
        name: name.to_string(),
        type_name,
        line,
        default,
        constraints,
    })
}

fn parse_constraint_block(
    text: &str,
    filename: Option<&str>,
    line: usize,
) -> HayuloResult<BTreeMap<String, Value>> {
    let mut constraints = BTreeMap::new();
    for raw in text.split(',') {
        let part = raw.trim();
        if part.is_empty() {
            continue;
        }
        let (key, raw_value) = part.split_once(':').ok_or_else(|| {
            HayuloError::new(
                Diagnostic::new(
                    "invalid_constraint",
                    format!("Invalid constraint entry: {part:?}."),
                )
                .file(filename)
                .line(Some(line))
                .suggestion("Use key: value entries inside the constraint block."),
            )
        })?;
        let key = key.trim();
        let raw_value = raw_value.trim();
        if !matches!(key, "min" | "max" | "unique" | "private") {
            return Err(HayuloError::new(
                Diagnostic::new(
                    "unknown_field_constraint",
                    format!("Unknown field constraint {key:?}."),
                )
                .file(filename)
                .line(Some(line))
                .suggestion("Supported constraints are min, max, unique, and private."),
            ));
        }
        let value = if key == "unique" || key == "private" {
            match raw_value {
                "true" => json!(true),
                "false" => json!(false),
                _ => {
                    return Err(HayuloError::new(
                        Diagnostic::new(
                            "invalid_constraint_value",
                            format!("Constraint {key:?} must be true or false."),
                        )
                        .file(filename)
                        .line(Some(line)),
                    ));
                }
            }
        } else if raw_value.contains('.') {
            json!(raw_value.parse::<f64>().map_err(|_| {
                HayuloError::new(
                    Diagnostic::new(
                        "invalid_constraint_value",
                        format!("Constraint {key:?} must be numeric."),
                    )
                    .file(filename)
                    .line(Some(line)),
                )
            })?)
        } else {
            json!(raw_value.parse::<i64>().map_err(|_| {
                HayuloError::new(
                    Diagnostic::new(
                        "invalid_constraint_value",
                        format!("Constraint {key:?} must be numeric."),
                    )
                    .file(filename)
                    .line(Some(line)),
                )
            })?)
        };
        constraints.insert(key.to_string(), value);
    }
    Ok(constraints)
}

fn parse_action_updates(
    text: &str,
    filename: Option<&str>,
    line: usize,
) -> HayuloResult<BTreeMap<String, Value>> {
    let mut updates = BTreeMap::new();
    for raw in text.split(',') {
        let part = raw.trim();
        if part.is_empty() {
            continue;
        }
        let (key, raw_value) = part.split_once(':').ok_or_else(|| {
            HayuloError::new(
                Diagnostic::new(
                    "route.invalid_action_update",
                    format!("Invalid action update entry: {part:?}."),
                )
                .file(filename)
                .line(Some(line))
                .suggestion("Use field: value entries inside set { ... }."),
            )
        })?;
        let raw_value = raw_value.trim();
        let value = if raw_value == "true" {
            json!(true)
        } else if raw_value == "false" {
            json!(false)
        } else if raw_value.starts_with('"') && raw_value.ends_with('"') {
            json!(&raw_value[1..raw_value.len() - 1])
        } else if raw_value.contains('.') {
            json!(raw_value.parse::<f64>().map_err(|_| {
                HayuloError::new(
                    Diagnostic::new(
                        "route.invalid_action_update",
                        format!("Unsupported action update value: {raw_value:?}."),
                    )
                    .file(filename)
                    .line(Some(line))
                    .suggestion("Use Text, Bool, Int, or Float literal values in this draft."),
                )
            })?)
        } else {
            json!(raw_value.parse::<i64>().map_err(|_| {
                HayuloError::new(
                    Diagnostic::new(
                        "route.invalid_action_update",
                        format!("Unsupported action update value: {raw_value:?}."),
                    )
                    .file(filename)
                    .line(Some(line))
                    .suggestion("Use Text, Bool, Int, or Float literal values in this draft."),
                )
            })?)
        };
        updates.insert(key.trim().to_string(), value);
    }
    Ok(updates)
}

fn compact_type(value: &str) -> String {
    value.split_whitespace().collect()
}

fn split_generic_type(type_name: &str) -> (&str, Option<&str>) {
    if let Some(start) = type_name.find('<') {
        if type_name.ends_with('>') {
            return (
                &type_name[..start],
                Some(&type_name[start + 1..type_name.len() - 1]),
            );
        }
    }
    (type_name, None)
}

fn validate_type(
    type_name: &str,
    records: &BTreeMap<String, ApiRecord>,
    filename: Option<&str>,
    line: Option<usize>,
    context: &str,
) -> HayuloResult<()> {
    let (base, inner) = split_generic_type(type_name);
    if let Some(inner) = inner {
        if base != "List" && base != "Id" {
            return Err(HayuloError::new(
                Diagnostic::new(
                    "unsupported_generic_type",
                    format!("Unsupported generic type {base:?} in {context}."),
                )
                .file(filename)
                .line(line)
                .detail("type", json!(type_name))
                .suggestion("Supported generics are List<T> and Id<T>."),
            ));
        }
        return validate_type(inner, records, filename, line, context);
    }
    if BUILTIN_TYPES.contains(&type_name) || records.contains_key(type_name) {
        Ok(())
    } else {
        Err(HayuloError::new(
            Diagnostic::new(
                "unknown_type",
                format!("Unknown type {type_name:?} in {context}."),
            )
            .file(filename)
            .line(line)
            .detail("type", json!(type_name))
            .suggestion("Define this record or use Text, Int, Bool, Time, Email, Status."),
        ))
    }
}

fn response_record_name(response_type: &str) -> Option<String> {
    if response_type == "Status" {
        return None;
    }
    let (base, inner) = split_generic_type(response_type);
    if base == "List" {
        inner.map(ToOwned::to_owned)
    } else {
        Some(response_type.to_string())
    }
}

fn check_api_spec(spec: &ApiSpec, filename: Option<&str>) -> HayuloResult<()> {
    if spec.records.is_empty() {
        return Err(HayuloError::new(
            Diagnostic::new(
                "api_without_records",
                "API app must define at least one record type.",
            )
            .file(filename),
        ));
    }
    if spec.routes.is_empty() {
        return Err(HayuloError::new(
            Diagnostic::new(
                "api_without_routes",
                "API app must define at least one route.",
            )
            .file(filename),
        ));
    }
    for record in spec.records.values() {
        let mut seen = BTreeSet::new();
        for field in &record.fields {
            if !seen.insert(field.name.clone()) {
                return Err(HayuloError::new(
                    Diagnostic::new(
                        "duplicate_field",
                        format!("Record {} repeats field {:?}.", record.name, field.name),
                    )
                    .file(filename)
                    .line(Some(field.line)),
                ));
            }
            validate_type(
                &field.type_name,
                &spec.records,
                filename,
                Some(field.line),
                &format!("field {}.{}", record.name, field.name),
            )?;
            if let (Some(min), Some(max)) =
                (field.constraints.get("min"), field.constraints.get("max"))
            {
                if min.as_f64().unwrap_or(0.0) > max.as_f64().unwrap_or(0.0) {
                    return Err(HayuloError::new(
                        Diagnostic::new(
                            "invalid_constraint_range",
                            format!("Field {:?} has min greater than max.", field.name),
                        )
                        .file(filename)
                        .line(Some(field.line)),
                    ));
                }
            }
        }
    }
    for route in &spec.routes {
        if route.action.is_none() {
            return Err(HayuloError::new(
                Diagnostic::new(
                    "route.missing_action",
                    format!(
                        "Route {} {} must declare an action.",
                        route.method, route.path
                    ),
                )
                .file(filename)
                .line(Some(route.line)),
            ));
        }
        if route.response_type != "Status" {
            validate_type(
                &route.response_type,
                &spec.records,
                filename,
                Some(route.line),
                &format!("route {} {}", route.method, route.path),
            )?;
        }
        if let Some(body_type) = &route.body_type {
            validate_type(
                body_type,
                &spec.records,
                filename,
                Some(route.line),
                &format!("body of route {} {}", route.method, route.path),
            )?;
        }
        if route.method == "POST" && route.body_type.is_none() {
            return Err(HayuloError::new(
                Diagnostic::new(
                    "post_without_body",
                    format!("POST route {} must declare a body input type.", route.path),
                )
                .file(filename)
                .line(Some(route.line))
                .suggestion("Add: body input: CreateTodo."),
            ));
        }
        validate_route_action(route, &spec.records, filename)?;
        let missing = required_effects_for_action(route)
            .into_iter()
            .filter(|effect| !route.effects.contains(effect))
            .collect::<Vec<_>>();
        if let Some(permission) = missing.first() {
            return Err(HayuloError::new(
                Diagnostic::new(
                    "route.missing_effect",
                    format!(
                        "Route {} {} is missing required effect {:?}.",
                        route.method, route.path, permission
                    ),
                )
                .file(filename)
                .line(Some(route.line))
                .detail("missing", json!(missing))
                .detail("effects", json!(route.effects))
                .suggestion(format!("Add: effect {permission}")),
            ));
        }
        if let (Some(response), Some(body_type)) =
            (response_record_name(&route.response_type), &route.body_type)
        {
            if let (Some(body_record), Some(response_record)) =
                (spec.records.get(body_type), spec.records.get(&response))
            {
                let extra = body_record
                    .field_names()
                    .difference(&response_record.field_names())
                    .cloned()
                    .collect::<Vec<_>>();
                if !extra.is_empty() {
                    return Err(HayuloError::new(
                        Diagnostic::new("body_field_not_in_response_record", format!("Body type {body_type} contains fields missing from {response}: {}.", extra.join(", ")))
                            .file(filename)
                            .line(Some(route.line))
                            .detail("unknown_fields", json!(extra)),
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_route_action(
    route: &ApiRoute,
    records: &BTreeMap<String, ApiRecord>,
    filename: Option<&str>,
) -> HayuloResult<()> {
    let action = route.action.as_ref().unwrap();
    if !records.contains_key(&action.record) {
        return Err(HayuloError::new(
            Diagnostic::new(
                "route.unknown_action_record",
                format!("Unknown action record {:?}.", action.record),
            )
            .file(filename)
            .line(action.line.or(Some(route.line)))
            .detail("record", json!(action.record))
            .suggestion("Use a record declared in this API source."),
        ));
    }
    match action.kind.as_str() {
        "list" => {
            if !route.response_type.starts_with("List<") {
                return simple_route_error(
                    "route.action_response_mismatch",
                    "list actions must return List<Record>.",
                    filename,
                    route.line,
                );
            }
            if route.body_type.is_some() {
                return simple_route_error(
                    "route.action_body_mismatch",
                    "list actions do not use request bodies.",
                    filename,
                    route.line,
                );
            }
        }
        "get" => {
            require_path_param(route, action, filename)?;
            if route.response_type != action.record {
                return simple_route_error(
                    "route.action_response_mismatch",
                    "get actions must return the action record.",
                    filename,
                    route.line,
                );
            }
            if route.body_type.is_some() {
                return simple_route_error(
                    "route.action_body_mismatch",
                    "get actions do not use request bodies.",
                    filename,
                    route.line,
                );
            }
        }
        "create" => {
            if action.source != route.body_name {
                return Err(HayuloError::new(
                    Diagnostic::new(
                        "route.action_body_mismatch",
                        format!(
                            "Create action source {:?} must match body binding {:?}.",
                            action.source, route.body_name
                        ),
                    )
                    .file(filename)
                    .line(action.line.or(Some(route.line)))
                    .suggestion("Use: action create Record from input."),
                ));
            }
            if route.response_type != action.record {
                return simple_route_error(
                    "route.action_response_mismatch",
                    "create actions must return the action record.",
                    filename,
                    route.line,
                );
            }
        }
        "update" => {
            require_path_param(route, action, filename)?;
            if route.response_type != action.record {
                return simple_route_error(
                    "route.action_response_mismatch",
                    "update actions must return the action record.",
                    filename,
                    route.line,
                );
            }
            if action.source.is_some() && action.source != route.body_name {
                return Err(HayuloError::new(
                    Diagnostic::new(
                        "route.action_body_mismatch",
                        format!(
                            "Update action source {:?} must match body binding {:?}.",
                            action.source, route.body_name
                        ),
                    )
                    .file(filename)
                    .line(action.line.or(Some(route.line)))
                    .suggestion("Use: action update Record by id from input."),
                ));
            }
        }
        "delete" => {
            require_path_param(route, action, filename)?;
            if route.response_type != "Status" {
                return simple_route_error(
                    "route.action_response_mismatch",
                    "delete actions must return Status.",
                    filename,
                    route.line,
                );
            }
        }
        _ => {
            return Err(HayuloError::new(
                Diagnostic::new(
                    "route.unsupported_action",
                    format!("Unsupported route action {:?}.", action.kind),
                )
                .file(filename)
                .line(action.line.or(Some(route.line))),
            ));
        }
    }
    if (action.kind == "create" || action.kind == "update")
        && action.source.is_some()
        && route.body_type.is_none()
    {
        return simple_route_error(
            "route.action_requires_body",
            &format!(
                "{} action uses a body source but the route has no body declaration.",
                action.kind
            ),
            filename,
            action.line.unwrap_or(route.line),
        );
    }
    if !action.updates.is_empty() {
        let unknown = action
            .updates
            .keys()
            .filter(|key| !records[&action.record].field_names().contains(*key))
            .cloned()
            .collect::<Vec<_>>();
        if let Some(field) = unknown.first() {
            return Err(HayuloError::new(
                Diagnostic::new(
                    "route.unknown_update_field",
                    format!("Action update references unknown field {field:?}."),
                )
                .file(filename)
                .line(action.line.or(Some(route.line)))
                .detail("field", json!(field))
                .detail("record", json!(action.record)),
            ));
        }
    }
    Ok(())
}

fn require_path_param(
    route: &ApiRoute,
    action: &ApiAction,
    filename: Option<&str>,
) -> HayuloResult<()> {
    if let Some(id) = &action.id_name {
        if path_parameter_names(route).contains(id) {
            return Ok(());
        }
        return Err(HayuloError::new(
            Diagnostic::new(
                "route.action_missing_path_param",
                format!("Action references missing path parameter {id:?}."),
            )
            .file(filename)
            .line(action.line.or(Some(route.line)))
            .suggestion("Use a name present in the route path, such as {id}."),
        ));
    }
    Ok(())
}

fn simple_route_error<T>(
    code: &str,
    message: &str,
    filename: Option<&str>,
    line: usize,
) -> HayuloResult<T> {
    Err(HayuloError::new(
        Diagnostic::new(code, message)
            .file(filename)
            .line(Some(line)),
    ))
}

fn path_parameter_names(route: &ApiRoute) -> BTreeSet<String> {
    Regex::new(r"\{([A-Za-z_]\w*)\}")
        .unwrap()
        .captures_iter(&route.path)
        .map(|c| c[1].to_string())
        .collect()
}

fn singularize(value: &str) -> String {
    if let Some(stem) = value.strip_suffix("ies") {
        format!("{stem}y")
    } else if let Some(stem) = value.strip_suffix('s') {
        stem.to_string()
    } else {
        value.to_string()
    }
}

fn infer_route_record(route: &ApiRoute, records: &BTreeMap<String, ApiRecord>) -> Option<String> {
    if let Some(action) = &route.action {
        if records.contains_key(&action.record) {
            return Some(action.record.clone());
        }
    }
    if let Some(response) = response_record_name(&route.response_type) {
        if records.contains_key(&response) {
            return Some(response);
        }
    }
    let first = singularize(route.path.trim_matches('/').split('/').next().unwrap_or(""));
    records
        .keys()
        .find(|name| name.eq_ignore_ascii_case(&first))
        .cloned()
}

fn infer_action(route: &ApiRoute) -> String {
    if let Some(action) = &route.action {
        return action.kind.clone();
    }
    if route.method == "GET" && route.response_type.starts_with("List<") {
        "list".to_string()
    } else if route.method == "GET" {
        "get".to_string()
    } else if route.method == "POST" {
        "create".to_string()
    } else if route.method == "PATCH" && route.path.trim_end_matches('/').ends_with("/done") {
        "mark_done".to_string()
    } else if route.method == "PATCH" {
        "update".to_string()
    } else if route.method == "DELETE" {
        "delete".to_string()
    } else {
        "not_implemented".to_string()
    }
}

pub fn required_api_permissions(spec: &ApiSpec) -> Vec<String> {
    let mut required = BTreeSet::new();
    for route in &spec.routes {
        required.extend(route.effects.iter().cloned());
    }
    required.into_iter().collect()
}

fn required_effects_for_action(route: &ApiRoute) -> Vec<String> {
    let action = infer_action(route);
    let mut required = BTreeSet::from(["storage.local".to_string()]);
    if action == "list" || action == "get" {
        required.insert("api.read".to_string());
    } else if action == "create" || action == "update" {
        required.insert("api.write".to_string());
    } else if action == "delete" {
        required.insert("api.delete".to_string());
    }
    required.into_iter().collect()
}

pub fn check_api_permissions(
    spec: &ApiSpec,
    permissions: &ProjectPermissions,
    filename: Option<&str>,
) -> HayuloResult<()> {
    let required = required_api_permissions(spec);
    let denied = required
        .iter()
        .filter(|permission| permissions.deny.contains(*permission))
        .cloned()
        .collect::<Vec<_>>();
    if let Some(permission) = denied.first() {
        return Err(HayuloError::new(
            Diagnostic::new("permission.denied", format!("Project deny-list blocks required permission {permission:?}."))
                .file(filename)
                .detail("permission", json!(permission))
                .detail("required", json!(required))
                .detail("deny", json!(permissions.deny))
                .suggestion("Remove the denied API action or update hayulo.toml if this behavior is intentional."),
        ));
    }
    let missing = required
        .iter()
        .filter(|permission| !permissions.allow.contains(*permission))
        .cloned()
        .collect::<Vec<_>>();
    if let Some(permission) = missing.first() {
        return Err(HayuloError::new(
            Diagnostic::new("permission.missing", format!("Project is missing required permission {permission:?}."))
                .file(filename)
                .detail("permission", json!(permission))
                .detail("required", json!(required))
                .detail("allow", json!(permissions.allow))
                .suggestion(format!("Add {permission:?} to [permissions].allow in hayulo.toml if this generated behavior is intended.")),
        ));
    }
    Ok(())
}

fn openapi_type(type_name: &str) -> Value {
    let (base, inner) = split_generic_type(type_name);
    if base == "List" {
        return json!({"type": "array", "items": openapi_type(inner.unwrap_or("Any"))});
    }
    if base == "Id" {
        return json!({"type": "integer"});
    }
    match type_name {
        "Text" => json!({"type": "string"}),
        "Email" => json!({"type": "string", "format": "email"}),
        "Int" => json!({"type": "integer"}),
        "Float" => json!({"type": "number"}),
        "Bool" => json!({"type": "boolean"}),
        "Time" => json!({"type": "string", "format": "date-time"}),
        "Status" => json!({"type": "object"}),
        _ => json!({"$ref": format!("#/components/schemas/{type_name}")}),
    }
}

pub fn build_openapi(spec: &ApiSpec) -> Value {
    let mut schemas = serde_json::Map::new();
    for record in spec.records.values() {
        let mut props = serde_json::Map::new();
        let mut required = Vec::new();
        for field in &record.fields {
            if field.constraints.get("private").and_then(Value::as_bool) == Some(true) {
                continue;
            }
            let mut schema = openapi_type(&field.type_name);
            if let Some(min) = field.constraints.get("min") {
                if schema.get("type").and_then(Value::as_str) == Some("string") {
                    schema["minLength"] = min.clone();
                } else {
                    schema["minimum"] = min.clone();
                }
            }
            if let Some(max) = field.constraints.get("max") {
                if schema.get("type").and_then(Value::as_str) == Some("string") {
                    schema["maxLength"] = max.clone();
                } else {
                    schema["maximum"] = max.clone();
                }
            }
            props.insert(field.name.clone(), schema);
            if field.default.is_none() && !field.type_name.starts_with("Id<") {
                required.push(field.name.clone());
            }
        }
        schemas.insert(
            record.name.clone(),
            json!({"type": "object", "additionalProperties": false, "properties": props, "required": required}),
        );
    }
    schemas.insert(
        "ErrorResponse".to_string(),
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "error": {
                    "type": "object",
                    "additionalProperties": true,
                    "properties": {"code": {"type": "string"}, "message": {"type": "string"}, "details": {}},
                    "required": ["code", "message"]
                }
            },
            "required": ["error"]
        }),
    );
    let mut paths = serde_json::Map::new();
    paths.insert(
        "/health".to_string(),
        json!({"get": {"operationId": "get_health", "summary": "Check API health", "responses": {"200": {"description": "API is healthy", "content": {"application/json": {"schema": {"type": "object", "properties": {"status": {"type": "string"}, "app": {"type": "string"}}, "required": ["status", "app"]}}}}}}}),
    );
    paths.insert(
        "/openapi.json".to_string(),
        json!({"get": {"operationId": "get_openapi_json", "summary": "Return this OpenAPI document", "responses": {"200": {"description": "OpenAPI document"}}}}),
    );
    for route in &spec.routes {
        let success = success_status_code(route);
        let mut op = json!({
            "operationId": operation_id(route),
            "summary": route_summary(route, &spec.records),
            "parameters": path_parameters(route),
            "responses": {
                success.clone(): response_for_status(&success, &route.response_type),
                "400": error_response("Bad request or validation failure"),
                "404": error_response("Route or resource not found")
            }
        });
        if route.method == "DELETE" {
            op["responses"] = json!({"204": {"description": "Deleted"}, "404": error_response("Resource not found")});
        }
        if let Some(body_type) = &route.body_type {
            op["requestBody"] = json!({"required": true, "content": {"application/json": {"schema": openapi_type(body_type)}}});
        }
        let path_entry = paths.entry(route.path.clone()).or_insert_with(|| json!({}));
        path_entry[route.method.to_lowercase()] = op;
    }
    json!({"openapi": "3.1.0", "info": spec.openapi, "paths": paths, "components": {"schemas": schemas}})
}

fn success_status_code(route: &ApiRoute) -> String {
    if route.method == "POST" {
        "201".to_string()
    } else if route.method == "DELETE" {
        "204".to_string()
    } else {
        "200".to_string()
    }
}

fn response_for_status(status: &str, type_name: &str) -> Value {
    if status == "204" {
        json!({"description": "No content"})
    } else {
        json!({"description": if status == "201" { "Created" } else { "OK" }, "content": {"application/json": {"schema": openapi_type(type_name)}}})
    }
}

fn error_response(description: &str) -> Value {
    json!({"description": description, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ErrorResponse"}}}})
}

fn path_parameters(route: &ApiRoute) -> Vec<Value> {
    path_parameter_names(route)
        .into_iter()
        .map(|name| {
            json!({
                "name": name,
                "in": "path",
                "required": true,
                "schema": if name == "id" { json!({"type": "integer"}) } else { json!({"type": "string"}) },
            })
        })
        .collect()
}

fn route_summary(route: &ApiRoute, records: &BTreeMap<String, ApiRecord>) -> String {
    let action = infer_action(route).replace('_', " ");
    if let Some(record) = infer_route_record(route, records) {
        format!("{} {record}", title_case(&action))
    } else {
        format!("{} resource", title_case(&action))
    }
}

fn title_case(value: &str) -> String {
    value
        .split_whitespace()
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
        .collect::<Vec<_>>()
        .join(" ")
}

fn operation_id(route: &ApiRoute) -> String {
    let path = route
        .path
        .trim_matches('/')
        .replace('{', "by_")
        .replace('}', "")
        .replace(['/', '-'], "_");
    format!(
        "{}_{}",
        route.method.to_lowercase(),
        if path.is_empty() {
            "root".to_string()
        } else {
            path
        }
    )
}

pub fn generate_api(
    spec: &ApiSpec,
    out_dir: &Path,
    clean: bool,
) -> HayuloResult<Vec<GeneratedFile>> {
    if clean && out_dir.exists() {
        std::fs::remove_dir_all(out_dir)
            .map_err(|error| io_error("api.output_remove_failed", out_dir, error))?;
    }
    std::fs::create_dir_all(out_dir)
        .map_err(|error| io_error("api.output_create_failed", out_dir, error))?;
    let routes = generated_routes(spec);
    let mut ir = spec.to_json();
    ir["routes"] = Value::Array(routes.clone());
    let files = vec![
        write_file(
            out_dir.join("hayulo.ir.json"),
            &serde_json::to_string_pretty(&ir).unwrap(),
            "Hayulo API intermediate representation",
        )?,
        write_file(
            out_dir.join("openapi.json"),
            &serde_json::to_string_pretty(&build_openapi(spec)).unwrap(),
            "OpenAPI 3.1 specification",
        )?,
        write_file(
            out_dir.join("package.json"),
            &generated_package_json(spec),
            "Node package manifest",
        )?,
        write_file(
            out_dir.join("server.mjs"),
            &generated_server(spec, &routes),
            "Runnable REST API server",
        )?,
        write_file(
            out_dir.join("smoke_test.mjs"),
            &generated_smoke_test(spec, &routes),
            "Generated smoke test",
        )?,
        write_file(
            out_dir.join("README.md"),
            &generated_readme(spec),
            "Generated API instructions",
        )?,
    ];
    Ok(files)
}

fn write_file(path: PathBuf, content: &str, description: &str) -> HayuloResult<GeneratedFile> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| io_error("api.output_create_failed", parent, error))?;
    }
    let mut text = content.to_string();
    if !text.ends_with('\n') {
        text.push('\n');
    }
    std::fs::write(&path, text)
        .map_err(|error| io_error("api.output_write_failed", &path, error))?;
    Ok(GeneratedFile::new(path, description))
}

fn io_error(code: &str, path: &Path, error: std::io::Error) -> HayuloError {
    HayuloError::new(
        Diagnostic::new(
            code,
            format!("Could not write API output {}: {error}.", path.display()),
        )
        .file(Some(&path.to_string_lossy())),
    )
}

fn generated_routes(spec: &ApiSpec) -> Vec<Value> {
    spec.routes
        .iter()
        .map(|route| {
            let mut data = route.to_public_json();
            data["action"] = json!(infer_action(route));
            data["record"] = json!(infer_route_record(route, &spec.records));
            data["updates"] = json!(
                route
                    .action
                    .as_ref()
                    .map(|a| &a.updates)
                    .cloned()
                    .unwrap_or_default()
            );
            data
        })
        .collect()
}

fn generated_package_json(spec: &ApiSpec) -> String {
    let name = spec
        .app_name
        .to_ascii_lowercase()
        .chars()
        .map(|ch| {
            if ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    serde_json::to_string_pretty(&json!({
        "name": if name.is_empty() { "hayulo-api" } else { &name },
        "version": spec.openapi.version,
        "private": true,
        "type": "module",
        "scripts": {"start": "node server.mjs", "dev": "node server.mjs", "test": "node smoke_test.mjs"}
    }))
    .unwrap()
}

fn generated_server(spec: &ApiSpec, routes: &[Value]) -> String {
    let meta = json!({
        "app": spec.app_name,
        "database": spec.database.clone().unwrap_or(ApiDatabase { kind: "memory".to_string(), value: "memory".to_string(), line: 0 }),
        "records": spec.records,
        "routes": routes,
    });
    format!(
        r#"// Generated by Hayulo. Edit the .hayulo source, not this file.
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import {{ fileURLToPath, pathToFileURL }} from 'node:url';
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const META = {meta};
const OPENAPI = {openapi};
const DB_FILE = path.join(__dirname, `${{META.database.value || META.app}}.json`);
function emptyStore(){{const data={{}},nextIds={{}};for(const n of Object.keys(META.records)){{data[n]=[];nextIds[n]=1;}}return{{data,nextIds}};}}
function loadStore(){{if(!fs.existsSync(DB_FILE))return emptyStore();try{{const p=JSON.parse(fs.readFileSync(DB_FILE,'utf8'));const e=emptyStore();return{{data:{{...e.data,...(p.data||{{}})}},nextIds:{{...e.nextIds,...(p.nextIds||{{}})}}}};}}catch{{return emptyStore();}}}}
let store=loadStore();
function save(){{fs.writeFileSync(DB_FILE,JSON.stringify(store,null,2)+'\n');}}
function send(res,status,body){{res.statusCode=status;res.setHeader('Access-Control-Allow-Origin','*');res.setHeader('Access-Control-Allow-Methods','GET,POST,PUT,PATCH,DELETE,OPTIONS');res.setHeader('Access-Control-Allow-Headers','content-type,authorization');if(status===204){{res.end();return;}}res.setHeader('content-type','application/json; charset=utf-8');res.end(JSON.stringify(body,null,2));}}
function error(res,status,code,message,details){{send(res,status,{{error:{{code,message,...(details?{{details}}:{{}})}}}});}}
function splitPath(v){{return v.replace(/^\/+|\/+$/g,'').split('/').filter(Boolean);}}
function matchRoute(pattern,actual){{const p=splitPath(pattern),a=splitPath(actual);if(p.length!==a.length)return null;const params={{}};for(let i=0;i<p.length;i++){{if(p[i].startsWith('{{')&&p[i].endsWith('}}'))params[p[i].slice(1,-1)]=decodeURIComponent(a[i]);else if(p[i]!==a[i])return null;}}return params;}}
async function readBody(req){{const chunks=[];for await(const c of req)chunks.push(c);const text=Buffer.concat(chunks).toString('utf8').trim();if(!text)return{{}};try{{return JSON.parse(text);}}catch{{const e=new Error('Request body must be valid JSON.');e.status=400;e.code='invalid_json';throw e;}}}}
function typeCheck(v,t){{if(t.startsWith('Id<'))return Number.isInteger(v);if(['Text','Email','Time'].includes(t))return typeof v==='string';if(t==='Int')return Number.isInteger(v);if(t==='Float')return typeof v==='number'&&Number.isFinite(v);if(t==='Bool')return typeof v==='boolean';return true;}}
function validateBody(typeName,input){{const record=META.records[typeName];if(!record)return{{ok:false,errors:[`Unknown body type ${{typeName}}.`]}};const value={{}},errors=[];for(const f of record.fields){{const present=Object.prototype.hasOwnProperty.call(input,f.name);if(!present){{if(f.default===undefined||f.default===null)errors.push(`Missing required field ${{f.name}}.`);continue;}}const raw=input[f.name];if(!typeCheck(raw,f.type)){{errors.push(`Field ${{f.name}} must be ${{f.type}}.`);continue;}}if(f.constraints?.min!==undefined){{if(typeof raw==='string'&&raw.length<f.constraints.min)errors.push(`Field ${{f.name}} is shorter than ${{f.constraints.min}}.`);if(typeof raw==='number'&&raw<f.constraints.min)errors.push(`Field ${{f.name}} is less than ${{f.constraints.min}}.`);}}if(f.constraints?.max!==undefined){{if(typeof raw==='string'&&raw.length>f.constraints.max)errors.push(`Field ${{f.name}} is longer than ${{f.constraints.max}}.`);if(typeof raw==='number'&&raw>f.constraints.max)errors.push(`Field ${{f.name}} is greater than ${{f.constraints.max}}.`);}}value[f.name]=raw;}}return errors.length?{{ok:false,errors}}:{{ok:true,value}};}}
function defaultValue(f,recordName){{if(f.type.startsWith('Id<')){{const id=store.nextIds[recordName]||1;store.nextIds[recordName]=id+1;return id;}}if(f.default==='now()'||f.type==='Time')return new Date().toISOString();if(f.default==='false')return false;if(f.default==='true')return true;if(f.default!==undefined&&f.default!==null){{if(/^-?\d+$/.test(f.default))return Number.parseInt(f.default,10);if(/^-?\d+\.\d+$/.test(f.default))return Number.parseFloat(f.default);return String(f.default).replace(/^"|"$/g,'');}}return null;}}
function createRecord(recordName,input){{const record=META.records[recordName],item={{}};for(const f of record.fields){{item[f.name]=Object.prototype.hasOwnProperty.call(input,f.name)?input[f.name]:defaultValue(f,recordName);}}store.data[recordName].push(item);save();return item;}}
function findById(recordName,id){{const n=Number.parseInt(id,10);return store.data[recordName].find(x=>x.id===n);}}
async function handle(route,params,req,res){{const recordName=route.record;if(route.action==='list')return send(res,200,store.data[recordName]||[]);if(route.action==='get'){{const item=findById(recordName,params.id);return item?send(res,200,item):error(res,404,'not_found',`${{recordName}} not found.`);}}if(route.action==='create'){{const body=await readBody(req),v=validateBody(route.body.type,body);return v.ok?send(res,201,createRecord(recordName,v.value)):error(res,400,'validation_failed','Request body failed validation.',v.errors);}}if(route.action==='update'){{const item=findById(recordName,params.id);if(!item)return error(res,404,'not_found',`${{recordName}} not found.`);if(route.updates&&Object.keys(route.updates).length){{Object.assign(item,route.updates);save();return send(res,200,item);}}const body=await readBody(req),v=validateBody(route.body.type,body);if(!v.ok)return error(res,400,'validation_failed','Request body failed validation.',v.errors);Object.assign(item,v.value);save();return send(res,200,item);}}if(route.action==='delete'){{const n=Number.parseInt(params.id,10),before=store.data[recordName].length;store.data[recordName]=store.data[recordName].filter(x=>x.id!==n);if(store.data[recordName].length===before)return error(res,404,'not_found',`${{recordName}} not found.`);save();return send(res,204,null);}}return error(res,501,'not_implemented',`Route action ${{route.action}} is not implemented.`);}}
export function createServer(){{return http.createServer(async(req,res)=>{{try{{if(req.method==='OPTIONS')return send(res,204,null);const url=new URL(req.url||'/','http://localhost');if(req.method==='GET'&&url.pathname==='/health')return send(res,200,{{status:'ok',app:META.app}});if(req.method==='GET'&&url.pathname==='/openapi.json')return send(res,200,OPENAPI);for(const r of META.routes){{if(r.method!==req.method)continue;const params=matchRoute(r.path,url.pathname);if(params)return await handle(r,params,req,res);}}return error(res,404,'route_not_found',`No route matches ${{req.method}} ${{url.pathname}}.`);}}catch(e){{return error(res,e.status||500,e.code||'internal_error',e.message||'Internal error.');}}}});}}
export function start(port=Number.parseInt(process.env.PORT||'3000',10)){{const s=createServer();s.listen(port,()=>{{console.log(`${{META.app}} listening on http://localhost:${{port}}`);console.log(`OpenAPI: http://localhost:${{port}}/openapi.json`);}});return s;}}
if(import.meta.url===pathToFileURL(process.argv[1]).href){{start();}}
"#,
        meta = serde_json::to_string_pretty(&meta).unwrap(),
        openapi = serde_json::to_string_pretty(&build_openapi(spec)).unwrap()
    )
}

fn sample_value(field: &ApiField) -> Value {
    match field.type_name.as_str() {
        "Text" | "Email" => {
            if field.name == "title" {
                json!("Build Hayulo")
            } else if field.name == "email" || field.type_name == "Email" {
                json!("ada@example.com")
            } else {
                json!(format!("sample {}", field.name))
            }
        }
        "Bool" => json!(false),
        "Int" | "Float" => json!(1),
        "Time" => json!("2026-01-01T00:00:00.000Z"),
        value if value.starts_with("Id<") => json!(1),
        _ => json!(format!("sample {}", field.name)),
    }
}

fn generated_smoke_test(spec: &ApiSpec, routes: &[Value]) -> String {
    let list_route = routes.iter().find(|r| r["action"] == "list");
    let get_route = routes.iter().find(|r| r["action"] == "get");
    let create_route = routes.iter().find(|r| r["action"] == "create");
    let done_route = routes
        .iter()
        .find(|r| r["action"] == "update" && r["updates"]["done"] == true);
    let delete_route = routes.iter().find(|r| r["action"] == "delete");
    let mut body = serde_json::Map::new();
    if let Some(route) = create_route {
        if let Some(body_type) = route["body"]["type"].as_str() {
            if let Some(record) = spec.records.get(body_type) {
                for field in &record.fields {
                    body.insert(field.name.clone(), sample_value(field));
                }
            }
        }
    }
    let mut tests = vec![
        "const health=await request('/health');".to_string(),
        "assert.equal(health.status,200);".to_string(),
        format!("assert.equal(health.body.app,{});", json!(spec.app_name)),
        "const openapi=await request('/openapi.json');".to_string(),
        "assert.equal(openapi.status,200);".to_string(),
        "assert.equal(openapi.body.openapi,'3.1.0');".to_string(),
    ];
    if let Some(route) = list_route {
        let path = route["path"].as_str().unwrap();
        tests.extend([
            format!("assert.ok(openapi.body.paths[{}]);", json!(path)),
            format!("const listBefore=await request({});", json!(path)),
            "assert.equal(listBefore.status,200);".to_string(),
            "assert.ok(Array.isArray(listBefore.body));".to_string(),
        ]);
    }
    if let Some(route) = create_route {
        let path = route["path"].as_str().unwrap();
        tests.extend([
            format!(
                "const invalidCreate=await request({},{{method:'POST',body:{{}}}});",
                json!(path)
            ),
            "assert.equal(invalidCreate.status,400);".to_string(),
            format!(
                "const created=await request({},{{method:'POST',body:{}}});",
                json!(path),
                Value::Object(body.clone())
            ),
            "assert.equal(created.status,201);".to_string(),
            "const createdId=created.body.id;".to_string(),
        ]);
        for (key, value) in &body {
            tests.push(format!(
                "assert.equal(created.body[{}],{});",
                json!(key),
                value
            ));
        }
    } else {
        tests.push("const createdId=1;".to_string());
    }
    if let Some(route) = get_route {
        let path = route["path"]
            .as_str()
            .unwrap()
            .replace("{id}", "${createdId}");
        tests.extend([
            format!("const fetched=await request(`{path}`);"),
            "assert.equal(fetched.status,200);".to_string(),
            "assert.equal(fetched.body.id,createdId);".to_string(),
        ]);
    }
    if let (Some(list), Some(_create)) = (list_route, create_route) {
        let path = list["path"].as_str().unwrap();
        tests.extend([
            format!("const listAfter=await request({});", json!(path)),
            "assert.equal(listAfter.status,200);".to_string(),
            "assert.ok(listAfter.body.some(item=>item.id===createdId));".to_string(),
        ]);
    }
    if let Some(route) = done_route {
        let path = route["path"]
            .as_str()
            .unwrap()
            .replace("{id}", "${createdId}");
        tests.extend([
            format!("const markedDone=await request(`{path}`,{{method:'PATCH'}});"),
            "assert.equal(markedDone.status,200);".to_string(),
            "assert.equal(markedDone.body.done,true);".to_string(),
        ]);
    }
    if let Some(route) = delete_route {
        let path = route["path"]
            .as_str()
            .unwrap()
            .replace("{id}", "${createdId}");
        tests.extend([
            format!("const deleted=await request(`{path}`,{{method:'DELETE'}});"),
            "assert.equal(deleted.status,204);".to_string(),
        ]);
    }
    if let (Some(get), Some(_delete)) = (get_route, delete_route) {
        let path = get["path"]
            .as_str()
            .unwrap()
            .replace("{id}", "${createdId}");
        tests.extend([
            format!("const fetchedAfterDelete=await request(`{path}`);"),
            "assert.equal(fetchedAfterDelete.status,404);".to_string(),
        ]);
    }
    format!(
        r#"// Generated by Hayulo. Basic integration smoke test.
import assert from 'node:assert/strict';
import http from 'node:http';
import {{ createServer }} from './server.mjs';
const server=createServer();
await new Promise(resolve=>server.listen(0,resolve));
const port=server.address().port;
function request(path,options={{}}){{return new Promise((resolve,reject)=>{{const payload=options.body===undefined?null:JSON.stringify(options.body);const req=http.request({{hostname:'127.0.0.1',port,path,method:options.method||'GET',headers:{{'content-type':'application/json',...(payload?{{'content-length':Buffer.byteLength(payload)}}:{{}})}}}},res=>{{let text='';res.on('data',chunk=>text+=chunk);res.on('end',()=>resolve({{status:res.statusCode,body:text?JSON.parse(text):null}}));}});req.on('error',reject);if(payload)req.write(payload);req.end();}});}}
try{{
  {}
  console.log('Hayulo generated API smoke test passed.');
}}finally{{
  await new Promise(resolve=>server.close(resolve));
}}
"#,
        tests.join("\n  ")
    )
}

fn generated_readme(spec: &ApiSpec) -> String {
    let routes = spec
        .routes
        .iter()
        .map(|route| {
            format!(
                "- `{} {}` -> `{}`",
                route.method, route.path, route.response_type
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"# {} generated API

This directory was generated by Hayulo from a `.hayulo` API source file.

## Run

```bash
npm start
```

## Test

```bash
npm test
```

## Endpoints

- `GET /health`
- `GET /openapi.json`
{}

This first MVP generator uses Node.js built-ins and a local JSON file store so the REST API can run without external dependencies. Future Hayulo versions can target TypeScript, Hono/Fastify, real SQLite migrations, auth adapters, and deployment targets.
"#,
        spec.app_name, routes
    )
}
