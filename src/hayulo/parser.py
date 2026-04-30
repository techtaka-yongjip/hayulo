from __future__ import annotations

from .ast import (
    Assign,
    Binary,
    Call,
    Expect,
    Expr,
    ExprStmt,
    FieldAccess,
    For,
    FunctionDecl,
    If,
    Index,
    ListLiteral,
    Literal,
    MapLiteral,
    Program,
    RecordLiteral,
    Return,
    Stmt,
    TestDecl,
    Unary,
    Variable,
)
from .diagnostics import Diagnostic, HayuloSyntaxError
from .lexer import Token


class Parser:
    def __init__(self, tokens: list[Token], filename: str | None = None):
        self.tokens = tokens
        self.filename = filename
        self.current = 0

    def parse(self) -> Program:
        module_name: str | None = None
        functions: dict[str, FunctionDecl] = {}
        tests: list[TestDecl] = []

        while not self._is_at_end():
            if self._match("MODULE"):
                module_name = self._parse_module_name()
                continue

            if self._match("INTENT"):
                self._skip_balanced_block("intent")
                continue

            self._match("PUB")  # accepted and currently ignored

            if self._match("FN"):
                fn = self._function_decl()
                if fn.name in functions:
                    self._error_here(
                        "duplicate_function",
                        f"Function {fn.name!r} is already defined.",
                        ["Rename one of the functions or remove the duplicate."],
                    )
                functions[fn.name] = fn
                continue

            if self._match("TEST"):
                tests.append(self._test_decl())
                continue

            if self._check("EOF"):
                break

            tok = self._peek()
            raise HayuloSyntaxError(
                Diagnostic(
                    code="unexpected_top_level_token",
                    message=f"Unexpected token {tok.value!r} at top level.",
                    file=self.filename,
                    line=tok.line,
                    column=tok.column,
                    suggestions=["Top-level Hayulo code currently supports module, intent, fn, pub fn, and test."],
                )
            )

        return Program(module=module_name, functions=functions, tests=tests)

    def _parse_module_name(self) -> str:
        first = self._consume("IDENT", "Expected module name after 'module'.")
        parts = [first.value]
        while self._match("DOT"):
            parts.append(self._consume("IDENT", "Expected identifier after '.'.").value)
        return ".".join(parts)

    def _function_decl(self) -> FunctionDecl:
        name = self._consume("IDENT", "Expected function name after 'fn'.")
        self._consume("LPAREN", "Expected '(' after function name.")
        params = self._params()
        self._consume("RPAREN", "Expected ')' after parameters.")

        if self._match("ARROW"):
            self._skip_type_until("LBRACE")

        body = self._block()
        return FunctionDecl(name=name.value, params=params, body=body, line=name.line)

    def _test_decl(self) -> TestDecl:
        name = self._consume("STRING", "Expected test name string after 'test'.")
        body = self._block()
        return TestDecl(name=name.value, body=body, line=name.line)

    def _params(self) -> list[str]:
        params: list[str] = []
        if self._check("RPAREN"):
            return params

        while True:
            param = self._consume("IDENT", "Expected parameter name.")
            params.append(param.value)
            if self._match("COLON"):
                self._skip_type_until("COMMA", "RPAREN")
            if not self._match("COMMA"):
                break
        return params

    def _block(self) -> list[Stmt]:
        self._consume("LBRACE", "Expected '{' to start a block.")
        body: list[Stmt] = []
        while not self._check("RBRACE") and not self._is_at_end():
            body.append(self._statement())
        self._consume("RBRACE", "Expected '}' to close the block.")
        return body

    def _statement(self) -> Stmt:
        if self._match("RETURN"):
            return Return(self._expression())

        if self._match("IF"):
            condition = self._expression()
            then_body = self._block()
            else_body: list[Stmt] = []
            if self._match("ELSE"):
                if self._match("IF"):
                    nested_condition = self._expression()
                    nested_then = self._block()
                    nested_else: list[Stmt] = []
                    if self._match("ELSE"):
                        nested_else = self._block()
                    else_body = [If(nested_condition, nested_then, nested_else)]
                else:
                    else_body = self._block()
            return If(condition, then_body, else_body)

        if self._match("FOR"):
            line = self._previous().line
            name = self._consume("IDENT", "Expected loop variable name after 'for'.")
            self._consume("IN", "Expected 'in' after loop variable.")
            iterable = self._expression()
            body = self._block()
            return For(name.value, iterable, body, line=line)

        if self._match("EXPECT"):
            line = self._previous().line
            return Expect(self._expression(), line=line)

        if self._check("IDENT") and self._check_next("EQUAL"):
            name = self._advance().value
            self._consume("EQUAL", "Expected '=' after variable name.")
            return Assign(name, self._expression())

        expr = self._expression()
        return ExprStmt(expr)

    def _expression(self) -> Expr:
        return self._or()

    def _or(self) -> Expr:
        expr = self._and()
        while self._match("OR"):
            op = self._previous().value
            right = self._and()
            expr = Binary(expr, op, right)
        return expr

    def _and(self) -> Expr:
        expr = self._equality()
        while self._match("AND"):
            op = self._previous().value
            right = self._equality()
            expr = Binary(expr, op, right)
        return expr

    def _equality(self) -> Expr:
        expr = self._comparison()
        while self._match("EQEQ", "BANGEQ"):
            op = self._previous().value
            right = self._comparison()
            expr = Binary(expr, op, right)
        return expr

    def _comparison(self) -> Expr:
        expr = self._term()
        while self._match("LT", "LTE", "GT", "GTE"):
            op = self._previous().value
            right = self._term()
            expr = Binary(expr, op, right)
        return expr

    def _term(self) -> Expr:
        expr = self._factor()
        while self._match("PLUS", "MINUS"):
            op = self._previous().value
            right = self._factor()
            expr = Binary(expr, op, right)
        return expr

    def _factor(self) -> Expr:
        expr = self._unary()
        while self._match("STAR", "SLASH", "PERCENT"):
            op = self._previous().value
            right = self._unary()
            expr = Binary(expr, op, right)
        return expr

    def _unary(self) -> Expr:
        if self._match("MINUS", "NOT"):
            op = self._previous().value
            right = self._unary()
            return Unary(op, right)
        return self._call()

    def _call(self) -> Expr:
        expr = self._primary()

        while True:
            if self._match("LPAREN"):
                if not isinstance(expr, Variable):
                    tok = self._previous()
                    raise HayuloSyntaxError(
                        Diagnostic(
                            code="invalid_call_target",
                            message="Only named functions can be called in the current prototype.",
                            file=self.filename,
                            line=tok.line,
                            column=tok.column,
                            suggestions=["Call a function by name, such as greet(\"Ada\")."],
                        )
                    )
                args: list[Expr] = []
                if not self._check("RPAREN"):
                    while True:
                        args.append(self._expression())
                        if not self._match("COMMA"):
                            break
                self._consume("RPAREN", "Expected ')' after arguments.")
                expr = Call(expr.name, args, line=self._previous().line)
                continue

            if self._match("LBRACKET"):
                line = self._previous().line
                index = self._expression()
                self._consume("RBRACKET", "Expected ']' after index expression.")
                expr = Index(expr, index, line=line)
                continue

            if self._match("DOT"):
                dot = self._previous()
                field = self._consume("IDENT", "Expected field name after '.'.")
                expr = FieldAccess(expr, field.value, line=dot.line)
                continue

            if isinstance(expr, Variable) and self._looks_like_record_literal():
                expr = self._record_literal(expr.name)
                continue

            break

        return expr

    def _primary(self) -> Expr:
        if self._match("INT"):
            return Literal(int(self._previous().value))
        if self._match("FLOAT"):
            return Literal(float(self._previous().value))
        if self._match("STRING"):
            return Literal(self._previous().value)
        if self._match("TRUE"):
            return Literal(True)
        if self._match("FALSE"):
            return Literal(False)
        if self._check("LBRACKET"):
            return self._list_literal()
        if self._check("LBRACE"):
            return self._map_literal()
        if self._match("IDENT"):
            return Variable(self._previous().value)
        if self._match("LPAREN"):
            expr = self._expression()
            self._consume("RPAREN", "Expected ')' after expression.")
            return expr

        tok = self._peek()
        raise HayuloSyntaxError(
            Diagnostic(
                code="expected_expression",
                message=f"Expected expression but found {tok.value!r}.",
                file=self.filename,
                line=tok.line,
                column=tok.column,
                suggestions=["Use a literal, variable, function call, or parenthesized expression here."],
            )
        )

    def _list_literal(self) -> ListLiteral:
        self._consume("LBRACKET", "Expected '[' to start list literal.")
        elements: list[Expr] = []
        if not self._check("RBRACKET"):
            while True:
                elements.append(self._expression())
                if not self._match("COMMA"):
                    break
                if self._check("RBRACKET"):
                    break
        self._consume("RBRACKET", "Expected ']' after list literal.")
        return ListLiteral(elements)

    def _map_literal(self) -> MapLiteral:
        self._consume("LBRACE", "Expected '{' to start map literal.")
        entries: list[tuple[Expr, Expr]] = []
        if not self._check("RBRACE"):
            while True:
                key = self._expression()
                self._consume("COLON", "Expected ':' between map key and value.")
                value = self._expression()
                entries.append((key, value))
                if not self._match("COMMA"):
                    break
                if self._check("RBRACE"):
                    break
        self._consume("RBRACE", "Expected '}' after map literal.")
        return MapLiteral(entries)

    def _record_literal(self, type_name: str) -> RecordLiteral:
        brace = self._consume("LBRACE", "Expected '{' to start record literal.")
        fields: list[tuple[str, Expr]] = []
        seen: set[str] = set()
        while True:
            field = self._consume("IDENT", "Expected record field name.")
            if field.value in seen:
                raise HayuloSyntaxError(
                    Diagnostic(
                        code="duplicate_record_field",
                        message=f"Record literal repeats field {field.value!r}.",
                        file=self.filename,
                        line=field.line,
                        column=field.column,
                        suggestions=["Remove the duplicate field or rename it."],
                    )
                )
            seen.add(field.value)
            self._consume("COLON", "Expected ':' between record field and value.")
            fields.append((field.value, self._expression()))
            if not self._match("COMMA"):
                break
            if self._check("RBRACE"):
                break
        self._consume("RBRACE", "Expected '}' after record literal.")
        return RecordLiteral(type_name, fields, line=brace.line)

    def _looks_like_record_literal(self) -> bool:
        return (
            self.current + 2 < len(self.tokens)
            and self.tokens[self.current].kind == "LBRACE"
            and self.tokens[self.current + 1].kind == "IDENT"
            and self.tokens[self.current + 2].kind == "COLON"
        )

    def _skip_type_until(self, *end_kinds: str) -> None:
        depth = 0
        while not self._is_at_end():
            if depth == 0 and self._check(*end_kinds):
                return
            if self._match("LT", "LBRACKET", "LPAREN"):
                depth += 1
                continue
            if self._match("GT", "RBRACKET", "RPAREN"):
                if depth > 0:
                    depth -= 1
                    continue
                return
            self._advance()

    def _skip_balanced_block(self, label: str) -> None:
        self._consume("LBRACE", f"Expected '{{' after {label}.")
        depth = 1
        while depth > 0 and not self._is_at_end():
            if self._match("LBRACE"):
                depth += 1
            elif self._match("RBRACE"):
                depth -= 1
            else:
                self._advance()
        if depth != 0:
            self._error_here(
                "unterminated_block",
                f"Unterminated {label} block.",
                ["Add a closing '}' for this block."],
            )

    def _match(self, *kinds: str) -> bool:
        for kind in kinds:
            if self._check(kind):
                self._advance()
                return True
        return False

    def _consume(self, kind: str, message: str) -> Token:
        if self._check(kind):
            return self._advance()
        tok = self._peek()
        raise HayuloSyntaxError(
            Diagnostic(
                code="syntax_error",
                message=message,
                file=self.filename,
                line=tok.line,
                column=tok.column,
                details={"expected": kind, "actual": tok.kind},
                suggestions=["Check punctuation near this location."],
            )
        )

    def _check(self, *kinds: str) -> bool:
        if self._is_at_end():
            return "EOF" in kinds
        return self._peek().kind in kinds

    def _check_next(self, kind: str) -> bool:
        if self.current + 1 >= len(self.tokens):
            return False
        return self.tokens[self.current + 1].kind == kind

    def _advance(self) -> Token:
        if not self._is_at_end():
            self.current += 1
        return self._previous()

    def _previous(self) -> Token:
        return self.tokens[self.current - 1]

    def _peek(self) -> Token:
        return self.tokens[self.current]

    def _is_at_end(self) -> bool:
        return self._peek().kind == "EOF"

    def _error_here(self, code: str, message: str, suggestions: list[str]) -> None:
        tok = self._peek()
        raise HayuloSyntaxError(
            Diagnostic(
                code=code,
                message=message,
                file=self.filename,
                line=tok.line,
                column=tok.column,
                suggestions=suggestions,
            )
        )
