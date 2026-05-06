use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{
    Expr, FunctionDecl, FunctionParam, LiteralValue, MatchCase, Program, Stmt, TestDecl,
};
use crate::diagnostics::{Diagnostic, HayuloError, HayuloResult};
use crate::lexer::Token;

pub fn parse(tokens: Vec<Token>, filename: Option<&str>) -> HayuloResult<Program> {
    Parser::new(tokens, filename).parse()
}

struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    filename: Option<&'a str>,
}

impl<'a> Parser<'a> {
    fn new(tokens: Vec<Token>, filename: Option<&'a str>) -> Self {
        Self {
            tokens,
            current: 0,
            filename,
        }
    }

    fn parse(&mut self) -> HayuloResult<Program> {
        let mut module = None;
        let mut functions = BTreeMap::new();
        let mut tests = Vec::new();

        while !self.is_at_end() {
            if self.matches(&["MODULE"]) {
                module = Some(self.parse_module_name()?);
                continue;
            }
            if self.matches(&["INTENT"]) {
                self.skip_balanced_block("intent")?;
                continue;
            }
            self.matches(&["PUB"]);
            if self.matches(&["FN"]) {
                let function = self.function_decl()?;
                if functions.contains_key(&function.name) {
                    return self.error_here(
                        "duplicate_function",
                        format!("Function {:?} is already defined.", function.name),
                        vec!["Rename one of the functions or remove the duplicate."],
                    );
                }
                functions.insert(function.name.clone(), function);
                continue;
            }
            if self.matches(&["TEST"]) {
                tests.push(self.test_decl()?);
                continue;
            }
            if self.check(&["EOF"]) {
                break;
            }
            let token = self.peek().clone();
            return Err(HayuloError::new(
                Diagnostic::new(
                    "unexpected_top_level_token",
                    format!("Unexpected token {:?} at top level.", token.value),
                )
                .file(self.filename)
                .line(Some(token.line))
                .column(Some(token.column))
                .suggestion("Top-level Hayulo code currently supports module, intent, fn, pub fn, and test."),
            ));
        }

        Ok(Program {
            module,
            functions,
            tests,
        })
    }

    fn parse_module_name(&mut self) -> HayuloResult<String> {
        let first = self.consume("IDENT", "Expected module name after 'module'.")?;
        let mut parts = vec![first.value];
        while self.matches(&["DOT"]) {
            parts.push(
                self.consume("IDENT", "Expected identifier after '.'.")?
                    .value,
            );
        }
        Ok(parts.join("."))
    }

    fn function_decl(&mut self) -> HayuloResult<FunctionDecl> {
        let name = self.consume("IDENT", "Expected function name after 'fn'.")?;
        self.consume("LPAREN", "Expected '(' after function name.")?;
        let params = self.params()?;
        self.consume("RPAREN", "Expected ')' after parameters.")?;
        let return_type = if self.matches(&["ARROW"]) {
            Some(self.type_until(&["LBRACE"])?)
        } else {
            None
        };
        let body = self.block()?;
        Ok(FunctionDecl {
            name: name.value,
            params,
            return_type,
            body,
            line: name.line,
        })
    }

    fn test_decl(&mut self) -> HayuloResult<TestDecl> {
        let name = self.consume("STRING", "Expected test name string after 'test'.")?;
        let body = self.block()?;
        Ok(TestDecl {
            name: name.value,
            body,
            line: name.line,
        })
    }

    fn params(&mut self) -> HayuloResult<Vec<FunctionParam>> {
        let mut params = Vec::new();
        if self.check(&["RPAREN"]) {
            return Ok(params);
        }
        loop {
            let param = self.consume("IDENT", "Expected parameter name.")?;
            let type_name = if self.matches(&["COLON"]) {
                Some(self.type_until(&["COMMA", "RPAREN"])?)
            } else {
                None
            };
            params.push(FunctionParam {
                name: param.value,
                type_name,
                line: param.line,
            });
            if !self.matches(&["COMMA"]) {
                break;
            }
        }
        Ok(params)
    }

