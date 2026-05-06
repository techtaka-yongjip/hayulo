use std::collections::{BTreeMap, BTreeSet};

use serde_json::json;

use crate::ast::{Expr, LiteralValue, Program, Stmt};
use crate::diagnostics::{Diagnostic, HayuloError, HayuloResult};

#[derive(Clone, Debug, PartialEq, Eq)]
struct StaticType {
    name: String,
    item: Option<Box<StaticType>>,
    key: Option<Box<StaticType>>,
    value: Option<Box<StaticType>>,
    fields: Option<BTreeMap<String, StaticType>>,
}

impl StaticType {
    fn named(name: &str) -> Self {
        Self {
            name: name.to_string(),
            item: None,
            key: None,
            value: None,
            fields: None,
        }
    }

    fn list(item: StaticType) -> Self {
        Self {
            name: "List".to_string(),
            item: Some(Box::new(item)),
            key: None,
            value: None,
            fields: None,
        }
    }

    fn map(key: StaticType, value: StaticType) -> Self {
        Self {
            name: "Map".to_string(),
            item: None,
            key: Some(Box::new(key)),
            value: Some(Box::new(value)),
            fields: None,
        }
    }

    fn option(item: StaticType) -> Self {
        Self {
            name: "Option".to_string(),
            item: Some(Box::new(item)),
            key: None,
            value: None,
            fields: None,
        }
    }

    fn result(ok: StaticType, err: StaticType) -> Self {
        Self {
            name: "Result".to_string(),
            item: None,
            key: Some(Box::new(ok)),
            value: Some(Box::new(err)),
            fields: None,
        }
    }

    fn record(name: &str, fields: BTreeMap<String, StaticType>) -> Self {
        Self {
            name: name.to_string(),
            item: None,
            key: None,
            value: None,
            fields: Some(fields),
        }
    }

    fn label(&self) -> String {
        match self.name.as_str() {
            "List" => format!(
                "List<{}>",
                self.item.as_deref().unwrap_or(&unknown()).label()
            ),
            "Map" => format!(
                "Map<{},{}>",
                self.key.as_deref().unwrap_or(&unknown()).label(),
                self.value.as_deref().unwrap_or(&unknown()).label()
            ),
            "Option" => format!(
                "Option<{}>",
                self.item.as_deref().unwrap_or(&unknown()).label()
            ),
            "Result" => format!(
                "Result<{},{}>",
                self.key.as_deref().unwrap_or(&unknown()).label(),
                self.value.as_deref().unwrap_or(&unknown()).label()
            ),
            _ => self.name.clone(),
        }
    }
}

#[derive(Clone)]
struct FunctionInfo {
    param_types: Vec<StaticType>,
    declared_return: Option<StaticType>,
    inferred_return: Option<StaticType>,
}

pub fn check_program(program: &Program, filename: Option<&str>) -> HayuloResult<()> {
    StaticChecker::new(program, filename).check()
}

struct StaticChecker<'a> {
    program: &'a Program,
    filename: Option<&'a str>,
    functions: BTreeMap<String, FunctionInfo>,
    checking: BTreeSet<String>,
    checked: BTreeSet<String>,
}

impl<'a> StaticChecker<'a> {
    fn new(program: &'a Program, filename: Option<&'a str>) -> Self {
        Self {
            program,
            filename,
            functions: BTreeMap::new(),
            checking: BTreeSet::new(),
            checked: BTreeSet::new(),
        }
    }

    fn check(mut self) -> HayuloResult<()> {
        for function in self.program.functions.values() {
            let mut seen = BTreeSet::new();
            for param in &function.params {
                if !seen.insert(param.name.clone()) {
                    return self.error(
                        "name.duplicate_definition",
                        format!(
                            "Parameter {:?} is already defined in function {:?}.",
                            param.name, function.name
                        ),
                        Some(param.line),
                        vec![
                            ("name", json!(param.name)),
                            ("function", json!(function.name)),
                        ],
                        vec!["Rename one of the duplicate parameters."],
                    );
                }
            }
            self.functions.insert(
                function.name.clone(),
                FunctionInfo {
                    param_types: function
                        .params
                        .iter()
                        .map(|param| type_from_annotation(param.type_name.as_deref()))
                        .collect(),
                    declared_return: function
                        .return_type
                        .as_deref()
                        .map(|value| type_from_annotation(Some(value))),
                    inferred_return: None,
                },
            );
        }
        for function in self.program.functions.values() {
            self.check_function(&function.name)?;
        }
        for test in &self.program.tests {
            self.check_block(&test.body, &mut BTreeMap::new(), None)?;
        }
        Ok(())
    }

