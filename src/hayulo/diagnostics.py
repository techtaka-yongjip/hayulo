from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any


@dataclass
class Diagnostic:
    code: str
    message: str
    file: str | None = None
    line: int | None = None
    column: int | None = None
    details: dict[str, Any] = field(default_factory=dict)
    suggestions: list[str] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        data: dict[str, Any] = {
            "code": self.code,
            "message": self.message,
        }
        if self.file is not None:
            data["file"] = self.file
        if self.line is not None:
            data["line"] = self.line
        if self.column is not None:
            data["column"] = self.column
        if self.details:
            data["details"] = self.details
        if self.suggestions:
            data["suggestions"] = self.suggestions
        return data


class HayuloError(Exception):
    """Base exception carrying a structured Hayulo diagnostic."""

    def __init__(self, diagnostic: Diagnostic):
        super().__init__(diagnostic.message)
        self.diagnostic = diagnostic


class HayuloSyntaxError(HayuloError):
    pass


class HayuloRuntimeError(HayuloError):
    pass