    fn block(&mut self) -> HayuloResult<Vec<Stmt>> {
        self.consume("LBRACE", "Expected '{' to start a block.")?;
        let mut body = Vec::new();
        while !self.check(&["RBRACE"]) && !self.is_at_end() {
            body.push(self.statement()?);
        }
        self.consume("RBRACE", "Expected '}' to close the block.")?;
        Ok(body)
    }

    fn statement(&mut self) -> HayuloResult<Stmt> {
        if self.matches(&["LET"]) {
            let token = self.consume("IDENT", "Expected variable name after 'let'.")?;
            self.consume("EQUAL", "Expected '=' after variable name.")?;
            return Ok(Stmt::Let {
                name: token.value,
                expr: self.expression()?,
                line: token.line,
            });
        }
        if self.matches(&["SET"]) {
            let token = self.consume("IDENT", "Expected variable name after 'set'.")?;
            self.consume("EQUAL", "Expected '=' after variable name.")?;
            return Ok(Stmt::Set {
                name: token.value,
                expr: self.expression()?,
                line: token.line,
            });
        }
        if self.matches(&["RETURN"]) {
            let line = self.previous().line;
            return Ok(Stmt::Return {
                expr: self.expression()?,
                line,
            });
        }
        if self.matches(&["IF"]) {
            let condition = self.expression()?;
            let then_body = self.block()?;
            let mut else_body = Vec::new();
            if self.matches(&["ELSE"]) {
                if self.matches(&["IF"]) {
                    let nested_condition = self.expression()?;
                    let nested_then = self.block()?;
                    let nested_else = if self.matches(&["ELSE"]) {
                        self.block()?
                    } else {
                        Vec::new()
                    };
                    else_body.push(Stmt::If {
                        condition: nested_condition,
                        then_body: nested_then,
                        else_body: nested_else,
                    });
                } else {
                    else_body = self.block()?;
                }
            }
            return Ok(Stmt::If {
                condition,
                then_body,
                else_body,
            });
        }
        if self.matches(&["FOR"]) {
            let line = self.previous().line;
            let name = self.consume("IDENT", "Expected loop variable name after 'for'.")?;
            self.consume("IN", "Expected 'in' after loop variable.")?;
            let iterable = self.expression()?;
            let body = self.block()?;
            return Ok(Stmt::For {
                name: name.value,
                iterable,
                body,
                line,
            });
        }
        if self.matches(&["EXPECT"]) {
            let line = self.previous().line;
            return Ok(Stmt::Expect {
                expr: self.expression()?,
                line,
            });
        }
        if self.check(&["IDENT"]) && self.check_next("EQUAL") {
            let token = self.peek().clone();
            return Err(HayuloError::new(
                Diagnostic::new(
                    "syntax.binding_requires_let_or_set",
                    "Hayulo 2.0 uses explicit let/set binding syntax.",
                )
                .file(self.filename)
                .line(Some(token.line))
                .column(Some(token.column))
                .suggestion("Use 'let name = value' for a new binding or 'set name = value' for reassignment."),
            ));
        }
        if self.matches(&["MATCH"]) {
            return self.match_stmt();
        }
        Ok(Stmt::ExprStmt {
            expr: self.expression()?,
        })
    }

    fn match_stmt(&mut self) -> HayuloResult<Stmt> {
        let line = self.previous().line;
        let target = self.expression()?;
        self.consume("LBRACE", "Expected '{' to start match block.")?;
        let mut cases = Vec::new();
        let mut seen = BTreeSet::new();
        while !self.check(&["RBRACE"]) && !self.is_at_end() {
            let variant = self.consume(
                "IDENT",
                "Expected match case variant such as Some, None, Ok, or Err.",
            )?;
            if !matches!(variant.value.as_str(), "Some" | "None" | "Ok" | "Err") {
                return Err(HayuloError::new(
                    Diagnostic::new(
                        "syntax.invalid_match_variant",
                        format!("Unsupported match variant {:?}.", variant.value),
                    )
                    .file(self.filename)
                    .line(Some(variant.line))
                    .column(Some(variant.column))
                    .suggestion("Use Some, None, Ok, or Err in this Hayulo 2.0 draft."),
                ));
            }
            if !seen.insert(variant.value.clone()) {
                return Err(HayuloError::new(
                    Diagnostic::new(
                        "syntax.duplicate_match_case",
                        format!("Duplicate match case {:?}.", variant.value),
                    )
                    .file(self.filename)
                    .line(Some(variant.line))
                    .column(Some(variant.column))
                    .suggestion("Remove the duplicate match case."),
                ));
            }
            let binding = if self.matches(&["LPAREN"]) {
                let name = self
                    .consume("IDENT", "Expected binding name in match case.")?
                    .value;
                self.consume("RPAREN", "Expected ')' after match binding.")?;
                Some(name)
            } else {
                None
            };
            self.consume("FAT_ARROW", "Expected '=>' after match case.")?;
            let body = self.block()?;
            cases.push(MatchCase {
                variant: variant.value,
                binding,
                body,
                line: variant.line,
            });
        }
        self.consume("RBRACE", "Expected '}' to close match block.")?;
        Ok(Stmt::Match {
            target,
            cases,
            line,
        })
    }