    fn check_function(&mut self, name: &str) -> HayuloResult<()> {
        if self.checked.contains(name) || self.checking.contains(name) {
            return Ok(());
        }
        let function = self.program.functions.get(name).unwrap();
        self.checking.insert(name.to_string());
        let info = self.functions.get(name).cloned().unwrap();
        let mut env = BTreeMap::new();
        for (index, param) in function.params.iter().enumerate() {
            env.insert(param.name.clone(), info.param_types[index].clone());
        }
        let returns = self.check_block(&function.body, &mut env, info.declared_return.as_ref())?;
        if !returns.is_empty() {
            self.functions.get_mut(name).unwrap().inferred_return = Some(common_type(&returns));
        } else if let Some(expected) = &info.declared_return {
            if !compatible(expected, &none()) {
                return self.error(
                    "type.return_mismatch",
                    format!("Function {:?} declares return type {} but may return None.", function.name, expected.label()),
                    Some(function.line),
                    vec![
                        ("expected", json!(expected.label())),
                        ("actual", json!(none().label())),
                        ("function", json!(function.name)),
                    ],
                    vec!["Return a value matching the declared type on every path or remove the return annotation."],
                );
            }
        }
        self.checking.remove(name);
        self.checked.insert(name.to_string());
        Ok(())
    }

    fn check_block(
        &mut self,
        body: &[Stmt],
        env: &mut BTreeMap<String, StaticType>,
        expected_return: Option<&StaticType>,
    ) -> HayuloResult<Vec<StaticType>> {
        let mut returns = Vec::new();
        for stmt in body {
            returns.extend(self.check_stmt(stmt, env, expected_return)?);
        }
        Ok(returns)
    }

    fn check_stmt(
        &mut self,
        stmt: &Stmt,
        env: &mut BTreeMap<String, StaticType>,
        expected_return: Option<&StaticType>,
    ) -> HayuloResult<Vec<StaticType>> {
        match stmt {
            Stmt::Let { name, expr, line } => {
                if env.contains_key(name) {
                    return self.error(
                        "name.duplicate_definition",
                        format!("Name {name:?} is already bound in this scope."),
                        Some(*line),
                        vec![("name", json!(name))],
                        vec!["Use set to reassign an existing binding or choose a different name."],
                    );
                }
                let ty = self.infer_expr(expr, env, expected_return)?;
                env.insert(name.clone(), ty);
                Ok(Vec::new())
            }
            Stmt::Set { name, expr, line } => {
                if !env.contains_key(name) {
                    return self.error(
                        "name.reassignment_before_binding",
                        format!("Cannot reassign {name:?} before it is bound."),
                        Some(*line),
                        vec![("name", json!(name))],
                        vec!["Use let to create a new binding before using set."],
                    );
                }
                let ty = self.infer_expr(expr, env, expected_return)?;
                env.insert(name.clone(), ty);
                Ok(Vec::new())
            }
            Stmt::Return { expr, line } => {
                let actual = self.infer_expr(expr, env, expected_return)?;
                if let Some(expected) = expected_return {
                    if !compatible(expected, &actual) {
                        return self.error(
                            "type.return_mismatch",
                            format!("Return value has type {} but function declares {}.", actual.label(), expected.label()),
                            Some(*line),
                            vec![
                                ("expected", json!(expected.label())),
                                ("actual", json!(actual.label())),
                            ],
                            vec!["Return a value matching the declared type or update the return annotation."],
                        );
                    }
                }
                Ok(vec![actual])
            }
            Stmt::ExprStmt { expr } => {
                self.infer_expr(expr, env, expected_return)?;
                Ok(Vec::new())
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
                self.infer_expr(condition, env, expected_return)?;
                let mut then_env = env.clone();
                let mut returns = self.check_block(then_body, &mut then_env, expected_return)?;
                let mut else_env = env.clone();
                returns.extend(self.check_block(else_body, &mut else_env, expected_return)?);
                Ok(returns)
            }
            Stmt::For {
                name,
                iterable,
                body,
                line,
            } => {
                let iterable = self.infer_expr(iterable, env, expected_return)?;
                let loop_type = self.loop_type(&iterable, *line)?;
                let mut loop_env = env.clone();
                loop_env.insert(name.clone(), loop_type);
                self.check_block(body, &mut loop_env, expected_return)
            }
            Stmt::Expect { expr, .. } => {
                self.infer_expr(expr, env, expected_return)?;
                Ok(Vec::new())
            }
            Stmt::Match {
                target,
                cases,
                line,
            } => {
                let target = self.infer_expr(target, env, expected_return)?;
                self.check_match(&target, cases, *line, env, expected_return)
            }
        }
    }

