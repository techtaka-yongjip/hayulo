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
    params: list[str]
    body: list["Stmt"]
    line: int


@dataclass
class TestDecl:
    name: str
    body: list["Stmt"]
    line: int


class Stmt:
    pass


@dataclass
class Assign(Stmt):
    name: str
    expr: "Expr"


@dataclass
class Return(Stmt):
    expr: "Expr"


@dataclass
class ExprStmt(Stmt):
    expr: "Expr"


@dataclass
class If(Stmt):
    condition: "Expr"
    then_body: list[Stmt]
    else_body: list[Stmt]


@dataclass
class Expect(Stmt):
    expr: "Expr"
    line: int


class Expr:
    pass


@dataclass
class Literal(Expr):
    value: Any


@dataclass
class Variable(Expr):
    name: str


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
class Call(Expr):
    callee: str
    args: list[Expr]
    line: int
