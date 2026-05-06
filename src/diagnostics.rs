use serde_json::{Map, Value, json};

pub const DIAGNOSTIC_SCHEMA: &str = "hayulo.diagnostics@0.1";
pub const TEST_SCHEMA: &str = "hayulo.test@0.1";

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub details: Map<String, Value>,
    pub suggestions: Vec<String>,
}

impl Diagnostic {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            file: None,
            line: None,
            column: None,
            details: Map::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn file(mut self, file: Option<&str>) -> Self {
        self.file = file.map(ToOwned::to_owned);
        self
    }

    pub fn line(mut self, line: Option<usize>) -> Self {
        self.line = line;
        self
    }

    pub fn column(mut self, column: Option<usize>) -> Self {
        self.column = column;
        self
    }

    pub fn suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    pub fn suggestions(mut self, suggestions: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.suggestions = suggestions.into_iter().map(Into::into).collect();
        self
    }

    pub fn detail(mut self, key: impl Into<String>, value: Value) -> Self {
        self.details.insert(key.into(), value);
        self
    }

    pub fn to_legacy_json(&self) -> Value {
        let mut data = Map::new();
        data.insert("code".to_string(), json!(self.code));
        data.insert("message".to_string(), json!(self.message));
        if let Some(file) = &self.file {
            data.insert("file".to_string(), json!(file));
        }
        if let Some(line) = self.line {
            data.insert("line".to_string(), json!(line));
        }
        if let Some(column) = self.column {
            data.insert("column".to_string(), json!(column));
        }
        if !self.details.is_empty() {
            data.insert("details".to_string(), Value::Object(self.details.clone()));
        }
        if !self.suggestions.is_empty() {
            data.insert("suggestions".to_string(), json!(self.suggestions));
        }
        Value::Object(data)
    }

    pub fn to_schema_json(&self) -> Value {
        json!({
            "code": self.code,
            "severity": "error",
            "message": self.message,
            "location": {
                "file": self.file,
                "line": self.line,
                "column": self.column,
            },
            "details": self.details,
            "suggestions": self.suggestions.iter().map(|message| json!({"message": message})).collect::<Vec<_>>(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct HayuloError {
    pub diagnostic: Diagnostic,
}

impl HayuloError {
    pub fn new(diagnostic: Diagnostic) -> Self {
        Self { diagnostic }
    }
}

impl std::fmt::Display for HayuloError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.diagnostic.message)
    }
}

impl std::error::Error for HayuloError {}

pub type HayuloResult<T> = Result<T, HayuloError>;

pub fn diagnostic_failure_payload(errors: &[HayuloError]) -> Value {
    json!({
        "schema": DIAGNOSTIC_SCHEMA,
        "status": "failed",
        "diagnostics": errors.iter().map(|error| error.diagnostic.to_schema_json()).collect::<Vec<_>>(),
        "errors": errors.iter().map(|error| error.diagnostic.to_legacy_json()).collect::<Vec<_>>(),
    })
}