    fn check_match(
        &mut self,
        target: &StaticType,
        cases: &[crate::ast::MatchCase],
        line: usize,
        env: &BTreeMap<String, StaticType>,
        expected_return: Option<&StaticType>,
    ) -> HayuloResult<Vec<StaticType>> {
        let variants: BTreeSet<_> = cases.iter().map(|case| case.variant.as_str()).collect();
        if target.name == "Option" {
            let required = BTreeSet::from(["Some", "None"]);
            if variants != required {
                return self.error(
                    "match.non_exhaustive",
                    "Match on Option must handle Some and None.",
                    Some(line),
                    vec![
                        ("required", json!(["None", "Some"])),
                        (
                            "actual",
                            json!(variants.iter().copied().collect::<Vec<_>>()),
                        ),
                    ],
                    vec!["Add both Some(value) and None cases."],
                );
            }
        } else if target.name == "Result" {
            let required = BTreeSet::from(["Ok", "Err"]);
            if variants != required {
                return self.error(
                    "match.non_exhaustive",
                    "Match on Result must handle Ok and Err.",
                    Some(line),
                    vec![
                        ("required", json!(["Err", "Ok"])),
                        (
                            "actual",
                            json!(variants.iter().copied().collect::<Vec<_>>()),
                        ),
                    ],
                    vec!["Add both Ok(value) and Err(error) cases."],
                );
            }
        } else if !matches!(target.name.as_str(), "Unknown" | "Any") {
            return self.error(
                "match.invalid_target",
                format!(
                    "Can only match Option or Result values, not {}.",
                    target.label()
                ),
                Some(line),
                vec![("actual", json!(target.label()))],
                vec!["Match on a value with type Option<T> or Result<T, E>."],
            );
        }

        let mut returns = Vec::new();
        for case in cases {
            let mut case_env = env.clone();
            if let Some(binding) = &case.binding {
                match case.variant.as_str() {
                    "Some" => {
                        case_env.insert(
                            binding.clone(),
                            target.item.as_deref().cloned().unwrap_or_else(unknown),
                        );
                    }
                    "Ok" => {
                        case_env.insert(
                            binding.clone(),
                            target.key.as_deref().cloned().unwrap_or_else(unknown),
                        );
                    }
                    "Err" => {
                        case_env.insert(
                            binding.clone(),
                            target.value.as_deref().cloned().unwrap_or_else(unknown),
                        );
                    }
                    "None" => {
                        return self.error(
                            "match.invalid_binding",
                            "None match cases do not bind a value.",
                            Some(case.line),
                            Vec::new(),
                            vec!["Use `None => { ... }` without parentheses."],
                        );
                    }
                    _ => {}
                }
            }
            returns.extend(self.check_block(&case.body, &mut case_env, expected_return)?);
        }
        Ok(returns)
    }

