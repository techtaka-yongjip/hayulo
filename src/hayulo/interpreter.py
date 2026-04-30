from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from .ast import (
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
    Let,
    ListLiteral,
    Literal,
    MapLiteral,
    Match,
    Program,
    RecordLiteral,
    Return,
    Set,
    Stmt,
    TestDecl,
    Try,
    Unary,
    Variable,
    VariantLiteral,
)
from .diagnostics import Diagnostic, HayuloRuntimeError


class _ReturnSignal(Exception):
    def __init__(self, value: Any):
        self.value = value


@dataclass
class RecordValue:
    type_name: str
    fields: dict[str, Any]


@dataclass
class OptionValue:
    kind: str
    value: Any = None


@dataclass
class ResultValue:
    kind: str
    value: Any


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
        self.return_type_stack: list[str | None] = []

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
            try:
                return len(args[0])
            except TypeError:
                self._runtime_error(
                    "invalid_len_target",
                    "len expects Text, List, or Map.",
                    details={"target_type": self._type_name(args[0])},
                    suggestions=["Pass a Text, List, or Map value to len."],
                )

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

        env = {param.name: value for param, value in zip(fn.params, args)}
        self.env_stack.append(env)
        self.return_type_stack.append(fn.return_type)
        try:
            self._exec_block(fn.body)
        except _ReturnSignal as signal:
            return signal.value
        finally:
            self.return_type_stack.pop()
            self.env_stack.pop()
        return None

    def _exec_block(self, body: list[Stmt]) -> None:
        for stmt in body:
            self._exec_stmt(stmt)

    def _exec_stmt(self, stmt: Stmt) -> None:
        if isinstance(stmt, Let):
            self._bind(stmt.name, self._eval(stmt.expr), stmt.line)
            return

        if isinstance(stmt, Set):
            self._reassign(stmt.name, self._eval(stmt.expr), stmt.line)
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

        if isinstance(stmt, For):
            for value in self._iterable_values(self._eval(stmt.iterable), stmt.line):
                self._bind_or_replace(stmt.name, value)
                self._exec_block(stmt.body)
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

        if isinstance(stmt, Match):
            self._exec_match(stmt)
            return

        self._runtime_error("unknown_statement", f"Cannot execute statement {stmt!r}.")

    def _exec_match(self, stmt: Match) -> None:
        target = self._eval(stmt.target)
        if isinstance(target, OptionValue):
            variant = target.kind
            value = target.value
        elif isinstance(target, ResultValue):
            variant = target.kind
            value = target.value
        else:
            self._runtime_error(
                "match_invalid_target",
                "match expects an Option or Result value.",
                details={"target_type": self._type_name(target)},
                suggestions=["Match on Some/None or Ok/Err values."],
                line=stmt.line,
            )
        for case in stmt.cases:
            if case.variant != variant:
                continue
            if case.binding:
                self.env_stack.append({case.binding: value})
                try:
                    self._exec_block(case.body)
                finally:
                    self.env_stack.pop()
            else:
                self._exec_block(case.body)
            return
        self._runtime_error(
            "match_non_exhaustive",
            f"No match case handled {variant}.",
            details={"variant": variant},
            suggestions=["Add all Option or Result variants to this match."],
            line=stmt.line,
        )

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

        if isinstance(expr, ListLiteral):
            return [self._eval(element) for element in expr.elements]

        if isinstance(expr, MapLiteral):
            result: dict[Any, Any] = {}
            for key_expr, value_expr in expr.entries:
                key = self._eval(key_expr)
                try:
                    hash(key)
                except TypeError:
                    self._runtime_error(
                        "invalid_map_key",
                        "Map keys must be hashable values.",
                        details={"key": repr(key)},
                        suggestions=["Use Text, Int, Float, or Bool values as map keys."],
                    )
                result[key] = self._eval(value_expr)
            return result

        if isinstance(expr, Index):
            return self._index_value(self._eval(expr.target), self._eval(expr.index), expr.line)

        if isinstance(expr, FieldAccess):
            return self._field_value(self._eval(expr.target), expr.field, expr.line)

        if isinstance(expr, RecordLiteral):
            return RecordValue(expr.type_name, {name: self._eval(value) for name, value in expr.fields})

        if isinstance(expr, VariantLiteral):
            if expr.variant == "None":
                return OptionValue("None")
            value = self._eval(expr.value) if expr.value else None
            if expr.variant == "Some":
                return OptionValue("Some", value)
            if expr.variant == "Ok":
                return ResultValue("Ok", value)
            if expr.variant == "Err":
                return ResultValue("Err", value)
            self._runtime_error("unknown_variant", f"Unknown variant {expr.variant!r}.", line=expr.line)

        if isinstance(expr, Try):
            return self._eval_try(expr)

        if isinstance(expr, Call):
            args = [self._eval(arg) for arg in expr.args]
            try:
                return self.call_function(expr.callee, args)
            except HayuloRuntimeError as error:
                if error.diagnostic.line is None:
                    error.diagnostic.line = expr.line
                raise

        self._runtime_error("unknown_expression", f"Cannot evaluate expression {expr!r}.")

    def _eval_try(self, expr: Try) -> Any:
        value = self._eval(expr.expr)
        if isinstance(value, OptionValue):
            if value.kind == "Some":
                return value.value
            raise _ReturnSignal(self._early_none_value())
        if isinstance(value, ResultValue):
            if value.kind == "Ok":
                return value.value
            raise _ReturnSignal(ResultValue("Err", value.value))
        self._runtime_error(
            "invalid_try_target",
            "try expects an Option or Result value.",
            details={"target_type": self._type_name(value)},
            suggestions=["Use try with a function returning Option<T> or Result<T, E>."],
            line=expr.line,
        )

    def _early_none_value(self) -> Any:
        return_type = self.return_type_stack[-1] if self.return_type_stack else None
        if return_type and return_type.replace(" ", "").startswith("Result<"):
            return ResultValue("Err", "None")
        return OptionValue("None")

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

    def _index_value(self, target: Any, index: Any, line: int) -> Any:
        if isinstance(target, list):
            if not isinstance(index, int):
                self._runtime_error(
                    "invalid_index_type",
                    "List indexes must be Int values.",
                    details={"index_type": type(index).__name__},
                    suggestions=["Use an Int index such as values[0]."],
                    line=line,
                )
            if index < 0 or index >= len(target):
                self._runtime_error(
                    "index_out_of_range",
                    f"List index {index} is out of range.",
                    details={"index": index, "length": len(target)},
                    suggestions=["Check len(value) before indexing."],
                    line=line,
                )
            return target[index]

        if isinstance(target, dict):
            if index not in target:
                self._runtime_error(
                    "missing_map_key",
                    f"Map key {index!r} was not found.",
                    details={"key": repr(index)},
                    suggestions=["Check that the key exists before indexing the map."],
                    line=line,
                )
            return target[index]

        self._runtime_error(
            "invalid_index_target",
            "Only lists and maps can be indexed.",
            details={"target_type": self._type_name(target)},
            suggestions=["Index a list with an Int or a map with an existing key."],
            line=line,
        )

    def _field_value(self, target: Any, field: str, line: int) -> Any:
        if isinstance(target, RecordValue):
            if field not in target.fields:
                self._runtime_error(
                    "unknown_field",
                    f"Record {target.type_name} has no field {field!r}.",
                    details={"record": target.type_name, "field": field},
                    suggestions=["Check the record field name."],
                    line=line,
                )
            return target.fields[field]

        self._runtime_error(
            "invalid_field_target",
            "Only records support field access in the current prototype.",
            details={"target_type": self._type_name(target), "field": field},
            suggestions=["Construct a record value before accessing its fields."],
            line=line,
        )

    def _iterable_values(self, value: Any, line: int) -> list[Any]:
        if isinstance(value, list):
            return value
        if isinstance(value, dict):
            return list(value.keys())
        self._runtime_error(
            "not_iterable",
            "For loops can iterate over lists and maps.",
            details={"target_type": self._type_name(value)},
            suggestions=["Use a list literal like [1, 2] or a map literal like {\"key\": 1}."],
            line=line,
        )

    def _lookup(self, name: str) -> Any:
        for env in reversed(self.env_stack):
            if name in env:
                return env[name]
        self._runtime_error(
            "unknown_variable",
            f"Unknown variable {name!r}.",
            suggestions=["Define the variable before using it or check for a typo."],
        )

    def _bind(self, name: str, value: Any, line: int) -> None:
        if not self.env_stack:
            self.env_stack.append({})
        if name in self.env_stack[-1]:
            self._runtime_error(
                "duplicate_binding",
                f"Name {name!r} is already bound in this scope.",
                suggestions=["Use set to reassign an existing binding."],
                line=line,
            )
        self.env_stack[-1][name] = value

    def _bind_or_replace(self, name: str, value: Any) -> None:
        if not self.env_stack:
            self.env_stack.append({})
        self.env_stack[-1][name] = value

    def _reassign(self, name: str, value: Any, line: int) -> None:
        for env in reversed(self.env_stack):
            if name in env:
                env[name] = value
                return
        self._runtime_error(
            "reassignment_before_binding",
            f"Cannot reassign {name!r} before it is bound.",
            suggestions=["Use let before set."],
            line=line,
        )

    def _truthy(self, value: Any) -> bool:
        return bool(value)

    def _stringify(self, value: Any) -> str:
        if value is True:
            return "true"
        if value is False:
            return "false"
        if value is None:
            return "none"
        if isinstance(value, list):
            return "[" + ", ".join(self._stringify(item) for item in value) + "]"
        if isinstance(value, dict):
            items = [f"{self._stringify(key)}: {self._stringify(val)}" for key, val in value.items()]
            return "{" + ", ".join(items) + "}"
        if isinstance(value, RecordValue):
            fields = [f"{name}: {self._stringify(val)}" for name, val in value.fields.items()]
            return f"{value.type_name} {{" + ", ".join(fields) + "}"
        if isinstance(value, OptionValue):
            return "None" if value.kind == "None" else f"Some({self._stringify(value.value)})"
        if isinstance(value, ResultValue):
            return f"{value.kind}({self._stringify(value.value)})"
        return str(value)

    def _type_name(self, value: Any) -> str:
        if isinstance(value, bool):
            return "Bool"
        if isinstance(value, int):
            return "Int"
        if isinstance(value, float):
            return "Float"
        if isinstance(value, str):
            return "Text"
        if isinstance(value, list):
            return "List"
        if isinstance(value, dict):
            return "Map"
        if isinstance(value, RecordValue):
            return value.type_name
        if isinstance(value, OptionValue):
            return "Option"
        if isinstance(value, ResultValue):
            return "Result"
        if value is None:
            return "None"
        return type(value).__name__

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
