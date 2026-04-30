from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass
class Program:
    module: str | None
    functions: dict[str, "FunctionDecl"]
    tests: list["TestDecl"]


@dataclass
class FunctionDecl:
    name: str
    params: list["FunctionParam"]
    return_type: str | None
    body: list["Stmt"]
    line: int


@dataclass
class FunctionParam:
    name: str
    type_name: str | None
    line: int


@dataclass
class TestDecl:
    name: str
    body: list["Stmt"]
    line: int


class Stmt:
    pass


@dataclass
class Let(Stmt):
    name: str
    expr: "Expr"
    line: int


@dataclass
class Set(Stmt):
    name: str
    expr: "Expr"
    line: int


@dataclass
class Return(Stmt):
    expr: "Expr"
    line: int


@dataclass
class ExprStmt(Stmt):
    expr: "Expr"


@dataclass
class If(Stmt):
    condition: "Expr"
    then_body: list[Stmt]
    else_body: list[Stmt]


@dataclass
class For(Stmt):
    name: str
    iterable: "Expr"
    body: list[Stmt]
    line: int


@dataclass
class Expect(Stmt):
    expr: "Expr"
    line: int


@dataclass
class MatchCase:
    variant: str
    binding: str | None
    body: list[Stmt]
    line: int


@dataclass
class Match(Stmt):
    target: "Expr"
    cases: list[MatchCase]
    line: int


class Expr:
    pass


@dataclass
class Literal(Expr):
    value: Any


@dataclass
class Variable(Expr):
    name: str
    line: int


@dataclass
class Unary(Expr):
    op: str
    right: Expr


@dataclass
class Binary(Expr):
    left: Expr
    op: str
    right: Expr


@dataclass
class ListLiteral(Expr):
    elements: list[Expr]


@dataclass
class MapLiteral(Expr):
    entries: list[tuple[Expr, Expr]]


@dataclass
class Index(Expr):
    target: Expr
    index: Expr
    line: int


@dataclass
class FieldAccess(Expr):
    target: Expr
    field: str
    line: int


@dataclass
class RecordLiteral(Expr):
    type_name: str
    fields: list[tuple[str, Expr]]
    line: int


@dataclass
class VariantLiteral(Expr):
    variant: str
    value: Expr | None
    line: int


@dataclass
class Try(Expr):
    expr: Expr
    line: int


@dataclass
class Call(Expr):
    callee: str
    args: list[Expr]
    line: int