    fn infer_expr(
        &mut self,
        expr: &Expr,
        env: &BTreeMap<String, StaticType>,
        expected_return: Option<&StaticType>,
    ) -> HayuloResult<StaticType> {
        match expr {
            Expr::Literal(value) => Ok(match value {
                LiteralValue::None => none(),
                LiteralValue::Bool(_) => bool_t(),
                LiteralValue::Int(_) => int_t(),
                LiteralValue::Float(_) => float_t(),
                LiteralValue::Text(_) => text_t(),
            }),
            Expr::Variable { name, line } => env.get(name).cloned().ok_or_else(|| {
                HayuloError::new(
                    Diagnostic::new("name.unknown_symbol", format!("Unknown name {name:?}."))
                        .file(self.filename)
                        .line(Some(*line))
                        .detail("name", json!(name))
                        .suggestion("Define the name before using it or check for a typo."),
                )
            }),
            Expr::Unary { op, right } => {
                let value = self.infer_expr(right, env, expected_return)?;
                if op == "not" {
                    return Ok(bool_t());
                }
                if op == "-" {
                    if matches!(value.name.as_str(), "Int" | "Float" | "Unknown" | "Any") {
                        return Ok(value);
                    }
                    return self.error(
                        "type.operator_mismatch",
                        format!("Unary operator - cannot be used with {}.", value.label()),
                        line_for_expr(right),
                        vec![("operator", json!("-")), ("actual", json!(value.label()))],
                        vec!["Use - with Int or Float values."],
                    );
                }
                Ok(unknown())
            }
            Expr::Binary { left, op, right } => {
                if op == "and" || op == "or" {
                    self.infer_expr(left, env, expected_return)?;
                    self.infer_expr(right, env, expected_return)?;
                    return Ok(bool_t());
                }
                let left_t = self.infer_expr(left, env, expected_return)?;
                let right_t = self.infer_expr(right, env, expected_return)?;
                if matches!(op.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=") {
                    return Ok(bool_t());
                }
                if op == "+" && left_t.name == "Text" && right_t.name == "Text" {
                    return Ok(text_t());
                }
                if matches!(op.as_str(), "+" | "-" | "*" | "/" | "%") {
                    if is_numberish(&left_t) && is_numberish(&right_t) {
                        if left_t.name == "Float" || right_t.name == "Float" || op == "/" {
                            return Ok(float_t());
                        }
                        if left_t.name == "Unknown" || right_t.name == "Unknown" {
                            return Ok(unknown());
                        }
                        return Ok(int_t());
                    }
                    return self.error(
                        "type.operator_mismatch",
                        format!(
                            "Operator {op} cannot be used with {} and {}.",
                            left_t.label(),
                            right_t.label()
                        ),
                        line_for_expr(expr),
                        vec![
                            ("operator", json!(op)),
                            ("left", json!(left_t.label())),
                            ("right", json!(right_t.label())),
                        ],
                        vec!["Use compatible operand types for this operator."],
                    );
                }
                Ok(unknown())
            }
            Expr::ListLiteral(elements) => {
                if elements.is_empty() {
                    Ok(StaticType::list(any()))
                } else {
                    let mut types = Vec::new();
                    for element in elements {
                        types.push(self.infer_expr(element, env, expected_return)?);
                    }
                    Ok(StaticType::list(common_type(&types)))
                }
            }
            Expr::MapLiteral(entries) => {
                if entries.is_empty() {
                    Ok(StaticType::map(any(), any()))
                } else {
                    let mut keys = Vec::new();
                    let mut values = Vec::new();
                    for (key, value) in entries {
                        keys.push(self.infer_expr(key, env, expected_return)?);
                        values.push(self.infer_expr(value, env, expected_return)?);
                    }
                    Ok(StaticType::map(common_type(&keys), common_type(&values)))
                }
            }
            Expr::Index {
                target,
                index,
                line,
            } => {
                let target_t = self.infer_expr(target, env, expected_return)?;
                let index_t = self.infer_expr(index, env, expected_return)?;
                if target_t.name == "List" {
                    if !compatible(&int_t(), &index_t) {
                        return self.error(
                            "type.invalid_index",
                            format!("List index must be Int, not {}.", index_t.label()),
                            Some(*line),
                            vec![
                                ("expected", json!("Int")),
                                ("actual", json!(index_t.label())),
                            ],
                            vec!["Use an Int index such as values[0]."],
                        );
                    }
                    return Ok(target_t.item.as_deref().cloned().unwrap_or_else(unknown));
                }
                if target_t.name == "Map" {
                    if let Some(key) = target_t.key.as_deref() {
                        if !compatible(key, &index_t) {
                            return self.error(
                                "type.invalid_index",
                                format!(
                                    "Map index has type {} but keys are {}.",
                                    index_t.label(),
                                    key.label()
                                ),
                                Some(*line),
                                vec![
                                    ("expected", json!(key.label())),
                                    ("actual", json!(index_t.label())),
                                ],
                                vec!["Use an index value matching the map key type."],
                            );
                        }
                    }
                    return Ok(target_t.value.as_deref().cloned().unwrap_or_else(unknown));
                }
                if matches!(target_t.name.as_str(), "Unknown" | "Any") {
                    return Ok(unknown());
                }
                self.error(
                    "type.invalid_index_target",
                    format!(
                        "Only lists and maps can be indexed, not {}.",
                        target_t.label()
                    ),
                    Some(*line),
                    vec![("target", json!(target_t.label()))],
                    vec!["Index a list with an Int or a map with a key."],
                )
            }
            Expr::FieldAccess {
                target,
                field,
                line,
            } => {
                let target_t = self.infer_expr(target, env, expected_return)?;
                if let Some(fields) = &target_t.fields {
                    return fields.get(field).cloned().ok_or_else(|| {
                        HayuloError::new(
                            Diagnostic::new(
                                "record.unknown_field",
                                format!("Record {} has no field {field:?}.", target_t.name),
                            )
                            .file(self.filename)
                            .line(Some(*line))
                            .detail("record", json!(target_t.name))
                            .detail("field", json!(field))
                            .detail("known_fields", json!(fields.keys().collect::<Vec<_>>()))
                            .suggestion("Use one of the fields defined on this record value."),
                        )
                    });
                }
                if matches!(target_t.name.as_str(), "Unknown" | "Any")
                    || is_record_name(&target_t.name)
                {
                    return Ok(unknown());
                }
                self.error(
                    "record.invalid_field_target",
                    format!(
                        "Only records support field access, not {}.",
                        target_t.label()
                    ),
                    Some(*line),
                    vec![("target", json!(target_t.label())), ("field", json!(field))],
                    vec!["Access fields on record values such as User { name: \"Ada\" }.name."],
                )
            }
            Expr::RecordLiteral {
                type_name, fields, ..
            } => {
                let mut field_types = BTreeMap::new();
                for (name, value) in fields {
                    field_types.insert(name.clone(), self.infer_expr(value, env, expected_return)?);
                }
                Ok(StaticType::record(type_name, field_types))
            }
            Expr::VariantLiteral { variant, value, .. } => {
                if variant == "None" {
                    return Ok(StaticType::option(any()));
                }
                let value = if let Some(value) = value {
                    self.infer_expr(value, env, expected_return)?
                } else {
                    unknown()
                };
                match variant.as_str() {
                    "Some" => Ok(StaticType::option(value)),
                    "Ok" => Ok(StaticType::result(value, any())),
                    "Err" => Ok(StaticType::result(any(), value)),
                    _ => Ok(unknown()),
                }
            }
            Expr::Try { expr, line } => {
                let target = self.infer_expr(expr, env, expected_return)?;
                if target.name == "Option" {
                    self.check_try_early_return(&target, expected_return, *line)?;
                    return Ok(target.item.as_deref().cloned().unwrap_or_else(unknown));
                }
                if target.name == "Result" {
                    self.check_try_early_return(&target, expected_return, *line)?;
                    return Ok(target.key.as_deref().cloned().unwrap_or_else(unknown));
                }
                if matches!(target.name.as_str(), "Unknown" | "Any") {
                    return Ok(unknown());
                }
                self.error(
                    "type.invalid_try_target",
                    format!("try expects Option or Result, not {}.", target.label()),
                    Some(*line),
                    vec![("actual", json!(target.label()))],
                    vec!["Use try with a function returning Option<T> or Result<T, E>."],
                )
            }
            Expr::Call { callee, args, line } => {
                self.infer_call(callee, args, *line, env, expected_return)
            }
        }
    }

