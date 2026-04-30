from __future__ import annotations

import re
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
from .diagnostics import Diagnostic, HayuloError


class HayuloStaticError(HayuloError):
    pass


@dataclass
class StaticType:
    name: str
    item: "StaticType | None" = None
    key: "StaticType | None" = None
    value: "StaticType | None" = None
    fields: dict[str, "StaticType"] | None = None

    def label(self) -> str:
        if self.name == "List" and self.item:
            return f"List<{self.item.label()}>"
        if self.name == "Map" and self.key and self.value:
            return f"Map<{self.key.label()},{self.value.label()}>"
        if self.name == "Option" and self.item:
            return f"Option<{self.item.label()}>"
        if self.name == "Result" and self.key and self.value:
            return f"Result<{self.key.label()},{self.value.label()}>"
        return self.name


@dataclass
class FunctionInfo:
    decl: FunctionDecl
    param_types: list[StaticType]
    declared_return: StaticType | None
    inferred_return: StaticType | None = None

    def return_type(self) -> StaticType:
        return self.declared_return or self.inferred_return or UNKNOWN


UNKNOWN = StaticType("Unknown")
ANY = StaticType("Any")
NONE = StaticType("None")
BOOL = StaticType("Bool")
INT = StaticType("Int")
FLOAT = StaticType("Float")
TEXT = StaticType("Text")
BUILTIN_TYPES = {"Int", "Float", "Text", "Bool", "Any", "None", "Time", "Email", "Status"}


def check_program(program: Program, filename: str | None = None) -> None:
    StaticChecker(program, filename).check()


