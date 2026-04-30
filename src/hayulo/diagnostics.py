from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any


DIAGNOSTIC_SCHEMA = "hayulo.diagnostics@0.1"
TEST_SCHEMA = "hayulo.test@0.1"


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

    def to_schema_dict(self, *, severity: str = "error") -> dict[str, Any]:
        return {
            "code": self.code,
            "severity": severity,
            "message": self.message,
            "location": {
                "file": self.file,
                "line": self.line,
                "column": self.column,
            },
            "details": self.details,
            "suggestions": [{"message": suggestion} for suggestion in self.suggestions],
        }


def diagnostic_failure_payload(errors: list["HayuloError"]) -> dict[str, Any]:
    return {
        "schema": DIAGNOSTIC_SCHEMA,
        "status": "failed",
        "diagnostics": [error.diagnostic.to_schema_dict() for error in errors],
        "errors": [error.diagnostic.to_dict() for error in errors],
    }


class HayuloError(Exception):
    """Base exception carrying a structured Hayulo diagnostic."""

    def __init__(self, diagnostic: Diagnostic):
        super().__init__(diagnostic.message)
        self.diagnostic = diagnostic


class HayuloSyntaxError(HayuloError):
    pass


class HayuloRuntimeError(HayuloError):
    pass