    fn infer_call(
        &mut self,
        callee: &str,
        args: &[Expr],
        line: usize,
        env: &BTreeMap<String, StaticType>,
        expected_return: Option<&StaticType>,
    ) -> HayuloResult<StaticType> {
        let mut arg_types = Vec::new();
        for arg in args {
            arg_types.push(self.infer_expr(arg, env, expected_return)?);
        }
        if callee == "print" {
            return Ok(none());
        }
        if callee == "len" {
            self.check_arity("len", 1, arg_types.len(), line)?;
            let target = &arg_types[0];
            if !matches!(
                target.name.as_str(),
                "Text" | "List" | "Map" | "Unknown" | "Any"
            ) {
                return self.error(
                    "type.argument_mismatch",
                    format!("len expects Text, List, or Map, not {}.", target.label()),
                    Some(line),
                    vec![
                        ("expected", json!("Text|List|Map")),
                        ("actual", json!(target.label())),
                    ],
                    vec!["Pass a Text, List, or Map value to len."],
                );
            }
            return Ok(int_t());
        }

        let info = self.functions.get(callee).cloned().ok_or_else(|| {
            HayuloError::new(
                Diagnostic::new(
                    "name.unknown_symbol",
                    format!("Unknown function {callee:?}."),
                )
                .file(self.filename)
                .line(Some(line))
                .detail("name", json!(callee))
                .suggestion("Define this function or check the function name for typos."),
            )
        })?;
        if info.declared_return.is_none() && info.inferred_return.is_none() {
            self.check_function(callee)?;
        }
        let info = self.functions.get(callee).cloned().unwrap();
        self.check_arity(callee, info.param_types.len(), arg_types.len(), line)?;
        for (index, (expected, actual)) in info.param_types.iter().zip(&arg_types).enumerate() {
            if !compatible(expected, actual) {
                return self.error(
                    "type.argument_mismatch",
                    format!(
                        "Argument {} to {callee} has type {} but parameter expects {}.",
                        index + 1,
                        actual.label(),
                        expected.label()
                    ),
                    Some(line),
                    vec![
                        ("function", json!(callee)),
                        ("argument", json!(index + 1)),
                        ("expected", json!(expected.label())),
                        ("actual", json!(actual.label())),
                    ],
                    vec!["Pass a value matching the parameter type."],
                );
            }
        }
        Ok(info
            .declared_return
            .or(info.inferred_return)
            .unwrap_or_else(unknown))
    }