class StaticChecker:
    def __init__(self, program: Program, filename: str | None):
        self.program = program
        self.filename = filename
        self.functions: dict[str, FunctionInfo] = {}
        self._checking: set[str] = set()
        self._checked: set[str] = set()

    def check(self) -> None:
        for fn in self.program.functions.values():
            self._add_function(fn)

        for fn in self.program.functions.values():
            self._check_function(fn)

        for test in self.program.tests:
            self._check_test(test)

    def _add_function(self, fn: FunctionDecl) -> None:
        seen: set[str] = set()
        for param in fn.params:
            if param.name in seen:
                self._error(
                    "name.duplicate_definition",
                    f"Parameter {param.name!r} is already defined in function {fn.name!r}.",
                    param.line,
                    details={"name": param.name, "function": fn.name},
                    suggestions=["Rename one of the duplicate parameters."],
                )
            seen.add(param.name)

        self.functions[fn.name] = FunctionInfo(
            decl=fn,
            param_types=[type_from_annotation(param.type_name) for param in fn.params],
            declared_return=type_from_annotation(fn.return_type) if fn.return_type else None,
        )

    def _check_function(self, fn: FunctionDecl) -> None:
        if fn.name in self._checked:
            return
        if fn.name in self._checking:
            return
        self._checking.add(fn.name)
        info = self.functions[fn.name]
        env = {param.name: info.param_types[index] for index, param in enumerate(fn.params)}
        returns = self._check_block(fn.body, env, expected_return=info.declared_return)
        if returns:
            info.inferred_return = common_type(returns)
        elif info.declared_return and not compatible(info.declared_return, NONE):
            self._error(
                "type.return_mismatch",
                f"Function {fn.name!r} declares return type {info.declared_return.label()} but may return None.",
                fn.line,
                details={"expected": info.declared_return.label(), "actual": NONE.label(), "function": fn.name},
                suggestions=["Return a value matching the declared type on every path or remove the return annotation."],
            )
        self._checking.remove(fn.name)
        self._checked.add(fn.name)

    def _check_test(self, test: TestDecl) -> None:
        self._check_block(test.body, {}, expected_return=None)

    def _check_block(self, body: list[Stmt], env: dict[str, StaticType], expected_return: StaticType | None) -> list[StaticType]:
        returns: list[StaticType] = []
        for stmt in body:
            returns.extend(self._check_stmt(stmt, env, expected_return))
        return returns

    def _check_stmt(self, stmt: Stmt, env: dict[str, StaticType], expected_return: StaticType | None) -> list[StaticType]:
        if isinstance(stmt, Let):
            if stmt.name in env:
                self._error(
                    "name.duplicate_definition",
                    f"Name {stmt.name!r} is already bound in this scope.",
                    stmt.line,
                    details={"name": stmt.name},
                    suggestions=["Use set to reassign an existing binding or choose a different name."],
                )
            env[stmt.name] = self._infer_expr(stmt.expr, env, expected_return)
            return []

        if isinstance(stmt, Set):
            if stmt.name not in env:
                self._error(
                    "name.reassignment_before_binding",
                    f"Cannot reassign {stmt.name!r} before it is bound.",
                    stmt.line,
                    details={"name": stmt.name},
                    suggestions=["Use let to create a new binding before using set."],
                )
            env[stmt.name] = self._infer_expr(stmt.expr, env, expected_return)
            return []

        if isinstance(stmt, Return):
            actual = self._infer_expr(stmt.expr, env, expected_return)
            if expected_return and not compatible(expected_return, actual):
                self._error(
                    "type.return_mismatch",
                    f"Return value has type {actual.label()} but function declares {expected_return.label()}.",
                    stmt.line,
                    details={"expected": expected_return.label(), "actual": actual.label()},
                    suggestions=["Return a value matching the declared type or update the return annotation."],
                )
            return [actual]

        if isinstance(stmt, ExprStmt):
            self._infer_expr(stmt.expr, env, expected_return)
            return []

        if isinstance(stmt, If):
            self._infer_expr(stmt.condition, env, expected_return)
            returns = self._check_block(stmt.then_body, dict(env), expected_return)
            returns.extend(self._check_block(stmt.else_body, dict(env), expected_return))
            return returns

        if isinstance(stmt, For):
            iterable = self._infer_expr(stmt.iterable, env, expected_return)
            loop_type = self._loop_type(iterable, stmt.line)
            loop_env = dict(env)
            loop_env[stmt.name] = loop_type
            return self._check_block(stmt.body, loop_env, expected_return)

        if isinstance(stmt, Expect):
            self._infer_expr(stmt.expr, env, expected_return)
            return []

        if isinstance(stmt, Match):
            target = self._infer_expr(stmt.target, env, expected_return)
            return self._check_match(stmt, target, env, expected_return)

        return []

    def _check_match(self, stmt: Match, target: StaticType, env: dict[str, StaticType], expected_return: StaticType | None) -> list[StaticType]:
        variants = {case.variant for case in stmt.cases}
        if target.name == "Option":
            required = {"Some", "None"}
            if variants != required:
                self._error(
                    "match.non_exhaustive",
                    "Match on Option must handle Some and None.",
                    stmt.line,
                    details={"required": sorted(required), "actual": sorted(variants)},
                    suggestions=["Add both Some(value) and None cases."],
                )
        elif target.name == "Result":
            required = {"Ok", "Err"}
            if variants != required:
                self._error(
                    "match.non_exhaustive",
                    "Match on Result must handle Ok and Err.",
                    stmt.line,
                    details={"required": sorted(required), "actual": sorted(variants)},
                    suggestions=["Add both Ok(value) and Err(error) cases."],
                )
        elif target.name not in {"Unknown", "Any"}:
            self._error(
                "match.invalid_target",
                f"Can only match Option or Result values, not {target.label()}.",
                stmt.line,
                details={"actual": target.label()},
                suggestions=["Match on a value with type Option<T> or Result<T, E>."],
            )

        returns: list[StaticType] = []
        for case in stmt.cases:
            case_env = dict(env)
            if case.binding:
                if case.variant == "Some":
                    case_env[case.binding] = target.item or UNKNOWN
                elif case.variant == "Ok":
                    case_env[case.binding] = target.key or UNKNOWN
                elif case.variant == "Err":
                    case_env[case.binding] = target.value or UNKNOWN
                elif case.variant == "None":
                    self._error(
                        "match.invalid_binding",
                        "None match cases do not bind a value.",
                        case.line,
                        suggestions=["Use `None => { ... }` without parentheses."],
                    )
            returns.extend(self._check_block(case.body, case_env, expected_return))
        return returns

    def _infer_expr(self, expr: Expr, env: dict[str, StaticType], expected_return: StaticType | None = None) -> StaticType:
        if isinstance(expr, Literal):
            if isinstance(expr.value, bool):
                return BOOL
            if isinstance(expr.value, int):
                return INT
            if isinstance(expr.value, float):
                return FLOAT
            if isinstance(expr.value, str):
                return TEXT
            if expr.value is None:
                return NONE
            return UNKNOWN

        if isinstance(expr, Variable):
            if expr.name in env:
                return env[expr.name]
            self._error(
                "name.unknown_symbol",
                f"Unknown name {expr.name!r}.",
                expr.line,
                details={"name": expr.name},
                suggestions=["Define the name before using it or check for a typo."],
            )

        if isinstance(expr, Unary):
            value = self._infer_expr(expr.right, env, expected_return)
            if expr.op == "not":
                return BOOL
            if expr.op == "-":
                if value.name in {"Int", "Float", "Unknown", "Any"}:
                    return value
                self._error(
                    "type.operator_mismatch",
                    f"Unary operator - cannot be used with {value.label()}.",
                    line_for_expr(expr.right),
                    details={"operator": "-", "actual": value.label()},
                    suggestions=["Use - with Int or Float values."],
                )
            return UNKNOWN

        if isinstance(expr, Binary):
            if expr.op in {"and", "or"}:
                self._infer_expr(expr.left, env, expected_return)
                self._infer_expr(expr.right, env, expected_return)
                return BOOL

            left = self._infer_expr(expr.left, env, expected_return)
            right = self._infer_expr(expr.right, env, expected_return)
            if expr.op in {"==", "!=", "<", "<=", ">", ">="}:
                return BOOL
            if expr.op == "+" and left.name == "Text" and right.name == "Text":
                return TEXT
            if expr.op in {"+", "-", "*", "/", "%"}:
                if left.name in {"Int", "Float", "Unknown", "Any"} and right.name in {"Int", "Float", "Unknown", "Any"}:
                    if left.name == "Float" or right.name == "Float" or expr.op == "/":
                        return FLOAT
                    if left.name == "Unknown" or right.name == "Unknown":
                        return UNKNOWN
                    return INT
                self._error(
                    "type.operator_mismatch",
                    f"Operator {expr.op} cannot be used with {left.label()} and {right.label()}.",
                    line_for_expr(expr),
                    details={"operator": expr.op, "left": left.label(), "right": right.label()},
                    suggestions=["Use compatible operand types for this operator."],
                )
            return UNKNOWN

        if isinstance(expr, ListLiteral):
            if not expr.elements:
                return StaticType("List", item=ANY)
            return StaticType("List", item=common_type([self._infer_expr(element, env, expected_return) for element in expr.elements]))

        if isinstance(expr, MapLiteral):
            if not expr.entries:
                return StaticType("Map", key=ANY, value=ANY)
            keys: list[StaticType] = []
            values: list[StaticType] = []
            for key_expr, value_expr in expr.entries:
                keys.append(self._infer_expr(key_expr, env, expected_return))
                values.append(self._infer_expr(value_expr, env, expected_return))
            return StaticType("Map", key=common_type(keys), value=common_type(values))

        if isinstance(expr, Index):
            target = self._infer_expr(expr.target, env, expected_return)
            index = self._infer_expr(expr.index, env, expected_return)
            if target.name == "List":
                if not compatible(INT, index):
                    self._error(
                        "type.invalid_index",
                        f"List index must be Int, not {index.label()}.",
                        expr.line,
                        details={"expected": "Int", "actual": index.label()},
                        suggestions=["Use an Int index such as values[0]."],
                    )
                return target.item or UNKNOWN
            if target.name == "Map":
                if target.key and not compatible(target.key, index):
                    self._error(
                        "type.invalid_index",
                        f"Map index has type {index.label()} but keys are {target.key.label()}.",
                        expr.line,
                        details={"expected": target.key.label(), "actual": index.label()},
                        suggestions=["Use an index value matching the map key type."],
                    )
                return target.value or UNKNOWN
            if target.name in {"Unknown", "Any"}:
                return UNKNOWN
            self._error(
                "type.invalid_index_target",
                f"Only lists and maps can be indexed, not {target.label()}.",
                expr.line,
                details={"target": target.label()},
                suggestions=["Index a list with an Int or a map with a key."],
            )

        if isinstance(expr, FieldAccess):
            target = self._infer_expr(expr.target, env, expected_return)
            if target.fields is not None:
                if expr.field not in target.fields:
                    self._error(
                        "record.unknown_field",
                        f"Record {target.name} has no field {expr.field!r}.",
                        expr.line,
                        details={"record": target.name, "field": expr.field, "known_fields": sorted(target.fields.keys())},
                        suggestions=["Use one of the fields defined on this record value."],
                    )
                return target.fields[expr.field]
            if target.name in {"Unknown", "Any"} or is_record_name(target.name):
                return UNKNOWN
            self._error(
                "record.invalid_field_target",
                f"Only records support field access, not {target.label()}.",
                expr.line,
                details={"target": target.label(), "field": expr.field},
                suggestions=["Access fields on record values such as User { name: \"Ada\" }.name."],
            )

        if isinstance(expr, RecordLiteral):
            return StaticType(expr.type_name, fields={name: self._infer_expr(value, env, expected_return) for name, value in expr.fields})

        if isinstance(expr, VariantLiteral):
            if expr.variant == "None":
                return StaticType("Option", item=ANY)
            value = self._infer_expr(expr.value, env, expected_return) if expr.value else UNKNOWN
            if expr.variant == "Some":
                return StaticType("Option", item=value)
            if expr.variant == "Ok":
                return StaticType("Result", key=value, value=ANY)
            if expr.variant == "Err":
                return StaticType("Result", key=ANY, value=value)
            return UNKNOWN

        if isinstance(expr, Try):
            target = self._infer_expr(expr.expr, env, expected_return)
            if target.name == "Option":
                self._check_try_early_return(target, expected_return, expr.line)
                return target.item or UNKNOWN
            if target.name == "Result":
                self._check_try_early_return(target, expected_return, expr.line)
                return target.key or UNKNOWN
            if target.name in {"Unknown", "Any"}:
                return UNKNOWN
            self._error(
                "type.invalid_try_target",
                f"try expects Option or Result, not {target.label()}.",
                expr.line,
                details={"actual": target.label()},
                suggestions=["Use try with a function returning Option<T> or Result<T, E>."],
            )

        if isinstance(expr, Call):
            return self._infer_call(expr, env, expected_return)

        return UNKNOWN

    def _infer_call(self, expr: Call, env: dict[str, StaticType], expected_return: StaticType | None = None) -> StaticType:
        args = [self._infer_expr(arg, env, expected_return) for arg in expr.args]

        if expr.callee == "print":
            return NONE

        if expr.callee == "len":
            self._check_arity("len", expected=1, actual=len(args), line=expr.line)
            target = args[0]
            if target.name not in {"Text", "List", "Map", "Unknown", "Any"}:
                self._error(
                    "type.argument_mismatch",
                    f"len expects Text, List, or Map, not {target.label()}.",
                    expr.line,
                    details={"expected": "Text|List|Map", "actual": target.label()},
                    suggestions=["Pass a Text, List, or Map value to len."],
                )
            return INT

        info = self.functions.get(expr.callee)
        if info is None:
            self._error(
                "name.unknown_symbol",
                f"Unknown function {expr.callee!r}.",
                expr.line,
                details={"name": expr.callee},
                suggestions=["Define this function or check the function name for typos."],
            )

        if info.declared_return is None and info.inferred_return is None:
            self._check_function(info.decl)

        self._check_arity(expr.callee, expected=len(info.param_types), actual=len(args), line=expr.line)
        for index, (expected, actual) in enumerate(zip(info.param_types, args)):
            if not compatible(expected, actual):
                self._error(
                    "type.argument_mismatch",
                    f"Argument {index + 1} to {expr.callee} has type {actual.label()} but parameter expects {expected.label()}.",
                    expr.line,
                    details={"function": expr.callee, "argument": index + 1, "expected": expected.label(), "actual": actual.label()},
                    suggestions=["Pass a value matching the parameter type."],
                )
        return info.return_type()

    def _check_try_early_return(self, target: StaticType, expected_return: StaticType | None, line: int) -> None:
        if expected_return is None or expected_return.name in {"Unknown", "Any"}:
            return
        if target.name == "Option":
            if expected_return.name in {"Option", "Result"}:
                return
            self._error(
                "type.try_return_mismatch",
                f"try on Option may return None, but the function declares {expected_return.label()}.",
                line,
                details={"try_target": target.label(), "function_return": expected_return.label()},
                suggestions=["Use try only inside a function returning Option<T> or Result<T, E>, or handle the Option with match."],
            )
        if target.name == "Result":
            if expected_return.name == "Result":
                if expected_return.value and target.value and not compatible(expected_return.value, target.value):
                    self._error(
                        "type.try_return_mismatch",
                        f"try may return Err({target.value.label()}) but the function declares {expected_return.label()}.",
                        line,
                        details={"try_target": target.label(), "function_return": expected_return.label()},
                        suggestions=["Use a compatible Result error type or handle the Result with match."],
                    )
                return
            self._error(
                "type.try_return_mismatch",
                f"try on Result may return Err, but the function declares {expected_return.label()}.",
                line,
                details={"try_target": target.label(), "function_return": expected_return.label()},
                suggestions=["Use try only inside a function returning Result<T, E>, or handle the Result with match."],
            )

    def _check_arity(self, name: str, *, expected: int, actual: int, line: int) -> None:
        if expected != actual:
            self._error(
                "call.arity_mismatch",
                f"Function {name!r} expects {expected} arguments but got {actual}.",
                line,
                details={"function": name, "expected": expected, "actual": actual},
                suggestions=["Pass the expected number of arguments or update the function signature."],
            )

    def _loop_type(self, iterable: StaticType, line: int) -> StaticType:
        if iterable.name == "List":
            return iterable.item or UNKNOWN
        if iterable.name == "Map":
            return iterable.key or UNKNOWN
        if iterable.name in {"Unknown", "Any"}:
            return UNKNOWN
        self._error(
            "type.not_iterable",
            f"For loops can iterate over lists and maps, not {iterable.label()}.",
            line,
            details={"actual": iterable.label()},
            suggestions=["Use a List or Map value after 'in'."],
        )

    def _error(
        self,
        code: str,
        message: str,
        line: int | None,
        *,
        details: dict[str, Any] | None = None,
        suggestions: list[str] | None = None,
    ) -> None:
        raise HayuloStaticError(
            Diagnostic(
                code=code,
                message=message,
                file=self.filename,
                line=line,
                details=details or {},
                suggestions=suggestions or [],
            )
        )


