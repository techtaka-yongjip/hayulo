from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from .ast import (
    Assign,
    Binary,
    Call,
    Expect,
    Expr,
    ExprStmt,
    FunctionDecl,
    If,
    Literal,
    Program,
    Return,
    Stmt,
    TestDecl,
    Unary,
    Variable,
)
from .diagnostics import Diagnostic, HayuloRuntimeError


class _ReturnSignal(Exception):
    def __init__(self, value: Any):
        self.value = value


@dataclass
class TestResult:
    name: str
    passed: bool
    error: str | None = None
    line: int | None = None

    def to_dict(self) -> dict[str, Any]:
        data: dict[str, Any] = {"name": self.name, "passed": self.passed}
        if self.error is not None:
            data["error"] = self.error
        if self.line is not None:
            data["line"] = self.line
        return data


class Interpreter:
    def __init__(self, program: Program, filename: str | None = None):
        self.program = program
        self.filename = filename
        self.output: list[str] = []
        self.env_stack: list[dict[str, Any]] = []

    def run_main(self) -> Any:
        if "main" not in self.program.functions:
            raise HayuloRuntimeError(
                Diagnostic(
                    code="missing_main",
                    message="No fn main() found.",
                    file=self.filename,
                    suggestions=["Add a function named main with no required parameters."],
                )
            )
        return self.call_function("main", [])

    def run_tests(self) -> list[TestResult]:
        results: list[TestResult] = []
        for test in self.program.tests:
            self.env_stack.append({})
            try:
                self._exec_block(test.body)
                results.append(TestResult(name=test.name, passed=True, line=test.line))
            except HayuloRuntimeError as error:
                d = error.diagnostic
                results.append(TestResult(name=test.name, passed=False, error=d.message, line=d.line or test.line))
            except _ReturnSignal:
                results.append(
                    TestResult(
                        name=test.name,
                        passed=False,
                        error="Tests cannot return values.",
                        line=test.line,
                    )
                )
            finally:
                self.env_stack.pop()
        return results

    def call_function(self, name: str, args: list[Any]) -> Any:
        if name == "print":
            self.output.append(" ".join(self._stringify(arg) for arg in args))
            return None
        if name == "len":
            if len(args) != 1:
                self._runtime_error(
                    "arity_mismatch",
                    "len expects exactly one argument.",
                    suggestions=["Call len(value) with a single Text or collection value."],
                )
            return len(args[0])

        fn = self.program.functions.get(name)
        if fn is None:
            self._runtime_error(
                "unknown_function",
                f"Unknown function {name!r}.",
                suggestions=["Define this function or check the function name for typos."],
            )

        if len(args) != len(fn.params):
            self._runtime_error(
                "arity_mismatch",
                f"Function {name!r} expects {len(fn.params)} arguments but got {len(args)}.",
                details={"expected": len(fn.params), "actual": len(args)},
                suggestions=["Pass the correct number of arguments or update the function signature."],
                line=fn.line,
            )

        env = dict(zip(fn.params, args))
        self.env_stack.append(env)
        try:
            self._exec_block(fn.body)
        except _ReturnSignal as signal:
            return signal.value
        finally:
            self.env_stack.pop()
        return None

    def _exec_block(self, body: list[Stmt]) -> None:
        for stmt in body:
            self._exec_stmt(stmt)

    def _exec_stmt(self, stmt: Stmt) -> None:
        if isinstance(stmt, Assign):
            self._assign(stmt.name, self._eval(stmt.expr))
            return

        if isinstance(stmt, Return):
            raise _ReturnSignal(self._eval(stmt.expr))

        if isinstance(stmt, ExprStmt):
            self._eval(stmt.expr)
            return

        if isinstance(stmt, If):
            if self._truthy(self._eval(stmt.condition)):
                self._exec_block(stmt.then_body)
            else:
                self._exec_block(stmt.else_body)
            return

        if isinstance(stmt, Expect):
            value = self._eval(stmt.expr)
            if not self._truthy(value):
                raise HayuloRuntimeError(
                    Diagnostic(
                        code="expectation_failed",
                        message="Expectation failed.",
                        file=self.filename,
                        line=stmt.line,
                        suggestions=["Inspect the expression after expect or update the implementation being tested."],
                    )
                )
            return

        self._runtime_error("unknown_statement", f"Cannot execute statement {stmt!r}.")

    def _eval(self, expr: Expr) -> Any:
        if isinstance(expr, Literal):
            return expr.value

        if isinstance(expr, Variable):
            return self._lookup(expr.name)

        if isinstance(expr, Unary):
            right = self._eval(expr.right)
            if expr.op == "-":
                return -right
            if expr.op == "not":
                return not self._truthy(right)
            self._runtime_error("unknown_operator", f"Unknown unary operator {expr.op!r}.")

        if isinstance(expr, Binary):
            if expr.op == "and":
                return self._truthy(self._eval(expr.left)) and self._truthy(self._eval(expr.right))
            if expr.op == "or":
                return self._truthy(self._eval(expr.left)) or self._truthy(self._eval(expr.right))

            left = self._eval(expr.left)
            right = self._eval(expr.right)
            return self._binary(left, expr.op, right)

        if isinstance(expr, Call):
            args = [self._eval(arg) for arg in expr.args]
            try:
                return self.call_function(expr.callee, args)
            except HayuloRuntimeError as error:
                if error.diagnostic.line is None:
                    error.diagnostic.line = expr.line
                raise

        self._runtime_error("unknown_expression", f"Cannot evaluate expression {expr!r}.")

    def _binary(self, left: Any, op: str, right: Any) -> Any:
        try:
            if op == "+":
                if isinstance(left, str) and isinstance(right, str):
                    return left + right
                if isinstance(left, (int, float)) and isinstance(right, (int, float)):
                    return left + right
                self._runtime_error(
                    "invalid_operator_types",
                    "Operator + supports number addition or Text concatenation, but not mixed values.",
                    details={"left": type(left).__name__, "right": type(right).__name__},
                    suggestions=["Convert values explicitly before using +."],
                )
            if op == "-":
                return left - right
            if op == "*":
                return left * right
            if op == "/":
                return left / right
            if op == "%":
                return left % right
            if op == "==":
                return left == right
            if op == "!=":
                return left != right
            if op == "<":
                return left < right
            if op == "<=":
                return left <= right
            if op == ">":
                return left > right
            if op == ">=":
                return left >= right
        except HayuloRuntimeError:
            raise
        except Exception as exc:
            self._runtime_error(
                "operator_error",
                f"Operator {op} failed: {exc}",
                details={"operator": op, "left": repr(left), "right": repr(right)},
                suggestions=["Check that both operands support this operator."],
            )

        self._runtime_error("unknown_operator", f"Unknown operator {op!r}.")

    def _lookup(self, name: str) -> Any:
        for env in reversed(self.env_stack):
            if name in env:
                return env[name]
        self._runtime_error(
            "unknown_variable",
            f"Unknown variable {name!r}.",
            suggestions=["Define the variable before using it or check for a typo."],
        )

    def _assign(self, name: str, value: Any) -> None:
        if not self.env_stack:
            self.env_stack.append({})
        self.env_stack[-1][name] = value

    def _truthy(self, value: Any) -> bool:
        return bool(value)

    def _stringify(self, value: Any) -> str:
        if value is True:
            return "true"
        if value is False:
            return "false"
        if value is None:
            return "none"
        return str(value)

    def _runtime_error(
        self,
        code: str,
        message: str,
        *,
        details: dict[str, Any] | None = None,
        suggestions: list[str] | None = None,
        line: int | None = None,
    ) -> None:
        raise HayuloRuntimeError(
            Diagnostic(
                code=code,
                message=message,
                file=self.filename,
                line=line,
                details=details or {},
                suggestions=suggestions or [],
            )
        )