    fn check_try_early_return(
        &self,
        target: &StaticType,
        expected_return: Option<&StaticType>,
        line: usize,
    ) -> HayuloResult<()> {
        let Some(expected) = expected_return else {
            return Ok(());
        };
        if matches!(expected.name.as_str(), "Unknown" | "Any") {
            return Ok(());
        }
        if target.name == "Option" {
            if expected.name == "Option" || expected.name == "Result" {
                return Ok(());
            }
            return self.error(
                "type.try_return_mismatch",
                format!("try on Option may return None, but the function declares {}.", expected.label()),
                Some(line),
                vec![
                    ("try_target", json!(target.label())),
                    ("function_return", json!(expected.label())),
                ],
                vec!["Use try only inside a function returning Option<T> or Result<T, E>, or handle the Option with match."],
            );
        }
        if target.name == "Result" {
            if expected.name == "Result" {
                if let (Some(expected_err), Some(target_err)) =
                    (expected.value.as_deref(), target.value.as_deref())
                {
                    if !compatible(expected_err, target_err) {
                        return self.error(
                            "type.try_return_mismatch",
                            format!("try may return Err({}) but the function declares {}.", target_err.label(), expected.label()),
                            Some(line),
                            vec![
                                ("try_target", json!(target.label())),
                                ("function_return", json!(expected.label())),
                            ],
                            vec!["Use a compatible Result error type or handle the Result with match."],
                        );
                    }
                }
                return Ok(());
            }
            return self.error(
                "type.try_return_mismatch",
                format!("try on Result may return Err, but the function declares {}.", expected.label()),
                Some(line),
                vec![
                    ("try_target", json!(target.label())),
                    ("function_return", json!(expected.label())),
                ],
                vec!["Use try only inside a function returning Result<T, E>, or handle the Result with match."],
            );
        }
        Ok(())
    }