def type_from_annotation(type_name: str | None) -> StaticType:
    if not type_name:
        return UNKNOWN
    value = re.sub(r"\s+", "", type_name)
    if value in BUILTIN_TYPES:
        return StaticType(value)
    if value.startswith("List<") and value.endswith(">"):
        return StaticType("List", item=type_from_annotation(value[5:-1]))
    if value.startswith("Map<") and value.endswith(">"):
        inner = value[4:-1]
        parts = split_top_level(inner, ",")
        if len(parts) == 2:
            return StaticType("Map", key=type_from_annotation(parts[0]), value=type_from_annotation(parts[1]))
        return StaticType("Map", key=ANY, value=ANY)
    if value.startswith("Option<") and value.endswith(">"):
        return StaticType("Option", item=type_from_annotation(value[7:-1]))
    if value.startswith("Result<") and value.endswith(">"):
        inner = value[7:-1]
        parts = split_top_level(inner, ",")
        if len(parts) == 2:
            return StaticType("Result", key=type_from_annotation(parts[0]), value=type_from_annotation(parts[1]))
        return StaticType("Result", key=ANY, value=ANY)
    return StaticType(value)


def split_top_level(value: str, separator: str) -> list[str]:
    parts: list[str] = []
    depth = 0
    start = 0
    for index, ch in enumerate(value):
        if ch == "<":
            depth += 1
        elif ch == ">":
            depth -= 1
        elif ch == separator and depth == 0:
            parts.append(value[start:index])
            start = index + 1
    parts.append(value[start:])
    return [part for part in parts if part]


