use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::json;

use crate::ast::{Expr, LiteralValue, MatchCase, Program, Stmt};
use crate::diagnostics::{Diagnostic, HayuloError, HayuloResult};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    List(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Record {
        type_name: String,
        fields: BTreeMap<String, Value>,
    },
    Option {
        kind: String,
        value: Option<Box<Value>>,
    },
    Result {
        kind: String,
        value: Box<Value>,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

enum Control {
    Value(Value),
    Return(Value),
}

pub struct Interpreter<'a> {
    program: &'a Program,
    filename: Option<String>,
    pub output: Vec<String>,
    env_stack: Vec<BTreeMap<String, Value>>,
    return_type_stack: Vec<Option<String>>,
}

impl<'a> Interpreter<'a> {
    pub fn new(program: &'a Program, filename: Option<&str>) -> Self {
        Self {
            program,
            filename: filename.map(ToOwned::to_owned),
            output: Vec::new(),
            env_stack: Vec::new(),
            return_type_stack: Vec::new(),
        }
    }

    pub fn run_main(&mut self) -> HayuloResult<Value> {
        if !self.program.functions.contains_key("main") {
            return self.runtime_error(
                "missing_main",
                "No fn main() found.",
                None,
                serde_json::Map::new(),
                vec!["Add a function named main with no required parameters."],
            );
        }
        self.call_function("main", Vec::new(), None)
    }

    pub fn run_tests(&mut self) -> Vec<TestResult> {
        let mut results = Vec::new();
        for test in &self.program.tests {
            self.env_stack.push(BTreeMap::new());
            let result = match self.exec_block(&test.body) {
                Ok(Some(_)) => TestResult {
                    name: test.name.clone(),
                    passed: false,
                    error: Some("Tests cannot return values.".to_string()),
                    line: Some(test.line),
                },
                Ok(None) => TestResult {
                    name: test.name.clone(),
                    passed: true,
                    error: None,
                    line: Some(test.line),
                },
                Err(error) => TestResult {
                    name: test.name.clone(),
                    passed: false,
                    error: Some(error.diagnostic.message.clone()),
                    line: error.diagnostic.line.or(Some(test.line)),
                },
            };
            self.env_stack.pop();
            results.push(result);
        }
        results
    }

    fn call_function(
        &mut self,
        name: &str,
        args: Vec<Value>,
        line: Option<usize>,
    ) -> HayuloResult<Value> {
        if name == "print" {
            self.output.push(
                args.iter()
                    .map(|arg| self.stringify(arg))
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            return Ok(Value::None);
        }
        if name == "len" {
            if args.len() != 1 {
                return self.runtime_error(
                    "arity_mismatch",
                    "len expects exactly one argument.",
                    line,
                    serde_json::Map::new(),
                    vec!["Call len(value) with a single Text or collection value."],
                );
            }
            return match &args[0] {
                Value::Text(value) => Ok(Value::Int(value.chars().count() as i64)),
                Value::List(values) => Ok(Value::Int(values.len() as i64)),
                Value::Map(entries) => Ok(Value::Int(entries.len() as i64)),
                other => self.runtime_error(
                    "invalid_len_target",
                    "len expects Text, List, or Map.",
                    line,
                    map_details(vec![("target_type", json!(self.type_name(other)))]),
                    vec!["Pass a Text, List, or Map value to len."],
                ),
            };
        }

        let function = self.program.functions.get(name).ok_or_else(|| {
            HayuloError::new(
                Diagnostic::new("unknown_function", format!("Unknown function {name:?}."))
                    .file(self.filename.as_deref())
                    .line(line)
                    .suggestion("Define this function or check the function name for typos."),
            )
        })?;

        if args.len() != function.params.len() {
            return self.runtime_error(
                "arity_mismatch",
                format!(
                    "Function {name:?} expects {} arguments but got {}.",
                    function.params.len(),
                    args.len()
                ),
                Some(function.line),
                map_details(vec![
                    ("expected", json!(function.params.len())),
                    ("actual", json!(args.len())),
                ]),
                vec!["Pass the correct number of arguments or update the function signature."],
            );
        }

        let env = function
            .params
            .iter()
            .zip(args)
            .map(|(param, value)| (param.name.clone(), value))
            .collect();
        self.env_stack.push(env);
        self.return_type_stack.push(function.return_type.clone());
        let result = match self.exec_block(&function.body) {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Ok(Value::None),
            Err(error) => Err(error),
        };
        self.return_type_stack.pop();
        self.env_stack.pop();
        result
    }

    fn exec_block(&mut self, body: &[Stmt]) -> HayuloResult<Option<Value>> {
        for stmt in body {
            if let Some(value) = self.exec_stmt(stmt)? {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }

    fn exec_stmt(&mut self, stmt: &Stmt) -> HayuloResult<Option<Value>> {
        match stmt {
            Stmt::Let { name, expr, line } => {
                let value = self.eval_value(expr)?;
                self.bind(name, value, *line)?;
                Ok(None)
            }
            Stmt::Set { name, expr, line } => {
                let value = self.eval_value(expr)?;
                self.reassign(name, value, *line)?;
                Ok(None)
            }
            Stmt::Return { expr, .. } => match self.eval(expr)? {
                Control::Value(value) | Control::Return(value) => Ok(Some(value)),
            },
            Stmt::ExprStmt { expr } => match self.eval(expr)? {
                Control::Value(_) => Ok(None),
                Control::Return(value) => Ok(Some(value)),
            },
            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
                let condition = self.eval_value(condition)?;
                if self.truthy(&condition) {
                    self.exec_block(then_body)
                } else {
                    self.exec_block(else_body)
                }
            }
            Stmt::For {
                name,
                iterable,
                body,
                line,
            } => {
                let iterable_value = self.eval_value(iterable)?;
                let values = self.iterable_values(iterable_value, *line)?;
                for value in values {
                    self.bind_or_replace(name, value);
                    if let Some(returned) = self.exec_block(body)? {
                        return Ok(Some(returned));
                    }
                }
                Ok(None)
            }
            Stmt::Expect { expr, line } => {
                let value = self.eval_value(expr)?;
                if !self.truthy(&value) {
                    return Err(HayuloError::new(
                        Diagnostic::new("expectation_failed", "Expectation failed.")
                            .file(self.filename.as_deref())
                            .line(Some(*line))
                            .suggestion("Inspect the expression after expect or update the implementation being tested."),
                    ));
                }
                Ok(None)
            }
            Stmt::Match {
                target,
                cases,
                line,
            } => self.exec_match(target, cases, *line),
        }
    }

    fn exec_match(
        &mut self,
        target: &Expr,
        cases: &[MatchCase],
        line: usize,
    ) -> HayuloResult<Option<Value>> {
        let target = self.eval_value(target)?;
        let (variant, value) = match target {
            Value::Option { kind, value } => (kind, value.map(|v| *v).unwrap_or(Value::None)),
            Value::Result { kind, value } => (kind, *value),
            other => {
                return self.runtime_error(
                    "match_invalid_target",
                    "match expects an Option or Result value.",
                    Some(line),
                    map_details(vec![("target_type", json!(self.type_name(&other)))]),
                    vec!["Match on Some/None or Ok/Err values."],
                );
            }
        };
        for case in cases {
            if case.variant != variant {
                continue;
            }
            if let Some(binding) = &case.binding {
                let mut env = BTreeMap::new();
                env.insert(binding.clone(), value);
                self.env_stack.push(env);
                let result = self.exec_block(&case.body);
                self.env_stack.pop();
                return result;
            }
            return self.exec_block(&case.body);
        }
        self.runtime_error(
            "match_non_exhaustive",
            format!("No match case handled {variant}."),
            Some(line),
            map_details(vec![("variant", json!(variant))]),
            vec!["Add all Option or Result variants to this match."],
        )
    }

    fn eval_value(&mut self, expr: &Expr) -> HayuloResult<Value> {
        match self.eval(expr)? {
            Control::Value(value) => Ok(value),
            Control::Return(value) => Ok(value),
        }
    }

    fn eval(&mut self, expr: &Expr) -> HayuloResult<Control> {
        match expr {
            Expr::Literal(value) => Ok(Control::Value(match value {
                LiteralValue::None => Value::None,
                LiteralValue::Bool(value) => Value::Bool(*value),
                LiteralValue::Int(value) => Value::Int(*value),
                LiteralValue::Float(value) => Value::Float(*value),
                LiteralValue::Text(value) => Value::Text(value.clone()),
            })),
            Expr::Variable { name, .. } => Ok(Control::Value(self.lookup(name)?)),
            Expr::Unary { op, right } => {
                let right = self.eval_value(right)?;
                match op.as_str() {
                    "-" => match right {
                        Value::Int(value) => Ok(Control::Value(Value::Int(-value))),
                        Value::Float(value) => Ok(Control::Value(Value::Float(-value))),
                        other => self.runtime_error(
                            "operator_error",
                            format!("Operator - failed for {}.", self.type_name(&other)),
                            None,
                            serde_json::Map::new(),
                            vec!["Use - with Int or Float values."],
                        ),
                    },
                    "not" => Ok(Control::Value(Value::Bool(!self.truthy(&right)))),
                    _ => self.runtime_error(
                        "unknown_operator",
                        format!("Unknown unary operator {op:?}."),
                        None,
                        serde_json::Map::new(),
                        Vec::new(),
                    ),
                }
            }
            Expr::Binary { left, op, right } => {
                if op == "and" {
                    let left_value = self.eval_value(left)?;
                    let right_value = self.eval_value(right)?;
                    let value = self.truthy(&left_value) && self.truthy(&right_value);
                    return Ok(Control::Value(Value::Bool(value)));
                }
                if op == "or" {
                    let left_value = self.eval_value(left)?;
                    let right_value = self.eval_value(right)?;
                    let value = self.truthy(&left_value) || self.truthy(&right_value);
                    return Ok(Control::Value(Value::Bool(value)));
                }
                let left = self.eval_value(left)?;
                let right = self.eval_value(right)?;
                Ok(Control::Value(self.binary(left, op, right)?))
            }
            Expr::ListLiteral(elements) => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.eval_value(element)?);
                }
                Ok(Control::Value(Value::List(values)))
            }
            Expr::MapLiteral(entries) => {
                let mut values = Vec::new();
                for (key_expr, value_expr) in entries {
                    let key = self.eval_value(key_expr)?;
                    if !is_hashable_key(&key) {
                        return self.runtime_error(
                            "invalid_map_key",
                            "Map keys must be hashable values.",
                            None,
                            map_details(vec![("key", json!(format!("{key:?}")))]),
                            vec!["Use Text, Int, Float, or Bool values as map keys."],
                        );
                    }
                    values.push((key, self.eval_value(value_expr)?));
                }
                Ok(Control::Value(Value::Map(values)))
            }
            Expr::Index {
                target,
                index,
                line,
            } => {
                let target = self.eval_value(target)?;
                let index = self.eval_value(index)?;
                Ok(Control::Value(self.index_value(target, index, *line)?))
            }
            Expr::FieldAccess {
                target,
                field,
                line,
            } => {
                let target = self.eval_value(target)?;
                Ok(Control::Value(self.field_value(target, field, *line)?))
            }
            Expr::RecordLiteral {
                type_name, fields, ..
            } => {
                let mut values = BTreeMap::new();
                for (name, expr) in fields {
                    values.insert(name.clone(), self.eval_value(expr)?);
                }
                Ok(Control::Value(Value::Record {
                    type_name: type_name.clone(),
                    fields: values,
                }))
            }
            Expr::VariantLiteral {
                variant,
                value,
                line,
            } => {
                if variant == "None" {
                    return Ok(Control::Value(Value::Option {
                        kind: "None".to_string(),
                        value: None,
                    }));
                }
                let value = if let Some(expr) = value {
                    self.eval_value(expr)?
                } else {
                    Value::None
                };
                match variant.as_str() {
                    "Some" => Ok(Control::Value(Value::Option {
                        kind: "Some".to_string(),
                        value: Some(Box::new(value)),
                    })),
                    "Ok" => Ok(Control::Value(Value::Result {
                        kind: "Ok".to_string(),
                        value: Box::new(value),
                    })),
                    "Err" => Ok(Control::Value(Value::Result {
                        kind: "Err".to_string(),
                        value: Box::new(value),
                    })),
                    _ => self.runtime_error(
                        "unknown_variant",
                        format!("Unknown variant {variant:?}."),
                        Some(*line),
                        serde_json::Map::new(),
                        Vec::new(),
                    ),
                }
            }
            Expr::Try { expr, line } => self.eval_try(expr, *line),
            Expr::Call { callee, args, line } => {
                let mut values = Vec::new();
                for arg in args {
                    values.push(self.eval_value(arg)?);
                }
                self.call_function(callee, values, Some(*line))
                    .map(Control::Value)
            }
        }
    }

    fn eval_try(&mut self, expr: &Expr, line: usize) -> HayuloResult<Control> {
        let value = self.eval_value(expr)?;
        match value {
            Value::Option { kind, value } if kind == "Some" => {
                Ok(Control::Value(value.map(|v| *v).unwrap_or(Value::None)))
            }
            Value::Option { .. } => Ok(Control::Return(self.early_none_value())),
            Value::Result { kind, value } if kind == "Ok" => Ok(Control::Value(*value)),
            Value::Result { value, .. } => Ok(Control::Return(Value::Result {
                kind: "Err".to_string(),
                value,
            })),
            other => self.runtime_error(
                "invalid_try_target",
                "try expects an Option or Result value.",
                Some(line),
                map_details(vec![("target_type", json!(self.type_name(&other)))]),
                vec!["Use try with a function returning Option<T> or Result<T, E>."],
            ),
        }
    }

    fn early_none_value(&self) -> Value {
        if self
            .return_type_stack
            .last()
            .and_then(|value| value.as_ref())
            .map(|value| value.replace(' ', "").starts_with("Result<"))
            .unwrap_or(false)
        {
            Value::Result {
                kind: "Err".to_string(),
                value: Box::new(Value::Text("None".to_string())),
            }
        } else {
            Value::Option {
                kind: "None".to_string(),
                value: None,
            }
        }
    }

    fn binary(&self, left: Value, op: &str, right: Value) -> HayuloResult<Value> {
        match op {
            "+" => match (left, right) {
                (Value::Text(left), Value::Text(right)) => Ok(Value::Text(format!("{left}{right}"))),
                (Value::Int(left), Value::Int(right)) => Ok(Value::Int(left + right)),
                (Value::Int(left), Value::Float(right)) => Ok(Value::Float(left as f64 + right)),
                (Value::Float(left), Value::Int(right)) => Ok(Value::Float(left + right as f64)),
                (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left + right)),
                (left, right) => Err(HayuloError::new(
                    Diagnostic::new("invalid_operator_types", "Operator + supports number addition or Text concatenation, but not mixed values.")
                        .file(self.filename.as_deref())
                        .detail("left", json!(self.type_name(&left)))
                        .detail("right", json!(self.type_name(&right)))
                        .suggestion("Convert values explicitly before using +."),
                )),
            },
            "-" | "*" | "/" | "%" => self.numeric_binary(left, op, right),
            "==" => Ok(Value::Bool(left == right)),
            "!=" => Ok(Value::Bool(left != right)),
            "<" | "<=" | ">" | ">=" => self.compare_binary(left, op, right),
            _ => Err(HayuloError::new(
                Diagnostic::new("unknown_operator", format!("Unknown operator {op:?}."))
                    .file(self.filename.as_deref()),
            )),
        }
    }

    fn numeric_binary(&self, left: Value, op: &str, right: Value) -> HayuloResult<Value> {
        let Some((left_n, left_int)) = numeric_value(&left) else {
            return self.operator_error(op, &left, &right);
        };
        let Some((right_n, right_int)) = numeric_value(&right) else {
            return self.operator_error(op, &left, &right);
        };
        let result = match op {
            "-" => left_n - right_n,
            "*" => left_n * right_n,
            "/" => left_n / right_n,
            "%" => left_n % right_n,
            _ => unreachable!(),
        };
        if op != "/" && left_int && right_int {
            Ok(Value::Int(result as i64))
        } else {
            Ok(Value::Float(result))
        }
    }

    fn compare_binary(&self, left: Value, op: &str, right: Value) -> HayuloResult<Value> {
        let result = match (&left, &right) {
            (Value::Int(_), Value::Int(_))
            | (Value::Int(_), Value::Float(_))
            | (Value::Float(_), Value::Int(_))
            | (Value::Float(_), Value::Float(_)) => {
                let left_n = numeric_value(&left).unwrap().0;
                let right_n = numeric_value(&right).unwrap().0;
                match op {
                    "<" => left_n < right_n,
                    "<=" => left_n <= right_n,
                    ">" => left_n > right_n,
                    ">=" => left_n >= right_n,
                    _ => false,
                }
            }
            (Value::Text(left), Value::Text(right)) => match op {
                "<" => left < right,
                "<=" => left <= right,
                ">" => left > right,
                ">=" => left >= right,
                _ => false,
            },
            _ => return self.operator_error(op, &left, &right),
        };
        Ok(Value::Bool(result))
    }

    fn operator_error(&self, op: &str, left: &Value, right: &Value) -> HayuloResult<Value> {
        Err(HayuloError::new(
            Diagnostic::new("operator_error", format!("Operator {op} failed."))
                .file(self.filename.as_deref())
                .detail("operator", json!(op))
                .detail("left", json!(self.stringify(left)))
                .detail("right", json!(self.stringify(right)))
                .suggestion("Check that both operands support this operator."),
        ))
    }

    fn index_value(&self, target: Value, index: Value, line: usize) -> HayuloResult<Value> {
        match target {
            Value::List(values) => match index {
                Value::Int(index) => {
                    if index < 0 || index as usize >= values.len() {
                        return Err(HayuloError::new(
                            Diagnostic::new(
                                "index_out_of_range",
                                format!("List index {index} is out of range."),
                            )
                            .file(self.filename.as_deref())
                            .line(Some(line))
                            .detail("index", json!(index))
                            .detail("length", json!(values.len()))
                            .suggestion("Check len(value) before indexing."),
                        ));
                    }
                    Ok(values[index as usize].clone())
                }
                other => Err(HayuloError::new(
                    Diagnostic::new("invalid_index_type", "List indexes must be Int values.")
                        .file(self.filename.as_deref())
                        .line(Some(line))
                        .detail("index_type", json!(self.type_name(&other)))
                        .suggestion("Use an Int index such as values[0]."),
                )),
            },
            Value::Map(entries) => entries
                .into_iter()
                .find(|(key, _)| key == &index)
                .map(|(_, value)| value)
                .ok_or_else(|| {
                    HayuloError::new(
                        Diagnostic::new(
                            "missing_map_key",
                            format!("Map key {:?} was not found.", self.stringify(&index)),
                        )
                        .file(self.filename.as_deref())
                        .line(Some(line))
                        .detail("key", json!(self.stringify(&index)))
                        .suggestion("Check that the key exists before indexing the map."),
                    )
                }),
            other => Err(HayuloError::new(
                Diagnostic::new(
                    "invalid_index_target",
                    "Only lists and maps can be indexed.",
                )
                .file(self.filename.as_deref())
                .line(Some(line))
                .detail("target_type", json!(self.type_name(&other)))
                .suggestion("Index a list with an Int or a map with an existing key."),
            )),
        }
    }

    fn field_value(&self, target: Value, field: &str, line: usize) -> HayuloResult<Value> {
        match target {
            Value::Record { type_name, fields } => fields.get(field).cloned().ok_or_else(|| {
                HayuloError::new(
                    Diagnostic::new(
                        "unknown_field",
                        format!("Record {type_name} has no field {field:?}."),
                    )
                    .file(self.filename.as_deref())
                    .line(Some(line))
                    .detail("record", json!(type_name))
                    .detail("field", json!(field))
                    .suggestion("Check the record field name."),
                )
            }),
            other => Err(HayuloError::new(
                Diagnostic::new(
                    "invalid_field_target",
                    "Only records support field access in the current prototype.",
                )
                .file(self.filename.as_deref())
                .line(Some(line))
                .detail("target_type", json!(self.type_name(&other)))
                .detail("field", json!(field))
                .suggestion("Construct a record value before accessing its fields."),
            )),
        }
    }

    fn iterable_values(&self, value: Value, line: usize) -> HayuloResult<Vec<Value>> {
        match value {
            Value::List(values) => Ok(values),
            Value::Map(entries) => Ok(entries.into_iter().map(|(key, _)| key).collect()),
            other => Err(HayuloError::new(
                Diagnostic::new("not_iterable", "For loops can iterate over lists and maps.")
                    .file(self.filename.as_deref())
                    .line(Some(line))
                    .detail("target_type", json!(self.type_name(&other)))
                    .suggestion(
                        "Use a list literal like [1, 2] or a map literal like {\"key\": 1}.",
                    ),
            )),
        }
    }

    fn lookup(&self, name: &str) -> HayuloResult<Value> {
        for env in self.env_stack.iter().rev() {
            if let Some(value) = env.get(name) {
                return Ok(value.clone());
            }
        }
        Err(HayuloError::new(
            Diagnostic::new("unknown_variable", format!("Unknown variable {name:?}."))
                .file(self.filename.as_deref())
                .suggestion("Define the variable before using it or check for a typo."),
        ))
    }

    fn bind(&mut self, name: &str, value: Value, line: usize) -> HayuloResult<()> {
        if self.env_stack.is_empty() {
            self.env_stack.push(BTreeMap::new());
        }
        let env = self.env_stack.last_mut().unwrap();
        if env.contains_key(name) {
            return Err(HayuloError::new(
                Diagnostic::new(
                    "duplicate_binding",
                    format!("Name {name:?} is already bound in this scope."),
                )
                .file(self.filename.as_deref())
                .line(Some(line))
                .suggestion("Use set to reassign an existing binding."),
            ));
        }
        env.insert(name.to_string(), value);
        Ok(())
    }

    fn bind_or_replace(&mut self, name: &str, value: Value) {
        if self.env_stack.is_empty() {
            self.env_stack.push(BTreeMap::new());
        }
        self.env_stack
            .last_mut()
            .unwrap()
            .insert(name.to_string(), value);
    }

    fn reassign(&mut self, name: &str, value: Value, line: usize) -> HayuloResult<()> {
        for env in self.env_stack.iter_mut().rev() {
            if env.contains_key(name) {
                env.insert(name.to_string(), value);
                return Ok(());
            }
        }
        Err(HayuloError::new(
            Diagnostic::new(
                "reassignment_before_binding",
                format!("Cannot reassign {name:?} before it is bound."),
            )
            .file(self.filename.as_deref())
            .line(Some(line))
            .suggestion("Use let before set."),
        ))
    }

    fn truthy(&self, value: &Value) -> bool {
        match value {
            Value::None => false,
            Value::Bool(value) => *value,
            Value::Int(value) => *value != 0,
            Value::Float(value) => *value != 0.0,
            Value::Text(value) => !value.is_empty(),
            Value::List(value) => !value.is_empty(),
            Value::Map(value) => !value.is_empty(),
            _ => true,
        }
    }

    pub fn stringify(&self, value: &Value) -> String {
        match value {
            Value::None => "none".to_string(),
            Value::Bool(true) => "true".to_string(),
            Value::Bool(false) => "false".to_string(),
            Value::Int(value) => value.to_string(),
            Value::Float(value) => {
                let text = value.to_string();
                text
            }
            Value::Text(value) => value.clone(),
            Value::List(values) => format!(
                "[{}]",
                values
                    .iter()
                    .map(|value| self.stringify(value))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Map(entries) => format!(
                "{{{}}}",
                entries
                    .iter()
                    .map(|(key, value)| format!(
                        "{}: {}",
                        self.stringify(key),
                        self.stringify(value)
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Record { type_name, fields } => format!(
                "{type_name} {{{}}}",
                fields
                    .iter()
                    .map(|(name, value)| format!("{name}: {}", self.stringify(value)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Option { kind, value } if kind == "None" => "None".to_string(),
            Value::Option { value, .. } => format!(
                "Some({})",
                value
                    .as_ref()
                    .map(|value| self.stringify(value))
                    .unwrap_or_else(|| "none".to_string())
            ),
            Value::Result { kind, value } => format!("{kind}({})", self.stringify(value)),
        }
    }

    fn type_name(&self, value: &Value) -> String {
        match value {
            Value::None => "None".to_string(),
            Value::Bool(_) => "Bool".to_string(),
            Value::Int(_) => "Int".to_string(),
            Value::Float(_) => "Float".to_string(),
            Value::Text(_) => "Text".to_string(),
            Value::List(_) => "List".to_string(),
            Value::Map(_) => "Map".to_string(),
            Value::Record { type_name, .. } => type_name.clone(),
            Value::Option { .. } => "Option".to_string(),
            Value::Result { .. } => "Result".to_string(),
        }
    }

    fn runtime_error<T>(
        &self,
        code: &str,
        message: impl Into<String>,
        line: Option<usize>,
        details: serde_json::Map<String, serde_json::Value>,
        suggestions: Vec<&str>,
    ) -> HayuloResult<T> {
        Err(HayuloError::new(
            Diagnostic::new(code, message.into())
                .file(self.filename.as_deref())
                .line(line)
                .suggestions(suggestions)
                .detail_map(details),
        ))
    }
}

trait DiagnosticDetailMap {
    fn detail_map(self, details: serde_json::Map<String, serde_json::Value>) -> Self;
}

impl DiagnosticDetailMap for Diagnostic {
    fn detail_map(mut self, details: serde_json::Map<String, serde_json::Value>) -> Self {
        self.details = details;
        self
    }
}

fn numeric_value(value: &Value) -> Option<(f64, bool)> {
    match value {
        Value::Int(value) => Some((*value as f64, true)),
        Value::Float(value) => Some((*value, false)),
        _ => None,
    }
}

fn is_hashable_key(value: &Value) -> bool {
    matches!(
        value,
        Value::None | Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::Text(_)
    )
}

fn map_details(
    entries: Vec<(&str, serde_json::Value)>,
) -> serde_json::Map<String, serde_json::Value> {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}