    fn check_arity(
        &self,
        name: &str,
        expected: usize,
        actual: usize,
        line: usize,
    ) -> HayuloResult<()> {
        if expected == actual {
            return Ok(());
        }
        self.error(
            "call.arity_mismatch",
            format!("Function {name:?} expects {expected} arguments but got {actual}."),
            Some(line),
            vec![
                ("function", json!(name)),
                ("expected", json!(expected)),
                ("actual", json!(actual)),
            ],
            vec!["Pass the expected number of arguments or update the function signature."],
        )
    }

    fn loop_type(&self, iterable: &StaticType, line: usize) -> HayuloResult<StaticType> {
        if iterable.name == "List" {
            return Ok(iterable.item.as_deref().cloned().unwrap_or_else(unknown));
        }
        if iterable.name == "Map" {
            return Ok(iterable.key.as_deref().cloned().unwrap_or_else(unknown));
        }
        if matches!(iterable.name.as_str(), "Unknown" | "Any") {
            return Ok(unknown());
        }
        self.error(
            "type.not_iterable",
            format!(
                "For loops can iterate over lists and maps, not {}.",
                iterable.label()
            ),
            Some(line),
            vec![("actual", json!(iterable.label()))],
            vec!["Use a List or Map value after 'in'."],
        )
    }

    fn error<T>(
        &self,
        code: &str,
        message: impl Into<String>,
        line: Option<usize>,
        details: Vec<(&str, serde_json::Value)>,
        suggestions: Vec<&str>,
    ) -> HayuloResult<T> {
        let mut diagnostic = Diagnostic::new(code, message.into())
            .file(self.filename)
            .line(line)
            .suggestions(suggestions);
        for (key, value) in details {
            diagnostic = diagnostic.detail(key, value);
        }
        Err(HayuloError::new(diagnostic))
    }
}

fn type_from_annotation(type_name: Option<&str>) -> StaticType {
    let Some(type_name) = type_name else {
        return unknown();
    };
    let value = type_name.split_whitespace().collect::<String>();
    if builtins().contains(value.as_str()) {
        return StaticType::named(&value);
    }
    if let Some(inner) = value
        .strip_prefix("List<")
        .and_then(|v| v.strip_suffix('>'))
    {
        return StaticType::list(type_from_annotation(Some(inner)));
    }
    if let Some(inner) = value.strip_prefix("Map<").and_then(|v| v.strip_suffix('>')) {
        let parts = split_top_level(inner, ',');
        if parts.len() == 2 {
            return StaticType::map(
                type_from_annotation(Some(parts[0])),
                type_from_annotation(Some(parts[1])),
            );
        }
        return StaticType::map(any(), any());
    }
    if let Some(inner) = value
        .strip_prefix("Option<")
        .and_then(|v| v.strip_suffix('>'))
    {
        return StaticType::option(type_from_annotation(Some(inner)));
    }
    if let Some(inner) = value
        .strip_prefix("Result<")
        .and_then(|v| v.strip_suffix('>'))
    {
        let parts = split_top_level(inner, ',');
        if parts.len() == 2 {
            return StaticType::result(
                type_from_annotation(Some(parts[0])),
                type_from_annotation(Some(parts[1])),
            );
        }
        return StaticType::result(any(), any());
    }
    StaticType::named(&value)
}

fn split_top_level(value: &str, separator: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0isize;
    let mut start = 0usize;
    for (index, ch) in value.char_indices() {
        if ch == '<' {
            depth += 1;
        } else if ch == '>' {
            depth -= 1;
        } else if ch == separator && depth == 0 {
            parts.push(&value[start..index]);
            start = index + ch.len_utf8();
        }
    }
    parts.push(&value[start..]);
    parts.into_iter().filter(|part| !part.is_empty()).collect()
}