def common_type(types: list[StaticType]) -> StaticType:
    if not types:
        return UNKNOWN
    result = types[0]
    for current in types[1:]:
        if result.name in {"Unknown", "Any"}:
            result = current
            continue
        if current.name in {"Unknown", "Any"}:
            continue
        if result.name != current.name:
            return UNKNOWN
        if result.name == "List":
            result = StaticType("List", item=common_type([result.item or UNKNOWN, current.item or UNKNOWN]))
        elif result.name == "Map":
            result = StaticType(
                "Map",
                key=common_type([result.key or UNKNOWN, current.key or UNKNOWN]),
                value=common_type([result.value or UNKNOWN, current.value or UNKNOWN]),
            )
        elif result.name == "Option":
            result = StaticType("Option", item=common_type([result.item or UNKNOWN, current.item or UNKNOWN]))
        elif result.name == "Result":
            result = StaticType(
                "Result",
                key=common_type([result.key or UNKNOWN, current.key or UNKNOWN]),
                value=common_type([result.value or UNKNOWN, current.value or UNKNOWN]),
            )
        elif result.fields is not None or current.fields is not None:
            if result.fields == current.fields:
                result = StaticType(result.name, fields=result.fields)
            else:
                result = StaticType(result.name)
    return result


def compatible(expected: StaticType, actual: StaticType) -> bool:
    if expected.name in {"Any", "Unknown"} or actual.name in {"Any", "Unknown"}:
        return True
    if expected.name == "Float" and actual.name == "Int":
        return True
    if expected.name != actual.name:
        return False
    if expected.name == "List" and expected.item and actual.item:
        return compatible(expected.item, actual.item)
    if expected.name == "Map" and expected.key and expected.value and actual.key and actual.value:
        return compatible(expected.key, actual.key) and compatible(expected.value, actual.value)
    if expected.name == "Option" and expected.item and actual.item:
        return compatible(expected.item, actual.item)
    if expected.name == "Result" and expected.key and expected.value and actual.key and actual.value:
        return compatible(expected.key, actual.key) and compatible(expected.value, actual.value)
    return True


def is_record_name(name: str) -> bool:
    return name not in BUILTIN_TYPES | {"List", "Map"}


def line_for_expr(expr: Expr) -> int | None:
    return getattr(expr, "line", None)