    fn expression(&mut self) -> HayuloResult<Expr> {
        self.or()
    }

    fn or(&mut self) -> HayuloResult<Expr> {
        let mut expr = self.and()?;
        while self.matches(&["OR"]) {
            let op = self.previous().value.clone();
            let right = self.and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn and(&mut self) -> HayuloResult<Expr> {
        let mut expr = self.equality()?;
        while self.matches(&["AND"]) {
            let op = self.previous().value.clone();
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn equality(&mut self) -> HayuloResult<Expr> {
        let mut expr = self.comparison()?;
        while self.matches(&["EQEQ", "BANGEQ"]) {
            let op = self.previous().value.clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> HayuloResult<Expr> {
        let mut expr = self.term()?;
        while self.matches(&["LT", "LTE", "GT", "GTE"]) {
            let op = self.previous().value.clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> HayuloResult<Expr> {
        let mut expr = self.factor()?;
        while self.matches(&["PLUS", "MINUS"]) {
            let op = self.previous().value.clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> HayuloResult<Expr> {
        let mut expr = self.unary()?;
        while self.matches(&["STAR", "SLASH", "PERCENT"]) {
            let op = self.previous().value.clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> HayuloResult<Expr> {
        if self.matches(&["TRY"]) {
            let line = self.previous().line;
            return Ok(Expr::Try {
                expr: Box::new(self.unary()?),
                line,
            });
        }
        if self.matches(&["MINUS", "NOT"]) {
            let op = self.previous().value.clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                op,
                right: Box::new(right),
            });
        }
        self.call()
    }

    fn call(&mut self) -> HayuloResult<Expr> {
        let mut expr = self.primary()?;
        loop {
            if self.matches(&["LPAREN"]) {
                let line = self.previous().line;
                let callee = match expr {
                    Expr::Variable { name, .. } => name,
                    _ => {
                        return Err(HayuloError::new(
                            Diagnostic::new(
                                "invalid_call_target",
                                "Only named functions can be called in the current prototype.",
                            )
                            .file(self.filename)
                            .line(Some(line))
                            .column(Some(self.previous().column))
                            .suggestion("Call a function by name, such as greet(\"Ada\")."),
                        ));
                    }
                };
                let mut args = Vec::new();
                if !self.check(&["RPAREN"]) {
                    loop {
                        args.push(self.expression()?);
                        if !self.matches(&["COMMA"]) {
                            break;
                        }
                    }
                }
                self.consume("RPAREN", "Expected ')' after arguments.")?;
                if matches!(callee.as_str(), "Some" | "Ok" | "Err") {
                    if args.len() != 1 {
                        let tok = self.previous().clone();
                        return Err(HayuloError::new(
                            Diagnostic::new(
                                "syntax.invalid_variant_arity",
                                format!("{callee} expects exactly one value."),
                            )
                            .file(self.filename)
                            .line(Some(tok.line))
                            .column(Some(tok.column))
                            .suggestion(format!("Use {callee}(value).")),
                        ));
                    }
                    expr = Expr::VariantLiteral {
                        variant: callee,
                        value: Some(Box::new(args.remove(0))),
                        line: self.previous().line,
                    };
                } else {
                    expr = Expr::Call { callee, args, line };
                }
                continue;
            }
            if self.matches(&["LBRACKET"]) {
                let line = self.previous().line;
                let index = self.expression()?;
                self.consume("RBRACKET", "Expected ']' after index expression.")?;
                expr = Expr::Index {
                    target: Box::new(expr),
                    index: Box::new(index),
                    line,
                };
                continue;
            }
            if self.matches(&["DOT"]) {
                let dot = self.previous().clone();
                let field = self.consume("IDENT", "Expected field name after '.'.")?;
                expr = Expr::FieldAccess {
                    target: Box::new(expr),
                    field: field.value,
                    line: dot.line,
                };
                continue;
            }
            if let Expr::Variable { name, .. } = &expr {
                if self.looks_like_record_literal() {
                    let type_name = name.clone();
                    expr = self.record_literal(type_name)?;
                    continue;
                }
            }
            break;
        }
        Ok(expr)
    }

    fn primary(&mut self) -> HayuloResult<Expr> {
        if self.matches(&["INT"]) {
            return Ok(Expr::Literal(LiteralValue::Int(
                self.previous().value.parse().unwrap_or(0),
            )));
        }
        if self.matches(&["FLOAT"]) {
            return Ok(Expr::Literal(LiteralValue::Float(
                self.previous().value.parse().unwrap_or(0.0),
            )));
        }
        if self.matches(&["STRING"]) {
            return Ok(Expr::Literal(LiteralValue::Text(
                self.previous().value.clone(),
            )));
        }
        if self.matches(&["TRUE"]) {
            return Ok(Expr::Literal(LiteralValue::Bool(true)));
        }
        if self.matches(&["FALSE"]) {
            return Ok(Expr::Literal(LiteralValue::Bool(false)));
        }
        if self.check(&["LBRACKET"]) {
            return self.list_literal();
        }
        if self.check(&["LBRACE"]) {
            return self.map_literal();
        }
        if self.matches(&["IDENT"]) {
            let token = self.previous().clone();
            if token.value == "None" {
                return Ok(Expr::VariantLiteral {
                    variant: "None".to_string(),
                    value: None,
                    line: token.line,
                });
            }
            return Ok(Expr::Variable {
                name: token.value,
                line: token.line,
            });
        }
        if self.matches(&["LPAREN"]) {
            let expr = self.expression()?;
            self.consume("RPAREN", "Expected ')' after expression.")?;
            return Ok(expr);
        }
        let token = self.peek().clone();
        Err(HayuloError::new(
            Diagnostic::new(
                "expected_expression",
                format!("Expected expression but found {:?}.", token.value),
            )
            .file(self.filename)
            .line(Some(token.line))
            .column(Some(token.column))
            .suggestion(
                "Use a literal, variable, function call, or parenthesized expression here.",
            ),
        ))
    }

    fn list_literal(&mut self) -> HayuloResult<Expr> {
        self.consume("LBRACKET", "Expected '[' to start list literal.")?;
        let mut elements = Vec::new();
        if !self.check(&["RBRACKET"]) {
            loop {
                elements.push(self.expression()?);
                if !self.matches(&["COMMA"]) {
                    break;
                }
                if self.check(&["RBRACKET"]) {
                    break;
                }
            }
        }
        self.consume("RBRACKET", "Expected ']' after list literal.")?;
        Ok(Expr::ListLiteral(elements))
    }

    fn map_literal(&mut self) -> HayuloResult<Expr> {
        self.consume("LBRACE", "Expected '{' to start map literal.")?;
        let mut entries = Vec::new();
        if !self.check(&["RBRACE"]) {
            loop {
                let key = self.expression()?;
                self.consume("COLON", "Expected ':' between map key and value.")?;
                let value = self.expression()?;
                entries.push((key, value));
                if !self.matches(&["COMMA"]) {
                    break;
                }
                if self.check(&["RBRACE"]) {
                    break;
                }
            }
        }
        self.consume("RBRACE", "Expected '}' after map literal.")?;
        Ok(Expr::MapLiteral(entries))
    }

    fn record_literal(&mut self, type_name: String) -> HayuloResult<Expr> {
        let brace = self.consume("LBRACE", "Expected '{' to start record literal.")?;
        let mut fields = Vec::new();
        let mut seen = BTreeSet::new();
        loop {
            let field = self.consume("IDENT", "Expected record field name.")?;
            if !seen.insert(field.value.clone()) {
                return Err(HayuloError::new(
                    Diagnostic::new(
                        "duplicate_record_field",
                        format!("Record literal repeats field {:?}.", field.value),
                    )
                    .file(self.filename)
                    .line(Some(field.line))
                    .column(Some(field.column))
                    .suggestion("Remove the duplicate field or rename it."),
                ));
            }
            self.consume("COLON", "Expected ':' between record field and value.")?;
            fields.push((field.value, self.expression()?));
            if !self.matches(&["COMMA"]) {
                break;
            }
            if self.check(&["RBRACE"]) {
                break;
            }
        }
        self.consume("RBRACE", "Expected '}' after record literal.")?;
        Ok(Expr::RecordLiteral {
            type_name,
            fields,
            line: brace.line,
        })
    }

    fn looks_like_record_literal(&self) -> bool {
        self.current + 2 < self.tokens.len()
            && self.tokens[self.current].kind == "LBRACE"
            && self.tokens[self.current + 1].kind == "IDENT"
            && self.tokens[self.current + 2].kind == "COLON"
    }

    fn type_until(&mut self, end_kinds: &[&str]) -> HayuloResult<String> {
        let mut parts = Vec::new();
        let mut depth = 0usize;
        while !self.is_at_end() {
            if depth == 0 && self.check(end_kinds) {
                break;
            }
            if self.check(&["LT", "LBRACKET", "LPAREN"]) {
                depth += 1;
            } else if self.check(&["GT", "RBRACKET", "RPAREN"]) {
                if depth > 0 {
                    depth -= 1;
                } else if self.check(end_kinds) {
                    break;
                }
            }
            parts.push(self.advance().value);
        }
        if parts.is_empty() {
            return self.error_here(
                "syntax_error",
                "Expected type annotation.",
                vec!["Add a type name such as Int, Text, or List<Int>."],
            );
        }
        Ok(parts.join(""))
    }

    fn skip_balanced_block(&mut self, label: &str) -> HayuloResult<()> {
        self.consume("LBRACE", &format!("Expected '{{' after {label}."))?;
        let mut depth = 1usize;
        while depth > 0 && !self.is_at_end() {
            if self.matches(&["LBRACE"]) {
                depth += 1;
            } else if self.matches(&["RBRACE"]) {
                depth -= 1;
            } else {
                self.advance();
            }
        }
        if depth != 0 {
            return self.error_here(
                "unterminated_block",
                format!("Unterminated {label} block."),
                vec!["Add a closing '}' for this block."],
            );
        }
        Ok(())
    }

    fn matches(&mut self, kinds: &[&str]) -> bool {
        for kind in kinds {
            if self.check(&[*kind]) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, kind: &str, message: &str) -> HayuloResult<Token> {
        if self.check(&[kind]) {
            return Ok(self.advance());
        }
        let token = self.peek().clone();
        Err(HayuloError::new(
            Diagnostic::new("syntax_error", message)
                .file(self.filename)
                .line(Some(token.line))
                .column(Some(token.column))
                .detail("expected", serde_json::json!(kind))
                .detail("actual", serde_json::json!(token.kind))
                .suggestion("Check punctuation near this location."),
        ))
    }

    fn check(&self, kinds: &[&str]) -> bool {
        if self.is_at_end() {
            return kinds.contains(&"EOF");
        }
        kinds.iter().any(|kind| self.peek().kind == *kind)
    }

    fn check_next(&self, kind: &str) -> bool {
        self.current + 1 < self.tokens.len() && self.tokens[self.current + 1].kind == kind
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous().clone()
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == "EOF"
    }

    fn error_here<T>(
        &self,
        code: &str,
        message: impl Into<String>,
        suggestions: Vec<&str>,
    ) -> HayuloResult<T> {
        let token = self.peek();
        Err(HayuloError::new(
            Diagnostic::new(code, message.into())
                .file(self.filename)
                .line(Some(token.line))
                .column(Some(token.column))
                .suggestions(suggestions),
        ))
    }
}