fn common_type(types: &[StaticType]) -> StaticType {
    if types.is_empty() {
        return unknown();
    }
    let mut result = types[0].clone();
    for current in &types[1..] {
        if matches!(result.name.as_str(), "Unknown" | "Any") {
            result = current.clone();
            continue;
        }
        if matches!(current.name.as_str(), "Unknown" | "Any") {
            continue;
        }
        if result.name != current.name {
            return unknown();
        }
        match result.name.as_str() {
            "List" => {
                result = StaticType::list(common_type(&[
                    result.item.as_deref().cloned().unwrap_or_else(unknown),
                    current.item.as_deref().cloned().unwrap_or_else(unknown),
                ]))
            }
            "Map" => {
                result = StaticType::map(
                    common_type(&[
                        result.key.as_deref().cloned().unwrap_or_else(unknown),
                        current.key.as_deref().cloned().unwrap_or_else(unknown),
                    ]),
                    common_type(&[
                        result.value.as_deref().cloned().unwrap_or_else(unknown),
                        current.value.as_deref().cloned().unwrap_or_else(unknown),
                    ]),
                )
            }
            "Option" => {
                result = StaticType::option(common_type(&[
                    result.item.as_deref().cloned().unwrap_or_else(unknown),
                    current.item.as_deref().cloned().unwrap_or_else(unknown),
                ]))
            }
            "Result" => {
                result = StaticType::result(
                    common_type(&[
                        result.key.as_deref().cloned().unwrap_or_else(unknown),
                        current.key.as_deref().cloned().unwrap_or_else(unknown),
                    ]),
                    common_type(&[
                        result.value.as_deref().cloned().unwrap_or_else(unknown),
                        current.value.as_deref().cloned().unwrap_or_else(unknown),
                    ]),
                )
            }
            _ if result.fields.is_some() || current.fields.is_some() => {
                if result.fields == current.fields {
                    result.fields = result.fields.clone();
                } else {
                    result.fields = None;
                }
            }
            _ => {}
        }
    }
    result
}

fn compatible(expected: &StaticType, actual: &StaticType) -> bool {
    if matches!(expected.name.as_str(), "Any" | "Unknown")
        || matches!(actual.name.as_str(), "Any" | "Unknown")
    {
        return true;
    }
    if expected.name == "Float" && actual.name == "Int" {
        return true;
    }
    if expected.name != actual.name {
        return false;
    }
    match expected.name.as_str() {
        "List" => match (expected.item.as_deref(), actual.item.as_deref()) {
            (Some(e), Some(a)) => compatible(e, a),
            _ => true,
        },
        "Map" => match (
            expected.key.as_deref(),
            expected.value.as_deref(),
            actual.key.as_deref(),
            actual.value.as_deref(),
        ) {
            (Some(ek), Some(ev), Some(ak), Some(av)) => compatible(ek, ak) && compatible(ev, av),
            _ => true,
        },
        "Option" => match (expected.item.as_deref(), actual.item.as_deref()) {
            (Some(e), Some(a)) => compatible(e, a),
            _ => true,
        },
        "Result" => match (
            expected.key.as_deref(),
            expected.value.as_deref(),
            actual.key.as_deref(),
            actual.value.as_deref(),
        ) {
            (Some(ek), Some(ev), Some(ak), Some(av)) => compatible(ek, ak) && compatible(ev, av),
            _ => true,
        },
        _ => true,
    }
}

fn line_for_expr(expr: &Expr) -> Option<usize> {
    match expr {
        Expr::Variable { line, .. }
        | Expr::Index { line, .. }
        | Expr::FieldAccess { line, .. }
        | Expr::RecordLiteral { line, .. }
        | Expr::VariantLiteral { line, .. }
        | Expr::Try { line, .. }
        | Expr::Call { line, .. } => Some(*line),
        Expr::Binary { left, .. } => line_for_expr(left),
        Expr::Unary { right, .. } => line_for_expr(right),
        _ => None,
    }
}

fn is_record_name(name: &str) -> bool {
    !builtins().contains(name) && name != "List" && name != "Map"
}

fn is_numberish(value: &StaticType) -> bool {
    matches!(value.name.as_str(), "Int" | "Float" | "Unknown" | "Any")
}

fn builtins() -> BTreeSet<&'static str> {
    BTreeSet::from([
        "Int", "Float", "Text", "Bool", "Any", "None", "Time", "Email", "Status",
    ])
}

fn unknown() -> StaticType {
    StaticType::named("Unknown")
}
fn any() -> StaticType {
    StaticType::named("Any")
}
fn none() -> StaticType {
    StaticType::named("None")
}
fn bool_t() -> StaticType {
    StaticType::named("Bool")
}
fn int_t() -> StaticType {
    StaticType::named("Int")
}
fn float_t() -> StaticType {
    StaticType::named("Float")
}
fn text_t() -> StaticType {
    StaticType::named("Text")
}
